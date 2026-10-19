# Ticket 03: instrument-rust-spatial-boundaries

**Model:** heavy
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] Rust emits diagnostic records at chemistry-producing physical sites, solvated-electron creation, pre-decay water states, post-dissociation products, and post-recombination alive survivors without changing RNG draws, conversion order, emitted species, or normal scoring.
- [ ] Records preserve history, available physical track/parent identity, and deterministic creation ordinal; excitation, ionisation, attachment, and terminal-electron physical sites remain distinct, while elastic and non-chemistry vibrational interactions are excluded.
- [ ] An ignored 900 keV diagnostic generates the five preregistered seeds x 40 histories in long-form CSV compatible with ticket 02 and prints the primary/secondary Rust summaries and achieved precision.
- [ ] Focused regression tests show diagnostics are observational: enabling them leaves the existing species list, physicochemical statistics, RNG consumption, and thread-count-independent normal summary bitwise unchanged.

**Files to Touch:**
- `crates/dna-core/src/physchem.rs`
- `crates/dna-physics/src/transport.rs`
- `crates/dna-chemistry/src/engine.rs`
- `crates/dna-sim/src/run.rs`
- `crates/dna-sim/src/spatial_diagnostic.rs`
- `crates/dna-sim/tests/spatial_diagnostic.rs`
- `crates/dna-sim/tests/physchem_reference.rs`

**Verification Step:**

Run:
```bash
cargo test -p dna-core -p dna-physics -p dna-chemistry -p dna-sim spatial_diagnostic
```

Expected:
All focused diagnostic and non-perturbation tests pass; ignored production statistics are not executed by the normal command.

**Notes:**

Instrumentation may add optional diagnostic records or observers but must not reorder the LIFO cascade or zero-time chemistry passes. Keep the public stage-boundary types minimal and document any identifier that has no exact counterpart in the other implementation.
