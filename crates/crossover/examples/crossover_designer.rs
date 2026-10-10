use audio_spice_crossover::{
    CrossoverDesign, CrossoverNetlistBuilder, CrossoverSimulator, CrossoverSynthesizer,
    CrossoverType, Driver, DriverDatabase, DriverRole, FilterSpec, FilterType,
    Polarity, SimulationConfig, TargetResponse,
};

fn main() {
    println!("=== Loudspeaker Passive Crossover Designer ===\n");

    // Create driver database
    let mut drivers = DriverDatabase::new();

    // Woofer driver
    let mut woofer = Driver::new("woofer".to_string(), "Woofer 6.5\"".to_string(), DriverRole::Woofer);
    woofer.re = 6.2;
    woofer.le = Some(0.5e-3);
    woofer.sensitivity_db = 87.0;
    woofer.nominal_impedance = 8.0;
    drivers.add_driver(woofer);

    // Tweeter driver
    let mut tweeter = Driver::new("tweeter".to_string(), "Tweeter 1\"".to_string(), DriverRole::Tweeter);
    tweeter.re = 4.8;
    tweeter.le = Some(0.05e-3);
    tweeter.sensitivity_db = 90.0;
    tweeter.nominal_impedance = 8.0;
    tweeter.polarity = Polarity::Inverted;
    tweeter.acoustic_offset_mm = 12.0;
    drivers.add_driver(tweeter);

    println!("Drivers loaded:");
    for (id, d) in &drivers.drivers {
        println!("  {} ({}): Re={:.1}Ω, Le={:.2}mH, Sens={:.1}dB", 
            d.name, id, d.re, d.le.unwrap_or(0.0)*1000.0, d.sensitivity_db);
    }

    // Create 2-way Linkwitz-Riley 4th order crossover at 2500 Hz
    let fc = 2500.0;
    let r_woofer = 8.0;
    let r_tweeter = 8.0;

    let woofer_filter = FilterSpec {
        filter_type: FilterType::LowPass,
        crossover_type: CrossoverType::LinkwitzRiley4,
        fc,
        components: CrossoverSynthesizer::synthesize_filter(
            FilterType::LowPass,
            CrossoverType::LinkwitzRiley4,
            fc,
            r_woofer,
            "W",
        ),
    };

    let tweeter_filter = FilterSpec {
        filter_type: FilterType::HighPass,
        crossover_type: CrossoverType::LinkwitzRiley4,
        fc,
        components: CrossoverSynthesizer::synthesize_filter(
            FilterType::HighPass,
            CrossoverType::LinkwitzRiley4,
            fc,
            r_tweeter,
            "T",
        ),
    };

    println!("\nWoofer Low-Pass (LR4 @ {} Hz):", fc);
    for c in &woofer_filter.components {
        println!("  {}: {}", c.name, format_component_value(c.value, c.component_type));
    }

    println!("\nTweeter High-Pass (LR4 @ {} Hz):", fc);
    for c in &tweeter_filter.components {
        println!("  {}: {}", c.name, format_component_value(c.value, c.component_type));
    }

    // Add Zobel network for woofer
    let zobel = CrossoverSynthesizer::synthesize_zobel(6.2, 0.5e-3, "W_Zobel");
    println!("\nWoofer Zobel Network:");
    for c in &zobel {
        println!("  {}: {}", c.name, format_component_value(c.value, c.component_type));
    }

    // Add L-pad for tweeter (3dB attenuation)
    let lpad = CrossoverSynthesizer::synthesize_lpad(8.0, 3.0, "T_LPad");
    println!("\nTweeter L-Pad (-3dB):");
    for c in &lpad {
        println!("  {}: {}", c.name, format_component_value(c.value, c.component_type));
    }

    // Create crossover design
    let design = CrossoverDesign {
        name: "2-Way LR4 Crossover".to_string(),
        num_ways: 2,
        crossover_frequencies: vec![fc],
        crossover_types: vec![CrossoverType::LinkwitzRiley4],
        drivers: vec!["woofer".to_string(), "tweeter".to_string()],
        filters: vec![woofer_filter, tweeter_filter],
        compensation: vec![],
        target: Some(TargetResponse::flat()),
    };

    // Standardize to E12 values
    let mut design_standard = design.clone();
    for filter in &mut design_standard.filters {
        CrossoverSynthesizer::standardize_components(&mut filter.components, audio_spice_crossover::StandardSeries::E12);
    }
    println!("\nStandardized to E12 values:");
    for filter in &design_standard.filters {
        for c in &filter.components {
            println!("  {}: {}", c.name, format_component_value(c.value, c.component_type));
        }
    }

    // Simulate
    let sim_config = SimulationConfig {
        f_min: 20.0,
        f_max: 20000.0,
        points_per_decade: 20,  // Reduced for faster testing
    };
    let simulator = CrossoverSimulator::new(design_standard, drivers.clone(), sim_config);
    
    println!("\nRunning AC simulation...");
    match simulator.simulate() {
        Ok(result) => {
            println!("Simulation complete!");
            println!("Frequency points: {}", result.frequencies.len());
            
            // Print some results
            println!("\nSample results (frequency, summed acoustic magnitude in dB):");
            for i in (0..result.frequencies.len()).step_by((result.frequencies.len() / 10).max(1)).take(10) {
                let f = result.frequencies[i];
                let mag = result.summed_acoustic[i].norm();
                let db = 20.0 * mag.log10();
                println!("  {:.1} Hz: {:.2} dB", f, db);
            }

            // System impedance
            println!("\nSystem impedance at key frequencies:");
            for &f in &[20.0, 100.0, 1000.0, 2500.0, 10000.0, 20000.0] {
                if let Some(idx) = result.frequencies.iter().position(|&x| (x - f).abs() < 1.0) {
                    let z = result.system_impedance[idx];
                    println!("  {:.0} Hz: {:.2} Ω ∠ {:.1}°", f, z.norm(), z.arg().to_degrees());
                }
            }
        }
        Err(e) => {
            eprintln!("Simulation error: {}", e);
        }
    }

    // Test netlist builder
    println!("\n=== Testing Netlist Builder ===");
    let builder = CrossoverNetlistBuilder::new(design, drivers.clone());
    let netlist = builder.build(1000.0);
    println!("Netlist built with {} components at 1000 Hz", netlist.components.len());
    for comp in &netlist.components {
        println!("  {}", comp.get_name().unwrap_or("unknown"));
    }
}

fn component_type_str(ct: audio_spice_crossover::ComponentType) -> &'static str {
    match ct {
        audio_spice_crossover::ComponentType::Resistor => "Ω",
        audio_spice_crossover::ComponentType::Capacitor => "F",
        audio_spice_crossover::ComponentType::Inductor => "H",
        audio_spice_crossover::ComponentType::ResistorSeries => "Ω (series)",
        audio_spice_crossover::ComponentType::CapacitorSeries => "F (series)",
        audio_spice_crossover::ComponentType::InductorSeries => "H (series)",
    }
}

fn format_component_value(value: f64, ct: audio_spice_crossover::ComponentType) -> String {
    match ct {
        audio_spice_crossover::ComponentType::Capacitor => {
            if value >= 1e-3 {
                format!("{:.3} mF", value * 1000.0)
            } else if value >= 1e-6 {
                format!("{:.3} µF", value * 1e6)
            } else if value >= 1e-9 {
                format!("{:.3} nF", value * 1e9)
            } else {
                format!("{:.3} pF", value * 1e12)
            }
        }
        audio_spice_crossover::ComponentType::Inductor => {
            if value >= 1.0 {
                format!("{:.3} H", value)
            } else if value >= 1e-3 {
                format!("{:.3} mH", value * 1000.0)
            } else {
                format!("{:.3} µH", value * 1e6)
            }
        }
        _ => format!("{:.3} Ω", value),
    }
}