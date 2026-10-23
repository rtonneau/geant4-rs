//! Deterministic checks of the validation-only Geant4 11.4.1 SBS oracle.

use approx::assert_relative_eq;
use dna_chemistry::diffusion::diffuse;
use dna_chemistry::sbs::{adaptive_time_step, step_tdc_pair};
use dna_core::testing::ScriptedRng;
use dna_core::units::{Diffusivity, Length, Time};
use dna_core::vec3::Vec3;

fn d(value: f64) -> Diffusivity {
    Diffusivity::from_nm2_per_ps(value)
}

#[test]
fn encounter_constant_uses_eight_and_reduces_to_32_d_for_hydrogen() {
    // H diffusivity in the option3 species table, in nm²/ps.
    let hydrogen = 0.007;
    let time =
        adaptive_time_step(Length::from_nm(3.0), Length::from_nm(1.0), [d(hydrogen); 2]).unwrap();
    assert_relative_eq!(time.ps(), 4.0 / (32.0 * hydrogen), epsilon = 1e-14);
    let unequal =
        adaptive_time_step(Length::from_nm(4.0), Length::from_nm(1.0), [d(1.0), d(4.0)]).unwrap();
    assert_eq!(unequal.ps(), 9.0 / (8.0 * (1.0 + 4.0 + 4.0)));
}

#[test]
fn initial_overlap_and_equality_are_immediate_without_draws() {
    for distance in [0.0, 0.5, 1.0] {
        let positions = [Vec3::ZERO, Vec3::new(distance, 0.0, 0.0)];
        let mut rng = ScriptedRng::new(&[]);
        let step = step_tdc_pair(
            positions,
            [d(1.0); 2],
            Length::from_nm(1.0),
            Time::from_ps(2.0),
            &mut rng,
        )
        .unwrap();
        assert!(step.reacted);
        assert!(!step.capped);
        assert_eq!(step.elapsed.ps(), 0.0);
        assert_eq!(step.positions, positions);
        assert_eq!(rng.consumed(), 0);
    }
}

#[test]
fn uncapped_step_diffuses_independently_and_only_checks_endpoint_contact() {
    let positions = [Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0)];
    // Opposing displacements toward each other produce endpoint contact.
    let script = [0.05, 1e-15, 0.5, 0.25, 0.05, 0.5, 0.5, 0.25];
    let mut expected_rng = ScriptedRng::new(&script);
    let expected = positions
        .map(|p| diffuse(p, d(1.0), Time::from_ps(1.0 / 32.0), &mut expected_rng).unwrap());
    let mut rng = ScriptedRng::new(&script);
    let step = step_tdc_pair(
        positions,
        [d(1.0); 2],
        Length::from_nm(1.0),
        Time::from_ps(1.0),
        &mut rng,
    )
    .unwrap();
    assert_eq!(step.positions, expected);
    assert_eq!(step.elapsed.ps(), 1.0 / 32.0);
    assert!(step.reacted);
    assert!(!step.capped);
    assert_eq!(rng.consumed(), 8);

    // Equal displacements preserve separation. Even a tiny next draw must not
    // trigger a bridge check when the adaptive step is not capped.
    let mut rng = ScriptedRng::new(&[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 1e-15]);
    let step = step_tdc_pair(
        positions,
        [d(1.0); 2],
        Length::from_nm(1.0),
        Time::from_ps(1.0 / 32.0),
        &mut rng,
    )
    .unwrap();
    assert!(!step.reacted);
    assert!(!step.capped); // Equality with the boundary did not shorten the step.
    assert_eq!(rng.consumed(), 8);
}

#[test]
fn capped_bridge_accepts_and_rejects_using_source_post_step_distance() {
    // Equal displacements preserve r = 2, but the source's resumed loop
    // counts x² twice, so its bridge post-distance is sqrt(8), not 2.
    // dt = 0.03 < 1/32; p_source = exp(-(sqrt(8)-1)/0.06) ≈ 5.83e-14.
    for (draw, reacted) in [(1e-14, true), (1e-10, false)] {
        let mut rng = ScriptedRng::new(&[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, draw]);
        let step = step_tdc_pair(
            [Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0)],
            [d(1.0); 2],
            Length::from_nm(1.0),
            Time::from_ps(0.03),
            &mut rng,
        )
        .unwrap();
        assert!(step.capped);
        assert_eq!(step.elapsed.ps(), 0.03);
        assert_eq!(step.reacted, reacted);
        assert_eq!(rng.consumed(), 9);
    }
}

#[test]
fn capped_endpoint_contact_skips_bridge_draw() {
    let mut rng = ScriptedRng::new(&[0.01, 1e-15, 0.5, 0.25, 0.01, 0.5, 0.5, 0.25]);
    let step = step_tdc_pair(
        [Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0)],
        [d(1.0); 2],
        Length::from_nm(1.0),
        Time::from_ps(0.03),
        &mut rng,
    )
    .unwrap();
    assert!(step.reacted);
    assert!(step.capped);
    assert_eq!(rng.consumed(), 8);
}

#[test]
fn exact_endpoint_equality_is_contact_without_a_bridge_draw() {
    let positions = [Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0)];
    let script = [1e-7, 1e-15, 0.5, 0.25, 1e-7, 0.5, 0.5, 0.25];
    let dt = Time::from_ps(0.001);
    let mut reference_rng = ScriptedRng::new(&script);
    let endpoints = positions.map(|p| diffuse(p, d(1.0), dt, &mut reference_rng).unwrap());
    let delta = endpoints[0] - endpoints[1];
    let radius = Length::from_nm(delta.x.hypot(delta.y).hypot(delta.z));
    assert!(
        adaptive_time_step(Length::from_nm(2.0), radius, [d(1.0); 2])
            .unwrap()
            .ps()
            > dt.ps()
    );
    let mut rng = ScriptedRng::new(&script);
    let step = step_tdc_pair(positions, [d(1.0); 2], radius, dt, &mut rng).unwrap();
    assert_eq!(step.positions, endpoints);
    assert!(step.reacted);
    assert!(step.capped);
    assert_eq!(rng.consumed(), 8);
}

#[test]
fn nonrepresentable_adaptive_steps_are_errors_without_draws() {
    for (separation, radius, diffusion) in [
        (1e200, 1.0, 1.0),
        (1e-200, 0.5e-200, 1.0),
        (2.0, 1.0, f64::MAX / 4.0),
    ] {
        let mut rng = ScriptedRng::new(&[]);
        assert!(
            step_tdc_pair(
                [Vec3::ZERO, Vec3::new(separation, 0.0, 0.0)],
                [d(diffusion); 2],
                Length::from_nm(radius),
                Time::from_ps(1.0),
                &mut rng,
            )
            .is_err()
        );
        assert_eq!(rng.consumed(), 0);
    }
}

#[test]
fn zero_observation_time_is_a_noop_for_a_separated_pair() {
    let positions = [Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0)];
    let mut rng = ScriptedRng::new(&[]);
    let step = step_tdc_pair(
        positions,
        [d(1.0); 2],
        Length::from_nm(1.0),
        Time::from_ps(0.0),
        &mut rng,
    )
    .unwrap();
    assert!(!step.reacted);
    assert_eq!(step.positions, positions);
    assert_eq!(step.elapsed.ps(), 0.0);
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn stationary_partner_consumes_no_diffusion_draws() {
    let mut rng = ScriptedRng::new(&[]);
    let positions = [Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0)];
    let step = step_tdc_pair(
        positions,
        [d(0.0), d(1.0)],
        Length::from_nm(1.0),
        Time::from_ps(1.0),
        &mut rng,
    )
    .unwrap();
    assert_eq!(step.positions[0], positions[0]);
    assert_eq!(step.elapsed.ps(), 1.0 / 8.0);
    assert_eq!(rng.consumed(), 4);
}

#[test]
fn invalid_inputs_are_rejected_before_any_rng_draw() {
    let mut rng = ScriptedRng::new(&[]);
    let positions = [Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0)];
    for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..2 {
            let mut diffusivities = [d(1.0); 2];
            diffusivities[index] = d(invalid);
            assert!(
                step_tdc_pair(
                    positions,
                    diffusivities,
                    Length::from_nm(1.0),
                    Time::from_ps(1.0),
                    &mut rng
                )
                .is_err()
            );
        }
        assert!(
            step_tdc_pair(
                positions,
                [d(1.0); 2],
                Length::from_nm(invalid),
                Time::from_ps(1.0),
                &mut rng
            )
            .is_err()
        );
        assert!(
            step_tdc_pair(
                positions,
                [d(1.0); 2],
                Length::from_nm(1.0),
                Time::from_ps(invalid),
                &mut rng
            )
            .is_err()
        );
        assert!(
            adaptive_time_step(Length::from_nm(invalid), Length::from_nm(1.0), [d(1.0); 2])
                .is_err()
        );
    }
    assert!(
        step_tdc_pair(
            positions,
            [d(0.0); 2],
            Length::from_nm(1.0),
            Time::from_ps(1.0),
            &mut rng
        )
        .is_err()
    );
    assert!(
        step_tdc_pair(
            positions,
            [d(1.0); 2],
            Length::from_nm(0.0),
            Time::from_ps(1.0),
            &mut rng
        )
        .is_err()
    );
    for bad in [
        Vec3::new(f64::NAN, 0.0, 0.0),
        Vec3::new(0.0, f64::INFINITY, 0.0),
        Vec3::new(0.0, 0.0, f64::NEG_INFINITY),
    ] {
        for pair in [[bad, positions[1]], [positions[0], bad]] {
            assert!(
                step_tdc_pair(
                    pair,
                    [d(1.0); 2],
                    Length::from_nm(1.0),
                    Time::from_ps(1.0),
                    &mut rng
                )
                .is_err()
            );
        }
    }
    assert_eq!(rng.consumed(), 0);
}
