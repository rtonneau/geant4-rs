# GPS roadmap audit and proposed migration

Date: 2026-10-26. Baseline: `1bdcb2d391b5ac88b8802ac360ef1ac2bc9934b6`. **Proposal only: no seed, session, pointer, registry, roadmap or run request was changed.** Implementation findings are in [development-state.md](development-state.md); replacement contracts are in [proposed-roadmap.md](proposed-roadmap.md).

## Executive decision

Do not execute the selected `m6-process-counts-geant4-request` session. Its motivating workflow has been overtaken by the source-proven recombination correction and explicit M6 closure. It also has only an unfilled grill template, despite its “Grill phase complete” label. Starting tickets from that state would substitute an obsolete title for a current scientific problem.

Replace the remaining backlog with a short dependency-driven sequence: trustworthy gates and inputs; complete option3 execution; independent pair validation; a deterministic per-history scheduler; G(t), CLI and integrated validation. Preserve original proton, track-segment, physical-assessment and extension requirements explicitly. Do not manufacture a new session for every finding or every reference-file import.

## 1. Exhaustive inventory and authority

The inventory reconciles three sources, not just GPS's pending list:

- Eight entries in local `.work/sessions/.pending-seeds.json` (ignored local state).
- The existing `.work/sessions/2026-10-26__m6-process-counts-geant4-request/01-grill/resume.md` and active-session pointer. Starting the seed removed it from the pending list but did not complete its objective.
- `m7-sbs-oracle-and-ode`, described in `docs/gps-backlog.md` and required by another seed, but absent from the pending registry and completed sessions.

This gives **ten distinct unfinished objectives**, not eight. Sources also include `docs/gps-backlog-m6-residual.md`, scout reports under `.work/sessions/scout-reports/`, session indexes/tickets, `docs/08-roadmap.md`, `docs/09-open-questions-and-risks.md`, and `validation/geant4/RUN_REQUESTS.md`. No remote-only issue inventory or unpublished plan was invented.

Disposition counts: **KEEP 0, REWRITE 2, MERGE 4, SPLIT 2, DELETE 2, DEFER 0** for these ten existing unfinished objectives. No unchanged seed is sufficiently precise and correctly ordered to warrant KEEP. Deferred original requirements outside this ten-item inventory are separately retained below. DELETE means remove from executable backlog after approval, not destroy historical evidence or reference data.

## 2. Every unfinished seed: disposition and traceability

| Existing objective | State / source | Decision | Proposed replacement(s) | Rationale and preserved obligation |
|---|---|---|---|---|
| `m6-process-counts-geant4-request` | Selected session, unfilled grill; residual backlog | **DELETE** | R01 records retirement; no new process-count campaign | Source-proven fix `5df0aa1`, residual audit and explicit closure supersede its old purpose. Process/model counts remain supported diagnostic observables, not a mandatory new Geant4 request without a fresh hypothesis. Do not reuse RR-002, which now denotes spatial work. |
| `m6-residual-close-gate` | Pending registry; requires the process-count seed | **DELETE** | R01 current-evidence ledger | Duplicate/obsolete closure after `635e91b` / `99dd76b`. Preserve M6's qualified approval and residual uncertainty. Do not claim spatial equivalence or erase historical failures. |
| `m7-sbs-oracle-and-ode` | Documentation-only orphan in main backlog | **SPLIT** | R04 pair/SBS/Geant4 validation; R05 network invariants; D01 deferred original closed-volume V10 | Independent pair oracle is needed now; closed well-mixed kinetics is not an isolated-pair property. Preserve V10 explicitly rather than passing a different SSA test under its name. |
| `m7-sbs-close-gate` | Pending; requires missing oracle seed | **MERGE** | R04 lifecycle and evidence verdict | Same pair-validation objective. One session can build the oracle, request external data and resume to consume it. “Failure documented” is completed diagnosis, not a passed gate. |
| `m7-multispecies-irt-scheduler` | Pending | **SPLIT** | R03 complete execution kernels; R05 history scheduler; R06 shared reference producer | Missing charged-PDC and unary/background execution must precede heap assembly. Keep spatial correlations, exact species identity and M6 boundary. Avoid duplicate C++ count/G(t) campaigns. |
| `m7-scheduler-close-gate` | Pending | **MERGE** | R07 integrated acceptance, separate scheduler verdict | Reuse the G(t) campaign, but retain raw count/time and scheduler-specific checks so scoring cannot mask scheduler failures. Do not close full historical M7 while V10 remains deferred. |
| `m8-gt-scoring-and-chem6-reference` | Pending | **REWRITE** | R06 full-slowing G(t)/reference; R09 track-segment extension | Freeze stage, constructor, 1 MeV endpoint, two estimators and batch/covariance semantics here rather than postponing another architectural grill. Preserve all seven V11 times and original A.4 track-segment obligation. |
| `m8-gt-close-gate` | Pending | **MERGE** | R07 integrated acceptance | One reference acquisition serves count and yield gates; independent verdicts remain. Preserve 100 keV and 1 MeV, pointwise/family/effect-size criteria, and proton 10 MeV through R11. |
| `m8-config-and-cli` | Pending | **REWRITE** | R08 versioned CLI and run schema | Still needed; add validated deserialization, verified data identity, installed/out-of-tree execution, explicit capabilities, no-output-on-error and compatibility semantics. No generic plugin/config framework. |
| `m8-reproducibility-v13` | Pending, placed last | **MERGE** | R05, R06, R08 implementation invariants; R07 final campaign | RNG ownership, tie ordering and reductions cannot be retrofitted safely. Preserve bitwise 1/2/8-thread and batch-size tests, observation-grid invariance, and independent-seed compatibility. |

Each replacement has a full objective, scope/exclusions, dependencies, deliverables, acceptance and priority/complexity in the proposed-roadmap report. R01/R02 are genuinely new prerequisite work; R03 exposes missing work hidden in the scheduler seed rather than redoing the finished pair-kernel session.

## 3. Critical review of the next selected GPS session

### What it was trying to accomplish

The residual backlog originally sought process-count evidence to distinguish transport/channel causes of remaining M6 yield differences, then a separate closure. That was reasonable before the recombination-order correction. The later history matters: `5df0aa1` corrected zero-time hole retry semantics; `4d3816f` recorded passing stage-matched evidence and did not require the process-count request; M6 received explicit approval in the 2026-10-19 closure session. RR-002 was subsequently used for a different spatial question, whose local conclusion is committed at `1bdcb2d`.

### Why it should not proceed as written

1. **No current falsifiable hypothesis.** The old discrepancy is not a new demonstrated defect after the correction. A new reference run needs a named unresolved observable and a decision it can change.
2. **Inconsistent dependency graph.** The residual-close seed assumes this work is still necessary to close M6, contrary to the explicit closure.
3. **Unusable starting specification.** The active resume contains `gps:fill` placeholders for problem, constraints, success metrics and approach. Its completed-phase label is not substantive approval.
4. **Reference-ID collision risk.** Calling this “RR-002” would confuse a process-count proposal with the actual spatial investigation and its immutable evidence.
5. **Opportunity cost.** It does not resolve current blockers: false-pass-capable reference readers, unbound data provenance, eleven unsupported charged-PDC reactions, twenty unscheduled unary/background channels, or the missing multispecies engine.

### Recommended action and safeguards

After approval, retire this session through GPS's supported lifecycle, preserve its history, remove the obsolete residual-close pending entry, and start **R01 `validation-evidence-and-ci`**. Do not run a generator or delete any CSV/provenance file. If a future regression produces a new process-count hypothesis, create a new narrowly scoped seed/run-request ID with its own preregistered observables; do not silently revive this old contract.

### Improvements to the intended next M7 work

The original sequence also placed an oracle/ODE seed before an underspecified scheduler. Improve it by (a) exposing complete reaction execution as R03, (b) keeping pair/SBS tests independent of production sampling in R04, (c) defining event/time/identity/search semantics in R05, and (d) using one stage-matched count/G(t) campaign in R06/R07. This creates measurable boundaries without splitting every scientific detail into a separate GPS session.

## 4. Completed sessions: retain, but qualify their claims

The completed record includes canonical-species migration, M6 comparison, M6 diagnosis/fix, GPS backlog migration, N1 thermalisation audit, residual Rust audit, option3 reaction table, 900 keV spatial investigation, M6 closure, and diffusion/two-body IRT. Preserve their indexes, grills, tickets and commit logs unchanged.

| Completed work | What it establishes | What it does not establish |
|---|---|---|
| Canonical species and option3 table | Source identities, values and table coverage | Runtime support for all 72 reactions |
| M6 source correction and closure | Qualified stage-matched physicochemistry acceptance | Independent multi-Geant4-seed covariance analysis or full chemistry |
| N1/N2/spatial investigation | Boundary diagnostics and a preregistered local spatial conclusion | Universal spatial equivalence or portable reproduction from a fresh clone |
| Diffusion/two-body IRT | Existing mathematical kernels, free diffusion and reproducibility checks | Charged PDC, unary channels, multispecies scheduler, SBS/Geant4 campaign, ODE or G(t) |
| GPS backlog migration | Historical workflow organization | Current dependency correctness or substantive completion of an unfilled template |

This review does not undo those sessions. It narrows overbroad interpretations and addresses deficiencies through new/revised work.

## 5. Original requirements not represented by the pending registry

| Requirement and source | Decision | Owner / activation |
|---|---|---|
| Accurate provenance, strict public input contracts, executable current gates | **NEW** | R01/R02; prerequisites to scientific expansion |
| Charged PDC, unary/background clocks, product placement | **NEW explicit scope** | R03, extracted from the vague scheduler promise |
| Full-slowing and track-segment modes, `PrimaryKiller`, escaped/remaining energy, abort/weight semantics (docs/01 A.4) | **KEEP requirement, sequence explicitly** | R06 full-slowing first; R09 track segment; no claim of LET/segment parity before R09 |
| Original 1 MeV electron case and seven V11 times | **KEEP with explicit profile** | R06/R07; pure-DNA endpoint policy, not silent substitution by 900 keV |
| V4 final-state sampling versus Geant4; V6 stopping/W assessment | **KEEP** | R07 coverage/assessment; new independent fixtures where original coverage is missing |
| V10 closed well-mixed 1% ODE criterion | **DEFER**, not declare passed | D01; outside first unbounded-history profile. R05 conservation tests are necessary but not a replacement. Full historical M7 remains qualified. |
| V12 literature G(t), with uncertainty and no invented pass threshold | **KEEP** | R07 electron and R11 proton scientific assessment |
| Proton/H charge transport, Miller-Green/Rudd/Born/ion elastic/Dingfelder; V2–V7/PSTAR | **KEEP, defer until electron chain accepted** | R10; bounded architecture expansion, not registration-only fiction |
| Proton 10 MeV G(t) | **KEEP** | R11 after R10 and the electron chemistry chain |
| Proton RPWBA 100–300 MeV after Born | **DEFER** | D02; explicit extended-energy requirement, not silently included in R10 estimates |
| Options4/6 electron alternatives (M4) | **DEFER** | D03; named scientific need and model-boundary validation |
| Profiling, scaling, benchmarks (M9) | **KEEP, reorder** | R12 after a validated electron workload; repeat after proton implementation |
| Extension guidance, source mapping, reproducible examples (M10) | **KEEP, continuous** | Each seed updates its contract; R12 consolidates release guidance |
| Higher electron energies, boundaries, other media/species, oxygen/scavengers, UHDR/intertrack chemistry, IRT-syn, intra-history parallelism (M11) | **DEFER** | D04–D07 in proposed roadmap; not prerequisites to current pure-water electron deliverable |
| Slow differential/rejection-path equivalence and atomic de-excitation alternatives (docs/08, Q4/Q12) | **DEFER** | D08; current fast-table/no-deexcitation profile remains explicit |

No requirement is removed merely because it lacks a pending GPS entry. Deferred requirements are not automatically registered as speculative implementation seeds; their activation and minimum acceptance are recorded in the proposed roadmap.

## 6. Workflow and CLAUDE.md guidance

`CLAUDE.md` currently contains only `@AGENTS.md`. This is a good anti-drift structure: **keep that delegation**. Put any approved shared policy improvements into `AGENTS.md`, not a second independent rule set. Suggested additions, not edits made here:

- A current capability profile and a pointer to the maintained gate manifest.
- Three separate statuses: implementation, evidence acquisition, scientific verdict; failed/inconclusive is never “validated.”
- A ban on treating missing reference data as success in a required gate, and a warning against blanket `--ignored` execution.
- Verified runtime input/provenance identity and explicit constructor/stage/time/geometry policies.
- Mandatory preregistration and immutable reference artifacts, with user-only Geant4 execution retained.
- Same-platform thread/batch determinism and observation-without-RNG-side-effects as interface contracts.

Do not embed changing seed lists, hardcoded machine roots or long historical results in either instruction file. The reports and later maintained roadmap should carry those details. Any future instruction change should be a small reviewed part of R01/R02, not an unrequested change in this audit.

## 7. Approval-ready migration plan

Approval requested is for **this mapping and the proposed contracts**, not automatic code execution.

1. Snapshot the local pending registry, active pointer and selected session through the supported GPS workflow. Preserve all historical sessions and all reference data. Recheck that no other session has changed state since this audit.
2. Retire `m6-process-counts-geant4-request` and `m6-residual-close-gate` from executable work, recording their supersession by the approved M6 closure. Do not physically erase session evidence.
3. Replace the remaining eight unfinished objectives using the exact table in section 2. R04 absorbs the missing oracle and its closure; R03/R05 split scheduler scope; R06/R07 coordinate count/G(t) evidence; R08 owns CLI; V13 is integrated throughout. Retain independent gate verdicts.
4. Register R01–R08 as the near-term sequence, with dependencies in the proposed roadmap. Retain R09–R12 as ordered later work and D01–D08 as explicit deferred requirements; do not create dozens of speculative seeds.
5. Update `docs/gps-backlog.md`, `docs/gps-backlog-m6-residual.md`, current-state sections of docs/08/docs/09 and registry dependencies together in one migration change. Preserve dated historical entries; mark them superseded rather than rewrite their results. Check that every dependency resolves and that no retired objective remains active.
6. Start **R01 `validation-evidence-and-ci`**. All future Geant4 executions remain user-owned. An external wait resumes the same seed; it does not require a new architectural planning session or a duplicate closure seed.

Current state after this audit: **migration not applied**. Code, active GPS state, original roadmap and reference evidence remain unchanged.
