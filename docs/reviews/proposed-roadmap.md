# Proposed GPS roadmap: validated native Rust track chemistry

Date: 2026-10-26. Baseline: `1bdcb2d391b5ac88b8802ac360ef1ac2bc9934b6`. **Approval proposal, not an active roadmap modification.** Evidence: [development-state.md](development-state.md). Exhaustive old-to-new mapping: [roadmap-review.md](roadmap-review.md).

## Executive summary and major decisions

Deliver one trustworthy electron-to-G(t) pipeline before expanding particle families or optimizing interacting chemistry. Keep the current five-crate native Rust architecture, immutable model data, separate stages, canonical Geant4 species keys and per-history RNG streams. Preserve the qualified M6 closure. Replace the stale process-count session with **R01 `validation-evidence-and-ci`**.

This plan contains eight near-term seeds and four later delivery seeds, not a seed per defect. Each seed below is a complete contract that GPS can turn into implementation tickets without another architectural-planning session. Scientific falsification may still require diagnosis; unavailable Geant4 data requires the user to run the prepared request, not invention of results or relaxation of thresholds.

Approval includes these explicit scope decisions:

1. No FFI rewrite or universal particle/material framework. Expand only for the second concrete particle family in R10.
2. All 72 option3 entries must have an executable, validated regime before “complete option3” is claimed. Runtime coverage is not table coverage.
3. Chemistry is serial within a history; histories are parallel. Count observation must not consume RNG or alter trajectories.
4. First chemistry profile: pure water, existing option3 data/constants, unbounded spatial domain, global 1 ps initialization, horizon at most 1 us, `G4DNAIRT` semantics—not IRT-syn. Preserve source-specific temperature/rate constants; do not normalize them to a newly assumed temperature.
5. First G(t) comparison uses full slowing down. Frozen A.4 track-segment/PrimaryKiller behavior is retained as R09, not silently omitted or mislabeled as already supported.
6. Retain original 100 keV and **1 MeV** electron acceptance cases. At exactly 1 MeV use a separately named pure-DNA Geant4 reference profile excluding the standard-EM overlap. Record this deliberate difference from the stock constructor. 900 keV may be a diagnostic, not a replacement endpoint.
7. Defer the original finite closed-system V10 criterion as D01. Network invariants and spatial reference agreement do not falsely discharge it. The first deliverable is a validated **unbounded electron profile**, not unqualified completion of every historical M7 requirement.
8. Keep independent physical assessment distinct from Geant4 agreement. No thresholds are invented for V6/V12 literature agreement.

## Delivery order and milestone gates

Priority: P0 = immediate evidence-integrity prerequisite; P1 = electron delivery path; P2 = retained follow-on capability. Complexity: M = a few cohesive implementation/validation tickets; L = multiple coupled scientific components; XL = a substantial milestone requiring internal vertical slices. These are relative estimates, not calendar promises; external reference-run latency is separate.

| Phase | Ordered seed(s) | Required predecessors | Milestone exit |
|---|---|---|---|
| A: trustworthy baseline | R01, then R02 | Existing checkout and approved migration | G0: strict maintained gates, honest support/provenance, malformed input rejected |
| B: chemistry kernels | R03, then R04 | R02; R04 also R01 | G1: every option3 regime executable; independent pair evidence passes |
| C: chemistry histories | R05 | R03 and R04 G1 | G2: deterministic serial scheduler, conservation/identity/time/search tests |
| D: electron application | R06, R08; then R07 | R06 needs R05/R02; R08 needs R06 schema; R07 needs R04/R06/R08 | G3: full-slowing electron G(t), CLI/V13 and separate count/yield gates accepted |
| E: retained capabilities | R09; R10 then R11 | R09 needs G3; R10 needs G3/R02; R11 needs R10 and G3 | G4a: track segment accepted; G4b: proton/H transport then 10 MeV G(t) accepted |
| F: measured hardening | R12 | G3/R08; rerun after G4 capabilities | G5: bounded/scaled workload evidence and current extension/release guidance |

R09 and R10 do not depend on each other. R12 may follow G3 before proton work if performance blocks practical campaigns. Only work whose scientific dependencies are satisfied may advance; external evidence waits do not justify calling a gate passed. R08 can proceed while the user acquires R06 reference data.

## Common contract for every seed

- Preserve existing reference CSV/provenance and generator modes. Add new names/modes for new campaigns. Agents may inspect upstream source and compile-check generators but **never run Geant4**. Add exact user commands and unique IDs to `validation/geant4/RUN_REQUESTS.md`; never reuse RR-001/RR-002.
- Keep three independent fields in the evidence record: implementation (`not-started/in-progress/complete`), evidence (`absent/requested/available/invalid`), verdict (`not-evaluated/pass/fail/inconclusive`). A session may finish its implementation with requested evidence; dependent scientific gates remain blocked until the required verdict passes or an explicitly approved scope change is recorded.
- Numerical/statistical contracts are committed before reading new comparison results. A malformed, incomplete or wrong-profile fixture is INVALID, not a scientific failure or a pass. Zero uncertainty with unequal deterministic values fails. Skipped tests/insufficient power are visible, not green evidence.
- New numeric budgets in this proposal (for example pair-oracle absolute error and spatial-cutoff sensitivity) are proposed acceptance decisions, not historical results or guarantees supplied by Geant4. Approval adopts them prospectively; established V1–V13 criteria retain their separately recorded history.
- Each implementation seed adds source-linked tests, updates capability/limitation documentation, and passes fmt, Clippy and workspace tests plus its named gates. No silent change to established scientific tolerances. External commands resolve environment paths; no hardcoded machine roots.
- Record source revision, implementation revision/dirty state, toolchain/target/platform, lockfile and loaded-data hashes, model policies, config/schema, seed/history IDs and observation convention. Distinguish same-platform reproducibility from cross-platform statistical agreement.
- Reference campaigns store compact per-history or independent-batch sufficient statistics with >=30 batches for uncertainty; preserve covariance needed for ratio estimators. Large raw diagnostics are optional external artifacts with identity/size/reproduction recipe, not mandatory repository fixtures.
- Every seed has an implementation exit and, where applicable, an evidence exit. The same seed resumes after user data acquisition; a new “close gate” session is not required merely to consume a file.

## Phase A — Trustworthy baseline

### R01 — `validation-evidence-and-ci`

**Title/objective:** Make scientific acceptance executable and fail closed. **Priority/complexity:** P0 / M. **Traceability:** NEW; F4–F6 and stale M6 closure work; original M0/M2/M3/M6 validation promises. **Dependencies:** none beyond migration approval; does not require new Geant4 data.

**Scope:** A small versioned inventory of maintained gates, fixture identities/profiles and historical exclusions; strict readers/comparators; CI enforcement of committed-data checks; truthful compiler support; current-status documentation. Choose supported pinned Rust 1.97.0 initially and align `rust-version` with that policy; do not advertise untested 1.85 support. Windows plus Linux smoke/build coverage, with the full statistical tier explicitly named.

**Exclusions:** New physical algorithms, new reference campaigns, blanket ignored tests, reopening M6, deleting old comparisons/data, a general workflow engine.

**Deliverables:** Current-gate manifest and runner; negative-fixture tests; required-data CI job and manual statistical entry point; stage-aware evidence summary; support-policy correction; concise current-state synopsis and approved shared instruction additions (keep `CLAUDE.md` as `@AGENTS.md`). Record RR-002's qualified conclusion and local-artifact limitation without requiring its huge CSV in CI.

**Acceptance:**

1. Maintained selection explicitly runs the four M2 tests, two M3 tests and named M6 prechem test; it excludes historical post-zero-time M6 gates and scratch-producing diagnostics.
2. Empty/truncated data, duplicate/missing energy/process/species rows, NaN/infinity/negative invalid quantities, wrong units/constructor/stage/seed metadata, invalid counts/moments and mismatched hashes all fail with actionable errors. Grid coverage is exact, not merely a minimum row count.
3. Deterministic zero-SE unequal means fail; equal values pass. Tests demonstrate each failure mode and a deliberately perturbed valid fixture is rejected.
4. Original valid fixtures retain their historical thresholds and disclosed exclusions; changed verdicts cause investigation, not automatic tolerance changes. M6 closure remains recorded even if a new harness issue is found, with a separate regression status.
5. A fresh clone's required gates fail on deliberately removed required data. CI does not silently skip table-dependent checks. Linux/Windows and pinned compiler status are explicit; inability to run a platform is recorded, not claimed passed.
6. README/current-roadmap summaries no longer present old failures/pending data as current facts. Historical sections remain intact and labeled. No process-count request is resurrected.

### R02 — `verified-inputs-and-provenance`

**Title/objective:** Bind simulation inputs, API invariants and result identity. **Priority/complexity:** P1 / M. **Traceability:** NEW; F1–F3/F6; original M0/M1/V1. **Dependencies:** R01.

**Scope:** Verified immutable data/model construction; exact layout/component constraints; finite-domain/overflow validation; provenance derived from actual loaded bytes including custom chemistry. Keep explicit synthetic/unverified constructors for tests, distinctly labeled. Prefer a deep construction boundary over extra public loader/validator/manifest parameters.

**Exclusions:** Physics formula changes, new file formats, generic material abstraction, new dependencies without justification, performance tuning.

**Deliverables:** Verified input bundle/context consumed by production orchestration; typed errors for invalid data/config; complete provenance envelope; focused regression/property tests and migration notes for affected library constructors.

**Acceptance:**

1. Changed table bytes cannot produce a result carrying the old manifest identity. Wrong manifest, missing file and checksum mismatch fail before transport. Custom chemistry identity is its actual data, not the embedded default hash.
2. Catalogue and direct model constructors reject too few/too many components. Total and partial cross sections describe the same channels. Scaled values and sums cannot overflow silently.
3. Zero-column angular and short-column ionisation tables, the narrow endpoint grid `[1.0, 1.0000000000001]`, non-finite positions and invalid ledger tolerance return typed errors or valid bounded interpolation as specified; never panic or silently bypass checks.
4. Provenance records implementation/toolchain/platform/lockfile/input/policy identities. Independent caller-supplied metadata cannot falsely label an already-constructed table.
5. Committed-data baseline physics/M6 results and RNG consumption remain unchanged for valid default inputs; R01 gates pass. Relocating the checkout/data directory does not change semantic identity.

## Phase B — Complete and independently check reaction execution

### R03 — `option3-reaction-execution`

**Title/objective:** Make the complete option3 network executable before scheduling it. **Priority/complexity:** P1 / L. **Traceability:** split from `m7-multispecies-irt-scheduler`; missing work revealed by F/M7 audit. **Dependencies:** R02 and the existing reaction-table/pair-kernel implementation.

**Scope:** Compile 72 table entries into explicit mobile-pair or unary/background channels. Add charged PDC for IDs 42–52, independent first-order hazards for IDs 53–72, validated product lists and source-faithful reaction-site/conditional parent placement. Use existing TDC/neutral-PDC kernels. Internal enum dispatch and immutable compiled network are sufficient; no generic plugin layer.

**Exclusions:** Multispecies event heap, boundary conditions, arbitrary scavenger configuration, IRT-syn, invented spin channels, silently fixing source rate/stoichiometry anomalies.

**Deliverables:** Coverage matrix mapping every reaction ID to execution regime and exact upstream source; typed network compilation; charged kernel/unary sampler/placement primitives; high-precision mathematical fixtures and conservation checks. Sources include upstream `G4DNAIRT::{Sampling,GetIndependentReactionTime,MakeReaction}` and `G4EmDNAChemistry_option3::ConstructReactionTable`.

**Acceptance:**

1. All 72 default entries compile with a supported execution descriptor; none are skipped. Unknown regime/invalid diffusion/rate/radius/product/overflow fails before history processing. Background configurations remain reservoirs/descriptors, not fake mobile species.
2. Charged-PDC tests cover all eleven configured channels, contact/inside/outside radii, attractive/repulsive effective parameters where applicable, long-time no-reaction mass and parameter limits. Existing charged TDC/neutral PDC results remain valid. Independent high-precision formula evaluation, not production-code-generated expectations, supplies deterministic anchors.
3. Each unary channel samples an exponential clock with the source's units/rate convention; zero rate means no event, invalid rate errors. Competing-channel tests recover total hazard and branching fractions using preregistered family alpha 0.001, N >=100,000 samples per tested family.
4. Product identity/order and one/two/three-product placement follow the source-specific branch. Test zero separation, unequal diffusion and degenerate cases; a source ambiguity becomes a documented explicit error/limitation, never an arbitrary Brownian placement.
5. Atom/charge accounting is tested for every channel with reservoir exchanges explicit; only the existing named reaction-53 exception is permitted. No blanket conservation exemption.

**R03 evidence boundary:** [The 72-ID execution matrix](../validation/option3-reaction-coverage.md) records constructor lines, IRT branches, ordered products and the exact reaction-53 balance exception. It covers descriptor execution and isolated primitives only. R04 independently assesses pair distributions and the Geant4 gate; R05 begins correlated many-species history scheduling only after R04/G1. Neither pair acceptance nor history-level G(t) follows from R03 coverage.

### R04 — `pair-oracles-and-reference-gate`

**Title/objective:** Independently validate reaction-time primitives across supported regimes. **Priority/complexity:** P1 / L. **Traceability:** split `m7-sbs-oracle-and-ode`, merged `m7-sbs-close-gate`, expanded V9. **Dependencies:** R01/R03. **External dependency:** new user-run Geant4 pair fixtures.

**Scope:** An independent test-only step-by-step Brownian oracle with appropriate interaction potential/radiation boundary for the tested regime; analytic/high-precision CDF and no-reaction probabilities; separate Geant4 pair mode, compile-check and user request. Test TDC neutral/charged and PDC neutral/charged, A+B and A+A. Enumerate applicable option3 representatives in a committed case matrix before results.

**Exclusions:** Production SBS engine, arbitrary box chemistry, V10 closed-system claim, scheduler/track simulations, reuse of production sampler inside the oracle.

**Deliverables:** Case matrix and independent oracle; convergence report; new generator mode/fixture schema/provenance; strict consuming test and separate analytic/SBS/Geant4 verdicts. Reuse existing V9 fixtures; do not overwrite them.

**Acceptance:**

1. For each represented regime use initial separations at contact, 1.5 and 3 times the applicable reaction radius, and positive observation scales 0.01/0.1/1/10/100 times `a^2/(D1+D2)`, plus eventual no-reaction probability. Explicitly distinguish dimensionless test cases from literal option3 channels.
2. Analytic tests use >=100,000 samples/case with precommitted seeds; report pointwise z and a Bonferroni family alpha 0.001. Retain existing legacy V9 verdicts separately rather than retroactively reinterpreting them.
3. SBS uses independently implemented stepping/boundary handling. Halve timestep through at least three levels; the last two CDF estimates must differ by <0.005 absolute and be compatible within three combined SE. Use >=10,000 independent pairs/case; increase by a precommitted doubling schedule up to 1,000,000 only for precision, not repeated significance hunting. Failure to establish convergence is INCONCLUSIVE.
4. Geant4 comparison uses independently seeded runs and exact matching units/parameters/algorithm. CDF/reaction-fraction differences must satisfy the same predeclared family test and <0.01 absolute effect-size bound. Precision must resolve that bound; insufficient precision is not pass. Tests include an injected biased sampler that fails.
5. G1 passes only when analytic, converged independent-oracle and Geant4 evidence all pass. Generator implementation can finish earlier with evidence REQUESTED; no invented data. Original V10 remains separately owned by D01.

## Phase C — A deterministic chemistry history

### R05 — `serial-irt-history-engine`

**Title/objective:** Evolve one complete correlated chemical history safely and deterministically. **Priority/complexity:** P1 / XL. **Traceability:** split scheduler seed; early V13; network-invariant part of the orphan oracle seed. **Dependencies:** R03 and R04/G1. Internal tickets should progress through two-particle, competing triplet, products/unary, then full-network histories—not independent public frameworks.

**Scope and fixed design:**

- `dna-chemistry` owns an arena with monotonic molecule IDs, species, position, last materialized time, alive flag and generation. Validate the M6 input list at common global 1 ps; assign IDs in existing creation order.
- A min-event queue stores absolute event time, reaction ID, participant IDs/generations and a stable insertion ordinal. Reject NaN/negative/backwards times. Define total ordering and deterministic ties; never depend on hash iteration or pointer addresses.
- Invalidate events involving consumed/version-changed participants; unaffected scheduled pairs are not resampled after every unrelated reaction. Schedule products in source order and preserve lazy partner-time and conditional placement semantics from upstream `G4DNAIRT`.
- Unary clocks are separate from pair clocks. Finite resource limits return typed incomplete-history errors; never silently drop reactions or emit a complete scientific score from a truncated history.
- Preserve the immutable M6 pre-IRT snapshot. Process contact events to closure at 1 ps; chemical count observations are right-continuous (all events at or before the observation time). Observe counts without RNG draws or moving survivors. Position diagnostics need a separate explicitly non-mutating contract.
- Use a source-faithful finite-horizon spatial index for the production profile (<=1 us), with configurable documented cutoff multiplier and a small-system all-pairs reference implementation. Canonicalize candidate ordering. Do not claim cutoff means mathematically zero omitted probability.

**Exclusions:** Intra-history parallelism, arbitrary chemical boundaries, track termination/scoring, full public CLI, event-heap persistence, IRT-syn. A well-mixed reaction-network oracle may check stoichiometry but is not V10 spatial validation.

**Deliverables:** A small history-engine interface accepting immutable network/input/options and caller RNG, returning counts/status/diagnostics; owned internals; deterministic trace fixtures; all-pairs oracle; source-semantic and spatial-search convergence report.

**Acceptance:**

1. Fixtures cover no reactions, pair contact, competing A+B/A+C, A+A, unary-vs-pair competition, three-way ties, repeated product reactions, stale events, empty input and explicit resource exhaustion. No dead molecule reacts; no duplicate pair initialization; time is monotonic and each event has exact lineage.
2. Accounting holds after every event, with explicit reservoir terms and only reaction 53's named exception. Full-network initialization fails fast on unsupported regimes.
3. Adding/removing intermediate observation times leaves the final state and chemistry RNG stream bitwise identical. Pre-IRT 1 ps and post-contact 1 ps outputs are differently labeled and separately tested.
4. Same history/network/seed is bitwise repeatable; wrapper execution at 1/2/8 threads and multiple batch sizes matches. No order derives from a randomized container.
5. Small-system indexed/all-pairs and cutoff multipliers 1/2/4 are compared on preregistered clustered and dilute ensembles. For non-rare count means, the 99% interval of relative difference must lie within +/-1%; for means <0.1 molecule/history, absolute difference interval must lie within +/-0.01 molecule/history. Inconclusive precision or unresolved cutoff bias blocks production acceptance; do not silently change the default until sensitivity is reported.
6. A representative high-molecule-count history has measured memory/event/candidate counts. Capacity limits are configurable and reported. G2 requires all local semantics/convergence checks, while the later full-track Geant4 count gate remains R07.

## Phase D — Electron G(t), CLI and scientific acceptance

### R06 — `electron-gt-pipeline-and-reference`

**Title/objective:** Produce reproducible time-dependent electron yields and the matching reference campaign. **Priority/complexity:** P1 / L. **Traceability:** rewrite `m8-gt-scoring-and-chem6-reference`, scheduler reference producer, V13. **Dependencies:** R02/R05. **External dependency:** user reference runs prepared here, consumed in R07.

**Scope:** Compose physics -> M6 -> chemistry -> compact scoring inside each history worker; ordered deterministic reduction. Full slowing down in unbounded water, every secondary transported. Keep all original physics ledgers and M6 output unchanged. Score all network species; mandatory V11 family is `e_aq`, `°OH`, `H`, `H2`, `H2O2` at 1/10/100 ps, 1/10/100 ns, 1 us. Chemistry times are absolute, not elapsed from zero.

**Exclusions:** Track-segment/finite scoring box (R09), proton transport, arbitrary time >1 us, raw molecule dumps by default, a second independent G(t)-only reference campaign.

**Deliverables:** G(t) scorer and versioned library output; compact per-history/batch sufficient statistics; bounded whole-history orchestration; one new Geant4 mode emitting raw species counts plus both G estimators/energy moments at the same times; profile/provenance/run request and strict consumer scaffold.

**Acceptance:**

1. Hand-computed fixtures verify mean of `100*N/E_dep` and `100*sum(N)/sum(E_dep)` separately, including correlated variable-energy histories where their large-sample targets differ. Report both; use the first for chem6-compatible comparison and the second as the declared primary scientific estimator, not as an interchangeable number.
2. `E_dep` is physical local deposit only; chemical decay energy is separate. Explicitly define zero-deposit, no-molecule, aborted and weighted histories. Compatibility output reproduces source conventions; scientific output reports eligibility/denominators instead of silently discarding cases. >=30 independent batches and covariance-aware ratio uncertainty are tested against a direct calculation.
3. Every accepted output contains pre-IRT snapshot identity and post-contact chemical time convention. Count insertion does not alter chemistry outcomes. 1/2/8 threads and batch sizes 1/7/64 yield identical scientific values on the same platform; environment metadata may differ only in named execution fields.
4. Worker memory holds only its active history plus bounded ordered summaries; it does not retain 1,024 full event histories by default. Measure peak memory on documented 100 keV and 1 MeV workloads; errors cannot produce a falsely successful partial output.
5. New reference profiles retain original 100 keV and 1 MeV acceptance cases. Record the exact pure-DNA endpoint process list; standard-EM overlap is excluded explicitly and labeled. Use a distinct full-slowing mode rather than claiming unchanged stock chem6/PrimaryKiller parity. 1/10/900 keV diagnostic cases are optional and cannot replace required cases.
6. Before any new data are examined, commit seed lists, history/batch counts, precision/power plan and all R07 test families. Use at least two independent Geant4 master seeds per required energy. The generator emits its own batch moments, not uncertainties borrowed from Rust. Existing modes/files stay untouched.

### R08 — `versioned-cli-and-reproducible-runs`

**Title/objective:** Make the verified electron pipeline usable outside development tests. **Priority/complexity:** P1 / M. **Traceability:** rewrite `m8-config-and-cli`, merge V13 integration. **Dependencies:** R02/R06 schema and implementation; may proceed during R06 external evidence wait.

**Scope:** Thin `dna-cli` with clap; `dna run config.toml`; validated serde/TOML input and versioned JSON output. Explicit unit-bearing keys, particle/model/geometry/time profiles, seed, histories, thread count, data resolution and output path. Reject unsupported capabilities rather than silently falling back. No simulation logic in CLI.

**Exclusions:** GUI, service, async runtime, plugins, scripting DSL, arbitrary user chemistry or automatic Geant4 execution.

**Deliverables:** CLI crate, schema/example configs, error/exit-code contract, output compatibility policy and installed/out-of-tree integration tests.

**Acceptance:**

1. Valid minimal full-slowing electron config runs; invalid units/ranges/species/profile/unknown keys/missing data fail before costly work. Times must be finite, positive, ordered, within the supported absolute window; seed/history range is validated.
2. Explicit data path and documented environment resolution work from an unrelated working directory; repository-relative developer fallback is not the only supported mechanism.
3. Unsupported proton, track-segment or >1 MeV requests are explicit errors until their seeds land. No schema option implies a capability absent from the library.
4. Interrupted/failed runs do not leave an apparently complete result. Existing output is not silently overwritten. Output contains schema, config, data/provenance, stage/profile, counts, both estimators and evidence-status fields.
5. CLI and library give identical scientific values; same-platform 1/2/8-thread and batch tests pass. At least one end-to-end example is exercised in CI. Help and docs state that reference acquisition is user-owned and scientific acceptance is not implied by command success.

### R07 — `electron-scientific-acceptance`

**Title/objective:** Accept or reject the complete unbounded electron profile with explicit evidence. **Priority/complexity:** P1 / L. **Traceability:** merge scheduler-close, G(t)-close and final V13; restore missing V4/V6/V12 obligations. **Dependencies:** R04/R06/R08 and valid new reference data. This is one coordinated campaign with separate verdicts, not one opaque pass bit.

**Scope:** Consume the R06 count/G(t) campaign; test scheduler counts, yield estimators, independent-seed behavior, existing transport/physchem gates and physical-assessment reports. Close missing Geant4 final-state sampling coverage with new compact fixtures where the inventory proves a gap, using original V4 observables/energies and a source-matched profile.

**Exclusions:** Tolerance fitting, replacing original evidence, inferring physical truth from Geant4 parity, declaring V10 passed, track-segment or proton claims.

**Deliverables:** Machine-readable comparison results and readable validation report; per-gate profile/coverage/exclusions; V4 coverage matrix and any new user requests; V6 electron stopping/W and V12 literature assessment with citations, units and uncertainties; explicit release capability statement.

**Acceptance:**

1. All R01 maintained gates pass on the candidate revision. Reference metadata covers exact constructor/processes, chemistry hash, stages, times, energy, compiler/platform, seeds and generator revision. Missing/invalid batches fail evidence validation.
2. Raw count/time scheduler gate covers all 15 mobile species and required times/energies, separately from the five-species G(t) family. Use preregistered independent-batch uncertainties and Bonferroni family alpha 0.01; report every point, including rare/zero populations and deterministic cases. No count failure may be hidden by normalization.
3. V11 retains the original requirements: every five-species/time/energy point within three combined SE; Bonferroni-corrected family p >0.01; relative systematic difference <5% at t >=1 ns; statistical relative error <2% on the nonzero comparison means. Explain that pointwise and family criteria are both required even where one dominates. If both values are deterministic zero, require equality; if a nonzero rare case cannot reach the required precision, mark it INCONCLUSIVE rather than substituting an easier criterion or dropping it.
4. Independent Geant4 seeds and independent Rust seeds are checked; same seed number is not cross-language pairing. Batch covariance and history counts are reported. Precision expansion follows the preregistered plan, not repeated testing until significance disappears.
5. V13 is bitwise within the supported platform/toolchain/data profile for thread counts 1/2/8 and multiple batches; different-seed compatibility uses the committed family alpha 0.01. Test through the CLI as well as library. Observation-grid invariance passes.
6. V4's Geant4 coverage is explicit: raw-table/independent-Python tests are L1/L2-like evidence, not automatically Geant4 samples. Original V4 distribution families/thresholds are preserved for new source-matched fixtures. If acquisition is incomplete, G3 is qualified rather than claiming full transport validation.
7. V6/V12 reports identify applicable primary published/experimental sources and quantify differences with uncertainty; no invented L4 pass threshold. If source data are unavailable, identify the exact missing evidence and do not claim physical validation.
8. G3 verdict is PASS only for the stated unbounded electron capability when required L1–L3 gates pass. A failed comparison yields a diagnosis/new narrowly scoped fix request; an inconclusive campaign yields a precision/evidence request. D01/V10 and R09 remain visible limitations, not implicitly closed M7/M8 obligations.

## Phase E — Preserve original scientific capabilities

### R09 — `track-segment-scoring-and-reference`

**Title/objective:** Implement the frozen A.4 controlled-energy-loss/LET comparison mode. **Priority/complexity:** P2 / L. **Traceability:** original docs/01 A.4 and M8 scope extracted from rewritten G(t) seed. **Dependencies:** G3/R08. **External dependency:** new user-run segment reference.

**Scope:** A separate physical scoring/termination policy matching `chem6/PrimaryKiller` and `ScoreSpecies`: post-step box exit, primary accumulated loss/minimum energy, event-abort maximum loss, track weights, escaped and remaining energy, per-event resets. Chemistry remains unbounded after initialization; a physical scoring box is not a reflective chemical boundary.

**Exclusions:** Chemical boundary conditions/V10, arbitrary geometry engine, changed full-slowing defaults, proton segment claims before R10/R11.

**Deliverables:** Typed termination/scoring profile and CLI schema extension; source audit closing the outstanding accumulator-reset question; ledger tests; new immutable reference mode/data/consumer and acceptance report.

**Acceptance:** Synthetic histories pin `>` versus `>=`, exact boundary equality, primary versus secondary behavior, reset between events, weight/abort/zero-energy semantics and escaped/remaining energy. Ledger residual remains within existing V7 limits; an additional <=1e-12 relative target from A.4 is reported and any conflict requires an explicit decision, not silent threshold replacement. Both G estimators/uncertainties are checked with variable deposits. Source-matched 100 keV and 1 MeV segment cases pass R07-style count/G(t) criteria with preregistered segment parameters and independent batches/seeds. Full-slowing regressions and V13 remain unchanged. G4a cannot pass while source reset semantics or external evidence are unresolved.

### R10 — `proton-hydrogen-transport`

**Title/objective:** Deliver the second real particle family without destabilizing electrons. **Priority/complexity:** P2 / XL. **Traceability:** original M5, audit G3, V2–V7/PSTAR. **Dependencies:** R02 and G3; not R09.

**Scope:** Explicit proton/neutral-H state and charge-changing transitions; Miller-Green, Rudd/Extended, Born, ion elastic and Dingfelder within source-supported windows up to 100 MeV. Secondary electrons reuse M3. Generalize only the event/secondary record and dispatch/state seams needed for p/H; preserve the tested electron API as a wrapper if appropriate. Data catalogue/provenance expands with actual source inputs.

**Exclusions:** RPWBA 100–300 MeV (D02), heavy ions/nuclear reactions, photons, arbitrary materials, generic C++ class hierarchy, proton G(t) acceptance (R11).

**Deliverables:** Source/window/data inventory; typed p/H transition model and transport; process/secondary ledger; table/unit tests; new reference generators/requests/consumers; PSTAR report and documented boundaries.

**Acceptance:** Every transition conserves the explicitly modeled energy/charge bookkeeping and emitted-electron identity; boundary/overlap and repeated charge-exchange tests pass. Unsupported energies error, never extrapolate. V2/V3 deterministic cross sections/probabilities, V4 secondary spectra, V5 ranges/stopping and V7 per-history accounting meet their original preregistered criteria with adequate independent statistics. Electrons' existing gates remain unchanged. PSTAR comparison is a quantified L4 report, not a newly invented pass threshold. 10 MeV proton histories produce valid stage records consumable by M6; chemistry validation is not claimed yet. Implement in internal vertical slices by model range and state transition, not separate architecture-only GPS seeds.

### R11 — `proton-gt-and-physical-assessment`

**Title/objective:** Complete the retained 10 MeV proton radiolysis requirement. **Priority/complexity:** P2 / L. **Traceability:** original M5/M8/V11/V12 and explicit deferral in `m8-gt-close-gate`. **Dependencies:** R10, G3; R09 only if segment output is requested.

**Scope:** Source-audit proton-event to physchem mapping; reuse the accepted option3 scheduler/scoring; new 10 MeV full-slowing Geant4 reference profile; independent physical assessment and CLI support. Verify rather than assume electron-derived event semantics suffice.

**Exclusions:** High-energy RPWBA, different chemistry, proton track segments unless separately enabled, new RNG model or copied scheduler.

**Deliverables:** Mapping tests/necessary source-proven conversion additions; immutable reference campaign; count/G(t)/energy and V13 reports; supported proton example; applicable V12 literature comparison.

**Acceptance:** Stage/energy accounting and canonical species semantics match the audited proton source path. All seven V11 times/five mandatory species satisfy the preregistered R07 statistical/effect-size/precision criteria at 10 MeV using >=30 batches and >=2 Geant4 seeds. Raw counts, both estimators and independent-seed results are reported. Whole-history memory is measured; capacity errors never masquerade as successful truncation. Electron gates remain green. G4b is scoped to the supported proton/H energy/profile; no RPWBA or generic-ion claim.

## Phase F — Measured performance and maintainable delivery

### R12 — `profile-harden-and-document`

**Title/objective:** Make validated workloads practical and extension contracts honest. **Priority/complexity:** P2 / M. **Traceability:** original M9/M10, F7/F8. **Dependencies:** G3/R08; repeat relevant measurements after R09/R11. May precede R10 if measured limits impede campaigns.

**Scope:** Criterion/manual benchmark tier, 1/2/8-thread independent-history scaling, peak memory/candidate/event counts, and evidence-driven optimization. Consolidate C++ mapping, supported model/domain matrix, extension guides and executable examples. Record immutable benchmark configuration and scientific outputs alongside timings.

**Exclusions:** Unsafe/GPU work, intra-history parallelism, broad speculative abstraction or arbitrary speedup promises.

**Deliverables:** Reproducible benchmark commands/fixtures/results, profiling analysis, any justified bounded optimizations with before/after evidence, current API/extension documentation and release checklist.

**Acceptance:** Report physics-only, M6 and full-chemistry costs separately for 1/10/100 keV and 1 MeV electron workloads, plus supported proton workload when available. Record hardware/toolchain/data/config, throughput, scaling and memory, including non-improvements. Any optimization preserves bitwise results where promised; otherwise requires an explicitly approved RNG/numerical-policy revision and all scientific gates. No skipped precision criteria for speed. A fresh user can reproduce a documented run outside the checkout; docs match actual capabilities, including deferred items below.

## Deferred requirements: retained, not silently deleted

These are **not additional near-term GPS seeds**. They are explicit activation contracts. When activated, register the named objective with these boundaries; do not redesign the current architecture merely to anticipate them.

| ID / proposed deferred objective | Trigger and dependencies | Scope, exclusions and concrete deliverable | Minimum acceptance / priority / complexity |
|---|---|---|---|
| D01 `closed-volume-kinetics-v10` | After G3; a closed/well-mixed domain is explicitly requested, or before claiming unqualified historical M7 completion | Define a physically valid finite-domain reaction-diffusion model and oracle; closed-system A+B and A+A kinetics against ODE. Do not apply infinite-space pair CDFs to minimum-image periodic distances and call that exact; do not relabel a separate Gillespie test as validation of production spatial IRT. Deliver boundary/mixing derivation, implementation and convergence report. | Preserve original <=1% long-time agreement with uncertainty resolving 1%, >=3 mixing/time/volume refinement levels, conservation and source/independent-oracle evidence. Until then V10 stays DEFERRED. P2 / XL. |
| D02 `proton-rpwba-extension` | R10/R11 accepted and >100 MeV proton use required | RPWBA excitation/ionisation and differential data over source-supported 100–300 MeV; boundary transition to Born; extend reference/schema. No heavy ions/nuclear physics. | Original V1–V7 applicable tests, explicit overlap/endpoints, source-matched reference and memory/runtime assessment. P2 / L. |
| D03 `electron-alternative-options` | G3 plus a named option4/6 scientific comparison | Emfietzoglou/Uehara/CPA100 as applicable, with 10/250 keV transitions, distinct fidelity profiles and fixtures. No replacing option2 defaults. | V1–V5 and G(t) comparisons with exact constructors/data; preserve baseline gates. P2 / L per requested option, not automatic proliferation. |
| D04 `extended-electron-energy` | G3 plus >1 MeV requirement and explicit choice of track-structure versus standard-EM semantics | Audit/source-supported new model range; no silent use of condensed-history EM under a DNA label. Deliver domain/profile, ledger and independent reference campaign. | Upper/lower-boundary tests, V2–V7 and stated chemical validity; unsupported ranges continue to error. P2 / XL. |
| D05 `chemical-environments-and-boundaries` | G3 and a specific oxygen/scavenger/material/finite-boundary use case | Add only requested environment, concentrations/reservoirs/boundaries and their source-backed rates; distinguish geometry from scoring. No arbitrary user reaction DSL initially. | Data/provenance/conservation, analytic boundary/kinetics tests, independent reference/physical assessment; D01 fulfilled if closed-system claims are made. P2 / XL. |
| D06 `intertrack-chemistry-and-dose-rate` | G3 plus explicit multi-track/UHDR requirement | Correlated tracks and timing in one interacting domain; independent-history superposition is not adequate. Deliver domain ownership/RNG/scheduling and a dose-rate validation campaign. | Limiting isolated-track recovery, intertrack timing/conservation and independent benchmark; no claim based solely on summing single-track G(t). P2 / XL. |
| D07 `alternative-chemistry-algorithms` | R12 demonstrates need or a scientific requirement names SBS/IRT-syn/intra-history parallelism | One requested alternative behind a meaningful history-engine seam; no speculative traits beforehand. Deliver algorithm/profile and comparison evidence. | Pair/history/G(t) gates and reproducibility/resource contract; measured benefit where performance motivated. P2 / XL. |
| D08 `alternative-physics-fidelity` | G3 plus a need for slow differential sampling or atomic de-excitation | Source-audit fast/slow sampling equivalence or Auger/fluorescence as separate explicit profiles. Preserve current no-deexcitation baseline; photons remain out of initial scope. | New final-state/ledger/reference evidence; Q4/Q12 limitations updated only after acceptance. P2 / L. |

Original exclusions remain exclusions unless separately approved: biological/DNA geometry, damage modeling, GUI/GPU, hadronic nuclear interactions and photon-induced transport. Additional particles/materials are extension goals, not permission to expand this delivery silently.

## Approval-ready migration and next action

Approve the following concise mapping; [roadmap-review.md](roadmap-review.md) contains the exhaustive rationale:

- **DELETE from executable backlog:** process-count request and residual-close; retain history and qualified M6 evidence.
- **SPLIT:** orphan SBS/ODE into R04 plus R05 invariants and deferred D01; scheduler into R03/R05 with its reference producer consolidated in R06.
- **MERGE:** pair-close into R04; scheduler-close and G(t)-close into R07 with separate verdicts; V13 into R05/R06/R08 and final R07 acceptance.
- **REWRITE:** G(t) scoring/reference as R06 with track-segment obligation R09; config/CLI as R08.
- **CREATE:** R01/R02 prerequisites, explicit R03 missing execution work, and retained later R09–R12 work. Preserve D01–D08 as deferred requirements, not near-term implementation churn.
- **KEEP:** existing code, completed-session history, immutable reference evidence and the core architecture. No unfinished seed is kept verbatim.

Apply the approved migration through supported GPS tooling in a separate change, reconcile registry/backlog/pointers atomically, and then start **R01 `validation-evidence-and-ci`**. This audit itself has not applied the migration or executed a seed.
