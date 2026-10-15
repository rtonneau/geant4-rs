//! `G4DNAChampionElasticModel`: elastic scattering of electrons in liquid water.
//!
//! Geant4 reference: `source/processes/electromagnetic/dna/models/src/G4DNAChampionElasticModel.cc`
//! (Geant4 11.4.1, tree commit `27ea526b`).
//!
//! * Window: `[7.4 eV, 1 MeV]`, closed (`:46-47`, `:253`). The option2 builder sets the same limits
//!   (`G4EmDNABuilder.cc:349-350`: low = `emaxT` = 7.4 eV, high = `lowEnergyMSC` = 1 MeV).
//! * Total cross section: `dna/sigma_elastic_e_champion.dat`, value scale `1e-16 cm^2`, log-log
//!   (`:129-136`); table range 7.4 eV - 1e7 eV. Cross section per volume = per molecule x water
//!   number density (`:251, 272`).
//! * Final state (`:277-312`): `cos(theta)` from the cumulated differential table
//!   `dna/sigmadiff_cumulated_elastic_e_champion_hp.dat`: `u` uniform, `theta[deg] = Theta(k, u)`
//!   (2-D lin-lin `QuadInterpolator`, `:316-363, 411-444`), `cos(theta*pi/180)` (`:448-462`); `phi =
//!   2 pi u'`; energy unchanged; **no local energy deposit** (`:310`).
//!
//! # Geant4 quirk: non-unit rotation axis (see [`ElasticRotation`])
//! `SampleSecondaries` builds `xVers = zVers.orthogonal()` and `yVers = zVers.cross(xVers)` and never
//! normalises them (`:297-299`). CLHEP's `orthogonal()` returns a vector of norm
//! `sqrt(1 - c^2)`, `c` being the smallest-magnitude component of `zVers`, i.e. between
//! `sqrt(2/3)` and 1. The new direction `a sin(theta)(cos(phi) x + sin(phi) y) + cos(theta) z`,
//! then normalised (`:306-308`), is therefore deflected by a smaller angle than the tabulated
//! `theta` whenever the incident direction is not along a coordinate axis:
//! `cos(theta_eff) = cos(theta) / sqrt(cos^2(theta) + a^2 sin^2(theta))`, `a^2 = 1 - c^2`.
//! This is reproduced by default ([`ElasticRotation::Geant4`]) because the project reproduces
//! Geant4 for comparison; [`ElasticRotation::Exact`] applies the tabulated angle exactly. The
//! effect is documented in docs/09 (Q13) and is a deviation to be decided by the project lead.

use super::{CrossSectionModel, FinalState, ProcessKind, Quirks, ensure_table_covers, window};
use crate::direction::clhep_orthogonal;
use crate::error::PhysicsError;
use crate::sampling::uniform;
use dna_core::units::{Area, Energy, EnergyRange};
use dna_core::vec3::Vec3;
use dna_data::blocks::AngularCdfTable;
use dna_data::catalog::{DnaFile, Table};
use dna_data::datadir::DataDir;
use dna_data::xs::CrossSectionTable;
use rand::Rng;
use std::f64::consts::PI;

/// Model window in eV (`G4DNAChampionElasticModel.cc:46-47`).
pub const CHAMPION_WINDOW_EV: (f64, f64) = (7.4, 1.0e6);

/// How the sampled polar angle is applied to the incident direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ElasticRotation {
    /// Geant4 11.4.1 behaviour: rotation axes from CLHEP `orthogonal()` without normalisation
    /// (direction-dependent shrinkage of the scattering angle). Default.
    #[default]
    Geant4,
    /// Orthonormal axes: the deflection is exactly the tabulated polar angle.
    Exact,
}

/// Champion elastic model: cross-section table plus cumulated angular table.
#[derive(Debug, Clone)]
pub struct ChampionElastic {
    sigma: CrossSectionTable,
    angles: AngularCdfTable,
    window: EnergyRange,
    rotation: ElasticRotation,
}

impl ChampionElastic {
    /// Build from loaded tables; both must cover the model window.
    pub fn new(
        sigma: CrossSectionTable,
        angles: AngularCdfTable,
        rotation: ElasticRotation,
    ) -> Result<Self, PhysicsError> {
        let window = window(CHAMPION_WINDOW_EV.0, CHAMPION_WINDOW_EV.1);
        ensure_table_covers(
            "sigma_elastic_e_champion",
            "G4DNAChampionElasticModel",
            sigma.energy_range(),
            window,
        )?;
        ensure_table_covers(
            "sigmadiff_cumulated_elastic_e_champion_hp",
            "G4DNAChampionElasticModel",
            angles.energy_range(),
            window,
        )?;
        Ok(Self {
            sigma,
            angles,
            window,
            rotation,
        })
    }

    /// Load both tables from `dir`.
    pub fn from_data_dir(dir: &DataDir, rotation: ElasticRotation) -> Result<Self, PhysicsError> {
        let sigma = match dir.load(DnaFile::ChampionElasticE)? {
            Table::CrossSection(t) => t,
            _ => unreachable!("catalog: ChampionElasticE is a cross-section table"),
        };
        let angles = match dir.load(DnaFile::ChampionElasticCdfE)? {
            Table::AngularCdf(t) => t,
            _ => unreachable!("catalog: ChampionElasticCdfE is an angular table"),
        };
        Self::new(sigma, angles, rotation)
    }

    pub fn rotation(&self) -> ElasticRotation {
        self.rotation
    }

    /// Polar-angle cosine for cumulative probability `p` (`RandomizeCosTheta` with the uniform
    /// variate supplied, `:448-462`).
    pub fn cos_theta_for(&self, energy: Energy, p: f64) -> Result<f64, PhysicsError> {
        let theta_deg = self.angles.angle_deg(energy, p)?;
        Ok((theta_deg * PI / 180.0).cos())
    }

    /// Sample the final state. Draw order as Geant4: one uniform for `cos(theta)`, one for `phi`.
    pub fn sample<R: Rng + ?Sized>(
        &self,
        energy: Energy,
        direction: Vec3,
        rng: &mut R,
    ) -> Result<FinalState, PhysicsError> {
        self.window.check(energy)?;
        let cos_theta = self.cos_theta_for(energy, uniform(rng))?;
        let phi = 2.0 * PI * uniform(rng);
        let direction_after = scatter(direction, cos_theta, phi, self.rotation);
        if !(direction_after.norm().is_finite()) {
            return Err(PhysicsError::NonPhysicalFinalState {
                model: "G4DNAChampionElasticModel",
                what: "direction",
            });
        }
        Ok(FinalState {
            energy_after: energy,
            direction_after,
            local_deposit: Energy::from_ev(0.0),
            secondary: None,
            channel: 0,
            quirks: Quirks::default(),
            killed: false,
        })
    }
}

/// New direction after a deflection `(cos_theta, phi)` of the unit direction `z`
/// (`G4DNAChampionElasticModel.cc:297-308`).
pub fn scatter(z: Vec3, cos_theta: f64, phi: f64, rotation: ElasticRotation) -> Vec3 {
    let mut x_axis = clhep_orthogonal(z);
    if rotation == ElasticRotation::Exact {
        x_axis = x_axis
            .normalised()
            .expect("orthogonal() of a unit vector is non-zero");
    }
    let y_axis = z.cross(x_axis);
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let x_dir = sin_theta * phi.cos();
    let y_dir = sin_theta * phi.sin();
    let v = x_axis * x_dir + y_axis * y_dir + z * cos_theta;
    v.normalised().unwrap_or(v)
}

impl CrossSectionModel for ChampionElastic {
    fn name(&self) -> &'static str {
        "G4DNAChampionElasticModel"
    }
    fn process(&self) -> ProcessKind {
        ProcessKind::Elastic
    }
    fn energy_range(&self) -> EnergyRange {
        self.window
    }
    fn cross_section(&self, energy: Energy) -> Result<Area, PhysicsError> {
        if !self.window.contains(energy) {
            return Ok(Area::from_nm2(0.0));
        }
        Ok(self.sigma.total(energy)?)
    }
}
