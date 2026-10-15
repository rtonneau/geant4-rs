# Geant4 reference CSVs (not yet generated)

This directory is where the Geant4 11.4.1 reference files go. They must be produced offline with
`validation/geant4/g4rs_ref` (instructions: `../geant4/README.md`) and committed together with
`provenance.toml`. Until then the tests in `crates/dna-sim/tests/geant4_reference.rs` are `#[ignore]`d.

Expected files: `xs_per_volume.csv`, `mfp_first_interaction.csv`, `track_cut100eV.csv`,
`provenance.toml`.

Do not hand-edit or synthesise these files: a reference number that did not come from Geant4 makes the
comparison meaningless (AGENTS.md: never invent reference values).
