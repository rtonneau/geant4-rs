# Session: m6-residual-rust-audit

**Date:** 2026-10-17T08:57:50.364Z
**Status:** Grill phase complete

## Problem Statement

RR-001 (2026-10-17) closed with the stage-matched M6 gate failed on 16 of 28 species/energy rows, all in the z <= 4.5 test (none in the 15% count or 20% RMS tests). Rust vs Geant4 offsets are 1-5%: H3O+ and e_aq high (up to +1.8% at 1 keV), H low (-2 to -4%), H2 and O low (up to -5%), OH passes except at 1 keV. The responsible stage (ionisation/excitation partition, dissociation channel fractions, DA channel, recombination, thermalisation) is unknown. Find the stage family using only existing data and the Geant4 11.4.1 sources, fix a source-proven Rust defect if one exists, and record the result.

## Context & Constraints

- **Current behavior:** `m6_prechem_counts_and_rms_match_geant4` (ignored, `crates/dna-sim/tests/physchem_reference.rs`) fails with the unchanged criteria. The Geant4 CSV `validation/reference/physchem_prechem.csv` holds only seven species (sum, sum2, sum r2) per energy from one seed (20261020); the Rust side uses master_seed 20261017 in-process. No ionisation/excitation/attachment counts exist on the Geant4 side.
- **Pain point:** M6 is not closed and M7/M8 work consuming these initial conditions stays gated.
- **Dependencies:** Reference files in `validation/reference/` and the Geant4 C++ repo are never modified. No tolerance is loosened (z <= 4.5, 15% count, 20% RMS). No invented constants. Rules of `docs/gps-backlog.md` and `validation/geant4/RUN_REQUESTS.md` apply; agent sessions never run Geant4.
- **Tech stack:** Rust workspace (`dna-core`, `dna-physics`, `dna-chemistry`, `dna-sim`), rayon, rand_chacha; Geant4 11.4.1 `.cc` sources as reference.

## Success Metrics

- A short "M6 residual audit" section in `docs/08-roadmap.md` names the responsible stage family (a relation holds on Geant4 and fails on Rust, other families excluded) or states "undecided -> `m6-process-counts-geant4-request`"; a Q21/R14 note is added to `docs/09-open-questions-and-risks.md`; backlog seeds are updated (keep or skip `m6-process-counts-geant4-request` per the verdict).
- Conservation relations (charge balance with +1 e_aq from the thermalised primary, H and O budgets per channel) are tested on both the Geant4 CSV and 5 extra Rust master seeds at all four energies; 4.5 sigma tolerances are fixed in the test header before results are evaluated; per-species mean +- spread and the z vs 1/sqrt(N) behaviour are reported (fallback: 2 seeds at 900 keV if runtime is prohibitive, stated explicitly).
- A source-comparison table (file:line, each row identical / differs / undecidable) covers `G4DNAWaterDissociationDisplacer`, `G4DNAWaterExcitationStructure`, `G4DNAElectronHoleRecombination`, `G4EmDNAChemistry_option3.cc`, and the mapping from each physics sub-process (ionisation shell, excitation level, DA) to a chemistry channel.
- If a Rust defect is source-proven and consistent with the data: fixed with its own test and commit, gate `m6_prechem_counts_and_rms_match_geant4` rerun with unchanged criteria and the pass/fail reported honestly; M6 closes only if all 28 rows pass; changed Rust-side baselines are re-baselined with documented justification.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` run and reported.

## Architecture & Approach

New `crates/dna-sim/tests/physchem_audit.rs` with `#[ignore]`d tests that dump Rust sum/sum2/sum r2 per species for multiple seeds, compute the relations on both sides and print the table; no new public API, no production code or reference file touched by the audit itself. Rust side of each relation uses its own seed-to-seed spread; the Geant4 side uses the conservative uncorrelated sum-of-variances bound (species within an event are correlated and covariance is unavailable, so this makes the test weaker; the limitation is stated in the table). Source re-read covers the chemistry-side files plus the process-to-channel mapping; thermalisation/solvation only if the relations point there. Verdict bar: stage family named if a relation holds on Geant4, fails on Rust, and other families are excluded by passing relations; a source difference alone is only a candidate.

## Assumptions & Trade-offs

- Assumes events are independent for variance estimates; per-event species covariance is unavailable from the CSV.
- Fix-in-session is allowed only when a Geant4 .cc line proves the Rust difference and the data agree; otherwise document and open a separate seed. This goes beyond the backlog's "No model is changed in this session"; the backlog text is updated to match.
- Not doing: new Geant4 runs, loosening tolerances, editing reference files, auditing thermalisation unless indicated, the `physchem_counts` generator (separate seed).

## Open Questions

- Whether the existing data can decide at all; if not, the verdict is "undecided" and `m6-process-counts-geant4-request` proceeds.
- Runtime of 5 seeds x 4 energies at 900 keV (gate took about 255 s for the full suite).

## Notes

Charge check on Geant4 1 keV: H3O+ - e_aq - OH- is about -1.0 per event, consistent with the thermalised primary adding one e_aq, so relations must account for it. Origin: `docs/gps-backlog-m6-residual.md`, section `m6-residual-rust-audit`. Evidence: docs/08 "RR-001 result", `provenance_m6_prechem.toml`.
