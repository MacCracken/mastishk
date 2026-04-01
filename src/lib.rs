//! Mastishk — Computational Neuroscience Engine
//!
//! **Mastishk** (Sanskrit: मस्तिष्क — brain) provides computational models of
//! neuroscience for the AGNOS ecosystem. Neurotransmitter dynamics, neural circuit
//! simulation, sleep architecture, HPA axis stress response, and default mode
//! network modeling.
//!
//! # Architecture
//!
//! Six domain modules plus integration:
//!
//! - [`neurotransmitter`] — Monoamine dynamics (serotonin, dopamine, norepinephrine),
//!   GABA/glutamate balance, neuropeptides (oxytocin, endorphins), acetylcholine,
//!   BDNF neuroplasticity. Synthesis, reuptake, degradation kinetics.
//! - [`circuit`] — Neural circuit primitives: excitatory/inhibitory populations,
//!   firing rates, synaptic weights, mean-field rate models.
//! - [`sleep`] — Sleep architecture: NREM stages 1-3, REM cycling, adenosine
//!   buildup (Process S), sleep debt, ultradian 90-min cycles.
//! - [`hpa`] — Hypothalamic-pituitary-adrenal axis: CRH → ACTH → cortisol
//!   cascade, negative feedback, chronic stress adaptation, allostatic load.
//! - [`dmn`] — Default mode network: self-referential processing, mind-wandering,
//!   meditation suppression, task-positive network switching.
//! - [`chronobiology`] — Melatonin synthesis from light input, cortisol circadian
//!   rhythm (CAR), core body temperature oscillation, SCN pacemaker model.
//! - [`coupling`] — Cross-module coupling functions: sleep→neurotransmitter,
//!   circadian→HPA, DMN→HPA, arousal→circuit. Composite metrics.
//! - [`brain`] — Unified [`brain::BrainState`] orchestrating all subsystems
//!   with a single `tick(dt)`.
//!
//! # Relationship to Other Crates
//!
//! ```text
//! mastishk (this) — neurotransmitter dynamics, neural circuits, sleep, HPA
//!   ↓ neurotransmitter levels feed into
//! bhava — emotion/personality (serotonin→mood, dopamine→preference, cortisol→stress)
//!   ↑ also bridges from
//! bodh — psychology (cognition, perception, learning models)
//! sharira — physiology (biomechanics, fatigue → energy)
//! rasayan — biochemistry (enzyme kinetics, metabolic pathways)
//! ```

pub mod brain;
pub mod chronobiology;
pub mod circuit;
pub mod coupling;
pub mod dmn;
pub mod error;
pub mod hpa;
pub mod neurotransmitter;
pub mod sleep;

#[cfg(feature = "logging")]
pub mod logging;

pub use error::MastishkError;
