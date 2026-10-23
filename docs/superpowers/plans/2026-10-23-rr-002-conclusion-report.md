# RR-002 Conclusion Report Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the stale RR-002 awaiting-data narrative with the complete, reproducible 80-histories-per-seed conclusion.

**Architecture:** Keep the full scientific interpretation in `docs/08-roadmap.md`. Mirror only the status and essential result in the risk register and run ledger, while the Geant4 README retains operational context.

**Tech Stack:** Markdown documentation; repository text search and diff verification.

**Spec:** Approved in the 2026-10-23 conversation: a four-file update with one authoritative report.

## Global Constraints

- Preserve the preregistered settings: five fixed seeds, analysis seed 20261026, 20,000 bootstrap replicates, and the 4% precision rule.
- Report the result exactly: ratio 0.981740457, 95% interval [0.949564151, 1.015007983], relative half-width 3.333%.
- State that inclusion of 1.0 means no statistically resolved primary difference, not proof of equivalence.
- State that artifacts and provenance remained local-only and were not promoted to committed reference evidence.

## Review Focus

- No stale “awaiting data” status remains in the four updated RR-002 sections.
- History counts distinguish 80 per seed, 400 per implementation, and 800 across both implementations.
- The conclusion does not overstate confidence or causation.
- The original 40/80/160 stopping rule remains historically accurate.
- RR-002 remains separate from the formal M6 closure decision.

---

### Task 1: Record and synchronize the RR-002 conclusion

**Files:**
- Modify: `docs/08-roadmap.md`
- Modify: `docs/09-open-questions-and-risks.md`
- Modify: `validation/geant4/README.md`
- Modify: `validation/geant4/RUN_REQUESTS.md`

**Interfaces:**
- Consumes: verified RR-002 80-history run output.
- Produces: one authoritative scientific report and three consistent status summaries.

- [x] **Step 1:** Replace the roadmap awaiting-data paragraph with the full design, result, interpretation, stopping decision, and local-only evidence limitation.
- [x] **Step 2:** Update the risk register to close the awaiting-data status while retaining the non-equivalence caveat.
- [x] **Step 3:** Update the Geant4 README heading and append the completed 80-history outcome without deleting the reproducible 40/80/160 procedure.
- [x] **Step 4:** Mark RR-002 complete in the run ledger and update its purpose section with exact results and artifact status.
- [x] **Step 5:** Run `rg` for stale RR-002 awaiting-data language and `git diff --check`; inspect the final diff.
