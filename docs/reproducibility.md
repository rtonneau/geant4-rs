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
