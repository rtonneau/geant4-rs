# Use IRT-syn for production chemistry

_ADR 0004, recorded 2026-11-08 (session 2026-11-08__irt-syn-parallel-checkpoint-direction)._

<!-- Format adapted from mattpocock-skills' domain-modeling (ADR-FORMAT.md): https://github.com/mattpocock/skills.git -->

The first production release uses synchronized independent reaction time (IRT-syn) for interacting tracks in one unbounded water volume, because independent-history parallelism cannot model reactions between tracks. This supersedes [ADR 0003](0003-option3-independent-reaction-time.md) **for production model selection only**: its option3 IRT kernels, evidence and historical verdicts remain intact, while IRT-syn receives separate source, serial, restart and spatial-parallel gates. Geant4 11.4.1 remains normative; the reviewed 11.4.3 files match byte-for-byte but any run against that installation must retain its actual version and cannot silently replace the baseline.

The [reference contract](../../validation/geant4/README.md#irt-syn-reference-contract-2026-11-08) distinguishes executable source behavior, unresolved source anomalies and prospective Rust extensions. Shared-memory worker arbitration and checkpoint continuation are Rust designs, not capabilities inferred from Geant4 synchronization; production defaults remain gated by [SYN validation](../../docs/07-validation-strategy.md#irt-syn-prospective-gates-2026-11-08).
