"""Audit archived R05-C1 evidence with Python's standard library; never sample.

Run from any directory with Python >=3.10. Exit 0 means the archived evidence
is intact and reproduces its reported verdict, not that scientific G2 passed.
Artifact hashes use UTF-8/LF bytes to survive Git's Windows line conversion.
"""

import argparse
from collections import Counter
import csv
from decimal import Decimal, localcontext
import hashlib
import io
import itertools
import json
from pathlib import Path


SPECIES = (
    "H3Op", "°OH", "OHm", "e_aq", "H", "H2", "H2O2", "HO2°",
    "HO2m", "Oxy", "Om", "O2", "O2m", "O3", "O3m",
)
ENSEMBLES = ("Clustered", "Dilute")
POLICIES = ("all-pairs", "1", "2", "4")
TIMES = ("10", "100", "1000", "10000", "100000", "1000000")
COMPARISONS = ("1/all-pairs", "2/all-pairs", "4/all-pairs", "1/2", "1/4", "2/4")
BATCH_HEADER = ("ensemble", "batch", "policy", "time_ps", "histories", *SPECIES)
ENDPOINT_HEADER = (
    "ensemble", "comparison", "time_ps", "species", "verdict", "scale",
    "left_mean", "reference_mean", "lower", "upper", "reason",
)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def canonical_bytes(path):
    return path.read_bytes().replace(b"\r\n", b"\n")


def read_rows(path, expected_header):
    reader = csv.DictReader(io.StringIO(canonical_bytes(path).decode("utf-8")))
    require(tuple(reader.fieldnames or ()) == expected_header, f"Wrong schema: {path.name}")
    rows = list(reader)
    require(all(None not in row and None not in row.values() for row in rows),
            f"Missing/extra fields: {path.name}")
    return rows


def audit(folder):
    provenance = json.loads((folder / "provenance.json").read_text(encoding="utf-8"))
    require(provenance["schema"] == "geant4-rs/r05-c1-evidence/v1", "Unknown provenance schema")
    require(provenance["protocol_commit"] == "c7188b35bd06420e1ba914c5f404e0c2c2fcfc5b",
            "Wrong preregistration commit")
    for name, path in (("batches.csv", folder / "batches.csv"),
                       ("endpoints.csv", folder / "endpoints.csv"),
                       ("audit_r05_c1.py", Path(__file__))):
        data = canonical_bytes(path)
        identity = provenance["artifacts"][name]
        require(len(data) == identity["bytes_lf_utf8"], f"Size mismatch: {name}")
        require(hashlib.sha256(data).hexdigest() == identity["sha256_lf_utf8"],
                f"SHA-256 mismatch: {name}")

    batches = {}
    for row in read_rows(folder / "batches.csv", BATCH_HEADER):
        key = row["ensemble"], int(row["batch"]), row["policy"], row["time_ps"]
        require(key not in batches, f"Duplicate batch: {key}")
        require(row["histories"] == "1024", f"Incomplete batch: {key}")
        require(all(row[s].isascii() and row[s].isdigit() for s in SPECIES),
                f"Non-integer count: {key}")
        counts = [int(row[s]) for s in SPECIES]
        require(sum(counts) <= 1024 * 100_000, f"Impossible retained population: {key}")
        batches[key] = dict(zip(SPECIES, (Decimal(n) / 1024 for n in counts)))
    expected_batches = set(itertools.product(ENSEMBLES, range(64), POLICIES, TIMES))
    require(set(batches) == expected_batches, "Missing/extra batch grid entries")

    categories = Counter()
    seen = set()
    max_error = Decimal(0)
    q = Decimal("2.66") ** 2 / 64
    band = Decimal("0.01")
    for row in read_rows(folder / "endpoints.csv", ENDPOINT_HEADER):
        ensemble, comparison, time, species = key = (
            row["ensemble"], row["comparison"], row["time_ps"], row["species"]
        )
        require(key not in seen, f"Duplicate endpoint: {key}")
        seen.add(key)
        lp, rp = comparison.split("/")
        x = [batches[ensemble, b, lp, time][species] for b in range(64)]
        y = [batches[ensemble, b, rp, time][species] for b in range(64)]
        diff = [a - b for a, b in zip(x, y)]
        mx, my, md = sum(x) / 64, sum(y) / 64, sum(diff) / 64
        require(mx == Decimal(row["left_mean"]) and my == Decimal(row["reference_mean"]),
                f"Mean mismatch: {key}")
        scale = "Absolute" if my < Decimal("0.1") else "Relative"
        require(row["scale"] == scale, f"Wrong interval units: {key}")
        interval = None
        if len(set(diff)) == 1:
            expected = "Inconclusive" if diff[0] == 0 else "Fail"
            reason = ("no observed paired differences" if diff[0] == 0
                      else "constant nonzero paired difference")
        else:
            vd = sum((v - md) ** 2 for v in diff) / 63
            vy = sum((v - my) ** 2 for v in y) / 63
            cov = sum((a - md) * (b - my) for a, b in zip(diff, y)) / 63
            if scale == "Absolute":
                interval = md - (q * vd).sqrt(), md + (q * vd).sqrt()
            else:
                # Original preregistered quadratic, independent of Rust's
                # regression-based evaluation; precision prevents cancellation.
                a = my * my - q * vy
                b = -2 * md * my + 2 * q * cov
                c = md * md - q * vd
                if a > 0:
                    disc = b * b - 4 * a * c
                    require(disc >= 0, f"Invalid Fieller discriminant: {key}")
                    interval = (-b - disc.sqrt()) / (2 * a), (-b + disc.sqrt()) / (2 * a)
            if interval is None:
                expected, reason = "Inconclusive", "unbounded Fieller interval"
            else:
                low, high = interval
                expected = ("Pass" if low >= -band and high <= band else
                            "Fail" if low > band or high < -band else "Inconclusive")
                reason = "interval inclusion"
        if interval is None:
            require(row["lower"] == row["upper"] == "", f"Spurious interval: {key}")
        else:
            reported = Decimal(row["lower"]), Decimal(row["upper"])
            require(all(v.is_finite() for v in reported), f"Non-finite interval: {key}")
            error = max(abs(a - b) for a, b in zip(interval, reported))
            max_error = max(max_error, error)
            # Numerical audit tolerance only, not the scientific effect band.
            require(error < Decimal("1e-12"), f"Interval mismatch: {key}")
        require(row["verdict"] == expected and row["reason"] == reason,
                f"Verdict/reason mismatch: {key}")
        categories[expected] += 1
    require(seen == set(itertools.product(ENSEMBLES, COMPARISONS, TIMES, SPECIES)),
            "Missing/extra endpoint grid entries")
    require(dict(categories) == provenance["endpoint_categories"], "Summary mismatch")
    scientific = next(v for v in ("Invalid", "Fail", "Inconclusive", "Pass") if categories[v])
    require(scientific == provenance["scientific_verdict"], "Scientific verdict mismatch")
    print("Evidence audit PASS: 3,072 batch rows / 46,080 count sums; 1,080 endpoint verdicts.")
    print("60-digit Decimal quadratic audit:", dict(categories))
    print("Maximum absolute interval-endpoint discrepancy:", max_error)
    print(f"Scientific verdict: {scientific.upper()}; G2 remains blocked.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence-dir", type=Path,
                        default=Path(__file__).parent / "evidence" / "r05_c1")
    args = parser.parse_args()
    with localcontext() as context:
        context.prec = 60
        audit(args.evidence_dir)
