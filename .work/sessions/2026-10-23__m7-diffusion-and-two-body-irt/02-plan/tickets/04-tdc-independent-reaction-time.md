# Ticket 04: tdc-independent-reaction-time

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.36)
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] The isolated sampler returns `Immediate`, `After(Time)`, or `NoReaction` for TDC pairs, with elapsed-time semantics and no negative-time sentinel.
- [ ] Neutral sampling implements the option3 ultimate-reaction probability and inverse first-passage law; charged sampling applies Geant4's effective-distance transform.
- [ ] Strict source contact behavior is preserved, equality is covered separately, and non-overlap tests include the probability mass at infinity.
- [ ] Scripted-RNG and fixed-seed tests cover immediate, finite, no-reaction, endpoint-adjacent, charged, A+A symmetry, and invalid-pair branches.

**Files to Touch:**
- `crates/dna-chemistry/src/irt.rs`
- `crates/dna-chemistry/src/error.rs`
- `crates/dna-chemistry/src/lib.rs`
- `crates/dna-chemistry/tests/irt_tdc.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test irt_tdc
```

Expected:
All neutral, charged, contact, A+A, scripted-draw, and no-reaction tests pass.

**Notes:**

Implement `G4DNAIRT::GetIndependentReactionTime` behavior, not the IRT-syn reaction model. Reuse Tickets 01-03 interfaces. Use TDD and commit as `feat: sample TDC independent reaction times`.
