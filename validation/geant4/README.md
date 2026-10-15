# Geant4 reference data for the M2 validation

**Status:** the C++ generator in `g4rs_ref/` is **untested** (written without a Geant4 installation,
never compiled). The Rust comparison tests are in `crates/dna-sim/tests/geant4_reference.rs` and are
`#[ignore]`d until the three CSV files below and `provenance.toml` exist in `validation/reference/`.
No Geant4 number has been invented or embedded anywhere in the repository.

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
