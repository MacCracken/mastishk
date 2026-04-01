//! Basic usage of mastishk — exploring neurotransmitter dynamics.

use mastishk::neurotransmitter::NeurotransmitterProfile;

fn main() {
    // Create a default neurochemical profile
    let mut profile = NeurotransmitterProfile::default();

    println!("=== Initial State ===");
    println!("Serotonin:      {:.3}", profile.serotonin.level);
    println!("Dopamine:       {:.3}", profile.dopamine.level);
    println!("Norepinephrine: {:.3}", profile.norepinephrine.level);
    println!("GABA:           {:.3}", profile.gaba.level);
    println!("Glutamate:      {:.3}", profile.glutamate.level);
    println!("Arousal:        {:.3}", profile.arousal());
    println!("Reward sens.:   {:.3}", profile.reward_sensitivity());
    println!("Plasticity:     {:.3}", profile.plasticity_rate());
    println!("Inhibition ratio: {:.3}", profile.inhibition_ratio());
    println!();

    // Simulate a reward event — dopamine spike
    println!("=== Dopamine Spike (reward event) ===");
    profile.dopamine.stimulate(0.4);
    println!("Dopamine:       {:.3} (elevated)", profile.dopamine.level);
    println!("Reward sens.:   {:.3}", profile.reward_sensitivity());
    println!();

    // Simulate stress — norepinephrine + cortisol (via glutamate)
    println!("=== Stress Response ===");
    profile.norepinephrine.stimulate(0.3);
    profile.glutamate.stimulate(0.2);
    println!("Norepinephrine: {:.3} (elevated)", profile.norepinephrine.level);
    println!("Arousal:        {:.3} (elevated)", profile.arousal());
    println!();

    // Tick forward 5 seconds — watch transmitters decay toward baseline
    println!("=== After 5 seconds (decay toward baseline) ===");
    profile.tick_all(5.0);
    println!("Dopamine:       {:.3}", profile.dopamine.level);
    println!("Norepinephrine: {:.3}", profile.norepinephrine.level);
    println!("Arousal:        {:.3}", profile.arousal());
}
