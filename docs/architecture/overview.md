# Architecture Overview

> **Mastishk** — computational neuroscience engine

## Module Map

```
mastishk/
├── src/
│   ├── lib.rs                — public API, module re-exports
│   ├── error.rs              — MastishkError enum (non_exhaustive)
│   ├── neurotransmitter.rs   — monoamines, GABA/glutamate, neuropeptides, BDNF
│   ├── circuit.rs            — neural populations, synapses, circuit simulation
│   ├── sleep.rs              — NREM/REM stages, adenosine, sleep debt
│   ├── hpa.rs                — CRH → ACTH → cortisol cascade, allostatic load
│   ├── dmn.rs                — DMN/TPN balance, meditation, rumination
│   ├── chronobiology.rs      — SCN pacemaker, melatonin, cortisol CAR, temperature
│   └── logging.rs            — optional MASTISHK_LOG env-based tracing init
├── benches/
│   └── benchmarks.rs         — criterion benchmarks
├── tests/
│   └── integration.rs        — cross-module integration tests
└── examples/
    └── basic.rs              — runnable usage example
```

## Data Flow

```
External input (stressor, light, task, rest)
  │
  ├─→ neurotransmitter — synthesis/release/reuptake/degradation kinetics
  │     ├── serotonin, dopamine, norepinephrine (monoamines)
  │     ├── GABA, glutamate (amino acid transmitters)
  │     ├── oxytocin, endorphins (neuropeptides)
  │     └── acetylcholine, BDNF (modulators)
  │
  ├─→ circuit — neural population firing rates, synaptic propagation
  │
  ├─→ sleep — adenosine accumulation, stage transitions, recovery
  │
  ├─→ hpa — stress cascade (CRH → ACTH → cortisol), feedback loops
  │
  ├─→ dmn — DMN/TPN anticorrelation, rumination, meditation
  │
  └─→ chronobiology — melatonin/cortisol rhythms, temperature, alertness
```

## Dependency Stack

```
mastishk (this crate)
  │
  ├── serde      — serialization for all types
  ├── thiserror  — error derivation
  └── tracing    — structured logging
```

## Downstream Consumers

```
rasayan (biochemistry)
  └─→ mastishk (this) — neuroscience layer
        ├─→ bhava     — emotion/personality (serotonin→mood, dopamine→reward, cortisol→stress)
        ├─→ bodh      — psychology (cognition, perception, learning)
        ├─→ kiran     — game engine (NPC neurochemistry)
        ├─→ joshua    — agent characters (personality grounded in neuroscience)
        └─→ agnosai   — agent orchestration (cognitive state modeling)
```

## Cross-Module Interactions

```
chronobiology.melatonin ──→ sleep.adenosine (melatonin promotes sleep onset)
chronobiology.cortisol  ──→ hpa.cortisol_baseline (circadian cortisol floor)
sleep.stage             ──→ neurotransmitter (ACh high in REM, serotonin low)
hpa.cortisol            ──→ neurotransmitter.norepinephrine (stress arousal)
dmn.rumination          ──→ hpa.stress (rumination as chronic stressor)
neurotransmitter.arousal ──→ circuit (modulatory input to neural populations)
```

## Design Principles

- **Biologically grounded**: Parameters from neuroscience literature, not arbitrary tuning
- **Composable**: Each module is independent — consumers integrate at the level they need
- **Tickable**: All models advance via `tick(dt)` for simulation-friendly integration
- **Serializable**: All types implement Serialize + Deserialize for state persistence
- **Extensible**: `#[non_exhaustive]` on all enums — new variants without breaking changes
