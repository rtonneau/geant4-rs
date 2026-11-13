---
bump: minor
floor: minor
---
- Add geometry primitives to `g4-core`: a box, a full G4Tubs and a full G4Sphere, each with a centre offset (nm, rad). Each solid has a Geant4-tolerant inside test (Inside, Surface or Outside) and a distance-to-exit along a ray, ported from Geant4 11.4.1. The documented `G4Sphere` deviations fix defects found in its `DistanceToOut`.
- Add an optional `geometry` field to `RunConfig`, validated when it is read. Runs still use infinite water, and setting a volume is rejected until transport honours boundaries.
