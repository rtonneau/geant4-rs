# geant4-rs: development-state and architecture audit

Date: 2026-10-26. Baseline: `1bdcb2d391b5ac88b8802ac360ef1ac2bc9934b6` (2026-10-23), plus the pre-existing untracked GPS start placeholder. This is an analysis/planning report, not an implementation or GPS migration. Paths in this report are repository-relative unless identified as upstream. See also [roadmap-review.md](roadmap-review.md) and [proposed-roadmap.md](proposed-roadmap.md).

## Executive summary

The project is a credible **native Rust research implementation of selected Geant4-DNA capabilities**, not a Geant4 binding and not yet a complete radiolysis application. Electron transport, secondary cascades, the stage-matched 1 ps physicochemical boundary and isolated reaction-time kernels are real implementations. Multispecies chemistry, time-dependent G-values, proton transport and a supported CLI remain absent.

Keep the five-crate architecture and validated electron path. Do not introduce FFI, translate the C++ inheritance tree, or build a universal particle framework now. The main weaknesses are scientific input contracts, provenance, reference-consumer rigor and missing chemistry execution semantics—not unsafe Rust or cross-runtime ownership.

The next selected GPS session, `m6-process-counts-geant4-request`, is obsolete in its present form. M6 has an explicit qualified approval; the old residual workflow should not be revived simply because its seed survived in local state. The recommended next session is **R01 `validation-evidence-and-ci`**, followed by verified inputs and complete option3 execution. The detailed critique and migration map are in the companion reports.

Readiness: suitable for controlled electron/physicochemical research and incremental development; **not ready for production G(t), proton claims, full Geant4-DNA compatibility or independent physical-validation claims**. Completed GPS sessions are evidence of delivered work, not proof of scientific correctness.

### Current R01 validation and CI status (2026-10-26)

The maintained gate inventory selects four M2 Geant4 comparisons, two M3 comparisons and the M6 RR-001 pre-IRT 1 ps comparison. `python scripts/run_validation_gates.py --tier required` fails on missing or altered committed data/reference evidence; `--tier statistical` runs the selected ignored comparisons manually. Historical post-zero-time M6 gates and exploratory or scratch-producing spatial diagnostics remain excluded. CI now specifies pinned Rust 1.97.0, format, Clippy, workspace tests and the required tier for Linux and Windows. Linux and remote Windows CI results are unverified until those jobs actually run.

M6's closure remains qualified. RR-002 met its precision target and did not reproduce the earlier 12–13% spatial RMS excess; the interval includes equality, so this is not an equivalence result. Its CSV/provenance remain local-only under `.scratch/rr-002-80/` and cannot be used as committed CI evidence. The baseline audit and historical findings below retain their original as-of context.

## 1. Scope, evidence and checks

Inspected `AGENTS.md`, README, the original `docs/01`–`docs/10` planning package, architecture/reproducibility/units documents, GPS backlogs, local registry, session grills/tickets/commit logs, available non-shallow Git history, all five crate boundaries, relevant source/tests/examples, CI, Geant4 generators/run requests and reference metadata. Specialist reviews covered history, Rust architecture, validation and chemistry/source fidelity.

The actual checkout is `E:/DEV/RUST/geant4-rs`; the supplied C: checkout path is stale. Environment values resolve `DEV_DIR` to `E:/DEV` and `G4_ROOT` to `E:/GEANT4/geant4-v11.4.1-install`. Upstream source was inspected under `${DEV_DIR}/GEANT4/geant4-v11.4.1-source/geant4-v11.4.1`. No `.claude/.claude-project.json` exists in the checkout. No Geant4 executable was run and no machine paths were added to application configuration.

Evidence distinctions: **verified** means inspected source or an executed check; **historical** means a recorded result not reproduced here; **inference** means a consequence of inspected code without a dedicated reproduction. No code-coverage percentage, measured memory bound, speedup, cross-platform equivalence or experimental validation is claimed.

### Executed verification

Windows, Rust/Cargo 1.97.0, locked/offline dependencies:

| Command | Observed result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | PASS |
| `cargo test --workspace --locked --offline` | PASS: 363 passed, 18 ignored, zero failures |
| `cargo test -p dna-sim --test geant4_reference --locked --offline -- --ignored` | PASS: four maintained M2/second-seed comparisons |
| `cargo test -p dna-sim --test slowing_down_reference --locked --offline -- --ignored` | PASS: two maintained M3 comparisons |
| Named M6 `m6_prechem_counts_and_rms_match_geant4` ignored test | Launched, but final output was unavailable after the process handle disappeared; no fresh pass claimed |

These are Rust consumers of existing Geant4 data, not new Geant4 campaigns. Their success does not negate the consumer defects below. A blanket ignored-test run was deliberately avoided: it includes superseded stage-mismatched comparisons and a local spatial diagnostic that writes scratch output. The approximately 20.5 GB local spatial CSV was not rescanned or regenerated. No MSRV compiler, Linux build, new generator build, fuzzing or benchmark campaign was executed.

## 2. Original objectives and subsequent decisions

The first substantive planning package is commit `45684d2` (2026-10-15). Earlier `02b4a67`, titled “Initial plan,” contains a README, not a missing detailed design. `8b589e8` introduced project conventions. Original intent is recoverable from `docs/01-scientific-requirements.md`, `02-cpp-reference-inventory.md`, `03-architecture-comparison.md`, `04-scientific-algorithms.md`, `05-parallelisation.md`, `07-validation-strategy.md` and `08-roadmap.md` at that commit.

Original scope: electron **and proton** irradiation of liquid water; discrete transport and secondaries; separated physical/physicochemical/chemical stages; spatially correlated chemistry; G(t), uncertainties and explicit energy normalization; headless configuration/CLI; extensibility without speculative abstraction. Priorities remain correctness, reproducibility, maintainability, then performance (`AGENTS.md`).

Architecture B favored Rust composition, strong units, immutable model data, per-history RNG streams and Rayon. It proposed core/data/physics/chemistry/simulation/CLI boundaries. Validation levels are L1 algorithms, L2 deterministic Geant4 parity, L3 statistical simulation parity and L4 independent physical evidence. V1–V13 in `docs/07-validation-strategy.md` are broader than ordinary tests.

| Development decision | Evidence | Consequence |
|---|---|---|
| Foundations, parsers, electron models, orchestration | `a3f1412`, `ab176bf`, `8fbf769`, `bd91dee` | Native implementation, not bindings |
| External-only data proposal became a committed subset | `10000df`, then `2872bbf` / `341e996`; docs/10 | Twenty real files, about 7 MB; an approved chronological change |
| Initial fixtures and amended M3 criteria | `9be2255`, `210000c` | Keep amendments visible; not every current criterion predates first comparison |
| Physicochemistry A/B and canonical species | `4418d22`, `b897636`, `f314019` | Preserve exact configuration keys and stage boundaries |
| GPS replaced `next.md` | `d85722d`, `f60fe91`; earlier `684d193` | Registry, backlog and sessions all require reconciliation |
| Stage-matched reference, then recombination correction | `c6bea6e`, `40be229`, `5df0aa1`, `4d3816f` | Snapshot mismatch and a real zero-time retry/order bug were separate problems |
| Thermalisation and option3-table audits | `920faa6`, `2553983`, `8458bae` | Source-faithful identities/rates, with explicit anomalies |
| Isolated IRT implementation | `abd616c`, `a43fcaf`, `9da7410`, `fb1591c`, `82b3430`, `4a7bbcf`, `9dd499a` | Math/diffusion/parameters/pair kernels, not full M7 |
| Qualified M6 closure | `635e91b`, `99dd76b`; `.work/sessions/2026-10-19__m6-close-gate/01-grill/resume.md` | Explicit lead approval with acknowledged uncertainty |
| RR-002 spatial conclusion | `1bdcb2d`; `docs/superpowers/plans/2026-10-23-rr-002-conclusion-report.md` | Old 12–13% discrepancy not reproduced; no equivalence proof |

Intentional limitations recorded in docs/09 Q12/Q13/Q16/Q17 include no K-shell Auger/fluorescence in the selected fidelity profile, Geant4-compatible non-unit elastic rotation by default with an orthonormal alternative, a 2 eV primary floor, and Meesungnoen2002 thermalisation only. Different RNG/free-path algorithms require statistical rather than bitwise cross-language comparison. None warrants retuning physical constants to fit yields.

The M7 decisions select `G4DNAIRT`, not `G4DNAIRT_syn`, and preserve executable option3 literals over inconsistent comments. Reaction 53's source anomaly has a narrow conservation exception. The original V9 box wording was refined to unbounded isolated-pair validation; V10's closed well-mixed system is a separate unmet requirement.

## 3. Original-plan status matrix

| Phase | Intended acceptance | Actual state and evidence | Remaining work |
|---|---|---|---|
| M0 foundations | Workspace, units, RNG, provenance, CI | Implemented: `dna-core`, toolchain, CI, determinism tests | Bind provenance; correct supported-compiler claim |
| M1 data | Real tables, interpolation, checksums, V1 | Implemented with gaps: `dna-data/src/{datadir,catalog,xs,blocks}.rs` | Exact shapes, safe boundaries, overflow, mandatory verified loading |
| M2 electron option2 | Elastic/excitation/ionisation and V2–V5 | Selected models implemented; four reference checks pass here | Full Geant4 V4 final-state coverage is not proved by hand-script tests |
| M3 low-energy cascade | Attachment/vibration/solvation, secondaries, ledger | Implemented: `transport_electron`, low-energy/cascade tests; two reference checks pass here | Honest exclusions and V6/L4 assessment; robust consumers |
| M4 alternatives | Electron options4/6 | Not implemented; optional | Defer until a named scientific use requires them |
| M5 proton/H | Charge states, models, stopping/range, chemistry | Not implemented; table/transport are electron-specific | Explicit bounded proton/H architecture, data and validation |
| M6 physicochemistry | Species/displacements at 1 ps | Implemented and formally closed with limitations; `dna-chemistry/src/engine.rs`, RR-001 | Preserve closure; portable evidence and residual-risk labeling |
| M7 chemistry | Network, diffusion, IRT, SBS/analytic/ODE validation | Partial: 72-entry data; free diffusion; charged/neutral TDC and neutral PDC | Charged PDC, unary/background channels, product placement, scheduler, independent validation |
| M8 G(t)/CLI/V13 | Time grid, estimators, uncertainty, headless execution | Missing end-to-end; `dna-sim` scores transport/1 ps; config serializes but does not load TOML | Scheduler-backed scoring, reference campaign, CLI/config, full-stage reproducibility |
| M9 performance | Profiling/scaling on equivalent workloads | Ignored baseline test exists; no completed chemistry campaign | Bound memory; profile only after correctness |
| M10 API/docs | Extension guides, mapping, validated examples | Partial source-linked docs/examples; summaries stale | Executable end-to-end examples and honest extension contracts |
| M11 extensions | Wider energies, boundaries, chemical systems, particles/parallelism | Not implemented | Explicit deferred requirements and activation conditions |

A completion percentage would be misleading: the remaining M7/M8 work is larger and riskier than counting session names suggests.

## 4. Architecture and Rust audit

| Crate | Actual role | Assessment |
|---|---|---|
| `dna-core` | Units, vectors, species/events, RNG, manifest/provenance, shared penetration/sampling | Useful vocabulary; “no physics” prose is too absolute |
| `dna-data` | Depends on core; parsers, interpolation, catalogue, data directory | Good boundary, incomplete construction invariants |
| `dna-physics` | Depends on core/data; electron models, process table, transport and ledger | Explicit science, not particle-generic |
| `dna-chemistry` | Depends on core; physchem, reaction data, diffusion, isolated IRT | Correct independence from transport implementation |
| `dna-sim` | Composes crates, parallelizes histories, scores/serializes | Correct owner for whole-history orchestration; batching needs revision |

No Rust `unsafe`, runtime foreign interface, bindgen or cxx layer was found. FFI ownership/lifetime risk is therefore not applicable to the runtime. Separate C++ reference programs link Geant4; Rust ports its selected algorithms/data. Ordinary owned history state, borrowed immutable tables and caller-owned RNGs are appropriate. There is no basis for an FFI rewrite.

Strengths: typed errors, explicit model windows/termination, compensated energy accounting, Welford/co-moment estimates, source-linked formulas, fixed-order reductions and `RngStreams` history/stage separation (`dna-core/src/rng.rs`, including a 62-bit history-ID guard). Dependencies fit the approved set. No async, GPU or plugin framework is needed. Existing developer examples do not substitute for a supported CLI.

### F1 — High: provenance is not bound to loaded input bytes

`DataDir::load` and optional `load_verified` are separate (`dna-data/src/datadir.rs:73`); production model `from_data_dir` paths use unchecked loading. `dna-sim/src/run.rs::run` accepts an independent manifest and constructs provenance after simulation. An external/modified table set can be labeled with an unrelated manifest. This is a source-verified API gap, not evidence that committed tables are corrupt. Use one immutable verified model/data bundle that supplies its own identity; explicitly mark synthetic/unverified test bundles.

### F2 — High: accepted table shapes can silently change the science

`DnaFile::n_components` exists but `load_from` does not enforce it (`dna-data/src/catalog.rs:166,305`). `BornExcitation::new` checks energy coverage; total cross section sums all components while final-state selection reads five (`dna-physics/src/models/born_excitation.rs:43,62`). An accepted sixth component changes rates without a matching final state. Inspect the same pattern in ionisation and overflow in scaled values/sums (`dna-data/src/xs.rs:107`). Static counterexample, not a default-data failure observed here.

### F3 — Medium: fallible public APIs can panic or accept invalid domains

`AngularCdfTable::from_blocks` and `IonisationCdfTable::from_blocks` index columns before validating layout (`dna-data/src/blocks.rs:190,285`). `bracket_energy:128` shifts an endpoint by a relative epsilon then subtracts from a partition index; the accepted grid `[1.0, 1.0000000000001]` exposes underflow/out-of-bounds. `transport_electron` does not validate finite primary position or finite/nonnegative ledger tolerance (`transport.rs:388,642`); NaN tolerance can bypass a comparison. Add focused regression/property tests and checked constructors, not an unrelated numerical rewrite. These paths were statically identified, not executed as reproducers.

### F4 — High: deficient reference evidence can be falsely accepted

`geant4_reference.rs::compare_cross_sections` returns no failures on an empty grid and does not reject all non-finite/negative values; duplicate keys can overwrite earlier rows (`:261,296,439`). `slowing_down_reference.rs:519` has similar issues; minimum row count is not exact coverage. M6's prechem reader is stronger, but its zero-standard-error path (`physchem_reference.rs:172`) needs exact equality when both estimates are deterministic. Make consumers fail closed on malformed/missing evidence before trusting new campaigns.

### F5 — Medium: CI does not enforce completed scientific gates

`.github/workflows/ci.yml` runs normal tests on pinned Ubuntu. Maintained reference tests are ignored. Some data-dependent tests can return successfully on missing files (`dna-physics/tests/common/mod.rs`). `statistical.yml` and `bench.yml` promised by docs/07 are absent. Add a named current-gate manifest/command, with explicit required-data failures and historical diagnostics kept separate. Do not enable all ignored tests indiscriminately.

### F6 — Medium: compiler/provenance promises are broader than tested support

`Cargo.toml:14` advertises Rust 1.85, but let chains in `dna-sim/src/config.rs:76` and `dna-physics/src/transport.rs:440` require a later compiler. Only pinned 1.97.0 was tested here. `dna-core/src/provenance.rs:14` lacks revision/dirty state, target/platform and lockfile identity required by `docs/reproducibility.md`. Package version 0.1.0 is not enough. Choose an honest compiler policy and record structured fidelity settings, not just counters.

### F7 — Medium: batching will amplify memory and serialize physchem

`run_physchem_summary_with_diagnostics` retains full physics-event batches, then converts histories serially (`dna-sim/src/run.rs:194–242`). Default batch size is 1,024; the config docs estimate roughly 100,000 events per 1 MeV history (`config.rs:30–54`). Multi-gigabyte payload risk is inferred, not measured. Run all stages inside each independent-history worker and reduce compact ordered summaries. This does not require parallelizing interacting molecules.

### F8 — Architectural limitation: promised extension seams do not exist yet

`ProcessTable` contains fixed `ElectronModel` chains (`dna-physics/src/table.rs:116`), water density is fixed and `transport_electron` owns electron-specific secondary/solvation behavior. Docs/03's “new particle through registration only” claim is not true today. Preserve the electron implementation and add only proton/H state and dispatch when M5 is undertaken. `ElectronModel::inner` also returns `&dyn CrossSectionModel`; “no virtual calls” is not an established property or a measured bottleneck.

## 5. Geant4 fidelity and scientific validation

### M6: preserve stage-matched evidence and qualified closure

Old `physchem.csv` includes zero-time reaction effects; `physchem_prechem.csv` is the pre-IRT 1 ps boundary. They are not interchangeable. RR-001 compares seven species at 1, 10, 100 and 900 keV with its registered z <= 4.5, count-relative and RMS criteria. Historical post-fix result: 28/28 rows passed, maximum z about 2.04 (`docs/08-roadmap.md`, `5df0aa1`, closure session). The reference is one Geant4 seed with 20,000/2,000/200/20 histories. Additional Rust seeds are not independent Geant4 campaigns. Missing cross-species covariance and source-derived order were explicitly accepted limitations.

RR-002 subsequently became a spatial investigation. Its recorded five seeds times 80 histories per implementation yielded RMS ratio 0.981740457, interval [0.949564151, 1.015007983], relative half-width 3.333%; the 160-history expansion was not triggered. Precision achieved and the former discrepancy not reproduced do **not** imply equivalence. `.scratch/rr-002-80` is local, and the large spatial CSV is ignored. Preserve the conclusion and artifact identities without making future development depend on an untracked 20 GB fixture.

### M7: data completeness is not runtime completeness

`dna-chemistry/src/reactions.rs` loads 72 entries: 52 mobile-pair channels and 20 first-order/background channels (IDs 53–72). IDs 42–52 are eleven charged partially diffusion-controlled channels; `irt.rs::sample_reaction_time` rejects these. Computing parameters in `reaction_parameters.rs` does not execute them. Upstream `reactionType` 0/1 must not be confused with literature classes I–VI; a generic “Type V spin engine” is not a demonstrated requirement of this option3 profile.

Normative upstream sources are `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc` (`Sampling`, `MakeReaction`) and `source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc`. Older-partner timing, unary exponential clocks, conditional reactant/product placement and insertion order matter. Independently Brownian-diffusing every molecule to each heap time is not a faithful scheduler. Stable identities, invalidation, ties and observation semantics need explicit tests.

The upstream finite-horizon search cutoff is an approximation, not proof of zero distant-pair reaction probability. All-pairs is a small-system oracle, not viable production enumeration for roughly 100,000 molecules. Reaction 53's source anomaly must remain the sole specific conservation exception. Current V9 high-precision CDF/edge tests are valuable L1 evidence, but are not Geant4 pair, independent SBS, multispecies, V10 or G(t) validation.

### Statistics and model boundaries

M3's history includes post-comparison subnormal/low-Kish-count amendments (`210000c`). Preserve them and identify excluded bins as excluded, not passed. Pooled within-history spectra are correlated; borrowed Rust variance for absent Geant4 moments is weaker than independent per-history/batch statistics. New campaigns need their own uncertainty/covariance data and preregistration; old evidence must not be overwritten.

Exactly 1 MeV is a configuration boundary: stock Geant4 can include standard electron ionisation alongside DNA (`docs/02-cpp-reference-inventory.md`, upstream `G4EmDNABuilder`). A 900 keV result cannot silently replace the original 1 MeV G(t) case. The proposed roadmap retains 1 MeV in an explicit pure-DNA reference profile and does not claim stock-constructor parity at that endpoint.

No completed V6 ESTAR/PSTAR or V12 literature G(t) assessment was found. Geant4 agreement is L2/L3, not L4. Also, mean-of-history-ratios and ratio-of-total-yield/energy generally estimate different quantities when energy varies; more histories do not necessarily make them equal. Both require named semantics and separate tests.

## 6. Documentation and instruction improvements

README describes an M2-era workspace/reference state and omits implemented chemistry. `docs/README.md`, early docs/03 prose and parts of docs/08 lag later decisions. Add a current-state synopsis while retaining historical amendments. The missing pending `m7-sbs-oracle-and-ode` registry entry, stale M6 residual seeds, 1 MeV ambiguity and closure wording (“pass or documented failure”) are operational hazards, not merely prose issues.

`CLAUDE.md` currently contains only `@AGENTS.md`. Keep that delegation: it avoids a second conflicting policy. Put any approved shared improvements into `AGENTS.md`, not duplicated instructions. Candidate additions are these project-specific invariants:

- State explicitly that this is a native Rust port and Geant4 is read-only reference code, not runtime FFI.
- Preserve the rule: agents may write/compile-check generators but never run Geant4. Runs are user-owned and logged in `validation/geant4/RUN_REQUESTS.md`.
- Require verified loaded-data identity and named model/stage/time/geometry profiles in scientific outputs; forbid attaching an unrelated manifest.
- Separate implementation status, evidence availability and scientific gate verdict. A documented failure is not a passed milestone.
- List the maintained gate command/manifest once R01 exists; forbid blanket ignored-test execution and silent missing-data skips in required gates.
- Require preregistered statistics, explicit exclusions, immutable reference fixtures and new filenames for new campaigns.
- State independent-history parallelism, immutable observation schedules and same-platform thread/batch reproducibility; do not promise cross-toolchain bitwise equality.
- Require each seed to preserve source/version traceability and update the current capability synopsis; never infer completion from session status alone.
- Resolve machine roots from `DEV_DIR`/`G4_ROOT`; stop if unset. Keep machine configuration out of global instructions.

Do not add instructions to “match Geant4 at all costs,” loosen tolerances on failure, or automatically generalize every model. These conflict with the existing scientific priorities. This report does not create or edit `CLAUDE.md` or `AGENTS.md`.

## 7. Recommended decisions

Approve the roadmap proposal, not code changes: retain the five crates and M6 closure; harden evidence and runtime inputs; complete option3 execution; independently validate pair kernels; implement a deterministic serial-per-history scheduler; then G(t), CLI and the combined reference campaign. Preserve track-segment, V10, proton/H, independent physical assessment and profiling obligations with explicit later milestones.

Start **R01 `validation-evidence-and-ci`** after migration approval. Do not start the currently selected process-count session. Every existing unfinished seed and proposed replacement is accounted for in [roadmap-review.md](roadmap-review.md); executable seed contracts are in [proposed-roadmap.md](proposed-roadmap.md).
