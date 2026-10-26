//! Process table: which model serves which process at which energy.
//!
//! Selection between several models of one process follows `G4EmModelManager`
//! (`dna`-independent, `processes/electromagnetic/utils/include/G4EmModelManager.hh:97-104`,
//! `G4RegionModels::SelectIndex`): start at the last model and step down while
//! `e <= low(model)`. A model therefore owns the **half-open** interval `(low, high]` and the
//! *lower* model owns a shared boundary, except the first model which also owns its lower bound:
//! `[low, high]` (docs/02 B.7.3 item 5). Each model additionally returns zero outside its own
//! closed window, so an energy in a gap between two models selects nothing here.

use crate::error::PhysicsError;
use crate::models::{
    BornExcitation, BornIonisation, ChampionElastic, CrossSectionModel, ElasticRotation,
    ElectronModel, MeltonAttachment, ProcessKind, SancheVibrational, Solvation,
};
use crate::water::WATER_MOLECULES_PER_NM3;
use g4_core::manifest::Manifest;
use g4_core::units::{Energy, EnergyRange};
use g4_data::datadir::{DataDir, VerifiedDataSet};
use serde::Serialize;

/// Models of one process ordered by ascending energy window.
#[derive(Debug, Clone)]
pub struct ModelChain<M> {
    models: Vec<M>,
}

impl<M: CrossSectionModel> ModelChain<M> {
    /// `models` must be non-empty and ordered so that each window starts at or above the end of
    /// the previous one (touching windows share their boundary, which the lower model owns).
    pub fn new(models: Vec<M>) -> Result<Self, PhysicsError> {
        if models.is_empty() {
            return Err(PhysicsError::InvalidModelChain("empty chain".into()));
        }
        for pair in models.windows(2) {
            let (a, b) = (pair[0].energy_range(), pair[1].energy_range());
            if b.min() < a.max() {
                return Err(PhysicsError::InvalidModelChain(format!(
                    "{} [{}, {}] eV overlaps {} [{}, {}] eV",
                    pair[0].name(),
                    a.min().ev(),
                    a.max().ev(),
                    pair[1].name(),
                    b.min().ev(),
                    b.max().ev()
                )));
            }
        }
        Ok(Self { models })
    }

    pub fn models(&self) -> &[M] {
        &self.models
    }

    /// Energy interval covered by at least one model (first low to last high).
    pub fn energy_range(&self) -> EnergyRange {
        let first = self.models[0].energy_range().min();
        let last = self.models[self.models.len() - 1].energy_range().max();
        EnergyRange::new(first, last).expect("chain windows are ordered")
    }

    /// Model serving `e`, if any (`(low, high]` between models, `[low, high]` for the first).
    pub fn select(&self, e: Energy) -> Option<&M> {
        let mut idx = self.models.len() - 1;
        while idx > 0 && e <= self.models[idx].energy_range().min() {
            idx -= 1;
        }
        let w = self.models[idx].energy_range();
        let inside = e <= w.max() && (idx > 0 || e >= w.min());
        inside.then(|| &self.models[idx])
    }
}

/// One line of the model inventory written to the output provenance.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelInfo {
    /// `None` for solvation, which is not cross-section driven.
    pub process: Option<ProcessKind>,
    pub geant4_process: &'static str,
    pub model: &'static str,
    pub low_ev: f64,
    pub high_ev: f64,
    pub geant4_source: &'static str,
    pub data_files: Vec<&'static str>,
}

/// Macroscopic cross sections (1/nm) of the cross-section-driven processes at one energy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacroscopicCrossSections {
    per_nm: [f64; ProcessKind::COUNT],
}

impl MacroscopicCrossSections {
    /// From per-process values in [`ProcessKind::ALL`] order (1/nm).
    pub const fn new(per_nm: [f64; ProcessKind::COUNT]) -> Self {
        Self { per_nm }
    }
    pub fn get(&self, process: ProcessKind) -> f64 {
        self.per_nm[process.index()]
    }
    pub fn total(&self) -> f64 {
        self.per_nm.iter().sum()
    }
    /// Sum of the processes that change the electron's energy or absorb it (everything but
    /// elastic scattering).
    pub fn inelastic(&self) -> f64 {
        self.total() - self.per_nm[ProcessKind::Elastic.index()]
    }
    pub fn as_array(&self) -> [f64; ProcessKind::COUNT] {
        self.per_nm
    }
}

/// The processes of one particle in one material.
#[derive(Debug, Clone)]
pub struct ProcessTable {
    chains: [Option<ModelChain<ElectronModel>>; ProcessKind::COUNT],
    solvation: Option<Solvation>,
    molecules_per_nm3: f64,
}

/// Production option2 models permanently bound to the manifest of their verified input bytes.
///
/// Construction accepts only a [`VerifiedDataSet`]; there is no operation that labels an
/// arbitrary [`ProcessTable`] as verified. Both the table and its identity are read-only.
#[derive(Debug, Clone)]
pub struct VerifiedProcessTable {
    table: ProcessTable,
    identity: Manifest,
}

impl VerifiedProcessTable {
    /// Build the standard option2 electron models from captured, checked tables only.
    /// The full supplied manifest was verified when the snapshot was loaded, including files
    /// not used by these models. Model windows and sampling policies match the unverified path.
    pub fn option2_electron(
        data: &VerifiedDataSet,
        rotation: ElasticRotation,
    ) -> Result<Self, PhysicsError> {
        let table = ProcessTable::from_models(
            vec![
                ElectronModel::Elastic(ChampionElastic::from_verified_data(data, rotation)?),
                ElectronModel::Excitation(BornExcitation::from_verified_data(data)?),
                ElectronModel::Ionisation(BornIonisation::from_verified_data(data)?),
                ElectronModel::Vibrational(SancheVibrational::from_verified_data(data)?),
                ElectronModel::Attachment(MeltonAttachment::from_verified_data(data)?),
            ],
            Some(Solvation::new()),
        )?;
        Ok(Self {
            table,
            identity: data.identity().clone(),
        })
    }

    /// The frozen process table, suitable for transport and inspection.
    pub fn table(&self) -> &ProcessTable {
        &self.table
    }

    /// Verified manifest identity, retaining its original file order and no source root.
    pub fn identity(&self) -> &Manifest {
        &self.identity
    }
}

impl ProcessTable {
    /// `G4EmDNAPhysics_option2` electron processes (M3: complete below 1 MeV): Champion elastic
    /// (7.4 eV - 1 MeV), Born excitation (9 eV - 1 MeV), Born1 ionisation (11 eV - 1 MeV), Sanche
    /// vibrational excitation (2 - 100 eV), Melton attachment (4 - 13 eV) and solvation (0 - 7.4
    /// eV), in liquid water of Geant4 `G4_WATER` density.
    ///
    /// Compatibility entry point without checksum verification or a verified identity.
    /// Production inputs should use [`VerifiedProcessTable::option2_electron`].
    pub fn option2_electron(
        dir: &DataDir,
        rotation: ElasticRotation,
    ) -> Result<Self, PhysicsError> {
        Self::option2_electron_unverified(dir, rotation)
    }

    /// Load option2 tables without checksum verification, for tests and exploratory input.
    /// This cannot produce a [`VerifiedProcessTable`] or claim a verified manifest identity.
    pub fn option2_electron_unverified(
        dir: &DataDir,
        rotation: ElasticRotation,
    ) -> Result<Self, PhysicsError> {
        Self::from_models(
            vec![
                ElectronModel::Elastic(ChampionElastic::from_data_dir(dir, rotation)?),
                ElectronModel::Excitation(BornExcitation::from_data_dir(dir)?),
                ElectronModel::Ionisation(BornIonisation::from_data_dir(dir)?),
                ElectronModel::Vibrational(SancheVibrational::from_data_dir(dir)?),
                ElectronModel::Attachment(MeltonAttachment::from_data_dir(dir)?),
            ],
            Some(Solvation::new()),
        )
    }

    /// Build from explicit models (grouped by [`ProcessKind`], each group ordered by ascending
    /// window) and an optional solvation component. Processes without a model contribute a zero
    /// cross section.
    pub fn from_models(
        models: Vec<ElectronModel>,
        solvation: Option<Solvation>,
    ) -> Result<Self, PhysicsError> {
        let mut groups: [Vec<ElectronModel>; ProcessKind::COUNT] = Default::default();
        for m in models {
            groups[m.process().index()].push(m);
        }
        let mut chains: [Option<ModelChain<ElectronModel>>; ProcessKind::COUNT] =
            Default::default();
        for (slot, group) in chains.iter_mut().zip(groups) {
            if !group.is_empty() {
                *slot = Some(ModelChain::new(group)?);
            }
        }
        if chains.iter().all(Option::is_none) {
            return Err(PhysicsError::InvalidModelChain(
                "no cross-section model registered".into(),
            ));
        }
        Ok(Self {
            chains,
            solvation,
            molecules_per_nm3: WATER_MOLECULES_PER_NM3,
        })
    }

    /// Water number density used to turn per-molecule into macroscopic cross sections (nm^-3).
    pub fn molecules_per_nm3(&self) -> f64 {
        self.molecules_per_nm3
    }

    /// Model chain of `process`, if the table has one.
    pub fn chain(&self, process: ProcessKind) -> Option<&ModelChain<ElectronModel>> {
        self.chains[process.index()].as_ref()
    }

    /// Solvation component (energies at or below its limit never reach the free-path draw).
    pub fn solvation(&self) -> Option<&Solvation> {
        self.solvation.as_ref()
    }

    /// Model of `process` serving energy `e`.
    pub fn select(&self, process: ProcessKind, e: Energy) -> Option<&ElectronModel> {
        self.chain(process).and_then(|c| c.select(e))
    }

    /// Energies a *primary* may start from: from the lowest to the highest window of any
    /// cross-section chain (2 eV - 1 MeV for option2; the upper end is the project's Q1 limit).
    /// Secondaries are not restricted: they are produced by validated processes and, below the
    /// lowest window, are solvated.
    pub fn primary_energy_range(&self) -> EnergyRange {
        let ranges = self.chains.iter().flatten().map(|c| c.energy_range());
        let lo = ranges
            .clone()
            .map(|r| r.min())
            .fold(None, |acc: Option<Energy>, e| match acc {
                Some(a) if a <= e => Some(a),
                _ => Some(e),
            })
            .expect("at least one chain");
        let hi = ranges
            .map(|r| r.max())
            .fold(None, |acc: Option<Energy>, e| match acc {
                Some(a) if a >= e => Some(a),
                _ => Some(e),
            })
            .expect("at least one chain");
        EnergyRange::new(lo, hi).expect("windows ordered")
    }

    /// Macroscopic cross sections at `e`: model cross section per molecule (zero when no model
    /// serves `e`) times the molecule density.
    pub fn macroscopic(&self, e: Energy) -> Result<MacroscopicCrossSections, PhysicsError> {
        let mut per_nm = [0.0; ProcessKind::COUNT];
        for p in ProcessKind::ALL {
            if let Some(model) = self.select(p, e) {
                let value = model.cross_section(e)?.nm2() * self.molecules_per_nm3;
                per_nm[p.index()] = checked_macroscopic_component(e, p, model.name(), value)?;
            }
        }
        let xs = MacroscopicCrossSections { per_nm };
        checked_macroscopic_total(e, xs.total())?;
        Ok(xs)
    }

    /// Inventory of the registered models for the output metadata.
    pub fn describe(&self) -> Vec<ModelInfo> {
        let mut out = Vec::new();
        for p in ProcessKind::ALL {
            let Some(chain) = self.chain(p) else { continue };
            for m in chain.models() {
                let w = m.energy_range();
                let (source, files): (&'static str, Vec<&'static str>) = match m {
                    ElectronModel::Elastic(_) => (
                        "dna/models/src/G4DNAChampionElasticModel.cc",
                        vec![
                            "dna/sigma_elastic_e_champion.dat",
                            "dna/sigmadiff_cumulated_elastic_e_champion_hp.dat",
                        ],
                    ),
                    ElectronModel::Excitation(_) => (
                        "dna/models/src/G4DNABornExcitationModel1.cc",
                        vec!["dna/sigma_excitation_e_born.dat"],
                    ),
                    ElectronModel::Ionisation(_) => (
                        "dna/models/src/G4DNABornIonisationModel1.cc",
                        vec![
                            "dna/sigma_ionisation_e_born.dat",
                            "dna/sigmadiff_cumulated_ionisation_e_born_hp.dat",
                        ],
                    ),
                    ElectronModel::Vibrational(_) => (
                        "dna/models/src/G4DNASancheExcitationModel.cc",
                        vec!["dna/sigma_excitationvib_e_sanche.dat"],
                    ),
                    ElectronModel::Attachment(_) => (
                        "dna/models/src/G4DNAMeltonAttachmentModel.cc",
                        vec!["dna/sigma_attachment_e_melton.dat"],
                    ),
                };
                out.push(ModelInfo {
                    process: Some(p),
                    geant4_process: p.geant4_name(),
                    model: m.name(),
                    low_ev: w.min().ev(),
                    high_ev: w.max().ev(),
                    geant4_source: source,
                    data_files: files,
                });
            }
        }
        if let Some(s) = &self.solvation {
            out.push(ModelInfo {
                process: None,
                geant4_process: "e-_G4DNAElectronSolvation",
                model: "G4DNAOneStepThermalizationModel<Meesungnoen2002>",
                low_ev: 0.0,
                high_ev: s.high_limit().ev(),
                geant4_source: "dna/models/src/G4DNAOneStepThermalizationModel.cc",
                data_files: Vec::new(),
            });
        }
        out
    }
}

fn checked_macroscopic_component(
    energy: Energy,
    process: ProcessKind,
    model: &'static str,
    value_per_nm: f64,
) -> Result<f64, PhysicsError> {
    if !value_per_nm.is_finite() || value_per_nm < 0.0 {
        return Err(PhysicsError::InvalidMacroscopicCrossSection {
            energy_ev: energy.ev(),
            process,
            model,
            value_per_nm,
        });
    }
    Ok(value_per_nm)
}

fn checked_macroscopic_total(energy: Energy, value_per_nm: f64) -> Result<(), PhysicsError> {
    if !value_per_nm.is_finite() || value_per_nm < 0.0 {
        return Err(PhysicsError::InvalidMacroscopicTotal {
            energy_ev: energy.ev(),
            value_per_nm,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use g4_core::units::Area;

    #[test]
    fn invalid_macroscopic_components_and_overflowed_total_are_errors() {
        let energy = Energy::from_ev(100.0);
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
            let err =
                checked_macroscopic_component(energy, ProcessKind::Elastic, "test model", invalid)
                    .unwrap_err();
            assert!(matches!(
                err,
                PhysicsError::InvalidMacroscopicCrossSection {
                    energy_ev: 100.0,
                    process: ProcessKind::Elastic,
                    model: "test model",
                    ..
                }
            ));
        }
        assert_eq!(
            checked_macroscopic_component(energy, ProcessKind::Elastic, "test model", 0.0),
            Ok(0.0)
        );
        let overflowed = MacroscopicCrossSections::new([f64::MAX, f64::MAX, 0.0, 0.0, 0.0]);
        assert!(matches!(
            checked_macroscopic_total(energy, overflowed.total()),
            Err(PhysicsError::InvalidMacroscopicTotal {
                energy_ev: 100.0,
                value_per_nm: f64::INFINITY,
            })
        ));
    }

    /// A model that only has a window, to test the selection rule in isolation.
    #[derive(Debug)]
    struct Window {
        name: &'static str,
        lo: f64,
        hi: f64,
    }

    impl CrossSectionModel for Window {
        fn name(&self) -> &'static str {
            self.name
        }
        fn process(&self) -> ProcessKind {
            ProcessKind::Excitation
        }
        fn energy_range(&self) -> EnergyRange {
            EnergyRange::new(Energy::from_ev(self.lo), Energy::from_ev(self.hi)).unwrap()
        }
        fn cross_section(&self, e: Energy) -> Result<Area, PhysicsError> {
            Ok(Area::from_nm2(if self.energy_range().contains(e) {
                1.0
            } else {
                0.0
            }))
        }
    }

    fn two_models() -> ModelChain<Window> {
        ModelChain::new(vec![
            Window {
                name: "low",
                lo: 10.0,
                hi: 100.0,
            },
            Window {
                name: "high",
                lo: 100.0,
                hi: 1000.0,
            },
        ])
        .unwrap()
    }

    fn pick(c: &ModelChain<Window>, e: f64) -> Option<&'static str> {
        c.select(Energy::from_ev(e)).map(|m| m.name)
    }

    fn next_up(x: f64) -> f64 {
        f64::from_bits(x.to_bits() + 1)
    }
    fn next_down(x: f64) -> f64 {
        f64::from_bits(x.to_bits() - 1)
    }

    #[test]
    fn lower_model_owns_the_shared_boundary() {
        let c = two_models();
        assert_eq!(pick(&c, 100.0), Some("low"));
        assert_eq!(pick(&c, next_down(100.0)), Some("low"));
        assert_eq!(pick(&c, next_up(100.0)), Some("high"));
    }

    #[test]
    fn first_model_owns_its_lower_bound_and_nothing_below() {
        let c = two_models();
        assert_eq!(pick(&c, 10.0), Some("low"));
        assert_eq!(pick(&c, next_up(10.0)), Some("low"));
        assert_eq!(pick(&c, next_down(10.0)), None);
    }

    #[test]
    fn last_model_owns_its_upper_bound_and_nothing_above() {
        let c = two_models();
        assert_eq!(pick(&c, 1000.0), Some("high"));
        assert_eq!(pick(&c, next_down(1000.0)), Some("high"));
        assert_eq!(pick(&c, next_up(1000.0)), None);
    }

    #[test]
    fn single_model_is_a_closed_window() {
        let c = ModelChain::new(vec![Window {
            name: "only",
            lo: 7.4,
            hi: 1.0e6,
        }])
        .unwrap();
        assert_eq!(pick(&c, 7.4), Some("only"));
        assert_eq!(pick(&c, 1.0e6), Some("only"));
        assert_eq!(pick(&c, next_down(7.4)), None);
        assert_eq!(pick(&c, next_up(1.0e6)), None);
    }

    #[test]
    fn gap_between_models_selects_nothing() {
        let c = ModelChain::new(vec![
            Window {
                name: "a",
                lo: 10.0,
                hi: 50.0,
            },
            Window {
                name: "b",
                lo: 100.0,
                hi: 200.0,
            },
        ])
        .unwrap();
        assert_eq!(pick(&c, 50.0), Some("a"), "a is closed at its upper bound");
        assert_eq!(pick(&c, next_up(50.0)), None);
        assert_eq!(pick(&c, 75.0), None);
        assert_eq!(pick(&c, 100.0), None, "b owns (100, 200] only");
        assert_eq!(pick(&c, next_up(100.0)), Some("b"));
    }

    #[test]
    fn three_touching_models_each_own_their_upper_bound() {
        let c = ModelChain::new(vec![
            Window {
                name: "a",
                lo: 1.0,
                hi: 10.0,
            },
            Window {
                name: "b",
                lo: 10.0,
                hi: 20.0,
            },
            Window {
                name: "c",
                lo: 20.0,
                hi: 30.0,
            },
        ])
        .unwrap();
        assert_eq!(pick(&c, 10.0), Some("a"));
        assert_eq!(pick(&c, 20.0), Some("b"));
        assert_eq!(pick(&c, next_up(20.0)), Some("c"));
        assert_eq!(pick(&c, 30.0), Some("c"));
    }

    #[test]
    fn overlapping_or_empty_chains_are_rejected() {
        assert!(ModelChain::<Window>::new(vec![]).is_err());
        let err = ModelChain::new(vec![
            Window {
                name: "a",
                lo: 1.0,
                hi: 10.0,
            },
            Window {
                name: "b",
                lo: 5.0,
                hi: 20.0,
            },
        ])
        .unwrap_err();
        assert!(matches!(err, PhysicsError::InvalidModelChain(_)));
    }
}
