#[cfg(test)]
mod synthesis_tests {
    use crate::synthesis::CrossoverSynthesizer;
    use crate::types::{FilterType, CrossoverType};

    #[test]
    fn test_butterworth_1st_order_lowpass() {
        let components = CrossoverSynthesizer::synthesize_filter(
            FilterType::LowPass,
            CrossoverType::Butterworth1,
            1000.0,
            8.0,
            "W",
        );
        
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].component_type, crate::types::ComponentType::Inductor);
        // L = R / (2πf) = 8 / (2π * 1000) ≈ 1.27mH
        let expected = 8.0 / (2.0 * std::f64::consts::PI * 1000.0);
        assert!((components[0].value - expected).abs() < expected * 0.01);
    }

    #[test]
    fn test_butterworth_1st_order_highpass() {
        let components = CrossoverSynthesizer::synthesize_filter(
            FilterType::HighPass,
            CrossoverType::Butterworth1,
            1000.0,
            8.0,
            "T",
        );
        
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].component_type, crate::types::ComponentType::Capacitor);
        // C = 1 / (2πfR) = 1 / (2π * 1000 * 8) ≈ 19.9µF
        let expected = 1.0 / (2.0 * std::f64::consts::PI * 1000.0 * 8.0);
        assert!((components[0].value - expected).abs() < expected * 0.01);
    }

    #[test]
    fn test_linkwitz_riley_4th_order_lowpass() {
        let components = CrossoverSynthesizer::synthesize_filter(
            FilterType::LowPass,
            CrossoverType::LinkwitzRiley4,
            2500.0,
            8.0,
            "W",
        );
        
        // LR4 lowpass: C1, L1, L2, C2 (4 components)
        assert_eq!(components.len(), 4);
        
        // Check component types
        assert_eq!(components[0].component_type, crate::types::ComponentType::Capacitor);
        assert_eq!(components[1].component_type, crate::types::ComponentType::Inductor);
        assert_eq!(components[2].component_type, crate::types::ComponentType::Inductor);
        assert_eq!(components[3].component_type, crate::types::ComponentType::Capacitor);
    }

    #[test]
    fn test_linkwitz_riley_4th_order_highpass() {
        let components = CrossoverSynthesizer::synthesize_filter(
            FilterType::HighPass,
            CrossoverType::LinkwitzRiley4,
            2500.0,
            8.0,
            "T",
        );
        
        assert_eq!(components.len(), 4);
        
        // Highpass: L1, C1, C2, L2
        assert_eq!(components[0].component_type, crate::types::ComponentType::Inductor);
        assert_eq!(components[1].component_type, crate::types::ComponentType::Capacitor);
        assert_eq!(components[2].component_type, crate::types::ComponentType::Capacitor);
        assert_eq!(components[3].component_type, crate::types::ComponentType::Inductor);
    }

    #[test]
    fn test_zobel_network() {
        let components = CrossoverSynthesizer::synthesize_zobel(6.2, 0.5e-3, "W_Zobel");
        
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].component_type, crate::types::ComponentType::Resistor);
        assert_eq!(components[1].component_type, crate::types::ComponentType::Capacitor);
        
        // R = Re * 1.25 = 7.75
        assert!((components[0].value - 7.75).abs() < 0.01);
        // C = Le / Re^2 = 0.5mH / 6.2^2 ≈ 13µF
        let expected_c = 0.5e-3 / (6.2 * 6.2);
        assert!((components[1].value - expected_c).abs() < expected_c * 0.01);
    }

    #[test]
    fn test_lpad_network() {
        let components = CrossoverSynthesizer::synthesize_lpad(8.0, 3.0, "T_LPad");
        
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].component_type, crate::types::ComponentType::Resistor);
        assert_eq!(components[1].component_type, crate::types::ComponentType::Resistor);
        
        // For 3dB attenuation: k = 10^(-3/20) ≈ 0.707
        // R_series = R * (k - 1) / (k + 1) ≈ 8 * (0.707 - 1) / (0.707 + 1) ≈ -1.37 (wait, this is negative?)
        // Actually k = 10^(3/20) ≈ 1.4125 for -3dB (attenuation)
        let k = 10.0_f64.powf(3.0 / 20.0);
        let expected_r_series = 8.0 * (k - 1.0) / (k + 1.0);
        let expected_r_parallel = 8.0 * (k * k - 1.0) / (2.0 * k);
        
        assert!((components[0].value - expected_r_series).abs() < 0.01);
        assert!((components[1].value - expected_r_parallel).abs() < 0.01);
    }

    #[test]
    fn test_notch_filter() {
        let components = CrossoverSynthesizer::synthesize_notch(1000.0, 5.0, 8.0, "Notch");
        
        assert_eq!(components.len(), 3);
        // C, L, R
    }

    #[test]
    fn test_baffle_step() {
        let components = CrossoverSynthesizer::synthesize_baffle_step(500.0, 8.0, 6.0, "BS");
        
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].component_type, crate::types::ComponentType::Resistor);
        assert_eq!(components[1].component_type, crate::types::ComponentType::Inductor);
    }

    #[test]
    fn test_round_to_e12() {
        // Test values that should round to E12 series
        let test_cases = vec![
            (1.0, 1.0),
            (1.1, 1.2),  // Rounds to 1.2 (floating point closer to 1.2)
            (1.2, 1.2),
            (1.3, 1.2),
            (1.5, 1.5),
            (1.8, 1.8),
            (2.2, 2.2),
            (2.7, 2.7),
            (3.3, 3.3),
            (3.9, 3.9),
            (4.7, 4.7),
            (5.6, 5.6),
            (6.8, 6.8),
            (8.2, 8.2),
            (10.0, 10.0),
            (100.0, 100.0),
            (1000.0, 1000.0),
        ];
        
        for (input, expected) in test_cases {
            let result = CrossoverSynthesizer::round_to_e12(input);
            assert!((result - expected).abs() < 0.01, "Input: {}, got: {}, expected: {}", input, result, expected);
        }
    }

    #[test]
    fn test_round_to_e24() {
        let test_cases = vec![
            (1.0, 1.0),
            (1.1, 1.1),
            (1.2, 1.2),
            (1.3, 1.3),
            (1.5, 1.5),
            (1.6, 1.6),
            (1.8, 1.8),
            (2.0, 2.0),
            (2.2, 2.2),
            (2.4, 2.4),
            (2.7, 2.7),
            (3.0, 3.0),
            (3.3, 3.3),
            (3.6, 3.6),
            (3.9, 3.9),
            (4.3, 4.3),
            (4.7, 4.7),
            (5.1, 5.1),
            (5.6, 5.6),
            (6.2, 6.2),
            (6.8, 6.8),
            (7.5, 7.5),
            (8.2, 8.2),
            (9.1, 9.1),
        ];
        
        for (input, expected) in test_cases {
            let result = CrossoverSynthesizer::round_to_e24(input);
            assert!((result - expected).abs() < 0.01, "Input: {}, got: {}, expected: {}", input, result, expected);
        }
    }

    #[test]
    fn test_standardize_components() {
        let mut components = vec![
            crate::types::ComponentValue { 
                name: "C1".into(), 
                component_type: crate::types::ComponentType::Capacitor, 
                value: 1.1e-6, 
                tolerance: Some(0.1) 
            },
            crate::types::ComponentValue { 
                name: "L1".into(), 
                component_type: crate::types::ComponentType::Inductor, 
                value: 1.1e-3, 
                tolerance: Some(0.1) 
            },
        ];
        
        CrossoverSynthesizer::standardize_components(&mut components, crate::synthesis::StandardSeries::E12);
        
        // 1.1 rounds to 1.2 in E12 (floating point)
        assert!((components[0].value - 1.2e-6).abs() < 1e-9);
        assert!((components[1].value - 1.2e-3).abs() < 1e-6);
    }
}