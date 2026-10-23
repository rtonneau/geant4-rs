# Ticket 02: molecule-and-diffusion

**Status:** ✅ Done

## Local Test Result

`cargo test -p dna-chemistry --test diffusion` passed: 7 passed, 0 failed, 0 ignored.

## Review Notes

Implementation by delegated model `gpt-6-sol` at high effort. Reviewed stable molecule identity, same-ID rejection, unchanged `SpeciesInstance`, Gaussian scale and draw order, zero-draw behavior, finite-domain validation, output overflow handling, and fixed-seed moment/isotropy coverage. Rand 0.10 exposes the caller-owned infallible stream through `rand::Rng`, which matches the existing Gaussian helper. Re-ran the verification in an isolated scratch target directory; no follow-up code changes were needed.

## Blockers / Challenges

None.

## Commits

- a43fcaf feat: add molecule identity and free diffusion (ticket 02)

## Time Spent

6m (ticket-start.js to ticket-complete.js)
