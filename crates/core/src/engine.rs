use crate::error::{SimError, SimResult};
use crate::mna::{MnaMatrix, MnaSystem, FreqDepType};
use crate::netlist::Netlist;
use crate::simulation::{AcResult, AcSweep, DcResult, TranResult, TranSweep, NoiseSweep, NoiseResult, NoiseModel, SensitivitySweep, SensitivityResult, MonteCarloConfig, DistributionType, MonteCarloResult};
use crate::solver::{solve_complex_matrix, solve_real_matrix};
use crate::types::Cx;
use std::collections::HashMap;
use std::f64::consts::PI;
use rayon::prelude::*;

pub fn simulate_ac(netlist: &Netlist, sweep: &AcSweep) -> SimResult<AcResult> {
    let frequencies = sweep.frequencies();
    
    // Build the MNA matrix once (topology is fixed)
    let mna_matrix = MnaMatrix::build_from_netlist(netlist);
    
    // Extract voltage source names
    let vsources: Vec<String> = mna_matrix.vsources.iter().map(|(n, _, _)| n.clone()).collect();
    
    // Parallel AC sweep
    let results: Result<Vec<(f64, Vec<Cx>, Vec<Cx>)>, SimError> = frequencies
        .par_iter()
        .map(|&f| {
            let omega = 2.0 * PI * f;
            let mut matrix = mna_matrix.clone();
            matrix.update_for_frequency(omega);
            
            let solution = matrix.solve()
                .ok_or(SimError::SingularMatrix { freq: f, cond: 0.0 })?;
            
            let num_nodes = matrix.num_nodes;
            let node_voltages = solution[..num_nodes].to_vec();
            let vsource_currents = solution[num_nodes..].to_vec();
            
            Ok((f, node_voltages, vsource_currents))
        })
        .collect();
    
    let results = results?;
    
    // Aggregate results
    let mut result_node_voltages = HashMap::new();
    for i in 1..=mna_matrix.num_nodes { result_node_voltages.insert(i, Vec::with_capacity(frequencies.len())); }
    let mut result_vsource_currents = HashMap::new();
    for name in &vsources { result_vsource_currents.insert(name.clone(), Vec::with_capacity(frequencies.len())); }
    
    for (f, node_voltages, vsource_currents) in results {
        for (i, v) in node_voltages.into_iter().enumerate() {
            result_node_voltages.get_mut(&(i + 1)).unwrap().push(v);
        }
        for (idx, name) in vsources.iter().enumerate() {
            result_vsource_currents.get_mut(name).unwrap().push(vsource_currents[idx]);
        }
    }
    
    Ok(AcResult { frequencies, node_voltages: result_node_voltages, vsource_currents: result_vsource_currents })
}

pub fn simulate_dc(netlist: &Netlist, _max_iter: usize, _tol: f64) -> SimResult<DcResult> {
    // Use MNA with omega=0 for DC operating point
    let mut mna = MnaSystem::build(netlist, 0.0);
    
    // Extract real parts for DC solution
    let mut y_matrix_real: Vec<Vec<f64>> = mna.y_matrix.iter()
        .map(|row| row.iter().map(|c| c.re).collect())
        .collect();
    let mut i_vector_real: Vec<f64> = mna.i_vector.iter().map(|c| c.re).collect();
    
    let solution = solve_real_matrix(&mut y_matrix_real, &mut i_vector_real)
        .ok_or(SimError::SingularMatrix { freq: 0.0, cond: 0.0 })?;
    
    let mut node_voltages = HashMap::new();
    for i in 1..=mna.num_nodes {
        node_voltages.insert(i, solution[i - 1]);
    }
    
    let mut vsource_currents = HashMap::new();
    for (idx, (name, _, _)) in mna.vsources.iter().enumerate() {
        vsource_currents.insert(name.clone(), solution[mna.num_nodes + idx]);
    }
    
    let converged = solution.iter().all(|v| v.is_finite());
    
    Ok(DcResult {
        node_voltages,
        vsource_currents,
        converged,
        iterations: 1,
    })
}

pub fn simulate_tran(netlist: &Netlist, sweep: &TranSweep) -> SimResult<TranResult> {
    let time_points = sweep.time_points();
    let mut result_node_voltages = HashMap::new();
    for i in 1..=netlist.max_node { result_node_voltages.insert(i, Vec::with_capacity(time_points.len())); }
    
    let mut vsources = Vec::new();
    for comp in &netlist.components { if let crate::Component::VoltageSource { name, .. } = comp { vsources.push(name.clone()); } }
    let mut result_vsource_currents = HashMap::new();
    for name in &vsources { result_vsource_currents.insert(name.clone(), Vec::with_capacity(time_points.len())); }
    
    // Store initial conditions (t=0) - all voltages start at 0
    for i in 1..=netlist.max_node {
        result_node_voltages.get_mut(&i).unwrap().push(0.0);
    }
    for name in &vsources {
        result_vsource_currents.get_mut(name).unwrap().push(0.0);
    }
    
    let mut prev_voltages = vec![0.0; netlist.max_node];
    let dt = sweep.t_step;
    
    // Skip t=0 since we already stored initial conditions
    for &_t in &time_points[1..] {
        let mut g_matrix = vec![vec![0.0; netlist.max_node]; netlist.max_node];
        let mut i_vector = vec![0.0; netlist.max_node];
        
        for (i, row) in g_matrix.iter_mut().enumerate().take(netlist.max_node) {
            row[i] += crate::types::GMIN;
        }
        
        for comp in &netlist.components {
            match comp {
                crate::Component::Resistor { n_pos, n_neg, value, .. } => {
                    let g = 1.0 / value;
                    add_conductance(&mut g_matrix, *n_pos, *n_neg, g);
                }
                crate::Component::Capacitor { n_pos, n_neg, value, .. } => {
                    let gc = *value / dt;
                    add_conductance(&mut g_matrix, *n_pos, *n_neg, gc);
                    if *n_pos > 0 { i_vector[*n_pos - 1] += gc * prev_voltages[*n_pos - 1]; }
                    if *n_neg > 0 { i_vector[*n_neg - 1] -= gc * prev_voltages[*n_neg - 1]; }
                }
                crate::Component::Inductor { n_pos, n_neg, value, .. } => {
                    let gl = dt / *value;
                    add_conductance(&mut g_matrix, *n_pos, *n_neg, gl);
                }
                crate::Component::CurrentSource { n_pos, n_neg, amplitude, .. } => {
                    if *n_pos > 0 { i_vector[*n_pos - 1] += *amplitude; }
                    if *n_neg > 0 { i_vector[*n_neg - 1] -= *amplitude; }
                }
                crate::Component::VoltageSource { n_pos, n_neg, amplitude, .. } => {
                    let g_vsrc = 1e6;
                    if *n_pos > 0 { 
                        i_vector[*n_pos - 1] += *amplitude * g_vsrc;
                        g_matrix[*n_pos - 1][*n_pos - 1] += g_vsrc;
                    }
                    if *n_neg > 0 { 
                        i_vector[*n_neg - 1] -= *amplitude * g_vsrc;
                        g_matrix[*n_neg - 1][*n_neg - 1] += g_vsrc;
                    }
                }
                _ => {}
            }
        }
        
        let solution = solve_real_matrix(&mut g_matrix, &mut i_vector)
            .ok_or(SimError::SingularMatrix { freq: 0.0, cond: 0.0 })?;
        prev_voltages = solution.clone();
        
        for i in 1..=netlist.max_node {
            if let Some(v) = result_node_voltages.get_mut(&i) {
                v.push(solution[i - 1]);
            } else {
                return Err(SimError::NodeOutOfBounds { node: i, max: netlist.max_node });
            }
        }
        for name in &vsources {
            if let Some(v) = result_vsource_currents.get_mut(name) {
                v.push(0.0);
            } else {
                return Err(SimError::ComponentNotFound { name: name.clone() });
            }
        }
    }
    
    Ok(TranResult { time_points, node_voltages: result_node_voltages, vsource_currents: result_vsource_currents })
}

fn add_conductance(g_matrix: &mut [Vec<f64>], n1: usize, n2: usize, g: f64) {
    if n1 > 0 && n2 > 0 {
        g_matrix[n1 - 1][n1 - 1] += g; g_matrix[n2 - 1][n2 - 1] += g;
        g_matrix[n1 - 1][n2 - 1] -= g; g_matrix[n2 - 1][n1 - 1] -= g;
    } else if n1 > 0 { g_matrix[n1 - 1][n1 - 1] += g;
    } else if n2 > 0 { g_matrix[n2 - 1][n2 - 1] += g; }
}

// Noise Analysis Implementation
pub fn simulate_noise(netlist: &Netlist, sweep: &NoiseSweep, noise_model: &NoiseModel) -> SimResult<NoiseResult> {
    let frequencies = sweep.frequencies();
    let output_node = sweep.output_node;
    let ref_node = sweep.reference_node.unwrap_or(0);
    let input_source = sweep.input_source.clone();

    // Build MNA matrix once
    let mna_matrix = MnaMatrix::build_from_netlist(netlist);
    let vsources: Vec<String> = mna_matrix.vsources.iter().map(|(n, _, _)| n.clone()).collect();
    let num_nodes = mna_matrix.num_nodes;

    let mut output_noise: Vec<f64> = Vec::with_capacity(frequencies.len());
    let mut input_referred_noise: Vec<f64> = Vec::with_capacity(frequencies.len());
    let mut noise_contributions: HashMap<String, Vec<f64>> = HashMap::new();
    let mut component_noise_sources: HashMap<String, f64> = HashMap::new();

    // Identify noise-generating components
    for comp in &netlist.components {
        match comp {
            crate::Component::Resistor { name, value, .. } => {
                // Thermal noise: en = sqrt(4*k*T*R)
                let en = (noise_model.thermal_coeff * *value).sqrt();
                component_noise_sources.insert(name.clone(), en);
            }
            crate::Component::Diode { name, .. } => {
                // Shot noise: in = sqrt(2*q*I)
                // Need DC current - simplified: use typical forward current
                let in_d = (noise_model.shot_coeff * 1e-3).sqrt(); // Assume 1mA
                component_noise_sources.insert(name.clone(), in_d);
            }
            crate::Component::BJT { name, .. } | crate::Component::MOSFET { name, .. } => {
                // Shot + flicker noise
                let in_d = (noise_model.shot_coeff * 1e-3).sqrt();
                component_noise_sources.insert(name.clone(), in_d);
            }
            _ => {}
        }
    }

    let mut output_noise_vec = Vec::with_capacity(frequencies.len());
    let mut input_referred_vec = Vec::with_capacity(frequencies.len());
    let mut noise_contributions: HashMap<String, Vec<f64>> = HashMap::new();

    for comp_name in component_noise_sources.keys() {
        noise_contributions.insert(comp_name.clone(), Vec::with_capacity(frequencies.len()));
    }

    // Solve at each frequency
    for &f in &frequencies {
        let omega = 2.0 * PI * f;
        let mut matrix = mna_matrix.clone();
        matrix.update_for_frequency(omega);

        // Solve for output noise using noise sources
        // For each noise source, we solve the circuit with that source active
        let mut total_output_noise = 0.0;

        // We use the adjoint method (efficient for single output)
        // 1. Solve original circuit for voltages
        let solution = matrix.solve()
            .ok_or(SimError::SingularMatrix { freq: f, cond: 0.0 })?;

        let num_nodes = matrix.num_nodes;

        // 2. Solve adjoint circuit (transpose MNA matrix)
        // For output noise at node 'output_node' referenced to 'ref_node'
        // Adjoint source: 1 at output, -1 at reference (ground)
        let mut adj_i_vector = vec![crate::types::Cx::new(0.0, 0.0); matrix.matrix_size];
        if output_node > 0 {
            adj_i_vector[output_node - 1] = crate::types::Cx::new(1.0, 0.0);
        }
        if ref_node > 0 {
            adj_i_vector[ref_node - 1] = crate::types::Cx::new(-1.0, 0.0);
        }

        // Build transpose matrix for adjoint
        let mut adj_y_matrix = vec![vec![crate::types::Cx::new(0.0, 0.0); matrix.matrix_size]; matrix.matrix_size];
        for i in 0..matrix.matrix_size {
            for j in 0..matrix.matrix_size {
                adj_y_matrix[j][i] = matrix.y_matrix[i][j];
            }
        }

        // Solve adjoint system
        let mut adj_matrix = crate::mna::MnaSystem {
            y_matrix: adj_y_matrix,
            i_vector: adj_i_vector,
            vsources: matrix.vsources.clone(),
            matrix_size: matrix.matrix_size,
            num_nodes: matrix.num_nodes,
        };

        let adj_solution = crate::solver::solve_complex_matrix(&mut adj_matrix.y_matrix, &mut adj_matrix.i_vector)
            .ok_or(SimError::SingularMatrix { freq: f, cond: 0.0 })?;

        // Calculate noise contributions from each component
        for comp in &netlist.components {
            match comp {
                crate::Component::Resistor { name, value, n_pos, n_neg, .. } => {
                    // Thermal noise voltage: en = sqrt(4*k*T*R)
                    let en = (noise_model.thermal_coeff * *value).sqrt();
                    
                    // Get adjoint voltages across resistor
                    let v_adj_pos = if *n_pos > 0 { adj_solution[*n_pos - 1] } else { crate::types::Cx::new(0.0, 0.0) };
                    let v_adj_neg = if *n_neg > 0 { adj_solution[*n_neg - 1] } else { crate::types::Cx::new(0.0, 0.0) };
                    let v_adj_diff = v_adj_pos - v_adj_neg;
                    
                    // Output noise contribution: |V_adj_diff| * en
                    let contrib = v_adj_diff.norm() * en;
                    total_output_noise += contrib * contrib; // Power addition
                    
                    noise_contributions.entry(name.clone()).or_default().push(contrib);
                }
                crate::Component::Diode { name, n_anode, n_cathode, .. } => {
                    // Shot noise current: in = sqrt(2*q*I)
                    let in_d = (noise_model.shot_coeff * 1e-3).sqrt(); // Simplified
                    
                    // Get adjoint voltages
                    let v_adj_anode = if *n_anode > 0 { adj_solution[*n_anode - 1] } else { crate::types::Cx::new(0.0, 0.0) };
                    let v_adj_cathode = if *n_cathode > 0 { adj_solution[*n_cathode - 1] } else { crate::types::Cx::new(0.0, 0.0) };
                    let v_adj_diff = v_adj_anode - v_adj_cathode;
                    
                    let contrib = v_adj_diff.norm() * in_d;
                    total_output_noise += contrib * contrib;
                    
                    noise_contributions.entry(name.clone()).or_default().push(contrib);
                }
                crate::Component::BJT { name, n_collector, n_base, n_emitter, .. } => {
                    // Shot + flicker noise
                    let in_shot = (noise_model.shot_coeff * 1e-3).sqrt();
                    let in_flicker = if f > 0.0 {
                        noise_model.flicker_k * (1e-3_f64).powf(noise_model.flicker_af) / f.powf(noise_model.flicker_bf)
                    } else { 0.0 };
                    
                    let v_adj_c = if *n_collector > 0 { adj_solution[*n_collector - 1] } else { crate::types::Cx::new(0.0, 0.0) };
                    let v_adj_e = if *n_emitter > 0 { adj_solution[*n_emitter - 1] } else { crate::types::Cx::new(0.0, 0.0) };
                    
                    let v_adj = v_adj_c - v_adj_e;
                    let contrib = v_adj.norm() * (in_shot + in_flicker);
                    total_output_noise += contrib * contrib;
                    
                    noise_contributions.entry(name.clone()).or_default().push(contrib);
                }
                crate::Component::MOSFET { name, n_drain, n_gate, n_source, n_body, .. } => {
                    // Shot + flicker noise
                    let in_shot = (noise_model.shot_coeff * 1e-3).sqrt();
                    let in_flicker = if f > 0.0 {
                        noise_model.flicker_k * (1e-3_f64).powf(noise_model.flicker_af) / f.powf(noise_model.flicker_bf)
                    } else { 0.0 };
                    
                    let v_adj_d = if *n_drain > 0 { adj_solution[*n_drain - 1] } else { crate::types::Cx::new(0.0, 0.0) };
                    let v_adj_s = if *n_source > 0 { adj_solution[*n_source - 1] } else { crate::types::Cx::new(0.0, 0.0) };
                    
                    let v_adj = v_adj_d - v_adj_s;
                    let contrib = v_adj.norm() * (in_shot + in_flicker);
                    total_output_noise += contrib * contrib;
                    
                    noise_contributions.entry(name.clone()).or_default().push(contrib);
                }
                _ => {}
            }
        }

        let total_output = total_output_noise.sqrt();
        output_noise_vec.push(total_output);

        // Input-referred noise: divide by gain if input source specified
        if let Some(ref src_name) = input_source {
            if let Some(idx) = vsources.iter().position(|n| n == src_name) {
                // Get transfer function from input to output
                let solution = matrix.solve().unwrap();
                let out_voltage = if output_node > 0 { solution[output_node - 1] } else { crate::types::Cx::new(0.0, 0.0) };
                let in_voltage = if idx < solution.len() { solution[matrix.num_nodes + idx] } else { crate::types::Cx::new(0.0, 0.0) };
                let gain = if in_voltage.norm() > 0.0 { out_voltage / in_voltage } else { crate::types::Cx::new(0.0, 0.0) };
                
                if gain.norm() > 0.0 {
                    input_referred_vec.push(total_output / gain.norm());
                } else {
                    input_referred_vec.push(f64::INFINITY);
                }
            } else {
                input_referred_vec.push(f64::INFINITY);
            }
        } else {
            input_referred_vec.push(f64::INFINITY);
        }
    }

    // Calculate total integrated RMS noise
    let total_rms = integrate_noise(&frequencies, &output_noise_vec);

    Ok(NoiseResult {
        frequencies,
        output_noise: output_noise_vec,
        input_referred_noise: input_referred_vec,
        noise_contributions,
        total_rms_noise: total_rms,
    })
}

fn integrate_noise(frequencies: &[f64], noise: &[f64]) -> f64 {
    if frequencies.len() != noise.len() || frequencies.len() < 2 {
        return 0.0;
    }
    
    let mut total = 0.0;
    for i in 1..frequencies.len() {
        let df = frequencies[i] - frequencies[i - 1];
        let avg_noise = (noise[i] + noise[i - 1]) / 2.0;
        total += avg_noise * avg_noise * df; // Power integration
    }
    total.sqrt()
}

// Sensitivity Analysis Implementation
pub fn simulate_sensitivity(netlist: &Netlist, sweep: &SensitivitySweep) -> SimResult<SensitivityResult> {
    let frequencies = sweep.frequencies();
    let output_node = sweep.output_node;
    let ref_node = sweep.reference_node.unwrap_or(0);
    let component_names = &sweep.component_names;

    // Build MNA matrix once
    let mna_matrix = MnaMatrix::build_from_netlist(netlist);

    let mut sensitivities: HashMap<String, Vec<f64>> = HashMap::new();
    let mut worst_case = HashMap::new();
    let mut relative_sensitivities: HashMap<String, Vec<f64>> = HashMap::new();

    for name in component_names {
        sensitivities.insert(name.clone(), Vec::with_capacity(frequencies.len()));
        relative_sensitivities.insert(name.clone(), Vec::with_capacity(frequencies.len()));
        worst_case.insert(name.clone(), 0.0);
    }

    for &f in &frequencies {
        let omega = 2.0 * PI * f;
        let mut matrix = mna_matrix.clone();
        matrix.update_for_frequency(omega);

        let solution = matrix.solve()
            .ok_or(SimError::SingularMatrix { freq: f, cond: 0.0 })?;

        // Nominal output voltage
        let vout_nominal = if output_node > 0 { solution[output_node - 1] } else { crate::types::Cx::new(0.0, 0.0) };
        let vout_mag = vout_nominal.norm();

        for name in component_names {
            // Find component
            let comp_opt = netlist.components.iter().find(|c| c.get_name() == Some(name));
            if comp_opt.is_none() {
                sensitivities.get_mut(name).unwrap().push(0.0);
                relative_sensitivities.get_mut(name).unwrap().push(0.0);
                continue;
            }
            let comp = comp_opt.unwrap();

            // Perturb component value by 1%
            let mut perturbed_matrix = mna_matrix.clone();
            perturb_component(&mut perturbed_matrix, comp, 0.01);
            perturbed_matrix.update_for_frequency(2.0 * PI * f);

            let perturbed_solution = perturbed_matrix.solve()
                .ok_or(SimError::SingularMatrix { freq: f, cond: 0.0 })?;

            let vout_perturbed = if output_node > 0 { perturbed_solution[output_node - 1] } else { crate::types::Cx::new(0.0, 0.0) };
            let vout_perturbed_mag = vout_perturbed.norm();

            // Sensitivity: (ΔVout/Vout) / (ΔP/P) ≈ (Vout_pert - Vout_nom) / Vout_nom / 0.01
            let sens = if vout_mag > 0.0 {
                (vout_perturbed_mag - vout_mag) / vout_mag / 0.01
            } else { 0.0 };

            sensitivities.get_mut(name).unwrap().push(sens);
            relative_sensitivities.get_mut(name).unwrap().push(sens.abs());

            let wc = worst_case.get_mut(name).unwrap();
            if sens.abs() > *wc { *wc = sens.abs(); }
        }
    }

    Ok(SensitivityResult {
        frequencies,
        sensitivities,
        worst_case,
        relative_sensitivities,
    })
}

fn perturb_component(matrix: &mut crate::mna::MnaMatrix, comp: &crate::Component, delta: f64) {
    // Find the corresponding entries in freq_dependent_entries or static_entries and perturb them
    match comp {
        crate::Component::Resistor { value, .. } => {
            // Resistors are in static_entries with value 1/R
            let new_r = *value * (1.0 + delta);
            let new_g = 1.0 / new_r;
            let old_g = 1.0 / *value;
            for entry in &mut matrix.static_entries {
                if (entry.value.re - old_g).abs() < 1e-12 {
                    entry.value = crate::types::Cx::new(new_g, 0.0);
                }
            }
        }
        crate::Component::Capacitor { value, .. } => {
            // Capacitors are in freq_dependent_entries
            let new_c = *value * (1.0 + delta);
for entry in &mut matrix.freq_dependent_entries {
            if (entry.value - *value).abs() < 1e-12 && matches!(entry.component_type, FreqDepType::Capacitor) {
                entry.value = new_c;
            }
        }
        }
        crate::Component::Inductor { value, .. } => {
            // Inductors are in freq_dependent_entries
            let new_l = *value * (1.0 + delta);
for entry in &mut matrix.freq_dependent_entries {
            if (entry.value - *value).abs() < 1e-12 && matches!(entry.component_type, FreqDepType::Inductor) {
                entry.value = new_l;
            }
        }
        }
        _ => {}
    }
}

// Monte Carlo Analysis
pub fn simulate_monte_carlo(netlist: &Netlist, sweep: &AcSweep, config: &MonteCarloConfig, output_node: usize) -> SimResult<MonteCarloResult> {
    let frequencies = sweep.frequencies();
    
    let mut all_outputs: Vec<Vec<f64>> = Vec::with_capacity(config.num_runs);
    
    for run in 0..config.num_runs {
        // Create perturbed netlist
        let mut perturbed_netlist = Netlist::new();
        for comp in &netlist.components {
            let mut new_comp = comp.clone();
            if let Some(tol) = config.component_tolerances.get(comp.get_name().unwrap_or("")) {
                perturb_component_value(&mut new_comp, *tol, config.distribution);
            }
            perturbed_netlist.add_component(new_comp);
        }
        
        let result = simulate_ac(&perturbed_netlist, sweep)?;
        let vout: Vec<f64> = result.node_voltages.get(&output_node)
            .map(|v| v.iter().map(|c| c.norm()).collect())
            .unwrap_or_else(|| vec![0.0; frequencies.len()]);
        
        all_outputs.push(vout);
    }
    
    // Compute statistics
    let mut output_mean = vec![0.0; frequencies.len()];
    let mut output_std = vec![0.0; frequencies.len()];
    let mut output_min = vec![f64::INFINITY; frequencies.len()];
    let mut output_max = vec![f64::NEG_INFINITY; frequencies.len()];
    
    for i in 0..frequencies.len() {
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        for run in 0..config.num_runs {
            let v = all_outputs[run][i];
            sum += v;
            sum_sq += v * v;
            if v < output_min[i] { output_min[i] = v; }
            if v > output_max[i] { output_max[i] = v; }
        }
        let mean = sum / config.num_runs as f64;
        let variance = (sum_sq / config.num_runs as f64) - mean * mean;
        output_mean[i] = mean;
        output_std[i] = variance.max(0.0).sqrt();
    }
    
    // Build histogram data
    let mut histogram_data = HashMap::new();
    let num_bins = 20;
    for i in 0..frequencies.len() {
        let mut hist = vec![0; num_bins];
        let min = output_min[i];
        let max = output_max[i];
        let range = max - min;
        if range > 0.0 {
            for run in 0..config.num_runs {
                let v = all_outputs[run][i];
                let bin = ((v - min) / range * num_bins as f64).floor() as usize;
                let bin = bin.min(num_bins - 1);
                hist[bin] += 1;
            }
        }
        histogram_data.insert(i, hist.into_iter().map(|x| x as f64).collect());
    }
    
    // Yield estimate (simplified: fraction within ±3σ)
    let mut in_spec = 0;
    for run in 0..config.num_runs {
        let mut all_in = true;
        for i in 0..frequencies.len() {
            let v = all_outputs[run][i];
            if (v - output_mean[i]).abs() > 3.0 * output_std[i] {
                all_in = false;
                break;
            }
        }
        if all_in { in_spec += 1; }
    }
    let yield_estimate = in_spec as f64 / config.num_runs as f64;
    
    Ok(MonteCarloResult {
        output_mean,
        output_std,
        output_min,
        output_max,
        histogram_data,
        yield_estimate,
    })
}

fn perturb_component_value(comp: &mut crate::Component, tolerance: f64, dist: DistributionType) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let delta = match dist {
        DistributionType::Uniform => rng.gen_range(-tolerance..tolerance),
        DistributionType::Gaussian => rng.gen::<f64>() * tolerance * 0.33, // ~3σ
        DistributionType::LogNormal => {
            let log_tol = tolerance.ln();
            let normal = rng.gen::<f64>() * log_tol * 0.33;
            (normal.exp() - 1.0)
        }
    };
    
    match comp {
        crate::Component::Resistor { value, .. } => *value *= 1.0 + delta,
        crate::Component::Capacitor { value, .. } => *value *= 1.0 + delta,
        crate::Component::Inductor { value, .. } => *value *= 1.0 + delta,
        _ => {}
    }
}
