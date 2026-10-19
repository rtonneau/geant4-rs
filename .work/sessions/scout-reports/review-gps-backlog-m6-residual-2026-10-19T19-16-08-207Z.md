# gps backlog: M6 residual after RR-001

Seeds for `/gps start <slug>`, registered with `scout-merge.js --from docs/gps-backlog-m6-residual.md`.
Origin: RR-001 (2026-10-17) closed with the stage-matched gate **failed** on 16 of 28 rows, all in the
z <= 4.5 test, none in the 15% count or 20% RMS tests (docs/08 "RR-001 result"). Evidence:
`validation/reference/physchem_prechem.csv`, `provenance_m6_prechem.toml`, test
`m6_prechem_counts_and_rms_match_geant4`. Standing rules of `docs/gps-backlog.md` apply (no loosened
tolerances, no invented constants, reference files never overwritten, Geant4 data runs follow
`validation/geant4/RUN_REQUESTS.md`).

## Observed residual

Rust vs Geant4 mean per event. H3O+ and e_aq are high (+1.8% at 1 keV, +0.8 to +1.1% at 10 and 100 keV);
H is low at every energy (-2 to -4%); H2 and O are low (up to -5%); OH is +0.5% at 1 keV and passes
elsewhere. The 1 keV H3O+ and e_aq rows have z of about 21. The signs suggest a shift in the ionisation/excitation partition or in the
dissociation channel fractions, not a scoring error: pre-IRT H3O+ equals the ionisation-derived yield minus
recombination. This is a hypothesis to test, not a finding.

**Update 2026-10-17 (audit seed done).** The hypothesis was partly right: the offset came from the
electron-hole recombination schedule (docs/08 "M6 residual audit (final record)"). The paragraph above
describes the engine before the fix; after it the gate `m6_prechem_counts_and_rms_match_geant4` passes
with unchanged criteria.

---

## m6-residual-rust-audit

**Goal.** Find which stage creates the 1-4% offset using only data that exist. (1) Derive, per energy,
the conservation relations that link the seven species counts (each ionisation gives H3O+ + OH + e;
excitation channels and the dissociative-attachment channel give fixed products) and test them on both
the Geant4 CSV and the Rust run; the relation that breaks first names the stage. (2) Re-read the Rust
ionisation/excitation channel tables, dissociation fractions, recombination and thermalisation against
`G4DNAWaterDissociationDisplacer`, `G4DNAWaterExcitationStructure`, `G4DNAElectronHoleRecombination` and
`G4EmDNAChemistry_option3.cc` (Geant4 11.4.1, `.cc` files). (3) Check that the Rust z values are not a
seed artefact by rerunning with at least two other master seeds and reporting the spread. A model is
changed only if the source comparison proves a defect, with its own tests (see Status).

**Status: done (2026-10-17, session `2026-10-17__m6-residual-rust-audit`).**
The session did change one model, because the source comparison proved a defect:
the electron-hole recombination schedule in `PhysChemEngine::convert_history` (commit `5df0aa1`), with
re-baselined `dna-chemistry` history tests. No tolerance, reference file or generator mode changed. The
relations (hold exactly on both sides, so they could not name a stage), the five-seed spread, the source
table, the verdict, the prototype table and the limits are in docs/08 "M6 residual audit (final record)".

**Requires.** Nothing.

**Deliverable.** A short table in docs/08 naming the stage(s) responsible, or stating that existing data
cannot decide and `m6-process-counts-geant4-request` is needed. Delivered: the stage is named, so no
further Geant4 data were needed.

## m6-process-counts-geant4-request

**Goal.** Add a new Geant4 generator mode `physchem_counts` (new name, existing modes untouched) that
scores per primary energy, with the same physics and chemistry as `physchem_prechem`: number of
ionisations, each excitation level, dissociative attachment, vibrational and elastic steps, solvation
and recombination events, and the dissociation channel chosen for each excitation and ionisation. Same
energies and seed convention as RR-001. Register RR-002 in `RUN_REQUESTS.md`, update README and
BUILD_NOTES, and write the Rust comparison as an `#[ignore]`d test with statistical families and
tolerances fixed in its header before any data exist. Compile-check only; never run.

**Requires.** `m6-residual-rust-audit` (to decide the counters and channels that matter; skip the seed if
the audit already identifies the stage from existing data).

**Status: skipped (2026-10-17).** No longer needed: the audit identified the stage (electron-hole
recombination scheduling) from existing data and the stage-matched gate now passes, so no new Geant4
counter run (RR-002) is requested. Revive it only if the project lead wants per-process counts for a
different reason; nothing else in the backlog depends on it.

## m6-residual-close-gate

**Goal (reworded 2026-10-17).** No new Geant4 run is required. What remains is the project-lead review of
docs/08 "M6 residual audit (final record)" and the decision on M6 status: whether the pass of
`m6_prechem_counts_and_rms_match_geant4` (28/28 rows, criteria unchanged, one Geant4 seed) is accepted
as closing the M6 gate, and therefore whether the M6-closed requirement of M7/M8 is lifted. The record
lists the limits the reviewer weighs (one Geant4 seed, no per-event covariance, assumed Rust/Geant4
tracking order, N1 clip not audited, 900 keV RMS radius +12 to +13 %). The agent records the decision in
docs/08 and docs/09 (Q21/R14) once it is given; it does not take it. Do not loosen any tolerance. A
further change to the Rust model, if the review finds a defect, is a separate seed with its own tests.

**Requires.** `m6-residual-rust-audit`. The decision itself belongs to the project lead.

## m6-900kev-spatial-rms-investigation

**Goal.** Determine whether the common +12.1% to +13.5% Rust excess in the 900 keV
pre-IRT survivor RMS radius is a statistical fluctuation, a broader physical track, a
solvation or dissociation displacement difference, or spatially biased recombination
selection. The current RR-001 aggregate has only 20 Geant4 histories and no per-history
spatial moments, so it cannot provide an uncertainty for the RMS estimator or localise the
stage where the difference appears.

**Tasks.**
1. Complete the N2 source audit from terminal electron through `e_aq` creation, checking the
   position handoff, time and survivor status, and the relationship between transport track
   order and molecule creation order in Rust and Geant4 11.4.1. Change no model unless the
   source comparison proves a defect.
2. Add a new, separately named Geant4 generator mode; do not modify `physchem_prechem` or any
   committed reference. At 900 keV it records per-history count and `sum_r2_nm2`, maximum
   radius, preregistered logarithmic radial bins, and longitudinal/transverse moments at the
   physical-interaction, solvation, pre-decay water-state, post-dissociation and post-
   recombination survivor boundaries. Keep the existing option2/option3 configuration and
   stage semantics.
3. Add the equivalent Rust diagnostic and a paired ignored consumer. Preregister the
   statistical analysis and criteria before any new Geant4 data exist: compare the pooled
   RMS with a bootstrap or batch confidence interval, the per-history RMS distribution,
   radial quantiles/bins, maxima, and longitudinal versus transverse extent. Use an initial
   design of five independent seeds and 40 histories per seed; a pilot may increase the
   count based only on estimator precision, never to force agreement.
4. Compile-check the generator, document its schema and build/run recipe, and append a new
   `RUN_REQUESTS.md` entry. The agent never runs the Geant4 executable. The project lead runs
   it and commits new data and provenance; a later close-gate seed consumes the result.

**Acceptance.** The committed diagnostic and preregistered comparison can distinguish where
the spatial scale first diverges and whether the observed 900 keV excess exceeds the
measured sampling variation. The result either identifies a source-proven defect for a
separate fix seed or records a quantified residual with confidence intervals and explicit
limits. Existing tolerances and reference files remain unchanged; this investigation does
not silently reopen or close M6.

**Requires.** `m6-residual-rust-audit`. It is independent of the project lead's formal
`m6-close-gate` decision and may run after closure as a residual-risk investigation.
