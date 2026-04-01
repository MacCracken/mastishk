//! HPA axis — hypothalamic-pituitary-adrenal stress response.
//!
//! Models the CRH → ACTH → cortisol cascade with negative feedback,
//! chronic stress adaptation, and allostatic load accumulation.

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
        }
    }
}

impl HpaState {
    /// Apply a stressor (0.0–1.0 intensity). Triggers CRH release.
    pub fn stress(&mut self, intensity: f32) {
        self.crh = (self.crh + intensity * 0.3).min(1.0);
    }

    /// Tick the cascade: CRH drives ACTH, ACTH drives cortisol,
    /// cortisol feeds back to suppress CRH.
    pub fn tick(&mut self, dt: f32) {
        // CRH → ACTH
        self.acth += (self.crh * 0.5 - self.acth * 0.3) * dt;
        self.acth = self.acth.clamp(0.0, 1.0);

        // ACTH → cortisol
        self.cortisol += (self.acth * 0.4 - (self.cortisol - self.cortisol_baseline) * 0.2) * dt;
        self.cortisol = self.cortisol.clamp(0.0, 1.0);

        // Negative feedback: cortisol suppresses CRH
        self.crh -= self.cortisol * self.feedback_gain * 0.1 * dt;
        self.crh = self.crh.clamp(0.0, 1.0);

        // Allostatic load accumulates when cortisol is above baseline
        if self.cortisol > self.cortisol_baseline + 0.1 {
            self.allostatic_load += (self.cortisol - self.cortisol_baseline) * 0.01 * dt;
        }
        // Slow recovery when cortisol is low
        if self.cortisol < self.cortisol_baseline + 0.05 {
            self.allostatic_load = (self.allostatic_load - 0.002 * dt).max(0.0);
        }
    }

    /// Whether the HPA axis is in an acute stress response.
    #[must_use]
    pub fn is_stressed(&self) -> bool {
        self.cortisol > self.cortisol_baseline + 0.15
    }

    /// Chronic stress indicator — allostatic load above threshold.
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
        // Tick to propagate cascade
        for _ in 0..10 {
            h.tick(0.5);
        }
        assert!(h.cortisol > h.cortisol_baseline);
    }

    #[test]
    fn test_negative_feedback() {
        let mut h = HpaState::default();
        h.stress(1.0);
        for _ in 0..50 {
            h.tick(0.5);
        }
        // CRH should be suppressed by cortisol feedback
        assert!(h.crh < 0.5);
    }

    #[test]
    fn test_allostatic_load() {
        let mut h = HpaState::default();
        for _ in 0..100 {
            h.stress(0.5);
            h.tick(1.0);
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
}
