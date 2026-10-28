# Ticket 07: convergence-and-g2

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.55)
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] Preregister clustered and dilute indexed/all-pairs and cutoff 1/2/4 comparisons before sampling; require the 99% relative interval within +/-1% for non-rare means and the absolute interval within +/-0.01 below 0.1 molecule/history; unresolved precision or bias blocks G2.
- [ ] Report measured resource counts and source semantics.

**Files to Touch:**
- `crates/g4-chemistry/tests/history_convergence.rs`
- `docs/validation/irt-history-convergence.md`
- `docs/reproducibility.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all pass, and the convergence verdict reports evidence honestly.

**Notes:**

G1 must pass before implementation. Later full-track Geant4 count acceptance belongs to R07.
