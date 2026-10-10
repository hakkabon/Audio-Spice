use crate::component::Component;
use crate::netlist::Netlist;
use crate::types::{Cx, GMIN};

pub struct MnaSystem {
    pub y_matrix: Vec<Vec<Cx>>,
    pub i_vector: Vec<Cx>,
    pub vsources: Vec<(String, usize, usize)>,
    pub matrix_size: usize,
    pub num_nodes: usize,
}

impl MnaSystem {
    pub fn build(netlist: &Netlist, omega: f64) -> Self {
        let num_nodes = netlist.max_node;
        let mut vsources = Vec::new();
        for comp in &netlist.components {
            if matches!(comp, Component::VoltageSource { .. } | Component::CCVS { .. } | Component::VCVS { .. }) {
                if let Component::VoltageSource { name, n_pos, n_neg, .. } = comp {
                    vsources.push((name.clone(), *n_pos, *n_neg));
                } else if let Component::CCVS { name, n_out_pos, n_out_neg, .. } = comp {
                    vsources.push((name.clone(), *n_out_pos, *n_out_neg));
                } else if let Component::VCVS { name, n_out_pos, n_out_neg, .. } = comp {
                    vsources.push((name.clone(), *n_out_pos, *n_out_neg));
                }
            }
        }
        
        let matrix_size = num_nodes + vsources.len();
        let mut y_matrix = vec![vec![Cx::new(0.0, 0.0); matrix_size]; matrix_size];
        let mut i_vector = vec![Cx::new(0.0, 0.0); matrix_size];
        
        for (i, row) in y_matrix.iter_mut().enumerate().take(num_nodes) {
            row[i] += Cx::new(GMIN, 0.0);
        }

        for comp in &netlist.components {
            match comp {
                Component::Resistor { n_pos, n_neg, .. } |
                Component::Capacitor { n_pos, n_neg, .. } |
                Component::Inductor { n_pos, n_neg, .. } => {
                    if let Some(y) = comp.admittance(omega) {
                        add_admittance(&mut y_matrix, *n_pos, *n_neg, y);
                    }
                }
                Component::CurrentSource { n_pos, n_neg, amplitude, phase_deg, .. } => {
                    let i_cx = Cx::from_polar(*amplitude, phase_deg.to_radians());
                    if *n_pos > 0 { i_vector[*n_pos - 1] += i_cx; }
                    if *n_neg > 0 { i_vector[*n_neg - 1] -= i_cx; }
                }
                Component::VoltageSource { name, n_pos, n_neg, amplitude, phase_deg, .. } => {
                    let v_cx = Cx::from_polar(*amplitude, phase_deg.to_radians());
                    let vs_idx = vsources.iter().position(|(n, p, neg)| n == name && *p == *n_pos && *neg == *n_neg).unwrap();
                    let m_idx = num_nodes + vs_idx;
                    
                    if *n_pos > 0 { y_matrix[*n_pos - 1][m_idx] += Cx::new(1.0, 0.0); }
                    if *n_neg > 0 { y_matrix[*n_neg - 1][m_idx] -= Cx::new(1.0, 0.0); }
                    if *n_pos > 0 { y_matrix[m_idx][*n_pos - 1] += Cx::new(1.0, 0.0); }
                    if *n_neg > 0 { y_matrix[m_idx][*n_neg - 1] -= Cx::new(1.0, 0.0); }
                    i_vector[m_idx] = v_cx;
                }
                Component::VCCS { n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, transconductance, .. } => {
                    let gm = Cx::new(*transconductance, 0.0);
                    if *n_out_pos > 0 && *n_ctrl_pos > 0 { y_matrix[*n_out_pos - 1][*n_ctrl_pos - 1] += gm; }
                    if *n_out_pos > 0 && *n_ctrl_neg > 0 { y_matrix[*n_out_pos - 1][*n_ctrl_neg - 1] -= gm; }
                    if *n_out_neg > 0 && *n_ctrl_pos > 0 { y_matrix[*n_out_neg - 1][*n_ctrl_pos - 1] -= gm; }
                    if *n_out_neg > 0 && *n_ctrl_neg > 0 { y_matrix[*n_out_neg - 1][*n_ctrl_neg - 1] += gm; }
                }
                Component::VCVS { name, n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, gain, .. } => {
                    let vs_idx = vsources.iter().position(|(n, _, _)| n == name).unwrap_or(0);
                    let m_idx = num_nodes + vs_idx;
                    let a = Cx::new(*gain, 0.0);
                    
                    // VCVS constraint: V_out_pos - V_out_neg - gain*V_ctrl_pos + gain*V_ctrl_neg = 0
                    if *n_out_pos > 0 { y_matrix[m_idx][*n_out_pos - 1] += Cx::new(1.0, 0.0); }
                    if *n_out_neg > 0 { y_matrix[m_idx][*n_out_neg - 1] -= Cx::new(1.0, 0.0); }
                    if *n_ctrl_pos > 0 { y_matrix[m_idx][*n_ctrl_pos - 1] -= a; }
                    if *n_ctrl_neg > 0 { y_matrix[m_idx][*n_ctrl_neg - 1] += a; }
                    
                    // Current variable I_E: +1 at out_pos, -1 at out_neg
                    if *n_out_pos > 0 { y_matrix[*n_out_pos - 1][m_idx] += Cx::new(1.0, 0.0); }
                    if *n_out_neg > 0 { y_matrix[*n_out_neg - 1][m_idx] -= Cx::new(1.0, 0.0); }
                    
                    i_vector[m_idx] = Cx::new(0.0, 0.0);
                }
                Component::CCCS { n_out_pos, n_out_neg, ctrl_vsrc_name, gain, .. } => {
                    if let Some(vs_idx) = vsources.iter().position(|(n, _, _)| n == ctrl_vsrc_name) {
                        let m_idx = num_nodes + vs_idx;
                        let a = Cx::new(*gain, 0.0);
                        
                        if *n_out_pos > 0 { y_matrix[*n_out_pos - 1][m_idx] += a; }
                        if *n_out_neg > 0 { y_matrix[*n_out_neg - 1][m_idx] -= a; }
                    }
                }
                Component::CCVS { n_out_pos, n_out_neg, ctrl_vsrc_name, transresistance, .. } => {
                    if let Some(vs_idx) = vsources.iter().position(|(n, _, _)| n == ctrl_vsrc_name) {
                        let m_idx = num_nodes + vs_idx;
                        let r = Cx::new(*transresistance, 0.0);
                        
                        if *n_out_pos > 0 { y_matrix[*n_out_pos - 1][m_idx] += Cx::new(1.0, 0.0); }
                        if *n_out_neg > 0 { y_matrix[*n_out_neg - 1][m_idx] -= Cx::new(1.0, 0.0); }
                        y_matrix[m_idx][m_idx] += r;
                    }
                }
                Component::OpAmp { n_out, n_in_pos, n_in_neg, gain, .. } => {
                    let a = Cx::new(*gain, 0.0);
                    if *n_out > 0 && *n_in_pos > 0 { y_matrix[*n_out - 1][*n_in_pos - 1] += a; }
                    if *n_out > 0 && *n_in_neg > 0 { y_matrix[*n_out - 1][*n_in_neg - 1] -= a; }
                }
                Component::Diode { n_anode, n_cathode, .. } => {
                    let gmin = Cx::new(GMIN, 0.0);
                    if *n_anode > 0 && *n_cathode > 0 {
                        add_admittance(&mut y_matrix, *n_anode, *n_cathode, gmin);
                    }
                }
                Component::BJT { n_collector, n_base, n_emitter, .. } => {
                    let gmin = Cx::new(GMIN, 0.0);
                    if *n_collector > 0 && *n_emitter > 0 {
                        add_admittance(&mut y_matrix, *n_collector, *n_emitter, gmin);
                    }
                    if *n_base > 0 && *n_emitter > 0 {
                        add_admittance(&mut y_matrix, *n_base, *n_emitter, gmin);
                    }
                }
                Component::MOSFET { n_drain, n_gate, n_source, n_body, .. } => {
                    let gmin = Cx::new(GMIN, 0.0);
                    if *n_drain > 0 && *n_source > 0 {
                        add_admittance(&mut y_matrix, *n_drain, *n_source, gmin);
                    }
                    if *n_gate > 0 && *n_source > 0 {
                        add_admittance(&mut y_matrix, *n_gate, *n_source, gmin);
                    }
                    if *n_body > 0 && *n_source > 0 {
                        add_admittance(&mut y_matrix, *n_body, *n_source, gmin);
                    }
                }
            }
        }
        Self { y_matrix, i_vector, vsources, matrix_size, num_nodes }
    }
}

fn add_admittance(y_matrix: &mut [Vec<Cx>], n1: usize, n2: usize, y: Cx) {
    if n1 > 0 && n2 > 0 {
        y_matrix[n1 - 1][n1 - 1] += y; y_matrix[n2 - 1][n2 - 1] += y;
        y_matrix[n1 - 1][n2 - 1] -= y; y_matrix[n2 - 1][n1 - 1] -= y;
    } else if n1 > 0 { y_matrix[n1 - 1][n1 - 1] += y;
    } else if n2 > 0 { y_matrix[n2 - 1][n2 - 1] += y; }
}
