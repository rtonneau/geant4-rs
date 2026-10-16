# Next round

## (i) What you need to do next

### 1. Repair and run the M6 Geant4 reference

Branch `m6-physchem-b` contains the Rust M6 implementation and a compiled `g4rs_ref physchem` mode, but the M6 validation gate is **not closed**. A local 2 x 1 keV smoke run against Geant4 11.4.1 exits with Windows access violation `-1073741819` when `G4DNAChemistryManager::Run()` starts; physical-only `slow` still exits 0. Fix the compact action/counter/scheduler lifecycle line-for-line against `examples/extended/medical/dna/chem6` (Q24). Do not accept an empty or crash-produced CSV.

Once the smoke run exits 0 and contains species rows, run the exact command and provenance recipe in `validation/geant4/README.md`. Commit exactly:

- `validation/reference/physchem.csv`
- `validation/reference/provenance_m6.toml`

Then run `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture`. Never loosen its pre-registered tolerances merely to pass. Diagnose differences as data/tag parsing, count-time semantics, spatial RMS semantics, event ordering/recombination (Q21/R14), or a model bug.

### 2. Decisions before M7

- Confirm that M7 should implement option3 reactions first and keep the same-decay M6 pair statistic distinct from any future RDF/nearest-neighbour observable (R16).
- Keep Q12 (atomic de-excitation skipped), Q13 (Geant4 elastic rotation default), Q16 (2 eV floor), and Q17 (Meesungnoen2002 only) unless the M6 reference provides contrary evidence.
- Decide whether M7 should begin before the M6 Geant4 gate closes. Recommended: repair/run M6 first because recombination and initial positions are M7 initial conditions.

### 3. Branch/PR

Review `m6-physchem-b`. No PR has been requested or opened. Ask for a PR only after the M6 reference gate is closed, or explicitly accept an open validation gate.

## (ii) Draft prompt for M7

```text
Start milestone M7 of geant4-rs: option3 water-radiolysis chemistry (reaction data, diffusion and IRT), one validated slice at a time. Read AGENTS.md, docs/next.md, docs/08 M6/M7, docs/02 B.7.6-B.7.7, docs/03, docs/04 D.2, docs/05, docs/07 V9-V10, docs/09 and docs/reproducibility.md first. Do not push main or open a PR unless asked; the Geant4 C++ repository is read-only.

Gate check: report whether the M6 `physchem` Geant4 reference is present and whether `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture` passes. If the generator still access-violates (Q24), do not invent reference numbers or weaken tolerances; state that M7 proceeds on an unclosed initial-condition gate only if I explicitly approve.

Scope this session to one M7 slice:
1. Port the option3 reaction table to validated TOML, including reactants/products, reaction type, k_obs/k_act and source line provenance. Extend the existing species table only where the reaction network requires it.
2. Implement free Brownian diffusion with deterministic per-history chemistry streams and an isolated two-body IRT sampler for A+B and A+A, including partially diffusion-controlled reactions and explicit domain/error policies. Preserve the 1 ps initial species positions from M6. Do not yet build the full many-species event scheduler unless this slice is validated first.
3. Add V9/V10 unit/statistical tests: analytic survival/reaction-time laws over three pre-registered D/R/k sets, A+A symmetry/no-self-pairing, atom/charge conservation for every reaction, boundary/extreme-value tests, thread-count independence, and an SBS oracle for one totally diffusion-controlled case. State Bonferroni families and thresholds before running.

Consult and cite the exact Geant4 11.4.1 sources (`G4EmDNAChemistry_option3`, reaction table, IRT/SBS models and utilities); record every intentional deviation. No diffusion/reaction constant may be guessed. Keep physics -> events -> physchem -> species -> chemistry separation and update docs/02, docs/03, docs/04, docs/08, docs/09, docs/10 and docs/reproducibility.md.

Run and report cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, and cargo test --workspace. Commit in small steps and push the feature branch; do not claim Geant4 validation that was not executed.
```
