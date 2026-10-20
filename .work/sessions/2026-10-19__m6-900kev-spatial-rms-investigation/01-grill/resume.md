# Session: m6-900kev-spatial-rms-investigation

**Date:** 2026-10-19T19:17:25.518Z
**Status:** Grill phase complete

## Problem Statement

The stage-matched M6 RR-001 gate passes after the source-proven recombination-schedule fix, but the seven shared survivor species at 900 keV all have Rust RMS radii 12.1% to 13.5% above the Geant4 11.4.1 reference. The reference contains only 20 histories and aggregate count and squared-radius sums, so it cannot estimate uncertainty in the RMS ratio or identify whether the spatial scale first differs in physical transport, solvation, dissociation, or spatially selective recombination. Investigate and localize this residual without changing the scientific model or existing validation evidence.

## Context & Constraints

- **Current behavior:** Rust and Geant4 use the same pooled origin-based survivor RMS definition, `sqrt(sum(r^2) / count)`. RR-001 passes 28/28 count and RMS rows under the unchanged M6 criteria, but a common 900 keV spatial excess remains. N1 established that Geant4's finite-geometry solvation clip cannot explain it; N2 and the mapping from physical cascade order to molecule insertion order remain unaudited.
- **Pain point:** The existing `physchem_prechem.csv` has one Geant4 seed, 20 histories at 900 keV, and no per-history or intermediate-boundary moments. It cannot distinguish sampling variation from a broader physical track, solvation or dissociation displacement, or biased recombination survivor selection.
- **Dependencies:** Build on `m6-residual-rust-audit`. Preserve every existing generator mode, reference/provenance file, gate, and tolerance. Verify behavior against Geant4 11.4.1 source. Agent sessions may write and compile-check a new generator but must never run it; the project lead performs the requested Geant4 run. A source-proven model defect is documented for a separate fix session rather than changed here.
- **Tech stack:** Rust workspace (`dna-physics`, `dna-chemistry`, `dna-sim`), Geant4 11.4.1 C++ reference generator under `validation/geant4/g4rs_ref`, CSV/TOML validation artifacts, deterministic Rust and Geant4 seeds, and an ignored Rust comparison test.

## Success Metrics

- Complete the N2 source audit from terminal physical electron through `e_aq` creation, including position, time, survivor state, parent/track identity, creation order, and the physical-event-to-molecule insertion mapping in both implementations.
- Add separate Rust and Geant4 diagnostics at five boundaries: chemistry-producing physical sites, solvated-electron creation, pre-decay water states, post-dissociation products, and post-recombination survivors. Per history and category/species, record stable identifiers, count, `sum_r2_nm2`, maximum radius, longitudinal and transverse moments, and preregistered half-decade radial-bin counts from 1 nm through 100 mm with underflow and overflow.
- Preregister and implement a deterministic statistical consumer whose primary estimand is the Rust/Geant4 ratio of pooled RMS over all shared final survivors. A seed-stratified hierarchical bootstrap with 20,000 replicates and a recorded analysis seed produces the 95% primary interval and simultaneous stage-localization intervals; the residual is statistically resolved when the primary interval excludes 1.0.
- Start with five independent seeds and 40 histories per seed per implementation. Increase uniformly to 80 and then 160 histories per seed only when the primary interval's relative half-width exceeds four percentage points, stopping at 800 histories per implementation and reporting the achieved precision.
- Compile-check the new Geant4 mode, document its schema and path-independent build/run recipe, and append a new `RUN_REQUESTS.md` entry requesting one long-form CSV covering all seeds and histories plus one matching provenance TOML. The session ends `awaiting data`; it neither reopens nor closes M6.

## Architecture & Approach

First finish the source trace and record the exact lifecycle and ordering semantics without changing behavior. Add diagnostic-only hooks or records at the five agreed boundaries in Rust and in a separately named Geant4 generator mode, tentatively `physchem_spatial_diagnostic`. The physical boundary includes only chemistry-producing excitation, ionisation, attachment, and terminal-electron sites, not elastic or vibrational interactions that produce no chemistry record.

Use a long-form schema keyed by seed, history, boundary, category/species, parent/track identity, and creation order. Store summary moments and radial-bin counts rather than every molecule position. Radial edges are `10^(k/2)` nm for integer `k` from 0 through 16, with underflow and overflow. Directional moments are `sum_z`, `sum_z2`, and `sum_x2_plus_y2`; maximum radius and `sum_r2_nm2` retain the spatial tails and pooled RMS.

The ignored Rust consumer treats implementations as independent samples. It performs a deterministic hierarchical bootstrap by resampling the five seed groups and then histories within selected seeds for 20,000 replicates. The final all-shared-survivor pooled RMS ratio is the sole primary comparison. Counts, per-history and per-species RMS, radial distributions and quantiles, maxima, and longitudinal/transverse extent are secondary diagnostics. Simultaneous localization intervals use the bootstrap distribution of the maximum absolute log-ratio deviation. The earliest boundary whose simultaneous interval excludes 1.0 is reported as the first detected divergence, not proof of causation.

## Assumptions & Trade-offs

The common seven-species shift motivates a single correlated primary estimand rather than seven nominally independent claims. Fixed logarithmic bins and summary moments keep the output tractable at 900 keV while retaining millimetre-scale track shape and nanometre-scale displacement information. Stable identifiers expose lifecycle and ordering behavior inside each implementation, but differing random-number engines prevent eventwise Rust/Geant4 pairing.

The existing 20% M6 RMS tolerance remains a gate criterion and is not an equivalence margin for this investigation. Excluding ratio 1.0 establishes a statistically detectable residual, not physical importance or causation. Different populations exist at successive boundaries, so localization must be interpreted with the source audit. The runtime cap favors a bounded reproducible request over indefinite precision. No behavioral fix, reference replacement, tolerance change, M6 closure decision, or Geant4 execution is in scope.

## Open Questions

None. Exact mode, test, artifact, and deterministic seed names may be chosen during planning as long as they preserve the approved schema, sample design, and provenance requirements.

## Notes

The committed Geant4 e_aq pooled RMS at 900 keV is approximately 1.810788 mm. The seven documented Rust relative excesses span 12.07% to 13.46%, while their counts agree within the current statistical checks. Existing five-seed Rust work measured count stability but did not measure RMS-estimator spread. The Geant4 source confirms that water states inherit physical interaction positions, solvated electrons inherit the sampled terminal-electron displacement, dissociation daughters inherit mother plus product displacement, and recombination selects and kills nearby `e_aq`; all five proposed boundaries are therefore relevant. No glossary or ADR update is required.
