# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **all modules** — All `tick` methods now return `Result<(), MastishkError>`, rejecting negative time deltas with `NegativeTimeDelta` error
- **circuit** — `add_synapse` now returns `Result`, validating population indices at creation time
- **hpa** — Replaced Euler integration with exponential decay for stable behavior at large dt values
- **chronobiology** — `CircadianState::default()` now derives rhythm values from `update_rhythms()` instead of hardcoded approximations

### Fixed
- **sleep** — Fixed `total_sleep` units: was incorrectly multiplying hours by 3600 (mixing seconds/hours); now consistently uses hours
- **docs** — Removed claims of Hebbian learning and lateral inhibition from circuit module (not yet implemented)

### Added
- **all modules** — `#[inline]` on all hot-path `tick`, getter, and computed property functions
- **all modules** — `tracing` instrumentation on all public state-mutating operations (`debug!` for discrete events, `trace!` for per-tick updates)
- **all modules** — Comprehensive test coverage: negative dt rejection, boundary conditions, untested getters (`alertness`, `consolidation_rate`, `is_stressed`, `is_chronic`, `self_referential`, `reward_sensitivity`, `plasticity_rate`), edge cases. Test count: 42 → 68

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
