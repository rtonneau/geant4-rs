#!/usr/bin/env python3
"""Independent arithmetic audit of the extracted option3 reaction table."""

from __future__ import annotations

import argparse
import collections
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
EXTRACTION = ROOT / "crates/dna-chemistry/tests/fixtures/cpp_reactions.json"

# (hydrogen atoms, oxygen atoms, charge). This table is deliberately independent of Rust.
COMPOSITION = {
    "H3Op": (3, 1, 1),
    "°OH": (1, 1, 0),
    "OHm": (1, 1, -1),
    "e_aq": (0, 0, -1),
    "H": (1, 0, 0),
    "H2": (2, 0, 0),
    "H2O2": (2, 2, 0),
    "HO2°": (1, 2, 0),
    "HO2m": (1, 2, -1),
    "Oxy": (0, 1, 0),
    "Om": (0, 1, -1),
    "O2": (0, 2, 0),
    "O2m": (0, 2, -1),
    "O3": (0, 3, 0),
    "O3m": (0, 3, -1),
    "H2O(B)": (2, 1, 0),
    "H3Op(B)": (3, 1, 1),
    "OHm(B)": (1, 1, -1),
    "NoneM": (0, 0, 0),
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", type=Path)
    parser.add_argument("--write", type=Path)
    return parser.parse_args()


def total(names: list[str]) -> tuple[int, int, int]:
    values = [COMPOSITION[name] for name in names]
    return tuple(sum(value[i] for value in values) for i in range(3))


def evaluate() -> dict[str, object]:
    extraction = json.loads(EXTRACTION.read_text(encoding="utf-8"))
    reactions = extraction["reactions"]
    if len(reactions) != 72:
        raise ValueError(f"expected 72 extracted reactions, found {len(reactions)}")

    balances = []
    anomaly_ids = []
    for reaction in reactions:
        reactants = total(reaction["reactants"])
        products = total(reaction["products"])
        implicit_water_products = reactants[1] - products[1]
        balanced = (
            reactants[2] == products[2]
            and reactants[0] - products[0] == 2 * implicit_water_products
        )
        if not balanced:
            anomaly_ids.append(reaction["id"])
            implicit_water_products = 0
        balances.append(
            {
                "id": reaction["id"],
                "reactant_composition": list(reactants),
                "product_composition": list(products),
                "implicit_water_products": implicit_water_products,
                "balanced": balanced,
            }
        )

    rate_orders = collections.Counter(reaction["rate_order"] for reaction in reactions)
    classes = collections.Counter(reaction["source_class"] for reaction in reactions)
    discrepancy_ids = [
        reaction["id"]
        for reaction in reactions
        if reaction["comment_rate_literal"] is not None
        and float(reaction["comment_rate_literal"]) != reaction["rate_value"]
    ]
    if anomaly_ids != [53]:
        raise ValueError(f"expected reaction 53 as the sole imbalance, found {anomaly_ids}")
    if discrepancy_ids != [59, 63, 65, 68, 70]:
        raise ValueError(f"unexpected comment/rate discrepancies: {discrepancy_ids}")
    return {
        "reaction_count": len(reactions),
        "rate_order_counts": dict(sorted(rate_orders.items())),
        "source_class_counts": dict(sorted(classes.items())),
        "set_reaction_type_count": sum(reaction["set_reaction_type"] for reaction in reactions),
        "rate_comment_discrepancy_ids": discrepancy_ids,
        "anomaly_ids": anomaly_ids,
        "balances": balances,
    }


def render(data: dict[str, object]) -> str:
    return json.dumps(data, indent=2, ensure_ascii=False, sort_keys=True) + "\n"


def main() -> None:
    args = parse_args()
    output = render(evaluate())
    if args.write:
        args.write.parent.mkdir(parents=True, exist_ok=True)
        args.write.write_text(output, encoding="utf-8", newline="\n")
    if args.check:
        if args.check.read_text(encoding="utf-8") != output:
            raise SystemExit(f"{args.check} is stale; regenerate it with scripts/hand_reactions.py")
        print(f"independent evaluation matches {args.check}")
    if not (args.write or args.check):
        sys.stdout.write(output)


if __name__ == "__main__":
    main()
