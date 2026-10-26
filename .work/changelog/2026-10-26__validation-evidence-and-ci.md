---
bump: patch
floor: patch
---
- Reject incomplete, malformed, or mismatched Geant4 reference evidence for the maintained M2, M3, and M6 comparisons.
- Add a versioned validation gate inventory with a required evidence check and an explicit seven-test statistical tier.
- Run required validation evidence, formatting, Clippy, and workspace tests in Linux and Windows CI with Rust 1.97.0.
