"""Audit prospective RR-008 follow-up streams without running Geant4."""

import hashlib
from pathlib import Path
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1] / "reference"
MOD1 = 2147483562
MOD2 = 2147483398


def seed_pair(replica, orientation, history):
    key = f"g4rs-rr008-followup-v1|{replica}|{orientation}|{history}"
    digest = hashlib.sha256(key.encode("ascii")).digest()
    return (1 + int.from_bytes(digest[:16], "big") % MOD1,
            1 + int.from_bytes(digest[16:], "big") % MOD2)


def main():
    prospective = [seed_pair(r, o, h) for r in (1, 2) for o in range(3)
                   for h in range(2048)]
    if len(set(prospective)) != 12288:
        raise ValueError("prospective seed collision")
    historical = set()
    for replica in (1, 2):
        with (ROOT / f"provenance_syn_scheduler_1141_r{replica}.toml").open("rb") as f:
            provenance = tomllib.load(f)
        data = (ROOT / f"syn_scheduler_1141_r{replica}.csv").read_bytes()
        if hashlib.sha256(data).hexdigest() != provenance["csv_sha256"]:
            raise ValueError("historical data hash mismatch")
        if provenance["completion_rows"] != 288 or provenance["trials_per_setting"] != 32:
            raise ValueError("unexpected historical campaign dimensions")
        for row in range(288):
            seed = provenance["seed"] + row
            historical.add((seed % MOD1 + 1, seed // MOD1 + 1))
    if historical.intersection(prospective):
        raise ValueError("historical seed overlap")
    print(f"12288 unique prospective pairs; 0 overlaps with {len(historical)} distinct historical pairs")
    if len(sys.argv) == 4 and sys.argv[1] == "--manifest":
        replica = int(sys.argv[2])
        if replica not in (1, 2):
            raise ValueError("replica must be 1 or 2")
        with open(sys.argv[3], "x", encoding="ascii", newline="\n") as output:
            for first, second in prospective[(replica - 1) * 6144:replica * 6144]:
                output.write(f"{first} {second}\n")
    elif len(sys.argv) != 1:
        raise ValueError("usage: rr008_seed_audit.py [--manifest REPLICA OUTPUT]")


if __name__ == "__main__":
    main()
