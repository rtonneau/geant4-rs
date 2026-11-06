# Ticket 07: convergence-and-g2

**Status:** ✅ Done

## Local Test Result

- `cargo test -p g4-chemistry --test history_convergence`: PASS, 6 deterministic
  tests passed; 1 explicitly ignored statistical gate. Comparator tests cover
  acceptance boundaries, positive/negative injected bias, insufficient precision,
  rare means and the exact 0.1 boundary, covariance, degeneracy and invalid data.
  Geometry tests verify creation order, position bounds and actual pruning.
- `cargo test -p g4-chemistry --test history_convergence preregistered_cutoff_convergence_gate -- --ignored --exact --nocapture`:
  **INCONCLUSIVE**, expected exit 101. Full fixed budget completed: 524,288
  executions, 46,080 batch rows, 1,080 endpoints; 687 PASS, 393 INCONCLUSIVE,
  0 FAIL, 0 INVALID. This scientific failure to establish equivalence blocks G2;
  it was not relabeled as ordinary test success or followed by sample expansion.
- Independent 60-digit Python Decimal audit of all retained batch sums directly
  solved the registered Fieller quadratic: all 1,080 verdicts reproduced;
  maximum interval-endpoint discrepancy 2.88e-16; 46,080 unique batch rows checked.
- `cargo test -p g4-chemistry --test history_spatial sparse_high_population_reports_bounded_storage_events_and_candidates -- --exact --nocapture`:
  PASS, 1 test. Measured 5,000 candidate visits/channels/scheduled and executed
  reactions/peak pending events; 15,000 arena/index identities; candidate row 1;
  2,630,432 peak retained engine payload bytes, not RSS.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS, exit 0; 561 passed, 0 failed, 22 ignored across
  66 summaries. The explicit statistical gate above is not included in this pass.
- `cargo check -p g4-sim --no-default-features`: PASS.
- `git diff --check`: PASS.

Evidence is retained under `.scratch/tests/2026-10-29__serial-irt-history-engine/`:
`ticket07-{provenance,campaign,resource,fmt,clippy,workspace,physics-only,audit}.log`,
`ticket07-endpoints.csv` and `ticket07-audit.py`. The full 2,171,398-byte campaign
log contains every count batch and every endpoint, including all unresolved
rows; SHA-256 `39a826333dd794b8c4f5534a67d7d18b042092018be347817ca9550a60865e2f`.
These are local scratch artifacts, explicitly not committed reference fixtures.

Correction-pass verification:
- `python validation/irt_history/audit_r05_c1.py`: PASS, exit 0; repository
  evidence hashes/sizes, 3,072 compact batch rows (46,080 integer count sums),
  the complete 1,080-endpoint grid and all original verdicts/intervals verified.
  Result remains 687 PASS / 393 INCONCLUSIVE; maximum interval-endpoint
  discrepancy remains 2.88e-16. No sampling or scratch input is required.
- A copied standalone bundle audited successfully outside the repository
  layout, and after CRLF conversion. Negative checks rejected altered bytes,
  missing batches, negative counts, duplicate endpoints and changed verdicts;
  structural/numerical cases were also rejected after updating their copied
  fixture hashes. Original evidence was untouched. Output:
  `ticket07-archive-negative-checks.log` in the same scratch directory.
- Formatting and `git diff --check` rechecked: PASS. New evidence/audit paths
  verified not ignored by Git. Rust implementation and harness bytes are
  unchanged from the full workspace checks and the original sampling run;
  no stochastic test was rerun during this correction.

## Review Notes

Base: f62e1e66df578ec194ba208248872b92a3c42e71. Pre-existing changes: GPS session config and newly started ticket log only; project tree clean. Recovery count: 0.

Prepared R05-C1 in `docs/validation/irt-history-convergence.md` before sampling:
two 65,536-history ensembles, 64 independent batches, fixed seed/ID ranges,
full option3 network, all 15 species at six observations, six search comparisons,
paired covariance-aware 99% intervals and explicit FAIL/INCONCLUSIVE/INVALID
rules. Parent committed the unchanged preregistration at
`c7188b35bd06420e1ba914c5f404e0c2c2fcfc5b`, then resumed implementation and
sampling. The original ticket base and recovery count above are preserved.
Directly read the environment-backed Geant4 11.4.1 G4DNAIRT.cc constructor,
binning, sampling, product insertion and clamped-bin semantics; verified header
version 1141. No Geant4 generator was run or source modified.

Added `history_convergence.rs` without production changes or dependencies.
It executes all registered comparisons, prints exact integer batch sums and
all endpoint means/intervals/verdicts, hashes its own compiled source/inputs,
and rejects malformed, incomplete or unresolved evidence. The Fieller solver
uses an algebraically equivalent regression form to avoid cancellation without
variance clipping; an independent high-precision audit confirms its results.
The test-only parallel batch wrapper preserves serial independent histories,
ordered batch results and paired chemistry-stream resets across policies.

Documented the actual source semantics, frozen criteria, complete comparison
summary, representative unresolved endpoints, local artifact identities,
measured resource totals/maxima, payload-versus-RSS distinction and current
G2/R06 block in the convergence report and reproducibility guide. All observed
reference means exceeded 0.1; this limitation is explicit, with no claim of
statistical rare-population coverage. Source 11.4.1 remains distinct from G1-F's
11.4.3 pair evidence; physical-law and later R07 acceptance remain separate.

Independent review round 1 (gpt-6-astra xhigh, base f62e1e66df578ec194ba208248872b92a3c42e71): P2 docs/validation/irt-history-convergence.md:315 retains batch statistics and Decimal audit only in ignored scratch, preventing fresh-checkout audit. Action: preserve compact evidence and audit script in repository, update docs and verify. Recovery/correction count: 1. Remaining review otherwise found no actionable defects; rare-population coverage remains absent.

P2 disposition: addressed in correction pass 1. Added
`validation/irt_history/audit_r05_c1.py` and
`validation/irt_history/evidence/r05_c1/{batches.csv,endpoints.csv,provenance.json}`.
The archived CSVs total 448,090 UTF-8/LF bytes and retain every original
count sum, mean, interval, verdict and reason. Extraction verified the original
campaign-log hash before reading it and created only new files; the original
scratch evidence, protocol, scientific thresholds and G2 verdict are preserved.
Provenance records the protocol/engine revisions, dirty state, raw input/code
hashes, exact command, seed and history ranges, platform/toolchain, resource
measurements and portable artifact hashes. The standalone standard-library
Python audit checks exact coverage/uniqueness, counts, paired means/covariance
and the original Fieller quadratic at 60-digit precision. The convergence and
reproducibility docs now point to this repository evidence and make clear that
audit success verifies an INCONCLUSIVE result, not a G2 pass.

Independent review round 2 (fresh gpt-6-astra xhigh, base f62e1e66df578ec194ba208248872b92a3c42e71): no actionable findings. P2 resolved by archived compact statistics, endpoints, provenance and standalone audit; hashes and verdict inspected. Accepted evidence with G2 INCONCLUSIVE and rare-population coverage absent; no independent execution by reviewer.

## Blockers / Challenges

No unresolved implementation blocker. **Scientific G2 remains blocked:** 114
finite intervals cross an acceptance boundary; 279 endpoints have no observed
paired differences under the registered conservative degeneracy rule. No
campaign reference mean was rare, so that branch has deterministic harness
coverage only. No default/criteria/seed/sample-budget change was attempted.
This is the ticket's honest evidence result, not an assertion of convergence.

The preregistration commit checkpoint was resolved by the parent before any
sampling. No commit/stage/state script was run here. As in prior tickets, the
optional user CONFIG_RESOLUTION.md and project .claude-project.json were absent;
the existing repository source-root convention located the verified 11.4.1
source without inventing config. One read-only dispatch was aborted before
starting and retried successfully; one documentation patch context mismatch
was corrected without changing unrelated files. No permission denial occurred.

## Commits

- 0039fea test: record IRT cutoff convergence evidence (ticket 07)

## Time Spent

32m (ticket-start.js to ticket-complete.js)
