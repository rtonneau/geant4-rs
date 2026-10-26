# Ticket 03: v10-lsoda-well-mixed-oracle

**Status:** ✅ Done

## Local Test Result

`cargo test -p dna-chemistry --test kinetics_ode -- --nocapture` passed: 1 test,
with fixed-seed SSA means `[999.17578125, 999.17578125, 9000.82421875,
9000.82421875]`, LSODA populations approximately `[1000, 1000, 9000, 9000]`,
and maximum relative error `0.000824` (0.0824%).

`cargo test -p dna-chemistry --lib` passed all 21 library tests.
`cargo clippy -p dna-chemistry --test kinetics_ode -- -D warnings` passed.
`git diff --check` passed.

## Review Notes

- `rsoda` is an HTTPS Git dev dependency pinned to the registered full commit;
  the lockfile resolves the same commit and contains no local path.
- The test reads option3 reaction ID 2 and asserts its reactants, products, and
  validated `2.50e10 M^-1 s^-1` rate before running 512 fixed-seed histories.
- Each history checks all four one-reaction conservation identities. Both
  reactant and both terminal-product ensemble means are included in the 1% check.
- The litre-volume and Avogadro conversion are explicit. SSA and LSODA helpers
  exist only in the integration test; no production scheduler or ODE API changed.

## Blockers / Challenges

None.

## Commits

- 1b0171b test: add LSODA well-mixed kinetics oracle (ticket 03)

## Time Spent

3m (ticket-start.js to ticket-complete.js)
