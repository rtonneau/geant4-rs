# Session: Pre-feed gps sessions from roadmap and next.md, replace next.md

**Date:** 2026-10-16T16:51:43.300Z
**Status:** Grill phase complete

## Problem Statement

Work on geant4-rs is steered by two things: `docs/next.md` (a hand-maintained to-do list and draft prompt for M7) and `docs/08-roadmap.md`. The aim is to drop `next.md` and drive everything through gps from here. This session pre-feeds the next gps sessions (scout-style seeds) so that the roadmap goals M6 closure, M7 and M8 can each be started with `/gps start <slug>`. Geant4 verification data is part of the plan: sessions write C++ Geant4 generators but never run them, and report which simulations the user must run.

## Context & Constraints

- **Current behavior:** `main` contains M0-M3 (validated against Geant4 11.4.1) and the Rust M6 implementation (PR #10 merged). The M6 Geant4 reference exists (`validation/reference/physchem.csv`, `provenance_m6.toml`; the Q24 crash is fixed in `g4rs_ref`). The M6 gate still fails: `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture` stops at `1000 eV H2O2: z=111.6` (re-run during this grill). Nothing else is known because the test stops at the first failure. `docs/next.md` section 3 (branch/PR `m6-physchem-b`) is obsolete. M7 and M8 are not started. One unexplained 900 keV stack overflow occurred in the reference run (BUILD_NOTES section 6).
- **Pain point:** Plans live in a free-form file outside gps; there is no record of sessions, dependencies or the Geant4 runs the user must perform.
- **Dependencies:** AGENTS.md rules stay in force: never overwrite or delete reference data or existing generator modes (new files and new modes only); generators live in `validation/geant4/`, with provenance and BUILD_NOTES/README updates; tolerances are pre-registered and never loosened to pass; the Geant4 C++ repository is read-only. `docs/README.md:30` and the comment at `crates/dna-sim/tests/physchem_a.rs:17` reference `docs/next.md`.
- **Tech stack:** Rust workspace (dna-core, dna-data, dna-physics, dna-chemistry, dna-sim), Geant4 11.4.1 C++ generator `validation/geant4/g4rs_ref` (MSVC, built and run by the user), gps scripts (`scout-merge.js --from`).

## Success Metrics

- `docs/gps-backlog.md` exists with one section per seed (goal, preconditions, C++ tasks, tests, pre-registered acceptance) and `scout-merge.js --from docs/gps-backlog.md` registers all 13 seeds, visible in `/gps status`.
- `validation/geant4/RUN_REQUESTS.md` exists with the compile-only/never-run convention and the request format (exact command, seed, statistics, expected output files, provenance fields, consuming Rust ignored test), linked from `validation/geant4/README.md` and AGENTS.md.
- `docs/next.md` is deleted; its content is fully carried by the backlog, docs/09 and docs/08; no remaining references to it (docs/README.md, physchem_a.rs comment, AGENTS.md).
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` still pass (no code behavior changes; only a comment edit).

## Architecture & Approach

Seeds (13), with dependencies written in each seed's problem text ("requires <slug>"):

M6 closure:
1. `m6-comparison-report`: non-asserting per-energy/species table (Geant4 mean +- SE, Rust mean +- SE, z, relative difference, RMS radius both); record the failure and evidence in docs/08 first; close Q24 in docs/09 with a pointer to BUILD_NOTES section 5; asserting gate and tolerances unchanged.
2. `m6-diagnose-and-fix` (requires 1): one hypothesis at a time against the C++ source (dissociation-channel branching incl. H2O2/H2/O/OH- in `G4ChemDissociationChannels_option1`; species born at exactly 1 ps vs later decays; recombination/event ordering Q21/R14; count-time semantics; RMS radius semantics; known differences Q12, Q13, Q16, Q17; same cascade incl. secondaries followed to solvation, fluo/Auger off). Fix or document each deviation. May add new C++ diagnostic modes (e.g. per-decay-state species counts at 1 ps) with RUN_REQUESTS entries. Ends with the gate closed or explicitly open. Regenerating `physchem.csv` is out of scope; suspected generator problems are reported and asked about. The 900 keV stack overflow: capture a stack trace if it recurs.
3. `m6-close-gate` (requires 2, only if diagnostic data was requested): provenance check, run the consuming test, record the result.

M7 chemistry (option3 first):
4. `m7-option3-reaction-table`: port the option3 reaction table to validated TOML with source line provenance, C++ cross-check script, atom/charge conservation test for every reaction; extend the species table only where needed. Independent of M6.
5. `m7-diffusion-and-two-body-irt` (requires 4; M6 closed before consuming M6 positions): deterministic per-history chemistry streams, free Brownian diffusion, isolated A+B and A+A IRT sampler including partially diffusion-controlled reactions, explicit domain/error policy; V9 over three pre-registered D/R/k sets, A+A symmetry/no self-pairing, boundary/extreme values, thread-count independence; Bonferroni families and thresholds stated before running.
6. `m7-sbs-oracle-and-ode` (requires 5): SBS oracle for one totally diffusion-controlled case (IRT vs SBS within 3 sigma), V10 well-mixed ODE harness; new C++ two-body reference mode (never run) and RUN_REQUESTS entry; `#[ignore]`d Rust comparison.
7. `m7-sbs-close-gate` (requires 6 and the user's data): provenance check, run, record.
8. `m7-multispecies-irt-scheduler` (requires 5, 7 and `m6-close-gate`/M6 closed): full many-species IRT scheduler on the M6 1 ps species; C++ reference mode for species counts over time; `#[ignore]`d comparison.
9. `m7-scheduler-close-gate` (requires 8 and data).

M8 G(t):
10. `m8-gt-scoring-and-chem6-reference` (requires 9): G(t) scorer with uncertainties, normalisation per docs/01 A.4; C++ chem6-like G(t) generator (e-, V11 time list) and run request.
11. `m8-gt-close-gate` (requires 10 and data): V11 comparison for e- 100 keV and 1 MeV.
12. `m8-config-and-cli` (requires 10): TOML config, `dna run config.toml`, JSON output with metadata.
13. `m8-reproducibility-v13` (requires 11, 12): thread-count and seed reproducibility end to end.

Conventions for every seed that needs Geant4 data: the producing session adds new C++ modes or programs under `validation/geant4/` (existing modes untouched), may compile-check them against the installed Geant4 but never runs them, appends an entry to `validation/geant4/RUN_REQUESTS.md`, writes the `#[ignore]`d Rust comparison with pre-registered tolerances, and stops with the gate "awaiting data". A paired `*-close-gate` seed finishes the comparison after the user commits the data.

Tickets of this session (docs and C++ conventions only; no physics code): (a) create `validation/geant4/RUN_REQUESTS.md`, link it from README and AGENTS.md; (b) write `docs/gps-backlog.md`; (c) record the confirmed decisions in docs/09 (and ADRs only where hard to reverse) and add seed pointers to docs/08 with the M8 proton-10-MeV deviation noted; (d) delete `docs/next.md`, fix `docs/README.md` and the `physchem_a.rs` comment; (e) run `scout-merge.js --from docs/gps-backlog.md` and verify `/gps status`.

## Assumptions & Trade-offs

- Horizon is M6 closure through M8; M5 protons, M9, M10, M11 stay roadmap lines and are scouted later. M8's proton 10 MeV acceptance is deferred until M5 and noted as a deviation.
- M6 closes before M7 consumes its output (hard dependency); only the reaction-table port (seed 4) is independent.
- Standing decisions confirmed (recorded in docs/09): M7 implements option3 reactions first; the M6 same-decay pair statistic stays distinct from any future RDF/nearest-neighbour observable (R16); Q12 (no atomic de-excitation), Q13 (Geant4 elastic rotation default), Q16 (2 eV floor), Q17 (Meesungnoen2002 only) are kept unless the M6 reference contradicts them.
- Not doing: running Geant4, regenerating or editing any reference file, changing tolerances, implementing any milestone, touching the C++ reference repository.
- Trade-off: paired close-gate seeds add sessions but keep each session finishable without waiting for manual Geant4 runs.

## Open Questions

- Whether M6 diagnosis will need diagnostic Geant4 data (decides if seed 3 is used).
- Exact V11 energies and time list for seed 10 (docs/07 V11 lists 1 ps-1 us; the roadmap names e- 100 keV and 1 MeV): fixed in that seed's grill.
- Q13 default for science runs stays open (default `Geant4`).

## Notes

- Seeds have no dependency field, so dependencies are written in each seed's problem text.
- The grill ran with `mattpocock-skills:grilling` and the router's question rule; `superpowers` also provides a grill skill.
- Facts verified this session: the M6 gate fails on `main` at `1000 eV H2O2: z=111.6`; `m6-physchem-b` was merged as PR #10.
