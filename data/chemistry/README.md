# Physicochemical-stage data (Geant4 11.4.1, chemistry constructor `G4EmDNAChemistry_option3`)

Hand-transcribed from the C++ reference (`rtonneau/geant4`, commit `27ea526b420dd72f510af81f6828edef0ff635f5`)
and checked against a machine extraction of the same text
(`scripts/extract_physchem_from_cpp.py` -> `crates/dna-chemistry/tests/fixtures/cpp_extract.json`).
`G4EmDNAChemistry_option3` takes its molecules and dissociation channels from
`G4ChemDissociationChannels_option1` (`physics_lists/constructors/electromagnetic/src/`, called `CH` below).

| File | Content |
|---|---|
| `species_option3.toml` | diffusion coefficient, charge, van der Waals radius of the 15 species (`CH:54-190`) |
| `dissociation_option3.toml` | decay channels of the excited / ionised / attached water molecule and of the recombined `H2Ovib` (`CH:194-466`) |
| `physchem_option3.toml` | birth time of the species, auto-ionisation electron energy, electron-hole recombination (Onsager) parameters |

Units in the files are those of the C++ (m^2/s, nm, eV, K, g/cm^3, ps); `dna-chemistry` converts them to the
internal units (nm, ps, eV, docs/units.md) at load time. Data are inputs of the simulation: any change needs a
re-run of the extraction check and a note in docs/08.
