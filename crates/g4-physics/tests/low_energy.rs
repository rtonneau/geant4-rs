//! M3 low-energy electron components on the committed tables: Sanche vibrational excitation,
//! Melton attachment and solvation (`G4DNASancheExcitationModel`, `G4DNAMeltonAttachmentModel`,
//! `G4DNAOneStepThermalizationModel<Meesungnoen2002>`).
//!
//! **Pre-registered acceptance (fixed before the first run):**
//! * Cross sections: 1e-9 relative to `scripts/hand_cross_sections.py`, an independent Python
//!   re-implementation of the C++ (Sanche: own reader, lin-lin interpolation of the node sums,
//!   `t (1 - 1e-12)` protection, factor 2, the 1e-30 mm^2 floor; Melton: log-log, scale
//!   1e-18 cm^2, last-node value beyond 12.3 eV), at every table node of the model window and at
//!   three off-node energies. Rust output was never used to produce them.
//! * Statistical tests use fixed seeds (so they are deterministic) and a Bonferroni-corrected
//!   level `alpha = 1e-3 / m` per family:
//!   - family A, Sanche level chi-square against the raw table at 2, 7.4, 31 and 100 eV: `m = 4`;
//!   - family B, solvation: 9 Kolmogorov-Smirnov tests (x, y, z components at 0.5, 3, 7.4 eV
//!     against a normal with `sigma_1D = 0.62665706865775 r_mean`), 3 z-tests that the mean
//!     displacement length equals `r_mean`, and 2 KS tests (cos theta, phi) of the
//!     `r_mean == 0` branch against the isotropic distribution: `m = 14`;
//!   - family C, Melton first-interaction fractions at 7.5 eV (elastic, vibrational,
//!     attachment): |z| < 4 each.

mod common;

use approx::assert_relative_eq;
use common::{chi2_merged, chi2_pvalue, data, ev, ks_pvalue, ks_statistic, raw_rows};
use g4_core::error::DnaError;
use g4_core::physchem::{PhysChemEvent, PhysChemKind};
use g4_core::rng::{RngStreams, Stage};
use g4_core::units::Area;
use g4_core::vec3::Vec3;
use g4_data::xs::{CrossSectionTable, Interpolation, XsSpec};
use g4_physics::PhysicsError;
use g4_physics::models::{
    CrossSectionModel, ElasticRotation, MeltonAttachment, ProcessKind, SANCHE_LEVELS,
    SancheVibrational, Solvation, meesungnoen_r_mean_nm, vibration_energy,
};
use g4_physics::table::ProcessTable;
use g4_physics::testing::ScriptedRng;
use g4_physics::transport::{
    HistoryObserver, NoObserver, Primary, SolvationEvent, TransportConfig, transport_electron,
};
use g4_physics::water::WATER_MOLECULES_PER_NM3;

fn next_up(x: f64) -> f64 {
    f64::from_bits(x.to_bits() + 1)
}
fn next_down(x: f64) -> f64 {
    f64::from_bits(x.to_bits() - 1)
}

const Z: Vec3 = Vec3::new(0.0, 0.0, 1.0);

// ------------------------------------------------------------------------ Sanche ----------

/// `(E eV, Sigma_vib /nm incl. the factor 2 and the water density)` at off-node energies, from the
/// hand script (`sigma_low_energy`).
const SANCHE_HAND: [(f64, f64); 5] = [
    (2.0, 0.5625259789579856),
    (2.5, 0.49722999930713),
    (31.0, 0.32037372131764524),
    (97.3, 0.2879485846650872),
    (100.0, 0.2861434739649086),
];

/// Partial cross sections (nm^2 per molecule, no factor 2) from the hand script. Exact zeros of
/// the interpolation appear as the Geant4 floor `1e-30 mm^2 = 1e-18 nm^2`.
const SANCHE_PARTIALS_HAND: [(f64, [f64; 9]); 3] = [
    (
        2.0,
        [
            1e-18,
            0.0019160000000000006,
            0.0011200000000000001,
            0.00447,
            0.00017,
            0.00064,
            2.4000000000000004e-05,
            6.199999999999999e-05,
            1.1999999999999997e-05,
        ],
    ),
    (
        31.0,
        [
            0.00017199999999999998,
            0.001286,
            0.001204,
            0.001726,
            0.000244,
            0.00016,
            1e-18,
            1e-18,
            1e-18,
        ],
    ),
    (
        97.3,
        [
            4.2699999999999994e-05,
            0.0011754000000000003,
            0.0013454000000000003,
            0.0012981000000000002,
            0.00026269999999999993,
            0.00018270000000000002,
            1e-18,
            1e-18,
            1e-18,
        ],
    ),
];

#[test]
fn sanche_cross_sections_equal_the_hand_computation_off_node_and_at_the_100_ev_edge() {
    let Some(dir) = data() else { return };
    let m = SancheVibrational::from_data_dir(&dir).unwrap();
    for (e, sigma) in SANCHE_HAND {
        let got = m.cross_section(ev(e)).unwrap().nm2() * WATER_MOLECULES_PER_NM3;
        assert_relative_eq!(got, sigma, max_relative = 1e-9);
    }
    for (e, partials) in SANCHE_PARTIALS_HAND {
        let got = m.partial_cross_sections(ev(e)).unwrap();
        for (level, (g, h)) in got.iter().zip(partials).enumerate() {
            assert_relative_eq!(*g, h, max_relative = 1e-9, epsilon = 0.0);
            assert!(
                *g > 0.0,
                "level {level} at {e}: the floor keeps it positive"
            );
        }
    }
}

/// At every node of the window the cross section is `2 x 1e-2 nm^2 x (sum of the nine columns)`:
/// literals straight from the raw file, no interpolation involved.
#[test]
fn sanche_cross_section_at_every_node_is_twice_the_row_sum_times_the_scale() {
    let Some(dir) = data() else { return };
    let m = SancheVibrational::from_data_dir(&dir).unwrap();
    let mut checked = 0;
    for r in raw_rows("sigma_excitationvib_e_sanche.dat")
        .iter()
        .filter(|r| (2.0..=100.0).contains(&r[0]))
    {
        let want = 2.0 * r[1..].iter().sum::<f64>() * 1e-16 * 1e14;
        // 100 eV is evaluated at 100 (1 - 1e-12): the protection changes the value by ~1e-12
        let tol = if r[0] == 100.0 { 1e-9 } else { 1e-12 };
        assert_relative_eq!(
            m.cross_section(ev(r[0])).unwrap().nm2(),
            want,
            max_relative = tol
        );
        checked += 1;
    }
    assert!(checked >= 30, "{checked} nodes in the window");
}

#[test]
fn sanche_level_energies_and_final_state_bookkeeping() {
    let Some(dir) = data() else { return };
    let m = SancheVibrational::from_data_dir(&dir).unwrap();
    let want = [
        0.010, 0.024, 0.061, 0.092, 0.204, 0.417, 0.460, 0.500, 0.835,
    ];
    for (l, w) in want.iter().enumerate() {
        assert_eq!(vibration_energy(l).ev(), *w);
    }
    let mut rng = RngStreams::new(1).stream(0, Stage::Physics).unwrap();
    for e in [2.0, 5.5, 50.0, 100.0] {
        for _ in 0..200 {
            let fs = m.sample(ev(e), Z, &mut rng).unwrap();
            let e_vib = vibration_energy(fs.channel).ev();
            assert_eq!(fs.local_deposit.ev(), e_vib);
            assert_eq!(fs.energy_after.ev(), e - e_vib);
            assert_eq!(fs.direction_after, Z, "direction is unchanged");
            assert!(fs.secondary.is_none() && !fs.killed);
            assert!(!fs.quirks.vibrational_noop);
        }
    }
}

#[test]
fn sanche_draws_exactly_one_uniform_and_selects_from_the_last_level_down() {
    let Some(dir) = data() else { return };
    let m = SancheVibrational::from_data_dir(&dir).unwrap();
    // at 31 eV levels 6, 7, 8 only have the floor (1e-18), level 5 is the last real one:
    // u -> 0 lands on the last level whose value exceeds the running remainder
    let mut rng = ScriptedRng::new(&[1e-9]);
    let fs = m.sample(ev(31.0), Z, &mut rng).unwrap();
    assert_eq!(rng.consumed(), 1);
    assert_eq!(fs.channel, 5);
    // u -> 1 falls through to level 0
    let mut rng = ScriptedRng::new(&[1.0 - 1e-9]);
    let fs = m.sample(ev(31.0), Z, &mut rng).unwrap();
    assert_eq!(fs.channel, 0);
}

/// Family A: chi-square of the sampled level against the partial cross sections interpolated
/// independently from the raw file (lin-lin, zero -> 1e-18 floor).
#[test]
fn sanche_level_distribution_matches_the_raw_table() {
    let Some(dir) = data() else { return };
    let m = SancheVibrational::from_data_dir(&dir).unwrap();
    let rows = raw_rows("sigma_excitationvib_e_sanche.dat");
    let partial_probs = |e: f64| -> [f64; 9] {
        let t = if e == rows.last().unwrap()[0] {
            e * (1.0 - 1e-12)
        } else {
            e
        };
        let k2 = rows.iter().position(|r| r[0] > t).unwrap();
        let k1 = k2 - 1;
        let mut v = [0.0; 9];
        for (l, slot) in v.iter_mut().enumerate() {
            let (e1, e2) = (rows[k1][0], rows[k2][0]);
            let (x1, x2) = (rows[k1][l + 1], rows[k2][l + 1]);
            let a = (x2 - x1) / (e2 - e1);
            let b = x2 - a * e2;
            let s = (a * t + b) * 1e-2;
            *slot = if s == 0.0 { 1e-18 } else { s };
        }
        let sum: f64 = v.iter().sum();
        v.map(|x| x / sum)
    };
    let alpha = 1e-3 / 4.0;
    let streams = RngStreams::new(31);
    const N: usize = 400_000;
    for (case, e) in [2.0, 7.4, 31.0, 100.0].into_iter().enumerate() {
        let mut rng = streams.stream(case as u64, Stage::Physics).unwrap();
        let mut counts = [0.0; SANCHE_LEVELS];
        for _ in 0..N {
            counts[m.sample(ev(e), Z, &mut rng).unwrap().channel] += 1.0;
        }
        let expected: Vec<f64> = partial_probs(e).iter().map(|p| p * N as f64).collect();
        let (chi2, dof) = chi2_merged(&counts, &expected);
        let p = chi2_pvalue(chi2, dof);
        eprintln!("Sanche level chi2 at {e} eV: {chi2:.2}/{dof}, p = {p:.3}");
        assert!(p > alpha, "{e} eV: chi2 = {chi2}, dof = {dof}, p = {p:e}");
    }
}

/// Unit scale of the 1e-30 floor: a level that is exactly zero over the whole table keeps the
/// Geant4 floor `1e-30 mm^2` (= 1e-18 nm^2), not zero and not `1e-30 nm^2`.
#[test]
fn sanche_zero_floor_is_one_e_minus_30_in_geant4_units() {
    let rows = "0 1 0 0 0 0 0 0 0 0\n1.7 1 0 0 0 0 0 0 0 0\n100 1 0 0 0 0 0 0 0 0\n";
    let t = CrossSectionTable::from_text(
        "synthetic-sanche",
        rows,
        XsSpec {
            scale: Area::from_cm2(1e-16),
            interpolation: Interpolation::LinLin,
        },
    )
    .unwrap();
    let m = SancheVibrational::new(t).unwrap();
    let p = m.partial_cross_sections(ev(50.0)).unwrap();
    assert_relative_eq!(p[0], 1e-2, max_relative = 1e-12);
    for v in &p[1..] {
        assert_eq!(*v, 1e-18);
    }
    // level 0 is selected for any u: the floor levels carry probability ~1e-16
    let mut rng = ScriptedRng::new(&[0.5]);
    assert_eq!(m.sample(ev(50.0), Z, &mut rng).unwrap().channel, 0);
}

// ------------------------------------------------------------------------ Melton ----------

/// `(E eV, Sigma_att /nm)` from the hand script: nodes are checked against the raw file below.
const MELTON_HAND: [(f64, f64); 8] = [
    (4.0, 3.342797593047217e-303),
    (4.1, 3.342797593047217e-303),
    (5.9, 0.0028255640266549023),
    (6.1, 0.013744258414271637),
    (7.3, 0.005881299202483132),
    (11.7, 3.5419007987008926e-143),
    (12.3, 0.0),
    (12.8, 0.0),
];

#[test]
fn melton_cross_sections_equal_the_hand_computation() {
    let Some(dir) = data() else { return };
    let m = MeltonAttachment::from_data_dir(&dir).unwrap();
    for (e, sigma) in MELTON_HAND {
        let got = m.cross_section(ev(e)).unwrap().nm2() * WATER_MOLECULES_PER_NM3;
        if sigma == 0.0 {
            assert_eq!(got, 0.0, "{e} eV");
        } else {
            assert_relative_eq!(got, sigma, max_relative = 1e-9, epsilon = 0.0);
        }
    }
}

#[test]
fn melton_cross_section_at_every_interior_node_is_the_file_value_times_the_scale() {
    let Some(dir) = data() else { return };
    let m = MeltonAttachment::from_data_dir(&dir).unwrap();
    for r in raw_rows("sigma_attachment_e_melton.dat")
        .iter()
        .filter(|r| (4.0..=12.3).contains(&r[0]))
    {
        let want = r[1] * 1e-18 * 1e14;
        let got = m.cross_section(ev(r[0])).unwrap().nm2();
        if want == 0.0 {
            // interior zero nodes give the 1e-300 floor of G4DNACrossSectionDataSet, ~0
            assert!(got < 1e-290, "{} eV: {got:e}", r[0]);
        } else {
            assert_relative_eq!(got, want, max_relative = 1e-12);
        }
    }
}

#[test]
fn melton_window_boundaries_and_the_12_3_to_13_ev_decision() {
    let Some(dir) = data() else { return };
    let m = MeltonAttachment::from_data_dir(&dir).unwrap();
    assert_eq!(m.table_last_energy_ev(), 12.3);
    assert_eq!(
        (m.energy_range().min().ev(), m.energy_range().max().ev()),
        (4.0, 13.0)
    );
    // below the window and above it: exactly zero
    assert_eq!(m.cross_section(ev(next_down(4.0))).unwrap().nm2(), 0.0);
    assert_eq!(m.cross_section(ev(next_up(13.0))).unwrap().nm2(), 0.0);
    // at the lower edge the log-log rule between the zero nodes 0 and 4.2 gives a tiny positive
    // value (the 1e-300 floors), reproduced from Geant4
    let low = m.cross_section(ev(4.0)).unwrap().nm2();
    assert!(low > 0.0 && low < 1e-280, "{low:e}");
    // DECISION (docs/02 B.7.1): between the last node (12.3 eV, value 0) and the window end
    // (13 eV) the cross section is zero, as in Geant4's constant clamp; the table layer itself
    // would raise UnsupportedEnergy there.
    for e in [12.3, next_up(12.3), 12.8, next_down(13.0), 13.0] {
        assert_eq!(m.cross_section(ev(e)).unwrap().nm2(), 0.0, "{e} eV");
    }
    // sampling is allowed anywhere in the window and outside it is an error
    let mut rng = ScriptedRng::new(&[0.5]);
    assert!(m.sample(ev(13.0), Z, &mut rng).is_ok());
    for e in [next_down(4.0), next_up(13.0)] {
        let err = m.sample(ev(e), Z, &mut rng).unwrap_err();
        assert!(matches!(
            err,
            PhysicsError::Core(DnaError::UnsupportedEnergy { .. })
        ));
    }
}

#[test]
fn melton_construction_rejects_a_non_zero_tail_because_the_clamp_would_extrapolate() {
    let spec = XsSpec {
        scale: Area::from_cm2(1e-18),
        interpolation: Interpolation::LogLog,
    };
    // last node (12.3 eV) non-zero: Geant4's clamp would be a constant extrapolation to 13 eV
    let t = CrossSectionTable::from_text("synthetic", "0 0\n4.2 0\n12.3 0.5\n", spec).unwrap();
    assert!(matches!(
        MeltonAttachment::new(t),
        Err(PhysicsError::TableTailNotZero { .. })
    ));
    // a table that starts above the window start is rejected as well
    let t = CrossSectionTable::from_text("synthetic", "5 0\n13 0\n", spec).unwrap();
    assert!(matches!(
        MeltonAttachment::new(t),
        Err(PhysicsError::TableDoesNotCoverWindow { .. })
    ));
    // a zero tail is accepted
    let t = CrossSectionTable::from_text("synthetic", "0 0\n4.2 0.1\n12.3 0\n", spec).unwrap();
    assert!(MeltonAttachment::new(t).is_ok());
}

#[test]
fn melton_absorbs_the_electron_depositing_its_whole_kinetic_energy_and_draws_nothing() {
    let Some(dir) = data() else { return };
    let m = MeltonAttachment::from_data_dir(&dir).unwrap();
    let mut rng = ScriptedRng::new(&[]);
    for e in [4.0, 6.1, 12.3, 13.0] {
        let fs = m.sample(ev(e), Z, &mut rng).unwrap();
        assert!(fs.killed);
        assert_eq!(fs.local_deposit.ev(), e);
        assert_eq!(fs.energy_after.ev(), 0.0);
        assert!(fs.secondary.is_none());
    }
    assert_eq!(rng.consumed(), 0);
}

/// Family C: the absorption branch against the raw table. 7.5 eV is a solvation-free energy
/// where elastic, vibrational and attachment compete; attachment is ~7e-4 of the first events.
#[test]
fn melton_absorption_fraction_at_7_5_ev_matches_the_cross_section_ratio() {
    let Some(dir) = data() else { return };
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    let xs = table.macroscopic(ev(7.5)).unwrap();
    let streams = RngStreams::new(75);
    let cfg = TransportConfig {
        max_interactions: Some(1),
        ..TransportConfig::default()
    };
    const N: usize = 1_000_000;
    let mut counts = [0usize; ProcessKind::COUNT];
    for id in 0..N as u64 {
        let mut rng = streams.stream(id, Stage::Physics).unwrap();
        let out = transport_electron(
            &table,
            &cfg,
            Primary {
                energy: ev(7.5),
                position: Vec3::ZERO,
                direction: Z,
                history_id: id,
            },
            &mut rng,
            &mut NoObserver,
        )
        .unwrap();
        counts[out.first_step.unwrap().process.index()] += 1;
        if out.counts.get(ProcessKind::Attachment) == 1 {
            // absorbed: the whole kinetic energy is the deposit, nothing remains
            assert_eq!(out.ledger.deposited_attachment, 7.5);
            assert_eq!(out.ledger.remaining_untracked, 0.0);
        }
    }
    for p in [
        ProcessKind::Elastic,
        ProcessKind::VibrationalExcitation,
        ProcessKind::Attachment,
    ] {
        let frac = xs.get(p) / xs.total();
        let sd = (frac * (1.0 - frac) / N as f64).sqrt();
        let z = (counts[p.index()] as f64 / N as f64 - frac) / sd;
        eprintln!(
            "7.5 eV first interaction {p:?}: {} (p = {frac:.6}), z = {z:+.2}",
            counts[p.index()]
        );
        assert!(z.abs() < 4.0, "{p:?}: z = {z}");
    }
    assert_eq!(counts[ProcessKind::Excitation.index()], 0);
    assert_eq!(counts[ProcessKind::Ionisation.index()], 0);
}

// ---------------------------------------------------------------------- Solvation ---------

const RMEAN_HAND: [(f64, f64); 9] = [
    (0.05, 0.0),
    (0.1, 0.0),
    (0.11, 0.1402528015064279),
    (0.2, 0.3912580596345324),
    (1.0, 1.7189556560510009),
    (2.5, 5.148425997474059),
    (5.0, 15.309887521273396),
    (7.0, 19.10265538232997),
    (7.4, 18.860116394854792),
];

#[test]
fn meesungnoen_r_mean_matches_the_hand_polynomial() {
    for (e, r) in RMEAN_HAND {
        if r == 0.0 {
            assert_eq!(meesungnoen_r_mean_nm(e), 0.0, "E = {e}");
        } else {
            assert_relative_eq!(meesungnoen_r_mean_nm(e), r, max_relative = 1e-9);
        }
    }
    // the fit is positive over its whole use (0.1 eV, 7.4 eV]
    for k in 1..=7300 {
        let e = 0.1 + k as f64 * 0.001;
        assert!(meesungnoen_r_mean_nm(e) > 0.0, "E = {e}");
    }
}

#[test]
fn solvation_window_is_closed_at_7_4_ev_and_deposits_the_kinetic_energy() {
    let s = Solvation::new();
    assert!(s.applies(ev(0.0)) && s.applies(ev(1e-12)) && s.applies(ev(7.4)));
    assert!(!s.applies(ev(next_up(7.4))));
    let mut rng = RngStreams::new(2).stream(0, Stage::Physics).unwrap();
    for e in [0.0, 0.05, 0.1, 0.11, 3.3, 7.4] {
        let out = s.sample(ev(e), &mut rng).unwrap();
        assert_eq!(out.deposit.ev(), e, "the whole kinetic energy is deposited");
        assert!(out.displacement_nm.norm().is_finite());
    }
    let err = s.sample(ev(next_up(7.4)), &mut rng).unwrap_err();
    assert!(matches!(
        err,
        PhysicsError::Core(DnaError::UnsupportedEnergy { .. })
    ));
}

#[test]
fn solvation_draw_counts_are_four_uniforms_or_two_in_the_zero_rmean_branch() {
    let s = Solvation::new();
    let mut rng = ScriptedRng::new(&[0.3, 0.4, 0.5, 0.6]);
    let out = s.sample(ev(3.0), &mut rng).unwrap();
    assert_eq!(rng.consumed(), 4);
    assert!(!out.zero_rmean_branch);
    // x and y come from the first Box-Muller pair, z from the cosine term of the second
    let sigma = out.r_mean_nm * 0.626_657_068_657_750_1;
    let r1 = (-2.0f64 * 0.3f64.ln()).sqrt();
    let r2 = (-2.0f64 * 0.5f64.ln()).sqrt();
    let tau = std::f64::consts::TAU;
    assert_relative_eq!(
        out.displacement_nm.x,
        sigma * r1 * (tau * 0.4).cos(),
        max_relative = 1e-12
    );
    assert_relative_eq!(
        out.displacement_nm.y,
        sigma * r1 * (tau * 0.4).sin(),
        max_relative = 1e-12
    );
    assert_relative_eq!(
        out.displacement_nm.z,
        sigma * r2 * (tau * 0.6).cos(),
        max_relative = 1e-12
    );

    let mut rng = ScriptedRng::new(&[0.25, 0.75]);
    let out = s.sample(ev(0.05), &mut rng).unwrap();
    assert_eq!(rng.consumed(), 2);
    assert!(out.zero_rmean_branch);
    assert_relative_eq!(out.displacement_nm.norm(), 1e-3, max_relative = 1e-12);
}

#[test]
fn transport_solvation_event_keeps_the_sampled_displacement_and_local_deposit() {
    let Some(dir) = data() else { return };
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    #[derive(Default)]
    struct Recorder {
        solvations: Vec<SolvationEvent>,
        physchem: Vec<PhysChemEvent>,
    }
    impl HistoryObserver for Recorder {
        fn on_solvation(&mut self, event: &SolvationEvent) {
            self.solvations.push(*event);
        }
        fn on_physchem(&mut self, event: &PhysChemEvent) {
            self.physchem.push(*event);
        }
    }

    // Use valid primaries, including the 7.4 eV boundary, at an off-origin thermalisation point.
    // The sampler distribution is tested separately; here only its transport handoff is checked.
    let origin = Vec3::new(17.0, -31.0, 5.0);
    let draws = [0.3, 0.4, 0.5, 0.6];
    for energy in [2.0, 7.4] {
        let mut sampler_rng = ScriptedRng::new(&draws);
        let sampled = table
            .solvation()
            .unwrap()
            .sample(ev(energy), &mut sampler_rng)
            .unwrap();
        let mut rng = ScriptedRng::new(&draws);
        let mut recorder = Recorder::default();
        let outcome = transport_electron(
            &table,
            &TransportConfig::default(),
            Primary {
                energy: ev(energy),
                position: origin,
                direction: Z,
                history_id: 42,
            },
            &mut rng,
            &mut recorder,
        )
        .unwrap();
        assert_eq!(rng.consumed(), sampler_rng.consumed());
        assert_eq!(recorder.solvations.len(), 1);
        let solvation = &recorder.solvations[0];
        assert_eq!(solvation.origin, origin);
        assert_eq!(solvation.displacement_nm, sampled.displacement_nm);
        assert_eq!(solvation.energy, sampled.deposit);
        assert_eq!(
            recorder.physchem,
            [PhysChemEvent {
                kind: PhysChemKind::Solvation,
                level_or_shell: 0,
                position: origin + sampled.displacement_nm,
                deposited_energy: ev(energy),
                parent_history: 42,
            }]
        );
        assert_eq!(outcome.solvations, 1);
        assert_eq!(outcome.steps, 0);
        assert_eq!(outcome.track_length_nm, 0.0);
        assert_eq!(outcome.end_position, origin);
        assert_eq!(outcome.ledger.deposited_solvation, energy);
        assert_eq!(outcome.ledger.residual(), 0.0);
    }
}

fn normal_cdf(z: f64) -> f64 {
    // Phi(z) = 1 - Q/2 for z >= 0, with Q = erfc(z/sqrt 2) = chi2 tail of z^2 with 1 dof
    let tail = 0.5 * chi2_pvalue(z * z, 1);
    if z >= 0.0 { 1.0 - tail } else { tail }
}

/// Family B (12 of its 14 tests): solvation displacement of `r_mean > 0` energies.
#[test]
fn solvation_displacement_is_a_gaussian_with_sigma_proportional_to_the_mean_radius() {
    let s = Solvation::new();
    let alpha = 1e-3 / 14.0;
    let streams = RngStreams::new(99);
    const N: usize = 100_000;
    for (case, e) in [0.5, 3.0, 7.4].into_iter().enumerate() {
        let mut rng = streams.stream(case as u64, Stage::Physics).unwrap();
        let r_mean = meesungnoen_r_mean_nm(e);
        let sigma = r_mean * 0.626_657_068_657_750_1;
        let (mut xs, mut ys, mut zs, mut lens) = (vec![], vec![], vec![], vec![]);
        for _ in 0..N {
            let d = s.sample(ev(e), &mut rng).unwrap().displacement_nm;
            xs.push(d.x);
            ys.push(d.y);
            zs.push(d.z);
            lens.push(d.norm());
        }
        for (name, mut v) in [("x", xs), ("y", ys), ("z", zs)] {
            v.sort_by(f64::total_cmp);
            let d = ks_statistic(&v, |x| normal_cdf(x / sigma));
            let p = ks_pvalue(d, N);
            eprintln!("solvation {name} at {e} eV: KS D = {d:.5}, p = {p:.3}");
            assert!(p > alpha, "{name} at {e} eV: p = {p:e}");
        }
        // mean of the Maxwell-distributed length = sigma * 2 sqrt(2/pi) = r_mean
        let mean = lens.iter().sum::<f64>() / N as f64;
        let var = lens.iter().map(|l| (l - mean) * (l - mean)).sum::<f64>() / (N as f64 - 1.0);
        let z = (mean - r_mean) / (var / N as f64).sqrt();
        eprintln!("solvation mean length at {e} eV: {mean:.4} vs r_mean {r_mean:.4}, z = {z:+.2}");
        // two-sided normal p-value of z
        assert!(chi2_pvalue(z * z, 1) > alpha, "{e} eV: z = {z}");
    }
}

/// Family B (last 2 tests) and the exact length of the `r_mean == 0` branch.
#[test]
fn solvation_below_0_1_ev_displaces_by_1e_minus_3_nm_in_an_isotropic_direction() {
    let s = Solvation::new();
    let alpha = 1e-3 / 14.0;
    let mut rng = RngStreams::new(5).stream(0, Stage::Physics).unwrap();
    const N: usize = 100_000;
    let (mut cos, mut phi) = (vec![], vec![]);
    for _ in 0..N {
        let out = s.sample(ev(0.05), &mut rng).unwrap();
        assert!(out.zero_rmean_branch);
        let d = out.displacement_nm;
        assert_relative_eq!(d.norm(), 1.0e-3, max_relative = 1e-12);
        cos.push(d.z / d.norm());
        phi.push(d.y.atan2(d.x));
    }
    cos.sort_by(f64::total_cmp);
    phi.sort_by(f64::total_cmp);
    let p_cos = ks_pvalue(ks_statistic(&cos, |c| (c + 1.0) / 2.0), N);
    let p_phi = ks_pvalue(
        ks_statistic(&phi, |p| (p + std::f64::consts::PI) / std::f64::consts::TAU),
        N,
    );
    eprintln!("zero-rmean branch: p(cos theta) = {p_cos:.3}, p(phi) = {p_phi:.3}");
    assert!(p_cos > alpha && p_phi > alpha);
}

// -------------------------------------------------------------------- table assembly ------

#[test]
fn the_default_table_registers_all_six_components_with_sources() {
    let Some(dir) = data() else { return };
    let table = ProcessTable::option2_electron(&dir, ElasticRotation::Geant4).unwrap();
    let info = table.describe();
    let names: Vec<&str> = info.iter().map(|i| i.model).collect();
    for want in [
        "G4DNAChampionElasticModel",
        "G4DNABornExcitationModel1",
        "G4DNABornIonisationModel1",
        "G4DNASancheExcitationModel",
        "G4DNAMeltonAttachmentModel",
        "G4DNAOneStepThermalizationModel<Meesungnoen2002>",
    ] {
        assert!(names.contains(&want), "{want} missing from {names:?}");
    }
    assert!(info.iter().all(|i| i.geant4_source.contains(".cc")));
    // a table without solvation or low-energy processes still assembles (tests use this)
    let elastic_only = ProcessTable::from_models(
        vec![
            table
                .select(ProcessKind::Elastic, ev(100.0))
                .unwrap()
                .clone(),
        ],
        None,
    )
    .unwrap();
    assert!(elastic_only.solvation().is_none());
}
