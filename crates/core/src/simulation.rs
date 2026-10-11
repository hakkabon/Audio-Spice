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

// Noise Analysis Types
#[derive(Debug, Clone)]
pub struct NoiseSweep {
    pub f_start: f64,
    pub f_stop: f64,
    pub points_per_decade: usize,
    pub output_node: usize,
    pub reference_node: Option<usize>,
    pub input_source: Option<String>,
}

impl NoiseSweep {
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
pub struct NoiseResult {
    pub frequencies: Vec<f64>,
    pub output_noise: Vec<f64>,           // V/√Hz at output
    pub input_referred_noise: Vec<f64>,   // V/√Hz referred to input
    pub noise_contributions: HashMap<String, Vec<f64>>, // Per-component noise
    pub total_rms_noise: f64,             // Integrated RMS noise
}

#[derive(Debug, Clone, Copy)]
pub enum NoiseType {
    Thermal,      // Johnson-Nyquist noise: √(4*k*T*R*Δf)
    Shot,         // Shot noise: √(2*q*I*Δf)
    Flicker,      // 1/f noise: K*I^a/f^b
}

#[derive(Debug, Clone)]
pub struct NoiseModel {
    pub thermal_coeff: f64,     // 4*k*T (default: 1.656e-20 at 300K)
    pub shot_coeff: f64,        // 2*q (default: 3.204e-19)
    pub flicker_k: f64,         // Flicker coefficient
    pub flicker_af: f64,        // Current exponent
    pub flicker_bf: f64,        // Frequency exponent
}

impl Default for NoiseModel {
    fn default() -> Self {
        Self {
            thermal_coeff: 1.656e-20,  // 4 * k * T at 300K
            shot_coeff: 3.204e-19,     // 2 * q
            flicker_k: 1e-12,
            flicker_af: 1.0,
            flicker_bf: 1.0,
        }
    }
}

// Sensitivity Analysis Types
#[derive(Debug, Clone)]
pub struct SensitivitySweep {
    pub f_start: f64,
    pub f_stop: f64,
    pub points_per_decade: usize,
    pub output_node: usize,
    pub reference_node: Option<usize>,
    pub component_names: Vec<String>,  // Components to analyze
}

impl SensitivitySweep {
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
pub struct SensitivityResult {
    pub frequencies: Vec<f64>,
    pub sensitivities: HashMap<String, Vec<f64>>,  // dVout/dP * P/Vout per component
    pub worst_case: HashMap<String, f64>,          // Max sensitivity magnitude
    pub relative_sensitivities: HashMap<String, Vec<f64>>,  // Normalized sensitivities
}

#[derive(Debug, Clone)]
pub struct MonteCarloConfig {
    pub num_runs: usize,
    pub component_tolerances: HashMap<String, f64>,  // Component -> tolerance (e.g., 0.05 for 5%)
    pub distribution: DistributionType,
}

#[derive(Debug, Clone, Copy)]
pub enum DistributionType {
    Uniform,
    Gaussian,
    LogNormal,
}

#[derive(Debug, Clone)]
pub struct MonteCarloResult {
    pub output_mean: Vec<f64>,
    pub output_std: Vec<f64>,
    pub output_min: Vec<f64>,
    pub output_max: Vec<f64>,
    pub histogram_data: HashMap<usize, Vec<f64>>, // Histogram at each frequency
    pub yield_estimate: f64,                       // Fraction meeting spec
}