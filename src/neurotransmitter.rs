//! Neurotransmitter dynamics — synthesis, release, reuptake, degradation.
//!
//! Models monoamines (serotonin, dopamine, norepinephrine), amino acid transmitters
//! (GABA, glutamate), neuropeptides (oxytocin, endorphins), acetylcholine, and BDNF.
//! Each neurotransmitter has a normalized level (0.0–1.0) representing relative
//! concentration, with kinetic parameters for synthesis and clearance.

use crate::error::{MastishkError, validate_dt};
use serde::{Deserialize, Serialize};

/// A neurotransmitter's current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransmitterState {
    /// Normalized level (0.0 = depleted, 1.0 = saturated).
    pub level: f32,
    /// Baseline level the system trends toward.
    pub baseline: f32,
    /// Synthesis rate (units per second toward baseline).
    pub synthesis_rate: f32,
    /// Clearance rate (reuptake + degradation, per second).
    pub clearance_rate: f32,
}

impl TransmitterState {
    /// Create a new transmitter at its baseline level.
    #[must_use]
    pub fn at_baseline(baseline: f32, synthesis_rate: f32, clearance_rate: f32) -> Self {
        Self {
            level: baseline,
            baseline,
            synthesis_rate,
            clearance_rate,
        }
    }

    /// Apply a stimulus (positive = release, negative = inhibition).
    /// Level is clamped to 0.0..=1.0.
    #[inline]
    pub fn stimulate(&mut self, delta: f32) {
        self.level = (self.level + delta).clamp(0.0, 1.0);
        tracing::debug!(delta, level = self.level, "transmitter stimulated");
    }

    /// Tick the transmitter toward baseline over `dt` seconds.
    /// Uses exponential decay toward baseline.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        self.tick_unchecked(dt);
        Ok(())
    }

    /// Tick without validating dt. Used by [`NeurotransmitterProfile::tick_all`]
    /// after a single validation pass.
    #[inline]
    pub(crate) fn tick_unchecked(&mut self, dt: f32) {
        let diff = self.baseline - self.level;
        let rate = if diff > 0.0 {
            self.synthesis_rate
        } else {
            self.clearance_rate
        };
        self.level += diff * (1.0 - (-rate * dt).exp());
        self.level = self.level.clamp(0.0, 1.0);
    }

    /// How far above or below baseline (negative = depleted, positive = elevated).
    #[inline]
    #[must_use]
    pub fn deviation(&self) -> f32 {
        self.level - self.baseline
    }
}

/// The major neurotransmitter systems modeled together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurotransmitterProfile {
    /// Serotonin (5-HT) — mood baseline, impulse control.
    pub serotonin: TransmitterState,
    /// Dopamine (DA) — tonic level: sustained motivation, effort, Go/NoGo balance.
    pub dopamine: TransmitterState,
    /// Dopamine phasic burst (−1.0 to +1.0). Transient RPE signal from VTA.
    /// Positive = better-than-expected reward, negative = worse. Decays rapidly.
    pub dopamine_phasic: f32,
    /// Norepinephrine (NE) — arousal, alertness, fight-or-flight.
    pub norepinephrine: TransmitterState,
    /// GABA — primary inhibitory, anxiolytic.
    pub gaba: TransmitterState,
    /// Glutamate — primary excitatory.
    pub glutamate: TransmitterState,
    /// Oxytocin — social bonding, trust.
    pub oxytocin: TransmitterState,
    /// Endorphins — pain dampening, stress recovery.
    pub endorphins: TransmitterState,
    /// Acetylcholine (ACh) — attention, memory consolidation.
    pub acetylcholine: TransmitterState,
    /// BDNF — neuroplasticity, trait adaptation rate.
    pub bdnf: TransmitterState,
    /// Histamine (HA) — primary wakefulness signal (tuberomammillary nucleus).
    /// High during wake, near-zero during sleep (Saper 2005 flip-flop model).
    #[serde(default = "default_histamine")]
    pub histamine: TransmitterState,
    /// Endocannabinoid (anandamide/2-AG) — stress buffer, retrograde CB1 signaling.
    /// Dampens both glutamate and GABA release, modulates HPA recovery and pain.
    #[serde(default = "default_endocannabinoid")]
    pub endocannabinoid: TransmitterState,
    /// Orexin/hypocretin — master wakefulness stabilizer (Saper 2005 flip-flop).
    /// High during wake, suppressed during sleep. Absence → narcolepsy.
    #[serde(default = "default_orexin")]
    pub orexin: TransmitterState,
}

fn default_histamine() -> TransmitterState {
    TransmitterState::at_baseline(0.6, 0.05, 0.06)
}

fn default_endocannabinoid() -> TransmitterState {
    TransmitterState::at_baseline(0.4, 0.01, 0.02)
}

fn default_orexin() -> TransmitterState {
    TransmitterState::at_baseline(0.6, 0.04, 0.05)
}

impl Default for NeurotransmitterProfile {
    fn default() -> Self {
        Self {
            serotonin: TransmitterState::at_baseline(0.5, 0.02, 0.03),
            dopamine: TransmitterState::at_baseline(0.4, 0.03, 0.05),
            dopamine_phasic: 0.0,
            norepinephrine: TransmitterState::at_baseline(0.3, 0.04, 0.06),
            gaba: TransmitterState::at_baseline(0.5, 0.03, 0.03),
            glutamate: TransmitterState::at_baseline(0.5, 0.04, 0.04),
            oxytocin: TransmitterState::at_baseline(0.3, 0.01, 0.02),
            endorphins: TransmitterState::at_baseline(0.2, 0.01, 0.03),
            acetylcholine: TransmitterState::at_baseline(0.4, 0.03, 0.04),
            bdnf: TransmitterState::at_baseline(0.5, 0.005, 0.005),
            histamine: default_histamine(),
            endocannabinoid: default_endocannabinoid(),
            orexin: default_orexin(),
        }
    }
}

impl NeurotransmitterProfile {
    /// Tick all transmitters toward their baselines.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn tick_all(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        tracing::trace!(dt, "ticking all neurotransmitters");
        self.serotonin.tick_unchecked(dt);
        self.dopamine.tick_unchecked(dt);
        self.norepinephrine.tick_unchecked(dt);
        self.gaba.tick_unchecked(dt);
        self.glutamate.tick_unchecked(dt);
        self.oxytocin.tick_unchecked(dt);
        self.endorphins.tick_unchecked(dt);
        self.acetylcholine.tick_unchecked(dt);
        self.bdnf.tick_unchecked(dt);
        self.histamine.tick_unchecked(dt);
        self.endocannabinoid.tick_unchecked(dt);
        self.orexin.tick_unchecked(dt);
        // Phasic DA decays rapidly (transient burst, ~500ms half-life)
        self.dopamine_phasic *= (-dt / 0.5).exp();
        Ok(())
    }

    /// Fire a phasic dopamine burst (reward prediction error signal).
    ///
    /// Positive values = better-than-expected reward, negative = disappointment.
    /// Decays rapidly during `tick_all`.
    #[inline]
    pub fn fire_dopamine_burst(&mut self, magnitude: f32) {
        self.dopamine_phasic = (self.dopamine_phasic + magnitude).clamp(-1.0, 1.0);
        tracing::debug!(
            magnitude,
            phasic = self.dopamine_phasic,
            "dopamine burst fired"
        );
    }

    /// GABA/glutamate ratio — >1.0 = inhibition dominant, <1.0 = excitation dominant.
    /// Returns at most 100.0 when glutamate is near zero (avoids infinity propagation).
    #[inline]
    #[must_use]
    pub fn inhibition_ratio(&self) -> f32 {
        if self.glutamate.level > f32::EPSILON {
            (self.gaba.level / self.glutamate.level).min(100.0)
        } else {
            100.0
        }
    }

    /// Overall arousal level derived from NE + glutamate - GABA.
    #[inline]
    #[must_use]
    pub fn arousal(&self) -> f32 {
        ((self.norepinephrine.level + self.glutamate.level - self.gaba.level) / 2.0).clamp(0.0, 1.0)
    }

    /// Reward sensitivity derived from dopamine level.
    #[inline]
    #[must_use]
    pub fn reward_sensitivity(&self) -> f32 {
        self.dopamine.level
    }

    /// Neuroplasticity rate derived from BDNF.
    #[inline]
    #[must_use]
    pub fn plasticity_rate(&self) -> f32 {
        self.bdnf.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transmitter_at_baseline() {
        let t = TransmitterState::at_baseline(0.5, 0.02, 0.03);
        assert!((t.level - 0.5).abs() < f32::EPSILON);
        assert!((t.deviation()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_stimulate_clamps() {
        let mut t = TransmitterState::at_baseline(0.5, 0.02, 0.03);
        t.stimulate(0.8);
        assert!((t.level - 1.0).abs() < f32::EPSILON);
        t.stimulate(-2.0);
        assert!(t.level >= 0.0);
    }

    #[test]
    fn test_tick_toward_baseline() {
        let mut t = TransmitterState::at_baseline(0.5, 0.1, 0.1);
        t.level = 0.9;
        t.tick(30.0).unwrap();
        assert!((t.level - t.baseline).abs() < 0.1);
    }

    #[test]
    fn test_profile_default() {
        let p = NeurotransmitterProfile::default();
        assert!(p.arousal() >= 0.0 && p.arousal() <= 1.0);
        assert!(p.inhibition_ratio() > 0.0);
    }

    #[test]
    fn test_tick_all() {
        let mut p = NeurotransmitterProfile::default();
        p.serotonin.stimulate(0.3);
        p.tick_all(1.0).unwrap();
        // Should have moved toward baseline
        assert!(p.serotonin.level < 0.8);
    }

    #[test]
    fn test_serde_roundtrip() {
        let p = NeurotransmitterProfile::default();
        let json = serde_json::to_string(&p).unwrap();
        let p2: NeurotransmitterProfile = serde_json::from_str(&json).unwrap();
        assert!((p2.serotonin.level - p.serotonin.level).abs() < f32::EPSILON);
    }

    #[test]
    fn test_negative_dt_rejected() {
        let mut t = TransmitterState::at_baseline(0.5, 0.1, 0.1);
        assert!(t.tick(-1.0).is_err());

        let mut p = NeurotransmitterProfile::default();
        assert!(p.tick_all(-0.5).is_err());
    }

    #[test]
    fn test_zero_dt_is_noop() {
        let mut t = TransmitterState::at_baseline(0.5, 0.1, 0.1);
        t.level = 0.8;
        t.tick(0.0).unwrap();
        assert!((t.level - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_reward_sensitivity() {
        let mut p = NeurotransmitterProfile::default();
        let base = p.reward_sensitivity();
        p.dopamine.stimulate(0.3);
        assert!(p.reward_sensitivity() > base);
    }

    #[test]
    fn test_plasticity_rate() {
        let mut p = NeurotransmitterProfile::default();
        let base = p.plasticity_rate();
        p.bdnf.stimulate(0.2);
        assert!(p.plasticity_rate() > base);
    }

    #[test]
    fn test_inhibition_ratio_zero_glutamate() {
        let mut p = NeurotransmitterProfile::default();
        p.glutamate.level = 0.0;
        assert!((p.inhibition_ratio() - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_arousal_boundary_clamp() {
        let mut p = NeurotransmitterProfile::default();
        // Max out excitatory, zero inhibitory
        p.norepinephrine.level = 1.0;
        p.glutamate.level = 1.0;
        p.gaba.level = 0.0;
        assert!(p.arousal() <= 1.0);

        // Zero everything
        p.norepinephrine.level = 0.0;
        p.glutamate.level = 0.0;
        p.gaba.level = 1.0;
        assert!(p.arousal() >= 0.0);
    }
}
