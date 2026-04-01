# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-03-31

### Core Neuroscience
- **neurotransmitter** — 12 transmitters: serotonin, dopamine (tonic + phasic RPE), norepinephrine, GABA, glutamate, oxytocin, endorphins, acetylcholine, BDNF, histamine, endocannabinoid, orexin. TransmitterState with exponential decay, synthesis/clearance kinetics
- **receptor** — 20 receptor subtypes: 5-HT1A/2A/2C, D1/D2, Alpha1/Alpha2/Beta-adrenergic, GABA-A/B, CB1, mu-opioid, NMDA, AMPA, OX1/OX2, nicotinic ACh alpha4beta2/alpha7, H1. Desensitization/upregulation ODE with occupancy EMA
- **circuit** — Rate-model neural populations, synaptic propagation, neuromodulatory gain (GANE model), tick_with_gain, Hebbian plasticity
- **spiking** — Full-fidelity Izhikevich (2003) with 4 presets (RS, FS, CH, IB) + LIF neurons, SpikingNetwork with synaptic delay, STDP learning rule, BCM sliding threshold. Standalone ms-timescale
- **sleep** — Borbely two-process model (exponential adenosine: tau_w=18.2h, tau_s=4.2h), automated 90-min ultradian cycle with stage transitions (NREM1→NREM2→NREM3→NREM2→REM), fall_asleep/wake_up
- **hpa** — CRH→ACTH→cortisol cascade with biologically realistic timing (tau 300/600/900s), negative feedback, allostatic load, stress sensitization/kindling (Post 1992)
- **dmn** — DMN/TPN anticorrelation, rumination (negative valence + high DMN), meditation suppression
- **chronobiology** — SCN pacemaker, melatonin (peak 3AM, light suppression), asymmetric cortisol CAR coupled to wake event (not fixed clock), core body temperature, photoperiod/seasonal serotonin effects (Lambert 2002)

### Brain Regions (10)
- **regions** — PFC (executive control, working memory with dopamine inverted-U, fatigue/ego depletion), amygdala (threat detection, fear conditioning/extinction, habituation, bidirectional PFC coupling), hippocampus (encoding, consolidation, context, BDNF-driven neurogenesis), basal ganglia (Go/NoGo with tonic DA, RPE with phasic DA, habit formation), cerebellum (motor precision, timing, error correction), VTA/NAc reward circuit (incentive salience/wanting, craving, sensitization), locus coeruleus (tonic/phasic NE modes — Aston-Jones 2005), raphe nuclei (5-HT source with autoreceptor feedback), anterior cingulate cortex (conflict monitoring, error detection, effort allocation), insula (interoceptive cortex: body awareness, disgust, empathy, pain)

### Pharmacology
- **pharmacology** — Drug profiles with Hill equation dose-response, transporter targets (SERT/DAT/NET), receptor bindings, two-phase PK lifecycle (absorption + elimination), ClearanceRateSnapshot for drift-free rate restoration. Mechanisms: Agonist, PartialAgonist, Antagonist, PositiveAllostericModulator, ReuptakeInhibitor. Withdrawal/rebound dynamics (receptor deviation feeds back into NT baselines post-drug)
- **pharmacology** — 6 preset drugs: fluoxetine (SSRI, 5-day effective half-life), sertraline (SSRI), diazepam (BZD PAM), alprazolam (BZD PAM), amphetamine (DA/NE releaser + DAT/NET blocker), methylphenidate (DAT/NET blocker)

### Body Systems
- **inflammation** — Microglial activation, pro-inflammatory cytokines, neuroinflammation, sickness behavior, IDO tryptophan depletion → reduced serotonin synthesis
- **gut_brain** — Enteric serotonin (95% of body 5-HT), vagal tone, microbiome diversity, central serotonin modifier
- **autonomic** — Sympathetic/parasympathetic reciprocal inhibition, HRV proxy
- **eeg** — Delta/theta/alpha/beta/gamma band powers derived from brain state (sleep stage, PFC focus, meditation, amygdala activation)

### Integration
- **brain** — Unified `BrainState` with 30+ step causal tick ordering all subsystems. `AgeProfile` (PFC maturation, DA decline, sleep depth), `InteroceptiveState` (Seth 2013 predictive processing), `SexHormoneState` (estradiol→5-HT, testosterone→amygdala), `TdLearner` (Schultz 1997 TD learning), `OpponentProcess` (Solomon & Corbit 1974 hedonic adaptation)
- **coupling** — 15+ cross-module coupling functions with tunable `CouplingParams`, `RegionCouplingParams`, `CircuitGainParams`. Sleep→NT, circadian→HPA, DMN→HPA, arousal→circuit, NT→all regions, amygdala↔PFC (bidirectional), amygdala→HPA, inflammation→HPA/NT, gut-brain→inflammation/NT, autonomic coupling, interoceptive coupling
- **bridge** — 28-field `BrainMoodEffect` composite struct: mood offset, reward sensitivity, arousal, anxiety, focus, pain dampening, stress multiplier, burnout, energy penalty, recovery rate, drowsiness, rumination stress, regulation boost, growth plasticity, executive control, working memory, fear level, emotional salience, learning rate, action drive, habit level, motor quality, sickness behavior, sympathetic, parasympathetic, HRV, interoceptive anxiety, seasonal modifier

### Infrastructure
- **error** — `MastishkError` with variants: LevelOutOfRange, InvalidCircuit, InvalidSleepTransition, NegativeTimeDelta, InvalidDrugParameter
- **logging** — Optional structured logging via `MASTISHK_LOG` env var (feature-gated)
- 228 tests (192 unit + 36 integration), 7 criterion benchmarks
- All types `Serialize + Deserialize + Clone + Debug` with `#[serde(default)]` backward compatibility
- All public enums `#[non_exhaustive]`, all pure functions `#[must_use]`, all hot paths `#[inline]`
- `tracing` instrumentation on all state-mutating operations
- Two external domain audits with 13+ accuracy improvements implemented
