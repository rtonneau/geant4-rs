"""Fail-closed tests for the inventory used by run_validation_gates.py."""

from __future__ import annotations

import json
import shutil
import tempfile
import unittest
from pathlib import Path

import run_validation_gates as gates


class GateInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        gates.SCRATCH.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="gate-inventory-", dir=gates.SCRATCH)
        self.root = Path(self.temp.name)
        inventory = json.loads(gates.GATES_PATH.read_text(encoding="utf-8"))
        required = {entry["path"] for entry in inventory["fixtures"].values()}
        required.add(inventory["data_manifest"]["path"])
        data_manifest = gates.REPO_ROOT / inventory["data_manifest"]["path"]
        for entry in gates._parse_data_manifest(data_manifest.read_text(encoding="utf-8")):
            required.add("data/" + entry["path"])
        for relative in required:
            source = gates.REPO_ROOT / relative
            destination = self.root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, destination)
        inventory_path = self.root / "validation" / "gates.json"
        inventory_path.parent.mkdir(parents=True, exist_ok=True)
        inventory_path.write_text(json.dumps(inventory), encoding="utf-8")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def read_manifest(self) -> dict:
        return json.loads((self.root / "validation" / "gates.json").read_text(encoding="utf-8"))

    def write_manifest(self, manifest: dict) -> None:
        (self.root / "validation" / "gates.json").write_text(json.dumps(manifest), encoding="utf-8")

    def test_unchanged_committed_fixtures_pass(self) -> None:
        gates.validate_manifest(self.read_manifest(), self.root)

    def test_removed_fixture_fails_with_actionable_path(self) -> None:
        path = self.root / "validation" / "reference" / "slow_radial.csv"
        path.unlink()
        with self.assertRaisesRegex(gates.GateError, "slow_radial.csv"):
            gates.validate_manifest(self.read_manifest(), self.root)

    def test_changed_recorded_hash_fails(self) -> None:
        manifest = self.read_manifest()
        manifest["fixtures"]["xs_per_volume.csv"]["sha256"] = "0" * 64
        self.write_manifest(manifest)
        with self.assertRaisesRegex(gates.GateError, "SHA-256 mismatch"):
            gates.validate_manifest(self.read_manifest(), self.root)

    def test_removed_registered_fixture_identity_fails(self) -> None:
        manifest = self.read_manifest()
        cascade = next(gate for gate in manifest["gates"] if gate["id"] == "M3-cascade")
        cascade["fixtures"].remove("slow_radial.csv")
        del manifest["fixtures"]["slow_radial.csv"]
        with self.assertRaisesRegex(gates.GateError, "wrong registered fixtures for M3-cascade"):
            gates.validate_manifest(manifest, self.root)

    def test_invalid_inventory_schema_fails(self) -> None:
        inventory_path = self.root / "validation" / "gates.json"
        inventory_path.write_text("{not valid JSON", encoding="utf-8")
        with self.assertRaisesRegex(gates.GateError, "invalid or unavailable gate inventory"):
            gates.load_and_preflight(self.root)

    def test_perturbed_valid_fixture_fails(self) -> None:
        path = self.root / "validation" / "reference" / "xs_per_volume.csv"
        path.write_bytes(path.read_bytes() + b"\n")
        with self.assertRaisesRegex(gates.GateError, "size mismatch|SHA-256 mismatch"):
            gates.validate_manifest(self.read_manifest(), self.root)

    def test_extra_or_reordered_statistical_registration_fails(self) -> None:
        manifest = self.read_manifest()
        manifest["gates"].reverse()
        with self.assertRaisesRegex(gates.GateError, "invalid gate registration"):
            gates.validate_manifest(manifest, self.root)


if __name__ == "__main__":
    unittest.main()
