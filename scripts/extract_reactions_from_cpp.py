#!/usr/bin/env python3
"""Extract the Geant4 11.4.1 option3 reaction table from its C++ text.

The extractor reads only executable statements in ``ConstructReactionTable``.
Adjacent comments are retained as audit evidence but never replace executable
rates or products.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


SOURCE = Path("source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc")
EXPECTED_REACTIONS = 72


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("geant4_root", type=Path)
    parser.add_argument("--check", type=Path)
    parser.add_argument("--write-json", type=Path)
    parser.add_argument("--write-toml", type=Path)
    return parser.parse_args()


def extract(root: Path) -> dict[str, object]:
    path = root / SOURCE
    lines = path.read_text(encoding="utf-8", errors="strict").splitlines()
    start = next(i for i, line in enumerate(lines) if "ConstructReactionTable(" in line)
    end = next(i for i in range(start + 1, len(lines)) if "ConstructTimeStepModel(" in lines[i])
    body = lines[start:end]

    variable_tags: dict[str, str] = {}
    declarations = "\n".join(body)
    for match in re.finditer(
        r"G4MolecularConfiguration\*\s+(\w+)\s*=\s*"
        r"G4MoleculeTable::Instance\(\)->GetConfiguration\(\"([^\"]+)\"\)",
        declarations,
    ):
        variable_tags[match.group(1)] = match.group(2)

    reactions: list[dict[str, object]] = []
    source_class = ""
    pending_comment = ""
    statement = ""
    statement_line = 0
    current: dict[str, object] | None = None

    for offset, raw_line in enumerate(body):
        line_no = start + offset + 1
        stripped = raw_line.strip()
        section = re.fullmatch(r"//\s*Type\s+(I|II|III|IV|VI)\s*(?://)?", stripped)
        if section:
            source_class = section.group(1)
            pending_comment = ""
            continue
        if stripped == "// Scavenging reactions":
            source_class = "scavenging"
            pending_comment = ""
            continue
        if stripped.startswith("//"):
            text = stripped[2:].strip()
            if text and set(text) != {"-"} and not text.startswith("Type "):
                pending_comment = text
            continue
        code = re.sub(r"//.*$", "", raw_line).strip()
        if not code:
            continue
        if not statement:
            statement_line = line_no
        statement = f"{statement} {code}".strip()
        if ";" not in statement:
            continue
        pieces = statement.split(";")
        statement = pieces.pop().strip()
        for piece in pieces:
            stmt = piece.strip()
            constructor = re.search(
                r"new\s+G4DNAMolecularReactionData\(\s*(.*?)\s*,\s*(\w+)\s*,\s*(\w+)\s*\)$",
                stmt,
            )
            if constructor:
                rate_expr, left_var, right_var = constructor.groups()
                coefficient_match = re.match(r"([0-9]+(?:\.[0-9]*)?(?:e[+-]?\d+)?)", rate_expr, re.I)
                if coefficient_match is None:
                    raise ValueError(f"line {statement_line}: cannot parse rate {rate_expr!r}")
                coefficient_literal = coefficient_match.group(1)
                bimolecular = "m3" in rate_expr
                comment_rate = None
                comment_matches = re.findall(
                    r"([0-9]+(?:\.[0-9]*)?(?:e[+-]?\d+)?)\s*(?:/\s*s|(?:M-1|M\^-1)\s*s-1)",
                    pending_comment,
                    re.I,
                )
                if comment_matches:
                    comment_rate = comment_matches[-1]
                current = {
                    "id": len(reactions) + 1,
                    "source_line": statement_line,
                    "source_class": source_class,
                    "reactants": [variable_tags[left_var], variable_tags[right_var]],
                    "products": [],
                    "rate_order": "bimolecular" if bimolecular else "first_order",
                    "rate_literal": coefficient_literal,
                    "rate_value": float(coefficient_literal),
                    "rate_unit": "m3_per_mol_s" if bimolecular else "per_s",
                    "kinetic_intent": (
                        "first_order" if source_class == "VI" else
                        "pseudo_first_order" if source_class == "scavenging" else
                        "totally_diffusion_controlled"
                    ),
                    "set_reaction_type": False,
                    "comment": pending_comment,
                    "comment_rate_literal": comment_rate,
                }
                pending_comment = ""
                continue
            product = re.fullmatch(r"reactionData->AddProduct\((\w+)\)", stmt)
            if product and current is not None:
                current["products"].append(variable_tags[product.group(1)])
                continue
            if stmt.startswith("SetReactionType(reactionData") and current is not None:
                current["set_reaction_type"] = True
                current["kinetic_intent"] = "partially_diffusion_controlled"
                continue
            if stmt == "theReactionTable->SetReaction(reactionData)" and current is not None:
                reactions.append(current)
                current = None
        statement_line = line_no if statement else 0

    if current is not None:
        raise ValueError("unterminated reaction record")
    if len(reactions) != EXPECTED_REACTIONS:
        raise ValueError(f"expected {EXPECTED_REACTIONS} reactions, found {len(reactions)}")
    if reactions[52]["source_class"] != "VI":
        raise ValueError("reaction 53 must be the Type VI entry")
    return {
        "geant4_release": "11.4.1",
        "source": SOURCE.as_posix(),
        "reactions": reactions,
    }


def json_text(data: dict[str, object]) -> str:
    return json.dumps(data, indent=2, ensure_ascii=False) + "\n"


def toml_text(data: dict[str, object]) -> str:
    out = [
        '# Executable reaction literals extracted from Geant4 11.4.1.',
        f'# Source: {data["source"]}:96-666',
        '# Adjacent comments are audit evidence only; executable literals are authoritative.',
        '',
        'geant4_release = "11.4.1"',
        f'source = "{data["source"]}"',
    ]
    for reaction in data["reactions"]:
        assert isinstance(reaction, dict)
        out.extend(["", "[[reaction]]"])
        out.append(f'id = {reaction["id"]}')
        out.append(f'source_line = {reaction["source_line"]}')
        out.append(f'source_class = {json.dumps(reaction["source_class"])}')
        out.append(f'reactants = {json.dumps(reaction["reactants"], ensure_ascii=False)}')
        out.append(f'products = {json.dumps(reaction["products"], ensure_ascii=False)}')
        out.append(f'kinetic_intent = {json.dumps(reaction["kinetic_intent"])}')
        out.append(f'rate_order = {json.dumps(reaction["rate_order"])}')
        rate_key = "rate_m3_per_mol_s" if reaction["rate_order"] == "bimolecular" else "rate_per_s"
        out.append(f'{rate_key} = {reaction["rate_literal"]}')
        out.append(f'set_reaction_type = {str(reaction["set_reaction_type"]).lower()}')
        if reaction["comment"]:
            out.append(f'comment = {json.dumps(reaction["comment"], ensure_ascii=False)}')
    return "\n".join(out) + "\n"


def check(path: Path, expected: str) -> None:
    actual = path.read_text(encoding="utf-8")
    if actual != expected:
        raise SystemExit(f"{path} is stale; regenerate it from Geant4 11.4.1")


def main() -> None:
    args = parse_args()
    data = extract(args.geant4_root)
    rendered_json = json_text(data)
    if args.write_json:
        args.write_json.parent.mkdir(parents=True, exist_ok=True)
        args.write_json.write_text(rendered_json, encoding="utf-8", newline="\n")
    if args.write_toml:
        args.write_toml.parent.mkdir(parents=True, exist_ok=True)
        args.write_toml.write_text(toml_text(data), encoding="utf-8", newline="\n")
    if args.check:
        check(args.check, rendered_json)
        print(f"72 extracted reactions match {args.check}")
    if not (args.write_json or args.write_toml or args.check):
        sys.stdout.write(rendered_json)


if __name__ == "__main__":
    main()
