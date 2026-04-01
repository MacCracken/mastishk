# Development Roadmap

> **Status**: Pre-1.0 | **Current**: 0.4.0

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

### 0.4.0 — Extended Neural Circuits (2026-03-31)

- [x] Prefrontal cortex model (PfcState: executive control, working memory with inverted-U dopamine modulation, fatigue/ego depletion, impulse control output)
- [x] Amygdala model (AmygdalaState: threat detection, fear conditioning/extinction, emotional salience, habituation, NE/serotonin/PFC modulation)
- [x] Hippocampus model (HippocampusState: memory encoding, consolidation, context signal, neurogenesis, ACh/BDNF/sleep modulation)
- [x] Basal ganglia model (BasalGangliaState: Go/No-Go pathways, reward prediction error, habit formation, dopamine D1/D2 modulation)
- [x] Cerebellum model (CerebellumState: motor precision, timing accuracy, error correction, coordination, BDNF/sleep modulation)
- [x] 6 region coupling functions with RegionCouplingParams
- [x] 10 bridge functions for region → bhava outputs
- [x] BrainState tick expanded to 20 steps with region integration

## Backlog

### 0.5.0 — AI Integration

- [ ] Daimon client for agent registration
- [ ] Hoosh client for LLM-powered neuroscience queries
- [ ] MCP tools: `mastishk_neurotransmitters`, `mastishk_sleep`, `mastishk_stress`, `mastishk_circadian`, `mastishk_circuit`

### Bhava Bridge Items (completed 2026-03-31)

- [x] `serotonin_mood_effect` — mood baseline floor (−1.0 to +1.0)
- [x] `dopamine_reward_sensitivity` — preference reinforcement (0.0–1.0)
- [x] `norepinephrine_arousal` — arousal/salience gain (0.0–1.0)
- [x] `gaba_glutamate_anxiety` — anxiety from GABA/glutamate ratio (0.0–1.0)
- [x] `acetylcholine_focus` — attention/flow threshold (0.0–1.0)
- [x] `endorphin_pain_dampening` — stress recovery boost (1.0–2.0×)
- [x] `cortisol_stress_amplifier` — stress accumulation multiplier (1.0–3.0)
- [x] `allostatic_load_fraction` — burnout proximity (0.0–1.0)
- [x] `sleep_debt_energy_penalty` — energy recovery reduction (0.0–1.0)
- [x] `sleep_stage_recovery_rate` — stage recovery multiplier (0.0–1.0)
- [x] `melatonin_sleep_pressure` — circadian drowsiness (0.0–1.0)
- [x] `rumination_stress_input` — chronic stress input (0.0–1.0)
- [x] `meditation_regulation_boost` — regulation effectiveness (1.0–2.0×)
- [x] `brain_mood_modifiers` → `BrainMoodEffect` composite struct with all outputs + growth_plasticity

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
