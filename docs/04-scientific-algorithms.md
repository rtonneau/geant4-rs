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
5. Tracking termination: only via solvation/attachment (electrons) or reaching the lower proton limit [U G3]. Energy ledger per history: `E_primary = E_deposited_local + E_binding_local + E_escaped + E_remaining` asserted at every step in debug and at end of history in release (relative tolerance 1e-12).

**Proton:** same loop with charge state in {p, H}; processes: elastic (ion), excitation (Miller-Green < 500 keV, Born above), ionisation (Rudd <= 500 keV, Born), charge decrease/increase. Secondary electrons from ionisation: energy/angle from Rudd (`G4DNARuddAngle`) [U]. Proton delta-electron sampling is the single biggest physics-fidelity item (Q3).

**Beam energy vs model range.** Electrons: DNA models to 1 MeV. Policy: M2-M6 reject E0 > 1 MeV with `UnsupportedEnergy`. For 1-10 MeV electron primaries Q1 weighs: (a) condensed-history MollerBhabha+msc above 1 MeV then DNA below (what Geant4 does; hybrid; deposit scoring must treat the handover; a faithful port of Urban/GS msc is a large project); (b) extend the DNA Born electron models upward with a relativistic correction (*not* provided by Geant4 for water; would be an intentional, validated deviation); (c) only support protons <=300 MeV and electrons <=1 MeV, and score 1-10 MeV beams by superposing the electron spectra of Geant4-generated phase space. Recommendation: (c)-style scope first, (a) as a dedicated milestone M11 if required.

## D.2 Physicochemical stage
For each event (all [V] in `G4ChemDissociationChannels_option1.cc`, `G4DNAWaterDissociationDisplacer.cc`):
- Ionisation of shell k: H2O+ -> OH + H3O+ (channel prob 1) with the electron thermalised via solvation.
- Excitation: A1B1: 0.35 relaxation (no species), 0.65 dissociation (OH + H, displacer A1B1). B1A1: 0.175 relax, 0.0325 (2 OH + H2), 0.50 autoionisation (OH + H3O+ + e_aq), 0.2535 (H + OH), 0.039 (2H + O). Rydberg/diffuse layers 3rd/2nd/1st: autoionisation 0.5 / relax 0.5.
- Dissociative attachment, double/triple/quad ionisation: displacer types exist; only those reachable with option2 are implemented first.
- Displacement: Gaussian with fixed RMS per product (e.g. 2.4 nm, 0.8, 1.1, 1.6 nm; H3O+/OH 0.8 nm for ionisation split 50:50 between assignments) and mother-molecule recoil; thermalisation length of the sub-excitation electron from Meesungnoen 2002 (mean r as function of energy, then 3-D Gaussian) or Terrisol 1990 table depending on model chosen by `GetMacroDefinedModel` [U G6].
Rust: channels and RMS are TOML data (`data/chemistry/`), loaded and validated (probabilities sum to 1 within 1e-12); displacer is a pure function `(event, rng) -> Vec<SpeciesInstance>` with its own distribution tests (mean/variance of displacement vs RMS).

## D.3 Chemical stage
Common data: species table (D, charge, reaction radius), reaction list with rate k (M^-1 s^-1 -> nm^3/ps), type (0 totally diffusion-controlled / 1 partially), products; box boundary; end time.
**Recommended first method: IRT** (Geant4's default for option3/chem6; independent-pair assumption; cost scales with number of close pairs, far faster than SBS).
Algorithm (after `G4DNAIRT`): bin species into cells of size R_cutoff; for each reactant pair within cut-off sample an independent reaction time from the Green function of the (partially) diffusion-controlled problem (`G4DiffusionControlledReactionModel::GetTimeToEncounter` - Smoluchowski radius from rate and D, `kdif = 4 pi D R N_A`, erfc); process pair reactions in time order; on a reaction, remove reactants, create products at a position determined by [U G7 - check `MakeReaction`], then **resample** reaction times for the new species against others; unreacted partners are propagated to the current time by Gaussian jumps with sigma = sqrt(2 D dt) per axis (as in `G4DNAIRT::Sampling`), which is the only place diffusion is explicit. Time axis log-spaced output grid for G(t).
**Also implement SBS** (small, in `chemistry::sbs`) as a *validation oracle*, not default: it is the reference method for totally diffusion-controlled systems (reaction type 0 only, as the chem6 README states), and exposes IRT approximation error. **Synchronised IRT (IRT-syn)** and mesoscopic methods are deferred (UHDR prototype; Q6).
Interchangeability: `trait ChemistryEngine { fn run(&self, species: &[SpeciesInstance], t_end: Time, observers: &mut [&mut dyn ChemObserver], rng: &mut Rng) -> ... }`.
Homogeneous ODE (well-mixed) network: implemented only in tests (`tests/kinetics_ode.rs`) to check rate bookkeeping in the long-time limit and Fricke-like yields; it is never a substitute.
Reaction radius for type 1 and its relation to k: **derive from Geant4 exactly (G7) before coding**; do not re-derive Collins-Kimball parameters independently without validation.

## D.4 Source dependencies for validation tasks
Physical: examples `range`, `mfp`, `slowing`, `spower`, `wvalue`, `dnaphysics`. Physchem: `chem6` with chemistry disabled after t0 (initial species + positions). Chemical: `chem6` G(t); `chem1-5` single reactions/Fricke; `scavenger`.
