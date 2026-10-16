# Ticket 04: drop-next-md

**Status:** ✅ Done

## Local Test Result

grep next.md outside .work/.VSCodeCounter: none after rewording; cargo fmt --all -- --check ok; cargo clippy --workspace --all-targets -- -D warnings finished clean.

## Review Notes

Content of next.md sections (i).1, (i).2 and the M7 prompt is in docs/gps-backlog.md and docs/09 (checked in tickets 02-03); section 3 obsolete (PR #10 merged). git rm docs/next.md; README and physchem_a.rs comment fixed (comment only).

## Blockers / Challenges

None. Untracked .VSCodeCounter/ output (an editor extension) still mentions next.md; not ours, not touched.

## Commits

- f60fe91 docs: drop next.md in favour of gps backlog (ticket 04)

## Time Spent

1m (ticket-start.js to ticket-complete.js)
