# Use the option3 independent reaction-time model

_ADR 0003, recorded 2026-10-23 (session 2026-10-23__m7-diffusion-and-two-body-irt)._

<!-- Format adapted from mattpocock-skills' domain-modeling (ADR-FORMAT.md): https://github.com/mattpocock/skills.git -->

Geant4 option3 selects `G4DNAIRT::GetIndependentReactionTime`, whose behavior differs from the separate IRT-syn diffusion-controlled model. Rust therefore treats the option3 independent reaction-time formulas as normative and does not mix the two models; it replaces Geant4's overloaded time sentinels and silent rejection-sampling exhaustion with explicit outcomes and typed errors so numerical failure cannot be mistaken for physical non-reaction.
