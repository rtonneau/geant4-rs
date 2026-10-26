# Crate ownership regression evidence

The S01 change moves species ownership from `dna-core` to `dna-chemistry`.
Package renaming and optional chemistry belong to later sessions. This snapshot
checks behavior across the ownership move; it does not extend a model's scientific
validation range or replace the maintained Geant4 reference gates.

## Species ownership

The public species interface is now
`dna_chemistry::{Species, SpeciesInstance, SPECIES_CSV_HEADER}`. Its implementation
and existing unit tests live in the private `species_types` module. The property
table remains at `dna_chemistry::species::SpeciesTable`. This is an import-path
break: `dna_core::species` is removed without a compatibility re-export.

Core retains the shared, water-specific `physchem` handoff records and
`penetration` law needed by both stages. Core has no chemistry dependency, and
physics and chemistry remain independent dependencies of simulation. Event fields,
shell/level numbering, placement and energy-deposit origins, species names and CSV
serialization, creation order, RNG draws and 1 ps initialization are unchanged.
The complete species implementation moves without algorithm changes; the new
`species_interface` integration test checks the public root exports.

## Exact snapshot contract

`crates/dna-sim/examples/crate_split_snapshot.rs` emits
`geant4-rs/crate-split-snapshot/v1` for `--physics` and `--physchem`.
Both use seed **20261026**, history IDs **3–10 inclusive**, primary energies
**100 eV, 1 keV and 10 keV**, one worker and batch size **7**. Input data is
the repository's committed `data/`, verified through `DataDir::verify` before
transport. Missing data fails the capture instead of skipping it.

Each history records every physical outcome and ledger field, ordered secondary
energy/direction/position/generation records, secondary spectra, radial deposits,
solvation statistics and all ordered `PhysChemEvent` fields. Detailed transport
uses `run_detailed`; an additional `run_history_with_observer` pass collects
secondaries and asserts that its complete history result is identical. Production
`run_with_sink` supplies the physical summary and event CSV. The coupled profile
also uses `PhysChemEngine::convert_history` on the independent per-history
`Stage::PhysChem` stream, and `run_with_physchem` for its production summary.
It records every species in creation order, all conversion statistics, the
15-species enum order, and the production species CSV header and rows.

Scientific JSON floating-point values are `f64::to_bits()` hex strings with 16
digits (including signed zero). Conversion occurs in memory before JSON text is
written. CSV remains verbatim text so formatting and header changes are detected
as well as numerical changes. Arrays retain order; no sorting, tolerances,
rounding or scientific-field exclusions are applied.

The checker compares all fields, including SHA-256 identities of `Cargo.lock`
and every file under `data/`, full `rustc -vV`, Cargo version, platform, build
profile and recorded compiler environment. Only **`metadata.revision`** may
differ: each value must be a full commit ID that exists and is an ancestor of the
current checkout's `HEAD`. No other build metadata difference is currently
allowlisted. Package identity and all input/toolchain changes fail. This is a
same-platform/toolchain regression oracle, not a cross-platform bitwise promise.

## Capture and compare

Run from the repository root in PowerShell:

```powershell
$scratch = Join-Path (Get-Location) '.scratch/tests/2026-10-26__crate-domain-ownership'
$env:CARGO_TARGET_DIR = "$scratch/cargo-target"
$env:PYTHONPYCACHEPREFIX = "$scratch/python-cache"
$revision = git rev-parse HEAD
$runId = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ') + '-' + [guid]::NewGuid().ToString('N')
$capture = "target/crate-split-baseline/$revision-$runId"
python scripts/check_crate_split.py --capture $capture --package dna-sim
python scripts/check_crate_split.py --compare "$baseline/physics.json" "$capture/physics.json"
python scripts/check_crate_split.py --compare "$baseline/physchem.json" "$capture/physchem.json"
```

Set `$baseline` to the baseline directory below. Capture creates both profiles
using an exclusive new directory; an existing or partial attempt is never
overwritten. The directory must be an ignored, direct child of
`target/crate-split-baseline/`, named `<full-revision>-<unique-run-id>`.
Build outputs go to the session scratch tree. Keep baseline JSON outside build
trees; the session finish operation may delete build trees.

## Pre-migration baseline, 2026-10-26

Runtime source revision: `2385672da77b70c1495a2ececea39cd2bf4a82cd`.
The working tree included the new snapshot/checker and session bookkeeping;
production runtime source and reference fixtures were unchanged.

Capture directory:

```text
target/crate-split-baseline/2385672da77b70c1495a2ececea39cd2bf4a82cd-20261026T125301Z-ecfc8be110da4d429e15c9d22d10e596/
```

Toolchain: `rustc 1.97.0 (2d8144b78 2026-07-07)`, LLVM `22.1.6`, host
`x86_64-pc-windows-msvc`; `cargo 1.97.0 (c980f4866 2026-06-30)`;
`Windows-11-10.0.26100-SP0`, `AMD64`, dev profile. All recorded compiler
environment overrides were unset. Full toolchain details and all 28 individual
input hashes are embedded in each JSON.

| Evidence | SHA-256 |
|---|---|
| `physics.json` (6,262,999 bytes) | `a98bab95b3552e496985d7c492a263d714d259e8c6715cf30577d4753126a9dc` |
| `physchem.json` (11,320,232 bytes) | `5b16cac5247dccbf283715a416e545011dad22310ff0af8ebe71666c66a007e8` |
| `Cargo.lock` | `76676ccb77ef1500ed2a786c2515e8bc5736a5778a00cf6661263473de17c6f3` |
| `data/MANIFEST.toml` | `746cb24790d4db0bb600aa688f3880aef2fdb095e7acb830888c449ec257f6c5` |
| `data/chemistry/species_option3.toml` | `4d727b7e87b5696889d4b7ad9dc69814cfe65200593f7f1d2fc5a9dbef965f87` |
| `data/chemistry/dissociation_option3.toml` | `ef05673ae1a01a544761b557f8c6732ae59ce4e6467f91b7f6d5872662a82824` |
| `data/chemistry/physchem_option3.toml` | `53a22c2fda9f0d5bdd4209fd9903be09656cc8ead919f692e38e29c1131c10e9` |
| `data/chemistry/reactions_option3.toml` | `8a992acb1f9973f84fa5149528a6599010cd270895fa6eef4682136babf2888d` |
| Entire input map, canonical JSON | `672f51bbc20dad52d82fda6dbca84fd8c0f782bc29f23efcc17550c5bd52ff1b` |

The input-map digest hashes UTF-8 `json.dumps(inputs, sort_keys=True,
separators=(",", ":"))`. Input hashes use exact checkout bytes; files are never
rewritten or normalized by this checker. Copies of the baseline JSON and run
logs are retained under
`.scratch/tests/2026-10-26__crate-domain-ownership/evidence/`.

## Required verification

```text
python -m unittest discover -s scripts -p test_check_crate_split.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p dna-sim --test geant4_reference --locked -- --ignored
cargo test -p dna-sim --test slowing_down_reference --locked -- --ignored
cargo test -p dna-sim --test physchem_reference --locked m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact
```

The checker mutation tests cover equal snapshots, validated revision changes,
one-bit scientific differences, species order, CSV headers, input hashes,
toolchain changes, invalid revisions and unlisted metadata. The reference
selectors use existing committed M2/M3/M6 evidence; no Geant4 generator runs.
Stop before migration if any starting gate unexpectedly fails.

Baseline results on 2026-10-26 (all commands exited 0):

| Check | Result | Evidence log under the session `evidence/` directory |
|---|---|---|
| Python checker | 10 tests passed | `checker.log` |
| Formatting | Passed | `fmt.log` |
| Workspace Clippy | Passed with warnings denied | `clippy.log` |
| Workspace tests, including doctests | 394 passed, 19 intentionally ignored, 0 failed | `workspace.log` |
| M2 reference selector | 4 passed, 11 filtered | `m2.log` |
| M3 reference selector | 2 passed, 8 filtered | `m3.log` |
| Exact M6 prechem selector | 1 passed, 8 filtered | `m6.log` |
| Both snapshot captures | Complete, 24 histories per profile | `capture.log`, `capture-audit.log` |
| Independent repeat capture | Both files byte-identical; both comparisons passed | `repeat-capture.log`, `compare-physics.log`, `compare-physchem.log` |

The repeat used a fresh directory named
`2385672da77b70c1495a2ececea39cd2bf4a82cd-repeat-381fdd15b9c54342909f6e0268a06db8`
under the same baseline parent. `baseline-path.txt` and `repeat-path.txt` retain
the relative capture paths. The audit also confirmed that both profiles contain
the same physical history fields and physical summary, with no decimal JSON
floating-point fields left in either snapshot. The 19 ignored workspace cases
were not treated as passing gates: the seven maintained cases were explicitly
selected by the three reference commands above.

## Species-move regression evidence, 2026-10-26

The post-move capture used revision
`0e5a50ba8a093ac133c6cf9ba693f0968d5e6177` with the species ownership changes
in the working tree. Its new capture directory is:

```text
target/crate-split-baseline/0e5a50ba8a093ac133c6cf9ba693f0968d5e6177-ticket-02-20261026T131313Z-70cdae51f7184494af294bbaf8aaf0b1/
```

| Evidence | SHA-256 |
|---|---|
| `physics.json` | `a9e0f37bc2263f2dfec9398aab1d311cea8ae9aa9b81afdf5b3f0d595c480d6f` |
| `physchem.json` | `b9b00c397f8df1ef2b390fd8dd880089f97ad38c493944b40b8ac58000742ee8` |

Both comparisons against the pre-migration baseline passed with exact scientific,
input and toolchain identity; only the validated revision metadata differs. Copies
are retained as `ticket-02-physics.json` and `ticket-02-physchem.json` in the session
`evidence/` directory, alongside `ticket-02-capture-path.txt`,
`ticket-02-capture.log` and `ticket-02-compare-{physics,physchem}.log`.

The public-interface test first failed to compile because the chemistry root did
not export the types, then passed all three cases after the move. The red/green
logs are `ticket-02-interface-{red,green}.log`. The source audit confirms the moved
implementation and unit tests are unchanged apart from imports and module prose,
and no Rust source references `dna_core::species`. Dependency trees confirm no
core-to-stage or cross-stage edge (`ticket-02-source-audit.log` and
`ticket-02-tree-{core,physics,chemistry}.log`).

All required checks exited 0: formatting, workspace Clippy with warnings denied,
workspace tests (397 passed, 19 intentionally ignored), M2 (4 selected tests),
M3 (2 selected tests) and the exact M6 prechem selector (1 selected test). Logs are
`ticket-02-{fmt,clippy,workspace,m2,m3,m6}.log` in the same evidence directory.
