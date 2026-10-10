use audio_spice_core::{DcResult, AcResult, TranResult, parse_spice_netlist, SimError, SimResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitProject {
    pub name: String,
    pub netlist_text: String,
    pub analyses: Vec<AnalysisType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisType {
    DC,
    AC,
    TRAN,
    NOISE,
    SENS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitSimulationConfig {
    pub dc: Option<DcConfig>,
    pub ac: Option<AcConfig>,
    pub tran: Option<TranConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcConfig {
    pub max_iter: usize,
    pub tol: f64,
}

impl Default for DcConfig {
    fn default() -> Self {
        Self { max_iter: 100, tol: 1e-6 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcConfig {
    pub f_start: f64,
    pub f_stop: f64,
    pub points_per_decade: usize,
}

impl Default for AcConfig {
    fn default() -> Self {
        Self { f_start: 1.0, f_stop: 1e6, points_per_decade: 10 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranConfig {
    pub t_start: f64,
    pub t_stop: f64,
    pub t_step: f64,
    pub t_max: f64,
}

impl Default for TranConfig {
    fn default() -> Self {
        Self { t_start: 0.0, t_stop: 1e-3, t_step: 1e-6, t_max: 1e-6 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitSimulationResult {
    pub dc: Option<DcResultData>,
    pub ac: Option<AcResultData>,
    pub tran: Option<TranResultData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcResultData {
    pub node_voltages: HashMap<usize, f64>,
    pub vsource_currents: HashMap<String, f64>,
    pub converged: bool,
    pub iterations: usize,
}

impl From<DcResult> for DcResultData {
    fn from(r: DcResult) -> Self {
        Self {
            node_voltages: r.node_voltages,
            vsource_currents: r.vsource_currents,
            converged: r.converged,
            iterations: r.iterations,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcResultData {
    pub frequencies: Vec<f64>,
    pub node_voltages: HashMap<usize, Vec<ComplexData>>,
    pub vsource_currents: HashMap<String, Vec<ComplexData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexData {
    pub re: f64,
    pub im: f64,
}

impl From<AcResult> for AcResultData {
    fn from(r: AcResult) -> Self {
        Self {
            frequencies: r.frequencies,
            node_voltages: r.node_voltages.into_iter().map(|(k, v)| (k, v.into_iter().map(|c| ComplexData { re: c.re, im: c.im }).collect())).collect(),
            vsource_currents: r.vsource_currents.into_iter().map(|(k, v)| (k, v.into_iter().map(|c| ComplexData { re: c.re, im: c.im }).collect())).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranResultData {
    pub time_points: Vec<f64>,
    pub node_voltages: HashMap<usize, Vec<f64>>,
    pub vsource_currents: HashMap<String, Vec<f64>>,
}

impl From<TranResult> for TranResultData {
    fn from(r: TranResult) -> Self {
        Self {
            time_points: r.time_points,
            node_voltages: r.node_voltages,
            vsource_currents: r.vsource_currents,
        }
    }
}

pub struct CircuitSimulator {
    project: CircuitProject,
    config: CircuitSimulationConfig,
}

impl CircuitSimulator {
    pub fn new(project: CircuitProject, config: CircuitSimulationConfig) -> Self {
        Self { project, config }
    }

    pub fn run(&self) -> SimResult<CircuitSimulationResult> {
        let (netlist, _models) = parse_spice_netlist(&self.project.netlist_text)?;
        
        let mut dc_result = None;
        let mut ac_result = None;
        let mut tran_result = None;

        if let Some(dc_cfg) = &self.config.dc {
            dc_result = Some(audio_spice_core::simulate_dc(&netlist, dc_cfg.max_iter, dc_cfg.tol)?.into());
        }

        if let Some(ac_cfg) = &self.config.ac {
            let sweep = audio_spice_core::AcSweep {
                f_start: ac_cfg.f_start,
                f_stop: ac_cfg.f_stop,
                points_per_decade: ac_cfg.points_per_decade,
            };
            ac_result = Some(audio_spice_core::simulate_ac(&netlist, &sweep)?.into());
        }

        if let Some(tran_cfg) = &self.config.tran {
            let sweep = audio_spice_core::TranSweep {
                t_start: tran_cfg.t_start,
                t_stop: tran_cfg.t_stop,
                t_step: tran_cfg.t_step,
                t_max: tran_cfg.t_max,
            };
            tran_result = Some(audio_spice_core::simulate_tran(&netlist, &sweep)?.into());
        }

        Ok(CircuitSimulationResult { dc: dc_result, ac: ac_result, tran: tran_result })
    }

    pub fn load_from_file(path: &Path) -> SimResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SimError::ParseError { line: 0, msg: format!("Failed to read file: {}", e) })?;
        
        let project = CircuitProject {
            name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
            netlist_text: content,
            analyses: vec![AnalysisType::DC, AnalysisType::AC, AnalysisType::TRAN],
        };
        
        Ok(Self::new(project, CircuitSimulationConfig::default()))
    }
}

impl Default for CircuitSimulationConfig {
    fn default() -> Self {
        Self {
            dc: Some(DcConfig::default()),
            ac: Some(AcConfig::default()),
            tran: Some(TranConfig::default()),
        }
    }
}

pub mod spice_export {
    use audio_spice_core::{Netlist, ModelLibrary};

    pub struct SpiceExport;

    impl SpiceExport {
        pub fn export_netlist(netlist: &Netlist, models: &ModelLibrary) -> String {
            let mut output = String::new();
            output.push_str("* Audio-Spice Generated Netlist\n\n");
            
            for comp in &netlist.components {
                output.push_str(&Self::component_to_spice(comp));
                output.push('\n');
            }
            
            output.push('\n');
            for (name, model) in &models.diodes {
                output.push_str(&Self::diode_model_to_spice(name, model));
                output.push('\n');
            }
            for (name, model) in &models.bjts {
                output.push_str(&Self::bjt_model_to_spice(name, model));
                output.push('\n');
            }
            for (name, model) in &models.mosfets {
                output.push_str(&Self::mosfet_model_to_spice(name, model));
                output.push('\n');
            }
            
            output.push_str(".END\n");
            output
        }

        fn component_to_spice(comp: &audio_spice_core::Component) -> String {
            match comp {
                audio_spice_core::Component::Resistor { name, n_pos, n_neg, value } => {
                    format!("{} {} {} {}", name, n_pos, n_neg, value)
                }
                audio_spice_core::Component::Capacitor { name, n_pos, n_neg, value } => {
                    format!("{} {} {} {}", name, n_pos, n_neg, value)
                }
                audio_spice_core::Component::Inductor { name, n_pos, n_neg, value } => {
                    format!("{} {} {} {}", name, n_pos, n_neg, value)
                }
                audio_spice_core::Component::VoltageSource { name, n_pos, n_neg, amplitude, phase_deg: _ } => {
                    format!("{} {} {} {}", name, n_pos, n_neg, amplitude)
                }
                audio_spice_core::Component::CurrentSource { name, n_pos, n_neg, amplitude, phase_deg: _ } => {
                    format!("{} {} {} {}", name, n_pos, n_neg, amplitude)
                }
                audio_spice_core::Component::VCVS { name, n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, gain } => {
                    format!("{} {} {} {} {} {}", name, n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, gain)
                }
                audio_spice_core::Component::VCCS { name, n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, transconductance } => {
                    format!("{} {} {} {} {} {}", name, n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, transconductance)
                }
                audio_spice_core::Component::CCCS { name, n_out_pos, n_out_neg, ctrl_vsrc_name, gain } => {
                    format!("{} {} {} {} {}", name, n_out_pos, n_out_neg, ctrl_vsrc_name, gain)
                }
                audio_spice_core::Component::CCVS { name, n_out_pos, n_out_neg, ctrl_vsrc_name, transresistance } => {
                    format!("{} {} {} {} {}", name, n_out_pos, n_out_neg, ctrl_vsrc_name, transresistance)
                }
                audio_spice_core::Component::Diode { name, n_anode, n_cathode, model, area } => {
                    format!("{} {} {} {} {}", name, n_anode, n_cathode, model, area)
                }
                audio_spice_core::Component::BJT { name, n_collector, n_base, n_emitter, model, area } => {
                    format!("{} {} {} {} {} {}", name, n_collector, n_base, n_emitter, model, area)
                }
                audio_spice_core::Component::MOSFET { name, n_drain, n_gate, n_source, n_body, model, w, l } => {
                    format!("{} {} {} {} {} {} W={} L={}", name, n_drain, n_gate, n_source, n_body, model, w, l)
                }
                audio_spice_core::Component::OpAmp { name, n_out, n_in_pos, n_in_neg, n_vcc, n_vee, gain: _, gbw: _, slew_rate: _ } => {
                    format!("{} {} {} {} {} {} OPAMP", name, n_out, n_in_pos, n_in_neg, n_vcc, n_vee)
                }
            }
        }

        fn diode_model_to_spice(name: &str, model: &audio_spice_core::DiodeModel) -> String {
            format!(".MODEL {} D (IS={} N={} RS={} TT={} CJO={} VJ={} M={} EG={} XTI={} KF={} AF={} FC={} BV={} IBV={})",
                name, model.is, model.n, model.rs, model.tt, model.cjo, model.vj, model.m,
                model.eg, model.xti, model.kf, model.af, model.fc, model.bv, model.ibv)
        }

        fn bjt_model_to_spice(name: &str, model: &audio_spice_core::BjtModel) -> String {
            format!(".MODEL {} NPN (IS={} BF={} NF={} VAF={} IKF={} ISE={} NE={} BR={} NR={} VAR={} IKR={} ISC={} NC={} RB={} IRB={} RBM={} RE={} RC={} CJE={} VJE={} MJE={} CJC={} VJC={} MJC={} CJS={} VJS={} MJS={} TF={} XTF={} VTF={} ITF={} TR={} FC={} XTB={})",
                name, model.is, model.bf, model.nf, model.vaf, model.ikf, model.ise, model.ne,
                model.br, model.nr, model.var, model.ikr, model.isc, model.nc, model.rb,
                model.irb, model.rbm, model.re, model.rc, model.cje, model.vje, model.mje,
                model.cjc, model.vjc, model.mjc, model.cjs, model.vjs, model.mjs, model.tf,
                model.xtf, model.vtf, model.itf, model.tr, model.fc, model.xtb)
        }

        fn mosfet_model_to_spice(name: &str, model: &audio_spice_core::MosfetModel) -> String {
            format!(".MODEL {} NMOS (LEVEL={} VTO={} KP={} GAMMA={} PHI={} LAMBDA={} RD={} RS={} CBD={} CBS={} IS={} PB={} CGSO={} CGDO={} CGBO={} RSH={} CJ={} MJ={} CJSW={} MJSW={} JS={} TOX={} LD={} U0={} UEXP={} UCRIT={} NSUB={} TPG={} NFS={} TPQ={} VMAX={} XJ={} KF={} AF={} FC={} DELTA={} THETA={} ETA={} KAPPA={})",
                name, model.level, model.vto, model.kp, model.gamma, model.phi, model.lambda,
                model.rd, model.rs, model.cbd, model.cbs, model.is, model.pb, model.cgso,
                model.cgdo, model.cgbo, model.rsh, model.cj, model.mj, model.cjsw, model.mjsw,
                model.js, model.tox, model.ld, model.u0, model.uexp, model.ucrit, model.nsub,
                model.tpg, model.nfs, model.tpq, model.vmax, model.xj, model.kf, model.af,
                model.fc, model.delta, model.theta, model.eta, model.kappa)
        }
    }
}