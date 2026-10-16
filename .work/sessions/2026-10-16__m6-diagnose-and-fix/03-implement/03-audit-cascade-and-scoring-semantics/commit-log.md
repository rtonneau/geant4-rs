# Ticket 03: audit-cascade-and-scoring-semantics

**Status:** ✅ Done

## Local Test Result

Ran the ticket verification command in the prescribed scratch target (`CARGO_TARGET_DIR=.scratch/tests/2026-10-16__m6-diagnose-and-fix/cargo`): `cargo test -p dna-sim physchem` passed (five matching tests), followed by `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture`. The latter finished in 255.40 s with **3 passed, 1 failed**, exit 101: the snapshot diagnostic, complete 60-row comparison and full four-energy ordering ablation passed; the unchanged original agreement gate failed at `1000 H2O2: z=111.55722825815035`, precisely the baseline. The ticket explicitly requires capturing that unchanged gate result, not making an after-zero-time-reactions reference match a dissociation-only engine. Report values are unchanged.

Also passed `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (**268 passed, 0 failed, 12 ignored**, exit 0). The full workspace run executes all five physchem_b tests, including its two previously existing names that the `physchem` filter does not select. `git diff --check` passed; git diff confirms the original gate, generator and all reference evidence are untouched. Saved logs: `.scratch/tests/2026-10-16__m6-diagnose-and-fix/ticket03-reference.log` and `ticket03-workspace.log`.

## Review Notes

- Re-read recent commits, the named runner/scorer/tests/generator/docs, and authoritative Geant4 11.4.1 source. Traced complete secondary transport, generator atomic-de-excitation removal, Champion's rotation, primary validity limits, the solvation factory/default, fixed global species births, and removal of killed chemical parents. No transport or scoring defect was established; no numerical implementation changed.
- Added `physchem_counts_and_radius_use_survivors_at_global_one_ps`: deterministic off-origin recombination consumes an electron at 101 nm and retains one at exactly 10 nm. It distinguishes produced counts from alive counts, checks all birth times, and separately validates H/OH radius from primary zero and displacement from their 100 nm parent event.
- Added `physchem_cascade_follows_all_electrons_without_a_secondary_floor`: completed cascades track every created secondary through solvation/attachment, retain zero abandoned energy, and solvate sub-2-eV electrons. The primary floor accepts 2 eV and rejects 1.99 eV. Documented the important qualification that Geant4 has no matching 2 eV tracking cut: equivalence holds at all committed primary energies; the floor is Rust's input-domain policy only.
- Documented Q12/Q13/Q16/Q17 and scoring evidence in docs/08 and docs/09, including exact Geant4 source paths and line numbers. Clarified runner/scorer rustdoc: global 1 ps, primary at zero, post-recombination alive population, and the reference's additional zero-time reactions. Q25/R17 remain open; equal count time does not imply equal stages.
- Reviewed scope and whitespace. The asserting gate, tolerances, committed reference CSV/provenance and existing generator mode are byte-for-byte unchanged. No C++ changes, generator runs, reference regeneration, dependency additions or M7 implementation were needed.

## Blockers / Challenges

No task blocker. The unchanged agreement gate's expected scientific failure is retained as evidence, as required by this ticket; M6 remains open. A Windows incremental-cache finalization warning reported Access is denied during workspace compilation; it did not stop compilation. Shared COMMON_CLAUDE.md was read with approved escalation. The optional .claude project config and ~/.claude/CONFIG_RESOLUTION.md were absent; the reference source was located from the configured DEV_DIR and its 11.4.1 source directory, without modifying it. Unrelated .gitignore/.VSCodeCounter changes were preserved.

## Commits

- baf986c test: audit M6 cascade and scoring semantics (ticket 03)

## Time Spent

8m (ticket-start.js to ticket-complete.js)
