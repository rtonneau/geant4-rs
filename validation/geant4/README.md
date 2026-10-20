# Geant4 reference data for the M2 and M3 validation

**Status.** M2: the generator modes `xs`, `mfp`, `track` were compiled and run by the project lead
(Geant4 11.4.1, Windows/MSVC); the CSVs are committed in `validation/reference/` and the M2 gate passes
(`cargo test -p dna-sim --test geant4_reference -- --ignored --nocapture`, docs/08). M3: the modes `xslow`
and `slow` (section "M3: full slowing-down reference" at the end) are **untested C++**: they were written
without a Geant4 installation and only syntax-checked against stub headers. The M3 comparison tests are in
`crates/dna-sim/tests/slowing_down_reference.rs` and are `#[ignore]`d until the files listed there exist.
No Geant4 number has been invented or embedded anywhere in the repository.

**New runs requested by agent sessions** are logged in [RUN_REQUESTS.md](RUN_REQUESTS.md): sessions write and compile-check the C++ but never run it.

## What the Rust code is compared against

Geant4 **11.4.1** (the reference tree is `rtonneau/geant4`, CMake version 11.4.1, `G4Version.hh`:
`geant4-11-04-patch-01`), data set **G4EMLOW 8.8**, physics constructor **`G4EmDNAPhysics_option2`**,
liquid water `G4_WATER` (1 g/cm3), primary electron. Two settings of the generator are deliberate:

| Setting | Why |
|---|---|
| Fluorescence / Auger **off** (`G4EmParameters::SetFluo/SetAuger(false)`) | geant4-rs M2 does not simulate the atomic de-excitation after oxygen K-shell ionisation (docs/09 Q12); with it on, Geant4 would book less than 539 eV locally for those events and the deposited-energy comparison would differ for a known reason, not a bug. |
| Energy lists avoid exactly 100 eV (Sanche window is closed at 100 eV) and exactly 1 MeV (standard `eIoni` is also active at exactly 1 MeV) | the Rust M2 has no Sanche/Melton/solvation and no standard EM; away from those points the physics of both codes is the same. |

Quantities that **cannot** be compared in M2 (they need Sanche vibrational excitation, Melton attachment
and solvation, i.e. M3): full slowing-down range, W-value, anything below 100 eV. This is why the
`track` mode stops the primary at a 100 eV tracking cut, as does the Rust run (`tracking_cut_ev`).

## Which files I need from you

Put these three CSVs (and `provenance.toml`) into `validation/reference/` and commit them. Budget:
well under 1 MB each.

| File | Mode | Contents | Compared by |
|---|---|---|---|
| `xs_per_volume.csv` | `xs` | `energy_eV,process,sigma_per_volume_per_nm` for `elastic`, `excitation`, `ionisation` (`G4EmCalculator::ComputeCrossSectionPerVolume`, 1/nm) at 208 energies (window boundaries 7.4, 9, 11 eV, 1 MeV, decade points, and 200 log-spaced energies 7.4 eV - 1 MeV) | V2 (1e-9 at nodes, 1e-6 elsewhere), V3 |
| `mfp_first_interaction.csv` | `mfp` | per energy {150, 300, 1e3, 3e3, 1e4, 3e4, 1e5, 3e5, 9e5} eV: `n_events`, `sum_L_nm`, `sum_L2_nm2` of the first free path and the counts of the process that ended it | V5: z < 3 and < 2 % |
| `track_cut100eV.csv` | `track` | per energy {200, 1e3, 1e4, 1e5, 9e5} eV, primary followed until its energy falls below 100 eV, secondaries killed: sums (and squares) of track length, deposited energy, remaining energy, secondary count and energy, and interaction counts per process | V5: z < 3 and < 2 % |

Recommended statistics: `mfp`: 10^6 events per energy (statistical error 0.1 %; with 10^5 the
2 % criterion is still meaningful); `track`: 10^4 events per energy (the 900 keV point is the slow
one, about 1.7e4 primary steps per event; reduce to 2000 events if needed, the CSV records `n_events`).
Use two different seeds and keep both runs (`mfp_first_interaction_seed2.csv`, ...) if you can: the
V5 text asks for the Geant4 run-to-run scatter to be estimated (docs/07 G.2).

## How to generate them

```bash
# 1. Geant4 11.4.1 with the data set G4EMLOW 8.8 (only dna/ is read)
source <geant4-install>/bin/geant4.sh            # sets G4LEDATA to G4EMLOW8.8
echo $G4LEDATA                                   # must end in G4EMLOW8.8

# 2. build the generator (outside the reference repository; it is read-only)
cd validation/geant4/g4rs_ref
cmake -S . -B build -DGeant4_DIR=<geant4-install>/lib/cmake/Geant4
cmake --build build -j

# 3. run (serial; each command is independent)
./build/g4rs_ref xs    ../../reference/xs_per_volume.csv
./build/g4rs_ref mfp   ../../reference/mfp_first_interaction.csv 1000000 20261
./build/g4rs_ref track ../../reference/track_cut100eV.csv        10000   20262 100
```

Arguments: `mfp <out> <events> <seed>`, `track <out> <events> <seed> <cut_eV>`. If it does not
compile, fix the API mismatch but **do not change** the physics settings (constructor, material,
fluo/auger off, energy lists, kill-secondaries rule): the Rust tests hard-code the same lists.

## `validation/reference/provenance.toml`

Required by the tests (every key must be present):

```toml
geant4_version      = "11.4.1"                  # as printed by the CSV header / G4Version
geant4_commit       = ""                        # SHA or tag of the build, if known
g4emlow_version     = "8.8"
g4emlow_md5         = "328330009df633f7e9b3a9f445745298"   # cmake/Modules/G4DatasetDefinitions.cmake
physics_constructor = "G4EmDNAPhysics_option2"  # the test asserts this exact string
generator           = "validation/geant4/g4rs_ref, git commit <sha of geant4-rs>"
date                = "YYYY-MM-DD"
command_lines       = [
  "g4rs_ref xs xs_per_volume.csv",
  "g4rs_ref mfp mfp_first_interaction.csv 1000000 20261",
  "g4rs_ref track track_cut100eV.csv 10000 20262 100",
]
fluo_auger          = "off"
compiler            = ""
platform            = ""
```

## Running the comparison

```bash
cargo test -p dna-sim --test geant4_reference -- --ignored --nocapture
```

Acceptance criteria are written in the header of `geant4_reference.rs` and were fixed before any
comparison (docs/07). A failure is investigated (data, units, a model bug, or a documented deviation
listed in docs/09), never accommodated by loosening a tolerance.

## Documented, expected differences (so they are not mistaken for bugs)

1. Non-unit rotation axis of Geant4's Champion elastic model (docs/09 Q13). The Rust default
   (`ElasticRotation::Geant4`) reproduces it, with the same +z primary direction as the generator;
   the `Exact` variant would deviate in the track quantities.
2. No atomic de-excitation (Q12) -> handled by switching it off in Geant4.
3. The termination guard of M2 (Q14) does not matter with the 100 eV cut.
4. Geant4 draws one number of interaction lengths per process, Rust draws one total free path
   and then selects the process: statistically identical, different random streams.

---

# M3: full slowing-down reference

Everything below is for the M3 comparison (secondary electrons, Sanche vibrational excitation, Melton
attachment, solvation). Same physics settings as M2: `G4EmDNAPhysics_option2`, `G4_WATER`, fluorescence and
Auger **off** (and `SetAtomDeexcitation(nullptr)` as in the M2 fix, K-shell binding energy stays local),
Geant4 11.4.1, G4EMLOW 8.8, serial run manager, primary along +z at the origin. **Nothing is killed or cut:**
the primary and every secondary are followed until they die by solvation or attachment (the Geant4
chemistry is not activated, so solvation only kills the electron and deposits its kinetic energy).

## Which files I need from you

Put these in `validation/reference/` and commit them (all small: the sums files have 7-8 rows; the spectra
file has `energies x 4 x 72` rows; the radial file `energies x 42`; `xs_lowenergy.csv` about 1100 rows):

| File | Mode | Contents | Compared by |
|---|---|---|---|
| `xs_lowenergy.csv` | `xslow` | `energy_eV,process,sigma_per_volume_per_nm` for `elastic, excitation, ionisation, vibexc, attachment` at 214 energies: 1.5, 2, 3.9, 4, 7.4, 9, 11, 12.3, 12.8, 13, 13.5, 100, 101, 150 eV and 200 log-spaced energies from 2 to 100 eV | `m3_v2_low_energy_cross_sections_match_geant4`: 1e-9 at nodes, 1e-6 elsewhere, zero windows must agree |
| `slow_sums.csv` | `slow` | per primary energy: `n_events` and, for each of the quantities primary track length `Lprim_nm`, track length of all tracks `Lall_nm`, deposited energy `edep_eV`, counts of elastic / excitation / ionisation / vibrational excitation / attachment / solvation interactions, number of secondaries, primary range `range_nm`: the sum over events and the sum of squares; plus the cross sum `sum_nsolv_x_edep` and the counters `n_other_process`, `n_other_secondary` (must be 0) | `m3_v5_full_slowing_down_matches_geant4` |
| `slow_spectra.csv` | `slow` | secondary-electron creation energy histograms, `energy_eV,generation_class,bin,count`; classes 1, 2, 3, 4 (= 4th generation or later); bins: 0 = underflow (< 0.1 eV), 1-70 = 0.1 eV x 10^(k/10), 71 = overflow (>= 1 MeV); the same edges as `dna-sim` | same test, two-sample chi-square per class |
| `slow_radial.csv` | `slow` | energy deposited per radial shell around the origin summed over all events: `energy_eV,bin,energy_eV_sum`; bin 0 = r < 0.1 nm, 1-40 = 0.1 nm x 10^(k/5), 41 = r >= 10 mm | same test, cumulative fractions |
| `provenance_m3.toml` | - | template below | the test refuses to run without it |

Optional but valuable (as for M2): a second run with another seed written to `slow2_*.csv` (the test reads
only the seed-1 files; the seed-2 run is for estimating Geant4's own scatter by hand or in a later test).

## How to generate them

Build the generator as for M2 (section "How to generate them" above), then, from
`validation/geant4/g4rs_ref`:

```bash
# cross sections of the five processes below 150 eV (seconds)
./build/g4rs_ref xslow ../../reference/xs_lowenergy.csv

# full slowing-down: writes ../../reference/slow_sums.csv, slow_spectra.csv and slow_radial.csv
./build/g4rs_ref slow  ../../reference/slow 20271
```

Arguments: `slow <out_prefix> <seed> [E_eV:events,E_eV:events,...]`. The default list (what the Rust test
assumes it can read; every energy found in `slow_sums.csv` is compared, with the Rust history counts of
`rust_histories()` in the test) is

```
8:100000,30:100000,100:100000,1000:20000,10000:2000,100000:200,900000:20
```

**Run a timing check first** (about a minute) and extrapolate:
`./build/g4rs_ref slow /tmp/slow_timing 1 8:1000,100:1000,1000:100,10000:10,100000:1`. Rust needs about
1.7e6 interactions per second per core; interactions per event are about 43 (8 eV), 130 (30 eV), 316 (100 eV), 2.8e3
(1 keV), 2.7e4 (10 keV), 2.6e5 (100 keV), 2.3e6 (900 keV). The default list is about 2.5e8 interactions in
total. Geant4 is usually 10-50 times slower per step than the Rust loop, so **the expected runtime is
between 40 minutes and 3 hours on one core, dominated by the 10 keV-900 keV entries (this is an estimate, I
could not measure Geant4)**. If it is too slow, reduce the counts of the last three entries first (the Rust
test copes with any `n_events >= 2`, only the statistical power drops: at 900 keV the standard deviation of
the primary track length is about 12 % of its mean, so 20 events give 2.7 % and 100 events 1.2 %; with 20 the
2 % relative cut of the test is automatically not applied, only the z-criterion).

## `validation/reference/provenance_m3.toml`

Every key must be present (the test checks these):

```toml
geant4_version      = "11.4.1"
geant4_commit       = "geant4-11-04-patch-01"
g4emlow_version     = "8.8"
g4emlow_md5         = "328330009df633f7e9b3a9f445745298"
physics_constructor = "G4EmDNAPhysics_option2"
generator           = "validation/geant4/g4rs_ref, geant4-rs git commit <sha>"
date                = "YYYY-MM-DD"
command_lines       = [
  "g4rs_ref xslow xs_lowenergy.csv",
  "g4rs_ref slow slow 20271 8:100000,30:100000,100:100000,1000:20000,10000:2000,100000:200,900000:20",
]
fluo_auger          = "off"      # asserted by the test
compiler            = ""
platform            = ""
```

## What to check if it does not compile or run

- The generator only adds code; `Slow*` functions and `RunSlow`/`RunXsLow` are new, the `xs/mfp/track`
  paths are unchanged. If a Geant4 API call fails to compile, fix only the API mismatch.
- In `slow_sums.csv` the counters `n_other_process` and `n_other_secondary` must be `0`: a non-zero value
  means a step by a process that is not one of the six listed (e.g. `Transportation` at the world edge) or a
  non-electron secondary (fluorescence/Auger photons: they must be off; the test refuses such a file).
- `edep_eV` per event must be exactly the primary energy (up to rounding); if it is smaller the sphere world
  is too small or a particle was killed, if larger something deposits twice.
- The first line of each CSV records `G4VERSION_NUMBER` and the physics settings.

## Documented, expected differences for M3 (so they are not mistaken for bugs)

1. Geant4 draws one number of interaction lengths per process, Rust one free path and a process choice;
   `G4RandGauss` versus Box-Muller for the solvation displacement: only distributions agree.
2. No atomic de-excitation in either code (Q12). K-shell ionisations book 539 eV locally on both sides.
3. The Champion rotation quirk (Q13) is reproduced by the default `ElasticRotation::Geant4`; the generator
   shoots along +z like the Rust runs. `Exact` is expected to differ in ranges and the radial profile
   at the 0.5-1 % level at 1-10 keV, not in anything else (docs/08).
4. Solvation: with chemistry off Geant4 kills the electron and deposits `k`; Rust does the same. The 25 meV
   proposal of the chemistry-on branch and the navigator correction of the displacement are not reproduced.
   With chemistry on (M6), `G4DNAOneStepThermalizationModel.hpp:225-242` actively calls `ComputeStep`
   and replaces the displacement by `0.8 * safety` along its direction when `safety <= displacementMag`.
   Rust models infinite water, so this clip is inapplicable. The N1 audit in docs/08 records why the clip
   is operationally unreachable for the committed reference geometry and workload (metre safety versus
   nanometre thermalisation); the aggregate CSV has no clip counter and the Gaussian has no hard cutoff.
5. Primaries start at the origin in an infinite medium; the Geant4 world is a 1 m sphere of water, large
   against the 900 keV range (about 4 mm).

---

# M6: initial physicochemical species at 1 ps

The original mode is `physchem <out.csv> <seed> [E_eV:events,...]`. It uses option2 electron physics and `G4EmDNAChemistry_option3`. Its intended pre-reaction interpretation was disproved by the 2026-10-16 source audit: the first positive-step pre-callback follows zero-time IRT reactions at global 1 ps (docs/08, Q25/R17). Its existing code, arguments, physics and output format remain historical evidence. CSV columns are `energy_eV,n_events,species,sum_count,sum_count2,sum_r2_nm2`, where the last quantity sums squared distances of every alive molecule from the primary origin. The unchanged original gate fails; it does not validate a dissociation-only Rust M6 snapshot.

Build and smoke test first:

```bash
cd validation/geant4/g4rs_ref
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DGeant4_DIR=<geant4-install>/lib/cmake/Geant4
cmake --build build -j
./build/g4rs_ref physchem /tmp/physchem_smoke.csv 1 1000:2
```

**Generator status:** Q24 fixed 2026-10-16 (IRT time-step model, snapshot at the first dt > 0 step, secondaries transported, atomic de-excitation pointer nulled; see `BUILD_NOTES.md` section 5, issues G-K). The production run was done on this machine; the data are in `validation/reference/physchem.csv` and `provenance_m6.toml`. To regenerate:



```bash
./build/g4rs_ref physchem ../../reference/physchem.csv 20261019 \
  1000:20000,10000:2000,100000:200,900000:20
```

The default is about 1.1e8 physical interactions plus the 1 ps stage. From the M3 timing estimate, expect roughly 20 minutes to 2 hours serial; time the smoke case and extrapolate on the actual machine. Counts may be reduced at high energy if necessary because each row records `n_events`, but keep at least 20 events at 900 keV.

Exactly two files are needed in `validation/reference/`:

- `physchem.csv`, from the successful command above.
- `provenance_m6.toml`, with `geant4_version`, `geant4_commit`, `g4emlow_version`, `g4emlow_md5`, `physics_constructor = "G4EmDNAPhysics_option2"`, `chemistry_constructor = "G4EmDNAChemistry_option3"`, `scheduler_snapshot_ps = 1.0`, generator commit, date, exact command line, seed, compiler and platform.

Then run `cargo test -p dna-sim --test physchem_reference -- --ignored --nocapture`. The ignored test contains the pre-registered count and RMS acceptance criteria. It must remain ignored until both files exist and the generator exits successfully.

## M6 pre-IRT diagnostic (RR-001, awaiting data)

`physchem_prechem <out.csv> <seed> <E_eV:events,...>` is a **new mode**, compiled and linked against
Geant4 11.4.1/MSVC on 2026-10-17, never run by an agent. It uses the same option2/option3 constructors,
IRT selection, full cascade, liquid water, +z beam, Meesungnoen2002 and fluo/Auger-off settings. Only its
snapshot hook differs: a zero-step **post** callback waits until no water molecule remains in the main
list, then records the merged surviving daughters and stops **before the next IRT time selection**.
`G4ITStepProcessor2.cc:110-185` merges daughters/removes killed parents before this hook;
`G4DNAIRTMoleculeEncounterStepper.cc:431-467` defers IRT initialization while stopped water remains,
and `G4ITModelProcessor.cc:118-131` initializes it on the next time selection (paths under
`source/processes/electromagnetic/dna/{management,models}/src/`). The reaction table and water
recombination/dissociation processes are unchanged. This isolates M6 without implementing M7 chemistry.

The new mode requires explicit seed and statistics, refuses to overwrite its output, and exits nonzero
if a positive step or inter-molecular chemical reaction precedes capture, an event lacks its snapshot,
any captured molecule is not alive at global 1 ps, or the completed event count is wrong. An exit-0
complete run is required; compilation alone does not validate these runtime assumptions. A failed run
must be investigated, never accepted by removing a guard.

The CSV uses the original six columns, with the new provenance marker:

```text
# snapshot_boundary: after_dissociation_before_irt; global_time_ps: 1; chemical_reactions: 0
energy_eV,n_events,species,sum_count,sum_count2,sum_r2_nm2
```

Each species/energy row contains the event count, summed alive count, sum of **squared per-event
counts**, and sum of squared distances of all survivors from the primary origin, in nm^2. Zero-count
events contribute zero to both count moments; the `n_events` denominator includes them. Native Geant4
species names are mapped by the Rust consumer. The requested production run must contain all seven
dissociation species (H3Op, °OH, OHm, e_aq, H, H2, Oxy) at all four energies: exactly 28 rows. Reaction-only
species, incomplete energies/statistics, invalid moments, and duplicate rows are rejected.

After the user smoke test in `BUILD_NOTES.md`, run from `validation/geant4/g4rs_ref` with a **new** output:

```bash
g4rs_ref physchem_prechem ../../reference/physchem_prechem.csv 20261020 1000:20000,10000:2000,100000:200,900000:20
```

Commit the complete new CSV and `provenance_m6_prechem.toml`; never replace `physchem.csv` or its
provenance. The new provenance must contain the following, plus a nonempty `geant4_commit`, `generator`
(repository commit), `date`, `compiler`, `platform`, and explicit `generator_dirty` (boolean):

```toml
geant4_version = "11.4.1"
g4emlow_version = "8.8"
g4emlow_md5 = "328330009df633f7e9b3a9f445745298"
physics_constructor = "G4EmDNAPhysics_option2"
chemistry_constructor = "G4EmDNAChemistry_option3"
chemistry_time_step_model = "IRT"
snapshot_boundary = "after_dissociation_before_irt"
scheduler_snapshot_ps = 1.0
fluo_auger = "off"
seed = 20261020
command_line = "g4rs_ref physchem_prechem ../../reference/physchem_prechem.csv 20261020 1000:20000,10000:2000,100000:200,900000:20"
```

Record the exact executable invocation in the provenance; the command above assumes `g4rs_ref` is on
PATH. The consumer accepts that registered command (use PATH or a shell alias for the built executable).
Run only the paired ignored test after data and provenance are committed:

```bash
cargo test -p dna-sim --test physchem_reference m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact --nocapture
```

Criteria registered before these data exist: the same **z <= 4.5**, **15%** count difference when both
means exceed 0.1/event, and **20%** RMS-radius difference as the original gate, for all 28 rows. Provenance,
boundary and completeness checks run first. A passing stage-matched gate is required before M6 closes;
a failure leaves the gate open and Q21/R14's scheduler projection under investigation. The original
post-reaction reference/gate stays unchanged and is not expected to pass Rust M6.

## M6 900 keV five-boundary spatial diagnostic (RR-002, awaiting data)

The additive mode is `physchem_spatial_diagnostic <out.csv> <histories_per_seed> <seed,...>`. It is
fixed at a 900 keV electron from the origin along +z and retains option2 physics, option3 chemistry,
IRT selection, the full secondary cascade, liquid water and fluo/Auger-off settings. The initial run is:

```bash
g4rs_ref physchem_spatial_diagnostic ../../reference/spatial_geant4_900kev_40.csv 40 20261021,20261022,20261023,20261024,20261025
```

The mode refuses overwrite. It writes provenance comments, the exact Ticket 02 long-form header,
contiguous `(seed,history)` blocks, and a final `# complete: true` marker only after all requested runs
return. The boundaries are `physical_sites` (chemistry-producing excitation, ionisation, attachment
and terminal-electron sites), `solvated_electron_creation`, `pre_decay_water`,
`dissociation_products` (births across interleaved zero-time passes), and `final_survivors` (alive
after recombination/decay and before IRT/diffusion). Entity rows have count 1; an absent category has
one explicit count-0 row. Namespaces, track/parent IDs where available, source/creation ordinals and
daughter pass/processing ordinals preserve within-implementation lineage. Optional values are `NA`.

Moment columns are `count,sum_r2_nm2,sum_z_nm,sum_z2_nm2,sum_x2_plus_y2_nm2,max_radius_nm`. The 17
edges are `10^(k/2)` nm for `k=0..16` (1 nm through 100 mm), giving 18 half-open bins: underflow,
sixteen finite intervals and overflow. Positions remain relative to the primary origin. Runtime guards
require zero-step global 1 ps semantics, no preceding intermolecular reaction, complete lineage,
recognized water states and a final snapshot per event. Require exit 0, five completion lines, the
final marker, exact five x 40 coverage and strict parsing. Never commit a partial or smoke CSV.

The matching provenance is `provenance_m6_spatial_900kev_40.toml`; besides nonempty
`geant4_commit`, `generator`, `date`, `compiler`, and `platform`, it requires:

```toml
geant4_version = "11.4.1"
g4emlow_version = "8.8"
physics_constructor = "G4EmDNAPhysics_option2"
chemistry_constructor = "G4EmDNAChemistry_option3"
chemistry_time_step_model = "IRT"
diagnostic_stage = "global_1_ps_pre_irt"
fluo_auger = "off"
primary_energy_eV = 900000
seeds = [20261021, 20261022, 20261023, 20261024, 20261025]
histories_per_seed = 40
analysis_seed = 20261026
bootstrap_replicates = 20000
generator_dirty = false
command_line = "g4rs_ref physchem_spatial_diagnostic ../../reference/spatial_geant4_900kev_40.csv 40 20261021,20261022,20261023,20261024,20261025"
```

After both files are committed, run:

```bash
cargo test --release -p dna-sim --test physchem_reference m6_900kev_spatial_diagnostic -- --ignored --exact --nocapture
```

The consumer generates an independent Rust sample and uses 20,000 hierarchical bootstrap replicates,
resampling seed groups then histories while keeping all boundaries together. The sole primary is the
Rust/Geant4 pooled seven-species final-survivor RMS ratio. It reports a 95% primary interval,
simultaneous maximum-log-deviation boundary intervals, first detected divergence (not causation), and
secondary category, per-history, radial, maximum, longitudinal and transverse summaries. If primary
relative half-width `(U-L)/(2R)` exceeds 0.04, request complete 80-history samples in both
implementations, then 160; stop at five x 160 regardless of significance. This diagnostic changes
neither the existing 20% M6 gate nor M6's closure state.

