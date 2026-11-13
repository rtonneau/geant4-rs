---
bump: minor
floor: minor
---
- In a finite volume, solvated electrons and dissociation products are placed with the Geant4 0.8 x safety clamp, so they no longer land outside the water.
- `Volume::safety` gives the Geant4 isotropic distance to the surface for box, tube and sphere solids.
- Chemistry runs in a volume report `physchem.boundary`: per-species counts of shortened placements and of species outside the volume at 1 ps (run schema v6).
- New `g4rs_ref finite_volume` generator mode and RR-009 request for the finite-volume Geant4 comparison.
