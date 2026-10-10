use crate::types::Cx;

#[derive(Debug, Clone)]
pub enum Component {
    Resistor { name: String, n_pos: usize, n_neg: usize, value: f64 },
    Capacitor { name: String, n_pos: usize, n_neg: usize, value: f64 },
    Inductor { name: String, n_pos: usize, n_neg: usize, value: f64 },
    VoltageSource { name: String, n_pos: usize, n_neg: usize, amplitude: f64, phase_deg: f64 },
    CurrentSource { name: String, n_pos: usize, n_neg: usize, amplitude: f64, phase_deg: f64 },
    VCVS { name: String, n_out_pos: usize, n_out_neg: usize, n_ctrl_pos: usize, n_ctrl_neg: usize, gain: f64 },
    VCCS { name: String, n_out_pos: usize, n_out_neg: usize, n_ctrl_pos: usize, n_ctrl_neg: usize, transconductance: f64 },
    CCCS { name: String, n_out_pos: usize, n_out_neg: usize, ctrl_vsrc_name: String, gain: f64 },
    CCVS { name: String, n_out_pos: usize, n_out_neg: usize, ctrl_vsrc_name: String, transresistance: f64 },
    OpAmp { name: String, n_out: usize, n_in_pos: usize, n_in_neg: usize, n_vcc: usize, n_vee: usize, gain: f64, gbw: f64, slew_rate: f64 },
    Diode { name: String, n_anode: usize, n_cathode: usize, model: String, area: f64 },
    BJT { name: String, n_collector: usize, n_base: usize, n_emitter: usize, model: String, area: f64 },
    MOSFET { name: String, n_drain: usize, n_gate: usize, n_source: usize, n_body: usize, model: String, w: f64, l: f64 },
}

impl Component {
    pub fn get_name(&self) -> Option<&str> {
        match self {
            Component::Resistor { name, .. } => Some(name),
            Component::Capacitor { name, .. } => Some(name),
            Component::Inductor { name, .. } => Some(name),
            Component::VoltageSource { name, .. } => Some(name),
            Component::CurrentSource { name, .. } => Some(name),
            Component::VCVS { name, .. } => Some(name),
            Component::VCCS { name, .. } => Some(name),
            Component::CCCS { name, .. } => Some(name),
            Component::CCVS { name, .. } => Some(name),
            Component::OpAmp { name, .. } => Some(name),
            Component::Diode { name, .. } => Some(name),
            Component::BJT { name, .. } => Some(name),
            Component::MOSFET { name, .. } => Some(name),
        }
    }

    pub fn is_voltage_source(&self) -> bool {
        matches!(self, Component::VoltageSource { .. } | Component::CCVS { .. })
    }

    pub fn is_current_source(&self) -> bool {
        matches!(self, Component::CurrentSource { .. } | Component::VCCS { .. } | Component::CCCS { .. })
    }

    pub fn admittance(&self, omega: f64) -> Option<Cx> {
        match self {
            Component::Resistor { value, .. } => Some(Cx::new(1.0 / value, 0.0)),
            Component::Capacitor { value, .. } => Some(Cx::new(0.0, omega * value)),
            Component::Inductor { value, .. } => Some(Cx::new(0.0, -1.0 / (omega * value))),
            _ => None,
        }
    }
}
