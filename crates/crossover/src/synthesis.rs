use crate::types::{ComponentType, ComponentValue, CrossoverType, FilterType};
use std::f64::consts::PI;

pub struct CrossoverSynthesizer;

impl CrossoverSynthesizer {
    pub fn synthesize_filter(
        filter_type: FilterType,
        crossover_type: CrossoverType,
        fc: f64,
        r_load: f64,
        prefix: &str,
    ) -> Vec<ComponentValue> {
        let omega = 2.0 * PI * fc;
        let mut components = Vec::new();

        match (filter_type, crossover_type) {
            (FilterType::LowPass, CrossoverType::Butterworth1) => {
                let l1 = r_load / omega;
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::HighPass, CrossoverType::Butterworth1) => {
                let c1 = 1.0 / (omega * r_load);
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::LowPass, CrossoverType::Butterworth2) => {
                let l1 = r_load * 1.414 / omega;
                let c1 = 1.414 / (omega * r_load);
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::HighPass, CrossoverType::Butterworth2) => {
                let c1 = 1.414 / (omega * r_load);
                let l1 = r_load * 1.414 / omega;
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::LowPass, CrossoverType::Butterworth3) => {
                let l1 = r_load * 1.0 / omega;
                let c1 = 2.0 / (omega * r_load);
                let l2 = r_load * 1.0 / omega;
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L2", prefix),
                    component_type: ComponentType::Inductor,
                    value: l2,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::HighPass, CrossoverType::Butterworth3) => {
                let c1 = 1.0 / (omega * r_load);
                let l1 = 2.0 * r_load / omega;
                let c2 = 1.0 / (omega * r_load);
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C2", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c2,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::LowPass, CrossoverType::Butterworth4) => {
                let l1 = r_load * 0.765 / omega;
                let c1 = 1.848 / (omega * r_load);
                let l2 = r_load * 1.848 / omega;
                let c2 = 0.765 / (omega * r_load);
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L2", prefix),
                    component_type: ComponentType::Inductor,
                    value: l2,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C2", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c2,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::HighPass, CrossoverType::Butterworth4) => {
                let c1 = 0.765 / (omega * r_load);
                let l1 = r_load * 1.848 / omega;
                let c2 = 1.848 / (omega * r_load);
                let l2 = r_load * 0.765 / omega;
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C2", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c2,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L2", prefix),
                    component_type: ComponentType::Inductor,
                    value: l2,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::LowPass, CrossoverType::LinkwitzRiley2) => {
                let l1 = r_load * 1.414 / omega;
                let c1 = 1.414 / (omega * r_load);
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::HighPass, CrossoverType::LinkwitzRiley2) => {
                let c1 = 1.414 / (omega * r_load);
                let l1 = r_load * 1.414 / omega;
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::LowPass, CrossoverType::LinkwitzRiley4) => {
                let c1 = 0.841 / (omega * r_load);
                let l1 = r_load * 1.414 / omega;
                let l2 = r_load * 0.414 / omega;
                let c2 = 1.414 / (omega * r_load);
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L2", prefix),
                    component_type: ComponentType::Inductor,
                    value: l2,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C2", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c2,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::HighPass, CrossoverType::LinkwitzRiley4) => {
                let l1 = r_load * 0.841 / omega;
                let c1 = 1.414 / (omega * r_load);
                let c2 = 0.414 / (omega * r_load);
                let l2 = r_load * 1.414 / omega;
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C2", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c2,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L2", prefix),
                    component_type: ComponentType::Inductor,
                    value: l2,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::LowPass, CrossoverType::LinkwitzRiley6) => {
                let l1 = r_load * 0.518 / omega;
                let c1 = 2.0 / (omega * r_load);
                let l2 = r_load * 2.414 / omega;
                let c2 = 1.0 / (omega * r_load);
                let l3 = r_load * 0.518 / omega;
                let c3 = 2.0 / (omega * r_load);
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L2", prefix),
                    component_type: ComponentType::Inductor,
                    value: l2,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C2", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c2,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L3", prefix),
                    component_type: ComponentType::Inductor,
                    value: l3,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C3", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c3,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::HighPass, CrossoverType::LinkwitzRiley6) => {
                let c1 = 0.518 / (omega * r_load);
                let l1 = r_load * 2.0 / omega;
                let c2 = r_load * 2.414 / omega;
                let l2 = r_load * 1.0 / omega;
                let c3 = 0.518 / (omega * r_load);
                let l3 = r_load * 2.0 / omega;
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C2", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c2,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L2", prefix),
                    component_type: ComponentType::Inductor,
                    value: l2,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C3", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c3,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L3", prefix),
                    component_type: ComponentType::Inductor,
                    value: l3,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::LowPass, CrossoverType::Bessel2) => {
                let l1 = r_load * 1.272 / omega;
                let c1 = 0.786 / (omega * r_load);
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::HighPass, CrossoverType::Bessel2) => {
                let c1 = 1.272 / (omega * r_load);
                let l1 = r_load * 0.786 / omega;
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::LowPass, CrossoverType::Bessel3) => {
                let l1 = r_load * 1.0 / omega;
                let c1 = 1.272 / (omega * r_load);
                let l2 = r_load * 1.272 / omega;
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L2", prefix),
                    component_type: ComponentType::Inductor,
                    value: l2,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::HighPass, CrossoverType::Bessel3) => {
                let c1 = 1.0 / (omega * r_load);
                let l1 = r_load * 1.272 / omega;
                let c2 = 1.272 / (omega * r_load);
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C2", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c2,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::LowPass, CrossoverType::Bessel4) => {
                let l1 = r_load * 0.564 / omega;
                let c1 = 1.596 / (omega * r_load);
                let l2 = r_load * 2.203 / omega;
                let c2 = 0.454 / (omega * r_load);
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L2", prefix),
                    component_type: ComponentType::Inductor,
                    value: l2,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C2", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c2,
                    tolerance: Some(0.1),
                });
            }
            (FilterType::HighPass, CrossoverType::Bessel4) => {
                let c1 = 0.564 / (omega * r_load);
                let l1 = r_load * 1.596 / omega;
                let c2 = r_load * 2.203 / omega;
                let l2 = r_load * 0.454 / omega;
                components.push(ComponentValue {
                    name: format!("{}_C1", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L1", prefix),
                    component_type: ComponentType::Inductor,
                    value: l1,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_C2", prefix),
                    component_type: ComponentType::Capacitor,
                    value: c2,
                    tolerance: Some(0.1),
                });
                components.push(ComponentValue {
                    name: format!("{}_L2", prefix),
                    component_type: ComponentType::Inductor,
                    value: l2,
                    tolerance: Some(0.1),
                });
            }
            _ => {}
        }

        components
    }

    pub fn synthesize_zobel(re: f64, le: f64, prefix: &str) -> Vec<ComponentValue> {
        let r = re * 1.25;
        let c = le / (re * re);
        vec![
            ComponentValue {
                name: format!("{}_R", prefix),
                component_type: ComponentType::Resistor,
                value: r,
                tolerance: Some(0.05),
            },
            ComponentValue {
                name: format!("{}_C", prefix),
                component_type: ComponentType::Capacitor,
                value: c,
                tolerance: Some(0.1),
            },
        ]
    }

    pub fn synthesize_lpad(r_load: f64, attenuation_db: f64, prefix: &str) -> Vec<ComponentValue> {
        let k = 10.0_f64.powf(attenuation_db / 20.0);
        let r_series = r_load * (k - 1.0) / (k + 1.0);
        let r_parallel = r_load * (k * k - 1.0) / (2.0 * k);
        vec![
            ComponentValue {
                name: format!("{}_Rseries", prefix),
                component_type: ComponentType::Resistor,
                value: r_series,
                tolerance: Some(0.05),
            },
            ComponentValue {
                name: format!("{}_Rparallel", prefix),
                component_type: ComponentType::Resistor,
                value: r_parallel,
                tolerance: Some(0.05),
            },
        ]
    }

    pub fn synthesize_notch(fc: f64, q: f64, r_load: f64, prefix: &str) -> Vec<ComponentValue> {
        let omega = 2.0 * PI * fc;
        let c = 1.0 / (omega * r_load * q);
        let l = r_load * q / omega;
        let r = r_load;
        vec![
            ComponentValue {
                name: format!("{}_C", prefix),
                component_type: ComponentType::Capacitor,
                value: c,
                tolerance: Some(0.1),
            },
            ComponentValue {
                name: format!("{}_L", prefix),
                component_type: ComponentType::Inductor,
                value: l,
                tolerance: Some(0.1),
            },
            ComponentValue {
                name: format!("{}_R", prefix),
                component_type: ComponentType::Resistor,
                value: r,
                tolerance: Some(0.05),
            },
        ]
    }

    pub fn synthesize_baffle_step(fc: f64, r_load: f64, step_db: f64, prefix: &str) -> Vec<ComponentValue> {
        let k = 10.0_f64.powf(step_db / 20.0);
        let r = r_load * (k - 1.0);
        let l = r / (2.0 * PI * fc);
        vec![
            ComponentValue {
                name: format!("{}_R", prefix),
                component_type: ComponentType::Resistor,
                value: r,
                tolerance: Some(0.05),
            },
            ComponentValue {
                name: format!("{}_L", prefix),
                component_type: ComponentType::Inductor,
                value: l,
                tolerance: Some(0.1),
            },
        ]
    }

    pub fn round_to_e12(value: f64) -> f64 {
        let e12 = [
            1.0, 1.2, 1.5, 1.8, 2.2, 2.7, 3.3, 3.9, 4.7, 5.6, 6.8, 8.2,
        ];
        let decade = value.log10().floor() as i32;
        let normalized = value / 10.0_f64.powi(decade);
        let closest = e12.iter().min_by(|a, b| {
            let diff_a = (*a - normalized).abs();
            let diff_b = (*b - normalized).abs();
            diff_a.partial_cmp(&diff_b).unwrap().then_with(|| a.partial_cmp(b).unwrap())
        }).unwrap();
        *closest * 10.0_f64.powi(decade)
    }

    pub fn round_to_e24(value: f64) -> f64 {
        let e24 = [
            1.0, 1.1, 1.2, 1.3, 1.5, 1.6, 1.8, 2.0, 2.2, 2.4, 2.7, 3.0,
            3.3, 3.6, 3.9, 4.3, 4.7, 5.1, 5.6, 6.2, 6.8, 7.5, 8.2, 9.1,
        ];
        let decade = value.log10().floor() as i32;
        let normalized = value / 10.0_f64.powi(decade);
        let closest = e24.iter().min_by(|a, b| {
            let diff_a = (*a - normalized).abs();
            let diff_b = (*b - normalized).abs();
            diff_a.partial_cmp(&diff_b).unwrap().then_with(|| a.partial_cmp(b).unwrap())
        }).unwrap();
        *closest * 10.0_f64.powi(decade)
    }

    pub fn standardize_components(components: &mut [ComponentValue], series: StandardSeries) {
        for comp in components {
            comp.value = match series {
                StandardSeries::E12 => Self::round_to_e12(comp.value),
                StandardSeries::E24 => Self::round_to_e24(comp.value),
            };
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardSeries {
    E12,
    E24,
}