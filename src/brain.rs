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
}

impl BrainState {
    /// Advance all subsystems by `dt` seconds, applying cross-module couplings.
    ///
    /// # Tick Order (causal flow)
    ///
    ///  1. Circadian tick — master clock
    ///  2. Circadian → HPA coupling
    ///  3. Sleep → NT coupling
    ///  4. Pharmacology tick
    ///  5. NT tick
    ///  6. NT → Amygdala coupling (sensory first)
    ///  7. NT → Hippocampus coupling (context)
    ///  8. NT → PFC coupling (executive)
    ///  9. Amygdala → HPA coupling (threat → stress)
    /// 10. DMN → HPA coupling
    /// 11. HPA tick
    /// 12. NT → Basal Ganglia coupling
    /// 13. NT → Cerebellum coupling
    /// 14. Region ticks (amygdala, hippocampus, PFC, basal ganglia, cerebellum)
    /// 19. Arousal → circuit coupling
    /// 20. Sleep tick
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

        // 11. HPA tick
        self.hpa.tick(dt)?;

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
