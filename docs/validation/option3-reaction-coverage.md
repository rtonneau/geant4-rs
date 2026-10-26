# Option3 reaction execution coverage (Geant4 11.4.1)

This matrix was reviewed against the executable constructor calls in `source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc:96-666`. Each line is the constructor call that adds the reaction data, as recorded by the independent C++ extraction fixture `crates/g4-chemistry/tests/fixtures/cpp_reactions.json`. The Rust embedded table is `data/chemistry/reactions_option3.toml`; `reaction_coverage.rs` checks every compiled descriptor against this matrix, including ordered reactants/products and source line. Empty product cells mean no mobile products.

The execution branches are in `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc`: **IRT type 0** = `GetIndependentReactionTime:298-310` (TDC); **IRT type 1** = `GetIndependentReactionTime:311-339` and `SamplePDC:360-390` (PDC, with charged `rc != 0` parameters at 322-329); **Sampling unary** = `Sampling:250-281` (first-/pseudo-first-order clock and fake zero-diffusion bulk partner). `MakeReaction:393-495` places the ordered products for all branches. “Charged electrostatics” means both reactants carry nonzero charge, so the source takes the `rc != 0` branch. A singly charged reactant with a neutral partner has `rc == 0`.

| ID | Constructor line | Regime | G4DNAIRT branch | Ordered reactants | Ordered mobile products |
|---:|---:|---|---|---|---|
| 1 | 146 | TDC pair | IRT type 0 | H, H | H2 |
| 2 | 152 | TDC pair | IRT type 0 | e_aq, H | OHm, H2 |
| 3 | 159 | TDC pair | IRT type 0 | H, Oxy | °OH |
| 4 | 165 | TDC pair | IRT type 0 | H, Om | OHm |
| 5 | 171 | TDC pair | IRT type 0 | °OH, Oxy | HO2° |
| 6 | 177 | TDC pair | IRT type 0 | HO2°, Oxy | O2, °OH |
| 7 | 184 | TDC pair | IRT type 0 | Oxy, Oxy | O2 |
| 8 | 192 | TDC pair | IRT type 0 | e_aq, e_aq | OHm, OHm, H2 |
| 9 | 200 | TDC pair | IRT type 0 | H3Op, OHm |  |
| 10 | 205 | TDC pair | IRT type 0 | H3Op, O3m | °OH, O2 |
| 11 | 215 | PDC pair, neutral electrostatics | IRT type 1 | °OH, H |  |
| 12 | 221 | PDC pair, neutral electrostatics | IRT type 1 | H, H2O2 | °OH |
| 13 | 228 | PDC pair, neutral electrostatics | IRT type 1 | H, OHm | e_aq |
| 14 | 235 | PDC pair, neutral electrostatics | IRT type 1 | H, O2 | HO2° |
| 15 | 242 | PDC pair, neutral electrostatics | IRT type 1 | H, HO2° | H2O2 |
| 16 | 249 | PDC pair, neutral electrostatics | IRT type 1 | H, O2m | HO2m |
| 17 | 256 | PDC pair, neutral electrostatics | IRT type 1 | °OH, °OH | H2O2 |
| 18 | 263 | PDC pair, neutral electrostatics | IRT type 1 | °OH, H2O2 | HO2° |
| 19 | 270 | PDC pair, neutral electrostatics | IRT type 1 | °OH, H2 | H |
| 20 | 277 | PDC pair, neutral electrostatics | IRT type 1 | e_aq, °OH | OHm |
| 21 | 284 | PDC pair, neutral electrostatics | IRT type 1 | °OH, OHm | Om |
| 22 | 291 | PDC pair, neutral electrostatics | IRT type 1 | °OH, HO2° | O2 |
| 23 | 298 | PDC pair, neutral electrostatics | IRT type 1 | °OH, O2m | O2, OHm |
| 24 | 306 | PDC pair, neutral electrostatics | IRT type 1 | °OH, HO2m | HO2°, OHm |
| 25 | 314 | PDC pair, neutral electrostatics | IRT type 1 | °OH, Om | HO2m |
| 26 | 321 | PDC pair, neutral electrostatics | IRT type 1 | °OH, O3m | O2m, HO2° |
| 27 | 329 | PDC pair, neutral electrostatics | IRT type 1 | e_aq, H2O2 | OHm, °OH |
| 28 | 337 | PDC pair, neutral electrostatics | IRT type 1 | H2O2, OHm | HO2m |
| 29 | 344 | PDC pair, neutral electrostatics | IRT type 1 | H2O2, Oxy | HO2°, °OH |
| 30 | 352 | PDC pair, neutral electrostatics | IRT type 1 | H2O2, Om | HO2°, OHm |
| 31 | 360 | PDC pair, neutral electrostatics | IRT type 1 | H2, Oxy | H, °OH |
| 32 | 368 | PDC pair, neutral electrostatics | IRT type 1 | H2, Om | H, OHm |
| 33 | 376 | PDC pair, neutral electrostatics | IRT type 1 | e_aq, O2 | O2m |
| 34 | 383 | PDC pair, neutral electrostatics | IRT type 1 | e_aq, HO2° | HO2m |
| 35 | 390 | PDC pair, neutral electrostatics | IRT type 1 | OHm, HO2° | O2m |
| 36 | 397 | PDC pair, neutral electrostatics | IRT type 1 | OHm, Oxy | HO2m |
| 37 | 404 | PDC pair, neutral electrostatics | IRT type 1 | O2, Oxy | O3 |
| 38 | 411 | PDC pair, neutral electrostatics | IRT type 1 | O2, Om | O3m |
| 39 | 418 | PDC pair, neutral electrostatics | IRT type 1 | HO2°, HO2° | H2O2, O2 |
| 40 | 426 | PDC pair, neutral electrostatics | IRT type 1 | HO2°, O2m | HO2m, O2 |
| 41 | 434 | PDC pair, neutral electrostatics | IRT type 1 | HO2m, Oxy | O2m, °OH |
| 42 | 444 | PDC pair, charged electrostatics | IRT type 1 | e_aq, H3Op | H |
| 43 | 451 | PDC pair, charged electrostatics | IRT type 1 | e_aq, O2m | H2O2, OHm, OHm |
| 44 | 460 | PDC pair, charged electrostatics | IRT type 1 | e_aq, HO2m | Om, OHm |
| 45 | 468 | PDC pair, charged electrostatics | IRT type 1 | e_aq, Om | OHm, OHm |
| 46 | 476 | PDC pair, charged electrostatics | IRT type 1 | H3Op, O2m | HO2° |
| 47 | 483 | PDC pair, charged electrostatics | IRT type 1 | H3Op, HO2m | H2O2 |
| 48 | 490 | PDC pair, charged electrostatics | IRT type 1 | H3Op, Om | °OH |
| 49 | 497 | PDC pair, charged electrostatics | IRT type 1 | O2m, Om | O2, OHm, OHm |
| 50 | 506 | PDC pair, charged electrostatics | IRT type 1 | HO2m, Om | O2m, OHm |
| 51 | 514 | PDC pair, charged electrostatics | IRT type 1 | Om, Om | H2O2, OHm, OHm |
| 52 | 523 | PDC pair, charged electrostatics | IRT type 1 | Om, O3m | O2m, O2m |
| 53 | 534 | first-order unary | Sampling unary | O3m, NoneM | H3Op, O2m |
| 54 | 544 | pseudo-first-order unary | Sampling unary | HO2°, H2O(B) | H3Op, O2m |
| 55 | 551 | pseudo-first-order unary | Sampling unary | H, H2O(B) | e_aq, H3Op |
| 56 | 558 | pseudo-first-order unary | Sampling unary | e_aq, H2O(B) | H, OHm |
| 57 | 565 | pseudo-first-order unary | Sampling unary | O2m, H2O(B) | HO2°, OHm |
| 58 | 572 | pseudo-first-order unary | Sampling unary | HO2m, H2O(B) | H2O2, OHm |
| 59 | 579 | pseudo-first-order unary | Sampling unary | Oxy, H2O(B) | °OH, °OH |
| 60 | 586 | pseudo-first-order unary | Sampling unary | Om, H2O(B) | °OH, OHm |
| 61 | 593 | pseudo-first-order unary | Sampling unary | e_aq, H3Op(B) | H |
| 62 | 599 | pseudo-first-order unary | Sampling unary | O2m, H3Op(B) | HO2° |
| 63 | 605 | pseudo-first-order unary | Sampling unary | OHm, H3Op(B) |  |
| 64 | 612 | pseudo-first-order unary | Sampling unary | H3Op, OHm(B) |  |
| 65 | 617 | pseudo-first-order unary | Sampling unary | HO2m, H3Op(B) | H2O2 |
| 66 | 623 | pseudo-first-order unary | Sampling unary | Om, H3Op(B) | °OH |
| 67 | 629 | pseudo-first-order unary | Sampling unary | O3m, H3Op(B) | °OH, O2 |
| 68 | 636 | pseudo-first-order unary | Sampling unary | H, OHm(B) | e_aq |
| 69 | 642 | pseudo-first-order unary | Sampling unary | °OH, OHm(B) | Om |
| 70 | 648 | pseudo-first-order unary | Sampling unary | H2O2, OHm(B) | HO2m |
| 71 | 654 | pseudo-first-order unary | Sampling unary | HO2°, OHm(B) | O2m |
| 72 | 660 | pseudo-first-order unary | Sampling unary | Oxy, OHm(B) | HO2m |

## Balance and scope

The conservation test counts H, O, and electric charge for mobile configurations, explicit `H2O(B)`, `H3Op(B)`, `OHm(B)` bulk exchanges, and signed implicit water (`+n` on the product side, `-n` on the reactant side). `NoneM` contributes zero. Only ID 53, exactly `O3m + NoneM -> H3Op + O2m`, is the existing Geant4 source anomaly and is asserted to be unbalanced; the exception is never generalized. No atom, charge, or water term is silently discarded.

R03 compiles all 72 channels and provides isolated pair outcomes, independent unary clocks, and conditional literal source placement. It does **not** evolve a correlated many-species history, select among competing reactions, or supply spatial boundaries. `MakeReaction` uses an asymmetric expression at line 433 as a Gaussian standard deviation although it has dimensions of length squared. Rust preserves the numeric literal in Geant4 native units rather than silently replacing it with a physically derived Brownian width. Arithmetic that cannot represent a placement returns a typed error.

Pair rates in the constructor are bimolecular `M^-1 s^-1` (the Geant4 expression multiplies the literal by `1e-3 m^3 mol^-1 s^-1`); unary observed rates are `s^-1`. The supported chemistry is Geant4 option3 for liquid water at its source-defined constants. This work does not extend the physical transport energy range: the existing option2 electron input policy ends at 1 MeV, and chemistry starts from the post-physicochemical species list at global 1 ps. No proton or arbitrary chemical environment agreement is established here.

Numerical policy: validated finite positive pair parameters and source identities are required; zero custom unary rates are explicit disabled descriptors. Pair/unary samplers use open-interval uniforms, typed domain/convergence errors, and a 10,000-rejection PDC cap; these differ intentionally from C++ negative-time sentinels. Unit, deterministic anchor, endpoint and statistical tests cover the primitives; the full workspace format/Clippy/test gate is required. These are source-fidelity and internal coverage results, not independent acceptance of charged pair distributions or a history-level yield. R04 owns independent analytic/SBS/Geant4 pair gates and must pass G1 before R05 builds and validates a serial correlated history.

## Verification result (2026-10-26)

All 72 IDs compiled to exactly one descriptor (52 pair, 20 unary), and all 72 matrix rows matched source lines, regimes, branches and ordered reactants/products. H/O/charge accounting passed for 71 balanced channels using the shared species atom mapping; the exact ID-53 source signature was the sole asserted exception. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` passed, with existing evidence-dependent ignored tests still ignored. The independent R04 pair gate and R05 correlated-history gate have no verdict from this coverage test.
