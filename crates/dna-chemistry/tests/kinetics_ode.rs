use std::error::Error;

use dna_chemistry::reactions::{ObservedRate, ReactionParticipant, ReactionTable};
use dna_core::rng::{RngStreams, Stage};
use dna_core::species::Species;
use rsoda::{ClosureSystem, Lsoda, OptionsBuilder, Tolerance};

const AVOGADRO_PER_MOL: f64 = 6.022_140_76e23;
const INITIAL_MOLECULES: u32 = 10_000;
const INITIAL_CONCENTRATION_MOLAR: f64 = 1.0e-6;
const HISTORIES: u64 = 512;
const MASTER_SEED: u64 = 2_026_102_303;
const END_TIME_S: f64 = 360.0e-6;

#[derive(Debug)]
struct Comparison {
    relative_error: f64,
}

#[derive(Debug, Clone, Copy)]
struct Population {
    electron: u32,
    hydrogen: u32,
    hydroxide: u32,
    hydrogen_molecule: u32,
}

#[test]
fn option3_reaction_2_ssa_agrees_with_lsoda_at_360_microseconds() -> Result<(), Box<dyn Error>> {
    let comparison = run_registered_oracle()?;

    assert!(comparison.relative_error <= 0.01);
    Ok(())
}

fn run_registered_oracle() -> Result<Comparison, Box<dyn Error>> {
    let reaction = ReactionTable::option3()?
        .get(2)
        .expect("validated option3 table must contain reaction 2")
        .clone();
    assert_eq!(
        reaction.reactants,
        [
            ReactionParticipant::Mobile(Species::e_aq),
            ReactionParticipant::Mobile(Species::H),
        ]
    );
    assert_eq!(
        reaction.products,
        [
            ReactionParticipant::Mobile(Species::OHm),
            ReactionParticipant::Mobile(Species::H2),
        ]
    );
    let ObservedRate::Bimolecular { per_molar_s } = reaction.rate else {
        panic!("option3 reaction 2 must have a bimolecular rate")
    };
    assert_eq!(per_molar_s, 2.50e10);

    // The volume is fixed by N = c N_A V. Dividing the macroscopic rate by
    // N_A V converts M^-1 s^-1 to the per-pair stochastic rate in s^-1.
    let volume_l = f64::from(INITIAL_MOLECULES) / (INITIAL_CONCENTRATION_MOLAR * AVOGADRO_PER_MOL);
    let stochastic_rate_per_pair_s = per_molar_s / (AVOGADRO_PER_MOL * volume_l);

    let streams = RngStreams::new(MASTER_SEED);
    let mut sums = [0_u64; 4];
    for history_id in 0..HISTORIES {
        let mut rng = streams.stream(history_id, Stage::Chemistry)?;
        let population = run_ssa(stochastic_rate_per_pair_s, &mut rng);
        let reactions = INITIAL_MOLECULES - population.electron;
        assert_eq!(population.electron, population.hydrogen);
        assert_eq!(population.hydroxide, population.hydrogen_molecule);
        assert_eq!(
            population.electron + population.hydroxide,
            INITIAL_MOLECULES
        );
        assert_eq!(
            population.hydrogen + population.hydrogen_molecule,
            INITIAL_MOLECULES
        );
        assert_eq!(population.hydroxide, reactions);
        for (sum, value) in sums.iter_mut().zip([
            population.electron,
            population.hydrogen,
            population.hydroxide,
            population.hydrogen_molecule,
        ]) {
            *sum += u64::from(value);
        }
    }

    let ode = solve_mass_action(stochastic_rate_per_pair_s)?;
    let mut relative_error = 0.0_f64;
    for (sum, expected) in sums.into_iter().zip(ode) {
        let mean = sum as f64 / HISTORIES as f64;
        relative_error = relative_error.max((mean - expected).abs() / expected);
    }
    println!(
        "reaction 2 at 360 us: SSA means = {:?}, LSODA = {ode:?}, max relative error = {:.6}",
        sums.map(|sum| sum as f64 / HISTORIES as f64),
        relative_error
    );
    Ok(Comparison { relative_error })
}

fn run_ssa(rate_per_pair_s: f64, rng: &mut dna_core::rng::HistoryRng) -> Population {
    let mut population = Population {
        electron: INITIAL_MOLECULES,
        hydrogen: INITIAL_MOLECULES,
        hydroxide: 0,
        hydrogen_molecule: 0,
    };
    let mut time = 0.0;
    while population.electron > 0 && population.hydrogen > 0 {
        let propensity =
            rate_per_pair_s * f64::from(population.electron) * f64::from(population.hydrogen);
        let waiting_time = -rng.uniform_open01().ln() / propensity;
        if time + waiting_time > END_TIME_S {
            break;
        }
        time += waiting_time;
        population.electron -= 1;
        population.hydrogen -= 1;
        population.hydroxide += 1;
        population.hydrogen_molecule += 1;
    }
    population
}

fn solve_mass_action(rate_per_pair_s: f64) -> Result<[f64; 4], rsoda::LsodaError> {
    let system = ClosureSystem::new(move |_time, population, derivative| {
        let reaction_rate = rate_per_pair_s * population[0] * population[1];
        derivative[0] = -reaction_rate;
        derivative[1] = -reaction_rate;
        derivative[2] = reaction_rate;
        derivative[3] = reaction_rate;
        Ok(())
    });
    let options = OptionsBuilder::new(4)
        .tolerance(Tolerance::scalar(1.0e-10, 1.0e-8))
        .build(&system)?;
    let initial = [
        f64::from(INITIAL_MOLECULES),
        f64::from(INITIAL_MOLECULES),
        0.0,
        0.0,
    ];
    let mut solver = Lsoda::new(system, options, 0.0, &initial)?;
    solver.integrate(END_TIME_S)?;
    Ok(solver.y().try_into().expect("four-equation LSODA state"))
}
