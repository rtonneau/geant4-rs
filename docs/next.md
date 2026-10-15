# Next round

Written after the M3 validation gate closed (branch `claude/inspiring-edison-v1kw1a`, **no PR opened yet**: open one
when you are ready; `main` is at `5c55626`, the merge of the M2 Geant4 reference data). Part (i) is for the human,
part (ii) is the draft of the next prompt for the coding agent. Update this file at the end of every round (the agent
is asked to do so).

## (i) What you need to do before the next round

### 1. Review and merge M3 (your call)
M3 is implemented, tested, and **now compared with Geant4 11.4.1** (gate closed 2026-10-15, details in docs/08 M3
status). Ask the agent to open a PR when you want one; CI (`fmt`, `clippy -D warnings`, `cargo test --workspace`)
has still never been observed on GitHub, all results come from local runs (the whole `cargo test --workspace` takes
about 100 s on 4 cores). If CI is red after the merge, paste the log into the next session first.

### 2. Geant4 M3 reference: DONE, nothing to generate
Committed in `validation/reference/`: `xs_lowenergy.csv`, `slow_{sums,spectra,radial}.csv`, second seed
`slow2_*.csv`, `provenance_m3.toml`. `cargo test -p dna-sim --test slowing_down_reference -- --ignored --nocapture`:
**2 passed** (about 45 s). Outcome:
- First run failed 3 comparisons, all artefacts of the comparison, not model bugs: V2 Melton attachment at 4 eV
  (Geant4 evaluates the 1e-300 floor in mm^2 as a subnormal, 1.63e-8 rounding; both physically zero) and two radial
  tail fractions (8 eV outside 10 nm, 900 keV inside 100 nm; Rust-sample error invalid for tails of a few
  histories). Fixes in the test: values < 1e-290 compare equal; radial comparisons with Kish n_eff < 10 on the
  minority side are skipped and logged. `Z_CRIT = 4` and the V2 tolerances are unchanged. The amendments were made
  **after** seeing the failures (stated in the test header and docs/08).
- Everything else passes: all V5 means, all 28 spectra chi-squares (lowest p = 0.0136, cut 3.6e-5).
- Geant4 agrees with the Rust W: ionisations per event 5.04 / 51.2 / 490 / 4796 at 100 eV / 1 / 10 / 100 keV
  (W = 19.5-21 eV), so the 20 % gap to the 25 eV prediction (R13) is the model, not a Rust bug.
- **Not tested** (8 radial comparisons skipped for lack of statistics): the far radial tails at 8 eV (> 10 nm),
  1 keV (100 nm), 100 keV (<= 1 nm), 900 keV (<= 100 nm). If these matter for M7/M8, a Geant4 run with more events
  at those energies (and per-event radial sums of squares in `g4rs_ref`) would close them; not a blocker.

### 3. Things to look at while you are at it
- A sync-tool duplicate `slowing_down_reference (# Name clash ... #).rs` appeared in `crates/dna-sim/tests/` and
  broke `clippy --all-targets`; it was deleted. Check your sync tool does not recreate it.
- The Sanche "kill-and-deposit" branch quoted in the M3 brief is commented out in 11.4.1 (docs/02 B.7.5); nothing
  to do unless you know of a reason it matters.
- `validation/rust_results/m3/*.json` are the Rust results for both elastic-rotation conventions (seed 20270103).

### 4. Decisions (answer in the prompt, or accept the recommended default)
| Id | Question | Recommended default |
|---|---|---|
| next milestone | M6 (physicochemical stage: electron radiolysis chain) or M5 (protons) first | **M6**: it turns the event record of M3 into species and unlocks M7/M8 for electrons; M5 can follow. M4 (options 4/6) stays optional (Q2) |
| Q12 | Auger/fluorescence after K-shell ionisation (0.25 % of ionisations, ~9 % of the binding energy booked) | keep skipping; revisit when the G(t) comparison exists |
| Q13 | default elastic rotation for science runs: `Geant4` (reproduces the quirk) or `Exact` | keep `Geant4` for comparisons; M6/M8 report both; effect so far: range -0.5 % (1 keV), -1.2 % (10 keV) with `Exact` |
| Q16 | lowest primary energy (now 2 eV) | keep |
| Q17 | solvation penetration law beyond `Meesungnoen2002` | implement the others only in M6 behind one enum if you want a sensitivity study |
| Q19 | `PrimaryKiller`-style track-segment mode | M8 |
| Q4 | slow Born1 path | still deferred |
| Q11 | non-conserving proton charge exchange | needed at M5 |

### 5. Housekeeping
- Delete merged branches on GitHub (`claude/m2-electron-transport`, `claude/next-steps`, older ones).
- Start the next session on a **new branch from `main`** after the M3 PR is merged.

### 6. What the next round delivers (so you can check it)
M6: from the `PhysChemEvent` list of M3 to a list of chemical species with positions (dissociation channels,
displacement, thermalisation of sub-excitation electrons, solvated-electron placement), initial yields per 100 eV
compared with Geant4 `chem6` at 1 ps. You will have to run one more Geant4 reference generation (the agent
extends the generator and gives exact commands).

---

## (ii) Draft of the next prompt

```text
Start milestone M6 (physicochemical stage) of the geant4-rs project.
The M3 validation gate is closed (step 0 is a check only).

Setup
- New branch from the updated `main` (or continue on claude/inspiring-edison-v1kw1a if the M3 PR is not merged yet).
  Do not push to `main`; open a PR only when I ask.
- Read first: AGENTS.md, docs/README.md, docs/next.md, docs/08-roadmap.md (M3 status and numbers, M6), docs/01 (A.4),
  docs/02 (B.5, B.7.3 item 4, B.7.5), docs/03, docs/04 (D.1 step 5, D.2), docs/05, docs/07, docs/09 (all open
  questions), docs/10, docs/units.md, docs/reproducibility.md, validation/geant4/README.md.
- The Geant4 C++ reference is the read-only repo `rtonneau/geant4`; clone it if absent and re-read the C++ of every
  model you port (G4DNAChemistryManager::CreateWaterMolecule / CreateSolvatedElectron, G4ChemDissociationChannels_option1,
  G4DNAWaterDissociationDisplacer, the molecule types for D and charge, G4EmDNAChemistry_option3 up to the pre-chemistry
  stage). Never modify it. Do not invent Geant4 numbers; reference data come only from files I commit to
  validation/reference/.

Status I provide (fill in or delete)
- CI on main: green / red (paste log).
- Geant4 M3 reference: committed; gate passed 2026-10-15 (2 ignored tests pass; 8 sparse radial-tail comparisons
  skipped, see docs/08). Geant4 W = 19.5-21 eV agrees with Rust.
- Decisions: Q12 keep skipping; Q13 default Geant4, report both; Q16 keep 2 eV; Q17 Meesungnoen2002 only.   <- edit

Step 0: sanity check only
- Run `cargo test -p dna-sim --test slowing_down_reference -- --ignored --nocapture` once on the branch to confirm
  the gate still passes (about 45 s). If it fails: stop, diagnose, report. Never loosen a tolerance; the two
  amendments already made (V2 zero floor 1e-290, radial n_eff >= 10 skip) are documented and must not be extended
  without evidence.

M6 scope
1. Species and chemistry data as TOML/data (docs/04 D.2): the dissociation channels of G4ChemDissociationChannels_option1
   (probabilities sum to 1 within 1e-12, validated at load), the species table (D, charge, radius) for the species reachable from
   option2 events (H2O+, H3O+, OH, H, H2, e_aq, O, OH-, ... as the C++ lists them), with source line numbers.
2. Event -> species: for every PhysChemEvent of M3 (excitation level, ionisation shell, attachment, solvation) choose
   the channel (one stream stage: Stage::PhysChem, docs/reproducibility.md), apply the displacer of the C++
   (RMS offsets per product, mother-molecule recoil) and return SpeciesInstance { kind, position, time = 0 }.
   Solvation events become e_aq at the displaced position already stored in the event; attachment events follow the C++.
3. Scoring: species counts per 100 eV at t = 0 (just after the physico-chemical stage) per event type and in total,
   G-values with both estimators (docs/01 A.4: mean of per-event ratios and ratio of totals), displacement
   distributions (RMS per product), spatial correlation (pair distances of OH-e_aq, OH-H3O+ ...).
4. Keep the stages separate: physics crate -> events -> physchem (new crate or module, decision in docs/03 with
   PhysChemEvent possibly moving to dna-core) -> species list. No diffusion, no reactions (M7).

Tests (thresholds fixed before running, Bonferroni families stated in the test header)
- Channel probabilities: chi-square of the sampled channel per level/shell against the table (V8); displacement RMS
  within 3 sigma of nominal; momentum/charge/atom balance of every channel; energy ledger untouched.
- Hand-computed (independent Python) channel tables and RMS values; thread-count independence of the species list.
- Physical sanity written before looking: initial yields per 100 eV of the order the literature gives for fast
  electrons (state your expectation for e_aq, OH, H3O+, H, H2 with reasoning), species conservation (H and O atoms).

Geant4 reference for M6 (I run this locally; Geant4 is not available in your container)
- Extend validation/geant4/g4rs_ref with a `physchem` mode: G4EmDNAChemistry_option3 with the scheduler ended at 1 ps
  (chem6 style), per primary energy {1e3, 1e4, 1e5, 9e5} eV, counting species at 1 ps (sums and sums of squares per
  species) and their RMS distance from the origin; same physics settings as M2/M3. Document exact commands and the
  expected runtime; keep the new Rust comparison tests #[ignore]d until the CSVs exist.
- Tell me exactly which files you need.

Definition of done
- cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace all pass (run
  them and report the output; mention which tests are ignored and why).
- Docs updated: docs/08 (M6 status, performance), docs/02 (anything newly read), docs/03, docs/04 D.2, docs/09 (new
  questions, risks), docs/10 if data semantics change, docs/reproducibility.md (physchem draw order), and docs/next.md:
  replace part (i) with what I must do next (Geant4 M6 reference run, decisions) and part (ii) with the draft prompt for
  M7.
- Commit in small steps, push the branch, summarise changes, limitations, and everything still unverified.
```
