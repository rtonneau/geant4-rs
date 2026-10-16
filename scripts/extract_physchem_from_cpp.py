#!/usr/bin/env python3
"""Machine-extract the physicochemical-stage data of Geant4 11.4.1 (option3 chemistry) from the C++ text.

Independent of the Rust code: the output (crates/dna-chemistry/tests/fixtures/cpp_extract.json) is what the
Rust data files (data/chemistry/*.toml) and the displacer constants are tested against. Run with the path of
the (read-only) reference repository:

    python3 scripts/extract_physchem_from_cpp.py /path/to/geant4 > crates/dna-chemistry/tests/fixtures/cpp_extract.json

Files read (relative to <geant4>/source):
  physics_lists/constructors/electromagnetic/src/G4ChemDissociationChannels_option1.cc
  processes/electromagnetic/dna/utils/src/G4DNAWaterExcitationStructure.cc
  processes/electromagnetic/dna/utils/src/G4DNAWaterIonisationStructure.cc
  processes/electromagnetic/dna/molecules/types/src/G4*.cc
  processes/electromagnetic/dna/processes/src/G4DNAWaterDissociationDisplacer.cc
"""
import json, re, subprocess, sys
from pathlib import Path

root = Path(sys.argv[1]) / "source"
CH = root / "physics_lists/constructors/electromagnetic/src/G4ChemDissociationChannels_option1.cc"
DNA = root / "processes/electromagnetic/dna"


def text(p):
    return Path(p).read_text(encoding="utf-8", errors="replace")


def strip_comments(s):
    s = re.sub(r"/\*.*?\*/", "", s, flags=re.S)
    return re.sub(r"//[^\n]*", "", s)


# ---------------------------------------------------------------- structures
def push_back_energies(path):
    return [float(x) for x in re.findall(r"energyConstant\.push_back\(([\d.]+)\*eV\)", strip_comments(text(path)))]


exc = push_back_energies(DNA / "utils/src/G4DNAWaterExcitationStructure.cc")
ion = push_back_energies(DNA / "utils/src/G4DNAWaterIonisationStructure.cc")

# ---------------------------------------------------------------- molecule definitions
UNIT = {"m * m / s": 1.0, "cm * cm / s": 1e-4}
defs = {}
for f in (DNA / "molecules/types/src").glob("G4*.cc"):
    t = strip_comments(text(f))
    m = re.search(r"new G4MoleculeDefinition\(name,\s*mass,\s*([\d.eE+-]+)\s*\*\s*\(([^)]*)\),\s*(-?\d+)", t)
    if not m:
        continue
    cls = f.stem
    rad = re.search(r"(?:\d+|\d+,)\s*,?\s*([\d.]+)\s*\*\s*(angstrom|nm)", t[m.end():m.end() + 160])
    defs[cls] = {
        "diffusion_m2_s": float(m.group(1)) * UNIT[m.group(2).strip()],
        "charge": int(m.group(3)),
        "radius_angstrom": (float(rad.group(1)) * (10.0 if rad.group(2) == "nm" else 1.0)) if rad else None,
    }

# ---------------------------------------------------------------- ConstructMolecule (species)
src = strip_comments(text(CH))
cm = src[src.index("ConstructMolecule()"): src.index("ConstructDissociationChannels()")]

inline_defs = {}
for m in re.finditer(
    r'auto (\w+) = new G4MoleculeDefinition\("([^"]+)",\s*[^,]+,\s*([\d.eE+-]+)\s*\*\s*\(m \* m / s\),\s*(-?\d+),\s*\d+,\s*([\d.]+)\s*\*\s*angstrom',
    cm,
):
    inline_defs[m.group(1)] = {
        "diffusion_m2_s": float(m.group(3)),
        "charge": int(m.group(4)),
        "radius_angstrom": float(m.group(5)),
    }

species = {}
BOOKKEEPING_CONFIGURATIONS = {"H2O(B)", "H3Op(B)", "OHm(B)", "NoneM"}
for m in re.finditer(
    r'(?:G4MolecularConfiguration\*\s+(\w+)\s*=\s*)?\s*molTable->CreateConfiguration\(\s*"([^"]+)"\s*,\s*(\w+)(?:::Definition\(\))?\s*(?:,\s*(-?\d+)\s*,\s*([\d.eE+-]+)\s*\*\s*\(m2 / s\))?\s*\)',
    cm,
):
    var, tag, d, charge, diff = m.groups()
    if tag in BOOKKEEPING_CONFIGURATIONS:
        continue
    base = defs.get(d) or inline_defs.get(d)
    if base is None:
        raise SystemExit(f"no definition for {d} ({tag})")
    species[tag] = {
        "definition": d,
        "diffusion_m2_s": float(diff) * 1.0 if diff else base["diffusion_m2_s"],
        "charge": int(charge) if charge is not None else base["charge"],
        "vdw_radius_nm": None,
        "definition_radius_nm": base["radius_angstrom"] / 10.0 if base["radius_angstrom"] else None,
    }
for m in re.finditer(r'GetConfiguration\("([^"]+)"\)->SetDiffusionCoefficient\(([\d.eE+-]+)\s*\*\s*\(m2 / s\)\)', cm):
    species[m.group(1)]["diffusion_m2_s"] = float(m.group(2))
for tag, v in re.findall(r'GetConfiguration\("([^"]+)"\)->SetVanDerVaalsRadius\(([\d.]+)\s*\*\s*nm\)', cm):
    species[tag]["vdw_radius_nm"] = float(v)
for var, v in re.findall(r"(\w+)->SetVanDerVaalsRadius\(([\d.]+)\s*\*\s*nm\)", cm):
    tag = re.search(r'G4MolecularConfiguration\*\s+%s\s*=\s*\s*molTable->CreateConfiguration\(\s*"([^"]+)"' % var, cm)
    if tag:
        species[tag.group(1)]["vdw_radius_nm"] = float(v)
for tag, v in species.items():
    if v["vdw_radius_nm"] is None:
        raise SystemExit(f"no van der Waals radius for {tag}")
if len(species) != 15:
    raise SystemExit(f"expected 15 chemical species configurations, found {len(species)}")
if BOOKKEEPING_CONFIGURATIONS.intersection(species):
    raise SystemExit("bookkeeping configurations entered the chemical species table")

# ---------------------------------------------------------------- ConstructDissociationChannels
dc = src[src.index("ConstructDissociationChannels()"):]
var_tag = {}
for var, tag in re.findall(r'G4MolecularConfiguration\*\s+(\w+)\s*=\s*molTable->GetConfiguration\("([^"]+)"\)', dc):
    var_tag[var] = tag

channels = {}  # object id -> dict
cur = {}  # variable -> object id
states = {}  # state -> {"orbits_vacated": [...], "orbits_filled": [...], "channels": [...]}
occ = {"vacated": [], "filled": []}
ground_reset = lambda: occ.update(vacated=[], filled=[])
counter = 0
decay_defs = {}

stmt = [s.strip() for s in re.split(r";", dc) if s.strip()]
for s in stmt:
    m = re.match(r'(\w+)\s*=\s*new G4MolecularDissociationChannel\("([^"]+)"\)$', s)
    if m:
        counter += 1
        cur[m.group(1)] = counter
        channels[counter] = {"name": m.group(2), "products": [], "probability": None, "energy_ev": 0.0, "displacement": None}
        continue
    m = re.match(r"(\w+)->AddProduct\((\w+)\)$", s)
    if m:
        channels[cur[m.group(1)]]["products"].append(var_tag[m.group(2)])
        continue
    m = re.match(r"(\w+)->SetProbability\(([\d.]+)\)$", s)
    if m:
        channels[cur[m.group(1)]]["probability"] = float(m.group(2))
        continue
    m = re.match(r"(\w+)->SetEnergy\(waterExcitation\.ExcitationEnergy\((\d)\)\)$", s)
    if m:
        channels[cur[m.group(1)]]["energy_ev"] = exc[int(m.group(2))]
        channels[cur[m.group(1)]]["energy_from_excitation_level"] = int(m.group(2))
        continue
    m = re.match(r"(\w+)->SetDisplacementType\(G4DNAWaterDissociationDisplacer::(\w+)\)$", s)
    if m:
        channels[cur[m.group(1)]]["displacement"] = m.group(2)
        continue
    m = re.match(r"\*occ = \*\(water->GetGroundStateElectronOccupancy\(\)\)$", s)
    if m:
        ground_reset()
        continue
    m = re.match(r"occ->RemoveElectron\((\d)(?:,\s*1)?\)$", s)
    if m:
        occ["vacated"].append(int(m.group(1)))
        continue
    m = re.match(r"occ->AddElectron\((\d),\s*1\)$", s)
    if m:
        occ["filled"].append(int(m.group(1)))
        continue
    m = re.match(r'water->NewConfigurationWithElectronOccupancy\("([^"]+)",\s*\*occ\)$', s)
    if m:
        states[m.group(1)] = {"orbits_vacated": list(occ["vacated"]), "orbits_filled": list(occ["filled"]), "channels": []}
        continue
    m = re.match(r'water->AddDecayChannel\("([^"]+)",\s*(\w+)\)$', s)
    if m:
        states[m.group(1)]["channels"].append(json.loads(json.dumps(channels[cur[m.group(2)]])))
        continue
    m = re.match(r'water->AddDecayChannel\("([^"]+)",\s*new G4MolecularDissociationChannel\(\*(\w+)\)\)$', s)
    if m:
        states[m.group(1)]["channels"].append(json.loads(json.dumps(channels[cur[m.group(2)]])))
        continue
    m = re.match(r'const auto pH2Ovib = G4H2O::Definition\(\)->NewConfiguration\("H2Ovib"\)', s)
    if m:
        states["H2Ovib"] = {"orbits_vacated": [], "orbits_filled": [], "channels": []}
        continue
    m = re.match(r"water->AddDecayChannel\(pH2Ovib,\s*(\w+)\)$", s)
    if m:
        states["H2Ovib"]["channels"].append(json.loads(json.dumps(channels[cur[m.group(1)]])))
        continue

# ---------------------------------------------------------------- displacer constants per case
dp = strip_comments(text(DNA / "processes/src/G4DNAWaterDissociationDisplacer.cc"))
disp = {"mother_rms_nm": {}, "products": {}}
mo = dp[dp.index("GetMotherMoleculeDisplacement"): dp.index("GetProductsDisplacement")]
for case, v in re.findall(r"case (\w+):\s*RMSMotherMoleculeDisplacement = ([\d.]+) \* nanometer;", mo):
    disp["mother_rms_nm"][case] = float(v)
po = dp[dp.index("GetProductsDisplacement"): dp.index("radialDistributionOfProducts(G4double Rrms)")]
for case, body in re.findall(r"case (\w+):\s*\{(.*?)\n    \}\n    case|case (\w+):\s*\{(.*?)\n    \}\n  \}", po, flags=re.S) and []:
    pass
parts = re.split(r"\n    case ", "\n    case " + po.split("switch (decayType) {", 1)[1])
for part in parts[1:]:
    name = part.split(":", 1)[0].strip()
    if name not in disp["mother_rms_nm"]:
        continue
    disp["products"][name] = {
        "rms_nm": [float(x) for x in re.findall(r"([\d.]+) \* nanometer", part)],
        "fractions": [x.replace(" ", "") for x in re.findall(r"([+-]?\s*\d+\.\d+\s*/\s*\d+\.\d+)\s*\*\s*RandDirection", part)],
        "half_factors": re.findall(r"= *([+-]?0\.5) \* (?:OH|H)Displacement", part),
        "branch_probability": re.findall(r"RdmValue < ([\d.]+)", part),
    }
ke = re.search(r"ke\(([\d.]+)\*eV\)", dp)
disp["autoionisation_electron_kinetic_energy_ev"] = float(ke.group(1))
disp["radial_distribution_of_products_sigma_per_component"] = "Rrms / sqrt(3)" if "Rrms * inverse_sqrt_3" in dp else "?"

# ---------------------------------------------------------------- electron-hole recombination (Onsager radius)
rc = strip_comments(text(DNA / "processes/src/G4DNAElectronHoleRecombination.cc"))
recomb = {}
for fn in ("Y", "A", "B", "S"):
    body = re.search(r"G4double %s\(G4double \w+\)\s*\{(.*?)\n\}" % fn, rc, flags=re.S).group(1)
    expr = body.replace("return", " ")
    terms = re.findall(r"([+-])?\s*(\d+\.\d+(?:e[+-]?\d+)?)(?:\s*\*\s*temp_inverse)*", expr)
    recomb[fn] = [float(("-" if sign == "-" else "") + num) for sign, num in terms]
recomb["C"] = "A - B - 3"
recomb["D"] = "B + 3"
recomb["search_radius_in_onsager_radii"] = float(re.search(r"10\. \* onsagerRadius", rc) and 10.0)
recomb["probability"] = "1 - exp(-onsagerRadius / distance); 1 if distance == 0" if "1. - G4Exp(-onsagerRadius" in rc else "?"
recomb["onsager_constant"] = re.search(r"onsager_constant = ([^;]+);", rc).group(1)
recomb["material_temperature_source"] = "track.GetMaterial()->GetTemperature()" if "GetMaterial()->GetTemperature()" in rc else "?"
recomb["density_in_g_cm3_source"] = "DNAMolecularMaterial density table / (g/cm3)" if "(g / (1e-2 * m * 1e-2 * m * 1e-2 * m))" in rc else "?"
clhep = root / "externals/clhep/include/CLHEP/Units"
recomb["clhep"] = {
    "k_Boltzmann": re.search(r"k_Boltzmann = ([\d.e-]+) \* MeV/kelvin", text(clhep / "PhysicalConstants.h")).group(1),
    "mu0": "4*pi*1.e-7 * henry/m" if "mu0      = 4*pi*1.e-7 * henry/m;" in text(clhep / "PhysicalConstants.h") else "?",
    "e_SI": re.search(r"e_SI\s+= ([\d.e-]+);", text(clhep / "SystemOfUnits.h")).group(1),
}
recomb["NTP_Temperature_K"] = float(re.search(r"NTP_Temperature = ([\d.]+) \* CLHEP::kelvin", text(root / "materials/include/G4Material.hh")).group(1))
recomb["G4_WATER_density_g_cm3"] = float(re.search(r'AddMaterial\("G4_WATER",\s*([\d.]+),', text(root / "materials/src/G4NistMaterialBuilder.cc")).group(1))
recomb["chemistry_global_temperature_K"] = float(re.search(r"fgTemperature = (\d+);", text(DNA / "molecules/management/src/G4MolecularConfiguration.cc")).group(1))
recomb["rest_process_ordering"] = {
    "dissociation": int(re.search(r"AddRestProcess\(dissociationProcess, (\d)\)", text(root / "physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc")).group(1)),
    "recombination": int(re.search(r"AddRestProcess\(new G4DNAElectronHoleRecombination\(\), (\d)\)", text(root / "physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc")).group(1)),
}

# ---------------------------------------------------------------- penetration law of the auto-ionisation electron
th = strip_comments(text(DNA / "models/src/G4DNAOneStepThermalizationModel.cc"))
m = re.search(r"Meesungnoen2002::gCoeff\[13\]\s*=\s*\{(.*?)\};", th, flags=re.S)
penetration = {
    "meesungnoen2002_gcoeff": [float(x) for x in re.findall(r"[+-]?\d+\.\d+e[+-]?\d+", m.group(1))],
    "sigma_1d_per_rmean": float(re.search(r"convertRmean3DToSigma1D = ([\d.]+);", th).group(1)),
}

sha = subprocess.run(["git", "-C", sys.argv[1], "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
print(json.dumps({
    "geant4_commit": sha,
    "excitation_energies_ev": exc,
    "ionisation_energies_ev": ion,
    "species": species,
    "states": states,
    "displacer": disp,
    "recombination": recomb,
    "penetration": penetration,
}, indent=1, sort_keys=True))
