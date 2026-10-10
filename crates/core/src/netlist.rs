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
