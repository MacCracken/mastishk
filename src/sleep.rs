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

    /// Tick adenosine (Process S) using Borbely two-process model.
    ///
    /// During wakefulness: exponential rise toward 1.0 with tau_w = 18.2 hours.
    /// During sleep: exponential decay toward 0.0 with tau_s = 4.2 hours.
    /// These time constants produce realistic sleep pressure dynamics:
    /// ~0.8 after 16hr wake, ~0.12 after 8hr sleep.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt_hours < 0.0`.
    #[inline]
    pub fn tick_adenosine(&mut self, dt_hours: f32) -> Result<(), MastishkError> {
        validate_dt(dt_hours)?;
        tracing::trace!(dt_hours, stage = ?self.stage, adenosine = self.adenosine, "ticking adenosine");
        if self.stage == SleepStage::Wake {
            // Process S rise: exponential approach to 1.0, tau_w = 18.2 hours
            let alpha_w = 1.0 - (-dt_hours / 18.2).exp();
            self.adenosine += (1.0 - self.adenosine) * alpha_w;
            self.sleep_debt += dt_hours.max(0.0) * 0.125; // ~1hr debt per 8hr awake
        } else {
            // Process S decay: exponential toward 0.0, tau_s = 4.2 hours
            self.adenosine *= (-dt_hours / 4.2).exp();
            self.sleep_debt = (self.sleep_debt - dt_hours * 0.25).max(0.0);
            self.total_sleep += dt_hours;
        }
        self.adenosine = self.adenosine.clamp(0.0, 1.0);
        Ok(())
    }

    /// Initiate sleep onset. Transitions from Wake to NREM1.
    ///
    /// Only takes effect if currently awake.
    #[inline]
    pub fn fall_asleep(&mut self) {
        if self.stage == SleepStage::Wake {
            self.stage = SleepStage::Nrem1;
            self.time_in_stage = 0.0;
            self.total_sleep = 0.0;
            self.cycles_completed = 0;
            tracing::debug!("sleep onset");
        }
    }

    /// Wake up. Transitions any sleep stage to Wake.
    #[inline]
    pub fn wake_up(&mut self) {
        if self.stage != SleepStage::Wake {
            self.stage = SleepStage::Wake;
            self.time_in_stage = 0.0;
            tracing::debug!(
                cycles = self.cycles_completed,
                total_sleep = self.total_sleep,
                "woke up"
            );
        }
    }

    /// Advance sleep stage transitions based on time in stage.
    ///
    /// Models the ~90-minute ultradian cycle:
    /// NREM1 → NREM2 → NREM3 → NREM2 → REM → (cycle complete) → NREM2 → ...
    ///
    /// Early cycles are NREM3-dominant (longer deep sleep), later cycles are
    /// REM-dominant (longer REM periods). Does nothing during Wake.
    #[inline]
    pub fn tick_stage_transitions(&mut self, dt_hours: f32) {
        if self.stage == SleepStage::Wake {
            return;
        }
        self.time_in_stage += dt_hours;

        // Stage durations (in hours) — vary by cycle
        let nrem3_duration = if self.cycles_completed < 2 { 0.5 } else { 0.2 };
        let rem_duration = if self.cycles_completed < 2 {
            0.17
        } else {
            0.33
        };

        let transition = match self.stage {
            SleepStage::Nrem1 if self.time_in_stage > 0.1 => Some(SleepStage::Nrem2),
            SleepStage::Nrem2 if self.time_in_stage > 0.33 => {
                // First pass through NREM2 → NREM3; return pass → REM
                // Use total_sleep to distinguish: early in cycle → NREM3
                if self.total_sleep < (self.cycles_completed as f32 + 0.5) * 1.5 {
                    Some(SleepStage::Nrem3)
                } else {
                    Some(SleepStage::Rem)
                }
            }
            SleepStage::Nrem3 if self.time_in_stage > nrem3_duration => Some(SleepStage::Nrem2),
            SleepStage::Rem if self.time_in_stage > rem_duration => {
                self.cycles_completed += 1;
                Some(SleepStage::Nrem2) // start new cycle
            }
            _ => None,
        };

        if let Some(next) = transition {
            tracing::trace!(from = ?self.stage, to = ?next, cycle = self.cycles_completed, "sleep stage transition");
            self.stage = next;
            self.time_in_stage = 0.0;
        }
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

    #[test]
    fn test_fall_asleep_transitions_to_nrem1() {
        let mut s = SleepState::default();
        s.fall_asleep();
        assert_eq!(s.stage, SleepStage::Nrem1);
    }

    #[test]
    fn test_wake_up_transitions_to_wake() {
        let mut s = SleepState {
            stage: SleepStage::Nrem3,
            ..Default::default()
        };
        s.wake_up();
        assert_eq!(s.stage, SleepStage::Wake);
    }

    #[test]
    fn test_ultradian_cycle_progresses() {
        let mut s = SleepState::default();
        s.fall_asleep();
        // Simulate 2 hours of sleep in 1-minute steps
        for _ in 0..120 {
            s.tick_stage_transitions(1.0 / 60.0);
            // Also tick adenosine to advance total_sleep
            s.tick_adenosine(1.0 / 60.0).unwrap();
        }
        // Should have progressed past NREM1 into deeper stages
        assert_ne!(s.stage, SleepStage::Nrem1);
        assert!(s.is_asleep());
    }

    #[test]
    fn test_full_night_completes_cycles() {
        let mut s = SleepState::default();
        s.fall_asleep();
        // Simulate 8 hours of sleep in 1-minute steps
        for _ in 0..480 {
            s.tick_stage_transitions(1.0 / 60.0);
            s.tick_adenosine(1.0 / 60.0).unwrap();
        }
        // Should have completed at least 3 cycles (~90 min each, 8hr = ~5 cycles)
        assert!(s.cycles_completed >= 3, "cycles={}", s.cycles_completed);
    }

    #[test]
    fn test_no_transitions_during_wake() {
        let mut s = SleepState::default();
        s.tick_stage_transitions(1.0);
        assert_eq!(s.stage, SleepStage::Wake);
    }
}
