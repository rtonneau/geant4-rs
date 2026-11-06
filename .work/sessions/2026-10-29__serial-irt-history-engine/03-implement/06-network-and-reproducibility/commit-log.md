# Ticket 06: network-and-reproducibility

**Status:** ✅ Done

## Local Test Result

- `cargo test -p g4-chemistry --test history_network`: PASS, 3 passed.
- `cargo test -p g4-sim --test history_reproducibility`: PASS, 1 passed.
- `cargo fmt --all -- --check`: PASS (exit 0).
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (exit 0).
- `cargo test --workspace`: PASS (exit 0), 555 passed, 0 failed, 21 ignored
  across 65 test targets including doc tests. Full output:
  `.scratch/tests/2026-10-29__serial-irt-history-engine/ticket06-workspace.log`.
- `cargo test -p g4-sim --no-default-features`: PASS (exit 0), 38 passed,
  0 failed, 7 ignored across 10 test targets including doc tests. The new
  feature-gated history wrapper compiles with zero tests in this profile.
  Full output: `.scratch/tests/2026-10-29__serial-irt-history-engine/ticket06-physics-only.log`.
- `git diff --check`: PASS; only the pre-existing session-config line-ending
  warning was emitted.

The ignored reference/statistical tests retain their existing opt-in status;
these commands do not claim execution of a Geant4 generator or passage of G2.

## Review Notes

Base: 638dc62991ec375cedad5667ed4de8dd14d48a1e. Pre-existing changes: GPS session config and newly started ticket log only; project tree clean. Recovery count: 0.

Added full option3 mixed-species execution tests that reconstruct all observed
populations from parent/product lineage and require pair/unary, descendant,
TDC/PDC and neutral/attractive/repulsive execution coverage. Added typed
fail-fast checks for unsupported PDC rates, changed source signatures and
out-of-domain horizons, including zero draws before rejection.

Added a feature-gated, test-only g4-sim wrapper over the serial engine: 64
histories (IDs 37..101 exclusive), seed 20261106016, 30 synthetic molecules,
six observation times through 1 us. Full result fields, raw float bits and
16-value continuations of all three stage streams match across 1/2/8 threads,
batches 1/7/64/128 and forward/reverse submission, independently for indexed
and all-pairs search. Physics/PhysChem streams are not consumed by chemistry.
No production integration, dependency or engine changes.

Updated reproducibility documentation with draw ordering, workload scope and
explicit separation from R05/G2, physical validation and R06 production work.
Read G4DNAIRT.cc and G4EmDNAChemistry_option3.cc from local reference tag
v11.4.1 (f01f313f122a19ae77c6f27192c9464ba0279b5e), rather than the newer
reference working tree. The 11.4.3 G1-F evidence does not change that baseline.

Independent review: gpt-6-astra, high effort, reviewed frozen tracked and untracked changes against base 638dc62991ec375cedad5667ed4de8dd14d48a1e, ticket criteria, scheduler and Geant4 source. No actionable findings. Saved logs support workspace and physics-only results; formatting and Clippy recorded by implementer. Accepted; G2, physical validation and production integration remain open.

## Blockers / Challenges

None. No Geant4 generator was run and no reference data or source was modified.

## Commits

- 516cfe1 test: verify history network reproducibility (ticket 06)

## Time Spent

11m (ticket-start.js to ticket-complete.js)
