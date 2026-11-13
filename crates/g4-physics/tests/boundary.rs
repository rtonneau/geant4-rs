//! Boundary-limited transport (docs/04 D.6).
//!
//! Pre-registered criteria (fixed before the first run):
//! * Unbounded (`None`) runs are bitwise identical to the infinite-medium code before the
//!   boundary was added. This was checked once, same platform and profile, against base commit
//!   00b9855 (docs/reproducibility.md); it is not pinned here because bitwise values are only
//!   promised per platform and toolchain (`powf` in the solvation range differs across
//!   platforms and build profiles).
//! * A volume much larger than every track (box of half-length 1e9 nm) gives outcomes and event
//!   streams equal (`==`) to `None`.
//! * A primary on the surface moving out escapes with no interaction and no random draw;
//!   a tiny volume (box of half-length 0.01 nm) makes every primary escape.
//! * With escapes, the ledger closes to 1e-9 relative for every history, escaped primaries end on
//!   the surface, and `escaped > 0` occurs for secondaries too.
//! * The event stream with a volume is a prefix of the unbounded one up to the first escape
//!   (same random-draw order until a boundary limits a step).
//! * A primary outside the volume is `PrimaryOutsideVolume`, with no draw.
//! * `on_escape` fires once per escaped track: on the volume Surface, with a unit direction;
//!   the summed exit energies equal `ledger.escaped` to 1e-12 relative, and at most one event is
//!   the primary's (exactly one when it terminates `Escaped`). Recording exits leaves the outcome
//!   unchanged.

mod common;

use common::{data, ev};
use g4_core::geometry::{Inside, Volume};
use g4_core::rng::{RngStreams, Stage};
use g4_core::vec3::Vec3;
use g4_physics::error::PhysicsError;
use g4_physics::models::ElasticRotation;
use g4_physics::table::ProcessTable;
use g4_physics::transport::{
    ExitEvent, HistoryObserver, HistoryOutcome, NoObserver, Primary, SolvationEvent, StepEvent,
    Termination, TransportConfig, transport_electron,
};
use rand::Rng;

fn table() -> Option<ProcessTable> {
    Some(ProcessTable::option2_electron(&data()?, ElasticRotation::Geant4).unwrap())
}

fn primary(e: f64, id: u64, position: Vec3, direction: Vec3) -> Primary {
    Primary {
        energy: ev(e),
        position,
        direction,
        history_id: id,
    }
}

/// Every interaction and solvation, in order, with all their fields.
#[derive(Default)]
struct Events(Vec<String>);

impl HistoryObserver for Events {
    fn on_interaction(&mut self, e: &StepEvent<'_>) {
        self.0.push(format!(
            "I {} {:?} {} {:?} {:?} {:?} {:?}",
            e.index,
            e.process,
            e.generation,
            e.free_path_nm.to_bits(),
            e.position,
            e.energy_before,
            e.final_state
        ));
    }
    fn on_solvation(&mut self, e: &SolvationEvent) {
        self.0.push(format!(
            "S {} {:?} {:?}",
            e.generation, e.origin, e.displacement_nm
        ));
    }
}

fn run(
    table: &ProcessTable,
    volume: Option<&Volume>,
    seed: u64,
    id: u64,
    e: f64,
) -> (HistoryOutcome, Vec<String>) {
    let mut rng = RngStreams::new(seed).stream(id, Stage::Physics).unwrap();
    let mut events = Events::default();
    let p = primary(e, id, Vec3::ZERO, Vec3::new(0.3, -0.4, 0.866));
    let out = transport_electron(
        table,
        &TransportConfig::default(),
        p,
        volume,
        &mut rng,
        &mut events,
    )
    .unwrap();
    (out, events.0)
}

#[test]
fn a_huge_volume_equals_the_infinite_medium() {
    let Some(table) = table() else { return };
    let huge = Volume::box_(Vec3::ZERO, Vec3::new(1.0e9, 1.0e9, 1.0e9)).unwrap();
    for e in [1.0e3, 1.0e4] {
        for id in 0..100 {
            let a = run(&table, None, 5, id, e);
            let b = run(&table, Some(&huge), 5, id, e);
            assert_eq!(a, b, "history {id} at {e} eV");
        }
    }
}

#[test]
fn surface_start_moving_out_escapes_without_a_draw() {
    let Some(table) = table() else { return };
    let v = Volume::box_(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0)).unwrap();
    let streams = RngStreams::new(3);
    let mut rng = streams.stream(0, Stage::Physics).unwrap();
    let p = primary(
        1.0e4,
        0,
        Vec3::new(0.0, 0.0, 10.0),
        Vec3::new(0.0, 0.0, 1.0),
    );
    let out = transport_electron(
        &table,
        &TransportConfig::default(),
        p,
        Some(&v),
        &mut rng,
        &mut NoObserver,
    )
    .unwrap();
    assert_eq!(out.termination, Termination::Escaped);
    assert_eq!(out.steps, 0);
    assert_eq!(out.n_escaped, 1);
    assert_eq!(out.ledger.escaped, 1.0e4);
    assert_eq!(out.track_length_nm, 0.0);
    let mut untouched = streams.stream(0, Stage::Physics).unwrap();
    assert_eq!(rng.next_u64(), untouched.next_u64());
}

#[test]
fn a_tiny_volume_lets_every_primary_escape() {
    let Some(table) = table() else { return };
    let tiny = Volume::box_(Vec3::ZERO, Vec3::new(0.01, 0.01, 0.01)).unwrap();
    for id in 0..100 {
        let (out, _) = run(&table, Some(&tiny), 9, id, 1.0e4);
        assert_eq!(out.termination, Termination::Escaped, "history {id}");
        assert_eq!(tiny.inside(out.end_position), Inside::Surface);
        assert!(out.ledger.residual().abs() <= 1e-9 * out.ledger.e_in);
        assert!(out.ledger.escaped > 0.0);
    }
}

#[test]
fn ledger_closes_with_escapes() {
    let Some(table) = table() else { return };
    let volumes = [
        Volume::sphere(
            Vec3::ZERO,
            0.0,
            20.0,
            0.0,
            std::f64::consts::TAU,
            0.0,
            std::f64::consts::PI,
        )
        .unwrap(),
        Volume::box_(Vec3::new(5.0, 0.0, 0.0), Vec3::new(100.0, 100.0, 20.0)).unwrap(),
        Volume::tubs(Vec3::ZERO, 0.0, 30.0, 200.0, 0.0, std::f64::consts::TAU).unwrap(),
    ];
    let mut secondary_escapes = 0u64;
    let mut primary_escapes = 0u64;
    for v in &volumes {
        for e in [1.0e3, 1.0e4] {
            for id in 0..200 {
                let (out, _) = run(&table, Some(v), 21, id, e);
                let l = out.ledger;
                assert!(
                    l.residual().abs() <= 1e-9 * l.e_in,
                    "residual {} at {e} eV, history {id}",
                    l.residual()
                );
                if out.termination == Termination::Escaped {
                    primary_escapes += 1;
                    assert_eq!(v.inside(out.end_position), Inside::Surface);
                    secondary_escapes += out.n_escaped - 1;
                } else {
                    secondary_escapes += out.n_escaped;
                }
            }
        }
    }
    assert!(primary_escapes > 0 && secondary_escapes > 0);
}

#[test]
fn draw_order_is_unchanged_until_the_first_escape() {
    let Some(table) = table() else { return };
    let v = Volume::box_(Vec3::ZERO, Vec3::new(30.0, 30.0, 30.0)).unwrap();
    let mut checked = 0;
    for id in 0..100 {
        let (_, free) = run(&table, None, 17, id, 1.0e4);
        let (out, bounded) = run(&table, Some(&v), 17, id, 1.0e4);
        if out.n_escaped == 0 {
            assert_eq!(free, bounded);
            continue;
        }
        if out.termination == Termination::Escaped {
            // The primary is followed first and its escape is the first boundary-limited step:
            // its interactions (the leading generation-0 events) must match the free run.
            let k = bounded
                .iter()
                .take_while(|s| s.starts_with('I') && s.split(' ').nth(3) == Some("0"))
                .count();
            assert_eq!(bounded[..k], free[..k], "history {id}");
            assert!(
                free.len() > k,
                "the free primary goes on after the escape point"
            );
            checked += 1;
        }
    }
    assert!(checked > 0);
}

#[test]
fn a_primary_outside_the_volume_is_rejected_without_a_draw() {
    let Some(table) = table() else { return };
    let v = Volume::box_(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0)).unwrap();
    let streams = RngStreams::new(1);
    let mut rng = streams.stream(0, Stage::Physics).unwrap();
    let p = primary(
        1.0e3,
        0,
        Vec3::new(0.0, 0.0, 11.0),
        Vec3::new(0.0, 0.0, -1.0),
    );
    let err = transport_electron(
        &table,
        &TransportConfig::default(),
        p,
        Some(&v),
        &mut rng,
        &mut NoObserver,
    )
    .unwrap_err();
    assert!(matches!(err, PhysicsError::PrimaryOutsideVolume { .. }));
    let mut untouched = streams.stream(0, Stage::Physics).unwrap();
    assert_eq!(rng.next_u64(), untouched.next_u64());
}

#[derive(Default)]
struct Exits(Vec<ExitEvent>);

impl HistoryObserver for Exits {
    fn on_escape(&mut self, e: &ExitEvent) {
        self.0.push(*e);
    }
}

#[test]
fn exit_events_lie_on_the_surface_and_match_the_ledger() {
    let Some(table) = table() else { return };
    let volumes = [
        Volume::sphere(
            Vec3::ZERO,
            0.0,
            20.0,
            0.0,
            std::f64::consts::TAU,
            0.0,
            std::f64::consts::PI,
        )
        .unwrap(),
        Volume::box_(Vec3::new(5.0, 0.0, 0.0), Vec3::new(100.0, 100.0, 20.0)).unwrap(),
        Volume::tubs(Vec3::ZERO, 0.0, 30.0, 200.0, 0.0, std::f64::consts::TAU).unwrap(),
    ];
    let mut n_events = 0u64;
    for v in &volumes {
        for id in 0..200 {
            let mut rng = RngStreams::new(21).stream(id, Stage::Physics).unwrap();
            let mut exits = Exits::default();
            let p = primary(1.0e4, id, Vec3::ZERO, Vec3::new(0.3, -0.4, 0.866));
            let out = transport_electron(
                &table,
                &TransportConfig::default(),
                p,
                Some(v),
                &mut rng,
                &mut exits,
            )
            .unwrap();
            assert_eq!(out, run(&table, Some(v), 21, id, 1.0e4).0, "history {id}");
            assert_eq!(exits.0.len() as u64, out.n_escaped);
            n_events += out.n_escaped;
            let mut sum = 0.0;
            for e in &exits.0 {
                assert_eq!(v.inside(e.position), Inside::Surface, "{e:?}");
                assert!((e.direction.norm() - 1.0).abs() < 1e-12);
                assert!(e.energy.ev() > 0.0);
                sum += e.energy.ev();
            }
            let tol = 1e-12 * out.ledger.e_in;
            assert!((sum - out.ledger.escaped).abs() <= tol, "history {id}");
            let primaries = exits.0.iter().filter(|e| e.generation == 0).count();
            assert!(primaries <= 1);
            if out.termination == Termination::Escaped {
                assert_eq!(primaries, 1);
                assert_eq!(
                    exits.0.iter().find(|e| e.generation == 0).unwrap().track_id,
                    0
                );
            }
        }
    }
    assert!(n_events > 0);
}

/// Solvated-electron positions, from the physicochemical events.
#[derive(Default)]
struct Solvated(Vec<(Vec3, Vec3)>, Vec<Vec3>);

impl HistoryObserver for Solvated {
    fn on_solvation(&mut self, e: &SolvationEvent) {
        self.0.push((e.origin, e.displacement_nm));
    }
    fn on_physchem(&mut self, e: &g4_physics::physchem::PhysChemEvent) {
        if e.kind == g4_physics::physchem::PhysChemKind::Solvation {
            self.1.push(e.position);
        }
    }
}

/// Geant4 thermalisation clamp (docs/04 D.6, `G4DNAOneStepThermalizationModel.hpp:224-239`):
/// with a small sphere many solvated electrons start near the surface. Pre-registered:
/// no solvated electron is Outside (D2); a shortened one lies on its displacement ray at
/// `0.8 x safety`; an unshortened one is `origin + d` bit for bit; the outcome counters equal
/// the counts recomputed from the events. That the clamp draws no random number is covered by
/// `a_huge_volume_equals_the_infinite_medium`.
#[test]
fn solvated_electrons_are_clamped_inside_the_volume() {
    let Some(table) = table() else { return };
    let sphere = Volume::sphere(
        Vec3::ZERO,
        0.0,
        15.0,
        0.0,
        std::f64::consts::TAU,
        0.0,
        std::f64::consts::PI,
    )
    .unwrap();
    let (mut shortened, mut total) = (0u64, 0u64);
    for id in 0..200 {
        let mut rng = RngStreams::new(11).stream(id, Stage::Physics).unwrap();
        let mut obs = Solvated::default();
        let p = primary(300.0, id, Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));
        let out = transport_electron(
            &table,
            &TransportConfig::default(),
            p,
            Some(&sphere),
            &mut rng,
            &mut obs,
        )
        .unwrap();
        assert_eq!(obs.0.len(), obs.1.len());
        let (mut s, mut n) = (0u64, 0u64);
        for (&(origin, d), &placed) in obs.0.iter().zip(&obs.1) {
            assert_ne!(sphere.inside(placed), Inside::Outside, "history {id}");
            if placed == origin + d {
                assert!(sphere.safety(origin).unwrap() > d.norm());
            } else {
                s += 1;
                let safety = sphere.safety(origin).unwrap();
                assert!(safety <= d.norm());
                let moved = placed - origin;
                assert!((moved.norm() - 0.8 * safety).abs() <= 1e-12 * (1.0 + safety));
                assert!(moved.dot(d) >= 0.0);
                assert!((moved.cross(d)).norm() <= 1e-9 * (1.0 + d.norm()) * (1.0 + safety));
            }
            n += u64::from(sphere.inside(placed) != Inside::Inside);
        }
        assert_eq!(out.solvations_shortened, s, "history {id}");
        assert_eq!(out.solvated_not_inside, n, "history {id}");
        assert_eq!(out.solvations, obs.0.len() as u64);
        shortened += s;
        total += obs.0.len() as u64;
    }
    assert!(shortened > 0, "the test geometry must exercise the clamp");
    assert!(shortened < total);
}
