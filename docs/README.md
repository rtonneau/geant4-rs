# geant4-rs planning package

Status: planning package, kept up to date as milestones land (M0 foundations, M1 data layer and M2 electron
transport are implemented, see 08-roadmap.md). Written against the
Geant4 C++ tree in `rtonneau/geant4` (CMake version 11.4.1; the tree is a single squashed
commit `27ea526b420dd72f510af81f6828edef0ff635f5`, so "v11.4.1" is established from
`CMakeLists.txt` lines 32-35, not from a git tag).

| Doc | Deliverable |
|---|---|
| [01-scientific-requirements.md](01-scientific-requirements.md) | A. Scope, observables, assumptions, units, G-value conventions |
| [02-cpp-reference-inventory.md](02-cpp-reference-inventory.md) | B. Geant4 source/data inventory and Rust mapping, audit gaps |
| [03-architecture-comparison.md](03-architecture-comparison.md) | C. Three architectures, recommendation |
| [04-scientific-algorithms.md](04-scientific-algorithms.md) | D. Physical, physicochemical, chemical algorithms |
| [05-parallelisation.md](05-parallelisation.md) | E. Concurrency, RNG, reductions |
| [06-dependencies.md](06-dependencies.md) | F. Dependency plan |
| [07-validation-strategy.md](07-validation-strategy.md) | G. Validation, acceptance criteria, CI |
| [08-roadmap.md](08-roadmap.md) | H. Milestones M0-M11 |
| [09-open-questions-and-risks.md](09-open-questions-and-risks.md) | I. Decisions pending, ranked risks |
| [10-data-requirements.md](10-data-requirements.md) | Which Geant4 data files are needed, where they live |

## What was and was not verified
Everything in these documents tagged **[V]** was read directly in the C++ source in this
session (path given). Items tagged **[U]** are *unverified* and are explicit audit tasks in
milestone M0/M1. No Geant4 build was run, the `G4EMLOW` dataset is **not** present in the
reference repository, and no numerical cross-section value has been inspected.
