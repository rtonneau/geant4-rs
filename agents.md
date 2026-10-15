# AGENTS.md — Geant4-DNA Rust

## Project

Develop a scientifically accurate, modular and high-performance Rust implementation of selected Geant4-DNA capabilities.

Initial scope:
- Electron and proton irradiation of liquid water.
- Discrete particle transport, including secondary electrons.
- Physicochemical transformations and spatially resolved water radiolysis.
- Time-dependent radiolytic yields (G-values).
- Headless execution through a CLI.

The architecture must remain extensible to additional particles, materials, physics models and chemical systems.

**Priorities:** scientific correctness > reproducibility > maintainability > performance.

## Repositories

- **Rust implementation (writable):** https://github.com/rtonneau/geant4-rs
- **Geant4 C++ reference (read-only):** https://github.com/rtonneau/geant4
- **Reference version:** Geant4 v11.4.1
- **Documentation:** https://geant4.web.cern.ch/docs/

Geant4 is the primary reference for algorithms and numerical methods, but agreement with Geant4 alone does not guarantee physical correctness.

Always verify relevant C++ source code rather than relying on assumptions. Record source paths, model versions and scientific data provenance.

Never modify the C++ reference repository.

## Scientific requirements

- Preserve the physical meaning of the original models.
- Document model applicability, energy ranges, assumptions and limitations.
- Never invent physical constants, cross sections or reaction parameters.
- Never silently extrapolate outside validated energy ranges.
- Distinguish physical, physicochemical and chemical stages.
- Preserve spatial correlations during track chemistry.
- Explicitly track energy deposition and secondary particles.
- Define G-value normalisation and scoring conventions.
- Use appropriate numerical precision and consistent physical units.
- Document and validate every intentional deviation from Geant4.

Do not sacrifice scientific correctness for performance.

## Rust architecture

Use modern, idiomatic Rust rather than mechanically translating the C++ class hierarchy.

Prefer:
- Composition over inheritance-style designs.
- Clear ownership and explicit data flow.
- Descriptive names and small, cohesive modules.
- Traits where interchangeable implementations are required.
- Enums for well-defined categories and states.
- Strong typing for physical quantities where useful.
- Minimal allocations in performance-critical code.
- Simple abstractions over unnecessary generalisation.

Keep particle transport, physics models, chemistry, scoring and simulation orchestration logically separate.

Avoid overengineering. Generalise concepts such as particles, materials and processes without introducing abstractions that have no practical use.

Follow standard Rust naming conventions (`snake_case`, `PascalCase`, `SCREAMING_SNAKE_CASE`).

## Approved dependencies

Core dependencies:
- `rayon` — CPU parallelism.
- `rand` — random sampling.
- `rand_chacha` — reproducible random-number generation.
- `serde` — serialisation.
- `toml` — configuration.
- `clap` — CLI.
- `thiserror` — error handling.

Additional approved dependencies:
- `approx` — numerical testing.
- `criterion` — benchmarking.
- `proptest` — property-based testing.
- `tokio` — only when asynchronous execution is genuinely needed.

Prefer the standard library when practical. Additional crates require justification based on reliability, maintenance, complexity and performance.

## Parallelism and performance

- Prefer independent-history parallelism using `rayon`.
- Use deterministic, history-based random-number streams.
- Avoid shared mutable RNG state.
- Aim for reproducibility independent of thread scheduling.
- Use thread-local simulation state and efficient data structures.
- Preserve correct chemical interactions when parallelising chemistry.
- Profile before optimising.
- Benchmark equivalent scientific workloads.
- Avoid unsafe code and GPU-specific implementations unless justified.

Optimisations must preserve validated scientific behaviour.

## Testing and validation

**Every new scientific feature must include tests.**

Use:
- Unit tests for numerical algorithms, units, interpolation and sampling.
- Integration tests for transport and chemistry.
- Regression tests against Geant4-DNA reference results.
- Statistical tests for Monte Carlo observables.
- Benchmarks for performance-critical components.

Validate energy accounting, physical model boundaries, random sampling and chemical kinetics.

Compare results with Geant4 v11.4.1 and, where possible, independent experimental or published data.

Define tolerances and statistical acceptance criteria before evaluating results.

Keep small reference datasets and validation scripts in the repository.

Run before completing a task:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If commands cannot run, explain why. Never claim tests passed without executing them.

## Documentation

Document:
- Scientific models and assumptions.
- Relevant Geant4 C++ source paths.
- Model energy limits and data provenance.
- Architectural decisions.
- Numerical algorithms and approximations.
- Validation methods and results.
- Public APIs and extension points.

Use rustdoc for public interfaces.

Prefer concise documentation explaining *why* an algorithm or design was chosen.

## Agent workflow

1. Inspect the current repository and relevant documentation.
2. Consult the authoritative Geant4 source when physics is involved.
3. Identify scientific and architectural implications.
4. Plan before making significant changes.
5. Implement incrementally with clear naming and minimal dependencies.
6. Add tests alongside new functionality.
7. Run relevant tests and validation.
8. Summarise changes, scientific limitations and remaining work.

Respect existing architecture unless a change is justified.

Do not rewrite unrelated code or introduce speculative features.

**When uncertain about physics, investigate rather than guess. When uncertain about architecture, prefer the simplest extensible solution.**

## Current development policy

Follow the approved implementation roadmap in `docs/` when available.

During planning-only tasks, produce documentation without implementing the simulation engine.

For implementation tasks, work on one validated milestone at a time.

Keep the codebase compilable and tests passing after each milestone.
