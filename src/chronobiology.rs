//! Chronobiology — melatonin, cortisol rhythm, core body temperature, SCN.
//!
//! Models the suprachiasmatic nucleus (SCN) pacemaker and its downstream
//! hormonal rhythms. Light input entrains the SCN, which drives melatonin
//! synthesis (dark-phase), cortisol awakening response (CAR), and core
//! body temperature oscillation.

use crate::error::{MastishkError, validate_dt};
use serde::{Deserialize, Serialize};

/// Circadian phase state driven by the SCN pacemaker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircadianState {
    /// Current circadian phase (0.0–24.0 hours, 0.0 = midnight).
    pub phase_hours: f32,
    /// Melatonin level (0.0–1.0). Peaks in darkness (~2-4 AM).
    pub melatonin: f32,
    /// Cortisol level from CAR (0.0–1.0). Peaks ~30min after waking.
    pub cortisol_circadian: f32,
    /// Core body temperature deviation from mean (°C, typically ±0.5).
    pub temperature_deviation: f32,
    /// Light exposure (lux, used for entrainment).
    pub light_exposure: f32,
}

impl Default for CircadianState {
    fn default() -> Self {
        let mut state = Self {
            phase_hours: 8.0, // morning
            melatonin: 0.0,
            cortisol_circadian: 0.0,
            temperature_deviation: 0.0,
            light_exposure: 500.0,
        };
        state.update_rhythms();
        state
    }
}

impl CircadianState {
    /// Advance the circadian clock by `dt` hours.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt_hours < 0.0`.
    #[inline]
    pub fn tick(&mut self, dt_hours: f32) -> Result<(), MastishkError> {
        validate_dt(dt_hours)?;
        tracing::trace!(
            dt_hours,
            phase = self.phase_hours,
            "ticking circadian clock"
        );
        self.phase_hours = (self.phase_hours + dt_hours) % 24.0;
        self.update_rhythms();
        Ok(())
    }

    /// Set current light exposure (lux).
    #[inline]
    pub fn set_light(&mut self, lux: f32) {
        self.light_exposure = lux.max(0.0);
        tracing::debug!(lux = self.light_exposure, "light exposure set");
    }

    /// Update hormonal rhythms based on current phase.
    fn update_rhythms(&mut self) {
        let h = self.phase_hours;

        // Melatonin: peaks at ~3 AM (phase 3.0), suppressed by light
        let melatonin_base = melatonin_curve(h);
        let light_suppression = if self.light_exposure > 100.0 {
            (1.0 - (self.light_exposure / 2000.0).min(1.0)) * 0.7
        } else {
            1.0
        };
        self.melatonin = (melatonin_base * light_suppression).clamp(0.0, 1.0);

        // Cortisol: peaks at ~8 AM (CAR), nadir at midnight
        self.cortisol_circadian = cortisol_curve(h);

        // Core body temperature: nadir ~4:30 AM, peak ~7 PM
        self.temperature_deviation = temperature_curve(h);
    }

    /// Alertness derived from circadian state (inverse melatonin, plus temperature).
    #[inline]
    #[must_use]
    pub fn alertness(&self) -> f32 {
        ((1.0 - self.melatonin) * 0.6 + (self.temperature_deviation + 0.5) * 0.4).clamp(0.0, 1.0)
    }
}

/// Melatonin curve: cosine with peak at 3 AM.
#[inline]
#[must_use]
fn melatonin_curve(hour: f32) -> f32 {
    // Peak at 3.0, trough at 15.0
    let phase = (hour - 3.0) * core::f32::consts::TAU / 24.0;
    (phase.cos() * 0.5 + 0.5).clamp(0.0, 1.0)
}

/// Cortisol CAR curve: cosine with peak at 8 AM.
#[inline]
#[must_use]
fn cortisol_curve(hour: f32) -> f32 {
    let phase = (hour - 8.0) * core::f32::consts::TAU / 24.0;
    (phase.cos() * 0.35 + 0.35).clamp(0.0, 1.0)
}

/// Core body temperature curve: cosine with peak at 19:00 (7 PM).
#[inline]
#[must_use]
fn temperature_curve(hour: f32) -> f32 {
    let phase = (hour - 19.0) * core::f32::consts::TAU / 24.0;
    phase.cos() * 0.5 // ±0.5°C
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_melatonin_peaks_at_night() {
        let night = melatonin_curve(3.0);
        let noon = melatonin_curve(15.0);
        assert!(night > noon);
    }

    #[test]
    fn test_cortisol_peaks_morning() {
        let morning = cortisol_curve(8.0);
        let midnight = cortisol_curve(0.0);
        assert!(morning > midnight);
    }

    #[test]
    fn test_tick_advances_phase() {
        let mut c = CircadianState::default();
        c.tick(6.0).unwrap();
        assert!((c.phase_hours - 14.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_light_suppresses_melatonin() {
        let mut dark = CircadianState {
            phase_hours: 3.0,
            light_exposure: 0.0,
            ..Default::default()
        };
        dark.tick(0.0).unwrap();
        let dark_mel = dark.melatonin;

        let mut bright = CircadianState {
            phase_hours: 3.0,
            light_exposure: 1000.0,
            ..Default::default()
        };
        bright.tick(0.0).unwrap();
        assert!(bright.melatonin < dark_mel);
    }

    #[test]
    fn test_serde_roundtrip() {
        let c = CircadianState::default();
        let json = serde_json::to_string(&c).unwrap();
        let c2: CircadianState = serde_json::from_str(&json).unwrap();
        assert!((c2.phase_hours - c.phase_hours).abs() < f32::EPSILON);
    }

    #[test]
    fn test_negative_dt_rejected() {
        let mut c = CircadianState::default();
        assert!(c.tick(-1.0).is_err());
    }

    #[test]
    fn test_alertness() {
        // Morning alertness should be high
        let mut morning = CircadianState::default(); // phase 8.0
        morning.tick(0.0).unwrap();
        let morning_alert = morning.alertness();

        // Middle of night alertness should be low
        let mut night = CircadianState {
            phase_hours: 3.0,
            light_exposure: 0.0,
            ..Default::default()
        };
        night.tick(0.0).unwrap();
        let night_alert = night.alertness();

        assert!(morning_alert > night_alert);
    }

    #[test]
    fn test_phase_wraps_at_24() {
        let mut c = CircadianState {
            phase_hours: 23.0,
            ..Default::default()
        };
        c.tick(3.0).unwrap();
        assert!((c.phase_hours - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_temperature_curve_range() {
        // Temperature deviation should be within ±0.5°C
        for hour in 0..24 {
            let t = temperature_curve(hour as f32);
            assert!((-0.5..=0.5).contains(&t), "temp at hour {hour}: {t}");
        }
    }
}
