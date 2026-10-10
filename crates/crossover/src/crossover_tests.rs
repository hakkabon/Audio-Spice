#[cfg(test)]
mod driver_tests {
    use crate::{Driver, DriverDatabase, DriverRole, Polarity};
    use crate::types::{FrequencyResponse, ImpedanceData};

    #[test]
    fn test_driver_new() {
        let driver = Driver::new("w".into(), "Woofer".into(), DriverRole::Woofer);
        
        assert_eq!(driver.id, "w");
        assert_eq!(driver.name, "Woofer");
        assert_eq!(driver.role, DriverRole::Woofer);
        assert_eq!(driver.nominal_impedance, 8.0);
        assert_eq!(driver.re, 6.0);
        assert_eq!(driver.sensitivity_db, 88.0);
        assert_eq!(driver.polarity, Polarity::Normal);
        assert_eq!(driver.acoustic_offset_mm, 0.0);
    }

    #[test]
    fn test_driver_impedance_no_data() {
        let mut driver = Driver::new("w".into(), "Woofer".into(), DriverRole::Woofer);
        driver.re = 6.0;
        driver.le = Some(0.5e-3);
        
        let z = driver.impedance_at(1000.0);
        
        // Z = Re + j*ω*Le
        let omega = 2.0 * std::f64::consts::PI * 1000.0;
        let expected_re = 6.0;
        let expected_im = omega * 0.5e-3;
        
        assert!((z.re - expected_re).abs() < 1e-6);
        assert!((z.im - expected_im).abs() < 1e-6);
    }

    #[test]
    fn test_driver_impedance_with_data() {
        let mut driver = Driver::new("w".into(), "Woofer".into(), DriverRole::Woofer);
        driver.impedance_data = Some(ImpedanceData::new(
            vec![100.0, 1000.0, 10000.0],
            vec![10.0, 8.0, 20.0],
            vec![0.0, 10.0, 45.0],
        ));
        
        // Test interpolation
        let z = driver.impedance_at(500.0); // Between 100 and 1000
        assert!(z.norm() > 0.0);
        
        let z = driver.impedance_at(50.0);  // Below range
        let z_at_100 = driver.impedance_at(100.0);
        assert!((z.norm() - z_at_100.norm()).abs() < 1e-6);
    }

    #[test]
    fn test_driver_acoustic_response() {
        let mut driver = Driver::new("t".into(), "Tweeter".into(), DriverRole::Tweeter);
        driver.frequency_response = Some(FrequencyResponse::new(
            vec![100.0, 1000.0, 10000.0],
            vec![1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0],
        ));
        
        let h = driver.acoustic_response_at(500.0);
        assert!((h.norm() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_driver_voltage_sensitivity() {
        let mut driver = Driver::new("w".into(), "Woofer".into(), DriverRole::Woofer);
        driver.sensitivity_db = 88.0;
        
        // 88dB = 20*log10(V) => V = 10^(88/20) ≈ 251.2
        let expected = 10.0_f64.powf(88.0 / 20.0);
        assert!((driver.voltage_sensitivity() - expected).abs() < 0.1);
    }

    #[test]
    fn test_driver_polarity_factor() {
        let mut driver_normal = Driver::new("w".into(), "Woofer".into(), DriverRole::Woofer);
        driver_normal.polarity = Polarity::Normal;
        assert_eq!(driver_normal.polarity.factor(), 1.0);
        
        let mut driver_inverted = Driver::new("t".into(), "Tweeter".into(), DriverRole::Tweeter);
        driver_inverted.polarity = Polarity::Inverted;
        assert_eq!(driver_inverted.polarity.factor(), -1.0);
    }

    #[test]
    fn test_driver_database() {
        let mut db = DriverDatabase::new();
        
        let driver1 = Driver::new("w".into(), "Woofer".into(), DriverRole::Woofer);
        let driver2 = Driver::new("t".into(), "Tweeter".into(), DriverRole::Tweeter);
        
        db.add_driver(driver1);
        db.add_driver(driver2);
        
        assert_eq!(db.drivers.len(), 2);
        assert!(db.get("w").is_some());
        assert!(db.get("t").is_some());
        assert!(db.get("missing").is_none());
        
        // Test mutable access
        if let Some(d) = db.get_mut("w") {
            d.sensitivity_db = 90.0;
        }
        assert_eq!(db.get("w").unwrap().sensitivity_db, 90.0);
    }

    #[test]
    fn test_driver_add_to_netlist() {
        use crate::netlist_builder::CrossoverNetlistBuilder;
        use audio_spice_core::{Component, Netlist};
        
        let mut driver = Driver::new("w".into(), "Woofer".into(), DriverRole::Woofer);
        driver.re = 6.0;
        driver.le = Some(0.5e-3);
        
        let mut netlist = Netlist::new();
        driver.add_to_netlist(&mut netlist, 1, 2, 1000.0);
        
        // Should add Re and Le
        assert_eq!(netlist.components.len(), 2);
        
        let has_resistor = netlist.components.iter().any(|c| matches!(c, Component::Resistor { .. }));
        let has_inductor = netlist.components.iter().any(|c| matches!(c, Component::Inductor { .. }));
        
        assert!(has_resistor);
        assert!(has_inductor);
    }
}

#[cfg(test)]
mod target_tests {
    use crate::types::TargetResponse;

    #[test]
    fn test_target_flat() {
        let target = TargetResponse::flat();
        
        assert_eq!(target.frequencies, vec![20.0, 20000.0]);
        assert_eq!(target.magnitudes, vec![1.0, 1.0]);
        
        assert!((target.interpolate(100.0) - 1.0).abs() < 1e-6);
        assert!((target.interpolate(10000.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_target_house_curve() {
        let target = TargetResponse::house_curve();
        
        assert_eq!(target.frequencies.len(), 5);
        assert_eq!(target.magnitudes[0], 1.2); // 20Hz
        assert_eq!(target.magnitudes[2], 1.0); // 1000Hz
        assert_eq!(target.magnitudes[4], 0.8); // 20000Hz
    }

    #[test]
    fn test_target_harman_like() {
        let target = TargetResponse::harman_like();
        
        assert_eq!(target.frequencies.len(), 7);
        assert!((target.interpolate(1000.0) - 1.0).abs() < 1e-6);
        assert!((target.interpolate(3000.0) - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_target_interpolation() {
        let target = TargetResponse::new(
            vec![100.0, 1000.0, 10000.0],
            vec![1.0, 0.5, 0.1],
        );
        
        // At 100Hz
        assert!((target.interpolate(100.0) - 1.0).abs() < 1e-6);
        
        // At 1000Hz
        assert!((target.interpolate(1000.0) - 0.5).abs() < 1e-6);
        
        // At 10000Hz
        assert!((target.interpolate(10000.0) - 0.1).abs() < 1e-6);
        
        // Logarithmic interpolation at 316Hz (geometric mean of 100 and 1000)
        let mid = target.interpolate(316.2);
        assert!(mid < 1.0 && mid > 0.5);
    }
}

#[cfg(test)]
mod netlist_builder_tests {
    use crate::{CrossoverDesign, CrossoverNetlistBuilder, CrossoverSimulator, SimulationConfig};
    use crate::types::{FilterType, CrossoverType, FilterSpec};
    use crate::DriverDatabase;
    use crate::synthesis::CrossoverSynthesizer;

    fn create_test_design() -> (CrossoverDesign, DriverDatabase) {
        let mut drivers = DriverDatabase::new();
        
        let mut woofer = crate::Driver::new("w".into(), "Woofer".into(), crate::DriverRole::Woofer);
        woofer.re = 6.0;
        woofer.le = Some(0.5e-3);
        woofer.sensitivity_db = 87.0;
        drivers.add_driver(woofer);
        
        let mut tweeter = crate::Driver::new("t".into(), "Tweeter".into(), crate::DriverRole::Tweeter);
        tweeter.re = 4.8;
        tweeter.le = Some(0.05e-3);
        tweeter.sensitivity_db = 90.0;
        drivers.add_driver(tweeter);
        
        let fc = 2500.0;
        let woofer_filter = FilterSpec {
            filter_type: FilterType::LowPass,
            crossover_type: CrossoverType::LinkwitzRiley4,
            fc,
            components: CrossoverSynthesizer::synthesize_filter(FilterType::LowPass, CrossoverType::LinkwitzRiley4, fc, 8.0, "W"),
        };
        
        let tweeter_filter = FilterSpec {
            filter_type: FilterType::HighPass,
            crossover_type: CrossoverType::LinkwitzRiley4,
            fc,
            components: CrossoverSynthesizer::synthesize_filter(FilterType::HighPass, CrossoverType::LinkwitzRiley4, fc, 8.0, "T"),
        };
        
        let design = CrossoverDesign {
            name: "Test".into(),
            num_ways: 2,
            crossover_frequencies: vec![fc],
            crossover_types: vec![CrossoverType::LinkwitzRiley4],
            drivers: vec!["w".into(), "t".into()],
            filters: vec![woofer_filter, tweeter_filter],
            compensation: vec![],
            target: None,
        };
        
        (design, drivers)
    }

    #[test]
    fn test_build_netlist() {
        let (design, drivers) = create_test_design();
        let builder = CrossoverNetlistBuilder::new(design.clone(), drivers.clone());
        
        let netlist = builder.build(1000.0);
        
        // Should have: V1, Rsource, W_C1, W_L1, W_L2, W_C2, woofer_Re, woofer_Le, T_L1, T_C1, T_C2, T_L2, tweeter_Re, tweeter_Le
        assert!(netlist.components.len() >= 14);
    }

    #[test]
    fn test_simulate() {
        let (design, drivers) = create_test_design();
        let config = SimulationConfig { f_min: 100.0, f_max: 10000.0, points_per_decade: 10 };
        let sim = CrossoverSimulator::new(design, drivers, config);
        
        let result = sim.simulate().expect("Simulation should succeed");
        
        assert!(!result.frequencies.is_empty());
        assert!(result.frequencies[0] >= 100.0);
        assert!(*result.frequencies.last().unwrap() <= 10000.0);
        
        // Should have results for both drivers
        assert!(result.driver_voltages.contains_key("w"));
        assert!(result.driver_voltages.contains_key("t"));
        assert!(result.driver_acoustic.contains_key("w"));
        assert!(result.driver_acoustic.contains_key("t"));
        
        // Summed acoustic should have same length as frequencies
        assert_eq!(result.summed_acoustic.len(), result.frequencies.len());
    }
}