# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed (Domain Accuracy — external review)
- **pharmacology** — SSRIs now correctly target SERT transporter (not 5-HT1A/2A receptors). Added `TransporterType` enum (Sert, Dat, Net), `TransporterBinding` struct, and `transporter_bindings` field on `DrugProfile`. Methylphenidate similarly moved to DAT/NET transporters. Amphetamine retains D1/D2 agonist bindings (vesicular release) plus DAT/NET transporter bindings
- **coupling** — Added bidirectional amygdala↔PFC coupling: high amygdala activation now impairs PFC executive control and working memory (Arnsten 2009 stress-cognition trade-off). Previously only PFC→amygdala inhibition existed
- **sleep** — Replaced linear adenosine dynamics with Borbely two-process model: exponential rise during wake (tau_w=18.2h) and exponential decay during sleep (tau_s=4.2h). Agents now properly recover from sleep (0.8→0.12 after 8hr sleep vs old 0.8→0.32)
- **neurotransmitter** — Added `dopamine_phasic` field (−1.0 to +1.0) for transient reward prediction error signals distinct from tonic dopamine level. `fire_dopamine_burst()` method. Phasic decays with ~500ms half-life. Tonic DA drives Go/NoGo motivation, phasic DA drives habit learning rate in basal ganglia coupling
- **hpa** — Slowed HPA cascade timing ~100x to match biological reality: CRH→ACTH tau≈300s (~5 min), ACTH→cortisol tau≈600s (~10 min), feedback tau≈900s (~15 min). Previously produced equilibrium in seconds

## [0.4.0] - 2026-03-31

### Added
- **regions** — New module: 5 brain region models — `PfcState` (executive function, impulse control, working memory with fatigue/ego depletion), `AmygdalaState` (threat detection, fear conditioning, emotional salience, habituation), `HippocampusState` (memory encoding, consolidation, context signal, neurogenesis), `BasalGangliaState` (Go/No-Go pathways, reward prediction error, habit formation), `CerebellumState` (motor precision, timing accuracy, error correction, coordination)
- **coupling** — 6 new brain region coupling functions: NT→PFC (dopamine inverted-U on WM, serotonin→impulse control, cortisol/sleep debt impairment), NT→amygdala (NE amplifies, serotonin/GABA/PFC dampen), NT→hippocampus (ACh→encoding, BDNF→neurogenesis, amygdala salience→emotional memory, sleep→consolidation), amygdala→HPA (threat→stress), NT→basal ganglia (dopamine Go/No-Go, PFC goal bias), NT→cerebellum (BDNF→adaptation, sleep debt→precision)
- **coupling** — `RegionCouplingParams` for tunable region coupling strengths
- **bridge** — 10 new region bridge functions: PFC executive/WM, amygdala fear/salience, hippocampus learning/context, basal ganglia action drive/habit, cerebellum motor quality. Extended `BrainMoodEffect` with 8 new fields
- **bridge** — Bhava bridge complete: 13 NT/HPA/sleep/DMN output functions + `BrainMoodEffect` composite struct with `brain_mood_modifiers()` aggregator

### Changed
- **brain** — Tick order expanded from 9 to 20 steps: region couplings and ticks integrated in correct causal order (sensory→executive→motor). 5 new `#[serde(default)]` region fields on `BrainState`

## [0.3.0] - 2026-03-31

### Added
- **receptor** — New module: `ReceptorSubtype` enum (Ht1a, Ht2a, D1, D2, Alpha1, Alpha2, Beta, GabaA, GabaB), `ReceptorState` with desensitization/upregulation ODE, `ReceptorMap` with per-receptor parameterized turnover rates, `ReceptorOccupancies` for aggregate drug occupancy
- **pharmacology** — New module: `DrugProfile` (receptor bindings, PK parameters), `ActiveDrug` (two-phase pharmacokinetics: absorption + elimination), `PharmacologyState` (receptor map + active drugs + NT coupling), Hill equation dose-response, Clark occupancy model
- **pharmacology** — Drug mechanism types: `ReuptakeInhibitor` (reduces clearance rate), `Agonist` (raises baseline), `Antagonist` (lowers baseline), `PositiveAllostericModulator` (GABA PAM multiplier for circuit gain)
- **pharmacology** — Preset drug constructors: `ssri_fluoxetine`, `ssri_sertraline`, `benzodiazepine_diazepam`, `benzodiazepine_alprazolam`, `stimulant_amphetamine`, `stimulant_methylphenidate`
- **pharmacology** — `ClearanceRateSnapshot` for drift-free rate restoration each tick
- **brain** — `BrainState::administer_drug()` convenience method, pharmacology tick step in causal order
- **error** — `InvalidDrugParameter` error variant

### Changed
- **coupling** — `compute_circuit_gain` and `apply_arousal_circuit_coupling` now accept `gaba_pam` parameter for benzodiazepine-class PAM amplification of GABA inhibition
- **brain** — Tick order expanded: pharmacology step inserted between sleep→NT coupling and NT tick; circuit gain now incorporates GABA PAM from active drugs
- **brain** — `BrainState.pharmacology` field added with `#[serde(default)]` for backward-compatible deserialization

## [0.2.0] - 2026-03-31

### Added
- **coupling** — New cross-module coupling functions: sleep→neurotransmitter (ACh peaks in REM, serotonin/NE suppressed), circadian→HPA (cortisol awakening response sets HPA baseline), DMN→HPA (rumination as chronic stressor with feedback gain impairment), arousal→circuit (NE/glutamate multiplicative gain via GANE model)
- **brain** — New `BrainState` struct orchestrating all 6 subsystems with a single `tick(dt)` applying couplings in correct causal order. Composite `arousal()` and `stress()` metrics
- **coupling** — `CouplingParams` and `CircuitGainParams` for consumer-tunable coupling strengths
- **coupling** — `composite_arousal()` and `composite_stress()` combining multi-module state
- **circuit** — `Circuit::tick_with_gain(gain, dt)` for neuromodulatory synaptic scaling without permanent weight mutation
- **all modules** — `#[inline]` on all hot-path `tick`, getter, and computed property functions
- **all modules** — `tracing` instrumentation on all public state-mutating operations (`debug!` for discrete events, `trace!` for per-tick updates)
- **all modules** — Comprehensive test coverage: negative dt rejection, boundary conditions, untested getters, edge cases, cross-module integration tests (24hr cycle, sleep deprivation, stress-rumination feedback). Test count: 42 → 105

### Changed
- **all modules** — All `tick` methods now return `Result<(), MastishkError>`, rejecting negative time deltas with `NegativeTimeDelta` error
- **circuit** — `add_synapse` now returns `Result`, validating population indices at creation time
- **hpa** — Replaced Euler integration with exponential decay for stable behavior at large dt values
- **chronobiology** — `CircadianState::default()` now derives rhythm values from `update_rhythms()` instead of hardcoded approximations

### Fixed
- **sleep** — Fixed `total_sleep` units: was incorrectly multiplying hours by 3600 (mixing seconds/hours); now consistently uses hours
- **docs** — Removed claims of Hebbian learning and lateral inhibition from circuit module (not yet implemented)

## [0.1.0] - 2026-03-31

### Added

- **neurotransmitter** — Monoamine dynamics (serotonin, dopamine, norepinephrine), GABA/glutamate balance, neuropeptides (oxytocin, endorphins), acetylcholine, BDNF neuroplasticity. TransmitterState with exponential decay toward baseline, NeurotransmitterProfile with arousal/reward/plasticity derivations
- **circuit** — Neural circuit primitives: NeuralPopulation with rate-model dynamics, Synapse connections, Circuit with tick-based propagation. Excitatory/inhibitory populations, mean-field rate models
- **sleep** — Sleep architecture: NREM1-3 and REM stages, adenosine buildup (Process S), sleep debt tracking, ultradian cycle counting. Recovery multiplier and memory consolidation rate per stage
- **hpa** — HPA axis stress response: CRH -> ACTH -> cortisol cascade with negative feedback, allostatic load accumulation, chronic stress detection
- **dmn** — Default mode network: DMN/TPN anticorrelation, task engagement, rest/mind-wandering, meditation depth, rumination from negative valence + high DMN
- **chronobiology** — SCN pacemaker model: melatonin curve (peak ~3 AM, light suppression), cortisol awakening response (peak ~8 AM), core body temperature oscillation, alertness derivation
- **error** — `MastishkError` with variants for level-out-of-range, invalid circuit, invalid sleep transition, negative time delta
- **logging** — Optional structured logging via `MASTISHK_LOG` env var (feature-gated)
- Initial criterion benchmarks for neurotransmitter tick, circuit tick, HPA tick
