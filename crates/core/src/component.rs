use crate::types::Cx;

#[derive(Debug, Clone)]
pub enum Component {
    Resistor { name: String, n_pos: usize, n_neg: usize, value: f64 },
    Capacitor { name: String, n_pos: usize, n_neg: usize, value: f64 },
    Inductor { name: String, n_pos: usize, n_neg: usize, value: f64 },
    VoltageSource { name: String, n_pos: usize, n_neg: usize, amplitude: f64, phase_deg: f64 },
    CurrentSource { name: String, n_pos: usize, n_neg: usize, amplitude: f64, phase_deg: f64 },
}

impl Component {
    pub fn get_name(&self) -> Option<&str> {
        match self {
            Component::Resistor { name, .. } => Some(name),
            Component::Capacitor { name, .. } => Some(name),
            Component::Inductor { name, .. } => Some(name),
            Component::VoltageSource { name, .. } => Some(name),
            Component::CurrentSource { name, .. } => Some(name),
        }
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
