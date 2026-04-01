//! Integration tests for mastishk — cross-module behavior and serde roundtrips.

use mastishk::chronobiology::CircadianState;
use mastishk::circuit::{Circuit, NeuralPopulation};
use mastishk::dmn::DmnState;
use mastishk::hpa::HpaState;
use mastishk::neurotransmitter::NeurotransmitterProfile;
use mastishk::sleep::{SleepStage, SleepState};

// ── Serde roundtrip tests ──────────────────────────────────────────

#[test]
fn test_neurotransmitter_profile_serde_roundtrip() {
    let profile = NeurotransmitterProfile::default();
    let json = serde_json::to_string(&profile).unwrap();
    let deserialized: NeurotransmitterProfile = serde_json::from_str(&json).unwrap();
    assert!((deserialized.serotonin.level - profile.serotonin.level).abs() < f32::EPSILON);
    assert!((deserialized.dopamine.level - profile.dopamine.level).abs() < f32::EPSILON);
    assert!((deserialized.norepinephrine.level - profile.norepinephrine.level).abs() < f32::EPSILON);
    assert!((deserialized.gaba.level - profile.gaba.level).abs() < f32::EPSILON);
    assert!((deserialized.glutamate.level - profile.glutamate.level).abs() < f32::EPSILON);
    assert!((deserialized.oxytocin.level - profile.oxytocin.level).abs() < f32::EPSILON);
    assert!((deserialized.endorphins.level - profile.endorphins.level).abs() < f32::EPSILON);
    assert!((deserialized.acetylcholine.level - profile.acetylcholine.level).abs() < f32::EPSILON);
    assert!((deserialized.bdnf.level - profile.bdnf.level).abs() < f32::EPSILON);
}

#[test]
fn test_hpa_state_serde_roundtrip() {
    let hpa = HpaState::default();
    let json = serde_json::to_string(&hpa).unwrap();
    let deserialized: HpaState = serde_json::from_str(&json).unwrap();
    assert!((deserialized.cortisol - hpa.cortisol).abs() < f32::EPSILON);
    assert!((deserialized.crh - hpa.crh).abs() < f32::EPSILON);
    assert!((deserialized.acth - hpa.acth).abs() < f32::EPSILON);
    assert!((deserialized.allostatic_load - hpa.allostatic_load).abs() < f32::EPSILON);
}

#[test]
fn test_sleep_state_serde_roundtrip() {
    let sleep = SleepState::default();
    let json = serde_json::to_string(&sleep).unwrap();
    let deserialized: SleepState = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.stage, SleepStage::Wake);
    assert!((deserialized.adenosine - sleep.adenosine).abs() < f32::EPSILON);
    assert!((deserialized.sleep_debt - sleep.sleep_debt).abs() < f32::EPSILON);
}

#[test]
fn test_dmn_state_serde_roundtrip() {
    let dmn = DmnState::default();
    let json = serde_json::to_string(&dmn).unwrap();
    let deserialized: DmnState = serde_json::from_str(&json).unwrap();
    assert!((deserialized.dmn_activation - dmn.dmn_activation).abs() < f32::EPSILON);
    assert!((deserialized.tpn_activation - dmn.tpn_activation).abs() < f32::EPSILON);
    assert!((deserialized.meditation_depth - dmn.meditation_depth).abs() < f32::EPSILON);
}

#[test]
fn test_circadian_state_serde_roundtrip() {
    let circadian = CircadianState::default();
    let json = serde_json::to_string(&circadian).unwrap();
    let deserialized: CircadianState = serde_json::from_str(&json).unwrap();
    assert!((deserialized.phase_hours - circadian.phase_hours).abs() < f32::EPSILON);
    assert!((deserialized.melatonin - circadian.melatonin).abs() < f32::EPSILON);
    assert!((deserialized.cortisol_circadian - circadian.cortisol_circadian).abs() < f32::EPSILON);
}

#[test]
fn test_circuit_serde_roundtrip() {
    let mut circuit = Circuit::new();
    let a = circuit.add_population(NeuralPopulation::new("excitatory", 0.3, 0.1, true));
    let b = circuit.add_population(NeuralPopulation::new("inhibitory", 0.2, 0.2, false));
    circuit.add_synapse(a, b, 0.5);
    circuit.add_synapse(b, a, -0.3);

    let json = serde_json::to_string(&circuit).unwrap();
    let deserialized: Circuit = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.populations.len(), 2);
    assert_eq!(deserialized.synapses.len(), 2);
    assert!((deserialized.populations[0].rate - circuit.populations[0].rate).abs() < f32::EPSILON);
}

// ── Error display tests ────────────────────────────────────────────

#[test]
fn test_error_display_level_out_of_range() {
    let err = mastishk::MastishkError::LevelOutOfRange {
        name: "serotonin".into(),
        value: 1.5,
        min: 0.0,
        max: 1.0,
    };
    let msg = err.to_string();
    assert!(msg.contains("serotonin"));
    assert!(msg.contains("1.5"));
    assert!(msg.contains("0"));
    assert!(msg.contains("1"));
}

#[test]
fn test_error_display_invalid_circuit() {
    let err = mastishk::MastishkError::InvalidCircuit("no populations".into());
    assert!(err.to_string().contains("no populations"));
}

#[test]
fn test_error_display_invalid_sleep_transition() {
    let err = mastishk::MastishkError::InvalidSleepTransition {
        from: "Wake".into(),
        to: "NREM3".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Wake"));
    assert!(msg.contains("NREM3"));
}

#[test]
fn test_error_display_negative_time_delta() {
    let err = mastishk::MastishkError::NegativeTimeDelta(-1.0);
    assert!(err.to_string().contains("-1"));
}

// ── Cross-module integration ───────────────────────────────────────

#[test]
fn test_stress_raises_arousal() {
    let mut profile = NeurotransmitterProfile::default();
    let baseline_arousal = profile.arousal();

    // Simulate stress: raise norepinephrine and glutamate
    profile.norepinephrine.stimulate(0.4);
    profile.glutamate.stimulate(0.2);
    assert!(profile.arousal() > baseline_arousal);
}

#[test]
fn test_sleep_pressure_rises_during_wake() {
    let mut sleep = SleepState::default();
    let initial_pressure = sleep.sleep_pressure();
    sleep.tick_adenosine(16.0); // 16 hours awake
    assert!(sleep.sleep_pressure() > initial_pressure);
}

#[test]
fn test_hpa_stress_cascade_propagates() {
    let mut hpa = HpaState::default();
    let initial_cortisol = hpa.cortisol;
    hpa.stress(0.9);
    for _ in 0..20 {
        hpa.tick(0.5);
    }
    assert!(hpa.cortisol > initial_cortisol);
}
