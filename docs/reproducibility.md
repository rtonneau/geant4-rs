# Reproducibility

## Random numbers (`dna-core::rng`)
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

## Provenance recorded with outputs (`dna-core::provenance`)
crate version, `rustc -V`, Geant4 reference version (11.4.1), dataset name/version, SHA-256 of `data/MANIFEST.toml`, master seed, internal units. Lockfile hash is added with the output writer (M8).

## Data integrity
`Manifest::verify` re-checks size and SHA-256 of every committed data file; `dna-core/tests/committed_data.rs` runs it on `data/` in CI. SHA-256 is implemented in-crate (`sha2` is not an approved dependency) and tested against the FIPS 180-4 / NIST vectors.

## Statistical test acceptance (fixed before running)
- KS uniformity: N = 1e5, `D < 1.95/sqrt(N)` (alpha ~ 0.001); fixed seed, so the test is deterministic.
- Mean of uniforms within 5 sigma of 0.5.

## Draw order of the M2 electron physics (stream `(seed, history, Physics)`)
Per step: 1 uniform for the free path (`-ln u / Sigma_tot`), 1 for the process, then the model draws: Champion 2 (`cos theta`, `phi`); Born excitation 1 (level); Born1 ionisation 1 per shell draw (repeated for shell 2 below 19 eV), 1 for the energy transfer, 1 or 2 or 0 for the Born polar angle (`W < 50 eV`, 50-200 eV, > 200 eV) and 1 for `phi`. Any change of this order or of a model changes every later number of every history (statistically equivalent, not bitwise comparable); `dna-sim` tests pin thread-count independence, not values, so a deliberate change needs no fixture update, but note it here. Geant4 draws one number of interaction lengths per process instead of one total free path; both are the same Poisson process, so only distributions, never individual histories, can be compared with Geant4.

## Draw order of the M3 physics (one stream `(seed, history, Physics)` for the whole cascade)
A history runs the primary and **all its secondaries on the same stream**. Tracks are processed from a LIFO stack: the primary is followed to its end, secondaries are pushed in order of production, and after a track ends the *last pushed* secondary is popped and followed (Geant4's urgent stack, docs/02 B.7.5 item 4). The sequence of draws is therefore: the whole life of the primary (all its steps, including the steps in which it ejected secondaries), then the whole life of the last secondary it ejected (together with everything *that* secondary ejected, depth first), and so on back to the first.
Per step of a track whose energy is above 7.4 eV: 1 uniform for the free path, 1 for the process, then the model draws:
- elastic (Champion): 2 (`cos theta`, `phi`);
- Born excitation: 1 (level); Sanche vibrational excitation: 1 (level);
- Born1 ionisation: 1 per shell draw (repeated for shell 2 below 19 eV), 1 for the energy transfer, 1 or 2 or 0 for the Born polar angle (`W < 50 eV`, 50-200 eV, > 200 eV) and 1 for `phi`;
- Melton attachment: 0.
A track whose energy is at or below 7.4 eV (including a secondary created below that energy) is solvated *without* the two step draws: 4 uniforms (two Box-Muller pairs: `x, y` from the first, `z` from the second; the second variate of the second pair is discarded) or 2 uniforms (isotropic direction, `cos theta` then `phi`) if the energy is at or below 0.1 eV (`r_mean = 0`). Box-Muller is stateless (no cached second value), so the number of draws never depends on earlier calls.
Consequences: a change of any model, of the process order in `ProcessKind::ALL` (elastic, excitation, ionisation, vibrational, attachment) or of the stack discipline changes every later number of every history (statistically equivalent, not bitwise comparable). Thread-count independence is unaffected because the whole cascade of a history is on one stream and is run by one thread (`dna-sim/tests/determinism.rs` pins 1/2/8-thread bitwise equality of every per-history field, of the spectra, of the radial profile and of the physicochemical events). With `transport_secondaries = false` (M2 comparison mode) secondaries are recorded, not followed, and the stream is the one of the primary alone.
Geant4 draws one number of interaction lengths per process and a different Gaussian generator: only distributions, never individual histories, can be compared with Geant4.

