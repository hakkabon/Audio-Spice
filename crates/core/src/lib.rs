pub mod component;
pub mod engine;
pub mod error;
pub mod mna;
pub mod models;
pub mod netlist;
pub mod simulation;
pub mod solver;
pub mod spice_parser;
pub mod types;

#[cfg(test)]
mod component_tests;
#[cfg(test)]
mod mna_tests;
#[cfg(test)]
mod solver_tests;
#[cfg(test)]
mod simulation_tests;
#[cfg(test)]
mod spice_parser_tests;
#[cfg(test)]
mod model_tests;

pub use component::Component;
pub use engine::{simulate_ac, simulate_dc, simulate_tran};
pub use error::{SimError, SimResult};
pub use models::{DiodeModel, BjtModel, MosfetModel, ModelLibrary};
pub use netlist::Netlist;
pub use simulation::{AcResult, AcSweep, DcResult, TranResult, TranSweep};
pub use spice_parser::{parse_spice_netlist, parse_spice_file};
