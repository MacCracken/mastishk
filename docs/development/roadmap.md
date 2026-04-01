# Development Roadmap

> **Status**: Pre-1.0 | **Current**: 0.7.0

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

### 0.6.0 — Domain Accuracy (high-priority gaps from external review)

- [x] Histamine transmitter — wakefulness signal, sleep-wake flip-flop (Saper 2005)
- [x] Endocannabinoid system (anandamide/2-AG, CB1 receptor) — stress buffer, HPA recovery
- [x] VTA/Nucleus Accumbens reward circuit — incentive salience/wanting distinct from Go/NoGo
- [x] Cortisol asymmetric waveform — sharp CAR Gaussian rise + slow exponential decay
- [x] Transporter targets (SERT, DAT, NET) — pharmacologically correct reuptake inhibitor routing
- [x] Dopamine tonic/phasic split — tonic drives Go/NoGo, phasic drives RPE learning
- [x] Borbely adenosine model — exponential rise/decay (tau_w=18.2h, tau_s=4.2h)
- [x] HPA cascade realistic timing — CRH→ACTH 5min, ACTH→cortisol 10min, feedback 15min
- [x] Bidirectional amygdala↔PFC coupling — stress impairs executive function (Arnsten 2009)
- [x] SSRI→SERT transporter fix — SSRIs correctly target transporter, not receptors
- [x] Mu-opioid + NMDA receptor subtypes — pain/endorphin system + glutamate learning
- [x] Automated sleep stage transitions — 90-min ultradian cycle with fall_asleep/wake_up/tick_stage_transitions
- [x] Stress sensitization / kindling — allostatic load → crh_sensitivity (Post 1992)
- [x] Sex hormone modulation — SexHormoneState: estradiol→serotonin synthesis, testosterone→amygdala reactivity

### 0.7.0 — Advanced Neural Dynamics (2026-03-31)

- [x] Spiking neural network models — Izhikevich (2003) with 4 presets + LIF, SpikingNetwork with STDP
- [x] LTP/LTD — STDP for spiking networks, BcmRule for rate-based, Circuit::apply_hebbian
- [x] Neuroinflammation — InflammationState: microglia, cytokines, sickness behavior, IDO tryptophan depletion
- [x] Gut-brain axis — GutBrainState: enteric serotonin, vagal tone, microbiome, central serotonin modifier
- [x] Autonomic nervous system — AutonomicState: sympathetic/parasympathetic reciprocal inhibition, HRV proxy
- [x] Interoceptive inference — InteroceptiveState: predictive processing (Seth 2013), PE → anxiety
- [x] Seasonal/photoperiod effects — photoperiod_hours, serotonin_photoperiod_modifier (Lambert 2002)
- [x] Age-related parameter curves — AgeProfile: pfc_maturation (~25), dopamine_capacity (~40+), deep_sleep_capacity
- [x] EEG signal generation — EegState: delta/theta/alpha/beta/gamma derived from brain state, EegBand enum

## v1.0 Criteria

- [x] All modules have cross-module integration tests (36 integration tests)
- [x] All modules have 80%+ test coverage (228 tests across 19 modules)
- [x] Criterion benchmarks with history (7 benchmarks, bench-history.csv)
- [x] Full serde roundtrip tests for all public types
- [ ] bhava consuming mastishk for emotion grounding (external dependency)
- [x] Documentation: architecture overview, usage guide, API docs
- [x] External domain review: 13 domain accuracy improvements from literature review

---

# Road to v2.0 — World-Class Completeness

> Based on exhaustive domain audit (2026-03-31). Every item below represents proven,
> published neuroscience with behavioral-level significance.

## v1.1 — Critical Fixes (parameter bugs + missing fundamentals)

### Parameter Corrections
- [ ] Cortisol CAR coupled to wake_up event, not fixed 8AM phase — shift workers get wrong timing
- [ ] Fluoxetine effective half-life → 4-6 days (norfluoxetine active metabolite)
- [ ] GABA-A receptor turnover → 5-7 days (from 3 days, for realistic BZD tolerance)

### Missing Fundamentals
- [ ] AMPA receptor — obligate partner to NMDA, mediates >90% fast glutamatergic transmission
- [ ] Orexin/hypocretin — master wakefulness stabilizer, required by Saper flip-flop model. Add as neuromodulator + OX1/OX2 receptors + sleep-wake coupling
- [ ] Partial agonism — `PartialAgonist { intrinsic_activity }` in DrugMechanism. Enables buspirone (5-HT1A), buprenorphine (mu-opioid), aripiprazole (D2)
- [ ] Withdrawal/rebound dynamics — receptor availability < baseline after drug removal → reduced signaling; availability > baseline → rebound symptoms

## v1.2 — Core Neuroscience Gaps (HIGH priority)

### Source Nuclei
- [ ] Locus coeruleus (LC) — NE source with tonic/phasic modes (Aston-Jones & Cohen 2005). Tonic = exploration, phasic = exploitation/focus
- [ ] Raphe nuclei — serotonin source, firing rate modulates 5-HT synthesis

### Receptors
- [ ] 5-HT2C — rate-limiting for SSRI therapeutic lag (Dremencov 2009)
- [ ] Nicotinic ACh alpha4beta2 — high affinity, nicotine target, desensitizes
- [ ] Nicotinic ACh alpha7 — fast, low affinity, cognitive enhancement
- [ ] H1 histamine receptor — wakefulness (antihistamines cause drowsiness)

### Brain Regions
- [ ] Anterior cingulate cortex (ACC) — conflict monitoring, error detection, effort allocation. Bridge between detecting need for control and recruiting PFC
- [ ] Insula — interoceptive cortex: body-state awareness, disgust, empathy, pain. Grounds InteroceptiveState in a brain region

### Computational Models
- [ ] TD learning — `dopamine_phasic` IS the RPE from Schultz 1997 but no learning rule consumes it. Add `TdLearner` with value estimates updated by phasic DA
- [ ] Opponent process — Solomon & Corbit 1974 a-process/b-process for hedonic adaptation. b-process grows with repeated exposure → tolerance, withdrawal, addiction dynamics

## v1.3 — Pharmacological Completeness

- [ ] Negative allosteric modulator (NAM) — flumazenil (BZD reversal), mGluR5 NAMs (anxiety)
- [ ] Inverse agonism — distinct from antagonism (reduces constitutive activity)
- [ ] CYP450 drug metabolism — isoform tags on DrugProfile, metabolic competition model. Fluoxetine is potent CYP2D6 inhibitor
- [ ] Buspirone preset — 5-HT1A partial agonist (requires partial agonism from v1.1)
- [ ] Aripiprazole preset — D2 partial agonist + 5-HT1A partial agonist (atypical antipsychotic)
- [ ] Buprenorphine preset — mu-opioid partial agonist (opioid maintenance therapy)

## v1.4 — Neuromodulator Completeness

- [ ] Neuropeptide Y (NPY) — strongest endogenous anxiolytic, opposes CRH in amygdala, stress resilience biomarker
- [ ] Vasopressin — social behavior, pair bonding (distinct from oxytocin), aggression, V1a receptor
- [ ] Dynorphin + kappa-opioid receptor — stress→dynorphin→dysphoria pathway, opposes mu-opioid hedonic effects
- [ ] Substance P + NK1 receptor — pain-mood coupling, NK1 antagonists have antidepressant properties
- [ ] NE inverted-U for PFC — Yerkes-Dodson: low NE = inattentive, optimal = focused, high = impaired

## v1.5 — Brain Region Expansion

- [ ] PFC subregions: dlPFC (working memory), vmPFC (value/emotion regulation), OFC (reversal learning)
- [ ] Periaqueductal gray (PAG) — pain modulation, defensive behavior (fight/flight/freeze/fawn)
- [ ] Bed nucleus of stria terminalis (BNST) — sustained anxiety (vs amygdala acute fear)
- [ ] Thalamus — sensory relay, attention gating (filters what reaches cortex)

## v1.6 — Computational Models

- [ ] Drift-diffusion model (DDM) — two-choice decision-making, reaction time, parameterized by NT states
- [ ] Rescorla-Wagner associative learning — foundation for conditioning, extinction
- [ ] Wilson-Cowan population dynamics — bridges spiking and rate models rigorously
- [ ] FitzHugh-Nagumo spiking model — simplified Hodgkin-Huxley
- [ ] Attractor networks — persistent activity for working memory, pattern completion

## v1.7 — Remaining Proven Systems

- [ ] Adenosine as general neuromodulator (beyond sleep — pain, inflammation, neuroprotection)
- [ ] Glycine co-transmission — obligate NMDA co-agonist
- [ ] Nitric oxide retrograde signaling — affects LTP at glutamatergic synapses
- [ ] H3 histamine autoreceptor
- [ ] Astrocyte regulation of synaptic transmission (tripartite synapse)

## v2.0 Criteria

- [ ] All CRITICAL and HIGH findings from domain audit addressed
- [ ] 30+ neuromodulators/transmitters modeled
- [ ] 20+ receptor subtypes
- [ ] 12+ brain regions with distinct dynamics
- [ ] Computational learning models (TD, Rescorla-Wagner, DDM)
- [ ] Complete pharmacological mechanism set (agonist, partial agonist, antagonist, inverse agonist, PAM, NAM, reuptake inhibitor)
- [ ] Withdrawal/rebound dynamics validated against clinical timelines
- [ ] All parameters validated against published literature with citations
- [ ] bhava v1.8+ consuming mastishk bridge outputs
- [ ] 500+ tests
- [ ] Benchmark regression tracking with 3+ point history
