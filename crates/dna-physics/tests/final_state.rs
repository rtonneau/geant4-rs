//! Final-state kinematics and the reproduced Geant4 quirks.
//!
//! Analytic checks (docs/02 B.7.3 item 3): Born angle formula, momentum balance of the primary,
//! the Champion rotation with and without CLHEP's non-unit axis, and each quirk on a synthetic
//! table that makes it reachable (they are unreachable in the real tables except through
//! rounding). Statistical thresholds are stated where used.

mod common;

use approx::assert_relative_eq;
use common::{data, ev};
use dna_core::rng::{RngStreams, Stage};
use dna_core::units::Area;
use dna_core::vec3::Vec3;
use dna_data::blocks::IonisationCdfTable;
use dna_data::xs::{CrossSectionTable, Interpolation, XsSpec};
use dna_physics::direction::clhep_orthogonal;
use dna_physics::models::{BornExcitation, BornIonisation, ElasticRotation, scatter};
use dna_physics::sampling::uniform;
use dna_physics::testing::ScriptedRng;
use dna_physics::water::{ELECTRON_MASS_C2_EV, IONISATION_ENERGIES_EV};

fn unit_xs(columns: [f64; 5]) -> CrossSectionTable {
    let row = |e: f64| {
        format!(
            "{e} {} {} {} {} {}\n",
            columns[0], columns[1], columns[2], columns[3], columns[4]
        )
    };
    let text = format!("{}{}", row(8.0), row(1.0e6));
    CrossSectionTable::from_text(
        "synthetic",
        &text,
        XsSpec {
            scale: Area::from_nm2(1.0),
            interpolation: Interpolation::LogLog,
        },
    )
    .unwrap()
}

/// Cumulated-table stand-in: every shell has the same inverse CDF `W(P)`: `(w0, 0)`, `(w1, 0.5)`,
/// `(w2, 1)`, `(w3, 2)` (the `P = 2` padding of the real files).
fn synthetic_cdf(w: [f64; 4]) -> IonisationCdfTable {
    let block = |e: f64| {
        let mut s = String::new();
        for (wi, p) in w.iter().zip([0.0, 0.5, 1.0, 2.0]) {
            s += &format!("{e} {wi} {p} {p} {p} {p} {p}\n");
        }
        s
    };
    IonisationCdfTable::from_text("synthetic-cdf", &(block(8.0) + &block(1.0e6))).unwrap()
}

const Z: Vec3 = Vec3::new(0.0, 0.0, 1.0);

// ------------------------------------------------------------------ quirks --------------

#[test]
fn excitation_with_non_positive_remainder_changes_nothing_but_is_flagged() {
    // only level 4 (13.77 eV) has a cross section; a 9 eV electron cannot lose 13.77 eV
    let model = BornExcitation::new(unit_xs([0.0, 0.0, 0.0, 0.0, 1.0])).unwrap();
    let mut rng = ScriptedRng::new(&[0.5]);
    let fs = model.sample(ev(9.0), Z, &mut rng).unwrap();
    assert_eq!(fs.channel, 4);
    assert!(fs.quirks.excitation_noop);
    assert_eq!(fs.energy_after.ev(), 9.0);
    assert_eq!(fs.local_deposit.ev(), 0.0);
    assert_eq!(fs.direction_after, Z);
    assert!(fs.secondary.is_none());
    assert_eq!(rng.consumed(), 1);

    // exactly zero remainder is also a no-op (`newEnergy > 0` is strict)
    let fs = model
        .sample(ev(13.77), Z, &mut ScriptedRng::new(&[0.5]))
        .unwrap();
    assert!(fs.quirks.excitation_noop);
    // just above: a normal excitation
    let e = f64::from_bits(13.77f64.to_bits() + 1);
    let fs = model
        .sample(ev(e), Z, &mut ScriptedRng::new(&[0.5]))
        .unwrap();
    assert!(!fs.quirks.excitation_noop);
    assert_eq!(fs.local_deposit.ev(), 13.77);
    assert!(fs.energy_after.ev() > 0.0);
}

#[test]
fn ionisation_below_the_binding_energy_returns_untouched() {
    // only shell 3 (32.3 eV) is open; a 20 eV electron cannot ionise it
    let model = BornIonisation::new(
        unit_xs([0.0, 0.0, 0.0, 1.0, 0.0]),
        synthetic_cdf([40.0, 50.0, 60.0, 70.0]),
    )
    .unwrap();
    let mut rng = ScriptedRng::new(&[0.5]);
    let fs = model.sample(ev(20.0), Z, &mut rng).unwrap();
    assert_eq!(fs.channel, 3);
    assert!(fs.quirks.ionisation_noop_below_binding);
    assert_eq!(fs.energy_after.ev(), 20.0);
    assert_eq!(fs.local_deposit.ev(), 0.0);
    assert_eq!(fs.direction_after, Z);
    assert!(fs.secondary.is_none());
    assert_eq!(
        rng.consumed(),
        1,
        "returns right after the shell draw (`:472`)"
    );
}

#[test]
fn negative_transfer_is_clamped_to_a_zero_energy_electron_that_is_not_created() {
    // W(P) < B_0 = 10.79 eV everywhere: Geant4 clamps to 0 (`:1003`), still draws the angle, and
    // creates no electron (`:492`); the binding energy is still deposited (`:525, 581`).
    let model = BornIonisation::new(
        unit_xs([1.0, 0.0, 0.0, 0.0, 0.0]),
        synthetic_cdf([5.0, 6.0, 7.0, 8.0]),
    )
    .unwrap();
    // draws: shell, transfer p, cos(theta) (W < 50 eV -> 1 draw), phi
    let mut rng = ScriptedRng::new(&[0.5, 0.25, 0.3, 0.6]);
    let fs = model.sample(ev(100.0), Z, &mut rng).unwrap();
    assert!(fs.quirks.transfer_clamped_to_zero);
    assert!(fs.secondary.is_none());
    assert_eq!(fs.local_deposit.ev(), 10.79);
    assert_relative_eq!(fs.energy_after.ev(), 100.0 - 10.79, max_relative = 1e-15);
    assert_eq!(rng.consumed(), 4);
    // no secondary -> no momentum transfer -> direction unchanged
    assert_relative_eq!(fs.direction_after.dot(Z), 1.0, epsilon = 1e-15);
}

#[test]
fn k_shell_ionisation_deposits_the_whole_binding_energy_and_is_flagged() {
    let model = BornIonisation::new(
        unit_xs([0.0, 0.0, 0.0, 0.0, 1.0]),
        synthetic_cdf([540.0, 600.0, 700.0, 800.0]),
    )
    .unwrap();
    let k = 1000.0;
    let fs = model
        .sample(ev(k), Z, &mut ScriptedRng::new(&[0.5, 0.5, 0.5, 0.5]))
        .unwrap();
    assert_eq!(fs.channel, 4);
    assert!(fs.quirks.k_shell_deexcitation_skipped);
    assert_eq!(fs.local_deposit.ev(), 539.0);
    let sec = fs.secondary.expect("W > 0");
    assert_relative_eq!(
        fs.energy_after.ev() + fs.local_deposit.ev() + sec.energy.ev(),
        k,
        max_relative = 1e-14
    );
}

#[test]
fn shell_two_below_19_ev_is_redrawn_until_another_shell_is_drawn() {
    // shell 2 dominates; the scripted draws first select it (u small: the walk starts from the
    // last shell and shell 2 holds the largest remaining share), twice, then shell 0.
    let model = BornIonisation::new(
        unit_xs([1.0, 0.0, 1000.0, 0.0, 0.0]),
        synthetic_cdf([12.0, 14.0, 16.0, 18.0]),
    )
    .unwrap();
    // u = 0.1 -> shell 2 (cumulative from the top: shell 2 owns u < 1000/1001); u = 0.9999 -> shell 0
    let mut rng = ScriptedRng::new(&[0.1, 0.2, 0.9999, 0.5, 0.5, 0.5]);
    let fs = model.sample(ev(18.5), Z, &mut rng).unwrap();
    assert_eq!(fs.quirks.shell2_redraws, 2);
    assert_eq!(fs.channel, 0);
    // the same electron above 19 eV keeps shell 2
    let fs = model
        .sample(ev(19.5), Z, &mut ScriptedRng::new(&[0.1, 0.5, 0.5, 0.5]))
        .unwrap();
    assert_eq!(fs.channel, 2);
    assert_eq!(fs.quirks.shell2_redraws, 0);
}

// ------------------------------------------------------- ionisation kinematics ----------

#[test]
fn primary_direction_follows_momentum_conservation_and_delta_angle_follows_born() {
    let Some(dir) = data() else { return };
    let model = BornIonisation::from_data_dir(&dir).unwrap();
    let mc2 = ELECTRON_MASS_C2_EV;
    let mut rng = RngStreams::new(31).stream(0, Stage::Physics).unwrap();
    let (mut n_iso, mut sum_cos_iso) = (0usize, 0.0f64);
    let mut checked_formula = 0usize;
    for e in [1.0e3, 1.0e4, 1.0e5] {
        for _ in 0..4000 {
            // random primary direction
            let c = 2.0 * uniform(&mut rng) - 1.0;
            let phi = 2.0 * std::f64::consts::PI * uniform(&mut rng);
            let s = (1.0 - c * c).sqrt();
            let d = Vec3::new(s * phi.cos(), s * phi.sin(), c);
            let fs = model.sample(ev(e), d, &mut rng).unwrap();
            let w = fs.secondary.map_or(0.0, |s| s.energy.ev());
            let p_in = (e * (e + 2.0 * mc2)).sqrt();
            let p_delta = (w * (w + 2.0 * mc2)).sqrt();
            let delta = fs.secondary.map_or(Vec3::ZERO, |s| s.direction);
            let expect = (d * p_in - delta * p_delta).normalised().unwrap();
            assert_relative_eq!(fs.direction_after.dot(expect), 1.0, epsilon = 1e-12);
            assert_relative_eq!(fs.direction_after.norm(), 1.0, epsilon = 1e-12);
            // energy bookkeeping: E' = E - B - W, deposit B
            let b = IONISATION_ENERGIES_EV[fs.channel];
            assert_eq!(fs.local_deposit.ev(), b);
            assert_relative_eq!(fs.energy_after.ev(), e - b - w, max_relative = 1e-14);

            if let Some(sec) = fs.secondary {
                let cos_delta = sec.direction.dot(d);
                if w > 200.0 {
                    // B.7.3: sin^2 = (1 - W/k) / (1 + W/(2 m c^2)), cos = sqrt(1 - sin^2)
                    let sin2 = (1.0 - w / e) / (1.0 + w / (2.0 * mc2));
                    assert_relative_eq!(cos_delta, (1.0 - sin2).sqrt(), epsilon = 1e-9);
                    checked_formula += 1;
                } else if w < 50.0 {
                    n_iso += 1;
                    sum_cos_iso += cos_delta;
                } else {
                    // 50-200 eV: isotropic (10 %) or cos in [0, sqrt(2)/2]; never backward unless isotropic
                    assert!((-1.0..=1.0).contains(&cos_delta));
                }
            }
        }
    }
    assert!(
        checked_formula > 100,
        "formula branch exercised ({checked_formula})"
    );
    // isotropic branch: mean cosine 0, sigma = 1/sqrt(3 n); threshold 5 sigma (pre-registered)
    let mean = sum_cos_iso / n_iso as f64;
    assert!(n_iso > 1000);
    assert!(
        mean.abs() < 5.0 / (3.0 * n_iso as f64).sqrt(),
        "mean cos {mean}, n {n_iso}"
    );
}

// ---------------------------------------------------------- Champion rotation ------------

#[test]
fn geant4_rotation_shrinks_the_angle_by_the_orthogonal_axis_norm_exact_does_not() {
    let mut rng = RngStreams::new(8).stream(0, Stage::Physics).unwrap();
    let mut min_ratio: f64 = 1.0;
    for _ in 0..5000 {
        let c = 2.0 * uniform(&mut rng) - 1.0;
        let phi = 2.0 * std::f64::consts::PI * uniform(&mut rng);
        let s = (1.0 - c * c).sqrt();
        let z = Vec3::new(s * phi.cos(), s * phi.sin(), c);
        let cos_theta = 2.0 * uniform(&mut rng) - 1.0;
        let psi = 2.0 * std::f64::consts::PI * uniform(&mut rng);

        let exact = scatter(z, cos_theta, psi, ElasticRotation::Exact);
        assert_relative_eq!(exact.dot(z), cos_theta, epsilon = 1e-12);
        assert_relative_eq!(exact.norm(), 1.0, epsilon = 1e-12);

        // Geant4: new = a sin(theta)(cos psi x + sin psi y) + cos(theta) z with |x| = |y| = a
        let a2 = clhep_orthogonal(z).dot(clhep_orthogonal(z));
        assert!(
            (2.0 / 3.0 - 1e-12..=1.0 + 1e-12).contains(&a2),
            "a^2 = {a2}"
        );
        let g4 = scatter(z, cos_theta, psi, ElasticRotation::Geant4);
        let expect =
            cos_theta / (cos_theta * cos_theta + a2 * (1.0 - cos_theta * cos_theta)).sqrt();
        assert_relative_eq!(g4.dot(z), expect, epsilon = 1e-12);
        assert_relative_eq!(g4.norm(), 1.0, epsilon = 1e-12);
        min_ratio = min_ratio.min(a2);
    }
    assert!(
        min_ratio < 0.7,
        "off-axis directions were exercised ({min_ratio})"
    );
}

#[test]
fn both_rotations_agree_along_the_coordinate_axes() {
    for z in [
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
    ] {
        for (c, psi) in [(0.3, 0.7), (-0.8, 2.0), (0.99, 5.0)] {
            let a = scatter(z, c, psi, ElasticRotation::Geant4);
            let b = scatter(z, c, psi, ElasticRotation::Exact);
            assert_relative_eq!(a.x, b.x, epsilon = 1e-14);
            assert_relative_eq!(a.y, b.y, epsilon = 1e-14);
            assert_relative_eq!(a.z, b.z, epsilon = 1e-14);
        }
    }
    // body diagonal, 45 degrees: hand value cos_eff = 0.70711 / sqrt(0.5 + 2/3 * 0.5) = 0.77460
    let d = Vec3::new(1.0, 1.0, 1.0).normalised().unwrap();
    let c = std::f64::consts::FRAC_1_SQRT_2;
    let g4 = scatter(d, c, 1.0, ElasticRotation::Geant4);
    assert_relative_eq!(g4.dot(d), 0.774_596_669, epsilon = 1e-8);
    let exact = scatter(d, c, 1.0, ElasticRotation::Exact);
    assert_relative_eq!(exact.dot(d), c, epsilon = 1e-14);
}
