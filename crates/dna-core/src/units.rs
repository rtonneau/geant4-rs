//! Unit newtypes. Internal units: energy eV, length nm, time ps.
//!
//! Geant4 uses MeV, mm and ns; mixing the two systems is the classic source of silent
//! errors, so every quantity here is a distinct `f64` newtype and conversions are explicit
//! (see `docs/units.md`). Geant4 reference for the unit system: `source/externals/clhep`.

use crate::error::DnaError;
use std::ops::{Add, Div, Mul, Sub};

/// Avogadro constant, exact in the 2019 SI (mol^-1).
const AVOGADRO: f64 = 6.022_140_76e23;

macro_rules! quantity {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
        pub struct $name(f64);

        impl Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }
        impl Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }
        impl Mul<f64> for $name {
            type Output = Self;
            fn mul(self, rhs: f64) -> Self {
                Self(self.0 * rhs)
            }
        }
        impl Div<f64> for $name {
            type Output = Self;
            fn div(self, rhs: f64) -> Self {
                Self(self.0 / rhs)
            }
        }
        /// Ratio of two like quantities is dimensionless.
        impl Div for $name {
            type Output = f64;
            fn div(self, rhs: Self) -> f64 {
                self.0 / rhs.0
            }
        }
    };
}

quantity!(
    /// Energy, stored in eV.
    Energy
);
quantity!(
    /// Length, stored in nm.
    Length
);
quantity!(
    /// Time, stored in ps.
    Time
);
quantity!(
    /// Diffusion coefficient, stored in nm^2/ps.
    Diffusivity
);
quantity!(
    /// Bimolecular rate constant, stored in nm^3/ps.
    RateConstant
);

impl Energy {
    pub const fn from_ev(v: f64) -> Self {
        Self(v)
    }
    pub fn from_kev(v: f64) -> Self {
        Self(v * 1.0e3)
    }
    pub fn from_mev(v: f64) -> Self {
        Self(v * 1.0e6)
    }
    pub const fn ev(self) -> f64 {
        self.0
    }
    pub fn kev(self) -> f64 {
        self.0 / 1.0e3
    }
    pub fn mev(self) -> f64 {
        self.0 / 1.0e6
    }
}

impl Length {
    pub const fn from_nm(v: f64) -> Self {
        Self(v)
    }
    pub fn from_um(v: f64) -> Self {
        Self(v * 1.0e3)
    }
    pub fn from_mm(v: f64) -> Self {
        Self(v * 1.0e6)
    }
    pub const fn nm(self) -> f64 {
        self.0
    }
    pub fn um(self) -> f64 {
        self.0 / 1.0e3
    }
    pub fn mm(self) -> f64 {
        self.0 / 1.0e6
    }
}

impl Time {
    pub const fn from_ps(v: f64) -> Self {
        Self(v)
    }
    pub fn from_ns(v: f64) -> Self {
        Self(v * 1.0e3)
    }
    pub fn from_us(v: f64) -> Self {
        Self(v * 1.0e6)
    }
    pub fn from_s(v: f64) -> Self {
        Self(v * 1.0e12)
    }
    pub const fn ps(self) -> f64 {
        self.0
    }
    pub fn ns(self) -> f64 {
        self.0 / 1.0e3
    }
    pub fn us(self) -> f64 {
        self.0 / 1.0e6
    }
    pub fn s(self) -> f64 {
        self.0 / 1.0e12
    }
}

impl Diffusivity {
    pub const fn from_nm2_per_ps(v: f64) -> Self {
        Self(v)
    }
    /// 1 m^2/s = 1e18 nm^2 / 1e12 ps = 1e6 nm^2/ps.
    pub fn from_m2_per_s(v: f64) -> Self {
        Self(v * 1.0e6)
    }
    pub const fn nm2_per_ps(self) -> f64 {
        self.0
    }
}

impl RateConstant {
    pub const fn from_nm3_per_ps(v: f64) -> Self {
        Self(v)
    }
    /// Convert from M^-1 s^-1 (= L mol^-1 s^-1): 1 L = 1e-3 m^3 = 1e24 nm^3, divided by
    /// N_A per molecule, and 1 s = 1e12 ps. Gives 1.66054e-12 nm^3/ps per M^-1 s^-1.
    pub fn from_per_molar_per_s(v: f64) -> Self {
        Self(v * 1.0e24 / AVOGADRO / 1.0e12)
    }
    pub const fn nm3_per_ps(self) -> f64 {
        self.0
    }
}

/// Closed energy interval `[min, max]` over which a model is validated.
///
/// Models call [`EnergyRange::check`] instead of clamping: values outside the range are an
/// error, never silently extrapolated (AGENTS.md, scientific requirements).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyRange {
    min: Energy,
    max: Energy,
}

impl EnergyRange {
    /// Returns `None` unless `min <= max` (NaN bounds are rejected).
    pub fn new(min: Energy, max: Energy) -> Option<Self> {
        (min <= max).then_some(Self { min, max })
    }
    pub fn min(&self) -> Energy {
        self.min
    }
    pub fn max(&self) -> Energy {
        self.max
    }
    pub fn contains(&self, e: Energy) -> bool {
        self.min <= e && e <= self.max
    }
    pub fn check(&self, e: Energy) -> Result<(), DnaError> {
        if self.contains(e) {
            Ok(())
        } else {
            Err(DnaError::UnsupportedEnergy {
                energy_ev: e.ev(),
                min_ev: self.min.ev(),
                max_ev: self.max.ev(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use proptest::prelude::*;

    #[test]
    fn energy_prefixes() {
        assert_eq!(Energy::from_kev(1.0).ev(), 1.0e3);
        assert_eq!(Energy::from_mev(1.0).ev(), 1.0e6);
        assert_eq!(Energy::from_mev(1.0).kev(), 1.0e3);
        assert_eq!(Energy::from_ev(7.4).mev(), 7.4e-6);
    }

    #[test]
    fn length_prefixes() {
        assert_eq!(Length::from_um(1.0).nm(), 1.0e3);
        assert_eq!(Length::from_mm(1.0).nm(), 1.0e6);
        assert_relative_eq!(Length::from_nm(2.5e6).mm(), 2.5);
    }

    #[test]
    fn time_prefixes() {
        assert_eq!(Time::from_ns(1.0).ps(), 1.0e3);
        assert_eq!(Time::from_us(1.0).ps(), 1.0e6);
        assert_eq!(Time::from_s(1.0).ps(), 1.0e12);
        assert_relative_eq!(Time::from_ps(1.0e6).us(), 1.0);
    }

    #[test]
    fn same_dimension_arithmetic() {
        let e = Energy::from_ev(10.0) + Energy::from_ev(5.0) - Energy::from_ev(3.0);
        assert_eq!(e, Energy::from_ev(12.0));
        assert_eq!(e * 2.0, Energy::from_ev(24.0));
        assert_eq!(e / 4.0, Energy::from_ev(3.0));
        assert_eq!(e / Energy::from_ev(6.0), 2.0);
        assert!(Energy::from_ev(1.0) < Energy::from_ev(2.0));
    }

    /// 1 m^2/s = 1e6 nm^2/ps (docs/01 A.5). Water self-diffusion check: 2.3e-9 m^2/s.
    #[test]
    fn diffusivity_from_si() {
        assert_relative_eq!(Diffusivity::from_m2_per_s(1.0).nm2_per_ps(), 1.0e6);
        assert_relative_eq!(
            Diffusivity::from_m2_per_s(2.3e-9).nm2_per_ps(),
            2.3e-3,
            max_relative = 1e-12
        );
    }

    /// 1 M^-1 s^-1 = 1.66054e-12 nm^3/ps (docs/01 A.5), from the exact SI N_A.
    #[test]
    fn rate_constant_from_molar() {
        let k = RateConstant::from_per_molar_per_s(1.0);
        assert_relative_eq!(k.nm3_per_ps(), 1.66054e-12, max_relative = 1e-5);
        // Typical diffusion-limited rate 1e10 M^-1 s^-1 ~ 1.66e-2 nm^3/ps.
        assert_relative_eq!(
            RateConstant::from_per_molar_per_s(1.0e10).nm3_per_ps(),
            1.66054e-2,
            max_relative = 1e-5
        );
    }

    #[test]
    fn energy_range_enforces_bounds_without_extrapolation() {
        let r = EnergyRange::new(Energy::from_ev(7.4), Energy::from_mev(1.0)).unwrap();
        assert!(r.check(Energy::from_ev(7.4)).is_ok());
        assert!(r.check(Energy::from_mev(1.0)).is_ok());
        let err = r.check(Energy::from_mev(1.0001)).unwrap_err();
        assert_eq!(
            err,
            crate::error::DnaError::UnsupportedEnergy {
                energy_ev: 1_000_100.0,
                min_ev: 7.4,
                max_ev: 1.0e6
            }
        );
        assert!(r.check(Energy::from_ev(7.0)).is_err());
        assert!(r.check(Energy::from_ev(f64::NAN)).is_err());
    }

    #[test]
    fn energy_range_rejects_inverted_bounds() {
        assert!(EnergyRange::new(Energy::from_mev(1.0), Energy::from_ev(7.4)).is_none());
    }

    proptest! {
        #[test]
        fn energy_round_trips(x in 1e-6f64..1e9) {
            assert_relative_eq!(Energy::from_kev(x).kev(), x, max_relative = 1e-14);
            assert_relative_eq!(Energy::from_mev(x).mev(), x, max_relative = 1e-14);
        }

        #[test]
        fn length_round_trips(x in 1e-6f64..1e9) {
            assert_relative_eq!(Length::from_um(x).um(), x, max_relative = 1e-14);
            assert_relative_eq!(Length::from_mm(x).mm(), x, max_relative = 1e-14);
        }

        #[test]
        fn time_round_trips(x in 1e-6f64..1e9) {
            assert_relative_eq!(Time::from_ns(x).ns(), x, max_relative = 1e-14);
            assert_relative_eq!(Time::from_us(x).us(), x, max_relative = 1e-14);
            assert_relative_eq!(Time::from_s(x).s(), x, max_relative = 1e-14);
        }
    }
}
