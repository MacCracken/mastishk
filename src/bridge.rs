//! Bhava bridge — f64 output functions for downstream emotion/personality consumers.
//!
//! Maps mastishk neural state to bhava-consumable values. All functions are pure,
//! returning `f64` for precision at the consumer boundary. No bhava types leak into
//! mastishk — consumers map these outputs to their own types.
//!
//! # Sections
//!
//! - **Neurotransmitter → Emotion**: serotonin mood, dopamine reward, NE arousal,
//!   GABA/glutamate anxiety, ACh focus, endorphin pain dampening
//! - **HPA → Stress**: cortisol amplifier, allostatic load fraction
//! - **Sleep → Energy**: sleep debt penalty, stage recovery rate, melatonin pressure
//! - **DMN → Cognition**: rumination stress input, meditation regulation boost
//! - **Composite**: [`BrainMoodEffect`] aggregating all outputs

use serde::{Deserialize, Serialize};

use crate::brain::BrainState;
use crate::chronobiology::CircadianState;
use crate::dmn::DmnState;
use crate::hpa::HpaState;
use crate::neurotransmitter::NeurotransmitterProfile;
use crate::regions::{
    AmygdalaState, BasalGangliaState, CerebellumState, HippocampusState, PfcState,
};
use crate::sleep::SleepState;

// ── Neurotransmitter → Emotion ─────────────────────────────────────

/// Serotonin level mapped to mood baseline floor.
///
/// Returns −1.0 (depleted) to +1.0 (optimal). Baseline serotonin (0.5) maps
/// to 0.0 (neutral mood). Below baseline → negative mood offset, above → positive.
#[inline]
#[must_use]
pub fn serotonin_mood_effect(state: &NeurotransmitterProfile) -> f64 {
    // Linear map: 0.0 → -1.0, 0.5 → 0.0, 1.0 → +1.0
    ((f64::from(state.serotonin.level) - 0.5) * 2.0).clamp(-1.0, 1.0)
}

/// Dopamine level mapped to preference reinforcement strength (0.0–1.0).
#[inline]
#[must_use]
pub fn dopamine_reward_sensitivity(state: &NeurotransmitterProfile) -> f64 {
    f64::from(state.dopamine.level).clamp(0.0, 1.0)
}

/// Norepinephrine level mapped to arousal/salience gain (0.0–1.0).
#[inline]
#[must_use]
pub fn norepinephrine_arousal(state: &NeurotransmitterProfile) -> f64 {
    f64::from(state.norepinephrine.level).clamp(0.0, 1.0)
}

/// GABA/glutamate balance mapped to anxiety level.
///
/// Returns 0.0 (calm, GABA dominant) to 1.0 (panic, glutamate dominant).
/// Uses inverse of inhibition ratio: high GABA → low anxiety.
#[inline]
#[must_use]
pub fn gaba_glutamate_anxiety(state: &NeurotransmitterProfile) -> f64 {
    let ratio = f64::from(state.inhibition_ratio());
    // ratio > 1.0 = GABA dominant (calm), < 1.0 = glutamate dominant (anxious)
    // Map: ratio 0.5 → anxiety 1.0, ratio 1.0 → 0.5, ratio 2.0 → 0.0
    (1.0 - (ratio - 0.5) / 1.5).clamp(0.0, 1.0)
}

/// Acetylcholine level mapped to attention/flow entry threshold modifier.
///
/// Returns 0.0 (poor focus) to 1.0 (sharp focus). Higher ACh lowers the
/// threshold for entering flow states.
#[inline]
#[must_use]
pub fn acetylcholine_focus(state: &NeurotransmitterProfile) -> f64 {
    f64::from(state.acetylcholine.level).clamp(0.0, 1.0)
}

/// Endorphin level mapped to stress recovery boost (1.0–2.0×).
///
/// Returns a multiplier: 1.0 (no endorphins) to 2.0 (saturated endorphins).
#[inline]
#[must_use]
pub fn endorphin_pain_dampening(state: &NeurotransmitterProfile) -> f64 {
    1.0 + f64::from(state.endorphins.level).clamp(0.0, 1.0)
}

// ── HPA Axis → Stress ──────────────────────────────────────────────

/// Cortisol level mapped to stress accumulation rate multiplier (1.0–3.0).
///
/// Higher cortisol accelerates stress accumulation in downstream consumers.
#[inline]
#[must_use]
pub fn cortisol_stress_amplifier(hpa: &HpaState) -> f64 {
    1.0 + f64::from(hpa.cortisol).clamp(0.0, 1.0) * 2.0
}

/// Allostatic load as burnout proximity (0.0–1.0).
///
/// 0.0 = no chronic stress damage, 1.0 = critical burnout threshold.
/// Allostatic load > 5.0 maps to 1.0 (saturated).
#[inline]
#[must_use]
pub fn allostatic_load_fraction(hpa: &HpaState) -> f64 {
    (f64::from(hpa.allostatic_load) / 5.0).clamp(0.0, 1.0)
}

// ── Sleep → Energy ─────────────────────────────────────────────────

/// Sleep debt mapped to energy recovery rate reduction.
///
/// Returns 0.0 (fully rested, no penalty) to 1.0 (severe debt, maximum penalty).
/// 24 hours of debt maps to 1.0.
#[inline]
#[must_use]
pub fn sleep_debt_energy_penalty(sleep: &SleepState) -> f64 {
    (f64::from(sleep.sleep_debt) / 24.0).clamp(0.0, 1.0)
}

/// Current sleep stage mapped to energy/stress recovery multiplier.
///
/// Deep NREM3 = best recovery (1.0), REM = moderate (0.5), wake = none (0.0).
#[inline]
#[must_use]
pub fn sleep_stage_recovery_rate(sleep: &SleepState) -> f64 {
    f64::from(sleep.recovery_multiplier())
}

/// Melatonin level mapped to circadian drowsiness (0.0–1.0).
#[inline]
#[must_use]
pub fn melatonin_sleep_pressure(circadian: &CircadianState) -> f64 {
    f64::from(circadian.melatonin).clamp(0.0, 1.0)
}

// ── DMN → Cognition ────────────────────────────────────────────────

/// DMN rumination level as chronic stress input (0.0–1.0).
#[inline]
#[must_use]
pub fn rumination_stress_input(dmn: &DmnState) -> f64 {
    f64::from(dmn.rumination).clamp(0.0, 1.0)
}

/// Meditation depth mapped to emotion regulation effectiveness multiplier.
///
/// Returns 1.0 (no meditation) to 2.0 (deep meditation doubles regulation).
#[inline]
#[must_use]
pub fn meditation_regulation_boost(dmn: &DmnState) -> f64 {
    1.0 + f64::from(dmn.meditation_depth).clamp(0.0, 1.0)
}

// ── Brain Regions ──────────────────────────────────────────────────

/// PFC executive function output (0.0–1.0). Impulse control strength.
#[inline]
#[must_use]
pub fn pfc_executive_function(pfc: &PfcState) -> f64 {
    f64::from(pfc.impulse_control())
}

/// PFC available working memory capacity (0.0–1.0).
#[inline]
#[must_use]
pub fn pfc_working_memory(pfc: &PfcState) -> f64 {
    f64::from(pfc.available_capacity())
}

/// Amygdala threat/fear response (0.0–1.0).
#[inline]
#[must_use]
pub fn amygdala_fear_level(amygdala: &AmygdalaState) -> f64 {
    f64::from(amygdala.threat_response())
}

/// Amygdala emotional salience output (0.0–1.0).
#[inline]
#[must_use]
pub fn amygdala_emotional_salience(amygdala: &AmygdalaState) -> f64 {
    f64::from(amygdala.emotional_salience())
}

/// Hippocampus memory formation rate (0.0–1.0).
#[inline]
#[must_use]
pub fn hippocampus_learning_rate(hippocampus: &HippocampusState) -> f64 {
    f64::from(hippocampus.memory_formation_rate())
}

/// Hippocampus context representation quality (0.0–1.0).
#[inline]
#[must_use]
pub fn hippocampus_context(hippocampus: &HippocampusState) -> f64 {
    f64::from(hippocampus.context_quality())
}

/// Basal ganglia net action drive — Go minus No-Go (0.0–1.0).
#[inline]
#[must_use]
pub fn basal_ganglia_action_drive(bg: &BasalGangliaState) -> f64 {
    f64::from(bg.action_selection())
}

/// Basal ganglia habit strength (0.0–1.0).
#[inline]
#[must_use]
pub fn basal_ganglia_habit_level(bg: &BasalGangliaState) -> f64 {
    f64::from(bg.habit_strength)
}

/// Cerebellum motor output quality (0.0–1.0).
#[inline]
#[must_use]
pub fn cerebellum_motor_quality(cerebellum: &CerebellumState) -> f64 {
    f64::from(cerebellum.motor_output_quality())
}

/// Cerebellum timing precision (0.0–1.0).
#[inline]
#[must_use]
pub fn cerebellum_timing(cerebellum: &CerebellumState) -> f64 {
    f64::from(cerebellum.timing_quality())
}

// ── Composite ──────────────────────────────────────────────────────

/// All bhava-relevant outputs from brain state in a single struct.
///
/// Consumers map these to their own emotion/personality types.
/// All values are f64 for precision at the API boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainMoodEffect {
    /// Serotonin-driven mood offset (−1.0 to +1.0).
    pub mood_offset: f64,
    /// Dopamine-driven reward sensitivity (0.0–1.0).
    pub reward_sensitivity: f64,
    /// NE-driven arousal/salience (0.0–1.0).
    pub arousal: f64,
    /// GABA/glutamate-driven anxiety level (0.0–1.0).
    pub anxiety: f64,
    /// ACh-driven focus/flow threshold (0.0–1.0).
    pub focus: f64,
    /// Endorphin-driven recovery boost (1.0–2.0×).
    pub pain_dampening: f64,
    /// Cortisol-driven stress accumulation multiplier (1.0–3.0).
    pub stress_multiplier: f64,
    /// Allostatic load burnout proximity (0.0–1.0).
    pub burnout: f64,
    /// Sleep debt energy penalty (0.0–1.0).
    pub energy_penalty: f64,
    /// Sleep stage recovery rate (0.0–1.0).
    pub recovery_rate: f64,
    /// Melatonin drowsiness (0.0–1.0).
    pub drowsiness: f64,
    /// Rumination chronic stress input (0.0–1.0).
    pub rumination_stress: f64,
    /// Meditation regulation boost (1.0–2.0×).
    pub regulation_boost: f64,
    /// BDNF-driven growth plasticity (0.0–1.0).
    pub growth_plasticity: f64,
    /// PFC executive/impulse control (0.0–1.0).
    pub executive_control: f64,
    /// PFC working memory availability (0.0–1.0).
    pub working_memory: f64,
    /// Amygdala fear/threat response (0.0–1.0).
    pub fear_level: f64,
    /// Amygdala emotional salience (0.0–1.0).
    pub emotional_salience: f64,
    /// Hippocampus memory formation rate (0.0–1.0).
    pub learning_rate: f64,
    /// Basal ganglia net action drive (0.0–1.0).
    pub action_drive: f64,
    /// Basal ganglia habit strength (0.0–1.0).
    pub habit_level: f64,
    /// Cerebellum motor output quality (0.0–1.0).
    pub motor_quality: f64,
}

/// Compute all bhava-relevant outputs from a complete brain state.
#[must_use]
pub fn brain_mood_modifiers(state: &BrainState) -> BrainMoodEffect {
    BrainMoodEffect {
        mood_offset: serotonin_mood_effect(&state.neurotransmitter),
        reward_sensitivity: dopamine_reward_sensitivity(&state.neurotransmitter),
        arousal: norepinephrine_arousal(&state.neurotransmitter),
        anxiety: gaba_glutamate_anxiety(&state.neurotransmitter),
        focus: acetylcholine_focus(&state.neurotransmitter),
        pain_dampening: endorphin_pain_dampening(&state.neurotransmitter),
        stress_multiplier: cortisol_stress_amplifier(&state.hpa),
        burnout: allostatic_load_fraction(&state.hpa),
        energy_penalty: sleep_debt_energy_penalty(&state.sleep),
        recovery_rate: sleep_stage_recovery_rate(&state.sleep),
        drowsiness: melatonin_sleep_pressure(&state.circadian),
        rumination_stress: rumination_stress_input(&state.dmn),
        regulation_boost: meditation_regulation_boost(&state.dmn),
        growth_plasticity: f64::from(state.neurotransmitter.plasticity_rate()).clamp(0.0, 1.0),
        executive_control: pfc_executive_function(&state.pfc),
        working_memory: pfc_working_memory(&state.pfc),
        fear_level: amygdala_fear_level(&state.amygdala),
        emotional_salience: amygdala_emotional_salience(&state.amygdala),
        learning_rate: hippocampus_learning_rate(&state.hippocampus),
        action_drive: basal_ganglia_action_drive(&state.basal_ganglia),
        habit_level: basal_ganglia_habit_level(&state.basal_ganglia),
        motor_quality: cerebellum_motor_quality(&state.cerebellum),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sleep::SleepStage;

    #[test]
    fn test_serotonin_mood_effect_range() {
        let mut p = NeurotransmitterProfile::default();
        // Baseline (0.5) → 0.0
        assert!((serotonin_mood_effect(&p) - 0.0).abs() < 0.01);
        // Depleted → -1.0
        p.serotonin.level = 0.0;
        assert!((serotonin_mood_effect(&p) - (-1.0)).abs() < 0.01);
        // Saturated → +1.0
        p.serotonin.level = 1.0;
        assert!((serotonin_mood_effect(&p) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_dopamine_reward_sensitivity() {
        let p = NeurotransmitterProfile::default();
        let r = dopamine_reward_sensitivity(&p);
        assert!((0.0..=1.0).contains(&r));
        assert!((r - f64::from(p.dopamine.level)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_norepinephrine_arousal() {
        let p = NeurotransmitterProfile::default();
        let a = norepinephrine_arousal(&p);
        assert!((0.0..=1.0).contains(&a));
    }

    #[test]
    fn test_gaba_glutamate_anxiety() {
        let mut p = NeurotransmitterProfile::default();
        // Equal GABA/glutamate (ratio 1.0) → moderate anxiety
        let mid = gaba_glutamate_anxiety(&p);
        assert!((0.0..=1.0).contains(&mid));

        // High GABA, low glutamate → low anxiety
        p.gaba.level = 0.9;
        p.glutamate.level = 0.3;
        let calm = gaba_glutamate_anxiety(&p);
        assert!(calm < mid);

        // Low GABA, high glutamate → high anxiety
        p.gaba.level = 0.2;
        p.glutamate.level = 0.8;
        let anxious = gaba_glutamate_anxiety(&p);
        assert!(anxious > mid);
    }

    #[test]
    fn test_acetylcholine_focus() {
        let p = NeurotransmitterProfile::default();
        let f = acetylcholine_focus(&p);
        assert!((0.0..=1.0).contains(&f));
    }

    #[test]
    fn test_endorphin_pain_dampening_range() {
        let mut p = NeurotransmitterProfile::default();
        p.endorphins.level = 0.0;
        assert!((endorphin_pain_dampening(&p) - 1.0).abs() < f64::EPSILON);
        p.endorphins.level = 1.0;
        assert!((endorphin_pain_dampening(&p) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cortisol_stress_amplifier_range() {
        let h_low = HpaState {
            cortisol: 0.0,
            ..Default::default()
        };
        assert!((cortisol_stress_amplifier(&h_low) - 1.0).abs() < f64::EPSILON);
        let h_high = HpaState {
            cortisol: 1.0,
            ..Default::default()
        };
        assert!((cortisol_stress_amplifier(&h_high) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_allostatic_load_fraction() {
        let h0 = HpaState::default(); // allostatic_load = 0.0
        assert!((allostatic_load_fraction(&h0) - 0.0).abs() < f64::EPSILON);
        let h5 = HpaState {
            allostatic_load: 5.0,
            ..Default::default()
        };
        assert!((allostatic_load_fraction(&h5) - 1.0).abs() < f64::EPSILON);
        let h10 = HpaState {
            allostatic_load: 10.0,
            ..Default::default()
        };
        assert!((allostatic_load_fraction(&h10) - 1.0).abs() < f64::EPSILON); // clamped
    }

    #[test]
    fn test_sleep_debt_energy_penalty() {
        let mut s = SleepState::default();
        assert!((sleep_debt_energy_penalty(&s) - 0.0).abs() < f64::EPSILON);
        s.sleep_debt = 12.0;
        assert!((sleep_debt_energy_penalty(&s) - 0.5).abs() < f64::EPSILON);
        s.sleep_debt = 24.0;
        assert!((sleep_debt_energy_penalty(&s) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sleep_stage_recovery_rate() {
        let mut s = SleepState::default();
        assert!((sleep_stage_recovery_rate(&s) - 0.0).abs() < f64::EPSILON); // Wake
        s.stage = SleepStage::Nrem3;
        assert!((sleep_stage_recovery_rate(&s) - 1.0).abs() < f64::EPSILON); // Deep sleep
        s.stage = SleepStage::Rem;
        assert!((sleep_stage_recovery_rate(&s) - 0.5).abs() < f64::EPSILON); // REM
    }

    #[test]
    fn test_melatonin_sleep_pressure() {
        let c = CircadianState::default();
        let p = melatonin_sleep_pressure(&c);
        assert!((0.0..=1.0).contains(&p));
    }

    #[test]
    fn test_rumination_stress_input() {
        let d = DmnState {
            rumination: 0.7,
            ..Default::default()
        };
        assert!((rumination_stress_input(&d) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_meditation_regulation_boost_range() {
        let d0 = DmnState::default(); // meditation_depth = 0.0
        assert!((meditation_regulation_boost(&d0) - 1.0).abs() < f64::EPSILON);
        let d1 = DmnState {
            meditation_depth: 1.0,
            ..Default::default()
        };
        assert!((meditation_regulation_boost(&d1) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_brain_mood_modifiers_all_finite() {
        let state = BrainState::default();
        let effect = brain_mood_modifiers(&state);
        assert!(effect.mood_offset.is_finite());
        assert!(effect.reward_sensitivity.is_finite());
        assert!(effect.arousal.is_finite());
        assert!(effect.anxiety.is_finite());
        assert!(effect.focus.is_finite());
        assert!(effect.pain_dampening.is_finite());
        assert!(effect.stress_multiplier.is_finite());
        assert!(effect.burnout.is_finite());
        assert!(effect.energy_penalty.is_finite());
        assert!(effect.recovery_rate.is_finite());
        assert!(effect.drowsiness.is_finite());
        assert!(effect.rumination_stress.is_finite());
        assert!(effect.regulation_boost.is_finite());
        assert!(effect.growth_plasticity.is_finite());
    }

    #[test]
    fn test_brain_mood_modifiers_serde_roundtrip() {
        let state = BrainState::default();
        let effect = brain_mood_modifiers(&state);
        let json = serde_json::to_string(&effect).unwrap();
        let effect2: BrainMoodEffect = serde_json::from_str(&json).unwrap();
        assert!((effect2.mood_offset - effect.mood_offset).abs() < f64::EPSILON);
        assert!((effect2.stress_multiplier - effect.stress_multiplier).abs() < f64::EPSILON);
    }
}
