//! Dissociation products in a finite volume: the Geant4 `0.8 x safety` clamp
//! (`G4DNAMolecularDissociation.cc:203-222`, docs/04 D.6 and D.7).
//!
//! Pre-registered criteria:
//! * D4: an engine with no volume gives the same species and statistics as the default engine,
//!   and consumes the same random numbers.
//! * The clamp draws no random number: after a bounded conversion the stream is at the same
//!   place as after an unbounded one.
//! * D2: with the history's events inside a small sphere, no product is Outside; the placement
//!   counters equal the products whose displacement was shortened or that are not strictly
//!   Inside.
//! * Each product lies on its unclamped displacement ray, at `min(0.8 x safety, |d|)`.

use g4_chemistry::PhysChemEngine;
use g4_core::geometry::{Inside, Volume};
use g4_core::physchem::{PhysChemEvent, PhysChemKind};
use g4_core::rng::{RngStreams, Stage};
use g4_core::units::Energy;
use g4_core::vec3::Vec3;

fn sphere(r: f64) -> Volume {
    Volume::sphere(
        Vec3::ZERO,
        0.0,
        r,
        0.0,
        std::f64::consts::TAU,
        0.0,
        std::f64::consts::PI,
    )
    .unwrap()
}

/// Ionisations and excitations spread over a sphere of radius 3 nm, some near its surface.
fn events() -> Vec<PhysChemEvent> {
    let mut out = Vec::new();
    for i in 0..60u32 {
        let t = f64::from(i);
        let r = 3.0 * (0.3 + 0.7 * ((t * 0.37).sin().abs()));
        let (st, ct) = (t * 1.3).sin_cos();
        let (sp, cp) = (t * 0.7).sin_cos();
        let position = Vec3::new(r * st * cp, r * st * sp, r * ct);
        let (kind, level) = if i % 3 == 0 {
            (PhysChemKind::Excitation, (i % 5) as u8)
        } else {
            (PhysChemKind::Ionisation, (i % 5) as u8)
        };
        out.push(PhysChemEvent {
            kind,
            level_or_shell: level,
            position,
            deposited_energy: Energy::from_ev(10.0),
            parent_history: 0,
        });
    }
    out
}

#[test]
fn no_volume_is_the_default_engine() {
    let a = PhysChemEngine::option3().unwrap();
    let b = PhysChemEngine::option3().unwrap().with_volume(None);
    let streams = RngStreams::new(3);
    let mut ra = streams.stream(0, Stage::PhysChem).unwrap();
    let mut rb = streams.stream(0, Stage::PhysChem).unwrap();
    let ha = a.convert_history(&events(), &mut ra).unwrap();
    let hb = b.convert_history(&events(), &mut rb).unwrap();
    assert_eq!(ha, hb);
    assert_eq!(ra.uniform_open01(), rb.uniform_open01());
    assert_eq!(ha.stats.placement, Default::default());
}

#[test]
fn products_are_clamped_inside_and_counted_without_random_numbers() {
    let volume = sphere(3.0);
    let free = PhysChemEngine::option3().unwrap();
    let bounded = free.clone().with_volume(Some(volume.clone()));
    let streams = RngStreams::new(9);
    let mut total_shortened = 0u64;
    let mut checked = 0usize;
    // One event per conversion: no e_aq is present, so there is no recombination and every
    // product's origin is the event position.
    for (k, event) in events().into_iter().enumerate() {
        for rep in 0..20u64 {
            let id = k as u64 * 100 + rep;
            let mut rf = streams.stream(id, Stage::PhysChem).unwrap();
            let mut rb = streams.stream(id, Stage::PhysChem).unwrap();
            let hf = free.convert_history(&[event], &mut rf).unwrap();
            let hb = bounded.convert_history(&[event], &mut rb).unwrap();
            assert_eq!(rf.uniform_open01(), rb.uniform_open01(), "history {id}");
            assert_eq!(hf.species.len(), hb.species.len());
            assert_eq!(hf.stats.channel_counts, hb.stats.channel_counts);

            let origin = event.position;
            let safety = volume.safety(origin).unwrap();
            let (mut shortened, mut not_inside) = ([0u64; 15], [0u64; 15]);
            for (f, b) in hf.species.iter().zip(&hb.species) {
                assert_eq!(f.species, b.species);
                assert_ne!(volume.inside(b.position), Inside::Outside, "history {id}");
                not_inside[b.species.index()] +=
                    u64::from(volume.inside(b.position) != Inside::Inside);
                let d = f.position - origin;
                let m = b.position - origin;
                let expected = (0.8 * safety).min(d.norm());
                assert!(
                    (m.norm() - expected).abs() <= 1e-12 * (1.0 + d.norm()),
                    "history {id}"
                );
                assert!(d.cross(m).norm() <= 1e-9 * (1.0 + d.norm()) * (1.0 + m.norm()));
                assert!(d.dot(m) >= 0.0);
                if expected < d.norm() {
                    shortened[b.species.index()] += 1;
                }
                checked += 1;
            }
            assert_eq!(hb.stats.placement.shortened, shortened, "history {id}");
            assert_eq!(hb.stats.placement.not_inside, not_inside, "history {id}");
            total_shortened += shortened.iter().sum::<u64>();
        }
    }
    assert!(checked > 0);
    assert!(total_shortened > 0, "the geometry must exercise the clamp");
}
