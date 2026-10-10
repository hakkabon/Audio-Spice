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

#[derive(Debug, Clone)]
pub struct DcResult {
    pub node_voltages: HashMap<usize, f64>,
    pub vsource_currents: HashMap<String, f64>,
    pub converged: bool,
    pub iterations: usize,
}

#[derive(Debug, Clone)]
pub struct TranSweep {
    pub t_start: f64,
    pub t_stop: f64,
    pub t_step: f64,
    pub t_max: f64,
}

impl TranSweep {
    pub fn time_points(&self) -> Vec<f64> {
        let mut points = Vec::new();
        let num_steps = ((self.t_stop - self.t_start) / self.t_step).ceil() as usize;
        for i in 0..=num_steps {
            let t = self.t_start + i as f64 * self.t_step;
            if t <= self.t_stop + 1e-12 {
                points.push(t);
            }
        }
        points
    }
}

#[derive(Debug, Clone)]
pub struct TranResult {
    pub time_points: Vec<f64>,
    pub node_voltages: HashMap<usize, Vec<f64>>,
    pub vsource_currents: HashMap<String, Vec<f64>>,
}