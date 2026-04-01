//! Error types for mastishk.

use thiserror::Error;

/// Errors that can occur in neuroscience computations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MastishkError {
    /// Neurotransmitter level out of physiological range.
    #[error("neurotransmitter level out of range: {name} = {value} (expected {min}..={max})")]
    LevelOutOfRange {
        name: String,
        value: f32,
        min: f32,
        max: f32,
    },

    /// Invalid circuit configuration.
    #[error("invalid circuit: {0}")]
    InvalidCircuit(String),

    /// Sleep stage transition not valid.
    #[error("invalid sleep transition: {from:?} -> {to:?}")]
    InvalidSleepTransition {
        from: String,
        to: String,
    },

    /// Negative time delta.
    #[error("negative time delta: {0}")]
    NegativeTimeDelta(f32),
}
