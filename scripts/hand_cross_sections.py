#!/usr/bin/env python3
"""Independent hand computation of the M2/M3 macroscopic cross sections (no Rust involved).

Re-implements, from the C++ algorithm, what the Rust models must return:
  * G4DNACrossSectionDataSet::LoadData (dna/utils/src/G4DNACrossSectionDataSet.cc:83-240):
    value scale from the model (Champion 1e-16 cm^2 :129, Born 1e-22/3.343 m^2 :112/:116),
    non-positive values replaced by 1e-300 before log10 (:148);
  * G4EMDataSet::FindValue (lowenergy/src/G4EMDataSet.cc:167-183) + FindLowerBound (:445-459)
    + G4LogLogInterpolation::Calculate (lowenergy/src/G4LogLogInterpolation.cc:95-127);
  * model windows (Champion 7.4 eV-1 MeV :253, Born excitation 9 eV-1 MeV :157, Born1 ionisation
    11 eV-1 MeV :363, Sanche 2-100 eV G4DNASancheExcitationModel.cc:207, Melton 4-13 eV
    G4DNAMeltonAttachmentModel.cc:199);
  * M3: G4DNASancheExcitationModel (models/src/G4DNASancheExcitationModel.cc): own reader of the
    9-column table (:139-178), `TotalCrossSection` = lin-lin interpolation of the sum of the nine
    columns at the nodes (:316-342), with the `t (1 - 1e-12)` protection at the last node
    (:327), factor 1e-16 cm^2 (:309) and the replacement of an exact zero by 1e-30 in Geant4
    internal units (mm^2 -> 1e-18 nm^2) (:312, :339), then `sigma * 2 * waterDensity` (:222);
    per-level `PartialCrossSection` (:281-313) used for the level choice;
  * M3: G4DNAMeltonAttachmentModel: one column, scale 1e-18 cm^2 (:141), log-log through
    G4DNACrossSectionDataSet; above the last node (12.3 eV) the last stored value (0) is
    returned by G4EMDataSet::FindValue (lowenergy/src/G4EMDataSet.cc:167-183);
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
    e1, e2 = lg(es[i]), lg(es[i + 1])   # the energy column is floored too (DataSet.cc:148)
    return 10 ** (d1 + (d2 - d1) * (math.log10(e) - e1) / (e2 - e1)) * scale_nm2


def sanche_sigma(rows, e, level=None):
    """G4DNASancheExcitationModel::TotalCrossSection / PartialCrossSection in nm^2 per molecule
    (no factor 2). Energies in eV; rows = [E, 9 levels]."""
    ts = [r[0] for r in rows]
    t = e
    if t == ts[-1]:
        t = t * (1.0 - 1e-12)
    k2 = next(k for k, x in enumerate(ts) if x > t)   # std::upper_bound
    k1 = k2 - 1

    def val(r):
        return sum(r[1:10]) if level is None else r[1 + level]

    xs1, xs2 = val(rows[k1]), val(rows[k2])
    a = (xs2 - xs1) / (ts[k2] - ts[k1])
    b = xs2 - a * ts[k2]
    sigma = (a * t + b) * 1e-2          # 1e-16 cm^2 = 1e-2 nm^2
    return sigma if sigma != 0.0 else 1e-18   # 1e-30 mm^2


champion = read_table("sigma_elastic_e_champion.dat")
born_exc = read_table("sigma_excitation_e_born.dat")
born_ion = read_table("sigma_ionisation_e_born.dat")
BORN = 1e-22 / 3.343 * M2
sanche = read_table("sigma_excitationvib_e_sanche.dat")
melton = read_table("sigma_attachment_e_melton.dat")


def sigma_per_molecule(e):
    el = log_log(champion, 1, 1e-16 * CM2, e) if 7.4 <= e <= 1e6 else 0.0
    ex = [log_log(born_exc, c, BORN, e) for c in range(1, 6)] if 9 <= e <= 1e6 else [0.0] * 5
    io = [log_log(born_ion, c, BORN, e) for c in range(1, 6)] if 11 <= e <= 1e6 else [0.0] * 5
    return el, ex, io


def sigma_low_energy(e):
    """(Sanche total per molecule incl. the factor 2, Sanche partials, Melton) in nm^2."""
    if 2.0 <= e <= 100.0:
        vib = 2.0 * sanche_sigma(sanche, e)
        parts = [sanche_sigma(sanche, e, lvl) for lvl in range(9)]
    else:
        vib, parts = 0.0, [0.0] * 9
    att = log_log(melton, 1, 1e-18 * CM2, e) if 4.0 <= e <= 13.0 else 0.0   # clamps beyond 12.3
    return vib, parts, att


def macroscopic_all(e):
    """Sigma/nm of (elastic, excitation, ionisation, vibrational, attachment) and total."""
    el, ex, io = sigma_per_molecule(e)
    vib, _, att = sigma_low_energy(e)
    s = [el * n_per_nm3, sum(ex) * n_per_nm3, sum(io) * n_per_nm3, vib * n_per_nm3, att * n_per_nm3]
    return s, sum(s)


def r_mean_nm(e):
    """Meesungnoen2002::GetRmean (G4DNAOneStepThermalizationModel.cc:96-107)."""
    g = [-4.06217193e-08, 3.06848412e-06, -9.93217814e-05, 1.80172797e-03, -2.01135480e-02,
         1.42939448e-01, -6.48348714e-01, 1.85227848e+00, -3.36450378e+00, 4.37785068e+00,
         -4.20557339e+00, 3.81679083e+00, -2.34069784e-01]
    if e > 0.1:
        return sum(g[12 - i] * e ** i for i in range(12, -1, -1))
    return 0.0


if __name__ == "__main__":
    print(f"n = {n_per_nm3!r} molecules/nm^3  (M = {molar!r} g/mol)")
    for e in [100.0, 4321.0, 250000.0]:
        el, ex, io = sigma_per_molecule(e)
        s = [el * n_per_nm3, sum(ex) * n_per_nm3, sum(io) * n_per_nm3]
        print(f"E = {e!r} eV")
        print(f"  Sigma_elastic    = {s[0]!r} /nm")
        print(f"  Sigma_excitation = {s[1]!r} /nm")
        print(f"  Sigma_ionisation = {s[2]!r} /nm")
        print(f"  Sigma_total(M2)  = {sum(s)!r} /nm   mfp = {1 / sum(s)!r} nm")
        print(f"  ionisation partial sigma (nm^2): {io!r}")
        print(f"  excitation partial sigma (nm^2): {ex!r}")
    print("--- M3: all five processes (elastic, excitation, ionisation, vibrational, attachment)")
    nodes_sanche = [r[0] for r in sanche]
    nodes_melton = [r[0] for r in melton]
    energies = sorted(set([2.0, 3.0, 4.0, 4.1, 5.8, 6.1, 7.3, 7.4, 8.0, 9.0, 11.7, 12.3, 12.8, 13.0, 15.0,
                           50.0, 77.7, 99.9, 100.0, 150.0] + nodes_sanche[1:] + nodes_melton[1:]))
    for e in energies:
        s, tot = macroscopic_all(e)
        print(f"E = {e!r} eV  Sigma[el,ex,io,vib,att] = {s!r}  total = {tot!r} /nm")
    print("--- Sanche partial sigma (nm^2/molecule, no factor 2) and level energies")
    for e in [2.0, 7.4, 31.0, 100.0]:
        print(f"E = {e!r}: {sigma_low_energy(e)[1]!r}")
    print("--- Meesungnoen2002 r_mean (nm)")
    for e in [0.05, 0.1, 0.11, 0.2, 1.0, 2.5, 5.0, 7.0, 7.4]:
        print(f"E = {e!r} eV: r_mean = {r_mean_nm(e)!r} nm")
