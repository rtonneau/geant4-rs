# Ticket 01: audit-n1-thermalisation-boundary

**Status:** ✅ Done

## Local Test Result

All ticket commands were run independently after the delegated implementation, with build and temporary artifacts under `.scratch/tests/2026-10-17__m6-n1-thermalisation-audit/`:

- `cargo test -p dna-physics --test low_energy`: exit 0, 21 passed.
- `cargo test -p dna-chemistry --test history solvation_creates_an_e_aq_at_the_event_position_without_random_numbers -- --exact`: exit 0, 1 passed.
- `cargo test -p dna-sim --test physchem_a species_conservation_and_untouched_ledger_on_real_cascades -- --exact`: exit 0, 1 passed.
- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace`: exit 0, 271 passed, 0 failed, 16 ignored.
- `git diff --check`: exit 0.

## Review Notes

Delegated implementation used `gpt-6.1-sol` at `xhigh`; the supervising session reviewed the complete diff and reran the verification without making follow-up edits. The review checked the Geant4 11.4.1 source/line trace, the active isotropic-safety semantics, the 1 m central reference geometry, the Rust transport-to-`PhysChemEvent` handoff, reuse of existing count and distribution tests, and the explicit limits of the Gaussian tail argument. N1 is closed only for infinite water and this committed workload; the text does not claim a hard displacement cutoff or measured zero clip count. N2, finite-volume support, the 900 keV RMS cause, and formal M6 closure remain out of scope. No production behavior, reference/provenance file, generator mode, tolerance, or run request changed.

## Blockers / Challenges

The first `ticket-start.js` attempt failed on a transient Windows `EPERM` atomic rename. A later user-authorized retry succeeded. The audit also corrected an initially over-strong use of "unreachable": because the Gaussian has unbounded support and the aggregate CSV has no clip counter or maximum radius, the final result states operational unreachability with a conservative conditional tail bound, not mathematical impossibility.

## Commits

- 920faa6 docs: audit M6 thermalisation boundary behavior (ticket 01)

## Time Spent

11m (ticket-start.js to ticket-complete.js)
