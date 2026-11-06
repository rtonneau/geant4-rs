import math
import unittest

import numpy as np

from coulomb_pde import solve


class CoulombPdeTests(unittest.TestCase):
    def test_committed_fixture_matches_complete_refinement_evidence(self):
        import hashlib
        import json
        from pathlib import Path
        from coulomb_pde import refinement_errors
        root = Path(__file__).resolve().parents[2]
        fixture = json.loads((root/'crates/g4-chemistry/tests/fixtures/'
                              'pair_coulomb_finite_anchors.json').read_text())
        evidence = json.loads((root/'validation/pair_oracles/evidence/'
                               'coulomb_pde_20261029.json').read_text())
        self.assertTrue(evidence['complete'])
        self.assertEqual(fixture['times'], [0.01, 0.1, 1.0, 10.0, 100.0])
        self.assertEqual(len(evidence['neutral_checks']), 6)
        self.assertEqual([c['name'] for c in fixture['cases']],
                         ['option3-8', 'option3-9', 'option3-42', 'option3-51',
                          'syn-charged-tdc-aa', 'syn-charged-pdc-ab'])
        for name, expected in fixture['provenance']['sha256'].items():
            self.assertEqual(hashlib.sha256((root/name).read_bytes()).hexdigest(), expected)
        for case, record in zip(fixture['cases'], evidence['cases']):
            self.assertEqual(case['name'], record['name'])
            runs = {name: np.array(values) for name, values in record['runs'].items()}
            errors = refinement_errors(runs, case['kappa'] is not None)
            self.assertEqual([p['ratio'] for p in case['points']], [1.0, 1.5, 3.0])
            np.testing.assert_array_equal([p['probabilities'] for p in case['points']], runs['fine'])
            np.testing.assert_array_equal([p['errors'] for p in case['points']], sum(errors.values()))
            for component, values in errors.items():
                np.testing.assert_array_equal(
                    [p['error_components'][component] for p in case['points']], values)
            self.assertLess(np.max(sum(errors.values())), fixture['accuracy_target'])

    def test_error_budget_keeps_numerical_sources_separate(self):
        from coulomb_pde import refinement_errors
        runs = {name: np.full((3, 5), value) for name, value in
                [('fine', .5), ('space_medium', .501), ('time_medium', .5001),
                 ('outer_medium', .50001), ('radiation_fine', .5002),
                 ('radiation_medium', .5008)]}
        errors = refinement_errors(runs, radiation=True)
        self.assertAlmostEqual(errors['space'][0, 0], .002)
        self.assertAlmostEqual(errors['time'][0, 0], .0002)
        self.assertAlmostEqual(errors['outer'][0, 0], .00002)
        self.assertAlmostEqual(errors['radiation'][0, 0], .0012)
        no_radiation = refinement_errors(runs, radiation=False)
        self.assertEqual(no_radiation['radiation'][0, 0], 0)

    def test_crank_nicolson_neutral_limits_and_independent_robin_stencil(self):
        from scipy.special import erfc, erfcx
        times = np.array([0.01, 0.1, 1.0, 10.0])
        radii = [1.0, 1.5, 3.0]
        for kappa in [None, 0.2, 1.0]:
            x = (np.array(radii)[:, None] - 1) / (2 * np.sqrt(times))
            expected = erfc(x) / np.array(radii)[:, None]
            if kappa is not None:
                expected = kappa / (1 + kappa) / np.array(radii)[:, None] * (
                    erfc(x) - np.exp(-x*x) * erfcx(x + (1+kappa)*np.sqrt(times)))
            for robin in ['ghost', 'onesided']:
                actual = solve(0.0, kappa, radii, list(times), 0.005, 50,
                               0.05, 0.01, theta=0.5, robin=robin)
                np.testing.assert_allclose(actual, expected, atol=0.00015, rtol=0)

    def test_invalid_discretization_options_rejected(self):
        for kwargs in [dict(theta=0.4), dict(robin='unknown')]:
            with self.assertRaises(ValueError):
                solve(0.0, 1.0, [1.0], [0.1], 0.02, 10, 0.01, 0.01, **kwargs)

    def test_neutral_absorbing_sphere_matches_exact_cdf(self):
        times = [0.01, 0.1, 1.0]
        radii = [1.0, 1.5, 3.0]
        actual = solve(0.0, None, radii, times, 0.02, 50.0, 0.005, 0.005)
        expected = np.array([[math.erfc((r - 1) / (2 * math.sqrt(t))) / r
                              for t in times] for r in radii])
        np.testing.assert_allclose(actual, expected, atol=0.0004, rtol=0)

    def test_radiation_contact_neutral_limit(self):
        actual = solve(0.0, 1.0, [1.0], [0.1], 0.01, 30.0, 0.001, 0.0025)
        self.assertAlmostEqual(actual[0, 0], 0.22319687310756076, delta=0.0003)

    def test_refinement_and_boundary_for_both_charge_signs(self):
        for charge in [-1.0, 1.0]:
            with self.subTest(charge=charge):
                a = solve(charge, 1.0, [1.0, 1.5, 3.0], [0.1, 1.0], 0.04, 30, 0.01, 0.01)
                b = solve(charge, 1.0, [1.0, 1.5, 3.0], [0.1, 1.0], 0.02, 40, 0.005, 0.005)
                self.assertLess(np.max(np.abs(a - b)), 0.002)


if __name__ == '__main__':
    unittest.main()
