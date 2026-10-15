# Next round

Written at the end of M3 (branch `claude/inspiring-edison-v1kw1a`, **no PR opened yet**: open one when you are
ready; `main` is at `5c55626`, the merge of the M2 Geant4 reference data). Part (i) is for the human, part (ii) is
the draft of the next prompt for the coding agent. Update this file at the end of every round (the agent is asked
to do so).

## (i) What you need to do before the next round

### 1. Review and merge M3 (your call)
M3 is implemented and fully tested **without any Geant4 comparison of the new processes**. Ask the agent to open a
PR when you want one; CI (`fmt`, `clippy -D warnings`, `cargo test --workspace`) has still never been observed on
GitHub, all results come from local runs in the agent's container (the whole `cargo test --workspace` takes about
100 s on 4 cores). If CI is red after the merge, paste the log into the next session first.

### 2. Generate the Geant4 M3 reference (the main blocker; about 1-3 hours of compute, 30 min of work)
Full instructions: `validation/geant4/README.md`, section "M3: full slowing-down reference". Short version, from
`validation/geant4/g4rs_ref` after rebuilding the generator (the new modes `xslow` and `slow` are **untested C++**,
syntax-checked only; if they do not compile, fix only the API mismatch):

```bash
./build/g4rs_ref xslow ../../reference/xs_lowenergy.csv                 # seconds
./build/g4rs_ref slow  ../../reference/slow 20271                      # default energy list, see below
```
Time it first: `./build/g4rs_ref slow /tmp/slow_timing 1 8:1000,100:1000,1000:100,10000:10,100000:1`, then run
the default list (omit the last argument) or a reduced one (default: `8:100000,30:100000,100:100000,1000:20000,
10000:2000,100000:200,900000:20`; the 900 keV entry is the slow one).

**Files I need from you, committed in `validation/reference/`:**
`xs_lowenergy.csv`, `slow_sums.csv`, `slow_spectra.csv`, `slow_radial.csv`, `provenance_m3.toml` (template in the
README; the test requires `fluo_auger = "off"` and the constructor name). Optional: a second seed
(`slow2_*.csv`) to estimate Geant4's own scatter.
Then run `cargo test -p dna-sim --test slowing_down_reference -- --ignored --nocapture` (or let the agent) and
tell the agent the outcome. A mismatch in M3 is cheaper to fix now than after M6 has built on it.

If you cannot get Geant4 running again: say so in the next prompt. The agent then continues with M6 on L1/L2
evidence only and marks every statement about the new processes as unverified (acceptable for one milestone, not
through M8: G(t) needs the low-energy physics right).

### 3. Things to look at while you are at it
- **W = E_in / N_ion is 19.5-21.0 eV** (docs/08, R13), 20 % below what the agent predicted (25 eV). The Geant4
  `slow_sums.csv` (`n_ionisation` per event) is the quick check: Rust 5.04 ionisations at 100 eV, 51.2 at 1 keV,
  490 at 10 keV, 4790 at 100 keV.
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
Before M6 proper, close M3's validation gate (step 0).

Setup
- New branch from the updated `main`. Do not push to `main`; open a PR only when I ask.
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
- Geant4 M3 reference (xs_lowenergy.csv, slow_sums.csv, slow_spectra.csv, slow_radial.csv, provenance_m3.toml):
  committed / not available (then skip step 0 and mark every Geant4-agreement claim about Sanche, Melton, solvation
  and the cascade as unverified).
- Decisions: Q12 keep skipping; Q13 default Geant4, report both; Q16 keep 2 eV; Q17 Meesungnoen2002 only.   <- edit

Step 0: M3 validation gate (only if the files exist)
- Run `cargo test -p dna-sim --test slowing_down_reference -- --ignored --nocapture`. Acceptance criteria are fixed in
  the test header (V2 1e-9 / 1e-6; V5 family z < 4 with the 2 % cut where it has power; spectra chi-square; radial
  cumulative fractions). Never loosen a tolerance.
- If everything passes: record the numbers in docs/08 (M3 status) and continue.
- If anything fails: stop M6 work, diagnose (data, units, model bug, documented deviation, generator settings: the
  `slow`/`xslow` modes were never compiled before, check the CSV headers and the n_other_* counters), fix, document,
  report before continuing. Use the diagnosing-bugs skill if useful. The W = E/N_ion = 19.5-21 eV result (R13) is the
  first thing to compare.

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
