# R04 isolated pair case matrix (preregistered 2026-10-27)

This matrix is fixed before R04 statistical runs. It tests isolated reaction-time
primitives, not a correlated chemical history. Source version: Geant4 11.4.1.
Literal IDs and source lines come from [the 72-ID execution matrix](option3-reaction-coverage.md),
which audits `G4EmDNAChemistry_option3.cc:96-666` and `G4DNAIRT.cc:286-390`.
The matching executable contract is `crates/g4-chemistry/tests/pair_case_matrix.rs`.

## Cases

`a` is `TwoBodyReactionParameters::reaction_radius()`: the TDC rate-derived
radius and the PDC sum of molecular contact radii. `D = D_A + D_B`, including
A+A. All distances start at `r0/a = {1, 1.5, 3}` and all finite observations
are at `tau = t D/a² = {0.01, 0.1, 1, 10, 100}`. Each distance also has a
separate eventual `P(NoReaction)` endpoint. A contact TDC event may be
immediate; it is counted at every positive time. PDC contact follows its
specified partial-reaction boundary. No-reaction is unconditional, never
conditioned on a reaction having occurred.

| Case | Kind | IRT regime | Ordered reactants | Electrostatics |
|---|---|---|---|---|
| option3 ID 1 | A+A | neutral TDC | H, H | none |
| option3 ID 2 | A+B | neutral TDC | e_aq, H | none (one charged species) |
| option3 ID 8 | A+A | charged TDC | e_aq, e_aq | repulsive |
| option3 ID 9 | A+B | charged TDC | H3Op, OHm | attractive |
| option3 ID 11 | A+B | neutral PDC | OH, H | none |
| option3 ID 17 | A+A | neutral PDC | OH, OH | none |
| option3 ID 42 | A+B | charged PDC | e_aq, H3Op | attractive |
| option3 ID 51 | A+A | charged PDC | Om, Om | repulsive |

These eight are **literal option3 channels**: use the embedded source rate,
diffusion coefficients, radii and charges without refitting. Charged means
both reactants have nonzero charge and the signed Onsager radius is nonzero.
The signed electrostatic distance mapping in Geant4 IRT is an approximation;
the independent charged Brownian physical-law verdict is reported separately.

The following four are **synthetic dimensionless cases**, with `a=1`,
`D_A+D_B=1`, and no Geant4 channel ID. They exercise controlled limits and
must never be labelled as Geant4 option3 parameters. For PDC,
`k_obs/k_diff=0.5`; the boundary and rate convention used to realise this
fraction must be stated in the oracle report.

| Case | Kind | Regime | Signed Onsager length / a | `k_obs/k_diff` |
|---|---|---|---:|---:|
| syn-neutral-tdc-ab | A+B | neutral TDC | 0 | n/a |
| syn-charged-tdc-aa | A+A | charged TDC | +1 | n/a |
| syn-neutral-pdc-aa | A+A | neutral PDC | 0 | 0.5 |
| syn-charged-pdc-ab | A+B | charged PDC | -1 | 0.5 |

There are 12 cases × 3 distances × (5 finite times + 1 eventual mass) =
**216 endpoints**. Finite-time CDF and survival are complements of the same
events and count as one comparison, not two. Synthetic cases are L1 analytic
and Brownian tests only; the Geant4 reference family contains the eight
literal channels, hence 144 endpoints.

## Frozen statistical and numerical contract

- Family-wise two-sided alpha is **0.001**, Bonferroni over all 216 analytic
  endpoints (`alpha_point = 0.001/216`) and, separately, over all 144 Geant4
  endpoints (`0.001/144`). Report every pointwise z, adjusted threshold and
  absolute difference. Correlated times do not reduce the multiplicity.
- Analytic sampler comparison uses **100,000 independent pairs per case and
  distance**, Rust seed `20261027401`. Derive per-pair RNG stream from
  `(case ordinal, distance ordinal, pair ordinal)` so loop order cannot change
  samples. The independent high-precision law supplies the expected CDF and
  eventual mass. Exact deterministic cases require exact agreement.
- Independent Brownian oracle starts at **10,000 independent pairs per case
  and distance per refinement level**, seed `20261027402`. Its maximum
  dimensionless step schedule is `h = 1/16, 1/32, 1/64, 1/128, 1/256`;
  each level halves h. Resolve observations exactly and document any adaptive
  smaller steps/boundary bridge. Use at least the first three levels; only
  advance through the frozen remaining levels when needed. Last two estimates
  at every endpoint must differ by **<0.005 absolute** and by at most **three
  combined standard errors**. Eventual mass has the same comparison rule.
- If Brownian Monte Carlo precision is inadequate, increase independent pairs
  at **10k, 20k, 40k, 80k, 160k, 320k, 640k, 1m**. The final cap is
  1,000,000, rather than 1,280,000. Continue on precision criteria alone,
  never on whether a significance test passed. Predetermine the precision
  need from interval width, before interpreting each result. Keep all level
  estimates in the convergence report.
- Geant4 uses **independent** seed `20261027404`; the Rust comparison uses
  seed `20261027403`, both with 100,000 pairs per case and distance initially.
  Their precision expansion is the same fixed doubling sequence
  `100k, 200k, 400k, 800k, 1m`. Use independent binomial standard errors,
  not paired-seed covariance. Both Bonferroni significance and an absolute
  effect **<0.01** are required. The Bonferroni two-sided confidence interval
  for the difference must lie strictly inside `(-0.01, +0.01)` to show that
  precision resolves the effect limit. This is an equivalence requirement;
  a nonsignificant z alone is insufficient.
- A Brownian level that fails the fixed convergence conditions after all five
  levels is **INCONCLUSIVE**. Precision that cannot resolve a required limit
  at the sample cap is **INCONCLUSIVE**. A resolved discrepancy outside a
  predeclared effect or family limit is **FAIL**. Missing or invalid Geant4
  evidence is **REQUESTED**, never PASS. G1 is PASS only when analytic,
  independent Brownian and Geant4 verdicts all pass. Source-faithful Geant4
  agreement cannot replace the independent physical-law verdict.

The prior V9 pointwise three-sigma, 100,000-pair, 18-curve-point and three
no-reaction-mass contract remains in `two_body_validation.rs`. RR-003 remains
its own six-point external H+H request. Neither is rescored under this R04
family or overwritten by its evidence.

## Reproducing the analytic family

`validation/pair_oracles/analytic.py` requires Python 3.11+ and mpmath 1.3.0.
From the repository root, run:

```text
python validation/pair_oracles/analytic.py --output crates/g4-chemistry/tests/fixtures/pair_analytic_anchors.json
cargo test -p g4-chemistry --test pair_analytic -- --nocapture
```

The generator evaluates decimal source literals at 80 digits and repeats at
160 digits, requiring identical rounded binary64 anchors. It never imports
Rust or samples trajectories. Equations and Geant4 source paths are recorded
in the generator; input provenance remains in the chemistry TOML tables.
The JSON records the mpmath version, precision, Geant4 version and two-sided
normal Bonferroni threshold `Phi^-1(1 - 0.001/(2*216))`. Rust reports each
signed z, absolute difference, threshold and separate per-case ANALYTIC verdict.
For probability zero or one it requires exact counts. The deliberately biased
sampler multiplies all finite elapsed times by four; the same gate must reject it.

Neutral TDC uses the absorbing-sphere erfc CDF; neutral PDC uses the radiation
boundary erfc/erfcx CDF. Charged anchors evaluate the Geant4 effective-distance
law, including its unsaturated TDC weight: the observable CDF is
`min(1, W*erfc(abs(L-a)/(2*sqrt(D*t))))`, and eventual nonreaction is
`1-min(1,W)`. The charged PDC anchor uses the original alpha/sinh/coth equations,
not the Rust sampler's algebraic reduction. Neither charged comparison claims
a physical Coulomb first-passage PASS; that remains a separate Brownian verdict.

Synthetic cases use length unit `a` and time unit `a²/D`, hence dimensionless
`a=D=1`. To realise charge with the fixed source Onsager constant, the charged
cases use that constant as their dimensional length unit. Their dimensional
relative diffusion is 1 nm²/ps. Each species receives half of the contact radius
and diffusion. PDC sets `kobs=0.5*kdiff`, with `kdiff=4*pi*D*R` for A+B and half
that for A+A; `kact=kobs*kdiff/(kdiff-kobs)`. Neutral PDC uses
`A=kact/(kobs*a)`; charged PDC retains the source A+A factor in alpha.
Synthetic rates in the fixture are dimensional embeddings, never option3 rates.
The Rust streams use history ordinal `(case*3+distance)*100000+pair`, stage
Chemistry and the preregistered analytic seed; all 3.6 million pairs are independent.
