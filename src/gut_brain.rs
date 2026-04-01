//! Gut-brain axis — enteric serotonin, vagal tone, microbiome diversity.
//!
//! ~95% of body serotonin is produced in the gut. The vagus nerve mediates
//! bidirectional gut-brain communication. Microbiome diversity affects
//! inflammation and serotonin production.

use serde::{Deserialize, Serialize};

use crate::error::{MastishkError, validate_dt};

/// Gut-brain axis state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GutBrainState {
    /// Gut serotonin production (0.0–1.0). ~95% of body 5-HT is enteric.
    pub gut_serotonin: f32,
    /// Vagal tone (0.0–1.0). Parasympathetic vagus nerve strength.
    pub vagal_tone: f32,
    /// Microbiome diversity (0.0–1.0). Trait-like, slow-changing.
    pub microbiome_diversity: f32,
    /// Interoceptive signal from gut (0.0–1.0). Visceral afferent — "gut feelings".
    pub interoceptive_signal: f32,
}

impl Default for GutBrainState {
    fn default() -> Self {
        Self {
            gut_serotonin: 0.5,
            vagal_tone: 0.5,
            microbiome_diversity: 0.6,
            interoceptive_signal: 0.3,
        }
    }
}

impl GutBrainState {
    /// Tick gut-brain dynamics.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        let alpha = 1.0 - (-0.02 * dt).exp();

        // Gut serotonin modulated by microbiome diversity
        let serotonin_target = 0.3 + self.microbiome_diversity * 0.4;
        self.gut_serotonin += (serotonin_target - self.gut_serotonin) * alpha;

        // Vagal tone decays toward resting
        self.vagal_tone += (0.5 - self.vagal_tone) * (1.0 - (-0.01 * dt).exp());

        // Interoceptive signal tracks gut state
        self.interoceptive_signal += (self.gut_serotonin * 0.5 - self.interoceptive_signal) * alpha;

        self.gut_serotonin = self.gut_serotonin.clamp(0.0, 1.0);
        self.vagal_tone = self.vagal_tone.clamp(0.0, 1.0);
        tracing::trace!(
            gut_5ht = self.gut_serotonin,
            vagal = self.vagal_tone,
            "gut-brain tick"
        );
        Ok(())
    }

    /// Central serotonin availability modifier (0.8–1.2).
    #[inline]
    #[must_use]
    pub fn central_serotonin_modifier(&self) -> f32 {
        0.8 + self.gut_serotonin * 0.4
    }

    /// Stress regulation bonus from vagal tone (0.0–0.3).
    #[inline]
    #[must_use]
    pub fn vagal_stress_buffer(&self) -> f32 {
        self.vagal_tone * 0.3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_central_serotonin_modifier_range() {
        let s = GutBrainState::default();
        let m = s.central_serotonin_modifier();
        assert!((0.8..=1.2).contains(&m));
    }

    #[test]
    fn test_vagal_stress_buffer() {
        let s = GutBrainState::default();
        assert!(s.vagal_stress_buffer() > 0.0);
        assert!(s.vagal_stress_buffer() <= 0.3);
    }

    #[test]
    fn test_serde_roundtrip() {
        let s = GutBrainState::default();
        let json = serde_json::to_string(&s).unwrap();
        let s2: GutBrainState = serde_json::from_str(&json).unwrap();
        assert!((s2.gut_serotonin - s.gut_serotonin).abs() < f32::EPSILON);
    }
}
