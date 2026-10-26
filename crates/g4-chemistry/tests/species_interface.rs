use g4_chemistry::{SPECIES_CSV_HEADER, Species, SpeciesInstance};
use g4_core::units::Time;
use g4_core::vec3::Vec3;

#[test]
fn chemistry_root_preserves_all_species_names_and_order() {
    let names = [
        "H3Op", "°OH", "OHm", "e_aq", "H", "H2", "H2O2", "HO2°", "HO2m", "Oxy", "Om", "O2", "O2m",
        "O3", "O3m",
    ];
    assert_eq!(Species::ALL.len(), names.len());
    for (index, (species, name)) in Species::ALL.into_iter().zip(names).enumerate() {
        assert_eq!(species.index(), index);
        assert_eq!(species.chemistry_name().as_bytes(), name.as_bytes());
        assert_eq!(Species::from_chemistry_name(name), Some(species));
        assert_eq!(Species::from_legacy_ascii_tag(name), Some(species));
    }
}

#[test]
fn chemistry_root_preserves_strict_and_legacy_parsing() {
    for (alias, species) in [("OH", Species::OH), ("HO2", Species::HO2)] {
        assert_eq!(Species::from_chemistry_name(alias), None);
        assert_eq!(Species::from_legacy_ascii_tag(alias), Some(species));
    }
    for invalid in ["", "oh", "Ho2", "HO_2°", "°OH^0", "H3Op(B)", "nonsense"] {
        assert_eq!(Species::from_chemistry_name(invalid), None);
        assert_eq!(Species::from_legacy_ascii_tag(invalid), None);
    }
}

#[test]
fn chemistry_root_preserves_species_csv() {
    let instance = SpeciesInstance {
        species: Species::e_aq,
        position: Vec3::new(1.0, -2.5, 0.25),
        time: Time::from_ps(1.0),
    };
    assert_eq!(SPECIES_CSV_HEADER, "species,x_nm,y_nm,z_nm,time_ps");
    assert_eq!(instance.csv_row(), "e_aq,1.0,-2.5,0.25,1.0");
}
