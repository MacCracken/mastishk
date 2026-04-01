# Development Roadmap

> **Status**: Pre-1.0 | **Current**: 0.3.0

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

### 0.2.0 — Cross-Module Integration (2026-03-31)

- [x] Sleep-neurotransmitter coupling (ACh peaks in REM, serotonin/NE suppressed per stage)
- [x] Circadian-HPA coupling (cortisol CAR feeds HPA baseline via exponential tracking)
- [x] DMN-HPA coupling (rumination as chronic stressor, feedback gain impairment)
- [x] Arousal-circuit modulation (NE/glutamate multiplicative gain via GANE model)
- [x] Integrated BrainState snapshot combining all modules with orchestrated tick
- [x] CouplingParams / CircuitGainParams for consumer-tunable coupling
- [x] Composite arousal and stress metrics
- [x] Circuit::tick_with_gain for neuromodulatory synaptic scaling

### 0.3.0 — Receptor Pharmacology (2026-03-31)

- [x] Receptor subtypes: 5-HT1A/2A, D1/D2, Alpha1/Alpha2/Beta adrenergic, GABA-A/B (ReceptorSubtype enum, ReceptorState with desensitization ODE, ReceptorMap)
- [x] Agonist/antagonist modeling (Hill equation dose-response, DrugMechanism enum: Agonist/Antagonist/PAM/ReuptakeInhibitor)
- [x] Receptor desensitization and upregulation (availability ODE with occupancy EMA, per-receptor turnover rates)
- [x] Drug interaction modeling: SSRI (fluoxetine, sertraline), benzodiazepine (diazepam, alprazolam), stimulant (amphetamine, methylphenidate) presets
- [x] PharmacologyState with two-phase PK (absorption + elimination), NT coupling (clearance rate modification + baseline shifting + PAM caching)
- [x] ClearanceRateSnapshot for drift-free rate restoration
- [x] GABA PAM integration into circuit gain computation

## Backlog

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

### Bhava Bridge Items (needed for bhava v1.8)

mastishk provides the neural dynamics that bhava's neuroscience bridge consumes. These are the specific f64 outputs bhava needs to map brain state → emotion/personality modules.

#### Neurotransmitter → Emotion Outputs

- [ ] `serotonin_mood_effect(state: &NeurotransmitterProfile) -> f64` — serotonin level mapped to mood baseline floor (−1.0 depleted → +1.0 optimal)
- [ ] `dopamine_reward_sensitivity(state: &NeurotransmitterProfile) -> f64` — dopamine level → preference reinforcement strength (0.0–1.0)
- [ ] `norepinephrine_arousal(state: &NeurotransmitterProfile) -> f64` — NE level → arousal/salience gain (0.0–1.0)
- [ ] `gaba_glutamate_anxiety(state: &NeurotransmitterProfile) -> f64` — GABA/glutamate ratio → anxiety level (0.0 calm → 1.0 panic)
- [ ] `acetylcholine_focus(state: &NeurotransmitterProfile) -> f64` — ACh level → attention/flow entry threshold modifier
- [ ] `endorphin_pain_dampening(state: &NeurotransmitterProfile) -> f64` — endorphin level → stress recovery boost (1.0–2.0×)

#### HPA Axis → Stress

- [ ] `cortisol_stress_amplifier(hpa: &HpaState) -> f64` — cortisol level → stress accumulation rate multiplier (1.0–3.0)
- [ ] `allostatic_load_fraction(hpa: &HpaState) -> f64` — chronic HPA activation → burnout proximity (0.0–1.0)

#### Sleep → Circadian/Energy

- [ ] `sleep_debt_energy_penalty(sleep: &SleepState) -> f64` — accumulated sleep debt → energy recovery rate reduction
- [ ] `sleep_stage_recovery_rate(sleep: &SleepState) -> f64` — current sleep stage → energy/stress recovery multiplier (deep NREM = best)
- [ ] `melatonin_sleep_pressure(chrono: &ChronobiologyState) -> f64` — melatonin level → circadian drowsiness (0.0–1.0)

#### DMN → Cognition/Regulation

- [ ] `rumination_stress_input(dmn: &DmnState) -> f64` — DMN rumination level → chronic stress input
- [ ] `meditation_regulation_boost(dmn: &DmnState) -> f64` — meditation depth → regulation effectiveness multiplier

#### BrainState → Composite

- [ ] `brain_mood_modifiers(state: &BrainState) -> BrainMoodEffect` — single composite struct with all bhava-relevant outputs (mood offsets, stress multiplier, energy modifier, arousal, flow threshold, growth plasticity)

All outputs as plain f64/struct — no bhava types leak into mastishk. bhava's bridge module consumes these and maps to its own MoodVector/StressState/EnergyState.

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
