# Architecture Overview

> **Mastishk** (Sanskrit: मस्तिष्क — brain) — computational neuroscience engine

## Module Map (19 modules)

```
mastishk/
├── src/
│   ├── lib.rs                — public API, module re-exports
│   ├── error.rs              — MastishkError enum
│   │
│   │  ── Core Neuroscience ──
│   ├── neurotransmitter.rs   — 11 transmitters: monoamines, GABA/glutamate, neuropeptides,
│   │                           histamine, endocannabinoid, BDNF. Tonic + phasic dopamine
│   ├── circuit.rs            — rate-model neural populations, synapses, Hebbian plasticity
│   ├── sleep.rs              — Borbely two-process model, NREM/REM ultradian cycles,
│   │                           adenosine, automated stage transitions
│   ├── hpa.rs                — CRH→ACTH→cortisol cascade, allostatic load, sensitization/kindling
│   ├── dmn.rs                — DMN/TPN anticorrelation, rumination, meditation
│   ├── chronobiology.rs      — SCN pacemaker, melatonin, asymmetric cortisol CAR,
│   │                           temperature, photoperiod/seasonal effects
│   │
│   │  ── Brain Regions ──
│   ├── regions.rs            — PFC (executive/WM), amygdala (threat/fear),
│   │                           hippocampus (memory), basal ganglia (Go/NoGo/habits),
│   │                           cerebellum (motor/timing), VTA/NAc reward circuit
│   │
│   │  ── Pharmacology ──
│   ├── receptor.rs           — 12 receptor subtypes (5-HT1A/2A, D1/D2, adrenergic,
│   │                           GABA-A/B, CB1, mu-opioid, NMDA), desensitization ODE
│   ├── pharmacology.rs       — drug profiles, PK lifecycle, Hill equation, transporters
│   │                           (SERT/DAT/NET), 6 preset drugs
│   │
│   │  ── Body Systems ──
│   ├── inflammation.rs       — microglia, cytokines, sickness behavior, IDO pathway
│   ├── gut_brain.rs          — enteric serotonin, vagal tone, microbiome
│   ├── autonomic.rs          — sympathetic/parasympathetic, HRV proxy
│   │
│   │  ── Advanced ──
│   ├── spiking.rs            — Izhikevich + LIF neurons, SpikingNetwork, STDP, BCM
│   ├── eeg.rs                — delta/theta/alpha/beta/gamma band powers
│   │
│   │  ── Integration ──
│   ├── coupling.rs           — cross-module coupling functions + parameter structs
│   ├── brain.rs              — BrainState (~30-step tick), AgeProfile, InteroceptiveState,
│   │                           SexHormoneState
│   ├── bridge.rs             — f64 output functions for bhava/kiran/joshua (28-field
│   │                           BrainMoodEffect composite)
│   └── logging.rs            — optional MASTISHK_LOG tracing init
│
├── benches/benchmarks.rs     — criterion benchmarks (7 benches)
├── tests/integration.rs      — cross-module integration tests (36 tests)
└── docs/
    ├── architecture/overview.md  — this file
    ├── development/roadmap.md    — completed + backlog
    └── guides/usage.md           — patterns, examples, philosophy
```

## BrainState Tick Order (~30 steps)

```
 1. Circadian tick (master clock)
 2. Circadian → HPA (cortisol baseline)
 3. Photoperiod → serotonin synthesis
 4. Sleep → NT (stage-driven baselines: ACh, 5-HT, NE, histamine)
 5. Pharmacology (drug PK, receptor desensitization, NT rate modification)
 6. NT tick (exponential decay toward baselines)
 7. Sex hormones (estradiol→5-HT synthesis, testosterone→amygdala)
 8. Age modifiers (PFC maturation, DA capacity)
 9. NT → Amygdala (NE amplifies, 5-HT/GABA/PFC dampen)
10. NT → Hippocampus (ACh→encoding, BDNF→neurogenesis, sleep→consolidation)
11. NT → PFC (DA inverted-U on WM, amygdala impairs executive function)
12. Amygdala → HPA (threat → stress)
13. DMN → HPA (rumination as chronic stressor)
14. Inflammation → HPA (cytokines as stressor)
15. HPA tick (cascade with sensitization/kindling)
16. Gut-brain → Inflammation (microbiome dampens)
17. Inflammation → NT (tryptophan depletion, sickness fatigue)
18. Inflammation tick
19. Gut-brain → NT (central serotonin modifier)
20. Gut-brain tick
21. NT → Basal Ganglia (tonic DA→Go/NoGo, phasic DA→habit learning)
22. NT → Cerebellum (BDNF→adaptation, sleep debt→precision)
23. Region ticks (amygdala, hippocampus, PFC, basal ganglia, cerebellum, reward circuit)
24. Autonomic coupling (NE/cortisol/amygdala→sympathetic, vagal→parasympathetic)
25. Autonomic tick
26. Interoceptive coupling (autonomic PE → anxiety)
27. Arousal → circuit (NE×glutamate gain + GABA PAM)
28. Sleep tick (Borbely adenosine + stage transitions)
29. EEG target derivation + smooth transition
30. Photoperiod → serotonin (seasonal, very slow)
```

## Downstream Consumers

```
mastishk (this) → bridge.rs → f64 outputs
  ├─→ bhava     — emotion/personality (BrainMoodEffect → MoodVector/StressState)
  ├─→ bodh      — psychology (cognition, perception, learning)
  ├─→ kiran     — game engine (NPC neurochemistry, provides dt)
  ├─→ joshua    — agent characters (personality grounded in neuroscience)
  └─→ agnosai   — agent orchestration (cognitive state modeling)
```

## Design Principles

- **No clock ownership**: mastishk accepts `dt` from the caller. The game engine or agent system owns the clock
- **Biologically grounded**: Parameters from neuroscience literature, validated by external domain review
- **Composable**: Use `BrainState` for the full system, or individual modules with `coupling.rs` functions
- **Tickable**: All models advance via `tick(dt)` → `Result<(), MastishkError>`
- **Serializable**: All types implement `Serialize + Deserialize` with `#[serde(default)]` for backward compatibility
- **Extensible**: `#[non_exhaustive]` on all enums
- **Observable**: `bridge.rs` exposes 28-field `BrainMoodEffect` for downstream consumers
- **Two timescales**: BrainState at seconds, SpikingNetwork at milliseconds (standalone)
