#!/usr/bin/env python3
"""Hand-computed (independent of the Rust code) reference values for the physicochemical stage.

Reads the machine-extracted C++ constants (crates/dna-chemistry/tests/fixtures/cpp_extract.json,
produced by scripts/extract_physchem_from_cpp.py) and evaluates, in plain Python:
  * the Onsager radius and the dielectric constant of G4DNAElectronHoleRecombination
    (11.4.1: D/processes/src/G4DNAElectronHoleRecombination.cc), with the CLHEP constants of the
    reference tree (k_Boltzmann = 8.617333e-11 MeV/K, mu0 = 4 pi 1e-7 henry/m, e_SI);
  * the nominal RMS of the product displacements (analytic, from the displacer cases);
  * the elemental balance of every channel.
Output: crates/dna-chemistry/tests/fixtures/hand_values.json

    python3 scripts/hand_physchem.py > crates/dna-chemistry/tests/fixtures/hand_values.json
"""
import json, math
from pathlib import Path

root = Path(__file__).resolve().parent.parent
x = json.loads((root / "crates/dna-chemistry/tests/fixtures/cpp_extract.json").read_text())
rc = x["recombination"]

# ---- Onsager radius ---------------------------------------------------------------------------
e_SI = float(rc["clhep"]["e_SI"])
c = 299792458.0  # m/s, exact (CLHEP c_light = 299.792458 mm/ns)
k_B_J_per_K = float(rc["clhep"]["k_Boltzmann"]) * 1.0e6 * e_SI  # 8.617333e-11 MeV/K -> eV/K -> J/K
assert rc["clhep"]["mu0"].startswith("4*pi*1.e-7")
onsager_constant_m_K = e_SI**2 * c**2 * 1e-7 / k_B_J_per_K  # e^2 / (4 pi eps0 k_B), eps0 = 1/(mu0 c^2)


def poly(coefs, T):  # c0 + c1/T + c2/T^2 + c3/T^3
    return sum(a * T ** (-i) for i, a in enumerate(coefs))


def epsilon(rho, T):
    Y = 1.0 / (1.0 + rc["Y"][0] / rho**2)
    A, B, S = poly(rc["A"], T), poly(rc["B"], T), poly(rc["S"], T)
    C, D = A - B - 3, B + 3
    lr = math.log10(rho)
    return 1 + 10 ** (Y * (C + (S - 1) * lr) + D + lr)


T = rc["NTP_Temperature_K"]
rho = rc["G4_WATER_density_g_cm3"]
eps = epsilon(rho, T)
r_onsager_nm = onsager_constant_m_K / (T * eps) * 1e9
onsager = {
    "temperature_K": T,
    "density_g_cm3": rho,
    "dielectric_constant": eps,
    "onsager_constant_m_K": onsager_constant_m_K,
    "onsager_radius_nm": r_onsager_nm,
    "probabilities_at_distance_nm": {str(d): 1 - math.exp(-r_onsager_nm / d) for d in (0.25, 0.5, 1.0, 2.0, 5.0, 7.0)},
    "epsilon_at": {str(t): epsilon(1.0, t) for t in (273.15, 293.15, 298.0)},
}

# ---- nominal displacement RMS (analytic) ---------------------------------------------------------
# radialDistributionOfProducts(R): three Gaussians of sigma = R/sqrt(3)  =>  <|d|^2> = R^2.
# Combination rules of the displacer cases, per product (see the case blocks of the C++):
rms = {
    # A1B1: R = 2.4 nm, OH: -1/18 R, H: +17/18 R
    "A1B1_DissociationDecay": {"OH": 2.4 / 18.0, "H": 2.4 * 17.0 / 18.0, "OH-H separation": 2.4},
    # B1A1: R = 0.8, H2: -16/18 R; each OH: 2/18 R + (+-0.5)*R_OH(1.1)  (independent Gaussians)
    "B1A1_DissociationDecay": {
        "H2": 0.8 * 16.0 / 18.0,
        "OH": math.sqrt((0.8 * 2.0 / 18.0) ** 2 + (0.5 * 1.1) ** 2),
    },
    # B1A1 2: O: 0; each H: +-0.5 * R(1.6)
    "B1A1_DissociationDecay2": {"O": 0.0, "H": 0.5 * 1.6},
    # Ionisation: one of (H3O+, OH) displaced by R = 0.8 (prob 1/2 each), plus the common mother shift 2.0
    "Ionisation_DissociationDecay": {
        "H3Op_or_OH_each_product_mean_square": (0.5 * 0.8**2) + 2.0**2,
        "separation_mean_square": 0.8**2,
    },
    # AutoIonisation: OH or H3O+ displaced 0.8 (1/2 each), e_aq by the Meesungnoen penetration at 1.7 eV; common shift 2.0
    "AutoIonisation": {"mother_shift_rms": 2.0, "OH_or_H3Op_extra_rms": 0.8},
    # Dissociative attachment: R = 0.8; H2: -16/18 R; OH: 2/18 R + 0.5 R(1.1); OHm: 0 (different G4MoleculeDefinition)
    "DissociativeAttachment": {
        "H2": 0.8 * 16.0 / 18.0,
        "OH": math.sqrt((0.8 * 2.0 / 18.0) ** 2 + (0.5 * 1.1) ** 2),
        "OHm": 0.0,
    },
}

# ---- elemental balance of each product list ----------------------------------------------------------
comp = {  # (H, O, charge)
    "H3Op": (3, 1, 1), "°OH": (1, 1, 0), "OHm": (1, 1, -1), "e_aq": (0, 0, -1), "H": (1, 0, 0),
    "H2": (2, 0, 0), "Oxy": (0, 1, 0),
}
parent = {"A^1B_1": (2, 1, 0), "B^1A_1": (2, 1, 0), "Exci3rdLayer": (2, 1, 0), "Exci2ndLayer": (2, 1, 0),
          "Exci1stLayer": (2, 1, 0), "Ioni1": (2, 1, 1), "Ioni2": (2, 1, 1), "Ioni3": (2, 1, 1), "Ioni4": (2, 1, 1),
          "Ioni5": (2, 1, 1), "DissociAttachment_ch1": (2, 1, -1), "H2Ovib": (2, 1, 0)}
balance = {}
for st, v in x["states"].items():
    for ch in v["channels"]:
        if not ch["products"]:
            balance[f"{st}/{ch['name']}"] = {"extra_waters": 0, "relax": True}
            continue
        H = sum(comp[p][0] for p in ch["products"])
        O = sum(comp[p][1] for p in ch["products"])
        q = sum(comp[p][2] for p in ch["products"])
        h0, o0, q0 = parent[st]
        extra_h2o_from_h, extra_h2o_from_o = (H - h0) / 2, O - o0
        assert extra_h2o_from_h == extra_h2o_from_o, (st, ch["name"], H, O)
        assert q == q0, (st, ch["name"], q, q0)
        balance[f"{st}/{ch['name']}"] = {"extra_waters": int(extra_h2o_from_o), "relax": False}

# ---- auto-ionisation electron: Meesungnoen2002 r_mean at ke, Gaussian of sigma_1D = 0.62665706865775 r_mean ---------
g = x["penetration"]["meesungnoen2002_gcoeff"]
ke = x["displacer"]["autoionisation_electron_kinetic_energy_ev"]
r_mean = sum(g[12 - i] * ke**i for i in range(13))
sigma1d = x["penetration"]["sigma_1d_per_rmean"] * r_mean
assert abs(x["penetration"]["sigma_1d_per_rmean"] - math.sqrt(math.pi) / 2 ** 1.5) < 1e-15
penetration = {"ke_ev": ke, "r_mean_nm": r_mean, "sigma_1d_nm": sigma1d, "mean_square_nm2": 3 * sigma1d**2}

# ---- expected mean square |d|^2 (nm^2) of each product position relative to the event, per channel kind ---------------
# (hand derivation; independent Gaussians add in quadrature; R = 3-D RMS of radialDistributionOfProducts)
ms = {}
ms["a1b1_dissociation_decay"] = {"OH": (2.4 / 18) ** 2, "H": (2.4 * 17 / 18) ** 2}
oh_b1a1 = (0.8 * 2 / 18) ** 2 + 0.25 * 1.1**2  # O-atom recoil + (+-0.5) of an independent 1.1 nm vector
ms["b1a1_dissociation_decay"] = {"H2": (0.8 * 16 / 18) ** 2, "OH": oh_b1a1}
ms["b1a1_dissociation_decay2"] = {"Oxy": 0.0, "H": 0.25 * 1.6**2}
mother = 2.0**2
extra = 0.5 * 0.8**2  # displaced by 0.8 nm in half of the cases
ms["ionisation_dissociation_decay"] = {"H3Op": mother + extra, "OH": mother + extra}
ms["auto_ionisation"] = {"OH": mother + extra, "H3Op": mother + extra, "e_aq": mother + penetration["mean_square_nm2"]}
ms["dissociative_attachment"] = {"H2": (0.8 * 16 / 18) ** 2, "OHm": 0.0, "OH": oh_b1a1}
# separations between products (mean square of the difference vector)
sep = {
    "a1b1_dissociation_decay/OH-H": 2.4**2,
    "ionisation_dissociation_decay/H3Op-OH": 0.8**2,
    "auto_ionisation/OH-H3Op": 2 * extra,  # one of the two displaced by 0.8 nm, the common shift cancels... see below
}
# OH-H3O+ in the auto-ionisation: displaced one has |d|^2 = 0.8^2 w.p. 1; the other 0 => separation ms = 0.8^2
sep["auto_ionisation/OH-H3Op"] = 0.8**2

print(json.dumps({"onsager": onsager, "penetration": penetration, "expected_mean_square_nm2": ms, "expected_separation_mean_square_nm2": sep, "nominal_rms_nm": rms, "channel_balance": balance}, indent=1, sort_keys=True))
