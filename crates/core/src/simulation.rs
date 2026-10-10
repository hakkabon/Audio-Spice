use crate::types::Cx;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AcSweep {
    pub f_start: f64,
    pub f_stop: f64,
    pub points_per_decade: usize,
}

impl AcSweep {
    pub fn frequencies(&self) -> Vec<f64> {
        let decades = (self.f_stop / self.f_start).log10();
        let total_points = (decades * self.points_per_decade as f64).ceil() as usize + 1;
        let mut freqs = Vec::with_capacity(total_points);
        for i in 0..total_points {
            let f = self.f_start * 10.0_f64.powf(i as f64 / self.points_per_decade as f64);
            if f <= self.f_stop {
                freqs.push(f);
            }
        }
        freqs
    }
}

#[derive(Debug, Clone)]
pub struct AcResult {
    pub frequencies: Vec<f64>,
    pub node_voltages: HashMap<usize, Vec<Cx>>,
    pub vsource_currents: HashMap<String, Vec<Cx>>,
}