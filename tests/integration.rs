//! Integration tests for mastishk — cross-module behavior and serde roundtrips.

use mastishk::brain::BrainState;
use mastishk::brain::{AgeProfile, InteroceptiveState, SexHormoneState};
use mastishk::bridge::brain_mood_modifiers;
use mastishk::chronobiology::CircadianState;
use mastishk::circuit::{Circuit, NeuralPopulation};
use mastishk::coupling::CouplingParams;
use mastishk::dmn::DmnState;
use mastishk::eeg::EegBand;
use mastishk::hpa::HpaState;
use mastishk::neurotransmitter::NeurotransmitterProfile;
use mastishk::pharmacology::DrugProfile;
use mastishk::sleep::{SleepStage, SleepState};
use mastishk::spiking::{
    BcmRule, IzhikevichNeuron, LifNeuron, SpikingNetwork, SpikingNeuron, StdpRule,
};

// ── Serde roundtrip tests ──────────────────────────────────────────

#[test]
fn test_neurotransmitter_profile_serde_roundtrip() {
    let profile = NeurotransmitterProfile::default();
    let json = serde_json::to_string(&profile).unwrap();
    let deserialized: NeurotransmitterProfile = serde_json::from_str(&json).unwrap();
    assert!((deserialized.serotonin.level - profile.serotonin.level).abs() < f32::EPSILON);
    assert!((deserialized.dopamine.level - profile.dopamine.level).abs() < f32::EPSILON);
    assert!(
        (deserialized.norepinephrine.level - profile.norepinephrine.level).abs() < f32::EPSILON
    );
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
    circuit.add_synapse(a, b, 0.5).unwrap();
    circuit.add_synapse(b, a, -0.3).unwrap();

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
    sleep.tick_adenosine(16.0).unwrap(); // 16 hours awake
    assert!(sleep.sleep_pressure() > initial_pressure);
}

#[test]
fn test_hpa_stress_cascade_propagates() {
    let mut hpa = HpaState::default();
    let initial_cortisol = hpa.cortisol;
    hpa.stress(0.9);
    for _ in 0..20 {
        hpa.tick(0.5).unwrap();
    }
    assert!(hpa.cortisol > initial_cortisol);
}

// ── BrainState integration ─────────────────────────────────────────

#[test]
fn test_brain_state_serde_roundtrip() {
    let mut brain = BrainState::default();
    brain.tick(1.0).unwrap();
    let json = serde_json::to_string(&brain).unwrap();
    let brain2: BrainState = serde_json::from_str(&json).unwrap();
    assert!(
        (brain2.neurotransmitter.serotonin.level - brain.neurotransmitter.serotonin.level).abs()
            < f32::EPSILON
    );
    assert!((brain2.hpa.cortisol - brain.hpa.cortisol).abs() < f32::EPSILON);
}

#[test]
fn test_sleep_deprivation_cascade() {
    let mut brain = BrainState::default();
    // Keep awake for 24 hours (in 1-minute steps)
    for _ in 0..1440 {
        brain.tick(60.0).unwrap();
    }
    // Adenosine should be high, sleep pressure elevated
    assert!(brain.sleep.adenosine > 0.8);
    assert!(brain.sleep.sleep_pressure() > 0.5);
    // Sleep debt should have accumulated
    assert!(brain.sleep.sleep_debt > 1.0);
}

#[test]
fn test_stress_rumination_feedback() {
    let mut brain = BrainState::default();
    // Set high rumination
    brain.dmn.rumination = 0.8;
    let initial_cortisol = brain.hpa.cortisol;

    // Tick for 2 hours — rumination drives cortisol up (realistic cascade timing)
    for _ in 0..120 {
        brain.tick(60.0).unwrap();
    }
    assert!(brain.hpa.cortisol > initial_cortisol);
    // Allostatic load should accumulate from chronic stress
    assert!(brain.hpa.allostatic_load > 0.0);
}

#[test]
fn test_full_day_subsystem_coherence() {
    let mut brain = BrainState::default();
    let a = brain
        .circuit
        .add_population(NeuralPopulation::new("exc", 0.5, 0.1, true));
    let b = brain
        .circuit
        .add_population(NeuralPopulation::new("inh", 0.2, 0.2, false));
    brain.circuit.add_synapse(a, b, 0.5).unwrap();
    brain.circadian.phase_hours = 6.0; // early morning

    // Simulate 24 hours in 5-minute steps
    for _ in 0..288 {
        brain.tick(300.0).unwrap();
    }

    // All values should remain in valid ranges
    assert!((0.0..=1.0).contains(&brain.arousal()));
    assert!((0.0..=1.0).contains(&brain.stress()));
    assert!((0.0..=1.0).contains(&brain.neurotransmitter.serotonin.level));
    assert!((0.0..=1.0).contains(&brain.hpa.cortisol));
    assert!((0.0..=1.0).contains(&brain.circadian.melatonin));
}

// ── Pharmacology integration ───────────────────────────────────────

#[test]
fn test_ssri_raises_serotonin_over_time() {
    let mut brain = BrainState::default();
    brain.administer_drug(DrugProfile::ssri_fluoxetine(), 0.8);

    // Simulate 2 hours (past onset, drug is active)
    for _ in 0..7200 {
        brain.tick(1.0).unwrap();
    }

    // Serotonin clearance should be reduced, causing level to stay elevated
    let default_clearance = NeurotransmitterProfile::default().serotonin.clearance_rate;
    assert!(brain.neurotransmitter.serotonin.clearance_rate < default_clearance);
}

#[test]
fn test_benzodiazepine_amplifies_gaba_in_circuit() {
    let mut brain = BrainState::default();
    let a = brain
        .circuit
        .add_population(NeuralPopulation::new("exc", 0.5, 0.1, true));
    let b = brain
        .circuit
        .add_population(NeuralPopulation::new("inh", 0.3, 0.2, false));
    brain.circuit.add_synapse(a, b, 0.5).unwrap();

    brain.administer_drug(DrugProfile::benzodiazepine_diazepam(), 0.7);

    // Past onset
    for _ in 0..3600 {
        brain.tick(1.0).unwrap();
    }

    assert!(brain.pharmacology.gaba_pam_multiplier() > 1.0);
}

#[test]
fn test_drug_with_brain_state_stability_24hr() {
    let mut brain = BrainState::default();
    brain.administer_drug(DrugProfile::ssri_fluoxetine(), 0.5);
    brain.administer_drug(DrugProfile::benzodiazepine_diazepam(), 0.3);

    // Simulate 24 hours in 1-minute steps
    for _ in 0..1440 {
        brain.tick(60.0).unwrap();
    }

    // All values should remain in valid ranges
    assert!((0.0..=1.0).contains(&brain.neurotransmitter.serotonin.level));
    assert!((0.0..=1.0).contains(&brain.hpa.cortisol));
    assert!((0.0..=1.0).contains(&brain.arousal()));
    assert!((0.0..=1.0).contains(&brain.stress()));
}

// ── 0.7.0 module integration tests ─────────────────────────────────

#[test]
fn test_inflammation_drives_sickness_in_brain() {
    let mut brain = BrainState::default();
    brain.inflammation.infect(0.9);

    // Tick 5 minutes — inflammation should produce sickness behavior + tryptophan depletion
    for _ in 0..5 {
        brain.tick(60.0).unwrap();
    }
    assert!(brain.inflammation.sickness_behavior > 0.0);
    assert!(brain.inflammation.tryptophan_depletion() > 0.0);
}

#[test]
fn test_autonomic_responds_to_threat_in_brain() {
    let mut brain = BrainState::default();
    brain.amygdala.perceive_threat(0.9);

    for _ in 0..60 {
        brain.tick(1.0).unwrap();
    }
    // Sympathetic should be elevated from amygdala threat
    assert!(brain.autonomic.sympathetic > 0.3);
}

#[test]
fn test_eeg_delta_dominant_during_nrem3() {
    let mut brain = BrainState::default();
    brain.sleep.fall_asleep();
    brain.sleep.stage = SleepStage::Nrem3;

    // Tick to let EEG converge
    for _ in 0..30 {
        brain.tick(1.0).unwrap();
    }
    assert_eq!(brain.eeg.dominant_band(), EegBand::Delta);
}

#[test]
fn test_photoperiod_winter_reduces_serotonin_modifier() {
    let mut brain = BrainState::default();
    brain.circadian.set_photoperiod(8.0); // winter
    let winter_mod = brain.circadian.serotonin_photoperiod_modifier();

    brain.circadian.set_photoperiod(16.0); // summer
    let summer_mod = brain.circadian.serotonin_photoperiod_modifier();

    assert!(winter_mod < summer_mod);
    assert!(winter_mod < 1.0);
    assert!(summer_mod > 1.0);
}

#[test]
fn test_age_pfc_maturation_curve() {
    use mastishk::brain::AgeProfile;
    let teen = AgeProfile { age_years: 15.0 };
    let adult = AgeProfile { age_years: 30.0 };
    let elder = AgeProfile { age_years: 70.0 };

    assert!(teen.pfc_maturation() < adult.pfc_maturation());
    assert!(elder.pfc_maturation() < adult.pfc_maturation());
}

#[test]
fn test_age_dopamine_decline() {
    use mastishk::brain::AgeProfile;
    let young = AgeProfile { age_years: 30.0 };
    let old = AgeProfile { age_years: 60.0 };

    assert!((young.dopamine_capacity() - 1.0).abs() < f32::EPSILON);
    assert!(old.dopamine_capacity() < 1.0);
}

#[test]
fn test_spiking_standalone_simulation() {
    let mut net = SpikingNetwork::new();
    let a = net.add_neuron(SpikingNeuron::Izhikevich(
        IzhikevichNeuron::regular_spiking(),
    ));
    let b = net.add_neuron(SpikingNeuron::Lif(LifNeuron::default_params()));
    net.add_synapse(a, b, 3.0, 1.0).unwrap();

    let mut a_spikes = 0;
    for _ in 0..2000 {
        net.tick(0.5).unwrap();
        for &idx in net.last_spikes() {
            if idx == a {
                a_spikes += 1;
            }
        }
        // Drive A with tonic input
        if let SpikingNeuron::Izhikevich(ref mut n) = net.neurons[a] {
            n.v += 5.0; // external drive
        }
    }
    assert!(a_spikes > 0, "A should spike");
}

#[test]
fn test_brain_mood_modifiers_all_fields_finite() {
    let mut brain = BrainState::default();
    brain.inflammation.infect(0.3);
    brain.amygdala.perceive_threat(0.5);
    brain.dmn.rumination = 0.4;

    for _ in 0..60 {
        brain.tick(60.0).unwrap();
    }

    let effect = brain_mood_modifiers(&brain);
    assert!(effect.mood_offset.is_finite());
    assert!(effect.sickness_behavior.is_finite());
    assert!(effect.sympathetic.is_finite());
    assert!(effect.hrv.is_finite());
    assert!(effect.interoceptive_anxiety.is_finite());
    assert!(effect.seasonal_modifier.is_finite());
    assert!(effect.executive_control.is_finite());
    assert!(effect.fear_level.is_finite());
    assert!(effect.motor_quality.is_finite());
}

#[test]
fn test_full_system_24hr_stability_with_all_subsystems() {
    let mut brain = BrainState::default();
    let a = brain
        .circuit
        .add_population(NeuralPopulation::new("exc", 0.5, 0.1, true));
    let b = brain
        .circuit
        .add_population(NeuralPopulation::new("inh", 0.2, 0.2, false));
    brain.circuit.add_synapse(a, b, 0.5).unwrap();
    brain.inflammation.infect(0.2);
    brain.circadian.set_photoperiod(9.0); // winter-ish
    brain.dmn.rumination = 0.3;
    brain.administer_drug(DrugProfile::ssri_fluoxetine(), 0.4);

    // 24 hours in 5-minute steps
    for _ in 0..288 {
        brain.tick(300.0).unwrap();
    }

    // Everything should remain in valid ranges
    assert!((0.0..=1.0).contains(&brain.neurotransmitter.serotonin.level));
    assert!((0.0..=1.0).contains(&brain.hpa.cortisol));
    assert!((0.0..=1.0).contains(&brain.autonomic.sympathetic));
    assert!((0.0..=1.0).contains(&brain.autonomic.parasympathetic));
    assert!((0.0..=1.0).contains(&brain.eeg.delta));
    assert!(brain.inflammation.sickness_behavior >= 0.0);
    assert!(brain.arousal().is_finite());
    assert!(brain.stress().is_finite());
}

// ── Serde completeness — missing type roundtrips ───────────────────

#[test]
fn test_serde_interoceptive_state() {
    let s = InteroceptiveState::default();
    let json = serde_json::to_string(&s).unwrap();
    let s2: InteroceptiveState = serde_json::from_str(&json).unwrap();
    assert!((s2.prediction_error - s.prediction_error).abs() < f32::EPSILON);
}

#[test]
fn test_serde_age_profile() {
    let a = AgeProfile { age_years: 45.0 };
    let json = serde_json::to_string(&a).unwrap();
    let a2: AgeProfile = serde_json::from_str(&json).unwrap();
    assert!((a2.age_years - 45.0).abs() < f32::EPSILON);
}

#[test]
fn test_serde_sex_hormone_state() {
    let h = SexHormoneState::default();
    let json = serde_json::to_string(&h).unwrap();
    let h2: SexHormoneState = serde_json::from_str(&json).unwrap();
    assert!((h2.estradiol - h.estradiol).abs() < f32::EPSILON);
}

#[test]
fn test_serde_coupling_params() {
    let c = CouplingParams::default();
    let json = serde_json::to_string(&c).unwrap();
    let c2: CouplingParams = serde_json::from_str(&json).unwrap();
    assert!((c2.dmn_hpa_gain - c.dmn_hpa_gain).abs() < f32::EPSILON);
}

#[test]
fn test_serde_stdp_rule() {
    let s = StdpRule::default();
    let json = serde_json::to_string(&s).unwrap();
    let s2: StdpRule = serde_json::from_str(&json).unwrap();
    assert!((s2.a_plus - s.a_plus).abs() < f32::EPSILON);
}

#[test]
fn test_serde_bcm_rule() {
    let b = BcmRule::default();
    let json = serde_json::to_string(&b).unwrap();
    let b2: BcmRule = serde_json::from_str(&json).unwrap();
    assert!((b2.theta_m - b.theta_m).abs() < f32::EPSILON);
}

#[test]
fn test_serde_drug_profile() {
    let d = DrugProfile::ssri_fluoxetine();
    let json = serde_json::to_string(&d).unwrap();
    let d2: DrugProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(d2.name, "fluoxetine");
    assert!((d2.half_life - d.half_life).abs() < f32::EPSILON);
}
