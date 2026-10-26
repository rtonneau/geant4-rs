//! Source-backed coverage and atom/charge audit for all option3 channels.

use g4_chemistry::Species;
use g4_chemistry::reactions::{BulkParticipant, KineticIntent, ReactionParticipant, ReactionTable};
use g4_chemistry::species::{SpeciesTable, atoms};

const MATRIX: &str = include_str!("../../../docs/validation/option3-reaction-coverage.md");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Composition {
    hydrogen: i32,
    oxygen: i32,
    charge: i32,
}

impl std::ops::Add for Composition {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            hydrogen: self.hydrogen + other.hydrogen,
            oxygen: self.oxygen + other.oxygen,
            charge: self.charge + other.charge,
        }
    }
}

fn composition(participant: ReactionParticipant, species: &SpeciesTable) -> Composition {
    match participant {
        ReactionParticipant::Mobile(mobile) => {
            let (hydrogen, oxygen) = atoms(mobile);
            Composition {
                hydrogen,
                oxygen,
                charge: i32::from(species.get(mobile).charge),
            }
        }
        ReactionParticipant::Bulk(bulk) => match bulk {
            BulkParticipant::H2OB => Composition {
                hydrogen: 2,
                oxygen: 1,
                charge: 0,
            },
            BulkParticipant::H3OpB => Composition {
                hydrogen: 3,
                oxygen: 1,
                charge: 1,
            },
            BulkParticipant::OHmB => Composition {
                hydrogen: 1,
                oxygen: 1,
                charge: -1,
            },
            BulkParticipant::NoneM => Composition {
                hydrogen: 0,
                oxygen: 0,
                charge: 0,
            },
        },
    }
}

fn sum(participants: &[ReactionParticipant], species: &SpeciesTable) -> Composition {
    participants.iter().copied().fold(
        Composition {
            hydrogen: 0,
            oxygen: 0,
            charge: 0,
        },
        |total, participant| total + composition(participant, species),
    )
}

#[test]
fn matrix_covers_every_descriptor_and_preserves_ordered_products() {
    let table = ReactionTable::option3().unwrap();
    let species = SpeciesTable::option3().unwrap();
    let network = table.compile(&species).unwrap();
    let rows = MATRIX
        .lines()
        .filter(|line| {
            line.starts_with("| ") && line.as_bytes().get(2).is_some_and(u8::is_ascii_digit)
        })
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 72, "matrix must cover every source ID once");
    assert_eq!(network.iter().count(), rows.len());

    for (index, row) in rows.iter().enumerate() {
        let cells = row.split('|').map(str::trim).collect::<Vec<_>>();
        assert_eq!(cells.len(), 8, "matrix row: {row}");
        let id = u16::try_from(index + 1).unwrap();
        assert_eq!(cells[1].parse::<u16>().unwrap(), id);
        let compiled = network.get(id).unwrap();
        let source = compiled.source();
        assert_eq!(
            cells[2].parse::<u32>().unwrap(),
            source.source_line,
            "ID {id}"
        );
        let regime = match (compiled.pair(), compiled.unary(), source.kinetic_intent) {
            (Some(_), None, KineticIntent::TotallyDiffusionControlled) => "TDC pair",
            (Some(_), None, KineticIntent::PartiallyDiffusionControlled) => {
                let charge_product = source
                    .reactants
                    .iter()
                    .map(|&participant| composition(participant, &species).charge)
                    .product::<i32>();
                if charge_product == 0 {
                    "PDC pair, neutral electrostatics"
                } else {
                    "PDC pair, charged electrostatics"
                }
            }
            (None, Some(_), KineticIntent::FirstOrder) => "first-order unary",
            (None, Some(_), KineticIntent::PseudoFirstOrder) => "pseudo-first-order unary",
            _ => panic!("ID {id} has no unique execution descriptor"),
        };
        assert_eq!(cells[3], regime, "ID {id}");
        let branch = match source.kinetic_intent {
            KineticIntent::TotallyDiffusionControlled => "IRT type 0",
            KineticIntent::PartiallyDiffusionControlled => "IRT type 1",
            KineticIntent::FirstOrder | KineticIntent::PseudoFirstOrder => "Sampling unary",
        };
        assert_eq!(cells[4], branch, "ID {id}");
        let names = |participants: &[ReactionParticipant]| {
            participants
                .iter()
                .map(|participant| participant.chemistry_name())
                .collect::<Vec<_>>()
                .join(", ")
        };
        assert_eq!(cells[5], names(&source.reactants), "ID {id} reactants");
        assert_eq!(
            cells[6],
            names(&source.products),
            "ID {id} ordered products"
        );
    }
}

#[test]
fn every_channel_balances_with_bulk_exchange_and_implicit_water_except_exact_53() {
    let table = ReactionTable::option3().unwrap();
    let species = SpeciesTable::option3().unwrap();
    let mut exceptions = Vec::new();
    for reaction in table.iter() {
        let left = sum(&reaction.reactants, &species);
        let products = sum(&reaction.products, &species);
        let water = Composition {
            hydrogen: 2 * reaction.implicit_water,
            oxygen: reaction.implicit_water,
            charge: 0,
        };
        if reaction.balance_exception {
            exceptions.push(reaction.id);
            assert_eq!(reaction.id, 53);
            assert_eq!(
                reaction.reactants,
                [
                    ReactionParticipant::Mobile(Species::O3m),
                    ReactionParticipant::Bulk(BulkParticipant::NoneM),
                ]
            );
            assert_eq!(
                reaction.products,
                [
                    ReactionParticipant::Mobile(Species::H3Op),
                    ReactionParticipant::Mobile(Species::O2m),
                ]
            );
            assert_eq!(reaction.implicit_water, 0);
            assert_ne!(left, products + water);
        } else {
            assert_eq!(left, products + water, "ID {}", reaction.id);
        }
    }
    assert_eq!(exceptions, [53]);
}
