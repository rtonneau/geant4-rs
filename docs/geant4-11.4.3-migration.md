# Geant4 reference bump: 11.4.1 → 11.4.3

Date: 2026-11-13. From this date the project reference is **Geant4 v11.4.3**. New source audits cite
the 11.4.3 release source. Audits dated before this note cite 11.4.1 and stay as written: they are
provenance, not stale text.

## Existing reference data

RR-001 … RR-008 data and their `provenance*.toml` files under `validation/reference/` were produced
with Geant4 **11.4.1**. They are kept unmodified (AGENTS.md forbids overwriting reference files) and
remain valid for comparison because the source differences below do not touch the code paths they
exercise. Re-running them under 11.4.3 is a separate decision for the user. Any re-run writes **new**
files.

## Source differences (release trees, `source/processes`)

Method: `diff -rq` between the unpacked 11.4.1 and 11.4.3 release sources.

| Area | Changed files | Relevance to this project |
|---|---|---|
| `electromagnetic/dna/**` | none (byte-identical) | none: all DNA physics, chemistry, IRT and scheduler code is unchanged |
| `cmake/Modules/G4DatasetDefinitions.cmake` | none | data-set versions (G4EMLOW etc.) unchanged |
| `transportation/src/G4Transportation.cc` | field configuration moved to `ConfigureFieldForTrack`, cached at track start | none: no fields are used. World-exit kill (`:714-716`) is unchanged in substance |
| `management/src/G4ProcessTable.cc` | minor | none observed for discrete DNA processes |
| `electromagnetic/utils` | `G4EmParameters` print flag made atomic (MT race); `G4EmCalculator` ion corrections; `G4ionEffectiveCharge` refactor and reproducibility fix | none for electrons. `G4ionEffectiveCharge` is referenced nowhere under `electromagnetic/dna` (grep; Rudd and Miller-Green use their own Slater charges). Re-check if ion support is added |
| `electromagnetic/standard`, `xrays` | positron annihilation models, scintillation | none |

Conclusion: no physics or chemistry behaviour in the current scope differs between the versions.

## Code follow-up (not done in this documents-only change)

The code still pins 11.4.1: `GEANT4_REFERENCE_VERSION` in `crates/g4-core/src/provenance.rs:12`
(written into every output envelope, see `docs/reproducibility.md`), the `geant4_release` check in
`crates/g4-chemistry/src/reactions.rs:199` against the embedded chemistry files, and the
`geant4_version` provenance checks in the `geant4_*` comparison tests (which correctly describe the
11.4.1 reference data). Bumping the constant changes the provenance envelope and must be its own
change. The embedded-data and test checks should keep 11.4.1 while their data is 11.4.1.
