# gps backlog: M12 finite water volume, remaining work

Seeds for `/gps start <slug>`, registered with `scout-merge.js --from docs/gps-backlog-m12-geometry.md`
on 2026-11-14. Origin: review of M12 after RR-009 (docs/08 "M12 Finite water volume"). Steps (1)-(4)
of M12 are done (PR #32, #33, #34, #36); the sphere endpoints of RR-009 pass (two 1000 nm / 10 keV
endpoints INCONCLUSIVE, accepted 2026-11-14). Standing rules of `docs/gps-backlog.md` apply; the
reference version is now Geant4 11.4.3 (`docs/geant4-11.4.3-migration.md`).

---

## m12-isotropic-source-box-endpoints

**Goal.** Close the RR-009 gate by running its six box endpoints (1000x1000x100 nm, 1/10/100 keV).

**Requires.** Nothing (RR-009 data committed, `aca5947`, provenance `d48318f`).

**State.** The Rust run layer has a fixed +z primary direction; Geant4 `g4rs_ref finite_volume` draws
isotropic primaries. The consumer `crates/g4-sim/tests/rr009_finite_volume.rs` reports the box rows
`NOT RUN` for that reason (header lines 13-16).

**Tasks.**
1. Verify the Geant4 source direction sampling in `validation/geant4/g4rs_ref` (`finite_volume` mode)
   and the Geant4 `.cc` it uses; reproduce the distribution, not the generator.
2. Add an isotropic primary-direction option to `RunConfig` (default unchanged, so existing runs and
   seeds are bitwise identical); record it in provenance; bump the run schema if the output changes.
   Document its place in the per-history draw order (docs/reproducibility.md).
3. Tests: isotropy (pre-registered KS / chi-square on cos theta and phi), default path bitwise
   unchanged, 1/2/8-thread equality.
4. Enable the box rows in the consumer with the unchanged criteria (z = 4.098, margin +-0.01); run it
   and record the verdict in docs/08 and docs/07; update `validation/gates.json`.

**C++.** None.

**Acceptance.** All 24 RR-009 endpoints evaluated; no criterion changed; gate status recorded.

## m12-reference-version-constant

**Goal.** Bump `GEANT4_REFERENCE_VERSION` (`crates/g4-core/src/provenance.rs:13`) from 11.4.1 to 11.4.3.

**Requires.** Nothing; independent of the other seeds.

**Tasks.** Change the constant and the provenance envelope tests; keep the embedded-chemistry
`geant4_release` check (`crates/g4-chemistry/src/reactions.rs`) and the `geant4_version` checks of
tests consuming 11.4.1 data unchanged; document in `docs/geant4-11.4.3-migration.md` and
docs/reproducibility.md.

**C++.** None.

**Acceptance.** Output envelopes state 11.4.3; every 11.4.1 data consumer still passes; no reference
file touched.

## m12-chemistry-boundary-rule

**Goal.** M12 step (5): define the chemical-stage behaviour of species in a finite volume.

**Requires.** Answers to docs/09 G-B1 (does IRT see the boundary?) and G-B2 (out-of-volume
dissociation products), and IRT in the run layer (M7 many-species scheduling). Blocked until then.

**State.** RR-009 is a partial answer: no outside species in Geant4 over 30,000 histories. G-B1/G-B2
stay open on the points RR-009 could not test (docs/09).

**Tasks.** Source audit of the Geant4 11.4.3 IRT/scheduler for geometry calls; if data are needed, a
new generator mode (compile-check only), a RUN_REQUESTS entry and an ignored consumer with
pre-registered criteria; implement the rule in the run layer.

**C++.** Possibly a new `g4rs_ref` mode; never run by the agent.

**Acceptance.** G-B1/G-B2 answered in docs/09 from source or data; rule implemented and tested; M12
close decision left to the project lead.
