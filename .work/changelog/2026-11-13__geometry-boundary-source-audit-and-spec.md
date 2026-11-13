---
bump: patch
floor: patch
---
- The project reference is now Geant4 11.4.3. A migration note shows the DNA sources are byte-identical to 11.4.1, and each reference file's provenance keeps its own version.
- Documented how Geant4 11.4.3 handles geometry boundaries: world-exit kill, interaction length carried over, and the 0.8 × safety displacement clamp.
- Specified the finite water volume: the `Volume` enum, fixed acceptance criteria, the M12 milestone and run request RR-009.
