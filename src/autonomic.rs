//! Autonomic nervous system — sympathetic/parasympathetic balance, HRV.
//!
//! Models the reciprocal inhibition between sympathetic (fight-or-flight)
//! and parasympathetic (rest-and-digest) branches. Heart rate variability
//! (HRV) serves as a proxy for autonomic regulation capacity.

use serde::{Deserialize, Serialize};

use crate::error::{MastishkError, validate_dt};

/// Autonomic nervous system state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomicState {
    /// Sympathetic activation (0.0–1.0). Fight-or-flight.
    pub sympathetic: f32,
    /// Parasympathetic activation (0.0–1.0). Rest-and-digest.
    pub parasympathetic: f32,
    /// Heart rate variability proxy (0.0–1.0). Higher = better regulation.
    pub hrv: f32,
}

impl Default for AutonomicState {
    fn default() -> Self {
        Self {
            sympathetic: 0.3,
            parasympathetic: 0.5,
            hrv: 0.6,
        }
    }
}

impl AutonomicState {
    /// Tick autonomic dynamics. Branches reciprocally inhibit; HRV derived.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        let alpha = 1.0 - (-0.08 * dt).exp();

        // Reciprocal inhibition: each branch suppresses the other
        let sym_target = (0.3 - self.parasympathetic * 0.2).max(0.05);
        let para_target = (0.5 - self.sympathetic * 0.3).max(0.05);

        self.sympathetic += (sym_target - self.sympathetic) * alpha;
        self.parasympathetic += (para_target - self.parasympathetic) * alpha;

        self.sympathetic = self.sympathetic.clamp(0.0, 1.0);
        self.parasympathetic = self.parasympathetic.clamp(0.0, 1.0);

        // HRV: higher when parasympathetic dominant, lower under sympathetic stress
        self.hrv = (self.parasympathetic * 0.7 + (1.0 - self.sympathetic) * 0.3).clamp(0.0, 1.0);

        tracing::trace!(
            sym = self.sympathetic,
            para = self.parasympathetic,
            hrv = self.hrv,
            "autonomic tick"
        );
        Ok(())
    }

    /// Autonomic balance: >0.5 parasympathetic dominant, <0.5 sympathetic dominant.
    #[inline]
    #[must_use]
    pub fn balance(&self) -> f32 {
        let total = self.sympathetic + self.parasympathetic;
        if total > f32::EPSILON {
            self.parasympathetic / total
        } else {
            0.5
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_parasympathetic_dominant() {
        let s = AutonomicState::default();
        assert!(s.balance() > 0.5);
    }

    #[test]
    fn test_hrv_reflects_balance() {
        let mut s = AutonomicState {
            sympathetic: 0.9,
            parasympathetic: 0.1,
            ..Default::default()
        };
        s.tick(0.0).unwrap();
        assert!(s.hrv < 0.4);
    }

    #[test]
    fn test_serde_roundtrip() {
        let s = AutonomicState::default();
        let json = serde_json::to_string(&s).unwrap();
        let s2: AutonomicState = serde_json::from_str(&json).unwrap();
        assert!((s2.sympathetic - s.sympathetic).abs() < f32::EPSILON);
    }
}
