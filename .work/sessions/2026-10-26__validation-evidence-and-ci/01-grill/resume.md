# Session: validation-evidence-and-ci

**Date:** 2026-10-26T09:42:13.123Z
**Status:** Grill phase complete

## Problem Statement

Scientific acceptance is not fully executable or fail closed: maintained reference comparisons are ignored by ordinary CI, several readers can accept incomplete or invalid evidence, and compiler support and current validation status are described more broadly than verified.

## Context & Constraints

- **Current behavior:** CI runs formatting, Clippy and ordinary workspace tests on Ubuntu. The four M2, two M3 and named stage-matched M6 reference tests require explicit ignored-test selections. The pinned toolchain is Rust 1.97.0 while `rust-version` says 1.85.
- **Pain point:** Missing or malformed committed data can be skipped or give a false pass; historical comparisons and current gate status are easy to confuse.
- **Dependencies:** Use existing committed Geant4 11.4.1 fixtures and preserve their bytes, provenance, scientific thresholds, exclusions and qualified M6 closure. Do not run a Geant4 generator or blanket-run ignored diagnostics. R01 precedes the approved S01 crate-domain-ownership implementation.
- **Tech stack:** Five-crate Rust workspace, pinned Rust/Cargo 1.97.0, existing Rust integration tests, small versioned gate inventory/runner, GitHub Actions on Linux and Windows.

## Success Metrics

- A versioned maintained-gate inventory and runner execute exactly four M2 tests, two M3 tests and the named M6 prechem test; historical post-zero-time M6 gates and scratch diagnostics remain excluded.
- Negative-fixture tests reject missing, empty, truncated, duplicate or incomplete rows, invalid numeric values/moments, wrong units/constructor/stage/seed metadata and mismatched hashes. Exact grid coverage and deterministic zero-standard-error unequal-mean rejection are proven; existing valid fixtures retain their historical verdicts and thresholds.
- Required-data CI fails when committed fixtures are removed and reports Linux/Windows and pinned compiler status honestly. The full statistical tier has an explicit manual entry point. Current docs distinguish maintained results, historical exclusions and RR-002's local-artifact limit.

## Architecture & Approach

Build a small versioned inventory of maintained tests, fixture identities/profiles and exclusions, with a runner that checks required evidence and invokes only registered selections. Tighten the existing reference readers and comparison boundaries, adding focused negative fixtures and tests before changes. Enforce the required-data tier in CI on Linux and Windows, keep the full statistical tier explicitly callable, align `rust-version` with the pinned 1.97.0 policy, and update current-status documentation. Preserve existing numerical criteria and investigate any changed verdict.

## Assumptions & Trade-offs

The committed fixtures are the evidence source; R01 creates no new reference campaign, physical algorithm or general workflow engine. The current pinned Rust 1.97.0 is the supported compiler claim until older versions are tested. The stage-matched M6 gate remains distinct from historical post-zero-time comparisons. Existing M6 closure is not reopened by a new harness defect; any regression is reported separately.

## Open Questions

None in the approved R01 scope. CI results for an unavailable platform must be recorded as unverified rather than claimed passed.

## Notes

On unchanged `main`, `cargo test --workspace --locked` passed, as did the maintained ignored selections: M2 4/4, M3 2/2 and named M6 prechem 1/1. These are local Windows baseline results, not CI or cross-platform claims. The existing roadmap's R01 scope was approved for this session.
