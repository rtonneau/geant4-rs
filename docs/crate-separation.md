# Crate ownership regression evidence

Species ownership moved from core to chemistry in S01; S02 renames the five
workspace packages to `g4-*`. S03 makes simulation chemistry optional, enabled
by default.
This snapshot checks behavior across both migrations; it does not extend a
model's scientific validation range or replace the maintained Geant4 reference gates.

## Import migration

| Previous package / Rust import | Current package / Rust import |
|---|---|
| `dna-core` / `dna_core` | `g4-core` / `g4_core` |
| `dna-data` / `dna_data` | `g4-data` / `g4_data` |
| `dna-physics` / `dna_physics` | `g4-physics` / `g4_physics` |
| `dna-chemistry` / `dna_chemistry` | `g4-chemistry` / `g4_chemistry` |
| `dna-sim` / `dna_sim` | `g4-sim` / `g4_sim` |

Directories and path dependencies follow the package names. There are no
compatibility re-exports. The persisted schema IDs `geant4-rs/dna-sim/run/v2`
and `geant4-rs/dna-sim/run/v3`, the scientific `data/dna` directory and
`G4RS_DATA_DIR` remain intentional, unchanged names. CSV headers, species names
and scientific fixture bytes are unchanged. Historical evidence below retains
its original names and revisions. `data/chemistry/README.md` is also retained
byte-for-byte because it belongs to the hashed input map; interpret its old
crate paths using the table above.

Commit the coherent rename before capturing: rename comparison verifies the
candidate lockfile against its recorded commit. Compare S01 baselines with
`--allow-crate-rename`; comparisons between two new captures use strict mode.

## Supported simulation profiles (S03)

| Build | Available behavior |
|---|---|
| Default (`chemistry` enabled) | Physical orchestration, raw handoff CSV, coupled physicochemistry and spatial diagnostics |
| `--no-default-features` | Physical orchestration and raw handoff CSV; no reachable `g4-chemistry` dependency |
| `--no-default-features --features chemistry` | Same APIs as the default build |

Library consumers select the physical profile with `default-features = false`
on their `g4-sim` dependency. In both profiles,
`g4_sim::run::{run, run_detailed, run_summary, run_summary_with_sink,
run_with_sink, RunOutput, PhysChemSink, CsvPhysChemWriter}` stay available.
`run_with_physchem`, `run_physchem_summary`,
`run_physchem_summary_with_diagnostics` and `PhysChemRunOutput` remain at their
existing `g4_sim::run` paths when `chemistry` is enabled. Chemistry-only scoring,
spatial diagnostics and associated error variants require that feature.

The historical `dna_core::species::*` import becomes
`g4_chemistry::{Species, SpeciesInstance, SPECIES_CSV_HEADER}`. Raw
`g4_core::physchem` records do not require chemistry. `g4-bio` is future-only:
it may consume physical events and chemical results; no biology crate or API
is implemented here.

`python scripts/check_crate_split.py --check-graph` separately resolves each
root's normal/build tree using `cargo tree -p ... --edges normal,build`.
It traverses reachable package identities, rejecting data/physics/chemistry/sim
below core, chemistry/sim below physics, data/physics/sim below chemistry,
and chemistry below no-default simulation. Disconnected workspace members
are harmless. Dev dependencies are excluded from this graph contract;
the simulation manifest has no chemistry dev-dependency and separate
package-scoped `--all-targets` checks/tests/Clippy exercise that build surface.
Workspace feature unification is not evidence for the no-default profile.

## Species ownership

The public species interface is now
`g4_chemistry::{Species, SpeciesInstance, SPECIES_CSV_HEADER}`. Its implementation
and existing unit tests live in the private `species_types` module. The property
table remains at `g4_chemistry::species::SpeciesTable`. This is an import-path
break: the former `dna_core::species` interface was removed without a
compatibility re-export.

Core retains the shared, water-specific `physchem` handoff records and
`penetration` law needed by both stages. Core has no chemistry dependency, and
physics and chemistry remain independent dependencies of simulation. Event fields,
shell/level numbering, placement and energy-deposit origins, species names and CSV
serialization, creation order, RNG draws and 1 ps initialization are unchanged.
The complete species implementation moves without algorithm changes; the new
`species_interface` integration test checks the public root exports.

## Exact snapshot contract

`crates/g4-sim/examples/crate_split_snapshot.rs` emits
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
current checkout's `HEAD`. Strict mode allows no other build metadata difference.
Rename mode additionally accepts only `dna-sim` to `g4-sim` package identity and the corresponding five
local Cargo.lock package/dependency identities, verified against committed
revisions. Third-party versions, sources, checksums and all data/toolchain
changes still fail. This is a same-platform/toolchain regression oracle, not a cross-platform bitwise promise.

## Capture and compare

Run from the repository root in PowerShell:

```powershell
$scratch = Join-Path (Get-Location) '.scratch/tests/2026-10-26__g4-crate-namespace-migration'
$env:CARGO_TARGET_DIR = "$scratch/cargo-target"
$env:PYTHONPYCACHEPREFIX = "$scratch/python-cache"
$revision = git rev-parse HEAD
$runId = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ') + '-' + [guid]::NewGuid().ToString('N')
$capture = "target/crate-split-baseline/$revision-$runId"
python scripts/check_crate_split.py --capture $capture --package g4-sim
python scripts/check_crate_split.py --compare "$baseline/physics.json" "$capture/physics.json" --allow-crate-rename
python scripts/check_crate_split.py --compare "$baseline/physchem.json" "$capture/physchem.json" --allow-crate-rename
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
cargo test -p g4-sim --test geant4_reference --locked -- --ignored
cargo test -p g4-sim --test slowing_down_reference --locked -- --ignored
cargo test -p g4-sim --test physchem_reference --locked m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact
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

## S03 physics-only boundary evidence, 2026-10-26

Runtime revision: `fa9da5474629905c5df15b5f2df62b68ea293882` (ticket 01
committed); checker, CI and documentation changes were in the working tree.
The pre-S01 runtime revision remains
`2385672da77b70c1495a2ececea39cd2bf4a82cd`. No physics algorithms, reference
fixtures or data files changed in this ticket. No Geant4 generator ran.
The recorded Windows/toolchain identity is the same as the pre-S01 baseline.
Linux CI has not been executed locally.

All S03 artifacts are under
`.scratch/tests/2026-10-26__physics-only-simulation-build/evidence/` (`$evidence`
below); Cargo output is in its sibling `cargo-target/` and Python bytecode in
`python-cache/`. Logs and captures are outside the disposable build tree.
`ticket02-matrix.json` records exact commands/exit codes; corresponding output
is retained in `ticket02-<check>.log`.

The snapshot checker also accepts exclusive, ignored capture directories
immediately below this session's evidence directory, using the same
`<full-revision>-<unique-run-id>` naming rule as the historical target location.
This keeps all new run artifacts in session scratch. Old capture paths and
commands above remain valid. Default capture writes both profiles;
`--no-default-features` writes only `physics.json`. Existing directories,
including incomplete attempts, are never reused.

Executed capture directories beneath `$evidence`:

- Default: `fa9da5474629905c5df15b5f2df62b68ea293882-default-00617e4cc2f149f6b718a81ba8bb66d9`
- No-default: `fa9da5474629905c5df15b5f2df62b68ea293882-no-default-4fbc62be7ce54eb9aab63dfec6178479`

With `$default` and `$physical` set to those full paths, the executed commands
were (all exited 0):

```powershell
python scripts/check_crate_split.py --capture $default --package g4-sim
python scripts/check_crate_split.py --capture $physical --package g4-sim --no-default-features
python scripts/check_crate_split.py --compare "$default/physics.json" "$physical/physics.json"
$old = '.scratch/tests/2026-10-26__crate-domain-ownership/evidence'
python scripts/check_crate_split.py --compare "$old/baseline-physics.json" "$default/physics.json" --allow-crate-rename
python scripts/check_crate_split.py --compare "$old/baseline-physchem.json" "$default/physchem.json" --allow-crate-rename
```

The preserved pre-S01 evidence copies were used because the old target capture
directory is no longer present. Both profiles contain 24 histories (eight per
energy). Default and no-default physics are byte-identical, including input
and toolchain metadata. Both default profiles match the pre-S01 scientific
payload exactly under the existing verified rename comparator. No additional
comparison exclusions were introduced. `ticket02-snapshots.log` contains the
commands and verdicts; `ticket02-snapshot-summary.json` and each capture retain
all 28 input hashes and full toolchain metadata.

| Evidence | SHA-256 |
|---|---|
| Both physics captures | `9d4540a118055be9e15ecdda0b6e28ffa8c2f93f1f61f9d2a2f5e29249d5f0ce` |
| Default physchem capture | `0b73c44d5952daa5d2b2031cf8a553ac03138bf6b062c4970433a5f4ead13274` |
| Current `Cargo.lock` | `ed521c73735f0bd4767e2763205c32b2cb4102e3c6da0216abb97f8b2944b4c1` |
| `data/MANIFEST.toml` | `746cb24790d4db0bb600aa688f3880aef2fdb095e7acb830888c449ec257f6c5` |
| Entire input map, canonical JSON | `02c927624685d3b3a992ee2c40152bb2183e8f9a14f39b9aeb59e7c04c9dc12d` |

The four chemistry input hashes remain those in the pre-S01 table. The only
input-map change from pre-S01 is the already-verified local crate rename in
`Cargo.lock`.

### Executed final verification matrix

All commands below exited 0 on the recorded Windows toolchain.

| Check/log suffix | Command | Executed result |
|---|---|---|
| checker | `python -m unittest discover -s scripts -p test_check_crate_split.py` | 24 Python tests passed |
| graph | `python scripts/check_crate_split.py --check-graph` | 5 roots/profiles passed; reachable counts 31/33/32/45/44 |
| fmt | `cargo fmt --all -- --check` | Passed |
| clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| workspace | `cargo test --workspace --locked` | 399 passed; 19 intentionally ignored |
| m2 | `cargo test -p g4-sim --test geant4_reference --locked -- --ignored` | 4 passed; 0 intentionally ignored |
| m3 | `cargo test -p g4-sim --test slowing_down_reference --locked -- --ignored` | 2 passed; 0 intentionally ignored |
| m6 | `cargo test -p g4-sim --test physchem_reference --locked m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact` | 1 passed; 0 intentionally ignored |
| physics-check | `cargo check -p g4-physics --all-targets --locked` | Passed |
| chemistry-check | `cargo check -p g4-chemistry --all-targets --locked` | Passed |
| no-default-check | `cargo check -p g4-sim --no-default-features --all-targets --locked` | Passed |
| no-default-test | `cargo test -p g4-sim --no-default-features --all-targets --locked` | 35 passed; 7 intentionally ignored |
| no-default-clippy | `cargo clippy -p g4-sim --no-default-features --all-targets --locked -- -D warnings` | Passed |
| chemistry-feature | `cargo test -p g4-sim --test chemistry_feature --locked` | 1 passed; 0 intentionally ignored |

The no-default suite executes `physics_only`, determinism, physical sanity and
the ordinary physical reference-test families; seven ignored cases are not
counted as passes. The default workspace suite executes the chemistry families,
and the separate `chemistry_feature` consumer executes one test. Selected ignored
reference comparisons are explicitly counted above, using the maintained M2/M3/M6
selection in `validation/gates.json`. CI retains its existing required-data gate
and adds package-scoped no-default check/test/Clippy plus checker unit and graph
checks. The statistical selection remains an explicit local gate, as before.

The checker tests cover direct/transitive forbidden paths, disconnected chemistry,
both simulation profiles, malformed trees and missing nodes, plus exclusive
captures with correct profile selection. The prior exact-comparison mutation
tests remain active. The final checker rerun is retained as
`ticket02-checker-final.log`. No science tolerance or baseline bytes were changed.
The final graph rerun also passed with CI's `CARGO_TERM_COLOR=always` setting
(`ticket02-graph-final.log`); the parser's Cargo invocation explicitly disables
color so ANSI escapes cannot corrupt depth-prefixed identities.
