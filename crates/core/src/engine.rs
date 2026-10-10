use crate::error::{SimError, SimResult};
use crate::mna::MnaSystem;
use crate::netlist::Netlist;
use crate::simulation::{AcResult, AcSweep, DcResult, TranResult, TranSweep};
use crate::solver::{solve_complex_matrix, solve_real_matrix};
use std::collections::HashMap;
use std::f64::consts::PI;

pub fn simulate_ac(netlist: &Netlist, sweep: &AcSweep) -> SimResult<AcResult> {
    let frequencies = sweep.frequencies();
    let mut result_node_voltages = HashMap::new();
    for i in 1..=netlist.max_node { result_node_voltages.insert(i, Vec::with_capacity(frequencies.len())); }
    
    let mut vsources = Vec::new();
    for comp in &netlist.components { if let crate::Component::VoltageSource { name, .. } = comp { vsources.push(name.clone()); } }
    let mut result_vsource_currents = HashMap::new();
    for name in &vsources { result_vsource_currents.insert(name.clone(), Vec::with_capacity(frequencies.len())); }

    for &f in &frequencies {
        let mut mna = MnaSystem::build(netlist, 2.0 * PI * f);
        let solution = solve_complex_matrix(&mut mna.y_matrix, &mut mna.i_vector)
            .ok_or(SimError::SingularMatrix { freq: f, cond: 0.0 })?;
        
        for i in 1..=mna.num_nodes {
            if let Some(v) = result_node_voltages.get_mut(&i) {
                v.push(solution[i - 1]);
            } else {
                return Err(SimError::NodeOutOfBounds { node: i, max: netlist.max_node });
            }
        }
        
        for (idx, name) in vsources.iter().enumerate() {
            if let Some(v) = result_vsource_currents.get_mut(name) {
                v.push(solution[mna.num_nodes + idx]);
            } else {
                return Err(SimError::ComponentNotFound { name: name.clone() });
            }
        }
    }
    Ok(AcResult { frequencies, node_voltages: result_node_voltages, vsource_currents: result_vsource_currents })
}

pub fn simulate_dc(netlist: &Netlist, max_iter: usize, tol: f64) -> SimResult<DcResult> {
    let mut voltages = vec![0.0; netlist.max_node];
    let mut prev_voltages = vec![0.0; netlist.max_node];
    
    let mut vsources: Vec<(String, usize, usize, f64)> = Vec::new();
    for comp in &netlist.components {
        if let crate::Component::VoltageSource { name, n_pos, n_neg, amplitude, .. } = comp {
            vsources.push((name.clone(), *n_pos, *n_neg, *amplitude));
        }
    }
    
    let mut converged = false;
    let mut iterations = 0;
    
    for iter in 0..max_iter {
        prev_voltages.clone_from(&voltages);
        
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
                crate::Component::CurrentSource { n_pos, n_neg, amplitude, .. } => {
                    if *n_pos > 0 { i_vector[*n_pos - 1] += *amplitude; }
                    if *n_neg > 0 { i_vector[*n_neg - 1] -= *amplitude; }
                }
                crate::Component::VoltageSource { n_pos, n_neg, amplitude, .. } => {
                    if *n_pos > 0 { i_vector[*n_pos - 1] += *amplitude * 1e6; }
                    if *n_neg > 0 { i_vector[*n_neg - 1] -= *amplitude * 1e6; }
                }
                crate::Component::VCCS { n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, transconductance, .. } => {
                    let gm = *transconductance;
                    if *n_out_pos > 0 && *n_ctrl_pos > 0 { g_matrix[*n_out_pos - 1][*n_ctrl_pos - 1] += gm; }
                    if *n_out_pos > 0 && *n_ctrl_neg > 0 { g_matrix[*n_out_pos - 1][*n_ctrl_neg - 1] -= gm; }
                    if *n_out_neg > 0 && *n_ctrl_pos > 0 { g_matrix[*n_out_neg - 1][*n_ctrl_pos - 1] -= gm; }
                    if *n_out_neg > 0 && *n_ctrl_neg > 0 { g_matrix[*n_out_neg - 1][*n_ctrl_neg - 1] += gm; }
                }
                crate::Component::VCVS { n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, gain, .. } => {
                    let a = *gain;
                    if *n_out_pos > 0 && *n_ctrl_pos > 0 { g_matrix[*n_out_pos - 1][*n_ctrl_pos - 1] += a; }
                    if *n_out_pos > 0 && *n_ctrl_neg > 0 { g_matrix[*n_out_pos - 1][*n_ctrl_neg - 1] -= a; }
                    if *n_out_neg > 0 && *n_ctrl_pos > 0 { g_matrix[*n_out_neg - 1][*n_ctrl_pos - 1] -= a; }
                    if *n_out_neg > 0 && *n_ctrl_neg > 0 { g_matrix[*n_out_neg - 1][*n_ctrl_neg - 1] += a; }
                }
                crate::Component::OpAmp { n_out, n_in_pos, n_in_neg, gain, .. } => {
                    let a = *gain;
                    if *n_out > 0 && *n_in_pos > 0 { g_matrix[*n_out - 1][*n_in_pos - 1] += a; }
                    if *n_out > 0 && *n_in_neg > 0 { g_matrix[*n_out - 1][*n_in_neg - 1] -= a; }
                }
                _ => {}
            }
        }
        
        let solution = solve_real_matrix(&mut g_matrix, &mut i_vector)
            .ok_or(SimError::SingularMatrix { freq: 0.0, cond: 0.0 })?;
        voltages = solution;
        
        let max_diff = voltages.iter().zip(prev_voltages.iter())
            .map(|(v, pv)| (v - pv).abs())
            .fold(0.0, f64::max);
        
        iterations = iter + 1;
        if max_diff < tol {
            converged = true;
            break;
        }
    }
    
    let mut node_voltages = HashMap::new();
    for i in 1..=netlist.max_node {
        node_voltages.insert(i, voltages[i - 1]);
    }
    
    let mut vsource_currents = HashMap::new();
    for (name, _, _, amp) in &vsources {
        vsource_currents.insert(name.clone(), *amp);
    }
    
    Ok(DcResult { node_voltages, vsource_currents, converged, iterations })
}

pub fn simulate_tran(netlist: &Netlist, sweep: &TranSweep) -> SimResult<TranResult> {
    let time_points = sweep.time_points();
    let mut result_node_voltages = HashMap::new();
    for i in 1..=netlist.max_node { result_node_voltages.insert(i, Vec::with_capacity(time_points.len())); }
    
    let mut vsources = Vec::new();
    for comp in &netlist.components { if let crate::Component::VoltageSource { name, .. } = comp { vsources.push(name.clone()); } }
    let mut result_vsource_currents = HashMap::new();
    for name in &vsources { result_vsource_currents.insert(name.clone(), Vec::with_capacity(time_points.len())); }
    
    let mut prev_voltages = vec![0.0; netlist.max_node];
    let dt = sweep.t_step;
    
    for &_t in &time_points {
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
                    if *n_pos > 0 { i_vector[*n_pos - 1] += *amplitude * 1e6; }
                    if *n_neg > 0 { i_vector[*n_neg - 1] -= *amplitude * 1e6; }
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
