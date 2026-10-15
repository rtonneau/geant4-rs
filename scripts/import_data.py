#!/usr/bin/env python3
"""Copy the Geant4 DNA data files listed in data/MANIFEST.toml into data/ of this repo.

Run locally, then commit and push:
  python scripts/import_data.py "C:\\DEV\\GEANT4\\Geant4-ressources-data\\G4EMLOW8.8"

Each file is copied byte-for-byte to data/<path> (e.g. data/dna/sigma_elastic_e_champion.dat)
and verified against the manifest (size + SHA-256). A source file that does not match the
manifest is NOT copied and the script exits with code 1. Existing destination files that
already match are left untouched. Requires Python 3.11+ (tomllib).
"""
import argparse, hashlib, os, shutil, sys, tomllib

def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()

def matches(path, entry):
    return (os.path.isfile(path) and os.path.getsize(path) == entry["bytes"]
            and sha256(path) == entry["sha256"])

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("g4emlow_dir", help="G4EMLOW8.8 folder (contains dna/)")
    ap.add_argument("--manifest", default="data/MANIFEST.toml")
    ap.add_argument("--dest", default="data")
    a = ap.parse_args()
    with open(a.manifest, "rb") as f:
        manifest = tomllib.load(f)
    copied = unchanged = 0
    problems = []
    for entry in manifest["file"]:
        rel = entry["path"]
        src = os.path.join(a.g4emlow_dir, *rel.split("/"))
        dst = os.path.join(a.dest, *rel.split("/"))
        if matches(dst, entry):
            unchanged += 1
            continue
        if not os.path.isfile(src):
            problems.append(f"missing source: {src}")
        elif not matches(src, entry):
            problems.append(f"source differs from manifest (size/SHA-256): {src}")
        else:
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.copyfile(src, dst)
            if not matches(dst, entry):
                problems.append(f"copy verification failed: {dst}")
            else:
                copied += 1
    print(f"copied {copied}, already up to date {unchanged}, problems {len(problems)}")
    for p in problems:
        print("  ", p)
    sys.exit(1 if problems else 0)

if __name__ == "__main__":
    main()
