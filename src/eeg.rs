//! EEG signal generation — band power derivation from brain state.
//!
//! Computes EEG frequency band powers as observable correlates of brain state.
//! Not simulated from spike trains — derived from sleep stage, arousal,
//! PFC activity, meditation depth, and amygdala activation.
//!
//! # Bands
//!
//! - **Delta** (0.5–4 Hz): Deep sleep (NREM3)
//! - **Theta** (4–8 Hz): Light sleep, meditation, memory encoding
//! - **Alpha** (8–12 Hz): Relaxed wakefulness, eyes closed
//! - **Beta** (12–30 Hz): Active thinking, focused attention
//! - **Gamma** (30–100 Hz): Active processing, perceptual binding

use serde::{Deserialize, Serialize};

use crate::error::{MastishkError, validate_dt};

/// EEG frequency band identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EegBand {
    Delta,
    Theta,
    Alpha,
    Beta,
    Gamma,
}

/// EEG band power state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EegState {
    /// Delta (0.5–4 Hz) power (0.0–1.0). Dominant in NREM3.
    pub delta: f32,
    /// Theta (4–8 Hz) power (0.0–1.0). Light sleep, meditation.
    pub theta: f32,
    /// Alpha (8–12 Hz) power (0.0–1.0). Relaxed wakefulness.
    pub alpha: f32,
    /// Beta (12–30 Hz) power (0.0–1.0). Active thinking, focus.
    pub beta: f32,
    /// Gamma (30–100 Hz) power (0.0–1.0). Active processing.
    pub gamma: f32,
}

impl Default for EegState {
    fn default() -> Self {
        // Relaxed awake defaults
        Self {
            delta: 0.1,
            theta: 0.15,
            alpha: 0.5,
            beta: 0.3,
            gamma: 0.1,
        }
    }
}

impl EegState {
    /// Smoothly transition band powers toward target values.
    #[inline]
    pub fn tick_toward(&mut self, target: &EegState, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        let alpha = 1.0 - (-0.5 * dt).exp(); // ~2 second transition

        self.delta += (target.delta - self.delta) * alpha;
        self.theta += (target.theta - self.theta) * alpha;
        self.alpha += (target.alpha - self.alpha) * alpha;
        self.beta += (target.beta - self.beta) * alpha;
        self.gamma += (target.gamma - self.gamma) * alpha;

        tracing::trace!(
            delta = self.delta,
            alpha_band = self.alpha,
            beta = self.beta,
            "EEG tick"
        );
        Ok(())
    }

    /// Dominant frequency band (highest power).
    #[inline]
    #[must_use]
    pub fn dominant_band(&self) -> EegBand {
        let bands = [
            (self.delta, EegBand::Delta),
            (self.theta, EegBand::Theta),
            (self.alpha, EegBand::Alpha),
            (self.beta, EegBand::Beta),
            (self.gamma, EegBand::Gamma),
        ];
        bands
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal))
            .map(|&(_, band)| band)
            .unwrap_or(EegBand::Alpha)
    }

    /// Total power (sum of all bands).
    #[inline]
    #[must_use]
    pub fn total_power(&self) -> f32 {
        self.delta + self.theta + self.alpha + self.beta + self.gamma
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_alpha_dominant() {
        let s = EegState::default();
        assert_eq!(s.dominant_band(), EegBand::Alpha);
    }

    #[test]
    fn test_tick_toward_converges() {
        let mut s = EegState::default();
        let target = EegState {
            delta: 0.8,
            theta: 0.1,
            alpha: 0.05,
            beta: 0.02,
            gamma: 0.01,
        };
        for _ in 0..100 {
            s.tick_toward(&target, 1.0).unwrap();
        }
        assert!((s.delta - 0.8).abs() < 0.05);
        assert_eq!(s.dominant_band(), EegBand::Delta);
    }

    #[test]
    fn test_total_power() {
        let s = EegState::default();
        assert!(s.total_power() > 0.0);
    }

    #[test]
    fn test_serde_roundtrip() {
        let s = EegState::default();
        let json = serde_json::to_string(&s).unwrap();
        let s2: EegState = serde_json::from_str(&json).unwrap();
        assert!((s2.alpha - s.alpha).abs() < f32::EPSILON);
    }
}
