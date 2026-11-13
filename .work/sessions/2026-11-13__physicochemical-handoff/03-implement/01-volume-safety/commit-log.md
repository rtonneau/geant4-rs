# Ticket 01: volume-safety

**Status:** ✅ Done

## Local Test Result

`cargo test -p g4-core geometry`: 120 passed, 0 failed. `PROPTEST_CASES=200000 cargo test --release -p g4-core safety_never`: 3 passed. clippy -D warnings clean.

## Review Notes

Base 54f4c9f65303b7a2aefb3bfc470228762e063740, clean tree. Inline implementation, reviewed inline (localized geometry change). Ported G4Box/G4Tubs/G4Sphere::DistanceToOut(p) from 11.4.1 source with line citations. Deviation: `None` for an Outside point (Geant4 returns 0), consistent with `distance_to_out`. Analytic tests per solid; proptest safety <= distance_to_out + kCarTolerance. docs/04 D.5 updated.

## Blockers / Challenges

None

## Commits

- ab82b54 feat(g4-core): Volume::safety from Geant4 DistanceToOut(p) (ticket 01)

## Time Spent

2m (ticket-start.js to ticket-complete.js)
