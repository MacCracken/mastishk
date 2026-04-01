# Development Roadmap

> **Status**: Released | **Current**: 1.0.0

## Backlog

### AI Integration

- [ ] Daimon client for agent registration
- [ ] Hoosh client for LLM-powered neuroscience queries
- [ ] MCP tools: `mastishk_neurotransmitters`, `mastishk_sleep`, `mastishk_stress`, `mastishk_circadian`, `mastishk_circuit`

---

# Road to v2.0 — World-Class Completeness

> Based on exhaustive domain audit (2026-03-31). Every item below represents proven,
> published neuroscience with behavioral-level significance.

## Pharmacological Completeness

- [ ] Negative allosteric modulator (NAM) — flumazenil (BZD reversal), mGluR5 NAMs (anxiety)
- [ ] Inverse agonism — distinct from antagonism (reduces constitutive activity)
- [ ] CYP450 drug metabolism — isoform tags on DrugProfile, metabolic competition model
- [ ] Buspirone preset — 5-HT1A partial agonist
- [ ] Aripiprazole preset — D2 partial agonist + 5-HT1A partial agonist
- [ ] Buprenorphine preset — mu-opioid partial agonist

## Neuromodulator Completeness

- [ ] Neuropeptide Y (NPY) — strongest endogenous anxiolytic, opposes CRH in amygdala
- [ ] Vasopressin — social behavior, pair bonding, aggression, V1a receptor
- [ ] Dynorphin + kappa-opioid receptor — stress→dysphoria pathway
- [ ] Substance P + NK1 receptor — pain-mood coupling
- [ ] NE inverted-U for PFC — Yerkes-Dodson

## Brain Region Expansion

- [ ] PFC subregions: dlPFC (working memory), vmPFC (value/emotion regulation), OFC (reversal learning)
- [ ] Periaqueductal gray (PAG) — pain modulation, defensive behavior
- [ ] Bed nucleus of stria terminalis (BNST) — sustained anxiety
- [ ] Thalamus — sensory relay, attention gating

## Computational Models

- [ ] Drift-diffusion model (DDM) — two-choice decision-making
- [ ] Rescorla-Wagner associative learning
- [ ] Wilson-Cowan population dynamics
- [ ] FitzHugh-Nagumo spiking model
- [ ] Attractor networks — working memory, pattern completion

## Remaining Proven Systems

- [ ] Adenosine as general neuromodulator
- [ ] Glycine co-transmission — obligate NMDA co-agonist
- [ ] Nitric oxide retrograde signaling
- [ ] H3 histamine autoreceptor
- [ ] Astrocyte regulation of synaptic transmission

## v2.0 Criteria

- [ ] 30+ neuromodulators/transmitters
- [ ] 25+ receptor subtypes
- [ ] 14+ brain regions
- [ ] Computational learning models (TD, Rescorla-Wagner, DDM)
- [ ] Complete pharmacological mechanism set (agonist, partial agonist, antagonist, inverse agonist, PAM, NAM, reuptake inhibitor)
- [ ] Withdrawal/rebound dynamics validated against clinical timelines
- [ ] All parameters validated against published literature with citations
- [ ] 500+ tests
- [ ] Benchmark regression tracking with 3+ point history
