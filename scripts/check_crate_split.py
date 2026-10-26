#!/usr/bin/env python3
"""Capture and compare exact, same-platform crate split regression snapshots."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "geant4-rs/crate-split-snapshot/v1"
SCRATCH = ROOT / ".scratch/tests/2026-10-26__crate-domain-ownership"


def command(args: list[str], *, root: Path = ROOT) -> str:
    return subprocess.check_output(args, cwd=root, text=True, encoding="utf-8").strip()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def input_identity(root: Path = ROOT) -> dict:
    # Every committed data file is covered, including the embedded chemistry tables.
    paths = [root / "Cargo.lock"] + sorted(p for p in (root / "data").rglob("*") if p.is_file())
    return {p.relative_to(root).as_posix(): sha256(p) for p in paths}


def toolchain_identity() -> dict:
    return {
        "rustc": command(["rustc", "-vV"]),
        "cargo": command(["cargo", "-V"]),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "build_profile": "dev",
        "build_environment": {key: os.environ.get(key) for key in (
            "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_TARGET", "RUSTC",
            "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTUP_TOOLCHAIN",
        )},
    }


def validate_revision(revision: object, root: Path = ROOT) -> None:
    if not isinstance(revision, str) or not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("metadata.revision must be a full Git commit ID")
    resolved = command(["git", "rev-parse", "--verify", revision + "^{commit}"], root=root)
    if resolved != revision:
        raise ValueError("metadata.revision is not a commit in this checkout")
    if subprocess.run(["git", "merge-base", "--is-ancestor", revision, "HEAD"], cwd=root,
                      stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode:
        raise ValueError("metadata.revision is not an ancestor of this checkout's HEAD")


def difference(a: object, b: object, path: str = "$") -> str | None:
    """Exact typed comparison; never sort arrays or coerce numbers."""
    if type(a) is not type(b):
        return f"{path}: type changed"
    if isinstance(a, dict):
        if a.keys() != b.keys():
            return f"{path}: keys changed"
        for key in a:
            found = difference(a[key], b[key], f"{path}.{key}")
            if found:
                return found
    elif isinstance(a, list):
        if len(a) != len(b):
            return f"{path}: length changed"
        for index, (left, right) in enumerate(zip(a, b)):
            found = difference(left, right, f"{path}[{index}]")
            if found:
                return found
    elif a != b:
        return f"{path}: value changed ({a!r} -> {b!r})"
    return None


def compare(baseline: dict, candidate: dict, *, root: Path = ROOT) -> None:
    for document in (baseline, candidate):
        if document.get("schema") != SCHEMA:
            raise ValueError("unsupported snapshot schema")
        if set(document.get("metadata", {})) != {"revision", "package"}:
            raise ValueError("metadata must contain exactly revision and package")
        if not document.get("inputs") or not document.get("toolchain") or not document.get("runs"):
            raise ValueError("missing inputs, toolchain or scientific runs")
        validate_revision(document["metadata"]["revision"], root)
    # Only this individually named field is allowed to differ, after checkout validation.
    # No wildcard metadata stripping; package, inputs, compiler and all science remain exact.
    left = {**baseline, "metadata": {**baseline["metadata"], "revision": candidate["metadata"]["revision"]}}
    changed = difference(left, candidate)
    if changed:
        raise ValueError(changed)


def capture(directory: Path, package: str, *, root: Path = ROOT) -> None:
    revision = command(["git", "rev-parse", "HEAD"], root=root)
    directory = directory.resolve()
    parent = (root / "target/crate-split-baseline").resolve()
    if directory.parent != parent or not directory.name.startswith(revision + "-"):
        raise ValueError(f"capture must be a new target/crate-split-baseline/{revision}-<unique-run-id> directory")
    if len(directory.name) <= len(revision) + 1:
        raise ValueError("capture directory needs a unique run ID")
    if subprocess.run(["git", "check-ignore", "--quiet", str(directory)], cwd=root).returncode:
        raise ValueError("capture directory must be ignored by Git")
    # mkdir is exclusive. An incomplete attempt is kept and never reused or overwritten.
    directory.mkdir(parents=True, exist_ok=False)
    inputs = input_identity(root)
    toolchain = toolchain_identity()
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(SCRATCH / "cargo-target")
    for profile in ("physics", "physchem"):
        args = ["cargo", "run", "--locked", "--quiet", "-p", package,
                "--example", "crate_split_snapshot", "--", "--" + profile]
        result = subprocess.run(args, cwd=root, env=env, check=True, capture_output=True, text=True, encoding="utf-8")
        document = json.loads(result.stdout)
        if document.get("schema") != SCHEMA or document.get("profile") != profile or len(document.get("runs", [])) != 3:
            raise ValueError("example emitted an incomplete snapshot")
        document.update(metadata={"revision": revision, "package": package}, inputs=inputs, toolchain=toolchain)
        if input_identity(root) != inputs or command(["git", "rev-parse", "HEAD"], root=root) != revision:
            raise ValueError("checkout inputs or revision changed during capture")
        path = directory / (profile + ".json")
        with path.open("x", encoding="utf-8", newline="\n") as output:
            json.dump(document, output, indent=2, ensure_ascii=False, allow_nan=False)
            output.write("\n")
        print(f"{profile}: {path} sha256={sha256(path)}", flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--capture", type=Path)
    mode.add_argument("--compare", nargs=2, type=Path, metavar=("BASELINE", "CANDIDATE"))
    parser.add_argument("--package", default="dna-sim")
    args = parser.parse_args()
    try:
        if args.capture:
            capture(args.capture, args.package)
        else:
            documents = [json.loads(p.read_text(encoding="utf-8")) for p in args.compare]
            compare(*documents)
            print("PASS: exact scientific, input and toolchain identity (validated revision may differ)")
    except (OSError, ValueError, subprocess.CalledProcessError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        if isinstance(exc, subprocess.CalledProcessError) and exc.stderr:
            print(exc.stderr, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
