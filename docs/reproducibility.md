# Reproducibility

## Random numbers (`g4-core::rng`)
- Generator: `rand_chacha::ChaCha12Rng`. The 64-bit `master_seed` is expanded to a 256-bit key with splitmix64 (tested against Vigna's reference outputs).
- **One ChaCha stream per `(history_id, stage)`**: stream number `(history_id << 2) | stage`, stages `Physics = 0`, `PhysChem = 1`, `Chemistry = 2`, `3` reserved. `history_id <= 2^62 - 1`, otherwise `HistoryIdOutOfRange`.
- A history's numbers depend only on seed, id and stage: not on thread, scheduling, creation order or other histories. There is no shared mutable RNG.
- Uniform variates in (0,1) use the top 52 bits, `(k + 0.5)/2^52`: neither endpoint is returned, so `ln(u)` is always finite. (53 bits would round to 1.0 at the top.)

## What is promised
- Same seed + same history ids => identical per-history results regardless of thread count (tested bitwise in `tests/thread_independence.rs`).
- Scoring reductions are done in history order (fixed-order fold), so sums are also thread-count independent (docs/05 E.3).
- **Bitwise** reproducibility is promised only for the same platform, toolchain (`rust-toolchain.toml`), `Cargo.lock` and data (manifest hash). Floating-point library functions may differ across platforms.
- Changing a physics model changes the number of draws and therefore all downstream numbers of every history: results stay statistically equivalent, not bitwise comparable.
- `golden_values_pin_generator_choice` pins the first words of a stream so that a `rand`/`rand_chacha` upgrade that changes the sequence is noticed (it forces regeneration of every fixed-seed regression value).

## Provenance recorded with outputs (`g4-core::provenance`)
crate version, `rustc -V`, Geant4 reference version (11.4.1), dataset name/version, SHA-256 of `data/MANIFEST.toml`, master seed, internal units. Lockfile hash is added with the output writer (M8).

## Data integrity
`Manifest::verify` re-checks size and SHA-256 of every committed data file; `g4-core/tests/committed_data.rs` runs it on `data/` in CI. SHA-256 is implemented in-crate (`sha2` is not an approved dependency) and tested against the FIPS 180-4 / NIST vectors.

The embedded chemistry files use exact Geant4 11.4.1 configuration keys. The 15 keys in the C++ extraction fixture are compared directly with `Species::chemistry_name()`; `°OH` and `HO2°` remain distinct from legacy ASCII input spellings `OH` and `HO2`. `H2O(B)`, `H3Op(B)`, `OHm(B)`, and `NoneM` are typed reaction-table bulk participants outside the `Species` set. The 72 option3 reactions are independently checked against the C++ source extraction and a hand-authored evaluator. Bimolecular literals are stored in `M^-1 s^-1`, matching the C++ `1e-3 m3/(mol s)` multiplier. The `chemistry_data_sha256` output field fingerprints the four embedded chemistry files in species/channels/parameters/reactions order; its current value is `46a49f0cde41062cd4c55fb93e7817f481c68918e3ad6c810686db9dda4cbbde`.

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

## M7 isolated diffusion and two-body IRT
Integrated histories own one `Stage::Chemistry` stream per history, created from the existing `RngStreams`; there is no shared mutable RNG. The IRT and diffusion primitives accept a caller-provided RNG and do not select streams themselves. Free diffusion consumes four open-(0,1) uniforms (two Box-Muller pairs) when both `D` and `dt` are positive; zero `D` or zero `dt` returns the original position bitwise and consumes none. IRT draw counts depend on the outcome branch: strict TDC overlap is immediate without a draw; otherwise TDC consumes one draw, using it both for the ultimate-probability decision and, when successful, the inverse-erfc time. Neutral PDC draws for contact or ultimate probability, then proposal/acceptance pairs up to 10,000 proposals; exact equality uses its continuum-limit draws after the ultimate-probability gate. Open uniforms deliberately omit Geant4's theoretical endpoints; invalid arithmetic and convergence exhaustion are typed errors, and outcomes are explicit (`Immediate`, `After`, `NoReaction`) rather than overloaded sentinel times (ADR 0003).

The pre-registered V9 workload uses seed 20261023006, 100,000 pairs per each of three option3 regimes, fixed history IDs `reaction_id*N + pair_index`, and an independent 80-digit analytic oracle. The 18 CDF/survival points and three no-reaction masses use absolute 3-sigma binomial bounds fixed before execution; correlated curve points are one family and the pointwise bound is not simultaneous coverage. The thread reproducibility workload uses seed 20261023106 and 1,024 histories, compares every raw float bit and serialized byte for Rayon pools of 1, 2 and 8 threads, and passes. This verified claim covers only that fixed isolated-pair-plus-diffusion workload; it does not cover a many-species scheduler or broader platform reproducibility.

The SBS regression uses independent master seed 20261023102 and 100,000 H+H pairs. Survival includes every pair not reacted by an observation boundary. The six tau values, analytic CDF, IRT seed and original combined pointwise three-sigma rule are fixed in `tests/sbs_validation.rs`; source fidelity now requires tau 0.25 to retain its observed excess residual while the other five points remain within their bounds. Exact observation-cap equality invokes the SBS bridge, whereas exact initial contact remains inclusive in SBS and strict overlap in IRT.

The V10 test uses master seed 2026102303, history IDs 0 through 511, 10,000 molecules of each reactant, 1 micromolar, and 360 us. Its explicit `N_A`/litre conversion determines the Gillespie per-pair propensity. The LSODA comparison is reproducible from public `rsoda` commit `ae76eee69d6d494be9253eaee819c77ee962c934`, pinned as a dev dependency and in `Cargo.lock`; solver errors propagate as test failures. This pin and the fixed Rust toolchain/lockfile cover the local oracle only. RR-003's external Geant4 run remains awaiting clean committed data and provenance for `m7-sbs-close-gate`.
