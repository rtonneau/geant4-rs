# Ticket 01: reference-contract

**Status:** ✅ Done

## Local Test Result

Executed `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; each exited 0. Full outputs are preserved in `.scratch/tests/2026-11-08__irt-syn-parallel-checkpoint-direction/01-{fmt,clippy,test}.log`. No ignored scientific consumers or Geant4 generators were run.

Documentation audit `python .scratch/tests/2026-11-08__irt-syn-parallel-checkpoint-direction/01-doc-audit.py` exited 0: all 14 source hashes match both 11.4.1 and 11.4.3, prior documentation content is preserved, and relative link targets exist. Output is `01-doc-audit.log`; static placement caller/assignment evidence is `01-placement-callers.log`. `git diff --check` passed.

## Review Notes

Implementation base: f5f2eafe257b8398dbe9c122e7a0da6ee334fc6a. Pre-existing changes: GPS-owned .session-config.json and this new ticket log from ticket-start; no simulation or documentation edits. Recovery/correction passes: 1 (parent documentation correction after independent review). Requested model/effort: inherit. Independent reviewer gps_review_01 inspected frozen diff against recorded base. Two P2 findings accepted: missing count bands and native-unit requirement for dimensionally inconsistent source placement. Parent corrected both; fresh reviewer gps_review_01_corrected checked the frozen corrected diff and source units: no findings. Parent accepts ticket 01 documentation criteria; prior Rust checks remain applicable because correction changes prose only, and git diff --check passed. Executable diagnostics remain explicitly future gates. No scientific results were observed or criteria tuned.

Added the source contract in `validation/geant4/README.md`, prospective SYN gates in `docs/07-validation-strategy.md`, production direction in `docs/08-roadmap.md`, and ADR 0004 through the authorized domain-document scaffolder. ADR 0003 and all historical reference evidence remain unchanged. Normative baseline remains 11.4.1; 11.4.3 is explicitly supplemental unless separately amended. Reviewed executable model, stepper, sampler, placement, scheduler, Brownian transport, reaction-table and example sources; no Rust engine changes. Source-only behavior and proposed unary/search/worker/restart extensions have separate validation requirements. Statistical bands, precision looks, multiplicity, endpoints and resource budgets are preregistered; exact consumer commands will be registered as tests are introduced.

## Blockers / Challenges

No blocker to this documentation ticket. The local config-resolution document and project JSON are absent; source discovery used the supplied environment-based layout directly, without inventing config or paths. Both unpacked reference trees lack git metadata; recorded version headers and source hashes instead. `apply_patch` initially refused the roadmap's pre-existing non-UTF8 degree byte; a byte-preserving insertion retained all original text, verified by the documentation audit.

Production gates remain open for source overlap candidate carryover, inactive positive-duration placement inferred from the full source call graph (requires executable confirmation), implicit unary execution, interruption/residual-clock semantics, and finite-search error control. The contract does not silently repair these or claim physical validity. Required future reference runs are authorized for the dedicated reference-run workflow; permission is not an active blocker. No ticket/delivery/session-state scripts, staging or commits were performed.

## Commits

- 7496eae docs: define IRT-syn reference contract (ticket 01)

## Time Spent

16m (ticket-start.js to ticket-complete.js)
