//! Full option3 compilation: exact counts/signatures, no statistical tolerance.
use g4_chemistry::ChemistryError;
use g4_chemistry::reactions::{
    BulkParticipant, DEFAULT_REACTIONS_TOML, ObservedRate, ReactionParticipant, ReactionTable,
};
use g4_chemistry::species::{DEFAULT_SPECIES_TOML, SpeciesTable};

fn changed(from: &str, to: &str) -> String {
    assert!(DEFAULT_REACTIONS_TOML.contains(from));
    DEFAULT_REACTIONS_TOML.replacen(from, to, 1)
}

#[test]
fn all_source_entries_compile_without_loss_or_reordering() {
    let table = ReactionTable::option3().unwrap();
    let network = table.compile(&SpeciesTable::option3().unwrap()).unwrap();
    assert_eq!(network.iter().count(), 72);
    assert_eq!(network.iter().filter(|r| r.pair().is_some()).count(), 52);
    assert_eq!(network.iter().filter(|r| r.unary().is_some()).count(), 20);
    for (source, compiled) in table.iter().zip(network.iter()) {
        assert_eq!(compiled.source(), source);
        assert_eq!(network.get(source.id), Some(compiled));
        if let Some(pair) = compiled.pair() {
            assert!(source.id <= 52);
            assert_eq!(pair.reaction_id(), source.id);
            assert_eq!(
                pair.reactants().map(ReactionParticipant::Mobile),
                source.reactants
            );
        } else {
            let unary = compiled.unary().unwrap();
            assert_eq!(unary.reaction_id(), source.id);
            assert_eq!(
                ReactionParticipant::Mobile(unary.mobile()),
                source.reactants[0]
            );
            assert_eq!(ReactionParticipant::Bulk(unary.bulk()), source.reactants[1]);
            assert_eq!(
                source.rate,
                ObservedRate::FirstOrder {
                    per_s: unary.rate_per_s()
                }
            );
            assert!(!unary.is_disabled());
        }
    }
    assert_eq!(
        network.get(53).unwrap().unary().unwrap().bulk(),
        BulkParticipant::NoneM
    );
    assert!(network.get(0).is_none());
    assert!(network.get(73).is_none());
}

#[test]
fn zero_custom_unary_channels_remain_present_and_explicitly_disabled() {
    for (id, literal) in [(53, "2.66e3"), (54, "7.15e5")] {
        let text = changed(&format!("rate_per_s = {literal}"), "rate_per_s = 0.0");
        let table = ReactionTable::from_toml(&text).unwrap();
        let network = table.compile(&SpeciesTable::option3().unwrap()).unwrap();
        let unary = network.get(id).unwrap().unary().unwrap();
        assert!(unary.is_disabled());
        assert_eq!(unary.rate_per_s(), 0.0);
        assert_eq!(network.iter().count(), 72);
        assert!(network.get(id + 1).unwrap().unary().is_some());
    }
    assert!(
        ReactionTable::from_toml(&changed(
            "rate_per_molar_s = 0.503e10",
            "rate_per_molar_s = 0.0"
        ))
        .is_err()
    );
}

#[test]
fn negative_and_nonfinite_rates_and_unknown_products_are_rejected() {
    for bad in ["-1.0", "nan", "inf", "-inf"] {
        for (key, literal) in [("rate_per_molar_s", "0.503e10"), ("rate_per_s", "2.66e3")] {
            assert!(
                ReactionTable::from_toml(&changed(
                    &format!("{key} = {literal}"),
                    &format!("{key} = {bad}")
                ))
                .is_err()
            );
        }
    }
    assert!(
        ReactionTable::from_toml(&changed("products = [\"H2\"]", "products = [\"unknown\"]"))
            .is_err()
    );
}

#[test]
fn source_signatures_and_execution_orders_cannot_be_reinterpreted() {
    let cases = [
        changed("source_line = 146", "source_line = 147"),
        changed("source_class = \"I\"", "source_class = \"II\""),
        changed(
            "products = [\"OHm\", \"H2\"]",
            "products = [\"H2\", \"OHm\"]",
        ),
        changed(
            "reactants = [\"HO2°\", \"H2O(B)\"]",
            "reactants = [\"H2O(B)\", \"HO2°\"]",
        ),
        changed(
            "kinetic_intent = \"pseudo_first_order\"",
            "kinetic_intent = \"first_order\"",
        ),
        changed(
            "kinetic_intent = \"totally_diffusion_controlled\"",
            "kinetic_intent = \"first_order\"",
        ),
        changed("set_reaction_type = false", "set_reaction_type = true"),
    ];
    for text in cases {
        assert!(
            ReactionTable::from_toml(&text)
                .and_then(|table| table.compile(&SpeciesTable::option3().unwrap()))
                .is_err()
        );
    }
}

#[test]
fn compilation_checks_numeric_species_inputs_and_arithmetic_before_sampling() {
    let table = ReactionTable::option3().unwrap();
    for (field, literal) in [
        ("diffusion_m2_per_s", "4.9e-9"),
        ("van_der_waals_radius_nm", "0.50"),
    ] {
        let target = format!("{field} = {literal}");
        assert!(DEFAULT_SPECIES_TOML.contains(&target), "{target}");
        for bad in ["nan", "inf", "-1.0", "0.0", "1.7976931348623157e308"] {
            let text = DEFAULT_SPECIES_TOML.replacen(&target, &format!("{field} = {bad}"), 1);
            assert!(
                SpeciesTable::from_toml(&text)
                    .and_then(|species| table.compile(&species))
                    .is_err(),
                "{field}: {bad}"
            );
        }
    }
    let extreme_rate = changed(
        "rate_per_molar_s = 0.503e10",
        "rate_per_molar_s = 1.7976931348623157e308",
    );
    assert!(matches!(
        ReactionTable::from_toml(&extreme_rate)
            .unwrap()
            .compile(&SpeciesTable::option3().unwrap()),
        Err(ChemistryError::ReactionParameterDomain { .. })
    ));
}

#[test]
fn custom_positive_rates_survive_compilation_without_mutating_source() {
    let table =
        ReactionTable::from_toml(&changed("rate_per_s = 7.15e5", "rate_per_s = 123.0")).unwrap();
    let network = table.compile(&SpeciesTable::option3().unwrap()).unwrap();
    assert_eq!(
        network.get(54).unwrap().unary().unwrap().rate_per_s(),
        123.0
    );
    assert_eq!(table.get(54).unwrap(), network.get(54).unwrap().source());
    assert_eq!(
        ReactionTable::option3().unwrap().get(54).unwrap().rate,
        ObservedRate::FirstOrder { per_s: 7.15e5 }
    );
}
