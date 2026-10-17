---
bump: patch
floor: patch
---
- Electron-hole recombination in the physicochemical stage now follows Geant4's zero-time scheduling, so pre-chemistry species counts match the Geant4 11.4.1 reference (the RR-001 stage-matched M6 gate passes with unchanged criteria). Rust results for the physicochemical stage change slightly.
- Added an ignored audit test with conservation relations and a five-seed Rust comparison, and documented the M6 residual audit.
