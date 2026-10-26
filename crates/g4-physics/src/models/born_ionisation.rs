//! `G4DNABornIonisationModel1`: ionisation of water by electrons (cumulated "fast" tables).
//!
//! Geant4 reference: `dna/models/src/G4DNABornIonisationModel1.cc`.
//!
//! * Window for electrons: `[11 eV, 1 MeV]`, closed (`:126-127`, tested at `:363` and `:445`).
//! * Cross section: `dna/sigma_ionisation_e_born.dat`, five shells, value scale
//!   `(1e-22/3.343) m^2`, log-log (`:116-134`); table range 8 eV - 1 MeV. Total = sum of the
//!   five shells (`G4DNACrossSectionDataSet::FindValue`). The proton-only ICRU49 scaling
//!   (`:377-383`) is not relevant.
//! * Fast mode: option2 sets `SetDNAFast(true)` (`G4EmDNAPhysics_option2.cc:54`, read at `:72`),
//!   so final states use `dna/sigmadiff_cumulated_ionisation_e_born_hp.dat`. The slow
//!   rejection-sampling path (`:598-815`) is **not implemented** (Q4 equivalence check, docs/09).
//! * Final state (`SampleSecondaries`, `:409-594`), in this draw order:
//!   1. shell by [`random_select`] over the five partial cross sections; for `k < 19 eV` a draw of
//!      shell 2 is repeated (`do ... while`, `:464-466`; the cumulated table is zero there);
//!   2. `B` = binding energy of the shell; if `k < B` return without changing anything (`:472`);
//!   3. `u` uniform, `W = TransferedEnergy(k, shell, u) - B` (`:995-1007`), negative -> 0;
//!   4. direction of the ejected electron from [`sample_born_electron_direction`] (called even
//!      when `W = 0`); the electron is created only if `W > 0` (`:492-496`);
//!   5. **primary direction from momentum conservation**, not from an angular law:
//!      `p_f = p_in - p_delta`, `p = sqrt(E (E + 2 m_e c^2))`, normalised (`:498-516`);
//!   6. `E' = k - B - W`, local deposit `B` (`:525, 578-582`).
//! * Not simulated (deviation, docs/09 Q12): atomic de-excitation after oxygen K-shell ionisation
//!   (`:527-569`). Geant4's option2 does enable it (`G4EmDNAPhysics.cc:75-77` sets Fluo/Auger and
//!   `G4EmDNABuilder.cc:143` installs `G4UAtomicDeexcitation`), so Auger electrons/fluorescence
//!   photons of the K hole would be emitted and subtracted from the deposit. Here the whole
//!   539 eV is deposited locally and the occurrence is counted
//!   ([`Quirks::k_shell_deexcitation_skipped`](super::Quirks)).
//! * Not modelled: the stationary mode (`statCode`, default `false`).

use super::{
    CrossSectionModel, FinalState, ProcessKind, Quirks, Secondary, ensure_table_covers,
    random_select, sample_born_electron_direction, window,
};
use crate::error::PhysicsError;
use crate::sampling::uniform;
use crate::water::{ELECTRON_MASS_C2_EV, ionisation_energy};
use g4_core::units::{Area, Energy, EnergyRange};
use g4_core::vec3::Vec3;
use g4_data::blocks::IonisationCdfTable;
use g4_data::catalog::{DnaFile, Table};
use g4_data::datadir::DataDir;
use g4_data::xs::CrossSectionTable;
use rand::Rng;

/// Model window in eV for electrons (`G4DNABornIonisationModel1.cc:126-127`).
pub const BORN_IONISATION_WINDOW_EV: (f64, f64) = (11.0, 1.0e6);

/// Below this energy a draw of shell 2 is repeated (`:466`, `19*eV`).
pub const SHELL2_REJECTION_BELOW_EV: f64 = 19.0;

/// Index of the oxygen K shell (binding 539 eV).
pub const K_SHELL_INDEX: usize = 4;

#[derive(Debug, Clone)]
pub struct BornIonisation {
    table: CrossSectionTable,
    cdf: IonisationCdfTable,
    window: EnergyRange,
}

impl BornIonisation {
    pub fn new(table: CrossSectionTable, cdf: IonisationCdfTable) -> Result<Self, PhysicsError> {
        // Five shells in G4DNAWaterIonisationStructure.cc (Geant4 11.4.1).
        table.require_components(
            DnaFile::BornIonisationE.relative_path(),
            DnaFile::BornIonisationE.n_components(),
        )?;
        let window = window(BORN_IONISATION_WINDOW_EV.0, BORN_IONISATION_WINDOW_EV.1);
        ensure_table_covers(
            "sigma_ionisation_e_born",
            "G4DNABornIonisationModel1",
            table.energy_range(),
            window,
        )?;
        ensure_table_covers(
            "sigmadiff_cumulated_ionisation_e_born_hp",
            "G4DNABornIonisationModel1",
            cdf.energy_range(),
            window,
        )?;
        Ok(Self { table, cdf, window })
    }

    pub fn from_data_dir(dir: &DataDir) -> Result<Self, PhysicsError> {
        let table = match dir.load(DnaFile::BornIonisationE)? {
            Table::CrossSection(t) => t,
            _ => unreachable!("catalog: BornIonisationE is a cross-section table"),
        };
        let cdf = match dir.load(DnaFile::BornIonisationCdfE)? {
            Table::IonisationCdf(t) => t,
            _ => unreachable!("catalog: BornIonisationCdfE is an ionisation CDF table"),
        };
        Self::new(table, cdf)
    }

    /// Partial cross sections of the five shells at `energy` (nm^2).
    pub fn partial_cross_sections(&self, energy: Energy) -> Result<[f64; 5], PhysicsError> {
        let mut v = [0.0; 5];
        for (shell, slot) in v.iter_mut().enumerate() {
            *slot = self.table.component(shell, energy)?.nm2();
        }
        Ok(v)
    }

    /// Energy transfer `W` (including binding) for a uniform variate `u` (`TransferedEnergy`).
    pub fn transferred_energy(
        &self,
        shell: usize,
        energy: Energy,
        u: f64,
    ) -> Result<Energy, PhysicsError> {
        Ok(Energy::from_ev(
            self.cdf.transferred_energy_ev(shell, energy, u)?,
        ))
    }

    /// Final state; see the module documentation for the draw order.
    pub fn sample<R: Rng + ?Sized>(
        &self,
        energy: Energy,
        direction: Vec3,
        rng: &mut R,
    ) -> Result<FinalState, PhysicsError> {
        self.window.check(energy)?;
        let k = energy;
        let mut quirks = Quirks::default();

        let partials = self.partial_cross_sections(k)?;
        let shell = loop {
            let shell = random_select(&partials, uniform(rng));
            if k.ev() < SHELL2_REJECTION_BELOW_EV && shell == 2 {
                quirks.shell2_redraws += 1;
                continue;
            }
            break shell;
        };

        let binding = ionisation_energy(shell);
        if k < binding {
            quirks.ionisation_noop_below_binding = true;
            return Ok(FinalState {
                energy_after: k,
                direction_after: direction,
                local_deposit: Energy::from_ev(0.0),
                secondary: None,
                channel: shell,
                quirks,
                killed: false,
            });
        }

        let transfer = self.transferred_energy(shell, k, uniform(rng))?;
        let mut w = transfer - binding;
        if w.ev() < 0.0 {
            quirks.transfer_clamped_to_zero = true;
            w = Energy::from_ev(0.0);
        }

        let delta_direction = sample_born_electron_direction(k, w, direction, rng);
        let secondary = (w.ev() > 0.0).then_some(Secondary {
            energy: w,
            direction: delta_direction,
        });

        // Momentum balance of the primary (`:498-513`), relativistic momenta in eV/c.
        let mc2 = ELECTRON_MASS_C2_EV;
        let p_primary = (k.ev() * (k.ev() + 2.0 * mc2)).sqrt();
        let p_delta = (w.ev() * (w.ev() + 2.0 * mc2)).sqrt();
        let p_final = direction * p_primary - delta_direction * p_delta;
        let direction_after = p_final
            .normalised()
            .ok_or(PhysicsError::NonPhysicalFinalState {
                model: "G4DNABornIonisationModel1",
                what: "zero final momentum",
            })?;

        quirks.k_shell_deexcitation_skipped = shell == K_SHELL_INDEX;
        Ok(FinalState {
            energy_after: k - binding - w,
            direction_after,
            local_deposit: binding,
            secondary,
            channel: shell,
            quirks,
            killed: false,
        })
    }
}

impl CrossSectionModel for BornIonisation {
    fn name(&self) -> &'static str {
        "G4DNABornIonisationModel1"
    }
    fn process(&self) -> ProcessKind {
        ProcessKind::Ionisation
    }
    fn energy_range(&self) -> EnergyRange {
        self.window
    }
    fn cross_section(&self, energy: Energy) -> Result<Area, PhysicsError> {
        if !self.window.contains(energy) {
            return Ok(Area::from_nm2(0.0));
        }
        Ok(self.table.total(energy)?)
    }
}
