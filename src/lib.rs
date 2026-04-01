//! Mastishk — Computational Neuroscience Engine
//!
//! **Mastishk** (Sanskrit: मस्तिष्क — brain) provides world-class behavioral-timescale
//! brain simulation for the AGNOS ecosystem. 12 neurotransmitters, 20 receptor subtypes,
//! 10 brain regions, pharmacology, spiking networks, and a 30+ step integrated brain tick.
//!
//! # Modules (19)
//!
//! **Core Neuroscience:**
//! - [`neurotransmitter`] — 12 transmitters with tonic + phasic dopamine
//! - [`receptor`] — 20 receptor subtypes with desensitization/upregulation ODE
//! - [`circuit`] — Rate-model populations, synaptic propagation, Hebbian plasticity
//! - [`spiking`] — Izhikevich + LIF neurons, SpikingNetwork, STDP (standalone ms-timescale)
//! - [`sleep`] — Borbely two-process model, automated ultradian cycle
//! - [`hpa`] — CRH→ACTH→cortisol cascade, allostatic load, sensitization/kindling
//! - [`dmn`] — DMN/TPN anticorrelation, rumination, meditation
//! - [`chronobiology`] — SCN pacemaker, melatonin, asymmetric cortisol CAR, photoperiod
//!
//! **Brain Regions:**
//! - [`regions`] — PFC, amygdala, hippocampus, basal ganglia, cerebellum, VTA/NAc,
//!   locus coeruleus, raphe, ACC, insula
//!
//! **Body Systems:**
//! - [`inflammation`] — Microglia, cytokines, sickness behavior, IDO pathway
//! - [`gut_brain`] — Enteric serotonin, vagal tone, microbiome
//! - [`autonomic`] — Sympathetic/parasympathetic, HRV proxy
//! - [`eeg`] — Delta/theta/alpha/beta/gamma band powers
//!
//! **Pharmacology:**
//! - [`pharmacology`] — Drug profiles, Hill equation, transporters, PK lifecycle,
//!   partial agonism, withdrawal/rebound. 6 preset drugs
//!
//! **Integration:**
//! - [`coupling`] — 15+ cross-module coupling functions
//! - [`brain`] — Unified [`brain::BrainState`] with 30+ step tick, AgeProfile,
//!   InteroceptiveState, TdLearner, OpponentProcess
//! - [`bridge`] — 28-field [`bridge::BrainMoodEffect`] for downstream consumers
//!
//! # Consumers
//!
//! ```text
//! mastishk (this) → bridge.rs → f64 outputs
//!   ├─→ bhava     — emotion/personality
//!   ├─→ bodh      — psychology/cognition
//!   ├─→ kiran     — game engine (provides dt)
//!   ├─→ joshua    — agent characters
//!   └─→ agnosai   — agent orchestration
//! ```

pub mod autonomic;
pub mod brain;
pub mod bridge;
pub mod chronobiology;
pub mod circuit;
pub mod coupling;
pub mod dmn;
pub mod eeg;
pub mod error;
pub mod gut_brain;
pub mod hpa;
pub mod inflammation;
pub mod neurotransmitter;
pub mod pharmacology;
pub mod receptor;
pub mod regions;
pub mod sleep;
pub mod spiking;

#[cfg(feature = "logging")]
pub mod logging;

pub use error::MastishkError;
