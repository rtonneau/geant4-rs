# Next round

Written after PR #5 (M2) was merged; `main` is at `744506a`. Part (i) is for the human, part (ii) is the draft of
the next prompt for the coding agent. Update this file at the end of every round (the agent is asked to do so).

## (i) What you need to do before the next round

### 1. Check that `main` builds in CI (5 min)
Open the Actions tab on GitHub and confirm the last run on `main` (fmt, clippy, test) is green. Nobody has
observed CI so far; all results in the repository come from local runs in the agent's container. If it is red,
paste the log into the next session before anything else.

### 2. Generate the Geant4 reference data (the main blocker, about half a day if Geant4 is not installed)
M2 is implemented but **not compared to Geant4**: three tests in `crates/dna-sim/tests/geant4_reference.rs`
are `#[ignore]`d until three CSV files exist. The generator `validation/geant4/g4rs_ref` was written without a
Geant4 installation and has never been compiled. Full instructions: `validation/geant4/README.md`. Short version:

1. Geant4 **11.4.1** installed, data set **G4EMLOW 8.8** (you already have it at
   `C:\DEV\GEANT4\Geant4-ressources-data\G4EMLOW8.8`; set `G4LEDATA` to that folder). Geant4 is much easier to build
   on Linux/WSL2 than on native Windows; use WSL2 if you can.
2. Build `g4rs_ref` (CMake, `-DGeant4_DIR=...`). If it does not compile, fix only the API mismatch; do **not** change
   the physics settings (constructor `G4EmDNAPhysics_option2`, `G4_WATER`, fluorescence/Auger off, energy lists,
   secondaries killed).
3. Run the three modes and put the results in `validation/reference/`:
   - `xs_per_volume.csv` (seconds)
   - `mfp_first_interaction.csv` (10^6 events per energy; minutes)
   - `track_cut100eV.csv` (10^4 events per energy; the 900 keV point is the slow one; 2000 events is acceptable)
   - optionally the same two Monte Carlo runs with a second seed (`..._seed2.csv`) to estimate Geant4's own scatter.
4. Write `validation/reference/provenance.toml` (template in the README; every key must be present).
5. Commit and push on a new branch, run `cargo test -p dna-sim --test geant4_reference -- --ignored --nocapture`
   (or let the agent run it), and tell the agent the outcome.

If you cannot get Geant4 running: say so in the next prompt. The agent will then continue with M3 on L1/L2 evidence
only and mark every statement about agreement with Geant4 as unverified. This is acceptable for a while but should not
last through M5.

### 3. Decisions (answer in the prompt, or accept the recommended default)
| Id | Question | Recommended default |
|---|---|---|
| Q5 | Energy-normalisation convention of docs/01 section A.4 (per-event ratio like `chem6`, plus ratio of totals) | confirm as written |
| Q12 | Atomic de-excitation after oxygen K-shell ionisation (Auger/fluorescence). Affects the split of deposited energy, not the total | keep skipping it in M3, compare against Geant4 with fluo/Auger **off**; revisit when the G(t) comparison exists |
| Q13 | Geant4's Champion elastic rotation uses a non-unit axis, so the deflection depends on beam orientation | keep `ElasticRotation::Geant4` as default for comparisons; every physics result is also produced with `Exact`, and both are reported side by side |
| Q4 | Implement the slow (rejection-sampling) path of Born1 and compare with the fast path | defer; not needed by option2 |
| Q14 | Elastic-only termination guard between 7.4 and 9 eV | closed by M3 (Sanche, Melton, solvation) |
| Q11 | Geant4's non-conserving proton charge exchange | not needed before M5 |

### 4. Housekeeping
- Delete the merged branches on GitHub (`claude/inspiring-edison-v1kw1a`, `claude/m2-electron-transport`).
- Merge the PR that carries this file (or paste part (ii) directly).
- Start the next session on a **new branch from `main`**.
- If you want the physics validated before more physics is added, run the Geant4 comparison first and start the next session
  with a failing/passing report; a mismatch in M2 is cheaper to fix than a mismatch discovered after M3.

### 5. What the next round delivers (so you can check it)
M3: electrons are followed all the way down (secondary electrons transported, vibrational excitation, attachment,
solvation), so that full slowing-down quantities exist: range, deposited-energy distribution, secondary spectra,
W-value-like ratios, and a list of physicochemical events for M6. You will then have to run a second Geant4 reference
generation (the agent extends the generator and gives exact commands).

---

## (ii) Draft of the next prompt

```text
Start milestone M3 (secondary electrons and low-energy electron processes) of the geant4-rs project.
Before M3 proper, close M2's validation gate (step 0).

Setup
- New branch from the updated `main`. Do not push to `main`; open a PR only when I ask.
- Read first: AGENTS.md, docs/README.md, docs/next.md, docs/08-roadmap.md (M2 status, M3), docs/01 (section A.4),
  docs/02 (B.2, B.7.1-B.7.4), docs/04 (D.1), docs/05, docs/07, docs/09 (all open questions), docs/10, docs/units.md,
  docs/reproducibility.md, validation/geant4/README.md.
- The Geant4 C++ reference is the read-only repo `rtonneau/geant4`; clone it if absent and re-read the C++ of every model
  you port. Never modify it. Do not invent Geant4 numbers; reference data come only from files I commit to
  validation/reference/.

Status I provide (fill in or delete)
- CI on main: green / red (paste log).
- Geant4 reference CSVs: committed in validation/reference/ with provenance.toml / not available (then skip step 0 and mark
  every Geant4-agreement claim as unverified).
- Decisions: Q5 confirmed; Q12 = keep skipping K-shell de-excitation (compare with fluo/Auger off); Q13 = default
  ElasticRotation::Geant4, always also run Exact; Q4 deferred.   <- edit if different

Step 0: M2 validation gate (only if the CSVs exist)
- Run `cargo test -p dna-sim --test geant4_reference -- --ignored --nocapture`. Acceptance criteria are fixed in the test
  header and in docs/07 (V2: 1e-9 at nodes and 1e-6 elsewhere; V3; V5: z < 3 and < 2 %). Never loosen a tolerance.
- If everything passes: record the numbers in docs/08 (M2 status) and continue.
- If anything fails: stop M3 work, diagnose (data, units, model bug, documented deviation, generator settings: the generator
  was never compiled before, check its CSV header), fix, document, and report before continuing. Use the diagnosing-bugs
  skill if useful.

M3 scope
1. Secondary stack: ionisation secondaries are transported (LIFO stack per history, each secondary uses the same history
   RNG stream; document the consumption order in docs/reproducibility.md). Energy ledger over the whole cascade:
   E_primary = deposited (local + binding) + escaped (none in infinite water) + remaining_untracked, asserted at 1e-9
   relative per history.
2. New models, each re-read in the C++ first and ported line by line with the quirks counted, not fixed:
   - Sanche vibrational excitation `G4DNASancheExcitationModel` (window 2-100 eV; `dna/sigma_excitationvib_e_sanche.dat`,
     lin-lin 2-D interpolation; includes a kill-and-deposit branch, see lines ~249-280).
   - Melton attachment `G4DNAMeltonAttachmentModel` (window 4-13 eV; table ends at 12.3 eV, decide the 12.3-13 eV behaviour
     explicitly and document it: Geant4 returns 0 there, the Rust table returns an error, docs/02 B.7.1).
   - Solvation of electrons below 7.4 eV (`G4DNAElectronSolvation` with the default `Meesungnoen2002` penetration model,
     docs/02 B.7.3 item 4): kill, deposit the full kinetic energy, record a solvation event with the displaced position.
     Also read `G4DNATransformElectronModel` (0-0.025 eV), which is still unread.
   - Remove the M2 termination guard (Q14) and the `NoInelasticChannel` reason once the low-energy processes exist; keep an
     interaction-count safety limit that reports, never silently stops.
3. Physicochemical event record (data model only; no chemistry): for every excitation, ionisation, attachment and
   solvation write `PhysChemEvent { kind, level_or_shell, position, deposited_energy, parent_history }` to the output.
   This is the interface to M6 (docs/03).
4. Scorers: full slowing-down track length (range), deposited-energy distribution, per-process counts, secondary spectra
   (generation and energy), number of solvation events per 100 eV deposited, a W-like quantity
   (E_primary / number of ionisations). Keep every output with provenance.
5. Elastic rotation: run with ElasticRotation::Geant4 and Exact, report both (Q13).
6. Process table: add the window-boundary behaviour for processes with several models (docs/02 B.7.3 item 5) if not yet
   present; unit-test exact boundary values.

Tests (thresholds fixed before running, Bonferroni families stated in the test header)
- Windows and boundaries for the three new components; hand-computed cross sections (extend scripts/hand_cross_sections.py
  as an independent Python re-implementation of the C++, not the Rust output) at nodes and three off-node energies.
- Sampling: Sanche final-state distribution and Melton absorption branch against the raw tables; solvation displacement
  distribution (mean radius r_mean, sigma_1D = 0.62665706865775 r_mean, fixed-seed distribution test); quirks on synthetic
  tables.
- Ledger with the full cascade at 100 eV, 1 keV, 10 keV, 100 keV, 1 MeV (1e4 histories at low energies, fewer at 1 MeV);
  thread-count independence (1, 2, 8 threads, bitwise per-history results).
- Physical sanity (not a Geant4 comparison): range monotonic in energy; W-like quantity in a plausible range with the
  reasoning written in docs/08 (state what you expect and why before looking at the number).

Geant4 reference for M3 (I run this locally; Geant4 is not available in your container)
- Extend validation/geant4/g4rs_ref with modes for full slowing-down: per primary energy {100 eV (if meaningful), 1e3, 1e4,
  1e5, 9e5}: track length of the primary and of all secondaries, deposited energy, number of ionisations, excitations,
  vibrational excitations, attachments, solvations, secondary electron energy histogram. Same physics settings as M2
  (option2, fluo/Auger off). Keep the CSV format machine-readable with sums and sums of squares; document exact commands and
  the expected runtime; keep the new Rust comparison tests `#[ignore]`d until the CSVs exist.
- Tell me exactly which files you need.

Definition of done
- cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace all pass (run
  them and report the output; mention which tests are ignored and why).
- Docs updated: docs/08 (M3 status, performance baseline), docs/02 (anything newly read), docs/09 (new questions, risks),
  docs/10 if data semantics change, docs/reproducibility.md (RNG consumption order with secondaries), and **docs/next.md**:
  replace part (i) with what I must do next (Geant4 M3 reference run, decisions) and part (ii) with the draft prompt for
  the following milestone.
- Commit in small steps, push the branch, summarise changes, limitations, and everything still unverified.
```
