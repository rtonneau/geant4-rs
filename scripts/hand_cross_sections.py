#!/usr/bin/env python3
"""Independent hand computation of the M2 macroscopic cross sections (no Rust involved).

Re-implements, from the C++ algorithm, what the Rust models must return:
  * G4DNACrossSectionDataSet::LoadData (dna/utils/src/G4DNACrossSectionDataSet.cc:83-240):
    value scale from the model (Champion 1e-16 cm^2 :129, Born 1e-22/3.343 m^2 :112/:116),
    non-positive values replaced by 1e-300 before log10 (:148);
  * G4EMDataSet::FindValue (lowenergy/src/G4EMDataSet.cc:167-183) + FindLowerBound (:445-459)
    + G4LogLogInterpolation::Calculate (lowenergy/src/G4LogLogInterpolation.cc:95-127);
  * model windows (Champion 7.4 eV-1 MeV :253, Born excitation 9 eV-1 MeV :157, Born1 ionisation
    11 eV-1 MeV :363);
  * water number density of G4DNAMolecularMaterial (see crates/dna-physics/src/water.rs).

Usage: scripts/hand_cross_sections.py [data_dir]   (default: <repo>/data)
Output: Sigma (1/nm) per process and total at the test energies; pasted into
crates/dna-physics/tests/cross_sections.rs.
"""
import math
import os
import sys

root = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), "..", "data")
CM2 = 1e14   # nm^2
M2 = 1e18    # nm^2

# --- water number density (G4NistElementBuilder / G4Material / G4DNAMolecularMaterial) ---
amu, me, mp, md = 931.494028, 0.510998910, 938.272013, 1875.613
b1 = (14.4381 * 1.0 ** 2.39 + 1.55468e-6 * 1.0 ** 5.35) * 1e-6
h1 = (mp + me - b1) / amu
h2 = (md + me - b1) / amu
a_h = (99.9885 * h1 + 0.0115 * h2) / 100.0
a_o = (99.757 * 15.9949 + 0.038 * 16.9991 + 0.205 * 17.9992) / 100.0
molar = 2 * a_h + a_o
n_per_nm3 = 1.0 * 6.02214076e23 / molar / 1e21


def read_table(name):
    rows = []
    with open(os.path.join(root, "dna", name)) as f:
        for line in f:
            t = line.split("#")[0].split()
            if t:
                rows.append([float(x) for x in t])
    return rows


def log_log(rows, col, scale_nm2, e):
    es = [r[0] for r in rows]
    if e <= es[0]:
        return rows[0][col] * scale_nm2
    if e >= es[-1]:
        return rows[-1][col] * scale_nm2
    i = max(k for k, x in enumerate(es) if x <= e)

    def lg(v):
        return math.log10(v if v > 0 else 1e-300)

    d1, d2 = lg(rows[i][col]), lg(rows[i + 1][col])
    e1, e2 = math.log10(es[i]), math.log10(es[i + 1])
    return 10 ** (d1 + (d2 - d1) * (math.log10(e) - e1) / (e2 - e1)) * scale_nm2


champion = read_table("sigma_elastic_e_champion.dat")
born_exc = read_table("sigma_excitation_e_born.dat")
born_ion = read_table("sigma_ionisation_e_born.dat")
BORN = 1e-22 / 3.343 * M2


def sigma_per_molecule(e):
    el = log_log(champion, 1, 1e-16 * CM2, e) if 7.4 <= e <= 1e6 else 0.0
    ex = [log_log(born_exc, c, BORN, e) for c in range(1, 6)] if 9 <= e <= 1e6 else [0.0] * 5
    io = [log_log(born_ion, c, BORN, e) for c in range(1, 6)] if 11 <= e <= 1e6 else [0.0] * 5
    return el, ex, io


if __name__ == "__main__":
    print(f"n = {n_per_nm3!r} molecules/nm^3  (M = {molar!r} g/mol)")
    for e in [100.0, 4321.0, 250000.0]:
        el, ex, io = sigma_per_molecule(e)
        s = [el * n_per_nm3, sum(ex) * n_per_nm3, sum(io) * n_per_nm3]
        print(f"E = {e!r} eV")
        print(f"  Sigma_elastic    = {s[0]!r} /nm")
        print(f"  Sigma_excitation = {s[1]!r} /nm")
        print(f"  Sigma_ionisation = {s[2]!r} /nm")
        print(f"  Sigma_total      = {sum(s)!r} /nm   mfp = {1 / sum(s)!r} nm")
        print(f"  ionisation partial sigma (nm^2): {io!r}")
        print(f"  excitation partial sigma (nm^2): {ex!r}")
