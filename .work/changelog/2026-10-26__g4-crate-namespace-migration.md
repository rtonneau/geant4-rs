---
bump: minor
floor: patch
reason: Deliberate source breaking package and import renames in the pre-release workspace
---
- Renamed the five Rust packages and import roots from `dna-*` to `g4-*`. Source consumers must update dependency and import paths; JSON schemas, CSV output and scientific data paths stay unchanged.
