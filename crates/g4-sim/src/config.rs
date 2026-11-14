//! Run configuration.

use crate::error::SimError;
use g4_core::geometry::{Inside, Volume};
use g4_core::rng::{RngStreams, Stage};
use g4_core::sampling::isotropic_direction;
use g4_core::units::Energy;
use g4_core::vec3::Vec3;
use g4_physics::transport::{Primary, TransportConfig};
use serde::{Deserialize, Serialize};

/// Parameters of a run of `n_histories` identical primaries.
///
/// Energies are eV, positions nm (field names carry the unit, docs/units.md). History `i`
/// (`first_history_id <= i < first_history_id + n_histories`) uses the stream
/// `(master_seed, i, Stage::Physics)`.
///
/// Unknown keys are rejected, so a misspelt or removed key (such as the pre-v5 `geometry`)
/// cannot silently fall back to a default such as infinite water.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunConfig {
    pub master_seed: u64,
    pub first_history_id: u64,
    pub n_histories: u64,
    pub primary_energy_ev: f64,
    /// Primary direction: a fixed vector `[x, y, z]` (normalised by the transport) or
    /// `"isotropic"` (one direction per history from its `Stage::Source` stream, ADR 0006).
    pub direction: PrimaryDirection,
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
    /// Optional liquid-water volume (nm, rad; `g4_core::geometry`, docs/04 D.5). Transport
    /// kills tracks at its boundary and books their energy as escaped (docs/04 D.6); `None` is
    /// infinite water. The primary starts at the origin, which must not be outside the volume.
    /// The chemistry stage ignores the boundary (docs/04 D.6). Omitted from serialised output
    /// when `None`, so existing run metadata is unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<Volume>,
}

impl RunConfig {
    /// Defaults: seed 1, histories 0..n along +z, no tracking cut, batches of 1024.
    pub fn new(primary_energy_ev: f64, n_histories: u64) -> Self {
        Self {
            master_seed: 1,
            first_history_id: 0,
            n_histories,
            primary_energy_ev,
            direction: PrimaryDirection::Fixed([0.0, 0.0, 1.0]),
            tracking_cut_ev: None,
            max_interactions: None,
            transport_secondaries: true,
            record_physchem: false,
            batch_size: 1024,
            threads: None,
            volume: None,
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
        if let PrimaryDirection::Fixed(d) = self.direction
            && !(d.iter().all(|c| c.is_finite()) && d.iter().any(|&c| c != 0.0))
        {
            return bad("a fixed direction must be finite and non-zero");
        }
        if let Some(v) = &self.volume
            && v.inside(Vec3::ZERO) == Inside::Outside
        {
            return bad("volume must contain the primary start position (the origin)");
        }
        Ok(())
    }

    /// The primary of history `history_id`. A fixed direction draws nothing; an isotropic one
    /// draws two uniforms from `(master_seed, history_id, Stage::Source)` (docs/reproducibility.md),
    /// so the `Physics` stream is the same in both modes.
    pub(crate) fn primary(
        &self,
        history_id: u64,
        streams: &RngStreams,
    ) -> Result<Primary, SimError> {
        let direction = match self.direction {
            PrimaryDirection::Fixed(d) => Vec3::new(d[0], d[1], d[2]),
            PrimaryDirection::Isotropic => {
                isotropic_direction(&mut streams.stream(history_id, Stage::Source)?)
            }
        };
        Ok(Primary {
            history_id,
            energy: Energy::from_ev(self.primary_energy_ev),
            position: Vec3::ZERO,
            direction,
        })
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

/// Direction of the primaries of a run.
///
/// Serialised as the bare array `[x, y, z]` (as before this type existed, so existing configs
/// and run metadata are unchanged) or as the string `"isotropic"`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "DirectionRepr", into = "DirectionRepr")]
pub enum PrimaryDirection {
    /// The same direction for every history.
    Fixed([f64; 3]),
    /// Uniform on the unit sphere, independently per history (Geant4 `G4RandomDirection()`
    /// distribution, `g4_core::sampling::isotropic_direction`).
    Isotropic,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum DirectionRepr {
    Fixed([f64; 3]),
    Keyword(String),
}

impl TryFrom<DirectionRepr> for PrimaryDirection {
    type Error = String;
    fn try_from(r: DirectionRepr) -> Result<Self, String> {
        match r {
            DirectionRepr::Fixed(d) => Ok(Self::Fixed(d)),
            DirectionRepr::Keyword(k) if k == "isotropic" => Ok(Self::Isotropic),
            DirectionRepr::Keyword(k) => Err(format!(
                "direction must be [x, y, z] or \"isotropic\", got \"{k}\""
            )),
        }
    }
}

impl From<PrimaryDirection> for DirectionRepr {
    fn from(d: PrimaryDirection) -> Self {
        match d {
            PrimaryDirection::Fixed(v) => Self::Fixed(v),
            PrimaryDirection::Isotropic => Self::Keyword("isotropic".to_owned()),
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
    fn volume_defaults_to_none_and_is_not_serialised() {
        let c = RunConfig::new(1000.0, 4);
        assert!(c.volume.is_none());
        assert!(!serde_json::to_string(&c).unwrap().contains("volume"));
        assert!(!toml::to_string(&c).unwrap().contains("volume"));
        let back: RunConfig = toml::from_str(BASE).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn volume_must_contain_the_origin() {
        let mut c = RunConfig::new(1000.0, 4);
        c.volume = Some(Volume::box_(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0)).unwrap());
        assert!(c.validate().is_ok());
        c.volume =
            Some(Volume::box_(Vec3::new(0.0, 0.0, 30.0), Vec3::new(10.0, 10.0, 10.0)).unwrap());
        let err = c.validate().unwrap_err().to_string();
        assert!(err.contains("origin"), "{err}");
    }

    #[test]
    fn volume_round_trips_for_each_shape() {
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
[volume]
{shape}"
            );
            let c: RunConfig = toml::from_str(&text).unwrap();
            assert!(c.volume.is_some());
            let back: RunConfig = toml::from_str(&toml::to_string(&c).unwrap()).unwrap();
            assert_eq!(back, c);
            let json: RunConfig =
                serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
            assert_eq!(json, c);
            let contains_origin = c.volume.as_ref().unwrap().inside(Vec3::ZERO) != Inside::Outside;
            assert_eq!(c.validate().is_ok(), contains_origin);
        }
    }

    #[test]
    fn invalid_volume_is_rejected_at_deserialisation() {
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
[volume]
{shape}"
            );
            assert!(toml::from_str::<RunConfig>(&text).is_err(), "{shape}");
        }
    }

    #[test]
    fn legacy_geometry_key_is_rejected() {
        let text = format!(
            "{BASE}
[geometry]
shape = \"box\"
centre = [0.0, 0.0, 0.0]
half_lengths = [1.0, 1.0, 1.0]
"
        );
        assert!(toml::from_str::<RunConfig>(&text).is_err());
    }

    #[test]
    fn direction_serialises_as_array_or_isotropic_keyword() {
        let fixed: RunConfig = toml::from_str(BASE).unwrap();
        assert_eq!(fixed.direction, PrimaryDirection::Fixed([0.0, 0.0, 1.0]));
        assert!(
            serde_json::to_string(&fixed)
                .unwrap()
                .contains("\"direction\":[0.0,0.0,1.0]")
        );
        let iso_src = BASE.replace("[0.0, 0.0, 1.0]", "\"isotropic\"");
        let iso: RunConfig = toml::from_str(&iso_src).unwrap();
        assert_eq!(iso.direction, PrimaryDirection::Isotropic);
        let json = serde_json::to_string(&iso).unwrap();
        assert!(json.contains("\"direction\":\"isotropic\""));
        assert_eq!(serde_json::from_str::<RunConfig>(&json).unwrap(), iso);
        assert_eq!(
            toml::from_str::<RunConfig>(&toml::to_string(&iso).unwrap()).unwrap(),
            iso
        );
        assert!(
            toml::from_str::<RunConfig>(&BASE.replace("[0.0, 0.0, 1.0]", "\"isotrope\"")).is_err()
        );
    }

    #[test]
    fn fixed_direction_must_be_finite_and_non_zero() {
        let mut c = RunConfig::new(1000.0, 1);
        c.direction = PrimaryDirection::Fixed([0.0, 0.0, 0.0]);
        assert!(c.validate().is_err());
        c.direction = PrimaryDirection::Fixed([f64::NAN, 0.0, 1.0]);
        assert!(c.validate().is_err());
        c.direction = PrimaryDirection::Isotropic;
        assert!(c.validate().is_ok());
    }

    #[test]
    fn isotropic_primary_uses_the_source_stream_only() {
        let streams = RngStreams::new(7);
        let mut c = RunConfig::new(1000.0, 1);
        let fixed = c.primary(3, &streams).unwrap();
        assert_eq!(fixed.direction, Vec3::new(0.0, 0.0, 1.0));
        c.direction = PrimaryDirection::Isotropic;
        let a = c.primary(3, &streams).unwrap();
        let expected = isotropic_direction(&mut streams.stream(3, Stage::Source).unwrap());
        assert_eq!(a.direction, expected);
        assert_ne!(a.direction, c.primary(4, &streams).unwrap().direction);
    }
}
