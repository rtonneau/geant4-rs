# F. Dependency plan

| Crate | Where | Use | Justified? |
|---|---|---|---|
| `rayon` | `g4-sim` | history parallelism, ordered collect | yes, core |
| `rand` | `g4-core` | `RngCore`/`Rng` traits, uniform sampling | yes. Use only trait + `Open01`; write inverse-CDF/table sampling ourselves; check `rand` major version API at M0 (0.9 renamed methods) |
| `rand_chacha` | `g4-core` | reproducible per-history streams (`set_stream`) | yes |
| `serde` (+derive) | `g4-core`, `g4-sim`, `g4-chemistry` | config, species/reaction tables, JSON metadata output | yes |
| `toml` | `g4-sim`, `g4-chemistry` | config and data tables | yes |
| `clap` (derive) | `dna-cli` | CLI | yes |
| `thiserror` | each lib crate | typed errors (`UnsupportedEnergy`, `MissingDataFile`, `InvalidConfig`) | yes |
| `approx` | dev | float comparisons | yes |
| `criterion` | dev, `benches/` | benchmarks | yes |
| `proptest` | dev | monotonicity/normalisation invariants of interpolation and CDF sampling, unit round trips | yes, scoped to these |
| `sha2` | `g4-core` | SHA-256 of committed data files and manifest hash (checksum verification) | **approved by user**; replaced the earlier hand-written implementation; RustCrypto, widely used |
| `rsoda` | `g4-chemistry` dev only | LSODA solution for the V10 well-mixed oracle | public HTTPS Git dependency pinned to commit `ae76eee69d6d494be9253eaee819c77ee962c934`; test graph only, because it is not published on crates.io |
| `tokio` | none | no async need | **not used** |
| `serde_json` | `g4-sim` | machine-readable output | **approved by user**: outputs need JSON; ubiquitous, stable, no std alternative |
Everything else in the production graph (data file parsing, interpolation, Vec3, statistics, erf/erfc) is std/own code. `erfc`: implement and test own (Numerical-Recipes-style rational approx is *not* accepted; use a documented high-accuracy algorithm, validated to 1e-14 against high-precision values), since Geant4 uses `G4ErrorFunction` (`D/utils/include/G4ErrorFunction.hh`) and agreement matters for IRT.
`Cargo.lock` committed; `rust-toolchain.toml` pins a stable version at M0; record `rustc -V` and lock hash in every output metadata block.
