use audio_spice_circuit::{CircuitProject, CircuitSimulationConfig, CircuitSimulator, DcConfig, AcConfig, TranConfig, AnalysisType};
use std::path::Path;

fn main() {
    println!("=== Audio-Spice Circuit Mode ===\n");

    // Example 1: RC filter from SPICE netlist
    let rc_netlist = r#"
* RC Low-pass filter
V1 1 0 AC 1 0
R1 1 2 1k
C1 2 0 1uF
.END
"#;

    println!("Example 1: RC Low-pass Filter");
    println!("Netlist:\n{}", rc_netlist);

    let project = CircuitProject {
        name: "RC Filter".to_string(),
        netlist_text: rc_netlist.to_string(),
        analyses: vec![AnalysisType::AC],
    };

    let config = CircuitSimulationConfig {
        ac: Some(AcConfig { f_start: 10.0, f_stop: 100000.0, points_per_decade: 20 }),
        ..Default::default()
    };

    let sim = CircuitSimulator::new(project, config);
    match sim.run() {
        Ok(result) => {
            if let Some(ac) = result.ac {
                println!("\nAC Analysis Results:");
                println!("  Frequency points: {}", ac.frequencies.len());
                println!("  V(2) at key frequencies:");
                for (i, &f) in ac.frequencies.iter().enumerate() {
                    if f >= 100.0 && f <= 10000.0 && (i % 10 == 0 || f >= 9000.0) {
                        if let Some(v) = ac.node_voltages.get(&2).and_then(|v| v.get(i)) {
                            let mag = (v.re * v.re + v.im * v.im).sqrt();
                            let phase = v.im.atan2(v.re).to_degrees();
                            println!("    {:.1} Hz: {:.6} V, {:.2}°", f, mag, phase);
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Simulation error: {}", e),
    }

    // Example 2: Simple op-amp non-inverting amplifier
    let opamp_netlist = r#"
* Op-amp non-inverting amplifier (gain = 11)
V1 1 0 DC 1
E1 2 0 1 3 1000
R1 3 0 1k
R2 2 3 10k
.END
"#;

    println!("\n\nExample 2: Op-amp Non-inverting Amplifier (DC)");
    println!("Netlist:\n{}", opamp_netlist);

    let project = CircuitProject {
        name: "OpAmp Buffer".to_string(),
        netlist_text: opamp_netlist.to_string(),
        analyses: vec![AnalysisType::DC],
    };

    let config = CircuitSimulationConfig {
        dc: Some(DcConfig { max_iter: 100, tol: 1e-6 }),
        ..Default::default()
    };

    let sim = CircuitSimulator::new(project, config);
    match sim.run() {
        Ok(result) => {
            if let Some(dc) = result.dc {
                println!("\nDC Operating Point:");
                println!("  Converged: {}, Iterations: {}", dc.converged, dc.iterations);
                for (node, v) in &dc.node_voltages {
                    println!("  V({}) = {:.4} V", node, v);
                }
            }
        }
        Err(e) => eprintln!("Simulation error: {}", e),
    }

    // Example 3: RC circuit transient response
    let rc_tran_netlist = r#"
* RC circuit step response
V1 1 0 DC 5
R1 1 2 1k
C1 2 0 1uF
.END
"#;

    println!("\n\nExample 3: RC Circuit Step Response (Transient)");
    println!("Netlist:\n{}", rc_tran_netlist);

    let project = CircuitProject {
        name: "RC Step Response".to_string(),
        netlist_text: rc_tran_netlist.to_string(),
        analyses: vec![AnalysisType::TRAN],
    };

    let config = CircuitSimulationConfig {
        tran: Some(TranConfig { t_start: 0.0, t_stop: 5e-3, t_step: 10e-6, t_max: 10e-6 }),
        ..Default::default()
    };

    let sim = CircuitSimulator::new(project, config);
    match sim.run() {
        Ok(result) => {
            if let Some(tran) = result.tran {
                println!("\nTransient Analysis:");
                println!("  Time points: {}", tran.time_points.len());
                println!("  V(2) at key times:");
                let step = tran.time_points.len() / 10;
                for i in (0..tran.time_points.len()).step_by(step.max(1)) {
                    let t = tran.time_points[i];
                    if let Some(v) = tran.node_voltages.get(&2).and_then(|v| v.get(i)) {
                        println!("    {:.3} ms: {:.3} V", t * 1000.0, v);
                    }
                }
            }
        }
        Err(e) => eprintln!("Simulation error: {}", e),
    }

    // Example 4: BJT common emitter amplifier
    let bjt_netlist = r#"
* Common-emitter amplifier
VCC 1 0 DC 12
VIN 2 0 AC 0.01 0
RB 3 2 100k
RC 1 4 1k
Q1 4 3 0 NPN_MODEL
.MODEL NPN_MODEL NPN IS=1e-15 BF=100 VAF=100
.END
"#;

    println!("\n\nExample 4: BJT Common-Emitter Amplifier (AC)");
    println!("Netlist:\n{}", bjt_netlist);

    let project = CircuitProject {
        name: "BJT Amplifier".to_string(),
        netlist_text: bjt_netlist.to_string(),
        analyses: vec![AnalysisType::AC],
    };

    let config = CircuitSimulationConfig {
        ac: Some(AcConfig { f_start: 10.0, f_stop: 100000.0, points_per_decade: 10 }),
        ..Default::default()
    };

    let sim = CircuitSimulator::new(project, config);
    match sim.run() {
        Ok(result) => {
            if let Some(ac) = result.ac {
                println!("\nAC Analysis Results:");
                println!("  Frequency points: {}", ac.frequencies.len());
                println!("  V(4) / V(2) gain:");
                for (i, &f) in ac.frequencies.iter().enumerate() {
                    if f >= 100.0 && f <= 10000.0 && (i % 10 == 0 || f >= 9000.0) {
                        if let (Some(vout), Some(vin)) = (
                            ac.node_voltages.get(&4).and_then(|v| v.get(i)),
                            ac.node_voltages.get(&2).and_then(|v| v.get(i)),
                        ) {
                            let gain = ((vout.re * vout.re + vout.im * vout.im) / 
                                       (vin.re * vin.re + vin.im * vin.im)).sqrt();
                            let gain_db = 20.0 * gain.log10();
                            println!("    {:.1} Hz: {:.2} dB", f, gain_db);
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Simulation error: {}", e),
    }

    // Example 5: Load from file and export to SPICE
    println!("\n\nExample 5: SPICE Export");
    let netlist_text = r#"
* Test circuit for export
V1 1 0 DC 5
R1 1 2 1k
C1 2 0 1uF
.MODEL 1N4148 D IS=1e-14
.END
"#;
    
    let (netlist, models) = audio_spice_core::parse_spice_netlist(netlist_text).unwrap();
    let exported = audio_spice_circuit::spice_export::SpiceExport::export_netlist(&netlist, &models);
    println!("Exported SPICE netlist:\n{}", exported);
}