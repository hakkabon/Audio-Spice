// db-engine-ffi/src/lib.rs — UniFFI proc-macro bridge
// No UDL file; all types annotated directly with uniffi proc-macros.

use std::sync::{Arc, Mutex};

use audio_spice_circuit::{
    AcConfig, CircuitProject, CircuitSimulationConfig, CircuitSimulator, DcConfig, TranConfig,
};
use audio_spice_crossover::{
    CompensationNetwork as RsCompNet, CompensationType as RsCompType,
    ComponentType as RsCompKind, ComponentValue as RsCompVal,
    CrossoverDesign as RsCrossoverDesign, CrossoverSimulator as RsCrossoverSim,
    CrossoverSynthesizer, CrossoverType as RsTopology,
    Driver as RsDriver, DriverDatabase,
    DriverRole as RsDriverRole, FilterSpec as RsFilterSpec, FilterType as RsFilterType,
    OptimizationConfig as RsOptConfig, Optimizer, Polarity,
    SimulationConfig, StandardSeries as RsSeries, TargetResponse,
};

uniffi::setup_scaffolding!();

// ─── Error ─────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum EngineError {
    #[error("Singular matrix")]             SingularMatrix,
    #[error("No convergence")]              NoConvergence,
    #[error("Parse error: {0}")]            ParseError(String),
    #[error("Component not found: {0}")]    ComponentNotFound(String),
    #[error("Node out of bounds")]          NodeOutOfBounds,
    #[error("Model not found: {0}")]        ModelNotFound(String),
    #[error("Invalid value: {0}")]          InvalidValue(String),
    #[error("Internal: {0}")]               Internal(String),
}

impl From<audio_spice_core::SimError> for EngineError {
    fn from(e: audio_spice_core::SimError) -> Self {
        use audio_spice_core::SimError::*;
        match e {
            SingularMatrix { .. }               => EngineError::SingularMatrix,
            NoConvergence  { .. }               => EngineError::NoConvergence,
            ParseError     { msg, .. }          => EngineError::ParseError(msg),
            ComponentNotFound { name }          => EngineError::ComponentNotFound(name),
            NodeOutOfBounds   { .. }            => EngineError::NodeOutOfBounds,
            ModelNotFound     { name }          => EngineError::ModelNotFound(name),
            InvalidValue      { name, value }   => EngineError::InvalidValue(format!("{}: {}", name, value)),
            Internal(s) | TransientError { reason: s, .. } => EngineError::Internal(s),
        }
    }
}
impl From<String> for EngineError { fn from(s: String) -> Self { EngineError::Internal(s) } }

// ─── Primitive records ─────────────────────────────────────────────────────

#[derive(Debug, Clone, uniffi::Record)]
pub struct FreqPoint {
    pub freq_hz:      f64,
    pub magnitude_db: f64,
    pub phase_deg:    f64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct DriverSimResult {
    pub driver_id:         String,
    pub voltage_response:  Vec<FreqPoint>,
    pub acoustic_response: Vec<FreqPoint>,
    pub impedance:         Vec<FreqPoint>,
    pub power_w:           Vec<f64>,
}

// ─── Enums ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, uniffi::Enum)]
pub enum ComponentKind {
    Resistor, Capacitor, Inductor, ResistorSeries, CapacitorSeries, InductorSeries,
}
fn ck_r(k: &ComponentKind) -> RsCompKind {
    match k {
        ComponentKind::Resistor        => RsCompKind::Resistor,
        ComponentKind::Capacitor       => RsCompKind::Capacitor,
        ComponentKind::Inductor        => RsCompKind::Inductor,
        ComponentKind::ResistorSeries  => RsCompKind::ResistorSeries,
        ComponentKind::CapacitorSeries => RsCompKind::CapacitorSeries,
        ComponentKind::InductorSeries  => RsCompKind::InductorSeries,
    }
}
fn ck_f(ct: &RsCompKind) -> ComponentKind {
    match ct {
        RsCompKind::Resistor        => ComponentKind::Resistor,
        RsCompKind::Capacitor       => ComponentKind::Capacitor,
        RsCompKind::Inductor        => ComponentKind::Inductor,
        RsCompKind::ResistorSeries  => ComponentKind::ResistorSeries,
        RsCompKind::CapacitorSeries => ComponentKind::CapacitorSeries,
        RsCompKind::InductorSeries  => ComponentKind::InductorSeries,
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum CrossoverTopology {
    Butterworth1, Butterworth2, Butterworth3, Butterworth4,
    LinkwitzRiley2, LinkwitzRiley4, LinkwitzRiley6, LinkwitzRiley8,
    Bessel2, Bessel3, Bessel4, Custom,
}
fn tp_r(t: &CrossoverTopology) -> RsTopology {
    match t {
        CrossoverTopology::Butterworth1   => RsTopology::Butterworth1,
        CrossoverTopology::Butterworth2   => RsTopology::Butterworth2,
        CrossoverTopology::Butterworth3   => RsTopology::Butterworth3,
        CrossoverTopology::Butterworth4   => RsTopology::Butterworth4,
        CrossoverTopology::LinkwitzRiley2 => RsTopology::LinkwitzRiley2,
        CrossoverTopology::LinkwitzRiley4 => RsTopology::LinkwitzRiley4,
        CrossoverTopology::LinkwitzRiley6 => RsTopology::LinkwitzRiley6,
        CrossoverTopology::LinkwitzRiley8 => RsTopology::LinkwitzRiley8,
        CrossoverTopology::Bessel2        => RsTopology::Bessel2,
        CrossoverTopology::Bessel3        => RsTopology::Bessel3,
        CrossoverTopology::Bessel4        => RsTopology::Bessel4,
        CrossoverTopology::Custom         => RsTopology::Custom,
    }
}
fn tp_f(t: &RsTopology) -> CrossoverTopology {
    match t {
        RsTopology::Butterworth1   => CrossoverTopology::Butterworth1,
        RsTopology::Butterworth2   => CrossoverTopology::Butterworth2,
        RsTopology::Butterworth3   => CrossoverTopology::Butterworth3,
        RsTopology::Butterworth4   => CrossoverTopology::Butterworth4,
        RsTopology::LinkwitzRiley2 => CrossoverTopology::LinkwitzRiley2,
        RsTopology::LinkwitzRiley4 => CrossoverTopology::LinkwitzRiley4,
        RsTopology::LinkwitzRiley6 => CrossoverTopology::LinkwitzRiley6,
        RsTopology::LinkwitzRiley8 => CrossoverTopology::LinkwitzRiley8,
        RsTopology::Bessel2        => CrossoverTopology::Bessel2,
        RsTopology::Bessel3        => CrossoverTopology::Bessel3,
        RsTopology::Bessel4        => CrossoverTopology::Bessel4,
        RsTopology::Custom         => CrossoverTopology::Custom,
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum PassFilterType { LowPass, HighPass, BandPass, BandStop, AllPass }
fn ft_r(t: &PassFilterType) -> RsFilterType {
    match t {
        PassFilterType::LowPass  => RsFilterType::LowPass,
        PassFilterType::HighPass => RsFilterType::HighPass,
        PassFilterType::BandPass => RsFilterType::BandPass,
        PassFilterType::BandStop => RsFilterType::BandStop,
        PassFilterType::AllPass  => RsFilterType::AllPass,
    }
}
fn ft_f(t: &RsFilterType) -> PassFilterType {
    match t {
        RsFilterType::LowPass  => PassFilterType::LowPass,
        RsFilterType::HighPass => PassFilterType::HighPass,
        RsFilterType::BandPass => PassFilterType::BandPass,
        RsFilterType::BandStop => PassFilterType::BandStop,
        RsFilterType::AllPass  => PassFilterType::AllPass,
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum DriverRole { Woofer, Midrange, Tweeter, SuperTweeter, Subwoofer, FullRange }
fn dr_r(r: &DriverRole) -> RsDriverRole {
    match r {
        DriverRole::Woofer       => RsDriverRole::Woofer,
        DriverRole::Midrange     => RsDriverRole::Midrange,
        DriverRole::Tweeter      => RsDriverRole::Tweeter,
        DriverRole::SuperTweeter => RsDriverRole::SuperTweeter,
        DriverRole::Subwoofer    => RsDriverRole::Subwoofer,
        DriverRole::FullRange    => RsDriverRole::FullRange,
    }
}
fn dr_f(r: &RsDriverRole) -> DriverRole {
    match r {
        RsDriverRole::Woofer       => DriverRole::Woofer,
        RsDriverRole::Midrange     => DriverRole::Midrange,
        RsDriverRole::Tweeter      => DriverRole::Tweeter,
        RsDriverRole::SuperTweeter => DriverRole::SuperTweeter,
        RsDriverRole::Subwoofer    => DriverRole::Subwoofer,
        RsDriverRole::FullRange    => DriverRole::FullRange,
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum CompensationKind { Zobel, Conjugate, Notch, BaffleStep, LPad, ImpedanceCorrection }
fn ck2_r(k: &CompensationKind) -> RsCompType {
    match k {
        CompensationKind::Zobel               => RsCompType::Zobel,
        CompensationKind::Conjugate           => RsCompType::Conjugate,
        CompensationKind::Notch               => RsCompType::Notch,
        CompensationKind::BaffleStep          => RsCompType::BaffleStep,
        CompensationKind::LPad                => RsCompType::LPad,
        CompensationKind::ImpedanceCorrection => RsCompType::ImpedanceCorrection,
    }
}
fn ck2_f(k: &RsCompType) -> CompensationKind {
    match k {
        RsCompType::Zobel               => CompensationKind::Zobel,
        RsCompType::Conjugate           => CompensationKind::Conjugate,
        RsCompType::Notch               => CompensationKind::Notch,
        RsCompType::BaffleStep          => CompensationKind::BaffleStep,
        RsCompType::LPad                => CompensationKind::LPad,
        RsCompType::ImpedanceCorrection => CompensationKind::ImpedanceCorrection,
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum StandardSeries { E12, E24 }
fn ss_r(s: &StandardSeries) -> RsSeries {
    match s { StandardSeries::E12 => RsSeries::E12, StandardSeries::E24 => RsSeries::E24 }
}

// ─── Record types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiComponentValue {
    pub name:      String,
    pub kind:      ComponentKind,
    pub value:     f64,
    pub tolerance: Option<f64>,
}
fn cv_r(cv: &FfiComponentValue) -> RsCompVal {
    RsCompVal { name: cv.name.clone(), component_type: ck_r(&cv.kind), value: cv.value, tolerance: cv.tolerance }
}
fn cv_f(cv: &RsCompVal) -> FfiComponentValue {
    FfiComponentValue { name: cv.name.clone(), kind: ck_f(&cv.component_type), value: cv.value, tolerance: cv.tolerance }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiDriver {
    pub id: String, pub name: String, pub role: DriverRole,
    pub nominal_impedance: f64, pub re: f64, pub le: Option<f64>,
    pub sensitivity_db: f64, pub inverted_polarity: bool, pub acoustic_offset_mm: f64,
    pub power_handling_w: Option<f64>,
    pub fs: Option<f64>, pub qts: Option<f64>, pub qes: Option<f64>, pub qms: Option<f64>,
    pub vas: Option<f64>, pub bl: Option<f64>, pub mms: Option<f64>, pub sd: Option<f64>,
}
fn drv_r(d: &FfiDriver) -> RsDriver {
    let mut drv = RsDriver::new(d.id.clone(), d.name.clone(), dr_r(&d.role));
    drv.nominal_impedance = d.nominal_impedance; drv.re = d.re; drv.le = d.le;
    drv.sensitivity_db = d.sensitivity_db;
    drv.polarity = if d.inverted_polarity { Polarity::Inverted } else { Polarity::Normal };
    drv.acoustic_offset_mm = d.acoustic_offset_mm; drv.power_handling_w = d.power_handling_w;
    drv.fs = d.fs; drv.qts = d.qts; drv.qes = d.qes; drv.qms = d.qms;
    drv.vas = d.vas; drv.bl = d.bl; drv.mms = d.mms; drv.sd = d.sd;
    drv
}
fn drv_f(d: &RsDriver) -> FfiDriver {
    FfiDriver {
        id: d.id.clone(), name: d.name.clone(), role: dr_f(&d.role),
        nominal_impedance: d.nominal_impedance, re: d.re, le: d.le,
        sensitivity_db: d.sensitivity_db,
        inverted_polarity: matches!(d.polarity, Polarity::Inverted),
        acoustic_offset_mm: d.acoustic_offset_mm, power_handling_w: d.power_handling_w,
        fs: d.fs, qts: d.qts, qes: d.qes, qms: d.qms,
        vas: d.vas, bl: d.bl, mms: d.mms, sd: d.sd,
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiFilterSpec {
    pub filter_type: PassFilterType,
    pub topology:    CrossoverTopology,
    pub fc_hz:       f64,
    pub components:  Vec<FfiComponentValue>,
}
fn fspec_r(s: &FfiFilterSpec) -> RsFilterSpec {
    RsFilterSpec { filter_type: ft_r(&s.filter_type), crossover_type: tp_r(&s.topology), fc: s.fc_hz, components: s.components.iter().map(cv_r).collect() }
}
fn fspec_f(s: &RsFilterSpec) -> FfiFilterSpec {
    FfiFilterSpec { filter_type: ft_f(&s.filter_type), topology: tp_f(&s.crossover_type), fc_hz: s.fc, components: s.components.iter().map(cv_f).collect() }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiCompensationNetwork {
    pub kind:       CompensationKind,
    pub driver_id:  String,
    pub components: Vec<FfiComponentValue>,
}
fn cn_r(n: &FfiCompensationNetwork) -> RsCompNet {
    RsCompNet { comp_type: ck2_r(&n.kind), driver_id: n.driver_id.clone(), components: n.components.iter().map(cv_r).collect() }
}
fn cn_f(n: &RsCompNet) -> FfiCompensationNetwork {
    FfiCompensationNetwork { kind: ck2_f(&n.comp_type), driver_id: n.driver_id.clone(), components: n.components.iter().map(cv_f).collect() }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiCrossoverDesign {
    pub name:                    String,
    pub num_ways:                u32,
    pub crossover_frequencies:   Vec<f64>,
    pub topologies:              Vec<CrossoverTopology>,
    pub driver_ids:              Vec<String>,
    pub filters:                 Vec<FfiFilterSpec>,
    pub compensation:            Vec<FfiCompensationNetwork>,
}
fn des_r(d: &FfiCrossoverDesign) -> RsCrossoverDesign {
    RsCrossoverDesign {
        name: d.name.clone(), num_ways: d.num_ways as usize,
        crossover_frequencies: d.crossover_frequencies.clone(),
        crossover_types: d.topologies.iter().map(tp_r).collect(),
        drivers: d.driver_ids.clone(),
        filters: d.filters.iter().map(fspec_r).collect(),
        compensation: d.compensation.iter().map(cn_r).collect(),
        target: None,
    }
}
fn des_f(d: &RsCrossoverDesign) -> FfiCrossoverDesign {
    FfiCrossoverDesign {
        name: d.name.clone(), num_ways: d.num_ways as u32,
        crossover_frequencies: d.crossover_frequencies.clone(),
        topologies: d.crossover_types.iter().map(tp_f).collect(),
        driver_ids: d.drivers.clone(),
        filters: d.filters.iter().map(fspec_f).collect(),
        compensation: d.compensation.iter().map(cn_f).collect(),
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CrossoverSimResult {
    pub frequencies:          Vec<f64>,
    pub summed_acoustic:      Vec<FreqPoint>,
    pub system_impedance:     Vec<FreqPoint>,
    pub drivers:              Vec<DriverSimResult>,
    pub component_power_json: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiOptimizationConfig {
    pub max_iterations:    u32,
    pub population_size:   u32,
    pub mutation_rate:     f64,
    pub crossover_rate:    f64,
    pub tolerance:         f64,
    pub standardize_values:bool,
    pub series:            StandardSeries,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiOptimizationResult {
    pub best_design:   FfiCrossoverDesign,
    pub best_error:    f64,
    pub error_history: Vec<f64>,
    pub converged:     bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiDcResult {
    pub node_names:      Vec<String>,
    pub node_voltages:   Vec<f64>,
    pub source_names:    Vec<String>,
    pub source_currents: Vec<f64>,
    pub converged:       bool,
    pub iterations:      u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiAcResult {
    pub frequencies:     Vec<f64>,
    pub node_names:      Vec<String>,
    pub node_responses:  Vec<Vec<FreqPoint>>,
    pub source_names:    Vec<String>,
    pub source_responses:Vec<Vec<FreqPoint>>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiTranResult {
    pub time_points:   Vec<f64>,
    pub node_names:    Vec<String>,
    pub node_waveforms:Vec<Vec<f64>>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiCircuitResult {
    pub dc:   Option<FfiDcResult>,
    pub ac:   Option<FfiAcResult>,
    pub tran: Option<FfiTranResult>,
}

// ─── Free functions ────────────────────────────────────────────────────────

#[uniffi::export]
pub fn format_component_value(kind: ComponentKind, value: f64) -> String {
    match kind {
        ComponentKind::Resistor | ComponentKind::ResistorSeries =>
            if value >= 1000.0 { format!("{:.2}kΩ", value / 1000.0) } else { format!("{:.2}Ω", value) },
        ComponentKind::Capacitor | ComponentKind::CapacitorSeries =>
            if value >= 1e-3       { format!("{:.2}mF", value * 1e3) }
            else if value >= 1e-6  { format!("{:.2}µF", value * 1e6) }
            else if value >= 1e-9  { format!("{:.2}nF", value * 1e9) }
            else                   { format!("{:.2}pF", value * 1e12) },
        ComponentKind::Inductor | ComponentKind::InductorSeries =>
            if value >= 1.0       { format!("{:.2}H",  value) }
            else if value >= 1e-3 { format!("{:.2}mH", value * 1e3) }
            else                  { format!("{:.2}µH", value * 1e6) },
    }
}

fn cx_fp(freq: f64, cx: audio_spice_crossover::types::Cx) -> FreqPoint {
    let mag = (cx.re * cx.re + cx.im * cx.im).sqrt();
    FreqPoint { freq_hz: freq, magnitude_db: if mag > 1e-15 { 20.0 * mag.log10() } else { -200.0 }, phase_deg: cx.im.atan2(cx.re).to_degrees() }
}

// ─── CrossoverEngine ───────────────────────────────────────────────────────

#[derive(uniffi::Object)]
pub struct CrossoverEngine { drivers: Mutex<DriverDatabase> }

#[uniffi::export]
impl CrossoverEngine {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> { Arc::new(Self { drivers: Mutex::new(DriverDatabase::new()) }) }

    pub fn add_driver(&self, driver: FfiDriver) { self.drivers.lock().unwrap().add_driver(drv_r(&driver)); }
    pub fn remove_driver(&self, driver_id: String) { self.drivers.lock().unwrap().drivers.remove(&driver_id); }
    pub fn list_drivers(&self) -> Vec<FfiDriver> { self.drivers.lock().unwrap().drivers.values().map(drv_f).collect() }

    pub fn synthesize_filter(&self, filter_type: PassFilterType, topology: CrossoverTopology, fc_hz: f64, r_load: f64) -> Vec<FfiComponentValue> {
        CrossoverSynthesizer::synthesize_filter(ft_r(&filter_type), tp_r(&topology), fc_hz, r_load, "F").iter().map(cv_f).collect()
    }
    pub fn synthesize_zobel(&self, re: f64, le: f64) -> Vec<FfiComponentValue> {
        CrossoverSynthesizer::synthesize_zobel(re, le, "Zobel").iter().map(cv_f).collect()
    }
    pub fn synthesize_lpad(&self, r_load: f64, attenuation_db: f64) -> Vec<FfiComponentValue> {
        CrossoverSynthesizer::synthesize_lpad(r_load, attenuation_db, "LPad").iter().map(cv_f).collect()
    }
    pub fn synthesize_notch(&self, fc_hz: f64, q: f64, r_load: f64) -> Vec<FfiComponentValue> {
        CrossoverSynthesizer::synthesize_notch(fc_hz, q, r_load, "Notch").iter().map(cv_f).collect()
    }
    pub fn synthesize_baffle_step(&self, fc_hz: f64, r_load: f64, step_db: f64) -> Vec<FfiComponentValue> {
        CrossoverSynthesizer::synthesize_baffle_step(fc_hz, r_load, step_db, "BaffleStep").iter().map(cv_f).collect()
    }
    pub fn standardize_to_series(&self, components: Vec<FfiComponentValue>, series: StandardSeries) -> Vec<FfiComponentValue> {
        let mut rs: Vec<RsCompVal> = components.iter().map(cv_r).collect();
        CrossoverSynthesizer::standardize_components(&mut rs, ss_r(&series));
        rs.iter().map(cv_f).collect()
    }

    pub fn simulate(&self, design: FfiCrossoverDesign, f_min: f64, f_max: f64, points_per_decade: u32) -> Result<CrossoverSimResult, EngineError> {
        let rs = des_r(&design);
        let ids = rs.drivers.clone();
        let db = self.drivers.lock().unwrap().clone();
        let cfg = SimulationConfig { f_min, f_max, points_per_decade: points_per_decade as usize };
        let r = RsCrossoverSim::new(rs, db, cfg).simulate().map_err(EngineError::from)?;
        let zero = audio_spice_crossover::types::Cx::new(0.0, 0.0);
        let freqs = &r.frequencies;

        let drivers = ids.iter().map(|id| {
            let vr  = freqs.iter().enumerate().map(|(i, &f)| cx_fp(f, r.driver_voltages.get(id).and_then(|v| v.get(i)).copied().unwrap_or(zero))).collect();
            let ar  = freqs.iter().enumerate().map(|(i, &f)| cx_fp(f, r.driver_acoustic.get(id).and_then(|v| v.get(i)).copied().unwrap_or(zero))).collect();
            let imp = freqs.iter().enumerate().map(|(i, &f)| {
                let v  = r.driver_voltages.get(id).and_then(|v| v.get(i)).copied().unwrap_or(zero);
                let ic = r.driver_currents.get(id).and_then(|v| v.get(i)).copied().unwrap_or(zero);
                let d2 = ic.re * ic.re + ic.im * ic.im;
                let z  = if d2 > 1e-30 { audio_spice_crossover::types::Cx::new((v.re*ic.re + v.im*ic.im)/d2, (v.im*ic.re - v.re*ic.im)/d2) } else { zero };
                cx_fp(f, z)
            }).collect();
            let pw = freqs.iter().enumerate().map(|(i, _)| r.component_power.get(id).and_then(|v| v.get(i)).copied().unwrap_or(0.0)).collect();
            DriverSimResult { driver_id: id.clone(), voltage_response: vr, acoustic_response: ar, impedance: imp, power_w: pw }
        }).collect();

        let summed_acoustic  = freqs.iter().enumerate().map(|(i, &f)| cx_fp(f, r.summed_acoustic.get(i).copied().unwrap_or(zero))).collect();
        let system_impedance = freqs.iter().enumerate().map(|(i, &f)| cx_fp(f, r.system_impedance.get(i).copied().unwrap_or(zero))).collect();
        Ok(CrossoverSimResult { frequencies: r.frequencies, summed_acoustic, system_impedance, drivers, component_power_json: serde_json::to_string(&r.component_power).unwrap_or_default() })
    }

    pub fn optimize(&self, design: FfiCrossoverDesign, config: FfiOptimizationConfig) -> Result<FfiOptimizationResult, EngineError> {
        let rs = des_r(&design);
        let db = self.drivers.lock().unwrap().clone();
        let oc = RsOptConfig { max_iterations: config.max_iterations as usize, population_size: config.population_size as usize, mutation_rate: config.mutation_rate, crossover_rate: config.crossover_rate, tolerance: config.tolerance, standardize_values: config.standardize_values, series: ss_r(&config.series) };
        let res = Optimizer::new(oc, TargetResponse::flat(), SimulationConfig::default(), db).optimize(rs);
        Ok(FfiOptimizationResult { best_design: des_f(&res.best_design), best_error: res.best_error, error_history: res.error_history, converged: res.converged })
    }
}

// ─── CircuitEngine ─────────────────────────────────────────────────────────

#[derive(uniffi::Object)]
pub struct CircuitEngine {}

#[uniffi::export]
impl CircuitEngine {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> { Arc::new(Self {}) }

    pub fn run_spice(&self, netlist: String, run_dc: bool, run_ac: bool, ac_f_start: f64, ac_f_stop: f64, ac_points_per_decade: u32, run_tran: bool, tran_t_stop: f64, tran_t_step: f64) -> Result<FfiCircuitResult, EngineError> {
        let project = CircuitProject { name: "circuit".into(), netlist_text: netlist, analyses: vec![] };
        let config = CircuitSimulationConfig {
            dc:   if run_dc   { Some(DcConfig::default()) } else { None },
            ac:   if run_ac   { Some(AcConfig { f_start: ac_f_start, f_stop: ac_f_stop, points_per_decade: ac_points_per_decade as usize }) } else { None },
            tran: if run_tran { Some(TranConfig { t_start: 0.0, t_stop: tran_t_stop, t_step: tran_t_step, t_max: tran_t_step }) } else { None },
        };
        let result = CircuitSimulator::new(project, config).run().map_err(EngineError::from)?;

        let ffi_dc = result.dc.map(|r| {
            let (node_names, node_voltages)   = r.node_voltages.iter().map(|(k, v)| (k.to_string(), *v)).unzip::<_, _, Vec<_>, Vec<_>>();
            let (source_names, source_currents) = r.vsource_currents.iter().map(|(k, v)| (k.clone(), *v)).unzip::<_, _, Vec<_>, Vec<_>>();
            FfiDcResult { node_names, node_voltages, source_names, source_currents, converged: r.converged, iterations: r.iterations as u32 }
        });

        let ffi_ac = result.ac.map(|r| {
            let (node_names, node_responses): (Vec<_>, Vec<_>) = r.node_voltages.iter().map(|(k, v)| {
                let pts: Vec<FreqPoint> = r.frequencies.iter().zip(v.iter()).map(|(&f, cd)| {
                    let mag = (cd.re * cd.re + cd.im * cd.im).sqrt();
                    FreqPoint { freq_hz: f, magnitude_db: if mag > 1e-15 { 20.0 * mag.log10() } else { -200.0 }, phase_deg: cd.im.atan2(cd.re).to_degrees() }
                }).collect();
                (k.to_string(), pts)
            }).unzip();
            let (source_names, source_responses): (Vec<_>, Vec<_>) = r.vsource_currents.iter().map(|(k, v)| {
                let pts: Vec<FreqPoint> = r.frequencies.iter().zip(v.iter()).map(|(&f, cd)| {
                    let mag = (cd.re * cd.re + cd.im * cd.im).sqrt();
                    FreqPoint { freq_hz: f, magnitude_db: if mag > 1e-15 { 20.0 * mag.log10() } else { -200.0 }, phase_deg: cd.im.atan2(cd.re).to_degrees() }
                }).collect();
                (k.clone(), pts)
            }).unzip();
            FfiAcResult { frequencies: r.frequencies, node_names, node_responses, source_names, source_responses }
        });

        let ffi_tran = result.tran.map(|r| {
            let (node_names, node_waveforms): (Vec<_>, Vec<_>) = r.node_voltages.into_iter().map(|(k, v)| (k.to_string(), v)).unzip();
            FfiTranResult { time_points: r.time_points, node_names, node_waveforms }
        });

        Ok(FfiCircuitResult { dc: ffi_dc, ac: ffi_ac, tran: ffi_tran })
    }

    pub fn export_spice(&self, netlist_json: String) -> Result<String, EngineError> {
        if netlist_json.contains('\n') || netlist_json.trim_start().starts_with('*') { Ok(netlist_json) }
        else { Err(EngineError::ParseError("Unrecognised netlist format".into())) }
    }
}
