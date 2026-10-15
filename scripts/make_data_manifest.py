#!/usr/bin/env python3
"""Write a MANIFEST.toml (size + SHA-256) for the Geant4 DNA files this project uses.

Run locally; the data are never committed.
  python scripts/make_data_manifest.py "C:\\DEV\\GEANT4\\Geant4-ressources-data\\G4EMLOW8.8" \
      --groups electron_option2 proton_option2 -o data/MANIFEST.toml
Files listed but missing are reported (not silently skipped) and exit code is 1.
"""
import argparse, hashlib, os, sys

GROUPS = {
    "electron_option2": [
        "sigma_elastic_e_champion", "sigmadiff_cumulated_elastic_e_champion_hp.dat",
        "sigma_excitation_e_born", "sigma_ionisation_e_born",
        "sigmadiff_cumulated_ionisation_e_born_hp.dat", "sigmadiff_ionisation_e_born.dat",
        "sigma_excitationvib_e_sanche.dat", "sigma_attachment_e_melton"],
    "proton_option2": [
        "sigma_excitation_p_born", "sigma_ionisation_p_born",
        "sigmadiff_cumulated_ionisation_p_born_hp.dat", "sigmadiff_ionisation_p_born.dat",
        "sigma_ionisation_p_rudd", "sigma_ionisation_h_rudd",
        "sigma_elastic_proton_HTran", "sigmadiff_cumulated_elastic_proton_HTran.dat",
        "sigma_excitation_p_RPWBA", "sigma_ionisation_p_RPWBA",
        "sigmadiff_cumulated_ionisation_p_RPWBA.dat", "sigmadiff_ionisation_p_RPWBA.dat"],
    "electron_option4": [
        "sigma_excitation_e_emfietzoglou", "sigma_ionisation_e_emfietzoglou",
        "sigmadiff_cumulated_ionisation_e_emfietzoglou.dat", "sigmadiff_ionisation_e_emfietzoglou.dat"],
    "electron_option6": [
        "sigma_elastic_e_cpa100", "sigmadiff_cumulated_elastic_e_cpa100",
        "sigma_excitation_e_cpa100", "sigma_ionisation_e_cpa100_form_rel",
        "sigmadiff_cumulated_ionisation_e_cpa100_rel", "sigmadiff_ionisation_e_cpa100_rel"],
}

def resolve(dna_dir, name):
    for cand in (name, name + ".dat"):
        p = os.path.join(dna_dir, cand)
        if os.path.isfile(p):
            return cand, p
    return None, None

def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("g4emlow_dir")
    ap.add_argument("--groups", nargs="+", default=["electron_option2", "proton_option2"], choices=GROUPS)
    ap.add_argument("-o", "--output", default="data/MANIFEST.toml")
    a = ap.parse_args()
    dna_dir = os.path.join(a.g4emlow_dir, "dna")
    if not os.path.isdir(dna_dir):
        sys.exit(f"not found: {dna_dir}")
    missing, lines = [], ['dataset = "G4EMLOW"', 'version = "8.8"', ""]
    for g in a.groups:
        for name in GROUPS[g]:
            rel, path = resolve(dna_dir, name)
            if path is None:
                missing.append(name); continue
            lines += ["[[file]]", f'path = "dna/{rel}"', f'group = "{g}"',
                      f"bytes = {os.path.getsize(path)}", f'sha256 = "{sha256(path)}"', ""]
    os.makedirs(os.path.dirname(a.output) or ".", exist_ok=True)
    with open(a.output, "w", newline="\n") as f:
        f.write("\n".join(lines))
    print(f"wrote {a.output}")
    if missing:
        print("MISSING:", *missing, sep="\n  ")
        sys.exit(1)

if __name__ == "__main__":
    main()
