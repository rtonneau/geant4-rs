# Next round

## (i) What you need to do next

### 1. Diagnose the failing M6 Geant4 gate (reference data now exists)

State on branch `m6-physchem-b`:

- The `g4rs_ref physchem` crash (Q24) is **fixed** in C++ (`validation/geant4/g4rs_ref/g4rs_ref.cc`): IRT time-step model set explicitly, snapshot taken at the first scheduler step with `dt > 0` (after the zero-time dissociation steps at 1 ps), secondaries transported, `SetAtomDeexcitation(nullptr)` also in physchem mode. Details, symptoms and fixes: `validation/geant4/BUILD_NOTES.md` section 5 (issues G-K). Close Q24 in `docs/09` with a pointer there.
- `validation/reference/physchem.csv` (1 keV x 20000, 10 keV x 2000, 100 keV x 200, 900 keV x 20 events, seed 20261019, 17.5 min) and `validation/reference/provenance_m6.toml` exist and are plausible (about 40 `e_aq` and 50 `°OH` per 1 keV event). The provenance records the exact committed generator revision; the data must not be regenerated merely to update metadata.
- CSV species names are native Geant4 (`H3O^1`, `°OH^0`, `HO_2°^0`, ...); `Species::from_geant4_name` (`crates/dna-core/src/species.rs`) maps them; `physchem_reference.rs` uses it.
- **The gate fails**: `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture` stops at the first assertion, `1000 eV H2O2: z = 111.6` (Geant4 books 0.689 `H2O2` per event at 1 ps; Rust is far from it). The test stops at the first failure, so nothing else is known yet.
- One unexplained event: the first production attempt died at 900 keV with a stack overflow (`0xC00000FD`); a rerun of the same command with `/STACK:536870912` completed. If it recurs, capture a stack trace (BUILD_NOTES section 6) instead of raising the limit.

Tasks, in order (do not change any pre-registered tolerance; record the failure and evidence in `docs/08` first):

1. Add a **non-asserting comparison report** (an `#[ignore]`d test or small example that prints a table): per energy and species, Geant4 mean +- SE, Rust mean +- SE, z, relative difference, and RMS radius for both. This shows at once whether the offset is limited to a few species (`H2O2`, `O_2`, `H_2`, `H`, `OH^-1`...) or broad. Keep the asserting gate unchanged.
2. Diagnose from the table, one hypothesis at a time, checking the Geant4 source (not memory): dissociation-channel branching of the excited and ionised states incl. the `H2O2`, `H2`, `O` and `OH^-` channels (`G4ChemDissociationChannels_option1`), what the Rust M6 emits at 1 ps vs Geant4 (species born at exactly 1 ps vs later decays), recombination/event ordering (Q21/R14), count-time semantics, spatial RMS semantics (Geant4 `r` is from the primary origin, all alive molecules), and the known Rust/Geant4 differences already documented (Q12, Q13, Q16, Q17). Also check that Rust M6 starts the same cascade (secondaries followed to solvation, fluo/Auger off).
3. Fix model bugs or document each intentional deviation, then re-run the gate. Never loosen a tolerance merely to pass. Regenerating `physchem.csv` is **not** part of this task; if the generator itself looks wrong, say so and ask (AGENTS.md: reference data is never overwritten without being asked).

### 2. Decisions before M7

- Confirm that M7 should implement option3 reactions first and keep the same-decay M6 pair statistic distinct from any future RDF/nearest-neighbour observable (R16).
- Keep Q12 (atomic de-excitation skipped), Q13 (Geant4 elastic rotation default), Q16 (2 eV floor), and Q17 (Meesungnoen2002 only) unless the M6 reference provides contrary evidence.
- Decide whether M7 should begin before the M6 Geant4 gate closes. Recommended: diagnose and close M6 first (section 1) because recombination and initial positions are M7 initial conditions.

### 3. Branch/PR

Review `m6-physchem-b`. The requested PR documents the still-open M6 reference gate; do not merge it as a completed validation milestone until that gate is closed or the open gate is explicitly accepted.

## (ii) Draft prompt for M7

```text
Start milestone M7 of geant4-rs: option3 water-radiolysis chemistry (reaction data, diffusion and IRT), one validated slice at a time. Read AGENTS.md, docs/next.md, docs/08 M6/M7, docs/02 B.7.6-B.7.7, docs/03, docs/04 D.2, docs/05, docs/07 V9-V10, docs/09 and docs/reproducibility.md first. Do not push main or open a PR unless asked; the Geant4 C++ repository is read-only.

Gate check: the M6 `physchem` Geant4 reference exists (`validation/reference/physchem.csv`, `provenance_m6.toml`; Q24 is fixed, see validation/geant4/BUILD_NOTES.md) but the gate previously failed at `1000 eV H2O2: z = 111.6`. Report whether `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture` passes now. If it still fails, do not weaken tolerances or regenerate/overwrite reference data (AGENTS.md); state that M7 proceeds on an unclosed initial-condition gate only if I explicitly approve, otherwise do the diagnosis in docs/next.md section (i).1 instead.

Scope this session to one M7 slice:
1. Port the option3 reaction table to validated TOML, including reactants/products, reaction type, k_obs/k_act and source line provenance. Extend the existing species table only where the reaction network requires it.
2. Implement free Brownian diffusion with deterministic per-history chemistry streams and an isolated two-body IRT sampler for A+B and A+A, including partially diffusion-controlled reactions and explicit domain/error policies. Preserve the 1 ps initial species positions from M6. Do not yet build the full many-species event scheduler unless this slice is validated first.
3. Add V9/V10 unit/statistical tests: analytic survival/reaction-time laws over three pre-registered D/R/k sets, A+A symmetry/no-self-pairing, atom/charge conservation for every reaction, boundary/extreme-value tests, thread-count independence, and an SBS oracle for one totally diffusion-controlled case. State Bonferroni families and thresholds before running.

Consult and cite the exact Geant4 11.4.1 sources (`G4EmDNAChemistry_option3`, reaction table, IRT/SBS models and utilities); record every intentional deviation. No diffusion/reaction constant may be guessed. Keep physics -> events -> physchem -> species -> chemistry separation and update docs/02, docs/03, docs/04, docs/08, docs/09, docs/10 and docs/reproducibility.md.

Run and report cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, and cargo test --workspace. Commit in small steps and push the feature branch; do not claim Geant4 validation that was not executed.
```
