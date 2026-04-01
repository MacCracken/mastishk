# Security Policy

## Scope

Mastishk is a pure computational neuroscience library providing neurotransmitter dynamics, neural circuit simulation, sleep architecture, HPA axis modeling, and chronobiology for Rust. The core library performs no I/O and contains no `unsafe` code.

## Attack Surface

| Area | Risk | Mitigation |
|------|------|------------|
| Floating-point arithmetic | NaN/Inf propagation from extreme inputs | All levels clamped to 0.0..=1.0; division guarded |
| Serde deserialization | Crafted JSON with out-of-range values | Struct validation via serde derive; consumers should validate |
| Circuit simulation | Large population/synapse counts | Bounded by consumer input; no unbounded allocation |
| Sleep/circadian tick | Extreme dt values | Exponential decay math remains stable; outputs clamped |
| AI client (opt-in) | Network I/O to daimon/hoosh | Feature-gated; not compiled by default |
| Dependencies | Supply chain compromise | cargo-deny, cargo-audit in CI; minimal core deps |

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x | Yes |

## Reporting

- Contact: **security@agnos.dev**
- Do not open public issues for security vulnerabilities
- 48-hour acknowledgement SLA
- 90-day coordinated disclosure

## Design Principles

- Zero `unsafe` code
- No `unwrap()` or `panic!()` in library code — all errors via `Result`
- All public types are `Send + Sync` (compile-time verified)
- No network I/O in core library (AI client is opt-in via feature flag)
- Minimal dependency surface (core depends only on serde, thiserror, tracing)
