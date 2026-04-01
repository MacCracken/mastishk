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
        }
    }
}

// ── Sleep → Neurotransmitter ───────────────────────────────────────

/// Target neurotransmitter baselines for a given sleep stage.
///
/// Returns `(acetylcholine, serotonin, norepinephrine)` target levels based on
/// neurophysiological data: ACh peaks during REM while monoamines (5-HT, NE)
/// are suppressed; during deep NREM all are low; during wake all are at baseline.
#[inline]
#[must_use]
pub fn sleep_neurotransmitter_targets(stage: SleepStage) -> (f32, f32, f32) {
    match stage {
        SleepStage::Wake => (0.4, 0.8, 0.8),
        SleepStage::Nrem1 | SleepStage::Nrem2 => (0.2, 0.3, 0.2),
        SleepStage::Nrem3 => (0.1, 0.2, 0.1),
        SleepStage::Rem => (0.9, 0.05, 0.05),
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
    let (ach_target, serotonin_target, ne_target) = sleep_neurotransmitter_targets(stage);
    let alpha = 1.0 - (-smoothing_rate * dt).exp();

    profile.acetylcholine.baseline += (ach_target - profile.acetylcholine.baseline) * alpha;
    profile.serotonin.baseline += (serotonin_target - profile.serotonin.baseline) * alpha;
    profile.norepinephrine.baseline += (ne_target - profile.norepinephrine.baseline) * alpha;

    tracing::trace!(
        ?stage,
        ach_baseline = profile.acetylcholine.baseline,
        serotonin_baseline = profile.serotonin.baseline,
        ne_baseline = profile.norepinephrine.baseline,
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
/// dampens it. The result is clamped to `[0.2, 3.0]` to prevent pathological
/// states.
#[inline]
#[must_use]
pub fn compute_circuit_gain(ne: f32, glutamate: f32, gaba: f32, params: &CircuitGainParams) -> f32 {
    (1.0 + params.k_ne * ne + params.k_glu * glutamate + params.k_interact * ne * glutamate
        - params.k_gaba * gaba)
        .clamp(0.2, 3.0)
}

/// Apply neuromodulatory gain to a circuit and tick it.
///
/// Computes gain from current neurotransmitter levels, then calls
/// [`Circuit::tick_with_gain`].
///
/// # Errors
/// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
#[inline]
pub fn apply_arousal_circuit_coupling(
    profile: &NeurotransmitterProfile,
    circuit: &mut Circuit,
    params: &CircuitGainParams,
    dt: f32,
) -> Result<(), MastishkError> {
    let gain = compute_circuit_gain(
        profile.norepinephrine.level,
        profile.glutamate.level,
        profile.gaba.level,
        params,
    );
    tracing::trace!(gain, "arousal-circuit coupling applied");
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

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Sleep-NT targets ───────────────────────────────────────────

    #[test]
    fn test_sleep_nt_targets_wake() {
        let (ach, serotonin, ne) = sleep_neurotransmitter_targets(SleepStage::Wake);
        assert!((ach - 0.4).abs() < f32::EPSILON);
        assert!((serotonin - 0.8).abs() < f32::EPSILON);
        assert!((ne - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sleep_nt_targets_rem() {
        let (ach, serotonin, ne) = sleep_neurotransmitter_targets(SleepStage::Rem);
        assert!(ach > 0.8);
        assert!(serotonin < 0.1);
        assert!(ne < 0.1);
    }

    #[test]
    fn test_sleep_nt_targets_nrem3() {
        let (ach, serotonin, ne) = sleep_neurotransmitter_targets(SleepStage::Nrem3);
        assert!(ach < 0.2);
        assert!(serotonin < 0.3);
        assert!(ne < 0.2);
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
        let gain = compute_circuit_gain(0.3, 0.5, 0.5, &params);
        // 1.0 + 0.3*0.3 + 0.2*0.5 + 0.1*0.3*0.5 - 0.4*0.5
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
        let gain = compute_circuit_gain(0.0, 0.0, 1.0, &params);
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
        let gain = compute_circuit_gain(1.0, 1.0, 0.0, &params);
        assert!((gain - 3.0).abs() < f32::EPSILON); // clamped to ceiling
    }

    #[test]
    fn test_apply_arousal_circuit_coupling() {
        let profile = NeurotransmitterProfile::default();
        let mut circuit = Circuit::new();
        let a = circuit.add_population(crate::circuit::NeuralPopulation::new("A", 0.5, 0.1, true));
        let b = circuit.add_population(crate::circuit::NeuralPopulation::new("B", 0.1, 0.1, true));
        circuit.add_synapse(a, b, 0.5).unwrap();

        apply_arousal_circuit_coupling(&profile, &mut circuit, &CircuitGainParams::default(), 0.5)
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
