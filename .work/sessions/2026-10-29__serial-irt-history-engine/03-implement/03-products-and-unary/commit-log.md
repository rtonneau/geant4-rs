# Ticket 03: products-and-unary

**Status:** ✅ Done

## Local Test Result

- `cargo test -p g4-chemistry --test history_reactions`: PASS, 12 passed, 0 failed. Includes execution/accounting of all 72 source channels, product counts 0–3, repeated-product identities, contact cascades, unary ties and pair competition, retained unary fallback, product-birth clocks, lazy partner sampling in both source orientations, inclusive final boundary, and explicit capacity/numerical errors.
- `cargo test -p g4-chemistry --lib history::tests`: PASS, 5 passed, 0 failed, including rejection of modified products and a noncanonical reaction-53 residual.
- `cargo test -p g4-chemistry --test history_pair --test history_competition`: PASS, 12 passed, 0 failed.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS. Full output: `.scratch/tests/2026-10-29__serial-irt-history-engine/ticket03-clippy.log`.
- `cargo test --workspace`: PASS, exit 0; 530 passed, 0 failed, 21 ignored across 61 test/doc-test summaries. Full output: `.scratch/tests/2026-10-29__serial-irt-history-engine/ticket03-workspace-tests.log`.
- Final `cargo fmt --all -- --check` and `git diff --check`: PASS after the adjacent placement API documentation correction.

## Review Notes

Base: ccdfef5674c8a0cbfa31a264a610b53934c5e951. Pre-existing changes: GPS session config and newly started ticket log only; project tree clean. Recovery count: 0.

- Extended the existing `PairHistory` constructor to accept pair and unary descriptors and evolve products. `HistoryReaction` represents one or two mobile parents using variable-length `participants`/`generations`, monotonic source-ordered product IDs, placement and checked `ReactionAccounting`; `PairReaction` remains a compatibility type alias. Bulk reservoirs never enter the mobile arena. Existing regression assertions were adapted for the new arena products/lineage and unary support while preserving their original competition, timing and RNG checks.
- Corrected only the module/function documentation in `placement.rs`: an older history partner retains its stored site, so the former isolated-pair requirement that both sites share a time was too restrictive. No placement formulas changed.
- Each new product samples only earlier live identities, in ID/channel order. Source-order siblings become candidates sequentially, each unordered pair is sampled once, and old-old pairs (including escaped clocks) are never resampled. Events retain the sampling time separately from both parents' stored materialization times. Consumed parents are invalidated before product scheduling. Capacity is checked before consuming parents; a later scheduling failure retains diagnostics and makes the history terminally failed.
- Read Geant4 **11.4.1** `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc`: `Sampling` 212–231 (temporary older-partner Brownian position used only for r0), 250–281 (independent unary clocks and fake stationary partner), and `MakeReaction` 393–508 (stored-site placement, source-ordered products and immediate product sampling). Also read `management/include/G4ITReaction.hh` 204–241/269–274 (pair deduplication/invalidation) and `source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc` 534–664 (literal unary products and rates). Compared complete local 11.4.1 and 11.4.3 `G4DNAIRT.cc` files with `git diff --no-index`: no differences. No C++ reference modifications or generator runs.
- Explicit source deviations retained/documented: canonical insertion ties and inclusive horizon; source-table parent ordering in the asymmetric placement kernel rather than upstream scheduling-track ordering; earliest unary clock retained even when a sampled pair is earlier. The latter prevents losing the unary fallback when another reaction consumes only that pair's other parent. Both deviations affecting product/unary behavior have targeted fixtures. Source-order placement can change product positions when a new product is source reactant B. Temporary diffusion never changes an older partner's position, generation or clocks. Normals retain the existing Box–Muller helper rather than CLHEP's sequence, and the placement kernel's documented native-unit Gaussian-width limitation remains.
- Each executed event checks actual mobile reactants/products and integer hydrogen/oxygen/charge changes against explicit consumed bulk composition and signed implicit-water products. Only exact ID 53, `O3m + NoneM -> H3Op + O2m`, may carry residual `[3, 0, 1]`; a blanket balance exemption is rejected. Accounting uses audited option3 species composition. These deterministic checks do not establish G2, many-body physical accuracy, search-cutoff convergence, or R06 production readiness.

Independent review: gpt-6-astra, xhigh effort, reviewed frozen tracked and untracked changes against base ccdfef5674c8a0cbfa31a264a610b53934c5e951, ticket criteria, evidence and Geant4 11.4.1 source. No actionable findings or verification gaps. Accepted; placement documentation and prior regression adaptations are justified adjacent changes. R05/G2 and independent physical validation remain open.

## Blockers / Challenges

None unresolved. Initial fixture calibration exposed an earlier unrelated unary clock and an incorrect test expectation that reaction 4 produces OH rather than its source OHm; corrected the fixtures from the independent clocks/source table. Required workspace checks passed. No reference data was overwritten and no Geant4 generator was run.

## Commits

- 779bd3d feat: evolve products and unary reactions (ticket 03)

## Time Spent

23m (ticket-start.js to ticket-complete.js)
