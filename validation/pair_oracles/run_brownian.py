"""Retain the frozen Brownian schedule, source hashes and complete stdout/stderr.

This runs Rust independent trajectories only; it never runs Geant4. Output
files must be new. A completed executable is not a scientific PASS.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--threads", type=int, default=6)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    paths = [root / "Cargo.lock", root / "Cargo.toml"]
    paths += list((root / "crates/g4-chemistry/tests/pair_oracle").glob("*.rs"))
    paths += [root / "crates/g4-chemistry/tests/pair_brownian.rs"]
    paths += list((root / "crates/g4-chemistry/tests/fixtures").glob("pair*anchors.json"))
    paths += [Path(__file__).resolve(), root / "docs/validation/pair-case-matrix.md"]
    env = os.environ.copy()
    env.update(PAIR_ORACLE_MAX_PAIRS="1000000", PAIR_ORACLE_LEVELS="5",
               PAIR_ORACLE_RADIUS="16", PAIR_ORACLE_HORIZON="100",
               RAYON_NUM_THREADS=str(args.threads))
    env.pop("PAIR_ORACLE_CASE", None)
    command = ["cargo", "test", "--release", "-p", "g4-chemistry", "--test",
               "pair_brownian", "brownian", "--", "--nocapture", "--test-threads=1"]
    metadata = dict(started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                    command=command, platform=platform.platform(),
                    python=platform.python_version(),
                    rustc=subprocess.check_output(["rustc", "-Vv"], text=True),
                    git_head=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
                    dirty_status=subprocess.check_output(["git", "status", "--porcelain"], cwd=root, text=True),
                    environment={k:env[k] for k in ("PAIR_ORACLE_MAX_PAIRS", "PAIR_ORACLE_LEVELS",
                        "PAIR_ORACLE_RADIUS", "PAIR_ORACLE_HORIZON", "RAYON_NUM_THREADS")},
                    files={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
                    completed=False)
    provenance = output / "provenance.json"
    provenance.write_text(json.dumps(metadata, indent=2) + "\n")
    started = time.monotonic()
    with (output / "run.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen(command, cwd=root, env=env, stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT, text=True, encoding="utf-8")
        with (output / "estimates.jsonl").open("w", encoding="utf-8") as records:
            for line in process.stdout:
                log.write(line)
                log.flush()
                if line.startswith("R04_JSON "):
                    record = json.loads(line[len("R04_JSON "):])
                    records.write(json.dumps(record, separators=(",", ":")) + "\n")
                    records.flush()
                    if record["record"] == "summary":
                        print(record, flush=True)
        code = process.wait()
    metadata.update(exit_code=code, elapsed_seconds=time.monotonic()-started,
                    ended_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    records = [json.loads(line) for line in (output / "estimates.jsonl").read_text().splitlines()]
    metadata["completed"] = (code == 0 and len([r for r in records if r["record"] == "complete"]) == 2
                             and len([r for r in records if r["record"] == "summary"]) == 36)
    metadata["unchanged_sources"] = all(hashlib.sha256((root/p).read_bytes()).hexdigest()==sha
                                         for p,sha in metadata["files"].items())
    provenance.write_text(json.dumps(metadata, indent=2) + "\n")
    print(json.dumps({k: metadata[k] for k in ("completed","exit_code","elapsed_seconds","unchanged_sources")}))
    raise SystemExit(0 if metadata["completed"] and metadata["unchanged_sources"] else 1)


if __name__ == "__main__":
    main()
