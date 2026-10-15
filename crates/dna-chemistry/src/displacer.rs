//! Displacement of the decay products of a water molecule: `G4DNAWaterDissociationDisplacer`
//! (`source/processes/electromagnetic/dna/processes/src/G4DNAWaterDissociationDisplacer.cc`), the cases
//! reachable from the option2 events.
//!
//! The position of product `j` is `event position + mother displacement + product displacement j`
//! (`G4DNAMolecularDissociation::DecayIt`, processes/src/G4DNAMolecularDissociation.cc:218-258; the
//! navigator clipping of the displacement, `0.8 x safety`, does not exist in an infinite medium). In the C++
//! the product displacements are drawn first (`GetProductsDisplacement`), then the mother displacement
//! (`GetMotherMoleculeDisplacement`); the same order is kept here.
//!
//! `radialDistributionOfProducts(R)` (cc:683-692) is three independent Gaussians of `sigma = R / sqrt(3)`,
//! so that `<|d|^2> = R^2`: `R` is the root-mean-square 3-D displacement. Geant4 identifies a product by its
//! *molecule definition*, which matters twice: `OH-` has a definition of its own (distinct from `OH`) and therefore
//! receives **no** displacement in the dissociative attachment, and the species map of the ionisation cases knows
//! only `H3O+` and `OH`.
//!
//! # Random-number draws (per channel, docs/reproducibility.md)
//! A "triple" is two Box-Muller pairs (four uniforms, the fourth variate discarded, as in the M3 solvation).
//! * `A1b1Dissociation`: triple(2.4 nm).
//! * `B1a1Dissociation`: triple(0.8 nm), then for each OH in product order triple(1.1 nm).
//! * `B1a1Dissociation2`: for each H in product order triple(1.6 nm).
//! * `IonisationDissociation`, `AutoIonisation`: one uniform `u` (`u < 0.5`: `OH` displaced by RMS 0.8 nm and
//!   `H3O+` not, otherwise the opposite), then for each product in order a triple(0.8 nm) if it is the displaced
//!   one; the `e_aq` of `AutoIonisation` takes the Meesungnoen2002 penetration at 1.7 eV (one
//!   [`gaussian_penetration`]); finally a triple(2.0 nm) common to all products (the mother displacement).
//! * `DissociativeAttachment`: triple(0.8 nm), then for each `OH` (not `OH-`) triple(1.1 nm).

use crate::channels::DisplacementKind;
use dna_core::penetration::{gaussian_penetration, meesungnoen_r_mean_nm};
use dna_core::sampling::{gaussian_pair, uniform};
use dna_core::species::Species;
use dna_core::vec3::Vec3;
use rand::Rng;

/// RMS of the common (mother-molecule) displacement of the ionisation and auto-ionisation channels, nm
/// (`GetMotherMoleculeDisplacement`, cc:~187-227).
pub const MOTHER_RMS_NM: f64 = 2.0;
/// RMS of the displaced product in the ionisation / auto-ionisation channels, nm.
pub const IONISATION_RMS_NM: f64 = 0.8;
/// `A1B1_DissociationDecay`: RMS of the OH-H separation vector, nm.
pub const A1B1_RMS_NM: f64 = 2.4;
/// `B1A1_DissociationDecay` and `DissociativeAttachment`: RMS of the H2-OH axis, nm.
pub const B1A1_AXIS_RMS_NM: f64 = 0.8;
/// `B1A1_DissociationDecay` and `DissociativeAttachment`: RMS of the OH-OH relative displacement, nm.
pub const OH_PAIR_RMS_NM: f64 = 1.1;
/// `B1A1_DissociationDecay2`: RMS of each hydrogen atom, nm.
pub const B1A1_2_H_RMS_NM: f64 = 1.6;

/// `radialDistributionOfProducts(rms)`: three Gaussians of `sigma = rms / sqrt(3)` (four uniforms).
pub fn radial_distribution<R: Rng + ?Sized>(rms_nm: f64, rng: &mut R) -> Vec3 {
    let sigma = rms_nm * (1.0 / 3.0f64.sqrt());
    let (x, y) = gaussian_pair(rng);
    let (z, _unused) = gaussian_pair(rng);
    Vec3::new(sigma * x, sigma * y, sigma * z)
}

/// Mean radius, nm, of the auto-ionisation electron: Meesungnoen2002 at `ke` (1.7 eV in Geant4).
pub fn autoionisation_r_mean_nm(kinetic_energy_ev: f64) -> f64 {
    meesungnoen_r_mean_nm(kinetic_energy_ev)
}

/// Displacements (nm) of the products of one channel, in product order, mother displacement included.
/// `electron_r_mean_nm` is [`autoionisation_r_mean_nm`] of the configured energy and must be positive.
pub fn displacements<R: Rng + ?Sized>(
    kind: DisplacementKind,
    products: &[Species],
    electron_r_mean_nm: f64,
    rng: &mut R,
) -> Vec<Vec3> {
    let zero = Vec3::new(0.0, 0.0, 0.0);
    let mut out = vec![zero; products.len()];
    match kind {
        DisplacementKind::None => {}
        DisplacementKind::A1b1Dissociation => {
            let axis = radial_distribution(A1B1_RMS_NM, rng);
            for (d, s) in out.iter_mut().zip(products) {
                match s {
                    Species::Oh => *d = axis * (-1.0 / 18.0),
                    Species::H => *d = axis * (17.0 / 18.0),
                    _ => {}
                }
            }
        }
        DisplacementKind::B1a1Dissociation | DisplacementKind::DissociativeAttachment => {
            let axis = radial_distribution(B1A1_AXIS_RMS_NM, rng);
            let mut n_oh = 0;
            for (d, s) in out.iter_mut().zip(products) {
                match s {
                    Species::H2 => *d = axis * (-16.0 / 18.0),
                    Species::Oh => {
                        let oxygen = axis * (2.0 / 18.0);
                        let pair = radial_distribution(OH_PAIR_RMS_NM, rng);
                        let pair = if n_oh == 0 { pair * 0.5 } else { pair * -0.5 };
                        n_oh += 1;
                        *d = pair + oxygen;
                    }
                    // OH- (Ohm) has its own molecule definition in Geant4: no displacement.
                    _ => {}
                }
            }
        }
        DisplacementKind::B1a1Dissociation2 => {
            let mut n_h = 0;
            for (d, s) in out.iter_mut().zip(products) {
                if *s == Species::H {
                    let h = radial_distribution(B1A1_2_H_RMS_NM, rng);
                    *d = if n_h == 0 { h * 0.5 } else { h * -0.5 };
                    n_h += 1;
                }
            }
        }
        DisplacementKind::IonisationDissociation | DisplacementKind::AutoIonisation => {
            let (h3o_rms, oh_rms) = if uniform(rng) < 0.5 {
                (0.0, IONISATION_RMS_NM)
            } else {
                (IONISATION_RMS_NM, 0.0)
            };
            for (d, s) in out.iter_mut().zip(products) {
                let rms = match s {
                    Species::H3Op => h3o_rms,
                    Species::Oh => oh_rms,
                    _ => 0.0,
                };
                if rms != 0.0 {
                    *d = radial_distribution(rms, rng);
                }
                if kind == DisplacementKind::AutoIonisation && *s == Species::EAq {
                    *d = gaussian_penetration(electron_r_mean_nm, rng);
                }
            }
            // The mother displacement is added to every product (cc, DecayIt).
            let mother = radial_distribution(MOTHER_RMS_NM, rng);
            for d in &mut out {
                *d = *d + mother;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use dna_core::testing::ScriptedRng;

    #[test]
    fn radial_distribution_uses_four_uniforms_and_sigma_over_root_three() {
        let mut rng = ScriptedRng::new(&[0.25, 0.125, 0.5, 0.5]);
        let v = radial_distribution(3.0, &mut rng);
        assert_eq!(rng.consumed(), 4);
        let sigma = 3.0 / 3.0f64.sqrt();
        let r = (-2.0f64 * 0.25f64.ln()).sqrt();
        let phi = 2.0 * std::f64::consts::PI * 0.125;
        assert!((v.x - sigma * r * phi.cos()).abs() < 1e-9);
        assert!((v.y - sigma * r * phi.sin()).abs() < 1e-9);
    }

    #[test]
    fn dissociative_attachment_leaves_the_hydroxide_in_place() {
        // H2, OH-, OH: draws = triple(0.8) + triple(1.1) = 8 uniforms
        let mut rng = ScriptedRng::new(&[0.3; 8]);
        let d = displacements(
            DisplacementKind::DissociativeAttachment,
            &[Species::H2, Species::Ohm, Species::Oh],
            1.0,
            &mut rng,
        );
        assert_eq!(rng.consumed(), 8);
        assert_eq!(d[1], Vec3::new(0.0, 0.0, 0.0));
        assert_ne!(d[0], Vec3::new(0.0, 0.0, 0.0));
        assert_ne!(d[2], Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn ionisation_draw_count_and_common_mother_shift() {
        // u (1) + triple for the displaced product (4) + mother triple (4) = 9 uniforms
        let mut rng = ScriptedRng::new(&[0.2, 0.3, 0.4, 0.5, 0.6, 0.35, 0.45, 0.55, 0.65]);
        let d = displacements(
            DisplacementKind::IonisationDissociation,
            &[Species::H3Op, Species::Oh],
            1.0,
            &mut rng,
        );
        assert_eq!(rng.consumed(), 9);
        // u = 0.2 < 0.5: H3O+ not displaced by the 0.8 nm law, OH is: the H3O+ displacement is the mother shift alone.
        let mother_only = d[0];
        let mut check = ScriptedRng::new(&[0.35, 0.45, 0.55, 0.65]);
        let m = radial_distribution(MOTHER_RMS_NM, &mut check);
        assert!((mother_only.x - m.x).abs() < 1e-9 && (mother_only.z - m.z).abs() < 1e-9);
    }

    #[test]
    fn branch_probability_boundary_is_strict() {
        // u == 0.5 is not < 0.5: H3O+ is the displaced one
        let mut rng = ScriptedRng::new(&[0.5, 0.3, 0.4, 0.5, 0.6, 0.35, 0.45, 0.55, 0.65]);
        let d = displacements(
            DisplacementKind::IonisationDissociation,
            &[Species::H3Op, Species::Oh],
            1.0,
            &mut rng,
        );
        let mut check = ScriptedRng::new(&[0.35, 0.45, 0.55, 0.65]);
        let m = radial_distribution(MOTHER_RMS_NM, &mut check);
        assert!((d[1].x - m.x).abs() < 1e-9, "OH carries only the mother shift");
        assert!((d[0].x - m.x).abs() > 1e-6, "H3O+ carries the extra 0.8 nm displacement");
    }
}
