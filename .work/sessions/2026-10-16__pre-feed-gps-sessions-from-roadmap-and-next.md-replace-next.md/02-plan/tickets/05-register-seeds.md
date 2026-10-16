# Ticket 05: register-seeds

**Model:** sonnet
**Effort:** medium

**Acceptance Criteria:**
- [ ] An entries JSON (`{ "sourceDirection": null, "candidates": [...] }`) is written to the scratch directory with 13 entries, one per backlog section, each with `slug` equal to the backlog heading, `strength`, `problem` (starts with the goal, includes "requires <slug>" where applicable, and the C++/RUN_REQUESTS rule where relevant), `solution`, and `files`.
- [ ] `scout-merge.js --from docs/gps-backlog.md <entries.json>` succeeds (fix the JSON on `❌` and rerun).
- [ ] `/gps status` lists the 13 seeds.

**Files to Touch:**
- `.scratch/tests/2026-10-16__pre-feed-gps-sessions-from-roadmap-and-next.md-replace-next.md/seeds.json`

**Verification Step:**

Run:
```bash
node C:/Users/rtonneau/.claude/plugins/cache/grill-plan-ship/grill-plan-ship/2.8.2/skills/gps/scripts/status.js
```

Expected:
The ideas list shows the 13 slugs of `docs/gps-backlog.md`.

**Notes:**

Read `references/scout.md` (`--from` section) for the entry fields; never edit `.pending-seeds.json` by hand. If `status.js` does not exist under that name, follow `references/status.md`. The JSON lives in scratch and is not committed.
