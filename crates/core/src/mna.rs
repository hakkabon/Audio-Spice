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
            if let Component::VoltageSource { name, n_pos, n_neg, .. } = comp {
                vsources.push((name.clone(), *n_pos, *n_neg));
            }
        }
        
        let matrix_size = num_nodes + vsources.len();
        let mut y_matrix = vec![vec![Cx::new(0.0, 0.0); matrix_size]; matrix_size];
        let mut i_vector = vec![Cx::new(0.0, 0.0); matrix_size];
        
        for i in 0..num_nodes { y_matrix[i][i] += Cx::new(GMIN, 0.0); }

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
            }
        }
        Self { y_matrix, i_vector, vsources, matrix_size, num_nodes }
    }
}

fn add_admittance(y_matrix: &mut Vec<Vec<Cx>>, n1: usize, n2: usize, y: Cx) {
    if n1 > 0 && n2 > 0 {
        y_matrix[n1 - 1][n1 - 1] += y; y_matrix[n2 - 1][n2 - 1] += y;
        y_matrix[n1 - 1][n2 - 1] -= y; y_matrix[n2 - 1][n1 - 1] -= y;
    } else if n1 > 0 { y_matrix[n1 - 1][n1 - 1] += y;
    } else if n2 > 0 { y_matrix[n2 - 1][n2 - 1] += y; }
}
