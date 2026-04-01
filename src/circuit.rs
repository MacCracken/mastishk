//! Neural circuit primitives — excitatory/inhibitory populations, firing rates.
//!
//! Simple rate-model neural populations with synaptic propagation. Not a
//! spiking simulator — these are mean-field models suitable
//! for personality/emotion simulation at behavioral timescales.

use crate::error::{MastishkError, validate_dt};
use serde::{Deserialize, Serialize};

/// A neural population's activity state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralPopulation {
    /// Population label.
    pub name: String,
    /// Firing rate (0.0 = silent, 1.0 = maximal).
    pub rate: f32,
    /// Resting rate.
    pub resting_rate: f32,
    /// Time constant for rate changes (seconds).
    pub tau: f32,
    /// Whether this is excitatory (true) or inhibitory (false).
    pub excitatory: bool,
}

impl NeuralPopulation {
    /// Create a new population.
    #[must_use]
    pub fn new(name: impl Into<String>, resting_rate: f32, tau: f32, excitatory: bool) -> Self {
        Self {
            name: name.into(),
            rate: resting_rate,
            resting_rate,
            tau,
            excitatory,
        }
    }

    /// Apply input drive and decay toward resting rate over `dt` seconds.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn tick(&mut self, input: f32, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        self.tick_unchecked(input, dt);
        Ok(())
    }

    /// Tick without validating dt. Used by [`Circuit::tick`] after a single
    /// validation pass.
    #[inline]
    pub(crate) fn tick_unchecked(&mut self, input: f32, dt: f32) {
        let target = (self.resting_rate + input).clamp(0.0, 1.0);
        let alpha = 1.0 - (-dt / self.tau).exp();
        self.rate += (target - self.rate) * alpha;
        self.rate = self.rate.clamp(0.0, 1.0);
    }

    /// How far from resting rate.
    #[inline]
    #[must_use]
    pub fn activation(&self) -> f32 {
        self.rate - self.resting_rate
    }
}

/// A synaptic connection between two populations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    /// Source population index.
    pub from: usize,
    /// Target population index.
    pub to: usize,
    /// Connection weight (-1.0 to 1.0, negative = inhibitory).
    pub weight: f32,
}

/// A simple circuit of connected neural populations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circuit {
    /// Neural populations.
    pub populations: Vec<NeuralPopulation>,
    /// Synaptic connections.
    pub synapses: Vec<Synapse>,
}

impl Circuit {
    /// Create an empty circuit.
    #[must_use]
    pub fn new() -> Self {
        Self {
            populations: Vec::new(),
            synapses: Vec::new(),
        }
    }

    /// Add a population, returns its index.
    pub fn add_population(&mut self, pop: NeuralPopulation) -> usize {
        let idx = self.populations.len();
        tracing::debug!(name = %pop.name, idx, excitatory = pop.excitatory, "population added");
        self.populations.push(pop);
        idx
    }

    /// Add a synapse between two populations.
    ///
    /// # Errors
    /// Returns [`MastishkError::InvalidCircuit`] if `from` or `to` is out of bounds.
    pub fn add_synapse(
        &mut self,
        from: usize,
        to: usize,
        weight: f32,
    ) -> Result<(), MastishkError> {
        let len = self.populations.len();
        if from >= len || to >= len {
            return Err(MastishkError::InvalidCircuit(format!(
                "synapse {from}->{to} out of bounds (population count: {len})"
            )));
        }
        tracing::debug!(from, to, weight, "synapse added");
        self.synapses.push(Synapse { from, to, weight });
        Ok(())
    }

    /// Tick the circuit: compute inputs from synapses, then update all populations.
    ///
    /// # Errors
    /// Returns [`MastishkError::NegativeTimeDelta`] if `dt < 0.0`.
    #[inline]
    pub fn tick(&mut self, dt: f32) -> Result<(), MastishkError> {
        validate_dt(dt)?;
        tracing::trace!(
            dt,
            populations = self.populations.len(),
            synapses = self.synapses.len(),
            "ticking circuit"
        );
        let mut inputs = vec![0.0_f32; self.populations.len()];
        for syn in &self.synapses {
            if syn.from < self.populations.len() && syn.to < self.populations.len() {
                inputs[syn.to] += self.populations[syn.from].rate * syn.weight;
            }
        }
        for (i, pop) in self.populations.iter_mut().enumerate() {
            pop.tick_unchecked(inputs[i], dt);
        }
        Ok(())
    }
}

impl Default for Circuit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_population_tick() {
        let mut pop = NeuralPopulation::new("test", 0.2, 0.5, true);
        pop.tick(0.5, 1.0).unwrap();
        assert!(pop.rate > 0.2);
    }

    #[test]
    fn test_circuit_excitation() {
        let mut c = Circuit::new();
        let a = c.add_population(NeuralPopulation::new("A", 0.5, 0.1, true));
        let b = c.add_population(NeuralPopulation::new("B", 0.1, 0.1, true));
        c.add_synapse(a, b, 0.5).unwrap();
        c.tick(0.5).unwrap();
        assert!(c.populations[b].rate > 0.1);
    }

    #[test]
    fn test_circuit_inhibition() {
        let mut c = Circuit::new();
        let a = c.add_population(NeuralPopulation::new("A", 0.8, 0.1, true));
        let b = c.add_population(NeuralPopulation::new("B", 0.5, 0.1, false));
        c.add_synapse(a, b, -0.5).unwrap();
        c.tick(0.5).unwrap();
        assert!(c.populations[b].rate < 0.5);
    }

    #[test]
    fn test_serde_roundtrip() {
        let c = Circuit::new();
        let json = serde_json::to_string(&c).unwrap();
        let c2: Circuit = serde_json::from_str(&json).unwrap();
        assert_eq!(c2.populations.len(), 0);
    }

    #[test]
    fn test_negative_dt_rejected() {
        let mut pop = NeuralPopulation::new("test", 0.2, 0.5, true);
        assert!(pop.tick(0.0, -1.0).is_err());

        let mut c = Circuit::new();
        c.add_population(NeuralPopulation::new("A", 0.5, 0.1, true));
        assert!(c.tick(-0.5).is_err());
    }

    #[test]
    fn test_activation() {
        let mut pop = NeuralPopulation::new("test", 0.2, 0.5, true);
        assert!((pop.activation() - 0.0).abs() < f32::EPSILON);
        pop.tick(0.5, 1.0).unwrap();
        assert!(pop.activation() > 0.0);
    }

    #[test]
    fn test_empty_circuit_tick() {
        let mut c = Circuit::new();
        c.tick(1.0).unwrap(); // should not panic
    }

    #[test]
    fn test_out_of_bounds_synapse_rejected() {
        let mut c = Circuit::new();
        c.add_population(NeuralPopulation::new("A", 0.5, 0.1, true));
        assert!(c.add_synapse(0, 99, 0.5).is_err()); // invalid target
        assert!(c.add_synapse(99, 0, 0.5).is_err()); // invalid source
    }
}
