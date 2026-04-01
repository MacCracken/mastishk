//! Receptor dynamics — subtypes, availability, desensitization, upregulation.
//!
//! Models receptor availability changes over time in response to agonist/antagonist
//! occupancy. Chronic agonist exposure desensitizes receptors (reduces availability);
//! chronic antagonist exposure or agonist withdrawal causes upregulation (increased
//! availability, potentially above baseline — rebound).

use serde::{Deserialize, Serialize};

use crate::error::{MastishkError, validate_dt};

/// Neurotransmitter reuptake transporter identifiers.
///
/// Transporters are the primary target of reuptake inhibitor drugs (SSRIs, SNRIs,
/// stimulants). Distinct from receptors — blocking a transporter increases synaptic
/// concentration of the corresponding neurotransmitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TransporterType {
    /// Serotonin transporter — target of SSRIs (fluoxetine, sertraline).
    Sert,
    /// Dopamine transporter — target of stimulants (methylphenidate), cocaine.
    Dat,
    /// Norepinephrine transporter — target of SNRIs, stimulants (amphetamine).
    Net,
}

/// Receptor subtype identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ReceptorSubtype {
    /// 5-HT1A — serotonin autoreceptor (raphe) + postsynaptic (hippocampus, PFC).
    Ht1a,
    /// 5-HT2A — serotonin postsynaptic (cortex, amygdala). Pro-excitatory, salience.
    Ht2a,
    /// Dopamine D1 — Go pathway. Motivational drive, reward-seeking.
    D1,
    /// Dopamine D2 — No-Go pathway. Impulse brake, antipsychotic target.
    D2,
    /// Adrenergic alpha-1 — arousal, attention, vasoconstriction.
    Alpha1,
    /// Adrenergic alpha-2 — autoreceptor, inhibitory feedback on NE.
    Alpha2,
    /// Adrenergic beta — arousal, fight-or-flight, cardiac.
    Beta,
    /// GABA-A — fast inhibition (ionotropic Cl⁻ channel). Benzodiazepine target.
    GabaA,
    /// GABA-B — slow inhibition (metabotropic). Muscle relaxation, presynaptic.
    GabaB,
    /// CB1 — cannabinoid receptor 1. Retrograde signaling, stress buffer, pain modulation.
    Cb1,
}

/// A single receptor's dynamic state — tracks availability and adaptation.
///
/// Availability starts at `baseline` (typically 1.0) and changes in response to
/// chronic agonist exposure (desensitization, availability drops) or chronic
/// antagonist/withdrawal (upregulation, availability rises, may overshoot).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorState {
    /// Current receptor availability (0.0 = fully desensitized, 1.0 = normal, >1.0 = upregulated).
    pub availability: f32,
    /// Baseline availability the system trends toward.
    pub baseline: f32,
    /// Exponential moving average of recent occupancy (tracks chronic exposure).
    pub occupancy_ema: f32,
    /// Receptor turnover time constant (seconds). Controls return-to-baseline speed.
    pub tau_turnover: f32,
    /// Desensitization rate constant (agonist-driven internalization).
    pub k_des: f32,
    /// Upregulation rate constant (antagonism/withdrawal-driven synthesis).
    pub k_up: f32,
}

impl ReceptorState {
    /// Create a new receptor state at baseline availability.
    #[must_use]
    pub fn new(baseline: f32, tau_turnover: f32, k_des: f32, k_up: f32) -> Self {
        Self {
            availability: baseline,
            baseline,
            occupancy_ema: 0.0,
            tau_turnover,
            k_des,
            k_up,
        }
    }

    /// Tick the receptor state given current occupancy.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn tick(&mut self, occupancy: f32, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        self.tick_unchecked(occupancy, dt);
        Ok(())
    }

    /// Tick without validating dt. Updates occupancy EMA then applies the
    /// desensitization ODE: `d(avail)/dt = (baseline - avail) / tau_turnover
    /// - k_des * occupancy_ema + k_up * (1 - occupancy_ema) * max(0, baseline - avail)`.
    #[inline]
    pub(crate) fn tick_unchecked(&mut self, occupancy: f32, dt: f32) {
        // Update occupancy EMA (time constant ~300s ≈ 5 minutes)
        let ema_alpha = 1.0 - (-dt / 300.0).exp();
        self.occupancy_ema += (occupancy - self.occupancy_ema) * ema_alpha;

        // Desensitization ODE
        let baseline_return = (self.baseline - self.availability) / self.tau_turnover;
        let desensitization = self.k_des * self.occupancy_ema;
        let upregulation =
            self.k_up * (1.0 - self.occupancy_ema) * (self.baseline - self.availability).max(0.0);

        self.availability += (baseline_return - desensitization + upregulation) * dt;
        self.availability = self.availability.clamp(0.0, 1.5);
    }
}

/// Occupancy values for all receptor subtypes (output of drug interaction computation).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReceptorOccupancies {
    pub ht1a: f32,
    pub ht2a: f32,
    pub d1: f32,
    pub d2: f32,
    pub alpha1: f32,
    pub alpha2: f32,
    pub beta: f32,
    pub gaba_a: f32,
    pub gaba_b: f32,
    pub cb1: f32,
}

impl ReceptorOccupancies {
    /// Get occupancy for a specific subtype.
    #[inline]
    #[must_use]
    pub fn get(&self, subtype: ReceptorSubtype) -> f32 {
        match subtype {
            ReceptorSubtype::Ht1a => self.ht1a,
            ReceptorSubtype::Ht2a => self.ht2a,
            ReceptorSubtype::D1 => self.d1,
            ReceptorSubtype::D2 => self.d2,
            ReceptorSubtype::Alpha1 => self.alpha1,
            ReceptorSubtype::Alpha2 => self.alpha2,
            ReceptorSubtype::Beta => self.beta,
            ReceptorSubtype::GabaA => self.gaba_a,
            ReceptorSubtype::GabaB => self.gaba_b,
            ReceptorSubtype::Cb1 => self.cb1,
        }
    }

    /// Set occupancy for a specific subtype, clamping to 0.0..=1.0.
    #[inline]
    pub fn add(&mut self, subtype: ReceptorSubtype, value: f32) {
        let field = match subtype {
            ReceptorSubtype::Ht1a => &mut self.ht1a,
            ReceptorSubtype::Ht2a => &mut self.ht2a,
            ReceptorSubtype::D1 => &mut self.d1,
            ReceptorSubtype::D2 => &mut self.d2,
            ReceptorSubtype::Alpha1 => &mut self.alpha1,
            ReceptorSubtype::Alpha2 => &mut self.alpha2,
            ReceptorSubtype::Beta => &mut self.beta,
            ReceptorSubtype::GabaA => &mut self.gaba_a,
            ReceptorSubtype::GabaB => &mut self.gaba_b,
            ReceptorSubtype::Cb1 => &mut self.cb1,
        };
        *field = (*field + value).min(1.0);
    }
}

/// Complete set of receptor states for a brain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorMap {
    /// 5-HT1A — serotonin autoreceptor. Slow turnover (~7 days).
    pub ht1a: ReceptorState,
    /// 5-HT2A — serotonin postsynaptic. Moderate turnover (~4 days).
    pub ht2a: ReceptorState,
    /// Dopamine D1. Moderate turnover (~5 days).
    pub d1: ReceptorState,
    /// Dopamine D2. Slow turnover (~14 days, antipsychotic upregulation risk).
    pub d2: ReceptorState,
    /// Adrenergic alpha-1.
    pub alpha1: ReceptorState,
    /// Adrenergic alpha-2 (autoreceptor). Fast turnover (~2 days).
    pub alpha2: ReceptorState,
    /// Adrenergic beta.
    pub beta: ReceptorState,
    /// GABA-A. Fast turnover (~3 days, rapid BZD tolerance).
    pub gaba_a: ReceptorState,
    /// GABA-B. Moderate turnover (~5 days).
    pub gaba_b: ReceptorState,
    /// CB1 — cannabinoid receptor 1. Moderate turnover (~5 days).
    #[serde(default = "default_cb1")]
    pub cb1: ReceptorState,
}

fn default_cb1() -> ReceptorState {
    ReceptorState::new(1.0, 432_000.0, 0.000_002, 0.000_001)
}

impl Default for ReceptorMap {
    fn default() -> Self {
        Self {
            //                              baseline  tau_turnover    k_des     k_up
            ht1a: ReceptorState::new(1.0, 604_800.0, 0.000_002, 0.000_001),
            ht2a: ReceptorState::new(1.0, 345_600.0, 0.000_003, 0.000_001_5),
            d1: ReceptorState::new(1.0, 432_000.0, 0.000_002_5, 0.000_001),
            d2: ReceptorState::new(1.0, 1_209_600.0, 0.000_001_5, 0.000_000_8),
            alpha1: ReceptorState::new(1.0, 345_600.0, 0.000_002, 0.000_001),
            alpha2: ReceptorState::new(1.0, 172_800.0, 0.000_003, 0.000_002),
            beta: ReceptorState::new(1.0, 345_600.0, 0.000_002, 0.000_001),
            gaba_a: ReceptorState::new(1.0, 259_200.0, 0.000_004, 0.000_002),
            gaba_b: ReceptorState::new(1.0, 432_000.0, 0.000_002, 0.000_001),
            cb1: default_cb1(),
        }
    }
}

impl ReceptorMap {
    /// Get a receptor state by subtype.
    #[inline]
    #[must_use]
    pub fn get(&self, subtype: ReceptorSubtype) -> &ReceptorState {
        match subtype {
            ReceptorSubtype::Ht1a => &self.ht1a,
            ReceptorSubtype::Ht2a => &self.ht2a,
            ReceptorSubtype::D1 => &self.d1,
            ReceptorSubtype::D2 => &self.d2,
            ReceptorSubtype::Alpha1 => &self.alpha1,
            ReceptorSubtype::Alpha2 => &self.alpha2,
            ReceptorSubtype::Beta => &self.beta,
            ReceptorSubtype::GabaA => &self.gaba_a,
            ReceptorSubtype::GabaB => &self.gaba_b,
            ReceptorSubtype::Cb1 => &self.cb1,
        }
    }

    /// Get a mutable receptor state by subtype.
    #[inline]
    pub fn get_mut(&mut self, subtype: ReceptorSubtype) -> &mut ReceptorState {
        match subtype {
            ReceptorSubtype::Ht1a => &mut self.ht1a,
            ReceptorSubtype::Ht2a => &mut self.ht2a,
            ReceptorSubtype::D1 => &mut self.d1,
            ReceptorSubtype::D2 => &mut self.d2,
            ReceptorSubtype::Alpha1 => &mut self.alpha1,
            ReceptorSubtype::Alpha2 => &mut self.alpha2,
            ReceptorSubtype::Beta => &mut self.beta,
            ReceptorSubtype::GabaA => &mut self.gaba_a,
            ReceptorSubtype::GabaB => &mut self.gaba_b,
            ReceptorSubtype::Cb1 => &mut self.cb1,
        }
    }

    /// Tick all receptors with the given occupancies.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn tick_all(
        &mut self,
        occupancies: &ReceptorOccupancies,
        dt: f32,
    ) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        self.ht1a.tick_unchecked(occupancies.ht1a, dt);
        self.ht2a.tick_unchecked(occupancies.ht2a, dt);
        self.d1.tick_unchecked(occupancies.d1, dt);
        self.d2.tick_unchecked(occupancies.d2, dt);
        self.alpha1.tick_unchecked(occupancies.alpha1, dt);
        self.alpha2.tick_unchecked(occupancies.alpha2, dt);
        self.beta.tick_unchecked(occupancies.beta, dt);
        self.gaba_a.tick_unchecked(occupancies.gaba_a, dt);
        self.gaba_b.tick_unchecked(occupancies.gaba_b, dt);
        self.cb1.tick_unchecked(occupancies.cb1, dt);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receptor_state_at_baseline() {
        let r = ReceptorState::new(1.0, 604_800.0, 0.000_002, 0.000_001);
        assert!((r.availability - 1.0).abs() < f32::EPSILON);
        assert!((r.occupancy_ema - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_desensitization_under_chronic_agonist() {
        let mut r = ReceptorState::new(1.0, 259_200.0, 0.000_01, 0.000_001);
        // Chronic high occupancy for simulated days
        for _ in 0..86400 {
            // 1 day in 1-second steps
            r.tick_unchecked(0.8, 1.0);
        }
        // Availability should have decreased
        assert!(r.availability < 1.0, "avail={}", r.availability);
    }

    #[test]
    fn test_upregulation_on_withdrawal() {
        let mut r = ReceptorState::new(1.0, 259_200.0, 0.000_01, 0.000_005);
        // First desensitize
        for _ in 0..86400 {
            r.tick_unchecked(0.9, 1.0);
        }
        let desensitized = r.availability;
        assert!(desensitized < 1.0);

        // Then withdraw (zero occupancy)
        for _ in 0..172800 {
            // 2 days recovery
            r.tick_unchecked(0.0, 1.0);
        }
        // Should have recovered toward baseline
        assert!(r.availability > desensitized);
    }

    #[test]
    fn test_availability_clamped() {
        let mut r = ReceptorState::new(1.0, 100.0, 0.0, 0.1);
        // High upregulation with no occupancy, fast turnover
        for _ in 0..1000 {
            r.tick_unchecked(0.0, 1.0);
        }
        assert!(r.availability <= 1.5);
    }

    #[test]
    fn test_negative_dt_rejected() {
        let mut r = ReceptorState::new(1.0, 604_800.0, 0.000_002, 0.000_001);
        assert!(r.tick(0.5, -1.0).is_err());
    }

    #[test]
    fn test_receptor_map_default_all_available() {
        let map = ReceptorMap::default();
        assert!((map.ht1a.availability - 1.0).abs() < f32::EPSILON);
        assert!((map.d2.availability - 1.0).abs() < f32::EPSILON);
        assert!((map.gaba_a.availability - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_receptor_map_get() {
        let map = ReceptorMap::default();
        assert!((map.get(ReceptorSubtype::Ht1a).availability - 1.0).abs() < f32::EPSILON);
        assert!((map.get(ReceptorSubtype::GabaA).tau_turnover - 259_200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_receptor_map_tick_all() {
        let mut map = ReceptorMap::default();
        let occ = ReceptorOccupancies {
            gaba_a: 0.8,
            ..Default::default()
        };
        // Tick for simulated days
        for _ in 0..86400 {
            map.tick_all(&occ, 1.0).unwrap();
        }
        // GABA-A should desensitize under chronic occupancy
        assert!(map.gaba_a.availability < 1.0);
        // Others should be near baseline (zero occupancy)
        assert!((map.ht1a.availability - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_occupancies_add_clamps() {
        let mut occ = ReceptorOccupancies::default();
        occ.add(ReceptorSubtype::GabaA, 0.6);
        occ.add(ReceptorSubtype::GabaA, 0.6); // should clamp to 1.0
        assert!((occ.gaba_a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_upregulation_does_not_apply_when_above_baseline() {
        // When availability > baseline, upregulation term should be zero
        // (the .max(0.0) in the ODE ensures this)
        let mut r = ReceptorState::new(1.0, 259_200.0, 0.0, 0.01);
        r.availability = 1.3; // artificially above baseline
        r.tick_unchecked(0.0, 86400.0); // 1 day, no occupancy
        // Should trend back toward baseline, not keep rising
        assert!(r.availability < 1.3);
        assert!(r.availability >= 1.0); // should approach but not undershoot baseline
    }

    #[test]
    fn test_zero_occupancy_maintains_baseline() {
        let mut r = ReceptorState::new(1.0, 604_800.0, 0.000_002, 0.000_001);
        for _ in 0..1000 {
            r.tick_unchecked(0.0, 1.0);
        }
        // With no occupancy, availability should stay near baseline
        assert!((r.availability - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_serde_roundtrip() {
        let map = ReceptorMap::default();
        let json = serde_json::to_string(&map).unwrap();
        let map2: ReceptorMap = serde_json::from_str(&json).unwrap();
        assert!((map2.ht1a.availability - map.ht1a.availability).abs() < f32::EPSILON);
    }
}
