# Isotropic primary direction drawn on a dedicated Source stream

_ADR 0006, recorded 2026-11-14 (session 2026-11-14__m12-isotropic-source-box-endpoints)._

<!-- Format adapted from mattpocock-skills' domain-modeling (ADR-FORMAT.md): https://github.com/mattpocock/skills.git -->

RR-009 box endpoints need isotropic primaries (Geant4 `G4RandomDirection()`, one per event). The direction is drawn from a new per-history stream `Stage::Source = 3` (the last reserved slot) rather than the head of the `Physics` stream, so the physics draw order and every existing digest stay unchanged and the physics draws of a history do not depend on the source mode. Consequence: no stage slot is left; a further stage needs a change of the stream-number layout.
