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

## A.4 G-value convention (to be fixed in M0, proposal)
G(X, t) = N_X(t) / (E_dep / 100 eV), reported in species per 100 eV (and SI mol/J = 1.0364e-7 x G).
Geant4 reference: example `chem6` README defines G as "(Number of species X)/(100 eV of deposited energy)".
- `E_dep` = sum of all energy locally deposited in the scoring volume during the physical stage,
  **including** energy of electrons below the transport cut (solvation) and vibrational/attachment losses,
  **excluding** energy carried away by particles leaving the volume. [U] Geant4's chem6 scorer
  convention (what is summed; PrimaryKiller behaviour) must be read and reproduced or the deviation documented.
- Uncertainty: standard error across independent histories with batch means (>=30 batches), propagating
  the ratio N/E_dep by the delta method (both fluctuate and are correlated).

## A.5 Units
Internal: **energy eV, length nm, time ps**, `f64` newtypes (`Energy`, `Length`, `Time`, `Diffusivity`,
`RateConstant`). Geant4 uses mm, ns, MeV: do not mix. Conversion facts: 1 m^2/s = 1e6 nm^2/ps;
1 M^-1 s^-1 (= 1 L mol^-1 s^-1) = 1.66054e-12 nm^3/ps (derived: 1e-3 m^3/N_A, 1e24 nm^3 per 1e-3 m^3,
1e12 ps/s). I/O: eV/keV/MeV, nm/um, ps/ns/us explicit in TOML keys (`energy_mev = 1.0`).

## A.6 Model validity (must be enforced in code, never silently extrapolated)
See 02 section B.2. Every `CrossSectionModel` exposes `energy_range()`; sampling outside the range is a typed error.
Beam energy vs. model range: DNA electron models end at **1 MeV** in all options (G4EmDNAPhysics_option2/4/6 `emaxDNA`).
Above that Geant4 silently hands over to *standard* condensed-history EM (MollerBhabha + Urban/GS msc + Seltzer-Berger).
This is not track structure; 1-10 MeV beams are therefore a separate, explicit decision (open question Q1).
