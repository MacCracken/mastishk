//! Default mode network — self-referential processing, mind-wandering, meditation.
//!
//! The DMN activates during rest and self-referential thought, deactivates during
//! focused external tasks. Models the DMN ↔ task-positive network (TPN) anticorrelation.

use crate::error::{MastishkError, validate_dt};
use serde::{Deserialize, Serialize};

/// DMN/TPN balance state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmnState {
    /// DMN activation (0.0–1.0). High = self-referential, mind-wandering.
    pub dmn_activation: f32,
    /// TPN activation (0.0–1.0). High = external focus, task engagement.
    pub tpn_activation: f32,
    /// Meditation depth (0.0–1.0). Sustained meditation suppresses DMN.
    pub meditation_depth: f32,
    /// Rumination tendency (0.0–1.0). High DMN + negative valence = rumination.
    pub rumination: f32,
}

impl Default for DmnState {
    fn default() -> Self {
        Self {
            dmn_activation: 0.5,
            tpn_activation: 0.3,
            meditation_depth: 0.0,
            rumination: 0.0,
        }
    }
}

impl DmnState {
    /// Engage in external task — shifts balance toward TPN.
    #[inline]
    pub fn engage_task(&mut self, intensity: f32) {
        self.tpn_activation = (self.tpn_activation + intensity * 0.3).min(1.0);
        self.dmn_activation = (self.dmn_activation - intensity * 0.2).max(0.0);
        tracing::debug!(
            intensity,
            dmn = self.dmn_activation,
            tpn = self.tpn_activation,
            "task engaged"
        );
    }

    /// Rest / disengage — shifts balance toward DMN.
    #[inline]
    pub fn rest(&mut self, duration_factor: f32) {
        self.dmn_activation = (self.dmn_activation + duration_factor * 0.2).min(1.0);
        self.tpn_activation = (self.tpn_activation - duration_factor * 0.1).max(0.0);
        tracing::debug!(duration_factor, dmn = self.dmn_activation, "resting");
    }

    /// Meditate — suppresses DMN without engaging TPN.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn meditate(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        tracing::trace!(dt, depth = self.meditation_depth, "meditating");
        self.meditation_depth = (self.meditation_depth + 0.05 * dt).min(1.0);
        self.dmn_activation = (self.dmn_activation - self.meditation_depth * 0.1 * dt).max(0.0);
        self.rumination = (self.rumination - self.meditation_depth * 0.15 * dt).max(0.0);
        Ok(())
    }

    /// Update rumination based on DMN activation and emotional valence.
    /// Negative valence + high DMN = rumination spiral.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn update_rumination(
        &mut self,
        emotional_valence: f32,
        dt: f32,
    ) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        if emotional_valence < 0.0 && self.dmn_activation > 0.5 {
            let push = self.dmn_activation * (-emotional_valence) * 0.1 * dt;
            self.rumination = (self.rumination + push).min(1.0);
        } else {
            self.rumination = (self.rumination - 0.02 * dt).max(0.0);
        }
        Ok(())
    }

    /// Net self-referential processing level.
    #[inline]
    #[must_use]
    pub fn self_referential(&self) -> f32 {
        (self.dmn_activation - self.tpn_activation * 0.5).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_engagement_suppresses_dmn() {
        let mut d = DmnState::default();
        d.engage_task(1.0);
        assert!(d.tpn_activation > 0.5);
        assert!(d.dmn_activation < 0.5);
    }

    #[test]
    fn test_rest_activates_dmn() {
        let mut d = DmnState {
            dmn_activation: 0.3,
            ..Default::default()
        };
        d.rest(1.0);
        assert!(d.dmn_activation > 0.3);
    }

    #[test]
    fn test_meditation_suppresses_dmn() {
        let mut d = DmnState {
            dmn_activation: 0.8,
            ..Default::default()
        };
        for _ in 0..20 {
            d.meditate(1.0).unwrap();
        }
        assert!(d.dmn_activation < 0.5);
    }

    #[test]
    fn test_rumination() {
        let mut d = DmnState {
            dmn_activation: 0.8,
            ..Default::default()
        };
        d.update_rumination(-0.7, 5.0).unwrap();
        assert!(d.rumination > 0.0);
    }

    #[test]
    fn test_serde_roundtrip() {
        let d = DmnState::default();
        let json = serde_json::to_string(&d).unwrap();
        let d2: DmnState = serde_json::from_str(&json).unwrap();
        assert!((d2.dmn_activation - d.dmn_activation).abs() < f32::EPSILON);
    }

    #[test]
    fn test_negative_dt_rejected() {
        let mut d = DmnState::default();
        assert!(d.meditate(-1.0).is_err());
        assert!(d.update_rumination(-0.5, -1.0).is_err());
    }

    #[test]
    fn test_self_referential() {
        let d = DmnState::default();
        let sr = d.self_referential();
        assert!((0.0..=1.0).contains(&sr));

        // High DMN, low TPN → high self-referential
        let high_dmn = DmnState {
            dmn_activation: 0.9,
            tpn_activation: 0.1,
            ..Default::default()
        };
        assert!(high_dmn.self_referential() > 0.5);

        // Low DMN, high TPN → low self-referential
        let low_dmn = DmnState {
            dmn_activation: 0.1,
            tpn_activation: 0.9,
            ..Default::default()
        };
        assert!((low_dmn.self_referential() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rumination_decays_with_positive_valence() {
        let mut d = DmnState {
            dmn_activation: 0.8,
            rumination: 0.5,
            ..Default::default()
        };
        d.update_rumination(0.5, 5.0).unwrap();
        assert!(d.rumination < 0.5);
    }
}
