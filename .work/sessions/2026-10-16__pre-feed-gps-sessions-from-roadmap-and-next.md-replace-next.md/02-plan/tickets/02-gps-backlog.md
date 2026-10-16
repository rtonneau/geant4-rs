# Ticket 02: gps-backlog

**Model:** sonnet
**Effort:** high

**Acceptance Criteria:**
- [ ] `docs/gps-backlog.md` exists with an intro (how seeds work, the RUN_REQUESTS convention, the "requires <slug>" rule) and one section per seed, 13 in total: `m6-comparison-report`, `m6-diagnose-and-fix`, `m6-close-gate`, `m7-option3-reaction-table`, `m7-diffusion-and-two-body-irt`, `m7-sbs-oracle-and-ode`, `m7-sbs-close-gate`, `m7-multispecies-irt-scheduler`, `m7-scheduler-close-gate`, `m8-gt-scoring-and-chem6-reference`, `m8-gt-close-gate`, `m8-config-and-cli`, `m8-reproducibility-v13`.
- [ ] Each section lists: goal, requires, C++ tasks (or "none"), Rust tests, pre-registered acceptance (tolerances are stated or marked "to be pre-registered in the grill before the first comparison"), docs to update.
- [ ] Section `m6-comparison-report` and `m6-diagnose-and-fix` carry everything from `next.md` section (i).1 (the failing `1000 eV H2O2: z = 111.6` evidence, tasks 1-3, the hypotheses list, "never loosen a tolerance", "regenerating physchem.csv is out of scope", the 900 keV stack overflow note).
- [ ] M7 sections carry everything from the `next.md` M7 prompt (reaction table with line provenance, per-history chemistry streams, A+B and A+A IRT, partially diffusion-controlled, V9/V10 items, SBS oracle, Bonferroni families stated before running, cited Geant4 11.4.1 sources, docs to update).
- [ ] Dependencies are explicit: M6 closure precedes anything consuming M6 species; `m7-option3-reaction-table` is independent; M8 proton 10 MeV is deferred to M5.
- [ ] Open items are listed (does M6 need diagnostic Geant4 data; V11 energies and times; Q13 default).

**Files to Touch:**
- `docs/gps-backlog.md`

**Verification Step:**

Run:
```bash
grep -c '^## `\?m[678]-' docs/gps-backlog.md && grep -n "z = 111.6\|stack overflow\|requires" docs/gps-backlog.md | head
```

Expected:
`13`, then lines showing the H2O2 evidence, the stack overflow note and several "requires" lines.

**Notes:**

Sources: `docs/next.md` (still present at this point), `docs/08-roadmap.md` M6-M8, `docs/07-validation-strategy.md` V8-V13, `docs/09`, and `.work/sessions/*pre-feed*/01-grill/resume.md` (Architecture & Approach lists the seeds and their order). Use `## <slug>` headings so ticket 5 can build entries from them. Do not invent tolerances or physics constants; cite docs/07 for existing criteria. Statistical wording must keep AGENTS.md's priority: correctness over agreement with Geant4.
