# E. Parallelisation strategy

## E.1 Unit of parallel work
**One primary history = one task**: physical transport of the primary and all its secondaries, then (default mode) physicochemistry and chemistry of that history's species. Histories share only immutable data (`Arc<PhysicsData>`, `Arc<ReactionNetwork>`). Per-history mutable state (secondary stack, species vectors, cell lists, ledger, scorer partial) lives in a thread-local `Workspace` reused across histories (`rayon::iter::ParallelIterator::for_each_init`) to avoid allocation.
Scientific caveat (must be documented in output metadata): per-history chemistry is the *isolated-track* approximation (no inter-track reactions). It is exact for G(t) of single primaries in chem6 style (Geant4 simulates one event at a time there [U confirm]) and invalid for high dose-rate/multi-track regimes (UHDR). Multi-track chemistry = a "chemistry batch" containing many primaries' species; see E.4.

## E.2 Random numbers
- `rand_chacha::ChaCha12Rng` (ChaCha8 is acceptable; choose 12 for margin, benchmark later), key from `master_seed` (u64 expanded with a documented splitmix64 to 256 bit), **stream = history_id** via `set_stream`, then disjoint sub-stream ranges per stage by word position or by stream id `(history_id << 2) | stage` (stage: 0 physics, 1 physchem, 2 chemistry, 3 reserved). Identity of a history is its integer id from the primary list, never a thread id. No shared RNG.
- Consequence: results are independent of thread count and scheduling provided scoring reduction is order-deterministic (E.3). Changing a model changes the number of draws and therefore all downstream numbers for that history; statistically equivalent, not bitwise comparable. Document in `docs/reproducibility.md` at M0; promise bitwise reproducibility only on the same platform/toolchain/lockfile and verify in CI on one OS.
- Tests: stream disjointness (no equal first N words across 10^4 ids), uniformity (KS), repeatability, thread-count independence.

## E.3 Scoring reductions
Each history returns a small `HistoryResult` (energy ledger, per-bin deposits, species counts at the output time grid, interaction counters). Reduction: `par_iter().with_min_len(k).map(...).collect::<Vec<_>>` into history order, then a **sequential fixed-order** fold (or fixed binary tree) into totals and into batch means. Cost O(N_hist * result size) memory; for 10^6 histories with ~30 time points x 10 species this is ~2.4 GB if kept raw, so batch: fixed-size batches (e.g. 1000 histories) are reduced in-order, giving determinism *and* batch-means statistics. Floating-point sums in fixed order are reproducible across thread counts.

## E.4 Chemistry parallelism (evaluated separately)
1. Default: across independent histories (E.1). Embarrassingly parallel, no races.
2. Within one large chemical system (many tracks): IRT cell lists allow spatial decomposition, but a reaction changes species set and re-samples times; naive parallel reaction processing alters the global time-ordering and thus probabilities. Safe design: *synchronised* time windows (IRT-syn idea): within a window, pairs in disjoint cell blocks are processed in parallel, conflicts (same species, cross-block) resolved by earliest-time rule at window barrier. Deferred to M11; requires equivalence test against the serial implementation using many seeds.
3. Never parallelise the event loop of a single IRT instance with shared mutable reaction lists.

## E.5 Expected serial bottlenecks
Data loading and table building (once; parallelisable per file), result reduction and I/O (small), pathological single histories (a 1 MeV electron has ~10^4-10^5 secondaries, heavy tail: use rayon work stealing with small chunks; avoid large static chunking), chemistry of very dense tracks (proton Bragg peak) where pair counts are quadratic within cell.

## E.6 Performance method
Baseline first (M7/M8 validated), then profile (`perf`, `cargo flamegraph`) on fixed workloads: 1 keV, 10 keV, 100 keV, 1 MeV e-; 1, 10 MeV p; chem at 1 ps-1 us. `criterion` benches live in `benches/`; scientific equivalence re-run (fixed seed regression + statistical test) after every optimisation. SoA/SIMD/`unsafe` only if a profile shows need.
