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
G1 `SampleSecondaries` of each used model (energy-transfer, angle, binding energy, fate of the incident e- below 7.4/11 eV) -> M2/M3.
G2 **CLOSED (M1, see B.7.1)** `G4DNACrossSectionDataSet` interpolation/extrapolation, file units/scale factors.
G3 Proton charge-increase limits; standard hadron physics handover -> M5.
G4 Exact `chem6` scoring/energy normalisation -> M8.
G5 SBS time-step computer and `G4Scheduler` -> M7 (oracle only).
G6 Default solvation model (`G4DNASolvationModelFactory::GetMacroDefinedModel`) and `G4DNATransformElectronModel` (0-0.025 eV) -> M6.
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
- **Stream-EOF artefacts in those readers are benign.** `while(!in.eof())` reads one extra, failed iteration when the file ends with a newline; the failed `>>` leaves the previous values untouched, so only a duplicate of the last entry is appended (this is why Born1 guards the scale factor with `!eof()`, `:197-198`). The Rust readers do not reproduce the duplicate.
- **Valid domain of the 2-D lookups.** `Theta` and `TransferedEnergy` index `upper_bound(k) - 1` in a vector that starts with a sentinel `0.`: the C++ is undefined for `k` below the first tabulated energy and, for Champion (no `1-1e-12` protection), for `k` equal to the last one. Ion elastic and Born1 apply `k = k (1 - 1e-12)` at the last energy (`G4DNAIonElasticModel.cc` `Theta`, `Born1.cc:1034, 1146`). The Rust lookups return `UnsupportedEnergy` below the first energy and treat the last energy as in Born1 (1e-12 protection) for all tables.

### B.7.2 G8 closed: model limits and kill behaviour [V, M1]
- **Validity window.** Every cross-section function returns 0 outside its closed window `[Low, High]`: Champion `G4DNAChampionElasticModel.cc:253`, Melton `:199`, Sanche `:207`, Born excitation1 `:157`, Born1 ionisation `:363` (separate windows per particle, e- 11 eV-1 MeV `:126-127`), RPWBA excitation `:158`. Ion elastic differs: window 0-1 MeV, `sigma = DBL_MAX` below `killBelowEnergy = 100 eV` (`G4DNAIonElasticModel.cc:50, 264-267`). Windows are evaluated at the pre-step energy.
- **Effective electron windows, option2** (`G4EmDNABuilder.cc::ConstructDNAElectronPhysics` L309-440 and model `Initialise`): solvation 0-7.4 eV, Champion 7.4 eV-1 MeV, Born excitation 9 eV-1 MeV (`BornExcitationModel1.cc:99-100`, overrides the builder's lower limit), Born1 ionisation 11 eV-1 MeV, Sanche 2-100 eV, Melton 4-13 eV. Born1 runs the cumulated ("fast") tables because option2 sets `SetDNAFast(true)` (`G4EmDNAPhysics_option2.cc:54`, read at `Born1.cc:72`). This closes the "lower limit [U]" of B.2: **9 eV** for excitation.
- **No energy cut.** DNA `SampleSecondaries` implementations ignore the `tmin/maxEnergy` arguments (unnamed parameters), so secondaries are *always* produced, whatever the production cuts. `G4VEmProcess.cc` contains no lowest-kinetic-energy kill, and option2 only sets `SetDNAFast` (grep of `G4EmDNAPhysics_option2.cc`). The only terminations are model-specific:
  - **Solvation** (`G4DNAOneStepThermalizationModel.hpp:137-146, 182-185`; `G4DNATransformElectronModel.cc:86-132`): cross section `DBL_MAX` for `E <= 7.4 eV` (the builder sets the high limit), then `fStopAndKill` with local deposit `= E_kin` and a solvated electron is created by the chemistry manager. Default model: `fMeesungnoen2002eSolvation` (`G4EmLowEParameters.cc:78`). Which positions are produced is M6 (G6).
  - **Melton attachment**: absorbed (`fStopAndKill`, local deposit `= E_kin`, `G4DNAMeltonAttachmentModel.cc:238-247`).
  - **Ion elastic** (proton/H): `E < 100 eV` -> `fStopAndKill`, deposit `E` (`IonElastic.cc:310-313`). **Rudd extended**: `E <= 100 eV` -> `fStopButAlive`, deposit `E` (`Rudd...Extended.cc:296-300`); below `fElow` the table is scaled linearly, `sigma = sigma(fElow) E/fElow` (`:265-266`), a *model-level* extrapolation (M5).
  - Born1 returns without changing the track if `E < binding energy` of the sampled shell (`Born1.cc:472`); Born excitation and Sanche change nothing when `E - E_exc <= 0`. Both are unreachable inside the windows above except through table rounding and are asserted in the M2 energy ledger.
- **Electron energy accounting at ionisation** (`Born1.cc:470-575`): scattered `= E - B - W_kin`, local deposit `= B` (minus any de-excitation secondaries); with `statCode` (stationary) the primary keeps `E` and deposits `E - scattered`.
- **Remaining [U] (do not block M1):** the shell-energy sampling and angle code itself (`SampleDirectionForShell`, G1/M2); solvation product positions (G6/M6); behaviour of non-DNA `G4VEmProcess` machinery for steps that end exactly on a window boundary (M2 test: windows are closed intervals in Geant4).
