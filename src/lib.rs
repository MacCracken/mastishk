//! Mastishk — Computational Neuroscience Engine
//!
//! **Mastishk** (Sanskrit: मस्तिष्क — brain) provides computational models of
//! neuroscience for the AGNOS ecosystem. Neurotransmitter dynamics, neural circuit
//! simulation, sleep architecture, HPA axis stress response, and default mode
//! network modeling.
//!
//! # Architecture
//!
//! Six domain modules:
//!
//! - [`neurotransmitter`] — Monoamine dynamics (serotonin, dopamine, norepinephrine),
//!   GABA/glutamate balance, neuropeptides (oxytocin, endorphins), acetylcholine,
//!   BDNF neuroplasticity. Synthesis, reuptake, degradation kinetics.
//! - [`circuit`] — Neural circuit primitives: excitatory/inhibitory populations,
//!   firing rates, synaptic weights, Hebbian learning, lateral inhibition.
//! - [`sleep`] — Sleep architecture: NREM stages 1-3, REM cycling, adenosine
//!   buildup (Process S), sleep debt, ultradian 90-min cycles.
//! - [`hpa`] — Hypothalamic-pituitary-adrenal axis: CRH → ACTH → cortisol
//!   cascade, negative feedback, chronic stress adaptation, allostatic load.
//! - [`dmn`] — Default mode network: self-referential processing, mind-wandering,
//!   meditation suppression, task-positive network switching.
//! - [`chronobiology`] — Melatonin synthesis from light input, cortisol circadian
//!   rhythm (CAR), core body temperature oscillation, SCN pacemaker model.
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

pub mod error;
pub mod neurotransmitter;
pub mod circuit;
pub mod sleep;
pub mod hpa;
pub mod dmn;
pub mod chronobiology;

#[cfg(feature = "logging")]
pub mod logging;

pub use error::MastishkError;
