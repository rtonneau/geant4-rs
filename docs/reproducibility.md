# Reproducibility

## Random numbers (`g4-core::rng`)
- Generator: `rand_chacha::ChaCha12Rng`. The 64-bit `master_seed` is expanded to a 256-bit key with splitmix64 (tested against Vigna's reference outputs).
- **One ChaCha stream per `(history_id, stage)`**: stream number `(history_id << 2) | stage`, stages `Physics = 0`, `PhysChem = 1`, `Chemistry = 2`, `3` reserved. `history_id <= 2^62 - 1`, otherwise `HistoryIdOutOfRange`.
- A history's numbers depend only on seed, id and stage: not on thread, scheduling, creation order or other histories. There is no shared mutable RNG.
- Uniform variates in (0,1) use the top 52 bits, `(k + 0.5)/2^52`: neither endpoint is returned, so `ln(u)` is always finite. (53 bits would round to 1.0 at the top.)

## What is promised

The `g4-sim` default `chemistry` feature adds coupled physicochemistry APIs.
Disabling default features retains physical histories and raw handoff events;
it does not change the physical RNG stream, batch order or output format.
The same-platform exact physics snapshot comparison covers both profiles.
Default physicochemistry is also compared with the immutable pre-S01 capture;
see [crate separation evidence](crate-separation.md) for commands, inputs,
revisions and executed test counts. These comparisons establish regression
identity within the recorded workload, not broader scientific validity.

- Same seed + same history ids => identical per-history results regardless of thread count (tested bitwise in `tests/thread_independence.rs`).
- Scoring reductions are done in history order (fixed-order fold), so sums are also thread-count independent (docs/05 E.3).
- **Bitwise** reproducibility is promised only for the same platform, toolchain (`rust-toolchain.toml`), `Cargo.lock` and data (manifest hash). Floating-point library functions may differ across platforms.
- Changing a physics model changes the number of draws and therefore all downstream numbers of every history: results stay statistically equivalent, not bitwise comparable.
- `golden_values_pin_generator_choice` pins the first words of a stream so that a `rand`/`rand_chacha` upgrade that changes the sequence is noticed (it forces regeneration of every fixed-seed regression value).

## Provenance recorded with outputs (`g4-core::provenance`)

The envelope records the crate version, the compiler's `rustc -V` output, Cargo's actual target triple and target components, the Geant4 reference version (11.4.1), dataset name/version, SHA-256 of the supplied manifest, master seed, and structured energy/length/time units. It also records stable numerical and model policy identifiers. `policy_lock_sha256` hashes length-prefixed lockfile and policy identifiers; changing either changes that fingerprint.

The build script captures the Git `HEAD` revision and `git status --porcelain` dirty state separately. A clean state is `false` only when that status command succeeds. Failed Git queries produce `null` fields, not a guessed revision or a clean claim. It hashes the bytes of the workspace `Cargo.lock`; a missing or unreadable lockfile produces `cargo_lock.sha256: null`. Compiler or target discovery failures likewise produce `null`. These are build-time observations, so the envelope cannot prove source state after the build or include uncommitted source bytes. A dirty build has a known base revision but its patch is not identified by that revision.

Production `run`, `run_with_sink` and `run_with_physchem` require a `VerifiedProcessTable` and emit `geant4-rs/dna-sim/run/v4`. Their provenance is read-only through `provenance()`. Construct a `VerifiedDataSet::load(&dir, &manifest)` first, then `VerifiedProcessTable::option2_electron(&snapshot, rotation)`. Every manifest file is captured and checked before models are built; parsing uses those exact bytes. The resulting table owns its data and identity, so neither editing the source files nor supplying another manifest can relabel a run. `table.table()` exposes a shared reference for lower-level histories and scorers.

`inputs.verification: "verified"` records the manifest-text hash plus every captured file's size and SHA-256 in manifest order. The separate `run_unverified_for_testing`, `run_unverified_with_sink_for_testing` and `run_unverified_with_physchem_for_testing` functions accept a raw `ProcessTable` without a manifest. They emit `geant4-rs/dna-sim/run-unverified/v1`, `inputs.verification: "unverified"`, a reason, and empty dataset/version/manifest identity strings. This escape hatch is for synthetic model tests. `ProcessTable::option2_electron_unverified` (and its compatibility alias `option2_electron`) cannot produce a verified output context. The file names are logical relative names; no machine data-root path enters the envelope or policy fingerprint. The chemistry field contains the constructed engine's semantic SHA-256 and, only for embedded option3, its source-file hash. A source-file hash alone does not identify custom chemistry.

This evidence supports bitwise comparison only under the same platform, target, compiler, lockfile, source state, verified inputs, model and numerical policy, seed and run configuration. Floating-point math libraries and target features can still differ even with a matching target triple, and a dirty or unavailable source/lock state cannot establish exact implementation identity. A matching hash establishes byte identity of the recorded input, not scientific validity.

## Data integrity
`Manifest::verify` re-checks size and SHA-256 of every committed data file; `g4-core/tests/committed_data.rs` runs it on `data/` in CI. The in-crate SHA-256 wrapper uses the approved `sha2` dependency and is tested against the FIPS 180-4 / NIST vectors. The build script uses the same dependency for `Cargo.lock` bytes.

The embedded chemistry files use exact Geant4 11.4.1 configuration keys. The 15 keys in the C++ extraction fixture are compared directly with `Species::chemistry_name()`; `°OH` and `HO2°` remain distinct from legacy ASCII input spellings `OH` and `HO2`. `H2O(B)`, `H3Op(B)`, `OHm(B)`, and `NoneM` are typed reaction-table bulk participants outside the `Species` set. The 72 option3 reactions are independently checked against the C++ source extraction and a hand-authored evaluator. Bimolecular literals are stored in `M^-1 s^-1`, matching the C++ `1e-3 m3/(mol s)` multiplier. The v4 `provenance.chemistry.source_file_sha256` field replaces the old unconditional `chemistry_data_sha256` field. For `PhysChemEngine::option3()` it fingerprints the four embedded chemistry files in species/channels/parameters/reactions order; its current value is `46a49f0cde41062cd4c55fb93e7817f481c68918e3ad6c810686db9dda4cbbde`. `provenance.chemistry.semantic_sha256` instead hashes the actual validated engine values (species, ordered channels, physical parameters and recombination switch). A custom `PhysChemEngine::new` has a null source-file hash, even when semantically equivalent to option3. An embedded engine with recombination disabled retains its source-file hash but changes its semantic hash. No full chemical reaction stage is run by this two-stage output API.

## R02 default-input regression evidence

The pre-R02 revision `baed05953d3cdf3612bd29025c63d5469993612f` was compared with the verified-input implementation on the same Windows toolchain and pinned external dependencies. The workload used seed 20261026, history IDs 3 through 10, and energies 100, 1000 and 10000 eV. All 24 detailed physical histories, secondaries, raw handoff CSV, physical summaries, species, physicochemical statistics and summaries matched at the exact floating-point bit level. Only the intentionally changed run schema and provenance fields were excluded. Scratch probes also matched the RNG draw counts and next stream values: physical totals 8287/91083/860895 and physicochemical totals 462/4530/43707 in energy order. This establishes compatibility for that fixed workload, not cross-platform bitwise identity or new physical validity.

The permanent output regressions additionally cover relocation, edits after byte capture, rejection of changed bytes under the old manifest, a new identity after manifest updates, and custom chemistry without an embedded-source claim. The seven maintained Geant4 gates retain their original fixtures and acceptance criteria; `--tier required` checks their evidence inventory, while `--tier statistical` executes the scientific comparisons.

## Model domains and retained limitations

Option2 accepts primary electrons in liquid water from 2 eV through 1 MeV. Solvation handles cascade electrons at or below 7.4 eV, including secondaries below the primary floor. Cross-section windows are Champion 7.4 eV to 1 MeV, Born excitation 9 eV to 1 MeV, Born1 ionisation 11 eV to 1 MeV, Sanche 2 to 100 eV and Melton 4 to 13 eV. The constructors require total and final-state tables to cover each source-backed window; they reject truncated coverage instead of changing a model limit. Invalid table domains, non-finite cross sections/positions and non-positive ledger tolerances are typed errors. This deliberately stricter input policy does not alter accepted inputs' interpolation, sampling or RNG draws.

The input checks do not resolve existing physical limitations: the K-shell binding energy remains local deposit without Auger/fluorescence (Q12), Champion uses Geant4's non-unit-axis convention by default with `Exact` available (Q13), and coupled output stops after physicochemistry at 1 ps without diffusion or IRT. See [the retained limitation register](09-open-questions-and-risks.md) and [C++ source inventory](02-cpp-reference-inventory.md).

## Statistical test acceptance (fixed before running)
- KS uniformity: N = 1e5, `D < 1.95/sqrt(N)` (alpha ~ 0.001); fixed seed, so the test is deterministic.
- Mean of uniforms within 5 sigma of 0.5.

## Draw order of the M2 electron physics (stream `(seed, history, Physics)`)
Per step: 1 uniform for the free path (`-ln u / Sigma_tot`), 1 for the process, then the model draws: Champion 2 (`cos theta`, `phi`); Born excitation 1 (level); Born1 ionisation 1 per shell draw (repeated for shell 2 below 19 eV), 1 for the energy transfer, 1 or 2 or 0 for the Born polar angle (`W < 50 eV`, 50-200 eV, > 200 eV) and 1 for `phi`. Any change of this order or of a model changes every later number of every history (statistically equivalent, not bitwise comparable); `g4-sim` tests pin thread-count independence, not values, so a deliberate change needs no fixture update, but note it here. Geant4 draws one number of interaction lengths per process instead of one total free path; both are the same Poisson process, so only distributions, never individual histories, can be compared with Geant4.

## Draw order of the M3 physics (one stream `(seed, history, Physics)` for the whole cascade)
A history runs the primary and **all its secondaries on the same stream**. Tracks are processed from a LIFO stack: the primary is followed to its end, secondaries are pushed in order of production, and after a track ends the *last pushed* secondary is popped and followed (Geant4's urgent stack, docs/02 B.7.5 item 4). The sequence of draws is therefore: the whole life of the primary (all its steps, including the steps in which it ejected secondaries), then the whole life of the last secondary it ejected (together with everything *that* secondary ejected, depth first), and so on back to the first.
Per step of a track whose energy is above 7.4 eV: 1 uniform for the free path, 1 for the process, then the model draws:
- elastic (Champion): 2 (`cos theta`, `phi`);
- Born excitation: 1 (level); Sanche vibrational excitation: 1 (level);
- Born1 ionisation: 1 per shell draw (repeated for shell 2 below 19 eV), 1 for the energy transfer, 1 or 2 or 0 for the Born polar angle (`W < 50 eV`, 50-200 eV, > 200 eV) and 1 for `phi`;
- Melton attachment: 0.
A track whose energy is at or below 7.4 eV (including a secondary created below that energy) is solvated *without* the two step draws: 4 uniforms (two Box-Muller pairs: `x, y` from the first, `z` from the second; the second variate of the second pair is discarded) or 2 uniforms (isotropic direction, `cos theta` then `phi`) if the energy is at or below 0.1 eV (`r_mean = 0`). Box-Muller is stateless (no cached second value), so the number of draws never depends on earlier calls.
Consequences: a change of any model, of the process order in `ProcessKind::ALL` (elastic, excitation, ionisation, vibrational, attachment) or of the stack discipline changes every later number of every history (statistically equivalent, not bitwise comparable). Thread-count independence is unaffected because the whole cascade of a history is on one stream and is run by one thread (`g4-sim/tests/determinism.rs` pins 1/2/8-thread bitwise equality of every per-history field, of the spectra, of the radial profile and of the physicochemical events). With `transport_secondaries = false` (M2 comparison mode) secondaries are recorded, not followed, and the stream is the one of the primary alone.
Geant4 draws one number of interaction lengths per process and a different Gaussian generator: only distributions, never individual histories, can be compared with Geant4.

## Draw order of the physicochemical stage (stream `(seed, history, PhysChem)`, `g4-chemistry`)
One stream per history. Solvation events come first and draw nothing. The excitation / ionisation / attachment events are then processed in passes that follow Geant4's zero-time steps (`g4-chemistry` engine module docs). Within a pass the order is descending decay-state index, then reverse event order within a state. A pass holds every water molecule at first, and later only the ionised molecules retested after a failed recombination. Per molecule and pass:
1. ionised molecule only: **1 uniform** for the recombination test, drawn *in every pass in which it is pending* (also when no electron is in range or the process is switched off); a retested molecule draws nothing else in that pass;
2. if the molecule decays in this pass, **1 uniform** for the channel of the selected decay state (`H2Ovib` after a recombination, otherwise the state of the event);
3. the displacer of the channel, with a "triple" = 4 uniforms (two Box-Muller pairs, the fourth variate discarded):
   `A1B1`: triple(2.4); `B1A1` and attachment: triple(0.8) + one triple(1.1) per `OH` (the `OH-` of the attachment draws nothing); `B1A1_2`: one triple(1.6) per H;
   ionisation: 1 uniform (which product is displaced) + one triple(0.8) for the displaced product + triple(2.0) for the mother shift;
   auto-ionisation: the same + one triple with sigma_1D = 0.62666 r_mean(1.7 eV) for the `e_aq`; relaxation channels: nothing.
Consequences: adding a species or a channel, changing the processing order (docs/09 Q21) or the number of draws changes every later number of the history. Geant4 draws differently (`G4RandGauss`, two calls of its recombination sampler); only distributions can be compared.

M6 B does not add random draws. Physics histories are computed in parallel; their event lists are then converted with the per-history `PhysChem` stream and reduced sequentially in ascending history id, batch by batch. `run_physchem_summary` is therefore bitwise independent of thread count and batch scheduling. Production attribution, displacement sums and same-decay pair distances are accumulated while the already-selected channel products are emitted and consume no RNG values.

## R05 serial history reproducibility

`g4-chemistry::history::run_history` executes the complete compiled option3
network on an immutable 1 ps snapshot, using a caller-owned chemistry RNG.
The 72-channel network includes 52 pair and 20 unary channels. Invalid source
signatures and unsupported PDC rates fail during compilation, before sampling;
history horizons outside 1 ps through 1 us fail before any RNG draw. No
unsupported channel is silently dropped or replaced with another kinetic law.

Initial candidate pairs are sampled in canonical molecule-ID order, with
matching channels in source-ID order, followed by unary clocks in molecule-ID
and source-ID order. Event ties use insertion order. Each event consumes its
parents once and creates monotonic product IDs in source product order. Each
new product then samples pairs against older live identities and its unary
channels. Temporary diffusion of older partners, IRT sampling and conditional
placement consume the same per-history stream. Invalidated events, count
observations and trace recording consume no draws. Search policy, cutoff,
input creation order or channel changes can change subsequent draws; bitwise
identity is not promised between different search policies.

`g4-chemistry/tests/history_network.rs` executes 64 synthetic mixed-species
histories with seed 20261106006, two molecules of each of the 15 species and
the full unchanged network. It reconstructs every observed population from
the lineage, checking parent consumption and source-ordered product identities.
The workload exercises TDC/PDC, neutral/attractive/repulsive pairs, unary events
and subsequent reactions of products. This complements the separate per-channel
execution and atom/charge accounting tests in `history_reactions.rs`.

`g4-sim/tests/history_reproducibility.rs` is a **test-only** Rayon wrapper.
Its fixed seed is 20261106016, with history IDs 37 through 100, 30 synthetic
molecules per history, and observations at 1, 10, 100, 1000, 100000 and 1000000 ps.
For each of indexed search and all-pairs search independently, it compares
every returned count, diagnostic and lineage field, all floating-point bits,
and 16 subsequent open-uniform values from each stage stream. Pools of 1, 2
and 8 threads, batch sizes 1, 7, 64 and 128, and forward/reverse history
submission produce identical results after ordering by history ID. Chemistry
leaves the Physics and PhysChem stream states unchanged. Synthetic positions
use the PhysChem stream only to provide reproducible varied inputs; this is
not a coupled irradiation workload.

The wrapper is feature-gated, and `cargo test -p g4-sim --no-default-features`
retains the physics-only build. Production orchestration still stops at
physicochemistry; integrating the serial history engine is R06. These exact
same-platform tests establish deterministic execution for the specified
workloads, not cutoff convergence, R05/G2 acceptance or many-body physical
accuracy. The source baseline remains Geant4 **11.4.1**:
`G4EmDNAChemistry_option3.cc:146-664` and `G4DNAIRT.cc:161-281,393-508`,
read from that version's tag. G1-F's 11.4.3 pair evidence does not validate the
scheduler or change its baseline. The documented history ordering, inclusive
horizon and unary-fallback deviations remain in force.

R05-C1 now provides a separate [cutoff convergence report](validation/irt-history-convergence.md).
Its protocol was committed before sampling at `c7188b35`. It compares all-pairs
with indexed multipliers 1/2/4 on 65,536 clustered and 65,536 dilute synthetic
inputs (seed 20261106007), retaining 64 paired batches per ensemble, all 15
species and six observations through 1 us. Each policy restarts the same
history's chemistry stream; covariance is retained even though omitted
candidates change subsequent draw assignments. A test-only Rayon batch wrapper
preserves serial histories and ordered output. The report records input/code
hashes, full reproduction command and measured candidate/event/payload counts.
The compact batch statistics, all endpoint results and provenance are preserved
under `validation/irt_history/evidence/r05_c1/`. A fresh checkout can verify
their hashes and independently recompute every interval/verdict with
`python validation/irt_history/audit_r05_c1.py` (Python 3.10+, standard library,
no simulation or scratch files required). This audit returning zero means the
recorded evidence is intact, not that its scientific verdict passed.

The result is **INCONCLUSIVE: 687 of 1,080 endpoints pass, 393 are unresolved**.
Of these, 114 intervals cross an acceptance boundary and 279 have no observed
paired differences, which the committed protocol does not treat as statistical
proof. No sampled reference mean fell below 0.1 molecule/history; the rare-count
branch is tested deterministically but lacks campaign evidence. **G2 and R06
production integration remain blocked.** Multiplier 1 remains an unaccepted
candidate default. The explicit ignored statistical gate exits nonzero for
this result; ordinary workspace test success is not a convergence pass.

## M7 isolated diffusion and two-body IRT
Integrated histories own one `Stage::Chemistry` stream per history, created from the existing `RngStreams`; there is no shared mutable RNG. The IRT and diffusion primitives accept a caller-provided RNG and do not select streams themselves. Free diffusion consumes four open-(0,1) uniforms (two Box-Muller pairs) when both `D` and `dt` are positive; zero `D` or zero `dt` returns the original position bitwise and consumes none. IRT draw counts depend on the outcome branch: strict TDC overlap is immediate without a draw; otherwise TDC consumes one draw, using it both for the ultimate-probability decision and, when successful, the inverse-erfc time. Neutral PDC draws for contact or ultimate probability, then proposal/acceptance pairs up to 10,000 proposals; exact equality uses its continuum-limit draws after the ultimate-probability gate. Open uniforms deliberately omit Geant4's theoretical endpoints; invalid arithmetic and convergence exhaustion are typed errors, and outcomes are explicit (`Immediate`, `After`, `NoReaction`) rather than overloaded sentinel times (ADR 0003).

The pre-registered V9 workload uses seed 20261023006, 100,000 pairs per each of three option3 regimes, fixed history IDs `reaction_id*N + pair_index`, and an independent 80-digit analytic oracle. The 18 CDF/survival points and three no-reaction masses use absolute 3-sigma binomial bounds fixed before execution; correlated curve points are one family and the pointwise bound is not simultaneous coverage. The thread reproducibility workload uses seed 20261023106 and 1,024 histories, compares every raw float bit and serialized byte for Rayon pools of 1, 2 and 8 threads, and passes. This verified claim covers only that fixed isolated-pair-plus-diffusion workload; it does not cover a many-species scheduler or broader platform reproducibility.

The SBS regression uses independent master seed 20261023102 and 100,000 H+H pairs. Survival includes every pair not reacted by an observation boundary. The six tau values, analytic CDF, IRT seed and original combined pointwise three-sigma rule are fixed in `tests/sbs_validation.rs`; source fidelity now requires tau 0.25 to retain its observed excess residual while the other five points remain within their bounds. Exact observation-cap equality invokes the SBS bridge, whereas exact initial contact remains inclusive in SBS and strict overlap in IRT.

The V10 test uses master seed 2026102303, history IDs 0 through 511, 10,000 molecules of each reactant, 1 micromolar, and 360 us. Its explicit `N_A`/litre conversion determines the Gillespie per-pair propensity. The LSODA comparison is reproducible from public `rsoda` commit `ae76eee69d6d494be9253eaee819c77ee962c934`, pinned as a dev dependency and in `Cargo.lock`; solver errors propagate as test failures. This pin and the fixed Rust toolchain/lockfile cover the local oracle only. RR-003's external Geant4 run remains awaiting clean committed data and provenance for `m7-sbs-close-gate`.
