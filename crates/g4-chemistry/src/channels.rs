//! Decay channels of the excited / ionised / attached water molecule (`data/chemistry/dissociation_option3.toml`,
//! Geant4 `G4ChemDissociationChannels_option1::ConstructDissociationChannels`).
//!
//! # Mapping from a physical-stage event to a decay state
//! `G4DNAChemistryManager::CreateWaterMolecule` (utils/src/G4DNAChemistryManager.cc:600-660) builds the water
//! molecule with `ExciteMolecule(4 - level)` / `IonizeMolecule(4 - shell)` / `AddElectron(5, 1)`; the
//! electronic configuration so obtained is the one registered under a name such as `A^1B_1` or `Ioni5`. The data
//! file lists those states with their `index` (the physics level or shell) and the `orbit_vacated`; the loader
//! checks `orbit_vacated == 4 - index`.
//!
//! # Channel selection
//! `G4DNAMolecularDissociation::DecayIt` (processes/src/G4DNAMolecularDissociation.cc:142-153): one uniform
//! number `u`; scan the channels in file order, select the first with `u < p`, otherwise `u -= p`. If rounding lets
//! the scan run past the end the last channel stays selected (the C++ keeps the pointer of the last iteration).
//!
//! All option1 configurations inherit the zero water lifetime. Product times are assigned by the engine
//! at 1 ps, with no additional channel delay (`G4H2O.cc:65`, `G4MolecularConfiguration.cc:96,439,481`,
//! `G4DNAMolecularDissociation.cc:258`). The `H2Ovib` relaxation has zero released energy: its constructor
//! default is never overridden, unlike the five excited-state relaxations (docs/02 B.7.8).

use crate::Species;
use crate::error::ChemistryError;
use crate::species::{SpeciesTable, atoms};
use g4_core::physchem::PhysChemKind;
use g4_core::units::Energy;
use serde::Deserialize;

const FILE: &str = "dissociation_option3.toml";

/// Embedded default data.
pub const DEFAULT_DISSOCIATION_TOML: &str =
    include_str!("../../../data/chemistry/dissociation_option3.toml");

/// Tolerance of the probability sum of a state (docs/04 D.2).
pub const PROBABILITY_SUM_TOLERANCE: f64 = 1e-12;

/// Displacement law of a channel: the cases of `G4DNAWaterDissociationDisplacer` used by option2 events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplacementKind {
    /// Channel without products (relaxation).
    None,
    /// `A1B1_DissociationDecay`: OH + H, bond axis of RMS 2.4 nm.
    A1b1Dissociation,
    /// `B1A1_DissociationDecay`: H2 + 2 OH.
    B1a1Dissociation,
    /// `B1A1_DissociationDecay2`: O + 2 H.
    B1a1Dissociation2,
    /// `AutoIonisation`: OH + H3O+ + e_aq.
    AutoIonisation,
    /// `Ionisation_DissociationDecay`: H3O+ + OH.
    IonisationDissociation,
    /// `DissociativeAttachment`: H2 + OH- + OH.
    DissociativeAttachment,
}

impl DisplacementKind {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "none" => Self::None,
            "a1b1_dissociation_decay" => Self::A1b1Dissociation,
            "b1a1_dissociation_decay" => Self::B1a1Dissociation,
            "b1a1_dissociation_decay2" => Self::B1a1Dissociation2,
            "auto_ionisation" => Self::AutoIonisation,
            "ionisation_dissociation_decay" => Self::IonisationDissociation,
            "dissociative_attachment" => Self::DissociativeAttachment,
            _ => return Option::None,
        })
    }

    /// The name of the case in `G4DNAWaterDissociationDisplacer` (for the cross-check with the extraction).
    pub const fn cpp_case(self) -> Option<&'static str> {
        match self {
            Self::None => Option::None,
            Self::A1b1Dissociation => Some("A1B1_DissociationDecay"),
            Self::B1a1Dissociation => Some("B1A1_DissociationDecay"),
            Self::B1a1Dissociation2 => Some("B1A1_DissociationDecay2"),
            Self::AutoIonisation => Some("AutoIonisation"),
            Self::IonisationDissociation => Some("Ionisation_DissociationDecay"),
            Self::DissociativeAttachment => Some("DissociativeAttachment"),
        }
    }
}

/// One decay channel.
#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    pub name: String,
    pub probability: f64,
    /// Energy booked as local deposit by the chemistry-stage decay (`DecayIt`). Not part of the
    /// physical-stage energy ledger (docs/09 Q20).
    pub decay_energy: Energy,
    pub products: Vec<Species>,
    pub displacement: DisplacementKind,
}

/// What kind of molecule decays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateKind {
    Excitation,
    Ionisation,
    Attachment,
    /// `H2Ovib`: the ionised molecule after electron-hole recombination.
    Recombined,
}

/// One decay state of the water molecule (an electronic configuration registered in Geant4).
#[derive(Debug, Clone, PartialEq)]
pub struct DecayState {
    pub name: String,
    pub kind: StateKind,
    /// Physics level (excitation) or shell (ionisation); 0 otherwise.
    pub index: u8,
    pub orbit_vacated: Option<u8>,
    pub channels: Vec<Channel>,
}

impl DecayState {
    /// Channel chosen by the uniform number `u` (see the module documentation); returns its index.
    pub fn select(&self, u: f64) -> usize {
        let mut rest = u;
        for (i, c) in self.channels.iter().enumerate() {
            if rest < c.probability {
                return i;
            }
            rest -= c.probability;
        }
        self.channels.len() - 1
    }
}

#[derive(Debug, Deserialize)]
struct RawFile {
    state: Vec<RawState>,
}

#[derive(Debug, Deserialize)]
struct RawState {
    name: String,
    kind: String,
    index: u8,
    orbit_vacated: Option<u8>,
    #[allow(dead_code)]
    cpp_line: u32,
    channel: Vec<RawChannel>,
}

#[derive(Debug, Deserialize)]
struct RawChannel {
    name: String,
    probability: f64,
    decay_energy_ev: f64,
    products: Vec<String>,
    displacement: String,
}

/// All decay states.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTable {
    states: Vec<DecayState>,
    excitation: [usize; 5],
    ionisation: [usize; 5],
    attachment: usize,
    recombined: usize,
}

impl ChannelTable {
    /// The embedded option3 table, validated against the embedded species table.
    pub fn option3(species: &SpeciesTable) -> Result<Self, ChemistryError> {
        Self::from_toml(DEFAULT_DISSOCIATION_TOML, species)
    }

    /// Parse and validate. Checks, per state: probabilities sum to 1 within [`PROBABILITY_SUM_TOLERANCE`];
    /// every product uses an exact Geant4 configuration key and every displacement name is known;
    /// a relax channel (no products) uses `none`; elemental
    /// balance against the parent molecule (excited H2O, H2O+, H2O-, or the recombined H2O) with 0 or 1
    /// molecule of the medium added (the H3O+ of an ionised molecule is H2O+ plus one water), and charge
    /// conservation; `orbit_vacated == 4 - index` for excitations and ionisations; every index 0..=4 present
    /// once for excitation and ionisation, exactly one attachment and one recombined state.
    pub fn from_toml(text: &str, species: &SpeciesTable) -> Result<Self, ChemistryError> {
        let raw: RawFile = toml::from_str(text).map_err(|e| ChemistryError::Parse {
            file: FILE,
            message: e.to_string(),
        })?;
        let mut states = Vec::with_capacity(raw.state.len());
        for rs in raw.state {
            let kind = match rs.kind.as_str() {
                "excitation" => StateKind::Excitation,
                "ionisation" => StateKind::Ionisation,
                "attachment" => StateKind::Attachment,
                "recombined" => StateKind::Recombined,
                other => {
                    return Err(ChemistryError::invalid(
                        FILE,
                        format!("state '{}': unknown kind '{other}'", rs.name),
                    ));
                }
            };
            if matches!(kind, StateKind::Excitation | StateKind::Ionisation)
                && (rs.index > 4 || rs.orbit_vacated != Some(4 - rs.index.min(4)))
            {
                return Err(ChemistryError::invalid(
                    FILE,
                    format!("state '{}': orbit_vacated must be 4 - index", rs.name),
                ));
            }
            let parent_charge: i32 = match kind {
                StateKind::Excitation | StateKind::Recombined => 0,
                StateKind::Ionisation => 1,
                StateKind::Attachment => -1,
            };
            let mut channels = Vec::with_capacity(rs.channel.len());
            let mut sum = 0.0;
            for rc in rs.channel {
                let displacement =
                    DisplacementKind::from_name(&rc.displacement).ok_or_else(|| {
                        ChemistryError::invalid(
                            FILE,
                            format!(
                                "channel '{}': unknown displacement '{}'",
                                rc.name, rc.displacement
                            ),
                        )
                    })?;
                let mut products = Vec::with_capacity(rc.products.len());
                for t in &rc.products {
                    products.push(Species::from_chemistry_name(t).ok_or_else(|| {
                        ChemistryError::invalid(
                            FILE,
                            format!("channel '{}': unknown product '{t}'", rc.name),
                        )
                    })?);
                }
                if products.is_empty() != (displacement == DisplacementKind::None) {
                    return Err(ChemistryError::invalid(
                        FILE,
                        format!("channel '{}': displacement 'none' iff no products", rc.name),
                    ));
                }
                if !products.is_empty() {
                    let h: i32 = products.iter().map(|&s| atoms(s).0).sum();
                    let o: i32 = products.iter().map(|&s| atoms(s).1).sum();
                    let q: i32 = products
                        .iter()
                        .map(|&s| i32::from(species.get(s).charge))
                        .sum();
                    // parent H2O has 2 H and 1 O; the products may hold n extra water molecules.
                    let extra_o = o - 1;
                    if !(0..=1).contains(&extra_o) || h - 2 != 2 * extra_o || q != parent_charge {
                        return Err(ChemistryError::invalid(
                            FILE,
                            format!(
                                "channel '{}' ({}): products H={h} O={o} charge={q} do not balance the parent \
                                 (charge {parent_charge}) plus 0 or 1 water",
                                rc.name, rs.name
                            ),
                        ));
                    }
                }
                if !(0.0..=1.0).contains(&rc.probability) || rc.decay_energy_ev < 0.0 {
                    return Err(ChemistryError::invalid(
                        FILE,
                        format!("channel '{}': probability or energy out of range", rc.name),
                    ));
                }
                sum += rc.probability;
                channels.push(Channel {
                    name: rc.name,
                    probability: rc.probability,
                    decay_energy: Energy::from_ev(rc.decay_energy_ev),
                    products,
                    displacement,
                });
            }
            if (sum - 1.0).abs() > PROBABILITY_SUM_TOLERANCE {
                return Err(ChemistryError::invalid(
                    FILE,
                    format!("state '{}': probabilities sum to {sum}, not 1", rs.name),
                ));
            }
            if channels.is_empty() {
                return Err(ChemistryError::invalid(
                    FILE,
                    format!("state '{}' has no channel", rs.name),
                ));
            }
            states.push(DecayState {
                name: rs.name,
                kind,
                index: rs.index,
                orbit_vacated: rs.orbit_vacated,
                channels,
            });
        }
        let find = |kind: StateKind, index: u8| -> Result<usize, ChemistryError> {
            let hits: Vec<usize> = states
                .iter()
                .enumerate()
                .filter(|(_, s)| s.kind == kind && s.index == index)
                .map(|(i, _)| i)
                .collect();
            match hits.as_slice() {
                [i] => Ok(*i),
                _ => Err(ChemistryError::invalid(
                    FILE,
                    format!(
                        "expected exactly one {kind:?} state with index {index}, found {}",
                        hits.len()
                    ),
                )),
            }
        };
        let mut excitation = [0; 5];
        let mut ionisation = [0; 5];
        for k in 0..5u8 {
            excitation[k as usize] = find(StateKind::Excitation, k)?;
            ionisation[k as usize] = find(StateKind::Ionisation, k)?;
        }
        let attachment = find(StateKind::Attachment, 0)?;
        let recombined = find(StateKind::Recombined, 0)?;
        Ok(Self {
            states,
            excitation,
            ionisation,
            attachment,
            recombined,
        })
    }

    pub fn states(&self) -> &[DecayState] {
        &self.states
    }

    /// Index in [`ChannelTable::states`] of the state created by a physical-stage event; solvation events
    /// create no water molecule.
    pub fn state_index(&self, kind: PhysChemKind, index: u8) -> Result<usize, ChemistryError> {
        let unknown = || ChemistryError::UnknownIndex {
            kind: kind.name(),
            index,
        };
        match kind {
            PhysChemKind::Excitation => self
                .excitation
                .get(usize::from(index))
                .copied()
                .ok_or_else(unknown),
            PhysChemKind::Ionisation => self
                .ionisation
                .get(usize::from(index))
                .copied()
                .ok_or_else(unknown),
            PhysChemKind::Attachment => {
                if index == 0 {
                    Ok(self.attachment)
                } else {
                    Err(unknown())
                }
            }
            PhysChemKind::Solvation => Err(unknown()),
        }
    }

    /// Index of the recombined (`H2Ovib`) state.
    pub fn recombined_index(&self) -> usize {
        self.recombined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> ChannelTable {
        ChannelTable::option3(&SpeciesTable::option3().unwrap()).unwrap()
    }

    #[test]
    fn embedded_table_loads_with_all_states() {
        let t = table();
        assert_eq!(t.states().len(), 5 + 5 + 1 + 1);
        let n_channels: usize = t.states().iter().map(|s| s.channels.len()).sum();
        assert_eq!(n_channels, 23);
        assert_eq!(
            t.states()[t.state_index(PhysChemKind::Excitation, 0).unwrap()].name,
            "A^1B_1"
        );
        assert_eq!(
            t.states()[t.state_index(PhysChemKind::Ionisation, 0).unwrap()].name,
            "Ioni5"
        );
        assert_eq!(
            t.states()[t.state_index(PhysChemKind::Ionisation, 4).unwrap()].name,
            "Ioni1"
        );
        assert_eq!(t.states()[t.recombined_index()].name, "H2Ovib");
    }

    #[test]
    fn selection_follows_the_cpp_scan_and_clamps_to_the_last_channel() {
        let t = table();
        let b1a1 = &t.states()[t.state_index(PhysChemKind::Excitation, 1).unwrap()];
        // p = 0.175, 0.0325, 0.5, 0.2535, 0.039 -> cumulative 0.175, 0.2075, 0.7075, 0.961, 1.0
        assert_eq!(b1a1.select(0.0), 0);
        assert_eq!(b1a1.select(0.174), 0);
        assert_eq!(b1a1.select(0.176), 1);
        assert_eq!(b1a1.select(0.21), 2);
        assert_eq!(b1a1.select(0.70), 2);
        assert_eq!(b1a1.select(0.71), 3);
        assert_eq!(b1a1.select(0.97), 4);
        assert_eq!(b1a1.select(1.5), 4); // past the end: the last channel
    }

    #[test]
    fn invalid_data_are_rejected() {
        let sp = SpeciesTable::option3().unwrap();
        let bad_sum =
            DEFAULT_DISSOCIATION_TOML.replacen("probability = 0.65", "probability = 0.64", 1);
        assert!(ChannelTable::from_toml(&bad_sum, &sp).is_err());
        let bad_tag = DEFAULT_DISSOCIATION_TOML.replacen("\"°OH\", \"H\"", "\"°OH\", \"HH\"", 1);
        assert!(ChannelTable::from_toml(&bad_tag, &sp).is_err());
        // an unbalanced channel: replace H by H2 in the A1B1 dissociation
        let unbalanced = DEFAULT_DISSOCIATION_TOML.replacen(
            "products = [\"°OH\", \"H\"]",
            "products = [\"°OH\", \"H2\"]",
            1,
        );
        assert!(ChannelTable::from_toml(&unbalanced, &sp).is_err());
        let bad_orbit =
            DEFAULT_DISSOCIATION_TOML.replacen("orbit_vacated = 4", "orbit_vacated = 3", 1);
        assert!(ChannelTable::from_toml(&bad_orbit, &sp).is_err());
    }

    #[test]
    fn legacy_product_aliases_are_rejected() {
        let sp = SpeciesTable::option3().unwrap();
        for (name, alias) in [("°OH", "OH"), ("°OH", "HO2"), ("Oxy", "O")] {
            let text = DEFAULT_DISSOCIATION_TOML.replacen(
                &format!("\"{name}\""),
                &format!("\"{alias}\""),
                1,
            );
            let err = ChannelTable::from_toml(&text, &sp).unwrap_err();
            assert!(
                err.to_string()
                    .contains(&format!("unknown product '{alias}'"))
            );
        }
    }

    #[test]
    fn unknown_indices_are_errors() {
        let t = table();
        assert!(t.state_index(PhysChemKind::Excitation, 5).is_err());
        assert!(t.state_index(PhysChemKind::Ionisation, 9).is_err());
        assert!(t.state_index(PhysChemKind::Attachment, 1).is_err());
        assert!(t.state_index(PhysChemKind::Solvation, 0).is_err());
    }
}
