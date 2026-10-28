# Ticket 06: network-and-reproducibility

**Model:** heavy
**Model (Jev):** opus-5.5 (confidence 0.42)
**Effort:** high

**Acceptance Criteria:**
- [ ] Validate full option3 execution, fail fast on unsupported regimes, and compare history outputs and RNG streams across 1/2/8 threads and batches with a test-only wrapper; preserve physics-only builds.

**Files to Touch:**
- `crates/g4-chemistry/tests/history_network.rs`
- `crates/g4-sim/tests/history_reproducibility.rs`
- `docs/reproducibility.md`

**Verification Step:**

Run:
```bash
cargo test --workspace
cargo test -p g4-sim --no-default-features
```

Expected: both pass.

**Notes:**

G1 must pass before implementation. Production integration is R06.
