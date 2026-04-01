# Development Roadmap

> **Status**: Pre-1.0 | **Current**: 0.1.0

## Completed

### 0.1.0 — Scaffold (2026-03-31)

- [x] Neurotransmitter system: TransmitterState with exponential decay, NeurotransmitterProfile with 9 transmitters
- [x] Neural circuit: NeuralPopulation, Synapse, Circuit with tick-based propagation
- [x] Sleep architecture: NREM1-3/REM stages, adenosine (Process S), sleep debt, recovery multiplier
- [x] HPA axis: CRH → ACTH → cortisol cascade, negative feedback, allostatic load
- [x] Default mode network: DMN/TPN anticorrelation, meditation, rumination
- [x] Chronobiology: SCN pacemaker, melatonin/cortisol/temperature curves, light entrainment
- [x] Error types with thiserror
- [x] Optional structured logging
- [x] Initial criterion benchmarks

## Backlog

### 0.2.0 — Cross-Module Integration

- [ ] Sleep-neurotransmitter coupling (ACh peaks in REM, serotonin suppressed)
- [ ] Circadian-HPA coupling (cortisol CAR feeds HPA baseline)
- [ ] DMN-HPA coupling (rumination drives chronic stress)
- [ ] Arousal-circuit modulation (NE/glutamate modulate firing rates)
- [ ] Integrated brain state snapshot combining all modules

### 0.3.0 — Receptor Pharmacology

- [ ] Receptor subtypes: 5-HT1A/2A, D1/D2, alpha/beta adrenergic, GABA-A/B
- [ ] Agonist/antagonist modeling (dose-response curves)
- [ ] Receptor desensitization and upregulation
- [ ] Drug interaction modeling (SSRIs, benzodiazepines, stimulants)

### 0.4.0 — Extended Neural Circuits

- [ ] Prefrontal cortex model (executive function, impulse control)
- [ ] Amygdala model (threat detection, fear conditioning)
- [ ] Hippocampus model (memory formation, spatial navigation)
- [ ] Basal ganglia model (habit formation, reward prediction error)
- [ ] Cerebellum model (motor learning, timing)

### 0.5.0 — AI Integration

- [ ] Daimon client for agent registration
- [ ] Hoosh client for LLM-powered neuroscience queries
- [ ] MCP tools: `mastishk_neurotransmitters`, `mastishk_sleep`, `mastishk_stress`, `mastishk_circadian`, `mastishk_circuit`

## Future (demand-gated)

- Spiking neural network models (Izhikevich, LIF)
- Long-term potentiation / depression (LTP/LTD)
- Neuroinflammation and microglial activation
- Gut-brain axis (enteric nervous system, vagus nerve)
- Neurogenesis (adult hippocampal neurogenesis modeling)
- EEG signal generation (alpha, beta, theta, delta, gamma bands)

## v1.0 Criteria

- [ ] All 6 modules have cross-module integration tests
- [ ] All modules have 80%+ test coverage
- [ ] Criterion benchmarks with 3-point trend history
- [ ] Full serde roundtrip tests for all public types
- [ ] bhava consuming mastishk for emotion grounding
- [ ] Documentation: architecture overview, usage guide, API docs
- [ ] Published on crates.io
