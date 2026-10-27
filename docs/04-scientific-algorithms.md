# D. Scientific algorithms

## D.1 Physical stage (track structure)
Loop per particle (electron, proton/H; secondaries pushed on a LIFO stack owned by the history):
1. Total macroscopic cross section Sigma(E) = n * sum_i sigma_i(E) over processes whose model range contains E (log-log interpolation of tabulated sigma, data in eV/units given by file; scale factor from `G4DNACrossSectionDataSet`).
2. Free path s = -ln(u)/Sigma(E) (DNA models are discrete with no continuous loss, so Sigma is evaluated at the pre-step energy [U G8: confirm against `G4VEmProcess` integral-approach settings used by DNA processes]). Infinite medium: no geometry boundary in M2-M8; a finite box is a later extension.
3. Select process with probability sigma_i/Sigma, then final state by model sampling:
   - elastic (Champion): scattering angle from tabulated cumulative differential table (`sigmadiff_cumulated_elastic_e_champion_hp.dat`), 2-D (energy, probability) interpolation (`QuadInterpolator`, log-log in energy).
   - excitation (Born): level chosen by partial cross sections; energy loss = level energy; no secondary.
   - ionisation (Born1): shell by partial cross section; secondary energy W from differential table; angles from `G4DNABornAngle`-style kinematics [U G1]; binding energy deposited locally (fate of binding energy is a documented-accounting item, Q5).
   - vibrational (Sanche, 2-100 eV), attachment (Melton, 4-13 eV): energy loss / absorption, absorbed electron becomes physicochemical event (dissociative attachment, displacer type `DissociativeAttachment`).
4. Below the thermalisation limit (7.4 eV opt2) the electron is handed to **solvation**: its kinetic energy is deposited and it produces a solvated electron at a displaced position (D.2).
5. **Termination (implemented in M3).** A track ends by solvation (at or below 7.4 eV; applied before the free-path draw because Geant4's cross section is `DBL_MAX` there, docs/02 B.7.5) or by Melton attachment; both deposit the whole kinetic energy locally. Sanche vibrational excitation (2-100 eV) and Melton (4-13 eV) therefore act between 7.4 eV and their upper windows, and the M2 guards (`BelowLowestWindow`, `NoInelasticChannel`, docs/09 Q14) are gone. Optional history terminations: a tracking cut on the primary (Geant4 comparison), a history-wide `max_interactions` (first-interaction mode), and a safety limit `max_steps` whose exceedance is an error (`StepLimitExceeded`), never a silent stop. `Sigma` is exact at the pre-step energy (DNA processes have no lambda tables or integral approach, docs/02 B.7.4 item 1).
   Secondary electrons are transported from a LIFO stack owned by the history (all on the history's stream, docs/reproducibility.md). Energy ledger per history, checked at its end against 1e-9 relative (observed <= 3e-13): `E_in = E_excitation + E_vibrational + E_binding + E_attachment + E_solvation + E_escaped (0, infinite medium) + E_untracked_secondaries + E_remaining_untracked` (the last two are zero when everything is transported and no cut is set).
**Proton:** same loop with charge state in {p, H}; processes: elastic (ion), excitation (Miller-Green < 500 keV, Born above), ionisation (Rudd <= 500 keV, Born), charge decrease/increase. Secondary electrons from ionisation: energy/angle from Rudd (`G4DNARuddAngle`) [U]. Proton delta-electron sampling is the single biggest physics-fidelity item (Q3).

**Beam energy vs model range.** Electrons: DNA models to 1 MeV. Policy: M2-M6 reject E0 > 1 MeV with `UnsupportedEnergy`. For 1-10 MeV electron primaries Q1 weighs: (a) condensed-history MollerBhabha+msc above 1 MeV then DNA below (what Geant4 does; hybrid; deposit scoring must treat the handover; a faithful port of Urban/GS msc is a large project); (b) extend the DNA Born electron models upward with a relativistic correction (*not* provided by Geant4 for water; would be an intentional, validated deviation); (c) only support protons <=300 MeV and electrons <=1 MeV, and score 1-10 MeV beams by superposing the electron spectra of Geant4-generated phase space. Recommendation: (c)-style scope first, (a) as a dedicated milestone M11 if required.

## D.2 Physicochemical stage
For each event (all [V] in `G4ChemDissociationChannels_option1.cc`, `G4DNAWaterDissociationDisplacer.cc`):
- Ionisation of shell k: H2O+ -> OH + H3O+ (channel prob 1) with the electron thermalised via solvation.
- Excitation: A1B1: 0.35 relaxation (no species), 0.65 dissociation (OH + H, displacer A1B1). B1A1: 0.175 relax, 0.0325 (2 OH + H2), 0.50 autoionisation (OH + H3O+ + e_aq), 0.2535 (H + OH), 0.039 (2H + O). Rydberg/diffuse layers 3rd/2nd/1st: autoionisation 0.5 / relax 0.5.
- Dissociative attachment, double/triple/quad ionisation: displacer types exist; only those reachable with option2 are implemented first.
- Displacement: Gaussian with fixed RMS per product (e.g. 2.4 nm, 0.8, 1.1, 1.6 nm; H3O+/OH 0.8 nm for ionisation split 50:50 between assignments) and mother-molecule recoil; thermalisation length of the sub-excitation electron from Meesungnoen 2002 (mean r as function of energy, then 3-D Gaussian) or Terrisol 1990 table depending on model chosen by `GetMacroDefinedModel` [U G6].
Rust (implemented, M6 A; details and C++ line numbers in docs/02 B.7.6): channels and species are TOML data (`data/chemistry/`) loaded and validated by `g4-chemistry` (probabilities sum to 1 within 1e-12, elemental balance, orbit mapping); the displacer is `displacements(kind, products, r_mean, rng)`; `PhysChemEngine::convert_history(events, rng)` returns the alive species of one history at 1 ps. Per history: (1) every solvation event makes an `e_aq` at its (already displaced) position; (2) the water-molecule events are processed in event order: an ionisation first tests the recombination with the nearest alive `e_aq` within 10 r_Onsager (`p = 1 - exp(-r_O/d)`, r_O = 0.7116 nm; success: the `e_aq` is removed and the `H2Ovib` channels replace `H3O+ + OH`), then one channel is selected and the products are placed at `event position + mother shift + product displacement`. Tests: chi-square of the channel selection, z-tests of the mean-square displacement against analytic values, exact conservation of H, O and charge on real cascades.

M6 B scoring uses the physical ledger denominator only. `G_s = 100 N_s/E_dep` is reported both as `sum(100 N_s)/sum(E_dep)` and `mean(100 N_s/E_dep)`. Alive totals are distinguished from per-event-type production totals, since recombination removes an electron produced earlier. Product displacement is relative to the parent event; radial RMS is relative to the primary origin; OH--e_aq and OH--H3O+ correlations mean same-decay pair distances. Rust values describe the 1 ps boundary before chemical reactions. R03's executable reaction descriptors do not move this boundary: no reaction is scheduled during M6 conversion.

### D.2.1 Conversion audit and recombination ordering study (2026-10-16)

> **Note (2026-10-17):** the tables in this subsection were measured on the engine *before* commit `5df0aa1`, which changed `PhysChemEngine::convert_history` to run recombination in Geant4-style zero-time passes (retest of failed ions, Geant4 DoIt order). The recombination fractions and yields below describe that older sequential algorithm and are kept as the record of the ordering sensitivity; the current engine gives, for example, 25.86 % at 1 keV. See docs/08 "M6 residual audit (final record)" and docs/09 Q21/R14.

All 23 channels, physical-state mapping, probabilities, ordered products, released energies, zero lifetimes and displacement selections agree with Geant4 11.4.1; the complete source-backed audit is docs/02 B.7.8. No conversion value or default behavior changed. In particular H2Ovib relaxation has zero energy, unlike excitation relaxation. Exhaustive branch execution and the existing C++ fixture cross-check pass; a two-hole/two-electron test demonstrates one versus two recombinations solely from reversing the competing holes.

Run `cargo test -p g4-sim --test physchem_reference m6_recombination_ordering_ablation -- --ignored --nocapture` to reproduce the real-cascade ablation. Each energy uses histories starting at 0, the existing reference statistics, option2 physics, default Geant4 elastic rotation, seed 20261017, and no diffusion or chemical reactions. Each physical cascade is generated once; five conversions differ only in water-event order, keeping solvated-electron insertion order fixed. Reverse flips all water events; shuffle uses a Fisher-Yates permutation on a separate `(seed 20261018, history, PhysChem)` stream; excitation-first / ionisation-first use stable partitions. Off retains event order and disables recombination. Each conversion restarts `(seed 20261017, history, PhysChem)`; permutation draws cannot advance the simulation stream. This pairs histories and starting streams, **not individual event variates**: changed branches consume different numbers of draws. The diagnostic reports paired recombination-count differences and standard errors; it does not select a physically preferred ordering or impose a new acceptance tolerance.

Recombination fraction below is `100 sum(recombinations) / sum(ionisation events)` (percent, not a mean of per-history ratios):

| Energy; histories | Event | Reverse | Shuffle | Excitation first | Ionisation first | Off |
|---|---:|---:|---:|---:|---:|---:|
| 1 keV; 20000 | 24.4957 | 25.4300 | 25.0890 | 25.0199 | 24.1294 | 0 |
| 10 keV; 2000 | 21.0016 | 21.7900 | 21.4742 | 21.4887 | 20.6775 | 0 |
| 100 keV; 200 | 19.4499 | 20.2541 | 19.9231 | 19.9914 | 19.2507 | 0 |
| 900 keV; 20 | 19.0158 | 19.8429 | 19.5049 | 19.5798 | 18.7887 | 0 |

Paired reverse-minus-event recombinations per history are 0.47855 +/- 0.02121, 3.8515 +/- 0.1992, 38.625 +/- 1.864, and 354.65 +/- 15.89 (one standard error) at those energies. These are sensitivity measurements on Rust's sequential algorithm, not corrections or a bound on all possible schedules. The normal test suite also verifies bitwise identical forward and reverse conversions on 1, 2 and 8 threads.

Alive means at 10 keV illustrate the R14 yield effect (per history):

| Order | e_aq | OH | H3Op | H | H2 | Oxy | OHm |
|---|---:|---:|---:|---:|---:|---:|---:|
| Event | 400.0000 | 497.6120 | 407.6785 | 85.8750 | 23.1500 | 16.7815 | 8.6785 |
| Reverse | 395.9295 | 496.2025 | 403.6080 | 88.5510 | 23.7605 | 17.3995 | 8.6785 |
| Shuffle | 397.6285 | 497.0625 | 405.3070 | 87.1910 | 23.6265 | 17.0050 | 8.6785 |
| Excitation first | 397.4505 | 497.0100 | 405.1290 | 87.5155 | 23.6585 | 17.1365 | 8.6785 |
| Ionisation first | 401.5320 | 498.5675 | 409.2105 | 84.7635 | 23.1520 | 16.5160 | 8.6785 |
| Off | 502.3895 | 535.5920 | 510.0680 | 17.1175 | 9.3235 | 0.7810 | 8.6785 |

Candidate percentages in the command's output count holes with a reachable alive electron; off skips this query, so its zero is not a measured absence of candidates. The full comparison report was rerun for 1, 10, 100 and 900 keV and reproduces the existing values in docs/08. The report completes, while the unchanged agreement gate remains failing (e.g. 1 keV H2O2 z = 111.557 in magnitude): reordering cannot create chemical-reaction products absent from every dissociation channel. Geant4's cached-candidate interaction-length/DoIt scheduling is a separate unresolved approximation (docs/02 B.7.8); retain Q21/R14 until a matched physicochemical reference tests it. No Geant4 generator was run and no reference data was changed.

## D.3 Chemical stage
**M7 implemented slice (isolated pairs only).** Free diffusion in an unbounded 3-D volume adds independent `N(0, 2 D dt)` displacements per axis; zero diffusivity or zero time returns the original position without RNG draws. Reaction parameters preserve source literals and derive relative diffusivity `D_A + D_B`, observed-rate conversion, Onsager length, contact/effective radii and (for PDC) diffusion and activation rates/contact probability. For TDC the effective radius is `k_obs/(4 pi D_rate)`; A+A uses `D_A` for this rate conversion while pair motion uses `2 D_A`. PDC uses summed physical radii and halves `k_diff` for A+A. Signed electrostatic distance mapping is `r_eff = r_c / expm1(r_c/r)`; neutral reactions reduce to `r_eff=r`.

The IRT kernel follows Geant4 11.4.1 `G4DNAIRT::GetIndependentReactionTime` and `SamplePDC`, not IRT-syn. TDC treats strict overlap (`sigma > r`) as `Immediate`; otherwise it samples the source inverse-erfc time law conditional on ultimate reaction probability `W_inf = sigma/r_eff`, returning `NoReaction` when the open uniform lies outside that probability. PDC first applies contact probability and then samples the source rejection law for an ultimate event; the charged branch uses the source's signed Onsager terms. Results are `Immediate`, `After(elapsed positive Time)`, or `NoReaction`; elapsed time is relative to the pair's current configuration and does not update molecule time or positions. Pair identities must differ, species must match the parameterized reaction, positions and arithmetic must be finite, and zero total diffusivity, invalid rates/radii, or unsupported bulk/order return typed errors. PDC rejection exhaustion after 10,000 rejected proposals is a convergence error.

Normative Geant4 11.4.1 source paths (relative to its source root): `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc:286-405` (TDC/PDC sampling); `source/processes/electromagnetic/dna/utils/src/G4DNAMolecularReactionTable.cc:127-151,296-335` (effective radius and reaction type); `source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc` (option3 reaction setup); and `source/processes/electromagnetic/dna/models/src/G4DNABrownianTransportation.cc` (Brownian motion). These Rust kernels intentionally make outcomes explicit, return typed invalid-domain/convergence errors, and use open-(0,1) uniforms that exclude theoretical endpoints. The charged TDC mapping reproduces Geant4's effective-distance prescription; it is not claimed as a general exact electrostatic first-passage solution. ADR 0003 records the model and outcome policy.

V9 validates three registered cases (neutral A+A TDC, neutral A+B PDC, charged A+B TDC), six dimensionless times each, with 100,000 independent pairs per case and 18 CDF/survival points plus three no-reaction masses. The analytic oracle is independently evaluated at 80 decimal digits; each point uses the pre-registered absolute 3-sigma binomial bound. See `crates/g4-chemistry/tests/two_body_validation.rs` and docs/08. This validates the isolated kernel only.

The validated M7 input is `ReactionTable::option3()`: 72 source-order reactions with explicit mobile or bulk participants, ordered products, observed-rate order and literal, source class, kinetic intent, and the `SetReactionType` call flag. R03 compiles every entry to an execution descriptor: 52 pair channels (TDC IDs 1–10, PDC IDs 11–52) and 20 unary channels (IDs 53–72). This supports isolated pair outcomes, independent unary clocks and conditional source-literal placement, but does not schedule a correlated many-species history. Pair rate literals are `M^-1 s^-1`; first- and pseudo-first-order observations retain `s^-1` rather than being converted to bimolecular units. The full ID/source/branch/product matrix and balance audit are in [option3-reaction-coverage.md](validation/option3-reaction-coverage.md).

The isolated kernel requires species diffusion coefficients and reaction radii/rates; these are available from embedded option3 tables and validated `TwoBodyReactionParameters`. The R03 unary sampler uses the source's independent exponential hazard; zero-rate custom unary channels remain disabled. The R03 placement primitive follows `G4DNAIRT::MakeReaction` at `G4DNAIRT.cc:393-495`, including its asymmetric line-433 Gaussian-width expression in native Geant4 units. That expression has dimensions of length squared although passed as a standard deviation; it is a source-fidelity limitation, not a validated physical Brownian width. Unrepresentable placement arithmetic returns a typed error. R05 will own bulk-reservoir scheduling, competition and end-time policy in an unbounded spatial domain; the current primitives do not schedule competing reactions.
**Recommended first method: IRT** (Geant4's default for option3/chem6; independent-pair assumption; cost scales with number of close pairs, far faster than SBS).
Planned R05 serial IRT history (after `G4DNAIRT`): index nearby species in spatial cells, sample pair times with the validated R03 pair kernels and unary clocks independently, then process events in time order. On a reaction, consume the parents and create ordered products with the R03 `G4DNAIRT::MakeReaction` placement primitive; invalidate events involving changed participants and sample the products' new candidate reactions without resampling unaffected pairs. Materialize survivor positions lazily with Gaussian diffusion when needed. R05 uses a finite observation horizon and reports cutoff sensitivity; R06 owns the log-spaced G(t) output grid.
**Also implement SBS** (small, in `chemistry::sbs`) as a *validation oracle*, not default: it is the reference method for totally diffusion-controlled systems (reaction type 0 only, as the chem6 README states), and exposes IRT approximation error. **Synchronised IRT (IRT-syn)** and mesoscopic methods are deferred (UHDR prototype; Q6).

**Implemented SBS oracle.** For a separated pair it uses
`dt = (r-R)^2 / [8(D_A+D_B+2 sqrt(D_A D_B))]`; for H+H the denominator is
`32 D_H`. Both molecules diffuse independently for the smaller of that adaptive
step and the remaining observation interval. Endpoint contact is inclusive
(`r <= R`). Only an observation-capped step applies the Brownian-bridge test,
including exact equality between the remaining interval and adaptive step. The
bridge deliberately preserves the executable `FindReaction` coordinate
double-count quirk; the nearby source comment claiming a factor 16 is stale.
Production IRT instead uses strict initial overlap (`R > r`), so exact contact
is a documented measure-zero distinction. The fixed-seed 100,000-pair H+H
comparison preserves the source-faithful early discrepancy: at tau 0.25 the
IRT/SBS CDF residual exceeds the combined pointwise three-sigma bound, while
the other five registered points satisfy it.

**V10 well-mixed oracle.** The closed test system enables only option3 reaction
ID 2, `e_aq + H -> OHm + H2`, with terminal products, 10,000 molecules of each
reactant at 1 micromolar, and `t_end = 360 us`. Its exact Gillespie propensity
is `a = k n_e n_H/(N_A V)`, with `V = N/(c N_A)` and the validated
`k = 2.50e10 M^-1 s^-1`. Across 512 fixed-seed histories every trajectory
checks reactant/product conservation. The four matching mass-action equations
are integrated with LSODA and all ensemble means must agree within 1%; the
observed maximum relative error is 0.0824%. Both implementations are test-only.
Interchangeability: `trait ChemistryEngine { fn run(&self, species: &[SpeciesInstance], t_end: Time, observers: &mut [&mut dyn ChemObserver], rng: &mut Rng) -> ... }`.
Homogeneous ODE (well-mixed) network: implemented only in tests (`tests/kinetics_ode.rs`) to check rate bookkeeping in the long-time limit and Fricke-like yields; it is never a substitute.

## D.4 Source dependencies for validation tasks
Physical: examples `range`, `mfp`, `slowing`, `spower`, `wvalue`, `dnaphysics`. Physchem: `chem6` with chemistry disabled after t0 (initial species + positions). Chemical: `chem6` G(t); `chem1-5` single reactions/Fricke; `scavenger`.
