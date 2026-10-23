# Ticket 03: v10-lsoda-well-mixed-oracle

**Model:** medium
**Model (Jev):** sonnet-5.5 (confidence 0.20)
**Effort:** high

**Acceptance Criteria:**
- [ ] Add `rsoda` as a public HTTPS Git dev dependency pinned to commit `ae76eee69d6d494be9253eaee819c77ee962c934`; no local path enters a manifest.
- [ ] Add a test-only exact Gillespie SSA enabling only option3 reaction ID 2, `e_aq + H -> OHm + H2`, with 10,000 molecules of each reactant at 1 micromolar, terminal products, 512 fixed-seed histories, and `t_end = 360 microseconds`.
- [ ] Every history satisfies the one-reaction conservation identities, and the ensemble mean reactant and product populations agree with the LSODA mass-action solution within 1% at the registered end time.
- [ ] SSA and ODE helpers remain test-only and introduce no production scheduler or ODE API.

**Files to Touch:**
- `crates/dna-chemistry/Cargo.toml`
- `Cargo.lock`
- `crates/dna-chemistry/tests/kinetics_ode.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test kinetics_ode -- --nocapture
```

Expected:
V10 conservation checks and the 1% LSODA comparison pass with the registered parameters.

**Notes:**

Use the validated option3 rate literal and explicit molar/volume conversion. Return typed test failures rather than hiding an LSODA error.
