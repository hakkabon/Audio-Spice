use crate::driver::DriverDatabase;
use crate::types::{
    ComponentType, ComponentValue, CompensationType,
    CrossoverDesign, FilterSpec, SimulationConfig, SimulationResult, Cx,
};
use audio_spice_core::{AcSweep, Component, Netlist, engine::simulate_ac};
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct NetlistBuilderConfig {
    pub source_impedance: f64,
    pub include_driver_models: bool,
}

impl Default for NetlistBuilderConfig {
    fn default() -> Self {
        Self {
            source_impedance: 0.1,
            include_driver_models: true,
        }
    }
}

pub struct CrossoverNetlistBuilder {
    design: CrossoverDesign,
    drivers: DriverDatabase,
    config: NetlistBuilderConfig,
}

impl CrossoverNetlistBuilder {
    pub fn new(design: CrossoverDesign, drivers: DriverDatabase) -> Self {
        Self {
            design,
            drivers,
            config: NetlistBuilderConfig::default(),
        }
    }

    pub fn with_config(mut self, config: NetlistBuilderConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build(&self, freq: f64) -> Netlist {
        let mut netlist = Netlist::new();
        let mut node_counter = 1;
        let ground = 0;

        let input_node = node_counter;
        node_counter += 1;
        let amp_node = node_counter;
        node_counter += 1;

        netlist.add_component(Component::VoltageSource {
            name: "V1".to_string(),
            n_pos: input_node,
            n_neg: ground,
            amplitude: 1.0,
            phase_deg: 0.0,
        });

        if self.config.source_impedance > 0.0 {
            netlist.add_component(Component::Resistor {
                name: "Rsource".to_string(),
                n_pos: input_node,
                n_neg: amp_node,
                value: self.config.source_impedance,
            });
        } else {
            node_counter -= 1;
        }

        let mut driver_output_nodes = HashMap::new();
        let mut driver_in_nodes = Vec::new();

        // First pass: allocate nodes for all drivers
        for (way_idx, driver_id) in self.design.drivers.iter().enumerate() {
            let filter_spec = &self.design.filters[way_idx];
            
            // Each filter needs (num_components - 1) intermediate nodes + driver_in + driver_out
            let num_intermediate = filter_spec.components.len().saturating_sub(1);
            let driver_in = node_counter;
            node_counter += 1 + num_intermediate;
            let driver_out = node_counter;
            node_counter += 1;
            
            driver_in_nodes.push((driver_id.clone(), driver_in, driver_out, num_intermediate));
        }

        // Second pass: build filter networks
        for (way_idx, (driver_id, driver_in, driver_out, _num_intermediate)) in driver_in_nodes.iter().enumerate() {
            let driver = self.drivers.get(driver_id).expect("Driver not found");
            let filter_spec = &self.design.filters[way_idx];
            
            driver_output_nodes.insert(driver_id.clone(), *driver_out);

            self.build_filter_network(
                &mut netlist,
                amp_node,
                *driver_in,
                *driver_out,
                filter_spec,
                freq,
                &format!("W{}", way_idx + 1),
            );

            if self.config.include_driver_models {
                driver.add_to_netlist(&mut netlist, *driver_in, *driver_out, freq);
            }
        }

        self.build_compensation_networks(&mut netlist, &driver_output_nodes, freq);

        netlist
    }

    fn build_filter_network(
        &self,
        netlist: &mut Netlist,
        amp_node: usize,
        driver_in: usize,
        driver_out: usize,
        filter_spec: &FilterSpec,
        _freq: f64,
        _prefix: &str,
    ) {
        let mut current_node = amp_node;

        for (idx, comp) in filter_spec.components.iter().enumerate() {
            let next_node = if idx < filter_spec.components.len() - 1 {
                driver_in + idx + 1
            } else {
                driver_in
            };

            match comp.component_type {
                ComponentType::Inductor => {
                    netlist.add_component(Component::Inductor {
                        name: comp.name.clone(),
                        n_pos: current_node,
                        n_neg: next_node,
                        value: comp.value,
                    });
                    current_node = next_node;
                }
                ComponentType::Capacitor => {
                    netlist.add_component(Component::Capacitor {
                        name: comp.name.clone(),
                        n_pos: current_node,
                        n_neg: next_node,
                        value: comp.value,
                    });
                    current_node = next_node;
                }
                ComponentType::Resistor => {
                    netlist.add_component(Component::Resistor {
                        name: comp.name.clone(),
                        n_pos: current_node,
                        n_neg: next_node,
                        value: comp.value,
                    });
                    current_node = next_node;
                }
                ComponentType::InductorSeries => {
                    netlist.add_component(Component::Inductor {
                        name: comp.name.clone(),
                        n_pos: driver_in,
                        n_neg: driver_out,
                        value: comp.value,
                    });
                }
                ComponentType::CapacitorSeries => {
                    netlist.add_component(Component::Capacitor {
                        name: comp.name.clone(),
                        n_pos: driver_in,
                        n_neg: driver_out,
                        value: comp.value,
                    });
                }
                ComponentType::ResistorSeries => {
                    netlist.add_component(Component::Resistor {
                        name: comp.name.clone(),
                        n_pos: driver_in,
                        n_neg: driver_out,
                        value: comp.value,
                    });
                }
            }
        }
    }

    fn build_compensation_networks(
        &self,
        netlist: &mut Netlist,
        driver_output_nodes: &HashMap<String, usize>,
        _freq: f64,
    ) {
        for comp_net in &self.design.compensation {
            if let Some(&driver_node) = driver_output_nodes.get(&comp_net.driver_id) {
                let ground = 0;
                match comp_net.comp_type {
                    CompensationType::Zobel => {
                        self.build_zobel_network(netlist, driver_node, ground, &comp_net.components);
                    }
                    CompensationType::LPad => {
                        self.build_lpad_network(netlist, driver_node, ground, &comp_net.components);
                    }
                    CompensationType::Notch => {
                        self.build_notch_network(netlist, driver_node, ground, &comp_net.components);
                    }
                    CompensationType::BaffleStep => {
                        self.build_baffle_step_network(netlist, driver_node, ground, &comp_net.components);
                    }
                    _ => {}
                }
            }
        }
    }

    fn build_zobel_network(
        &self,
        netlist: &mut Netlist,
        driver_node: usize,
        ground: usize,
        components: &[ComponentValue],
    ) {
        let mut r_val = 0.0;
        let mut c_val = 0.0;
        for comp in components {
            match comp.component_type {
                ComponentType::Resistor => r_val = comp.value,
                ComponentType::Capacitor => c_val = comp.value,
                _ => {}
            }
        }
        if r_val > 0.0 && c_val > 0.0 {
            let zobel_node = driver_node + 2000;
            netlist.add_component(Component::Resistor {
                name: format!("{}_R", components[0].name.split('_').next().unwrap_or("Zobel")),
                n_pos: driver_node,
                n_neg: zobel_node,
                value: r_val,
            });
            netlist.add_component(Component::Capacitor {
                name: format!("{}_C", components[0].name.split('_').next().unwrap_or("Zobel")),
                n_pos: zobel_node,
                n_neg: ground,
                value: c_val,
            });
        }
    }

    fn build_lpad_network(
        &self,
        netlist: &mut Netlist,
        driver_node: usize,
        ground: usize,
        components: &[ComponentValue],
    ) {
        let mut r_series = 0.0;
        let mut r_parallel = 0.0;
        for comp in components {
            match comp.component_type {
                ComponentType::Resistor => {
                    if comp.name.contains("series") || comp.name.contains("Series") {
                        r_series = comp.value;
                    } else {
                        r_parallel = comp.value;
                    }
                }
                _ => {}
            }
        }
        if r_series > 0.0 {
            netlist.add_component(Component::Resistor {
                name: format!("{}_Rseries", components[0].name.split('_').next().unwrap_or("LPad")),
                n_pos: driver_node,
                n_neg: driver_node + 3000,
                value: r_series,
            });
        }
        if r_parallel > 0.0 {
            netlist.add_component(Component::Resistor {
                name: format!("{}_Rparallel", components[0].name.split('_').next().unwrap_or("LPad")),
                n_pos: driver_node + 3000,
                n_neg: ground,
                value: r_parallel,
            });
        }
    }

    fn build_notch_network(
        &self,
        netlist: &mut Netlist,
        driver_node: usize,
        ground: usize,
        components: &[ComponentValue],
    ) {
        let mut c_val = 0.0;
        let mut l_val = 0.0;
        let mut r_val = 0.0;
        for comp in components {
            match comp.component_type {
                ComponentType::Capacitor => c_val = comp.value,
                ComponentType::Inductor => l_val = comp.value,
                ComponentType::Resistor => r_val = comp.value,
                _ => {}
            }
        }
        if c_val > 0.0 && l_val > 0.0 && r_val > 0.0 {
            let notch_node = driver_node + 4000;
            netlist.add_component(Component::Capacitor {
                name: format!("{}_C", components[0].name.split('_').next().unwrap_or("Notch")),
                n_pos: driver_node,
                n_neg: notch_node,
                value: c_val,
            });
            netlist.add_component(Component::Inductor {
                name: format!("{}_L", components[0].name.split('_').next().unwrap_or("Notch")),
                n_pos: notch_node,
                n_neg: notch_node + 1,
                value: l_val,
            });
            netlist.add_component(Component::Resistor {
                name: format!("{}_R", components[0].name.split('_').next().unwrap_or("Notch")),
                n_pos: notch_node + 1,
                n_neg: ground,
                value: r_val,
            });
        }
    }

    fn build_baffle_step_network(
        &self,
        netlist: &mut Netlist,
        driver_node: usize,
        ground: usize,
        components: &[ComponentValue],
    ) {
        let mut r_val = 0.0;
        let mut l_val = 0.0;
        for comp in components {
            match comp.component_type {
                ComponentType::Resistor => r_val = comp.value,
                ComponentType::Inductor => l_val = comp.value,
                _ => {}
            }
        }
        if r_val > 0.0 && l_val > 0.0 {
            let bs_node = driver_node + 5000;
            netlist.add_component(Component::Resistor {
                name: format!("{}_R", components[0].name.split('_').next().unwrap_or("BaffleStep")),
                n_pos: driver_node,
                n_neg: bs_node,
                value: r_val,
            });
            netlist.add_component(Component::Inductor {
                name: format!("{}_L", components[0].name.split('_').next().unwrap_or("BaffleStep")),
                n_pos: bs_node,
                n_neg: ground,
                value: l_val,
            });
        }
    }
}

pub struct CrossoverSimulator {
    design: CrossoverDesign,
    drivers: DriverDatabase,
    config: SimulationConfig,
}

impl CrossoverSimulator {
    pub fn new(design: CrossoverDesign, drivers: DriverDatabase, config: SimulationConfig) -> Self {
        Self {
            design,
            drivers,
            config,
        }
    }

    pub fn simulate(&self) -> Result<SimulationResult, String> {
        let frequencies = self.config.frequencies();

        let builder = CrossoverNetlistBuilder::new(self.design.clone(), self.drivers.clone());

        let mut driver_voltages: HashMap<String, Vec<Cx>> = HashMap::new();
        let mut driver_currents: HashMap<String, Vec<Cx>> = HashMap::new();
        let mut driver_acoustic: HashMap<String, Vec<Cx>> = HashMap::new();
        let mut summed_acoustic = vec![Cx::new(0.0, 0.0); frequencies.len()];
        let mut system_impedance = vec![Cx::new(0.0, 0.0); frequencies.len()];
        let mut component_currents: HashMap<String, Vec<Cx>> = HashMap::new();

        for driver_id in &self.design.drivers {
            driver_voltages.insert(driver_id.clone(), Vec::with_capacity(frequencies.len()));
            driver_currents.insert(driver_id.clone(), Vec::with_capacity(frequencies.len()));
            driver_acoustic.insert(driver_id.clone(), Vec::with_capacity(frequencies.len()));
        }

        for (i, &freq) in frequencies.iter().enumerate() {
            let netlist = builder.build(freq);
            let sweep = AcSweep {
                f_start: freq,
                f_stop: freq,
                points_per_decade: 1,
            };
            let result = simulate_ac(&netlist, &sweep)?;

            let amp_node = 2;
            let v_amp_cx = result.node_voltages.get(&amp_node).unwrap()[0];
            let v_amp = Cx::new(v_amp_cx.re, v_amp_cx.im);

            let mut total_current = Cx::new(0.0, 0.0);

            for driver_id in &self.design.drivers {
                let driver = self.drivers.get(driver_id).unwrap();
                let driver_idx = self.design.drivers.iter().position(|d| d == driver_id).unwrap();
                let _filter_spec = &self.design.filters[driver_idx];

                let driver_out_node = 3 + driver_idx * 2 + 1;
                let v_driver_cx = result.node_voltages.get(&driver_out_node).unwrap_or(&vec![num_complex::Complex::new(0.0, 0.0)])[0];
                let v_driver = Cx::new(v_driver_cx.re, v_driver_cx.im);

                let z_driver = driver.impedance_at(freq);
                let i_driver = if z_driver.norm() > 1e-12 { v_driver / z_driver } else { Cx::new(0.0, 0.0) };

                let acoustic_resp = driver.acoustic_response_at(freq);
                let acoustic = v_driver * acoustic_resp * driver.voltage_sensitivity() * driver.polarity.factor();

                let delay_samples = driver.acoustic_offset_mm / 343.0 * freq * 2.0 * PI;
                let delay_phase = Cx::from_polar(1.0, -delay_samples);
                let acoustic_delayed = acoustic * delay_phase;

                driver_voltages.get_mut(driver_id).unwrap().push(v_driver);
                driver_currents.get_mut(driver_id).unwrap().push(i_driver);
                driver_acoustic.get_mut(driver_id).unwrap().push(acoustic_delayed);

                total_current += i_driver;
                summed_acoustic[i] += acoustic_delayed;
            }

            system_impedance[i] = if total_current.norm() > 1e-12 { v_amp / total_current } else { Cx::new(1e12, 0.0) };

            for (comp_name, comp_current) in &result.vsource_currents {
                let cx = Cx::new(comp_current[0].re, comp_current[0].im);
                component_currents.entry(comp_name.clone()).or_insert_with(Vec::new).push(cx);
            }
        }

        Ok(SimulationResult {
            frequencies,
            driver_voltages,
            driver_currents,
            driver_acoustic,
            summed_acoustic,
            system_impedance,
            component_voltages: HashMap::new(),
            component_currents,
            component_power: HashMap::new(),
        })
    }
}