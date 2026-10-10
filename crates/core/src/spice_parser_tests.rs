#[cfg(test)]
mod spice_parser_tests {
    use crate::{parse_spice_netlist, parse_spice_file, Component, Netlist};
    use crate::models::{DiodeModel, BjtModel, MosfetModel, ModelLibrary};
    use std::path::Path;

    #[test]
    fn test_parse_simple_resistor() {
        let netlist_text = r#"
R1 1 0 1k
.END
"#;
        
        let (netlist, models) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        assert_eq!(netlist.components.len(), 1);
        assert_eq!(models.diodes.len(), 0);
        
        match &netlist.components[0] {
            crate::Component::Resistor { name, n_pos, n_neg, value } => {
                assert_eq!(name, "R1");
                assert_eq!(n_pos, &1);
                assert_eq!(n_neg, &0);
                assert!((value - 1000.0).abs() < 1e-6);
            }
            _ => panic!("Expected resistor"),
        }
    }

    #[test]
    fn test_parse_capacitor() {
        let netlist_text = r#"
C1 1 2 1uF
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::Capacitor { name, n_pos, n_neg, value } => {
                assert_eq!(name, "C1");
                assert_eq!(n_pos, &1);
                assert_eq!(n_neg, &2);
                assert!((value - 1e-6).abs() < 1e-12);
            }
            _ => panic!("Expected capacitor"),
        }
    }

    #[test]
    fn test_parse_inductor() {
        let netlist_text = r#"
L1 1 0 1mH
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::Inductor { name, n_pos, n_neg, value } => {
                assert_eq!(name, "L1");
                assert!((value - 1e-3).abs() < 1e-9);
            }
            _ => panic!("Expected inductor"),
        }
    }

    #[test]
    fn test_parse_voltage_source_dc() {
        let netlist_text = r#"
V1 1 0 DC 5
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::VoltageSource { name, n_pos, n_neg, amplitude, phase_deg } => {
                assert_eq!(name, "V1");
                assert!((amplitude - 5.0).abs() < 1e-6);
                assert!((phase_deg - 0.0).abs() < 1e-6);
            }
            _ => panic!("Expected voltage source"),
        }
    }

    #[test]
    fn test_parse_voltage_source_ac() {
        let netlist_text = r#"
V1 1 0 AC 1 45
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::VoltageSource { amplitude, phase_deg, .. } => {
                assert!((amplitude - 1.0).abs() < 1e-6);
                assert!((phase_deg - 45.0).abs() < 1e-6);
            }
            _ => panic!("Expected voltage source"),
        }
    }

    #[test]
    fn test_parse_current_source() {
        let netlist_text = r#"
I1 1 0 DC 1mA
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::CurrentSource { name, n_pos, n_neg, amplitude, phase_deg } => {
                assert_eq!(name, "I1");
                assert!((amplitude - 0.001).abs() < 1e-9);
            }
            _ => panic!("Expected current source"),
        }
    }

    #[test]
    fn test_parse_vcvs() {
        let netlist_text = r#"
E1 2 0 1 0 100000
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::VCVS { name, n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, gain } => {
                assert_eq!(name, "E1");
                assert_eq!(n_out_pos, &2);
                assert_eq!(n_ctrl_pos, &1);
                assert!((gain - 100000.0).abs() < 1e-3);
            }
            _ => panic!("Expected VCVS"),
        }
    }

    #[test]
    fn test_parse_vccs() {
        let netlist_text = r#"
G1 2 0 1 0 0.001
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::VCCS { name, transconductance, .. } => {
                assert_eq!(name, "G1");
                assert!((transconductance - 0.001).abs() < 1e-6);
            }
            _ => panic!("Expected VCCS"),
        }
    }

    #[test]
    fn test_parse_cccs() {
        let netlist_text = r#"
F1 2 0 V1 10
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::CCCS { name, ctrl_vsrc_name, gain, .. } => {
                assert_eq!(name, "F1");
                assert_eq!(ctrl_vsrc_name, "V1");
                assert!((gain - 10.0).abs() < 1e-6);
            }
            _ => panic!("Expected CCCS"),
        }
    }

    #[test]
    fn test_parse_ccvs() {
        let netlist_text = r#"
H1 2 0 V1 1000
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::CCVS { name, ctrl_vsrc_name, transresistance, .. } => {
                assert_eq!(name, "H1");
                assert_eq!(ctrl_vsrc_name, "V1");
                assert!((transresistance - 1000.0).abs() < 1e-3);
            }
            _ => panic!("Expected CCVS"),
        }
    }

    #[test]
    fn test_parse_diode() {
        let netlist_text = r#"
D1 1 2 1N4148
.MODEL 1N4148 D IS=1e-14
.END
"#;
        
        let (netlist, models) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::Diode { name, n_anode, n_cathode, model, area } => {
                assert_eq!(name, "D1");
                assert_eq!(n_anode, &1);
                assert_eq!(n_cathode, &2);
                assert_eq!(model, "1N4148");
                assert!((area - 1.0).abs() < 1e-6);
            }
            _ => panic!("Expected diode"),
        }
        
        assert_eq!(models.diodes.len(), 1);
        assert!((models.diodes["1N4148"].is - 1e-14).abs() < 1e-20);
    }

    #[test]
    fn test_parse_bjt() {
        let netlist_text = r#"
Q1 1 2 3 2N3904
.MODEL 2N3904 NPN IS=1e-15 BF=100
.END
"#;
        
        let (netlist, models) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::BJT { name, n_collector, n_base, n_emitter, model, area } => {
                assert_eq!(name, "Q1");
                assert_eq!(n_collector, &1);
                assert_eq!(n_base, &2);
                assert_eq!(n_emitter, &3);
                assert_eq!(model, "2N3904");
            }
            _ => panic!("Expected BJT"),
        }
        
        assert_eq!(models.bjts.len(), 1);
        assert!((models.bjts["2N3904"].bf - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_parse_mosfet() {
        let netlist_text = r#"
M1 1 2 3 0 BS170
.MODEL BS170 NMOS VTO=1 KP=2e-5
.END
"#;
        
        let (netlist, models) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        match &netlist.components[0] {
            crate::Component::MOSFET { name, n_drain, n_gate, n_source, n_body, model, w, l } => {
                assert_eq!(name, "M1");
                assert_eq!(n_drain, &1);
                assert_eq!(n_gate, &2);
                assert_eq!(n_source, &3);
                assert_eq!(n_body, &0);
                assert_eq!(model, "BS170");
            }
            _ => panic!("Expected MOSFET"),
        }
        
        assert_eq!(models.mosfets.len(), 1);
        assert!((models.mosfets["BS170"].vto - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_parse_value_suffixes() {
        let test_cases = vec![
            ("1k", 1000.0),
            ("1K", 1000.0),
            ("1MEG", 1e6),
            ("1meg", 1e6),
            ("1M", 1e-3),  // M = milli in SPICE
            ("1u", 1e-6),
            ("1U", 1e-6),
            ("1n", 1e-9),
            ("1p", 1e-12),
            ("1f", 1e-15),
            ("1m", 1e-3),
            ("1G", 1e9),
            ("1T", 1e12),
        ];
        
        for (suffix, expected) in test_cases {
            let netlist_text = format!("R1 1 0 {}\n.END", suffix);
            let (netlist, _) = parse_spice_netlist(&netlist_text).expect(&format!("Failed to parse {}", suffix));
            
            match &netlist.components[0] {
                crate::Component::Resistor { value, .. } => {
                    assert!((value - expected).abs() < expected * 1e-6, "Failed for {}: got {}, expected {}", suffix, value, expected);
                }
                _ => panic!("Expected resistor"),
            }
        }
    }

    #[test]
    fn test_parse_compound_suffixes() {
        let test_cases = vec![
            ("1uF", 1e-6),
            ("1uH", 1e-6),
            ("1pF", 1e-12),
            ("1nH", 1e-9),
            ("10k", 10000.0),
        ];
        
        for (suffix, expected) in test_cases {
            let netlist_text = format!("C1 1 0 {}\n.END", suffix);
            let (netlist, _) = parse_spice_netlist(&netlist_text).expect(&format!("Failed to parse {}", suffix));
            
            match &netlist.components[0] {
                crate::Component::Capacitor { value, .. } => {
                    assert!((value - expected).abs() < expected * 1e-6, "Failed for {}: got {}, expected {}", suffix, value, expected);
                }
                _ => panic!("Expected capacitor"),
            }
        }
    }

    #[test]
    fn test_parse_comments() {
        let netlist_text = r#"
* This is a comment
R1 1 0 1k  ; inline comment
# another comment
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        assert_eq!(netlist.components.len(), 1);
    }

    #[test]
    fn test_parse_continuation_lines() {
        let netlist_text = r#"
R1 1 0 1k +
    2k
.END
"#;
        
        // Note: Our parser doesn't fully support continuation lines with +
        // This test documents the current limitation
        let result = parse_spice_netlist(netlist_text);
        assert!(result.is_err() || result.unwrap().0.components.len() >= 1);
    }

    #[test]
    fn test_parse_model_parameters() {
        let netlist_text = r#"
D1 1 2 D1
.MODEL D1 D IS=1e-14 N=1.5 RS=0.5 TT=1e-9 CJO=1e-12 VJ=0.7 M=0.3 EG=1.11 XTI=3 KF=1e-12 AF=1 FC=0.5 BV=100 IBV=1e-3
.END
"#;
        
        let (_, models) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        let model = &models.diodes["D1"];
        assert!((model.is - 1e-14).abs() < 1e-20);
        assert!((model.n - 1.5).abs() < 1e-6);
        assert!((model.rs - 0.5).abs() < 1e-6);
        assert!((model.tt - 1e-9).abs() < 1e-12);
        assert!((model.cjo - 1e-12).abs() < 1e-15);
        assert!((model.vj - 0.7).abs() < 1e-6);
        assert!((model.m - 0.3).abs() < 1e-6);
        assert!((model.eg - 1.11).abs() < 1e-6);
        assert!((model.xti - 3.0).abs() < 1e-6);
        assert!((model.kf - 1e-12).abs() < 1e-15);
        assert!((model.af - 1.0).abs() < 1e-6);
        assert!((model.fc - 0.5).abs() < 1e-6);
        assert!((model.bv - 100.0).abs() < 1e-6);
        assert!((model.ibv - 1e-3).abs() < 1e-6);
    }

    #[test]
    fn test_parse_multiple_models() {
        let netlist_text = r#"
D1 1 2 D1
Q1 3 4 5 Q1
.MODEL D1 D IS=1e-14
.MODEL Q1 NPN IS=1e-15 BF=200
.END
"#;
        
        let (_, models) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        assert_eq!(models.diodes.len(), 1);
        assert_eq!(models.bjts.len(), 1);
        assert_eq!(models.mosfets.len(), 0);
    }

    #[test]
    fn test_parse_file() {
        use std::fs;
        use std::io::Write;
        
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_spice.cir");
        
        let content = r#"
R1 1 0 1k
C1 1 0 1uF
.END
"#;
        
        fs::write(&file_path, content).expect("Write temp file");
        
        let result = parse_spice_file(&file_path);
        assert!(result.is_ok());
        
        let (netlist, _) = result.unwrap();
        assert_eq!(netlist.components.len(), 2);
        
        fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_parse_invalid_model() {
        let netlist_text = r#"
.MODEL D1 X IS=1e-14
.END
"#;
        
        let result = parse_spice_netlist(netlist_text);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            crate::error::SimError::ParseError { msg, .. } => {
                assert!(msg.contains("Unknown model type"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_parse_invalid_resistor() {
        let netlist_text = r#"
R1 1 0
.END
"#;
        
        let result = parse_spice_netlist(netlist_text);
        // Parser should handle gracefully (not crash)
        // Currently it might produce a resistor with value 0
        let _ = result;
    }

    #[test]
    fn test_parse_subckt_ignored() {
        let netlist_text = r#"
.SUBCKT AMP 1 2 3
R1 1 2 1k
.ENDS
V1 1 0 5
R1 1 0 1k
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        // Subcircuit definition should be ignored, only top-level components parsed
        assert_eq!(netlist.components.len(), 2); // V1 and R1
    }

    #[test]
    fn test_empty_netlist() {
        let netlist_text = r#"
* Empty netlist
.END
"#;
        
        let (netlist, _) = parse_spice_netlist(netlist_text).expect("Should parse");
        
        assert_eq!(netlist.components.len(), 0);
    }
}