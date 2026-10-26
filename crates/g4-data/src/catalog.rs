//! The 20 committed `G4EMLOW 8.8` `dna/` files: layout, units and the C++ code that reads them.
//!
//! Every unit and scale factor below is transcribed from the Geant4 11.4.1 loader named in the
//! `source` strings (paths relative to `source/processes/electromagnetic/dna/models/src`); none is
//! guessed or derived. Files carry no unit information themselves: energies are eV in all files,
//! and the value scale is the `scaleFactor` the loading model passes to
//! `G4DNACrossSectionDataSet` (or applies itself).
//!
//! Rows of `windows` give the validity window of the model that consumes the file (closed
//! interval, eV). A test checks that the table covers the window, except for the documented
//! Melton case.

use crate::blocks::{AngularCdfTable, IonisationCdfTable, IonisationDcsTable};
use crate::error::DataError;
use crate::xs::{CrossSectionTable, Interpolation, XsSpec};
use g4_core::units::{Area, EnergyRange};
use std::path::Path;

/// `(1e-22 / 3.343) m^2`: scale of the Born tables (`G4DNABornExcitationModel1.cc:112`,
/// `G4DNABornIonisationModel1.cc:116`). The constant 3.343 is Geant4's own.
fn born_scale() -> Area {
    Area::from_m2(1.0e-22 / 3.343)
}

/// A model that reads a file, with its validity window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Consumer {
    /// Geant4 class, particle in parentheses where the file is shared.
    pub model: &'static str,
    /// Loader location (file:line) of the file name and scale factor.
    pub source: &'static str,
    /// `[low, high]` eV in which the model queries the table, if it has a fixed window.
    pub window_ev: Option<(f64, f64)>,
}

/// File layout and how to read it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    /// `E [eV]`, then one cross-section column per level/shell.
    CrossSection(XsSpec),
    /// `E [eV]`, cumulative probability, polar angle [deg].
    AngularCdf,
    /// `E [eV]`, energy transfer `W` [eV], 5 x cumulative probability.
    IonisationCdf,
    /// `E [eV]`, energy transfer `W` [eV], 5 x `d sigma/dW` (raw numbers).
    IonisationDcs,
}

/// A loaded table of any of the four layouts.
#[derive(Debug, Clone)]
pub enum Table {
    CrossSection(CrossSectionTable),
    AngularCdf(AngularCdfTable),
    IonisationCdf(IonisationCdfTable),
    IonisationDcs(IonisationDcsTable),
}

impl Table {
    /// Incident-energy interval covered by the table.
    pub fn energy_range(&self) -> EnergyRange {
        match self {
            Table::CrossSection(t) => t.energy_range(),
            Table::AngularCdf(t) => t.energy_range(),
            Table::IonisationCdf(t) => t.energy_range(),
            Table::IonisationDcs(t) => t.energy_range(),
        }
    }
}

macro_rules! files {
    ($($variant:ident => $path:literal,)+) => {
        /// One of the 20 files listed in `data/MANIFEST.toml`.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum DnaFile { $($variant,)+ }
        impl DnaFile {
            /// All files, in manifest order of groups.
            pub const ALL: &'static [DnaFile] = &[$(DnaFile::$variant,)+];
            /// Path relative to the data root, as in the manifest.
            pub fn relative_path(self) -> &'static str {
                match self { $(DnaFile::$variant => $path,)+ }
            }
        }
    };
}

files! {
    MeltonAttachmentE => "dna/sigma_attachment_e_melton.dat",
    ChampionElasticE => "dna/sigma_elastic_e_champion.dat",
    HtranElasticProton => "dna/sigma_elastic_proton_HTran.dat",
    BornExcitationE => "dna/sigma_excitation_e_born.dat",
    RpwbaExcitationP => "dna/sigma_excitation_p_RPWBA.dat",
    BornExcitationP => "dna/sigma_excitation_p_born.dat",
    SancheVibrationalE => "dna/sigma_excitationvib_e_sanche.dat",
    BornIonisationE => "dna/sigma_ionisation_e_born.dat",
    RuddIonisationH => "dna/sigma_ionisation_h_rudd.dat",
    RpwbaIonisationP => "dna/sigma_ionisation_p_RPWBA.dat",
    BornIonisationP => "dna/sigma_ionisation_p_born.dat",
    RuddIonisationP => "dna/sigma_ionisation_p_rudd.dat",
    ChampionElasticCdfE => "dna/sigmadiff_cumulated_elastic_e_champion_hp.dat",
    HtranElasticCdfProton => "dna/sigmadiff_cumulated_elastic_proton_HTran.dat",
    BornIonisationCdfE => "dna/sigmadiff_cumulated_ionisation_e_born_hp.dat",
    RpwbaIonisationCdfP => "dna/sigmadiff_cumulated_ionisation_p_RPWBA.dat",
    BornIonisationCdfP => "dna/sigmadiff_cumulated_ionisation_p_born_hp.dat",
    BornIonisationDcsE => "dna/sigmadiff_ionisation_e_born.dat",
    RpwbaIonisationDcsP => "dna/sigmadiff_ionisation_p_RPWBA.dat",
    BornIonisationDcsP => "dna/sigmadiff_ionisation_p_born.dat",
}

fn log_log(scale: Area) -> Format {
    Format::CrossSection(XsSpec {
        scale,
        interpolation: Interpolation::LogLog,
    })
}

impl DnaFile {
    /// Layout, value scale and interpolation rule.
    pub fn format(self) -> Format {
        use DnaFile::*;
        match self {
            // G4DNAMeltonAttachmentModel.cc:141 `1e-18*cm2`
            MeltonAttachmentE => log_log(Area::from_cm2(1e-18)),
            // G4DNAChampionElasticModel.cc:129 `1e-16*cm*cm`; G4DNAIonElasticModel.cc:121
            ChampionElasticE | HtranElasticProton => log_log(Area::from_cm2(1e-16)),
            // G4DNABornExcitationModel1.cc:112; G4DNABornIonisationModel1.cc:116
            BornExcitationE | BornExcitationP | BornIonisationE | BornIonisationP => {
                log_log(born_scale())
            }
            // G4DNARPWBAExcitationModel.cc:101, G4DNARPWBAIonisationModel.cc:99 `1*cm*cm`
            RpwbaExcitationP | RpwbaIonisationP => log_log(Area::from_cm2(1.0)),
            // G4DNARuddIonisationExtendedModel.cc:61 `CLHEP::m*CLHEP::m`
            RuddIonisationH | RuddIonisationP => log_log(Area::from_m2(1.0)),
            // G4DNASancheExcitationModel.cc:309 `conv_factor = 1e-16*cm*cm`, own lin-lin reader
            SancheVibrationalE => Format::CrossSection(XsSpec {
                scale: Area::from_cm2(1e-16),
                interpolation: Interpolation::LinLin,
            }),
            ChampionElasticCdfE | HtranElasticCdfProton => Format::AngularCdf,
            BornIonisationCdfE | BornIonisationCdfP | RpwbaIonisationCdfP => Format::IonisationCdf,
            BornIonisationDcsE | BornIonisationDcsP | RpwbaIonisationDcsP => Format::IonisationDcs,
        }
    }

    /// Number of value columns after the energy column(s): levels/shells for cross sections,
    /// 1 angle column, or 5 shells for the ionisation tables.
    pub fn n_components(self) -> usize {
        use DnaFile::*;
        match self {
            MeltonAttachmentE
            | ChampionElasticE
            | HtranElasticProton
            | ChampionElasticCdfE
            | HtranElasticCdfProton => 1,
            SancheVibrationalE => 9,
            _ => 5,
        }
    }

    /// Column meaning, transcribed from the loaders (docs/10 holds the full table).
    pub fn columns(self) -> &'static str {
        use DnaFile::*;
        match self {
            MeltonAttachmentE | ChampionElasticE | HtranElasticProton => {
                "E [eV]; total cross section per molecule"
            }
            BornExcitationE | RpwbaExcitationP | BornExcitationP => {
                "E [eV]; cross section for each of the 5 excitation levels (A1B1, B1A1, Ryd A+B, Ryd C+D, diffuse bands; 8.22, 10.00, 11.24, 12.61, 13.77 eV)"
            }
            BornIonisationE | RuddIonisationH | RpwbaIonisationP | BornIonisationP
            | RuddIonisationP => {
                "E [eV]; cross section for each of the 5 ionisation shells (binding 10.79, 13.39, 16.05, 32.30, 539.0 eV)"
            }
            SancheVibrationalE => {
                "E [eV]; cross section for each of the 9 vibrational levels (0.010, 0.024, 0.061, 0.092, 0.204, 0.417, 0.460, 0.500, 0.835 eV)"
            }
            ChampionElasticCdfE | HtranElasticCdfProton => {
                "E [eV]; cumulative probability P in [0, 1]; polar angle theta [deg]"
            }
            BornIonisationCdfE | BornIonisationCdfP | RpwbaIonisationCdfP => {
                "E [eV]; energy transfer W [eV] (binding energy included); cumulative probability per shell (5 columns)"
            }
            BornIonisationDcsE | BornIonisationDcsP | RpwbaIonisationDcsP => {
                "E [eV]; energy transfer W [eV]; d sigma/dW per shell (5 columns, raw numbers)"
            }
        }
    }

    /// Models that read the file and their windows.
    pub fn consumers(self) -> &'static [Consumer] {
        use DnaFile::*;
        macro_rules! c {
            ($model:expr, $source:expr, $window:expr $(,)?) => {
                Consumer {
                    model: $model,
                    source: $source,
                    window_ev: $window,
                }
            };
        }
        match self {
            MeltonAttachmentE => &[c!(
                "G4DNAMeltonAttachmentModel (e-)",
                "G4DNAMeltonAttachmentModel.cc:141-148 (window :49-50)",
                Some((4.0, 13.0)),
            )],
            ChampionElasticE => &[c!(
                "G4DNAChampionElasticModel (e-)",
                "G4DNAChampionElasticModel.cc:129-136 (window :46-47)",
                Some((7.4, 1.0e6)),
            )],
            HtranElasticProton => &[c!(
                "G4DNAIonElasticModel (p, H)",
                "G4DNAIonElasticModel.cc:121,153,176 (window 0-1 MeV :51-52; DBL_MAX below 100 eV :267)",
                Some((100.0, 1.0e6)),
            )],
            BornExcitationE => &[c!(
                "G4DNABornExcitationModel1 (e-)",
                "G4DNABornExcitationModel1.cc:98-114 (window 9 eV-1 MeV :99-100)",
                Some((9.0, 1.0e6)),
            )],
            BornExcitationP => &[c!(
                "G4DNABornExcitationModel1 (proton)",
                "G4DNABornExcitationModel1.cc:104-114 (window 500 keV-100 MeV :105-106)",
                Some((5.0e5, 1.0e8)),
            )],
            RpwbaExcitationP => &[c!(
                "G4DNARPWBAExcitationModel (proton)",
                "G4DNARPWBAExcitationModel.cc:89-104 (window 100-300 MeV :90-91)",
                Some((1.0e8, 3.0e8)),
            )],
            SancheVibrationalE => &[c!(
                "G4DNASancheExcitationModel (e-)",
                "G4DNASancheExcitationModel.cc:139-178 reader, :309 scale, :407 lin-lin (window :48-49)",
                Some((2.0, 100.0)),
            )],
            BornIonisationE => &[c!(
                "G4DNABornIonisationModel1 (e-)",
                "G4DNABornIonisationModel1.cc:107,116,132 (window 11 eV-1 MeV :126-127)",
                Some((11.0, 1.0e6)),
            )],
            BornIonisationP => &[c!(
                "G4DNABornIonisationModel1 (proton)",
                "G4DNABornIonisationModel1.cc:108,116,218 (window 500 keV-100 MeV :212-213)",
                Some((5.0e5, 1.0e8)),
            )],
            RpwbaIonisationP => &[c!(
                "G4DNARPWBAIonisationModel (proton)",
                "G4DNARPWBAIonisationModel.cc:98-114 (window 100-300 MeV :101-102)",
                Some((1.0e8, 3.0e8)),
            )],
            RuddIonisationP => &[c!(
                "G4DNARuddIonisationExtendedModel (proton)",
                "G4DNARuddIonisationExtendedModel.cc:61,109-111 (lowest 100 eV :78; upper 500 keV from the builder, docs/02 B.3)",
                Some((100.0, 5.0e5)),
            )],
            RuddIonisationH => &[c!(
                "G4DNARuddIonisationExtendedModel (hydrogen)",
                "G4DNARuddIonisationExtendedModel.cc:61,105-107 (lowest 100 eV :78)",
                Some((100.0, 5.0e5)),
            )],
            ChampionElasticCdfE => &[c!(
                "G4DNAChampionElasticModel (e-), Theta()",
                "G4DNAChampionElasticModel.cc:150-195 reader, :316-378 Theta, :448-462",
                Some((7.4, 1.0e6)),
            )],
            HtranElasticCdfProton => &[c!(
                "G4DNAIonElasticModel (p, H), Theta() (centre-of-mass angle)",
                "G4DNAIonElasticModel.cc:156-212 reader, :369-424 Theta, :320-330 CM -> lab",
                Some((100.0, 1.0e6)),
            )],
            BornIonisationCdfE => &[c!(
                "G4DNABornIonisationModel1 (e-), fasterCode = true",
                "G4DNABornIonisationModel1.cc:139-205 reader, :1012-1290 TransferedEnergy",
                Some((11.0, 1.0e6)),
            )],
            BornIonisationCdfP => &[c!(
                "G4DNABornIonisationModel1 (proton), fasterCode = true",
                "G4DNABornIonisationModel1.cc:226-262 reader, :1146-1290 TransferedEnergy",
                Some((5.0e5, 1.0e8)),
            )],
            RpwbaIonisationCdfP => &[c!(
                "G4DNARPWBAIonisationModel (proton), fasterCode = true",
                "G4DNARPWBAIonisationModel.cc:119-165 reader (own TransferedEnergy not yet audited)",
                Some((1.0e8, 3.0e8)),
            )],
            BornIonisationDcsE => &[c!(
                "G4DNABornIonisationModel1 (e-), fasterCode = false",
                "G4DNABornIonisationModel1.cc:141,178-205 reader (x scale :199), :598-815 rejection sampling",
                Some((11.0, 1.0e6)),
            )],
            BornIonisationDcsP => &[c!(
                "G4DNABornIonisationModel1 (proton), fasterCode = false",
                "G4DNABornIonisationModel1.cc:228,242-262 reader (x scale :259), :598-815",
                Some((5.0e5, 1.0e8)),
            )],
            RpwbaIonisationDcsP => &[c!(
                "G4DNARPWBAIonisationModel (proton), fasterCode = false",
                "G4DNARPWBAIonisationModel.cc:121,132-165 reader (x cm^2 :157)",
                Some((1.0e8, 3.0e8)),
            )],
        }
    }

    /// Read and build the table from `path` (the file for `self`, wherever it lives).
    pub fn load_from(self, path: &Path) -> Result<Table, DataError> {
        Ok(match self.format() {
            Format::CrossSection(spec) => Table::CrossSection(CrossSectionTable::load(path, spec)?),
            Format::AngularCdf => Table::AngularCdf(AngularCdfTable::load(path)?),
            Format::IonisationCdf => Table::IonisationCdf(IonisationCdfTable::load(path)?),
            Format::IonisationDcs => Table::IonisationDcs(IonisationDcsTable::load(path)?),
        })
    }
}
