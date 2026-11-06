# Ticket 04: observation-and-limits

**Status:** ✅ Done

## Local Test Result

- `cargo test -p g4-chemistry --test history_observation`: PASS, 10 passed, 0 failed. Covers separately labeled pre-IRT/post-contact counts, duplicate observations, exact ties and inclusive final horizons, contact-product closure, final arena/lineage/RNG invariance across 16 chemistry streams, trace-on/off equivalence, empty/nonreacting inputs, all five resource limits, partial execution diagnostics, and terminal numerical errors.
- `cargo test -p g4-chemistry --test history_pair --test history_competition --test history_reactions --lib`: PASS, 59 passed, 0 failed.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS. Full output: `.scratch/tests/2026-10-29__serial-irt-history-engine/ticket04-clippy.log`.
- `cargo test --workspace`: PASS, exit 0; 540 passed, 0 failed, 21 ignored across 62 test/doc-test summaries. Repeated after strengthening the delayed-event fixture; final output: `.scratch/tests/2026-10-29__serial-irt-history-engine/ticket04-workspace-tests.log`.
- `cargo check -p g4-sim --no-default-features`: PASS (physics-only build).
- `git diff --check`: PASS.

## Review Notes

Base: 8ef36916846d0f7493e15dc14e134f32e1ff7cb3. Pre-existing changes: GPS session config and newly started ticket log only; project tree clean. Recovery count: 0.

- Added `run_history` over the existing engine, accepting immutable `HistoryInput`, a validated `ReactionNetwork`, nondecreasing observation times, finite `HistoryOptions` and caller RNG. `HistoryResult` is returned only after the inclusive horizon completes; its distinctly labeled pre-IRT, post-contact, requested and final populations cover every mobile species in `Species::ALL` order. Empty schedules still evolve through the horizon. Duplicate times are supported. No position sampling or G-value normalization was introduced.
- Added incremental `observe_counts` and aggregate diagnostics. Contact observations process all immediate product cascades before counting. The immutable input is preserved; only reactions materialize parent positions. Sixteen seeded full-network fixtures compare final identities, alive/generation fields, coordinate/time bits, lineage, pending events and 128 following RNG words with and without intermediate observations; an explicit assertion requires positive-time reactions so this exercises delayed placement as well as contacts. Default no-trace and enabled-trace runs also match their final populations and following RNG words.
- The complete-network runner disables lineage retention by default. Explicit `Some(n)` tracing has a separate finite limit checked before executing the next event. A separate executed-reaction counter enforces the reaction budget even without retained records. Existing `PairHistory::new` remains a bounded, trace-retaining diagnostic interface for prior selected-channel fixtures; both interfaces use the same scheduler and placement paths. Its compatibility behavior is documented.
- `IncompleteHistory` carries the original typed cause and aggregate time, executed-reaction, arena, pending/peak-queue and trace-record diagnostics. Initialization failures retain available diagnostics; execution failures discard any sampled count outputs. Trace exhaustion is an error, never silent truncation. Budget units are retained records, not allocator bytes; consumed molecule identities remain in the arena. Failed runs do not roll back RNG. Invalid observation schedules and horizons fail before drawing RNG.
- Read Geant4 **11.4.1** `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc`: `Sampling` 212-281 (lazy partner sampling and strict final-time scheduling), `MakeReaction` 393-508 (parent materialization and immediate source-ordered product scheduling), `FindReaction` 513-549 (events at or before global time). Confirmed `G4Version.hh` reports 1141 and compared complete local 11.4.1 and 11.4.3 `G4DNAIRT.cc` files with `git diff --no-index`: identical. Existing inclusive final-boundary, tie-order and placement deviations remain documented and unchanged. No Geant4 reference source/data was modified and no generator was run.
- Scope remains the serial all-pairs history interface. Spatial-index convergence, representative workload evidence, R05/G2 acceptance, independent physical validation and R06 production integration remain subsequent work. No dependency or unrelated source changes.

Independent review: gpt-6-astra, xhigh effort, reviewed frozen tracked and untracked changes against base 8ef36916846d0f7493e15dc14e134f32e1ff7cb3, ticket criteria and Geant4 source. No actionable findings. Saved test and Clippy logs support results; fmt and physics-only status recorded by implementer. Accepted; spatial convergence and R05 acceptance remain open.

## Blockers / Challenges

None unresolved. The configured user `CONFIG_RESOLUTION.md` and project `.claude/.claude-project.json` are absent, as recorded in earlier tickets; no replacement paths/config were invented. The existing repository source-root convention and environment-backed 11.4.1 junction located the read-only source. The first test-first compile exposed missing API and fixture RNG method/dependency assumptions; the fixtures were corrected to existing `RngStreams::stream` without adding dependencies, then verified to fail solely for the absent history API before implementation. All focused checks subsequently passed. No permission denial occurred.

## Commits

- 5bb56ed feat: expose bounded history observations (ticket 04)

## Time Spent

19m (ticket-start.js to ticket-complete.js)
