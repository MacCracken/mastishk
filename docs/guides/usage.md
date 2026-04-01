# Usage Guide

> Patterns, philosophy, and code examples for mastishk

## Quick Start

```rust
use mastishk::brain::BrainState;

// Create a default brain (all subsystems at resting state)
let mut brain = BrainState::default();

// Advance 1 second — all 30+ tick steps execute in causal order
brain.tick(1.0).unwrap();

// Query composite outputs
println!("Arousal: {:.2}", brain.arousal());
println!("Stress: {:.2}", brain.stress());
```

## Philosophy

**mastishk owns no clock.** All tick methods accept `dt` (seconds) from the caller. The simulation consumer (kiran game engine, joshua agent system, bhava in non-game contexts) controls simulation speed, pause, time-skip, slow-mo. mastishk is a pure physics engine — it computes the next state given a time delta.

**Behavioral timescale.** All models operate at seconds-to-days resolution suitable for personality/emotion simulation, not cellular-level detail. The one exception is the spiking module, which operates at millisecond timescale independently.

**Composable.** Use `BrainState` for the full integrated system, or use individual modules (neurotransmitter, sleep, hpa, etc.) directly with coupling functions from `coupling.rs`.

## BrainState — The Full System

```rust
use mastishk::brain::BrainState;
use mastishk::circuit::NeuralPopulation;

let mut brain = BrainState::default();

// Optional: add neural circuit populations
let exc = brain.circuit.add_population(
    NeuralPopulation::new("excitatory", 0.5, 0.1, true)
);
let inh = brain.circuit.add_population(
    NeuralPopulation::new("inhibitory", 0.2, 0.2, false)
);
brain.circuit.add_synapse(exc, inh, 0.5).unwrap();

// Configure character profile
brain.age.age_years = 25.0;                // young adult
brain.hormones.estradiol = 0.7;            // higher estradiol
brain.hormones.testosterone = 0.3;
brain.circadian.set_photoperiod(10.0);     // late autumn
brain.circadian.phase_hours = 8.0;         // morning

// Simulate a game frame (60fps = 16ms)
brain.tick(0.016).unwrap();

// Simulate 1 hour in 1-second steps
for _ in 0..3600 {
    brain.tick(1.0).unwrap();
}
```

## Applying Stimuli

```rust
// Stress event
brain.hpa.stress(0.7);

// Threat perception
brain.amygdala.perceive_threat(0.8);

// Reward
brain.basal_ganglia.receive_reward(0.9);
brain.reward_circuit.reward_cue(0.6);
brain.reward_circuit.receive_reward(0.8);

// Dopamine burst (phasic RPE)
brain.neurotransmitter.fire_dopamine_burst(0.5);

// Meditation
brain.dmn.meditate(1.0).unwrap();

// Task engagement
brain.dmn.engage_task(0.8);

// Memory encoding
brain.hippocampus.encode(0.7);

// Motor error
brain.cerebellum.signal_error(0.5);

// Infection/illness
brain.inflammation.infect(0.6);

// Sleep
brain.sleep.fall_asleep();
// ... tick for 8 hours ...
brain.sleep.wake_up();
```

## Drug Administration

```rust
use mastishk::pharmacology::DrugProfile;

// Preset drugs
brain.administer_drug(DrugProfile::ssri_fluoxetine(), 0.5);
brain.administer_drug(DrugProfile::benzodiazepine_diazepam(), 0.3);
brain.administer_drug(DrugProfile::stimulant_amphetamine(), 0.7);
brain.administer_drug(DrugProfile::stimulant_methylphenidate(), 0.4);

// Drugs have pharmacokinetics: absorption → peak → elimination
// SSRI effects build over ~2 weeks due to 5-HT1A receptor desensitization
// Benzodiazepines act within minutes but tolerance builds in days
```

## Bridge — Consuming in bhava/kiran/joshua

```rust
use mastishk::bridge::{brain_mood_modifiers, BrainMoodEffect};

let effect: BrainMoodEffect = brain_mood_modifiers(&brain);

// Neurotransmitter → emotion
effect.mood_offset;          // -1.0 to +1.0 (serotonin-driven)
effect.reward_sensitivity;   // 0.0-1.0 (dopamine)
effect.arousal;              // 0.0-1.0 (norepinephrine)
effect.anxiety;              // 0.0-1.0 (GABA/glutamate ratio)
effect.focus;                // 0.0-1.0 (acetylcholine)

// HPA → stress
effect.stress_multiplier;    // 1.0-3.0 (cortisol)
effect.burnout;              // 0.0-1.0 (allostatic load)

// Body systems
effect.sickness_behavior;    // 0.0-1.0 (neuroinflammation)
effect.sympathetic;          // 0.0-1.0 (fight-or-flight)
effect.hrv;                  // 0.0-1.0 (autonomic regulation)
effect.interoceptive_anxiety; // 0.0-1.0 (body prediction error)

// Brain regions
effect.executive_control;    // 0.0-1.0 (PFC)
effect.fear_level;           // 0.0-1.0 (amygdala)
effect.learning_rate;        // 0.0-1.0 (hippocampus)
effect.action_drive;         // 0.0-1.0 (basal ganglia Go-NoGo)
effect.motor_quality;        // 0.0-1.0 (cerebellum)
```

## Spiking Networks (Standalone)

The spiking module operates independently at millisecond timescale:

```rust
use mastishk::spiking::*;

let mut net = SpikingNetwork::new();
let a = net.add_neuron(SpikingNeuron::Izhikevich(
    IzhikevichNeuron::regular_spiking()
));
let b = net.add_neuron(SpikingNeuron::Lif(LifNeuron::default_params()));
net.add_synapse(a, b, 1.5, 2.0).unwrap(); // weight=1.5, delay=2ms

// Enable STDP learning
net.stdp = Some(StdpRule::default());

// Simulate 100ms in 0.5ms steps
for _ in 0..200 {
    net.tick(0.5).unwrap();
    for &idx in net.last_spikes() {
        println!("Neuron {} spiked at {:.1}ms", idx, net.time_ms);
    }
}
```

## Serialization

All types support serde. Save and restore complete brain state:

```rust
let json = serde_json::to_string(&brain).unwrap();
let restored: BrainState = serde_json::from_str(&json).unwrap();
```

New fields use `#[serde(default)]` — old saved states deserialize safely into new versions with defaults for missing fields.

## Coupling Parameters

All coupling strengths are tunable via `brain.coupling`:

```rust
brain.coupling.sleep_nt_smoothing = 0.3;      // faster sleep-NT transition
brain.coupling.dmn_hpa_gain = 0.6;            // stronger rumination→stress
brain.coupling.circuit_gain.k_ne = 0.5;       // stronger NE→circuit modulation
brain.coupling.region.ne_amygdala_gain = 0.4;  // stronger NE→amygdala amplification
```
