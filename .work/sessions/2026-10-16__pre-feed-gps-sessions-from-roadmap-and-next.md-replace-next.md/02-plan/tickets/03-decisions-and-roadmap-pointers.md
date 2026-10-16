# Ticket 03: decisions-and-roadmap-pointers

**Model:** sonnet
**Effort:** medium

**Acceptance Criteria:**
- [ ] docs/09 has a new dated entry recording the confirmed decisions: M7 implements option3 reactions first; the M6 same-decay pair statistic stays distinct from any RDF/nearest-neighbour observable (R16 pointer); Q12, Q13, Q16, Q17 kept unless the M6 reference contradicts them; M6 closes before M7 consumes it; M8 proton 10 MeV deferred to M5.
- [ ] Q24 is marked closed with a pointer to `validation/geant4/BUILD_NOTES.md` section 5 (issues G-K); the M6 gate is stated as still open (`1000 eV H2O2: z = 111.6`).
- [ ] docs/08 M6, M7 and M8 sections each gain one line pointing to their seeds in `docs/gps-backlog.md`; M8 notes the proton deviation. No existing sentence or number in docs/08 or docs/09 is altered or removed.
- [ ] No ADR is created unless a decision is hard to reverse and surprising (none expected; say so in the commit log).

**Files to Touch:**
- `docs/09-open-questions-and-risks.md`
- `docs/08-roadmap.md`

**Verification Step:**

Run:
```bash
git diff --stat -- docs && git diff docs/08-roadmap.md docs/09-open-questions-and-risks.md | grep '^-' | grep -v '^---' | wc -l
```

Expected:
Two files changed; `0` removed lines except the Q24 status line if it was edited in place (at most 1).

**Notes:**

Q24 text is at docs/09 line ~59 (an "M6 part A additions" bullet). Prefer appending "CLOSED" text over rewriting it. Backlog section names are in ticket 2.
