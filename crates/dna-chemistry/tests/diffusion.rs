//! Free 3-D Brownian diffusion and molecule identity at the M7 boundary.

use dna_chemistry::ChemistryError;
use dna_chemistry::diffusion::diffuse;
use dna_chemistry::molecule::{Molecule, MoleculeId, MoleculePair};
use dna_chemistry::{Species, SpeciesInstance};
use dna_core::rng::{RngStreams, Stage};
use dna_core::testing::ScriptedRng;
use dna_core::units::{Diffusivity, Time};
use dna_core::vec3::Vec3;

fn instance(species: Species) -> SpeciesInstance {
    SpeciesInstance {
        species,
        position: Vec3::new(1.0, -2.0, 3.0),
        time: Time::from_ps(4.0),
    }
}

#[test]
fn pair_uses_molecule_identity_and_allows_same_species() {
    let a = Molecule::new(MoleculeId(7), instance(Species::OH));
    let b = Molecule::new(MoleculeId(8), instance(Species::OH));
    let pair = MoleculePair::new(&a, &b).unwrap();
    assert_eq!(pair.first().id(), MoleculeId(7));
    assert_eq!(pair.second().id(), MoleculeId(8));
    assert_eq!(pair.first().instance(), &instance(Species::OH));
    assert_eq!(pair.first().instance(), pair.second().instance());

    let same_id_different_species = Molecule::new(MoleculeId(7), instance(Species::H));
    assert!(matches!(
        MoleculePair::new(&a, &same_id_different_species),
        Err(ChemistryError::DuplicateMoleculeId { id: MoleculeId(7) })
    ));
}

#[test]
fn zero_time_or_diffusivity_preserves_bits_and_rng_state() {
    let position = Vec3::new(-0.0, 1.25, -3.5);
    let mut rng = ScriptedRng::new(&[]);
    for (d, dt) in [(0.0, 10.0), (1.0, 0.0), (0.0, 0.0)] {
        let out = diffuse(
            position,
            Diffusivity::from_nm2_per_ps(d),
            Time::from_ps(dt),
            &mut rng,
        )
        .unwrap();
        assert_eq!(out.x.to_bits(), position.x.to_bits());
        assert_eq!(out.y.to_bits(), position.y.to_bits());
        assert_eq!(out.z.to_bits(), position.z.to_bits());
    }
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn three_axes_use_two_gaussian_pairs_and_four_uniform_draws() {
    let mut rng = ScriptedRng::new(&[0.25, 0.125, 0.5, 0.25]);
    let out = diffuse(
        Vec3::ZERO,
        Diffusivity::from_nm2_per_ps(2.0),
        Time::from_ps(1.0),
        &mut rng,
    )
    .unwrap();
    // sigma = sqrt(2 D dt) = 2; first pair has angle pi/4, second pi/2.
    let r = (-2.0 * 0.25f64.ln()).sqrt();
    assert!((out.x - 2.0 * r / 2.0f64.sqrt()).abs() < 1e-12);
    assert!((out.y - 2.0 * r / 2.0f64.sqrt()).abs() < 1e-12);
    assert!(out.z.abs() < 1e-12);
    assert_eq!(rng.consumed(), 4);
}

#[test]
fn invalid_inputs_fail_before_drawing() {
    let mut rng = ScriptedRng::new(&[]);
    for d in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            diffuse(
                Vec3::ZERO,
                Diffusivity::from_nm2_per_ps(d),
                Time::from_ps(0.0),
                &mut rng
            ),
            Err(ChemistryError::InvalidDiffusionInput {
                field: "diffusivity",
                ..
            })
        ));
    }
    for dt in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            diffuse(
                Vec3::ZERO,
                Diffusivity::from_nm2_per_ps(0.0),
                Time::from_ps(dt),
                &mut rng
            ),
            Err(ChemistryError::InvalidDiffusionInput { field: "time", .. })
        ));
    }
    for position in [
        Vec3::new(f64::NAN, 0.0, 0.0),
        Vec3::new(0.0, f64::INFINITY, 0.0),
        Vec3::new(0.0, 0.0, f64::NEG_INFINITY),
    ] {
        assert!(matches!(
            diffuse(
                position,
                Diffusivity::from_nm2_per_ps(0.0),
                Time::from_ps(0.0),
                &mut rng
            ),
            Err(ChemistryError::InvalidDiffusionInput { .. })
        ));
    }
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn overflowed_scale_and_output_are_typed_errors() {
    let mut rng = ScriptedRng::new(&[0.5, 0.125, 0.5, 0.25]);
    assert!(matches!(
        diffuse(
            Vec3::ZERO,
            Diffusivity::from_nm2_per_ps(f64::MAX),
            Time::from_ps(f64::MAX),
            &mut rng
        ),
        Err(ChemistryError::NonFiniteDiffusionScale { .. })
    ));
    assert_eq!(rng.consumed(), 0);
    assert!(matches!(
        diffuse(
            Vec3::new(f64::MAX, 0.0, 0.0),
            Diffusivity::from_nm2_per_ps(1e308),
            Time::from_ps(1e308),
            &mut rng
        ),
        Err(ChemistryError::NonFiniteDiffusionOutput { axis: "x", .. })
    ));
    assert_eq!(rng.consumed(), 4);
}

#[test]
fn same_seed_preserves_translation() {
    let streams = RngStreams::new(20261023);
    let mut a = streams.stream(15, Stage::Chemistry).unwrap();
    let mut b = streams.stream(15, Stage::Chemistry).unwrap();
    let origin = diffuse(
        Vec3::ZERO,
        Diffusivity::from_nm2_per_ps(0.003),
        Time::from_ps(10.0),
        &mut a,
    )
    .unwrap();
    let shifted = diffuse(
        Vec3::new(100.0, -200.0, 300.0),
        Diffusivity::from_nm2_per_ps(0.003),
        Time::from_ps(10.0),
        &mut b,
    )
    .unwrap();
    assert!((shifted.x - 100.0 - origin.x).abs() < 2e-14);
    assert!((shifted.y + 200.0 - origin.y).abs() < 3e-14);
    assert!((shifted.z - 300.0 - origin.z).abs() < 3e-14);
}

#[test]
fn fixed_seed_samples_have_expected_variance_and_isotropy() {
    const N: usize = 100_000;
    let mut rng = RngStreams::new(20261023)
        .stream(16, Stage::Chemistry)
        .unwrap();
    let mut sums = [0.0; 3];
    let mut squares = [0.0; 3];
    let mut xy = 0.0;
    for _ in 0..N {
        let p = diffuse(
            Vec3::ZERO,
            Diffusivity::from_nm2_per_ps(0.002),
            Time::from_ps(250.0),
            &mut rng,
        )
        .unwrap();
        let axes = [p.x, p.y, p.z];
        for i in 0..3 {
            sums[i] += axes[i];
            squares[i] += axes[i] * axes[i];
        }
        xy += p.x * p.y;
    }
    // Each coordinate has E[delta] = 0 and E[delta^2] = 2 D dt = 1 nm^2.
    for i in 0..3 {
        let mean = sums[i] / N as f64;
        let second_moment = squares[i] / N as f64;
        assert!(mean.abs() < 0.015, "axis {i}: mean {mean}");
        assert!(
            (second_moment - 1.0).abs() < 0.025,
            "axis {i}: second moment {second_moment}"
        );
    }
    assert!((xy / N as f64).abs() < 0.015);
}
