# geant4-rs glossary

This project models selected Geant4-DNA particle transport and water-radiolysis chemistry in Rust while preserving the scientific meaning and reference vocabulary of Geant4 11.4.1.

<!-- Format adapted from mattpocock-skills' domain-modeling (GLOSSARY-FORMAT.md): https://github.com/mattpocock/skills.git -->

<!--
How to add a term (the moment it is resolved, not in a batch at the end):

**Term**:
One or two sentences: what it IS, not what it does.
_Avoid_: the other words people use for it

- Be opinionated: one word per concept; list the others under _Avoid_.
- Only terms specific to this project. General programming concepts and
  implementation details don't belong here: this is a glossary, not a spec.
- Group terms under "### <area>" subheadings once natural clusters emerge.
-->

## Language

**Chemistry species name**:
The byte-for-byte `G4MoleculeTable` configuration key used by Geant4 11.4.1, including spelling, case, punctuation, and symbols such as `°OH`, `HO2°`, and `e_aq`. The same name is canonical in data, serialization, and user-visible output, exposed by `Species::chemistry_name()` in Rust. `Species::OH` and `Species::HO2` are legal identifiers at the Rust syntax boundary. The four bookkeeping configurations `H2O(B)`, `H3Op(B)`, `OHm(B)`, and `NoneM` are outside `Species`.
_Avoid_: treating the legacy ASCII input spellings `OH` or `HO2` as canonical keys
