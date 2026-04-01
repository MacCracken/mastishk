//! HPA axis — hypothalamic-pituitary-adrenal stress response.
//!
//! Models the CRH → ACTH → cortisol cascade with negative feedback,
//! chronic stress adaptation, and allostatic load accumulation.

use crate::error::{MastishkError, validate_dt};
use serde::{Deserialize, Serialize};

/// HPA axis state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpaState {
    /// CRH (corticotropin-releasing hormone) level (0.0–1.0).
    pub crh: f32,
    /// ACTH (adrenocorticotropic hormone) level (0.0–1.0).
    pub acth: f32,
    /// Cortisol level (0.0–1.0).
    pub cortisol: f32,
    /// Cortisol baseline — chronic stress raises this.
    pub cortisol_baseline: f32,
    /// Allostatic load — cumulative wear from chronic stress (0.0+).
    pub allostatic_load: f32,
    /// Negative feedback strength (higher = faster cortisol suppresses CRH).
    pub feedback_gain: f32,
    /// Stress sensitization / kindling (0.0–1.0). Chronic stress lowers the
    /// threshold for future HPA activation (Post 1992 kindling model).
    /// Driven by allostatic_load accumulation.
    #[serde(default)]
    pub sensitization: f32,
}

impl Default for HpaState {
    fn default() -> Self {
        Self {
            crh: 0.1,
            acth: 0.1,
            cortisol: 0.2,
            cortisol_baseline: 0.2,
            allostatic_load: 0.0,
            feedback_gain: 0.5,
            sensitization: 0.0,
        }
    }
}

impl HpaState {
    /// Apply a stressor (0.0–1.0 intensity). Triggers CRH release.
    ///
    /// Sensitization amplifies the effective intensity: repeated stress makes
    /// the HPA axis more reactive to future stressors (kindling effect).
    #[inline]
    pub fn stress(&mut self, intensity: f32) {
        let effective = intensity * (1.0 + self.sensitization * 0.5);
        self.crh = (self.crh + effective * 0.3).min(1.0);
        tracing::debug!(intensity, effective, crh = self.crh, "stressor applied");
    }

    /// Tick the cascade: CRH drives ACTH, ACTH drives cortisol,
    /// cortisol feeds back to suppress CRH.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        tracing::trace!(
            dt,
            cortisol = self.cortisol,
            crh = self.crh,
            allostatic_load = self.allostatic_load,
            "ticking HPA axis"
        );
        // CRH → ACTH (exponential approach, tau ≈ 300s / ~5 min)
        let acth_rate = 1.0 / 300.0;
        let acth_target = (self.crh * 1.67).min(1.0); // CRH drives ACTH target
        let acth_alpha = 1.0 - (-acth_rate * dt).exp();
        self.acth += (acth_target - self.acth) * acth_alpha;
        self.acth = self.acth.clamp(0.0, 1.0);

        // ACTH → cortisol (exponential approach, tau ≈ 600s / ~10 min)
        let cort_rate = 1.0 / 600.0;
        let cort_target = (self.cortisol_baseline + self.acth * 0.8).min(1.0);
        let cort_alpha = 1.0 - (-cort_rate * dt).exp();
        self.cortisol += (cort_target - self.cortisol) * cort_alpha;
        self.cortisol = self.cortisol.clamp(0.0, 1.0);

        // Negative feedback: cortisol suppresses CRH (tau ≈ 900s / ~15 min)
        let fb_rate = self.cortisol * self.feedback_gain * (1.0 / 900.0);
        self.crh *= (-fb_rate * dt).exp();
        self.crh = self.crh.clamp(0.0, 1.0);

        // Allostatic load accumulates when cortisol is above baseline
        if self.cortisol > self.cortisol_baseline + 0.1 {
            self.allostatic_load += (self.cortisol - self.cortisol_baseline) * 0.01 * dt;
        }
        // Slow recovery when cortisol is low
        if self.cortisol < self.cortisol_baseline + 0.05 {
            self.allostatic_load = (self.allostatic_load - 0.002 * dt).max(0.0);
        }

        // Stress sensitization driven by allostatic load (Post 1992 kindling)
        // High load → increased sensitization, low load → slow recovery
        let sens_target = (self.allostatic_load / 3.0).min(1.0);
        let sens_alpha = 1.0 - (-0.0001 * dt).exp(); // very slow (days timescale)
        self.sensitization += (sens_target - self.sensitization) * sens_alpha;
        self.sensitization = self.sensitization.clamp(0.0, 1.0);

        Ok(())
    }

    /// Whether the HPA axis is in an acute stress response.
    #[inline]
    #[must_use]
    pub fn is_stressed(&self) -> bool {
        self.cortisol > self.cortisol_baseline + 0.15
    }

    /// Chronic stress indicator — allostatic load above threshold.
    #[inline]
    #[must_use]
    pub fn is_chronic(&self) -> bool {
        self.allostatic_load > 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_response() {
        let mut h = HpaState::default();
        h.stress(0.8);
        assert!(h.crh > 0.2);
        // Tick 20 minutes (cascade needs ~15 min to propagate)
        for _ in 0..1200 {
            h.tick(1.0).unwrap();
        }
        assert!(h.cortisol > h.cortisol_baseline);
    }

    #[test]
    fn test_negative_feedback() {
        let mut h = HpaState::default();
        h.stress(1.0);
        // 30 minutes for feedback to take effect
        for _ in 0..1800 {
            h.tick(1.0).unwrap();
        }
        // CRH should be suppressed by cortisol feedback
        assert!(h.crh < 0.5);
    }

    #[test]
    fn test_allostatic_load() {
        let mut h = HpaState::default();
        // Repeated stress over 1 hour
        for _ in 0..3600 {
            h.stress(0.5);
            h.tick(1.0).unwrap();
        }
        assert!(h.allostatic_load > 0.0);
    }

    #[test]
    fn test_serde_roundtrip() {
        let h = HpaState::default();
        let json = serde_json::to_string(&h).unwrap();
        let h2: HpaState = serde_json::from_str(&json).unwrap();
        assert!((h2.cortisol - h.cortisol).abs() < f32::EPSILON);
    }

    #[test]
    fn test_negative_dt_rejected() {
        let mut h = HpaState::default();
        assert!(h.tick(-1.0).is_err());
    }

    #[test]
    fn test_is_stressed() {
        let mut h = HpaState::default();
        assert!(!h.is_stressed());
        h.stress(1.0);
        // 20 minutes for cortisol to rise
        for _ in 0..1200 {
            h.tick(1.0).unwrap();
        }
        assert!(h.is_stressed());
    }

    #[test]
    fn test_is_chronic() {
        let mut h = HpaState::default();
        assert!(!h.is_chronic());
        // Sustained stress for 2 hours
        for _ in 0..7200 {
            h.stress(0.8);
            h.tick(1.0).unwrap();
        }
        assert!(h.is_chronic());
    }

    #[test]
    fn test_allostatic_load_recovers() {
        let mut h = HpaState::default();
        // Build up load: stress every minute for 1 hour
        for _ in 0..60 {
            h.stress(0.8);
            h.tick(60.0).unwrap();
        }
        let peak_load = h.allostatic_load;
        assert!(peak_load > 0.0, "load should accumulate");
        // Let it recover: 12 hours no stress (in 1-minute steps)
        for _ in 0..720 {
            h.tick(60.0).unwrap();
        }
        assert!(
            h.allostatic_load < peak_load,
            "load={}, peak={}",
            h.allostatic_load,
            peak_load
        );
    }
}
