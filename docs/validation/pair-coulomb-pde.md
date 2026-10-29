# Independent finite-time Coulomb PDE anchors

The numerical fixture `crates/g4-chemistry/tests/fixtures/pair_coulomb_finite_anchors.json`
supplies all 90 charged finite-time endpoints in the frozen R04 matrix. It is
independent of the Brownian trajectories and the Geant4 effective-distance
sampling law. Its uncertainty arrays are **empirical numerical error estimates,
not rigorous bounds**. A completed numerical calculation does not establish a
Brownian or G1 PASS.

## Equation and physical boundary

In relative units `a=D_A+D_B=1`, with signed `c=rc/a`, solve

```text
u_t = u_rr + (2/r + c/r^2) u_r,      1 < r < R
u(r,0) = 0,                         r > 1
u(R,t) = 0
```

Positive c is repulsion: `U/kT=c/r` produces outward force. Negative c is
attraction. TDC uses `u(1,t)=1`; exact initial contact reacts immediately.
PDC uses `u_r(1,t)=kappa*(u(1,t)-1)` and starts without an immediate reaction.
The physical radiation coefficient is
`kappa=f/(1-f)*c/(1-exp(-c))`, with `f=kobs/kdiff`; its zero-charge limit is
`f/(1-f)`. The A+A half factor is included in both macroscopic rates, while
relative diffusion is the sum of the two diffusivities. This is the same
physical convention as the independent Brownian oracle.

Inputs are the previously generated 80/160-digit physical Coulomb fixture,
not fitted finite-time data. Its provenance derives the dimensionless charge,
boundary coefficient and exact eventual law from the option3 molecular data.
The continuum solution is bounded above by
`f*expm1(c/r)/expm1(c)` (f=1 for TDC), which the generator checks at every point.

The read-only Geant4 11.4.1 source
`source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc:286-356`
was checked directly. It uses `r0=rc/expm1(rc/r0)` and the charged PDC
alpha/sinh/coth expressions. Those expressions are a separate law, not the
Coulomb PDE solved here. No Geant4 generator was run or reference file changed.

The final source audit used the versioned 11.4.1 source tree and verified
`G4DNAIRT.cc` against tag `v11.4.1`, commit
`f01f313f122a19ae77c6f27192c9464ba0279b5e`. The generic checkout inspected
initially was an 11.5 beta tree; it is not the final reference identity.
The release file and tag match after line-ending normalization, and the
relevant contact/mapping formulas were rechecked in that release file.
See `validation/pair_oracles/evidence/geant4_source_audit_20261029.json`.

## Discretization and independent checks

`validation/pair_oracles/coulomb_pde.py` uses centered spatial differences and
Crank–Nicolson time integration. The spatial off-diagonal rates must be
nonnegative. Time steps grow geometrically from `1e-7`, are capped by maxdt,
and land exactly on each observation time. The tiny initial steps resolve the
incompatible TDC initial/contact values without a large first CN step. No
Richardson extrapolation or clipping of numerical probabilities is used.

Two second-order Robin discretizations are compared independently:

- The primary ghost stencil imposes a central derivative at contact and
  eliminates its ghost value through the boundary condition.
- The alternative forward derivative
  `(-3*u0+4*u1-u2)/(2*dr)=kappa*(u0-1)` eliminates u0 from the first interior
  equation. Its contact value is reconstructed after each observation.

The final values use dr=0.005, R=100, maxdt=0.025, relative step=0.0025 and
the ghost stencil. Every charged case retains these variations, changing
only the indicated numerical component:

| Component | Coarse | Medium | Fine |
|---|---:|---:|---:|
| Spatial step | 0.02 | 0.01 | 0.005 |
| Maximum time step | 0.1 | 0.05 | 0.025 |
| Relative time step (with maximum) | 0.01 | 0.005 | 0.0025 |
| Outer radius | 60 | 80 | 100 |
| Alternative Robin spatial step | — | 0.01 | 0.005 |

The per-endpoint spatial and temporal estimates are twice the absolute
medium-to-fine difference, without dividing by an assumed convergence order.
The outer estimate is twice the R80-to-R100 difference, with a 1e-11 floor.
The radiation estimate is twice the larger of the alternative-to-primary
fine-grid difference and the alternative stencil's medium-to-fine difference.
A 1e-11 floating-point allowance is added. The components are summed; some
boundary/spatial error is intentionally counted twice. The floating-point
allowance is not a certified rounding analysis.

Spatial and temporal fine differences must contract relative to the coarse
differences, allowing 1e-8 numerical noise. All probabilities must be finite,
within [0,1] to 1e-10, increasing in time, below the exact eventual law within
the error estimate, and have total estimated error below the prospectively
chosen 0.0005 numerical target. That target is one tenth of the frozen 0.005
effect limit; it does not change any statistical or Brownian acceptance rule.

The finite outer boundary removes trajectories that could return and react;
the corresponding continuum bias is downward. A tiny domain-refinement
difference is evidence of stability, not a rigorous infinite-domain error
bound. Likewise, contraction of a finite number of grids cannot prove all
unmeasured discretization errors absent. These limitations remain attached to
the fixture whenever it is used as an anchor.

Every one of the six neutral matrix cases is checked against its existing
independent analytic CDF at all 15 endpoints, including both Robin stencils
where applicable. The required maximum neutral discrepancy is 0.0001.
Unit tests also cover the absorbing-sphere law, radiation contact, both force
signs, alternate Robin stencils, invalid numerical options, and separation of
error components.

## Evidence and reproduction

The 2026-10-29 complete run passed its numerical criteria. Maximum component
estimates below are over each case's 15 finite-time endpoints; the maximum
total is computed endpoint by endpoint, so it need not equal the sum of
the separate column maxima.

| Case | Spatial | Temporal | Radiation | Total |
|---|---:|---:|---:|---:|
| option3-8 | 3.69473e-5 | 1.16490e-7 | 0 | 3.69475e-5 |
| option3-9 | 1.46746e-5 | 3.48683e-7 | 0 | 1.46990e-5 |
| option3-42 | 1.46801e-5 | 5.08447e-8 | 3.87615e-5 | 5.34468e-5 |
| option3-51 | 1.21084e-5 | 9.42822e-9 | 2.26947e-5 | 3.32491e-5 |
| syn-charged-tdc-aa | 1.10351e-5 | 2.60320e-7 | 0 | 1.10534e-5 |
| syn-charged-pdc-ab | 6.07662e-5 | 1.54765e-7 | 1.61197e-4 | 2.21989e-4 |

All outer components hit the conservative numerical floor of 1e-11; the
largest measured R80-to-R100 difference is 4.01e-13. Across all neutral
cases, the maximum absolute error against the exact CDF is 3.26251e-5 for
the primary stencil and 7.71450e-5 for the alternative stencil. Spatial and
time difference maxima shrink by approximately a factor of four when their
steps halve, consistent with second-order convergence on these grids.

The effective-distance law and physical PDE differ substantially in all
three charged TDC cases. Their maximum finite-time absolute discrepancies
are approximately 1.00000 (option3-8), 0.996051 (option3-9), and 0.994643
(synthetic charged TDC). The maximum PDC discrepancies are 0.00177394
(option3-42), 0.000901055 (option3-51), and 0.00748926 (synthetic charged
PDC); the last has three endpoints above 0.005. These are direct numerical
model-law comparisons, with the retained PDE error estimates, not completed
Brownian convergence verdicts or permission to alter the frozen laws.

`validation/pair_oracles/evidence/coulomb_pde_20261029.json` retains every
charged coarse/medium/fine array, each component's contraction differences,
all neutral comparisons, and effective-distance-minus-PDE differences. Both
JSON files record the generator and input SHA-256 hashes, git revision and
dirty status, command, UTC date, platform, Python/NumPy/SciPy versions, and
thread limits. The fixture contains six named cases with three ratios and
five finite probabilities/errors per ratio; it contains no eventual endpoint.

From the repository root in PowerShell, use new paths for every reproduction:

```powershell
$env:OPENBLAS_NUM_THREADS='1'
$env:OMP_NUM_THREADS='1'
$env:MKL_NUM_THREADS='1'
python -m unittest discover -s validation/pair_oracles -p test_coulomb_pde.py
python validation/pair_oracles/coulomb_pde.py --output .scratch/pair_coulomb_finite_reproduction.json --evidence .scratch/pair_coulomb_pde_reproduction.json
```

The generator refuses existing output/evidence paths. It writes complete
fixtures only after every charged and neutral check succeeds. No random
seed is used: this deterministic PDE calculation samples no trajectories.
The Brownian seed and frozen refinement/sample schedules remain unchanged.
