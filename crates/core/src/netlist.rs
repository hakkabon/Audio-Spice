use crate::component::Component;

#[derive(Debug, Clone)]
pub struct Netlist {
    pub components: Vec<Component>,
    pub max_node: usize,
}

impl Default for Netlist {
    fn default() -> Self {
        Self::new()
    }
}

impl Netlist {
    pub fn new() -> Self {
        Self { components: Vec::new(), max_node: 0 }
    }

    pub fn add_component(&mut self, component: Component) {
        self.max_node = self.max_node.max(match &component {
            Component::Resistor { n_pos, n_neg, .. } => *n_pos.max(n_neg),
            Component::Capacitor { n_pos, n_neg, .. } => *n_pos.max(n_neg),
            Component::Inductor { n_pos, n_neg, .. } => *n_pos.max(n_neg),
            Component::VoltageSource { n_pos, n_neg, .. } => *n_pos.max(n_neg),
            Component::CurrentSource { n_pos, n_neg, .. } => *n_pos.max(n_neg),
            Component::VCVS { n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, .. } => 
                *n_out_pos.max(n_out_neg).max(n_ctrl_pos).max(n_ctrl_neg),
            Component::VCCS { n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, .. } => 
                *n_out_pos.max(n_out_neg).max(n_ctrl_pos).max(n_ctrl_neg),
            Component::CCCS { n_out_pos, n_out_neg, .. } => *n_out_pos.max(n_out_neg),
            Component::CCVS { n_out_pos, n_out_neg, .. } => *n_out_pos.max(n_out_neg),
            Component::OpAmp { n_out, n_in_pos, n_in_neg, n_vcc, n_vee, .. } => 
                *n_out.max(n_in_pos).max(n_in_neg).max(n_vcc).max(n_vee),
            Component::Diode { n_anode, n_cathode, .. } => *n_anode.max(n_cathode),
            Component::BJT { n_collector, n_base, n_emitter, .. } => 
                *n_collector.max(n_base).max(n_emitter),
            Component::MOSFET { n_drain, n_gate, n_source, n_body, .. } => 
                *n_drain.max(n_gate).max(n_source).max(n_body),
        });
        self.components.push(component);
    }
}
