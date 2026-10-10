use crate::mna::MnaSystem;
use crate::netlist::Netlist;
use crate::simulation::{AcResult, AcSweep};
use crate::solver::solve_complex_matrix;
use std::collections::HashMap;
use std::f64::consts::PI;

pub fn simulate_ac(netlist: &Netlist, sweep: &AcSweep) -> Result<AcResult, String> {
    let frequencies = sweep.frequencies();
    let mut result_node_voltages = HashMap::new();
    for i in 1..=netlist.max_node { result_node_voltages.insert(i, Vec::with_capacity(frequencies.len())); }
    
    let mut vsources = Vec::new();
    for comp in &netlist.components { if let crate::Component::VoltageSource { name, .. } = comp { vsources.push(name.clone()); } }
    let mut result_vsource_currents = HashMap::new();
    for name in &vsources { result_vsource_currents.insert(name.clone(), Vec::with_capacity(frequencies.len())); }

    for &f in &frequencies {
        let mut mna = MnaSystem::build(netlist, 2.0 * PI * f);
        let solution = solve_complex_matrix(&mut mna.y_matrix, &mut mna.i_vector).ok_or_else(|| format!("Singular matrix at {} Hz", f))?;
        for i in 1..=mna.num_nodes { result_node_voltages.get_mut(&i).unwrap().push(solution[i - 1]); }
        for (idx, name) in vsources.iter().enumerate() { result_vsource_currents.get_mut(name).unwrap().push(solution[mna.num_nodes + idx]); }
    }
    Ok(AcResult { frequencies, node_voltages: result_node_voltages, vsource_currents: result_vsource_currents })
}
