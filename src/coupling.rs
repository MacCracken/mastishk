//! Cross-module coupling functions for neuroscience subsystem integration.
//!
//! Each function reads from one module's state and writes to another, implementing
//! biologically grounded coupling equations. These can be called independently or
//! orchestrated via [`crate::brain::BrainState`].
//!
//! # Couplings
//!
//! - **Sleep → Neurotransmitter**: Sleep stage drives ACh, serotonin, norepinephrine baselines
//! - **Circadian → HPA**: Cortisol awakening response sets HPA cortisol baseline
//! - **DMN → HPA**: Rumination acts as chronic stressor, impairs HPA feedback
//! - **Arousal → Circuit**: NE/glutamate multiplicatively modulate synaptic gain

use serde::{Deserialize, Serialize};

use crate::chronobiology::CircadianState;
use crate::circuit::Circuit;
use crate::dmn::DmnState;
use crate::error::{MastishkError, validate_dt};
use crate::hpa::HpaState;
use crate::neurotransmitter::NeurotransmitterProfile;
use crate::regions::{
    AmygdalaState, BasalGangliaState, CerebellumState, HippocampusState, PfcState,
};
use crate::sleep::{SleepStage, SleepState};

// ── Parameter Structs ──────────────────────────────────────────────

/// Neuromodulatory gain parameters for circuit coupling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitGainParams {
    /// Norepinephrine gain coefficient.
    pub k_ne: f32,
    /// Glutamate gain coefficient.
    pub k_glu: f32,
    /// NE × glutamate interaction coefficient.
    pub k_interact: f32,
    /// GABA inhibitory dampening coefficient.
    pub k_gaba: f32,
}

impl Default for CircuitGainParams {
    fn default() -> Self {
        Self {
            k_ne: 0.3,
            k_glu: 0.2,
            k_interact: 0.1,
            k_gaba: 0.4,
        }
    }
}

/// All cross-module coupling parameters, tunable per consumer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingParams {
    /// Sleep → neurotransmitter baseline smoothing rate.
    pub sleep_nt_smoothing: f32,
    /// Circadian → HPA baseline smoothing rate.
    pub circadian_hpa_smoothing: f32,
    /// DMN → HPA rumination threshold (below this, no stress applied).
    pub dmn_hpa_threshold: f32,
    /// DMN → HPA stress intensity gain.
    pub dmn_hpa_gain: f32,
    /// DMN → HPA feedback gain reduction rate.
    pub dmn_hpa_feedback_reduction: f32,
    /// Default HPA feedback gain (restored when rumination subsides).
    pub dmn_hpa_feedback_default: f32,
    /// Minimum HPA feedback gain floor.
    pub dmn_hpa_feedback_floor: f32,
    /// Circuit neuromodulation parameters.
    pub circuit_gain: CircuitGainParams,
    /// Brain region coupling parameters.
    pub region: RegionCouplingParams,
}

/// Tunable parameters for brain region couplings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionCouplingParams {
    /// NE → amygdala amplification gain.
    pub ne_amygdala_gain: f32,
    /// Serotonin → amygdala dampening gain.
    pub serotonin_amygdala_dampen: f32,
    /// PFC → amygdala inhibition gain.
    pub pfc_amygdala_inhibit: f32,
    /// Amygdala threat → HPA stress gain.
    pub amygdala_hpa_gain: f32,
    /// ACh → hippocampus encoding boost.
    pub ach_hippocampus_encoding: f32,
    /// BDNF → hippocampus neurogenesis rate.
    pub bdnf_neurogenesis: f32,
    /// Dopamine → basal ganglia Go/NoGo modulation.
    pub dopamine_bg_gain: f32,
}

impl Default for RegionCouplingParams {
    fn default() -> Self {
        Self {
            ne_amygdala_gain: 0.3,
            serotonin_amygdala_dampen: 0.2,
            pfc_amygdala_inhibit: 0.25,
            amygdala_hpa_gain: 0.3,
            ach_hippocampus_encoding: 0.2,
            bdnf_neurogenesis: 0.1,
            dopamine_bg_gain: 0.3,
        }
    }
}

impl Default for CouplingParams {
    fn default() -> Self {
        Self {
            sleep_nt_smoothing: 0.2,
            circadian_hpa_smoothing: 0.1,
            dmn_hpa_threshold: 0.3,
            dmn_hpa_gain: 0.4,
            dmn_hpa_feedback_reduction: 0.15,
            dmn_hpa_feedback_default: 0.5,
            dmn_hpa_feedback_floor: 0.1,
            circuit_gain: CircuitGainParams::default(),
            region: RegionCouplingParams::default(),
        }
    }
}

// ── Sleep → Neurotransmitter ───────────────────────────────────────

/// Target neurotransmitter baselines for a given sleep stage.
///
/// Returns `(acetylcholine, serotonin, norepinephrine, histamine)` target levels.
/// ACh peaks during REM; monoamines (5-HT, NE) suppressed in REM/NREM3;
/// histamine high during wake, near-zero during all sleep stages (Saper 2005).
#[inline]
#[must_use]
pub fn sleep_neurotransmitter_targets(stage: SleepStage) -> (f32, f32, f32, f32) {
    //                                    ACh   5-HT   NE    HA
    match stage {
        SleepStage::Wake => (0.4, 0.8, 0.8, 0.7),
        SleepStage::Nrem1 | SleepStage::Nrem2 => (0.2, 0.3, 0.2, 0.1),
        SleepStage::Nrem3 => (0.1, 0.2, 0.1, 0.02),
        SleepStage::Rem => (0.9, 0.05, 0.05, 0.02),
    }
}

/// Apply sleep-stage-driven neurotransmitter baseline adjustment.
///
/// Exponentially smooths ACh, serotonin, and norepinephrine baselines toward
/// stage-appropriate targets. Does not modify other transmitters.
///
/// # Errors
/// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
#[inline]
pub fn apply_sleep_neurotransmitter_coupling(
    stage: SleepStage,
    profile: &mut NeurotransmitterProfile,
    smoothing_rate: f32,
    dt: f32,
) -> Result<(), MastishkError> {
    validate_dt(dt)?;
    let (ach_target, serotonin_target, ne_target, ha_target) =
        sleep_neurotransmitter_targets(stage);
    let alpha = 1.0 - (-smoothing_rate * dt).exp();

    profile.acetylcholine.baseline += (ach_target - profile.acetylcholine.baseline) * alpha;
    profile.serotonin.baseline += (serotonin_target - profile.serotonin.baseline) * alpha;
    profile.norepinephrine.baseline += (ne_target - profile.norepinephrine.baseline) * alpha;
    profile.histamine.baseline += (ha_target - profile.histamine.baseline) * alpha;

    tracing::trace!(
        ?stage,
        ach_baseline = profile.acetylcholine.baseline,
        ha_baseline = profile.histamine.baseline,
        "sleep-neurotransmitter coupling applied"
    );
    Ok(())
}

// ── Circadian → HPA ────────────────────────────────────────────────

/// Apply circadian cortisol rhythm to HPA cortisol baseline.
///
/// The HPA `cortisol_baseline` exponentially tracks the circadian cortisol
/// level (CAR — cortisol awakening response), creating a time-of-day floor
/// for stress reactivity.
///
/// # Errors
/// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
#[inline]
pub fn apply_circadian_hpa_coupling(
    circadian: &CircadianState,
    hpa: &mut HpaState,
    smoothing_rate: f32,
    dt: f32,
) -> Result<(), MastishkError> {
    validate_dt(dt)?;
    let alpha = 1.0 - (-smoothing_rate * dt).exp();
    hpa.cortisol_baseline += (circadian.cortisol_circadian - hpa.cortisol_baseline) * alpha;

    tracing::trace!(
        circadian_cortisol = circadian.cortisol_circadian,
        hpa_baseline = hpa.cortisol_baseline,
        "circadian-HPA coupling applied"
    );
    Ok(())
}

// ── DMN → HPA ──────────────────────────────────────────────────────

/// Apply rumination-driven stress to the HPA axis.
///
/// When rumination exceeds the threshold in `params`, it acts as a chronic
/// psychological stressor: applying tonic stress input and reducing HPA
/// negative feedback gain (impairing habituation). Below threshold, feedback
/// gain slowly restores toward its default.
///
/// # Errors
/// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
#[inline]
pub fn apply_dmn_hpa_coupling(
    dmn: &DmnState,
    hpa: &mut HpaState,
    params: &CouplingParams,
    dt: f32,
) -> Result<(), MastishkError> {
    validate_dt(dt)?;
    if dmn.rumination > params.dmn_hpa_threshold {
        // Rumination as tonic stressor (scaled by dt for tick-rate independence)
        hpa.stress(dmn.rumination * params.dmn_hpa_gain * dt);
        // Impair negative feedback (rumination blocks habituation)
        hpa.feedback_gain = (hpa.feedback_gain
            - params.dmn_hpa_feedback_reduction * dmn.rumination * dt)
            .max(params.dmn_hpa_feedback_floor);
    } else {
        // Slowly restore feedback gain when not ruminating
        let restore_alpha = 1.0 - (-0.05 * dt).exp();
        hpa.feedback_gain += (params.dmn_hpa_feedback_default - hpa.feedback_gain) * restore_alpha;
    }

    tracing::trace!(
        rumination = dmn.rumination,
        feedback_gain = hpa.feedback_gain,
        crh = hpa.crh,
        "DMN-HPA coupling applied"
    );
    Ok(())
}

// ── Arousal → Circuit ──────────────────────────────────────────────

/// Compute neuromodulatory gain for circuit synaptic weights.
///
/// NE and glutamate multiplicatively increase gain (GANE model), while GABA
/// dampens it. `gaba_pam` is a multiplier from benzodiazepine-class drugs
/// (1.0 = no drug effect, >1.0 = amplified GABA inhibition). The result is
/// clamped to `[0.2, 3.0]` to prevent pathological states.
#[inline]
#[must_use]
pub fn compute_circuit_gain(
    ne: f32,
    glutamate: f32,
    gaba: f32,
    gaba_pam: f32,
    params: &CircuitGainParams,
) -> f32 {
    (1.0 + params.k_ne * ne + params.k_glu * glutamate + params.k_interact * ne * glutamate
        - params.k_gaba * gaba * gaba_pam)
        .clamp(0.2, 3.0)
}

/// Apply neuromodulatory gain to a circuit and tick it.
///
/// Computes gain from current neurotransmitter levels (with optional GABA PAM
/// amplification from benzodiazepines), then calls [`Circuit::tick_with_gain`].
///
/// # Errors
/// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
#[inline]
pub fn apply_arousal_circuit_coupling(
    profile: &NeurotransmitterProfile,
    circuit: &mut Circuit,
    params: &CircuitGainParams,
    gaba_pam: f32,
    dt: f32,
) -> Result<(), MastishkError> {
    let gain = compute_circuit_gain(
        profile.norepinephrine.level,
        profile.glutamate.level,
        profile.gaba.level,
        gaba_pam,
        params,
    );
    tracing::trace!(gain, gaba_pam, "arousal-circuit coupling applied");
    circuit.tick_with_gain(gain, dt)
}

// ── Composite Metrics ──────────────────────────────────────────────

/// Composite arousal combining neurotransmitter, circadian, and sleep state.
///
/// When asleep, returns a low value (with a small bump during REM).
/// When awake, returns a weighted combination of neurotransmitter arousal,
/// circadian alertness, and inverse sleep pressure.
#[inline]
#[must_use]
pub fn composite_arousal(
    profile: &NeurotransmitterProfile,
    circadian: &CircadianState,
    sleep: &SleepState,
) -> f32 {
    if sleep.is_asleep() {
        if sleep.stage == SleepStage::Rem {
            0.15
        } else {
            0.05
        }
    } else {
        (profile.arousal() * 0.4
            + circadian.alertness() * 0.4
            + (1.0 - sleep.sleep_pressure()) * 0.2)
            .clamp(0.0, 1.0)
    }
}

/// Composite stress combining HPA cortisol, DMN rumination, and sleep debt.
#[inline]
#[must_use]
pub fn composite_stress(hpa: &HpaState, dmn: &DmnState, sleep: &SleepState) -> f32 {
    (hpa.cortisol * 0.5 + dmn.rumination * 0.3 + (sleep.sleep_debt / 12.0).min(1.0) * 0.2)
        .clamp(0.0, 1.0)
}

// ── Region Couplings ───────────────────────────────────────────────

/// Apply neurotransmitter modulation to PFC.
///
/// Dopamine follows inverted-U on working memory (optimal at 0.5), serotonin
/// supports impulse control, cortisol and sleep debt impair executive function.
/// High amygdala activation disrupts PFC (Arnsten 2009 — stress impairs PFC).
#[inline]
pub fn apply_nt_pfc_coupling(
    profile: &NeurotransmitterProfile,
    pfc: &mut PfcState,
    hpa: &HpaState,
    sleep: &SleepState,
    amygdala: &AmygdalaState,
    dt: f32,
) -> Result<(), MastishkError> {
    validate_dt(dt)?;
    let alpha = 1.0 - (-0.1 * dt).exp();

    // Dopamine inverted-U on working memory capacity: peaks at DA=0.5
    let da_effect = 1.0 - 4.0 * (profile.dopamine.level - 0.5).powi(2);
    let wm_target = 0.7 * da_effect.clamp(0.0, 1.0);
    pfc.working_memory_capacity += (wm_target - pfc.working_memory_capacity) * alpha;
    pfc.working_memory_capacity = pfc.working_memory_capacity.clamp(0.3, 1.0);

    // Serotonin supports impulse control
    let serotonin_boost = (profile.serotonin.level - 0.5) * 0.2;
    pfc.executive_control = (pfc.executive_control + serotonin_boost * alpha).clamp(0.0, 1.0);

    // High cortisol impairs PFC
    let cortisol_impairment = (hpa.cortisol - 0.3).max(0.0) * 0.15;
    pfc.fatigue = (pfc.fatigue + cortisol_impairment * alpha).clamp(0.0, 1.0);

    // Sleep debt degrades executive function
    let debt_penalty = (sleep.sleep_debt / 24.0).min(1.0) * 0.1;
    pfc.fatigue = (pfc.fatigue + debt_penalty * alpha).clamp(0.0, 1.0);

    // Amygdala→PFC: high amygdala activation disrupts executive function
    // (Arnsten 2009 — stress signaling impairs PFC structure and function)
    let amygdala_impairment = (amygdala.activation - 0.3).max(0.0) * 0.2;
    pfc.executive_control = (pfc.executive_control - amygdala_impairment * alpha).clamp(0.0, 1.0);
    // High threat also degrades working memory capacity
    let threat_wm_penalty = amygdala.threat_response() * 0.15;
    pfc.working_memory_capacity =
        (pfc.working_memory_capacity - threat_wm_penalty * alpha).clamp(0.3, 1.0);

    tracing::trace!(da_effect, amygdala_impairment, "NT-PFC coupling applied");
    Ok(())
}

/// Apply neurotransmitter and PFC modulation to amygdala.
///
/// NE amplifies, serotonin and GABA dampen, PFC inhibits activation.
#[inline]
pub fn apply_nt_amygdala_coupling(
    profile: &NeurotransmitterProfile,
    amygdala: &mut AmygdalaState,
    pfc: &PfcState,
    params: &RegionCouplingParams,
    dt: f32,
) -> Result<(), MastishkError> {
    validate_dt(dt)?;
    let alpha = 1.0 - (-0.15 * dt).exp();

    // NE amplifies amygdala (stress sensitization)
    let ne_boost = profile.norepinephrine.level * params.ne_amygdala_gain;
    amygdala.activation = (amygdala.activation + ne_boost * alpha).min(1.0);

    // Serotonin dampens amygdala reactivity
    let serotonin_dampen = profile.serotonin.level * params.serotonin_amygdala_dampen;
    amygdala.activation = (amygdala.activation - serotonin_dampen * alpha).max(0.0);

    // GABA dampens activation
    let gaba_dampen = profile.gaba.level * 0.15;
    amygdala.activation = (amygdala.activation - gaba_dampen * alpha).max(0.0);

    // PFC top-down inhibition
    let pfc_inhibit = pfc.impulse_control() * params.pfc_amygdala_inhibit;
    amygdala.activation = (amygdala.activation - pfc_inhibit * alpha).max(0.0);

    tracing::trace!(
        activation = amygdala.activation,
        "NT-amygdala coupling applied"
    );
    Ok(())
}

/// Apply neurotransmitter, amygdala, and sleep modulation to hippocampus.
///
/// ACh enhances encoding, BDNF drives neurogenesis, amygdala salience boosts
/// emotional memory encoding, sleep stage boosts consolidation.
#[inline]
pub fn apply_nt_hippocampus_coupling(
    profile: &NeurotransmitterProfile,
    hippocampus: &mut HippocampusState,
    amygdala: &AmygdalaState,
    sleep: &SleepState,
    params: &RegionCouplingParams,
    dt: f32,
) -> Result<(), MastishkError> {
    validate_dt(dt)?;
    let alpha = 1.0 - (-0.1 * dt).exp();

    // ACh enhances encoding
    let ach_boost = profile.acetylcholine.level * params.ach_hippocampus_encoding;
    hippocampus.encoding_strength = (hippocampus.encoding_strength + ach_boost * alpha).min(1.0);

    // BDNF drives neurogenesis
    let bdnf_target = profile.bdnf.level;
    hippocampus.neurogenesis +=
        (bdnf_target - hippocampus.neurogenesis) * (1.0 - (-params.bdnf_neurogenesis * dt).exp());

    // Amygdala emotional salience boosts encoding
    let salience_boost = amygdala.emotional_salience() * 0.15;
    hippocampus.encoding_strength =
        (hippocampus.encoding_strength + salience_boost * alpha).min(1.0);

    // Sleep stage boosts consolidation (NREM3 best)
    let sleep_boost = sleep.consolidation_rate() * 0.3;
    hippocampus.consolidation_rate =
        (hippocampus.consolidation_rate + sleep_boost * alpha).min(1.0);

    tracing::trace!(
        encoding = hippocampus.encoding_strength,
        "NT-hippocampus coupling applied"
    );
    Ok(())
}

/// Apply amygdala threat response as stress input to HPA axis.
#[inline]
pub fn apply_amygdala_hpa_coupling(
    amygdala: &AmygdalaState,
    hpa: &mut HpaState,
    gain: f32,
    dt: f32,
) -> Result<(), MastishkError> {
    validate_dt(dt)?;
    let threat = amygdala.threat_response();
    if threat > 0.05 {
        hpa.stress(threat * gain * dt);
        tracing::trace!(threat, "amygdala-HPA coupling applied");
    }
    Ok(())
}

/// Apply dopamine and PFC/hippocampus modulation to basal ganglia.
///
/// Tonic dopamine (sustained level) modulates Go/NoGo balance: D1 enhances Go,
/// D2 enhances No-Go. Phasic dopamine bursts modulate learning rate for habit
/// formation. PFC goal maintenance biases goal-directed over habitual.
#[inline]
pub fn apply_nt_basal_ganglia_coupling(
    profile: &NeurotransmitterProfile,
    bg: &mut BasalGangliaState,
    pfc: &PfcState,
    hippocampus: &HippocampusState,
    params: &RegionCouplingParams,
    dt: f32,
) -> Result<(), MastishkError> {
    validate_dt(dt)?;
    let alpha = 1.0 - (-0.1 * dt).exp();

    // Tonic DA → Go/No-Go balance (sustained motivation/effort)
    let tonic_da = profile.dopamine.level;
    let go_boost = tonic_da * params.dopamine_bg_gain;
    bg.go_signal = (bg.go_signal + go_boost * alpha).min(1.0);
    let nogo_boost = (1.0 - tonic_da) * params.dopamine_bg_gain * 0.5;
    bg.nogo_signal = (bg.nogo_signal + nogo_boost * alpha).min(1.0);

    // Phasic DA → learning rate modulation (RPE-driven habit updates)
    // Positive phasic bursts accelerate habit strengthening
    if profile.dopamine_phasic > 0.0 {
        bg.habit_strength = (bg.habit_strength + profile.dopamine_phasic * 0.01 * dt).min(1.0);
    }

    // PFC goal maintenance biases toward goal-directed (suppresses habit)
    let goal_bias = pfc.goal_maintenance * 0.1;
    bg.go_signal = (bg.go_signal + goal_bias * alpha).min(1.0);

    // Hippocampus context modulates reward prediction
    let context_mod = hippocampus.context_quality() * 0.1;
    bg.reward_prediction = (bg.reward_prediction + context_mod * alpha).min(1.0);

    tracing::trace!(
        go = bg.go_signal,
        nogo = bg.nogo_signal,
        "NT-BG coupling applied"
    );
    Ok(())
}

/// Apply neurotransmitter, basal ganglia, and sleep modulation to cerebellum.
///
/// BDNF modulates adaptation rate, sleep debt impairs precision, NE modulates
/// learning rate.
#[inline]
pub fn apply_nt_cerebellum_coupling(
    profile: &NeurotransmitterProfile,
    cerebellum: &mut CerebellumState,
    sleep: &SleepState,
    dt: f32,
) -> Result<(), MastishkError> {
    validate_dt(dt)?;
    let alpha = 1.0 - (-0.1 * dt).exp();

    // BDNF modulates adaptation rate (plasticity)
    let bdnf_target = profile.bdnf.level;
    cerebellum.adaptation_rate += (bdnf_target - cerebellum.adaptation_rate) * alpha;

    // Sleep debt impairs motor precision
    let debt_penalty = (sleep.sleep_debt / 24.0).min(1.0) * 0.1;
    cerebellum.motor_precision = (cerebellum.motor_precision - debt_penalty * alpha).max(0.0);

    // NE modulates learning rate
    let ne_mod = profile.norepinephrine.level * 0.1;
    cerebellum.adaptation_rate = (cerebellum.adaptation_rate + ne_mod * alpha).min(1.0);

    tracing::trace!(
        adaptation = cerebellum.adaptation_rate,
        "NT-cerebellum coupling applied"
    );
    Ok(())
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Sleep-NT targets ───────────────────────────────────────────

    #[test]
    fn test_sleep_nt_targets_wake() {
        let (ach, serotonin, ne, ha) = sleep_neurotransmitter_targets(SleepStage::Wake);
        assert!((ach - 0.4).abs() < f32::EPSILON);
        assert!((serotonin - 0.8).abs() < f32::EPSILON);
        assert!((ne - 0.8).abs() < f32::EPSILON);
        assert!(ha > 0.5); // histamine high during wake
    }

    #[test]
    fn test_sleep_nt_targets_rem() {
        let (ach, serotonin, ne, ha) = sleep_neurotransmitter_targets(SleepStage::Rem);
        assert!(ach > 0.8);
        assert!(serotonin < 0.1);
        assert!(ne < 0.1);
        assert!(ha < 0.1); // histamine suppressed during sleep
    }

    #[test]
    fn test_sleep_nt_targets_nrem3() {
        let (ach, serotonin, ne, ha) = sleep_neurotransmitter_targets(SleepStage::Nrem3);
        assert!(ach < 0.2);
        assert!(serotonin < 0.3);
        assert!(ne < 0.2);
        assert!(ha < 0.1); // histamine suppressed during sleep
    }

    #[test]
    fn test_apply_sleep_nt_coupling_converges() {
        let mut profile = NeurotransmitterProfile::default();
        // Repeatedly apply REM coupling — ACh baseline should approach 0.9
        for _ in 0..100 {
            apply_sleep_neurotransmitter_coupling(SleepStage::Rem, &mut profile, 0.2, 1.0).unwrap();
        }
        assert!((profile.acetylcholine.baseline - 0.9).abs() < 0.05);
        assert!((profile.serotonin.baseline - 0.05).abs() < 0.05);
        assert!((profile.norepinephrine.baseline - 0.05).abs() < 0.05);
    }

    #[test]
    fn test_sleep_nt_coupling_negative_dt() {
        let mut profile = NeurotransmitterProfile::default();
        assert!(
            apply_sleep_neurotransmitter_coupling(SleepStage::Wake, &mut profile, 0.2, -1.0)
                .is_err()
        );
    }

    // ── Circadian-HPA ──────────────────────────────────────────────

    #[test]
    fn test_circadian_hpa_tracks_morning() {
        let mut circadian = CircadianState::default(); // phase 8.0, morning
        circadian.tick(0.0).unwrap(); // derive rhythm values
        let mut hpa = HpaState::default();
        let initial_baseline = hpa.cortisol_baseline;

        // Apply coupling — morning cortisol is high
        for _ in 0..50 {
            apply_circadian_hpa_coupling(&circadian, &mut hpa, 0.1, 1.0).unwrap();
        }
        // HPA baseline should have moved toward circadian cortisol
        assert!(
            (hpa.cortisol_baseline - circadian.cortisol_circadian).abs()
                < (initial_baseline - circadian.cortisol_circadian).abs()
        );
    }

    #[test]
    fn test_circadian_hpa_tracks_night() {
        let mut circadian = CircadianState {
            phase_hours: 0.0, // midnight
            ..Default::default()
        };
        circadian.tick(0.0).unwrap();
        let mut hpa = HpaState::default();

        for _ in 0..100 {
            apply_circadian_hpa_coupling(&circadian, &mut hpa, 0.1, 1.0).unwrap();
        }
        // Should converge near circadian cortisol (low at midnight)
        assert!((hpa.cortisol_baseline - circadian.cortisol_circadian).abs() < 0.05);
    }

    // ── DMN-HPA ────────────────────────────────────────────────────

    #[test]
    fn test_dmn_hpa_below_threshold_no_stress() {
        let dmn = DmnState {
            rumination: 0.1,
            ..Default::default()
        };
        let mut hpa = HpaState::default();
        let initial_crh = hpa.crh;

        apply_dmn_hpa_coupling(&dmn, &mut hpa, &CouplingParams::default(), 1.0).unwrap();
        // No stress applied — CRH should not increase
        assert!(hpa.crh <= initial_crh + f32::EPSILON);
    }

    #[test]
    fn test_dmn_hpa_above_threshold_stresses() {
        let dmn = DmnState {
            rumination: 0.7,
            ..Default::default()
        };
        let mut hpa = HpaState::default();
        let initial_crh = hpa.crh;

        apply_dmn_hpa_coupling(&dmn, &mut hpa, &CouplingParams::default(), 1.0).unwrap();
        assert!(hpa.crh > initial_crh);
    }

    #[test]
    fn test_dmn_hpa_reduces_feedback_gain() {
        let dmn = DmnState {
            rumination: 0.8,
            ..Default::default()
        };
        let mut hpa = HpaState::default();
        let initial_gain = hpa.feedback_gain;

        for _ in 0..10 {
            apply_dmn_hpa_coupling(&dmn, &mut hpa, &CouplingParams::default(), 1.0).unwrap();
        }
        assert!(hpa.feedback_gain < initial_gain);
        assert!(hpa.feedback_gain >= 0.1); // floor
    }

    #[test]
    fn test_dmn_hpa_restores_feedback() {
        let mut hpa = HpaState {
            feedback_gain: 0.2,
            ..Default::default()
        };

        let dmn = DmnState {
            rumination: 0.0,
            ..Default::default()
        };
        for _ in 0..100 {
            apply_dmn_hpa_coupling(&dmn, &mut hpa, &CouplingParams::default(), 1.0).unwrap();
        }
        // Should restore toward default 0.5
        assert!((hpa.feedback_gain - 0.5).abs() < 0.1);
    }

    // ── Arousal-Circuit ────────────────────────────────────────────

    #[test]
    fn test_circuit_gain_default_params() {
        let params = CircuitGainParams::default();
        // Default NT levels: NE=0.3, Glu=0.5, GABA=0.5
        let gain = compute_circuit_gain(0.3, 0.5, 0.5, 1.0, &params);
        // 1.0 + 0.3*0.3 + 0.2*0.5 + 0.1*0.3*0.5 - 0.4*0.5*1.0
        // = 1.0 + 0.09 + 0.10 + 0.015 - 0.20 = 1.005
        assert!((gain - 1.005).abs() < 0.01);
    }

    #[test]
    fn test_circuit_gain_clamped_low() {
        let params = CircuitGainParams {
            k_ne: 0.0,
            k_glu: 0.0,
            k_interact: 0.0,
            k_gaba: 10.0, // extreme inhibition
        };
        let gain = compute_circuit_gain(0.0, 0.0, 1.0, 1.0, &params);
        assert!((gain - 0.2).abs() < f32::EPSILON); // clamped to floor
    }

    #[test]
    fn test_circuit_gain_clamped_high() {
        let params = CircuitGainParams {
            k_ne: 10.0,
            k_glu: 10.0,
            k_interact: 10.0,
            k_gaba: 0.0,
        };
        let gain = compute_circuit_gain(1.0, 1.0, 0.0, 1.0, &params);
        assert!((gain - 3.0).abs() < f32::EPSILON); // clamped to ceiling
    }

    #[test]
    fn test_apply_arousal_circuit_coupling() {
        let profile = NeurotransmitterProfile::default();
        let mut circuit = Circuit::new();
        let a = circuit.add_population(crate::circuit::NeuralPopulation::new("A", 0.5, 0.1, true));
        let b = circuit.add_population(crate::circuit::NeuralPopulation::new("B", 0.1, 0.1, true));
        circuit.add_synapse(a, b, 0.5).unwrap();

        apply_arousal_circuit_coupling(
            &profile,
            &mut circuit,
            &CircuitGainParams::default(),
            1.0,
            0.5,
        )
        .unwrap();
        // B should have moved from resting rate
        assert!(circuit.populations[b].rate > 0.1);
    }

    // ── Composite Metrics ──────────────────────────────────────────

    #[test]
    fn test_composite_arousal_awake() {
        let profile = NeurotransmitterProfile::default();
        let circadian = CircadianState::default();
        let sleep = SleepState::default(); // Wake
        let arousal = composite_arousal(&profile, &circadian, &sleep);
        assert!((0.0..=1.0).contains(&arousal));
        assert!(arousal > 0.2); // should be reasonably alert in morning
    }

    #[test]
    fn test_composite_arousal_asleep_nrem() {
        let profile = NeurotransmitterProfile::default();
        let circadian = CircadianState::default();
        let sleep = SleepState {
            stage: SleepStage::Nrem3,
            ..Default::default()
        };
        let arousal = composite_arousal(&profile, &circadian, &sleep);
        assert!((arousal - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn test_composite_arousal_asleep_rem() {
        let profile = NeurotransmitterProfile::default();
        let circadian = CircadianState::default();
        let sleep = SleepState {
            stage: SleepStage::Rem,
            ..Default::default()
        };
        let arousal = composite_arousal(&profile, &circadian, &sleep);
        assert!((arousal - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn test_composite_stress() {
        let hpa = HpaState::default();
        let dmn = DmnState::default();
        let sleep = SleepState::default();
        let stress = composite_stress(&hpa, &dmn, &sleep);
        assert!((0.0..=1.0).contains(&stress));
    }

    #[test]
    fn test_composite_stress_high_components() {
        let hpa = HpaState {
            cortisol: 0.9,
            ..Default::default()
        };
        let dmn = DmnState {
            rumination: 0.8,
            ..Default::default()
        };
        let sleep = SleepState {
            sleep_debt: 12.0,
            ..Default::default()
        };
        let stress = composite_stress(&hpa, &dmn, &sleep);
        assert!(stress > 0.7);
    }
}
