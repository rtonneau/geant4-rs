# Ticket 02: charged-pdc

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] The charged branch of `G4DNAIRT::GetIndependentReactionTime` and `SamplePDC` executes all eleven option3 IDs 42–52 with caller-owned RNG and explicit `Immediate`/`After`/`NoReaction` outcomes.
- [ ] Tests cover contact, exact radius, inside/outside radii, attractive and repulsive configurations where applicable, neutral/parameter limits, long-time no-reaction mass, and numerical failure as typed errors.
- [ ] Deterministic values use independently evaluated high-precision formula anchors, and existing TDC/neutral-PDC validation remains passing.

**Files to Touch:**
- `crates/g4-chemistry/src/irt.rs`
- `crates/g4-chemistry/src/reaction_parameters.rs`
- `crates/g4-chemistry/src/error.rs`
- `crates/g4-chemistry/tests/charged_pdc.rs` (new)
- `crates/g4-chemistry/tests/fixtures/charged_pdc_anchors.json` (new)

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test charged_pdc && cargo test -p g4-chemistry --test irt_pdc && cargo test -p g4-chemistry --test two_body_validation
```

Expected:
The charged anchors and both existing pair suites pass with exit 0.

**Notes:**

Audit `G4DNAIRT.cc:285-390` directly. Evaluate reference formulas outside production Rust code before asserting anchors. Treat source singularities that lack a defensible limit as typed errors; never convert sampler failure to `NoReaction`.
