# Session: investigate-irt-syn-rr008-coverage

**Date:** 2026-11-09T09:11:59.334Z
**Status:** Grill phase complete

## Problem Statement

Explain the missing channel 11 +z reactions in RR-008 and specify a new ordinary-scheduler experiment without altering the old evidence or claiming the 233 forced positive-duration endpoints passed.

## Context & Constraints

- **Current behavior:** RR-008 replicas each have 288 complete rows and channel 11 +z 0/32; both are INCONCLUSIVE under their unchanged coverage gate. The analytic marginal campaign exhausted its three looks with 233 positive-duration endpoints INCONCLUSIVE.
- **Pain point:** The two nominal replicas overlap 31 of 32 seed pairs within +z; sequential Ranecu seeds create correlated early draws. They are not independent replications.
- **Dependencies:** Retain all datasets, provenance, ticket history, caps, criteria and verdicts. No existing campaign gets extra samples. No Geant4 generator run is authorized in this task. No existing generator mode changes.
- **Tech stack:** Geant4 11.4.1 installed library and source, C++ standalone chemistry harness, Rust validation consumer; independent deterministic seed derivation using SHA-256.

## Success Metrics

- Document reproducible seed reconstruction, source call chain, and matching of all 192 channel 11 outcomes, including limitations of the inference.
- Freeze a distinct channel 11 ordinary-scheduler request with independent pseudorandom seed derivation, fixed settings, exact simultaneous binomial criteria, coverage, resources, and pass/fail/inconclusive outcomes.
- Keep RR-008 and all 233 endpoints INCONCLUSIVE; no generator run, data replacement, criterion change or ticket completion.

## Architecture & Approach

### Source diagnosis

`SynScheduler.cc` resets Ranecu per row to ((master+row)%2147483562+1, (master+row)/2147483562+1). The two masters differ by one. `G4Molecule::BuildTrack` consumes two uniforms per parent for directions: draws 1–4. `G4DNAIndependentReactionTimeStepper::CalculateStep` encounters the pair once using checked-track bookkeeping. `G4DiffusionControlledReactionModel::GetTimeToEncounter` tests draw 5 against a/r=1/3; channel 11 is type II, so on encounter it tests draw 6 against kact/(kact+kdif)=kobs/kdif, with kobs=1.55e10 M^-1 s^-1, D1+D2=0.0092 nm²/ps, a=0.41 nm and kdif=4*pi*(D1+D2)*a*N_A ≈2.8545e10 M^-1 s^-1, giving threshold ≈0.543. If accepted, further random draws generate the reaction delay and the finite horizon can censor it. Source: `source/externals/clhep/src/RanecuEngine.cc`; `source/processes/electromagnetic/dna/molecules/management/src/G4Molecule.cc`; `models/src/G4DNAIndependentReactionTimeStepper.cc`, `models/src/G4DiffusionControlledReactionModel.cc`, `utils/include/G4IRTUtils.hh` (latter three under `source/processes/electromagnetic/dna/`).

Reconstructed fifth and sixth Ranecu draws from committed seeds, with the exact integer recurrences in `RanecuEngine.cc`; acceptance predicate draw5 < 1/3 and draw6 <= 0.54299927246 matched all 192 channel 11 outcomes in the two CSVs, including all orientations. For +z, fifth draw passed in 11/32 per replica; sixth draw for all trials spans 0.68914266–0.97663372 and 0.67986875–0.96735981, respectively, wholly above activation threshold. Thus the deterministic seed/draw mechanism explains observed missing reactions conditional on the source call sequence, strongly corroborated by all recorded outcomes. The CSV does not record draws or private call order; a logging diagnostic would be needed for direct runtime instrumentation. Seed spacing causes correlations; no unbiased probability or orientation effect follows from the 0/32 counts.

`G4DNAMakeReaction` initializes private fTimeStep to zero; TestReactibility assigns it. The ordinary IRT-syn stepper calls MakeReaction directly without that setter, and the placement function changes mobile parent coordinates only when dt>0. The forced RR-007 source-body diagnostic explicitly calls TestReactibility with a positive dt; its branch and moments are diagnostic, not proof of ordinary scheduling. RR-008's observed parent sites support the zero-duration inference for observed settings but neither directly reads private dt nor proves every scheduler context. New coverage can add evidence for the ordinary fixed pair, not pass the 233 forced-branch endpoints, establish its physical correctness, or retroactively pass RR-008.

### Prospective new scheduler reference (draft contract; no run authorized)

Implement as a separate executable/mode with new output and provenance; retain ordinary installed Geant4 11.4.1 option2/option3 IRT_syn setup, guard actual installed model and use standalone initialization as in RR-008. No source inclusion, replacement, interception or forced TestReactibility. Use only channel 11, both mobile species, a=0.41 nm, D1=0.0022 and D2=0.007 nm²/ps, separation 3a; birth 1 ps, end 1000000 ps, 1 m water sphere. Orientations: +x at (0,0,0), +z at (0,0,0), translated +x at (10a,-7a,3a). Two fixed replicas with 2048 histories per orientation, ordered replica then orientation then history, 12288 rows total, including censored rows; stop each history on first reaction. Fixed seeds are derived from UTF-8 ASCII bytes `g4rs-rr008-followup-v1|R|O|H` with R=1 or 2, O=0,1,2, H=0..2047 in canonical unsigned decimal without leading zeros. Hash SHA-256; let x and y be the first and second 16-byte big-endian integers. Ranecu seeds are (1 + x mod 2147483562, 1 + y mod 2147483398). Before any run, enumerate all 12288 pairs and verify uniqueness and no overlap with the 576 historical RR-008 pairs; reject the contract on collision rather than change seeds after seeing results. Hash-based seed states are reproducible pseudorandom streams, not a mathematical proof of statistical independence. Replicas and orientations have distinct domain inputs, no sequential state increments. Freeze this mapping and code hash before execution.

The two replicas are separate commands with new files, each exactly 6144 ordered data rows and a completion footer. Record full seed pair, channel, orientation, index, outcome, fixed inputs, event time and step, initial/reacting positions, product count and parent-removal flag; NA for censored event fields and channel 11 centroid. Validate version, seeds, model type, order, row count, complete footer, finite parameters, no duplicate streams, hashes, process exit and provenance. Capture source and binary hashes, dataset versions, build/platform, exact commands, dirty state and data hash as in RR-008. Prepare outputs in new scratch paths; preserve existing references; incomplete outputs are not evidence.

Structural acceptance for each actual reaction: positive selected step, event time >1 ps and <=1000000 ps; initial and reacting parent positions unchanged to 1e-12 relative plus 1e-12 nm absolute, zero products and both parents removed in post-step callback. Require at least one positive event in each of six replica/orientation settings. Invalid complete data or a verified structural violation is FAIL; incomplete execution/schema or missing setting coverage is INCONCLUSIVE, not a physics failure.

Prospective statistical diagnostic: for each of six settings, construct the exact two-sided Clopper–Pearson interval for the reaction fraction at simultaneous family alpha 0.001 (per interval alpha=0.001/6). One fixed look, n=2048; declare the frequency check PASS when every interval lies strictly inside [0.10,0.25], FAIL if any interval lies wholly outside that band, otherwise INCONCLUSIVE. The equivalent PASS count is 259 through 438 inclusive for each setting. This deliberately broad operational band covers the source-implied eventual probability approximately (1/3)*0.543=0.181; the finite horizon can only lower it. Before running, independently calculate or rigorously bound the finite-horizon source-model probability inside the band for the specified inputs; otherwise this frequency gate is not scientifically justified and must not run. It is a coarse diagnostic, not a distributional or physical-validation gate. Overall new experiment PASS requires all structural, coverage and frequency checks PASS; a verified violation yields FAIL; otherwise INCONCLUSIVE. Correct simultaneous bounds for six settings regardless of their outcomes. No sequential looks, rerolls, extension or adaptive changes.

Resource limits: separate dry validation of manifest and seed enumeration without running Geant4; separately request one scratch smoke with one history per orientation per replica followed by the two fixed production invocations only if smoke has complete schema, footer and exit 0. Smoke carries no coverage/statistical inference. Each invocation <=2 hours and <=8 GiB, total execution <=24 hours, each output <=10 MiB; abort and mark incomplete on breach, retaining logs. No automatic retry or new samples. Output/provenance publication requires complete validation and explicit user authorization for runs. Compile-only work may proceed later under project policy.

### Implementation steps

1. Preserve a checked, independently reviewable reconstruction script and results, verifying both committed CSV hashes/provenance and every row outcome; never run Geant4.
2. Derive and verify the finite-horizon probability by independent quadrature or bound; freeze numerical assumptions and precision before the experiment. Check seed uniqueness and the exact binomial thresholds with independent calculations and unit tests.
3. Create a new scheduler producer, strict consumer and new request/provenance schema; compile-check only, run normal workspace checks, document source hashes and bounded commands. Do not edit an existing producer mode or reference file. If feasibility or contract fails, stop INCONCLUSIVE.
4. Only after the request is fully reviewable and the user explicitly requests that exact new RR identifier, perform authorized runs per project policy; otherwise leave registered request unexecuted. A later successful experiment is distinct evidence and cannot change old verdicts or unlock the prior ticket without a separate explicit governance decision.

## Assumptions & Trade-offs

Choose a targeted ordinary channel 11 study with all three prior orientations; extra direct-branch sampling would be a separate scientific campaign and cannot repair the exhausted 233 endpoints. The frequency band is broad and intentionally tests a gross rate inconsistency rather than full reaction-time distribution. The count criterion assumes the hash-derived streams behave as independent random samples; this standard PRNG assumption is not proved by uniqueness. Activation threshold used for reconstruction is rounded; all tested +z values are far above it. Only empirical 192-row agreement corroborates call order, not instrumentation. Finite-horizon verification remains a prerequisite.

## Open Questions

No user scope decision required now. Exact finite-horizon probability, independent seed collision audit, source hash check, implementation and compile-check remain to be completed before a concrete registered run exists. No reference run is authorized by this design.

## Notes

Original source tree is read-only. Normative source v11.4.1 was inspected at its DEV_DIR-derived path and the installed runtime provenance says 11.4.1; current G4_ROOT points to 11.4.3, so do not use it as the 11.4.1 runtime. Reference output and existing worktree changes remain untouched. New experiment cannot establish global IRT-syn correctness.
