# Ticket 01: validate-rename-snapshots

**Model:** medium
**Effort:** high

**Acceptance Criteria:**
- [ ] `scripts/check_crate_split.py --compare` remains strict by default; a dedicated `--allow-crate-rename` mode accepts only `dna-sim` to `g4-sim` and the five planned local package identity changes, with both snapshot revision IDs validated against Git.
- [ ] The mode verifies each snapshot's recorded `Cargo.lock` hash against its revision, then proves the old and new lockfiles differ only in the five local package names and their local dependency references. Every third-party version, source, and checksum remains identical.
- [ ] Exact comparison still rejects a one-bit scientific change, a changed data hash, schema/CSV/species order change, toolchain difference, unlisted metadata, or an unexpected package rename.
- [ ] Python checker unit tests pass on the pre-rename workspace; existing snapshot comparison remains compatible.

**Files to Touch:**
- `scripts/check_crate_split.py`
- `scripts/test_check_crate_split.py`

**Verification Step:**

Run:
```bash
python -m unittest discover -s scripts -p test_check_crate_split.py
python scripts/check_crate_split.py --compare .scratch/tests/2026-10-26__crate-domain-ownership/evidence/baseline-physics.json .scratch/tests/2026-10-26__crate-domain-ownership/evidence/ticket-02-physics.json
```

Expected:
The Python suite passes and the existing strict comparison reports exact scientific, input, and toolchain identity. New mutation cases fail as specified.

**Notes:**

Keep the scientific `runs` payload, `data/` hashes, toolchain, schema and CSV comparison exact. Baseline files are in S01's retained `evidence/`; the original `target/crate-split-baseline/` path is absent. The lockfile exception is narrow and can use Git's committed lockfile at each recorded revision. Do not alter captures or reference fixtures.
