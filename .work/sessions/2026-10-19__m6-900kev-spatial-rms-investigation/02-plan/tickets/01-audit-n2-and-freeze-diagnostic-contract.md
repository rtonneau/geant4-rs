# Ticket 01: audit-n2-and-freeze-diagnostic-contract

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] `docs/02-cpp-reference-inventory.md` traces terminal physical electrons through Geant4 11.4.1 `e_aq` creation and the equivalent Rust path, with exact source paths for position, time, alive/killed state, parent/track identity, delayed/secondary-list insertion, and creation/processing order.
- [ ] `docs/08-roadmap.md` records the N2 verdict and distinguishes source-proven behavior from the still-untraced physical-cascade-event to molecule-insertion mapping; no model behavior changes.
- [ ] `docs/09-open-questions-and-risks.md` records the remaining ordering limitation and the approved five-boundary diagnostic contract, primary estimand, bootstrap rule, precision rule, and interpretation of “first detected divergence.”
- [ ] The contract fixes radial edges to `10^(k/2)` nm for `k = 0..16`, underflow/overflow bins, required identity and moment fields, five seeds x 40 histories, 20,000 bootstrap replicates, four-percentage-point relative half-width, uniform 80/160 expansions, and the 800-history-per-implementation cap.

**Files to Touch:**
- `docs/02-cpp-reference-inventory.md`
- `docs/08-roadmap.md`
- `docs/09-open-questions-and-risks.md`

**Verification Step:**

Run:
```bash
rg -n "N2|first detected divergence|20,000|10\^\(k/2\)|800 histories" docs/02-cpp-reference-inventory.md docs/08-roadmap.md docs/09-open-questions-and-risks.md
```

Expected:
Each preregistered rule and the source-backed N2 trace appears in the intended documentation; `git diff --check` reports no errors.

**Notes:**

Read the Geant4 11.4.1 source, not comments or prior summaries alone. Do not modify Rust or C++ behavior in this ticket. Treat identifiers as within-implementation lifecycle evidence because the two RNG engines do not support eventwise pairing.
