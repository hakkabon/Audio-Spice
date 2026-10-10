use crate::{Component, Netlist};
use crate::models::{DiodeModel, BjtModel, MosfetModel, ModelLibrary};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn parse_spice_netlist(content: &str) -> Result<(Netlist, ModelLibrary), String> {
    let mut netlist = Netlist::new();
    let mut models = ModelLibrary::new();
    let mut subckt_stack = Vec::new();
    
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('*') || line.starts_with('#') {
            continue;
        }
        
        // Handle continuation lines
        let mut full_line = line.to_string();
        while full_line.ends_with('+') {
            full_line.pop();
            full_line = full_line.trim().to_string();
        }
        
        let tokens: Vec<&str> = full_line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        
        let keyword = tokens[0].to_uppercase();
        
        match keyword.as_str() {
            ".MODEL" => {
                if tokens.len() < 3 {
                    return Err(format!("Line {}: Invalid .MODEL syntax", line_num + 1));
                }
                let model_name = tokens[1].to_string();
                let model_type = tokens[2].to_uppercase();
                
                let mut params = HashMap::new();
                for token in &tokens[3..] {
                    if let Some(eq_pos) = token.find('=') {
                        let key = token[..eq_pos].to_uppercase();
                        let val = token[eq_pos + 1..].parse::<f64>().unwrap_or(0.0);
                        params.insert(key, val);
                    }
                }
                
                match model_type.as_str() {
                    "D" => {
                        let mut model = DiodeModel::default();
                        model.name = model_name.clone();
                        apply_diode_params(&mut model, &params);
                        models.add_diode(model);
                    }
                    "NPN" | "PNP" => {
                        let mut model = BjtModel::default();
                        model.name = model_name.clone();
                        apply_bjt_params(&mut model, &params);
                        models.add_bjt(model);
                    }
                    "NMOS" | "PMOS" => {
                        let mut model = MosfetModel::default();
                        model.name = model_name.clone();
                        apply_mosfet_params(&mut model, &params);
                        models.add_mosfet(model);
                    }
                    _ => return Err(format!("Line {}: Unknown model type {}", line_num + 1, model_type)),
                }
            }
            ".SUBCKT" => {
                subckt_stack.push(tokens[1].to_string());
            }
            ".ENDS" => {
                subckt_stack.pop();
            }
            ".END" => break,
            _ => {
                if !subckt_stack.is_empty() {
                    continue;
                }
                
                let comp_type = keyword.chars().next().unwrap();
                match comp_type {
                    'R' => {
                        if tokens.len() >= 4 {
                            let name = tokens[0].to_string();
                            let n_pos = tokens[1].parse().unwrap_or(0);
                            let n_neg = tokens[2].parse().unwrap_or(0);
                            let value = parse_value(tokens[3])?;
                            netlist.add_component(Component::Resistor { name, n_pos, n_neg, value });
                        }
                    }
                    'C' => {
                        if tokens.len() >= 4 {
                            let name = tokens[0].to_string();
                            let n_pos = tokens[1].parse().unwrap_or(0);
                            let n_neg = tokens[2].parse().unwrap_or(0);
                            let value = parse_value(tokens[3])?;
                            netlist.add_component(Component::Capacitor { name, n_pos, n_neg, value });
                        }
                    }
                    'L' => {
                        if tokens.len() >= 4 {
                            let name = tokens[0].to_string();
                            let n_pos = tokens[1].parse().unwrap_or(0);
                            let n_neg = tokens[2].parse().unwrap_or(0);
                            let value = parse_value(tokens[3])?;
                            netlist.add_component(Component::Inductor { name, n_pos, n_neg, value });
                        }
                    }
                    'V' => {
                        if tokens.len() >= 4 {
                            let name = tokens[0].to_string();
                            let n_pos = tokens[1].parse().unwrap_or(0);
                            let n_neg = tokens[2].parse().unwrap_or(0);
                            
                            // Parse source specification: DC, AC, SIN, PULSE, etc.
                            let mut amplitude = 0.0;
                            let mut phase_deg = 0.0;
                            
                            for i in 3..tokens.len() {
                                let tok = tokens[i].to_uppercase();
                                if tok == "DC" && i + 1 < tokens.len() {
                                    amplitude = parse_value(tokens[i + 1])?;
                                } else if tok == "AC" && i + 1 < tokens.len() {
                                    amplitude = parse_value(tokens[i + 1])?;
                                    if i + 2 < tokens.len() {
                                        phase_deg = tokens[i + 2].parse().unwrap_or(0.0);
                                    }
                                }
                            }
                            
                            // If no DC/AC found, try parsing as direct value
                            if amplitude == 0.0 {
                                amplitude = parse_value(tokens[3])?;
                                if tokens.len() > 4 {
                                    phase_deg = tokens[4].parse().unwrap_or(0.0);
                                }
                            }
                            
                            netlist.add_component(Component::VoltageSource { name, n_pos, n_neg, amplitude, phase_deg });
                        }
                    }
                    'I' => {
                        if tokens.len() >= 4 {
                            let name = tokens[0].to_string();
                            let n_pos = tokens[1].parse().unwrap_or(0);
                            let n_neg = tokens[2].parse().unwrap_or(0);
                            
                            let mut amplitude = 0.0;
                            let mut phase_deg = 0.0;
                            
                            for i in 3..tokens.len() {
                                let tok = tokens[i].to_uppercase();
                                if tok == "DC" && i + 1 < tokens.len() {
                                    amplitude = parse_value(tokens[i + 1])?;
                                } else if tok == "AC" && i + 1 < tokens.len() {
                                    amplitude = parse_value(tokens[i + 1])?;
                                    if i + 2 < tokens.len() {
                                        phase_deg = tokens[i + 2].parse().unwrap_or(0.0);
                                    }
                                }
                            }
                            
                            if amplitude == 0.0 {
                                amplitude = parse_value(tokens[3])?;
                                if tokens.len() > 4 {
                                    phase_deg = tokens[4].parse().unwrap_or(0.0);
                                }
                            }
                            
                            netlist.add_component(Component::CurrentSource { name, n_pos, n_neg, amplitude, phase_deg });
                        }
                    }
                    'E' => {
                        if tokens.len() >= 6 {
                            let name = tokens[0].to_string();
                            let n_out_pos = tokens[1].parse().unwrap_or(0);
                            let n_out_neg = tokens[2].parse().unwrap_or(0);
                            let n_ctrl_pos = tokens[3].parse().unwrap_or(0);
                            let n_ctrl_neg = tokens[4].parse().unwrap_or(0);
                            let gain = parse_value(tokens[5])?;
                            netlist.add_component(Component::VCVS { name, n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, gain });
                        }
                    }
                    'G' => {
                        if tokens.len() >= 6 {
                            let name = tokens[0].to_string();
                            let n_out_pos = tokens[1].parse().unwrap_or(0);
                            let n_out_neg = tokens[2].parse().unwrap_or(0);
                            let n_ctrl_pos = tokens[3].parse().unwrap_or(0);
                            let n_ctrl_neg = tokens[4].parse().unwrap_or(0);
                            let transconductance = parse_value(tokens[5])?;
                            netlist.add_component(Component::VCCS { name, n_out_pos, n_out_neg, n_ctrl_pos, n_ctrl_neg, transconductance });
                        }
                    }
                    'F' => {
                        if tokens.len() >= 5 {
                            let name = tokens[0].to_string();
                            let n_out_pos = tokens[1].parse().unwrap_or(0);
                            let n_out_neg = tokens[2].parse().unwrap_or(0);
                            let ctrl_vsrc_name = tokens[3].to_string();
                            let gain = parse_value(tokens[4])?;
                            netlist.add_component(Component::CCCS { name, n_out_pos, n_out_neg, ctrl_vsrc_name, gain });
                        }
                    }
                    'H' => {
                        if tokens.len() >= 5 {
                            let name = tokens[0].to_string();
                            let n_out_pos = tokens[1].parse().unwrap_or(0);
                            let n_out_neg = tokens[2].parse().unwrap_or(0);
                            let ctrl_vsrc_name = tokens[3].to_string();
                            let transresistance = parse_value(tokens[4])?;
                            netlist.add_component(Component::CCVS { name, n_out_pos, n_out_neg, ctrl_vsrc_name, transresistance });
                        }
                    }
                    'D' => {
                        if tokens.len() >= 4 {
                            let name = tokens[0].to_string();
                            let n_anode = tokens[1].parse().unwrap_or(0);
                            let n_cathode = tokens[2].parse().unwrap_or(0);
                            let model = tokens[3].to_string();
                            let area = if tokens.len() > 4 { tokens[4].parse().unwrap_or(1.0) } else { 1.0 };
                            netlist.add_component(Component::Diode { name, n_anode, n_cathode, model, area });
                        }
                    }
                    'Q' => {
                        if tokens.len() >= 5 {
                            let name = tokens[0].to_string();
                            let n_collector = tokens[1].parse().unwrap_or(0);
                            let n_base = tokens[2].parse().unwrap_or(0);
                            let n_emitter = tokens[3].parse().unwrap_or(0);
                            let model = tokens[4].to_string();
                            let area = if tokens.len() > 5 { tokens[5].parse().unwrap_or(1.0) } else { 1.0 };
                            netlist.add_component(Component::BJT { name, n_collector, n_base, n_emitter, model, area });
                        }
                    }
                    'M' => {
                        if tokens.len() >= 6 {
                            let name = tokens[0].to_string();
                            let n_drain = tokens[1].parse().unwrap_or(0);
                            let n_gate = tokens[2].parse().unwrap_or(0);
                            let n_source = tokens[3].parse().unwrap_or(0);
                            let n_body = tokens[4].parse().unwrap_or(0);
                            let model = tokens[5].to_string();
                            let w = if tokens.len() > 6 { tokens[6].parse().unwrap_or(1e-6) } else { 1e-6 };
                            let l = if tokens.len() > 7 { tokens[7].parse().unwrap_or(1e-6) } else { 1e-6 };
                            netlist.add_component(Component::MOSFET { name, n_drain, n_gate, n_source, n_body, model, w, l });
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    Ok((netlist, models))
}

fn parse_value(s: &str) -> Result<f64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty value".to_string());
    }
    
    let mut mult = 1.0;
    let mut num_str = s.to_string();
    let upper_str = num_str.to_uppercase();
    
    // Handle compound suffixes first (e.g., uF, pF, nH, uH, mH)
    if upper_str.ends_with("UF") || upper_str.ends_with("µF") {
        mult = 1e-6;
        num_str = num_str[..num_str.len()-2].to_string();
    } else if upper_str.ends_with("PF") {
        mult = 1e-12;
        num_str = num_str[..num_str.len()-2].to_string();
    } else if upper_str.ends_with("NF") {
        mult = 1e-9;
        num_str = num_str[..num_str.len()-2].to_string();
    } else if upper_str.ends_with("MH") {
        mult = 1e-3;
        num_str = num_str[..num_str.len()-2].to_string();
    } else if upper_str.ends_with("UH") || upper_str.ends_with("µH") {
        mult = 1e-6;
        num_str = num_str[..num_str.len()-2].to_string();
    } else if upper_str.ends_with("NH") {
        mult = 1e-9;
        num_str = num_str[..num_str.len()-2].to_string();
    } else if upper_str.ends_with("K") || upper_str.ends_with("KOHM") || upper_str.ends_with("KΩ") {
        mult = 1e3;
        if upper_str.ends_with("KOHM") || upper_str.ends_with("KΩ") {
            num_str = num_str[..num_str.len()-4].to_string();
        } else {
            num_str = num_str[..num_str.len()-1].to_string();
        }
    } else if upper_str.ends_with("MEG") || upper_str.ends_with("MEGOHM") || upper_str.ends_with("MΩ") {
        mult = 1e6;
        if upper_str.ends_with("MEGOHM") {
            num_str = num_str[..num_str.len()-6].to_string();
        } else if upper_str.ends_with("MΩ") {
            num_str = num_str[..num_str.len()-2].to_string();
        } else {
            num_str = num_str[..num_str.len()-3].to_string();
        }
    } else if upper_str.ends_with("OHM") || upper_str.ends_with("Ω") {
        mult = 1.0;
        if upper_str.ends_with("OHM") {
            num_str = num_str[..num_str.len()-3].to_string();
        } else {
            num_str = num_str[..num_str.len()-1].to_string();
        }
    } else if upper_str.ends_with("V") {
        mult = 1.0;
        num_str = num_str[..num_str.len()-1].to_string();
    } else if upper_str.ends_with("A") {
        mult = 1.0;
        num_str = num_str[..num_str.len()-1].to_string();
    } else if upper_str.ends_with("HZ") {
        mult = 1.0;
        num_str = num_str[..num_str.len()-2].to_string();
    } else if upper_str.ends_with("DB") {
        mult = 1.0;
        num_str = num_str[..num_str.len()-2].to_string();
    } else if upper_str.ends_with("S") {
        mult = 1.0;
        num_str = num_str[..num_str.len()-1].to_string();
    } else {
        // Handle single-character suffixes
        if upper_str.ends_with("MEG") {
            mult = 1e6;
            num_str = num_str[..num_str.len()-3].to_string();
        } else if let Some(last) = num_str.chars().last() {
            match last.to_ascii_uppercase() {
                'T' => { mult = 1e12; }
                'G' => { mult = 1e9; }
                'M' => { mult = 1e-3; }
                'K' => { mult = 1e3; }
                'U' | 'µ' => { mult = 1e-6; }
                'N' => { mult = 1e-9; }
                'P' => { mult = 1e-12; }
                'F' => { mult = 1e-15; }
                _ => {}
            }
        }
    }
    
    // Remove any trailing unit letters (V, A, H, F, etc.)
    while let Some(last) = num_str.chars().last() {
        if last.is_ascii_alphabetic() {
            num_str = num_str[..num_str.len()-1].to_string();
        } else {
            break;
        }
    }
    
    num_str.parse::<f64>().map(|v| v * mult)
        .map_err(|_| format!("Invalid value: {}", s))
}

fn apply_diode_params(model: &mut DiodeModel, params: &HashMap<String, f64>) {
    if let Some(v) = params.get("IS") { model.is = *v; }
    if let Some(v) = params.get("N") { model.n = *v; }
    if let Some(v) = params.get("RS") { model.rs = *v; }
    if let Some(v) = params.get("TT") { model.tt = *v; }
    if let Some(v) = params.get("CJO") { model.cjo = *v; }
    if let Some(v) = params.get("VJ") { model.vj = *v; }
    if let Some(v) = params.get("M") { model.m = *v; }
    if let Some(v) = params.get("EG") { model.eg = *v; }
    if let Some(v) = params.get("XTI") { model.xti = *v; }
    if let Some(v) = params.get("KF") { model.kf = *v; }
    if let Some(v) = params.get("AF") { model.af = *v; }
    if let Some(v) = params.get("FC") { model.fc = *v; }
    if let Some(v) = params.get("BV") { model.bv = *v; }
    if let Some(v) = params.get("IBV") { model.ibv = *v; }
}

fn apply_bjt_params(model: &mut BjtModel, params: &HashMap<String, f64>) {
    if let Some(v) = params.get("IS") { model.is = *v; }
    if let Some(v) = params.get("BF") { model.bf = *v; }
    if let Some(v) = params.get("NF") { model.nf = *v; }
    if let Some(v) = params.get("VAF") { model.vaf = *v; }
    if let Some(v) = params.get("IKF") { model.ikf = *v; }
    if let Some(v) = params.get("ISE") { model.ise = *v; }
    if let Some(v) = params.get("NE") { model.ne = *v; }
    if let Some(v) = params.get("BR") { model.br = *v; }
    if let Some(v) = params.get("NR") { model.nr = *v; }
    if let Some(v) = params.get("VAR") { model.var = *v; }
    if let Some(v) = params.get("IKR") { model.ikr = *v; }
    if let Some(v) = params.get("ISC") { model.isc = *v; }
    if let Some(v) = params.get("NC") { model.nc = *v; }
    if let Some(v) = params.get("RB") { model.rb = *v; }
    if let Some(v) = params.get("IRB") { model.irb = *v; }
    if let Some(v) = params.get("RBM") { model.rbm = *v; }
    if let Some(v) = params.get("RE") { model.re = *v; }
    if let Some(v) = params.get("RC") { model.rc = *v; }
    if let Some(v) = params.get("CJE") { model.cje = *v; }
    if let Some(v) = params.get("VJE") { model.vje = *v; }
    if let Some(v) = params.get("MJE") { model.mje = *v; }
    if let Some(v) = params.get("CJC") { model.cjc = *v; }
    if let Some(v) = params.get("VJC") { model.vjc = *v; }
    if let Some(v) = params.get("MJC") { model.mjc = *v; }
    if let Some(v) = params.get("CJS") { model.cjs = *v; }
    if let Some(v) = params.get("VJS") { model.vjs = *v; }
    if let Some(v) = params.get("MJS") { model.mjs = *v; }
    if let Some(v) = params.get("TF") { model.tf = *v; }
    if let Some(v) = params.get("XTF") { model.xtf = *v; }
    if let Some(v) = params.get("VTF") { model.vtf = *v; }
    if let Some(v) = params.get("ITF") { model.itf = *v; }
    if let Some(v) = params.get("TR") { model.tr = *v; }
    if let Some(v) = params.get("FC") { model.fc = *v; }
    if let Some(v) = params.get("XTB") { model.xtb = *v; }
}

fn apply_mosfet_params(model: &mut MosfetModel, params: &HashMap<String, f64>) {
    if let Some(v) = params.get("LEVEL") { model.level = *v as i32; }
    if let Some(v) = params.get("VTO") { model.vto = *v; }
    if let Some(v) = params.get("KP") { model.kp = *v; }
    if let Some(v) = params.get("GAMMA") { model.gamma = *v; }
    if let Some(v) = params.get("PHI") { model.phi = *v; }
    if let Some(v) = params.get("LAMBDA") { model.lambda = *v; }
    if let Some(v) = params.get("RD") { model.rd = *v; }
    if let Some(v) = params.get("RS") { model.rs = *v; }
    if let Some(v) = params.get("CBD") { model.cbd = *v; }
    if let Some(v) = params.get("CBS") { model.cbs = *v; }
    if let Some(v) = params.get("IS") { model.is = *v; }
    if let Some(v) = params.get("PB") { model.pb = *v; }
    if let Some(v) = params.get("CGSO") { model.cgso = *v; }
    if let Some(v) = params.get("CGDO") { model.cgdo = *v; }
    if let Some(v) = params.get("CGBO") { model.cgbo = *v; }
    if let Some(v) = params.get("RSH") { model.rsh = *v; }
    if let Some(v) = params.get("CJ") { model.cj = *v; }
    if let Some(v) = params.get("MJ") { model.mj = *v; }
    if let Some(v) = params.get("CJSW") { model.cjsw = *v; }
    if let Some(v) = params.get("MJSW") { model.mjsw = *v; }
    if let Some(v) = params.get("JS") { model.js = *v; }
    if let Some(v) = params.get("TOX") { model.tox = *v; }
    if let Some(v) = params.get("LD") { model.ld = *v; }
    if let Some(v) = params.get("U0") { model.u0 = *v; }
    if let Some(v) = params.get("UEXP") { model.uexp = *v; }
    if let Some(v) = params.get("UCRIT") { model.ucrit = *v; }
    if let Some(v) = params.get("NSUB") { model.nsub = *v; }
    if let Some(v) = params.get("TPG") { model.tpg = *v as i32; }
    if let Some(v) = params.get("NFS") { model.nfs = *v; }
    if let Some(v) = params.get("TPQ") { model.tpq = *v; }
    if let Some(v) = params.get("VMAX") { model.vmax = *v; }
    if let Some(v) = params.get("XJ") { model.xj = *v; }
    if let Some(v) = params.get("KF") { model.kf = *v; }
    if let Some(v) = params.get("AF") { model.af = *v; }
    if let Some(v) = params.get("FC") { model.fc = *v; }
    if let Some(v) = params.get("DELTA") { model.delta = *v; }
    if let Some(v) = params.get("THETA") { model.theta = *v; }
    if let Some(v) = params.get("ETA") { model.eta = *v; }
    if let Some(v) = params.get("KAPPA") { model.kappa = *v; }
}

pub fn parse_spice_file(path: &Path) -> Result<(Netlist, ModelLibrary), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    parse_spice_netlist(&content)
}