use crate::synthesis::{CrossoverSynthesizer, StandardSeries};
use crate::types::{CrossoverDesign, SimulationConfig, TargetResponse};
use crate::CrossoverSimulator;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub max_iterations: usize,
    pub population_size: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub tolerance: f64,
    pub standardize_values: bool,
    pub series: StandardSeries,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            population_size: 50,
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            tolerance: 1e-4,
            standardize_values: true,
            series: StandardSeries::E12,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub best_design: CrossoverDesign,
    pub best_error: f64,
    pub error_history: Vec<f64>,
    pub converged: bool,
}

pub struct Optimizer {
    config: OptimizationConfig,
    target: TargetResponse,
    sim_config: SimulationConfig,
    drivers: crate::driver::DriverDatabase,
}

impl Optimizer {
    pub fn new(
        config: OptimizationConfig,
        target: TargetResponse,
        sim_config: SimulationConfig,
        drivers: crate::driver::DriverDatabase,
    ) -> Self {
        Self {
            config,
            target,
            sim_config,
            drivers,
        }
    }

    pub fn optimize(&self, initial_design: CrossoverDesign) -> OptimizationResult {
        let mut population = self.initialize_population(&initial_design);
        let mut best_design = initial_design.clone();
        let mut best_error = f64::INFINITY;
        let mut error_history = Vec::new();

        for _iteration in 0..self.config.max_iterations {
            let mut errors = Vec::new();

            for design in &population {
                let error = self.evaluate_design(design);
                errors.push((design.clone(), error));
            }

            errors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            if errors[0].1 < best_error {
                best_error = errors[0].1;
                best_design = errors[0].0.clone();
            }

            error_history.push(best_error);

            if best_error < self.config.tolerance {
                return OptimizationResult {
                    best_design,
                    best_error,
                    error_history,
                    converged: true,
                };
            }

            let elite_count = (self.config.population_size as f64 * 0.1).ceil() as usize;
            let mut new_population: Vec<CrossoverDesign> = errors.iter().take(elite_count).map(|e| e.0.clone()).collect();

            while new_population.len() < self.config.population_size {
                let parent1 = self.select_parent(&errors);
                let parent2 = self.select_parent(&errors);

                if rand::thread_rng().gen::<f64>() < self.config.crossover_rate {
                    let child = self.crossover(&parent1, &parent2);
                    new_population.push(self.mutate(child));
                } else {
                    new_population.push(self.mutate(parent1));
                }
            }

            population = new_population;
        }

        OptimizationResult {
            best_design,
            best_error,
            error_history,
            converged: false,
        }
    }

    fn initialize_population(&self, base_design: &CrossoverDesign) -> Vec<CrossoverDesign> {
        let mut population = Vec::with_capacity(self.config.population_size);
        population.push(base_design.clone());

        for _ in 1..self.config.population_size {
            population.push(self.mutate(base_design.clone()));
        }

        population
    }

    fn evaluate_design(&self, design: &CrossoverDesign) -> f64 {
        let simulator = CrossoverSimulator::new(design.clone(), self.drivers.clone(), self.sim_config.clone());
        let result = match simulator.simulate() {
            Ok(r) => r,
            Err(_) => return f64::INFINITY,
        };

        let mut error = 0.0;
        let mut count = 0;

        for (i, &freq) in result.frequencies.iter().enumerate() {
            let target_mag = self.target.interpolate(freq);
            let actual_mag = result.summed_acoustic[i].norm();

            if target_mag > 0.0 && actual_mag > 0.0 {
                let err_db = 20.0 * (actual_mag / target_mag).log10().abs();
                error += err_db * err_db;
                count += 1;
            }
        }

        if count > 0 {
            (error / count as f64).sqrt()
        } else {
            f64::INFINITY
        }
    }

    fn select_parent(&self, errors: &[(CrossoverDesign, f64)]) -> CrossoverDesign {
        let tournament_size = 3;
        let mut best = &errors[0].0;
        let mut best_err = errors[0].1;

        for _ in 1..tournament_size {
            let idx = rand::thread_rng().gen_range(0..errors.len());
            if errors[idx].1 < best_err {
                best = &errors[idx].0;
                best_err = errors[idx].1;
            }
        }

        best.clone()
    }

    fn crossover(&self, parent1: &CrossoverDesign, parent2: &CrossoverDesign) -> CrossoverDesign {
        let mut child = parent1.clone();

        for (i, filter) in child.filters.iter_mut().enumerate() {
            if i < parent2.filters.len() && rand::thread_rng().gen::<f64>() < 0.5 {
                filter.components = parent2.filters[i].components.clone();
            }
        }

        for (i, comp_net) in child.compensation.iter_mut().enumerate() {
            if i < parent2.compensation.len() && rand::thread_rng().gen::<f64>() < 0.5 {
                comp_net.components = parent2.compensation[i].components.clone();
            }
        }

        child
    }

    fn mutate(&self, mut design: CrossoverDesign) -> CrossoverDesign {
        let mut rng = rand::thread_rng();

        for filter in &mut design.filters {
            for comp in &mut filter.components {
                if rng.gen::<f64>() < self.config.mutation_rate {
                    let factor = 10.0_f64.powf(rng.gen_range(-0.3..0.3));
                    comp.value *= factor;

                    if self.config.standardize_values {
                        comp.value = match self.config.series {
                            StandardSeries::E12 => CrossoverSynthesizer::round_to_e12(comp.value),
                            StandardSeries::E24 => CrossoverSynthesizer::round_to_e24(comp.value),
                        };
                    }
                }
            }
        }

        for comp_net in &mut design.compensation {
            for comp in &mut comp_net.components {
                if rng.gen::<f64>() < self.config.mutation_rate {
                    let factor = 10.0_f64.powf(rng.gen_range(-0.3..0.3));
                    comp.value *= factor;

                    if self.config.standardize_values {
                        comp.value = match self.config.series {
                            StandardSeries::E12 => CrossoverSynthesizer::round_to_e12(comp.value),
                            StandardSeries::E24 => CrossoverSynthesizer::round_to_e24(comp.value),
                        };
                    }
                }
            }
        }

        design
    }
}