//! V4-style statistical tests of the final-state sampling against the raw data files.
//!
//! **Pre-registered acceptance (fixed before the first run):**
//! * Every test is one member of a family of at most `FAMILY_SIZE = 40` tests (5 elastic KS,
//!   19 ejected-energy KS, 7 shell chi-square, 5 level chi-square, 2 mixture KS, 1 selection-order
//!   chi-square). The
//!   family-wise false-alarm level is 0.001, so each test must reach `p > 0.001 / 40 = 2.5e-5`
//!   (Bonferroni).
//! * Samples per test: 100 000 (KS and chi-square). Seeds are fixed, so outcomes are
//!   deterministic; a failure is investigated, never "fixed" by changing the seed.
//! * KS tests use the asymptotic p-value with Stephens' correction; chi-square merges bins with an
//!   expected count < 5.
//! * Oracles are built from the raw `data/dna` files by `common` (not by `dna-data`), at table
//!   nodes where Geant4's 2-D interpolation reduces to one block.

mod common;

use common::{chi2_merged, chi2_pvalue, data, ev, ks_pvalue, ks_statistic, raw_blocks, raw_rows};
use dna_core::rng::{RngStreams, Stage};
use dna_core::vec3::Vec3;
use dna_physics::models::{
    BornExcitation, BornIonisation, ChampionElastic, ElasticRotation, random_select,
};
use dna_physics::sampling::uniform;
use dna_physics::water::{EXCITATION_ENERGIES_EV, IONISATION_ENERGIES_EV};

const FAMILY_SIZE: f64 = 40.0;
const ALPHA: f64 = 0.001 / FAMILY_SIZE;
const N: usize = 100_000;
const BORN_SCALE_NM2: f64 = 1e-22 / 3.343 * 1e18;

fn rng(case: u64) -> dna_core::rng::HistoryRng {
    RngStreams::new(2026).stream(case, Stage::Physics).unwrap()
}

// ---------------------------------------------------------------- elastic angle ----------

/// CDF of the polar angle (degrees) of the block at node energy `e`: piecewise linear through the
/// `(theta, P)` rows (the lookup interpolates theta linearly in P, hence P linearly in theta).
fn champion_theta_cdf(e: f64) -> impl Fn(f64) -> f64 {
    let blocks = raw_blocks("sigmadiff_cumulated_elastic_e_champion_hp.dat");
    let (_, rows) = blocks
        .into_iter()
        .find(|(be, _)| *be == e)
        .unwrap_or_else(|| panic!("{e} eV is not a node of the Champion angular table"));
    // rows are [P, theta]; geant4 prepends (P = 0, theta = 0) when the first P is positive
    let mut nodes: Vec<(f64, f64)> = rows.iter().map(|r| (r[1], r[0])).collect();
    if nodes[0].1 > 0.0 {
        nodes.insert(0, (0.0, 0.0));
    }
    move |theta| {
        if theta <= nodes[0].0 {
            return nodes[0].1;
        }
        let i = nodes.partition_point(|n| n.0 <= theta);
        if i >= nodes.len() {
            return 1.0;
        }
        let ((t0, p0), (t1, p1)) = (nodes[i - 1], nodes[i]);
        p0 + (theta - t0) / (t1 - t0) * (p1 - p0)
    }
}

fn champion_nodes_near(targets: &[f64]) -> Vec<f64> {
    let energies: Vec<f64> = raw_blocks("sigmadiff_cumulated_elastic_e_champion_hp.dat")
        .into_iter()
        .map(|(e, _)| e)
        .filter(|&e| (7.4..=1.0e6).contains(&e))
        .collect();
    targets
        .iter()
        .map(|&t| {
            *energies
                .iter()
                .min_by(|a, b| (a.ln() - t.ln()).abs().total_cmp(&(b.ln() - t.ln()).abs()))
                .unwrap()
        })
        .collect()
}

#[test]
fn elastic_polar_angle_follows_the_tabulated_cdf() {
    let Some(dir) = data() else { return };
    let model = ChampionElastic::from_data_dir(&dir, ElasticRotation::Exact).unwrap();
    for (case, e) in champion_nodes_near(&[100.0, 1.0e3, 1.0e4, 1.0e5, 9.0e5])
        .into_iter()
        .enumerate()
    {
        let cdf = champion_theta_cdf(e);
        let mut r = rng(100 + case as u64);
        let mut theta: Vec<f64> = (0..N)
            .map(|_| {
                let c = model.cos_theta_for(ev(e), uniform(&mut r)).unwrap();
                c.acos().to_degrees()
            })
            .collect();
        theta.sort_by(f64::total_cmp);
        let d = ks_statistic(&theta, &cdf);
        let p = ks_pvalue(d, N);
        assert!(p > ALPHA, "elastic angle at {e} eV: D = {d}, p = {p:e}");
    }
}

// ------------------------------------------------------------ ionisation: ejected energy ----

/// CDF of the energy transfer `W` for one shell at node energy `e` (block read from the raw
/// cumulated table): nodes `(P, W)` keep the last `W` of each equal `P` (`:192-195`), `log10 W`
/// is linear in `P` between nodes (`Interpolate`, fast mode, `:819-880`).
fn born_w_cdf(e: f64, shell: usize) -> impl Fn(f64) -> f64 {
    let blocks = raw_blocks("sigmadiff_cumulated_ionisation_e_born_hp.dat");
    let (_, rows) = blocks
        .into_iter()
        .find(|(be, _)| *be == e)
        .unwrap_or_else(|| panic!("{e} eV is not a node of the Born ionisation CDF table"));
    let mut by_p: Vec<(f64, f64)> = Vec::new(); // (P, W) with the last W of each P
    for r in &rows {
        let (w, p) = (r[0], r[1 + shell]);
        match by_p.last_mut() {
            Some(last) if last.0 == p => last.1 = w,
            _ => by_p.push((p, w)),
        }
    }
    move |w| {
        if w <= by_p[0].1 {
            return 0.0;
        }
        let i = by_p.partition_point(|n| n.1 <= w);
        if i >= by_p.len() {
            return 1.0;
        }
        let ((p0, w0), (p1, w1)) = (by_p[i - 1], by_p[i]);
        let f = p0 + (w.ln() - w0.ln()) / (w1.ln() - w0.ln()) * (p1 - p0);
        f.min(1.0)
    }
}

#[test]
fn ejected_energy_transfer_follows_the_cumulated_table_for_every_shell() {
    let Some(dir) = data() else { return };
    let model = BornIonisation::from_data_dir(&dir).unwrap();
    let mut case = 0u64;
    // 100 eV: shells 0-3 open (shell 3 opens at 35 eV, shell 4 at 545 eV)
    for (e, shells) in [
        (100.0, 0..4usize),
        (1.0e3, 0..5),
        (1.0e4, 0..5),
        (1.0e5, 0..5),
    ] {
        for shell in shells {
            let cdf = born_w_cdf(e, shell);
            let mut r = rng(200 + case);
            case += 1;
            let mut w: Vec<f64> = (0..N)
                .map(|_| {
                    model
                        .transferred_energy(shell, ev(e), uniform(&mut r))
                        .unwrap()
                        .ev()
                })
                .collect();
            w.sort_by(f64::total_cmp);
            let d = ks_statistic(&w, &cdf);
            let p = ks_pvalue(d, N);
            assert!(p > ALPHA, "W at {e} eV shell {shell}: D = {d}, p = {p:e}");
        }
    }
    assert_eq!(case, 19);
}

// -------------------------------------------------------------- shell / level fractions ----

fn raw_row_at(name: &str, e: f64) -> Vec<f64> {
    raw_rows(name)
        .into_iter()
        .find(|r| r[0] == e)
        .unwrap_or_else(|| panic!("{e} eV is not a node of {name}"))
}

#[test]
fn ionisation_shell_fractions_equal_partial_over_total_cross_section() {
    let Some(dir) = data() else { return };
    let model = BornIonisation::from_data_dir(&dir).unwrap();
    let dirn = Vec3::new(0.0, 0.0, 1.0);
    for (case, e) in [15.0, 18.0, 100.0, 1.0e3, 1.0e4, 1.0e5, 1.0e6]
        .into_iter()
        .enumerate()
    {
        // partial cross sections straight from the raw file (nodes: no interpolation)
        let row = raw_row_at("sigma_ionisation_e_born.dat", e);
        let mut weights: Vec<f64> = row[1..].iter().map(|v| v * BORN_SCALE_NM2).collect();
        // `k < 19 eV`: shell 2 is re-drawn, i.e. removed from the distribution
        let rejected = e < 19.0;
        if rejected {
            weights[2] = 0.0;
        }
        let total: f64 = weights.iter().sum();
        let mut r = rng(300 + case as u64);
        let mut observed = [0.0f64; 5];
        let mut redraws = 0u64;
        for _ in 0..N {
            let fs = model.sample(ev(e), dirn, &mut r).unwrap();
            observed[fs.channel] += 1.0;
            redraws += u64::from(fs.quirks.shell2_redraws);
        }
        if rejected {
            assert_eq!(
                observed[2], 0.0,
                "shell 2 must never be returned below 19 eV"
            );
        }
        if e == 18.0 {
            // shell 2 opens at 18 eV in the table (first non-zero value), so it is drawn with
            // probability ~1e-3 and re-drawn: ~130 occurrences expected in 1e5 events
            assert!(
                redraws > 50,
                "18 eV: shell 2 is drawn and re-drawn ({redraws})"
            );
        }
        let expected: Vec<f64> = weights.iter().map(|w| w / total * N as f64).collect();
        let (chi2, dof) = chi2_merged(&observed, &expected);
        let p = chi2_pvalue(chi2, dof);
        assert!(
            p > ALPHA,
            "shell fractions at {e} eV: chi2 = {chi2} ({dof} dof), p = {p:e}, observed {observed:?}"
        );
    }
}

#[test]
fn excitation_level_fractions_equal_partial_over_total_cross_section() {
    let Some(dir) = data() else { return };
    let model = BornExcitation::from_data_dir(&dir).unwrap();
    let dirn = Vec3::new(0.0, 0.0, 1.0);
    for (case, e) in [10.0, 20.0, 100.0, 1.0e3, 1.0e5].into_iter().enumerate() {
        let row = raw_row_at("sigma_excitation_e_born.dat", e);
        let weights: Vec<f64> = row[1..].iter().map(|v| v * BORN_SCALE_NM2).collect();
        let total: f64 = weights.iter().sum();
        let mut r = rng(400 + case as u64);
        let mut observed = [0.0f64; 5];
        for _ in 0..N {
            let fs = model.sample(ev(e), dirn, &mut r).unwrap();
            observed[fs.channel] += 1.0;
            // energy bookkeeping of this very event
            let exc = EXCITATION_ENERGIES_EV[fs.channel];
            if e - exc > 0.0 {
                assert_eq!(fs.local_deposit.ev(), exc);
                assert_eq!(fs.energy_after.ev(), e - exc);
                assert!(!fs.quirks.excitation_noop);
            } else {
                assert_eq!(fs.local_deposit.ev(), 0.0);
                assert_eq!(fs.energy_after.ev(), e);
                assert!(fs.quirks.excitation_noop);
            }
            assert_eq!(fs.direction_after, dirn);
            assert!(fs.secondary.is_none());
        }
        let expected: Vec<f64> = weights.iter().map(|w| w / total * N as f64).collect();
        let (chi2, dof) = chi2_merged(&observed, &expected);
        let p = chi2_pvalue(chi2, dof);
        assert!(
            p > ALPHA,
            "level fractions at {e} eV: chi2 = {chi2} ({dof} dof), p = {p:e}, observed {observed:?}"
        );
    }
}

// ------------------------------------------------------- full ionisation event: mixture ----

/// Secondary-electron energy of whole `sample()` events at a node energy: mixture over shells of
/// `W - B`, conditioned on a secondary being produced.
#[test]
fn secondary_energy_of_full_events_follows_the_shell_mixture() {
    let Some(dir) = data() else { return };
    let model = BornIonisation::from_data_dir(&dir).unwrap();
    let dirn = Vec3::new(0.0, 0.0, 1.0);
    for (case, e) in [1.0e3, 1.0e5].into_iter().enumerate() {
        let row = raw_row_at("sigma_ionisation_e_born.dat", e);
        let weights: Vec<f64> = row[1..].iter().map(|v| v * BORN_SCALE_NM2).collect();
        let total: f64 = weights.iter().sum();
        let cdfs: Vec<_> = (0..5).map(|s| born_w_cdf(e, s)).collect();
        let mixture = |w: f64| -> f64 {
            (0..5)
                .map(|s| weights[s] / total * cdfs[s](w + IONISATION_ENERGIES_EV[s]))
                .sum()
        };
        let mass_at_zero = mixture(0.0);

        let mut r = rng(500 + case as u64);
        let mut secondary = Vec::with_capacity(N);
        let mut without = 0usize;
        for _ in 0..N {
            let fs = model.sample(ev(e), dirn, &mut r).unwrap();
            match fs.secondary {
                Some(s) => secondary.push(s.energy.ev()),
                None => without += 1,
            }
        }
        // events without a secondary: binomial against the mass at W <= B
        let sd = (mass_at_zero * (1.0 - mass_at_zero) * N as f64)
            .sqrt()
            .max(1.0);
        let z = (without as f64 - mass_at_zero * N as f64) / sd;
        assert!(
            z.abs() < 5.0,
            "{e} eV: {without} events without secondary, expected {}",
            mass_at_zero * N as f64
        );

        secondary.sort_by(f64::total_cmp);
        let n = secondary.len();
        let d = ks_statistic(&secondary, |w| {
            (mixture(w) - mass_at_zero) / (1.0 - mass_at_zero)
        });
        let p = ks_pvalue(d, n);
        assert!(p > ALPHA, "secondary energy at {e} eV: D = {d}, p = {p:e}");
    }
}

// -------------------------------------------------------- extras: selection order, sums ----

/// `random_select` agrees with a direct CDF pick for the real partial cross sections (guards the
/// downward walk against an off-by-one in the component order).
#[test]
fn downward_selection_has_the_same_distribution_as_a_cdf_pick() {
    let Some(dir) = data() else { return };
    let model = BornIonisation::from_data_dir(&dir).unwrap();
    let e = 1.0e4;
    let partials = model.partial_cross_sections(ev(e)).unwrap();
    let total: f64 = partials.iter().sum();
    let mut r = rng(600);
    let mut observed = [0.0f64; 5];
    for _ in 0..N {
        observed[random_select(&partials, uniform(&mut r))] += 1.0;
    }
    let expected: Vec<f64> = partials.iter().map(|p| p / total * N as f64).collect();
    let (chi2, dof) = chi2_merged(&observed, &expected);
    assert!(chi2_pvalue(chi2, dof) > ALPHA);
}
