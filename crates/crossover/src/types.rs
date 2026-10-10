use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cx {
    pub re: f64,
    pub im: f64,
}

impl Cx {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    pub fn norm(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    pub fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }
}

impl std::ops::Add for Cx {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }
}

impl std::ops::AddAssign for Cx {
    fn add_assign(&mut self, other: Self) {
        self.re += other.re;
        self.im += other.im;
    }
}

impl std::ops::Sub for Cx {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }
}

impl std::ops::Mul for Cx {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
}

impl std::ops::Mul<f64> for Cx {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self::new(self.re * scalar, self.im * scalar)
    }
}

impl std::ops::Div for Cx {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        let denom = other.re * other.re + other.im * other.im;
        Self::new(
            (self.re * other.re + self.im * other.im) / denom,
            (self.im * other.re - self.re * other.im) / denom,
        )
    }
}

impl std::ops::Div<f64> for Cx {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::new(self.re / scalar, self.im / scalar)
    }
}

impl From<num_complex::Complex<f64>> for Cx {
    fn from(c: num_complex::Complex<f64>) -> Self {
        Self::new(c.re, c.im)
    }
}

impl From<Cx> for num_complex::Complex<f64> {
    fn from(c: Cx) -> Self {
        num_complex::Complex::new(c.re, c.im)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyResponse {
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<f64>,
    pub phases: Vec<f64>,
}

impl FrequencyResponse {
    pub fn new(frequencies: Vec<f64>, magnitudes: Vec<f64>, phases: Vec<f64>) -> Self {
        assert_eq!(frequencies.len(), magnitudes.len());
        assert_eq!(frequencies.len(), phases.len());
        Self { frequencies, magnitudes, phases }
    }

    pub fn interpolate(&self, freq: f64) -> Cx {
        if self.frequencies.is_empty() {
            return Cx::new(1.0, 0.0);
        }
        if freq <= self.frequencies[0] {
            return Cx::from_polar(self.magnitudes[0], self.phases[0].to_radians());
        }
        if freq >= self.frequencies[self.frequencies.len() - 1] {
            let idx = self.frequencies.len() - 1;
            return Cx::from_polar(self.magnitudes[idx], self.phases[idx].to_radians());
        }

        let idx = self.frequencies.partition_point(|&f| f < freq);
        let f1 = self.frequencies[idx - 1];
        let f2 = self.frequencies[idx];
        let t = (freq - f1).ln() / (f2 - f1).ln();

        let mag1 = self.magnitudes[idx - 1].ln();
        let mag2 = self.magnitudes[idx].ln();
        let mag = (mag1 + t * (mag2 - mag1)).exp();

        let phase1 = self.phases[idx - 1];
        let phase2 = self.phases[idx];
        let phase = phase1 + t * (phase2 - phase1);

        Cx::from_polar(mag, phase.to_radians())
    }

    pub fn from_frd_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let mut frequencies = Vec::new();
        let mut magnitudes = Vec::new();
        let mut phases = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('*') || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let (Ok(f), Ok(mag), Ok(phase)) = (
                    parts[0].parse::<f64>(),
                    parts[1].parse::<f64>(),
                    parts[2].parse::<f64>(),
                ) {
                    frequencies.push(f);
                    magnitudes.push(mag);
                    phases.push(phase);
                }
            }
        }

        Ok(Self::new(frequencies, magnitudes, phases))
    }

    pub fn from_txt_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_frd_file(path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpedanceData {
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<f64>,
    pub phases: Vec<f64>,
}

impl ImpedanceData {
    pub fn new(frequencies: Vec<f64>, magnitudes: Vec<f64>, phases: Vec<f64>) -> Self {
        assert_eq!(frequencies.len(), magnitudes.len());
        assert_eq!(frequencies.len(), phases.len());
        Self { frequencies, magnitudes, phases }
    }

    pub fn interpolate(&self, freq: f64) -> Cx {
        if self.frequencies.is_empty() {
            return Cx::new(8.0, 0.0);
        }
        if freq <= self.frequencies[0] {
            return Cx::from_polar(self.magnitudes[0], self.phases[0].to_radians());
        }
        if freq >= self.frequencies[self.frequencies.len() - 1] {
            let idx = self.frequencies.len() - 1;
            return Cx::from_polar(self.magnitudes[idx], self.phases[idx].to_radians());
        }

        let idx = self.frequencies.partition_point(|&f| f < freq);
        let f1 = self.frequencies[idx - 1];
        let f2 = self.frequencies[idx];
        let t = (freq - f1).ln() / (f2 - f1).ln();

        let mag1 = self.magnitudes[idx - 1].ln();
        let mag2 = self.magnitudes[idx].ln();
        let mag = (mag1 + t * (mag2 - mag1)).exp();

        let phase1 = self.phases[idx - 1];
        let phase2 = self.phases[idx];
        let phase = phase1 + t * (phase2 - phase1);

        Cx::from_polar(mag, phase.to_radians())
    }

    pub fn from_zma_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let mut frequencies = Vec::new();
        let mut magnitudes = Vec::new();
        let mut phases = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('*') || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let (Ok(f), Ok(mag), Ok(phase)) = (
                    parts[0].parse::<f64>(),
                    parts[1].parse::<f64>(),
                    parts[2].parse::<f64>(),
                ) {
                    frequencies.push(f);
                    magnitudes.push(mag);
                    phases.push(phase);
                }
            }
        }

        Ok(Self::new(frequencies, magnitudes, phases))
    }

    pub fn from_txt_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_zma_file(path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetResponse {
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<f64>,
}

impl TargetResponse {
    pub fn new(frequencies: Vec<f64>, magnitudes: Vec<f64>) -> Self {
        assert_eq!(frequencies.len(), magnitudes.len());
        Self { frequencies, magnitudes }
    }

    pub fn interpolate(&self, freq: f64) -> f64 {
        if self.frequencies.is_empty() {
            return 1.0;
        }
        if freq <= self.frequencies[0] {
            return self.magnitudes[0];
        }
        if freq >= self.frequencies[self.frequencies.len() - 1] {
            return self.magnitudes[self.frequencies.len() - 1];
        }

        let idx = self.frequencies.partition_point(|&f| f < freq);
        let f1 = self.frequencies[idx - 1];
        let f2 = self.frequencies[idx];
        let t = (freq - f1).ln() / (f2 - f1).ln();

        let mag1 = self.magnitudes[idx - 1].ln();
        let mag2 = self.magnitudes[idx].ln();
        (mag1 + t * (mag2 - mag1)).exp()
    }

    pub fn flat() -> Self {
        let frequencies = vec![20.0, 20000.0];
        let magnitudes = vec![1.0, 1.0];
        Self::new(frequencies, magnitudes)
    }

    pub fn house_curve() -> Self {
        let frequencies = vec![20.0, 100.0, 1000.0, 10000.0, 20000.0];
        let magnitudes = vec![1.2, 1.1, 1.0, 0.9, 0.8];
        Self::new(frequencies, magnitudes)
    }

    pub fn harman_like() -> Self {
        let frequencies = vec![20.0, 100.0, 200.0, 1000.0, 3000.0, 10000.0, 20000.0];
        let magnitudes = vec![1.3, 1.1, 1.0, 1.0, 0.95, 0.85, 0.7];
        Self::new(frequencies, magnitudes)
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let mut frequencies = Vec::new();
        let mut magnitudes = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('*') || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let (Ok(f), Ok(mag)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    frequencies.push(f);
                    magnitudes.push(mag);
                }
            }
        }

        Ok(Self::new(frequencies, magnitudes))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossoverType {
    Butterworth1,
    Butterworth2,
    Butterworth3,
    Butterworth4,
    LinkwitzRiley2,
    LinkwitzRiley4,
    LinkwitzRiley6,
    LinkwitzRiley8,
    Bessel2,
    Bessel3,
    Bessel4,
    Custom,
}

impl CrossoverType {
    pub fn order(&self) -> usize {
        match self {
            CrossoverType::Butterworth1 => 1,
            CrossoverType::Butterworth2 => 2,
            CrossoverType::Butterworth3 => 3,
            CrossoverType::Butterworth4 => 4,
            CrossoverType::LinkwitzRiley2 => 2,
            CrossoverType::LinkwitzRiley4 => 4,
            CrossoverType::LinkwitzRiley6 => 6,
            CrossoverType::LinkwitzRiley8 => 8,
            CrossoverType::Bessel2 => 2,
            CrossoverType::Bessel3 => 3,
            CrossoverType::Bessel4 => 4,
            CrossoverType::Custom => 0,
        }
    }

    pub fn is_linkwitz_riley(&self) -> bool {
        matches!(
            self,
            CrossoverType::LinkwitzRiley2
                | CrossoverType::LinkwitzRiley4
                | CrossoverType::LinkwitzRiley6
                | CrossoverType::LinkwitzRiley8
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    BandStop,
    AllPass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterSpec {
    pub filter_type: FilterType,
    pub crossover_type: CrossoverType,
    pub fc: f64,
    pub components: Vec<ComponentValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentValue {
    pub name: String,
    pub component_type: ComponentType,
    pub value: f64,
    pub tolerance: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    Resistor,
    Capacitor,
    Inductor,
    ResistorSeries,
    CapacitorSeries,
    InductorSeries,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationNetwork {
    pub comp_type: CompensationType,
    pub driver_id: String,
    pub components: Vec<ComponentValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompensationType {
    Zobel,
    Conjugate,
    Notch,
    BaffleStep,
    LPad,
    ImpedanceCorrection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossoverDesign {
    pub name: String,
    pub num_ways: usize,
    pub crossover_frequencies: Vec<f64>,
    pub crossover_types: Vec<CrossoverType>,
    pub drivers: Vec<String>,
    pub filters: Vec<FilterSpec>,
    pub compensation: Vec<CompensationNetwork>,
    pub target: Option<TargetResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub f_min: f64,
    pub f_max: f64,
    pub points_per_decade: usize,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            f_min: 20.0,
            f_max: 20000.0,
            points_per_decade: 200,
        }
    }
}

impl SimulationConfig {
    pub fn frequencies(&self) -> Vec<f64> {
        let decades = (self.f_max / self.f_min).log10();
        let total_points = (decades * self.points_per_decade as f64).ceil() as usize + 1;
        let mut freqs = Vec::with_capacity(total_points);
        for i in 0..total_points {
            let f = self.f_min * 10.0_f64.powf(i as f64 / self.points_per_decade as f64);
            if f <= self.f_max {
                freqs.push(f);
            }
        }
        freqs
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub frequencies: Vec<f64>,
    pub driver_voltages: HashMap<String, Vec<Cx>>,
    pub driver_currents: HashMap<String, Vec<Cx>>,
    pub driver_acoustic: HashMap<String, Vec<Cx>>,
    pub summed_acoustic: Vec<Cx>,
    pub system_impedance: Vec<Cx>,
    pub component_voltages: HashMap<String, Vec<Cx>>,
    pub component_currents: HashMap<String, Vec<Cx>>,
    pub component_power: HashMap<String, Vec<f64>>,
}