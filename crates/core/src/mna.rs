use crate::component::Component;
use crate::netlist::Netlist;
use crate::types::{Cx, GMIN};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct MnaMatrix {
    pub y_matrix: Vec<Vec<Cx>>,
    pub i_vector: Vec<Cx>,
    pub vsources: Vec<(String, usize, usize)>,
    pub matrix_size: usize,
    pub num_nodes: usize,
    // Static topology info for reuse
    pub static_entries: Vec<StaticEntry>,
    pub freq_dependent_entries: Vec<FreqDependentEntry>,
    pub source_entries: Vec<SourceEntry>,
}

#[derive(Debug, Clone, Copy)]
pub struct StaticEntry {
    pub row: usize,
    pub col: usize,
    pub value: Cx,
}

#[derive(Debug, Clone)]
pub struct FreqDependentEntry {
    pub row: usize,
    pub col: usize,
    pub component_type: FreqDepType,
    pub value: f64, // Component value (C, L, R)
}

#[derive(Debug, Clone, Copy)]
pub enum FreqDepType {
    Resistor,  // 1/R
    Capacitor, // j*ω*C
    Inductor,  // -j/(ω*L)
}

#[derive(Debug, Clone)]
pub struct SourceEntry {
    pub row: usize,
    pub col: usize,
    pub amplitude: f64,
    pub phase_deg: f64,
    pub is_voltage: bool,
}

impl MnaMatrix {
    pub fn build_from_netlist(netlist: &Netlist) -> Self {
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
        
        let mut static_entries = Vec::new();
        let mut freq_dependent_entries = Vec::new();
        let mut source_entries = Vec::new();
        
        // Add GMIN to diagonal as static entries
        let gmin = Cx::new(GMIN, 0.0);
        for i in 0..num_nodes {
            static_entries.push(StaticEntry { row: i, col: i, value: gmin });
        }
        
        // Build static entries and record frequency-dependent ones
        for comp in &netlist.components {
            match comp {
                Component::Resistor { n_pos, n_neg, value, .. } => {
                    if *n_pos > 0 && *n_neg > 0 {
                        static_entries.push(StaticEntry { row: *n_pos - 1, col: *n_pos - 1, value: Cx::new(1.0 / *value, 0.0) });
                        static_entries.push(StaticEntry { row: *n_neg - 1, col: *n_neg - 1, value: Cx::new(1.0 / *value, 0.0) });
                        static_entries.push(StaticEntry { row: *n_pos - 1, col: *n_neg - 1, value: Cx::new(-1.0 / *value, 0.0) });
                        static_entries.push(StaticEntry { row: *n_neg - 1, col: *n_pos - 1, value: Cx::new(-1.0 / *value, 0.0) });
                    } else if *n_pos > 0 {
                        static_entries.push(StaticEntry { row: *n_pos - 1, col: *n_pos - 1, value: Cx::new(1.0 / *value, 0.0) });
                    } else if *n_neg > 0 {
                        static_entries.push(StaticEntry { row: *n_neg - 1, col: *n_neg - 1, value: Cx::new(1.0 / *value, 0.0) });
                    }
                }
                Component::Capacitor { n_pos, n_neg, value, .. } => {
                    let n_pos = *n_pos;
                    let n_neg = *n_neg;
                    let row_pos = if n_pos > 0 { n_pos - 1 } else { 0 };
                    
                    if n_neg == 0 {
                        // Connected to ground: only add to positive node diagonal
                        freq_dependent_entries.push(FreqDependentEntry {
                            row: row_pos,
                            col: row_pos,
                            component_type: FreqDepType::Capacitor,
                            value: *value,
                        });
                    } else {
                        let row_neg = n_neg - 1;
                        freq_dependent_entries.push(FreqDependentEntry {
                            row: row_pos,
                            col: row_pos,
                            component_type: FreqDepType::Capacitor,
                            value: *value,
                        });
                        freq_dependent_entries.push(FreqDependentEntry {
                            row: n_neg - 1,
                            col: n_neg - 1,
                            component_type: FreqDepType::Capacitor,
                            value: *value,
                        });
                        freq_dependent_entries.push(FreqDependentEntry {
                            row: row_pos,
                            col: n_neg - 1,
                            component_type: FreqDepType::Capacitor,
                            value: -*value,
                        });
                        freq_dependent_entries.push(FreqDependentEntry {
                            row: n_neg - 1,
                            col: row_pos,
                            component_type: FreqDepType::Capacitor,
                            value: -*value,
                        });
                    }
                }
                Component::Inductor { n_pos, n_neg, value, .. } => {
                    let n_pos = *n_pos;
                    let n_neg = *n_neg;
                    let row_pos = if n_pos > 0 { n_pos - 1 } else { 0 };
                    
                    if n_neg == 0 {
                        // Connected to ground: only add to positive node diagonal
                        freq_dependent_entries.push(FreqDependentEntry {
                            row: row_pos,
                            col: row_pos,
                            component_type: FreqDepType::Inductor,
                            value: *value,
                        });
                    } else {
                        let row_neg = n_neg - 1;
                        freq_dependent_entries.push(FreqDependentEntry {
                            row: row_pos,
                            col: row_pos,
                            component_type: FreqDepType::Inductor,
                            value: *value,
                        });
                        freq_dependent_entries.push(FreqDependentEntry {
                            row: n_neg - 1,
                            col: n_neg - 1,
                            component_type: FreqDepType::Inductor,
                            value: *value,
                        });
                        freq_dependent_entries.push(FreqDependentEntry {
                            row: row_pos,
                            col: n_neg - 1,
                            component_type: FreqDepType::Inductor,
                            value: -*value,
                        });
                        freq_dependent_entries.push(FreqDependentEntry {
                            row: n_neg - 1,
                            col: row_pos,
                            component_type: FreqDepType::Inductor,
                            value: -*value,
                        });
                    }
                }
                Component::CurrentSource { n_pos, n_neg, amplitude, phase_deg, .. } => {
                    if *n_pos > 0 {
                        source_entries.push(SourceEntry {
                            row: *n_pos - 1,
                            col: 0,
                            amplitude: *amplitude,
                            phase_deg: *phase_deg,
                            is_voltage: false,
                        });
                    }
                    if *n_neg > 0 {
                        source_entries.push(SourceEntry {
                            row: *n_neg - 1,
                            col: 0,
                            amplitude: *amplitude,
                            phase_deg: *phase_deg,
                            is_voltage: false,
                        });
                    }
                }
                Component::VoltageSource { name, n_pos, n_neg, amplitude, phase_deg, .. } => {
                    if let Some(vs_idx) = vsources.iter().position(|(n, p, neg)| n == name && *p == *n_pos && *neg == *n_neg) {
                        let m_idx = num_nodes + vs_idx;
                        
                        if *n_pos > 0 {
                            static_entries.push(StaticEntry { row: *n_pos - 1, col: m_idx, value: Cx::new(1.0, 0.0) });
                            static_entries.push(StaticEntry { row: m_idx, col: *n_pos - 1, value: Cx::new(1.0, 0.0) });
                        }
                        if *n_neg > 0 {
                            static_entries.push(StaticEntry { row: *n_neg - 1, col: m_idx, value: Cx::new(-1.0, 0.0) });
                            static_entries.push(StaticEntry { row: m_idx, col: *n_neg - 1, value: Cx::new(-1.0, 0.0) });
                        }
                        
                        source_entries.push(SourceEntry {
                            row: m_idx,
                            col: 0,
                            amplitude: *amplitude,
                            phase_deg: *phase_deg,
                            is_voltage: true,
                        });
                    }
                }
                Component::VCCS { n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, transconductance, .. } => {
                    let gm = Cx::new(*transconductance, 0.0);
                    if *n_out_pos > 0 && *n_ctrl_pos > 0 {
                        static_entries.push(StaticEntry { row: *n_out_pos - 1, col: *n_ctrl_pos - 1, value: gm });
                    }
                    if *n_out_pos > 0 && *n_ctrl_neg > 0 {
                        static_entries.push(StaticEntry { row: *n_out_pos - 1, col: *n_ctrl_neg - 1, value: Cx::new(-gm.re, -gm.im) });
                    }
                    if *n_out_neg > 0 && *n_ctrl_pos > 0 {
                        static_entries.push(StaticEntry { row: *n_out_neg - 1, col: *n_ctrl_pos - 1, value: Cx::new(-gm.re, -gm.im) });
                    }
                    if *n_out_neg > 0 && *n_ctrl_neg > 0 {
                        static_entries.push(StaticEntry { row: *n_out_neg - 1, col: *n_ctrl_neg - 1, value: gm });
                    }
                }
                Component::VCVS { name, n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, gain, .. } => {
                    if let Some(vs_idx) = vsources.iter().position(|(n, _, _)| n == name) {
                        let m_idx = num_nodes + vs_idx;
                        let a = Cx::new(*gain, 0.0);
                        
                        if *n_out_pos > 0 {
                            static_entries.push(StaticEntry { row: m_idx, col: *n_out_pos - 1, value: Cx::new(1.0, 0.0) });
                            static_entries.push(StaticEntry { row: *n_out_pos - 1, col: m_idx, value: Cx::new(1.0, 0.0) });
                        }
                        if *n_out_neg > 0 {
                            static_entries.push(StaticEntry { row: m_idx, col: *n_out_neg - 1, value: Cx::new(-1.0, 0.0) });
                            static_entries.push(StaticEntry { row: *n_out_neg - 1, col: m_idx, value: Cx::new(-1.0, 0.0) });
                        }
                        if *n_ctrl_pos > 0 {
                            static_entries.push(StaticEntry { row: m_idx, col: *n_ctrl_pos - 1, value: Cx::new(-a.re, -a.im) });
                        }
                        if *n_ctrl_neg > 0 {
                            static_entries.push(StaticEntry { row: m_idx, col: *n_ctrl_neg - 1, value: a });
                        }
                        
                        // Current variable for VCVS
                        if *n_out_pos > 0 {
                            static_entries.push(StaticEntry { row: *n_out_pos - 1, col: m_idx, value: Cx::new(1.0, 0.0) });
                        }
                        if *n_out_neg > 0 {
                            static_entries.push(StaticEntry { row: m_idx, col: *n_out_neg - 1, value: Cx::new(-1.0, 0.0) });
                        }
                    }
                }
                Component::CCCS { n_out_pos, n_out_neg, ctrl_vsrc_name, gain, .. } => {
                    if let Some(vs_idx) = vsources.iter().position(|(n, _, _)| n == ctrl_vsrc_name) {
                        let m_idx = num_nodes + vs_idx;
                        let a = Cx::new(*gain, 0.0);
                        
                        if *n_out_pos > 0 {
                            static_entries.push(StaticEntry { row: *n_out_pos - 1, col: m_idx, value: a });
                        }
                        if *n_out_neg > 0 {
                            static_entries.push(StaticEntry { row: *n_out_neg - 1, col: m_idx, value: Cx::new(-a.re, -a.im) });
                        }
                    }
                }
                Component::CCVS { n_out_pos, n_out_neg, ctrl_vsrc_name, transresistance, .. } => {
                    if let Some(vs_idx) = vsources.iter().position(|(n, _, _)| n == ctrl_vsrc_name) {
                        let m_idx = num_nodes + vs_idx;
                        let r = Cx::new(*transresistance, 0.0);
                        
                        if *n_out_pos > 0 {
                            static_entries.push(StaticEntry { row: *n_out_pos - 1, col: m_idx, value: Cx::new(1.0, 0.0) });
                        }
                        if *n_out_neg > 0 {
                            static_entries.push(StaticEntry { row: *n_out_neg - 1, col: m_idx, value: Cx::new(-1.0, 0.0) });
                        }
                        static_entries.push(StaticEntry { row: m_idx, col: m_idx, value: r });
                    }
                }
                Component::OpAmp { n_out, n_in_pos, n_in_neg, gain, .. } => {
                    let a = Cx::new(*gain, 0.0);
                    if *n_out > 0 && *n_in_pos > 0 {
                        static_entries.push(StaticEntry { row: *n_out - 1, col: *n_in_pos - 1, value: a });
                    }
                    if *n_out > 0 && *n_in_neg > 0 {
                        static_entries.push(StaticEntry { row: *n_out - 1, col: *n_in_neg - 1, value: Cx::new(-a.re, -a.im) });
                    }
                }
                Component::Diode { n_anode, n_cathode, .. } => {
                    let gmin = Cx::new(GMIN, 0.0);
                    if *n_anode > 0 && *n_cathode > 0 {
                        static_entries.push(StaticEntry { row: *n_anode - 1, col: *n_anode - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_cathode - 1, col: *n_cathode - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_anode - 1, col: *n_cathode - 1, value: Cx::new(-gmin.re, -gmin.im) });
                        static_entries.push(StaticEntry { row: *n_cathode - 1, col: *n_anode - 1, value: Cx::new(-gmin.re, -gmin.im) });
                    }
                }
                Component::BJT { n_collector, n_base, n_emitter, .. } => {
                    let gmin = Cx::new(GMIN, 0.0);
                    if *n_collector > 0 && *n_emitter > 0 {
                        static_entries.push(StaticEntry { row: *n_collector - 1, col: *n_collector - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_emitter - 1, col: *n_emitter - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_collector - 1, col: *n_emitter - 1, value: Cx::new(-gmin.re, -gmin.im) });
                        static_entries.push(StaticEntry { row: *n_emitter - 1, col: *n_collector - 1, value: Cx::new(-gmin.re, -gmin.im) });
                    }
                    if *n_base > 0 && *n_emitter > 0 {
                        static_entries.push(StaticEntry { row: *n_base - 1, col: *n_base - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_emitter - 1, col: *n_emitter - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_base - 1, col: *n_emitter - 1, value: Cx::new(-gmin.re, -gmin.im) });
                        static_entries.push(StaticEntry { row: *n_emitter - 1, col: *n_base - 1, value: Cx::new(-gmin.re, -gmin.im) });
                    }
                }
                Component::MOSFET { n_drain, n_gate, n_source, n_body, .. } => {
                    let gmin = Cx::new(GMIN, 0.0);
                    if *n_drain > 0 && *n_source > 0 {
                        static_entries.push(StaticEntry { row: *n_drain - 1, col: *n_drain - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_source - 1, col: *n_source - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_drain - 1, col: *n_source - 1, value: Cx::new(-gmin.re, -gmin.im) });
                        static_entries.push(StaticEntry { row: *n_source - 1, col: *n_drain - 1, value: Cx::new(-gmin.re, -gmin.im) });
                    }
                    if *n_gate > 0 && *n_source > 0 {
                        static_entries.push(StaticEntry { row: *n_gate - 1, col: *n_gate - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_source - 1, col: *n_source - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_gate - 1, col: *n_source - 1, value: Cx::new(-gmin.re, -gmin.im) });
                        static_entries.push(StaticEntry { row: *n_source - 1, col: *n_gate - 1, value: Cx::new(-gmin.re, -gmin.im) });
                    }
                    if *n_body > 0 && *n_source > 0 {
                        static_entries.push(StaticEntry { row: *n_body - 1, col: *n_body - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_source - 1, col: *n_source - 1, value: gmin });
                        static_entries.push(StaticEntry { row: *n_body - 1, col: *n_source - 1, value: Cx::new(-gmin.re, -gmin.im) });
                        static_entries.push(StaticEntry { row: *n_source - 1, col: *n_body - 1, value: Cx::new(-gmin.re, -gmin.im) });
                    }
                }
                _ => {}
            }
        }
        
        Self {
            y_matrix,
            i_vector,
            vsources,
            matrix_size,
            num_nodes,
            static_entries,
            freq_dependent_entries,
            source_entries,
        }
    }
    
    pub fn update_for_frequency(&mut self, omega: f64) {
        // Reset matrix and vector
        for row in &mut self.y_matrix {
            for val in row {
                val.re = 0.0;
                val.im = 0.0;
            }
        }
        for val in &mut self.i_vector {
            val.re = 0.0;
            val.im = 0.0;
        }
        
        // Apply static entries (GMIN, resistors, controlled sources, GMIN from diodes/BJTs/MOSFETs)
        for entry in &self.static_entries {
            self.y_matrix[entry.row][entry.col] += entry.value;
        }
        
        // Apply frequency-dependent entries
        for entry in &self.freq_dependent_entries {
            let y = match entry.component_type {
                FreqDepType::Resistor => Cx::new(1.0 / entry.value, 0.0),
                FreqDepType::Capacitor => Cx::new(0.0, omega * entry.value),
                FreqDepType::Inductor => Cx::new(0.0, -1.0 / (omega * entry.value)),
            };
            self.y_matrix[entry.row][entry.col] += y;
        }
        
        // Apply source entries
        for entry in &self.source_entries {
            let val = Cx::from_polar(entry.amplitude, entry.phase_deg.to_radians());
            if entry.is_voltage {
                // Voltage source: goes into i_vector at the expanded row
                self.i_vector[entry.row] = val;
            } else {
                // Current source: goes into i_vector at the node row
                self.i_vector[entry.row] += val;
            }
        }
    }
    
    pub fn solve(&mut self) -> Option<Vec<Cx>> {
        crate::solver::solve_complex_matrix(&mut self.y_matrix, &mut self.i_vector)
    }
}

// Keep the original MnaSystem for backward compatibility
pub struct MnaSystem {
    pub y_matrix: Vec<Vec<Cx>>,
    pub i_vector: Vec<Cx>,
    pub vsources: Vec<(String, usize, usize)>,
    pub matrix_size: usize,
    pub num_nodes: usize,
}

impl MnaSystem {
    pub fn build(netlist: &Netlist, omega: f64) -> Self {
        let mut matrix = MnaMatrix::build_from_netlist(netlist);
        matrix.update_for_frequency(omega);
        
        Self {
            y_matrix: matrix.y_matrix,
            i_vector: matrix.i_vector,
            vsources: matrix.vsources,
            matrix_size: matrix.matrix_size,
            num_nodes: matrix.num_nodes,
        }
    }
}

fn add_admittance(y_matrix: &mut [Vec<Cx>], n1: usize, n2: usize, y: Cx) {
    if n1 > 0 && n2 > 0 {
        y_matrix[n1 - 1][n1 - 1] += y; y_matrix[n2 - 1][n2 - 1] += y;
        y_matrix[n1 - 1][n2 - 1] -= y; y_matrix[n2 - 1][n1 - 1] -= y;
    } else if n1 > 0 { y_matrix[n1 - 1][n1 - 1] += y;
    } else if n2 > 0 { y_matrix[n2 - 1][n2 - 1] += y; }
}