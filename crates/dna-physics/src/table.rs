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
    ElectronModel, ProcessKind,
};
use crate::water::WATER_MOLECULES_PER_NM3;
use dna_core::units::{Energy, EnergyRange};
use dna_data::datadir::DataDir;
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
    pub process: ProcessKind,
    pub geant4_process: &'static str,
    pub model: &'static str,
    pub low_ev: f64,
    pub high_ev: f64,
    pub geant4_source: &'static str,
    pub data_files: Vec<&'static str>,
}

/// Macroscopic cross sections (1/nm) of the three processes at one energy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacroscopicCrossSections {
    per_nm: [f64; 3],
}

impl MacroscopicCrossSections {
    /// From per-process values in [`ProcessKind::ALL`] order (1/nm).
    pub const fn new(per_nm: [f64; 3]) -> Self {
        Self { per_nm }
    }
    pub fn get(&self, process: ProcessKind) -> f64 {
        self.per_nm[process.index()]
    }
    pub fn total(&self) -> f64 {
        self.per_nm.iter().sum()
    }
    /// Sum of the processes that change the energy (excitation + ionisation).
    pub fn inelastic(&self) -> f64 {
        self.per_nm[1] + self.per_nm[2]
    }
    pub fn as_array(&self) -> [f64; 3] {
        self.per_nm
    }
}

/// The processes of one particle in one material.
#[derive(Debug, Clone)]
pub struct ProcessTable {
    chains: [ModelChain<ElectronModel>; 3],
    molecules_per_nm3: f64,
}

impl ProcessTable {
    /// `G4EmDNAPhysics_option2` electron processes available in M2: Champion elastic, Born
    /// excitation, Born1 ionisation, in liquid water of Geant4 `G4_WATER` density.
    ///
    /// Not included (M3): solvation below 7.4 eV, Sanche vibrational excitation (2-100 eV),
    /// Melton attachment (4-13 eV).
    pub fn option2_electron(
        dir: &DataDir,
        rotation: ElasticRotation,
    ) -> Result<Self, PhysicsError> {
        Self::from_models(
            vec![ElectronModel::Elastic(ChampionElastic::from_data_dir(
                dir, rotation,
            )?)],
            vec![ElectronModel::Excitation(BornExcitation::from_data_dir(
                dir,
            )?)],
            vec![ElectronModel::Ionisation(BornIonisation::from_data_dir(
                dir,
            )?)],
        )
    }

    /// Build from explicit chains (each non-empty, models matching the process).
    pub fn from_models(
        elastic: Vec<ElectronModel>,
        excitation: Vec<ElectronModel>,
        ionisation: Vec<ElectronModel>,
    ) -> Result<Self, PhysicsError> {
        for (models, kind) in [
            (&elastic, ProcessKind::Elastic),
            (&excitation, ProcessKind::Excitation),
            (&ionisation, ProcessKind::Ionisation),
        ] {
            if let Some(m) = models.iter().find(|m| m.process() != kind) {
                return Err(PhysicsError::InvalidModelChain(format!(
                    "{} registered under {:?}",
                    m.name(),
                    kind
                )));
            }
        }
        Ok(Self {
            chains: [
                ModelChain::new(elastic)?,
                ModelChain::new(excitation)?,
                ModelChain::new(ionisation)?,
            ],
            molecules_per_nm3: WATER_MOLECULES_PER_NM3,
        })
    }

    /// Water number density used to turn per-molecule into macroscopic cross sections (nm^-3).
    pub fn molecules_per_nm3(&self) -> f64 {
        self.molecules_per_nm3
    }

    pub fn chain(&self, process: ProcessKind) -> &ModelChain<ElectronModel> {
        &self.chains[process.index()]
    }

    /// Model of `process` serving energy `e`.
    pub fn select(&self, process: ProcessKind, e: Energy) -> Option<&ElectronModel> {
        self.chain(process).select(e)
    }

    /// Energies a primary may start from: from the lowest to the highest model window of any
    /// process (7.4 eV - 1 MeV for option2; the upper end is the project's Q1 limit).
    pub fn primary_energy_range(&self) -> EnergyRange {
        let lo = self
            .chains
            .iter()
            .map(|c| c.energy_range().min())
            .fold(None, |acc: Option<Energy>, e| match acc {
                Some(a) if a <= e => Some(a),
                _ => Some(e),
            })
            .expect("three chains");
        let hi = self
            .chains
            .iter()
            .map(|c| c.energy_range().max())
            .fold(None, |acc: Option<Energy>, e| match acc {
                Some(a) if a >= e => Some(a),
                _ => Some(e),
            })
            .expect("three chains");
        EnergyRange::new(lo, hi).expect("windows ordered")
    }

    /// Macroscopic cross sections at `e`: model cross section per molecule (zero when no model
    /// serves `e`) times the molecule density.
    pub fn macroscopic(&self, e: Energy) -> Result<MacroscopicCrossSections, PhysicsError> {
        let mut per_nm = [0.0; 3];
        for p in ProcessKind::ALL {
            if let Some(model) = self.select(p, e) {
                per_nm[p.index()] = model.cross_section(e)?.nm2() * self.molecules_per_nm3;
            }
        }
        Ok(MacroscopicCrossSections { per_nm })
    }

    /// Inventory of the registered models for the output metadata.
    pub fn describe(&self) -> Vec<ModelInfo> {
        let mut out = Vec::new();
        for p in ProcessKind::ALL {
            for m in self.chain(p).models() {
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
                };
                out.push(ModelInfo {
                    process: p,
                    geant4_process: p.geant4_name(),
                    model: m.name(),
                    low_ev: w.min().ev(),
                    high_ev: w.max().ev(),
                    geant4_source: source,
                    data_files: files,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dna_core::units::Area;

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
