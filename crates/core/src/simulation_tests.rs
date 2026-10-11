#[cfg(test)]
mod simulation_tests {
    use crate::{Component, Netlist, AcSweep, TranSweep, simulate_ac, simulate_dc, simulate_tran};
    use crate::types::Cx;

    #[test]
    fn test_ac_sweep_frequencies() {
        let sweep = AcSweep { f_start: 10.0, f_stop: 10000.0, points_per_decade: 10 };
        let freqs = sweep.frequencies();
        
        assert!(freqs.len() > 10);
        assert!((freqs[0] - 10.0).abs() < 1e-10);
        assert!(freqs.last().unwrap() <= &10000.0);
        
        // Check logarithmic spacing
        for i in 1..freqs.len() {
            let ratio = freqs[i] / freqs[i-1];
            let expected_ratio = 10.0_f64.powf(1.0/10.0);
            assert!((ratio - expected_ratio).abs() < 1e-2, "Ratio {} != {}", ratio, expected_ratio);
        }
    }

    #[test]
    fn test_ac_rc_lowpass() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 });
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 2, value: 1000.0 });
        netlist.add_component(Component::Capacitor { name: "C1".into(), n_pos: 2, n_neg: 0, value: 1e-6 });
        
        let sweep = AcSweep { f_start: 10.0, f_stop: 100000.0, points_per_decade: 20 };
        let result = simulate_ac(&netlist, &sweep).expect("AC simulation should succeed");
        
        assert_eq!(result.frequencies.len(), result.node_voltages.get(&2).unwrap().len());
        
        // At DC (10Hz), Vout ≈ Vin
        let vout_dc = result.node_voltages.get(&2).unwrap()[0];
        eprintln!("DC voltage: {}", vout_dc.norm());
        assert!((vout_dc.norm() - 1.0).abs() < 0.01);
        
        // Check a few frequencies
        for (i, &f) in result.frequencies.iter().enumerate() {
            let v = result.node_voltages.get(&2).unwrap()[i];
            eprintln!("f={:.1}Hz, Vout={:.6}", f, v.norm());
        }
        
        // At high frequency (100kHz), Vout should be attenuated
        let vout_hf = result.node_voltages.get(&2).unwrap().last().unwrap();
        eprintln!("HF voltage: {}, freq: {}", vout_hf.norm(), result.frequencies.last().unwrap());
        assert!(vout_hf.norm() < 0.1);
    }

    #[test]
    fn test_ac_voltage_divider() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 10.0, phase_deg: 0.0 });
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 2, value: 1000.0 });
        netlist.add_component(Component::Resistor { name: "R2".into(), n_pos: 2, n_neg: 0, value: 1000.0 });
        
        let sweep = AcSweep { f_start: 1000.0, f_stop: 1000.0, points_per_decade: 1 };
        let result = simulate_ac(&netlist, &sweep).expect("AC simulation should succeed");
        
        let vout = result.node_voltages.get(&2).unwrap()[0];
        
        // Should be 5V (half of 10V)
        assert!((vout.norm() - 5.0).abs() < 1e-6);
        assert!(vout.arg().abs() < 1e-10); // Purely real
    }

    #[test]
    fn test_ac_phase_shift() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 });
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 2, value: 1000.0 });
        netlist.add_component(Component::Capacitor { name: "C1".into(), n_pos: 2, n_neg: 0, value: 1e-6 });
        
        // At corner frequency fc = 1/(2πRC) ≈ 159Hz
        let fc = 1.0 / (2.0 * std::f64::consts::PI * 1000.0 * 1e-6);
        let sweep = AcSweep { f_start: fc, f_stop: fc, points_per_decade: 1 };
        let result = simulate_ac(&netlist, &sweep).expect("AC simulation should succeed");
        
        let vout = result.node_voltages.get(&2).unwrap()[0];
        
        // At corner frequency, phase should be -45 degrees
        assert!((vout.arg().to_degrees() + 45.0).abs() < 2.0);
        
        // Magnitude should be -3dB (0.707)
        assert!((vout.norm() - 0.707).abs() < 0.02);
    }

    #[test]
    fn test_dc_simple() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 5.0, phase_deg: 0.0 });
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 0, value: 1000.0 });
        
        let result = simulate_dc(&netlist, 100, 1e-6).expect("DC simulation should succeed");
        
        assert!(result.converged);
        assert!(result.iterations > 0);
        
        let v1 = result.node_voltages.get(&1).unwrap();
        assert!((v1 - 5.0).abs() < 1e-3); // Should be 5V
    }

    #[test]
    fn test_dc_voltage_divider() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 10.0, phase_deg: 0.0 });
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 2, value: 1000.0 });
        netlist.add_component(Component::Resistor { name: "R2".into(), n_pos: 2, n_neg: 0, value: 1000.0 });
        
        let result = simulate_dc(&netlist, 100, 1e-6).expect("DC simulation should succeed");
        
        assert!(result.converged);
        
        let v2 = result.node_voltages.get(&2).unwrap();
        assert!((v2 - 5.0).abs() < 1e-3); // Should be 5V
    }

    #[test]
    fn test_dc_current_source() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::CurrentSource { name: "I1".into(), n_pos: 1, n_neg: 0, amplitude: 0.001, phase_deg: 0.0 });
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 0, value: 1000.0 });
        
        let result = simulate_dc(&netlist, 100, 1e-6).expect("DC simulation should succeed");
        
        assert!(result.converged);
        
        // V = I * R = 1mA * 1k = 1V
        let v1 = result.node_voltages.get(&1).unwrap();
        assert!((v1 - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_tran_rc_step() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 5.0, phase_deg: 0.0 });
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 2, value: 1000.0 });
        netlist.add_component(Component::Capacitor { name: "C1".into(), n_pos: 2, n_neg: 0, value: 1e-6 });
        
        let sweep = TranSweep { t_start: 0.0, t_stop: 5e-3, t_step: 1e-5, t_max: 1e-5 };
        let result = simulate_tran(&netlist, &sweep).expect("Transient simulation should succeed");
        
        assert_eq!(result.time_points.len(), result.node_voltages.get(&2).unwrap().len());
        
        // Initial voltage should be 0 (stored as initial condition)
        let v2_0 = result.node_voltages.get(&2).unwrap()[0];
        assert!(v2_0.abs() < 0.1);
        
        // Check voltage at a few time points to verify charging behavior
        let v2_at_1ms = result.node_voltages.get(&2).unwrap()[100]; // 1ms = 100 * 10us
        // At 1ms (1 tau), voltage should be 5 * (1 - e^-1) ≈ 3.16V
        assert!((v2_at_1ms - 3.16).abs() < 0.2);
        
        // Final voltage should approach 5V (time constant = 1ms, 5ms = 5τ)
        let v2_final = *result.node_voltages.get(&2).unwrap().last().unwrap();
        assert!((v2_final - 5.0).abs() < 0.2); // More tolerance
    }

    #[test]
    fn test_tran_time_points() {
        let sweep = TranSweep { t_start: 0.0, t_stop: 1e-3, t_step: 1e-4, t_max: 1e-4 };
        let points = sweep.time_points();
        
        assert_eq!(points.len(), 11); // 0, 0.1ms, 0.2ms, ..., 1ms
        assert!((points[0] - 0.0).abs() < 1e-12);
        assert!((points[10] - 1e-3).abs() < 1e-12);
    }

    #[test]
    fn test_ac_vccs() {
        // Test voltage-controlled current source
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 });
        netlist.add_component(Component::VCCS { name: "G1".into(), n_out_pos: 2, n_out_neg: 0, n_ctrl_pos: 1, n_ctrl_neg: 0, transconductance: 0.001 });
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 2, n_neg: 0, value: 1000.0 });
        
        let sweep = AcSweep { f_start: 1000.0, f_stop: 1000.0, points_per_decade: 1 };
        let result = simulate_ac(&netlist, &sweep).expect("AC simulation should succeed");
        
        // Iout = gm * Vctrl = 0.001 * 1 = 1mA
        // Vout = Iout * R = 1mA * 1000 = 1V
        let vout = result.node_voltages.get(&2).unwrap()[0];
        assert!((vout.norm() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_ac_vcvs() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 });
        netlist.add_component(Component::VCVS { name: "E1".into(), n_out_pos: 2, n_out_neg: 0, n_ctrl_pos: 1, n_ctrl_neg: 0, gain: 10.0 });
        // Add load resistor to prevent floating node
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 2, n_neg: 0, value: 1000.0 });
        
        let sweep = AcSweep { f_start: 1000.0, f_stop: 1000.0, points_per_decade: 1 };
        let result = simulate_ac(&netlist, &sweep).expect("AC simulation should succeed");
        
        let vout = result.node_voltages.get(&2).unwrap()[0];
        assert!((vout.norm() - 10.0).abs() < 0.1);
    }
}