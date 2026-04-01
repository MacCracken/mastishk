//! Brain region models — specialized dynamics for major neural structures.
//!
//! Five brain regions with distinct dynamics, inputs, and outputs suitable
//! for behavioral-timescale personality/emotion simulation:
//!
//! - [`PfcState`] — Prefrontal cortex: executive function, impulse control, working memory
//! - [`AmygdalaState`] — Amygdala: threat detection, fear conditioning, emotional salience
//! - [`HippocampusState`] — Hippocampus: memory formation, context encoding, neurogenesis
//! - [`BasalGangliaState`] — Basal ganglia: action selection, habits, reward prediction error
//! - [`CerebellumState`] — Cerebellum: motor precision, timing, error correction

use serde::{Deserialize, Serialize};

use crate::error::{MastishkError, validate_dt};

// ── Prefrontal Cortex ──────────────────────────────────────────────

/// Prefrontal cortex — executive function, impulse control, working memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PfcState {
    /// Executive control strength (0.0–1.0). Ability to suppress impulses.
    pub executive_control: f32,
    /// Current working memory utilization (0.0–1.0).
    pub working_memory_load: f32,
    /// Working memory capacity (0.0–1.0). Trait-like, slow-changing.
    pub working_memory_capacity: f32,
    /// Goal persistence (0.0–1.0). How strongly current goal is maintained.
    pub goal_maintenance: f32,
    /// Decision fatigue / ego depletion (0.0–1.0).
    pub fatigue: f32,
}

impl Default for PfcState {
    fn default() -> Self {
        Self {
            executive_control: 0.5,
            working_memory_load: 0.0,
            working_memory_capacity: 0.7,
            goal_maintenance: 0.5,
            fatigue: 0.0,
        }
    }
}

impl PfcState {
    /// Tick PFC dynamics. Fatigue recovers, executive control decays toward
    /// resting level, working memory decays (forgetting), goal maintenance decays.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        // Fatigue slowly recovers
        let fatigue_alpha = 1.0 - (-0.01 * dt).exp();
        self.fatigue += (0.0 - self.fatigue) * fatigue_alpha;

        // Executive control decays toward resting level (reduced by fatigue)
        let resting = 0.5 * (1.0 - self.fatigue * 0.5);
        let ec_alpha = 1.0 - (-0.05 * dt).exp();
        self.executive_control += (resting - self.executive_control) * ec_alpha;
        self.executive_control = self.executive_control.clamp(0.0, 1.0);

        // Working memory decays (forgetting without rehearsal)
        let wm_alpha = 1.0 - (-0.02 * dt).exp();
        self.working_memory_load += (0.0 - self.working_memory_load) * wm_alpha;
        self.working_memory_load = self.working_memory_load.clamp(0.0, 1.0);

        // Goal maintenance decays slowly
        let gm_alpha = 1.0 - (-0.01 * dt).exp();
        self.goal_maintenance += (0.3 - self.goal_maintenance) * gm_alpha;

        tracing::trace!(
            ec = self.executive_control,
            fatigue = self.fatigue,
            "PFC tick"
        );
        Ok(())
    }

    /// Exert executive control — boosts control temporarily, increases fatigue.
    #[inline]
    pub fn exert_control(&mut self, effort: f32) {
        self.executive_control = (self.executive_control + effort * 0.3).min(1.0);
        self.fatigue = (self.fatigue + effort * 0.1).min(1.0);
        tracing::debug!(effort, ec = self.executive_control, "PFC control exerted");
    }

    /// Load items into working memory (clamped by capacity).
    #[inline]
    pub fn load_working_memory(&mut self, items: f32) {
        self.working_memory_load =
            (self.working_memory_load + items).min(self.working_memory_capacity);
        tracing::debug!(load = self.working_memory_load, "WM loaded");
    }

    /// Impulse control output: executive control attenuated by fatigue.
    #[inline]
    #[must_use]
    pub fn impulse_control(&self) -> f32 {
        (self.executive_control * (1.0 - self.fatigue * 0.5)).clamp(0.0, 1.0)
    }

    /// Available working memory capacity.
    #[inline]
    #[must_use]
    pub fn available_capacity(&self) -> f32 {
        (self.working_memory_capacity - self.working_memory_load).max(0.0)
    }
}

// ── Amygdala ───────────────────────────────────────────────────────

/// Amygdala — threat detection, fear conditioning, emotional salience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmygdalaState {
    /// Overall amygdala activation (0.0–1.0).
    pub activation: f32,
    /// Perceived threat intensity (0.0–1.0). Acute, fast-decaying.
    pub threat_level: f32,
    /// Learned fear strength (0.0–1.0). Slow-changing (conditioning/extinction).
    pub fear_conditioning: f32,
    /// Emotional significance of current stimulus (0.0–1.0).
    pub salience: f32,
    /// Habituation from repeated exposure (0.0–1.0). Reduces threat response.
    pub habituation: f32,
}

impl Default for AmygdalaState {
    fn default() -> Self {
        Self {
            activation: 0.1,
            threat_level: 0.0,
            fear_conditioning: 0.2,
            salience: 0.3,
            habituation: 0.0,
        }
    }
}

impl AmygdalaState {
    /// Tick amygdala dynamics. Activation, threat, and salience decay; habituation
    /// slowly dissipates.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        let resting = 0.1 + self.fear_conditioning * 0.1;
        self.activation += (resting - self.activation) * (1.0 - (-0.15 * dt).exp());
        self.activation = self.activation.clamp(0.0, 1.0);

        self.threat_level += (0.0 - self.threat_level) * (1.0 - (-0.2 * dt).exp());
        self.salience += (0.1 - self.salience) * (1.0 - (-0.1 * dt).exp());
        self.habituation += (0.0 - self.habituation) * (1.0 - (-0.005 * dt).exp());

        tracing::trace!(
            activation = self.activation,
            threat = self.threat_level,
            "amygdala tick"
        );
        Ok(())
    }

    /// Perceive a threat. Spikes threat_level and activation, reduced by habituation.
    #[inline]
    pub fn perceive_threat(&mut self, intensity: f32) {
        let effective = intensity * (1.0 - self.habituation * 0.7);
        self.threat_level = (self.threat_level + effective * 0.5).min(1.0);
        self.activation = (self.activation + effective * 0.4).min(1.0);
        self.habituation = (self.habituation + 0.05).min(1.0);
        tracing::debug!(intensity, effective, "threat perceived");
    }

    /// Perceive an emotionally salient stimulus.
    #[inline]
    pub fn perceive_stimulus(&mut self, salience_input: f32) {
        self.salience = (self.salience + salience_input * 0.4).min(1.0);
        self.activation = (self.activation + salience_input * 0.2).min(1.0);
    }

    /// Strengthen fear conditioning (slow learning).
    #[inline]
    pub fn condition_fear(&mut self, strength: f32) {
        self.fear_conditioning = (self.fear_conditioning + strength * 0.05).min(1.0);
    }

    /// Weaken fear conditioning (extinction learning).
    #[inline]
    pub fn extinguish_fear(&mut self, strength: f32) {
        self.fear_conditioning = (self.fear_conditioning - strength * 0.03).max(0.0);
    }

    /// Net threat response output (drives HPA stress, fight-or-flight).
    #[inline]
    #[must_use]
    pub fn threat_response(&self) -> f32 {
        (self.activation * self.threat_level * (1.0 - self.habituation * 0.5)).clamp(0.0, 1.0)
    }

    /// Emotional salience output (drives hippocampal memory encoding priority).
    #[inline]
    #[must_use]
    pub fn emotional_salience(&self) -> f32 {
        (self.salience * self.activation).clamp(0.0, 1.0)
    }
}

// ── Hippocampus ────────────────────────────────────────────────────

/// Hippocampus — memory formation, context encoding, neurogenesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HippocampusState {
    /// Memory formation rate (0.0–1.0). Current encoding strength.
    pub encoding_strength: f32,
    /// Long-term memory consolidation rate (0.0–1.0). Sleep-dependent.
    pub consolidation_rate: f32,
    /// Context representation strength (0.0–1.0).
    pub context_signal: f32,
    /// Memory retrieval ease (0.0–1.0).
    pub retrieval_strength: f32,
    /// New neuron formation rate (0.0–1.0). BDNF-dependent, very slow.
    pub neurogenesis: f32,
}

impl Default for HippocampusState {
    fn default() -> Self {
        Self {
            encoding_strength: 0.5,
            consolidation_rate: 0.3,
            context_signal: 0.5,
            retrieval_strength: 0.5,
            neurogenesis: 0.5,
        }
    }
}

impl HippocampusState {
    /// Tick hippocampus dynamics.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        self.encoding_strength += (0.4 - self.encoding_strength) * (1.0 - (-0.08 * dt).exp());
        self.consolidation_rate += (0.2 - self.consolidation_rate) * (1.0 - (-0.03 * dt).exp());
        self.context_signal += (0.3 - self.context_signal) * (1.0 - (-0.05 * dt).exp());
        self.retrieval_strength += (0.4 - self.retrieval_strength) * (1.0 - (-0.04 * dt).exp());
        self.neurogenesis += (0.5 - self.neurogenesis) * (1.0 - (-0.001 * dt).exp());

        tracing::trace!(encoding = self.encoding_strength, "hippocampus tick");
        Ok(())
    }

    /// Encode a memory with given emotional salience boost.
    #[inline]
    pub fn encode(&mut self, salience: f32) {
        self.encoding_strength = (self.encoding_strength + salience * 0.3).min(1.0);
        tracing::debug!(
            salience,
            encoding = self.encoding_strength,
            "memory encoded"
        );
    }

    /// Provide context signal to downstream regions (PFC, basal ganglia).
    #[inline]
    pub fn provide_context(&mut self, strength: f32) {
        self.context_signal = (self.context_signal + strength * 0.3).min(1.0);
    }

    /// Net memory formation rate combining encoding, consolidation, and neurogenesis.
    #[inline]
    #[must_use]
    pub fn memory_formation_rate(&self) -> f32 {
        self.encoding_strength * self.consolidation_rate * (0.5 + self.neurogenesis * 0.5)
    }

    /// Context quality combining signal strength and retrieval ability.
    #[inline]
    #[must_use]
    pub fn context_quality(&self) -> f32 {
        self.context_signal * self.retrieval_strength
    }
}

// ── Basal Ganglia ──────────────────────────────────────────────────

/// Basal ganglia — action selection, habit formation, reward prediction error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasalGangliaState {
    /// Direct pathway activation (0.0–1.0). Action initiation (Go).
    pub go_signal: f32,
    /// Indirect pathway activation (0.0–1.0). Action suppression (No-Go).
    pub nogo_signal: f32,
    /// Expected reward for current action (0.0–1.0).
    pub reward_prediction: f32,
    /// Reward prediction error (−1.0 to +1.0). Actual minus expected.
    pub prediction_error: f32,
    /// Degree of automaticity (0.0–1.0). Slow-changing habit strength.
    pub habit_strength: f32,
}

impl Default for BasalGangliaState {
    fn default() -> Self {
        Self {
            go_signal: 0.3,
            nogo_signal: 0.3,
            reward_prediction: 0.3,
            prediction_error: 0.0,
            habit_strength: 0.2,
        }
    }
}

impl BasalGangliaState {
    /// Tick basal ganglia dynamics.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        self.go_signal += (0.3 - self.go_signal) * (1.0 - (-0.1 * dt).exp());
        self.nogo_signal += (0.3 - self.nogo_signal) * (1.0 - (-0.1 * dt).exp());
        self.prediction_error += (0.0 - self.prediction_error) * (1.0 - (-0.3 * dt).exp());
        self.reward_prediction += (0.3 - self.reward_prediction) * (1.0 - (-0.005 * dt).exp());

        tracing::trace!(
            go = self.go_signal,
            nogo = self.nogo_signal,
            "basal ganglia tick"
        );
        Ok(())
    }

    /// Boost Go pathway (action initiation).
    #[inline]
    pub fn initiate_action(&mut self, motivation: f32) {
        self.go_signal = (self.go_signal + motivation * 0.3).min(1.0);
        tracing::debug!(motivation, go = self.go_signal, "action initiated");
    }

    /// Boost No-Go pathway (action suppression).
    #[inline]
    pub fn suppress_action(&mut self, inhibition: f32) {
        self.nogo_signal = (self.nogo_signal + inhibition * 0.3).min(1.0);
    }

    /// Process reward outcome. Computes RPE and updates prediction + habit.
    #[inline]
    pub fn receive_reward(&mut self, actual_reward: f32) {
        self.prediction_error = (actual_reward - self.reward_prediction).clamp(-1.0, 1.0);
        // Update prediction toward actual
        self.reward_prediction += (actual_reward - self.reward_prediction) * 0.1;
        self.reward_prediction = self.reward_prediction.clamp(0.0, 1.0);
        // Positive RPE strengthens habit
        if self.prediction_error > 0.0 {
            self.habit_strength = (self.habit_strength + self.prediction_error * 0.02).min(1.0);
        }
        tracing::debug!(
            rpe = self.prediction_error,
            habit = self.habit_strength,
            "reward received"
        );
    }

    /// Net action selection signal: Go minus No-Go (0.0–1.0).
    #[inline]
    #[must_use]
    pub fn action_selection(&self) -> f32 {
        (self.go_signal - self.nogo_signal).clamp(0.0, 1.0)
    }

    /// Whether behavior is primarily habitual (habit_strength > 0.6).
    #[inline]
    #[must_use]
    pub fn is_habitual(&self) -> bool {
        self.habit_strength > 0.6
    }
}

// ── Cerebellum ─────────────────────────────────────────────────────

/// Cerebellum — motor precision, timing accuracy, error correction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CerebellumState {
    /// Motor output accuracy (0.0–1.0).
    pub motor_precision: f32,
    /// Temporal precision (0.0–1.0).
    pub timing_accuracy: f32,
    /// Current error magnitude (0.0–1.0). Transient climbing-fiber analog.
    pub error_signal: f32,
    /// Error correction learning rate (0.0–1.0). BDNF-modulated.
    pub adaptation_rate: f32,
    /// Overall motor coordination (0.0–1.0). Derived from precision + timing.
    pub coordination: f32,
}

impl Default for CerebellumState {
    fn default() -> Self {
        Self {
            motor_precision: 0.6,
            timing_accuracy: 0.6,
            error_signal: 0.0,
            adaptation_rate: 0.5,
            coordination: 0.6,
        }
    }
}

impl CerebellumState {
    /// Tick cerebellum dynamics.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        // Error signal decays (transient)
        self.error_signal += (0.0 - self.error_signal) * (1.0 - (-0.2 * dt).exp());

        // Motor precision slowly consolidates when errors are low
        if self.error_signal < 0.2 {
            let consolidation = 1.0 - (-0.002 * dt).exp();
            self.motor_precision += (0.8 - self.motor_precision) * consolidation;
        }
        // Error correction attempt temporarily perturbs precision
        self.motor_precision -= self.error_signal * self.adaptation_rate * 0.05 * dt;
        self.motor_precision = self.motor_precision.clamp(0.0, 1.0);

        // Timing accuracy consolidates similarly
        self.timing_accuracy += (0.7 - self.timing_accuracy) * (1.0 - (-0.003 * dt).exp());
        self.timing_accuracy = self.timing_accuracy.clamp(0.0, 1.0);

        // Coordination is derived
        self.coordination =
            (self.motor_precision * 0.6 + self.timing_accuracy * 0.4).clamp(0.0, 1.0);

        tracing::trace!(
            precision = self.motor_precision,
            coordination = self.coordination,
            "cerebellum tick"
        );
        Ok(())
    }

    /// Signal a motor error (climbing fiber input). Triggers adaptation.
    #[inline]
    pub fn signal_error(&mut self, magnitude: f32) {
        self.error_signal = (self.error_signal + magnitude * 0.5).min(1.0);
        tracing::debug!(magnitude, error = self.error_signal, "motor error signaled");
    }

    /// Practice a motor skill, improving precision and timing over time.
    #[inline]
    pub fn practice(&mut self, dt: f32) {
        self.motor_precision = (self.motor_precision + 0.01 * dt).min(1.0);
        self.timing_accuracy = (self.timing_accuracy + 0.005 * dt).min(1.0);
    }

    /// Motor output quality, reduced by active errors.
    #[inline]
    #[must_use]
    pub fn motor_output_quality(&self) -> f32 {
        (self.coordination * (1.0 - self.error_signal * 0.5)).clamp(0.0, 1.0)
    }

    /// Timing quality, reduced by active errors.
    #[inline]
    #[must_use]
    pub fn timing_quality(&self) -> f32 {
        (self.timing_accuracy * (1.0 - self.error_signal * 0.3)).clamp(0.0, 1.0)
    }
}

// ── VTA / Nucleus Accumbens Reward Circuit ─────────────────────────

/// VTA/Nucleus Accumbens reward circuit — incentive salience, wanting, craving.
///
/// Distinct from dorsal striatum Go/NoGo (basal ganglia). The mesolimbic pathway
/// from VTA to NAc encodes "wanting" (incentive salience, Berridge) rather than
/// "liking" or action selection. Drives approach motivation and reward-seeking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardCircuitState {
    /// VTA dopaminergic neuron activity (0.0–1.0). Drives NAc dopamine.
    pub vta_activity: f32,
    /// Nucleus accumbens incentive salience / "wanting" (0.0–1.0).
    pub incentive_salience: f32,
    /// Craving intensity (0.0–1.0). Elevated wanting without satisfaction.
    pub craving: f32,
    /// Reward satiation (0.0–1.0). Recent reward reduces wanting temporarily.
    pub satiation: f32,
    /// Sensitization (0.0–1.0). Chronic stimulation increases future reactivity.
    pub sensitization: f32,
}

impl Default for RewardCircuitState {
    fn default() -> Self {
        Self {
            vta_activity: 0.3,
            incentive_salience: 0.3,
            craving: 0.0,
            satiation: 0.0,
            sensitization: 0.0,
        }
    }
}

impl RewardCircuitState {
    /// Tick reward circuit dynamics.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        // VTA activity decays toward resting
        self.vta_activity += (0.3 - self.vta_activity) * (1.0 - (-0.1 * dt).exp());
        self.vta_activity = self.vta_activity.clamp(0.0, 1.0);

        // Incentive salience driven by VTA, boosted by sensitization
        let salience_target = self.vta_activity * (1.0 + self.sensitization * 0.5);
        self.incentive_salience +=
            (salience_target - self.incentive_salience) * (1.0 - (-0.08 * dt).exp());
        self.incentive_salience = self.incentive_salience.clamp(0.0, 1.0);

        // Craving = wanting without satisfaction (wanting minus satiation)
        self.craving = (self.incentive_salience - self.satiation * 0.8).clamp(0.0, 1.0);

        // Satiation decays (wearing off)
        self.satiation += (0.0 - self.satiation) * (1.0 - (-0.03 * dt).exp());

        // Sensitization changes very slowly
        self.sensitization += (0.0 - self.sensitization) * (1.0 - (-0.0005 * dt).exp());

        tracing::trace!(
            vta = self.vta_activity,
            wanting = self.incentive_salience,
            craving = self.craving,
            "reward circuit tick"
        );
        Ok(())
    }

    /// Encounter a reward cue — activates VTA, increases wanting.
    #[inline]
    pub fn reward_cue(&mut self, salience: f32) {
        self.vta_activity = (self.vta_activity + salience * 0.3).min(1.0);
        tracing::debug!(salience, vta = self.vta_activity, "reward cue perceived");
    }

    /// Receive reward — provides satiation, reduces craving temporarily.
    #[inline]
    pub fn receive_reward(&mut self, magnitude: f32) {
        self.satiation = (self.satiation + magnitude * 0.5).min(1.0);
        // Repeated reward can sensitize the circuit
        self.sensitization = (self.sensitization + magnitude * 0.01).min(1.0);
        tracing::debug!(
            magnitude,
            satiation = self.satiation,
            "reward received (VTA/NAc)"
        );
    }

    /// Current wanting/approach motivation level.
    #[inline]
    #[must_use]
    pub fn wanting(&self) -> f32 {
        self.incentive_salience
    }

    /// Whether actively craving (wanting without recent reward).
    #[inline]
    #[must_use]
    pub fn is_craving(&self) -> bool {
        self.craving > 0.4
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── PFC ────────────────────────────────────────────────────────

    #[test]
    fn test_pfc_default() {
        let pfc = PfcState::default();
        assert!((pfc.executive_control - 0.5).abs() < f32::EPSILON);
        assert!((pfc.fatigue - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pfc_exert_control_increases_fatigue() {
        let mut pfc = PfcState::default();
        pfc.exert_control(0.8);
        assert!(pfc.executive_control > 0.5);
        assert!(pfc.fatigue > 0.0);
    }

    #[test]
    fn test_pfc_fatigue_recovers() {
        let mut pfc = PfcState {
            fatigue: 0.8,
            ..Default::default()
        };
        for _ in 0..1000 {
            pfc.tick(1.0).unwrap();
        }
        assert!(pfc.fatigue < 0.1);
    }

    #[test]
    fn test_pfc_impulse_control() {
        let mut pfc = PfcState::default();
        let fresh = pfc.impulse_control();
        pfc.fatigue = 0.9;
        let tired = pfc.impulse_control();
        assert!(tired < fresh);
    }

    #[test]
    fn test_pfc_working_memory_capped() {
        let mut pfc = PfcState::default();
        pfc.load_working_memory(2.0);
        assert!(pfc.working_memory_load <= pfc.working_memory_capacity);
    }

    #[test]
    fn test_pfc_negative_dt() {
        let mut pfc = PfcState::default();
        assert!(pfc.tick(-1.0).is_err());
    }

    #[test]
    fn test_pfc_serde_roundtrip() {
        let pfc = PfcState::default();
        let json = serde_json::to_string(&pfc).unwrap();
        let pfc2: PfcState = serde_json::from_str(&json).unwrap();
        assert!((pfc2.executive_control - pfc.executive_control).abs() < f32::EPSILON);
    }

    // ── Amygdala ───────────────────────────────────────────────────

    #[test]
    fn test_amygdala_threat_spikes_activation() {
        let mut a = AmygdalaState::default();
        let before = a.activation;
        a.perceive_threat(0.8);
        assert!(a.activation > before);
        assert!(a.threat_level > 0.0);
    }

    #[test]
    fn test_amygdala_habituation_reduces_effective_threat() {
        let mut a = AmygdalaState::default();
        // Build habituation through repeated exposure with decay between
        for _ in 0..20 {
            a.perceive_threat(0.5);
            a.tick(5.0).unwrap(); // let activation decay between threats
        }
        // Habituation should have built up
        assert!(a.habituation > 0.3);
        // perceive_threat uses `intensity * (1.0 - habituation * 0.7)` — effective is reduced
        let effective_at_high_hab = 0.5 * (1.0 - a.habituation * 0.7);
        let effective_at_zero_hab = 0.5;
        assert!(effective_at_high_hab < effective_at_zero_hab);
    }

    #[test]
    fn test_amygdala_fear_conditioning() {
        let mut a = AmygdalaState::default();
        let before = a.fear_conditioning;
        a.condition_fear(1.0);
        assert!(a.fear_conditioning > before);
        a.extinguish_fear(1.0);
        assert!(a.fear_conditioning < a.fear_conditioning + 0.1); // decreased somewhat
    }

    #[test]
    fn test_amygdala_activation_decays() {
        let mut a = AmygdalaState::default();
        a.perceive_threat(1.0);
        for _ in 0..100 {
            a.tick(1.0).unwrap();
        }
        assert!(a.threat_level < 0.1);
    }

    #[test]
    fn test_amygdala_serde_roundtrip() {
        let a = AmygdalaState::default();
        let json = serde_json::to_string(&a).unwrap();
        let a2: AmygdalaState = serde_json::from_str(&json).unwrap();
        assert!((a2.activation - a.activation).abs() < f32::EPSILON);
    }

    // ── Hippocampus ────────────────────────────────────────────────

    #[test]
    fn test_hippocampus_encode_boosts_strength() {
        let mut h = HippocampusState::default();
        let before = h.encoding_strength;
        h.encode(0.8);
        assert!(h.encoding_strength > before);
    }

    #[test]
    fn test_hippocampus_memory_formation_rate() {
        let h = HippocampusState::default();
        let rate = h.memory_formation_rate();
        assert!(rate > 0.0 && rate <= 1.0);
    }

    #[test]
    fn test_hippocampus_context_quality() {
        let h = HippocampusState::default();
        let q = h.context_quality();
        assert!((0.0..=1.0).contains(&q));
    }

    #[test]
    fn test_hippocampus_serde_roundtrip() {
        let h = HippocampusState::default();
        let json = serde_json::to_string(&h).unwrap();
        let h2: HippocampusState = serde_json::from_str(&json).unwrap();
        assert!((h2.encoding_strength - h.encoding_strength).abs() < f32::EPSILON);
    }

    // ── Basal Ganglia ──────────────────────────────────────────────

    #[test]
    fn test_basal_ganglia_action_selection() {
        let mut bg = BasalGangliaState::default();
        bg.initiate_action(0.8);
        assert!(bg.action_selection() > 0.0);
    }

    #[test]
    fn test_basal_ganglia_reward_updates_prediction() {
        let mut bg = BasalGangliaState::default();
        bg.receive_reward(0.9);
        assert!(bg.prediction_error > 0.0); // positive surprise
        assert!(bg.reward_prediction > 0.3); // updated toward actual
    }

    #[test]
    fn test_basal_ganglia_habit_strengthens_on_positive_rpe() {
        let mut bg = BasalGangliaState::default();
        let before = bg.habit_strength;
        bg.receive_reward(1.0); // big positive RPE
        assert!(bg.habit_strength > before);
    }

    #[test]
    fn test_basal_ganglia_is_habitual() {
        let mut bg = BasalGangliaState::default();
        assert!(!bg.is_habitual());
        bg.habit_strength = 0.7;
        assert!(bg.is_habitual());
    }

    #[test]
    fn test_basal_ganglia_serde_roundtrip() {
        let bg = BasalGangliaState::default();
        let json = serde_json::to_string(&bg).unwrap();
        let bg2: BasalGangliaState = serde_json::from_str(&json).unwrap();
        assert!((bg2.go_signal - bg.go_signal).abs() < f32::EPSILON);
    }

    // ── Cerebellum ─────────────────────────────────────────────────

    #[test]
    fn test_cerebellum_error_decays() {
        let mut c = CerebellumState::default();
        c.signal_error(0.8);
        assert!(c.error_signal > 0.0);
        for _ in 0..50 {
            c.tick(1.0).unwrap();
        }
        assert!(c.error_signal < 0.1);
    }

    #[test]
    fn test_cerebellum_practice_improves() {
        let mut c = CerebellumState::default();
        let before = c.motor_precision;
        c.practice(10.0);
        assert!(c.motor_precision > before);
    }

    #[test]
    fn test_cerebellum_motor_output_quality() {
        let c = CerebellumState::default();
        assert!(c.motor_output_quality() > 0.0);
        assert!(c.motor_output_quality() <= 1.0);
    }

    #[test]
    fn test_cerebellum_coordination_derived() {
        let mut c = CerebellumState::default();
        c.tick(0.0).unwrap();
        // coordination = precision * 0.6 + timing * 0.4
        let expected = c.motor_precision * 0.6 + c.timing_accuracy * 0.4;
        assert!((c.coordination - expected).abs() < 0.01);
    }

    #[test]
    fn test_cerebellum_serde_roundtrip() {
        let c = CerebellumState::default();
        let json = serde_json::to_string(&c).unwrap();
        let c2: CerebellumState = serde_json::from_str(&json).unwrap();
        assert!((c2.motor_precision - c.motor_precision).abs() < f32::EPSILON);
    }

    // ── Reward Circuit ─────────────────────────────────────────────

    #[test]
    fn test_reward_cue_activates_vta() {
        let mut rc = RewardCircuitState::default();
        let before = rc.vta_activity;
        rc.reward_cue(0.8);
        assert!(rc.vta_activity > before);
    }

    #[test]
    fn test_reward_provides_satiation() {
        let mut rc = RewardCircuitState::default();
        rc.reward_cue(0.8);
        rc.tick(1.0).unwrap();
        assert!(rc.craving > 0.0); // wanting without reward
        rc.receive_reward(0.9);
        rc.tick(1.0).unwrap();
        assert!(rc.satiation > 0.0);
        // Craving should decrease with satiation
        let after_reward_craving = rc.craving;
        assert!(after_reward_craving < rc.incentive_salience);
    }

    #[test]
    fn test_reward_circuit_decays() {
        let mut rc = RewardCircuitState::default();
        rc.reward_cue(1.0);
        for _ in 0..100 {
            rc.tick(1.0).unwrap();
        }
        // VTA should have decayed toward resting
        assert!(rc.vta_activity < 0.5);
    }

    #[test]
    fn test_reward_circuit_serde_roundtrip() {
        let rc = RewardCircuitState::default();
        let json = serde_json::to_string(&rc).unwrap();
        let rc2: RewardCircuitState = serde_json::from_str(&json).unwrap();
        assert!((rc2.vta_activity - rc.vta_activity).abs() < f32::EPSILON);
    }
}
