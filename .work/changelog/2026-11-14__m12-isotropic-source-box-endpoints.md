---
bump: minor
floor: minor
---
- Run configs accept `direction = "isotropic"`: one isotropic primary direction per history, drawn from a dedicated per-history RNG stream; fixed-direction runs are unchanged.
- The RR-009 finite-volume gate is closed: the six box endpoints pass against Geant4 with the isotropic source.
