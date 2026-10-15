//! Total/partial cross-section tables: `energy  sigma_1 ... sigma_n` files read by
//! `G4DNACrossSectionDataSet` (log-log) or by the model itself (Sanche, lin-lin).
//!
//! # Geant4 behaviour reproduced (audit G2, `docs/02-cpp-reference-inventory.md` B.7.1)
//! * Log-log: `G4DNACrossSectionDataSet::LoadData` (`dna/utils/src/G4DNACrossSectionDataSet.cc:83-240`)
//!   builds `log10` copies of every column in which values `<= 0` are replaced by `1e-300`
//!   (`:148`); `G4LogLogInterpolation::Calculate` (`lowenergy/src/G4LogLogInterpolation.cc:95-127`)
//!   then interpolates linearly in those logs. At an *interior* node with value 0 the result is
//!   therefore `1e-300 x scale`, not 0, exactly as in Geant4.
//! * Lin-lin (Sanche): `G4DNASancheExcitationModel::LinInterpolate`
//!   (`models/src/G4DNASancheExcitationModel.cc:407-418`) with the `t (1 - 1e-12)` protection at
//!   the last tabulated energy (`:290-296`).
//!
//! # Deliberate deviation (documented, tested)
//! `G4EMDataSet::FindValue` (`lowenergy/src/G4EMDataSet.cc:167-183`) returns the first/last node
//! value for energies outside the table. Here such a query is an
//! [`UnsupportedEnergy`](dna_core::error::DnaError::UnsupportedEnergy) error: the models' own
//! windows (docs/10) lie inside the tables except Melton (table ends at 12.3 eV, window 13 eV).

use crate::error::DataError;
use crate::parse::RawTable;
use dna_core::units::{Area, Energy, EnergyRange};
use std::path::Path;

/// Floor Geant4 substitutes for non-positive values before taking `log10` (`...DataSet.cc:148`).
const LOG_FLOOR: f64 = 1e-300;

/// Interpolation rule of a cross-section table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    /// `G4LogLogInterpolation` through `G4DNACrossSectionDataSet` (all total-cross-section files
    /// except Sanche).
    LogLog,
    /// Linear in energy and value (`G4DNASancheExcitationModel`).
    LinLin,
}

/// How to read a cross-section file: the value scale (what one unit of the file means, from
/// the loading model's `scaleFactor`) and the interpolation rule. Energies are always eV.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XsSpec {
    pub scale: Area,
    pub interpolation: Interpolation,
}

/// A read-only cross-section table with one energy column and `n_components()` value columns
/// (excitation levels, ionisation shells, vibrational levels, or a single total).
///
/// Immutable after construction, hence `Send + Sync`: share it with `Arc`.
#[derive(Debug, Clone)]
pub struct CrossSectionTable {
    interpolation: Interpolation,
    range: EnergyRange,
    /// Node energies, eV.
    energies: Vec<f64>,
    /// `components[c][i]`: component `c` at node `i`, nm^2.
    components: Vec<Vec<f64>>,
    /// Sum over components at each node (Sanche interpolates this sum), nm^2.
    node_sums: Vec<f64>,
    /// `log10` of the energy column (floored); log-log only.
    log_energies: Vec<f64>,
    /// `log10(max(value, 1e-300)) + log10(scale)` per component; log-log only.
    log_components: Vec<Vec<f64>>,
}

impl CrossSectionTable {
    /// Build from file text. `name` appears in error messages.
    pub fn from_text(name: &str, text: &str, spec: XsSpec) -> Result<Self, DataError> {
        Self::from_raw(name, &RawTable::parse(name, text)?, spec)
    }

    /// Read and build from a file.
    pub fn load(path: &Path, spec: XsSpec) -> Result<Self, DataError> {
        let text = crate::read_text(path)?;
        Self::from_text(&path.display().to_string(), &text, spec)
    }

    pub fn from_raw(name: &str, raw: &RawTable, spec: XsSpec) -> Result<Self, DataError> {
        let invalid = |message: String| DataError::InvalidTable {
            file: name.to_string(),
            message,
        };
        if raw.n_cols() < 2 || raw.n_rows() < 2 {
            return Err(invalid(format!(
                "need at least 2 columns and 2 rows, found {} x {}",
                raw.n_cols(),
                raw.n_rows()
            )));
        }
        let scale_nm2 = spec.scale.nm2();
        if !(scale_nm2 > 0.0 && scale_nm2.is_finite()) {
            return Err(invalid(format!("scale {scale_nm2} nm^2 must be positive")));
        }
        let energies: Vec<f64> = raw.rows().map(|r| r[0]).collect();
        if energies[0] < 0.0 || energies.windows(2).any(|w| w[1] <= w[0]) {
            return Err(invalid(
                "energies must be non-negative and strictly increasing".to_string(),
            ));
        }
        let n_comp = raw.n_cols() - 1;
        let mut components = vec![Vec::with_capacity(energies.len()); n_comp];
        for (i, row) in raw.rows().enumerate() {
            for (c, &v) in row[1..].iter().enumerate() {
                if v < 0.0 {
                    return Err(invalid(format!("negative value {v} in row {}", i + 1)));
                }
                components[c].push(v * scale_nm2);
            }
        }
        let node_sums: Vec<f64> = (0..energies.len())
            .map(|i| components.iter().map(|c| c[i]).sum())
            .collect();
        let (log_energies, log_components) = match spec.interpolation {
            Interpolation::LogLog => {
                let log_of = |v: f64| if v <= 0.0 { LOG_FLOOR } else { v }.log10();
                let le = energies.iter().map(|&e| log_of(e)).collect();
                let lc = (0..n_comp)
                    .map(|c| {
                        raw.rows()
                            .map(|r| log_of(r[c + 1]) + scale_nm2.log10())
                            .collect()
                    })
                    .collect();
                (le, lc)
            }
            Interpolation::LinLin => (Vec::new(), Vec::new()),
        };
        let range = EnergyRange::new(
            Energy::from_ev(energies[0]),
            Energy::from_ev(*energies.last().unwrap_or(&energies[0])),
        )
        .ok_or_else(|| invalid("invalid energy range".to_string()))?;
        Ok(Self {
            interpolation: spec.interpolation,
            range,
            energies,
            components,
            node_sums,
            log_energies,
            log_components,
        })
    }

    /// Closed energy interval covered by the table.
    pub fn energy_range(&self) -> EnergyRange {
        self.range
    }

    pub fn n_components(&self) -> usize {
        self.components.len()
    }

    pub fn n_nodes(&self) -> usize {
        self.energies.len()
    }

    pub fn interpolation(&self) -> Interpolation {
        self.interpolation
    }

    /// Tabulated node energies, eV.
    pub fn node_energies_ev(&self) -> &[f64] {
        &self.energies
    }

    /// Tabulated values of one component at the nodes (already scaled), nm^2.
    pub fn node_values_nm2(&self, component: usize) -> Result<&[f64], DataError> {
        self.components
            .get(component)
            .map(Vec::as_slice)
            .ok_or(DataError::ComponentOutOfRange {
                index: component,
                count: self.components.len(),
            })
    }

    /// Cross section of one component (level/shell) at `energy`.
    ///
    /// Equivalent to `G4DNACrossSectionDataSet::GetComponent(c)->FindValue(E)` inside the table;
    /// outside it returns `UnsupportedEnergy`.
    pub fn component(&self, component: usize, energy: Energy) -> Result<Area, DataError> {
        if component >= self.components.len() {
            return Err(DataError::ComponentOutOfRange {
                index: component,
                count: self.components.len(),
            });
        }
        self.range.check(energy)?;
        let x = energy.ev();
        Ok(Area::from_nm2(match self.interpolation {
            Interpolation::LogLog => self.log_log_value(component, x),
            Interpolation::LinLin => self.lin_lin_value(&self.components[component], x),
        }))
    }

    /// Sum over all components at `energy` (`G4DNACrossSectionDataSet::FindValue`; for the
    /// lin-lin Sanche table the interpolation of the node sums, as `TotalCrossSection`).
    pub fn total(&self, energy: Energy) -> Result<Area, DataError> {
        self.range.check(energy)?;
        let x = energy.ev();
        Ok(Area::from_nm2(match self.interpolation {
            Interpolation::LogLog => (0..self.components.len())
                .map(|c| self.log_log_value(c, x))
                .sum(),
            Interpolation::LinLin => self.lin_lin_value(&self.node_sums, x),
        }))
    }

    /// `G4EMDataSet::FindValue` for in-range `x`: first/last node exactly, else log-log between
    /// the bracketing nodes (`bin` = last node `<= x`, `G4EMDataSet::FindLowerBound`).
    fn log_log_value(&self, c: usize, x: f64) -> f64 {
        let n = self.energies.len();
        if x <= self.energies[0] {
            return self.components[c][0];
        }
        if x >= self.energies[n - 1] {
            return self.components[c][n - 1];
        }
        let bin = self.energies.partition_point(|&e| e <= x) - 1;
        let log_x = x.log10();
        let (le1, le2) = (self.log_energies[bin], self.log_energies[bin + 1]);
        let (ld1, ld2) = (self.log_components[c][bin], self.log_components[c][bin + 1]);
        10f64.powf(ld1 + (ld2 - ld1) * (log_x - le1) / (le2 - le1))
    }

    /// Sanche `PartialCrossSection`/`TotalCrossSection` interpolation of `values` at `x`.
    fn lin_lin_value(&self, values: &[f64], x: f64) -> f64 {
        let n = self.energies.len();
        // "Protection against out of boundary access" (Sanche.cc:290-296).
        let t = if x == self.energies[n - 1] {
            x * (1.0 - 1e-12)
        } else {
            x
        };
        let t2 = self.energies.partition_point(|&e| e <= t);
        let t1 = t2 - 1;
        let (e1, e2) = (self.energies[t1], self.energies[t2]);
        let (xs1, xs2) = (values[t1], values[t2]);
        let a = (xs2 - xs1) / (e2 - e1);
        let b = xs2 - a * e2;
        a * t + b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use dna_core::error::DnaError;

    fn loglog(scale_nm2: f64) -> XsSpec {
        XsSpec {
            scale: Area::from_nm2(scale_nm2),
            interpolation: Interpolation::LogLog,
        }
    }

    const TWO_COMP: &str = "10 1.0 4.0\n100 100.0 400.0\n1000 10.0 0.0\n";

    #[test]
    fn exact_nodes_and_scale() {
        let t = CrossSectionTable::from_text("t", TWO_COMP, loglog(0.5)).unwrap();
        assert_eq!(t.n_components(), 2);
        // First and last node: stored value exactly.
        assert_eq!(t.component(0, Energy::from_ev(10.0)).unwrap().nm2(), 0.5);
        assert_eq!(t.component(1, Energy::from_ev(1000.0)).unwrap().nm2(), 0.0);
        // Interior node: through the logs, accurate to rounding.
        assert_relative_eq!(
            t.component(0, Energy::from_ev(100.0)).unwrap().nm2(),
            50.0,
            max_relative = 1e-13
        );
    }

    #[test]
    fn log_log_hand_checked_midpoint() {
        // Between (10, 1) and (100, 100): power law y = x^2 / 100 -> y(sqrt(10*100)) = 10.
        let t = CrossSectionTable::from_text("t", TWO_COMP, loglog(1.0)).unwrap();
        let e = (10.0f64 * 100.0).sqrt();
        assert_relative_eq!(
            t.component(0, Energy::from_ev(e)).unwrap().nm2(),
            10.0,
            max_relative = 1e-12
        );
        // total = sum of components: 10 + 40.
        assert_relative_eq!(t.total(Energy::from_ev(e)).unwrap().nm2(), 50.0, max_relative = 1e-12);
    }

    #[test]
    fn interior_zero_node_reproduces_geant4_floor() {
        // Component 1 is 4, 400, 0 (last node zero) and component 0 is fine; build a table whose
        // *interior* node is zero: 1e-300 floor, not 0.
        let t = CrossSectionTable::from_text("t", "1 5.0\n10 0.0\n100 5.0\n", loglog(1.0)).unwrap();
        let at_zero = t.component(0, Energy::from_ev(10.0)).unwrap().nm2();
        assert!(at_zero > 0.0 && at_zero < 1e-290, "{at_zero}");
        // Log-midpoint of the rising interval (10, 100): log10 v = (-300 + log10 5)/2.
        let e = (10.0f64 * 100.0).sqrt();
        let expected = 10f64.powf((-300.0 + 5.0f64.log10()) / 2.0);
        assert_relative_eq!(
            t.component(0, Energy::from_ev(e)).unwrap().nm2(),
            expected,
            max_relative = 1e-9
        );
    }

    #[test]
    fn zero_energy_node_is_floored_like_geant4() {
        // Melton/Sanche tables start at E = 0; log10(0) would be -inf.
        let t = CrossSectionTable::from_text("t", "0 0.0\n4 0.0\n6 3.0\n", loglog(1.0)).unwrap();
        let v = t.component(0, Energy::from_ev(2.0)).unwrap().nm2();
        assert!(v.is_finite() && v < 1e-290, "{v}");
        assert_eq!(t.component(0, Energy::from_ev(0.0)).unwrap().nm2(), 0.0);
    }

    #[test]
    fn out_of_range_is_a_typed_error_never_clamped() {
        let t = CrossSectionTable::from_text("t", TWO_COMP, loglog(1.0)).unwrap();
        for e in [9.999, 1000.001, 0.0, 1e9] {
            let err = t.component(0, Energy::from_ev(e)).unwrap_err();
            assert_eq!(
                err,
                DataError::Core(DnaError::UnsupportedEnergy {
                    energy_ev: e,
                    min_ev: 10.0,
                    max_ev: 1000.0
                })
            );
            assert!(t.total(Energy::from_ev(e)).is_err());
        }
        assert!(t.component(0, Energy::from_ev(f64::NAN)).is_err());
    }

    #[test]
    fn bad_component_index() {
        let t = CrossSectionTable::from_text("t", TWO_COMP, loglog(1.0)).unwrap();
        assert_eq!(
            t.component(2, Energy::from_ev(50.0)).unwrap_err(),
            DataError::ComponentOutOfRange { index: 2, count: 2 }
        );
    }

    #[test]
    fn lin_lin_matches_sanche_formula() {
        let spec = XsSpec {
            scale: Area::from_nm2(2.0),
            interpolation: Interpolation::LinLin,
        };
        let t = CrossSectionTable::from_text("t", "0 0 1\n10 10 3\n30 50 3\n", spec).unwrap();
        // E = 5 in (0, 10): c0 = 5, c1 = 2 (x scale 2).
        assert_relative_eq!(t.component(0, Energy::from_ev(5.0)).unwrap().nm2(), 10.0, max_relative = 1e-14);
        assert_relative_eq!(t.component(1, Energy::from_ev(5.0)).unwrap().nm2(), 4.0, max_relative = 1e-14);
        // Node sums at 10 and 30 are 13 and 53: E = 20 -> 33 -> 66 nm^2.
        assert_relative_eq!(t.total(Energy::from_ev(20.0)).unwrap().nm2(), 66.0, max_relative = 1e-14);
        // First node (energy 0 is valid for lin-lin) and last node (1e-12 protection).
        assert_eq!(t.component(0, Energy::from_ev(0.0)).unwrap().nm2(), 0.0);
        assert_relative_eq!(t.component(0, Energy::from_ev(30.0)).unwrap().nm2(), 100.0, max_relative = 1e-11);
    }

    #[test]
    fn rejects_structurally_invalid_tables() {
        let bad = |text: &str| CrossSectionTable::from_text("t", text, loglog(1.0)).unwrap_err();
        assert!(matches!(bad("1 2\n"), DataError::InvalidTable { .. }));
        assert!(matches!(bad("1\n2\n3\n"), DataError::InvalidTable { .. }));
        assert!(matches!(bad("1 2\n1 3\n"), DataError::InvalidTable { .. }));
        assert!(matches!(bad("2 2\n1 3\n"), DataError::InvalidTable { .. }));
        assert!(matches!(bad("1 -2\n2 3\n"), DataError::InvalidTable { .. }));
    }
}
