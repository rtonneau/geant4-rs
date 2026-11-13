# Ticket 03: finite-volume-spec

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] The ADR records enum Volume { Unbounded, Box{half:[f64;3]}, Sphere{r} }, centred on the origin, enum rather than trait, with its rationale
- [ ] docs/09 lists the residual questions (IRT boundary handling with no navigator; fate of out-of-volume dissociation products), each tied to the new run request
- [ ] docs/07 holds fixed acceptance criteria: escape fraction and energy deposited inside the volume, compared with Geant4 under tolerances defined in advance
- [ ] RUN_REQUESTS.md logs the new request (not run), marking its generator as to be written later
- [ ] docs/08 adds a finite-volume milestone
- [ ] cargo fmt --check, clippy -D warnings and test pass

**Files to Touch:**
- `docs/adr/`
- `docs/07-validation-strategy.md`
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`
- `validation/geant4/RUN_REQUESTS.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected:
All succeed.

**Notes:**

docs/07 and RUN_REQUESTS.md already carry uncommitted RR-008 edits. Commit only this ticket's hunks (`git add -p` is unavailable non-interactively, so apply the patch with `git apply --cached`).
