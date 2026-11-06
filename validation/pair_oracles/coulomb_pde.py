"""Finite-domain backward equation for a Coulomb radial diffusion.

This numerical solver is independent of the trajectory sampler. The CLI retains
separate domain, space, time and radiation-boundary checks for every charged
case. Its error estimates are empirical, not rigorous bounds. An absorbing
outer boundary underestimates the continuum infinite-domain reaction probability.
Units are a=D=1. See docs/validation/pair-coulomb-pde.md.
"""
import math

import numpy as np
from scipy.linalg import solve_banded


def solve(charge, kappa, radii, times, dr, outer, maxdt, relstep,
          *, theta=1.0, robin='ghost'):
    """Integrate with theta=1 (backward Euler) or 0.5 (Crank--Nicolson).

    ``onesided`` independently imposes the second-order forward Robin
    derivative by eliminating the contact value from the first interior row.
    ``ghost`` eliminates a central-derivative ghost through the PDE at contact.
    """
    if (not math.isfinite(charge) or not math.isfinite(dr) or dr <= 0
            or not math.isfinite(outer) or outer <= 1
            or not math.isfinite(maxdt) or maxdt <= 0
            or not math.isfinite(relstep) or relstep <= 0
            or (kappa is not None and (not math.isfinite(kappa) or kappa <= 0))
            or not radii or any(not 1 <= r < outer for r in radii)
            or not times or any(not math.isfinite(t) or t <= 0 for t in times)
            or any(a >= b for a, b in zip(times, times[1:]))
            or theta not in (0.5, 1.0) or robin not in ('ghost', 'onesided')):
        raise ValueError('invalid Coulomb PDE inputs')
    intervals = round((outer - 1) / dr)
    if intervals < 2 or not math.isclose(1 + intervals * dr, outer, abs_tol=1e-10):
        raise ValueError('outer boundary must align with the grid')
    r = 1 + dr * np.arange(intervals)
    drift = 2 / r + charge / r**2
    lower = 1 / dr**2 - drift / (2 * dr)
    upper = 1 / dr**2 + drift / (2 * dr)
    if np.any(lower < 0) or np.any(upper < 0):
        raise ValueError('grid too coarse for monotone centered discretization')
    diagonal = np.full(intervals, -2 / dr**2)
    source = np.zeros(intervals)
    if kappa is None:
        diagonal[0] = 0
        upper[0] = 0
    elif robin == 'ghost':
        diagonal[0] -= lower[0] * 2 * dr * kappa
        upper[0] += lower[0]
        source[0] = lower[0] * 2 * dr * kappa
    else:
        denominator = 3 + 2 * dr * kappa
        contact_constant = 2 * dr * kappa / denominator
        diagonal[1] += lower[1] * 4 / denominator
        upper[1] -= lower[1] / denominator
        source[1] = lower[1] * contact_constant
        r, lower, upper, diagonal, source = (
            a[1:] for a in (r, lower, upper, diagonal, source))
        intervals -= 1
    lower[0] = 0
    upper[-1] = 0
    values = np.zeros(intervals)
    if kappa is None:
        values[0] = 1
    current = 0.0
    output = []
    for target in times:
        while current < target:
            dt = min(maxdt, max(1e-7, relstep * current), target - current)
            bands = np.zeros((3, intervals))
            bands[0, 1:] = -theta * dt * upper[:-1]
            bands[1] = 1 - theta * dt * diagonal
            bands[2, :-1] = -theta * dt * lower[1:]
            rhs = values + dt * source
            if theta != 1:
                rhs += (1-theta) * dt * diagonal * values
                rhs[1:] += (1-theta) * dt * lower[1:] * values[:-1]
                rhs[:-1] += (1-theta) * dt * upper[:-1] * values[1:]
            if kappa is None:
                rhs[0] = 1
            values = solve_banded((1, 1), bands, rhs, check_finite=False)
            current = min(target, current + dt)
        grid = np.append(r, outer)
        solution = np.append(values, 0.0)
        if kappa is not None and robin == 'onesided':
            contact = (4*values[0]-values[1]+2*dr*kappa) / denominator
            grid = np.insert(grid, 0, 1.0)
            solution = np.insert(solution, 0, contact)
        output.append(np.interp(radii, grid, solution))
    return np.array(output).T


def refinement_errors(runs, radiation):
    """Conservative empirical budgets without assuming an asymptotic order.

    Doubling observed differences instead of Richardson division deliberately
    retains some overlapping spatial/boundary error. A 1e-11 outer floor
    prevents an unchanged binary64 value being described as exact zero error.
    """
    fine = runs['fine']
    return dict(
        space=2*np.abs(fine-runs['space_medium']),
        time=2*np.abs(fine-runs['time_medium']),
        outer=np.maximum(2*np.abs(fine-runs['outer_medium']), 1e-11),
        radiation=(2*np.maximum(np.abs(fine-runs['radiation_fine']),
                               np.abs(runs['radiation_fine']-runs['radiation_medium']))
                   if radiation else np.zeros_like(fine)),
        rounding=np.full_like(fine, 1e-11))


def main():
    import argparse
    from datetime import datetime, timezone
    import hashlib
    import json
    import os
    from pathlib import Path
    import platform
    import subprocess
    import sys
    import scipy

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.evidence.exists():
        parser.error('new output and evidence paths required; existing evidence is never overwritten')
    root = Path(__file__).resolve().parents[2]
    fixture_dir = root/'crates/g4-chemistry/tests/fixtures'
    physical_path = fixture_dir/'pair_coulomb_anchors.json'
    analytic_path = fixture_dir/'pair_analytic_anchors.json'
    physical = json.loads(physical_path.read_text())
    analytic = json.loads(analytic_path.read_text())
    times = [0.01, 0.1, 1.0, 10.0, 100.0]
    radii = [1.0, 1.5, 3.0]
    # Predeclared accuracy target: one tenth of the frozen .005 effect limit.
    accuracy_target = 0.0005
    base = dict(dr=.005, outer=100., maxdt=.025, relstep=.0025,
                theta=.5, robin='ghost')
    configs = {
        'space_coarse': dict(base, dr=.02),
        'space_medium': dict(base, dr=.01),
        'time_coarse': dict(base, maxdt=.1, relstep=.01),
        'time_medium': dict(base, maxdt=.05, relstep=.005),
        'outer_coarse': dict(base, outer=60.),
        'outer_medium': dict(base, outer=80.),
        'fine': base,
        'radiation_medium': dict(base, dr=.01, robin='onesided'),
        'radiation_fine': dict(base, robin='onesided'),
    }
    provenance = dict(
        created_utc=datetime.now(timezone.utc).isoformat(),
        command=['python', 'validation/pair_oracles/coulomb_pde.py', *sys.argv[1:]],
        generator_git_commit=subprocess.check_output(
            ['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
        working_tree_dirty=bool(subprocess.check_output(
            ['git', 'status', '--porcelain'], cwd=root, text=True).strip()),
        python=platform.python_version(), numpy=np.__version__, scipy=scipy.__version__,
        platform=platform.platform(),
        thread_limits={key: os.environ.get(key) for key in
                       ['OPENBLAS_NUM_THREADS', 'OMP_NUM_THREADS', 'MKL_NUM_THREADS']},
        sha256={str(path.relative_to(root)).replace('\\', '/'):
                hashlib.sha256(path.read_bytes()).hexdigest()
                for path in [physical_path, analytic_path, Path(__file__).resolve()]})
    evidence = dict(provenance=provenance, times=times, radii=radii,
                    configurations=configs, cases=[], neutral_checks=[])
    cases = []
    analytic_by_name = {c['name']: c for c in analytic['cases']}
    for case in physical['cases']:
        name, charge, kappa = case['name'], case['charge'], case['kappa']
        runs = {}
        for label, config in configs.items():
            if label.startswith('radiation') and kappa is None:
                continue
            runs[label] = solve(charge, kappa, radii, times, **config)
            print(f'{name} {label} complete', flush=True)
        components = refinement_errors(runs, kappa is not None)
        errors = sum(components.values())
        fine = runs['fine']
        if not np.all(np.isfinite(fine)) or np.min(fine) < -1e-10 or np.max(fine) > 1+1e-10:
            raise ValueError(f'{name}: invalid probabilities')
        if np.any(np.diff(fine, axis=1) < -1e-10):
            raise ValueError(f'{name}: nonmonotone CDF')
        eventual = np.array([p['eventual'] for p in case['points']])[:, None]
        if np.any(fine > eventual+errors):
            raise ValueError(f'{name}: CDF exceeds eventual mass')
        if np.max(errors) >= accuracy_target:
            raise ValueError(f'{name}: empirical uncertainty {np.max(errors)} exceeds target')
        # Record contraction rather than hide endpoints outside the asymptotic range.
        contraction = {}
        for axis in ['space', 'time', 'outer']:
            coarse_diff = np.abs(runs[axis+'_coarse']-runs[axis+'_medium'])
            fine_diff = np.abs(runs[axis+'_medium']-fine)
            contraction[axis] = dict(coarse_difference=coarse_diff.tolist(),
                                     fine_difference=fine_diff.tolist())
            if axis != 'outer' and np.any(fine_diff > coarse_diff+1e-8):
                raise ValueError(f'{name}: noncontracting {axis} refinement')
        approximate = np.array([p['probabilities'][:5]
                                for p in analytic_by_name[name]['points']])
        evidence['cases'].append(dict(name=name, charge=charge, kappa=kappa,
            runs={key: value.tolist() for key, value in runs.items()},
            contraction=contraction, effective_distance_minus_pde=(approximate-fine).tolist(),
            maximum_error=float(np.max(errors))))
        points = [dict(ratio=ratio, probabilities=fine[i].tolist(), errors=errors[i].tolist(),
                       error_components={key: value[i].tolist() for key, value in components.items()})
                  for i, ratio in enumerate(radii)]
        cases.append(dict(name=name, charge=charge, kappa=kappa, points=points))
        print(f'{name}: maximum empirical error {np.max(errors):.9g}', flush=True)

    charged_names = {c['name'] for c in physical['cases']}
    for case in analytic['cases']:
        if case['name'] in charged_names:
            continue
        fraction = 1-case['points'][0]['probabilities'][5]
        kappa = fraction/(1-fraction) if fraction < 1 else None
        exact = np.array([p['probabilities'][:5] for p in case['points']])
        checks = {}
        for robin in (['ghost', 'onesided'] if kappa is not None else ['ghost']):
            numerical = solve(0., kappa, radii, times, **dict(base, robin=robin))
            difference = np.abs(numerical-exact)
            if np.max(difference) >= 0.0001:
                raise ValueError(f"{case['name']}: neutral analytic limit failed")
            checks[robin] = dict(probabilities=numerical.tolist(),
                                 absolute_errors=difference.tolist(),
                                 maximum_error=float(np.max(difference)))
        evidence['neutral_checks'].append(dict(name=case['name'], kappa=kappa, checks=checks))
        print(f"{case['name']}: neutral exact limit checked", flush=True)
    evidence['complete'] = True
    result = dict(schema_version=1, law='Physical Coulomb finite-time backward equation',
        error_kind='empirical discretization estimates; NOT rigorous bounds',
        accuracy_target=accuracy_target, times=times, provenance=provenance, cases=cases)
    for path, content in [(args.evidence, evidence), (args.output, result)]:
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open('x', encoding='utf-8') as stream:
            json.dump(content, stream, indent=2, allow_nan=False)
            stream.write('\n')
    print('COMPLETE: six charged cases and six neutral exact-limit checks', flush=True)


if __name__ == '__main__':
    main()
