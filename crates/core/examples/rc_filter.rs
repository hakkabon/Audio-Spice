use audio_spice_core::{Component, Netlist, AcSweep, engine::simulate_ac};

fn main() {
    let mut netlist = Netlist::new();
    
    netlist.add_component(Component::VoltageSource { name: "V1".to_string(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 });
    netlist.add_component(Component::Resistor { name: "R1".to_string(), n_pos: 1, n_neg: 2, value: 1000.0 });
    netlist.add_component(Component::Capacitor { name: "C1".to_string(), n_pos: 2, n_neg: 0, value: 1e-6 });
    
    let sweep = AcSweep { f_start: 10.0, f_stop: 10000.0, points_per_decade: 10 };
    let result = simulate_ac(&netlist, &sweep).unwrap();
    
    println!("Frequency (Hz) | V_out (Mag) | V_out (Phase)");
    for (i, &f) in result.frequencies.iter().enumerate() {
        let v_out = result.node_voltages.get(&2).unwrap()[i];
        println!("{:14.2} | {:11.6} | {:11.2} deg", f, v_out.norm(), v_out.arg().to_degrees());
    }
}
