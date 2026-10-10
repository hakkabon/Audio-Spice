pub mod driver;
pub mod netlist_builder;
pub mod optimizer;
pub mod synthesis;
pub mod types;

#[cfg(test)]
mod synthesis_tests;
#[cfg(test)]
mod crossover_tests;

pub use driver::{Driver, DriverDatabase, DriverRole, Polarity};
pub use netlist_builder::{CrossoverNetlistBuilder, CrossoverSimulator, NetlistBuilderConfig};
pub use optimizer::{OptimizationConfig, OptimizationResult, Optimizer};
pub use synthesis::{CrossoverSynthesizer, StandardSeries};
pub use types::{
    ComponentType, ComponentValue, CompensationNetwork, CompensationType,
    CrossoverDesign, CrossoverType, FilterSpec, FilterType, FrequencyResponse, ImpedanceData,
    SimulationConfig, SimulationResult, TargetResponse,
};