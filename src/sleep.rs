//! Sleep architecture — NREM/REM cycling, adenosine, sleep debt.
//!
//! Models the two-process model of sleep regulation (Borbély):
//! Process S (homeostatic sleep pressure from adenosine) and Process C
//! (circadian alertness from SCN). Sleep stages cycle in ~90-min ultradian
//! periods: Wake → NREM1 → NREM2 → NREM3 → NREM2 → REM → repeat.

use crate::error::{MastishkError, validate_dt};
use serde::{Deserialize, Serialize};

/// Sleep stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SleepStage {
    Wake,
    Nrem1,
    Nrem2,
    Nrem3,
    Rem,
}

/// Sleep state tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepState {
    /// Current sleep stage.
    pub stage: SleepStage,
    /// Adenosine level (Process S) — 0.0 = fully rested, 1.0 = extreme pressure.
    pub adenosine: f32,
    /// Hours of accumulated sleep debt.
    pub sleep_debt: f32,
    /// Time in current stage (hours).
    pub time_in_stage: f32,
    /// Total sleep time in current cycle (hours).
    pub total_sleep: f32,
    /// Number of completed ultradian cycles.
    pub cycles_completed: u32,
}

impl Default for SleepState {
    fn default() -> Self {
        Self {
            stage: SleepStage::Wake,
            adenosine: 0.3,
            sleep_debt: 0.0,
            time_in_stage: 0.0,
            total_sleep: 0.0,
            cycles_completed: 0,
        }
    }
}

impl SleepState {
    /// Sleep pressure (0.0–1.0) combining adenosine and debt.
    #[inline]
    #[must_use]
    pub fn sleep_pressure(&self) -> f32 {
        (self.adenosine * 0.7 + (self.sleep_debt / 24.0) * 0.3).clamp(0.0, 1.0)
    }

    /// Whether the entity is asleep (any NREM or REM stage).
    #[inline]
    #[must_use]
    pub fn is_asleep(&self) -> bool {
        self.stage != SleepStage::Wake
    }

    /// Energy recovery multiplier based on current stage.
    /// Deep sleep (NREM3) recovers most, REM moderate, lighter stages less.
    #[inline]
    #[must_use]
    pub fn recovery_multiplier(&self) -> f32 {
        match self.stage {
            SleepStage::Wake => 0.0,
            SleepStage::Nrem1 => 0.3,
            SleepStage::Nrem2 => 0.6,
            SleepStage::Nrem3 => 1.0,
            SleepStage::Rem => 0.5,
        }
    }

    /// Memory consolidation rate — highest during REM and NREM3.
    #[inline]
    #[must_use]
    pub fn consolidation_rate(&self) -> f32 {
        match self.stage {
            SleepStage::Wake => 0.0,
            SleepStage::Nrem1 => 0.1,
            SleepStage::Nrem2 => 0.3,
            SleepStage::Nrem3 => 0.7,
            SleepStage::Rem => 1.0,
        }
    }

    /// Tick adenosine: rises during wake, falls during sleep.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt_hours < 0.0`.
    #[inline]
    pub fn tick_adenosine(&mut self, dt_hours: f32) -> Result<(), MastishkError> {
        validate_dt(dt_hours)?;
        tracing::trace!(dt_hours, stage = ?self.stage, adenosine = self.adenosine, "ticking adenosine");
        if self.stage == SleepStage::Wake {
            // Adenosine rises ~0.04/hr during wakefulness
            self.adenosine = (self.adenosine + 0.04 * dt_hours).min(1.0);
            self.sleep_debt += dt_hours.max(0.0) * 0.125; // ~1hr debt per 8hr awake
        } else {
            // Adenosine clears during sleep
            self.adenosine = (self.adenosine - 0.06 * dt_hours).max(0.0);
            self.sleep_debt = (self.sleep_debt - dt_hours * 0.25).max(0.0);
            self.total_sleep += dt_hours;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_awake() {
        let s = SleepState::default();
        assert_eq!(s.stage, SleepStage::Wake);
        assert!(!s.is_asleep());
    }

    #[test]
    fn test_sleep_pressure_rises() {
        let mut s = SleepState::default();
        let initial = s.sleep_pressure();
        s.tick_adenosine(8.0).unwrap();
        assert!(s.sleep_pressure() > initial);
    }

    #[test]
    fn test_adenosine_clears_during_sleep() {
        let mut s = SleepState {
            adenosine: 0.8,
            stage: SleepStage::Nrem3,
            ..Default::default()
        };
        s.tick_adenosine(4.0).unwrap();
        assert!(s.adenosine < 0.8);
    }

    #[test]
    fn test_recovery_multiplier() {
        let mut s = SleepState::default();
        assert!((s.recovery_multiplier() - 0.0).abs() < f32::EPSILON);
        s.stage = SleepStage::Nrem3;
        assert!((s.recovery_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_serde_roundtrip() {
        let s = SleepState::default();
        let json = serde_json::to_string(&s).unwrap();
        let s2: SleepState = serde_json::from_str(&json).unwrap();
        assert_eq!(s2.stage, SleepStage::Wake);
    }

    #[test]
    fn test_negative_dt_rejected() {
        let mut s = SleepState::default();
        assert!(s.tick_adenosine(-1.0).is_err());
    }

    #[test]
    fn test_consolidation_rate() {
        let mut s = SleepState::default();
        assert!((s.consolidation_rate() - 0.0).abs() < f32::EPSILON);
        s.stage = SleepStage::Rem;
        assert!((s.consolidation_rate() - 1.0).abs() < f32::EPSILON);
        s.stage = SleepStage::Nrem3;
        assert!((s.consolidation_rate() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_total_sleep_accumulates() {
        let mut s = SleepState {
            stage: SleepStage::Nrem2,
            ..Default::default()
        };
        s.tick_adenosine(4.0).unwrap();
        assert!((s.total_sleep - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sleep_debt_accumulates_during_wake() {
        let mut s = SleepState::default();
        s.tick_adenosine(8.0).unwrap();
        assert!(s.sleep_debt > 0.0);
    }

    #[test]
    fn test_is_asleep_stages() {
        let mut s = SleepState::default();
        assert!(!s.is_asleep());
        for stage in [
            SleepStage::Nrem1,
            SleepStage::Nrem2,
            SleepStage::Nrem3,
            SleepStage::Rem,
        ] {
            s.stage = stage;
            assert!(s.is_asleep());
        }
    }
}
