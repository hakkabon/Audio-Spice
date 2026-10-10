#[cfg(test)]
mod model_tests {
    use crate::models::{DiodeModel, BjtModel, MosfetModel, ModelLibrary};

    #[test]
    fn test_diode_model_default() {
        let model = DiodeModel::default();
        
        assert_eq!(model.name, "D");
        assert!((model.is - 1e-14).abs() < 1e-20);
        assert!((model.n - 1.0).abs() < 1e-6);
        assert!((model.rs - 0.0).abs() < 1e-6);
        assert!((model.tt - 0.0).abs() < 1e-6);
        assert!((model.cjo - 0.0).abs() < 1e-6);
        assert!((model.vj - 1.0).abs() < 1e-6);
        assert!((model.m - 0.5).abs() < 1e-6);
        assert!((model.eg - 1.11).abs() < 1e-6);
        assert!((model.xti - 3.0).abs() < 1e-6);
        assert!((model.kf - 0.0).abs() < 1e-6);
        assert!((model.af - 1.0).abs() < 1e-6);
        assert!((model.fc - 0.5).abs() < 1e-6);
        assert!((model.bv - 1e99).abs() < 1e90);
        assert!((model.ibv - 1e-3).abs() < 1e-6);
    }

    #[test]
    fn test_bjt_model_default() {
        let model = BjtModel::default();
        
        assert_eq!(model.name, "Q");
        assert!((model.is - 1e-16).abs() < 1e-20);
        assert!((model.bf - 100.0).abs() < 1e-6);
        assert!((model.nf - 1.0).abs() < 1e-6);
        assert!((model.vaf - 100.0).abs() < 1e-6);
        assert!((model.br - 1.0).abs() < 1e-6);
        assert!((model.nr - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mosfet_model_default() {
        let model = MosfetModel::default();
        
        assert_eq!(model.name, "M");
        assert_eq!(model.level, 1);
        assert!((model.vto - 1.0).abs() < 1e-6);
        assert!((model.kp - 2e-5).abs() < 1e-9);
        assert!((model.gamma - 0.0).abs() < 1e-6);
        assert!((model.phi - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_model_library() {
        let mut lib = ModelLibrary::new();
        
        assert_eq!(lib.diodes.len(), 0);
        assert_eq!(lib.bjts.len(), 0);
        assert_eq!(lib.mosfets.len(), 0);
        
        let diode = DiodeModel { name: "1N4148".into(), ..Default::default() };
        lib.add_diode(diode);
        
        assert_eq!(lib.diodes.len(), 1);
        assert!(lib.get_diode("1N4148").is_some());
        assert!(lib.get_diode("1N4001").is_none());
        
        let bjt = BjtModel { name: "2N3904".into(), ..Default::default() };
        lib.add_bjt(bjt);
        
        assert_eq!(lib.bjts.len(), 1);
        assert!(lib.get_bjt("2N3904").is_some());
        
        let mosfet = MosfetModel { name: "BS170".into(), ..Default::default() };
        lib.add_mosfet(mosfet);
        
        assert_eq!(lib.mosfets.len(), 1);
        assert!(lib.get_mosfet("BS170").is_some());
    }

    #[test]
    fn test_model_library_overwrite() {
        let mut lib = ModelLibrary::new();
        
        let diode1 = DiodeModel { name: "D1".into(), is: 1e-14, ..Default::default() };
        lib.add_diode(diode1);
        
        let diode2 = DiodeModel { name: "D1".into(), is: 1e-15, ..Default::default() };
        lib.add_diode(diode2);
        
        assert_eq!(lib.diodes.len(), 1);
        assert!((lib.get_diode("D1").unwrap().is - 1e-15).abs() < 1e-20);
    }
}