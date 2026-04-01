//! Neural circuit primitives — excitatory/inhibitory populations, firing rates.
//!
//! Simple rate-model neural populations with Hebbian learning and lateral
//! inhibition. Not a spiking simulator — these are mean-field models suitable
//! for personality/emotion simulation at behavioral timescales.

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
    pub fn tick(&mut self, input: f32, dt: f32) {
        let target = (self.resting_rate + input).clamp(0.0, 1.0);
        let alpha = 1.0 - (-dt / self.tau).exp();
        self.rate += (target - self.rate) * alpha;
        self.rate = self.rate.clamp(0.0, 1.0);
    }

    /// How far from resting rate.
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
        self.populations.push(pop);
        idx
    }

    /// Add a synapse.
    pub fn add_synapse(&mut self, from: usize, to: usize, weight: f32) {
        self.synapses.push(Synapse { from, to, weight });
    }

    /// Tick the circuit: compute inputs from synapses, then update all populations.
    pub fn tick(&mut self, dt: f32) {
        let mut inputs = vec![0.0_f32; self.populations.len()];
        for syn in &self.synapses {
            if syn.from < self.populations.len() && syn.to < self.populations.len() {
                inputs[syn.to] += self.populations[syn.from].rate * syn.weight;
            }
        }
        for (i, pop) in self.populations.iter_mut().enumerate() {
            pop.tick(inputs[i], dt);
        }
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
        pop.tick(0.5, 1.0);
        assert!(pop.rate > 0.2);
    }

    #[test]
    fn test_circuit_excitation() {
        let mut c = Circuit::new();
        let a = c.add_population(NeuralPopulation::new("A", 0.5, 0.1, true));
        let b = c.add_population(NeuralPopulation::new("B", 0.1, 0.1, true));
        c.add_synapse(a, b, 0.5);
        c.tick(0.5);
        assert!(c.populations[b].rate > 0.1);
    }

    #[test]
    fn test_circuit_inhibition() {
        let mut c = Circuit::new();
        let a = c.add_population(NeuralPopulation::new("A", 0.8, 0.1, true));
        let b = c.add_population(NeuralPopulation::new("B", 0.5, 0.1, false));
        c.add_synapse(a, b, -0.5);
        c.tick(0.5);
        assert!(c.populations[b].rate < 0.5);
    }

    #[test]
    fn test_serde_roundtrip() {
        let c = Circuit::new();
        let json = serde_json::to_string(&c).unwrap();
        let c2: Circuit = serde_json::from_str(&json).unwrap();
        assert_eq!(c2.populations.len(), 0);
    }
}
