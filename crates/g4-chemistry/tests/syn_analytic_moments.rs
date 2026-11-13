//! Separate analytic marginal gate; no Geant4 data or mapping fixture is used.
mod syn_support;
use g4_chemistry::{
    SpeciesInstance,
    diffusion::diffuse,
    molecule::{Molecule, MoleculeId, MoleculePair},
    species::SpeciesTable,
    syn::place_pair_diagnostic,
};
use g4_core::{
    rng::{RngStreams, Stage},
    units::{Diffusivity, Time},
    vec3::Vec3,
};
use syn_support::*;
const ANALYTIC_DRAWS: usize = 333;
const FAMILY: usize = PLACEMENT_ENDPOINTS + DIFFUSION_ENDPOINTS;
const SEEDS: [u64; 2] = [20261109081, 20261109082];
fn assess(label: &str, batches: &[Vec<f64>], expected: &[f64], scales: &[f64]) -> usize {
    let mut unresolved = 0;
    for endpoint in 0..expected.len() {
        let values: Vec<_> = batches.iter().map(|b| b[endpoint]).collect();
        let (lo, hi, mean, variance) = interval(
            &values,
            None,
            expected[endpoint],
            0.001 / (3.0 * FAMILY as f64),
        );
        // Numerical oracle error is bounded by the separately checked refinements.
        let verdict = if scales[endpoint] == 0.0 {
            if values.iter().all(|v| {
                (*v - expected[endpoint]).abs() <= 1e-12 + 1e-12 * expected[endpoint].abs()
            }) {
                "PASS"
            } else {
                "FAIL"
            }
        } else {
            syn_support::verdict(
                lo - 1e-8 * scales[endpoint],
                hi + 1e-8 * scales[endpoint],
                0.01 * scales[endpoint],
            )
        };
        println!(
            "{label} endpoint={endpoint} batches={} mean={mean} batch_variance={variance} oracle={} scale={} interval=[{lo},{hi}] {verdict}",
            batches.len(),
            expected[endpoint],
            scales[endpoint]
        );
        unresolved += usize::from(verdict != "PASS");
    }
    unresolved
}
#[test]
fn analytic_manifest_counts_all_endpoints() {
    let network = network();
    let mut placement = 0;
    for id in PLACEMENT_IDS {
        let reaction = network.get(id).unwrap();
        let dimension = if reaction.source().products.is_empty() {
            6
        } else {
            9
        };
        placement += 2 * 2 * 3 * 3 * (dimension + dimension * (dimension + 1) / 2 + 2);
    }
    assert_eq!(placement, PLACEMENT_ENDPOINTS);
    assert_eq!(
        2 * (SpeciesTable::option3().unwrap().iter().count() + 1) * 4 * 9,
        DIFFUSION_ENDPOINTS
    );
    for n in [300, 1000, 3000] {
        assert!(n >= 30);
        assert!(n * ANALYTIC_DRAWS <= 1000000);
    }
}
#[test]
#[ignore = "SYN-2 independent analytic marginals; frozen manifest in README"]
fn syn_marginals_match_independent_analytic_moments() {
    let histories: usize =
        std::env::var("SYN_MOMENT_HISTORIES").map_or(300, |v| v.parse().unwrap());
    assert!([300, 1000, 3000].contains(&histories));
    let network = network();
    let species = SpeciesTable::option3().unwrap();
    let mut unresolved = 0;
    let mut settings = 0;
    for seed in SEEDS {
        let streams = RngStreams::new(seed);
        for (species_index, d) in std::iter::once(Diffusivity::from_nm2_per_ps(0.0))
            .chain(species.iter().map(|s| s.diffusivity))
            .enumerate()
        {
            for dt in [0.0, 1.0, 1000.0, 1000000.0] {
                let variance = 2.0 * d.nm2_per_ps() * dt;
                let expected = vec![0.0, 0.0, 0.0, variance, 0.0, 0.0, variance, 0.0, variance];
                let scales = vec![
                    variance.sqrt(),
                    variance.sqrt(),
                    variance.sqrt(),
                    variance,
                    variance,
                    variance,
                    variance,
                    variance,
                    variance,
                ];
                let mut batches = vec![];
                for batch in 0..histories {
                    let mut moments = Moments::new(3);
                    for history in
                        batch * (histories / histories)..(batch + 1) * (histories / histories)
                    {
                        let mut rng = streams.stream(history as u64, Stage::Chemistry).unwrap();
                        for _ in 0..ANALYTIC_DRAWS {
                            let p = diffuse(Vec3::ZERO, d, Time::from_ps(dt), &mut rng).unwrap();
                            moments.push(&flat(p));
                        }
                    }
                    batches.push(moments.endpoints());
                }
                unresolved += assess(
                    &format!(
                        "diffusion seed={seed} species={species_index} dt_ps={dt} histories={histories}"
                    ),
                    &batches,
                    &expected,
                    &scales,
                );
                settings += 1;
            }
        }
        for id in PLACEMENT_IDS {
            for ratio in RATIOS {
                for dt in DURATIONS {
                    for orientation in 0..3 {
                        let reaction = network.get(id).unwrap();
                        let p = reaction.pair().unwrap();
                        let points = positions(p.effective_radius().nm(), ratio, orientation);
                        let molecules: [Molecule; 2] = std::array::from_fn(|i| {
                            Molecule::new(
                                MoleculeId(i as u64),
                                SpeciesInstance {
                                    species: p.reactants()[i],
                                    position: points[i],
                                    time: Time::from_ps(0.0),
                                },
                            )
                        });
                        let pair = MoleculePair::new(&molecules[0], &molecules[1]).unwrap();
                        let oracle = expected(reaction, ratio, dt, orientation, 8192);
                        let (means, scales) = oracle_endpoints(&oracle);
                        let mut batches = vec![];
                        for batch in 0..histories {
                            let mut moments = Moments::new(oracle.mean.len());
                            for history in batch * (histories / histories)
                                ..(batch + 1) * (histories / histories)
                            {
                                let mut rng =
                                    streams.stream(history as u64, Stage::Chemistry).unwrap();
                                for _ in 0..ANALYTIC_DRAWS {
                                    let out = place_pair_diagnostic(
                                        pair,
                                        reaction,
                                        Time::from_ps(dt),
                                        &mut rng,
                                    )
                                    .unwrap();
                                    let distance =
                                        (out.parent_positions[0] - out.parent_positions[1]).norm();
                                    let target = p.effective_radius().nm()
                                        * if dt == 0.0 { ratio } else { 1.0 };
                                    assert!((distance - target).abs() <= 1e-12 + 1e-12 * target);
                                    assert_eq!(
                                        out.products.len(),
                                        reaction.source().products.len()
                                    );
                                    let mut values = flat(out.parent_positions[0]).to_vec();
                                    values.extend(flat(out.parent_positions[1]));
                                    if let Some(product) = out.products.first() {
                                        values.extend(flat(product.position));
                                        assert!(
                                            out.products
                                                .iter()
                                                .all(|v| v.position == product.position)
                                        );
                                    }
                                    moments.push(&values);
                                }
                            }
                            batches.push(moments.endpoints());
                        }
                        unresolved += assess(
                            &format!(
                                "placement seed={seed} channel={id} ratio={ratio} dt_ps={dt} orientation={orientation} histories={histories}"
                            ),
                            &batches,
                            &means,
                            &scales,
                        );
                        println!(
                            "placement seed={seed} channel={id} ratio={ratio} dt_ps={dt} orientation={orientation} distance/product-count identities PASS"
                        );
                        settings += 1;
                    }
                }
            }
        }
    }
    assert_eq!(settings, 236);
    assert_eq!(
        unresolved, 0,
        "SYN-2 analytic moments have unresolved endpoints"
    );
}
