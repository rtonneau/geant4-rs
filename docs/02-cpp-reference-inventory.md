# B. C++ reference inventory

Root: `rtonneau/geant4` @ `27ea526b420dd72f510af81f6828edef0ff635f5` (squashed; CMake version 11.4.1).
`D` = `source/processes/electromagnetic/dna`, `C` = `source/physics_lists/constructors/electromagnetic`.
Tags: **[V]** read in this session; **[U]** not yet read (audit task).
A pre-built include graph of the DNA folder exists: `D/geant4_dna_nodes.csv`, `D/geant4_dna_edges.csv`,
`D/geant4_dna_graph_summary.txt` (209 files, 612 internal edges). Use it to find dependencies; do not trust it as physics evidence.

## B.1 Physics-list entry points [V]
| Item | File | Notes |
|---|---|---|
| Option dispatch (opt 2/4/6/8) | `C/src/G4EmDNAPhysics_option{2,4,6,8}.cc` | call `G4EmDNABuilder::ConstructDNAElectronPhysics(1 MeV, opt,...)`, `ConstructDNAProtonPhysics(500 keV, 300 MeV, opt,...)` |
| Model registration & energy handover | `C/src/G4EmDNABuilder.cc` (electron ~L309-440, proton ~L440-520) | constants L109-113: `lowEnergyRPWBA=100 MeV`, `lowEnergyMSC=1 MeV`, `lowEnergyProtonIoni=2 MeV`, `highEnergyMillerGrean=0.5 MeV`, `highEnergyChargeExchange=100 MeV` |
| Standard EM above DNA range | `G4EmDNABuilder::ConstructStandardEmPhysics` (L130-) | e-: MollerBhabha activated >= 1 MeV, Seltzer-Berger brems, Urban/GS/WentzelVI msc active >= 1 MeV. Photon/positron too (out of scope) |
| Chemistry constructors | `C/src/G4EmDNAChemistry{,_option1,_option2,_option3}.cc` | option3 = PARTRAC pre-chemistry + RITRACKS chemistry parameters, IRT default; used by `chem6` |
| Dissociation channels | `C/src/G4ChemDissociationChannels_option1.cc` | probabilities in D.2 |

## B.2 Electron models (liquid water) and energy ranges [V]
| Process | Option 2 (default; recommended first target) | Option 4 | Option 6 |
|---|---|---|---|
| Solvation (thermalisation) | `G4DNAElectronSolvation` below 7.4 eV | below 10 eV | below 11 eV |
| Elastic | `G4DNAChampionElasticModel` 7.4 eV-1 MeV | `G4DNAUeharaScreenedRutherfordElasticModel` 10 eV(builder)-1 MeV (model's own 9 eV low limit) | `G4DNACPA100ElasticModel` to 250 keV, then Champion to 1 MeV |
| Excitation | `G4DNABornExcitationModel` (alias of `...Model1`, `D/models/include/G4DNABornExcitationModel.hh` L39-40), table `dna/sigma_excitation_e_born`, upper limit 1 MeV; lower limit [U] | `G4DNAEmfietzoglouExcitationModel` 8 eV-10 keV, Born above 10 keV | CPA100 to 250 keV then Born |
| Ionisation | `G4DNABornIonisationModel1` 11 eV-1 MeV | Emfietzoglou 10 eV-10 keV, Born1 above | CPA100 to 250 keV then Born1 |
| Vibrational excitation | `G4DNASancheExcitationModel` 2-100 eV | none | none |
| Attachment | `G4DNAMeltonAttachmentModel` 4-13 eV | none | none |
Model-own limits: `D/models/src/G4DNAChampionElasticModel.cc:46-47`, `G4DNABornIonisationModel1.cc:126-127`,
`G4DNASancheExcitationModel.cc:48-49`, `G4DNAMeltonAttachmentModel.cc:49-50`, `G4DNAEmfietzoglouIonisationModel.cc:78-79`.
**Other electron files not examined [U]:** `G4DNABornAngle`, `G4DNAEmfietzoglouExcitationModel` details, secondary-energy/angle sampling code in each `SampleSecondaries`, `G4DNADoubleIonisation/Triple/Quadruple` (appear in option8/PTB lines only).

## B.3 Proton models, option 2 [V]
| Process | Model | Range |
|---|---|---|
| Elastic | `G4DNAIonElasticModel` (`dna/sigmadiff_cumulated_elastic_proton_HTran.dat`) | 0-1 MeV (`G4DNAIonElasticModel.cc:51-52`) |
| Excitation | `G4DNAMillerGreenExcitationModel` | 10 eV-500 keV (`...cc:107-108`); then `G4DNABornExcitationModel` 500 keV-100 MeV; then `G4DNARPWBAExcitationModel` 100-300 MeV |
| Ionisation | `G4DNARuddIonisationExtendedModel` (data `dna/sigma_ionisation_{p,h,...}_rudd`) up to 500 keV; `G4DNABornIonisationModel1` (proton 500 keV-100 MeV, `...Model1.cc:212-213`); `G4DNARPWBAIonisationModel` 100-300 MeV |
| Charge decrease (p -> H) | `G4DNADingfelderChargeDecreaseModel` 100 eV-100 MeV (`...cc:100-101`) | |
| Charge increase (H -> p) | `G4DNADingfelderChargeIncreaseModel` [U] range | |
Protons must be tracked with a **charge state (p / H)**; secondary-electron production comes from ionisation of both. Above ~100 MeV RPWBA; above 300 MeV not DNA.
Standard hadron/ion physics register for protons above DNA range: `StandardHadronPhysics(...)` (`G4EmDNABuilder.cc:226`) [U details].
Not in scope initially: alpha/He (Miller-Green, Rudd, Ion models exist), ions (`G4DNAGeneralIonIonisationModel`).

## B.4 Datasets [V names, U contents]
All via `G4LEDATA`, `dna/` subfolder of dataset **G4EMLOW 8.8** (`cmake/Modules/G4DatasetDefinitions.cmake:20-26`, MD5 `328330009df633f7e9b3a9f445745298`).
The data are **not in the reference repo**; they must be fetched and checksummed by a script (M1), licence reviewed before any copy is committed.
Files named in code: `sigma_elastic_e_champion` (cross section; [U] exact name), `sigmadiff_cumulated_elastic_e_champion_hp.dat`,
`sigma_excitation_e_born`, `sigma_excitation_p_born`, `sigmadiff_ionisation_e_born.dat`, `sigmadiff_cumulated_ionisation_e_born_hp.dat`,
`sigmadiff_{,cumulated_}ionisation_p_born{,_hp}.dat`, `sigma_excitationvib_e_sanche.dat`, Melton attachment table [U name],
`sigma_ionisation_{p,h,alphaplusplus,alphaplus,he,li,be,b,c,n,o,si,fe}_rudd`, `sigmadiff_cumulated_elastic_proton_HTran.dat`,
CPA100 / Emfietzoglou tables (options 4/6). Interpolation: `G4DNACrossSectionDataSet` + `G4LogLogInterpolation` (log-log) for total cross sections;
differential tables use 2-D interpolation inside the model (`QuadInterpolator`, `LogLogInterpolate`, `LinLogInterpolate`).
"hp"/"fasterCode" selects *cumulated* tables (pre-integrated CDF) vs. on-the-fly integration of the differential table: **two distinct sampling algorithms**; choose one (Q4).
Extrapolation/boundary behaviour of `G4DNACrossSectionDataSet`: see B.7.1 [V].

## B.5 Physicochemical & chemical stage [V unless noted]
| Concept | C++ | Rust target |
|---|---|---|
| Hand-off from physics | `D/utils/src/G4DNAChemistryManager.cc` (`CreateWaterMolecule` L600, `CreateSolvatedElectron` L668) | `physchem::events_to_species` |
| Dissociation/decay channels | `G4ChemDissociationChannels_option1.cc` | data table in `data/chemistry/` (TOML) |
| Displacement of products | `D/processes/src/G4DNAWaterDissociationDisplacer.cc` (RMS offsets, e.g. 2.4/0.8/1.1/1.6 nm) | `physchem::displacer` |
| Electron thermalisation | `D/models/src/G4DNAOneStepThermalizationModel.cc` (Meesungnoen 2002 fits, Terrisol 1990 table, Gaussian 3-D penetration from mean) ; `G4DNAElectronSolvation` | `physchem::thermalisation` |
| Species | `D/molecules/types/src/G4*.cc` (D, charge, radius) | `chemistry::species` |
| Reactions (option3) | `C/src/G4EmDNAChemistry_option3.cc` L~100-640 | `data/chemistry/reactions_option3.toml` |
| Reaction types | `SetReactionType(data, model)`: type 1 (partially diffusion-controlled) unless SBS (L770-774) | `ReactionKind` enum |
| IRT | `D/models/src/G4DNAIRT.cc` (space binning by `fRCutOff`, per-reactant sampling, Gaussian propagation to common time), `G4DNAIndependentReactionTimeStepper.cc`, `G4IRTUtils.cc` | `chemistry::irt` |
| Partially diffusion-controlled time sampling | `G4DiffusionControlledReactionModel.cc` (L82-170: Smoluchowski radius, `kdif=4*pi*D*R*N_A`, erfc) | `chemistry::reaction_model` |
| Smoluchowski (totally diffusion-controlled) | `G4DNASmoluchowskiReactionModel.cc` | same |
| SBS | `G4DNAMolecularStepByStepModel`, `G4DNAMoleculeEncounterStepper`, `G4DNASmoluchowskiDiffusion`, `G4Scheduler` [U details] | `chemistry::sbs` (test oracle) |
| Brownian step | `G4DNABrownianTransportation`; IRT uses sigma = sqrt(2 D dt) per axis (G4DNAIRT.cc ~L215) | `chemistry::diffusion` |
| Scavengers/IT infrastructure | `G4DNAScavengerMaterial`, `G4Scheduler`, `G4ITTrackHolder`, KD-tree, octree [U] | not in M1-M9 |
| Mesoscopic/IRT-syn | `G4DNAEventScheduler`, `G4DNAUpdateSystemModel`, `G4DNAGillespieDirectMethod`; example `UHDR` [U] | later |

Species diffusion coefficients in `D/molecules/types/src` [V]: e_aq 4.9e-9, OH 2.8e-9, H 7e-9, H2 5e-9, H2O2 1.4e-9, H3O+ 9e-9, HO2 2.3e-9, O2 2.4e-9 m^2/s;
e_aq radius 0.23 nm ("K.D. Jordan, Science 306 p.618"). Others [U].
Rate constants in option3 are `x * (1e-3 m3 / (mole s))` = M^-1 s^-1 (e.g. `H+H` 0.503e10, `e_aq+H` 2.50e10, `OH+OH` 0.55e10, `e_aq+OH` 2.95e10, `H3O+ + OH-` 1.13e11).
Complete verified table to be transcribed in M7 with a script that diffs against the C++ source.

## B.6 Examples [V names]
`examples/extended/medical/dna/`: `dnaphysics`, `mfp`, `range`, `slowing`, `spower`, `wvalue`, `microdosimetry`, `chem1-6`, `scavenger`, `UHDR`.
- `chem6`: G(t) vs LET, G4EmDNAPhysics_option2 + G4EmDNAChemistry_option3 (IRT default), "pseudo-infinite" water box, paper J. Appl. Phys. 125 (2019) 104301.
  **Primary chemistry validation target.**
- `UHDR`: prototype (README says so) with bounded box 3.2/1.6 um, pulse structure, periodic boundaries, mesoscopic + IRT-syn. Deferred to M11; used to define boundary-condition and multi-track requirements now (see 09, Q6).
- `chem1-5`, `scavenger`: Fricke, oxygen, scavenger cases for kinetics tests.
- `range`, `mfp`, `slowing`, `spower`: reference generators for physical validation (M2-M6).

## B.7 Audit gaps (must be closed before the corresponding milestone)
G1 **CLOSED (B.7.3 item 3)** `SampleSecondaries` of the option2 electron models and proton Rudd/Born/Miller-Green (energy-transfer, angle, binding energy, fate of the incident e-). `G4DNARuddAngle` remains for M5.
G2 **CLOSED (M1, see B.7.1)** `G4DNACrossSectionDataSet` interpolation/extrapolation, file units/scale factors.
G3 **CLOSED for the DNA range (B.7.3 item 6)**: charge-increase window 100 eV-100 MeV. Standard hadron physics above the DNA range is out of scope (energy limit decision Q1).
G4 Exact `chem6` scoring/energy normalisation -> M8.
G5 SBS time-step computer and `G4Scheduler` -> M7 (oracle only).
G6 **CLOSED (B.7.3 item 4)** default solvation model = Meesungnoen2002. `G4DNATransformElectronModel` (0-0.025 eV) not yet read -> M6.
G7 Temperature and `G4IRTUtils::GetRCutOff`, time-step parameters (`G4EmParameters` chemistry section ~L1570-1630) -> M7.
G8 **CLOSED (M1, see B.7.2)** Electron production cut/tracking cut behaviour for DNA models (`SetLowEnergyLimit` kill below limit?). Residual [U] items are listed in B.7.2.

### B.7.1 G2 closed: `G4DNACrossSectionDataSet`, `G4EMDataSet`, `G4LogLogInterpolation` [V, M1]
Paths relative to `source/processes/electromagnetic`: `DS` = `dna/utils/src/G4DNACrossSectionDataSet.cc`, `EM` = `lowenergy/src/G4EMDataSet.cc`, `LL` = `lowenergy/src/G4LogLogInterpolation.cc`.
- **File location.** `$G4LEDATA/<name>.dat` (`DS:466-480`); `G4EMDataSet`/`LoadData` receive the name *without* `.dat`.
- **Reader** (`DS:83-240`, `LoadData`). Byte-wise state machine: `#` starts a comment that runs to the end of the line (`DS:163`); `\r` and `\n` both terminate a line (`DS:117-118`), so CRLF and LF files are equivalent and blank lines are skipped; space and tab are interchangeable separators. Column 0 is the energy, every further column is one *component* (component `i` = column `i+1`). A file with fewer than two columns, or any column shorter than column 0, is `FatalException` (`DS:192-215`). Numbers are read with `operator>>` into `double` (a malformed token silently ends the row; the Rust parser reports an error instead).
- **Units.** Energy values are multiplied by `unitEnergies` and data values by `unitData` given to the constructor (`DS:226-227`). The scale factor is supplied by each *model* (table in docs/10); it is never stored in the file.
- **Log arrays.** In the log copy any value `<= 0` (in any column, including the energy column) is replaced by `1e-300` before `log10` (`DS:148`), then `log10(unit)` is added (`DS:228-229`). **Consequence kept in Rust:** at an *interior* node whose tabulated value is 0 the interpolation returns `1e-300 x unit`, not 0, and between a zero node and a non-zero node the value rises as `10^(-300 (1-f)) x v2` (`f` = fractional log-position), i.e. it is numerically zero except in the last ~1 % of the interval. Exactly at the first or last node the stored value (possibly 0) is returned.
- **Lookup.** `G4DNACrossSectionDataSet::FindValue(E)` (`DS:483-498`) = sum over components of `G4EMDataSet::FindValue`; the DNA models mostly call `GetComponent(i)->FindValue(E)` per level/shell and sum or sample themselves.
- **Outside the table: constant clamp.** `G4EMDataSet::FindValue` (`EM:167-183`): `E <= first node` returns `data[0]`, `E >= last node` returns `data[last]`, otherwise `FindLowerBound` (`EM:445-459`, last node `<= E`) and `Calculate`. This is a *silent constant extrapolation*; **Rust deviates on purpose:** outside `[E_first, E_last]` the table returns `UnsupportedEnergy`. The clamp is not reachable inside a model's own validity window for the option2 tables except for Melton (table ends at 12.3 eV, value 0, while the model window is 4-13 eV, `G4DNAMeltonAttachmentModel.cc:49-50`): Geant4 returns 0 for 12.3-13 eV, Rust (M1) returns the typed error and the M2 model layer must decide explicitly. Docs/10 lists table range vs model window for every file.
- **Interpolation.** `G4LogLogInterpolation::Calculate` with log arrays (`LL:95-127`): `log d = log d1 + (log d2 - log d1)(log E - log E1)/(log E2 - log E1)`, `d = 10^log d`. The 4-argument overload without log arrays (`LL:59-90`) is *not* used by `LoadData` (it returns 0 when a node is zero) and is not reproduced.
- **Different scheme in two readers.** `G4DNASancheExcitationModel` and the differential-table readers (Champion, Ion elastic, Born1, RPWBA) do not use this class: they read the file with `operator>>` at `Initialise` and interpolate lin-lin / log-lin as described in docs/10 (Sanche `G4DNASancheExcitationModel.cc:407-418`; Champion `G4DNAChampionElasticModel.cc:316-445`; Ion elastic `G4DNAIonElasticModel.cc:369-495`; Born1 `G4DNABornIonisationModel1.cc:819-905, 1012-1290`).
- **Stream-EOF artefacts in those readers are benign.** `while(!in.eof())` reads one extra, failed iteration when the file ends with a newline; the failed `>>` leaves the previous values untouched, so only a duplicate of the last entry is appended (this is why Born1 guards the scale factor with `!eof()`, `:199`). The Rust readers do not reproduce the duplicate.
- **Valid domain of the 2-D lookups.** `Theta` and `TransferedEnergy` index `upper_bound(k) - 1` in a vector that starts with a sentinel `0.`: the C++ is undefined for `k` below the first tabulated energy and, for Champion (no `1-1e-12` protection), for `k` equal to the last one. Ion elastic and Born1 apply `k = k (1 - 1e-12)` at the last energy (`G4DNAIonElasticModel.cc:385`, `G4DNABornIonisationModel1.cc:1034, 1146`). The Rust lookups return `UnsupportedEnergy` below the first energy and treat the last energy as in Born1 (1e-12 protection) for all tables.

### B.7.2 G8 closed: model limits and kill behaviour [V, M1]
- **Validity window.** Every cross-section function returns 0 outside its closed window `[Low, High]`: Champion `G4DNAChampionElasticModel.cc:253`, Melton `:199`, Sanche `:207`, Born excitation1 `:157`, Born1 ionisation `:363` (separate windows per particle, e- 11 eV-1 MeV `:126-127`), RPWBA excitation `:158`. Ion elastic differs: window 0-1 MeV, `sigma = DBL_MAX` below `killBelowEnergy = 100 eV` (`G4DNAIonElasticModel.cc:50, 264-267`). Windows are evaluated at the pre-step energy.
- **Effective electron windows, option2** (`G4EmDNABuilder.cc::ConstructDNAElectronPhysics` L309-440 and model `Initialise`): solvation 0-7.4 eV, Champion 7.4 eV-1 MeV, Born excitation 9 eV-1 MeV (`BornExcitationModel1.cc:99-100`, overrides the builder's lower limit), Born1 ionisation 11 eV-1 MeV, Sanche 2-100 eV, Melton 4-13 eV. Born1 runs the cumulated ("fast") tables because option2 sets `SetDNAFast(true)` (`G4EmDNAPhysics_option2.cc:54`, read at `Born1.cc:72`). This closes the "lower limit [U]" of B.2: **9 eV** for excitation.
- **No energy cut.** DNA `SampleSecondaries` implementations ignore the `tmin/maxEnergy` arguments (unnamed parameters), so secondaries are *always* produced, whatever the production cuts. `G4VEmProcess.cc` contains no lowest-kinetic-energy kill, and option2 only sets `SetDNAFast` (grep of `G4EmDNAPhysics_option2.cc`). The only terminations are model-specific:
  - **Solvation** (`G4DNAOneStepThermalizationModel.hpp:137-146, 182-185`; `G4DNATransformElectronModel.cc:86-132`): cross section `DBL_MAX` for `E <= 7.4 eV` (the builder sets the high limit), then `fStopAndKill` with local deposit `= E_kin` and a solvated electron is created by the chemistry manager. Default model: `fMeesungnoen2002eSolvation` (`G4EmLowEParameters.cc:78`). Which positions are produced is M6 (G6).
  - **Melton attachment**: absorbed (`fStopAndKill`, local deposit `= E_kin`, `G4DNAMeltonAttachmentModel.cc:238-247`).
  - **Ion elastic** (proton/H): `E < 100 eV` -> `fStopAndKill`, deposit `E` (`IonElastic.cc:310-313`). **Rudd extended**: `E <= 100 eV` -> `fStopButAlive`, deposit `E` (`Rudd...Extended.cc:296-300`); below `fElow` the table is scaled linearly, `sigma = sigma(fElow) E/fElow` (`:265-266`), a *model-level* extrapolation (M5).
  - Born1 returns without changing the track if `E < binding energy` of the sampled shell (`Born1.cc:472`); Born excitation and Sanche change nothing when `E - E_exc <= 0`. Both are unreachable inside the windows above except through table rounding and are asserted in the M2 energy ledger.
- **Electron energy accounting at ionisation** (`Born1.cc:470-575`): scattered `= E - B - W_kin`, local deposit `= B` (minus any de-excitation secondaries); with `statCode` (stationary) the primary keeps `E` and deposits `E - scattered`.
- **Remaining [U]:** none of the items listed here at the end of M1 remain open; they are resolved in B.7.3 (G1, G3, G6 and the window-boundary rule).

### B.7.3 Resolution of the residual [U] items [V, source read after M1]
Paths relative to `source/processes/electromagnetic` unless stated. `M` = `dna/models/src`, `U` = `utils`.

**1. Rudd shell order vs. binding-energy lists (M5).** Table column `i+1` of `sigma_ionisation_{p,h}_rudd` is shell `i` (`G4DNARuddIonisationExtendedModel::SelectShell` loops `GetComponent(0..4)`, `M/...Extended.cc:371-388`). Shell index 0..4 is ascending binding energy, 4 = oxygen K shell (`:559`, atomic de-excitation only for `shell == 4`, `:321`). Confirmed on the committed tables: first energy with non-zero cross section per column is 11, 14, 18, 35, 545 eV (`sigma_ionisation_e_born`) and 551 eV for column 5 of `sigma_ionisation_p_rudd`. **Two different binding-energy sets are used in the same model:** `Bj = {12.60, 14.70, 18.40, 32.20, 539} eV` (`:66-67`) inside the Rudd probability function (`MaxEnergy`, `:425`, sets `bEnergy`) and `G4DNAWaterIonisationStructure = {10.79, 13.39, 16.05, 32.30, 539.0} eV` (`U/src/G4DNAWaterIonisationStructure.cc:31-35`) for the energy bookkeeping (`useDNAWaterStructure{true}`, header `:175`; `SampleSecondaries` `:304-305`). Rust must keep both: `Bj` for the secondary-energy distribution, water-structure energies for `scattered = E - B - W` and the local deposit `exc = B - (de-excitation energy)`.

**2. RPWBA `TransferedEnergy` and its own code (M5).** `G4DNARPWBAIonisationModel` (`M/G4DNARPWBAIonisationModel.cc:262-340, 505-700`) is a copy of the Born1 algorithm restricted to protons: same shell sampling (`RandomSelect`, with the `k < 19 eV` rejection loop that only applies to electrons), same 2-D table lookup (`upper_bound` on the energy grid, then on the cumulative-probability column, `random <= last cumulative` branch vs. the "cum xs is zero for k1" branch), same `Interpolate` (log-log if not `fasterCode`, log-lin or lin-lin if `fasterCode`, see item 3). One implementation in Rust, parameterised by the table set, covers Born1-electron, Born1-proton and RPWBA. RPWBA excitation is the Born excitation algorithm with its own table (`sigma_excitation_p_RPWBA`).

**3. Sampling code of each `SampleSecondaries` (G1, M2/M3/M5).**
- *Born1 ionisation* (`M/G4DNABornIonisationModel1.cc:440-600, 926-1290`): (a) shell: pick shell with probability proportional to the per-shell cross section at `k` (`RandomSelect`, from the **last** component downwards, `:945-966`; equivalent to a CDF pick, order only matters for the random-number stream); in fast mode an electron below 19 eV re-draws while shell == 2 (`:445-451`). (b) `B = IonisationEnergy(shell)`; return unchanged if `k < B`. (c) Ejected energy (fast mode): `W = TransferedEnergy(k/eV, shell, u)*eV - B`, `W = 0` if negative (`:985-1007`); `TransferedEnergy` is a 2-D inverse-CDF lookup between the two tabulated energies bracketing `k` with `Interpolate` (item 3b). (d) Ejected direction from `G4DNABornAngle` (below). (e) **Primary direction (electron only) from momentum conservation**: `p_final = p_in - p_delta`, normalised (`:487-497`); a proton keeps its direction. (f) `scattered = k - B - W`, deposit `B`; for shell 4 and a non-null atomic de-excitation the Auger/fluorescence secondaries are subtracted from `B` (`:511-535`), otherwise unchanged; negative remaining deposit is a fatal error.
- *Interpolation inside the Born1/RPWBA lookups* (`Interpolate`, `:819-880`): not fast mode: log10-log10; fast mode: `log10(value)` linear in `e` (log-lin) when both values are non-zero, lin-lin when either is zero (cumulative probability 0). The explicit `Interpolate(valueK1, valueK2, k, 0., value2)` branch (`random >` last cumulative probability of the lower energy) interpolates between 0 and the upper-energy value. `QuadInterpolator` order: energy at fixed probability on both bracketing energies, then across energy (`:883-905`).
- *`G4DNABornAngle`* (`M/G4DNABornAngle.cc`, fully read): electron: `W < 50 eV` isotropic; `50 <= W <= 200 eV` with probability 0.1 isotropic, otherwise `cos(theta) = u (sqrt(2)/2)`; `W > 200 eV`: `sin^2 = (1 - W/k)/(1 + W/(2 m_e c^2))`, `cos = sqrt(1 - sin^2)`. Proton/H: `W > 100 eV`: `cos = min(sqrt(W/W_max), 1)`, `W_max = 2 m_e c^2 tau (tau + 2)`; else isotropic. `phi` uniform; direction rotated onto the primary direction.
- *`G4DNARuddAngle`* is the Rudd counterpart (not yet read, used for p-ionisation <= 500 keV: M5 audit item).
- *Rudd extended ionisation* (`M/...ExtendedModel.cc:285-368`): `E <= fLowestEnergy` -> stop (`fStopButAlive`, deposit E); shell via `SelectShell(E*fMassRate)`; ejected energy by rejection sampling of `ProbabilityFunction` with Dingfelder/Rudd parameters (`MaxEnergy`, `:390-440`: K-shell set A1..alpha = 1.25, 0.5, 1.0, 1.0, 3.0, 1.10, 1.30, 1.0, 0.0, 0.66; other shells 1.02, 82.0, 0.45, -0.80, 0.38, 1.07, 11.6, 0.60, 0.04, 0.64); `scattered = E - B - W`, deposit `B` (minus de-excitation), chemistry species created for the shell.
- *Champion elastic* (`M/G4DNAChampionElasticModel.cc:277-365`): `cos(theta)` from `RandomizeCosTheta`/`Theta(k, u)` = 2-D lookup in `sigmadiff_cumulated_elastic_e_champion_hp.dat` with `QuadInterpolator`, `phi` uniform, energy unchanged, **no energy deposit**.
- *Born excitation* (`M/G4DNABornExcitationModel1.cc:177-212`): level by partial cross sections; `newEnergy = k - E_exc(level)` with the **excitation structure {8.22, 10.00, 11.24, 12.61, 13.77} eV** (`U/src/G4DNAWaterExcitationStructure.cc:31-35`); direction unchanged; deposit `E_exc`. **Quirk to reproduce knowingly:** when `newEnergy <= 0` the particle change is not updated (no loss, no deposit) but the excited molecule is still created for the chemistry (`:208-212`). Unreachable inside the window except at table rounding; assert in the M2 ledger and count it.

**4. Solvation product positions (G6, M6).** `G4TDNAOneStepThermalizationModel::SampleSecondaries` (`M/G4DNAOneStepThermalizationModel.hpp`, lines 112-148 of the stripped text): for `E <= HighEnergyLimit()` (7.4 eV, option2): kill, **deposit the full kinetic energy**, and, if chemistry is active, create one solvated electron at `position + displacement` (`CreateSolvatedElectron`). The displacement comes from the penetration model chosen by `G4EmLowEParameters::dnaElectronSolvation`, whose initial value is `fMeesungnoen2002eSolvation` (`utils/src/G4EmLowEParameters.cc:78`): `Meesungnoen2002` (`M/G4DNAOneStepThermalizationModel.cc:95-107, 123-142`): `r_mean(E) = nm * sum_{i=0}^{12} gCoeff[12-i] E[eV]^i` for `E > 0.1 eV` (0 otherwise); displacement = three independent Gaussian components with `sigma_1D = r_mean sqrt(pi)/2^{3/2} = 0.62665706865775 r_mean` (so the mean radius of the Maxwell-distributed 3-D displacement equals `r_mean`); if `r_mean == 0` the electron is placed `1e-3 nm` away in a random direction (to avoid coinciding with the water molecule). Alternatives in the same factory: `Meesungnoen2002_amorphous` (6th-order fit), `Terrisol1990` (table 0.2-9 eV, Gaussian 3-D with `sigma_1D = sigma_3D sqrt(1/(3-8/pi))`, **FatalException above 9 eV**), `Ritchie1994` (`r_mean = 1.8 nm E[eV]`, same Gaussian), `Kreipl2009` (`Meesungnoen2002` `r_mean`, radius `r_mean * Gamma(shape 2, rate 2)` (`G4RandGamma::shoot(2,2)`, mean 1) along a random direction). If the straight segment leaves the geometry the electron is placed at `0.8 x` the navigator safety along the displacement. The chemistry state after solvation (`e_aq` thermalised, 25 meV proposed kinetic energy) follows. Rust M6: implement `Meesungnoen2002` first (default), keep the others behind the same trait; the fit validity range above ~7 eV is not stated in the source (document as unverified in M6).

**5. Window boundaries (M2 test).** Two layers, both [V]:
- *Within one model*: closed window `[Low, High]` (item G8 above).
- *Between models of one process*: `G4EmModelManager`/`G4RegionModels::SelectIndex(e)` (`U/include/G4EmModelManager.hh:97-104`) starts at the last model and decrements while `e <= lowKineticEnergy[idx]`; the model for `e` is therefore the one with `low < e <= high` (the **lower** model owns the shared boundary, e.g. exactly 500 keV is Miller-Green for proton excitation, exactly 1 MeV the DNA model, exactly 250 keV CPA100).
- DNA processes are discrete and the energy does not change along a step, so a "step ending on a boundary" means a post-interaction energy that equals a boundary exactly; the next step then uses the rule above. Rust: `ProcessTable::select(e)` implements `(low, high]` for the model chain and `[low, high]` for the first model, with unit tests at the exact boundary values and at `nextafter` neighbours.

**6. Miller-Green and Dingfelder parameters (formulas in code, no files) (M5).** All values read from the source:
- *Miller-Green excitation* (`M/G4DNAMillerGreenExcitationModel.cc:397-470`, Dingfelder et al., Radiat. Phys. Chem. 59 (2000) 255, Eq. 34, Table 2): `sigma_j(T) = zEff^2 sigma0 (z a_j)^{w_j} (T - E_j)^nu / (J_j^{w_j+nu} + T^{w_j+nu})`, `sigma0 = 1e8 barn`, `nu = 1`, `z = 10`; `a_j = {876, 2084, 1373, 692, 900} eV`, `J_j = {19820, 23490, 27770, 30830, 33080} eV`, `w_j = {0.85, 0.88, 0.88, 0.78, 0.78}`, **level energies `E_j = {8.17, 10.13, 11.31, 12.91, 14.50} eV`** (used for both the cross section and the energy loss, `:362`; they differ from the water excitation structure {8.22, 10.00, 11.24, 12.61, 13.77} eV used by Born, an inconsistency in Geant4 to be documented, not corrected). Proton: `zEff = 1` (charge + lepton number, Slater corrections set to 0), `T` not rescaled; hydrogen atom: `a_j` replaced by `0.75 a_j`, `zEff = 1`. Window 10 eV-500 keV; level chosen proportional to partial cross section; deposit `E_j`; molecule created only when `newEnergy > 0`.
- *Dingfelder charge decrease p -> H* (`M/G4DNADingfelderChargeDecreaseModel.cc:469-540`): `sigma(T) = f0 10^{y(x)} m^2`, `x = log10(T/eV)`, `y = a0 x + b0` for `x < x0`; `a0 x + b0 - c0 (x - x0)^{d0}` for `x0 <= x < x1`; `a1 x + b1` for `x >= x1`. Proton: `f0 = 1, a0 = -0.180, a1 = -3.600, b0 = -18.22, b1 = -1.997, c0 = 0.215, d0 = 3.550, x0 = 3.450, x1 = 5.251`; window 100 eV-100 MeV. Final state (`:279-350`): `outK = E - E m_e/m_p - 10.79 eV + 13.6 eV`, track killed, deposit `10.79 eV` (water binding), a hydrogen atom emitted with `outK`, ionised-molecule chemistry species created. **Energy is not conserved in Geant4's bookkeeping**: out `= E - E m_e/m_p + 13.6 eV`, i.e. a net `+13.6 eV - E m_e/m_p` per capture (the `E m_e/m_p` of the captured electron is simply dropped). Rust: reproduce for Geant4 comparison, report the ledger violation separately (`ledger_nonconservation_ev`) so that it does not hide real errors; decision on correcting it is open (new Q11).
- *Dingfelder charge increase H -> p* (`M/G4DNADingfelderChargeIncreaseModel.cc:82-128, 286-306`): same functional form (`f0 = 1, a0 = 2.25, a1 = -0.75, b0 = -32.10, c0 = 0.600, d0 = 2.40, x0 = 4.60`; `x1`, `b1` computed from the continuity condition at first use, `x1 = x0 + ((a0-a1)/(c0 d0))^{1/(d0-1)}`); window for hydrogen 100 eV-100 MeV; `outK = E - 13.6 eV` (incoming-particle binding), deposit `13.6 eV` (`:300-306`); note that the proton deposit/E balance here is conserved. Closes G3's range question.

**7. Uehara screened-Rutherford elastic (option4 only, M4).** No data file. `G4DNAUeharaScreenedRutherfordElasticModel` (`M/...ElasticModel.cc`, 604 lines) holds polynomial coefficients in `Initialise` (`betaCoeff`, `deltaCoeff`, `gamma035_10Coeff`, `gamma10_100Coeff`, `gamma100_200Coeff`, six/five/three terms each) and uses `intermediateEnergyLimit = 200 eV`, window 9 eV-1 MeV; above 200 eV the screened-Rutherford form `etaC K z^{2/3} / (tau (2 + tau))` with `etaC = 1.198` for `E < 50 keV` else `1.13 + 3.76 z^2/(137^2 beta^2)`, `tau = E/(m_e c^2)`. Not needed for option2; transcribe and test in M4 if option4 is requested.

New open point from this audit: **Q11** - whether to reproduce or correct Geant4's non-conserving charge-exchange bookkeeping (item 6). Default: reproduce for L3 comparison, flag in outputs.
