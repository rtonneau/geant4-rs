# Ticket 01: source-faithful-sbs-kernel

**Status:** ✅ Done

## Local Test Result

`cargo test -p dna-chemistry --test sbs` passed: 10 tests passed, 0 failed. `git diff --check` also passed. Evidence from the implementer run is preserved at `.scratch/tests/2026-10-23__m7-sbs-oracle-and-ode/evidence/ticket-01-sbs-tests.log`.

## Review Notes

Reviewed by the parent session after implementation by `gpt-6-astra/high`. The module uses the executable Geant4 encounter constant, calls the existing diffusion kernel once per particle, keeps inclusive contact, and restricts the bridge draw to observation-capped steps. The deterministic suite covers all acceptance boundaries and RNG draw counts. The source's resumed-loop coordinate double-count in `FindReaction` is reproduced only inside the validation oracle, documented, and pinned by a test whose outcome differs from the rotationally invariant formula. No follow-up edit was required.

## Blockers / Challenges

Geant4 11.4.1 double-counts the first coordinate that exceeds the radius test when it resumes the Brownian-bridge distance loop. Ruling: preserve the executable quirk because this module is a source-faithful reference oracle; isolate and document it rather than correcting it or propagating it into production APIs.

## Commits

- bd3b2b1 feat: add source-faithful SBS oracle (ticket 01)

## Time Spent

7m (ticket-start.js to ticket-complete.js)
