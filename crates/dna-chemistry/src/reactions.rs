//! Validated option3 chemical reaction table from Geant4 11.4.1.
//!
//! This module preserves source-table facts only. Reaction radii, diffusion-limited rates,
//! activation-controlled rates and scheduler behavior belong to later M7 modules.

use std::collections::HashMap;

use crate::error::ChemistryError;
use crate::species::{SpeciesTable, atoms};
use dna_core::species::Species;
use serde::Deserialize;

const FILE: &str = "reactions_option3.toml";
const EXPECTED_REACTIONS: usize = 72;

/// Embedded Geant4 11.4.1 option3 reaction data.
pub const DEFAULT_REACTIONS_TOML: &str =
    include_str!("../../../data/chemistry/reactions_option3.toml");

/// A non-mobile configuration used to encode a bulk reservoir or sentinel reaction partner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BulkParticipant {
    H2OB,
    H3OpB,
    OHmB,
    NoneM,
}

impl BulkParticipant {
    pub const fn chemistry_name(self) -> &'static str {
        match self {
            Self::H2OB => "H2O(B)",
            Self::H3OpB => "H3Op(B)",
            Self::OHmB => "OHm(B)",
            Self::NoneM => "NoneM",
        }
    }
}

/// A reactant or product identity in the option3 table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReactionParticipant {
    Mobile(Species),
    Bulk(BulkParticipant),
}

impl ReactionParticipant {
    pub fn from_name(name: &str) -> Option<Self> {
        Species::from_chemistry_name(name)
            .map(Self::Mobile)
            .or(match name {
                "H2O(B)" => Some(Self::Bulk(BulkParticipant::H2OB)),
                "H3Op(B)" => Some(Self::Bulk(BulkParticipant::H3OpB)),
                "OHm(B)" => Some(Self::Bulk(BulkParticipant::OHmB)),
                "NoneM" => Some(Self::Bulk(BulkParticipant::NoneM)),
                _ => None,
            })
    }

    pub const fn chemistry_name(self) -> &'static str {
        match self {
            Self::Mobile(species) => species.chemistry_name(),
            Self::Bulk(bulk) => bulk.chemistry_name(),
        }
    }
}

/// Section label used by the C++ source table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum SourceClass {
    I,
    II,
    III,
    IV,
    VI,
    #[serde(rename = "scavenging")]
    Scavenging,
}

/// Kinetic intent recorded by the constructor independently of a runtime chemistry model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KineticIntent {
    TotallyDiffusionControlled,
    PartiallyDiffusionControlled,
    FirstOrder,
    PseudoFirstOrder,
}

/// The executable observed-rate literal, kept in the source unit appropriate to its order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObservedRate {
    /// The C++ numeric literal multiplied by `1e-3 m^3/(mol s)`, equivalently M^-1 s^-1.
    Bimolecular {
        per_molar_s: f64,
    },
    FirstOrder {
        per_s: f64,
    },
}

/// One validated source-table entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Reaction {
    pub id: u16,
    pub source_line: u32,
    pub source_class: SourceClass,
    pub reactants: [ReactionParticipant; 2],
    pub products: Vec<ReactionParticipant>,
    pub kinetic_intent: KineticIntent,
    pub rate: ObservedRate,
    pub set_reaction_type: bool,
    /// Signed count of implicit water products; a negative value means implicit reactant water.
    pub implicit_water: i32,
    /// True only for the exact executable reaction-53 signature in Geant4 11.4.1.
    pub balance_exception: bool,
}

#[derive(Debug, Deserialize)]
struct RawFile {
    geant4_release: String,
    source: String,
    reaction: Vec<RawReaction>,
}

#[derive(Debug, Deserialize)]
struct RawReaction {
    id: u16,
    source_line: u32,
    source_class: SourceClass,
    reactants: [String; 2],
    products: Vec<String>,
    kinetic_intent: KineticIntent,
    rate_order: RateOrder,
    rate_per_molar_s: Option<f64>,
    rate_per_s: Option<f64>,
    set_reaction_type: bool,
    #[allow(dead_code)]
    comment: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RateOrder {
    Bimolecular,
    FirstOrder,
}

/// Immutable option3 reaction network indexed by stable source ID and mobile reactant pair.
#[derive(Debug, Clone)]
pub struct ReactionTable {
    reactions: Vec<Reaction>,
    mobile_pairs: HashMap<(Species, Species), usize>,
}

impl ReactionTable {
    pub fn option3() -> Result<Self, ChemistryError> {
        Self::from_toml(DEFAULT_REACTIONS_TOML)
    }

    pub fn from_toml(text: &str) -> Result<Self, ChemistryError> {
        let raw: RawFile = toml::from_str(text).map_err(|e| ChemistryError::Parse {
            file: FILE,
            message: e.to_string(),
        })?;
        if raw.geant4_release != "11.4.1"
            || raw.source
                != "source/physics_lists/constructors/electromagnetic/src/G4EmDNAChemistry_option3.cc"
        {
            return Err(ChemistryError::invalid(
                FILE,
                "expected Geant4 11.4.1 G4EmDNAChemistry_option3.cc provenance",
            ));
        }
        if raw.reaction.len() != EXPECTED_REACTIONS {
            return Err(ChemistryError::invalid(
                FILE,
                format!(
                    "expected {EXPECTED_REACTIONS} reactions, found {}",
                    raw.reaction.len()
                ),
            ));
        }

        let species = SpeciesTable::option3()?;
        let mut reactions: Vec<Reaction> = Vec::with_capacity(raw.reaction.len());
        let mut mobile_pairs = HashMap::new();
        for (index, item) in raw.reaction.into_iter().enumerate() {
            let expected_id = u16::try_from(index + 1).expect("72 IDs fit u16");
            if item.id != expected_id {
                return Err(ChemistryError::invalid(
                    FILE,
                    format!(
                        "reaction ID {} is not contiguous; expected {expected_id}",
                        item.id
                    ),
                ));
            }
            if item.source_line == 0 {
                return Err(ChemistryError::invalid(
                    FILE,
                    format!("reaction {} has source line 0", item.id),
                ));
            }
            let reactants = [
                parse_participant(item.id, &item.reactants[0])?,
                parse_participant(item.id, &item.reactants[1])?,
            ];
            let products = item
                .products
                .iter()
                .map(|name| parse_participant(item.id, name))
                .collect::<Result<Vec<_>, _>>()?;
            let rate = parse_rate(
                item.id,
                item.rate_order,
                item.rate_per_molar_s,
                item.rate_per_s,
            )?;
            validate_intent(
                item.id,
                item.kinetic_intent,
                item.rate_order,
                item.set_reaction_type,
            )?;

            let exact_anomaly = is_exact_geant4_anomaly(item.id, &reactants, &products);
            if item.id == 53 && !exact_anomaly {
                return Err(ChemistryError::invalid(
                    FILE,
                    "reaction 53 does not match the audited Geant4 anomaly signature",
                ));
            }
            let implicit_water = balance(item.id, &reactants, &products, &species, exact_anomaly)?;

            if let [
                ReactionParticipant::Mobile(a),
                ReactionParticipant::Mobile(b),
            ] = reactants
            {
                let pair = canonical_pair(a, b);
                if let Some(previous) = mobile_pairs.insert(pair, index) {
                    return Err(ChemistryError::invalid(
                        FILE,
                        format!(
                            "reaction {} duplicates unordered mobile pair from reaction {}",
                            item.id, reactions[previous].id
                        ),
                    ));
                }
            }
            reactions.push(Reaction {
                id: item.id,
                source_line: item.source_line,
                source_class: item.source_class,
                reactants,
                products,
                kinetic_intent: item.kinetic_intent,
                rate,
                set_reaction_type: item.set_reaction_type,
                implicit_water,
                balance_exception: exact_anomaly,
            });
        }
        Ok(Self {
            reactions,
            mobile_pairs,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Reaction> {
        self.reactions.iter()
    }

    pub fn get(&self, id: u16) -> Option<&Reaction> {
        let index = usize::from(id.checked_sub(1)?);
        self.reactions
            .get(index)
            .filter(|reaction| reaction.id == id)
    }

    pub fn find_mobile_pair(&self, a: Species, b: Species) -> Option<&Reaction> {
        self.mobile_pairs
            .get(&canonical_pair(a, b))
            .and_then(|&index| self.reactions.get(index))
    }
}

fn parse_participant(id: u16, name: &str) -> Result<ReactionParticipant, ChemistryError> {
    ReactionParticipant::from_name(name).ok_or_else(|| {
        ChemistryError::invalid(
            FILE,
            format!("reaction {id} has unknown participant '{name}'"),
        )
    })
}

fn parse_rate(
    id: u16,
    order: RateOrder,
    bimolecular: Option<f64>,
    first_order: Option<f64>,
) -> Result<ObservedRate, ChemistryError> {
    let rate = match (order, bimolecular, first_order) {
        (RateOrder::Bimolecular, Some(value), None) => {
            ObservedRate::Bimolecular { per_molar_s: value }
        }
        (RateOrder::FirstOrder, None, Some(value)) => ObservedRate::FirstOrder { per_s: value },
        _ => {
            return Err(ChemistryError::invalid(
                FILE,
                format!("reaction {id} has a rate field inconsistent with its order"),
            ));
        }
    };
    let value = match rate {
        ObservedRate::Bimolecular { per_molar_s } => per_molar_s,
        ObservedRate::FirstOrder { per_s } => per_s,
    };
    if !value.is_finite() || value <= 0.0 {
        return Err(ChemistryError::invalid(
            FILE,
            format!("reaction {id} has a non-positive or non-finite observed rate"),
        ));
    }
    Ok(rate)
}

fn validate_intent(
    id: u16,
    intent: KineticIntent,
    order: RateOrder,
    set_reaction_type: bool,
) -> Result<(), ChemistryError> {
    let valid = matches!(
        (order, intent, set_reaction_type),
        (
            RateOrder::Bimolecular,
            KineticIntent::PartiallyDiffusionControlled,
            true
        ) | (
            RateOrder::Bimolecular,
            KineticIntent::TotallyDiffusionControlled,
            false
        ) | (RateOrder::FirstOrder, KineticIntent::FirstOrder, false)
            | (
                RateOrder::FirstOrder,
                KineticIntent::PseudoFirstOrder,
                false
            )
    );
    if valid {
        Ok(())
    } else {
        Err(ChemistryError::invalid(
            FILE,
            format!("reaction {id} has inconsistent kinetic intent/type fields"),
        ))
    }
}

fn canonical_pair(a: Species, b: Species) -> (Species, Species) {
    if a <= b { (a, b) } else { (b, a) }
}

fn participant_composition(
    participant: ReactionParticipant,
    species: &SpeciesTable,
) -> (i32, i32, i32) {
    match participant {
        ReactionParticipant::Mobile(mobile) => {
            let (hydrogen, oxygen) = atoms(mobile);
            (hydrogen, oxygen, i32::from(species.get(mobile).charge))
        }
        ReactionParticipant::Bulk(BulkParticipant::H2OB) => (2, 1, 0),
        ReactionParticipant::Bulk(BulkParticipant::H3OpB) => (3, 1, 1),
        ReactionParticipant::Bulk(BulkParticipant::OHmB) => (1, 1, -1),
        ReactionParticipant::Bulk(BulkParticipant::NoneM) => (0, 0, 0),
    }
}

fn sum_composition(
    participants: &[ReactionParticipant],
    species: &SpeciesTable,
) -> (i32, i32, i32) {
    participants.iter().fold((0, 0, 0), |sum, &participant| {
        let value = participant_composition(participant, species);
        (sum.0 + value.0, sum.1 + value.1, sum.2 + value.2)
    })
}

fn balance(
    id: u16,
    reactants: &[ReactionParticipant; 2],
    products: &[ReactionParticipant],
    species: &SpeciesTable,
    allow_exact_anomaly: bool,
) -> Result<i32, ChemistryError> {
    let reactant = sum_composition(reactants, species);
    let product = sum_composition(products, species);
    let implicit_water_products = reactant.1 - product.1;
    let balanced = reactant.2 == product.2 && reactant.0 - product.0 == 2 * implicit_water_products;
    if balanced {
        return Ok(implicit_water_products);
    }
    if allow_exact_anomaly {
        return Ok(0);
    }
    Err(ChemistryError::invalid(
        FILE,
        format!("reaction {id} does not conserve atoms and charge with implicit water"),
    ))
}

fn is_exact_geant4_anomaly(
    id: u16,
    reactants: &[ReactionParticipant; 2],
    products: &[ReactionParticipant],
) -> bool {
    id == 53
        && reactants
            == &[
                ReactionParticipant::Mobile(Species::O3m),
                ReactionParticipant::Bulk(BulkParticipant::NoneM),
            ]
        && products
            == [
                ReactionParticipant::Mobile(Species::H3Op),
                ReactionParticipant::Mobile(Species::O2m),
            ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use dna_core::species::Species;

    fn replace_once(source: &str, from: &str, to: &str) -> String {
        assert!(source.contains(from), "fixture edit target missing: {from}");
        source.replacen(from, to, 1)
    }

    #[test]
    fn embedded_table_loads_all_source_ordered_reactions() {
        let table = ReactionTable::option3().unwrap();
        assert_eq!(table.iter().count(), 72);
        assert_eq!(table.get(1).unwrap().id, 1);
        assert_eq!(table.get(72).unwrap().id, 72);
        assert_eq!(
            table.get(1).unwrap().rate,
            ObservedRate::Bimolecular {
                per_molar_s: 0.503e10
            }
        );
        let anomaly = table.get(53).unwrap();
        assert_eq!(anomaly.source_class, SourceClass::VI);
        assert_eq!(anomaly.rate, ObservedRate::FirstOrder { per_s: 2.66e3 });
        assert_eq!(
            anomaly.reactants,
            [
                ReactionParticipant::Mobile(Species::O3m),
                ReactionParticipant::Bulk(BulkParticipant::NoneM),
            ]
        );
        assert_eq!(
            anomaly.products,
            [
                ReactionParticipant::Mobile(Species::H3Op),
                ReactionParticipant::Mobile(Species::O2m),
            ]
        );
    }

    #[test]
    fn mobile_pair_lookup_is_unordered() {
        let table = ReactionTable::option3().unwrap();
        let forward = table.find_mobile_pair(Species::H, Species::OH).unwrap();
        let reverse = table.find_mobile_pair(Species::OH, Species::H).unwrap();
        assert_eq!(forward.id, reverse.id);
        assert_eq!(forward.id, 11);
        assert!(
            table
                .find_mobile_pair(Species::H3Op, Species::H3Op)
                .is_none()
        );
    }

    #[test]
    fn bulk_participants_do_not_parse_as_mobile_species() {
        for (name, bulk) in [
            ("H2O(B)", BulkParticipant::H2OB),
            ("H3Op(B)", BulkParticipant::H3OpB),
            ("OHm(B)", BulkParticipant::OHmB),
            ("NoneM", BulkParticipant::NoneM),
        ] {
            assert_eq!(Species::from_chemistry_name(name), None);
            assert_eq!(
                ReactionParticipant::from_name(name),
                Some(ReactionParticipant::Bulk(bulk))
            );
        }
    }

    #[test]
    fn malformed_ids_rates_units_tags_and_duplicate_pairs_are_rejected() {
        let cases = [
            replace_once(DEFAULT_REACTIONS_TOML, "id = 2", "id = 1"),
            replace_once(DEFAULT_REACTIONS_TOML, "id = 2", "id = 3"),
            replace_once(
                DEFAULT_REACTIONS_TOML,
                "rate_per_molar_s = 0.503e10",
                "rate_per_molar_s = 0.0",
            ),
            replace_once(
                DEFAULT_REACTIONS_TOML,
                "rate_per_molar_s = 0.503e10",
                "rate_per_s = 0.503e10",
            ),
            replace_once(
                DEFAULT_REACTIONS_TOML,
                "reactants = [\"H\", \"H\"]",
                "reactants = [\"°OH\", \"H\"]",
            ),
            replace_once(
                DEFAULT_REACTIONS_TOML,
                "reactants = [\"H\", \"H\"]",
                "reactants = [\"not-a-species\", \"H\"]",
            ),
        ];
        for text in cases {
            assert!(ReactionTable::from_toml(&text).is_err());
        }
        assert!(ReactionTable::from_toml("not = [valid").is_err());
    }

    #[test]
    fn balanced_duplicate_mobile_pair_is_rejected_as_a_duplicate() {
        let duplicate = replace_once(
            DEFAULT_REACTIONS_TOML,
            "reactants = [\"e_aq\", \"H\"]\nproducts = [\"OHm\", \"H2\"]",
            "reactants = [\"H\", \"H\"]\nproducts = [\"H2\"]",
        );
        let error = ReactionTable::from_toml(&duplicate).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("duplicates unordered mobile pair")
        );
    }

    #[test]
    fn conservation_uses_implicit_water_and_only_the_exact_anomaly_is_allowed() {
        let table = ReactionTable::option3().unwrap();
        assert_eq!(table.get(10).unwrap().implicit_water, 1);
        assert!(table.get(53).unwrap().balance_exception);
        assert_eq!(table.iter().filter(|r| r.balance_exception).count(), 1);

        let changed_id = replace_once(DEFAULT_REACTIONS_TOML, "id = 53", "id = 153");
        assert!(ReactionTable::from_toml(&changed_id).is_err());
        let changed_product = replace_once(
            DEFAULT_REACTIONS_TOML,
            "reactants = [\"O3m\", \"NoneM\"]\nproducts = [\"H3Op\", \"O2m\"]",
            "reactants = [\"O3m\", \"NoneM\"]\nproducts = [\"Om\", \"O2\"]",
        );
        assert!(ReactionTable::from_toml(&changed_product).is_err());
        let new_imbalance = replace_once(
            DEFAULT_REACTIONS_TOML,
            "products = [\"H2\"]",
            "products = [\"H3Op\"]",
        );
        assert!(ReactionTable::from_toml(&new_imbalance).is_err());
    }
}
