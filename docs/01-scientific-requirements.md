# A. Scientific requirements

## A.1 Scope
Radiation chemistry of **pure liquid water** (density 1 g/cm3, T = 298 K as assumed by
the Geant4-DNA chemistry; [U] confirm the temperature constant used by diffusion/rate data).
Incident particles: electrons and protons. Secondary electrons are always transported
explicitly. Excluded: DNA/cell geometry, biological damage, GUI, GPU, hadronic nuclear
reactions, photons (photon-induced electrons are not a goal; Geant4 option2 registers
Livermore photoelectric / Klein-Nishina for gamma, we do not).

## A.2 Stages and their separation
1. **Physical**: discrete track structure; produces interaction events
   `{process, model, shell/level, position, energy deposited, secondaries}`.
2. **Physicochemical**: converts each event into initial chemical species with positions
   (Geant4: `G4DNAChemistryManager`, `G4DNAWaterDissociationDisplacer`, dissociation channels).
3. **Chemical**: diffusion-reaction up to a chosen end time (Geant4: SBS, IRT, IRT-syn).

The interface between 1 and 2 is a serialisable list of `PhysChemEvent`; between 2 and 3 a list of
`ChemicalSpeciesInstance{species, position, t0}`. Each stage must be testable and runnable alone.

## A.3 Observables
- Physical: total/mean free path, stopping power, CSDA/track range, energy-deposition
  distributions, secondary spectra, interaction counts per process/model.
- Chemical: G(t) for e_aq, OH, H, H2, H2O2 (+ H3O+, OH-, HO2, O2-, etc. as the network defines)
  at logarithmic times 1 ps ... 1 us (extendable), with statistical uncertainties.

## A.4 Energy accounting and G-value convention (decision Q5, evidence from C++)

**What Geant4 `chem6` does** [V: `examples/extended/medical/dna/chem6/src/ScoreSpecies.cc` L110-118, L171-176; `PrimaryKiller.cc`]:
- `E_dep` of an event = sum of `aStep->GetTotalEnergyDeposit()` over all steps (of every particle, primary and secondary) in the scored volume, i.e. the energy each interaction *proposes as local deposit*.
- For each event and each time t: `G_event(X,t) = N_X(t) / (E_dep / eV) * 100`. The reported G is the **mean over events of the per-event ratio** (plus sum of squares for the error), not total molecules divided by total energy.
- `PrimaryKiller` kills any track leaving a box and stops the primary electron once it has lost a chosen energy (`eLossMin`) or fallen to a minimum energy. So an "event" is a *track segment* with a controlled energy loss, which is how G is tied to LET.

**What counts as deposited energy in the DNA models** [V, option2 electron models]:
| Interaction | Energy booked as local deposit |
|---|---|
| Excitation (Born) | the excitation energy (`G4DNABornExcitationModel1.cc:203`) |
| Ionisation (Born1) | the shell binding energy only (`...IonisationModel1.cc:581`); the rest goes to the secondary electron's kinetic energy, which is transported and deposited later |
| Vibrational excitation (Sanche) | the vibrational energy lost; absorbed electrons deposit their whole kinetic energy (`...Sanche...:255-261`) |
| Attachment (Melton) | the electron's whole kinetic energy (`...Attachment...:246`) |
| Elastic | nothing (energy unchanged, `...Champion...:310`) |
| Solvation (< 7.4 eV) | [U G1/G6: expected: the remaining kinetic energy] |
So a thermalised sub-excitation electron's energy **is** counted, and energy leaving the box (killed tracks) is **not**.

**Rust convention (proposed final, adopted unless you object):**
1. `E_dep` = same definition as Geant4: sum of all local deposits listed above, for all particles, in the scoring volume. Energy of particles that leave is tracked separately as `E_escaped`.
2. Ledger identity checked for every history: `E_primary = E_dep + E_escaped + E_remaining` (relative error < 1e-12). Binding energy is part of `E_dep` (as in Geant4); it is *not* converted into chemical energy.
3. Report **two** estimators, both always in the output: (a) Geant4-compatible mean of per-event ratios; (b) ratio of totals `sum N / sum E_dep`, with its uncertainty by the delta method (batch means). They agree for large samples; (a) is slightly biased for few events, so (b) is the primary scientific number and (a) is used for Geant4 comparison.
4. Track-segment mode (needed to compare with `chem6` at a given LET) is a scoring/termination option (`primary_energy_loss_mev`, `min_kinetic_energy`, box size), reproducing `PrimaryKiller` semantics; the plain full-slowing-down mode is the default.
5. Units: species per 100 eV; mol/J = G x 1.0364e-7.
Remaining check [U]: confirm solvation deposit and `eLossMin`/`kineticE_Min` kill semantics at M0/M2; any difference is documented here as a deviation.

## A.5 Units
Internal: **energy eV, length nm, time ps**, `f64` newtypes (`Energy`, `Length`, `Time`, `Diffusivity`,
`RateConstant`). Geant4 uses mm, ns, MeV: do not mix. Conversion facts: 1 m^2/s = 1e6 nm^2/ps;
1 M^-1 s^-1 (= 1 L mol^-1 s^-1) = 1.66054e-12 nm^3/ps (derived: 1e-3 m^3/N_A, 1e24 nm^3 per 1e-3 m^3,
1e12 ps/s). I/O: eV/keV/MeV, nm/um, ps/ns/us explicit in TOML keys (`energy_mev = 1.0`).

## A.6 Model validity (must be enforced in code, never silently extrapolated)
See 02 section B.2. Every `CrossSectionModel` exposes `energy_range()`; sampling outside the range is a typed error.
Beam energy vs. model range: DNA electron models end at **1 MeV** in all options (G4EmDNAPhysics_option2/4/6 `emaxDNA`).
Above that Geant4 silently hands over to *standard* condensed-history EM (MollerBhabha + Urban/GS msc + Seltzer-Berger).
This is not track structure. **Decision Q1: the initial implementation is limited to primary energies <= 1 MeV** (electrons and the DNA electron models); larger energies return an `UnsupportedEnergy` error. 1-10 MeV beams are a possible later project.
