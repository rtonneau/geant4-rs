# G1-F: Geant4 source-fidelity gate

Approved prospectively by the project lead on 2026-11-05 after the R04/G1
grill. This is a documentation decision, not an executed validation result.
**G1-F is OPEN; reference evidence is REQUESTED.**

## Decision and historical evidence

G1-F replaces G1 as the prerequisite for integrating real chemical models
into R05. It establishes agreement with Geant4 11.4.1 on the declared pair
domain, not independent physical validity or history-level G(t) accuracy.
The original combined G1 contract is superseded, not passed. Its frozen
[matrix](pair-case-matrix.md), evidence and verdicts remain unchanged.

The [2026-10-29 report](r04-independent-2026-10-29.md) records ANALYTIC PASS,
physical agreement FAIL at exact charged absorbing contact, general Brownian
completion INCONCLUSIVE and GEANT4 REQUESTED. These limitations remain
visible. Brownian completion is no longer a prerequisite for G1-F; no physical
failure is reclassified as success. V9, V10 and RR-003 retain their own scope.

## Prospective comparison domain

Retain the eight literal option3 channels, source parameters, units, distance
ordinals and six endpoints per distance in the original matrix. Exclude only
exact PDC contact (`r0/a = 1`) from the statistical Geant4 comparison: IDs 11,
17, 42 and 51. Retain their distances 1.5 and 3 and all three TDC distances.
This gives 120 included endpoints and 24 explicitly excluded endpoints.
Synthetic cases are not Geant4 reference rows.

The exclusion addresses the invalid-return issue recorded in
[RR-004](../../validation/geant4/RUN_REQUESTS.md). A separate deterministic
regression must record the invalid-contact behavior and the Rust handling,
with source version and branch evidence. It must not fabricate a valid
reaction time, shift contact, or substitute a limiting-law sampler. The
regression is required before G1-F can pass; excluded rows are never counted
as statistical passes. If source inspection reveals a different behavior
for a listed regime, resolve the contract explicitly before generating data.

## Acceptance requirements

- Retain the 216-endpoint analytic source-law regression and biased-sampler
  rejection. Its PASS does not imply physical agreement.
- For the 120 comparable endpoints retain family alpha 0.001 and the original
  conservative 144-endpoint Bonferroni threshold, `zcrit = 4.495384196564731`.
  Do not relax the threshold because exclusions reduce the family size.
- At every included endpoint require `abs(z) <= zcrit` and
  `abs(difference) + zcrit * SE < 0.01`. Retain independent binomial variance
  estimates and the existing half-count stabilization. Nonsignificance alone
  is insufficient; deterministic analytic endpoints retain exact checks.
- Retain independent Rust seed 20261027403 and Geant4 seed 20261027404,
  stable case/distance identities and the precision-only sample schedule
  100k, 200k, 400k, 800k, 1m per included case/distance. Do not expand samples
  in response to significance. Underpowered evidence is INCONCLUSIVE;
  resolved disagreement is FAIL; missing or invalid evidence is REQUESTED.
- Require complete, schema-checked reference data, provenance, checksums and
  a successful strict consumer. No partial run is reference evidence.
- G1-F passes only when the analytic regression, invalid-contact regression
  and all included Geant4 statistical comparisons pass. Record each verdict
  separately and identify the exclusions in every gate summary.

## Implementation and external-run boundary

The existing RR-004 mode, consumer and reference contract remain unchanged.
The additive `pair_matrix_irt_g1f` mode and `geant4_pair_g1f` consumer implement
this domain under schema `g1f_pair_v1`. [RR-005](../../validation/geant4/RUN_REQUESTS.md)
registers the user-run command; README specifies the new CSV/provenance files.
The producer directly verifies `SamplePDC(1,0) == -1` before resetting its RNG
for statistical sampling. This isolates the zero-gap proposal failure, not
all charged caller branches. The Rust regression pins the existing continuum
limit as an intentional difference, not a Geant4 return. The producer was
compile-checked against 11.4.1; no new reference evidence has been generated.

Agents may implement and compile-check the producer; Geant4 runs remain
user-performed and logged in RUN_REQUESTS. Do not overwrite historical
reference data or modify the Geant4 reference repository.

## R05 boundary

Before G1-F passes, R05 may implement time management, molecule/event
identities and competition between events using deterministic synthetic
reactions. Tests should cover monotone time, identity validity, invalidation
of competing stale events and deterministic ordering. This establishes
scheduler infrastructure only, not chemistry or G(t) validation.

Integration of real chemical models remains blocked until G1-F passes.
Passing G1-F permits that integration; it does not pass R05/G2 or subsequent
history/count/yield gates. This documentation change implements no scheduler.
