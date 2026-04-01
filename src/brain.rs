//! Integrated brain state — unified orchestration of all neuroscience subsystems.
//!
//! [`BrainState`] owns all six domain modules and applies cross-module
//! couplings in the correct causal order during each tick. Consumers who
//! want the full integrated system use this; those who need individual
//! modules can use them directly with [`crate::coupling`] functions.

use serde::{Deserialize, Serialize};

use crate::chronobiology::CircadianState;
use crate::circuit::Circuit;
use crate::coupling::{
    CouplingParams, apply_amygdala_hpa_coupling, apply_arousal_circuit_coupling,
    apply_circadian_hpa_coupling, apply_dmn_hpa_coupling, apply_nt_amygdala_coupling,
    apply_nt_basal_ganglia_coupling, apply_nt_cerebellum_coupling, apply_nt_hippocampus_coupling,
    apply_nt_pfc_coupling, apply_sleep_neurotransmitter_coupling, composite_arousal,
    composite_stress,
};
use crate::dmn::DmnState;
use crate::error::{MastishkError, validate_dt};
use crate::hpa::HpaState;
use crate::neurotransmitter::NeurotransmitterProfile;
use crate::pharmacology::{DrugProfile, PharmacologyState};
use crate::regions::{
    AmygdalaState, BasalGangliaState, CerebellumState, HippocampusState, PfcState,
    RewardCircuitState,
};
use crate::sleep::SleepState;

/// Unified brain state combining all neuroscience subsystems with cross-module coupling.
///
/// Provides a single `tick(dt)` that advances all subsystems in the correct
/// causal order, applying biological couplings between them.
///
/// # Time Units
///
/// `tick` accepts `dt` in **seconds**. Internally converts to hours for
/// circadian and sleep subsystems.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrainState {
    /// Neurotransmitter dynamics (serotonin, dopamine, NE, GABA, glutamate, etc.).
    pub neurotransmitter: NeurotransmitterProfile,
    /// Neural circuit (populations + synapses).
    pub circuit: Circuit,
    /// Sleep architecture (stage, adenosine, debt).
    pub sleep: SleepState,
    /// HPA axis stress response (CRH → ACTH → cortisol).
    pub hpa: HpaState,
    /// Default mode network (DMN/TPN balance, rumination).
    pub dmn: DmnState,
    /// Circadian rhythms (SCN pacemaker, melatonin, cortisol CAR).
    pub circadian: CircadianState,
    /// Cross-module coupling parameters.
    pub coupling: CouplingParams,
    /// Receptor pharmacology (active drugs, receptor states).
    #[serde(default)]
    pub pharmacology: PharmacologyState,
    /// Prefrontal cortex (executive function, impulse control, working memory).
    #[serde(default)]
    pub pfc: PfcState,
    /// Amygdala (threat detection, fear conditioning, emotional salience).
    #[serde(default)]
    pub amygdala: AmygdalaState,
    /// Hippocampus (memory formation, context encoding, neurogenesis).
    #[serde(default)]
    pub hippocampus: HippocampusState,
    /// Basal ganglia (action selection, habits, reward prediction error).
    #[serde(default)]
    pub basal_ganglia: BasalGangliaState,
    /// Cerebellum (motor precision, timing, error correction).
    #[serde(default)]
    pub cerebellum: CerebellumState,
    /// VTA/Nucleus Accumbens reward circuit (incentive salience, wanting, craving).
    #[serde(default)]
    pub reward_circuit: RewardCircuitState,
    /// Sex hormone levels (slow-changing, modulate NT synthesis and region reactivity).
    #[serde(default)]
    pub hormones: SexHormoneState,
    /// Neuroinflammation state (microglia, cytokines, sickness behavior).
    #[serde(default)]
    pub inflammation: crate::inflammation::InflammationState,
    /// Gut-brain axis (enteric serotonin, vagal tone, microbiome).
    #[serde(default)]
    pub gut_brain: crate::gut_brain::GutBrainState,
    /// Autonomic nervous system (sympathetic/parasympathetic, HRV).
    #[serde(default)]
    pub autonomic: crate::autonomic::AutonomicState,
    /// Interoceptive inference state (predictive processing of body state).
    #[serde(default)]
    pub interoception: InteroceptiveState,
    /// EEG band power state (observable correlates).
    #[serde(default)]
    pub eeg: crate::eeg::EegState,
    /// Age profile (modulates PFC maturation, dopamine capacity, sleep depth).
    #[serde(default)]
    pub age: AgeProfile,
}

/// Sex hormone levels — slow-changing modulators of neurotransmitter dynamics
/// and brain region reactivity.
///
/// Estradiol enhances serotonin synthesis and reduces MAO-B activity (mood stabilization).
/// Testosterone modulates amygdala reactivity and risk-taking via androgen receptors.
/// Both are trait-like (change over weeks/months), set by consumer per character profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SexHormoneState {
    /// Estradiol level (0.0–1.0). Higher → enhanced serotonin synthesis, mood stability.
    pub estradiol: f32,
    /// Testosterone level (0.0–1.0). Higher → increased amygdala reactivity, risk-taking.
    pub testosterone: f32,
}

impl Default for SexHormoneState {
    fn default() -> Self {
        Self {
            estradiol: 0.5,
            testosterone: 0.5,
        }
    }
}

/// Interoceptive inference state — predictive processing of body state (Seth 2013).
///
/// The brain predicts body state; mismatch (prediction error) contributes to
/// anxiety and panic. Higher interoceptive accuracy means better body-state sensing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteroceptiveState {
    /// Brain's prediction of body state (0.0–1.0).
    pub body_prediction: f32,
    /// Prediction error (0.0–1.0). Mismatch between prediction and actual.
    pub prediction_error: f32,
    /// Interoceptive accuracy (0.0–1.0). Trait-like body-sensing ability.
    pub interoceptive_accuracy: f32,
}

impl Default for InteroceptiveState {
    fn default() -> Self {
        Self {
            body_prediction: 0.5,
            prediction_error: 0.0,
            interoceptive_accuracy: 0.5,
        }
    }
}

impl InteroceptiveState {
    /// Update prediction error from actual body state (autonomic balance).
    #[inline]
    pub fn update_from_autonomic(&mut self, autonomic_balance: f32, dt: f32) {
        let actual = autonomic_balance;
        let raw_error = (self.body_prediction - actual).abs() * self.interoceptive_accuracy;
        let alpha = 1.0 - (-0.2 * dt).exp();
        self.prediction_error += (raw_error - self.prediction_error) * alpha;
        self.prediction_error = self.prediction_error.clamp(0.0, 1.0);
        // Slowly update prediction toward actual (learning)
        let learn_alpha = 1.0 - (-0.05 * dt).exp();
        self.body_prediction += (actual - self.body_prediction) * learn_alpha;
    }

    /// Anxiety contribution from interoceptive prediction error (0.0–1.0).
    #[inline]
    #[must_use]
    pub fn anxiety_contribution(&self) -> f32 {
        (self.prediction_error * 1.5).clamp(0.0, 1.0)
    }
}

/// Age-related parameter modifiers for neurobiological development and decline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeProfile {
    /// Age in years.
    pub age_years: f32,
}

impl Default for AgeProfile {
    fn default() -> Self {
        Self { age_years: 30.0 }
    }
}

impl AgeProfile {
    /// PFC maturation factor (0.0–1.0). Executive function peaks ~25 years.
    #[inline]
    #[must_use]
    pub fn pfc_maturation(&self) -> f32 {
        let rising = 1.0 / (1.0 + (-0.3 * (self.age_years - 18.0)).exp());
        let decline = if self.age_years > 60.0 {
            1.0 - (self.age_years - 60.0) * 0.005
        } else {
            1.0
        };
        (rising * decline).clamp(0.0, 1.0)
    }

    /// Dopaminergic capacity (0.0–1.0). ~10% decline per decade after 40.
    #[inline]
    #[must_use]
    pub fn dopamine_capacity(&self) -> f32 {
        if self.age_years <= 40.0 {
            1.0
        } else {
            (1.0 - 0.01 * (self.age_years - 40.0)).clamp(0.5, 1.0)
        }
    }

    /// Deep sleep capacity (0.0–1.0). NREM3 decreases with age from ~20.
    #[inline]
    #[must_use]
    pub fn deep_sleep_capacity(&self) -> f32 {
        (1.0 - 0.008 * (self.age_years - 20.0).max(0.0)).clamp(0.3, 1.0)
    }
}

impl BrainState {
    /// Advance all subsystems by `dt` seconds, applying cross-module couplings.
    ///
    /// # Tick Order (~30 steps, causal flow)
    ///
    /// Circadian → HPA/NT couplings → Pharmacology → NT tick → Hormones →
    /// Age → Regions (amygdala→hippocampus→PFC) → HPA stress inputs →
    /// Inflammation chain → Gut-brain → Basal ganglia → Cerebellum →
    /// Region ticks → Autonomic → Interoception → Circuit → Sleep → EEG →
    /// Photoperiod → Age modifiers
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        let dt_hours = dt / 3600.0;

        tracing::trace!(dt, dt_hours, "brain state tick");

        // 1. Circadian clock (master pacemaker)
        self.circadian.tick(dt_hours)?;

        // 2. Circadian → HPA (cortisol baseline from CAR)
        apply_circadian_hpa_coupling(
            &self.circadian,
            &mut self.hpa,
            self.coupling.circadian_hpa_smoothing,
            dt,
        )?;

        // 3. Sleep → neurotransmitter (stage-driven baselines)
        apply_sleep_neurotransmitter_coupling(
            self.sleep.stage,
            &mut self.neurotransmitter,
            self.coupling.sleep_nt_smoothing,
            dt,
        )?;

        // 4. Pharmacology (drug PK, receptor desensitization, NT modification)
        self.pharmacology.tick(dt, &mut self.neurotransmitter)?;

        // 5. Neurotransmitter tick
        self.neurotransmitter.tick_all(dt)?;

        // 5.5 Sex hormone modulation (slow-acting, modulates NT synthesis + region reactivity)
        {
            let alpha = 1.0 - (-0.01 * dt).exp();
            // Estradiol → serotonin synthesis boost (mood stabilization)
            let estradiol_boost = (self.hormones.estradiol - 0.5) * 0.02;
            self.neurotransmitter.serotonin.synthesis_rate += estradiol_boost * alpha;
            self.neurotransmitter.serotonin.synthesis_rate = self
                .neurotransmitter
                .serotonin
                .synthesis_rate
                .clamp(0.005, 0.1);
            // Testosterone → amygdala activation boost (risk-taking, reactivity)
            let testosterone_boost = (self.hormones.testosterone - 0.5) * 0.05;
            self.amygdala.activation =
                (self.amygdala.activation + testosterone_boost * alpha).clamp(0.0, 1.0);
        }

        // 6. NT → Amygdala coupling (sensory region first)
        apply_nt_amygdala_coupling(
            &self.neurotransmitter,
            &mut self.amygdala,
            &self.pfc,
            &self.coupling.region,
            dt,
        )?;

        // 7. NT → Hippocampus coupling (context)
        apply_nt_hippocampus_coupling(
            &self.neurotransmitter,
            &mut self.hippocampus,
            &self.amygdala,
            &self.sleep,
            &self.coupling.region,
            dt,
        )?;

        // 8. NT → PFC coupling (executive, reads amygdala/hippocampus)
        apply_nt_pfc_coupling(
            &self.neurotransmitter,
            &mut self.pfc,
            &self.hpa,
            &self.sleep,
            &self.amygdala,
            dt,
        )?;

        // 9. Amygdala → HPA coupling (threat → stress)
        apply_amygdala_hpa_coupling(
            &self.amygdala,
            &mut self.hpa,
            self.coupling.region.amygdala_hpa_gain,
            dt,
        )?;

        // 10. DMN → HPA (rumination as chronic stressor)
        apply_dmn_hpa_coupling(&self.dmn, &mut self.hpa, &self.coupling, dt)?;

        // 10.5 Inflammation → HPA (cytokines as stressor)
        if self.inflammation.cytokine_level > 0.1 {
            self.hpa.stress(self.inflammation.cytokine_level * 0.2 * dt);
        }

        // 11. HPA tick
        self.hpa.tick(dt)?;

        // 11.5 Gut-brain → inflammation (microbiome dampens inflammation)
        {
            let dampen = self.gut_brain.microbiome_diversity * 0.1;
            let alpha = 1.0 - (-0.02 * dt).exp();
            self.inflammation.microglial_activation =
                (self.inflammation.microglial_activation - dampen * alpha).max(0.0);
        }

        // 11.6 Inflammation → NT (tryptophan depletion → serotonin, cytokine fatigue → dopamine)
        {
            let depletion = self.inflammation.tryptophan_depletion();
            let alpha = 1.0 - (-0.05 * dt).exp();
            self.neurotransmitter.serotonin.synthesis_rate *= 1.0 - depletion * 0.3 * alpha;
            self.neurotransmitter.serotonin.synthesis_rate = self
                .neurotransmitter
                .serotonin
                .synthesis_rate
                .clamp(0.005, 0.1);
            // Sickness behavior reduces dopamine motivation
            let fatigue = self.inflammation.sickness_behavior * 0.15;
            self.neurotransmitter.dopamine.baseline =
                (self.neurotransmitter.dopamine.baseline - fatigue * alpha).max(0.1);
        }

        // 11.7 Inflammation tick
        self.inflammation.tick(dt)?;

        // 11.8 Gut-brain → NT (gut serotonin modulates central synthesis)
        {
            let modifier = self.gut_brain.central_serotonin_modifier();
            let alpha = 1.0 - (-0.01 * dt).exp();
            self.neurotransmitter.serotonin.synthesis_rate *= 1.0 + (modifier - 1.0) * alpha;
            self.neurotransmitter.serotonin.synthesis_rate = self
                .neurotransmitter
                .serotonin
                .synthesis_rate
                .clamp(0.005, 0.1);
        }

        // 11.9 Gut-brain tick
        self.gut_brain.tick(dt)?;

        // 12. NT → Basal Ganglia coupling
        apply_nt_basal_ganglia_coupling(
            &self.neurotransmitter,
            &mut self.basal_ganglia,
            &self.pfc,
            &self.hippocampus,
            &self.coupling.region,
            dt,
        )?;

        // 13. NT → Cerebellum coupling
        apply_nt_cerebellum_coupling(
            &self.neurotransmitter,
            &mut self.cerebellum,
            &self.sleep,
            dt,
        )?;

        // 14-18. Region ticks
        self.amygdala.tick(dt)?;
        self.hippocampus.tick(dt)?;
        self.pfc.tick(dt)?;
        self.basal_ganglia.tick(dt)?;
        self.cerebellum.tick(dt)?;
        self.reward_circuit.tick(dt)?;

        // 18.5 Autonomic coupling (driven by NE, cortisol, amygdala, vagal tone)
        {
            let alpha = 1.0 - (-0.1 * dt).exp();
            // NE + cortisol → sympathetic
            let sym_drive = self.neurotransmitter.norepinephrine.level * 0.3
                + self.hpa.cortisol * 0.2
                + self.amygdala.threat_response() * 0.25;
            self.autonomic.sympathetic = (self.autonomic.sympathetic + sym_drive * alpha).min(1.0);
            // Vagal tone → parasympathetic
            let para_drive = self.gut_brain.vagal_tone * 0.3;
            self.autonomic.parasympathetic =
                (self.autonomic.parasympathetic + para_drive * alpha).min(1.0);
        }

        // 18.6 Autonomic tick
        self.autonomic.tick(dt)?;

        // 18.7 Interoceptive coupling (PE from autonomic vs prediction)
        self.interoception
            .update_from_autonomic(self.autonomic.balance(), dt);

        // 19. Arousal → circuit coupling
        apply_arousal_circuit_coupling(
            &self.neurotransmitter,
            &mut self.circuit,
            &self.coupling.circuit_gain,
            self.pharmacology.gaba_pam_multiplier(),
            dt,
        )?;

        // 20. Sleep adenosine dynamics
        self.sleep.tick_adenosine(dt_hours)?;
        self.sleep.tick_stage_transitions(dt_hours);

        // 20.5 EEG target computation + smooth transition
        {
            use crate::eeg::EegState;
            use crate::sleep::SleepStage;
            let target = match self.sleep.stage {
                SleepStage::Nrem3 => EegState {
                    delta: 0.8,
                    theta: 0.1,
                    alpha: 0.02,
                    beta: 0.02,
                    gamma: 0.01,
                },
                SleepStage::Nrem2 => EegState {
                    delta: 0.4,
                    theta: 0.3,
                    alpha: 0.1,
                    beta: 0.05,
                    gamma: 0.02,
                },
                SleepStage::Nrem1 => EegState {
                    delta: 0.2,
                    theta: 0.4,
                    alpha: 0.2,
                    beta: 0.1,
                    gamma: 0.05,
                },
                SleepStage::Rem => EegState {
                    delta: 0.1,
                    theta: 0.3,
                    alpha: 0.1,
                    beta: 0.3,
                    gamma: 0.2,
                },
                SleepStage::Wake => {
                    let focus = self.pfc.executive_control;
                    let meditation = self.dmn.meditation_depth;
                    EegState {
                        delta: 0.05,
                        theta: 0.1 + meditation * 0.3,
                        alpha: 0.4 * (1.0 - focus) + meditation * 0.2,
                        beta: 0.3 * focus + 0.1,
                        gamma: 0.1 + self.amygdala.activation * 0.2,
                    }
                }
            };
            self.eeg.tick_toward(&target, dt)?;
        }

        // 20.6 Photoperiod → serotonin synthesis (seasonal effect)
        {
            let modifier = self.circadian.serotonin_photoperiod_modifier();
            let alpha = 1.0 - (-0.001 * dt).exp(); // very slow (seasonal timescale)
            self.neurotransmitter.serotonin.synthesis_rate *= 1.0 + (modifier - 1.0) * alpha;
            self.neurotransmitter.serotonin.synthesis_rate = self
                .neurotransmitter
                .serotonin
                .synthesis_rate
                .clamp(0.005, 0.1);
        }

        // 20.7 Age modifiers (multiplicative, slow)
        {
            let pfc_factor = self.age.pfc_maturation();
            self.pfc.working_memory_capacity *= pfc_factor;
            self.pfc.working_memory_capacity = self.pfc.working_memory_capacity.clamp(0.2, 1.0);
        }

        Ok(())
    }

    /// Administer a drug at the given normalized dose (0.0–1.0).
    pub fn administer_drug(&mut self, profile: DrugProfile, dose: f32) {
        self.pharmacology.administer(profile, dose);
    }

    /// Composite arousal level (0.0–1.0) combining neurotransmitter, circadian,
    /// and sleep contributions.
    #[inline]
    #[must_use]
    pub fn arousal(&self) -> f32 {
        composite_arousal(&self.neurotransmitter, &self.circadian, &self.sleep)
    }

    /// Composite stress level (0.0–1.0) combining HPA cortisol, DMN rumination,
    /// and sleep debt.
    #[inline]
    #[must_use]
    pub fn stress(&self) -> f32 {
        composite_stress(&self.hpa, &self.dmn, &self.sleep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::NeuralPopulation;
    use crate::sleep::SleepStage;

    fn brain_with_circuit() -> BrainState {
        let mut brain = BrainState::default();
        let a = brain
            .circuit
            .add_population(NeuralPopulation::new("excitatory", 0.5, 0.1, true));
        let b = brain
            .circuit
            .add_population(NeuralPopulation::new("inhibitory", 0.2, 0.2, false));
        brain.circuit.add_synapse(a, b, 0.5).unwrap();
        brain.circuit.add_synapse(b, a, -0.3).unwrap();
        brain
    }

    #[test]
    fn test_default_creates_valid_state() {
        let brain = BrainState::default();
        assert!((0.0..=1.0).contains(&brain.arousal()));
        assert!((0.0..=1.0).contains(&brain.stress()));
    }

    #[test]
    fn test_tick_no_panic() {
        let mut brain = brain_with_circuit();
        brain.tick(1.0).unwrap();
        brain.tick(0.016).unwrap(); // ~60fps
        brain.tick(60.0).unwrap(); // 1 minute
    }

    #[test]
    fn test_tick_negative_dt_rejected() {
        let mut brain = BrainState::default();
        assert!(brain.tick(-1.0).is_err());
    }

    #[test]
    fn test_tick_zero_dt() {
        let mut brain = brain_with_circuit();
        brain.tick(0.0).unwrap(); // should be a no-op, no panic
    }

    #[test]
    fn test_serde_roundtrip() {
        let brain = brain_with_circuit();
        let json = serde_json::to_string(&brain).unwrap();
        let brain2: BrainState = serde_json::from_str(&json).unwrap();
        assert!(
            (brain2.neurotransmitter.serotonin.level - brain.neurotransmitter.serotonin.level)
                .abs()
                < f32::EPSILON
        );
        assert_eq!(brain2.circuit.populations.len(), 2);
    }

    #[test]
    fn test_circadian_drives_hpa_baseline() {
        let mut brain = BrainState::default();
        // Set to morning (high circadian cortisol)
        brain.circadian.phase_hours = 8.0;
        let initial_baseline = brain.hpa.cortisol_baseline;

        // Tick for a while
        for _ in 0..100 {
            brain.tick(1.0).unwrap();
        }
        // HPA baseline should have moved toward circadian cortisol
        let distance_now = (brain.hpa.cortisol_baseline - brain.circadian.cortisol_circadian).abs();
        let distance_initial = (initial_baseline - brain.circadian.cortisol_circadian).abs();
        assert!(distance_now < distance_initial);
    }

    #[test]
    fn test_sleep_rem_drives_ach_baseline() {
        let mut brain = BrainState::default();
        brain.sleep.stage = SleepStage::Rem;

        for _ in 0..100 {
            brain.tick(1.0).unwrap();
        }
        // ACh baseline should have risen toward 0.9 (REM target)
        assert!(brain.neurotransmitter.acetylcholine.baseline > 0.6);
        // Serotonin should have dropped toward 0.05
        assert!(brain.neurotransmitter.serotonin.baseline < 0.3);
    }

    #[test]
    fn test_rumination_drives_cortisol() {
        let mut brain = BrainState::default();
        brain.dmn.rumination = 0.8; // high rumination
        let initial_cortisol = brain.hpa.cortisol;

        for _ in 0..50 {
            brain.tick(1.0).unwrap();
        }
        assert!(brain.hpa.cortisol > initial_cortisol);
    }

    #[test]
    fn test_full_day_cycle() {
        let mut brain = brain_with_circuit();
        brain.circadian.phase_hours = 0.0; // start at midnight

        // Simulate 24 hours in 1-minute steps
        for _ in 0..1440 {
            brain.tick(60.0).unwrap();
        }
        // Phase should be back near 0 (midnight)
        assert!(brain.circadian.phase_hours < 1.0 || brain.circadian.phase_hours > 23.0);
        // System should still be in valid state
        assert!((0.0..=1.0).contains(&brain.arousal()));
        assert!((0.0..=1.0).contains(&brain.stress()));
    }

    #[test]
    fn test_arousal_drops_during_sleep() {
        let mut brain = BrainState::default();
        let awake_arousal = brain.arousal();

        brain.sleep.stage = SleepStage::Nrem3;
        let asleep_arousal = brain.arousal();

        assert!(asleep_arousal < awake_arousal);
    }

    #[test]
    fn test_all_values_finite_after_extended_simulation() {
        let mut brain = brain_with_circuit();
        brain.administer_drug(crate::pharmacology::DrugProfile::ssri_fluoxetine(), 0.5);
        brain.dmn.rumination = 0.6;
        brain.sleep.stage = SleepStage::Rem;

        // 1 hour in 1-second steps with diverse active state
        for _ in 0..3600 {
            brain.tick(1.0).unwrap();
        }

        // Verify no NaN/Inf in any subsystem
        assert!(brain.neurotransmitter.serotonin.level.is_finite());
        assert!(brain.neurotransmitter.dopamine.level.is_finite());
        assert!(brain.hpa.cortisol.is_finite());
        assert!(brain.hpa.allostatic_load.is_finite());
        assert!(brain.circadian.melatonin.is_finite());
        assert!(brain.arousal().is_finite());
        assert!(brain.stress().is_finite());
        assert!(brain.pharmacology.gaba_pam_multiplier().is_finite());
    }
}
