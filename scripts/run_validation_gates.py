#!/usr/bin/env python3
"""Fail-closed preflight and execution for the maintained Geant4 validation tier."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


REPO_ROOT = Path(__file__).resolve().parents[1]
GATES_PATH = REPO_ROOT / "validation" / "gates.json"
SCRATCH = REPO_ROOT / ".scratch" / "tests" / "2026-10-26__validation-evidence-and-ci"
EXPECTED_GATES = (
    ("M2-xs", "M2", "cross sections V2/V3", "dna-sim/geant4_reference", "v2_v3_cross_sections_per_volume_match_geant4"),
    ("M2-mfp", "M2", "mean free path V5, seed 1", "dna-sim/geant4_reference", "v5_first_interaction_mean_free_path_matches_geant4"),
    ("M2-track", "M2", "primary track with 100 eV cut V5, seed 1", "dna-sim/geant4_reference", "v5_primary_track_with_100_ev_cut_matches_geant4"),
    ("M2-seed2", "M2", "independent seed-2 consistency against seed 1 and Rust", "dna-sim/geant4_reference", "seed2_geant4_runs_are_consistent_with_seed1_and_with_rust"),
    ("M3-xs", "M3", "low-energy cross sections V2", "dna-sim/slowing_down_reference", "m3_v2_low_energy_cross_sections_match_geant4"),
    ("M3-cascade", "M3", "full-cascade slowing-down V5, spectra and radial profile", "dna-sim/slowing_down_reference", "m3_v5_full_slowing_down_matches_geant4"),
    ("M6-prechem", "M6", "RR-001 dissociation-only snapshot at 1 ps before IRT", "dna-sim/physchem_reference", "m6_prechem_counts_and_rms_match_geant4"),
)
EXPECTED_FIXTURES = {
    "M2-xs": ("xs_per_volume.csv", "provenance.toml"),
    "M2-mfp": ("mfp_first_interaction.csv", "provenance.toml"),
    "M2-track": ("track_cut100eV.csv", "provenance.toml"),
    "M2-seed2": ("mfp_first_interaction.csv", "track_cut100eV.csv", "mfp_first_interaction_seed2.csv", "track_cut100eV_seed2.csv", "provenance.toml"),
    "M3-xs": ("xs_lowenergy.csv", "provenance_m3.toml"),
    "M3-cascade": ("slow_sums.csv", "slow_spectra.csv", "slow_radial.csv", "provenance_m3.toml"),
    "M6-prechem": ("physchem_prechem.csv", "provenance_m6_prechem.toml"),
}
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


class GateError(Exception):
    """An invalid or unavailable gate input with an actionable diagnostic."""


def _safe_file(root: Path, relative: str, label: str) -> Path:
    if not isinstance(relative, str) or not relative:
        raise GateError(f"{label}: expected a nonempty repository-relative path")
    candidate = (root / relative).resolve()
    try:
        candidate.relative_to(root.resolve())
    except ValueError as exc:
        raise GateError(f"{label}: path escapes repository: {relative}") from exc
    return candidate


def _verify_file(root: Path, relative: str, expected: dict, label: str, *, text_file: bool = False) -> None:
    if not isinstance(expected, dict):
        raise GateError(f"{label}: invalid inventory entry")
    if set(expected) != {"path", "bytes", "sha256"} or expected["path"] != relative:
        raise GateError(f"{label}: invalid inventory identity for {relative}")
    size, digest = expected["bytes"], expected["sha256"]
    if not isinstance(size, int) or size <= 0 or not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
        raise GateError(f"{label}: invalid byte count or SHA-256 for {relative}")
    path = _safe_file(root, relative, label)
    try:
        content = path.read_bytes()
    except OSError as exc:
        raise GateError(f"{label}: required evidence unavailable at {path}: {exc}") from exc
    if not content:
        raise GateError(f"{label}: required evidence is empty at {path}")
    # Git checks text files out with either LF or CRLF. Inventory identities use
    # the canonical LF bytes stored in Git, without changing the evidence file.
    if text_file:
        content = content.replace(b"\r\n", b"\n")
    if len(content) != size:
        raise GateError(f"{label}: size mismatch at {path}: expected {size} bytes, found {len(content)}")
    actual = hashlib.sha256(content).hexdigest()
    if actual != digest:
        raise GateError(f"{label}: SHA-256 mismatch at {path}: expected {digest}, found {actual}")


def _parse_data_manifest(text: str) -> list[dict]:
    """Read this repository's deliberately simple TOML data manifest with stdlib only."""
    files: list[dict] = []
    current: dict | None = None
    for line_number, raw in enumerate(text.splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line == "[[file]]":
            current = {}
            files.append(current)
            continue
        if "=" not in line:
            raise GateError(f"data/MANIFEST.toml:{line_number}: malformed entry")
        key, value = (part.strip() for part in line.split("=", 1))
        if key in {"dataset", "version"} and current is None:
            continue
        if current is None or key not in {"path", "bytes", "sha256", "group"} or key in current:
            raise GateError(f"data/MANIFEST.toml:{line_number}: unexpected or duplicate field {key!r}")
        if value.startswith('"') and value.endswith('"'):
            current[key] = value[1:-1]
        elif key == "bytes" and value.isdecimal():
            current[key] = int(value)
        else:
            raise GateError(f"data/MANIFEST.toml:{line_number}: invalid value for {key!r}")
    if not files:
        raise GateError("data/MANIFEST.toml: no [[file]] entries")
    for index, entry in enumerate(files, 1):
        if not {"path", "bytes", "sha256", "group"} <= entry.keys():
            raise GateError(f"data/MANIFEST.toml: file entry {index} is incomplete")
    return files


def validate_manifest(manifest: object, root: Path = REPO_ROOT) -> None:
    if not isinstance(manifest, dict) or set(manifest) != {
        "schema_version", "description", "data_manifest", "gates", "fixtures", "historical_exclusions"
    }:
        raise GateError("validation/gates.json: invalid top-level schema")
    if manifest["schema_version"] != 1 or not isinstance(manifest["description"], str):
        raise GateError("validation/gates.json: unsupported schema_version or missing description")
    data_manifest = manifest["data_manifest"]
    if not isinstance(data_manifest, dict) or set(data_manifest) != {"path", "sha256"}:
        raise GateError("validation/gates.json: invalid data_manifest identity")
    if data_manifest["path"] != "data/MANIFEST.toml" or not SHA256_RE.fullmatch(str(data_manifest["sha256"])):
        raise GateError("validation/gates.json: invalid data/MANIFEST.toml identity")
    manifest_path = _safe_file(root, data_manifest["path"], "committed data manifest")
    try:
        data_bytes = manifest_path.read_bytes()
    except OSError as exc:
        raise GateError(f"committed data manifest unavailable at {manifest_path}: {exc}") from exc
    data_bytes = data_bytes.replace(b"\r\n", b"\n")
    if hashlib.sha256(data_bytes).hexdigest() != data_manifest["sha256"]:
        raise GateError(f"committed data manifest SHA-256 mismatch at {manifest_path}")
    data_entries = _parse_data_manifest(data_bytes.decode("utf-8"))
    seen_data: set[str] = set()
    for entry in data_entries:
        rel = "data/" + entry["path"]
        if rel in seen_data:
            raise GateError(f"data/MANIFEST.toml: duplicate data identity {rel}")
        seen_data.add(rel)
        _verify_file(root, rel, {"path": rel, "bytes": entry["bytes"], "sha256": entry["sha256"]}, "committed Geant4 data")

    gates = manifest["gates"]
    if not isinstance(gates, list) or len(gates) != len(EXPECTED_GATES):
        raise GateError("validation/gates.json: expected exactly the seven registered M2/M3/M6 gates")
    fixtures = manifest["fixtures"]
    if not isinstance(fixtures, dict):
        raise GateError("validation/gates.json: fixtures must be an object")
    for gate, expected in zip(gates, EXPECTED_GATES, strict=True):
        if not isinstance(gate, dict) or set(gate) != {"id", "milestone", "profile", "target", "test", "fixtures"}:
            raise GateError("validation/gates.json: malformed gate entry")
        actual = tuple(gate[key] for key in ("id", "milestone", "profile", "target", "test"))
        if actual != expected:
            raise GateError(f"validation/gates.json: invalid gate registration {actual!r}; expected {expected!r}")
        if not isinstance(gate["fixtures"], list) or not gate["fixtures"] or len(set(gate["fixtures"])) != len(gate["fixtures"]):
            raise GateError(f"validation/gates.json: invalid fixtures for {gate['id']}")
        if tuple(gate["fixtures"]) != EXPECTED_FIXTURES[gate["id"]]:
            raise GateError(f"validation/gates.json: wrong registered fixtures for {gate['id']}")
        for fixture in gate["fixtures"]:
            if fixture not in fixtures:
                raise GateError(f"validation/gates.json: {gate['id']} references unknown fixture {fixture!r}")
    for name, fixture in fixtures.items():
        if not isinstance(name, str) or Path(name).name != name:
            raise GateError(f"validation/gates.json: invalid fixture identity {name!r}")
        entry = fixture if isinstance(fixture, dict) else {}
        relative = entry.get("path")
        _verify_file(root, relative, entry, f"required fixture {name}", text_file=True)
    exclusions = manifest["historical_exclusions"]
    if not isinstance(exclusions, list) or not exclusions:
        raise GateError("validation/gates.json: historical_exclusions must document excluded tests")
    for exclusion in exclusions:
        if not isinstance(exclusion, dict) or set(exclusion) != {"test", "reason"} or not all(isinstance(exclusion[k], str) and exclusion[k] for k in ("test", "reason")):
            raise GateError("validation/gates.json: malformed historical exclusion")


def load_and_preflight(root: Path = REPO_ROOT) -> dict:
    path = root / "validation" / "gates.json"
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise GateError(f"invalid or unavailable gate inventory at {path}: {exc}") from exc
    validate_manifest(manifest, root)
    return manifest


def run_statistical(manifest: dict, root: Path = REPO_ROOT) -> bool:
    env = os.environ.copy()
    target_dir = root / ".scratch" / "tests" / "2026-10-26__validation-evidence-and-ci" / "cargo-target"
    target_dir.mkdir(parents=True, exist_ok=True)
    env["CARGO_TARGET_DIR"] = str(target_dir)
    passed = True
    for gate in manifest["gates"]:
        target, test = gate["target"].split("/", 1)
        command = [
            "cargo", "test", "--locked", "-p", target, "--test", test,
            gate["test"], "--", "--ignored", "--exact",
        ]
        print(f"[{gate['id']}] RUN {' '.join(command)}", flush=True)
        result = subprocess.run(command, cwd=root, env=env, check=False)
        status = "PASS" if result.returncode == 0 else f"FAIL (exit {result.returncode})"
        print(f"[{gate['id']}] {status}", flush=True)
        passed = passed and result.returncode == 0
    return passed


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tier", choices=("required", "statistical"), required=True)
    args = parser.parse_args(argv)
    try:
        manifest = load_and_preflight()
    except GateError as exc:
        print(f"FAIL evidence preflight: {exc}", file=sys.stderr)
        return 1
    print(f"PASS evidence preflight: {len(manifest['gates'])} gates, fixture hashes and committed data verified")
    if args.tier == "required":
        return 0
    return 0 if run_statistical(manifest) else 1


if __name__ == "__main__":
    raise SystemExit(main())
