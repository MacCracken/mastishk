//! Pharmacology — drug profiles, pharmacokinetics, dose-response, NT coupling.
//!
//! Models drug compounds interacting with receptor subtypes to modify
//! neurotransmitter dynamics. Supports reuptake inhibitors (SSRIs, stimulants),
//! receptor agonists/antagonists, and positive allosteric modulators (benzodiazepines).
//! Includes preset constructors for common drug classes.

use serde::{Deserialize, Serialize};

use crate::error::{MastishkError, validate_dt};
use crate::neurotransmitter::NeurotransmitterProfile;
use crate::receptor::{ReceptorMap, ReceptorOccupancies, ReceptorSubtype};

// ── Math Utilities ─────────────────────────────────────────────────

/// Hill equation dose-response curve.
///
/// Returns the effect magnitude (0.0–emax) for a given concentration.
/// `ec50` is the half-maximal effective concentration, `hill_coeff` controls
/// steepness, `emax` is the maximal effect.
#[inline]
#[must_use]
pub fn hill_equation(concentration: f32, ec50: f32, hill_coeff: f32, emax: f32) -> f32 {
    if concentration <= 0.0 || ec50 <= 0.0 {
        return 0.0;
    }
    let conc_n = concentration.powf(hill_coeff);
    let ec50_n = ec50.powf(hill_coeff);
    emax * conc_n / (ec50_n + conc_n)
}

// ── Drug Types ─────────────────────────────────────────────────────

/// How a drug interacts with a receptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DrugMechanism {
    /// Activates the receptor, increasing downstream signaling.
    Agonist,
    /// Blocks the receptor, reducing downstream signaling.
    Antagonist,
    /// Enhances endogenous ligand effect without direct activation (e.g., benzodiazepines on GABA-A).
    PositiveAllostericModulator,
    /// Blocks reuptake transporter, increasing synaptic transmitter concentration.
    ReuptakeInhibitor,
}

/// A drug's binding profile for a single receptor subtype.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorBinding {
    /// Target receptor subtype.
    pub receptor: ReceptorSubtype,
    /// Half-maximal effective concentration (normalized 0.0–1.0).
    pub ec50: f32,
    /// Hill coefficient (steepness of dose-response curve).
    pub hill_coeff: f32,
    /// Maximal effect magnitude (0.0–1.0).
    pub emax: f32,
    /// Mechanism of action at this receptor.
    pub mechanism: DrugMechanism,
}

/// Static pharmacological profile of a drug compound.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugProfile {
    /// Drug name.
    pub name: String,
    /// Receptor binding affinities and mechanisms.
    pub bindings: Vec<ReceptorBinding>,
    /// Plasma elimination half-life (seconds).
    pub half_life: f32,
    /// Time to peak absorption (seconds).
    pub onset_delay: f32,
}

impl DrugProfile {
    /// Fluoxetine (Prozac) — SSRI. Long half-life, serotonin reuptake inhibitor.
    #[must_use]
    pub fn ssri_fluoxetine() -> Self {
        Self {
            name: "fluoxetine".into(),
            bindings: vec![
                ReceptorBinding {
                    receptor: ReceptorSubtype::Ht1a,
                    ec50: 0.10,
                    hill_coeff: 1.5,
                    emax: 0.85,
                    mechanism: DrugMechanism::ReuptakeInhibitor,
                },
                ReceptorBinding {
                    receptor: ReceptorSubtype::Ht2a,
                    ec50: 0.12,
                    hill_coeff: 1.5,
                    emax: 0.80,
                    mechanism: DrugMechanism::ReuptakeInhibitor,
                },
            ],
            half_life: 86_400.0, // ~1 day (active metabolite much longer)
            onset_delay: 3600.0, // 1 hour absorption
        }
    }

    /// Sertraline (Zoloft) — SSRI. More selective than fluoxetine.
    #[must_use]
    pub fn ssri_sertraline() -> Self {
        Self {
            name: "sertraline".into(),
            bindings: vec![
                ReceptorBinding {
                    receptor: ReceptorSubtype::Ht1a,
                    ec50: 0.08,
                    hill_coeff: 1.8,
                    emax: 0.90,
                    mechanism: DrugMechanism::ReuptakeInhibitor,
                },
                ReceptorBinding {
                    receptor: ReceptorSubtype::Ht2a,
                    ec50: 0.10,
                    hill_coeff: 1.8,
                    emax: 0.85,
                    mechanism: DrugMechanism::ReuptakeInhibitor,
                },
            ],
            half_life: 93_600.0, // ~26 hours
            onset_delay: 3600.0,
        }
    }

    /// Diazepam (Valium) — benzodiazepine. GABA-A PAM, long half-life.
    #[must_use]
    pub fn benzodiazepine_diazepam() -> Self {
        Self {
            name: "diazepam".into(),
            bindings: vec![ReceptorBinding {
                receptor: ReceptorSubtype::GabaA,
                ec50: 0.15,
                hill_coeff: 1.2,
                emax: 0.80,
                mechanism: DrugMechanism::PositiveAllostericModulator,
            }],
            half_life: 172_800.0, // ~2 days
            onset_delay: 1800.0,  // 30 min
        }
    }

    /// Alprazolam (Xanax) — benzodiazepine. GABA-A PAM, short half-life.
    #[must_use]
    pub fn benzodiazepine_alprazolam() -> Self {
        Self {
            name: "alprazolam".into(),
            bindings: vec![ReceptorBinding {
                receptor: ReceptorSubtype::GabaA,
                ec50: 0.10,
                hill_coeff: 2.0,
                emax: 0.90,
                mechanism: DrugMechanism::PositiveAllostericModulator,
            }],
            half_life: 21_600.0, // ~6 hours
            onset_delay: 900.0,  // 15 min
        }
    }

    /// Amphetamine — stimulant. DA/NE agonist (release + reuptake block).
    #[must_use]
    pub fn stimulant_amphetamine() -> Self {
        Self {
            name: "amphetamine".into(),
            bindings: vec![
                ReceptorBinding {
                    receptor: ReceptorSubtype::D1,
                    ec50: 0.12,
                    hill_coeff: 1.5,
                    emax: 0.85,
                    mechanism: DrugMechanism::Agonist,
                },
                ReceptorBinding {
                    receptor: ReceptorSubtype::D2,
                    ec50: 0.15,
                    hill_coeff: 1.5,
                    emax: 0.70,
                    mechanism: DrugMechanism::Agonist,
                },
                ReceptorBinding {
                    receptor: ReceptorSubtype::Alpha1,
                    ec50: 0.20,
                    hill_coeff: 1.3,
                    emax: 0.60,
                    mechanism: DrugMechanism::Agonist,
                },
                ReceptorBinding {
                    receptor: ReceptorSubtype::Beta,
                    ec50: 0.25,
                    hill_coeff: 1.3,
                    emax: 0.50,
                    mechanism: DrugMechanism::Agonist,
                },
            ],
            half_life: 36_000.0, // ~10 hours
            onset_delay: 1800.0, // 30 min
        }
    }

    /// Methylphenidate (Ritalin) — stimulant. DA/NE reuptake inhibitor.
    #[must_use]
    pub fn stimulant_methylphenidate() -> Self {
        Self {
            name: "methylphenidate".into(),
            bindings: vec![
                ReceptorBinding {
                    receptor: ReceptorSubtype::D1,
                    ec50: 0.15,
                    hill_coeff: 1.3,
                    emax: 0.75,
                    mechanism: DrugMechanism::ReuptakeInhibitor,
                },
                ReceptorBinding {
                    receptor: ReceptorSubtype::D2,
                    ec50: 0.18,
                    hill_coeff: 1.3,
                    emax: 0.65,
                    mechanism: DrugMechanism::ReuptakeInhibitor,
                },
                ReceptorBinding {
                    receptor: ReceptorSubtype::Alpha1,
                    ec50: 0.25,
                    hill_coeff: 1.2,
                    emax: 0.50,
                    mechanism: DrugMechanism::ReuptakeInhibitor,
                },
            ],
            half_life: 10_800.0, // ~3 hours
            onset_delay: 1200.0, // 20 min
        }
    }
}

// ── Active Drug ────────────────────────────────────────────────────

/// A drug currently active in the system with pharmacokinetic state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveDrug {
    /// Drug pharmacological profile.
    pub profile: DrugProfile,
    /// Current plasma concentration (0.0–1.0 normalized).
    pub plasma_concentration: f32,
    /// Time since administration (seconds).
    pub time_since_admin: f32,
    /// Administered dose (0.0–1.0 normalized).
    pub dose: f32,
}

impl ActiveDrug {
    /// Create a new active drug at time of administration.
    #[must_use]
    pub fn new(profile: DrugProfile, dose: f32) -> Self {
        Self {
            profile,
            plasma_concentration: 0.0,
            time_since_admin: 0.0,
            dose: dose.clamp(0.0, 1.0),
        }
    }

    /// Advance pharmacokinetics by `dt` seconds.
    #[inline]
    pub(crate) fn tick_pk(&mut self, dt: f32) {
        self.time_since_admin += dt;
        if self.time_since_admin < self.profile.onset_delay {
            // Absorption phase: linear ramp to dose
            self.plasma_concentration =
                self.dose * (self.time_since_admin / self.profile.onset_delay);
        } else {
            // Elimination phase: exponential decay from peak
            let elapsed = self.time_since_admin - self.profile.onset_delay;
            let k_elim = core::f32::consts::LN_2 / self.profile.half_life;
            self.plasma_concentration = self.dose * (-k_elim * elapsed).exp();
        }
    }

    /// Whether the drug has been effectively eliminated.
    ///
    /// Only true after the drug has passed its onset delay (absorption phase)
    /// and concentration has fallen below threshold.
    #[inline]
    #[must_use]
    pub fn is_negligible(&self) -> bool {
        self.time_since_admin > self.profile.onset_delay && self.plasma_concentration < 0.001
    }

    /// Compute occupancy at a specific receptor binding.
    #[inline]
    #[must_use]
    pub fn occupancy_at(&self, binding: &ReceptorBinding) -> f32 {
        hill_equation(
            self.plasma_concentration,
            binding.ec50,
            binding.hill_coeff,
            binding.emax,
        )
    }
}

// ── Clearance Rate Snapshot ────────────────────────────────────────

/// Snapshot of drug-free clearance rates for drift prevention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ClearanceRateSnapshot {
    pub serotonin: f32,
    pub dopamine: f32,
    pub norepinephrine: f32,
    pub gaba: f32,
    pub glutamate: f32,
    pub oxytocin: f32,
    pub endorphins: f32,
    pub acetylcholine: f32,
    pub bdnf: f32,
}

impl ClearanceRateSnapshot {
    pub fn capture(profile: &NeurotransmitterProfile) -> Self {
        Self {
            serotonin: profile.serotonin.clearance_rate,
            dopamine: profile.dopamine.clearance_rate,
            norepinephrine: profile.norepinephrine.clearance_rate,
            gaba: profile.gaba.clearance_rate,
            glutamate: profile.glutamate.clearance_rate,
            oxytocin: profile.oxytocin.clearance_rate,
            endorphins: profile.endorphins.clearance_rate,
            acetylcholine: profile.acetylcholine.clearance_rate,
            bdnf: profile.bdnf.clearance_rate,
        }
    }

    pub fn restore(&self, profile: &mut NeurotransmitterProfile) {
        profile.serotonin.clearance_rate = self.serotonin;
        profile.dopamine.clearance_rate = self.dopamine;
        profile.norepinephrine.clearance_rate = self.norepinephrine;
        profile.gaba.clearance_rate = self.gaba;
        profile.glutamate.clearance_rate = self.glutamate;
        profile.oxytocin.clearance_rate = self.oxytocin;
        profile.endorphins.clearance_rate = self.endorphins;
        profile.acetylcholine.clearance_rate = self.acetylcholine;
        profile.bdnf.clearance_rate = self.bdnf;
    }
}

impl Default for ClearanceRateSnapshot {
    fn default() -> Self {
        Self::capture(&NeurotransmitterProfile::default())
    }
}

// ── Pharmacology State ─────────────────────────────────────────────

/// Complete pharmacological state — receptors, active drugs, and coupling logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PharmacologyState {
    /// Receptor availability states.
    pub receptors: ReceptorMap,
    /// Currently active drug compounds.
    pub active_drugs: Vec<ActiveDrug>,
    /// Drug-free clearance rates (for drift prevention).
    clearance_snapshot: ClearanceRateSnapshot,
    /// Cached GABA PAM multiplier from last tick.
    gaba_pam_cache: f32,
}

impl Default for PharmacologyState {
    fn default() -> Self {
        Self {
            receptors: ReceptorMap::default(),
            active_drugs: Vec::new(),
            clearance_snapshot: ClearanceRateSnapshot::default(),
            gaba_pam_cache: 1.0,
        }
    }
}

impl PharmacologyState {
    /// Administer a drug at the given normalized dose (0.0–1.0).
    pub fn administer(&mut self, profile: DrugProfile, dose: f32) {
        tracing::debug!(drug = %profile.name, dose, "drug administered");
        self.active_drugs.push(ActiveDrug::new(profile, dose));
    }

    /// Current GABA PAM multiplier (1.0 = no PAM effect).
    #[inline]
    #[must_use]
    pub fn gaba_pam_multiplier(&self) -> f32 {
        self.gaba_pam_cache
    }

    /// Tick pharmacology: advance PK, compute occupancies, tick receptors,
    /// apply effects to neurotransmitter profile.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn tick(&mut self, dt: f32, nt: &mut NeurotransmitterProfile) -> Result<(), MastishkError> {
        validate_dt(dt)?;

        // Short-circuit when no drugs active
        if self.active_drugs.is_empty() {
            self.gaba_pam_cache = 1.0;
            return Ok(());
        }

        tracing::trace!(active_drugs = self.active_drugs.len(), "pharmacology tick");

        // 1. Tick PK for all drugs
        for drug in &mut self.active_drugs {
            drug.tick_pk(dt);
        }

        // 2. Remove negligible drugs
        self.active_drugs.retain(|d| !d.is_negligible());

        if self.active_drugs.is_empty() {
            // All drugs eliminated — restore original clearance rates
            self.clearance_snapshot.restore(nt);
            self.gaba_pam_cache = 1.0;
            return Ok(());
        }

        // 3. Compute aggregate occupancies per receptor
        let mut occupancies = ReceptorOccupancies::default();
        for drug in &self.active_drugs {
            for binding in &drug.profile.bindings {
                let occ = drug.occupancy_at(binding);
                occupancies.add(binding.receptor, occ);
            }
        }

        // 4. Tick receptor desensitization
        self.receptors.tick_all(&occupancies, dt)?;

        // 5. Restore drug-free clearance rates, then apply drug effects
        self.clearance_snapshot.restore(nt);
        let mut gaba_pam = 1.0_f32;

        for drug in &self.active_drugs {
            for binding in &drug.profile.bindings {
                let occ = drug.occupancy_at(binding);
                let receptor_avail = self.receptors.get(binding.receptor).availability;
                let effective = occ * receptor_avail;

                match binding.mechanism {
                    DrugMechanism::ReuptakeInhibitor => {
                        let transmitter = receptor_to_transmitter(binding.receptor);
                        let rate = get_clearance_rate_mut(nt, transmitter);
                        // Reduce clearance (block reuptake), minimum 30% of original
                        *rate *= (1.0 - effective * 0.7).max(0.3);
                    }
                    DrugMechanism::Agonist => {
                        let transmitter = receptor_to_transmitter(binding.receptor);
                        let baseline = get_baseline_mut(nt, transmitter);
                        *baseline = (*baseline + effective * 0.3).min(1.0);
                    }
                    DrugMechanism::Antagonist => {
                        let transmitter = receptor_to_transmitter(binding.receptor);
                        let baseline = get_baseline_mut(nt, transmitter);
                        *baseline = (*baseline - effective * 0.3).max(0.0);
                    }
                    DrugMechanism::PositiveAllostericModulator => {
                        // PAM: amplify GABA effect (doesn't modify TransmitterState directly)
                        gaba_pam += effective * 1.5; // up to ~2.5× amplification
                    }
                }
            }
        }

        self.gaba_pam_cache = gaba_pam;
        Ok(())
    }
}

// ── Helpers ────────────────────────────────────────────────────────

/// Which transmitter a receptor subtype primarily modulates.
#[derive(Debug, Clone, Copy)]
enum TransmitterTarget {
    Serotonin,
    Dopamine,
    Norepinephrine,
    Gaba,
}

fn receptor_to_transmitter(subtype: ReceptorSubtype) -> TransmitterTarget {
    match subtype {
        ReceptorSubtype::Ht1a | ReceptorSubtype::Ht2a => TransmitterTarget::Serotonin,
        ReceptorSubtype::D1 | ReceptorSubtype::D2 => TransmitterTarget::Dopamine,
        ReceptorSubtype::Alpha1 | ReceptorSubtype::Alpha2 | ReceptorSubtype::Beta => {
            TransmitterTarget::Norepinephrine
        }
        ReceptorSubtype::GabaA | ReceptorSubtype::GabaB => TransmitterTarget::Gaba,
    }
}

fn get_clearance_rate_mut(nt: &mut NeurotransmitterProfile, target: TransmitterTarget) -> &mut f32 {
    match target {
        TransmitterTarget::Serotonin => &mut nt.serotonin.clearance_rate,
        TransmitterTarget::Dopamine => &mut nt.dopamine.clearance_rate,
        TransmitterTarget::Norepinephrine => &mut nt.norepinephrine.clearance_rate,
        TransmitterTarget::Gaba => &mut nt.gaba.clearance_rate,
    }
}

fn get_baseline_mut(nt: &mut NeurotransmitterProfile, target: TransmitterTarget) -> &mut f32 {
    match target {
        TransmitterTarget::Serotonin => &mut nt.serotonin.baseline,
        TransmitterTarget::Dopamine => &mut nt.dopamine.baseline,
        TransmitterTarget::Norepinephrine => &mut nt.norepinephrine.baseline,
        TransmitterTarget::Gaba => &mut nt.gaba.baseline,
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Hill equation ──────────────────────────────────────────────

    #[test]
    fn test_hill_equation_zero_concentration() {
        assert!((hill_equation(0.0, 0.5, 1.0, 1.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hill_equation_at_ec50() {
        // At ec50, effect should be emax/2
        let effect = hill_equation(0.5, 0.5, 1.0, 1.0);
        assert!((effect - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_hill_equation_high_concentration() {
        let effect = hill_equation(10.0, 0.5, 1.0, 1.0);
        assert!(effect > 0.9);
    }

    #[test]
    fn test_hill_equation_steep_curve() {
        // Hill coeff > 1 makes curve steeper
        let shallow = hill_equation(0.3, 0.5, 1.0, 1.0);
        let steep = hill_equation(0.3, 0.5, 3.0, 1.0);
        assert!(steep < shallow); // Below ec50, steeper curve gives lower effect
    }

    // ── ActiveDrug PK ──────────────────────────────────────────────

    #[test]
    fn test_pk_absorption_phase() {
        let mut drug = ActiveDrug::new(DrugProfile::ssri_fluoxetine(), 0.8);
        // At t=0, concentration is 0
        assert!(drug.plasma_concentration < f32::EPSILON);

        // Halfway through onset
        drug.tick_pk(1800.0); // 30 min, onset is 3600s
        assert!((drug.plasma_concentration - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_pk_peak_and_elimination() {
        let mut drug = ActiveDrug::new(DrugProfile::benzodiazepine_alprazolam(), 1.0);
        // Absorb fully
        drug.tick_pk(900.0); // onset_delay = 900s
        assert!((drug.plasma_concentration - 1.0).abs() < 0.01);

        // After one half-life (6 hours), concentration should be ~0.5
        drug.tick_pk(21_600.0);
        assert!((drug.plasma_concentration - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_pk_elimination_to_negligible() {
        let mut drug = ActiveDrug::new(DrugProfile::stimulant_methylphenidate(), 0.5);
        // Skip through many half-lives (3hr half-life, go 30 hours)
        drug.tick_pk(1200.0); // absorb
        drug.tick_pk(108_000.0); // ~10 half-lives
        assert!(drug.is_negligible());
    }

    // ── Preset drugs ───────────────────────────────────────────────

    #[test]
    fn test_preset_constructors() {
        let drugs = [
            DrugProfile::ssri_fluoxetine(),
            DrugProfile::ssri_sertraline(),
            DrugProfile::benzodiazepine_diazepam(),
            DrugProfile::benzodiazepine_alprazolam(),
            DrugProfile::stimulant_amphetamine(),
            DrugProfile::stimulant_methylphenidate(),
        ];
        for drug in &drugs {
            assert!(!drug.name.is_empty());
            assert!(!drug.bindings.is_empty());
            assert!(drug.half_life > 0.0);
            assert!(drug.onset_delay > 0.0);
        }
    }

    // ── PharmacologyState ──────────────────────────────────────────

    #[test]
    fn test_empty_pharmacology_is_noop() {
        let mut pharm = PharmacologyState::default();
        let mut nt = NeurotransmitterProfile::default();
        let original_clearance = nt.serotonin.clearance_rate;
        pharm.tick(1.0, &mut nt).unwrap();
        assert!((nt.serotonin.clearance_rate - original_clearance).abs() < f32::EPSILON);
        assert!((pharm.gaba_pam_multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ssri_reduces_serotonin_clearance() {
        let mut pharm = PharmacologyState::default();
        let mut nt = NeurotransmitterProfile::default();
        let original_clearance = nt.serotonin.clearance_rate;

        pharm.administer(DrugProfile::ssri_fluoxetine(), 0.8);
        // Absorb past onset (3600s) and into active phase
        for _ in 0..7200 {
            pharm.tick(1.0, &mut nt).unwrap();
        }
        assert!(
            nt.serotonin.clearance_rate < original_clearance,
            "clearance={}, original={}",
            nt.serotonin.clearance_rate,
            original_clearance
        );
    }

    #[test]
    fn test_stimulant_raises_dopamine_baseline() {
        let mut pharm = PharmacologyState::default();
        let mut nt = NeurotransmitterProfile::default();
        let original_baseline = nt.dopamine.baseline;

        pharm.administer(DrugProfile::stimulant_amphetamine(), 0.7);
        // Past onset (1800s) and into active phase
        for _ in 0..3600 {
            pharm.tick(1.0, &mut nt).unwrap();
        }
        assert!(
            nt.dopamine.baseline > original_baseline,
            "baseline={}, original={}",
            nt.dopamine.baseline,
            original_baseline
        );
    }

    #[test]
    fn test_benzodiazepine_sets_gaba_pam() {
        let mut pharm = PharmacologyState::default();
        let mut nt = NeurotransmitterProfile::default();

        pharm.administer(DrugProfile::benzodiazepine_diazepam(), 0.8);
        // Past onset (1800s) and into active phase
        for _ in 0..3600 {
            pharm.tick(1.0, &mut nt).unwrap();
        }
        assert!(
            pharm.gaba_pam_multiplier() > 1.0,
            "pam={}",
            pharm.gaba_pam_multiplier()
        );
    }

    #[test]
    fn test_drug_elimination_restores_clearance() {
        let mut pharm = PharmacologyState::default();
        let mut nt = NeurotransmitterProfile::default();
        let original_clearance = nt.serotonin.clearance_rate;

        // Short-acting drug analog
        pharm.administer(
            DrugProfile {
                name: "test_ssri".into(),
                bindings: vec![ReceptorBinding {
                    receptor: ReceptorSubtype::Ht1a,
                    ec50: 0.1,
                    hill_coeff: 1.0,
                    emax: 0.9,
                    mechanism: DrugMechanism::ReuptakeInhibitor,
                }],
                half_life: 100.0, // very short
                onset_delay: 10.0,
            },
            0.5,
        );

        // Let it absorb and act
        for _ in 0..100 {
            pharm.tick(1.0, &mut nt).unwrap();
        }
        assert!(nt.serotonin.clearance_rate < original_clearance);

        // Let it fully eliminate (~10 half-lives)
        for _ in 0..2000 {
            pharm.tick(1.0, &mut nt).unwrap();
        }
        // Clearance should be restored
        assert!(
            (nt.serotonin.clearance_rate - original_clearance).abs() < 0.001,
            "clearance={}, expected={}",
            nt.serotonin.clearance_rate,
            original_clearance
        );
    }

    #[test]
    fn test_negative_dt_rejected() {
        let mut pharm = PharmacologyState::default();
        let mut nt = NeurotransmitterProfile::default();
        assert!(pharm.tick(-1.0, &mut nt).is_err());
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut pharm = PharmacologyState::default();
        pharm.administer(DrugProfile::ssri_fluoxetine(), 0.5);
        let json = serde_json::to_string(&pharm).unwrap();
        let pharm2: PharmacologyState = serde_json::from_str(&json).unwrap();
        assert_eq!(pharm2.active_drugs.len(), 1);
        assert!(
            (pharm2.receptors.ht1a.availability - pharm.receptors.ht1a.availability).abs()
                < f32::EPSILON
        );
    }
}
