# Ticket 02: competing-events

**Status:** ✅ Done

## Local Test Result

`cargo test -p g4-chemistry --test history_competition` exited 0: 4 passed,
0 failed, 0 ignored. Before implementation all four fixtures failed on the
old `CompetingHistoryPair` rejection, confirming they exercise the new behavior.

Additional verification on 2026-11-05:
- `cargo test -p g4-chemistry --test history_competition --test history_pair`:
  4 competition and 8 existing pair tests passed.
- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace`: exit 0; aggregate 517 passed, 0 failed,
  21 ignored (existing opt-in validation tests, not claimed as passing).
- `cargo check -p g4-sim --no-default-features`: exit 0.
- `git diff --check`: exit 0.

Logs: `.scratch/tests/2026-10-29__serial-irt-history-engine/ticket-02-competition.log`,
`ticket-02-fmt.log`, `ticket-02-clippy.log`, `ticket-02-workspace.log`, and
`ticket-02-physics-only.log` in that same scratch directory.

## Review Notes

Base: ffd5f92be6c0ea3277d231a59a53f43fc526fc22. Pre-existing changes: GPS session config and newly started ticket log only; project tree clean. Recovery count: 0.

Implemented canonical identity-pair enumeration with source reaction ID ordering,
streaming candidates into the bounded event queue instead of allocating a
quadratic candidate list. Source reactant order remains separate for placement
and recorded parent lineage. Each A+A pair contains distinct identities and is
sampled once. Events with dead or version-changed parents are discarded before
time advancement, placement RNG, or reaction-capacity checks. Unaffected queued
events retain their original clock and insertion ordinal. Queue occupancy reports
physical entries, including stale entries awaiting lazy removal; the exposed next
event is live after advancement.

Tests cover A+B/A+C contact competition, source-ordered lineage and generations,
descriptor-order invariance, a three-way A+A exact tie, a later-inserted clock
winning by time, discarded stale events at a full reaction budget, untouched
survivors, and unaffected clock/placement and RNG equality across intermediate
advances. A unit fixture independently checks both parent generations, both alive
flags, and rejection of a self-pair. Updated the obsolete competition-rejection
assertion and header in `tests/history_pair.rs`; its remaining regressions pass.

Inspected the local Geant4 11.4.1 source, verified `G4VERSION_NUMBER 1141` in
`source/global/management/include/G4Version.hh`. Read
`source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc` (`IRTSampling`,
`Sampling`, `MakeReaction`, `FindReaction`),
`source/processes/electromagnetic/dna/management/include/G4ITReaction.hh`
(`CanAddThisReaction`, `SelectThisReaction`), and its `src/G4ITReaction.cc`
(`RemoveMe`, `compReactionPerTime`, `GetHash`). Upstream eagerly removes the
participants' reaction sets; alive/generation checks implement equivalent
invalidation lazily. Upstream exact ties use a track hash, so the approved Rust
canonical insertion-order policy is an intentional documented deviation, not a
claim of exact tie parity. No Geant4 generator was run or reference data changed.

Products remain recorded but unevolved. Unary channels, product cascades,
production spatial search, full-network results, and R05/G2 or independent
physical acceptance remain outside this ticket. The source baseline remains
11.4.1; the 11.4.3 G1-F result does not validate scheduler semantics.

Implementer inspected the final diff against the ticket criteria. No files were
staged or committed; session configuration and script-owned log fields preserved.

Independent review: gpt-6-astra, high effort, reviewed frozen tracked and untracked changes against base ffd5f92be6c0ea3277d231a59a53f43fc526fc22, ticket criteria and Geant4 11.4.1 source. No actionable findings. Saved test evidence supports results; formatting exit status is recorded by implementer. Accepted; updated pair regression is necessary for new competition behavior.

## Blockers / Challenges

None unresolved. The project configuration and user CONFIG_RESOLUTION.md were
absent; DEV_DIR/G4_ROOT were set, and the explicit approved 11.4.1 source tree
was present and version-verified. No configuration or machine paths were added.

## Commits

- 7b6b73d feat: schedule competing pair events (ticket 02)

## Time Spent

12m (ticket-start.js to ticket-complete.js)
