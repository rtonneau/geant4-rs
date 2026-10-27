# Ticket 03: unary-clocks

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] All 20 source first- and pseudo-first-order channels sample independent exponential delays in `s^-1` with `ReactionTime` semantics; a disabled custom channel returns `NoReaction` without consuming RNG.
- [ ] Non-finite, negative, or overflowing rates/times fail with typed errors; positive rates produce finite positive delays.
- [ ] Competing-channel tests check total hazard and branching fractions for each tested family with preregistered family alpha 0.001 and N >= 100,000 samples, plus deterministic clock anchors.

**Files to Touch:**
- `crates/g4-chemistry/src/reaction_network.rs`
- `crates/g4-chemistry/src/unary.rs` (new)
- `crates/g4-chemistry/src/error.rs`
- `crates/g4-chemistry/src/lib.rs`
- `crates/g4-chemistry/tests/unary_clocks.rs` (new)

**Verification Step:**

Run:
```bash
cargo test -p g4-chemistry --test unary_clocks
```

Expected:
Deterministic and preregistered statistical tests pass with exit 0.

**Notes:**

Use `G4DNAIRT::Sampling` lines 250–281 for source rate convention and independent clocks. The event scheduler and its time-window filter are R05 scope. Preserve each unary channel's bulk reactant as a reservoir descriptor, not a mobile molecule.
