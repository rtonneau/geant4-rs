# Preserve executable Geant4 reaction literals

_ADR 0002, recorded 2026-10-17 (session 2026-10-17__m7-option3-reaction-table)._

<!-- Format adapted from mattpocock-skills' domain-modeling (ADR-FORMAT.md): https://github.com/mattpocock/skills.git -->

The option3 reaction table preserves the executable literals in Geant4 11.4.1 even where nearby comments disagree or a source entry is chemically anomalous. Extraction records those discrepancies, and validation uses explicit, documented exceptions rather than silently correcting the reference, because exact behavioral provenance is more reproducible than choosing an unverified replacement value.
