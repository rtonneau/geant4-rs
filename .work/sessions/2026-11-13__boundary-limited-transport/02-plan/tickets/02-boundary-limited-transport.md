# Ticket 02: boundary-limited-transport

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] `transport_electron` takes `volume: Option<&Volume>`; all callers updated (None).
- [ ] d_out is computed before the draw: 0 means escape with no draw; else s is drawn; `d_out < s` means move to the surface, add the energy to `ledger.escaped`, end the track; otherwise the step is unchanged.
- [ ] Outside primary → `PhysicsError::PrimaryOutsideVolume`; `Termination::Escaped` for the primary; `HistoryOutcome` counts escaped tracks.
- [ ] Tests: None bitwise identical; a huge volume equals None; a tiny volume means the primary escapes; the ledger closes to 1e-9; the draw sequence matches the unbounded one up to the first limited step; Surface-out primary; Outside error.

**Files to Touch:**
- `crates/g4-physics/src/transport.rs`
- `crates/g4-physics/src/error.rs`
- `crates/g4-physics/tests/transport.rs`
- `crates/g4-physics/tests/low_energy.rs`
- `crates/g4-physics/tests/windows.rs`
- `crates/g4-sim/src/history.rs`
- `crates/g4-sim/tests/spatial_diagnostic.rs`

**Verification Step:**

Run:
```bash
cargo test -p g4-physics && cargo clippy --workspace --all-targets -- -D warnings
```

Expected:
All tests pass, no warnings.

**Notes:**

The debug in-flight ledger assertion must include escaped energy. g4-sim's scoring match on Termination needs an arm (map it to a new index in ticket 3, or add it now to compile).
