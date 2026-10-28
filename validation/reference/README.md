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

RR-004 R04 option3 IRT pair evidence is **REQUESTED**. The proposed new files
are `pair_matrix_irt_r04_n100000.csv` and
`provenance_r04_pair_matrix_n100000.toml`; neither is present yet. The
144-endpoint consumer is `crates/g4-chemistry/tests/geant4_pair_r04.rs` and
remains ignored during ordinary tests. After a complete user run and committed
provenance, invoke:

```text
cargo test -p g4-chemistry --test geant4_pair_r04 r04_option3_irt_matches_geant4 -- --ignored --exact --nocapture
```

The consumer requires exact ordered row coverage, option3 parameters,
independent seed and pair count, completion marker, clean generator commit,
and matching CSV SHA-256. It applies the frozen separate 144-point family in
`docs/validation/pair-case-matrix.md`. Missing or invalid evidence never
closes G1. The older V9 analytic verdict and RR-003 H+H SBS request are
unchanged.

The comparison tests are `#[ignore]`d only so that a plain `cargo test` stays fast; run them with
`cargo test -p dna-sim --test geant4_reference -- --ignored --nocapture` (prints tolerance margins).

Do not hand-edit or synthesise these files: a reference number that did not come from Geant4 makes the
comparison meaningless (AGENTS.md: never invent reference values).
