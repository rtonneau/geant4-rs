# Implementation Plan

**Session:** verified-inputs-and-provenance
**Date:** 2026-10-26T18:36:38.600Z
**Estimated effort:** 4-7 days

## Strategy

Harden parsed data first, then capture and verify the full manifest from one byte snapshot. Build the production option2 model through that snapshot. Add transport domain checks and chemistry identity before changing output APIs. Finally bind all identities and build metadata to output, migrate callers, and run the scientific gates. Each ticket ends with passing targeted tests and a compilable workspace.

## Tickets Overview

- **Ticket 1:** Reject malformed table layouts and unrepresentable cross sections at construction.
- **Ticket 2:** Verify the full manifest from one immutable byte snapshot and construct option2 models from it.
- **Ticket 3:** Reject invalid transport positions, ledger tolerances and macroscopic cross sections.
- **Ticket 4:** Derive custom chemistry identity from validated engine values.
- **Ticket 5:** Record build, lockfile, input and model-policy provenance without machine paths.
- **Ticket 6:** Require verified inputs in output APIs, migrate callers, and run all R01 and crate-boundary gates.

## Sequencing Rationale

Tickets 1-3 close scientific input defects before a verified label can be issued. Ticket 4 makes chemistry identity independent of the default embedded data. Ticket 5 supplies the output envelope. Ticket 6 makes that envelope mandatory for production and proves that the existing scientific outputs and RNG streams remain unchanged.

## Risks & Mitigation

- **Risk:** Verification and parsing read different file versions. → **Mitigation:** Hash, validate size and parse from one captured byte buffer per file; test byte mutation after snapshot capture.
- **Risk:** Stronger checks perturb valid interpolation or sampling. → **Mitigation:** Preserve formulas, compare valid default outputs and RNG consumption, and run the maintained Geant4 gates.
- **Risk:** Custom chemistry hashes depend on map order or paths. → **Mitigation:** Canonicalize validated semantic fields in stable source order and test equivalent objects and relocations.
- **Risk:** Breaking public APIs leaves an unchecked output path that looks verified. → **Mitigation:** Migrate callers at compile time and mark the separate synthetic path as unverified in serialized output.

## Assumptions

- R01's maintained gate runner is `python scripts/run_validation_gates.py --tier required`.
- Full-manifest verification includes all 20 files even when the option2 electron model consumes fewer; the checked byte snapshot is about 7.3 MB with the current data.
- New code must preserve the physics-only `g4-sim --no-default-features` build and must not add chemistry dependencies to `g4-core`, `g4-data` or `g4-physics`.
- Every ticket may change its named tests and nearby rustdoc; no Geant4 generator or committed reference data changes are needed.
