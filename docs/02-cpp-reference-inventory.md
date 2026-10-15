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
Extrapolation/boundary behaviour of `G4DNACrossSectionDataSet` [U].

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
G2 `G4DNACrossSectionDataSet` interpolation/extrapolation, file units/scale factors -> M1.
G3 Proton charge-increase limits; standard hadron physics handover -> M5.
G4 Exact `chem6` scoring/energy normalisation -> M8.
G5 SBS time-step computer and `G4Scheduler` -> M7 (oracle only).
G6 Default solvation model (`G4DNASolvationModelFactory::GetMacroDefinedModel`) and `G4DNATransformElectronModel` (0-0.025 eV) -> M6.
G7 Temperature and `G4IRTUtils::GetRCutOff`, time-step parameters (`G4EmParameters` chemistry section ~L1570-1630) -> M7.
G8 Electron production cut/tracking cut behaviour for DNA models (`SetLowEnergyLimit` kill below limit?) -> M2.
