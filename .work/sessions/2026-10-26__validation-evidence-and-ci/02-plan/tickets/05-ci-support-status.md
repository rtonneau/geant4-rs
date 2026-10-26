# Ticket 05: ci-support-status

**Model:** medium
**Effort:** medium

**Acceptance Criteria:**
- [ ] CI runs the required-data tier plus format, Clippy and workspace tests on Linux and Windows using the pinned Rust 1.97.0 toolchain; removing a required fixture makes the required tier fail.
- [ ] `rust-version` matches the supported pinned 1.97.0 policy, without an untested claim for Rust 1.85.
- [ ] README and current validation/roadmap summaries name the maintained selection, manual statistical command, historical exclusions, RR-002's qualified conclusion and local-artifact limitation, and any platform result not actually observed as unverified.
- [ ] Historical reference/provenance files, old review findings and scientific thresholds are unchanged.

**Files to Touch:**
- `.github/workflows/ci.yml`
- `Cargo.toml`
- `README.md`
- `docs/07-validation-strategy.md`
- `docs/reviews/development-state.md` (current-status section only)

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python scripts/run_validation_gates.py --tier required
```

Expected:
All local commands pass; CI config selects the required-data tier on both platforms. Remote CI status is reported only after its jobs actually run.

**Notes:**

Keep `CLAUDE.md` delegation and historical evidence unchanged. Do not claim a Linux pass from local Windows tests; read CI results before reporting them.
