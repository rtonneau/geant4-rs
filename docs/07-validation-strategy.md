# G. Validation strategy

Four levels (never conflated; reports state which level a claim has):
L1 mathematical/algorithmic unit tests; L2 numerical comparison with Geant4 algorithms (same inputs, deterministic quantities: cross sections, CDF tables); L3 statistical comparison with Geant4 simulations (distributions); L4 comparison with published/experimental data. **Agreement at L2/L3 does not establish L4.**

## G.1 Reference data generation
Offline, by a developer with Geant4 11.4.1 + G4EMLOW 8.8: small C++ apps in `validation/geant4/` (read-only use of the reference tree; built outside it) derived from examples `mfp`, `range`, `slowing`, `spower`, `dnaphysics`, `chem6`. Outputs: CSV + `provenance.toml` (Geant4 version/SHA, dataset version+MD5, physics constructor, seed, number of histories, date, command line) in `validation/reference/`. Size budget: < 1 MB per fixture, < 20 MB total. CI never builds Geant4.
Pre-registered acceptance criteria below are fixed **before** the first comparison and changed only via a documented commit.

## G.2 Tests and criteria
| ID | Quantity | Level | Criterion |
|---|---|---|---|
| V1 | Table loading, units, boundaries | L1 | exact table nodes returned to 1e-12 rel; out-of-range -> typed error |
| V2 | Total cross sections vs Geant4 (`G4VEmModel::CrossSectionPerVolume`) at nodes and 200 log-spaced off-node energies per model | L2 | nodes 1e-9 rel; off-node 1e-6 rel (same log-log law); any larger deviation is investigated, not tolerated |
| V3 | Interaction-type probabilities at 20 energies | L2 | 1e-9 rel |
| V4 | Sampled secondary energy / scattering angle distributions per model at 10 energies, 10^6 samples | L3 | two-sample KS and chi2 (equal-probability bins) p > 0.001 each; with 10 energies x 3 observables x models, expect false alarms at p<0.001 for ~1 in 1000 tests; use Bonferroni over the test family for the overall verdict |
| V5 | Mean free path, track length, CSDA-like range (electron 100 eV-1 MeV; proton within the model ranges only) | L3 | difference of means within 3 combined standard errors AND < 2 % absolute (relative systematic tolerance) |
| V6 | Stopping power / W-value | L3, L4 | L3 as V5; L4 vs ICRU/NIST ESTAR/PSTAR liquid water: report, no pass/fail until Geant4's own deviation (known) is tabulated |
| V7 | Energy conservation | L1 | per history |E_in - sum(parts)| < 1e-9 eV-relative; 10^4 histories at each energy |
| V8 | Physchem: species yield per event type, displacement RMS | L1, L3 | channel branching: chi2 p>0.001; displacement RMS within 3 sigma of nominal |
| V9 | Chemistry kinetics single reaction (A+B, A+A) in a box vs analytic Smoluchowski / Collins-Kimball | L1 | within 3 sigma of the analytic curve, N>=10^4 pairs |
| V10 | Well-mixed limit vs ODE | L1 | long-time agreement 1 % for a closed-system test case |
| V11 | G(t) for e_aq, OH, H, H2, H2O2 at {1 ps, 10 ps, 100 ps, 1 ns, 10 ns, 100 ns, 1 us}, e- 1 keV..1 MeV and p, comparing to Geant4 chem6 (IRT) run with identical reaction list | L3 | per species and time |G_rs - G_g4| < 3 sqrt(s_rs^2 + s_g4^2) at every point and Bonferroni-corrected family p > 0.01; additionally relative systematic < 5 % at t >= 1 ns for N_hist chosen so statistical error < 2 % |
| V12 | G(t) vs published (e.g. J. Appl. Phys. 125 (2019) 104301 data cited by chem6; Elliot/Bartels e_aq data) | L4 | report with literature uncertainty; no pass/fail threshold set a priori beyond "documented" |
| V13 | Thread-count / seed reproducibility | L1 | identical results for 1, 2, 8 threads (same platform); different seeds -> statistically compatible |
Statistical details: batch means (>=30 batches) for correlated scores; Geant4 reference errors from its own batch means; independent-seed repeat of each reference run to estimate its run-to-run scatter; report both z and effect size. No result is called "scientifically valid" from L2/L3 alone.

**M2 implementation notes.** V7 and V13 run in `cargo test` (ledger: 10^4 histories at five energies, about one minute with `[profile.test] opt-level = 3`). V2-like and V4 run against the raw data files and an independent Python re-implementation (`scripts/hand_cross_sections.py`), not against Geant4. V2/V3/V5 against Geant4 are in `crates/dna-sim/tests/geant4_reference.rs`, `#[ignore]`d until `validation/reference/*.csv` and `provenance.toml` exist (`validation/geant4/README.md`); their criteria are stated in that file's header. Performance baseline: `crates/dna-sim/tests/baseline.rs` (`--ignored`).

**M3 implementation notes.** V7 now runs on the whole cascade (normal run 10^4/10^4/2000/200/20 histories at 100 eV, 1, 10, 100 keV, 1 MeV, about 1 min; the `--ignored` full run with up to 10^4 histories and 200 at 1 MeV takes 5 min and was executed once: max residual 5.7e-16 ... 2.5e-13). V13 additionally pins 1/2/8-thread bitwise equality of spectra, radial profile and the physicochemical-event stream. The new cross sections are checked V2-style against `scripts/hand_cross_sections.py` (Sanche, Melton, `r_mean`), V4-style against the raw tables (Sanche levels, Melton absorption fraction, solvation displacement; families and alphas in `crates/dna-physics/tests/low_energy.rs`). Physical sanity (not Geant4): `crates/dna-sim/tests/physical_sanity.rs`, thresholds written in docs/08 before the numbers were seen. Geant4 comparison of the cascade: `crates/dna-sim/tests/slowing_down_reference.rs` (ignored until `validation/reference/{xs_lowenergy,slow_sums,slow_spectra,slow_radial}.csv` and `provenance_m3.toml` exist); criteria in its header: V2 1e-9 / 1e-6, V5 family with z < 4 (77 comparisons, family-wise false-alarm 0.5 %; docs/07's 3 sigma is for a single quantity) and the 2 % relative cut applied only where the combined standard error is below 0.7 %, two-sample chi-square for the spectra (alpha 1e-3/28), radial cumulative fractions with z < 4.

## G.3 Regression fixtures
(A *fixture* is a small stored input or expected-output file that a test compares against, e.g. a CSV of Geant4 reference values or a saved result of a fixed-seed run. Not the same as the `data/` cross-section tables, which are simulation inputs.)
Deterministic tiny runs (fixed seed, 10^3 histories) store summary scalars to 1e-12 relative in `tests/fixtures/`; intentionally regenerated via `cargo xtask`-like script with a changelog entry whenever model or RNG consumption changes.

## G.4 CI (GitHub Actions)
- `ci.yml` (every PR): `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace` (includes doctests and small deterministic regressions, < 5 min).
- `statistical.yml` (manual + nightly): V4, V5, V9, V11 at reduced statistics against stored fixtures.
- `bench.yml` (manual): criterion benches; results as artifacts, never gating.
- Data: committed subset in `data/dna/` (see `10-data-requirements.md`), checksum-verified by a test; CI therefore runs real-table tests too.
