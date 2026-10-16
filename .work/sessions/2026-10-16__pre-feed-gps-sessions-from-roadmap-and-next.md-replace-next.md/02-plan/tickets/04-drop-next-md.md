# Ticket 04: drop-next-md

**Model:** haiku
**Effort:** low

**Acceptance Criteria:**
- [ ] Before deleting, every item of `docs/next.md` sections (i).1, (i).2 and the M7 prompt is found in `docs/gps-backlog.md` or docs/09 (section 3 is obsolete: `m6-physchem-b` was merged as PR #10).
- [ ] `docs/next.md` is deleted with `git rm`.
- [ ] `docs/README.md` line 30 points to `gps-backlog.md` and `/gps status` instead.
- [ ] The comment in `crates/dna-sim/tests/physchem_a.rs` (line ~17) says `docs/gps-backlog.md` (seed `m6-comparison-report`) instead of `docs/next.md`.
- [ ] No remaining reference to `next.md` outside `.work/sessions`.

**Files to Touch:**
- `docs/next.md`
- `docs/README.md`
- `crates/dna-sim/tests/physchem_a.rs`

**Verification Step:**

Run:
```bash
grep -rn "next\.md" --include=*.md --include=*.rs --include=*.toml . --exclude-dir=.work --exclude-dir=target; cargo fmt --all -- --check
```

Expected:
No grep output; `cargo fmt` exits 0.

**Notes:**

Comment-only change in the Rust file; do not touch code. Run `cargo clippy --workspace --all-targets -- -D warnings` too if time allows (nothing should change).
