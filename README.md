# Mastishk

> **Mastishk** (Sanskrit: मस्तिष्क — brain) — computational neuroscience engine for AGNOS

World-class behavioral-timescale brain simulation: 12 neurotransmitters, 20 receptor subtypes, 10 brain regions, pharmacology with 6 drug presets, spiking neural networks, and 30+ step integrated brain state tick. All parameters validated against neuroscience literature.

Used by [bhava](https://github.com/MacCracken/bhava) (emotion/personality), [bodh](https://github.com/MacCracken/bodh) (psychology/cognition), [kiran](https://github.com/MacCracken/kiran) (game engine), [joshua](https://github.com/MacCracken/joshua) (agent characters), and [agnosai](https://github.com/MacCracken/agnosai) (agent orchestration).

## Modules (19)

| Module | Description |
|--------|-------------|
| `neurotransmitter` | 12 transmitters: serotonin, dopamine (tonic+phasic), norepinephrine, GABA, glutamate, oxytocin, endorphins, acetylcholine, BDNF, histamine, endocannabinoid, orexin |
| `receptor` | 20 receptor subtypes: 5-HT1A/2A/2C, D1/D2, Alpha1/Alpha2/Beta, GABA-A/B, CB1, mu-opioid, NMDA, AMPA, OX1/OX2, nicotinic ACh alpha4beta2/alpha7, H1. Desensitization/upregulation ODE |
| `pharmacology` | Drug profiles with Hill equation, transporters (SERT/DAT/NET), receptor bindings, PK lifecycle, partial agonism, withdrawal/rebound. Presets: fluoxetine, sertraline, diazepam, alprazolam, amphetamine, methylphenidate |
| `regions` | 10 brain regions: PFC (executive/WM), amygdala (threat/fear), hippocampus (memory), basal ganglia (Go/NoGo/habits), cerebellum (motor/timing), VTA/NAc (wanting/craving), locus coeruleus (NE tonic/phasic), raphe (5-HT source), ACC (conflict monitoring), insula (interoception) |
| `circuit` | Rate-model neural populations, synaptic propagation, neuromodulatory gain, Hebbian plasticity |
| `spiking` | Full-fidelity Izhikevich (4 presets) + LIF neurons, SpikingNetwork with STDP, BCM rule. Standalone ms-timescale |
| `sleep` | Borbely two-process model, automated ultradian cycle (NREM1-3/REM), stage transitions |
| `hpa` | CRH->ACTH->cortisol cascade (realistic 5-15min timing), allostatic load, stress sensitization/kindling |
| `dmn` | DMN/TPN anticorrelation, rumination, meditation suppression |
| `chronobiology` | SCN pacemaker, melatonin, asymmetric cortisol CAR (wake-coupled), temperature, photoperiod/seasonal effects |
| `inflammation` | Microglia, cytokines, sickness behavior, IDO tryptophan depletion pathway |
| `gut_brain` | Enteric serotonin (95% of body 5-HT), vagal tone, microbiome diversity |
| `autonomic` | Sympathetic/parasympathetic reciprocal inhibition, HRV proxy |
| `eeg` | Delta/theta/alpha/beta/gamma band powers derived from brain state |
| `coupling` | 15+ cross-module coupling functions with tunable parameters |
| `brain` | Unified `BrainState` with 30+ step tick, `AgeProfile`, `InteroceptiveState`, `SexHormoneState`, `TdLearner`, `OpponentProcess` |
| `bridge` | 28-field `BrainMoodEffect` composite for downstream consumers (bhava/kiran/joshua) |

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | Standard library support |
| `logging` | no | Structured logging via `MASTISHK_LOG` env var |
| `full` | -- | Enables all features |

## Quick Start

```toml
[dependencies]
mastishk = "1"
```

```rust
use mastishk::brain::BrainState;
use mastishk::bridge::brain_mood_modifiers;

// Create a default brain (all subsystems at resting state)
let mut brain = BrainState::default();

// Advance 1 second — all 30+ tick steps execute in causal order
brain.tick(1.0).unwrap();

// Query composite outputs
println!("Arousal: {:.2}", brain.arousal());
println!("Stress: {:.2}", brain.stress());

// Get all outputs for downstream consumers
let effect = brain_mood_modifiers(&brain);
println!("Mood: {:.2}, Anxiety: {:.2}", effect.mood_offset, effect.anxiety);
```

## Key Design Principles

- **No clock ownership** — accepts `dt` from the caller (kiran game engine, joshua agents). Caller controls simulation speed, pause, time-skip
- **Biologically grounded** — parameters from neuroscience literature, validated by two external domain audits
- **Composable** — use `BrainState` for the full system, or individual modules with `coupling.rs` functions
- **Two timescales** — `BrainState` at seconds (behavioral), `SpikingNetwork` at milliseconds (neural)
- **Serializable** — all types implement `Serialize + Deserialize` with backward-compatible `#[serde(default)]`

## Architecture

```text
mastishk (this) -> bridge.rs -> f64 outputs
  |-> bhava     — emotion/personality (BrainMoodEffect -> MoodVector/StressState)
  |-> bodh      — psychology (cognition, perception, learning)
  |-> kiran     — game engine (NPC neurochemistry, provides dt)
  |-> joshua    — agent characters (personality grounded in neuroscience)
  |-> agnosai   — agent orchestration (cognitive state modeling)
```

## Development

```bash
make check     # fmt + clippy + test + audit
make bench     # Run benchmarks with history tracking
make coverage  # Generate coverage report
make doc       # Build documentation
```

## Documentation

- [Architecture Overview](docs/architecture/overview.md)
- [Usage Guide](docs/guides/usage.md)
- [Development Roadmap](docs/development/roadmap.md)
- [API Documentation](https://docs.rs/mastishk)

## License

GPL-3.0-only. See [LICENSE](LICENSE).
