# Geant4 reference CSVs

Geant4 11.4.1 reference output (`G4EmDNAPhysics_option2`, G4EMLOW 8.8), produced offline with
`validation/geant4/g4rs_ref` (instructions: `../geant4/README.md`, build notes:
`../geant4/BUILD_NOTES.md`). Provenance: `provenance.toml`.

| File | Used by |
|---|---|
| `xs_per_volume.csv` | V2/V3 test |
| `mfp_first_interaction.csv`, `track_cut100eV.csv` | V5 tests (seed 1) |
| `*_seed2.csv` | seed-2 consistency test |
| `*.log`, `*.err`, `exitcodes.txt` | raw run output |

The comparison tests are `#[ignore]`d only so that a plain `cargo test` stays fast; run them with
`cargo test -p dna-sim --test geant4_reference -- --ignored --nocapture` (prints tolerance margins).

Do not hand-edit or synthesise these files: a reference number that did not come from Geant4 makes the
comparison meaningless (AGENTS.md: never invent reference values).
