//! V8-style test of the displacer: mean-square displacement of every product (and of the separation of the pairs
//! that matter) against the analytic values computed independently in Python (`scripts/hand_physchem.py`,
//! `tests/fixtures/hand_values.json`).
//!
//! Acceptance criteria, fixed before the first run:
//! * `N = 200 000` samples per channel kind, master seed 20261015, stream `(seed, kind, PhysChem)`;
//! * z-test of the sample mean of `|d|^2` (standard error from the sample): `|z| < 4.1` (Bonferroni over the 20 tests
//!   below, overall false-alarm probability about 1e-3);
//! * products whose expected displacement is zero (OH-, the oxygen atom) must be exactly the zero vector (in the
//!   channels where the mother shift is 0);
//! * the isotropy of one displaced hydrogen atom: the three components each carry one third of the mean square
//!   (3 of the 20 tests).

mod common;

use common::fixture;
use dna_chemistry::Species;
use dna_chemistry::channels::DisplacementKind;
use dna_chemistry::displacer::{autoionisation_r_mean_nm, displacements};
use dna_core::rng::{RngStreams, Stage};
use dna_core::vec3::Vec3;

const N: usize = 200_000;
const SEED: u64 = 20261015;
const Z_MAX: f64 = 4.1;

fn mean_and_se(values: &[f64]) -> (f64, f64) {
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (mean, (var / n).sqrt())
}

fn check(label: &str, values: &[f64], expected: f64) {
    let (mean, se) = mean_and_se(values);
    let z = (mean - expected) / se;
    eprintln!("{label:>34}: <|d|^2> = {mean:9.5} (expected {expected:9.5}), z = {z:6.2}");
    assert!(z.abs() < Z_MAX, "{label}: z = {z}");
}

fn sample(kind: DisplacementKind, products: &[Species], stream: u64) -> Vec<Vec<Vec3>> {
    let r_mean = autoionisation_r_mean_nm(1.7);
    let mut rng = RngStreams::new(SEED)
        .stream(stream, Stage::PhysChem)
        .unwrap();
    (0..N)
        .map(|_| displacements(kind, products, r_mean, &mut rng))
        .collect()
}

fn sq(v: Vec3) -> f64 {
    v.x * v.x + v.y * v.y + v.z * v.z
}

#[test]
fn mean_square_displacements_match_the_hand_values() {
    let h = fixture("hand_values.json");
    let ms = |kind: &str, product: &str| {
        h["expected_mean_square_nm2"][kind][product]
            .as_f64()
            .unwrap()
    };
    let sep = |key: &str| {
        h["expected_separation_mean_square_nm2"][key]
            .as_f64()
            .unwrap()
    };

    // A^1B_1 type: OH, H and their separation (3 tests)
    let a1b1 = sample(
        DisplacementKind::A1b1Dissociation,
        &[Species::OH, Species::H],
        1,
    );
    check(
        "A1B1 OH",
        &a1b1.iter().map(|d| sq(d[0])).collect::<Vec<_>>(),
        ms("a1b1_dissociation_decay", "°OH"),
    );
    check(
        "A1B1 H",
        &a1b1.iter().map(|d| sq(d[1])).collect::<Vec<_>>(),
        ms("a1b1_dissociation_decay", "H"),
    );
    check(
        "A1B1 OH-H separation",
        &a1b1.iter().map(|d| sq(d[1] - d[0])).collect::<Vec<_>>(),
        sep("a1b1_dissociation_decay/°OH-H"),
    );

    // B1A1: H2 + 2 OH (3 tests)
    let b1a1 = sample(
        DisplacementKind::B1a1Dissociation,
        &[Species::H2, Species::OH, Species::OH],
        2,
    );
    check(
        "B1A1 H2",
        &b1a1.iter().map(|d| sq(d[0])).collect::<Vec<_>>(),
        ms("b1a1_dissociation_decay", "H2"),
    );
    check(
        "B1A1 OH(1)",
        &b1a1.iter().map(|d| sq(d[1])).collect::<Vec<_>>(),
        ms("b1a1_dissociation_decay", "°OH"),
    );
    check(
        "B1A1 OH(2)",
        &b1a1.iter().map(|d| sq(d[2])).collect::<Vec<_>>(),
        ms("b1a1_dissociation_decay", "°OH"),
    );

    // B1A1 2: O + 2 H (2 tests + exact zero)
    let b2 = sample(
        DisplacementKind::B1a1Dissociation2,
        &[Species::Oxy, Species::H, Species::H],
        3,
    );
    assert!(
        b2.iter().all(|d| d[0] == Vec3::ZERO),
        "the oxygen atom is not displaced"
    );
    check(
        "B1A1_2 H(1)",
        &b2.iter().map(|d| sq(d[1])).collect::<Vec<_>>(),
        ms("b1a1_dissociation_decay2", "H"),
    );
    check(
        "B1A1_2 H(2)",
        &b2.iter().map(|d| sq(d[2])).collect::<Vec<_>>(),
        ms("b1a1_dissociation_decay2", "H"),
    );

    // Ionisation: H3O+ + OH and their separation (3 tests)
    let ion = sample(
        DisplacementKind::IonisationDissociation,
        &[Species::H3Op, Species::OH],
        4,
    );
    check(
        "Ioni H3O+",
        &ion.iter().map(|d| sq(d[0])).collect::<Vec<_>>(),
        ms("ionisation_dissociation_decay", "H3Op"),
    );
    check(
        "Ioni OH",
        &ion.iter().map(|d| sq(d[1])).collect::<Vec<_>>(),
        ms("ionisation_dissociation_decay", "°OH"),
    );
    check(
        "Ioni H3O+-OH separation",
        &ion.iter().map(|d| sq(d[0] - d[1])).collect::<Vec<_>>(),
        sep("ionisation_dissociation_decay/H3Op-°OH"),
    );

    // Auto-ionisation: OH, H3O+, e_aq and the OH-H3O+ separation (4 tests)
    let auto = sample(
        DisplacementKind::AutoIonisation,
        &[Species::OH, Species::H3Op, Species::e_aq],
        5,
    );
    check(
        "Auto OH",
        &auto.iter().map(|d| sq(d[0])).collect::<Vec<_>>(),
        ms("auto_ionisation", "°OH"),
    );
    check(
        "Auto H3O+",
        &auto.iter().map(|d| sq(d[1])).collect::<Vec<_>>(),
        ms("auto_ionisation", "H3Op"),
    );
    check(
        "Auto e_aq",
        &auto.iter().map(|d| sq(d[2])).collect::<Vec<_>>(),
        ms("auto_ionisation", "e_aq"),
    );
    check(
        "Auto OH-H3O+ separation",
        &auto.iter().map(|d| sq(d[0] - d[1])).collect::<Vec<_>>(),
        sep("auto_ionisation/°OH-H3Op"),
    );

    // Dissociative attachment: H2, OH-, OH (2 tests + exact zero)
    let att = sample(
        DisplacementKind::DissociativeAttachment,
        &[Species::H2, Species::OHm, Species::OH],
        6,
    );
    assert!(
        att.iter().all(|d| d[1] == Vec3::ZERO),
        "OH- keeps the event position (own molecule definition)"
    );
    check(
        "Attach H2",
        &att.iter().map(|d| sq(d[0])).collect::<Vec<_>>(),
        ms("dissociative_attachment", "H2"),
    );
    check(
        "Attach OH",
        &att.iter().map(|d| sq(d[2])).collect::<Vec<_>>(),
        ms("dissociative_attachment", "°OH"),
    );

    // isotropy of the hydrogen atom of A1B1: each component carries a third of the mean square (3 tests)
    let h_ms = ms("a1b1_dissociation_decay", "H");
    check(
        "A1B1 H x^2",
        &a1b1.iter().map(|d| d[1].x.powi(2)).collect::<Vec<_>>(),
        h_ms / 3.0,
    );
    check(
        "A1B1 H y^2",
        &a1b1.iter().map(|d| d[1].y.powi(2)).collect::<Vec<_>>(),
        h_ms / 3.0,
    );
    check(
        "A1B1 H z^2",
        &a1b1.iter().map(|d| d[1].z.powi(2)).collect::<Vec<_>>(),
        h_ms / 3.0,
    );
}
