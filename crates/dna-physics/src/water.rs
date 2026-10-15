//! Liquid-water constants used by the option2 electron models, with their Geant4 sources.
//!
//! Paths are relative to the Geant4 11.4.1 tree (`rtonneau/geant4`, squashed commit
//! `27ea526b`). Nothing here is invented: every number is copied from, or derived from, a line
//! cited below.

use dna_core::units::Energy;

/// Electron rest energy in eV: CLHEP `electron_mass_c2 = 0.510998910 MeV`
/// (`source/externals/clhep/include/CLHEP/Units/PhysicalConstants.h:80`). This is the value of
/// the Geant4 tree in use (CODATA 2006), not the current CODATA value 510 998.950 eV; it enters
/// the Born angular distribution and the primary momentum balance (`G4DNABornAngle.cc:87`,
/// `G4DNABornIonisationModel1.cc:500`).
pub const ELECTRON_MASS_C2_EV: f64 = 510_998.910;

/// Binding energies of the five water ionisation shells, eV, ascending: 1b1, 3a1, 1b2, 2a1, 1a1
/// (oxygen K). `dna/utils/src/G4DNAWaterIonisationStructure.cc:31-35`.
pub const IONISATION_ENERGIES_EV: [f64; 5] = [10.79, 13.39, 16.05, 32.30, 539.0];

/// Excitation energies of the five water excitation levels, eV: A1B1, B1A1, Rydberg A+B,
/// Rydberg C+D, diffuse bands. `dna/utils/src/G4DNAWaterExcitationStructure.cc:31-35`.
pub const EXCITATION_ENERGIES_EV: [f64; 5] = [8.22, 10.00, 11.24, 12.61, 13.77];

/// Binding energy of ionisation shell `shell` (`G4DNAWaterIonisationStructure::IonisationEnergy`,
/// which returns 0 for an out-of-range index).
pub fn ionisation_energy(shell: usize) -> Energy {
    Energy::from_ev(IONISATION_ENERGIES_EV.get(shell).copied().unwrap_or(0.0))
}

/// Excitation energy of level `level` (`G4DNAWaterExcitationStructure::ExcitationEnergy`, which
/// returns 0 for an out-of-range index).
pub fn excitation_energy(level: usize) -> Energy {
    Energy::from_ev(EXCITATION_ENERGIES_EV.get(level).copied().unwrap_or(0.0))
}

/// Number of water molecules per nm^3 as Geant4-DNA computes it for `G4_WATER`.
///
/// Every DNA model returns `sigma_per_molecule * (*fpMolWaterDensity)[material index]`
/// (e.g. `G4DNAChampionElasticModel.cc:251,272`). That table is `G4DNAMolecularMaterial::
/// GetNumMolPerVolTableFor(G4_WATER)`: `numMolPerVol = massFraction * density / massOfMolecule`
/// (`dna/utils/src/G4DNAMolecularMaterial.cc:197-252`), with
/// * `density` = 1.0 g/cm^3 (`materials/src/G4NistMaterialBuilder.cc:712`),
/// * `massFraction` = 1 (`G4_WATER` is its own molecular component, `:292-325`),
/// * `massOfMolecule = A_mol / N_A` with `A_mol = 2 A_H + A_O` from the natural isotope
///   composition of `G4NistElementBuilder` (`materials/src/G4Material.cc:395-417`,
///   `G4NistElementBuilder.cc:305-326, 395-412`): H-1 99.9885 %, H-2 0.0115 %; O-16 99.757 %,
///   O-17 0.038 %, O-18 0.205 %; isotope masses in amu below,
/// * `N_A = 6.02214076e23 /mol` (`PhysicalConstants.h:50`).
///
/// The H-1 and H-2 atomic masses are made consistent with Geant4's particle masses in
/// `G4NistElementBuilder.cc:317-319` (`proton_mass_c2 = 938.272013 MeV`, deuteron 1875.613 MeV,
/// `electron_mass_c2`, hydrogen binding `14.4381 Z^2.39 + 1.55468e-6 Z^5.35` eV from `:305`,
/// `amu_c2 = 931.494028 MeV`).
///
/// Result: 3.342797593e22 cm^-3 = 33.42797593 nm^-3 (M = 18.01527192 g/mol). Cross-check: the
/// Born tables use the scale `1e-22/3.343 m^2` (`G4DNABornExcitationModel1.cc:112`), i.e. they
/// were generated for 3.343e28 m^-3.
pub const WATER_MOLECULES_PER_NM3: f64 = water_number_density_per_nm3();

const fn water_number_density_per_nm3() -> f64 {
    const AMU_C2_MEV: f64 = 931.494028;
    const ELECTRON_MASS_MEV: f64 = 0.510998910;
    const PROTON_MASS_MEV: f64 = 938.272013;
    const DEUTERON_MASS_MEV: f64 = 1875.613;
    const EV_IN_MEV: f64 = 1e-6;
    const AVOGADRO_PER_MOL: f64 = 6.022_140_76e23;
    const DENSITY_G_PER_CM3: f64 = 1.0;
    const NM3_PER_CM3: f64 = 1e21;

    let binding_h_mev = (14.4381 + 1.55468e-6) * EV_IN_MEV;
    let h1 = (PROTON_MASS_MEV + ELECTRON_MASS_MEV - binding_h_mev) / AMU_C2_MEV;
    let h2 = (DEUTERON_MASS_MEV + ELECTRON_MASS_MEV - binding_h_mev) / AMU_C2_MEV;
    let a_h = (99.9885 * h1 + 0.0115 * h2) / 100.0;
    let a_o = (99.757 * 15.9949 + 0.038 * 16.9991 + 0.205 * 17.9992) / 100.0;
    let molar_mass_g_per_mol = 2.0 * a_h + a_o;
    DENSITY_G_PER_CM3 * AVOGADRO_PER_MOL / molar_mass_g_per_mol / NM3_PER_CM3
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn structure_energies_match_geant4_and_are_ascending() {
        assert_eq!(ionisation_energy(0).ev(), 10.79);
        assert_eq!(ionisation_energy(4).ev(), 539.0);
        assert_eq!(excitation_energy(0).ev(), 8.22);
        assert_eq!(excitation_energy(4).ev(), 13.77);
        assert!(IONISATION_ENERGIES_EV.windows(2).all(|w| w[0] < w[1]));
        assert!(EXCITATION_ENERGIES_EV.windows(2).all(|w| w[0] < w[1]));
        // Geant4 returns 0 for an out-of-range index.
        assert_eq!(ionisation_energy(5).ev(), 0.0);
        assert_eq!(excitation_energy(9).ev(), 0.0);
    }

    /// Hand computation (Python, same inputs): H-1 1.0078250308, H-2 2.0141019997, A_H =
    /// 1.00794075267, A_O = 15.9993904110, M = 18.01527191633 g/mol, n = 3.342797593047e22 cm^-3.
    #[test]
    fn water_number_density_matches_hand_computation() {
        assert_relative_eq!(
            WATER_MOLECULES_PER_NM3,
            33.427_975_930_47,
            max_relative = 1e-12
        );
    }

    /// The Born tables were generated for 3.343e22 cm^-3 (the `3.343` in their scale factor).
    #[test]
    fn water_number_density_is_consistent_with_the_born_table_scale() {
        assert_relative_eq!(WATER_MOLECULES_PER_NM3, 33.43, max_relative = 2e-4);
    }
}
