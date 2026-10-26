# Ticket 04: gate-inventory-runner

**Model:** heavy
**Effort:** high

**Acceptance Criteria:**
- [ ] A versioned inventory names the exact four M2, two M3 and one M6 maintained tests, their required committed fixture identities and profiles, and the historical exclusions.
- [ ] A standard-library runner validates required file presence, nonempty content and recorded hashes, plus committed data availability, before invoking any registered statistical comparison; missing or mismatched evidence exits nonzero with an actionable path.
- [ ] `--tier required` performs fail-closed evidence preflight; `--tier statistical` runs only the seven registered ignored selections and reports each stage's result.
- [ ] Runner unit tests prove a removed fixture, changed hash, invalid manifest and a deliberately perturbed valid fixture fail; the unchanged committed fixtures pass.

**Files to Touch:**
- `validation/gates.json`
- `scripts/run_validation_gates.py`
- `scripts/test_run_validation_gates.py`

**Verification Step:**

Run:
```bash
python -m unittest discover -s scripts -p test_run_validation_gates.py
python scripts/run_validation_gates.py --tier required
python scripts/run_validation_gates.py --tier statistical
```

Expected:
Runner tests and required preflight pass; statistical tier executes exactly four M2, two M3 and one named M6 test, and all seven pass.

**Notes:**

Use explicit test names with `--exact` so a new ignored test cannot join the maintained tier accidentally. Do not include historical post-zero-time gates or scratch diagnostics. Reference/data hashes identify evidence, not new scientific acceptance thresholds.
