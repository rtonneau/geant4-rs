# Ticket 01: pair-history

**Status:** ✅ Done

## Local Test Result

`cargo test -p g4-chemistry --test history_pair` — exit 0:

```text
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Required workspace checks also passed: `cargo fmt --all -- --check` (exit 0), `cargo clippy --workspace --all-targets -- -D warnings` (exit 0), and `cargo test --workspace` (exit 0). Final verification after the last code/test edits reran `cargo test --workspace --quiet` (exit 0), including all eight pair-history integration tests and three history queue unit tests. Existing ignored statistical/reference tests remain ignored; no reference generator was run. `git diff --check` passed; Git emitted existing checkout line-ending notices only.

## Review Notes

Base: d6f2ec3aba90c84d2b27a95809ca60c19981f8b8. Pre-existing change: session .session-config.json (GPS state); no project changes. Recovery count: 0.

Implemented validated immutable 1 ps inputs, creation-order molecule IDs, liveness/generations, an absolute-time min-heap with insertion-ordinal ties, canonical pair initialization, isolated pair execution and initial contact closure. Conditional parent placement and ordered product records reuse existing validated kernels. Survivors retain their last materialized time. The end boundary is inclusive by the approved design. Molecule, queued-event and executed-reaction budgets return typed errors; execution failures prevent retrying a partially evaluated history.

This is explicitly the disjoint-pair slice: selected compiled pair descriptors are accepted; unary channels, duplicate descriptors and competing candidate pairs fail before RNG draws. Products are recorded but have no arena identities or subsequent reactions yet. Tickets 02/03 retain competition, stale-event invalidation, product scheduling and unary execution. This slice returns no complete-network count score and does not establish R05/G2 or independent physical validation.

Read the actual Geant4 11.4.1 `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc`: IRTSampling/Sampling (161-244), MakeReaction (393-508), FindReaction (513-549). The source version header reports 1141. The existing repository audit `validation/pair_oracles/evidence/geant4_source_audit_20261029.json` and `docs/reviews/development-state.md` establish the source-root expression; the local install is 11.4.3, but was not used to infer the scheduler baseline. Source pointers, inclusive-horizon deviation and current limitations are documented in history.rs. No reference source/data or generator was changed or executed.

Eight integration fixtures cover empty/nonreacting input, stable IDs/source parent order, deterministic contact closure, one-clock A+A initialization, positive-delay placement, inclusive horizons, no-reaction/beyond-horizon survivors, invalid input/time domains, unsupported slices, capacities, precision-lost absolute times, numerical placement failure, descriptor-order independence and preservation of unaffected events/RNG. Three unit fixtures cover queue time validation, total ordering and ordinal exhaustion. Initial test-first run failed because history APIs and typed errors were absent; the focused suite passes after implementation.

Independent review: gpt-6.1-sol, high effort, reviewed frozen tracked and untracked changes against base d6f2ec3aba90c84d2b27a95809ca60c19981f8b8, ticket criteria, surrounding kernels and Geant4 source. No actionable findings; verification relies on recorded successful commands. Accepted for isolated-pair scope; later ticket limitations remain explicit.

## Blockers / Challenges

None outstanding. `.claude/.claude-project.json` and the specified user CONFIG_RESOLUTION.md were absent; no replacement config was invented. The authoritative repository audit and development-state documentation identify the existing 11.4.1 source tree, which was read directly. No permission denial occurred. One verification tool dispatch was aborted before starting and was retried successfully.

## Commits

- 0cbeccc feat: add isolated pair history (ticket 01)

## Time Spent

16m (ticket-start.js to ticket-complete.js)
