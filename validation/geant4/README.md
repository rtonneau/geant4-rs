# Geant4 reference data for the M2 and M3 validation

## RR-007 paired source mapping (approved 2026-11-09)

This prospective finite-grid diagnostic does not pass the historical independent
SYN-2 moment comparison. The one-million cap and resource budgets are unchanged.
It validates only the numerical source mapping for supplied primitive inputs;
it does not validate RNG/distribution quality, a whole scheduler, or physical
correctness. Independent analytic and scheduler gates remain separate.

Frozen grid: channels [1,2,11], r/a=[1,3], dt_ps=[0,1,1000], orientations
[x,z,translated_x], with translated origin (10a,-7a,3a): 54 settings.
Add channel2 parent-only edges: each fixed parent [0,1], dt_ps=[0,1,1000],
coincident=[false,true] (12 settings). Add mobile-coincident errors for each
channel/duration (nine settings). Each setting has 16 common-input tapes,
**1200 ordered rows**, including 144 expected source coincidence errors.
No centroid exists for channel11 or parent-only edge rows; write `NA`.

For tape t=0..15, let j=(t+seed%16)%16. The independent implementations use
normal primitives zx=(j%7-3)*0.75, zy=((3j+1)%7-3)*0.5,
zz=((5j+2)%7-3)*1.25; and uniform primitives
u_phi=(j%8+0.5)/8, u_theta=((3j+1)%8+0.5)/8,
u_product=((5j+2)%8+0.5)/8. Production seed=20261109071.
These are deterministic test inputs, not a claim of Gaussian/uniform sampling.
The primitive tape id, all six values and the observed semantic consumption
mask are exported. Mask bits are zx=1,zy=2,zz=4,phi=8,theta=16,product=32.
Positive-duration mobile placement consumes31, plus32 iff products exist;
zero-duration mobile placement consumes only the optional32; immobile
parent-only and coincident error rows consume0. Unknown/repeated labels fail.

The opt-in C++ adapter includes unmodified 11.4.1 G4DNAMakeReaction.cc,
SHA-256 `8ae87604437c1f7d57f410961c01d51adf9215e5a35f6702247ad6592a1f0559`,
under class name G4rsSynMakeReaction. Interception uses source-line semantic
labels: normals188/189/190, phi192, theta195, product107. This avoids C++
argument-evaluation-order dependence. Configure records the source root and
hash, rejects mismatches, and records adapter compile definitions in provenance.
The linked original library is not modified. Rust derives physical inputs from
embedded data and this manifest; only the agreed raw primitives are shared.

All8064 parent/product coordinates must satisfy
`abs(delta)<=1e-12 nm+1e-12*abs(source)`; each base pair distance is also checked
against the source formula (r at dt0, a at positive dt). All branch masks,
products, parent-removal and expected source-error outcomes must match exactly.
No stochastic interval, Bonferroni correction or1% moment PASS is inferred
from this exhaustive grid. The independent1% analytic criteria remain in force.

Build the optional `g4rs_syn_ref` target with
`-DG4RS_SYN_SOURCE_ROOT=<verified read-only 11.4.1 source root>`, using the
normative install and other prerequisites in BUILD_NOTES. Compile definition
`G4RS_SYN_MAKE_SOURCE` names the pinned .cc; all renaming/draw macro definitions
are local to SynPaired.cc and do not affect `g4rs_ref` or its existing modes.
Run: `g4rs_syn_ref paired ../../reference/syn_paired_1141.csv 20261109071 16`.
One-tape smoke runs use new scratch files and are not reference evidence.
The producer and consumer enforce <1,000,000 bytes per fixture.

Schema `syn_paired_v1` is the exact 35-column header in SynPaired.cc and its
strict consumer: scenario identity (kind/channel/ratio/duration/orientation/
fixed parent/tape), six raw primitives, consumption mask/status, radius and
two diffusivities, six input parent coordinates, six output parent coordinates,
product count, common centroid and parent-removal flag. Lengths are nm,
diffusivities nm^2/ps and duration ps. Missing outputs use `NA`; coincidence
is `coincident_error`, never a no-reaction observation. The two leading metadata
lines and exact final marker `# complete: true; seed: 20261109071; tapes: 16; rows: 1200`
are required, together with exact ordered coverage. Source product species/order
and colocation are checked in the producer before writing a row.

Consumer: `cargo test --release -p g4-chemistry --test geant4_syn_paired syn_paired_mapping_matches_geant4 -- --ignored --exact --nocapture`.
The provenance filename is `provenance_syn_paired_1141.toml`. Required keys:
`geant4_version="11.4.1"`, `physics_constructor="G4EmDNAPhysics_option2"`,
`chemistry_constructor="G4EmDNAChemistry_option3"`, `timestep_model="IRT_syn"`,
`mode="paired"`, `schema="syn_paired_v1"`, `claim="finite-input mapping only"`,
`source_sha256` equal to the pinned hash above, `g4emlow_version="8.8"`,
`seed=20261109071`, `tapes_per_setting=16`, `completion_rows=1200`, canonical-LF
`csv_sha256`, `generator_commit`, `generator_dirty`, named
`generator_source_sha256` for SynPaired.cc and CMakeLists.txt, `compiler`,
`platform`, `date`, `actual_command`, `compile_definitions` (both the configured
source macro and local class/draw macros), `g4emlow_md5`, and the registered
`command` above. Record dirty state truthfully; never overwrite RR-006 data.

## RR-006: isolated IRT-syn pair diagnostics

Run result (2026-11-08): the dedicated authorized runner produced two complete
11.4.1 replicates at each registered look. The 100000 and 200000 looks were
precision-INCONCLUSIVE; both are preserved. At 1000000, all 448 stochastic
comparisons and 16 deterministic diagnostic endpoints passed. This closes
RR-006 source diagnostics only. It does not resolve source overlap clocks or
pass full SYN-1/2. Data committed in 9607036827be6954fb5423f294275d36c383661c; provenance in f3c169f.

The [SYN-2 precision feasibility decision](SYN2_FEASIBILITY.md) was resolved
prospectively on 2026-11-09 by the paired mapping diagnostic above. The old
independent-moment campaign was not run or passed. Existing source evidence
is unchanged.

New mode `g4rs_ref syn_pair_matrix OUTPUT SEED SAMPLES` exclusively creates
OUTPUT and requires library version 1141. Physics is option2, chemistry option3,
with IRT_syn selected before initialization. No existing mode or fixture changes.
The mode calls the actual `G4DiffusionControlledReactionModel::GetTimeToEncounter`
for r/a >= 1. At r/a=0.5 it evaluates only the stepper's initial probability
branch: rejection is `unresolved_overlap`, not an invented no-event clock.
Post-start overlap is a separate deterministic branch diagnostic; it does not
claim to execute a scheduler or resolve the inherited-candidate anomaly.

Frozen manifest: channels [1,2,8,9,11,17,42,51], effective-radius ratios
[0.5,1,1.5,3], tau=D*t/a^2=[0.01,0.1,1,10,100]. Each initial setting has
five CDF rows, eventual `no_reaction`, and `unresolved_overlap`; each channel
adds one post-start ratio-0.5 unresolved-overlap row: **232 rows per replicate**.
Geant4 seeds are 20261108061/20261108062; independent Rust seeds are
20261108063/20261108064. Source coefficients come from the initialized table
and are exported as a_nm, D_nm2_per_ps and contact_probability; the strict
consumer checks them against the embedded data to 1e-12 relative before
comparing any observations. No energy or scheduler horizon is used.

Schema `syn_pair_v1`: `reaction_id,context,r_over_a,a_nm,D_nm2_per_ps,contact_probability,endpoint,tau,samples,count`.
The two metadata lines, exact header, ordered 232 rows and final complete marker
are mandatory. Tau=-1 labels non-CDF rows, not a negative time sample.
Nonfinite or unexpected source returns abort; partial files are not evidence.
Counts include every draw; CDFs are unconditional and never renormalized after
an unresolved outcome. Exact post-start diagnostics are checked deterministically.

Only source-diagnostic equivalence is assessed here. This does **not** close
SYN-1's full independent analytic gate, SYN-2 placement/diffusion, the scheduler
trace, or SYN-3 unary extension. L1 tests independently integrate the type-1
first-passage/delay convolution but use a regression tolerance, not SYN acceptance.
Acceptance follows docs/07: alpha=0.001, three looks, family m=464 (both
replicates), each source/Rust Clopper-Pearson interval gets alpha/(3*m*2).
Subtract bounds; PASS requires the entire difference interval strictly inside
[-0.01,0.01], FAIL entirely outside, otherwise INCONCLUSIVE. Report all counts
and intervals. Samples per setting are 100000, then 200000, then 1000000 only
for insufficient precision; new filenames preserve all earlier evidence.

Consumer: `cargo test --release -p g4-chemistry --test geant4_syn_pair syn_pair_source_diagnostics_match_geant4 -- --ignored --exact --nocapture`.
`SYN_PAIR_SAMPLES` chooses a registered look (default 100000).
Files are `syn_pair_1141_r{1,2}_n{SAMPLES}.csv` and corresponding
`provenance_syn_pair_1141_r{1,2}_n{SAMPLES}.toml` in `validation/reference`.
Provenance requires `geant4_version="11.4.1"`, `timestep_model="IRT_syn"`,
the named physics/chemistry constructors, `mode="syn_pair_matrix"`,
`schema="syn_pair_v1"`, `g4emlow_version="8.8"`, `seed`,
`samples_per_setting`, `completion_rows=232`, canonical-LF `csv_sha256`,
`generator_commit`, `generator_dirty`, `compiler`, `platform`, `date`,
`g4emlow_md5`, `geant4_source_revision`, `source_sha256` (named hashes of
the contract's reviewed source files), `generator_source_sha256` (named hashes
of both g4rs_ref.cc and SynKernels.hh), `actual_command`, and `command`.
The latter is the path-independent registered command from RUN_REQUESTS;
actual_command records scratch execution if its executable/output paths differ.
The existing 2-hour/8-GiB invocation and fixture-size limits apply.

### Frozen independent analytic pair gate (PASS at second look)

Exact test: `cargo test --release -p g4-chemistry --test geant4_syn_pair syn_pair_matches_independent_analytic_law -- --ignored --exact --nocapture`.
Seeds 20261108065/20261108066; the same eight channels, four separations,
five tau values, no-event and explicit unresolved-overlap endpoints as RR-006,
plus each post-start overlap identity. Family m=464, alpha=0.001, three looks
at 100000/200000/1000000 draws per setting, selected by `SYN_ANALYTIC_SAMPLES`.
Single-sample Clopper-Pearson intervals use alpha/(3*m); analytic probability
uncertainty expands each difference interval by 1e-8 before the same +/-0.01
equivalence decision. All counts and bounds are reported. No source CSV is
used. The numerical oracle integrates erfcx independently of the sampler and
checks its closed-form type-1 CDF against independent first-passage/delay
convolution quadrature at every separated type-1 grid point. Quadrature
refinement is a unit test, not the statistical acceptance test. This gate is
executed on 2026-11-09: 100000 look had 426 PASS and 22 INCONCLUSIVE endpoints, no FAIL; 200000 look passed all 448 stochastic comparisons and 16 post-start identities. Logs: session scratch 02-analytic-{100k,200k}.log. Inherited overlap clock state remains outside this isolated law acceptance.

## IRT-syn reference contract (2026-11-08)

This prospective contract accompanies [ADR 0004](../../.work/adr/0004-irt-syn-production-contract.md)
and the [SYN gates](../../docs/07-validation-strategy.md#irt-syn-prospective-gates-2026-11-08).
It does not change existing modes, data, provenance or acceptance criteria.
No IRT-syn comparison has passed merely because this contract is recorded.

### Version and source-review record

Geant4 **11.4.1 remains the normative baseline**. On 2026-11-08 both versioned
source trees under `${DEV_DIR}/GEANT4/geant4-v<version>-source/geant4-v<version>`
were read directly, following their junctions. Their `source/global/management/include/G4Version.hh`
identifies 1141 / `geant4-11-04-patch-01` and 1143 / `geant4-11-04-patch-03`.
They are unpacked trees without git metadata: no commit identity is inferred.
`${G4_ROOT}` currently names an 11.4.3 installation. This is not authorization
to relabel it 11.4.1 or migrate existing fixtures. The absent local
`.claude/.claude-project.json` and user `CONFIG_RESOLUTION.md` were not invented;
the environment-based versioned source layout above was inspected directly.

The following reviewed files are byte-identical in the two local trees.
`D` denotes `source/processes/electromagnetic/dna`, `P` denotes
`source/physics_lists/constructors/electromagnetic/src`, and `E` denotes
`examples/extended/medical/dna`. SHA-256 pins the inspected bytes in **both**
versions; it does not certify an entire distribution or binary.

| Path | SHA-256 (both trees) |
|---|---|
| `D/models/src/G4DNAIndependentReactionTimeModel.cc` | `250f2fb7d3fe458fecd7d48aa7b8ef6405a94a89f8c2cb1c5e8a89bd21f96d16` |
| `D/models/src/G4DNAIndependentReactionTimeStepper.cc` | `abb9670a93d21ac656d489f63a29c000e8d44e9c6516cfc1e0b1e56e9ee5dbcc` |
| `D/models/src/G4DiffusionControlledReactionModel.cc` | `a05e2d3ebd98a792f10f064a9eceb4a2af2fe3bbe40d76efc5bcb6984d759e3e` |
| `D/models/src/G4DNAMakeReaction.cc` | `8ae87604437c1f7d57f410961c01d51adf9215e5a35f6702247ad6592a1f0559` |
| `D/utils/src/G4DNAMolecularReactionTable.cc` | `6d0b42c0add6298f072469c14038c136d50170b9c35c413cfd32dcfe9d710e6b` |
| `D/utils/src/G4IRTUtils.cc` | `bcb3fc7fea2f0c8386986f14b604a923598d341988549243ea683552d9acb340` |
| `D/processes/src/G4DNABrownianTransportation.cc` | `4039fc87550686d4b14ca592bb8a38990024cd78434bf259ff0375ea49735123` |
| `D/management/src/G4Scheduler.cc` | `4279b4129817870efd92818f3448bb0a1bd870fd90f13b48cf80a85fc4112c05` |
| `D/management/src/G4ITModelProcessor.cc` | `a5ce7ca23be8966db0bdec3d68ced96162d0c7e97d4950f56c6d3f557d2c0701` |
| `P/G4EmDNAChemistry_option3.cc` | `b71251a01606632a56647fcaf3d3023e8c95ab8ca38f2aed9a9838c55b912855` |
| `D/processes/src/G4DNASecondOrderReaction.cc` | `57ba6639146498820b33d97c3918ce472bba58e81b85a636de337e7f32ea6b3b` |
| `D/processes/src/G4DNAScavengerProcess.cc` | `533da4fc6d4fc8e0431e3ea5e660bbb138c77e26cac1e6bde46ebf8eb72d645d` |
| `E/chem6/src/PhysicsList.cc` | `392980c3c339b64889877081dbb0899d9a935141d583d0204662ab3b0358af7f` |
| `E/UHDR/src/EmDNAChemistry.cc` | `9d604ef382514125c46c6094d79b4df75755979a17acd4cf5ae6151e26a161a0` |

Additional inspected context: `E/chem6/beam.in`, `E/UHDR/src/PhysicsList.cc`,
`P/G4EmDNAChemistry_option2.cc`, `P/G4ChemDissociationChannels_option1.cc`,
and the inline `GetKact` definition in `D/utils/include/G4IRTUtils.hh`.
The normative reference configuration is `G4EmDNAPhysics_option2` plus
`G4EmDNAChemistry_option3` with `/process/chem/TimeStepModel IRT_syn` selected
**before initialization**. Chemistry option2 is not a synonym for IRT-syn.
The chem6 default macro selects IRT, so an unchanged chem6 run is not a SYN reference.
UHDR demonstrates IRT-syn registration with additional scavenger processes and
bounded/periodic transport; its complete physics setup is not the unbounded-water baseline.

New 11.4.3 runs may supply explicitly versioned supplemental evidence. Closing
a normative 11.4.1 gate requires 11.4.1 evidence, or a separately approved,
documented baseline amendment with dependency/data review and new filenames.
Matching these source files alone does not prove equivalence of whole simulations.
Every new run records the actual library version, constructor, model selection,
data versions/checksums, source hashes, generator revision/dirty state, compiler,
platform, exact command, independent seed and complete statistics. Required
runs for this approved session are user-authorized through the dedicated
reference-run subagent after their RUN_REQUESTS entry and consumer exist;
the historical user-run-only wording elsewhere is not an active permission blocker.

### Executable semantics and prospective Rust boundary

1. **Synchronization.** `IndependentReactionTimeModel::Initialize` wires the
   independent-time stepper, diffusion-controlled sampler and `G4DNAMakeReaction`.
   `Stepper::Prepare` clears candidates and saved positions after an interaction
   step; collision steps retain pending candidates, and products add candidates.
   `CalculateStep` visits each unordered mobile pair once; event times are
   absolute `globalTime + elapsed`. `Scheduler::Stepping` chooses the smaller
   process/pair step, gives process steps priority at equality, transports before
   committing reactions, and clears candidates when capped at its stop time.
   Selected pairs restore their saved sampling positions before placement.
   Therefore synchronizing is not permission to redraw every pair at every
   observation, injection or checkpoint. Rust must preserve pending clocks,
   saved positions and invalidation state or prove the replacement conditional law.
   Exact common times and canonical identity-based ties are Rust requirements;
   Geant4's loose synchronization check and container tie order are not proofs
   of worker safety. No reviewed source implements our region ownership or
   coordinator arbitration: validate those against the serial state machine.
2. **Diffusion.** In homogeneous free water, Brownian `ComputeStep` draws three
   independent normal increments with standard deviation `sqrt(2 D dt)`.
   Source species coefficients and constructor overrides are authoritative;
   no temperature rescaling or forces are added. Zero time or zero diffusivity
   must preserve position. Reaction placement below is a separate conditional
   operation, not a second unconditional diffusion of selected reactants.
3. **Pairs.** With separation `r >= a > 0`, `D = D1 + D2 > 0`, the sampler
   uses `W = a/r`, uniform `U`, and, if `U < W`,
   `t1 = (r-a)^2 / (4 D erfcInv(U/W)^2)`; otherwise no reaction.
   Type 0 returns `t1`. Type 1 sets `kdif = 4 pi D a NA` (halved for
   identical configurations), `kact = kdif*kobs/(kdif-kobs)`, accepts with
   probability `kact/(kact+kdif)`, and adds
   `t2 = (X*a*kdif/(kact+kdif))^2/D`, where `X = -ln(U2)/abs(N(0,2))`
   and `N(0,2)` here denotes variance 2. Charged pairs use the table's effective
   radius; this sampler does not apply the option3 `G4DNAIRT` charged-distance
   substitution. `ComputeEffectiveRadius` uses one species' D for identical
   reactants, while the encounter uses their sum. `SetReactionType(1)` uses
   van der Waals radii and the source Onsager mapping. Audit reuse field by field;
   common reaction literals do not imply common kernels. Negative-time sentinels
   become explicit no-reaction outcomes; invalid/nonfinite arithmetic is an error.
4. **Contact.** The stepper uses strict `r*r < a*a`. At scheduler start only,
   it attempts an immediate event with the table probability; equality goes
   through the encounter sampler (type 0 gives zero time, type 1 can add delay).
   Initial overlap, exact contact and overlaps after injection/products need
   distinct tests. The source resets `fSampledMinTimeStep` per track, not per
   pair: rejected overlap can inherit a previous candidate time. This is an
   unresolved source anomaly, not an approved physical rule or a silent Rust fix.
   Coincident mobile reactants can reach a fatal placement error.
5. **Placement.** `MakeReaction::UpdatePositionForReaction` places the mobile
   partner on an immobile reactant. For two mobile reactants and positive step
   duration it samples a contact vector `S1` of length `a`, with phi uniform
   and `cos(theta) = 1 + ln(1-U*(1-exp(-2 alpha)))/alpha`,
   `alpha = a*r/(2*(D1+D2)*dt)`. It builds
   `S2 = p1+(D1/D2)*p2 + Gaussian3(0, sigma)`, with the literal source
   `sigma = 2*D1*dt + (2*D1*dt)^2/(2*D2*dt)` passed as a Gaussian width;
   then `R1=(D1*S1+D2*S2)/(D1+D2)` and `R2=D2*(S2-S1)/(D1+D2)`.
   For direct-kernel source diagnostics evaluate this literal expression in
   Geant4 native mm/ns units, then convert positions to nm: its dimensional
   inconsistency makes numerical evaluation in other units non-equivalent.
   The width's dimensional meaning and lab-axis angular construction require
   executable diagnostics. Call-path review of
   `D/management/src/G4ITModelProcessor.cc::ComputeTrackReaction` finds it calls
   `FindReaction` directly; that path reaches `MakeReaction` without calling
   `TestReactibility`. Since `fTimeStep` starts at zero and only that latter
   method assigns it, static call-path analysis implies the ordinary IRT-syn
   path leaves this positive-duration branch inactive. A search of the entire
   `source/` tree found the only `TestReactibility` call in the separate
   `G4DNAMolecularReaction::FindReaction` implementation, not the IRT-syn path;
   the base interface has no implicit call. This is source evidence, not an
   executed trace: the scheduler-level diagnostic must confirm it. Positive-dt tests must be
   labelled direct-kernel diagnostics, not normal scheduler placement behavior.
   Do not insert a square root, rotate the axis or substitute elapsed pair time
   without a documented deviation decision. At zero duration mobile positions
   remain unchanged. All products share `(reactionSite + randP)/2`, where
   `reactionSite=(sqrt(D2)*R1+sqrt(D1)*R2)/(sqrt(D1)+sqrt(D2))`
   and `randP=(1-U)*R1+U*R2`; parents are killed. Source placement agreement
   and physical translation/rotation/dimensional consistency are separate gates.
6. **Unary and bulk channels.** The option3 table contains first/pseudo-first-order
   entries, but its `ConstructProcess` does not register their scavenger processes.
   The IRT-syn pair stepper searches mobile tracks, not implicit bulk participants.
   Thus the standard baseline must not be claimed to execute all 72 table entries.
   UHDR explicitly registers `G4DNAScavengerProcess`: water propensity is the
   rate literal; other bulk propensity is `N*k/(V*NA)`, and products use the
   parent position. It selects a channel with total propensity but uses that
   selected channel's inverse rate as the clock scale. This is not a verified
   competing-exponential oracle. `G4DNASecondOrderReaction` separately retains
   residual interaction length at scale `1/(k*concentration)` and kills the parent.
   A Rust unary law `T=-ln(U)/sum(lambda)` with rate-weighted channel choice is
   a prospective extension requiring an analytic competing-hazard gate and an
   explicit custom reference constructor, not a claimed default IRT-syn behavior.
   Until resolved, source-only pair validation excludes implicit bulk/unary events;
   a production network requiring them cannot silently omit them.

### Search, applicability and unresolved release gates

The reviewed stepper calls the **no-argument** `G4IRTUtils::GetRCutOff`:
`R = 1.45 nm + 2 sqrt((2*9.46e9 nm^2/s)*(1000 ns))*erfcInv(0.01)`.
It rejects sampled elapsed times greater than scheduler **end time**, not
explicitly the remaining duration. These are executable finite-search/time
policies, not exact unbounded Brownian propagation or a global one-percent
error guarantee. A geometric worker boundary must never act as this cutoff.

Rust's correctness oracle enumerates all eligible unordered pairs for a finite
molecule population in an unbounded medium. Any production finite search uses
an explicit remaining horizon and an auditable omitted-pair probability bound;
sum the pair upper bounds to at most `1e-6` per completed history, including
all rebuilds, or fall back to all pairs. A per-pair `0.01` bound cannot be reused
as that history bound. For diffusion-controlled pairs a bound is
`(a/r)*erfc((r-a)/sqrt(4*D*h))`; only use it for other channels after proving
domination. Record the accumulated bound and cutoff policy in provenance and
checkpoints. This is a prospective deviation from the source cutoff and needs
the separate all-pairs sensitivity gate, not altered Geant4 fixtures.

Supported numerical kernel inputs are finite positions/times, nonnegative D,
positive pair radius and total D, and type-1 rates `0 < kobs < kdif` with finite
derived values. Unsupported limits fail explicitly; they do not become physical
non-reaction. Source-fixed water parameters (including the table's 293.15 K /
80.1 Onsager constants) are not a validated temperature range. The prospective
integrated envelope is the existing electron window, chemical start at each
track's validated physicochemical handoff, through 1 microsecond, no walls or
external reservoirs. Source quirks in contact, positive-duration placement,
unary execution and stop/injection synchronization remain **OPEN release gates**.
Tickets may implement diagnostics and isolated kernels; none may advertise
validated production chemistry until applicable exclusions are resolved or
explicitly narrowed in a reviewed contract amendment before production results.

New modes must freeze their exact input manifest, seed family, schema and
commands in RUN_REQUESTS.md and register exact consumers in `validation/gates.json`
before generating evidence. This ticket adds no generator mode or scientific run.

**Status.** M2: the generator modes `xs`, `mfp`, `track` were compiled and run by the project lead
(Geant4 11.4.1, Windows/MSVC); the CSVs are committed in `validation/reference/` and the M2 gate passes
(`cargo test -p g4-sim --test geant4_reference -- --ignored --nocapture`, docs/08). M3: the modes `xslow`
and `slow` (section "M3: full slowing-down reference" at the end) are **untested C++**: they were written
without a Geant4 installation and only syntax-checked against stub headers. The M3 comparison tests are in
`crates/g4-sim/tests/slowing_down_reference.rs` and are `#[ignore]`d until the files listed there exist.
No Geant4 number has been invented or embedded anywhere in the repository.

**New runs requested by agent sessions** are logged in [RUN_REQUESTS.md](RUN_REQUESTS.md): sessions write and compile-check the C++ but never run it.

## What the Rust code is compared against

Geant4 **11.4.1** (the reference tree is `rtonneau/geant4`, CMake version 11.4.1, `G4Version.hh`:
`geant4-11-04-patch-01`), data set **G4EMLOW 8.8**, physics constructor **`G4EmDNAPhysics_option2`**,
liquid water `G4_WATER` (1 g/cm3), primary electron. Two settings of the generator are deliberate:

| Setting | Why |
|---|---|
| Fluorescence / Auger **off** (`G4EmParameters::SetFluo/SetAuger(false)`) | geant4-rs M2 does not simulate the atomic de-excitation after oxygen K-shell ionisation (docs/09 Q12); with it on, Geant4 would book less than 539 eV locally for those events and the deposited-energy comparison would differ for a known reason, not a bug. |
| Energy lists avoid exactly 100 eV (Sanche window is closed at 100 eV) and exactly 1 MeV (standard `eIoni` is also active at exactly 1 MeV) | the Rust M2 has no Sanche/Melton/solvation and no standard EM; away from those points the physics of both codes is the same. |

Quantities that **cannot** be compared in M2 (they need Sanche vibrational excitation, Melton attachment
and solvation, i.e. M3): full slowing-down range, W-value, anything below 100 eV. This is why the
`track` mode stops the primary at a 100 eV tracking cut, as does the Rust run (`tracking_cut_ev`).

## Which files I need from you

Put these three CSVs (and `provenance.toml`) into `validation/reference/` and commit them. Budget:
well under 1 MB each.

| File | Mode | Contents | Compared by |
|---|---|---|---|
| `xs_per_volume.csv` | `xs` | `energy_eV,process,sigma_per_volume_per_nm` for `elastic`, `excitation`, `ionisation` (`G4EmCalculator::ComputeCrossSectionPerVolume`, 1/nm) at 208 energies (window boundaries 7.4, 9, 11 eV, 1 MeV, decade points, and 200 log-spaced energies 7.4 eV - 1 MeV) | V2 (1e-9 at nodes, 1e-6 elsewhere), V3 |
| `mfp_first_interaction.csv` | `mfp` | per energy {150, 300, 1e3, 3e3, 1e4, 3e4, 1e5, 3e5, 9e5} eV: `n_events`, `sum_L_nm`, `sum_L2_nm2` of the first free path and the counts of the process that ended it | V5: z < 3 and < 2 % |
| `track_cut100eV.csv` | `track` | per energy {200, 1e3, 1e4, 1e5, 9e5} eV, primary followed until its energy falls below 100 eV, secondaries killed: sums (and squares) of track length, deposited energy, remaining energy, secondary count and energy, and interaction counts per process | V5: z < 3 and < 2 % |

Recommended statistics: `mfp`: 10^6 events per energy (statistical error 0.1 %; with 10^5 the
2 % criterion is still meaningful); `track`: 10^4 events per energy (the 900 keV point is the slow
one, about 1.7e4 primary steps per event; reduce to 2000 events if needed, the CSV records `n_events`).
Use two different seeds and keep both runs (`mfp_first_interaction_seed2.csv`, ...) if you can: the
V5 text asks for the Geant4 run-to-run scatter to be estimated (docs/07 G.2).

## How to generate them

```bash
# 1. Geant4 11.4.1 with the data set G4EMLOW 8.8 (only dna/ is read)
source <geant4-install>/bin/geant4.sh            # sets G4LEDATA to G4EMLOW8.8
echo $G4LEDATA                                   # must end in G4EMLOW8.8

# 2. build the generator (outside the reference repository; it is read-only)
cd validation/geant4/g4rs_ref
cmake -S . -B build -DGeant4_DIR=<geant4-install>/lib/cmake/Geant4
cmake --build build -j

# 3. run (serial; each command is independent)
./build/g4rs_ref xs    ../../reference/xs_per_volume.csv
./build/g4rs_ref mfp   ../../reference/mfp_first_interaction.csv 1000000 20261
./build/g4rs_ref track ../../reference/track_cut100eV.csv        10000   20262 100
```

Arguments: `mfp <out> <events> <seed>`, `track <out> <events> <seed> <cut_eV>`. If it does not
compile, fix the API mismatch but **do not change** the physics settings (constructor, material,
fluo/auger off, energy lists, kill-secondaries rule): the Rust tests hard-code the same lists.

## `validation/reference/provenance.toml`

Required by the tests (every key must be present):

```toml
geant4_version      = "11.4.1"                  # as printed by the CSV header / G4Version
geant4_commit       = ""                        # SHA or tag of the build, if known
g4emlow_version     = "8.8"
g4emlow_md5         = "328330009df633f7e9b3a9f445745298"   # cmake/Modules/G4DatasetDefinitions.cmake
physics_constructor = "G4EmDNAPhysics_option2"  # the test asserts this exact string
generator           = "validation/geant4/g4rs_ref, git commit <sha of geant4-rs>"
date                = "YYYY-MM-DD"
command_lines       = [
  "g4rs_ref xs xs_per_volume.csv",
  "g4rs_ref mfp mfp_first_interaction.csv 1000000 20261",
  "g4rs_ref track track_cut100eV.csv 10000 20262 100",
]
fluo_auger          = "off"
compiler            = ""
platform            = ""
```

## Running the comparison

```bash
cargo test -p g4-sim --test geant4_reference -- --ignored --nocapture
```

Acceptance criteria are written in the header of `geant4_reference.rs` and were fixed before any
comparison (docs/07). A failure is investigated (data, units, a model bug, or a documented deviation
listed in docs/09), never accommodated by loosening a tolerance.

## Documented, expected differences (so they are not mistaken for bugs)

1. Non-unit rotation axis of Geant4's Champion elastic model (docs/09 Q13). The Rust default
   (`ElasticRotation::Geant4`) reproduces it, with the same +z primary direction as the generator;
   the `Exact` variant would deviate in the track quantities.
2. No atomic de-excitation (Q12) -> handled by switching it off in Geant4.
3. The termination guard of M2 (Q14) does not matter with the 100 eV cut.
4. Geant4 draws one number of interaction lengths per process, Rust draws one total free path
   and then selects the process: statistically identical, different random streams.

---

# M3: full slowing-down reference

Everything below is for the M3 comparison (secondary electrons, Sanche vibrational excitation, Melton
attachment, solvation). Same physics settings as M2: `G4EmDNAPhysics_option2`, `G4_WATER`, fluorescence and
Auger **off** (and `SetAtomDeexcitation(nullptr)` as in the M2 fix, K-shell binding energy stays local),
Geant4 11.4.1, G4EMLOW 8.8, serial run manager, primary along +z at the origin. **Nothing is killed or cut:**
the primary and every secondary are followed until they die by solvation or attachment (the Geant4
chemistry is not activated, so solvation only kills the electron and deposits its kinetic energy).

## Which files I need from you

Put these in `validation/reference/` and commit them (all small: the sums files have 7-8 rows; the spectra
file has `energies x 4 x 72` rows; the radial file `energies x 42`; `xs_lowenergy.csv` about 1100 rows):

| File | Mode | Contents | Compared by |
|---|---|---|---|
| `xs_lowenergy.csv` | `xslow` | `energy_eV,process,sigma_per_volume_per_nm` for `elastic, excitation, ionisation, vibexc, attachment` at 214 energies: 1.5, 2, 3.9, 4, 7.4, 9, 11, 12.3, 12.8, 13, 13.5, 100, 101, 150 eV and 200 log-spaced energies from 2 to 100 eV | `m3_v2_low_energy_cross_sections_match_geant4`: 1e-9 at nodes, 1e-6 elsewhere, zero windows must agree |
| `slow_sums.csv` | `slow` | per primary energy: `n_events` and, for each of the quantities primary track length `Lprim_nm`, track length of all tracks `Lall_nm`, deposited energy `edep_eV`, counts of elastic / excitation / ionisation / vibrational excitation / attachment / solvation interactions, number of secondaries, primary range `range_nm`: the sum over events and the sum of squares; plus the cross sum `sum_nsolv_x_edep` and the counters `n_other_process`, `n_other_secondary` (must be 0) | `m3_v5_full_slowing_down_matches_geant4` |
| `slow_spectra.csv` | `slow` | secondary-electron creation energy histograms, `energy_eV,generation_class,bin,count`; classes 1, 2, 3, 4 (= 4th generation or later); bins: 0 = underflow (< 0.1 eV), 1-70 = 0.1 eV x 10^(k/10), 71 = overflow (>= 1 MeV); the same edges as `g4-sim` | same test, two-sample chi-square per class |
| `slow_radial.csv` | `slow` | energy deposited per radial shell around the origin summed over all events: `energy_eV,bin,energy_eV_sum`; bin 0 = r < 0.1 nm, 1-40 = 0.1 nm x 10^(k/5), 41 = r >= 10 mm | same test, cumulative fractions |
| `provenance_m3.toml` | - | template below | the test refuses to run without it |

Optional but valuable (as for M2): a second run with another seed written to `slow2_*.csv` (the test reads
only the seed-1 files; the seed-2 run is for estimating Geant4's own scatter by hand or in a later test).

## How to generate them

Build the generator as for M2 (section "How to generate them" above), then, from
`validation/geant4/g4rs_ref`:

```bash
# cross sections of the five processes below 150 eV (seconds)
./build/g4rs_ref xslow ../../reference/xs_lowenergy.csv

# full slowing-down: writes ../../reference/slow_sums.csv, slow_spectra.csv and slow_radial.csv
./build/g4rs_ref slow  ../../reference/slow 20271
```

Arguments: `slow <out_prefix> <seed> [E_eV:events,E_eV:events,...]`. The default list (what the Rust test
assumes it can read; every energy found in `slow_sums.csv` is compared, with the Rust history counts of
`rust_histories()` in the test) is

```
8:100000,30:100000,100:100000,1000:20000,10000:2000,100000:200,900000:20
```

**Run a timing check first** (about a minute) and extrapolate:
`./build/g4rs_ref slow /tmp/slow_timing 1 8:1000,100:1000,1000:100,10000:10,100000:1`. Rust needs about
1.7e6 interactions per second per core; interactions per event are about 43 (8 eV), 130 (30 eV), 316 (100 eV), 2.8e3
(1 keV), 2.7e4 (10 keV), 2.6e5 (100 keV), 2.3e6 (900 keV). The default list is about 2.5e8 interactions in
total. Geant4 is usually 10-50 times slower per step than the Rust loop, so **the expected runtime is
between 40 minutes and 3 hours on one core, dominated by the 10 keV-900 keV entries (this is an estimate, I
could not measure Geant4)**. If it is too slow, reduce the counts of the last three entries first (the Rust
test copes with any `n_events >= 2`, only the statistical power drops: at 900 keV the standard deviation of
the primary track length is about 12 % of its mean, so 20 events give 2.7 % and 100 events 1.2 %; with 20 the
2 % relative cut of the test is automatically not applied, only the z-criterion).

## `validation/reference/provenance_m3.toml`

Every key must be present (the test checks these):

```toml
geant4_version      = "11.4.1"
geant4_commit       = "geant4-11-04-patch-01"
g4emlow_version     = "8.8"
g4emlow_md5         = "328330009df633f7e9b3a9f445745298"
physics_constructor = "G4EmDNAPhysics_option2"
generator           = "validation/geant4/g4rs_ref, geant4-rs git commit <sha>"
date                = "YYYY-MM-DD"
command_lines       = [
  "g4rs_ref xslow xs_lowenergy.csv",
  "g4rs_ref slow slow 20271 8:100000,30:100000,100:100000,1000:20000,10000:2000,100000:200,900000:20",
]
fluo_auger          = "off"      # asserted by the test
compiler            = ""
platform            = ""
```

## What to check if it does not compile or run

- The generator only adds code; `Slow*` functions and `RunSlow`/`RunXsLow` are new, the `xs/mfp/track`
  paths are unchanged. If a Geant4 API call fails to compile, fix only the API mismatch.
- In `slow_sums.csv` the counters `n_other_process` and `n_other_secondary` must be `0`: a non-zero value
  means a step by a process that is not one of the six listed (e.g. `Transportation` at the world edge) or a
  non-electron secondary (fluorescence/Auger photons: they must be off; the test refuses such a file).
- `edep_eV` per event must be exactly the primary energy (up to rounding); if it is smaller the sphere world
  is too small or a particle was killed, if larger something deposits twice.
- The first line of each CSV records `G4VERSION_NUMBER` and the physics settings.

## Documented, expected differences for M3 (so they are not mistaken for bugs)

1. Geant4 draws one number of interaction lengths per process, Rust one free path and a process choice;
   `G4RandGauss` versus Box-Muller for the solvation displacement: only distributions agree.
2. No atomic de-excitation in either code (Q12). K-shell ionisations book 539 eV locally on both sides.
3. The Champion rotation quirk (Q13) is reproduced by the default `ElasticRotation::Geant4`; the generator
   shoots along +z like the Rust runs. `Exact` is expected to differ in ranges and the radial profile
   at the 0.5-1 % level at 1-10 keV, not in anything else (docs/08).
4. Solvation: with chemistry off Geant4 kills the electron and deposits `k`; Rust does the same. The 25 meV
   proposal of the chemistry-on branch and the navigator correction of the displacement are not reproduced.
   With chemistry on (M6), `G4DNAOneStepThermalizationModel.hpp:225-242` actively calls `ComputeStep`
   and replaces the displacement by `0.8 * safety` along its direction when `safety <= displacementMag`.
   Rust models infinite water, so this clip is inapplicable. The N1 audit in docs/08 records why the clip
   is operationally unreachable for the committed reference geometry and workload (metre safety versus
   nanometre thermalisation); the aggregate CSV has no clip counter and the Gaussian has no hard cutoff.
5. Primaries start at the origin in an infinite medium; the Geant4 world is a 1 m sphere of water, large
   against the 900 keV range (about 4 mm).

---

# M6: initial physicochemical species at 1 ps

The original mode is `physchem <out.csv> <seed> [E_eV:events,...]`. It uses option2 electron physics and `G4EmDNAChemistry_option3`. Its intended pre-reaction interpretation was disproved by the 2026-10-16 source audit: the first positive-step pre-callback follows zero-time IRT reactions at global 1 ps (docs/08, Q25/R17). Its existing code, arguments, physics and output format remain historical evidence. CSV columns are `energy_eV,n_events,species,sum_count,sum_count2,sum_r2_nm2`, where the last quantity sums squared distances of every alive molecule from the primary origin. The unchanged original gate fails; it does not validate a dissociation-only Rust M6 snapshot.

Build and smoke test first:

```bash
cd validation/geant4/g4rs_ref
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DGeant4_DIR=<geant4-install>/lib/cmake/Geant4
cmake --build build -j
./build/g4rs_ref physchem /tmp/physchem_smoke.csv 1 1000:2
```

**Generator status:** Q24 fixed 2026-10-16 (IRT time-step model, snapshot at the first dt > 0 step, secondaries transported, atomic de-excitation pointer nulled; see `BUILD_NOTES.md` section 5, issues G-K). The production run was done on this machine; the data are in `validation/reference/physchem.csv` and `provenance_m6.toml`. To regenerate:



```bash
./build/g4rs_ref physchem ../../reference/physchem.csv 20261019 \
  1000:20000,10000:2000,100000:200,900000:20
```

The default is about 1.1e8 physical interactions plus the 1 ps stage. From the M3 timing estimate, expect roughly 20 minutes to 2 hours serial; time the smoke case and extrapolate on the actual machine. Counts may be reduced at high energy if necessary because each row records `n_events`, but keep at least 20 events at 900 keV.

Exactly two files are needed in `validation/reference/`:

- `physchem.csv`, from the successful command above.
- `provenance_m6.toml`, with `geant4_version`, `geant4_commit`, `g4emlow_version`, `g4emlow_md5`, `physics_constructor = "G4EmDNAPhysics_option2"`, `chemistry_constructor = "G4EmDNAChemistry_option3"`, `scheduler_snapshot_ps = 1.0`, generator commit, date, exact command line, seed, compiler and platform.

Then run `cargo test -p g4-sim --test physchem_reference -- --ignored --nocapture`. The ignored test contains the pre-registered count and RMS acceptance criteria. It must remain ignored until both files exist and the generator exits successfully.

## M6 pre-IRT diagnostic (RR-001, awaiting data)

`physchem_prechem <out.csv> <seed> <E_eV:events,...>` is a **new mode**, compiled and linked against
Geant4 11.4.1/MSVC on 2026-10-17, never run by an agent. It uses the same option2/option3 constructors,
IRT selection, full cascade, liquid water, +z beam, Meesungnoen2002 and fluo/Auger-off settings. Only its
snapshot hook differs: a zero-step **post** callback waits until no water molecule remains in the main
list, then records the merged surviving daughters and stops **before the next IRT time selection**.
`G4ITStepProcessor2.cc:110-185` merges daughters/removes killed parents before this hook;
`G4DNAIRTMoleculeEncounterStepper.cc:431-467` defers IRT initialization while stopped water remains,
and `G4ITModelProcessor.cc:118-131` initializes it on the next time selection (paths under
`source/processes/electromagnetic/dna/{management,models}/src/`). The reaction table and water
recombination/dissociation processes are unchanged. This isolates M6 without implementing M7 chemistry.

The new mode requires explicit seed and statistics, refuses to overwrite its output, and exits nonzero
if a positive step or inter-molecular chemical reaction precedes capture, an event lacks its snapshot,
any captured molecule is not alive at global 1 ps, or the completed event count is wrong. An exit-0
complete run is required; compilation alone does not validate these runtime assumptions. A failed run
must be investigated, never accepted by removing a guard.

The CSV uses the original six columns, with the new provenance marker:

```text
# snapshot_boundary: after_dissociation_before_irt; global_time_ps: 1; chemical_reactions: 0
energy_eV,n_events,species,sum_count,sum_count2,sum_r2_nm2
```

Each species/energy row contains the event count, summed alive count, sum of **squared per-event
counts**, and sum of squared distances of all survivors from the primary origin, in nm^2. Zero-count
events contribute zero to both count moments; the `n_events` denominator includes them. Native Geant4
species names are mapped by the Rust consumer. The requested production run must contain all seven
dissociation species (H3Op, °OH, OHm, e_aq, H, H2, Oxy) at all four energies: exactly 28 rows. Reaction-only
species, incomplete energies/statistics, invalid moments, and duplicate rows are rejected.

After the user smoke test in `BUILD_NOTES.md`, run from `validation/geant4/g4rs_ref` with a **new** output:

```bash
g4rs_ref physchem_prechem ../../reference/physchem_prechem.csv 20261020 1000:20000,10000:2000,100000:200,900000:20
```

Commit the complete new CSV and `provenance_m6_prechem.toml`; never replace `physchem.csv` or its
provenance. The new provenance must contain the following, plus a nonempty `geant4_commit`, `generator`
(repository commit), `date`, `compiler`, `platform`, and explicit `generator_dirty` (boolean):

```toml
geant4_version = "11.4.1"
g4emlow_version = "8.8"
g4emlow_md5 = "328330009df633f7e9b3a9f445745298"
physics_constructor = "G4EmDNAPhysics_option2"
chemistry_constructor = "G4EmDNAChemistry_option3"
chemistry_time_step_model = "IRT"
snapshot_boundary = "after_dissociation_before_irt"
scheduler_snapshot_ps = 1.0
fluo_auger = "off"
seed = 20261020
command_line = "g4rs_ref physchem_prechem ../../reference/physchem_prechem.csv 20261020 1000:20000,10000:2000,100000:200,900000:20"
```

Record the exact executable invocation in the provenance; the command above assumes `g4rs_ref` is on
PATH. The consumer accepts that registered command (use PATH or a shell alias for the built executable).
Run only the paired ignored test after data and provenance are committed:

```bash
cargo test -p g4-sim --test physchem_reference m6_prechem_counts_and_rms_match_geant4 -- --ignored --exact --nocapture
```

Criteria registered before these data exist: the same **z <= 4.5**, **15%** count difference when both
means exceed 0.1/event, and **20%** RMS-radius difference as the original gate, for all 28 rows. Provenance,
boundary and completeness checks run first. A passing stage-matched gate is required before M6 closes;
a failure leaves the gate open and Q21/R14's scheduler projection under investigation. The original
post-reaction reference/gate stays unchanged and is not expected to pass Rust M6.

## M6 900 keV five-boundary spatial diagnostic (RR-002, completed locally)

The additive mode is `physchem_spatial_diagnostic <out.csv> <histories_per_seed> <seed,...>`. It is
fixed at a 900 keV electron from the origin along +z and retains option2 physics, option3 chemistry,
IRT selection, the full secondary cascade, liquid water and fluo/Auger-off settings. The initial run is:

```bash
g4rs_ref physchem_spatial_diagnostic ../../reference/spatial_geant4_900kev_40.csv 40 20261021,20261022,20261023,20261024,20261025
```

The mode refuses overwrite. It writes provenance comments, the exact Ticket 02 long-form header,
contiguous `(seed,history)` blocks, and a final `# complete: true` marker only after all requested runs
return. The boundaries are `physical_sites` (chemistry-producing excitation, ionisation, attachment
and terminal-electron sites), `solvated_electron_creation`, `pre_decay_water`,
`dissociation_products` (births across interleaved zero-time passes), and `final_survivors` (alive
after recombination/decay and before IRT/diffusion). Entity rows have count 1; an absent category has
one explicit count-0 row. Namespaces, track/parent IDs where available, source/creation ordinals and
daughter pass/processing ordinals preserve within-implementation lineage. Optional values are `NA`.

Moment columns are `count,sum_r2_nm2,sum_z_nm,sum_z2_nm2,sum_x2_plus_y2_nm2,max_radius_nm`. The 17
edges are `10^(k/2)` nm for `k=0..16` (1 nm through 100 mm), giving 18 half-open bins: underflow,
sixteen finite intervals and overflow. Positions remain relative to the primary origin. Runtime guards
require zero-step global 1 ps semantics, no preceding intermolecular reaction, complete lineage,
recognized water states and a final snapshot per event. Require exit 0, five completion lines, the
final marker, exact five x 40 coverage and strict parsing. Never commit a partial or smoke CSV.

The matching provenance is `provenance_m6_spatial_900kev_40.toml`; besides nonempty
`geant4_commit`, `generator`, `date`, `compiler`, and `platform`, it requires:

```toml
geant4_version = "11.4.1"
g4emlow_version = "8.8"
physics_constructor = "G4EmDNAPhysics_option2"
chemistry_constructor = "G4EmDNAChemistry_option3"
chemistry_time_step_model = "IRT"
diagnostic_stage = "global_1_ps_pre_irt"
fluo_auger = "off"
primary_energy_eV = 900000
seeds = [20261021, 20261022, 20261023, 20261024, 20261025]
histories_per_seed = 40
analysis_seed = 20261026
bootstrap_replicates = 20000
generator_dirty = false
command_line = "g4rs_ref physchem_spatial_diagnostic ../../reference/spatial_geant4_900kev_40.csv 40 20261021,20261022,20261023,20261024,20261025"
```

After both files are committed, run:

```bash
cargo test --release -p g4-sim --test physchem_reference m6_900kev_spatial_diagnostic -- --ignored --exact --nocapture
```

The consumer generates an independent Rust sample and uses 20,000 hierarchical bootstrap replicates,
resampling seed groups then histories while keeping all boundaries together. The sole primary is the
Rust/Geant4 pooled seven-species final-survivor RMS ratio. It reports a 95% primary interval,
simultaneous maximum-log-deviation boundary intervals, first detected divergence (not causation), and
secondary category, per-history, radial, maximum, longitudinal and transverse summaries. If primary
relative half-width `(U-L)/(2R)` exceeds 0.04, request complete 80-history samples in both
implementations, then 160; stop at five x 160 regardless of significance. This diagnostic changes
neither the existing 20% M6 gate nor M6's closure state.

### RR-002 result (2026-10-23)

The precision-only rule expanded the comparison to new, complete 80-histories-per-seed artifacts for
both implementations (seeds 20261021--20261025; 400 histories each). Geant4 and the Rust consumer
both exited 0 with exact coverage. With the frozen analysis seed 20261026 and 20,000 hierarchical
bootstrap replicates, the primary Rust/Geant4 pooled final-survivor RMS ratio was 0.981740457, with a
95% interval of [0.949564151, 1.015007983] and a 3.333% relative interval half-width. The <= 4%
precision target was reached, so the 160-history level was not run. The interval includes 1.0: no
primary difference was statistically resolved, although the result is not an equivalence proof.

The 80-history CSVs and provenance were kept under `.scratch/rr-002-80/` as local-only artifacts and
were not promoted to `validation/reference/`. The commands and 40/80/160 procedure above remain the
reproduction contract; the authoritative scientific interpretation is in `docs/08-roadmap.md`.

## M7 H+H two-body SBS reference (RR-003, awaiting clean data)

`two_body_h_h <out.csv> <seed> <pairs>` injects two neutral H molecules at +/-0.1 nm at time zero,
uses `G4EmDNAChemistry_option3` with the SBS time-step model and observes option3 reaction id 1 at
tau `[0.25, 0.5, 1, 2, 4, 16]`. The corresponding observation times use
`0.19705088916617716 ps * tau`. The mode refuses overwrite and accepts only positive seeds representable
by the platform `long`; overflow is rejected instead of silently changing the seed.

After the smoke procedure in `BUILD_NOTES.md` section 9, run from `validation/geant4/g4rs_ref` with
the built executable on `PATH`:

```powershell
g4rs_ref two_body_h_h ../../reference/two_body_h_h.csv 202610231 100000 *> "$env:TEMP/two_body_h_h_production.log"
if ($LASTEXITCODE -ne 0) { throw 'two-body production failed; do not commit partial output' }
```

The CSV header is
`tau,observation_time_ps,total_pairs,reacted_count,surviving_count`. Require six ordered rows, exact
100,000-pair coverage in every row, monotonic reacted counts, the requested seed and the trailing
`# complete: true` marker. Never reuse a smoke path or overwrite an existing reference.

Create `validation/reference/provenance_m7_two_body_h_h.toml` with these exact contract fields and
real nonempty values for the final five metadata fields:

```toml
geant4_version = "11.4.1"
g4emlow_version = "8.8"
chemistry_constructor = "G4EmDNAChemistry_option3"
time_step_model = "SBS"
reaction_id = 1
initial_separation_nm = 0.2
pairs = 100000
seed = 202610231
tau = [0.25, 0.5, 1.0, 2.0, 4.0, 16.0]
command_line = "g4rs_ref two_body_h_h ../../reference/two_body_h_h.csv 202610231 100000"
generator_commit = "<commit containing the corrected generator>"
generator_dirty = false
compiler = "<compiler and version>"
platform = "<OS and version>"
generated_at = "<YYYY-MM-DD>"
```

Only after both new files are committed, run the paired close-gate command:

```bash
cargo test --release -p g4-chemistry --test geant4_two_body m7_irt_survival_matches_geant4_sbs -- --ignored --exact --nocapture
```

The consumer validates provenance and exact CSV coverage before generating the fixed-seed Rust IRT
sample. At each point it applies the unchanged V9 pointwise combined-binomial three-sigma rule. A
failure is evidence about the Geant4 executable semantics; it must be recorded, not tuned away.



## R04: isolated option3 IRT matrix (RR-004, user-run only)

New mode: `g4rs_ref pair_matrix_irt <new-output.csv> <seed> <pairs-per-case-distance>`.
This calls Geant4 11.4.1 `G4DNAIRT::GetIndependentReactionTime` directly with the
unmodified `G4EmDNAChemistry_option3` reaction table and IRT selected before initialization.
It does not simulate transport, competing reactions, products or a chemical history.
Source: `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc` and
`source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc`.
IDs 1,2,8,9,11,17,42,51 use source-order reactants and native table parameters.
TDC `a` is the effective radius; PDC `a` is the contact radius. Relative diffusion
is D_A+D_B even for A+A. No parameters or source contact branches are refitted.

From `validation/geant4/g4rs_ref`, with the built executable on PATH:

```powershell
g4rs_ref pair_matrix_irt ../../reference/pair_matrix_irt_r04_n100000.csv 20261027404 100000
if ($LASTEXITCODE -ne 0) { throw 'RR-004 failed; retain diagnostic output only' }
```

Output creation is exclusive. Never overwrite RR-003 or an earlier R04 run.
The 64-bit decimal seed is encoded injectively into CLHEP RanecuEngine seeds
`[seed % 2147483562 + 1, seed / 2147483562 + 1]`, with index 0. Accepted seeds
are 1 through `2147483562*2147483398-1`; the full requested seed and both engine
seeds are recorded. Sampling resets this engine after initialization and proceeds
in the fixed channel, distance, pair order; it uses independent random draws, not
paired Rust seeds. Counts may be 1..1,000,000 for smoke tests, but only the frozen
100k,200k,400k,800k,1m sequence is eligible for production evidence.

CSV schema `r04_pair_v1` (144 ordered data rows, comment lines begin `#`):

```text
reaction_id,r0_over_a,a_nm,D_nm2_per_ps,onsager_nm,kobs_M_inv_s_inv,endpoint,tau,observation_time_ps,total_pairs,event_count
```

For each ID, `r0_over_a` is 1,1.5,3; for each distance the five `cdf` rows use
`tau=t*D/a^2` of 0.01,0.1,1,10,100. `event_count` counts sampled times <= t.
The sixth row is `no_reaction,inf,inf`: its count is the kernel's exact -1 ps
sentinel, never finite-time censoring. CDF and survival are complements, not
independent endpoints. Rates are M^-1 s^-1, distances nm, diffusion nm²/ps,
and elapsed time ps. Channel comments retain canonical reactants, effective
radius and contact probability. Reject any nonfinite/unrecognized kernel return.
The trailing marker must be exactly the requested seed/count and `rows: 144`:
`# complete: true; seed: 20261027404; pairs: 100000; rows: 144`.
Require exit 0, eight channel completion messages, exact coverage/order,
monotonic CDF counts, counts in [0,total_pairs], and CDF+eventual nonreaction
<= total_pairs. A partial file or smoke output is never reference evidence.

Create new `validation/reference/provenance_r04_pair_matrix_n100000.toml` with
`geant4_version`, `geant4_version_number`, `g4emlow_version`, `physics_constructor`,
`chemistry_constructor`, `timestep_model`, `mode`, `schema`, `seed`, `rng_engine`,
`engine_seeds`, `pairs_per_case_distance`, `reaction_ids`, `r0_over_a`, `tau`,
`command`, `working_directory`, `date`, `generator_commit`, `generator_dirty`,
`compiler`, `platform`, `csv_sha256` and `completion_rows`. Record 11.4.1/1141,
G4EMLOW 8.8, option2, option3, IRT, RanecuEngine, and the actual clean generator
commit (`generator_dirty=false`). The exact command above reproduces the file;
record runtime dataset versions even though this kernel reads no cross sections.

Acceptance is the separately preregistered 144-endpoint Geant4 family in
`docs/validation/pair-case-matrix.md`: alpha 0.001 Bonferroni, independent
binomial errors, absolute effect <0.01 and the adjusted confidence interval
strictly inside (-0.01,+0.01). Precision expansion alone permits a new complete
run with a new filename/provenance. Missing data remain REQUESTED. This is
source-fidelity evidence, separate from the Brownian physical-law verdict.

The consumer selects a precision level with `R04_GEANT4_PAIRS` (default
`100000`), restricted to 100000, 200000, 400000, 800000 and 1000000. It
expects `pair_matrix_irt_r04_n<N>.csv` and
`provenance_r04_pair_matrix_n<N>.toml`, and validates the level-specific
command, counts, markers, source-ordered Geant4 configuration names and hash.
For example, after an independently authorized and fully validated 200000-pair
run, set `R04_GEANT4_PAIRS=200000` and invoke the ignored consumer with
`cargo test --release -p g4-chemistry --test geant4_pair_r04 r04_option3_irt_matches_geant4 -- --ignored --exact --nocapture`.
The existence of a supported filename does not authorize or establish a run.


## G1-F: source-fidelity matrix (RR-005, user-run only)

Mode: `g4rs_ref pair_matrix_irt_g1f <new-output.csv> <seed> <pairs-per-case-distance>`.
This additive mode preserves RR-004. Source: Geant4 11.4.3
`source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc`,
`GetIndependentReactionTime` and `SamplePDC`; option3 parameters are unchanged.
The [prospective contract](../../docs/validation/g1-f-source-fidelity.md)
is source fidelity only, not independent physical validation.

Before statistical sampling, the mode directly calls the public
`SamplePDC(1,0)` and requires its rejection fallback `-1`. It then restores
Ranecu to the requested seeds. This diagnostic tests the singular proposal,
not every charged caller branch. The caller's fallback is divided by D and
is distinct from the `-1 ps` nonreaction sentinel. The Rust regression
`rust_pdc_contact_is_a_documented_continuum_limit_not_source_fallback` pins
Rust's deliberately different finite positive contact-limit sampling.
No reference output is inferred from that Rust test.

The CSV schema is `g1f_pair_v1`: the same 11 columns and parameter checks as
RR-004, but **120 ordered data rows**. Iterate IDs 1,2,8,9,11,17,42,51 in
that order; TDC distances are 1,1.5,3; PDC distances are 1.5,3. Each distance
has five CDF rows at tau 0.01,0.1,1,10,100 and one eventual nonreaction row.
All four PDC exact-contact blocks (24 endpoints) are excluded, not shifted.
The first comment identifies mode `pair_matrix_irt_g1f` and schema
`g1f_pair_v1`. Immediately after the column header the required comment is:

```text
# exclusions: PDC exact contact; reaction_ids: 11,17,42,51; r0_over_a: 1; endpoints: 24; contact_regression: SamplePDC(1,0)=-1
```

The remaining metadata, channel comments and completion marker follow RR-004,
with `rows: 120`. Reject partial output, missing metadata or unexpected times.
Use a clean committed generator revision before producing reference evidence.
From `validation/geant4/g4rs_ref`, with the compiled executable on PATH:

```powershell
g4rs_ref pair_matrix_irt_g1f ../../reference/pair_matrix_irt_g1f_n100000.csv 20261027404 100000
if ($LASTEXITCODE -ne 0) { throw 'G1-F failed; output is diagnostic only' }
```

Create `validation/reference/provenance_g1f_pair_matrix_n100000.toml`:

```toml
geant4_version = "11.4.3"
geant4_version_number = 1143
g4emlow_version = "8.8"
physics_constructor = "G4EmDNAPhysics_option2"
chemistry_constructor = "G4EmDNAChemistry_option3"
timestep_model = "IRT"
mode = "pair_matrix_irt_g1f"
schema = "g1f_pair_v1"
seed = 20261027404
rng_engine = "RanecuEngine"
engine_seeds = [933675347, 10]
pairs_per_case_distance = 100000
reaction_ids = [1, 2, 8, 9, 11, 17, 42, 51]
r0_over_a = [1.0, 1.5, 3.0] # union; exclusion_policy removes PDC contact
tau = [0.01, 0.1, 1.0, 10.0, 100.0]
excluded_endpoints = 24
exclusion_policy = "pdc_exact_contact"
contact_regression = "SamplePDC(1,0)=-1"
completion_rows = 120
command = "g4rs_ref pair_matrix_irt_g1f ../../reference/pair_matrix_irt_g1f_n100000.csv 20261027404 100000"
working_directory = "validation/geant4/g4rs_ref"
date = "<actual run date>"
generator_commit = "<actual 40-character commit>"
generator_dirty = false
compiler = "<actual compiler/version>"
platform = "<actual OS/platform>"
csv_sha256 = "<actual SHA-256 of complete CSV>"
```

The strict consumer runs from the repository root:

```powershell
cargo test --release -p g4-chemistry --test geant4_pair_g1f g1f_option3_irt_matches_geant4 -- --ignored --exact --nocapture
```

Independent Rust seed: 20261027403. Retain the 144-family two-sided Bonferroni
threshold 4.495384196564731 at alpha 0.001, independent stabilized binomial
SE, `abs(z) <= zcrit`, and `abs(diff)+zcrit*SE < 0.01` at all 120 endpoints.
Insufficient precision is INCONCLUSIVE. Expand only for precision through
100k,200k,400k,800k,1m, never to chase significance. For another registered N,
set `G1F_GEANT4_PAIRS=N`, use new `pair_matrix_irt_g1f_n<N>.csv` and
`provenance_g1f_pair_matrix_n<N>.toml`, and change command/statistics accordingly.
The Rust case/distance ordinals retain holes for excluded contacts.
The consumer reports GEANT4 only; G1-F also requires the analytic and contact
regressions. It does not close physical validity, R05/G2 or G(t) gates.

RR-007 adapter recovery (2026-11-09): diagnostic product cleanup uses the owning track list's erase path before clearing empty holder lists. Only immobile cases temporarily unfinalize/set/refinalize the selected reactant diffusion coefficient, restoring it after the setting. Cached reaction radius and all registered inputs, transformations and tolerances are unchanged. Scratch smoke passed; subsequent dedicated production and strict consumer passed 1200 rows, 144 expected errors and 8064 coordinates. Data/provenance commits: 9607036827be6954fb5423f294275d36c383661c / f3c169f. This remains finite-grid mapping only.

### SYN-2 independent analytic marginal manifest (2026-11-09; prospective)

Exact ignored test: `cargo test --release -p g4-chemistry --test syn_analytic_moments syn_marginals_match_independent_analytic_moments -- --ignored --exact --nocapture`.
`SYN_MOMENT_HISTORIES` selects 300, 1000, then 3000 histories, each with333 draws;
maximum999000 draws per setting. Each history is one independent batch
(df299/999/2999), exceeding the required30. Master seeds20261109081/20261109082
are independent; each setting resets its history RNG to stream(history,
Chemistry). This intentionally couples raw uniforms across settings before
any outcomes are inspected. Free-diffusion normalized coordinates coincide
across D/time scalings; placement transformations consume the same raw RNG
stream but different source parameters. Translation-equivalent settings share
exact primitives; other placement distributions remain distinct, correlated
transforms. No source CSV or Geant4 RNG is used. Within each setting, history
streams and consecutive draws remain independent in the model; correlations
between settings do not invalidate Bonferroni's union bound.

All15 embedded mobile species in SpeciesTable order plus D=0, at dt0/1/1000/
1000000ps, contribute3 means and6 upper-triangular covariances each. Analytic
mean0, covariance2Ddt*I; scales sqrt(2Ddt) for means and2Ddt for covariance.
Both seeds give1152 endpoints. Placement uses channels1/2/11, r/a1/3,
dt0/1/1000ps, x/z/translated-x orientations from SYN-2; parent coordinates
plus shared product centroid (undefined and absent for channel11), each mean
and upper-triangular covariance. Independent angular quadrature and Gaussian/
uniform moment algebra in syn_support provide expectations and characteristic
scales (coordinate SD, products of coordinate SDs).4860 moment endpoints plus
108 distance identities and108 product-count identities give5076 placement
endpoints. Full family m6228, alpha.001, three looks; one-sample t difference
interval from independent batches at alpha/(3m), inside +/-1% scale. The
separately refinement-checked oracle error expands stochastic intervals by
1e-8 of scale. Zero-scale deterministic identities use the registered1e-12
relative/nm floor; source product count is exact. Every draw also checks
parent distance (initial separation for dt0, a for positive dt), product
count and common product site. No statistical family endpoints are dropped.

Power is finite: at1m and infinite df, a normal variance has expected interval
half-width0.7692% and approximate individual equivalence power0.8973. With
coupled diffusion settings there are six distinct coordinate-variance streams
(three axes,two seeds), not270 independent variances: roughly0.52 joint power
for those six alone. Placement transforms add dependent constraints; no full
family PASS is promised. All three registered looks and final INCONCLUSIVE
handling remain unchanged. This validates marginal sampling against analytic
source distributions, not physical correctness of the anomalous dormant
positive-duration source branch or ordinary scheduler applicability.

SYN-2 analytic results (2026-11-09): all registered looks completed with no
FAIL endpoints. Moment endpoints (6012) plus216 per-setting distance/product
identities constitute the6228 family:

| Histories | Draws/setting | Moment PASS | Moment INCONCLUSIVE | Identity PASS |
|---:|---:|---:|---:|---:|
|300|99900|1914|4098|216|
|1000|333000|3272|2740|216|
|3000|999000|5779|233|216|

Final result is INCONCLUSIVE, not PASS. All1152 free-diffusion endpoints and
all dt0 placement endpoints passed; the233 unresolved endpoints are dormant
positive-duration placement diagnostics (195 at1ps,38 at1ns). This does not
establish a failure of production physics or make that branch reachable.
Example: seed20261109081,channel2,r/a1,dt1ps,x,endpoint26 has oracle/scale
0.001165475962414208 nm^2 and difference interval
[-0.00001423979691432,0.00001671727395930] nm^2, exceeding its frozen band
+/-0.00001165475962414 nm^2. No further precision look is authorized by this
manifest. Logs: session scratch `02-marginal-{300,1000,3000}.log`; final run
92.52s. Ordinary library scheduler trace is registered below as RR-008 and awaits execution;
its zero-duration applicability statement is still static source inference.

## RR-008 ordinary installed-library scheduler placement trace

This new program calls the installed11.4.1 chemistry manager/scheduler with
option2/option3 IRT_syn. It neither replaces the reaction process nor invokes
MakeReaction/TestReactibility directly; no source-body inclusion or random-draw
interception occurs. The trace tests whether actual positive selected event
steps preserve parent sites, supporting the static inference that ordinary
placement uses its default zero-duration branch. It does not directly observe
the private fTimeStep field or establish all scheduler behaviors.

Frozen settings per replica: channels1,2,11; r/a=3 only; orientations+x,+z,
and translated+x origin(10a,-7a,3a). Birth1ps, end1000000ps, unbounded on this
scale inside1m water world.32 independent trials per setting, channel then
orientation then trial order,288rows. Master seeds20261109091/092. Per-row
Ranecu seed integer=master+zero-based-row; the existing injective encoding is
(seed%2147483562+1,seed/2147483562+1). Both parent tracks are pushed normally
into the holder and the chemistry manager runs the ordinary installed model.
The reaction callback captures only the first reaction, checks source product
species/order/colocation and requests scheduler Stop; the following post-step
callback checks both parent IDs have disappeared from the main list. Scheduler
owns and cleans its tracks; no adapter cleanup of populated delayed lists.

Commands (working directory validation/geant4/g4rs_ref):
```text
g4rs_syn_scheduler_ref trace ../../reference/syn_scheduler_1141_r1.csv 20261109091 32
g4rs_syn_scheduler_ref trace ../../reference/syn_scheduler_1141_r2.csv 20261109092 32
```
Smoke uses a new scratch filename,seed1,trials1 (9rows); coverage acceptance
is only applied to complete32-trial replicas. Cap32trials,2h/8GiB per invocation,
<1MB per CSV; no rerolling seeds or adding trials to satisfy coverage.
CSV is exactly:
```text
# schema: syn_scheduler_v1; Geant4: 1141; claim: ordinary-library first-reaction trace
# seed: <seed>; trials: 32; birth_ps: 1; horizon_ps: 1000000
channel,orientation,trial,outcome,a_nm,d1_nm2_ps,d2_nm2_ps,p1x,p1y,p1z,p2x,p2y,p2z,event_time_ps,event_step_ps,r1x,r1y,r1z,r2x,r2y,r2z,products,px,py,pz,parents_removed
```
Footer `# complete: true; seed: <seed>; trials: 32; rows: 288` is required.
Outcome `reaction` records positive event step/time, canonical parent positions,
product count/centroid and post-step removal bit1. Outcome `censored` records
all thirteen columns after p2z asNA; it is not eventual nonreaction probability.
Zero-product channel11 has products0 andNA centroid. Event times and steps are
inps, coordinates/radius innm, diffusion innm^2/ps.

Acceptance: exact ordered288rows per replica, source parameters to1e-12
relative, input and reacting parent coordinates unchanged to1e-12relative+
1e-12nm absolute. Every recorded event has time>1ps and<=horizon, step>0 and
<=event time, correct product count and parent-removal bit1. Product positions
must lie on the original pair segment with fractional coordinate
[(1-q)/2,(2-q)/2],q=sqrt(D2)/(sqrt(D1)+sqrt(D2)), within1e-12 numerical floor;
source producer separately verifies product identity/order/colocation. At
least one positive-time event in every channel/orientation per replica is
required for coverage; missing coverage is INCONCLUSIVE, not a physical FAIL.
No event-count or reaction-time distribution inference is attached. Preserved
3a parent separation with a positive event step distinguishes this path from
the direct positive-duration source branch, which forces separation a.

Exact consumer:
`cargo test --release -p g4-chemistry --test geant4_syn_scheduler syn_ordinary_scheduler_preserves_parent_sites -- --ignored --exact --nocapture`.
Each `provenance_syn_scheduler_1141_r{1,2}.toml` must record top-level strings
geant4_version11.4.1,physics_constructorG4EmDNAPhysics_option2,
chemistry_constructorG4EmDNAChemistry_option3,timestep_modelIRT_syn,mode trace,
schema syn_scheduler_v1,claim "ordinary-library first-reaction trace",
g4emlow_version8.8; integers seed,trials_per_setting32,completion_rows288;
canonical-LF csv_sha256; exact registered command above; actual_command,
generator_commit,generator_dirty(bool),generator_source_sha256(named hashes of
SynScheduler.cc/CMakeLists.txt),compiler,platform,date,compile_definitions,
g4emlow_md5,source_hashes(named normative G4DNAMakeReaction.cc,
G4DNAIndependentReactionTimeStepper.cc,G4Scheduler.cc and linked Geant4 DLLs).
Data-commit identity is filled after committing new validated evidence.
No RR-006/RR-007 evidence or producer mode is changed.

RR-008 recovery 3 (2026-11-09): native stacks identified two adapter initialization omissions. G4EmDNAPhysics construction resets the requested timestep model to Unknown; selecting IRT_syn after physics construction restores registration. With no transport BeamOn, Brownian transportation had no water-density table; the supported chemistry UseAsStandalone(true) initialization builds molecular process tables and closes geometry. Guards verify the enum before process construction/after initialization and the actual installed IRT model, stepper and reaction process in StartProcessing. Corrected scratch seed1/trials1 exited native 0 with nine ordered rows, three reactions and six censored histories; schema, footer, parent sites, product support/count and removal checks passed. This is recovery smoke only: independent review, fresh dedicated smoke and registered production/consumer remain pending. No reference-source, random stream, manifest, tolerance or statistical result changed; 233 positive-duration marginal endpoints remain INCONCLUSIVE.

Interpretation of earlier RR-006/RR-007 evidence: those adapters selected the timestep enum before constructing electromagnetic physics, whose constructor resets it. Their direct diagnostic calls explicitly instantiate the pair sampler or renamed source reaction implementation; they do not execute scheduler model selection. Their IRT-syn labels identify the intended direct algorithm, not an observed registered scheduler model. The relevant direct reaction setup follows the same non-SBS branch for Unknown and IRT_syn. Existing source-diagnostic and finite-grid mapping results remain limited to those direct calls; no ordinary scheduler claim follows from them. Their producer modes, data and provenance are preserved unchanged. RR-008 adds explicit checks of actual registered model types.


### RR-008 recovery and registered result (2026-11-09)

The user-authorized recovery corrected trace-harness initialization order and
standalone molecular physics-table initialization. Independent review found no
actionable defect. A subsequent fresh smoke passed before production.
Both registered production runs exited 0 with 288 ordered rows and complete
footers. Replicate 1 has 80 reactions/208 censored trials; replicate 2 has
79 reactions/209 censored trials. All observed parent-site/product/removal
checks passed, but channel 11, +z had zero positive-time events in both runs.
Verdict: **INCONCLUSIVE**, not PASS. The exact registered Rust consumer exited
101 at the coverage gap in replicate 1, before examining replicate 2; a
separate full-output validator checked both files. No additional samples or
seed changes were made. The 233 analytic moment endpoints remain INCONCLUSIVE.

Complete data commit: `2264b14ffe024b2f44accb2c53b29ae40ee93d37`.
Provenance commit: `9b4aaf4`. Final comparison log is
`.scratch/tests/2026-11-08__irt-syn-parallel-checkpoint-direction/02-rr008/production-consumer-complete.log`.
RR-006/RR-007 were not rerun or replaced. Ticket 02 remains incomplete pending
the registered coverage gate; no later production acceptance follows from
these observed reactions alone.
