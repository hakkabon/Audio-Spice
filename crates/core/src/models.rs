use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiodeModel {
    pub name: String,
    pub is: f64,
    pub n: f64,
    pub rs: f64,
    pub tt: f64,
    pub cjo: f64,
    pub vj: f64,
    pub m: f64,
    pub eg: f64,
    pub xti: f64,
    pub kf: f64,
    pub af: f64,
    pub fc: f64,
    pub bv: f64,
    pub ibv: f64,
}

impl Default for DiodeModel {
    fn default() -> Self {
        Self {
            name: "D".to_string(),
            is: 1e-14,
            n: 1.0,
            rs: 0.0,
            tt: 0.0,
            cjo: 0.0,
            vj: 1.0,
            m: 0.5,
            eg: 1.11,
            xti: 3.0,
            kf: 0.0,
            af: 1.0,
            fc: 0.5,
            bv: 1e99,
            ibv: 1e-3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BjtModel {
    pub name: String,
    pub is: f64,
    pub bf: f64,
    pub nf: f64,
    pub vaf: f64,
    pub ikf: f64,
    pub ise: f64,
    pub ne: f64,
    pub br: f64,
    pub nr: f64,
    pub var: f64,
    pub ikr: f64,
    pub isc: f64,
    pub nc: f64,
    pub rb: f64,
    pub irb: f64,
    pub rbm: f64,
    pub re: f64,
    pub rc: f64,
    pub cje: f64,
    pub vje: f64,
    pub mje: f64,
    pub cjc: f64,
    pub vjc: f64,
    pub mjc: f64,
    pub cjs: f64,
    pub vjs: f64,
    pub mjs: f64,
    pub tf: f64,
    pub xtf: f64,
    pub vtf: f64,
    pub itf: f64,
    pub tr: f64,
    pub fc: f64,
    pub xtb: f64,
}

impl Default for BjtModel {
    fn default() -> Self {
        Self {
            name: "Q".to_string(),
            is: 1e-16,
            bf: 100.0,
            nf: 1.0,
            vaf: 100.0,
            ikf: 0.0,
            ise: 0.0,
            ne: 1.5,
            br: 1.0,
            nr: 1.0,
            var: 100.0,
            ikr: 0.0,
            isc: 0.0,
            nc: 2.0,
            rb: 0.0,
            irb: 0.0,
            rbm: 0.0,
            re: 0.0,
            rc: 0.0,
            cje: 0.0,
            vje: 0.75,
            mje: 0.33,
            cjc: 0.0,
            vjc: 0.75,
            mjc: 0.33,
            cjs: 0.0,
            vjs: 0.75,
            mjs: 0.5,
            tf: 0.0,
            xtf: 0.0,
            vtf: 0.0,
            itf: 0.0,
            tr: 0.0,
            fc: 0.5,
            xtb: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MosfetModel {
    pub name: String,
    pub level: i32,
    pub vto: f64,
    pub kp: f64,
    pub gamma: f64,
    pub phi: f64,
    pub lambda: f64,
    pub rd: f64,
    pub rs: f64,
    pub cbd: f64,
    pub cbs: f64,
    pub is: f64,
    pub pb: f64,
    pub cgso: f64,
    pub cgdo: f64,
    pub cgbo: f64,
    pub rsh: f64,
    pub cj: f64,
    pub mj: f64,
    pub cjsw: f64,
    pub mjsw: f64,
    pub js: f64,
    pub tox: f64,
    pub ld: f64,
    pub u0: f64,
    pub uexp: f64,
    pub ucrit: f64,
    pub nsub: f64,
    pub tpg: i32,
    pub nfs: f64,
    pub tpq: f64,
    pub vmax: f64,
    pub xj: f64,
    pub kf: f64,
    pub af: f64,
    pub fc: f64,
    pub delta: f64,
    pub theta: f64,
    pub eta: f64,
    pub kappa: f64,
}

impl Default for MosfetModel {
    fn default() -> Self {
        Self {
            name: "M".to_string(),
            level: 1,
            vto: 1.0,
            kp: 2e-5,
            gamma: 0.0,
            phi: 0.6,
            lambda: 0.0,
            rd: 0.0,
            rs: 0.0,
            cbd: 0.0,
            cbs: 0.0,
            is: 1e-14,
            pb: 0.8,
            cgso: 0.0,
            cgdo: 0.0,
            cgbo: 0.0,
            rsh: 0.0,
            cj: 0.0,
            mj: 0.5,
            cjsw: 0.0,
            mjsw: 0.33,
            js: 0.0,
            tox: 1e-7,
            ld: 0.0,
            u0: 600.0,
            uexp: 0.0,
            ucrit: 0.0,
            nsub: 0.0,
            tpg: 1,
            nfs: 0.0,
            tpq: 0.0,
            vmax: 0.0,
            xj: 0.0,
            kf: 0.0,
            af: 1.0,
            fc: 0.5,
            delta: 0.0,
            theta: 0.0,
            eta: 0.0,
            kappa: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLibrary {
    pub diodes: HashMap<String, DiodeModel>,
    pub bjts: HashMap<String, BjtModel>,
    pub mosfets: HashMap<String, MosfetModel>,
}

impl ModelLibrary {
    pub fn new() -> Self {
        Self {
            diodes: HashMap::new(),
            bjts: HashMap::new(),
            mosfets: HashMap::new(),
        }
    }

    pub fn add_diode(&mut self, model: DiodeModel) {
        self.diodes.insert(model.name.clone(), model);
    }

    pub fn add_bjt(&mut self, model: BjtModel) {
        self.bjts.insert(model.name.clone(), model);
    }

    pub fn add_mosfet(&mut self, model: MosfetModel) {
        self.mosfets.insert(model.name.clone(), model);
    }

    pub fn get_diode(&self, name: &str) -> Option<&DiodeModel> {
        self.diodes.get(name)
    }

    pub fn get_bjt(&self, name: &str) -> Option<&BjtModel> {
        self.bjts.get(name)
    }

    pub fn get_mosfet(&self, name: &str) -> Option<&MosfetModel> {
        self.mosfets.get(name)
    }
}

impl Default for ModelLibrary {
    fn default() -> Self {
        Self::new()
    }
}