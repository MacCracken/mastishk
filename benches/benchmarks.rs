use criterion::{Criterion, black_box, criterion_group, criterion_main};

use mastishk::circuit::{Circuit, NeuralPopulation};
use mastishk::hpa::HpaState;
use mastishk::neurotransmitter::NeurotransmitterProfile;

fn bench_neurotransmitter_tick(c: &mut Criterion) {
    let mut profile = NeurotransmitterProfile::default();
    profile.dopamine.stimulate(0.3);
    profile.norepinephrine.stimulate(0.2);

    c.bench_function("neurotransmitter_tick_all", |b| {
        b.iter(|| {
            let mut p = profile.clone();
            p.tick_all(black_box(0.016)).unwrap();
            p
        })
    });
}

fn bench_circuit_tick(c: &mut Criterion) {
    let mut circuit = Circuit::new();
    let a = circuit.add_population(NeuralPopulation::new("excitatory_1", 0.5, 0.1, true));
    let b = circuit.add_population(NeuralPopulation::new("inhibitory_1", 0.3, 0.2, false));
    let d = circuit.add_population(NeuralPopulation::new("excitatory_2", 0.4, 0.15, true));
    let e = circuit.add_population(NeuralPopulation::new("inhibitory_2", 0.2, 0.25, false));
    circuit.add_synapse(a, b, 0.5).unwrap();
    circuit.add_synapse(b, a, -0.3).unwrap();
    circuit.add_synapse(a, d, 0.4).unwrap();
    circuit.add_synapse(d, e, 0.6).unwrap();
    circuit.add_synapse(e, b, -0.2).unwrap();

    c.bench_function("circuit_tick_4pop_5syn", |b| {
        b.iter(|| {
            let mut c = circuit.clone();
            c.tick(black_box(0.016)).unwrap();
            c
        })
    });
}

fn bench_hpa_tick(c: &mut Criterion) {
    let mut hpa = HpaState::default();
    hpa.stress(0.7);

    c.bench_function("hpa_tick", |b| {
        b.iter(|| {
            let mut h = hpa.clone();
            h.tick(black_box(0.016)).unwrap();
            h
        })
    });
}

criterion_group!(
    benches,
    bench_neurotransmitter_tick,
    bench_circuit_tick,
    bench_hpa_tick
);
criterion_main!(benches);
