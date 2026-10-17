# Ticket 01: extract-option3-reactions

**Status:** ✅ Done

## Local Test Result

`python scripts/extract_reactions_from_cpp.py "$env:DEV_DIR/GEANT4/geant4-v11.4.1-source/geant4-v11.4.1" --check crates/dna-chemistry/tests/fixtures/cpp_reactions.json` exited 0 and reported that all 72 extracted reactions match the committed fixture.

## Review Notes

Checked the 72-record count and source-class distribution (I 7, III 3, II 31, IV 11, VI 1, scavenging 19). Confirmed five executable/comment rate discrepancies and reaction 53 as `O3m + NoneM -> H3Op + O2m`, Type VI, `2.66e3 / s`. A RED check caught Type VI headings without a trailing delimiter; the parser now accepts both heading forms and asserts reaction 53's class.

## Blockers / Challenges

The Type VI heading differs syntactically from the earlier section headings; handled with a source-shape assertion and parser fix. No remaining blocker.

## Commits

- 84c67ff feat: extract option3 reaction table (ticket 01)

## Time Spent

3m (ticket-start.js to ticket-complete.js)
