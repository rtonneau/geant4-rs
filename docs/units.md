# Units

Internal units (fixed, `dna-core::units`): **energy eV, length nm, time ps**. Geant4/CLHEP
uses MeV, mm, ns; nothing from the C++ side is copied numerically without conversion.

| Quantity | Type | Stored as | Constructors | Notes |
|---|---|---|---|---|
| Energy | `Energy` | eV | `from_ev/kev/mev` | |
| Length | `Length` | nm | `from_nm/um/mm` | |
| Time | `Time` | ps | `from_ps/ns/us/s` | |
| Area, cross section | `Area` | nm^2 | `from_nm2/cm2/m2` | 1 cm^2 = 1e14 nm^2; Geant4 cross sections are per molecule |
| Diffusion coefficient | `Diffusivity` | nm^2/ps | `from_m2_per_s` | 1 m^2/s = 1e6 nm^2/ps |
| Bimolecular rate constant | `RateConstant` | nm^3/ps | `from_per_molar_per_s` | 1 M^-1 s^-1 = 1.66054e-12 nm^3/ps, from exact N_A = 6.02214076e23 |

Rules:
- Same-dimension `+ - ` and scaling by `f64` are provided; ratio of two equal quantities is a bare `f64`. Cross-dimension products are deliberately absent until a use exists.
- `Vec3` is unit-agnostic (`f64`): positions are nm, directions are dimensionless.
- Configuration keys carry the unit in their name (`energy_mev = 1.0`); output carries the unit in the field name or header.
- Energy-range validity is enforced with `EnergyRange::check` (typed `UnsupportedEnergy` error); values are never clamped or extrapolated. Initial beam limit: electron primaries <= 1 MeV (decision Q1).
- G-value: species per 100 eV; mol/J = G x 1.0364e-7 (docs/01 A.4/A.5).

Cross sections and densities (M2): the tables hold cross sections **per molecule**; the models return them in nm^2 (`Area`). The macroscopic cross section in nm^-1 is `sigma x n` with `n = 33.42797593 nm^-3`, the number density of Geant4's `G4_WATER` as derived in `dna-physics::water` (docs/02 B.7.4 item 2). Transport lengths are nm, free paths `-ln(u)/Sigma` in nm, energies eV, positions nm, directions dimensionless. Secondary-spectrum bins are in eV (0.1 eV - 1 MeV, 10 bins per decade).
