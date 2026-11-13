# Ticket 01: reference-version-bump

**Status:** ✅ Done

## Local Test Result

`git diff --name-only HEAD -- validation/reference; grep -n "11.4.3" AGENTS.md`: no reference paths listed; AGENTS.md:22 and :114 name v11.4.3.

## Review Notes

Base 20a5c8f5391524eca5fbeecb48ee1ea8a96ab99f. Pre-existing uncommitted RR-008 work (other session) is not part of this ticket and is not staged.
- AGENTS.md reference lines bumped. docs/02 has a header stating 11.4.3 and that the dated 11.4.1 audits stay valid. docs/reproducibility.md notes that the code constant is still 11.4.1.
- docs/geant4-11.4.3-migration.md: `diff -rq` of the release sources. `electromagnetic/dna` and the dataset definitions are byte-identical. The other differences (field configuration in transport, atomic flag, ion effective charge, standard-EM models) are out of scope. `G4ionEffectiveCharge` is referenced nowhere under dna.
- Deviation from plan: validation/geant4/README.md was not edited. It carries uncommitted RR-008 hunks, and the RR flag lives in AGENTS.md and the migration note instead.
- Code follow-up recorded rather than made: `GEANT4_REFERENCE_VERSION` (g4-core provenance.rs:12) and the reactions.rs:199 release check still say 11.4.1 (documents-only scope).

## Blockers / Challenges

None.

## Commits

- 438dad0 docs: bump Geant4 reference to 11.4.3 and record source diff (ticket 01)

## Time Spent

1m (ticket-start.js to ticket-complete.js)
