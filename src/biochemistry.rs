//! Biochemistry bridge — rasayan integration for precursor-driven neurotransmitter kinetics.
//!
//! Uses rasayan's enzyme kinetics and bridge functions to modulate mastishk's
//! neurotransmitter synthesis rates based on metabolic precursor availability.
//!
//! Requires the `biochemistry` feature.
//!
//! # Coupling Points
//!
//! - **Tryptophan → Serotonin**: TPH kinetics via [`rasayan::neurotransmitter::serotonin_synthesis_rate`]
//! - **Tyrosine → Dopamine**: TH kinetics via [`rasayan::neurotransmitter::dopamine_level`]
//! - **Dopamine → Norepinephrine**: DBH kinetics via [`rasayan::neurotransmitter::norepinephrine_level`]
//! - **GABA/Glutamate balance**: via [`rasayan::neurotransmitter::gaba_glutamate_ratio`]
//! - **Choline + Acetyl-CoA → ACh**: ChAT kinetics via [`rasayan::neurotransmitter::acetylcholine_level`]
//! - **POMC → Endorphins**: via [`rasayan::neurotransmitter::endorphin_level`]
//! - **HPA cortisol**: via [`rasayan::hormonal::cortisol_from_hpa`]
//! - **Serotonin → Melatonin**: via [`rasayan::hormonal::melatonin_from_serotonin`]

use serde::{Deserialize, Serialize};

use crate::neurotransmitter::NeurotransmitterProfile;

/// Metabolic precursor state driving neurotransmitter synthesis.
///
/// These values represent normalized availability (0.0–1.0) of biochemical
/// precursors. In a full simulation, they would come from rasayan's metabolic
/// pathway modules. For standalone use, defaults represent well-fed resting state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecursorState {
    /// Tryptophan availability (serotonin precursor). Depleted by IDO in inflammation.
    pub tryptophan: f32,
    /// Tyrosine availability (dopamine/NE precursor).
    pub tyrosine: f32,
    /// Choline availability (acetylcholine precursor).
    pub choline: f32,
    /// Acetyl-CoA availability (ACh co-substrate, from metabolism).
    pub acetyl_coa: f32,
    /// Glutamine availability (GABA/glutamate precursor).
    pub glutamine: f32,
    /// POMC processing rate (endorphin precursor, stress/exercise-driven).
    pub pomc_activity: f32,
    /// Overall enzyme activity scaling (1.0 = normal, reduced by fatigue/illness).
    pub enzyme_activity: f32,
}

impl Default for PrecursorState {
    fn default() -> Self {
        Self {
            tryptophan: 0.8,
            tyrosine: 0.8,
            choline: 0.7,
            acetyl_coa: 0.8,
            glutamine: 0.7,
            pomc_activity: 0.3,
            enzyme_activity: 1.0,
        }
    }
}

/// Output of biochemistry-driven synthesis rate modulation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BiochemFlux {
    /// Serotonin synthesis rate from TPH kinetics.
    pub serotonin_synthesis: f32,
    /// Dopamine steady-state from TH kinetics.
    pub dopamine_level: f32,
    /// Norepinephrine from DBH acting on dopamine.
    pub norepinephrine_level: f32,
    /// GABA/glutamate ratio.
    pub gaba_glutamate_ratio: f32,
    /// ACh level from ChAT kinetics.
    pub acetylcholine_level: f32,
    /// Endorphin level from POMC processing.
    pub endorphin_level: f32,
}

/// Compute biochemistry-driven neurotransmitter synthesis using rasayan kinetics.
///
/// Returns a [`BiochemFlux`] with enzyme-kinetic synthesis rates for each
/// neurotransmitter. These can be used to modulate mastishk's
/// [`NeurotransmitterProfile`] baselines.
#[must_use]
pub fn compute_biochem_flux(precursors: &PrecursorState) -> BiochemFlux {
    let ea = f64::from(precursors.enzyme_activity);

    BiochemFlux {
        serotonin_synthesis: rasayan::neurotransmitter::serotonin_synthesis_rate(
            f64::from(precursors.tryptophan),
            ea,
        ) as f32,
        dopamine_level: rasayan::neurotransmitter::dopamine_level(
            f64::from(precursors.tyrosine),
            ea,
            0.05, // default reuptake rate
        ) as f32,
        norepinephrine_level: rasayan::neurotransmitter::norepinephrine_level(
            f64::from(precursors.tyrosine) * 0.5, // DA precursor fraction
            ea,
            0.03, // NE degradation rate
        ) as f32,
        gaba_glutamate_ratio: rasayan::neurotransmitter::gaba_glutamate_ratio(
            f64::from(precursors.glutamine) * 0.4, // GAD fraction
            f64::from(precursors.glutamine) * 0.6, // glutaminase fraction
        ) as f32,
        acetylcholine_level: rasayan::neurotransmitter::acetylcholine_level(
            f64::from(precursors.choline),
            f64::from(precursors.acetyl_coa),
            0.2, // AChE rate
        ) as f32,
        endorphin_level: rasayan::neurotransmitter::endorphin_level(
            f64::from(precursors.pomc_activity),
            0.03, // peptidase degradation
        ) as f32,
    }
}

/// Apply biochemistry flux to modulate neurotransmitter baselines.
///
/// Blends the enzyme-kinetic synthesis rates into the mastishk neurotransmitter
/// profile. The `blend` factor (0.0–1.0) controls how much biochemistry
/// influences the baselines vs. the existing neural-level model.
///
/// - `blend = 0.0` → no biochemistry influence (pure mastishk model)
/// - `blend = 1.0` → fully biochemistry-driven baselines
pub fn apply_biochem_flux(profile: &mut NeurotransmitterProfile, flux: &BiochemFlux, blend: f32) {
    let b = blend.clamp(0.0, 1.0);
    let inv = 1.0 - b;

    // Modulate synthesis rates (which drive baseline recovery in tick_all)
    profile.serotonin.synthesis_rate =
        inv * profile.serotonin.synthesis_rate + b * flux.serotonin_synthesis.max(0.005);
    profile.dopamine.synthesis_rate =
        inv * profile.dopamine.synthesis_rate + b * flux.dopamine_level.max(0.005);
    profile.norepinephrine.synthesis_rate =
        inv * profile.norepinephrine.synthesis_rate + b * flux.norepinephrine_level.max(0.005);
    profile.acetylcholine.synthesis_rate =
        inv * profile.acetylcholine.synthesis_rate + b * flux.acetylcholine_level.max(0.005);
    profile.endorphins.synthesis_rate =
        inv * profile.endorphins.synthesis_rate + b * flux.endorphin_level.max(0.001);

    // GABA/glutamate balance: shift baselines based on ratio
    let ratio = flux.gaba_glutamate_ratio;
    if ratio > 0.5 {
        // GABA-dominant: slightly raise GABA baseline, lower glutamate
        profile.gaba.baseline = (inv * profile.gaba.baseline + b * 0.55).clamp(0.2, 0.8);
        profile.glutamate.baseline = (inv * profile.glutamate.baseline + b * 0.45).clamp(0.2, 0.8);
    } else {
        // Glutamate-dominant
        profile.gaba.baseline = (inv * profile.gaba.baseline + b * 0.45).clamp(0.2, 0.8);
        profile.glutamate.baseline = (inv * profile.glutamate.baseline + b * 0.55).clamp(0.2, 0.8);
    }

    tracing::trace!(
        serotonin_synth = flux.serotonin_synthesis,
        dopamine = flux.dopamine_level,
        blend = b,
        "applied biochem flux to NT profile"
    );
}

/// Compute HPA cortisol with rasayan's kinetic model.
///
/// Wraps [`rasayan::hormonal::cortisol_from_hpa`] for use with mastishk's HPA state.
#[must_use]
#[inline]
pub fn cortisol_kinetic(crh: f64, acth: f64, feedback: f64) -> f64 {
    rasayan::hormonal::cortisol_from_hpa(crh, acth, feedback)
}

/// Compute melatonin synthesis with rasayan's kinetic model.
///
/// Wraps [`rasayan::hormonal::melatonin_from_serotonin`] for circadian integration.
#[must_use]
#[inline]
pub fn melatonin_kinetic(serotonin: f64, nat_activity: f64, light_suppression: f64) -> f64 {
    rasayan::hormonal::melatonin_from_serotonin(serotonin, nat_activity, light_suppression)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_precursors() {
        let p = PrecursorState::default();
        assert!(p.tryptophan > 0.0);
        assert!(p.enzyme_activity > 0.0);
    }

    #[test]
    fn test_compute_biochem_flux() {
        let flux = compute_biochem_flux(&PrecursorState::default());
        assert!(flux.serotonin_synthesis > 0.0);
        assert!(flux.dopamine_level > 0.0);
        assert!(flux.norepinephrine_level > 0.0);
        assert!(flux.acetylcholine_level > 0.0);
        assert!(flux.endorphin_level > 0.0);
    }

    #[test]
    fn test_depleted_tryptophan_reduces_serotonin() {
        let normal = compute_biochem_flux(&PrecursorState::default());
        let depleted = compute_biochem_flux(&PrecursorState {
            tryptophan: 0.1,
            ..PrecursorState::default()
        });
        assert!(
            depleted.serotonin_synthesis < normal.serotonin_synthesis,
            "Low tryptophan should reduce serotonin synthesis"
        );
    }

    #[test]
    fn test_apply_biochem_flux_zero_blend() {
        let mut profile = NeurotransmitterProfile::default();
        let original_rate = profile.serotonin.synthesis_rate;
        let flux = compute_biochem_flux(&PrecursorState::default());
        apply_biochem_flux(&mut profile, &flux, 0.0);
        assert!(
            (profile.serotonin.synthesis_rate - original_rate).abs() < f32::EPSILON,
            "Zero blend should not change synthesis rate"
        );
    }

    #[test]
    fn test_apply_biochem_flux_full_blend() {
        let mut profile = NeurotransmitterProfile::default();
        let flux = compute_biochem_flux(&PrecursorState::default());
        apply_biochem_flux(&mut profile, &flux, 1.0);
        // Should have been modified
        assert!(profile.serotonin.synthesis_rate > 0.0);
    }

    #[test]
    fn test_cortisol_kinetic() {
        let c = cortisol_kinetic(0.5, 0.3, 0.1);
        assert!(c > 0.0);
    }

    #[test]
    fn test_melatonin_kinetic() {
        let dark = melatonin_kinetic(0.5, 1.0, 0.0);
        let bright = melatonin_kinetic(0.5, 1.0, 1.0);
        assert!(dark > bright, "Light should suppress melatonin");
    }
}
