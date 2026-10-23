# Ticket 05: preregister-geant4-comparison

**Model:** medium
**Model (Jev):** sonnet-5.5 (confidence 0.20)
**Effort:** high

**Acceptance Criteria:**
- [ ] Add an ignored Rust consumer for the future two-body CSV and provenance whose header fixes the schema, six-point comparison family, sample count, seed policy, and tolerances before data exists.
- [ ] Validate provenance and exact row coverage before comparing Rust IRT survival counts with the Geant4 counts; malformed, partial, or mismatched inputs fail explicitly.
- [ ] Add one open `RUN_REQUESTS.md` entry and document exact project-lead smoke/production commands, non-overwrite rule, output guards, provenance fields, and the paired close-gate test command.
- [ ] No reference CSV or provenance file is created or run; ordinary workspace tests pass with the external comparison ignored and the gate remains `awaiting data`.

**Files to Touch:**
- `crates/dna-chemistry/tests/geant4_two_body.rs`
- `validation/geant4/RUN_REQUESTS.md`
- `validation/geant4/README.md`
- `validation/geant4/BUILD_NOTES.md`

**Verification Step:**

Run:
```bash
cargo test -p dna-chemistry --test geant4_two_body
```

Expected:
The test binary passes with the external-data comparison reported as ignored; no reference data is generated.

**Notes:**

Do not invoke the ignored test before its registered CSV and provenance both exist. The close-gate session will validate and run it.
