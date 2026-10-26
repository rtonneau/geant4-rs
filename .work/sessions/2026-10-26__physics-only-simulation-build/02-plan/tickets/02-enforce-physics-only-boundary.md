# Ticket 02: enforce-physics-only-boundary

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] Checker `--check-graph` walks each selected root's resolved normal/build dependency graph and rejects forbidden direct or transitive edges; a disconnected chemistry workspace member does not fail. Package-scoped no-default `g4-sim` graph excludes `g4-chemistry`.
- [ ] Checker `--capture <new-directory> --package g4-sim --no-default-features` creates only physics output; existing target directories are not overwritten. Default and no-default physics captures have exactly equal scientific payloads, and default physchem still matches pre-S01.
- [ ] CI runs package-scoped no-default check/test/Clippy and graph checks alongside existing default gates. Current guides document the feature matrix, imports, future-only biology and the executed verification evidence.
- [ ] Full matrix in Tasks 4–5 passes: format, workspace Clippy/tests, selected M2/M3/M6 reference gates, standalone physics/chemistry checks, no-default simulation tests/Clippy, checker unit tests and graph check. Record commands, exit codes, selected test counts, input hashes and revisions.

**Files to Touch:**
- `scripts/check_crate_split.py`
- `scripts/test_check_crate_split.py`
- `.github/workflows/ci.yml`
- `docs/crate-separation.md`
- `docs/reproducibility.md` and current architecture/command guides if they list supported profiles

**Verification Step:**

Run:
```bash
python -m unittest discover -s scripts -p test_check_crate_split.py
python scripts/check_crate_split.py --check-graph
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p g4-sim --no-default-features --all-targets --locked
cargo clippy -p g4-sim --no-default-features --all-targets --locked -- -D warnings
```

Expected:
All exit 0. Checker fixtures detect valid, forbidden direct, forbidden transitive, disconnected-member, default-sim and no-default-sim graphs. The plan's selected M2/M3/M6 commands each execute their registered comparisons and exit 0. Exact same-platform snapshots compare equal; the no-default dependency graph has no chemistry node reachable from `g4-sim`.

**Notes:**

Run separate Cargo invocations to avoid workspace feature unification. Inspect dev-dependency and `--all-targets` behavior. Consume existing committed Geant4 reference data only; do not run generators or blanket-run ignored tests. Preserve historical fixtures/provenance and old run commands. Commit checker, CI, evidence and docs, then finish through GPS. Do not start R02 automatically.
