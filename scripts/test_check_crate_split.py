"""Mutation tests for the exact crate-ownership snapshot comparator."""
import copy
import unittest
from unittest.mock import patch

import check_crate_split as checker


class SnapshotTests(unittest.TestCase):
    def setUp(self):
        self.revision = checker.command(["git", "rev-parse", "HEAD"])
        self.baseline = {
            "schema": checker.SCHEMA,
            "metadata": {"revision": self.revision, "package": "dna-sim"},
            "inputs": {"Cargo.lock": "abc", "data/MANIFEST.toml": "def"},
            "toolchain": {"rustc": "same compiler"},
            "runs": [{"energy_ev": "0x4059000000000000", "species": ["H3Op", "°OH"],
                      "species_csv_header": "species,x_nm,y_nm,z_nm,time_ps"}],
        }
        self.candidate = copy.deepcopy(self.baseline)

    def test_equal_pair(self):
        checker.compare(self.baseline, self.candidate)

    def test_checkout_validated_revision_change(self):
        self.baseline["metadata"]["revision"] = checker.command(["git", "rev-parse", "HEAD~1"])
        checker.compare(self.baseline, self.candidate)

    def test_one_bit_change(self):
        self.candidate["runs"][0]["energy_ev"] = "0x4059000000000001"
        with self.assertRaisesRegex(ValueError, "energy_ev"):
            checker.compare(self.baseline, self.candidate)

    def test_species_order_change(self):
        self.candidate["runs"][0]["species"].reverse()
        with self.assertRaisesRegex(ValueError, "species"):
            checker.compare(self.baseline, self.candidate)

    def test_header_change(self):
        self.candidate["runs"][0]["species_csv_header"] += ",extra"
        with self.assertRaisesRegex(ValueError, "header"):
            checker.compare(self.baseline, self.candidate)

    def test_input_changes(self):
        for name in self.baseline["inputs"]:
            candidate = copy.deepcopy(self.baseline)
            candidate["inputs"][name] += "changed"
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "inputs"):
                checker.compare(self.baseline, candidate)

    def test_toolchain_change(self):
        self.candidate["toolchain"]["rustc"] = "other compiler"
        with self.assertRaisesRegex(ValueError, "toolchain"):
            checker.compare(self.baseline, self.candidate)

    def test_unvalidated_revision_rejected(self):
        self.candidate["metadata"]["revision"] = "0" * 40
        with self.assertRaises((ValueError, checker.subprocess.CalledProcessError)):
            checker.compare(self.baseline, self.candidate)

    def test_unlisted_metadata_is_not_ignored(self):
        self.candidate["metadata"]["anything"] = "changed"
        with self.assertRaisesRegex(ValueError, "metadata"):
            checker.compare(self.baseline, self.candidate)

    def test_revision_validation_runs_even_for_equal_pair(self):
        with patch.object(checker, "validate_revision", side_effect=ValueError("invalid checkout")):
            with self.assertRaisesRegex(ValueError, "invalid checkout"):
                checker.compare(self.baseline, self.candidate)


if __name__ == "__main__":
    unittest.main()
