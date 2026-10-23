# Ticket 05: pdc-independent-reaction-time

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.24)
**Effort:** high

**Acceptance Criteria:**
- [ ] Neutral PDC sampling implements the option3 `a`/`b` parameters, ultimate-reaction probability, and exact rejection envelope using caller-provided randomness.
- [ ] Overlap uses the source contact probability; equality and non-overlap use the continuum branch exactly as agreed.
- [ ] Exhausting 10,000 rejection attempts returns a typed convergence error and is never reported as `NoReaction`.
- [ ] Scripted-RNG tests pin accepted/rejected proposals, contact/no-contact outcomes, attempt boundaries, numerical extremes, and random-draw order.

**Files to Touch:**
- `crates/dna-chemistry/src/irt.rs`
- `crates/dna-chemistry/src/error.rs`
- `crates/dna-chemistry/tests/irt_pdc.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test irt_pdc
```

Expected:
All PDC distribution-branch, contact, convergence, extreme-value, and draw-order tests pass.

**Notes:**

Do not add charged-PDC analytic validation or IRT-syn activation-delay behavior. Reuse the public outcome and parameter types from earlier tickets. Use TDD and commit as `feat: sample PDC independent reaction times`.
