# Ticket 01: restore-geo0-spec

**Status:** ✅ Done

## Local Test Result

Base 55ba0f8dccb7d17dd037c49ec47e557d6db7eb9b, clean tree. Ran `git merge-base --is-ancestor 4cf47d6 HEAD` → NOT, and `gh pr view 30` → merged into `test/rr008-scheduler-followup`, not main. Docs-only change; no build needed.

## Review Notes

Scope changed by user decision: nothing was deleted from main (PR #30 is on an unmerged branch together with the RR-008 work), so instead of restoring it, docs/04 gains D.6 with the boundary rules, cited from the local 11.4.1 source (G4Transportation.cc:735-741, G4SteppingManager.cc:346-356). This corrects the grill: rejecting an Outside primary matches Geant4 (fatal Tracking0010) and is not a deviation. D.1 step 2 points to D.6.

## Blockers / Challenges

The grill's premise (spec deleted) was wrong; resolved by asking the user.

## Commits

- accdac8 docs: boundary-limited transport rules from Geant4 11.4.1 (ticket 01)

## Time Spent

2m (ticket-start.js to ticket-complete.js)
