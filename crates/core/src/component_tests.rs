#[cfg(test)]
mod component_tests {
    use crate::{Component, Netlist};

    #[test]
    fn test_resistor_admittance() {
        let r = Component::Resistor { 
            name: "R1".into(), 
            n_pos: 1, 
            n_neg: 0, 
            value: 1000.0 
        };
        
        let y = r.admittance(1000.0).expect("Resistor should have admittance");
        
        // At any frequency, resistor admittance is 1/R
        assert!((y.re - 0.001).abs() < 1e-10);
        assert!(y.im.abs() < 1e-10);
    }

    #[test]
    fn test_capacitor_admittance() {
        let c = Component::Capacitor { 
            name: "C1".into(), 
            n_pos: 1, 
            n_neg: 0, 
            value: 1e-6 
        };
        
        let omega = 2.0 * std::f64::consts::PI * 1000.0;
        let y = c.admittance(omega).expect("Capacitor should have admittance");
        
        // Y = j*omega*C
        assert!(y.re.abs() < 1e-10);
        assert!((y.im - omega * 1e-6).abs() < 1e-10);
    }

    #[test]
    fn test_inductor_admittance() {
        let l = Component::Inductor { 
            name: "L1".into(), 
            n_pos: 1, 
            n_neg: 0, 
            value: 1e-3 
        };
        
        let omega = 2.0 * std::f64::consts::PI * 1000.0;
        let y = l.admittance(omega).expect("Inductor should have admittance");
        
        // Y = 1/(j*omega*L) = -j/(omega*L)
        assert!(y.re.abs() < 1e-10);
        assert!((y.im + 1.0/(omega * 1e-3)).abs() < 1e-10);
    }

    #[test]
    fn test_voltage_source_no_admittance() {
        let v = Component::VoltageSource { 
            name: "V1".into(), 
            n_pos: 1, 
            n_neg: 0, 
            amplitude: 1.0, 
            phase_deg: 0.0 
        };
        
        let y = v.admittance(1000.0);
        
        assert!(y.is_none(), "Voltage source should not have admittance");
    }

    #[test]
    fn test_current_source_no_admittance() {
        let i = Component::CurrentSource { 
            name: "I1".into(), 
            n_pos: 1, 
            n_neg: 0, 
            amplitude: 1.0, 
            phase_deg: 0.0 
        };
        
        let y = i.admittance(1000.0);
        
        assert!(y.is_none(), "Current source should not have admittance");
    }

    #[test]
    fn test_vccs_admittance() {
        let g = Component::VCCS { 
            name: "G1".into(), 
            n_out_pos: 1, 
            n_out_neg: 0, 
            n_ctrl_pos: 2, 
            n_ctrl_neg: 0, 
            transconductance: 0.001 
        };
        
        let y = g.admittance(1000.0);
        
        assert!(y.is_none(), "VCCS should not have simple admittance");
    }

    #[test]
    fn test_component_names() {
        let components = vec![
            Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 0, value: 1000.0 },
            Component::Capacitor { name: "C1".into(), n_pos: 1, n_neg: 0, value: 1e-6 },
            Component::Inductor { name: "L1".into(), n_pos: 1, n_neg: 0, value: 1e-3 },
            Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 },
            Component::CurrentSource { name: "I1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 },
        ];
        
        for comp in components {
            assert!(comp.get_name().is_some());
            assert!(!comp.get_name().unwrap().is_empty());
        }
    }

    #[test]
    fn test_is_voltage_source() {
        let v = Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 };
        let ccvs = Component::CCVS { name: "H1".into(), n_out_pos: 1, n_out_neg: 0, ctrl_vsrc_name: "V1".into(), transresistance: 1000.0 };
        let r = Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 0, value: 1000.0 };
        
        assert!(v.is_voltage_source());
        assert!(ccvs.is_voltage_source());
        assert!(!r.is_voltage_source());
    }

    #[test]
    fn test_is_current_source() {
        let i = Component::CurrentSource { name: "I1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 };
        let vccs = Component::VCCS { name: "G1".into(), n_out_pos: 1, n_out_neg: 0, n_ctrl_pos: 2, n_ctrl_neg: 0, transconductance: 0.001 };
        let cccs = Component::CCCS { name: "F1".into(), n_out_pos: 1, n_out_neg: 0, ctrl_vsrc_name: "V1".into(), gain: 10.0 };
        let r = Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 0, value: 1000.0 };
        
        assert!(i.is_current_source());
        assert!(vccs.is_current_source());
        assert!(cccs.is_current_source());
        assert!(!r.is_current_source());
    }
}

#[cfg(test)]
mod netlist_tests {
    use crate::{Component, Netlist};

    #[test]
    fn test_netlist_new() {
        let netlist = Netlist::new();
        
        assert_eq!(netlist.components.len(), 0);
        assert_eq!(netlist.max_node, 0);
    }

    #[test]
    fn test_netlist_add_resistor() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::Resistor { 
            name: "R1".into(), 
            n_pos: 1, 
            n_neg: 0, 
            value: 1000.0 
        });
        
        assert_eq!(netlist.components.len(), 1);
        assert_eq!(netlist.max_node, 1);
    }

    #[test]
    fn test_netlist_multiple_components() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 0, value: 1000.0 });
        netlist.add_component(Component::Capacitor { name: "C1".into(), n_pos: 1, n_neg: 2, value: 1e-6 });
        netlist.add_component(Component::Inductor { name: "L1".into(), n_pos: 2, n_neg: 0, value: 1e-3 });
        
        assert_eq!(netlist.components.len(), 3);
        assert_eq!(netlist.max_node, 2);
    }

    #[test]
    fn test_netlist_controlled_sources() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::VCCS { 
            name: "G1".into(), 
            n_out_pos: 1, 
            n_out_neg: 0, 
            n_ctrl_pos: 2, 
            n_ctrl_neg: 3, 
            transconductance: 0.001 
        });
        
        assert_eq!(netlist.max_node, 3);
    }

    #[test]
    fn test_netlist_vccs_nodes() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::VCCS { 
            name: "G1".into(), 
            n_out_pos: 1, 
            n_out_neg: 0, 
            n_ctrl_pos: 2, 
            n_ctrl_neg: 3, 
            transconductance: 0.001 
        });
        
        assert_eq!(netlist.max_node, 3);
    }

    #[test]
    fn test_netlist_vcvs_nodes() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::VCVS { 
            name: "E1".into(), 
            n_out_pos: 1, 
            n_out_neg: 0, 
            n_ctrl_pos: 2, 
            n_ctrl_neg: 3, 
            gain: 100000.0 
        });
        
        assert_eq!(netlist.max_node, 3);
    }

    #[test]
    fn test_netlist_cccs_nodes() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::CCCS { 
            name: "F1".into(), 
            n_out_pos: 1, 
            n_out_neg: 0, 
            ctrl_vsrc_name: "V1".into(), 
            gain: 10.0 
        });
        
        assert_eq!(netlist.max_node, 1);
    }

    #[test]
    fn test_netlist_ccvs_nodes() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::CCVS { 
            name: "H1".into(), 
            n_out_pos: 1, 
            n_out_neg: 0, 
            ctrl_vsrc_name: "V1".into(), 
            transresistance: 1000.0 
        });
        
        assert_eq!(netlist.max_node, 1);
    }

    #[test]
    fn test_netlist_opamp_nodes() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::OpAmp { 
            name: "U1".into(), 
            n_out: 1, 
            n_in_pos: 2, 
            n_in_neg: 3, 
            n_vcc: 4, 
            n_vee: 5, 
            gain: 100000.0, 
            gbw: 1e6, 
            slew_rate: 1e6 
        });
        
        assert_eq!(netlist.max_node, 5);
    }

    #[test]
    fn test_netlist_diode_nodes() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::Diode { 
            name: "D1".into(), 
            n_anode: 1, 
            n_cathode: 2, 
            model: "1N4148".into(), 
            area: 1.0 
        });
        
        assert_eq!(netlist.max_node, 2);
    }

    #[test]
    fn test_netlist_bjt_nodes() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::BJT { 
            name: "Q1".into(), 
            n_collector: 1, 
            n_base: 2, 
            n_emitter: 3, 
            model: "2N3904".into(), 
            area: 1.0 
        });
        
        assert_eq!(netlist.max_node, 3);
    }

    #[test]
    fn test_netlist_mosfet_nodes() {
        let mut netlist = Netlist::new();
        
        netlist.add_component(Component::MOSFET { 
            name: "M1".into(), 
            n_drain: 1, 
            n_gate: 2, 
            n_source: 3, 
            n_body: 4, 
            model: "BS170".into(), 
            w: 1e-6, 
            l: 1e-6 
        });
        
        assert_eq!(netlist.max_node, 4);
    }

    #[test]
    fn test_netlist_default() {
        let netlist = Netlist::default();
        
        assert_eq!(netlist.components.len(), 0);
        assert_eq!(netlist.max_node, 0);
    }
}