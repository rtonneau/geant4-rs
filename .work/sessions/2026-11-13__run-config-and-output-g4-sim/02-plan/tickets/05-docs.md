# Ticket 05: docs

**Model:** fast
**Effort:** low

**Acceptance Criteria:**
- [ ] crate-separation.md, docs/04 D.5/D.6 and CHANGELOG describe volume, provenance, schema v5/v2, escape summary and CSV

**Files to Touch:**
- `docs/crate-separation.md`
- `docs/04-scientific-algorithms.md`
- `CHANGELOG.md`

**Verification Step:**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected:
All pass

**Notes:**

Concise.
