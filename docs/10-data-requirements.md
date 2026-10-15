# Data requirements (G4EMLOW 8.8, `dna/` subfolder only)

**Policy (decision Q7, revised):** the Geant4 data stay **outside the repository** (full `G4EMLOW8.8` is ~2.1 GB / 43k files; only `dna/` is relevant).
- Location is given by env var `DNA_DATA_DIR` (pointing at the `G4EMLOW8.8` folder, i.e. the directory that contains `dna/`) or the TOML key `data_dir`. Local example: `C:\DEV\GEANT4\Geant4-ressources-data\G4EMLOW8.8`.
- Tests needing the data are `#[ignore]`d/skipped with an explicit message when `DNA_DATA_DIR` is unset. Cloud sessions and CI therefore run only data-free tests plus small committed fixtures.
- The repo stores `data/MANIFEST.toml`: file names, sizes, SHA-256, consuming model. Generated locally by `scripts/make_data_manifest.py`; at run time the loader verifies checksums when `verify_data = true`.
- Output metadata always records dataset name/version and the manifest hash.
- Licence: Geant4 Software License attribution applies if any derived fixture is committed (check before doing so).

File names below are from the C++ loaders (`source/processes/electromagnetic/dna/models/src`). `.dat` is appended by the loader where the name has no extension. Existence in your folder is checked by the script, not assumed.

## Group `electron_option2` (milestones M2-M3)
- `dna/sigma_elastic_e_champion`, `dna/sigmadiff_cumulated_elastic_e_champion_hp.dat`
- `dna/sigma_excitation_e_born`
- `dna/sigma_ionisation_e_born`, `dna/sigmadiff_cumulated_ionisation_e_born_hp.dat` (fast path), `dna/sigmadiff_ionisation_e_born.dat` (slow path, for the Q4 equivalence check)
- `dna/sigma_excitationvib_e_sanche.dat`
- `dna/sigma_attachment_e_melton`
## Group `proton_option2` (M5)
- `dna/sigma_excitation_p_born`, `dna/sigma_ionisation_p_born`, `dna/sigmadiff_cumulated_ionisation_p_born_hp.dat`, `dna/sigmadiff_ionisation_p_born.dat`
- `dna/sigma_ionisation_p_rudd`, `dna/sigma_ionisation_h_rudd`
- `dna/sigma_elastic_proton_HTran`, `dna/sigmadiff_cumulated_elastic_proton_HTran.dat`
- `dna/sigma_excitation_p_RPWBA`, `dna/sigma_ionisation_p_RPWBA`, `dna/sigmadiff_cumulated_ionisation_p_RPWBA.dat`, `dna/sigmadiff_ionisation_p_RPWBA.dat`
- Miller-Green excitation and Dingfelder charge change use formulas/parameters in code: **no files** (confirm in M5 audit, [U]).
## Group `electron_option4` / `electron_option6` (optional, M4)
- Emfietzoglou: `dna/sigma_excitation_e_emfietzoglou`, `dna/sigma_ionisation_e_emfietzoglou`, `dna/sigmadiff_{,cumulated_}ionisation_e_emfietzoglou.dat`
- CPA100 (water only): `dna/sigma_elastic_e_cpa100`, `dna/sigmadiff_cumulated_elastic_e_cpa100`, `dna/sigma_excitation_e_cpa100`, `dna/sigma_ionisation_e_cpa100_form_rel`, `dna/sigmadiff_{,cumulated_}ionisation_e_cpa100_rel`. DNA-base variants (guanine etc.) are not needed.
- Uehara elastic (option4) uses no table file ([U] confirm).
## Not needed
Other `G4EMLOW8.8` subfolders, `G4PARTICLEXS`, `G4NDL`, etc. Ion tables beyond p/H Rudd (He, Li ... Fe) are out of scope.
## Format knowledge
Loaders are `G4DNACrossSectionDataSet`/`G4DNAEmfietzoglou...::LoadData` (total cross section: energy/value columns with unit and scale factor) and per-model readers for differential tables. Format will be read from the C++ loaders in M1; a few `head -20` samples pasted by you into an issue/chat are enough to confirm, no data upload needed.
