use crate::types::{Cx, FrequencyResponse, ImpedanceData};
use audio_spice_core::{Component, Netlist};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Driver {
    pub id: String,
    pub name: String,
    pub role: DriverRole,
    pub nominal_impedance: f64,
    pub re: f64,
    pub le: Option<f64>,
    pub sensitivity_db: f64,
    pub polarity: Polarity,
    pub acoustic_offset_mm: f64,
    pub impedance_data: Option<ImpedanceData>,
    pub frequency_response: Option<FrequencyResponse>,
    pub power_handling_w: Option<f64>,
    pub fs: Option<f64>,
    pub qts: Option<f64>,
    pub qes: Option<f64>,
    pub qms: Option<f64>,
    pub vas: Option<f64>,
    pub bl: Option<f64>,
    pub mms: Option<f64>,
    pub sd: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriverRole {
    Woofer,
    Midrange,
    Tweeter,
    SuperTweeter,
    Subwoofer,
    FullRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Polarity {
    Normal,
    Inverted,
}

impl Polarity {
    pub fn factor(&self) -> f64 {
        match self {
            Polarity::Normal => 1.0,
            Polarity::Inverted => -1.0,
        }
    }
}

impl Driver {
    pub fn new(id: String, name: String, role: DriverRole) -> Self {
        Self {
            id,
            name,
            role,
            nominal_impedance: 8.0,
            re: 6.0,
            le: None,
            sensitivity_db: 88.0,
            polarity: Polarity::Normal,
            acoustic_offset_mm: 0.0,
            impedance_data: None,
            frequency_response: None,
            power_handling_w: None,
            fs: None,
            qts: None,
            qes: None,
            qms: None,
            vas: None,
            bl: None,
            mms: None,
            sd: None,
        }
    }

    pub fn impedance_at(&self, freq: f64) -> Cx {
        if let Some(ref z_data) = self.impedance_data {
            z_data.interpolate(freq)
        } else if let Some(le) = self.le {
            let omega = 2.0 * PI * freq;
            Cx::new(self.re, omega * le)
        } else {
            Cx::new(self.re, 0.0)
        }
    }

    pub fn acoustic_response_at(&self, freq: f64) -> Cx {
        if let Some(ref fr_data) = self.frequency_response {
            fr_data.interpolate(freq)
        } else {
            Cx::new(1.0, 0.0)
        }
    }

    pub fn voltage_sensitivity(&self) -> f64 {
        10.0_f64.powf(self.sensitivity_db / 20.0)
    }

    pub fn add_to_netlist(&self, netlist: &mut Netlist, node_in: usize, node_out: usize, freq: f64) {
        let z = self.impedance_at(freq);
        if z.norm() > 1e-12 {
            let r_eq = z.re;
            let l_eq = if z.im > 0.0 { z.im / (2.0 * PI * freq) } else { 0.0 };
            
            if r_eq > 1e-6 {
                netlist.add_component(Component::Resistor {
                    name: format!("{}_Re", self.id),
                    n_pos: node_in,
                    n_neg: node_out,
                    value: r_eq,
                });
            }
            if l_eq > 1e-12 {
                netlist.add_component(Component::Inductor {
                    name: format!("{}_Le", self.id),
                    n_pos: node_in,
                    n_neg: node_out,
                    value: l_eq,
                });
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverDatabase {
    pub drivers: HashMap<String, Driver>,
}

impl DriverDatabase {
    pub fn new() -> Self {
        Self { drivers: HashMap::new() }
    }

    pub fn add_driver(&mut self, driver: Driver) {
        self.drivers.insert(driver.id.clone(), driver);
    }

    pub fn get(&self, id: &str) -> Option<&Driver> {
        self.drivers.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Driver> {
        self.drivers.get_mut(id)
    }
}

impl Default for DriverDatabase {
    fn default() -> Self {
        Self::new()
    }
}