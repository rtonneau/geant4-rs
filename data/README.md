# Geant4-DNA data subset

Cross-section tables from the Geant4 dataset **G4EMLOW 8.8** (`dna/` subfolder), copied unmodified.
The list of files, their sizes and SHA-256 checksums is in `MANIFEST.toml`; the models that read them
are listed in `docs/10-data-requirements.md`.

Import (maintainer, local machine with the full dataset):
`python scripts/import_data.py <path-to>/G4EMLOW8.8`

Files must not be edited or have line endings converted (`.gitattributes` enforces this).

Licence and attribution: these data are distributed with Geant4 under the Geant4 Software License
(see `LICENSE` in https://github.com/rtonneau/geant4 and https://geant4.web.cern.ch/download/license).
Cite the Geant4-DNA publications listed in the Geant4 example READMEs when using results.
