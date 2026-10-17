# Ticket 03: verdict-and-source-proven-fix

**Model:** opus
**Model (Jev):** opus-5.5, confidence 0.69
**Effort:** xhigh

**Acceptance Criteria:**
- [ ] Verdict from tickets 01 and 02 is a stage family (a relation holds on Geant4, fails on Rust, other families excluded) or "undecided".
- [ ] A Rust fix is made only if a Geant4 `.cc` line proves the difference and the data agree; it has its own test and its own commit. Otherwise no model change.
- [ ] After a fix, `m6_prechem_counts_and_rms_match_geant4` is rerun with unchanged criteria (z <= 4.5, 15%, 20%) and the result is reported honestly.
- [ ] Rust baselines that change are re-baselined with a documented justification in the commit; Geant4 reference files untouched.

**Files to Touch:**
- Only the Rust files the proven defect requires (none if undecided)
- the affected Rust-side test baselines, if any

**Verification Step:**

Run:
```bash
cargo test -p dna-sim --test physchem_reference m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact --nocapture
```

Expected:
Reported as run; pass or fail is stated as printed. If no fix was made, the output is unchanged from RR-001 (16 failing checks).

**Notes:**

No tolerance is loosened. M6 closes only if all 28 rows pass. If the fix is not clearly source-proven, stop, document and leave a new seed for it.
