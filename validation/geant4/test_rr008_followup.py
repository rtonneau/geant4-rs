import tempfile
import unittest
from pathlib import Path
from rr008_followup_consumer import HEADER, Incomplete, frequency, validate
from rr008_seed_audit import seed_pair


def fixture(path, replica, trials=1, reacted=False):
    lines = ['# schema: syn_scheduler_followup_v1; Geant4: 1141',
             f'# replica: {replica}; histories: {trials}; birth_ps: 1; horizon_ps: 1000000',
             ','.join(HEADER)]
    for o in range(3):
        for h in range(trials):
            origin = [4.1, -2.87, 1.23] if o == 2 else [0., 0., 0.]
            second = origin.copy()
            second[2 if o == 1 else 0] += 1.23
            ending = ['2', '1', *map(str, origin), *map(str, second), '0', 'NA', 'NA', 'NA', '1'] if reacted else ['NA'] * 13
            lines.append(','.join([*map(str, seed_pair(replica, o, h)), '11', str(o), str(h),
                                   'reaction' if reacted else 'censored', '.41', '.0022', '.007',
                                   *map(str, origin), *map(str, second), *ending]))
    lines.append(f'# complete: true; replica: {replica}; histories: {trials}; rows: {3*trials}')
    path.write_text('\n'.join(lines) + '\n', encoding='ascii')


class ContractTests(unittest.TestCase):
    def test_frequency_boundaries(self):
        for count, expected in [(0,'FAIL'), (258,'INCONCLUSIVE'), (259,'PASS'),
                                (438,'PASS'), (439,'INCONCLUSIVE'), (2048,'FAIL')]:
            self.assertEqual(frequency(count), expected)

    def test_smoke_schema_and_structure(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'data.csv'
            fixture(path, 1, reacted=True)
            self.assertEqual(validate(path, 1, 1), ([1, 1, 1], False))
            text = path.read_text().replace(',2,1,0.0,0.0,0.0,', ',1,1,0.0,0.0,0.0,', 1)
            path.write_text(text)
            self.assertTrue(validate(path, 1, 1)[1])
            path.write_text(path.read_text().replace('# complete:', '# broken:'))
            with self.assertRaises(Incomplete):
                validate(path, 1, 1)

    def test_bad_seed_is_invalid_complete_data(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'data.csv'
            fixture(path, 1)
            text = path.read_text().replace(str(seed_pair(1, 0, 0)[0]), '1', 1)
            path.write_text(text)
            with self.assertRaises(ValueError):
                validate(path, 1, 1)


if __name__ == '__main__':
    unittest.main()
