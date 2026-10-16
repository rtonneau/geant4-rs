# gps backlog: M6 closure, M7, M8

Seeds for `/gps start <slug>`, registered with `scout-merge.js --from docs/gps-backlog.md`. They replace
the former hand-written to-do file. Check progress with `/gps status`.

## How seeds work

- **Dependencies.** Seeds have no dependency field: every seed states `requires <slug>` in its text. A
  seed that consumes M6 species or positions requires M6 to be closed (decision recorded in docs/09).
- **Geant4 data.** A seed that needs Geant4 data follows the convention in
  `validation/geant4/RUN_REQUESTS.md`: new C++ modes or programs under `validation/geant4/` (existing
  modes untouched), compile-check allowed, **never run**; one request entry per run; the Rust comparison is
  written as an `#[ignore]`d test with tolerances and statistical families fixed in its header before any
  data exists; the session ends with the gate **awaiting data**. A paired `*-close-gate` seed finishes the
  comparison after the user has committed data and `provenance*.toml`.
- **Standing rules (AGENTS.md).** Never overwrite or delete reference files or existing generator modes.
  Never loosen a pre-registered tolerance to pass. Never invent physical constants, cross sections or
  reaction parameters. Verify against the Geant4 11.4.1 `.cc` sources, not memory. Run `cargo fmt --all --
  --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` before
  finishing.
- **Horizon.** M6 closure through M8. M5 (protons), M9, M10, M11 stay roadmap lines and are scouted later.
  The proton 10 MeV acceptance of M8 is therefore deferred until M5 exists.

## Open items

- Whether the M6 diagnosis needs new diagnostic Geant4 data (decides whether `m6-close-gate` is used).
- Exact V11 energies and time list for `m8-gt-scoring-and-chem6-reference` (docs/07 V11: 1 ps to 1 us;
  roadmap: e- 100 keV and 1 MeV). Fixed in that seed's grill.
- Q13 default for science runs stays `Geant4` (docs/09).

---

## m6-comparison-report

**Goal.** Make the failing M6 gate readable. Add a **non-asserting** comparison report (an `#[ignore]`d
test or a small example that prints a table): per energy (1 keV, 10 keV, 100 keV, 900 keV) and species,
Geant4 mean +- SE, Rust mean +- SE, z, relative difference, and RMS radius for both. It shows whether the
offset is limited to a few species (`H2O2`, `O_2`, `H_2`, `H`, `OH^-1`, ...) or broad.

**Requires.** Nothing.

**State.** `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture` stops at the first
assertion, `1000 eV H2O2: z = 111.6` (Geant4 books 0.689 `H2O2` per event at 1 ps; Rust is far from it).
The test stops at the first failure, so nothing else is known. Reference data exist
(`validation/reference/physchem.csv`: 1 keV x 20000, 10 keV x 2000, 100 keV x 200, 900 keV x 20 events,
seed 20261019; `provenance_m6.toml`). The `g4rs_ref physchem` crash (Q24) is fixed in C++, see
`validation/geant4/BUILD_NOTES.md` section 5 (issues G-K).

**Tasks.**
1. Record the failure and its evidence in docs/08 first.
2. Write the report. Keep the asserting gate and every pre-registered tolerance unchanged.
3. Close Q24 in docs/09 with a pointer to BUILD_NOTES section 5.

**C++.** None.

**Acceptance.** The report runs and prints the full table for all energies; the asserting test is
byte-for-byte unchanged in thresholds; docs/08 and docs/09 updated; no file in `validation/reference/`
touched.

## m6-diagnose-and-fix

**Goal.** Find why Rust M6 disagrees with Geant4 at 1 ps and fix model bugs or document each intentional
deviation, then re-run the gate.

**Requires.** `m6-comparison-report`.

**Method.** One hypothesis at a time, checking the Geant4 source (not memory):
- dissociation-channel branching of the excited and ionised states, including the `H2O2`, `H2`, `O` and
  `OH^-` channels (`G4ChemDissociationChannels_option1`);
- what Rust M6 emits at 1 ps versus Geant4: species born at exactly 1 ps versus later decays;
- recombination and event ordering (Q21, R14);
- count-time semantics (the snapshot is taken at the first scheduler step with `dt > 0`);
- spatial RMS semantics (Geant4 `r` is from the primary origin, all alive molecules);
- the documented Rust/Geant4 differences Q12, Q13, Q16, Q17;
- that Rust M6 starts the same cascade (secondaries followed to solvation, fluo/Auger off).

**Rules.** Do not change any pre-registered tolerance; never loosen one to pass. Regenerating
`physchem.csv` is **not** part of this seed: if the generator itself looks wrong, say so and ask
(AGENTS.md). One unexplained event: the first production attempt died at 900 keV with a stack overflow
(`0xC00000FD`); a rerun with `/STACK:536870912` completed. If it recurs, capture a stack trace
(BUILD_NOTES section 6) instead of raising the limit.

**C++.** Allowed if a hypothesis needs data Geant4 has not produced: add a **new** diagnostic mode (e.g.
species counts per decay state at 1 ps), compile-check, never run, log a request in
`validation/geant4/RUN_REQUESTS.md`.

**Acceptance.** The gate passes, or the remaining open gate is explicitly documented with evidence; every
deviation is listed in docs/09; docs/08 M6 status updated. If diagnostic data was requested, the seed ends
"awaiting data" and `m6-close-gate` follows.

## m6-close-gate

**Goal.** Finish the M6 comparison once diagnostic Geant4 data are committed.

**Requires.** `m6-diagnose-and-fix` and the user's committed data and provenance. Used only if that seed
requested data.

**Tasks.** Check the provenance (version, data sets, constructors, exact command, seed, date, generator
commit, compiler, platform); run the consuming test and the main gate; record pass or fail with evidence in
docs/08; update the RUN_REQUESTS entry status.

**Acceptance.** The M6 gate is closed (all pre-registered comparisons pass) or the open gate is accepted
explicitly by the user. This unblocks everything that consumes M6 species.

## m7-option3-reaction-table

**Goal.** Port the Geant4 option3 reaction table to validated TOML.

**Requires.** Nothing (independent of M6).

**Tasks.** Reactants, products, reaction type, k_obs / k_act, source line provenance from
`G4EmDNAChemistry_option3` and the reaction table of Geant4 11.4.1. Extend the existing species table only
where the network requires it. A C++ cross-check script (as `scripts/extract_physchem_from_cpp.py` did for
M6) and an independent Python evaluation. Atom and charge conservation test for every reaction.

**C++.** None (extraction is textual).

**Acceptance.** Every rate constant matches the C++ text exactly; all reactions conserve atoms and
charge; the table loads with validation errors for malformed entries; no constant is guessed. Update
docs/02 B.7.6-B.7.7, docs/04 D.2, docs/10, docs/reproducibility.md.

## m7-diffusion-and-two-body-irt

**Goal.** Free Brownian diffusion and an isolated two-body IRT sampler for A+B and A+A.

**Requires.** `m7-option3-reaction-table`. Preserving 1 ps M6 initial positions requires M6 closed.

**Tasks.** Deterministic per-history chemistry RNG streams; free diffusion; IRT reaction-time sampling
including partially diffusion-controlled reactions (Collins-Kimball style, as in the Geant4 IRT code);
explicit domain and error policies; Geant4 11.4.1 IRT/SBS sources cited. Do not build the full
many-species scheduler yet.

**Tests (V9, V10-style unit tests).** Analytic survival and reaction-time laws over **three pre-registered
D/R/k sets**; A+A symmetry and no self-pairing; boundary and extreme-value tests; thread-count
independence (1, 2, 8 threads, bitwise). State the Bonferroni families and thresholds in the test
headers **before** running. V9 asks within 3 sigma of the analytic curve with N >= 1e4 pairs (docs/07).

**C++.** None.

**Acceptance.** All tests pass with thresholds fixed beforehand; deviations from Geant4 recorded; docs/03,
04, 08, 09, 10 and reproducibility.md updated.

## m7-sbs-oracle-and-ode

**Goal.** Independent oracles for the sampler: SBS for one totally diffusion-controlled case and the
well-mixed ODE limit.

**Requires.** `m7-diffusion-and-two-body-irt`.

**Tasks.** SBS (step-by-step) reference implementation in Rust for one totally diffusion-controlled case,
with IRT versus SBS agreement within 3 sigma (docs/08 M7); V10 well-mixed limit versus an ODE, long-time
agreement 1 % for a closed-system case. A **new** C++ two-body reference mode (never run) producing
reaction-time or survival data from Geant4, with a RUN_REQUESTS entry and an `#[ignore]`d Rust comparison
with pre-registered tolerances.

**C++.** Yes: new mode in `validation/geant4/g4rs_ref`, compile-check only; README and BUILD_NOTES
updated.

**Acceptance.** Oracle tests pass; the Geant4 comparison is written and the gate is "awaiting data".

## m7-sbs-close-gate

**Goal.** Close the two-body Geant4 comparison.

**Requires.** `m7-sbs-oracle-and-ode` and the committed data and provenance.

**Tasks.** Provenance check, run the consuming test, record the result in docs/08, update RUN_REQUESTS.

**Acceptance.** Pre-registered comparison passes, or the failure is documented with evidence; no
tolerance loosened.

## m7-multispecies-irt-scheduler

**Goal.** The full many-species IRT scheduler on the M6 initial species.

**Requires.** `m7-diffusion-and-two-body-irt`, `m7-sbs-close-gate`, and M6 closed (`m6-diagnose-and-fix`
or `m6-close-gate`).

**Tasks.** Reaction network from the validated table; time-ordered reaction events; species positions
from the M6 1 ps record; thread-count independence; spatial correlations preserved (AGENTS.md). Keep the
M6 same-decay pair statistic distinct from any RDF/nearest-neighbour observable (R16): if one is needed,
pre-register bins and cutoffs as a new observable. Add a C++ reference mode for species counts over time
(never run), RUN_REQUESTS entry, and an `#[ignore]`d comparison with pre-registered tolerances.

**C++.** Yes: new mode, compile-check only.

**Acceptance.** Unit and statistical tests pass; the comparison is written; gate "awaiting data".

## m7-scheduler-close-gate

**Goal.** Close the scheduler comparison.

**Requires.** `m7-multispecies-irt-scheduler` and the committed data and provenance.

**Tasks.** Provenance check, run the consuming test, record the result in docs/08, update RUN_REQUESTS.

**Acceptance.** Comparison passes or is documented with evidence. M7 is then closed in docs/08.

## m8-gt-scoring-and-chem6-reference

**Goal.** G(t) scoring with uncertainties and the Geant4 chem6-like reference generator.

**Requires.** `m7-scheduler-close-gate`.

**Tasks.** G(t) scorer for e_aq, OH, H, H2, H2O2 at the V11 times (docs/07: 1 ps, 10 ps, 100 ps, 1 ns,
10 ns, 100 ns, 1 us), normalisation as frozen in docs/01 A.4, batch-mean uncertainties (>= 30 batches);
new C++ chem6-like G(t) generator (electrons) with a RUN_REQUESTS entry; `#[ignore]`d V11 test written
with criteria fixed beforehand. The energies and time list are fixed in this seed's grill.

**C++.** Yes: new program or mode; compile-check only.

**Acceptance.** Scorer tests pass (hand-computed G on synthetic inputs, both estimators); gate "awaiting
data".

## m8-gt-close-gate

**Goal.** V11 comparison for e- 100 keV and 1 MeV against the chem6 reference.

**Requires.** `m8-gt-scoring-and-chem6-reference` and the committed data and provenance.

**Tasks.** Provenance check; run the V11 test (per species and time, |G_rs - G_g4| < 3 sqrt(s_rs^2 +
s_g4^2), Bonferroni family p > 0.01, relative systematic < 5 % at t >= 1 ns); record the result in docs/08.

**Acceptance.** V11 passes or the failure is documented with evidence. The proton 10 MeV case is deferred
to M5.

## m8-config-and-cli

**Goal.** Headless execution: TOML config, `dna run config.toml`, JSON output with metadata.

**Requires.** `m8-gt-scoring-and-chem6-reference`.

**Tasks.** Config schema with validation errors; clap CLI; JSON output with provenance (versions,
checksums, seed, physics options); documentation of the config and output schema.

**Acceptance.** CLI integration tests (valid and invalid configs); output schema versioned; docs updated.

## m8-reproducibility-v13

**Goal.** V13 end to end: thread-count and seed reproducibility of the whole pipeline.

**Requires.** `m8-gt-close-gate`, `m8-config-and-cli`.

**Tasks.** Identical G(t) output for 1, 2, 8 threads on the same platform; different seeds give
statistically compatible results; fixed-order reductions; document the reproducibility claim's scope
(docs/reproducibility.md).

**Acceptance.** Bitwise equality across thread counts; seed compatibility test with thresholds fixed
beforehand.
