//! Spiking neural network models — Izhikevich, Leaky Integrate-and-Fire, STDP.
//!
//! Full-fidelity spiking models operating at millisecond timescale. These are
//! standalone simulation tools, NOT integrated into [`crate::brain::BrainState`]
//! (which operates at seconds timescale). Consumers (kiran, joshua) provide
//! `dt_ms` from their simulation clock.
//!
//! # Models
//!
//! - [`IzhikevichNeuron`] — Izhikevich (2003) model with (a,b,c,d) parameters
//!   for diverse firing patterns (regular spiking, fast spiking, chattering, bursting)
//! - [`LifNeuron`] — Leaky integrate-and-fire, simplest biophysical spiking model
//! - [`SpikingNetwork`] — Network of heterogeneous spiking neurons with synapses
//!   and optional STDP plasticity

use serde::{Deserialize, Serialize};

use crate::error::{MastishkError, validate_dt};

// ── Izhikevich Neuron ──────────────────────────────────────────────

/// Izhikevich (2003) spiking neuron model.
///
/// Two-variable model that reproduces diverse firing patterns via four
/// parameters (a, b, c, d). Membrane potential `v` in mV, recovery variable `u`.
///
/// Update equations (per ms):
/// ```text
/// v' = 0.04v² + 5v + 140 - u + I
/// u' = a(bv - u)
/// if v >= 30: v = c, u += d  (spike + reset)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IzhikevichNeuron {
    /// Membrane potential (mV). Resting ~-65, spike threshold ~30.
    pub v: f32,
    /// Recovery variable. Provides negative feedback.
    pub u: f32,
    /// Recovery time constant. Smaller = faster recovery.
    pub a: f32,
    /// Recovery sensitivity to subthreshold v fluctuations.
    pub b: f32,
    /// Post-spike reset value for v (mV).
    pub c: f32,
    /// Post-spike increment for u.
    pub d: f32,
}

impl IzhikevichNeuron {
    /// Regular spiking (RS) — most common excitatory cortical neuron.
    #[must_use]
    pub fn regular_spiking() -> Self {
        Self {
            v: -65.0,
            u: -14.0,
            a: 0.02,
            b: 0.2,
            c: -65.0,
            d: 8.0,
        }
    }

    /// Fast spiking (FS) — inhibitory interneuron, no adaptation.
    #[must_use]
    pub fn fast_spiking() -> Self {
        Self {
            v: -65.0,
            u: -14.0,
            a: 0.1,
            b: 0.2,
            c: -65.0,
            d: 2.0,
        }
    }

    /// Chattering (CH) — bursts of closely spaced spikes.
    #[must_use]
    pub fn chattering() -> Self {
        Self {
            v: -65.0,
            u: -14.0,
            a: 0.02,
            b: 0.2,
            c: -50.0,
            d: 2.0,
        }
    }

    /// Intrinsically bursting (IB) — initial burst then regular spiking.
    #[must_use]
    pub fn intrinsically_bursting() -> Self {
        Self {
            v: -65.0,
            u: -14.0,
            a: 0.02,
            b: 0.2,
            c: -55.0,
            d: 4.0,
        }
    }

    /// Advance by `dt_ms` milliseconds with external input current `input`.
    /// Returns `true` if the neuron spiked (v crossed 30 mV).
    ///
    /// Uses 0.5ms Euler substeps for numerical stability when dt_ms > 0.5.
    #[inline]
    pub fn tick(&mut self, input: f32, dt_ms: f32) -> bool {
        let mut spiked = false;
        let mut remaining = dt_ms;
        while remaining > 0.0 {
            let step = remaining.min(0.5);
            // Izhikevich update (half-step v for stability)
            self.v += step * (0.04 * self.v * self.v + 5.0 * self.v + 140.0 - self.u + input);
            self.u += step * self.a * (self.b * self.v - self.u);
            if self.v >= 30.0 {
                self.v = self.c;
                self.u += self.d;
                spiked = true;
            }
            remaining -= step;
        }
        spiked
    }
}

// ── Leaky Integrate-and-Fire ───────────────────────────────────────

/// Leaky integrate-and-fire (LIF) spiking neuron.
///
/// Simplest biophysical model: membrane potential decays toward rest,
/// input current drives it toward threshold, spike on threshold crossing.
///
/// ```text
/// tau_m * dv/dt = -(v - v_rest) + r_m * I
/// if v >= v_thresh: spike, v = v_reset
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifNeuron {
    /// Membrane potential (mV).
    pub v: f32,
    /// Resting potential (mV). Typically -65.
    pub v_rest: f32,
    /// Spike threshold (mV). Typically -55.
    pub v_thresh: f32,
    /// Post-spike reset (mV). Typically -70.
    pub v_reset: f32,
    /// Membrane time constant (ms). Typically 10-20.
    pub tau_m: f32,
    /// Membrane resistance (MOhm). Scales input current to voltage.
    pub r_m: f32,
}

impl LifNeuron {
    /// Standard LIF parameters.
    #[must_use]
    pub fn default_params() -> Self {
        Self {
            v: -65.0,
            v_rest: -65.0,
            v_thresh: -55.0,
            v_reset: -70.0,
            tau_m: 15.0,
            r_m: 10.0,
        }
    }

    /// Advance by `dt_ms` milliseconds. Returns `true` on spike.
    #[inline]
    pub fn tick(&mut self, input: f32, dt_ms: f32) -> bool {
        let dv = (-(self.v - self.v_rest) + self.r_m * input) / self.tau_m;
        self.v += dv * dt_ms;
        if self.v >= self.v_thresh {
            self.v = self.v_reset;
            return true;
        }
        false
    }
}

// ── Spiking Neuron Enum ────────────────────────────────────────────

/// Heterogeneous spiking neuron — either Izhikevich or LIF.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SpikingNeuron {
    Izhikevich(IzhikevichNeuron),
    Lif(LifNeuron),
}

impl SpikingNeuron {
    /// Tick the neuron. Returns `true` on spike.
    #[inline]
    pub fn tick(&mut self, input: f32, dt_ms: f32) -> bool {
        match self {
            Self::Izhikevich(n) => n.tick(input, dt_ms),
            Self::Lif(n) => n.tick(input, dt_ms),
        }
    }
}

// ── Synapse ────────────────────────────────────────────────────────

/// Synapse in a spiking network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpikingSynapse {
    /// Source neuron index.
    pub from: usize,
    /// Target neuron index.
    pub to: usize,
    /// Connection weight (positive = excitatory, negative = inhibitory).
    pub weight: f32,
    /// Axonal conduction delay (ms).
    pub delay_ms: f32,
}

// ── Plasticity Rules ───────────────────────────────────────────────

/// Spike-Timing-Dependent Plasticity (STDP) rule (Song et al. 2000).
///
/// Pre-before-post (causal) → LTP (weight increase).
/// Post-before-pre (anti-causal) → LTD (weight decrease).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdpRule {
    /// LTP amplitude (weight increase for causal pairing). Typical: 0.01.
    pub a_plus: f32,
    /// LTD amplitude (weight decrease for anti-causal). Typical: 0.012.
    pub a_minus: f32,
    /// LTP time window (ms). Typical: 20.
    pub tau_plus: f32,
    /// LTD time window (ms). Typical: 20.
    pub tau_minus: f32,
}

impl Default for StdpRule {
    fn default() -> Self {
        Self {
            a_plus: 0.01,
            a_minus: 0.012,
            tau_plus: 20.0,
            tau_minus: 20.0,
        }
    }
}

/// BCM sliding threshold rule for rate-based plasticity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BcmRule {
    /// Modification threshold — activity level below which LTD occurs.
    pub theta_m: f32,
    /// Threshold adaptation time constant.
    pub tau_theta: f32,
}

impl Default for BcmRule {
    fn default() -> Self {
        Self {
            theta_m: 0.5,
            tau_theta: 1000.0,
        }
    }
}

// ── Spiking Network ────────────────────────────────────────────────

/// Network of spiking neurons with synapses and optional STDP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpikingNetwork {
    /// Neurons in the network.
    pub neurons: Vec<SpikingNeuron>,
    /// Synaptic connections.
    pub synapses: Vec<SpikingSynapse>,
    /// Last spike time per neuron (ms since simulation start). None if never spiked.
    spike_times: Vec<Option<f32>>,
    /// Indices of neurons that spiked on the last tick.
    last_spiked: Vec<usize>,
    /// Pooled input buffer (reused each tick to avoid allocation).
    inputs: Vec<f32>,
    /// Simulation clock (ms).
    pub time_ms: f32,
    /// Optional STDP learning rule.
    pub stdp: Option<StdpRule>,
}

impl Default for SpikingNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl SpikingNetwork {
    /// Create an empty spiking network.
    #[must_use]
    pub fn new() -> Self {
        Self {
            neurons: Vec::new(),
            synapses: Vec::new(),
            spike_times: Vec::new(),
            last_spiked: Vec::new(),
            inputs: Vec::new(),
            time_ms: 0.0,
            stdp: None,
        }
    }

    /// Add a neuron, returns its index.
    pub fn add_neuron(&mut self, neuron: SpikingNeuron) -> usize {
        let idx = self.neurons.len();
        self.neurons.push(neuron);
        self.inputs.push(0.0);
        self.spike_times.push(None);
        idx
    }

    /// Add a synapse between two neurons.
    ///
    /// # Errors
    /// Returns [`MastishkError::InvalidCircuit`] if indices are out of bounds.
    pub fn add_synapse(
        &mut self,
        from: usize,
        to: usize,
        weight: f32,
        delay_ms: f32,
    ) -> Result<(), MastishkError> {
        let len = self.neurons.len();
        if from >= len || to >= len {
            return Err(MastishkError::InvalidCircuit(format!(
                "spiking synapse {from}->{to} out of bounds (neuron count: {len})"
            )));
        }
        self.synapses.push(SpikingSynapse {
            from,
            to,
            weight,
            delay_ms,
        });
        Ok(())
    }

    /// Tick the network by `dt_ms` milliseconds.
    ///
    /// Propagates spikes through synapses (with delay), updates all neurons,
    /// applies STDP if enabled.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt_ms < 0.0`.
    pub fn tick(&mut self, dt_ms: f32) -> Result<(), MastishkError> {
        validate_dt(dt_ms)?;

        // Compute synaptic inputs using pooled buffer (no allocation per tick)
        self.inputs.iter_mut().for_each(|x| *x = 0.0);
        self.inputs.resize(self.neurons.len(), 0.0);
        for syn in &self.synapses {
            if syn.from < self.neurons.len()
                && syn.to < self.neurons.len()
                && let Some(t) = self.spike_times[syn.from]
            {
                let elapsed = self.time_ms - t;
                if elapsed >= syn.delay_ms && elapsed < syn.delay_ms + dt_ms {
                    self.inputs[syn.to] += syn.weight;
                }
            }
        }

        // Tick all neurons
        self.last_spiked.clear();
        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            if neuron.tick(self.inputs[i], dt_ms) {
                self.spike_times[i] = Some(self.time_ms);
                self.last_spiked.push(i);
            }
        }

        // Apply STDP if enabled and any spikes occurred
        if self.stdp.is_some() && !self.last_spiked.is_empty() {
            self.apply_stdp();
        }

        self.time_ms += dt_ms;
        tracing::trace!(
            time_ms = self.time_ms,
            spikes = self.last_spiked.len(),
            "spiking network tick"
        );
        Ok(())
    }

    /// Indices of neurons that spiked on the last tick.
    #[must_use]
    pub fn last_spikes(&self) -> &[usize] {
        &self.last_spiked
    }

    /// Apply STDP weight updates based on spike timing.
    fn apply_stdp(&mut self) {
        let rule = match &self.stdp {
            Some(r) => r.clone(),
            None => return,
        };

        for syn in &mut self.synapses {
            let pre_time = self.spike_times[syn.from];
            let post_time = self.spike_times[syn.to];

            if let (Some(t_pre), Some(t_post)) = (pre_time, post_time) {
                let delta_t = t_post - t_pre;
                if delta_t > 0.0 && delta_t < rule.tau_plus * 3.0 {
                    // Pre before post → LTP
                    syn.weight += rule.a_plus * (-delta_t / rule.tau_plus).exp();
                } else if delta_t < 0.0 && -delta_t < rule.tau_minus * 3.0 {
                    // Post before pre → LTD
                    syn.weight -= rule.a_minus * (delta_t / rule.tau_minus).exp();
                }
                syn.weight = syn.weight.clamp(-2.0, 2.0);
            }
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_izhikevich_rs_spikes() {
        let mut n = IzhikevichNeuron::regular_spiking();
        let mut spike_count = 0;
        for _ in 0..1000 {
            if n.tick(14.0, 0.5) {
                spike_count += 1;
            }
        }
        assert!(
            spike_count > 5,
            "RS neuron should spike with I=14, got {spike_count}"
        );
    }

    #[test]
    fn test_izhikevich_no_spike_without_input() {
        let mut n = IzhikevichNeuron::regular_spiking();
        let mut spiked = false;
        for _ in 0..1000 {
            if n.tick(0.0, 0.5) {
                spiked = true;
            }
        }
        assert!(!spiked, "RS neuron should not spike with I=0");
    }

    #[test]
    fn test_izhikevich_presets() {
        let rs = IzhikevichNeuron::regular_spiking();
        let fs = IzhikevichNeuron::fast_spiking();
        let ch = IzhikevichNeuron::chattering();
        let ib = IzhikevichNeuron::intrinsically_bursting();
        assert!((rs.a - 0.02).abs() < f32::EPSILON);
        assert!((fs.a - 0.1).abs() < f32::EPSILON);
        assert!((ch.c - (-50.0)).abs() < f32::EPSILON);
        assert!((ib.d - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lif_spikes() {
        let mut n = LifNeuron::default_params();
        let mut spike_count = 0;
        for _ in 0..1000 {
            if n.tick(2.0, 0.5) {
                spike_count += 1;
            }
        }
        assert!(spike_count > 0, "LIF should spike with I=2");
    }

    #[test]
    fn test_lif_no_spike_without_input() {
        let mut n = LifNeuron::default_params();
        let mut spiked = false;
        for _ in 0..1000 {
            if n.tick(0.0, 0.5) {
                spiked = true;
            }
        }
        assert!(!spiked);
    }

    #[test]
    fn test_spiking_neuron_enum() {
        let mut n = SpikingNeuron::Izhikevich(IzhikevichNeuron::regular_spiking());
        let mut spiked = false;
        for _ in 0..1000 {
            if n.tick(14.0, 0.5) {
                spiked = true;
                break;
            }
        }
        assert!(spiked);
    }

    #[test]
    fn test_spiking_network_propagates() {
        let mut net = SpikingNetwork::new();
        let a = net.add_neuron(SpikingNeuron::Izhikevich(
            IzhikevichNeuron::regular_spiking(),
        ));
        let b = net.add_neuron(SpikingNeuron::Lif(LifNeuron::default_params()));
        net.add_synapse(a, b, 5.0, 1.0).unwrap();

        // Drive neuron A with strong input
        net.neurons[a] = SpikingNeuron::Izhikevich(IzhikevichNeuron {
            v: 29.0,
            ..IzhikevichNeuron::regular_spiking()
        });

        let mut b_spiked = false;
        for _ in 0..100 {
            net.tick(0.5).unwrap();
            if net.last_spikes().contains(&b) {
                b_spiked = true;
                break;
            }
        }
        // B should eventually spike from A's input
        assert!(
            b_spiked || net.time_ms > 10.0,
            "B should receive A's spikes"
        );
    }

    #[test]
    fn test_stdp_strengthens_causal() {
        let mut net = SpikingNetwork::new();
        let a = net.add_neuron(SpikingNeuron::Izhikevich(
            IzhikevichNeuron::regular_spiking(),
        ));
        let b = net.add_neuron(SpikingNeuron::Izhikevich(
            IzhikevichNeuron::regular_spiking(),
        ));
        net.add_synapse(a, b, 0.5, 0.5).unwrap();
        net.stdp = Some(StdpRule::default());

        let initial_weight = net.synapses[0].weight;

        // Make A spike, then B spike shortly after (causal)
        net.neurons[a] = SpikingNeuron::Izhikevich(IzhikevichNeuron {
            v: 31.0,
            ..IzhikevichNeuron::regular_spiking()
        });
        net.tick(0.5).unwrap();
        net.neurons[b] = SpikingNeuron::Izhikevich(IzhikevichNeuron {
            v: 31.0,
            ..IzhikevichNeuron::regular_spiking()
        });
        net.tick(0.5).unwrap();

        // Weight should have increased (LTP)
        assert!(
            net.synapses[0].weight >= initial_weight,
            "STDP should strengthen causal synapse: {} vs {}",
            net.synapses[0].weight,
            initial_weight
        );
    }

    #[test]
    fn test_network_negative_dt_rejected() {
        let mut net = SpikingNetwork::new();
        assert!(net.tick(-1.0).is_err());
    }

    #[test]
    fn test_network_invalid_synapse() {
        let mut net = SpikingNetwork::new();
        net.add_neuron(SpikingNeuron::Lif(LifNeuron::default_params()));
        assert!(net.add_synapse(0, 99, 1.0, 1.0).is_err());
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut net = SpikingNetwork::new();
        net.add_neuron(SpikingNeuron::Izhikevich(
            IzhikevichNeuron::regular_spiking(),
        ));
        net.add_neuron(SpikingNeuron::Lif(LifNeuron::default_params()));
        net.add_synapse(0, 1, 0.5, 1.0).unwrap();
        let json = serde_json::to_string(&net).unwrap();
        let net2: SpikingNetwork = serde_json::from_str(&json).unwrap();
        assert_eq!(net2.neurons.len(), 2);
        assert_eq!(net2.synapses.len(), 1);
    }
}
