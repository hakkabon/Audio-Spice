use crate::error::{SimError, SimResult};
use crate::mna::{MnaMatrix, MnaSystem};
use crate::netlist::Netlist;
use crate::simulation::{AcResult, AcSweep, DcResult, TranResult, TranSweep};
use crate::solver::solve_real_matrix;
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
