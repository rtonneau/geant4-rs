# Ticket 01: source-faithful-sbs-kernel

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.11)
**Effort:** high

**Acceptance Criteria:**
- [ ] Add a validation-only SBS module for an isolated totally diffusion-controlled pair that reproduces Geant4 11.4.1's adaptive encounter step, including `8 * (D_A + D_B + 2 * sqrt(D_A * D_B))` and its `32 * D_H` reduction for H+H.
- [ ] Diffuse both molecules independently through the existing `diffuse` function, treat `r <= R` as contact, and apply the Smoluchowski Brownian bridge only when the adaptive step is capped by the observation boundary.
- [ ] Deterministic tests pin the adaptive timestep, uncapped endpoint contact, capped bridge acceptance and rejection, inclusive equality contact, invalid inputs, and RNG consumption.
- [ ] The module remains outside any production scheduler and does not change the production IRT interface.

**Files to Touch:**
- `crates/dna-chemistry/src/sbs.rs`
- `crates/dna-chemistry/src/lib.rs`
- `crates/dna-chemistry/tests/sbs.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test sbs
```

Expected:
All deterministic SBS tests pass.

**Notes:**

Follow executable Geant4 code, not the nearby stale source comment that states the wrong multiplier. Preserve Geant4's inclusive SBS boundary even though production IRT handles exact equality differently.
