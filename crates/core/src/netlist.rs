use crate::component::Component;

#[derive(Debug, Clone)]
pub struct Netlist {
    pub components: Vec<Component>,
    pub max_node: usize,
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
        });
        self.components.push(component);
    }
}
