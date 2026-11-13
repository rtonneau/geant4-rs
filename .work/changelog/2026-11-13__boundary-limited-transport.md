---
bump: minor
floor: minor
---
- Electron transport can run in a finite water volume (`RunConfig.geometry`): tracks are killed at the boundary and their energy is reported as escaped (`terminations.escaped`, `energy.escaped_ev`).
- A primary outside the volume is rejected, as in Geant4.
- `transport_electron` takes a new `volume: Option<&Volume>` argument; `None` gives bitwise-identical infinite-medium results.
