//! Neuroinflammation — microglial activation, cytokines, sickness behavior.
//!
//! Models the neuroimmune interface: infection/stress activates microglia,
//! which release pro-inflammatory cytokines (IL-1β, IL-6, TNF-α simplified
//! to a single value). Cytokines drive sickness behavior (fatigue, anhedonia)
//! and deplete tryptophan via IDO enzyme → reduced serotonin synthesis.

use serde::{Deserialize, Serialize};

use crate::error::{MastishkError, validate_dt};

/// Neuroinflammation and microglial activation state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflammationState {
    /// Microglial activation (0.0–1.0). Resting → activated by stress/infection.
    pub microglial_activation: f32,
    /// Pro-inflammatory cytokine level (0.0–1.0). Simplified IL-1β/IL-6/TNF-α.
    pub cytokine_level: f32,
    /// Overall neuroinflammation (0.0–1.0).
    pub neuroinflammation: f32,
    /// Sickness behavior intensity (0.0–1.0). Fatigue, anhedonia, social withdrawal.
    pub sickness_behavior: f32,
}

impl Default for InflammationState {
    fn default() -> Self {
        Self {
            microglial_activation: 0.05,
            cytokine_level: 0.05,
            neuroinflammation: 0.0,
            sickness_behavior: 0.0,
        }
    }
}

impl InflammationState {
    /// Apply an infection or injury signal (0.0–1.0).
    #[inline]
    pub fn infect(&mut self, intensity: f32) {
        self.microglial_activation = (self.microglial_activation + intensity * 0.4).min(1.0);
        self.cytokine_level = (self.cytokine_level + intensity * 0.3).min(1.0);
        tracing::debug!(intensity, "infection/injury signal");
    }

    /// Tick inflammation dynamics.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        let alpha = 1.0 - (-0.05 * dt).exp();

        // Microglia drives cytokines
        let cytokine_target = self.microglial_activation * 0.8;
        self.cytokine_level += (cytokine_target - self.cytokine_level) * alpha;

        // Cytokines drive neuroinflammation
        self.neuroinflammation += (self.cytokine_level * 0.7 - self.neuroinflammation) * alpha;

        // Sickness behavior follows neuroinflammation
        self.sickness_behavior += (self.neuroinflammation * 0.9 - self.sickness_behavior) * alpha;

        // All decay toward resting
        self.microglial_activation +=
            (0.05 - self.microglial_activation) * (1.0 - (-0.02 * dt).exp());
        self.cytokine_level = self.cytokine_level.clamp(0.0, 1.0);
        self.neuroinflammation = self.neuroinflammation.clamp(0.0, 1.0);
        self.sickness_behavior = self.sickness_behavior.clamp(0.0, 1.0);

        tracing::trace!(cytokines = self.cytokine_level, "inflammation tick");
        Ok(())
    }

    /// Tryptophan depletion factor (0.0–1.0). High inflammation depletes
    /// tryptophan via IDO enzyme → less serotonin synthesis substrate.
    #[inline]
    #[must_use]
    pub fn tryptophan_depletion(&self) -> f32 {
        (self.neuroinflammation * 0.6).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_low_inflammation() {
        let s = InflammationState::default();
        assert!(s.neuroinflammation < 0.01);
        assert!(s.sickness_behavior < 0.01);
    }

    #[test]
    fn test_infection_raises_cytokines() {
        let mut s = InflammationState::default();
        s.infect(0.8);
        // Check shortly after infection (10 seconds)
        for _ in 0..10 {
            s.tick(1.0).unwrap();
        }
        assert!(s.cytokine_level > 0.1, "cytokines={}", s.cytokine_level);
        assert!(s.sickness_behavior > 0.0);
    }

    #[test]
    fn test_tryptophan_depletion() {
        let mut s = InflammationState::default();
        assert!(s.tryptophan_depletion() < 0.01);
        s.neuroinflammation = 0.8;
        assert!(s.tryptophan_depletion() > 0.3);
    }

    #[test]
    fn test_inflammation_decays() {
        let mut s = InflammationState::default();
        s.infect(1.0);
        for _ in 0..500 {
            s.tick(1.0).unwrap();
        }
        assert!(s.microglial_activation < 0.2);
    }

    #[test]
    fn test_serde_roundtrip() {
        let s = InflammationState::default();
        let json = serde_json::to_string(&s).unwrap();
        let s2: InflammationState = serde_json::from_str(&json).unwrap();
        assert!((s2.microglial_activation - s.microglial_activation).abs() < f32::EPSILON);
    }
}
