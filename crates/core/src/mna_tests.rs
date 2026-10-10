#[cfg(test)]
mod mna_tests {
    use crate::{Component, Netlist};
    use crate::mna::MnaSystem;
    use crate::types::Cx;

    #[test]
    fn test_mna_simple_resistor() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 });
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 0, value: 1000.0 });
        
        let mna = MnaSystem::build(&netlist, 2.0 * std::f64::consts::PI * 1000.0);
        
        // 1 node + 1 voltage source = 2x2 matrix
        assert_eq!(mna.matrix_size, 2);
        assert_eq!(mna.num_nodes, 1);
        assert_eq!(mna.vsources.len(), 1);
    }

    #[test]
    fn test_mna_rc_circuit() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 });
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 2, value: 1000.0 });
        netlist.add_component(Component::Capacitor { name: "C1".into(), n_pos: 2, n_neg: 0, value: 1e-6 });
        
        let omega = 2.0 * std::f64::consts::PI * 1000.0;
        let mna = MnaSystem::build(&netlist, omega);
        
        // 2 nodes + 1 voltage source = 3x3 matrix
        assert_eq!(mna.matrix_size, 3);
        assert_eq!(mna.num_nodes, 2);
        assert_eq!(mna.vsources.len(), 1);
    }

    #[test]
    fn test_mna_vccs() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VCCS { 
            name: "G1".into(), 
            n_out_pos: 1, 
            n_out_neg: 0, 
            n_ctrl_pos: 2, 
            n_ctrl_neg: 0, 
            transconductance: 0.001 
        });
        
        let mna = MnaSystem::build(&netlist, 1000.0);
        
        // 2 nodes, no voltage sources
        assert_eq!(mna.matrix_size, 2);
        assert_eq!(mna.num_nodes, 2);
        assert_eq!(mna.vsources.len(), 0);
        
        // Check that transconductance is in the matrix
        // G(1,2) = gm, G(2,1) = 0 (VCCS is not reciprocal)
        let gm = 0.001;
        assert!((mna.y_matrix[0][1].re - gm).abs() < 1e-10);
        assert!(mna.y_matrix[1][0].re.abs() < 1e-10);
    }

    #[test]
    fn test_mna_vcvs() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VCVS { 
            name: "E1".into(), 
            n_out_pos: 1, 
            n_out_neg: 0, 
            n_ctrl_pos: 2, 
            n_ctrl_neg: 0, 
            gain: 100000.0 
        });
        
        let mna = MnaSystem::build(&netlist, 1000.0);
        
        // 2 nodes + 1 VCVS (treated as voltage source) = 3x3
        assert_eq!(mna.matrix_size, 3);
        assert_eq!(mna.num_nodes, 2);
        assert_eq!(mna.vsources.len(), 1);
    }

    #[test]
    fn test_mna_cccs() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 3, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 });
        netlist.add_component(Component::CCCS { 
            name: "F1".into(), 
            n_out_pos: 1, 
            n_out_neg: 0, 
            ctrl_vsrc_name: "V1".into(), 
            gain: 10.0 
        });
        
        let mna = MnaSystem::build(&netlist, 1000.0);
        
        // 3 nodes + 1 voltage source = 4x4
        assert_eq!(mna.matrix_size, 4);
        assert_eq!(mna.num_nodes, 3);
        assert_eq!(mna.vsources.len(), 1);
    }

    #[test]
    fn test_mna_ccvs() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 3, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 });
        netlist.add_component(Component::CCVS { 
            name: "H1".into(), 
            n_out_pos: 1, 
            n_out_neg: 0, 
            ctrl_vsrc_name: "V1".into(), 
            transresistance: 1000.0 
        });
        
        let mna = MnaSystem::build(&netlist, 1000.0);
        
        // 3 nodes + 2 voltage sources (V1 and H1) = 5x5
        assert_eq!(mna.matrix_size, 5);
        assert_eq!(mna.num_nodes, 3);
        assert_eq!(mna.vsources.len(), 2);
    }

    #[test]
    fn test_mna_opamp() {
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
        
        let mna = MnaSystem::build(&netlist, 1000.0);
        
        // 5 nodes, no voltage sources
        assert_eq!(mna.matrix_size, 5);
        assert_eq!(mna.num_nodes, 5);
        assert_eq!(mna.vsources.len(), 0);
        
        // Check gain from in_pos to out
        assert!((mna.y_matrix[0][1].re - 100000.0).abs() < 1e-5);
        assert!((mna.y_matrix[0][2].re + 100000.0).abs() < 1e-5);
    }

    #[test]
    fn test_mna_diode_gmin() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::Diode { 
            name: "D1".into(), 
            n_anode: 1, 
            n_cathode: 2, 
            model: "1N4148".into(), 
            area: 1.0 
        });
        
        let mna = MnaSystem::build(&netlist, 1000.0);
        
        // GMIN should be added between anode and cathode (plus base GMIN on diagonal)
        let gmin = crate::types::GMIN;
        assert!((mna.y_matrix[0][0].re - 2.0 * gmin).abs() < 1e-15);
        assert!((mna.y_matrix[1][1].re - 2.0 * gmin).abs() < 1e-15);
        assert!((mna.y_matrix[0][1].re + gmin).abs() < 1e-15);
        assert!((mna.y_matrix[1][0].re + gmin).abs() < 1e-15);
    }

    #[test]
    fn test_mna_gmin_added_to_diagonal() {
        let mut netlist = Netlist::new();
        netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 0, value: 1000.0 });
        
        let mna = MnaSystem::build(&netlist, 1000.0);
        
        // Diagonal should have GMIN + 1/R
        let expected = crate::types::GMIN + 1.0/1000.0;
        assert!((mna.y_matrix[0][0].re - expected).abs() < 1e-10);
    }
}