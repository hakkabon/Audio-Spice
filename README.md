# Audio-Spice

A modular circuit simulation platform with two specialized workflows built on a shared simulation engine:

1. **Loudspeaker Passive Crossover Designer** - Domain-specific tool for passive loudspeaker crossover design
2. **General SPICE-like Circuit Simulator** - General-purpose analog circuit simulation

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
├──────────────────────────────┬──────────────────────────────┤
│  Crossover Mode              │  Circuit Mode                │
│  (audio-spice-crossover)     │  (audio-spice-circuit)       │
├──────────────────────────────┼──────────────────────────────┤
│         Domain Layer         │                              │
│  Drivers, Filters, Targets,  │  Netlist, Analyses, Models   │
│  Optimization, BOM           │                              │
├──────────────────────────────┴──────────────────────────────┤
│              Core Simulation Engine (audio-spice-core)      │
│  Netlist • MNA Solver • AC/DC/TRAN • SPICE Parser/Export   │
└─────────────────────────────────────────────────────────────┘
```

## Crates

### `audio-spice-core` - Simulation Engine

The reusable heart of the platform.

**Components:**
- Passive: Resistor, Capacitor, Inductor
- Sources: VoltageSource (DC/AC), CurrentSource (DC/AC)
- Controlled: VCVS, VCCS, CCCS, CCVS, OpAmp
- Semiconductors: Diode, BJT, MOSFET (Level 1)

**Analyses:**
- **AC** - Frequency-domain small-signal analysis with logarithmic sweeps
- **DC** - Operating point with Newton-Raphson iteration
- **TRAN** - Time-domain transient with backward Euler integration

**Features:**
- Modified Nodal Analysis (MNA) with complex matrix solver
- SPICE netlist parser (`.MODEL`, controlled sources, semiconductors)
- SPICE netlist export with model libraries
- Parameterizable frequency sweeps (points/decade)

### `audio-spice-crossover` - Loudspeaker Crossover Designer

Specialized workflow for passive loudspeaker crossover design.

**Driver Model:**
- Measured impedance curves (`.zma`, `.txt`)
- Measured frequency response (`.frd`, `.txt`)
- Thiele-Small parameters
- Acoustic offset, polarity, sensitivity

**Crossover Synthesis:**
- Butterworth (1st-4th order)
- Linkwitz-Riley (2nd, 4th, 6th, 8th order)
- Bessel (2nd-4th order)
- Custom ladder networks

**Compensation Networks:**
- Zobel (impedance correction)
- L-pad (attenuation)
- Notch filters
- Baffle-step correction

**Simulation:**
- Electrical transfer functions into driver loads
- Acoustic summation: `H_acoustic = V_driver × H_driver(f)`
- Time alignment & polarity inversion
- System impedance, component stress/power

**Optimization:**
- Genetic algorithm for component values
- Target curve matching (flat, house curve, Harman, custom)
- E12/E24 standard value constraints

### `audio-spice-circuit` - General Circuit Simulator

SPICE-like environment for general analog design.

**Features:**
- SPICE netlist input (string or file)
- Multi-analysis: DC, AC, TRAN in single run
- Model library management (diodes, BJTs, MOSFETs)
- Results export (JSON-serializable)
- SPICE netlist export for interoperability

## Quick Start

### Build

```bash
cargo build --workspace
```

### Run Examples

**Core AC Analysis (RC Filter):**
```bash
cargo run --example rc_filter
```

**Crossover Designer:**
```bash
cargo run -p audio-spice-crossover --example crossover_designer
```

**Circuit Simulator:**
```bash
cargo run -p audio-spice-circuit --example circuit_simulator
```

## Usage

### Core Engine (Rust API)

```rust
use audio_spice_core::{Component, Netlist, AcSweep, simulate_ac};

let mut netlist = Netlist::new();
netlist.add_component(Component::VoltageSource { name: "V1".into(), n_pos: 1, n_neg: 0, amplitude: 1.0, phase_deg: 0.0 });
netlist.add_component(Component::Resistor { name: "R1".into(), n_pos: 1, n_neg: 2, value: 1000.0 });
netlist.add_component(Component::Capacitor { name: "C1".into(), n_pos: 2, n_neg: 0, value: 1e-6 });

let sweep = AcSweep { f_start: 10.0, f_stop: 100000.0, points_per_decade: 20 };
let result = simulate_ac(&netlist, &sweep)?;
```

### Crossover Design

```rust
use audio_spice_crossover::{Driver, DriverDatabase, DriverRole, CrossoverSynthesizer, CrossoverType, FilterType, CrossoverDesign, CrossoverSimulator, SimulationConfig};

// Create drivers
let mut woofer = Driver::new("w".into(), "Woofer".into(), DriverRole::Woofer);
woofer.re = 6.2; woofer.le = Some(0.5e-3); woofer.sensitivity_db = 87.0;
let mut tweeter = Driver::new("t".into(), "Tweeter".into(), DriverRole::Tweeter);
tweeter.re = 4.8; tweeter.le = Some(0.05e-3); tweeter.sensitivity_db = 90.0;
tweeter.polarity = Polarity::Inverted; tweeter.acoustic_offset_mm = 12.0;

let mut drivers = DriverDatabase::new();
drivers.add_driver(woofer); drivers.add_driver(tweeter);

// Synthesize LR4 crossover at 2.5kHz
let woofer_filter = FilterSpec { filter_type: FilterType::LowPass, crossover_type: CrossoverType::LinkwitzRiley4, fc: 2500.0, components: CrossoverSynthesizer::synthesize_filter(FilterType::LowPass, CrossoverType::LinkwitzRiley4, 2500.0, 8.0, "W") };
let tweeter_filter = FilterSpec { filter_type: FilterType::HighPass, crossover_type: CrossoverType::LinkwitzRiley4, fc: 2500.0, components: CrossoverSynthesizer::synthesize_filter(FilterType::HighPass, CrossoverType::LinkwitzRiley4, 2500.0, 8.0, "T") };

let design = CrossoverDesign { name: "2-way".into(), num_ways: 2, crossover_frequencies: vec![2500.0], crossover_types: vec![CrossoverType::LinkwitzRiley4], drivers: vec!["w".into(), "t".into()], filters: vec![woofer_filter, tweeter_filter], compensation: vec![], target: None };

let sim = CrossoverSimulator::new(design, drivers, SimulationConfig::default());
let result = sim.simulate()?;
```

### Circuit Simulation

```rust
use audio_spice_circuit::{CircuitProject, CircuitSimulationConfig, CircuitSimulator, AnalysisType, AcConfig};

let netlist = r#"
* RC Low-pass
V1 1 0 AC 1 0
R1 1 2 1k
C1 2 0 1uF
.END
"#;

let project = CircuitProject { name: "RC".into(), netlist_text: netlist.into(), analyses: vec![AnalysisType::AC] };
let config = CircuitSimulationConfig { ac: Some(AcConfig { f_start: 10.0, f_stop: 100000.0, points_per_decade: 20 }), ..Default::default() };

let sim = CircuitSimulator::new(project, config);
let result = sim.run()?;
if let Some(ac) = result.ac { /* process results */ }
```

### SPICE Netlist Parsing

```rust
use audio_spice_core::parse_spice_netlist;

let (netlist, models) = parse_spice_netlist(r#"
V1 1 0 DC 5
R1 1 2 1k
C1 2 0 1uF
D1 2 0 1N4148
.MODEL 1N4148 D IS=1e-14 N=1
.END
"#)?;
```

## Supported SPICE Subset

| Element | Syntax | Status |
|---------|--------|--------|
| Resistor | `Rname n+ n- value` | ✓ |
| Capacitor | `Cname n+ n- value` | ✓ |
| Inductor | `Lname n+ n- value` | ✓ |
| Voltage Source | `Vname n+ n- DC/AC value [phase]` | ✓ |
| Current Source | `Iname n+ n- DC/AC value [phase]` | ✓ |
| VCVS | `Ename n+ n- nc+ nc- gain` | ✓ |
| VCCS | `Gname n+ n- nc+ nc- gm` | ✓ |
| CCCS | `Fname n+ n- Vctrl gain` | ✓ |
| CCVS | `Hname n+ n- Vctrl rm` | ✓ |
| Diode | `Dname n+ n- model [area]` | ✓ |
| BJT | `Qname nc nb ne model [area]` | ✓ |
| MOSFET | `Mname nd ng ns nb model [W= L=]` | ✓ |
| `.MODEL` | Diode/NPN/PNP/NMOS/PMOS | ✓ |

**Value suffixes:** T(1e12), G(1e9), MEG(1e6), K(1e3), M/m(1e-3), U/µ(1e-6), N(1e-9), P(1e-12), F(1e-15). Compound suffixes: uF, pF, nH, kΩ, MEG.

## File Formats

**Driver Data:**
- `.frd` - Frequency response: `freq magnitude phase`
- `.zma` - Impedance: `freq magnitude phase`
- `.txt` - Generic whitespace-separated

**Netlist:** Standard SPICE format with `.MODEL`, `.END`

## Project Structure

```
Audio-Spice/
├── Cargo.toml                    # Workspace
├── crates/
│   ├── core/                     # Simulation engine
│   │   ├── src/
│   │   │   ├── component.rs      # Component definitions
│   │   │   ├── netlist.rs        # Netlist container
│   │   │   ├── mna.rs            # MNA matrix builder
│   │   │   ├── solver.rs         # Gaussian elimination
│   │   │   ├── engine.rs         # AC/DC/TRAN simulations
│   │   │   ├── simulation.rs     # Sweep/result types
│   │   │   ├── models.rs         # Semiconductor models
│   │   │   ├── spice_parser.rs   # SPICE netlist parser
│   │   │   └── types.rs          # Complex number type
│   │   └── examples/rc_filter.rs
│   ├── crossover/                # Loudspeaker crossover app
│   │   ├── src/
│   │   │   ├── driver.rs         # Driver model & import
│   │   │   ├── synthesis.rs      # Filter synthesis
│   │   │   ├── netlist_builder.rs# Crossover→netlist
│   │   │   ├── optimizer.rs      # Genetic optimization
│   │   │   └── types.rs          # Domain types
│   │   └── examples/crossover_designer.rs
│   └── circuit/                  # General circuit app
│       ├── src/types.rs          # Project, config, simulator
│       └── examples/circuit_simulator.rs
```

## License

MIT