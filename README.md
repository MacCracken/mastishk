# Mastishk

> **Mastishk** (Sanskrit: मस्तिष्क — brain) — computational neuroscience engine for AGNOS

Neurotransmitter dynamics, neural circuit simulation, sleep architecture, HPA axis stress response, default mode network modeling, and chronobiology. Provides the biological substrate that drives emotion, cognition, and behavior in the AGNOS stack.

Used by [bhava](https://github.com/MacCracken/bhava) (emotion/personality), [bodh](https://github.com/MacCracken/bodh) (psychology/cognition), [kiran](https://github.com/MacCracken/kiran) (game engine), [joshua](https://github.com/MacCracken/joshua) (agent characters), and [agnosai](https://github.com/MacCracken/agnosai) (agent orchestration).

## Modules

| Module | Description |
|--------|-------------|
| `neurotransmitter` | Monoamine dynamics (serotonin, dopamine, norepinephrine), GABA/glutamate balance, neuropeptides (oxytocin, endorphins), acetylcholine, BDNF neuroplasticity. Synthesis, reuptake, degradation kinetics |
| `circuit` | Neural circuit primitives: excitatory/inhibitory populations, firing rates, synaptic weights, Hebbian learning, lateral inhibition |
| `sleep` | Sleep architecture: NREM stages 1-3, REM cycling, adenosine buildup (Process S), sleep debt, ultradian 90-min cycles |
| `hpa` | Hypothalamic-pituitary-adrenal axis: CRH -> ACTH -> cortisol cascade, negative feedback, chronic stress adaptation, allostatic load |
| `dmn` | Default mode network: self-referential processing, mind-wandering, meditation suppression, task-positive network switching |
| `chronobiology` | Melatonin synthesis from light input, cortisol circadian rhythm (CAR), core body temperature oscillation, SCN pacemaker model |

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | Standard library support |
| `logging` | no | Structured logging via `MASTISHK_LOG` env var |
| `full` | -- | Enables all features |

## Quick Start

```toml
[dependencies]
mastishk = "0.1"
```

```rust
use mastishk::neurotransmitter::NeurotransmitterProfile;

// Create a default neurochemical profile
let mut profile = NeurotransmitterProfile::default();

// Stimulate dopamine (reward event)
profile.dopamine.stimulate(0.3);
println!("Arousal: {:.2}", profile.arousal());

// Tick the system forward 1 second — transmitters decay toward baseline
profile.tick_all(1.0);
println!("Arousal after tick: {:.2}", profile.arousal());
```

## Architecture

```text
rasayan — biochemistry (enzyme kinetics, metabolic pathways)
  | feeds molecular-level kinetics
mastishk (this) — neurotransmitter dynamics, neural circuits, sleep, HPA, DMN, circadian
  | neurotransmitter levels feed into
bhava — emotion/personality (serotonin->mood, dopamine->preference, cortisol->stress)
  | personality drives
bodh — psychology (cognition, perception, learning models)
  | cognition feeds
sharira — physiology (biomechanics, fatigue, energy)
```

Also feeds:
- **kiran** — game engine (NPC neurochemistry for believable behavior)
- **joshua** — agent character simulation (personality grounded in neuroscience)
- **agnosai** — agent orchestration (cognitive state modeling)

## Development

```bash
make check     # fmt + clippy + test + audit
make bench     # Run benchmarks with history tracking
make coverage  # Generate coverage report
make doc       # Build documentation
```

## License

GPL-3.0-only. See [LICENSE](LICENSE).
