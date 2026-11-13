//! Run configuration.

use crate::error::SimError;
use g4_core::geometry::Volume;
use g4_core::units::Energy;
use g4_core::vec3::Vec3;
use g4_physics::transport::{Primary, TransportConfig};
use serde::{Deserialize, Serialize};

/// Parameters of a run of `n_histories` identical primaries.
///
/// Energies are eV, positions nm (field names carry the unit, docs/units.md). History `i`
/// (`first_history_id <= i < first_history_id + n_histories`) uses the stream
/// `(master_seed, i, Stage::Physics)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunConfig {
    pub master_seed: u64,
    pub first_history_id: u64,
    pub n_histories: u64,
    pub primary_energy_ev: f64,
    /// Primary direction (normalised by the transport).
    pub direction: [f64; 3],
    /// Stop the primary below this energy (e.g. to compare with a Geant4 run with a tracking cut).
    pub tracking_cut_ev: Option<f64>,
    /// Stop each history after this many interactions (`Some(1)` reproduces the Geant4 `mfp`
    /// example, which kills the event after the first interaction).
    pub max_interactions: Option<u64>,
    /// Follow the ionisation secondaries (default `true`, M3). `false` records them without
    /// transporting them (M2 behaviour; what the Geant4 `track` reference does).
    pub transport_secondaries: bool,
    /// Keep the physicochemical events of every history in memory ([`HistoryResult::physchem`]);
    /// off by default (a 1 MeV history produces about 1e5 events).
    ///
    /// [`HistoryResult::physchem`]: crate::history::HistoryResult
    pub record_physchem: bool,
    /// Histories reduced together (memory bound; does not affect results, docs/05 E.3).
    pub batch_size: usize,
    /// Worker threads; `None` uses the global rayon pool. Does not affect results.
    pub threads: Option<usize>,
    /// Optional primitive volume (nm, rad; `g4_core::geometry`, docs/04 D.5). Not used by the
    /// transport yet, which still runs in infinite liquid water, so [`RunConfig::validate`]
    /// rejects `Some(_)` rather than ignore it silently. Omitted from serialised output when
    /// `None`, so existing run metadata is unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<Volume>,
}

impl RunConfig {
    /// Defaults: seed 1, histories 0..n along +z, no tracking cut, batches of 1024.
    pub fn new(primary_energy_ev: f64, n_histories: u64) -> Self {
        Self {
            master_seed: 1,
            first_history_id: 0,
            n_histories,
            primary_energy_ev,
            direction: [0.0, 0.0, 1.0],
            tracking_cut_ev: None,
            max_interactions: None,
            transport_secondaries: true,
            record_physchem: false,
            batch_size: 1024,
            threads: None,
            geometry: None,
        }
    }

    pub fn validate(&self) -> Result<(), SimError> {
        let bad = |m: &str| Err(SimError::InvalidConfig(m.to_owned()));
        if !(self.primary_energy_ev.is_finite() && self.primary_energy_ev > 0.0) {
            return bad("primary_energy_ev must be a positive finite number");
        }
        if self.n_histories == 0 {
            return bad("n_histories must be at least 1");
        }
        if self.batch_size == 0 {
            return bad("batch_size must be at least 1");
        }
        if self.threads == Some(0) {
            return bad("threads must be at least 1");
        }
        if self.max_interactions == Some(0) {
            return bad("max_interactions must be at least 1 when set");
        }
        if let Some(cut) = self.tracking_cut_ev
            && !(cut.is_finite() && cut >= 0.0)
        {
            return bad("tracking_cut_ev must be finite and non-negative");
        }
        if self.geometry.is_some() {
            return bad(
                "geometry is not supported yet: transport does not honour volume boundaries (infinite water); leave it unset",
            );
        }
        Ok(())
    }

    pub(crate) fn primary(&self, history_id: u64) -> Primary {
        Primary {
            history_id,
            energy: Energy::from_ev(self.primary_energy_ev),
            position: Vec3::ZERO,
            direction: Vec3::new(self.direction[0], self.direction[1], self.direction[2]),
        }
    }

    pub(crate) fn transport(&self) -> TransportConfig {
        TransportConfig {
            tracking_cut: self.tracking_cut_ev.map(Energy::from_ev),
            max_interactions: self.max_interactions,
            transport_secondaries: self.transport_secondaries,
            ..TransportConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_validate_and_bad_values_do_not() {
        assert!(RunConfig::new(1000.0, 10).validate().is_ok());
        assert!(RunConfig::new(0.0, 10).validate().is_err());
        assert!(RunConfig::new(f64::NAN, 10).validate().is_err());
        assert!(RunConfig::new(1000.0, 0).validate().is_err());
        let mut c = RunConfig::new(1000.0, 1);
        c.batch_size = 0;
        assert!(c.validate().is_err());
        c.batch_size = 1;
        c.threads = Some(0);
        assert!(c.validate().is_err());
        c.threads = None;
        c.tracking_cut_ev = Some(-1.0);
        assert!(c.validate().is_err());
    }

    const BASE: &str = "master_seed = 1
first_history_id = 0
n_histories = 4
        primary_energy_ev = 1000.0
direction = [0.0, 0.0, 1.0]
transport_secondaries = true
        record_physchem = false
batch_size = 1024
";

    #[test]
    fn geometry_defaults_to_none_and_is_not_serialised() {
        let c = RunConfig::new(1000.0, 4);
        assert!(c.geometry.is_none());
        assert!(!serde_json::to_string(&c).unwrap().contains("geometry"));
        assert!(!toml::to_string(&c).unwrap().contains("geometry"));
        let back: RunConfig = toml::from_str(BASE).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn geometry_some_is_rejected_by_validate() {
        let mut c = RunConfig::new(1000.0, 4);
        c.geometry = Some(Volume::box_(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0)).unwrap());
        let err = c.validate().unwrap_err().to_string();
        assert!(err.contains("geometry"), "{err}");
    }

    #[test]
    fn geometry_round_trips_for_each_shape() {
        let shapes = [
            "shape = \"box\"
centre = [1.0, 2.0, 3.0]
half_lengths = [10.0, 10.0, 5.0]
",
            "shape = \"tubs\"
centre = [0.0, 0.0, 0.0]
rmin = 0.0
rmax = 50.0
half_z = 20.0
             start_phi = 0.0
delta_phi = 3.141592653589793
",
            "shape = \"sphere\"
centre = [0.0, 0.0, 0.0]
rmin = 5.0
rmax = 50.0
             start_phi = 0.0
delta_phi = 6.283185307179586
start_theta = 0.0
             delta_theta = 3.141592653589793
",
        ];
        for shape in shapes {
            let text = format!(
                "{BASE}
[geometry]
{shape}"
            );
            let c: RunConfig = toml::from_str(&text).unwrap();
            assert!(c.geometry.is_some());
            let back: RunConfig = toml::from_str(&toml::to_string(&c).unwrap()).unwrap();
            assert_eq!(back, c);
            let json: RunConfig =
                serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
            assert_eq!(json, c);
            assert!(c.validate().is_err());
        }
    }

    #[test]
    fn invalid_geometry_is_rejected_at_deserialisation() {
        for shape in [
            "shape = \"box\"
centre = [0.0, 0.0, 0.0]
half_lengths = [-1.0, 1.0, 1.0]
",
            "shape = \"cone\"
centre = [0.0, 0.0, 0.0]
",
            "shape = \"sphere\"
centre = [0.0, 0.0, 0.0]
rmin = 5.0
rmax = 1.0
             start_phi = 0.0
delta_phi = 1.0
start_theta = 0.0
delta_theta = 1.0
",
        ] {
            let text = format!(
                "{BASE}
[geometry]
{shape}"
            );
            assert!(toml::from_str::<RunConfig>(&text).is_err(), "{shape}");
        }
    }
}
