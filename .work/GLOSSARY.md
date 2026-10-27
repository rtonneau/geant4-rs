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

**Reaction participant**:
A named entity occupying a reactant or product position in the Geant4 option3 reaction table. It is either one of the 15 mobile chemistry species or a bulk reaction participant.
_Avoid_: assuming every reaction participant is a mobile species

**Bulk reaction participant**:
One of the non-mobile Geant4 bookkeeping configurations `H2O(B)`, `H3Op(B)`, `OHm(B)`, or `NoneM`, representing a bulk reservoir or sentinel in first- and pseudo-first-order reactions. It is not a member of the mobile `Species` set.
_Avoid_: reservoir species, ordinary species

**Observed reaction rate**:
The `k_obs` literal encoded by `G4EmDNAChemistry_option3` for a reaction, expressed according to that reaction's order. It is source data, unlike the activation-controlled rate `k_act` derived later by a selected reaction model.
_Avoid_: treating `k_act` as a source-table constant

**Isolated two-body system**:
A chemistry system containing exactly one candidate reactant pair, either A+B or A+A, whose diffusion and possible reaction are studied without competition from any other molecule or reaction channel.
_Avoid_: many-species chemistry system, reaction scheduler

**Independent reaction-time outcome**:
The result of sampling one isolated reactant pair: an immediate reaction, a reaction at a finite positive time, or no eventual reaction. Numerical sampling failure is an error rather than a physical no-reaction outcome.
_Avoid_: encoding no reaction or numerical failure as a negative time

**Step-by-step (SBS) oracle**:
A validation-only Rust implementation of the Geant4 11.4.1 adaptive molecular encounter stepper and Smoluchowski reaction model for an isolated totally diffusion-controlled pair. It is independently implemented from the C++ source and is not the production chemistry scheduler.
_Avoid_: fixed-step Brownian oracle, production SBS engine

**Independent Brownian pair oracle**:
A validation-only simulation of the relative motion and reaction of an isolated molecular pair using Brownian steps and the interaction and reaction boundary appropriate to its kinetic regime. It is distinct from the Geant4-faithful SBS oracle.
_Avoid_: calling the source-faithful SBS oracle an independent Brownian pair oracle

**Molecule identity**:
A stable identifier for one individual mobile molecule within a chemistry history. Two molecules may have the same species but must have different identities; a molecule cannot form a reaction pair with itself.
_Avoid_: using species equality as molecule identity
