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
| `reactions_option3.toml` | 72 executable option3 reaction definitions, their observed-rate literals, source classes and kinetic intent (`G4EmDNAChemistry_option3.cc:96-666`) |

The `tag` values are the exact Geant4 11.4.1 `G4MoleculeTable` configuration keys, including `°OH` and `HO2°`.
`Species::OH` and `Species::HO2` are legal Rust identifiers for those two keys; `chemistry_name()` returns the
original key. The old ASCII spellings `OH` and `HO2` are accepted only by the explicit legacy parser and are
never canonical data or output. The C++ bookkeeping configurations `H2O(B)`, `H3Op(B)`, `OHm(B)`, and `NoneM`
are outside the 15-member `Species` set and are represented as explicit bulk participants in the reaction table. The extraction fixture retains the exact C++ keys, and
`cpp_crosscheck` compares all 15 without translating aliases.

Each reaction has a stable source-order `id`, source line and class, ordered reactants and products, kinetic
intent, observed-rate order and literal, and whether the constructor calls `SetReactionType`. Bimolecular
literals retain `M^-1 s^-1`: the C++ expression multiplies each by `1e-3 m^3/(mol s)`, which is the same unit.
First- and pseudo-first-order rates retain `s^-1`. Adjacent C++ comments
are stored only as audit evidence: the executable literals are authoritative. In particular, comment/rate
discrepancies at IDs 59, 63, 65, 68 and 70 are preserved without substituting the comment values. Reaction 53
is the sole source-level atom/charge anomaly and is allowlisted only for its exact
`O3m + NoneM -> H3Op + O2m` signature and `2.66e3 s^-1` literal.

`scripts/extract_reactions_from_cpp.py <geant4-source>` extracts the C++ table and can check
`crates/dna-chemistry/tests/fixtures/cpp_reactions.json`; `scripts/hand_reactions.py --check` independently
checks participant composition, conservation summaries and table counts. The Rust loader validates the TOML
against both fixtures. The source checkout used for this transcription was the unpacked Geant4 11.4.1 release,
so provenance is the release plus the exact source path and lines; no repository commit is claimed.

Units in the files are those of the C++ (m^2/s, nm, eV, K, g/cm^3, ps); `dna-chemistry` converts them to the
internal units (nm, ps, eV, docs/units.md) at load time. Data are inputs of the simulation: any change needs a
re-run of the extraction check and a note in docs/08.

`chemistry_data_sha256` concatenates the embedded bytes in this fixed order:
`species_option3.toml`, `dissociation_option3.toml`, `physchem_option3.toml`, then
`reactions_option3.toml`.
