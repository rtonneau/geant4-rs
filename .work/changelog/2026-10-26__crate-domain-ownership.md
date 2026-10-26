---
bump: minor
floor: patch
reason: The public species import path moved from dna_core to dna_chemistry.
---
- Chemical species types are now exported from `dna_chemistry`; imports from `dna_core::species` must move to the chemistry crate.
- Added exact physical and physicochemical snapshots for checking future crate changes against the current scientific output.
