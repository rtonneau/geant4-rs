//! Placement checks against Geant4 11.4.1 G4DNAIRT.cc:393-495.
//! Deterministic comparisons use 3e-12 relative / 1e-14 nm absolute tolerance,
//! fixed before execution. The independent oracle evaluates the literal source
//! in mm/ns, including its dimensionally inconsistent Gaussian width.

use approx::assert_relative_eq;
use g4_chemistry::molecule::{Molecule, MoleculeId, MoleculePair};
use g4_chemistry::placement::{
    conditional_parent_positions, place_pair_reaction, place_unary_reaction,
};
use g4_chemistry::reaction_network::ReactionNetwork;
use g4_chemistry::reactions::{DEFAULT_REACTIONS_TOML, ReactionParticipant, ReactionTable};
use g4_chemistry::species::SpeciesTable;
use g4_chemistry::{ChemistryError, Species, SpeciesInstance};
use g4_core::sampling::uniform;
use g4_core::testing::ScriptedRng;
use g4_core::units::{Diffusivity, Length, Time};
use g4_core::vec3::Vec3;

fn network() -> ReactionNetwork {
    ReactionTable::option3()
        .unwrap()
        .compile(&SpeciesTable::option3().unwrap())
        .unwrap()
}

fn molecule(id: u64, species: Species, position: Vec3) -> Molecule {
    Molecule::new(
        MoleculeId(id),
        SpeciesInstance {
            species,
            position,
            time: Time::from_ps(1.0),
        },
    )
}

fn parents(network: &ReactionNetwork, id: u16, positions: [Vec3; 2]) -> [Molecule; 2] {
    let reactants = network.get(id).unwrap().pair().unwrap().reactants();
    [
        molecule(1, reactants[0], positions[0]),
        molecule(2, reactants[1], positions[1]),
    ]
}

fn close(actual: Vec3, expected: Vec3) {
    for (a, b) in [
        (actual.x, expected.x),
        (actual.y, expected.y),
        (actual.z, expected.z),
    ] {
        assert_relative_eq!(a, b, epsilon = 1e-14, max_relative = 3e-12);
    }
}

fn conditional(
    positions: [Vec3; 2],
    diffusion: [f64; 2],
    radius: f64,
    dt: f64,
    rng: &mut ScriptedRng,
) -> Result<[Vec3; 2], ChemistryError> {
    conditional_parent_positions(
        2,
        positions,
        diffusion.map(Diffusivity::from_nm2_per_ps),
        Length::from_nm(radius),
        Time::from_ps(dt),
        rng,
    )
}

// Literal, independent mm/ns oracle. The normal draws are the project's
// documented Box-Muller law, evaluated directly rather than calling its helper.
fn source_oracle(
    positions: [Vec3; 2],
    diffusion: [f64; 2],
    radius: f64,
    dt: f64,
    script: &[f64],
) -> [Vec3; 2] {
    let [a, mut b] = positions.map(|v| v * 1e-6);
    if a == b {
        b.z += 1e-9;
    }
    let delta = a - b;
    let r0 = delta.norm();
    let [d1, d2] = diffusion.map(|d| d * 1e-9);
    let time = dt * 1e-3;
    let radius = radius * 1e-6;
    let s12 = 2.0 * d1 * time;
    let s22 = 2.0 * d2 * time;
    let alpha = radius * r0 / (2.0 * (d1 + d2) * time);
    let sigma = s12 + s22 * s22 / s12;
    let mut rng = ScriptedRng::new(script);
    let u: Vec<_> = (0..6).map(|_| uniform(&mut rng)).collect();
    let x = (-2.0 * u[0].ln()).sqrt() * (std::f64::consts::TAU * u[1]).cos();
    let y = (-2.0 * u[0].ln()).sqrt() * (std::f64::consts::TAU * u[1]).sin();
    let z = (-2.0 * u[2].ln()).sqrt() * (std::f64::consts::TAU * u[3]).cos();
    let s2 = a + b * (s12 / s22) + Vec3::new(x, y, z) * sigma;
    let phi = if delta.x == 0.0 && delta.y == 0.0 {
        0.0
    } else {
        std::f64::consts::TAU * u[4]
    };
    let theta = (1.0 + (1.0 - u[5] * (1.0 - (-2.0 * alpha).exp())).ln() / alpha).acos();
    let s1 = Vec3::new(
        theta.sin() * phi.cos(),
        theta.sin() * phi.sin(),
        theta.cos(),
    ) * radius;
    [
        (s1 * d1 + s2 * d2) * (1e6 / (d1 + d2)),
        (s2 - s1) * (1e6 * d2 / (d1 + d2)),
    ]
}

#[test]
fn equal_time_places_zero_one_two_three_products_in_source_order() {
    let network = network();
    let positions = [Vec3::new(1.0, 2.0, 3.0), Vec3::new(-4.0, 5.0, 7.0)];
    for (id, count) in [(9, 0), (3, 1), (2, 2), (8, 3)] {
        let reaction = network.get(id).unwrap();
        let parents = parents(&network, id, positions);
        let mut rng = ScriptedRng::new(&[]);
        let result = place_pair_reaction(
            MoleculePair::new(&parents[0], &parents[1]).unwrap(),
            reaction,
            Time::from_ps(0.0),
            &mut rng,
        )
        .unwrap();
        assert_eq!(result.parent_positions, positions);
        assert_eq!(result.products.len(), count);
        assert_eq!(rng.consumed(), 0);
        let [a, b] = reaction
            .reactant_diffusivities()
            .map(|d| d.nm2_per_ps().sqrt());
        let site = (positions[0] * b + positions[1] * a) * (1.0 / (a + b));
        let expected = match count {
            0 => vec![],
            1 => vec![site],
            2 => positions.to_vec(),
            3 => vec![site, positions[0], positions[1]],
            _ => unreachable!(),
        };
        for ((product, position), source) in result
            .products
            .iter()
            .zip(expected)
            .zip(&reaction.source().products)
        {
            assert_eq!(ReactionParticipant::Mobile(product.species), *source);
            close(product.position, position);
        }
    }
}

#[test]
fn positive_delay_matches_literal_native_unit_formula_for_unequal_diffusion() {
    let positions = [Vec3::new(1.0, 2.0, 3.0), Vec3::new(-4.0, 5.0, 7.0)];
    let script = [0.27, 0.41, 0.73, 0.19, 0.37, 0.61];
    for diffusion in [[0.0049, 0.007], [0.007, 0.0049], [0.007, 0.007]] {
        let expected = source_oracle(positions, diffusion, 0.7, 80.0, &script);
        let mut rng = ScriptedRng::new(&script);
        let actual = conditional(positions, diffusion, 0.7, 80.0, &mut rng).unwrap();
        for i in 0..2 {
            close(actual[i], expected[i]);
        }
        assert_eq!(rng.consumed(), 6);
        assert_relative_eq!((actual[0] - actual[1]).norm(), 0.7, epsilon = 1e-14);
    }
}

#[test]
fn gaussian_width_retains_native_mm_scale_and_is_not_square_rooted() {
    let diffusion = [0.5, 1.0];
    let positions = [Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO];
    let script = [0.3, 0.125, 0.4, 0.2, 0.6, 0.7];
    let actual = conditional(
        positions,
        diffusion,
        1.0,
        2.0,
        &mut ScriptedRng::new(&script),
    )
    .unwrap();
    // s12=2 nm^2, s22=4 nm^2 => literal sigma = 10e-6 nm.
    // S2 = A + (D1/D2) B, so the parent combination cancels S1.
    let recovered_s2 = actual[0] + actual[1] * 0.5;
    let mut draws = ScriptedRng::new(&script);
    let u1 = uniform(&mut draws);
    let u2 = uniform(&mut draws);
    let expected_noise = 10e-6 * (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos();
    assert_relative_eq!(recovered_s2.x - 1.0, expected_noise, epsilon = 1e-14);
}

#[test]
fn coincident_and_axial_parents_retain_clhep_zero_azimuth() {
    let script = [0.27, 0.41, 0.73, 0.19, 0.37, 0.61];
    for positions in [[Vec3::ZERO; 2], [Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0)]] {
        let expected = source_oracle(positions, [0.007, 0.007], 0.7, 0.1, &script);
        let mut rng = ScriptedRng::new(&script);
        let actual = conditional(positions, [0.007, 0.007], 0.7, 0.1, &mut rng).unwrap();
        for i in 0..2 {
            close(actual[i], expected[i]);
        }
        assert_eq!(actual[0].y - actual[1].y, 0.0);
        assert_eq!(rng.consumed(), 6);
    }
}

#[test]
fn coincident_equal_time_parents_get_source_z_offset_without_rng() {
    let position = Vec3::new(1.0, -2.0, 3.0);
    let mut rng = ScriptedRng::new(&[]);
    let result = conditional([position; 2], [0.0049, 0.007], 0.7, 0.0, &mut rng).unwrap();
    assert_eq!(result, [position, position + Vec3::new(0.0, 0.0, 0.001)]);
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn one_immobile_parent_uses_its_site_without_brownian_sampling() {
    for positions in [
        [Vec3::new(1.0, 2.0, 3.0), Vec3::new(-2.0, 3.0, 4.0)],
        [Vec3::ZERO; 2],
    ] {
        for immobile in [0, 1] {
            let mut diffusion = [0.007; 2];
            diffusion[immobile] = 0.0;
            let mut rng = ScriptedRng::new(&[]);
            let result = conditional(positions, diffusion, 0.0, 100.0, &mut rng).unwrap();
            let site = positions[immobile]
                + if immobile == 1 && positions[0] == positions[1] {
                    Vec3::new(0.0, 0.0, 0.001)
                } else {
                    Vec3::ZERO
                };
            assert_eq!(result, [site; 2]);
            assert_eq!(rng.consumed(), 0);
            let equal_time = conditional(positions, diffusion, 0.0, 0.0, &mut rng).unwrap();
            assert_eq!(equal_time[0], positions[0]);
            assert_eq!(
                equal_time[1],
                positions[1]
                    + if positions[0] == positions[1] {
                        Vec3::new(0.0, 0.0, 0.001)
                    } else {
                        Vec3::ZERO
                    }
            );
        }
    }
}

#[test]
fn all_72_channels_preserve_products_in_both_elapsed_time_branches() {
    let network = network();
    let origin = Vec3::new(1.0, 2.0, 3.0);
    for reaction in network.iter() {
        for dt in [0.0, 100.0] {
            let mut rng = ScriptedRng::new(&[0.27, 0.41, 0.73, 0.19, 0.37, 0.61]);
            let result = if let Some(unary) = reaction.unary() {
                let parent = molecule(1, unary.mobile(), origin);
                let result =
                    place_unary_reaction(&parent, reaction, Time::from_ps(dt), &mut rng).unwrap();
                let offset_site = origin + Vec3::new(0.0, 0.0, 0.001);
                assert_eq!(
                    result.parent_positions,
                    [if dt == 0.0 { origin } else { offset_site }, offset_site]
                );
                if dt > 0.0 || result.products.len() == 1 {
                    assert!(result.products.iter().all(|p| p.position == offset_site));
                }
                assert_eq!(rng.consumed(), 0);
                result
            } else {
                let parents = parents(&network, reaction.source().id, [origin, Vec3::ZERO]);
                let result = place_pair_reaction(
                    MoleculePair::new(&parents[0], &parents[1]).unwrap(),
                    reaction,
                    Time::from_ps(dt),
                    &mut rng,
                )
                .unwrap();
                assert_eq!(rng.consumed(), if dt == 0.0 { 0 } else { 6 });
                result
            };
            assert_eq!(
                result
                    .products
                    .iter()
                    .map(|p| ReactionParticipant::Mobile(p.species))
                    .collect::<Vec<_>>(),
                reaction.source().products
            );
            match result.products.as_slice() {
                [_, second, third] => {
                    assert_eq!(second.position, result.parent_positions[0]);
                    assert_eq!(third.position, result.parent_positions[1]);
                }
                [first, second] => {
                    assert_eq!(first.position, result.parent_positions[0]);
                    assert_eq!(second.position, result.parent_positions[1]);
                }
                _ => {}
            }
        }
    }
}

#[test]
fn zero_product_unary_channels_return_no_positions_or_rng_draws_for_products() {
    let network = network();
    for id in [63, 64] {
        let reaction = network.get(id).unwrap();
        let parent = molecule(1, reaction.unary().unwrap().mobile(), Vec3::ZERO);
        let mut rng = ScriptedRng::new(&[]);
        let result =
            place_unary_reaction(&parent, reaction, Time::from_ps(10.0), &mut rng).unwrap();
        assert!(result.products.is_empty());
        assert_eq!(result.parent_positions, [Vec3::new(0.0, 0.0, 0.001); 2]);
        assert_eq!(rng.consumed(), 0);
    }
}

#[test]
fn placement_rejects_wrong_species_order_kind_and_disabled_channels() {
    let network = network();
    let parents = parents(&network, 2, [Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)]);
    let mut rng = ScriptedRng::new(&[]);
    assert!(matches!(
        place_pair_reaction(
            MoleculePair::new(&parents[1], &parents[0]).unwrap(),
            network.get(2).unwrap(),
            Time::from_ps(0.0),
            &mut rng
        ),
        Err(ChemistryError::ReactionSpeciesMismatch { reaction_id: 2 })
    ));
    assert!(matches!(
        place_pair_reaction(
            MoleculePair::new(&parents[0], &parents[1]).unwrap(),
            network.get(53).unwrap(),
            Time::from_ps(0.0),
            &mut rng
        ),
        Err(ChemistryError::UnsupportedReactionOrder { reaction_id: 53 })
    ));
    assert!(matches!(
        place_unary_reaction(
            &parents[0],
            network.get(2).unwrap(),
            Time::from_ps(0.0),
            &mut rng
        ),
        Err(ChemistryError::UnsupportedReactionOrder { reaction_id: 2 })
    ));
    assert!(matches!(
        place_unary_reaction(
            &parents[0],
            network.get(53).unwrap(),
            Time::from_ps(0.0),
            &mut rng
        ),
        Err(ChemistryError::ReactionSpeciesMismatch { reaction_id: 53 })
    ));
    let disabled = ReactionTable::from_toml(
        &DEFAULT_REACTIONS_TOML.replace("rate_per_s = 2.66e3", "rate_per_s = 0.0"),
    )
    .unwrap()
    .compile(&SpeciesTable::option3().unwrap())
    .unwrap();
    let reaction = disabled.get(53).unwrap();
    let parent = molecule(1, reaction.unary().unwrap().mobile(), Vec3::ZERO);
    assert_eq!(
        place_unary_reaction(&parent, reaction, Time::from_ps(1.0), &mut rng),
        Err(ChemistryError::DisabledReactionPlacement { reaction_id: 53 })
    );
    assert_eq!(rng.consumed(), 0);
}

#[test]
fn invalid_inputs_and_ambiguous_or_unrepresentable_branches_are_typed_errors() {
    let positions = [Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)];
    for (diffusion, radius, dt) in [
        ([0.0, 0.0], 1.0, 0.0),
        ([1.0, 1.0], 0.0, 1.0),
        ([1.0, 1.0], 1e-300, 1e100),
        ([1.0, 1.0], 1.0, 1e308),
        ([1e-300, 1e-300], 1.0, 1e-100),
        ([1e-200, 1e100], 1.0, 1.0),
    ] {
        assert!(
            matches!(
                conditional(positions, diffusion, radius, dt, &mut ScriptedRng::new(&[])),
                Err(ChemistryError::PlacementDomain { .. })
            ),
            "{diffusion:?} {radius} {dt}"
        );
    }
    for (diffusion, radius, dt) in [
        ([-1.0, 1.0], 1.0, 1.0),
        ([f64::NAN, 1.0], 1.0, 1.0),
        ([1.0, 1.0], -1.0, 1.0),
        ([1.0, 1.0], 1.0, -1.0),
        ([1.0, 1.0], 1.0, f64::INFINITY),
    ] {
        assert!(matches!(
            conditional(positions, diffusion, radius, dt, &mut ScriptedRng::new(&[])),
            Err(ChemistryError::InvalidReactionParameter { .. })
        ));
    }
    for positions in [
        [Vec3::new(f64::NAN, 0.0, 0.0), Vec3::ZERO],
        [Vec3::new(0.0, 0.0, 1e100); 2],
        [
            Vec3::new(f64::MAX, 0.0, 0.0),
            Vec3::new(-f64::MAX, 0.0, 0.0),
        ],
    ] {
        assert!(matches!(
            conditional(positions, [1.0, 1.0], 1.0, 1.0, &mut ScriptedRng::new(&[])),
            Err(ChemistryError::PlacementDomain { .. })
        ));
    }
}
