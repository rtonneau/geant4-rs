# Ticket 05: spatial-index

**Status:** ✅ Done

## Local Test Result

- `cargo test -p g4-chemistry --test history_spatial -- --nocapture`: PASS, 9 passed, 0 failed. Covers the source cutoff and multiplier domain, whole-cell/newer-parent initial queries, duplicate exclusion, compact indexed/all-pairs equivalence for 16 streams and multipliers 1/2/4, configuration reporting, invalid grids, explicit capacity failures and the 10,000-molecule resource fixture. Output: `.scratch/tests/2026-10-29__serial-irt-history-engine/ticket05-spatial.log`.
- `cargo test -p g4-chemistry --test history_observation --test history_reactions --test history_competition --test history_pair --lib`: PASS, 71 passed, 0 failed (run together with the then-eight spatial tests). Includes two new internal tests: exhaustive three-dimensional source-cell oracle and fixed-edge product insertion/dead/self/future-sibling exclusion.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS. Output: `.scratch/tests/2026-10-29__serial-irt-history-engine/ticket05-clippy.log`.
- `cargo test --workspace`: PASS, exit 0; 551 passed, 0 failed, 21 ignored across 63 test/doc-test summaries. Output: `.scratch/tests/2026-10-29__serial-irt-history-engine/ticket05-workspace-tests.log`.
- `cargo check -p g4-sim --no-default-features`: PASS (physics-only build).
- `git diff --check`: PASS.

## Review Notes

Base: cd3308bb6d1a704719ea9c4763c0bbe4292a3a17. Pre-existing changes: GPS session config and newly started ticket log only; project tree clean. Recovery count: 0.

- Added owned sparse source-style bins in `history_spatial.rs`, privately integrated through `history.rs`. `HistoryOptions::search` defaults to indexed multiplier 1; `CandidateSearch::AllPairs` explicitly selects the untruncated small-system oracle. `PairHistory::new` keeps its existing all-pairs behavior, while `with_search` selects either policy for channel experiments. Both paths retain canonical identity-pair/reaction-ID order, source-ordered product scheduling, stale-event checks and unchanged lazy partner sampling. One bounded candidate row is reused; no quadratic pair list is retained.
- Read actual Geant4 **11.4.1** `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc`: constructor 75-76, `SpaceBinning` 128-158, `IRTSampling` 161-174, `Sampling` 189-244, `FindBin` 344-358 and product insertion 497-504. Confirmed `G4Version.hh` reports 1141; complete local 11.4.1/11.4.3 `G4DNAIRT.cc` comparison is identical. Formula is `multiplier * (1.45 nm + 2 sqrt(8 * 0.00946 nm^2/ps * absolute_end_ps))`, fixed for the full history, not remaining time. Initial extrema seeds, fixed bin widths, whole cells intersecting the query box, newer-parent query direction and clamped edge products are retained. Search uses stored sites before temporary diffusion. Reverse initial queries preserve established Rust canonical pair/RNG order rather than adopting upstream track traversal order. Invalid/unrepresentable G4int grids fail explicitly instead of invoking source integer overflow.
- Added diagnostics for configured horizon/search/cutoff, visited candidate pairs, attempted matching pair channels, total successfully queued events, maximum temporary candidate row, retained index identities and peak retained heap payload. Dead index entries are bounded by the existing retained-arena molecule limit and skipped during search. Payload accounting uses vector capacities and map key/value payloads and explicitly excludes allocator/node spare overhead, stack, transient placement and caller-owned input/network/observation output; it is a lower bound, not process RSS.
- Measured deterministic high-population fixture: 10,000 H atoms in 5,000 isolated contacting pairs on a 100-by-50 grid, 20 nm spacing, 0.01 nm pair separation, absolute horizon 1 ps, multiplier 1, full option3 network, tracing disabled. Default limits: 100,000 retained molecules/pending events/executed reactions, 1,024 observations. Results: 5,000 visited candidates, 5,000 sampled pair channels, 5,000 scheduled/executed reactions, 5,000 peak pending events, 15,000 retained arena/index identities, maximum candidate row 1, **2,630,432 bytes** peak retained engine payload. All-pairs initialization of the same input would visit 49,995,000 identity pairs; that quadratic run was not performed. The full network samples two H unary clocks (55/68) per initial atom, giving 20,000 draws despite the contact-only executed events.
- Separately executed the focused compiled fixture with `--exact sparse_high_population_reports_bounded_storage_events_and_candidates --nocapture`, polling Windows `Process.PeakWorkingSet64` at 1 ms intervals: maximum sampled process high-water mark **9,981,952 bytes**, exit 0. This includes test harness/input/network/allocator overhead and is a sampled process measurement, not an exact engine allocation total. Evidence: `ticket05-memory.log`, `ticket05-memory.stdout.log`, `ticket05-memory.stderr.log` in the same scratch directory. Platform: rustc 1.97.0 (2d8144b78), x86_64-pc-windows-msvc, optimized test profile.
- Necessary adjacent changes: boxed `IncompleteHistory` diagnostics in `error.rs` so expanded instrumentation does not enlarge every chemistry Result/error (Clippy rejects the unboxed form); updated one exhaustive options literal and selected all-pairs explicitly in the existing extreme-coordinate placement-error fixture, preserving that fixture's original placement purpose while separately testing earlier indexed-grid errors.
- Finite cutoffs do not imply zero omitted reaction probability. The compact exact comparisons and synthetic high-population fixture establish local semantics/resource behavior only; clustered/dilute convergence, independent physical validation, R05/G2 acceptance and R06 production integration remain subsequent work. No reference data was overwritten, no Geant4 source was modified and no generator was executed.

Independent review: gpt-6-astra, xhigh effort, reviewed frozen tracked and untracked changes against base cd3308bb6d1a704719ea9c4763c0bbe4292a3a17, ticket criteria and Geant4 source. No actionable findings. Saved evidence supports tests and measurement; no independent rerun. Accepted with cutoff convergence and G2 deferred to ticket 07. Error boxing and fixture adjustment are justified adjacent changes.

## Blockers / Challenges

None unresolved. The user CONFIG_RESOLUTION.md and project `.claude/.claude-project.json` are absent as recorded in prior tickets; no configuration was invented. The existing repository source-root expression and environment-backed 11.4.1 tree were used and version-verified. Initial test-first compilation failed on the missing search APIs as intended. Subsequent checks exposed an incorrect fixture assumption about full-network unary RNG draws, an exhaustive options literal needing the new field and Clippy's large-error warning; these were corrected and all required checks passed. One read-only dispatch was aborted before starting and retried; one patch context mismatch was corrected without changing unrelated content. No permission denial occurred. No staging, commit or GPS state script was run; script-owned log fields and recovery count are preserved.

## Commits

- bc8d1ef feat: add bounded spatial candidate search (ticket 05)

## Time Spent

20m (ticket-start.js to ticket-complete.js)
