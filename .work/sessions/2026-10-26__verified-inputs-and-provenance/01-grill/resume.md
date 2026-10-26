# Session: verified-inputs-and-provenance

**Date:** 2026-10-26T18:32:06.802Z
**Status:** Grill phase complete

## Problem Statement

Production electron models can be built from unchecked table files, while output APIs accept a separately supplied manifest. A result can therefore claim an input identity unrelated to the data that shaped its trajectory. Malformed component layouts, numeric overflow and invalid transport inputs can also change sampling, panic or bypass validation. R02 establishes a verified construction boundary and truthful provenance before later chemistry and simulation milestones rely on these inputs.

## Context & Constraints

- **Current behavior:** `DataDir::load` is unchecked and production `from_data_dir` constructors use it. `run` and `run_with_physchem` accept a manifest independent of their already-built models; custom chemistry output reports the embedded option3 hash. Some table constructors and interpolation paths accept invalid layouts or numerical domains.
- **Pain point:** A changed table can retain an old output identity; extra or missing components can make totals disagree with sampled channels; malformed angular/ionisation tables, a narrow energy grid, non-finite positions or invalid ledger tolerance can panic or evade checks.
- **Dependencies:** R01 `validation-evidence-and-ci` is complete and its maintained gates must pass. Preserve the five-crate ownership, the physics-only `g4-sim` profile, valid default scientific values and RNG consumption. The C++ Geant4 11.4.1 source and committed data are the reference; do not change the C++ repository or reference fixtures.
- **Tech stack:** Rust workspace crates `g4-core`, `g4-data`, `g4-physics`, `g4-chemistry` and `g4-sim`; existing manifest and SHA-256 support, serde JSON output and Cargo lockfile.

## Success Metrics

- Production construction verifies every manifest entry, rejects a wrong manifest, missing file or checksum mismatch before transport, and parses the same checked bytes. Changed bytes cannot retain an old verified identity; relocating the checkout/data directory preserves semantic identity.
- Catalogued and direct model constructors reject too few or too many components. Validated partial and total cross sections describe the same channels; scaled values and sums cannot overflow silently. Malformed zero-column angular and short-column ionisation tables return typed errors. The accepted narrow endpoint grid `[1.0, 1.0000000000001]` yields bounded interpolation or a typed error, never a panic.
- Non-finite primary positions and invalid ledger tolerances fail with typed errors. Custom chemistry identity hashes validated engine values. Provenance records implementation revision/dirty state, compiler/target/platform, lockfile and actual input hashes, seed and numerical/model policies. Output APIs cannot attach independent caller-supplied manifest metadata to an already-constructed model.
- Explicit synthetic/unverified test construction remains available and is visibly marked in output. Valid default scientific results and RNG consumption remain unchanged. Focused regression/property tests, R01 maintained gates, workspace checks and crate-boundary/physics-only checks pass.

## Architecture & Approach

Introduce an immutable verified input/model bundle for production orchestration. Verify the full declared manifest and parse consumed tables from the same checked byte buffers, so verification and parsing cannot observe different file contents. Bind the verified input identity to the resulting models and have output-producing `run` APIs require that bundle, forcing existing library callers to migrate. Keep separately named explicit unverified paths for synthetic tests, and label their output as unverified. Validate catalog layouts and direct model component counts at construction; check finite, nonnegative and representable scaled values, sums, interpolation domains, positions and ledger tolerances before use. Keep chemistry-specific validation and a stable hash of actual validated species, channels and parameters in `g4-chemistry`; retain the embedded option3 source hash as additional evidence. Build the provenance envelope from the constructed bundle and chemistry engine, with path-independent semantic identities and structured implementation/toolchain/lockfile/policy fields.

## Assumptions & Trade-offs

The existing option2 production profile verifies all 20 manifest entries, including tables it does not currently consume. Public output API migration is intentional. Programmatically assembled custom chemistry is identified by canonical validated values, since source bytes need not exist. Dirty builds remain distinguishable through recorded state. No physics formula, scientific constants, model windows, reference file, data format or additional scientific model changes are in scope; avoid generic frameworks and new dependencies without justification.

## Open Questions

No design questions remain. Implementation must confirm exact source-backed component layouts and the established R01 gate commands before changing constructors.

## Notes

The scout seed traces R02 to audit findings F1-F3 and F6 in `docs/reviews/development-state.md` and the acceptance contract in `docs/reviews/proposed-roadmap.md`. The current glossary and ADRs preserve canonical Geant4 chemistry names and option3 semantics. Geant4 11.4.1 Born ionisation code reads five shells and Sanche code reads nine levels; these are source-backed layout constraints. Keep chemistry out of `g4-core`, `g4-data`, `g4-physics` and the physics-only simulation profile.
