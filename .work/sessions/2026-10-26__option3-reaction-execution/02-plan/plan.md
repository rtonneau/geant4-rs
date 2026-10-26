# Implementation Plan

**Session:** option3-reaction-execution
**Date:** 2026-10-26T20:47:09.813Z
**Estimated effort:** 8–12 working days

## Strategy

Compile the source table first so every later primitive receives validated, typed inputs. Add charged pair sampling, unary clocks, and placement in separate testable steps. Finish with a source-line coverage matrix and the full scientific and workspace verification gates. Each ticket leaves the crate compilable.

## Tickets Overview

- **Ticket 1:** Compile all 72 reactions into explicit, validated pair or unary descriptors.
- **Ticket 2:** Implement and independently check charged-PDC reaction-time sampling.
- **Ticket 3:** Implement first-order clocks and competing unary hazards.
- **Ticket 4:** Implement source-order product and conditional parent placement.
- **Ticket 5:** Complete ID-level coverage, conservation, documentation, and workspace gates.

## Sequencing Rationale

Network compilation establishes the types and validation used by all other tickets. The pair and unary samplers can then be checked independently. Placement consumes compiled reaction outputs. The final coverage pass checks the complete network without introducing a scheduler.

## Risks & Mitigation

- **Risk:** Charged-PDC expressions have singular and limiting cases. → **Mitigation:** Pin independent high-precision anchors and explicit domain errors before adding the sampler; retain existing TDC and neutral-PDC regression tests.
- **Risk:** Geant4 conditional placement contains a dimensionally suspect Gaussian argument. → **Mitigation:** Reproduce the literal expression as approved, record the limitation, and test deterministic source-formula cases.
- **Risk:** A zero-rate custom unary entry could be silently omitted. → **Mitigation:** Keep an explicit disabled descriptor, with the embedded option3 table validated strictly.
- **Risk:** Bulk bookkeeping or reaction 53 could mask conservation failures. → **Mitigation:** Account for reservoir exchanges channel by channel and permit only the exact existing reaction-53 exception.

## Assumptions

- R02 is on `main`; Geant4 11.4.1 source and the existing option3 data remain normative.
- This plan changes only `g4-chemistry`, its tests, and chemistry documentation. It does not build a multispecies scheduler, arbitrary scavenger configuration, boundaries, IRT-syn, or new spin channels.
- Existing source literals, pair behavior, caller-owned RNG, typed units, and no new dependencies are retained.
