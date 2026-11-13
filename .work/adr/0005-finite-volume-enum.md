# Represent the finite water volume as a closed enum

_ADR 0005, recorded 2026-11-13 (session 2026-11-13__geometry-boundary-source-audit-and-spec)._

<!-- Format adapted from mattpocock-skills' domain-modeling (ADR-FORMAT.md): https://github.com/mattpocock/skills.git -->

The single homogeneous liquid-water world is represented as `enum Volume { Unbounded, Box { half: [f64; 3] }, Sphere { r: f64 } }`, centred on the origin, axis-aligned, with lengths in the project's internal length unit. The model maps one-to-one onto a Geant4 world `G4Box(hx, hy, hz)` or `G4Orb(r)`. A closed enum was chosen over a geometry trait because the shape set is small and known, the boundary rules in docs/04 D.5 need only `inside`, `distance_to_out` along a ray and isotropic `safety`, and a trait would add dynamic dispatch and an open extension point that nothing uses. Out of scope: nested volumes, other materials, rotations or offsets. Each of these would need a new decision (docs/04 D.5 shows multi-material boundaries change the interaction-length bookkeeping).
