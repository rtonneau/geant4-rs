# Ticket 02: rename-five-crates

**Status:** ✅ Done

## Local Test Result

All commands in the ticket Verification Step exited 0 in an independent inline rerun after review. `cargo metadata --format-version 1 --no-deps --locked` resolved exactly the five `g4-*` workspace packages. Standalone `g4-physics` and `g4-chemistry` checks, `cargo fmt --all -- --check`, workspace Clippy with warnings denied, and `cargo test --workspace --locked` passed (397 tests passed, 19 intentionally ignored). The selected M2, M3 and M6 commands ran 4, 2 and 1 tests respectively, all passing. The Python checker suite passed 18 tests. Logs are `.scratch/tests/2026-10-26__g4-crate-namespace-migration/evidence/ticket02-inline-*.log`.

After code commit `e33bee3397b674be34fa63edf6e179adb0683670`, `python scripts/check_crate_split.py --capture target/crate-split-baseline/e33bee3397b674be34fa63edf6e179adb0683670-20261026T153253Z-311453000b934e688e73fb60439c3b99 --package g4-sim` created both profiles successfully. Their SHA-256 hashes are `fe376a10c1077ce366807c6de2f973fe08a107bff3633d1b6b8c32620a1d5f74` (physics) and `e831dfe49fb439fca69c2c140b05a4dd7c2e542ffbb7867d47647dee93c5f305` (physchem). Each `--compare` against the corresponding `.scratch/tests/2026-10-26__crate-domain-ownership/evidence/baseline-*.json` with `--allow-crate-rename` passed: exact scientific, data and toolchain identity with verified local crate rename.

## Review Notes

Reviewed the staged rename and `Cargo.lock` changes before the code commit. The five directories, package identities, path dependencies, dev profile keys, Rust imports, active scripts, gate registry and current guides use `g4-*`; intentional `dna-sim` schema IDs, `data/dna`, `G4RS_DATA_DIR`, CSV/species names and all scientific data/reference files are unchanged. The delegate's source audit reports crate contents differ only by planned names and the lockfile only by five local identities (`evidence/ticket02-audit.log`). Inline review restored `docs/08-roadmap.md` and `docs/09-open-questions-and-risks.md` to preserve historical records and corrected the current Geant4 build note to locate `Species` in `g4-chemistry`. `git diff --cached --check` passed before commit. The code commit was made before capture because the checker validates the candidate lockfile at its recorded commit; GPS records that one commit after the snapshot gates.

## Blockers / Challenges

The candidate snapshot required a committed renamed lockfile. The single ticket code commit preceded capture; GPS ticket completion follows the passing post-commit comparison. No remaining blocker.

## Commits

- e33bee3 refactor: rename five crates to g4 namespace (ticket 02)

## Time Spent

19m (ticket-start.js to ticket-complete.js)
