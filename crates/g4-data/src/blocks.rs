//! Differential and cumulative tables organised in blocks of constant incident energy.
//!
//! Three file layouts exist (column meaning transcribed from the loaders, see docs/10):
//!
//! | layout | columns | files |
//! |---|---|---|
//! | [`AngularCdfTable`] | `E [eV]`, `P` (cumulative probability, 0..1), `theta [deg]` | `sigmadiff_cumulated_elastic_e_champion_hp`, `sigmadiff_cumulated_elastic_proton_HTran` |
//! | [`IonisationCdfTable`] | `E [eV]`, `W [eV]` (energy transfer incl. binding), 5 x cumulative probability per shell | `sigmadiff_cumulated_ionisation_{e_born_hp,p_born_hp,p_RPWBA}` |
//! | [`IonisationDcsTable`] | `E [eV]`, `W [eV]`, 5 x `d sigma / dW` per shell | `sigmadiff_ionisation_{e_born,p_born,p_RPWBA}` |
//!
//! The lookups below are the *deterministic* table functions of the Geant4 models
//! (`Theta`, `TransferedEnergy`); drawing the random argument is sampling and belongs to M2.

use crate::error::DataError;
use crate::interp::{self, Scheme};
use crate::parse::RawTable;
use g4_core::units::{Energy, EnergyRange};
use std::path::Path;

/// Number of water ionisation shells in the differential files (`for (j<5)` in the loaders).
pub const N_SHELLS: usize = 5;

/// `k (1 - 1e-12)` protection at the last tabulated energy (`G4DNAIonElasticModel.cc:385`,
/// `G4DNABornIonisationModel1.cc:1034, 1146`).
const LAST_ENERGY_PROTECTION: f64 = 1.0 - 1e-12;

/// All rows sharing one incident energy.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    keys: Vec<f64>,
    cols: Vec<Vec<f64>>,
}

impl Block {
    /// The second column of the file (probability or energy transfer), strictly increasing.
    pub fn keys(&self) -> &[f64] {
        &self.keys
    }
    /// Value column `j` (third column of the file is `column(0)`).
    pub fn column(&self, j: usize) -> &[f64] {
        &self.cols[j]
    }
}

/// A table split into blocks of equal first-column value, energies strictly increasing.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockTable {
    energies: Vec<f64>,
    blocks: Vec<Block>,
}

impl BlockTable {
    /// Parse `text` expecting `E key v_1 ... v_n` rows with `n == n_value_cols`.
    pub fn from_text(name: &str, text: &str, n_value_cols: usize) -> Result<Self, DataError> {
        Self::from_raw(name, &RawTable::parse(name, text)?, n_value_cols)
    }

    pub fn from_raw(name: &str, raw: &RawTable, n_value_cols: usize) -> Result<Self, DataError> {
        let invalid = |message: String| DataError::InvalidTable {
            file: name.to_string(),
            message,
        };
        if Some(raw.n_cols()) != n_value_cols.checked_add(2) {
            return Err(invalid(format!(
                "expected two key columns and {n_value_cols} value columns, found {} columns",
                raw.n_cols()
            )));
        }
        let mut energies: Vec<f64> = Vec::new();
        let mut blocks: Vec<Block> = Vec::new();
        for (i, row) in raw.rows().enumerate() {
            let e = row[0];
            if row[1..].iter().any(|&v| v < 0.0) {
                return Err(invalid(format!("negative value in row {}", i + 1)));
            }
            if energies.last() != Some(&e) {
                if e <= 0.0 || energies.last().is_some_and(|&prev| e < prev) {
                    return Err(invalid(format!(
                        "row {}: incident energies must be positive, grouped and strictly increasing",
                        i + 1
                    )));
                }
                energies.push(e);
                blocks.push(Block {
                    keys: Vec::new(),
                    cols: vec![Vec::new(); n_value_cols],
                });
            }
            let block = blocks.last_mut().expect("block pushed above");
            if block.keys.last().is_some_and(|&prev| row[1] <= prev) {
                return Err(invalid(format!(
                    "row {}: second column must be strictly increasing inside an energy block",
                    i + 1
                )));
            }
            block.keys.push(row[1]);
            for (col, &v) in block.cols.iter_mut().zip(&row[2..]) {
                col.push(v);
            }
        }
        if blocks.len() < 2 {
            return Err(invalid("need at least two incident energies".to_string()));
        }
        Ok(Self { energies, blocks })
    }

    /// Incident energies of the blocks, eV, strictly increasing.
    pub fn energies_ev(&self) -> &[f64] {
        &self.energies
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    fn require_columns(&self, name: &str, expected: usize) -> Result<(), DataError> {
        for (index, block) in self.blocks.iter().enumerate() {
            if block.cols.len() != expected {
                return Err(DataError::InvalidTable {
                    file: name.to_string(),
                    message: format!(
                        "block {index}: expected {expected} value columns, found {}",
                        block.cols.len()
                    ),
                });
            }
        }
        Ok(())
    }

    pub fn energy_range(&self) -> EnergyRange {
        EnergyRange::new(
            Energy::from_ev(self.energies[0]),
            Energy::from_ev(self.energies[self.energies.len() - 1]),
        )
        .expect("energies are strictly increasing")
    }
}

/// Geant4 energy bracketing: `i2` is the first block with energy `> k`, `i1 = i2 - 1`; at the
/// last tabulated energy `k` is first lowered by `1e-12` relative. Returns `(i1, i2, k)`.
fn bracket_energy(
    energies: &[f64],
    range: &EnergyRange,
    energy: Energy,
) -> Result<(usize, usize, f64), DataError> {
    range.check(energy)?;
    let mut k = energy.ev();
    if k == energies[energies.len() - 1] {
        k *= LAST_ENERGY_PROTECTION;
    }
    let i2 = energies.partition_point(|&e| e <= k);
    // The relative endpoint adjustment need not fit inside a very narrow grid.
    if i2 == 0 || i2 == energies.len() {
        return Err(DataError::UndefinedLookup(format!(
            "adjusted energy {k} eV has no interpolation interval"
        )));
    }
    Ok((i2 - 1, i2, k))
}

fn check_probability(p: f64) -> Result<(), DataError> {
    if (0.0..=1.0).contains(&p) {
        Ok(())
    } else {
        Err(DataError::InvalidProbability(p))
    }
}

/// Bracketing nodes `(lo, hi)` of `p` in a node list sorted by probability: `hi` is the first
/// node with key `> p`. When `p` equals the last key Geant4 reads past the vector end; here the
/// last interval is used (continuity). `None` if `p` lies below the first node or the list has
/// fewer than two nodes.
fn bracket_nodes(nodes: &[(f64, f64)], p: f64) -> Option<((f64, f64), (f64, f64))> {
    let hi = nodes
        .partition_point(|n| n.0 <= p)
        .min(nodes.len().checked_sub(1)?);
    if hi == 0 {
        return None;
    }
    Some((nodes[hi - 1], nodes[hi]))
}

/// Cumulative angular distributions: polar angle as a function of incident energy and
/// cumulative probability.
///
/// Used by `G4DNAChampionElasticModel` (lab angle of the scattered electron, `Theta` at
/// `...ChampionElasticModel.cc:316-378`, 2-D lin-lin `QuadInterpolator` `:411-445`) and
/// `G4DNAIonElasticModel` (centre-of-mass angle, `...IonElasticModel.cc:369-495`). Geant4 prepends
/// the sentinel node `(P = 0, theta = 0)` to every block (`:176`); a file whose first row is
/// `P = 0` overrides the sentinel value, as the maps do in C++.
#[derive(Debug, Clone)]
pub struct AngularCdfTable {
    energies: Vec<f64>,
    range: EnergyRange,
    /// Per block: `(P, theta_deg)` nodes, first `P` is 0, last is 1.
    nodes: Vec<Vec<(f64, f64)>>,
}

impl AngularCdfTable {
    pub fn from_text(name: &str, text: &str) -> Result<Self, DataError> {
        Self::from_blocks(name, &BlockTable::from_text(name, text, 1)?)
    }

    pub fn load(path: &Path) -> Result<Self, DataError> {
        let text = crate::read_text(path)?;
        Self::from_text(&path.display().to_string(), &text)
    }

    pub fn from_blocks(name: &str, table: &BlockTable) -> Result<Self, DataError> {
        table.require_columns(name, 1)?;
        let invalid = |message: String| DataError::InvalidTable {
            file: name.to_string(),
            message,
        };
        let mut nodes = Vec::with_capacity(table.blocks.len());
        for (b, block) in table.blocks.iter().enumerate() {
            let e = table.energies[b];
            let keys = block.keys();
            let angles = block.column(0);
            if keys[0] < 0.0 || keys[keys.len() - 1] != 1.0 {
                return Err(invalid(format!(
                    "E = {e} eV: cumulative probability must start >= 0 and end at exactly 1"
                )));
            }
            if angles.windows(2).any(|w| w[1] < w[0]) || angles.iter().any(|&a| a > 180.0) {
                return Err(invalid(format!(
                    "E = {e} eV: angles must be non-decreasing and within [0, 180] degrees"
                )));
            }
            let mut v: Vec<(f64, f64)> = Vec::with_capacity(keys.len() + 1);
            if keys[0] > 0.0 {
                v.push((0.0, 0.0));
            }
            v.extend(keys.iter().copied().zip(angles.iter().copied()));
            if v.len() < 2 {
                return Err(invalid(format!("E = {e} eV: fewer than two nodes")));
            }
            nodes.push(v);
        }
        Ok(Self {
            energies: table.energies.clone(),
            range: table.energy_range(),
            nodes,
        })
    }

    pub fn energy_range(&self) -> EnergyRange {
        self.range
    }

    pub fn energies_ev(&self) -> &[f64] {
        &self.energies
    }

    /// Polar angle in degrees for incident `energy` and cumulative probability `p` in `[0, 1]`
    /// (`Theta(k, integrDiff)`): for each of the two bracketing energies find the probability
    /// interval containing `p`, interpolate the angle linearly in `p`, then linearly in energy.
    /// Returns exactly `0` if all four bracketing angles are zero, as Geant4.
    ///
    /// Energies outside the table give `UnsupportedEnergy` (Geant4 reads out of bounds there).
    pub fn angle_deg(&self, energy: Energy, p: f64) -> Result<f64, DataError> {
        check_probability(p)?;
        let (i1, i2, k) = bracket_energy(&self.energies, &self.range, energy)?;
        let n1 = bracket_nodes(&self.nodes[i1], p).expect("first node has P = 0");
        let n2 = bracket_nodes(&self.nodes[i2], p).expect("first node has P = 0");
        if n1.0.1 == 0.0 && n1.1.1 == 0.0 && n2.0.1 == 0.0 && n2.1.1 == 0.0 {
            return Ok(0.0);
        }
        Ok(interp::quad(
            Scheme::LinLin,
            (n1.0.0, n1.1.0, n1.0.1, n1.1.1),
            (n2.0.0, n2.1.0, n2.0.1, n2.1.1),
            (self.energies[i1], self.energies[i2]),
            k,
            p,
        ))
    }
}

/// Cumulative energy-transfer distributions for the five water ionisation shells
/// ("hp"/fast tables of `G4DNABornIonisationModel1`, `SetDNAFast(true)`).
///
/// For every (energy, shell) the file lists `W` against the cumulative probability `P`. Plateaus
/// (`P = 0` below the threshold, `P = 2` padding above the kinematic limit) occur; Geant4 keeps in
/// its inverse map `W(P)` the *last* `W` of each equal-`P` run (`eNrjTransfData[j][t][P] = W`,
/// `G4DNABornIonisationModel1.cc:192-195`), which this type reproduces.
#[derive(Debug, Clone)]
pub struct IonisationCdfTable {
    energies: Vec<f64>,
    range: EnergyRange,
    /// `[block][shell]`: distinct `(P, W)` nodes with `P` increasing.
    nodes: Vec<Vec<Vec<(f64, f64)>>>,
}

impl IonisationCdfTable {
    pub fn from_text(name: &str, text: &str) -> Result<Self, DataError> {
        Self::from_blocks(name, &BlockTable::from_text(name, text, N_SHELLS)?)
    }

    pub fn load(path: &Path) -> Result<Self, DataError> {
        let text = crate::read_text(path)?;
        Self::from_text(&path.display().to_string(), &text)
    }

    pub fn from_blocks(name: &str, table: &BlockTable) -> Result<Self, DataError> {
        table.require_columns(name, N_SHELLS)?;
        let mut nodes = Vec::with_capacity(table.blocks.len());
        for (b, block) in table.blocks.iter().enumerate() {
            let mut shells = Vec::with_capacity(N_SHELLS);
            for s in 0..N_SHELLS {
                let cum = block.column(s);
                if cum.windows(2).any(|w| w[1] < w[0]) {
                    return Err(DataError::InvalidTable {
                        file: name.to_string(),
                        message: format!(
                            "E = {} eV, shell {s}: cumulative probability decreases",
                            table.energies[b]
                        ),
                    });
                }
                let mut distinct: Vec<(f64, f64)> = Vec::new();
                for (&p, &w) in cum.iter().zip(block.keys()) {
                    match distinct.last_mut() {
                        Some(last) if last.0 == p => last.1 = w,
                        _ => distinct.push((p, w)),
                    }
                }
                shells.push(distinct);
            }
            nodes.push(shells);
        }
        Ok(Self {
            energies: table.energies.clone(),
            range: table.energy_range(),
            nodes,
        })
    }

    pub fn energy_range(&self) -> EnergyRange {
        self.range
    }

    pub fn energies_ev(&self) -> &[f64] {
        &self.energies
    }

    /// Energy transfer `W` in eV (including the binding energy: callers subtract it to get the
    /// secondary's kinetic energy) for incident `energy`, ionisation `shell` (0..5) and cumulative
    /// probability `p` in `[0, 1]`: `G4DNABornIonisationModel1::TransferedEnergy`
    /// (`...Model1.cc:1012-1290`).
    ///
    /// Geant4 branches reproduced, with `back(k)` the largest tabulated `P` of the shell:
    /// * `p <= back(k1)` and `p <= back(k2)`: bracket `p` in both blocks and apply the log-lin
    ///   2-D interpolation; `0` if any of the four bracketing `W` is zero.
    /// * `p > back(k1)` (shell closed at `k1`): interpolate `W` in block `k2` only, then scale
    ///   linearly in energy from `W = 0` at `k1`.
    /// * `p <= back(k1)` but `p > back(k2)`: Geant4 falls through and returns `0`.
    ///
    /// # Errors
    /// `UnsupportedEnergy` outside the table; `InvalidProbability`; `UndefinedLookup` where Geant4
    /// would read past a vector (`p` above every node of the only usable block, or below the first
    /// node).
    pub fn transferred_energy_ev(
        &self,
        shell: usize,
        energy: Energy,
        p: f64,
    ) -> Result<f64, DataError> {
        if shell >= N_SHELLS {
            return Err(DataError::ComponentOutOfRange {
                index: shell,
                count: N_SHELLS,
            });
        }
        check_probability(p)?;
        let (i1, i2, k) = bracket_energy(&self.energies, &self.range, energy)?;
        let (k1, k2) = (self.energies[i1], self.energies[i2]);
        let (u1, u2) = (&self.nodes[i1][shell], &self.nodes[i2][shell]);
        let undefined = |block: f64| {
            DataError::UndefinedLookup(format!(
                "shell {shell}, E = {block} eV: probability {p} is not bracketed by the table"
            ))
        };
        let back1 = u1[u1.len() - 1].0;
        let back2 = u2[u2.len() - 1].0;

        if p <= back1 && p <= back2 {
            let n1 = bracket_nodes(u1, p).ok_or_else(|| undefined(k1))?;
            let n2 = bracket_nodes(u2, p).ok_or_else(|| undefined(k2))?;
            let (w11, w12, w21, w22) = (n1.0.1, n1.1.1, n2.0.1, n2.1.1);
            if w11 * w12 * w21 * w22 == 0.0 {
                return Ok(0.0);
            }
            return Ok(interp::quad(
                Scheme::LogLinOrLin,
                (n1.0.0, n1.1.0, w11, w12),
                (n2.0.0, n2.1.0, w21, w22),
                (k1, k2),
                k,
                p,
            ));
        }
        if p > back1 {
            // Shell closed at k1 ("zeros are explicitly set"): Born1.cc:1100-1135.
            if p > back2 {
                return Err(undefined(k2));
            }
            let n2 = bracket_nodes(u2, p).ok_or_else(|| undefined(k2))?;
            let w = interp::log_lin_or_lin(n2.0.0, n2.1.0, p, n2.0.1, n2.1.1);
            return Ok(interp::log_lin_or_lin(k1, k2, k, 0.0, w));
        }
        Ok(0.0)
    }
}

/// Differential cross sections `d sigma / dW` per shell (slow path of
/// `G4DNABornIonisationModel1`, `SetDNAFast(false)`; kept for the Q4 equivalence check).
///
/// Values are the raw numbers of the file. Geant4 multiplies them by the model's scale factor
/// (`(1e-22/3.343) m^2` for Born, `1 cm^2` for RPWBA, `Born1.cc:197-198`) and only ever uses them
/// as relative weights in a rejection sampling, so the absolute unit is not needed by M1-M2 and is
/// not asserted here. The 2-D lookup of this slow path is M2 work.
#[derive(Debug, Clone, PartialEq)]
pub struct IonisationDcsTable {
    table: BlockTable,
}

impl IonisationDcsTable {
    pub fn from_text(name: &str, text: &str) -> Result<Self, DataError> {
        Ok(Self {
            table: BlockTable::from_text(name, text, N_SHELLS)?,
        })
    }

    pub fn load(path: &Path) -> Result<Self, DataError> {
        let text = crate::read_text(path)?;
        Self::from_text(&path.display().to_string(), &text)
    }

    pub fn blocks(&self) -> &BlockTable {
        &self.table
    }

    pub fn energy_range(&self) -> EnergyRange {
        self.table.energy_range()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use proptest::prelude::*;

    /// Two energies, three probability nodes each (first row P = 0 as in the real files).
    const ANGLES: &str = "\
10 0.0 0.0
10 0.5 20.0
10 1.0 40.0
20 0.0 0.0
20 0.25 10.0
20 1.0 70.0
";

    fn angles() -> AngularCdfTable {
        AngularCdfTable::from_text("t", ANGLES).unwrap()
    }

    #[test]
    fn groups_blocks_and_rejects_bad_layout() {
        let t = BlockTable::from_text("t", ANGLES, 1).unwrap();
        assert_eq!(t.energies_ev(), &[10.0, 20.0]);
        assert_eq!(t.blocks()[1].keys(), &[0.0, 0.25, 1.0]);
        assert_eq!(t.blocks()[1].column(0), &[0.0, 10.0, 70.0]);
        // wrong number of value columns
        assert!(BlockTable::from_text("t", ANGLES, 2).is_err());
        // energy block reappears / goes down
        assert!(BlockTable::from_text("t", "10 0 1\n20 0 1\n10 0 1\n", 1).is_err());
        // key not increasing inside a block
        assert!(BlockTable::from_text("t", "10 0 1\n10 0 2\n20 0 1\n", 1).is_err());
        // single energy
        assert!(BlockTable::from_text("t", "10 0 1\n10 1 2\n", 1).is_err());
    }

    #[test]
    fn angle_exact_node_and_hand_values() {
        let t = angles();
        // E = 10 (block node), p = 0.25: halfway between (0, 0) and (0.5, 20) -> 10 degrees.
        assert_relative_eq!(
            t.angle_deg(Energy::from_ev(10.0), 0.25).unwrap(),
            10.0,
            epsilon = 0.0,
            max_relative = 1e-14
        );
        // E = 15: block 10 gives 10, block 20 at p=0.25 is exactly its node 10 -> mean 10.
        assert_relative_eq!(
            t.angle_deg(Energy::from_ev(15.0), 0.25).unwrap(),
            10.0,
            epsilon = 0.0,
            max_relative = 1e-14
        );
        // E = 15, p = 0.75: block 10 -> 30; block 20: 10 + (0.75-0.25)/0.75*60 = 50 -> 40.
        assert_relative_eq!(
            t.angle_deg(Energy::from_ev(15.0), 0.75).unwrap(),
            40.0,
            epsilon = 0.0,
            max_relative = 1e-14
        );
        // p = 0 selects the (P = 0) node of both blocks, whose angle is 0.
        assert_eq!(t.angle_deg(Energy::from_ev(15.0), 0.0).unwrap(), 0.0);
    }

    #[test]
    fn angle_at_last_energy_and_p_one_use_continuity() {
        let t = angles();
        // Last energy: Geant4 lowers k by 1e-12 relative; the value is the block-20 value to 1e-11.
        assert_relative_eq!(
            t.angle_deg(Energy::from_ev(20.0), 0.625).unwrap(),
            40.0,
            epsilon = 0.0,
            max_relative = 1e-10
        );
        // p = 1 reads past the end in C++; here it is the end node of the last interval.
        assert_relative_eq!(
            t.angle_deg(Energy::from_ev(10.0), 1.0).unwrap(),
            40.0,
            epsilon = 0.0,
            max_relative = 1e-14
        );
    }

    #[test]
    fn angle_rejects_out_of_range_arguments() {
        let t = angles();
        for e in [9.999, 20.001] {
            assert!(matches!(
                t.angle_deg(Energy::from_ev(e), 0.5),
                Err(DataError::Core(
                    g4_core::error::DnaError::UnsupportedEnergy { .. }
                ))
            ));
        }
        for p in [-0.001, 1.001, f64::NAN] {
            assert!(matches!(
                t.angle_deg(Energy::from_ev(15.0), p),
                Err(DataError::InvalidProbability(_))
            ));
        }
    }

    #[test]
    fn angular_table_validation() {
        // last probability must be exactly 1
        assert!(AngularCdfTable::from_text("t", "10 0 0\n10 0.9 5\n20 0 0\n20 1 5\n").is_err());
        // angle decreasing
        assert!(AngularCdfTable::from_text("t", "10 0 0\n10 1 5\n20 0 9\n20 1 5\n").is_err());
        // angle beyond 180
        assert!(AngularCdfTable::from_text("t", "10 0 0\n10 1 181\n20 0 0\n20 1 5\n").is_err());
        // first probability above 0 gets the (0, 0) sentinel node
        let t = AngularCdfTable::from_text("t", "10 0.5 8\n10 1 10\n20 0.5 8\n20 1 10\n").unwrap();
        assert_relative_eq!(
            t.angle_deg(Energy::from_ev(10.0), 0.25).unwrap(),
            4.0,
            epsilon = 0.0,
            max_relative = 1e-14
        );
    }

    /// Two energies, one shell varies. Shell 0 carries the hand-checked data; other shells copy it.
    fn cdf_text() -> String {
        // columns: E W cum0..cum4 (cum of shells 1-4 equal shell 0 except shell 4: closed at E=10)
        let rows10 = [
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.25),
            (4.0, 0.5),
            (5.0, 1.0),
            (6.0, 2.0),
            (7.0, 2.0),
        ];
        let rows20 = [(1.0, 0.0), (2.0, 0.1), (4.0, 0.6), (8.0, 1.0), (9.0, 2.0)];
        let mut s = String::new();
        for (e, rows, closed) in [(10.0, &rows10[..], true), (20.0, &rows20[..], false)] {
            for &(w, p) in rows {
                let p4 = if closed { 0.0 } else { p };
                s += &format!("{e} {w} {p} {p} {p} {p} {p4}\n");
            }
        }
        s
    }

    #[test]
    fn inverse_map_keeps_last_w_of_each_probability_plateau() {
        let t = IonisationCdfTable::from_text("t", &cdf_text()).unwrap();
        // E = 10: P = 0 plateau covers W = 1, 2 -> map keeps W = 2; P = 2 plateau keeps W = 7.
        // p = 0.125 lies between (0, 2) and (0.25, 3): log-lin W = 10^(log 2 + (log 3 - log 2)/2).
        let w = t
            .transferred_energy_ev(0, Energy::from_ev(10.0), 0.125)
            .unwrap();
        assert_relative_eq!(
            w,
            (2.0f64 * 3.0).sqrt(),
            epsilon = 0.0,
            max_relative = 1e-12
        );
        // p = 0 returns (log-lin with zero?) both nodes non-zero -> W at the interval start: 2.
        let w0 = t
            .transferred_energy_ev(0, Energy::from_ev(10.0), 0.0)
            .unwrap();
        assert_relative_eq!(w0, 2.0, epsilon = 0.0, max_relative = 1e-12);
    }

    #[test]
    fn transferred_energy_between_two_blocks_is_log_lin_in_energy() {
        let t = IonisationCdfTable::from_text("t", &cdf_text()).unwrap();
        // p = 0.5: block 10 hits node (0.5, 4) -> bracket (0.25,3)-(0.5? no): upper_bound(0.5)
        // gives the first node > 0.5, i.e. (1.0, 5); lower is (0.5, 4) -> W = 4 at weight 0.
        // block 20 brackets (0.1, 2)-(0.6, 4): W = 10^(log 2 + (log 4 - log 2)*0.4/0.5).
        let w1 = 4.0f64;
        let w2 = 2.0f64 * (2.0f64).powf(0.8);
        let expect = 10f64.powf(w1.log10() + (w2.log10() - w1.log10()) * (15.0 - 10.0) / 10.0);
        let got = t
            .transferred_energy_ev(0, Energy::from_ev(15.0), 0.5)
            .unwrap();
        assert_relative_eq!(got, expect, epsilon = 0.0, max_relative = 1e-12);
    }

    #[test]
    fn closed_shell_scales_from_zero_and_gaps_fall_through() {
        let t = IonisationCdfTable::from_text("t", &cdf_text()).unwrap();
        // Shell 4 is closed at E = 10 (all P = 0): p = 0.3 > back(10) = 0 -> use block 20 only.
        // block 20 shell 4: (0.1, 2)-(0.6, 4) at p = 0.3: 10^(log2 + log2 * 0.2/0.5), then
        // linear scaling from 0 at E = 10 to this value at E = 20; E = 12.5 -> 25 %.
        let w2 = 2.0f64 * 2.0f64.powf(0.4);
        let got = t
            .transferred_energy_ev(4, Energy::from_ev(12.5), 0.3)
            .unwrap();
        assert_relative_eq!(got, 0.25 * w2, epsilon = 0.0, max_relative = 1e-12);
        // Shell 4 open at 20 but p above its back at the *upper* block cannot happen here; a
        // shell closed at both blocks is undefined:
        let closed = "10 1 0 0 0 0 0\n10 2 1 1 1 1 0\n20 1 0 0 0 0 0\n20 2 1 1 1 1 0\n";
        let t = IonisationCdfTable::from_text("t", closed).unwrap();
        assert!(matches!(
            t.transferred_energy_ev(4, Energy::from_ev(15.0), 0.5),
            Err(DataError::UndefinedLookup(_))
        ));
    }

    #[test]
    fn ionisation_cdf_validation_and_arguments() {
        // cumulative probability must not decrease
        assert!(
            IonisationCdfTable::from_text(
                "t",
                "10 1 1 0 0 0 0\n10 2 0.5 0 0 0 0\n20 1 0 0 0 0 0\n20 2 1 1 0 0 0\n"
            )
            .is_err()
        );
        let t = IonisationCdfTable::from_text("t", &cdf_text()).unwrap();
        assert!(matches!(
            t.transferred_energy_ev(5, Energy::from_ev(15.0), 0.5),
            Err(DataError::ComponentOutOfRange { index: 5, count: 5 })
        ));
        assert!(
            t.transferred_energy_ev(0, Energy::from_ev(9.0), 0.5)
                .is_err()
        );
        assert!(
            t.transferred_energy_ev(0, Energy::from_ev(21.0), 0.5)
                .is_err()
        );
        assert!(matches!(
            t.transferred_energy_ev(0, Energy::from_ev(15.0), 1.5),
            Err(DataError::InvalidProbability(_))
        ));
        // Upper end of the table works through the 1e-12 protection.
        assert!(
            t.transferred_energy_ev(0, Energy::from_ev(20.0), 0.5)
                .is_ok()
        );
    }

    #[test]
    fn dcs_table_parses_and_exposes_blocks() {
        let text = "10 1.0 0 0 0 0 1\n10 2.0 1 1 1 1 0\n20 1.0 0 0 0 0 0\n20 2.0 2 2 2 2 2\n";
        let t = IonisationDcsTable::from_text("t", text).unwrap();
        assert_eq!(t.blocks().blocks()[1].column(4), &[0.0, 2.0]);
        assert!(IonisationDcsTable::from_text("t", "10 1 0\n").is_err());
    }

    #[test]
    fn from_blocks_rejects_missing_and_extra_value_columns() {
        for count in 0..=6 {
            let values = " 1".repeat(count);
            let text = format!("1 0{values}\n1 1{values}\n2 0{values}\n2 1{values}\n");
            let blocks = BlockTable::from_text("shape", &text, count).unwrap();
            if count != 1 {
                assert!(matches!(
                    AngularCdfTable::from_blocks("shape", &blocks),
                    Err(DataError::InvalidTable { .. })
                ));
            }
            if count != N_SHELLS {
                assert!(matches!(
                    IonisationCdfTable::from_blocks("shape", &blocks),
                    Err(DataError::InvalidTable { .. })
                ));
            }
        }
    }

    #[test]
    fn narrow_block_endpoint_is_a_typed_error() {
        assert_narrow_endpoint(1.0000000000001);
    }

    fn assert_narrow_endpoint(upper: f64) {
        let angle = AngularCdfTable::from_text(
            "narrow",
            &format!("1 0 0\n1 1 90\n{upper} 0 0\n{upper} 1 180\n"),
        )
        .unwrap();
        let cdf = IonisationCdfTable::from_text(
            "narrow",
            &format!("1 1 0 0 0 0 0\n1 2 1 1 1 1 1\n{upper} 1 0 0 0 0 0\n{upper} 3 1 1 1 1 1\n"),
        )
        .unwrap();
        assert!(matches!(
            angle.angle_deg(Energy::from_ev(upper), 0.5),
            Err(DataError::UndefinedLookup(_))
        ));
        assert!(matches!(
            cdf.transferred_energy_ev(0, Energy::from_ev(upper), 0.5),
            Err(DataError::UndefinedLookup(_))
        ));
    }

    /// Strategy: probability nodes `0 < ... < 1` with non-decreasing angles in `[0, 180]`,
    /// bracketed by the `(0, 0)` and `(1, 180)` end nodes.
    fn block_strategy() -> impl Strategy<Value = Vec<(f64, f64)>> {
        prop::collection::vec((0.001f64..0.999, 0.0f64..180.0), 1..30).prop_map(|mut v| {
            v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            v.dedup_by(|a, b| a.0 == b.0);
            let mut angles: Vec<f64> = v.iter().map(|x| x.1).collect();
            angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut out = vec![(0.0, 0.0)];
            out.extend(v.iter().map(|x| x.0).zip(angles));
            out.push((1.0, 180.0));
            out
        })
    }

    proptest! {
        #[test]
        fn narrow_block_upper_endpoints_never_panic(ulps in 1_u64..4000) {
            assert_narrow_endpoint(f64::from_bits(1.0_f64.to_bits() + ulps));
        }

        #[test]
        fn malformed_block_shapes_are_typed_errors(count in 0_usize..20) {
            let values = " 1".repeat(count);
            let text = format!("1 0{values}\n1 1{values}\n2 0{values}\n2 1{values}\n");
            let blocks = BlockTable::from_text("prop", &text, count).unwrap();
            prop_assert_eq!(AngularCdfTable::from_blocks("prop", &blocks).is_ok(), count == 1);
            prop_assert_eq!(IonisationCdfTable::from_blocks("prop", &blocks).is_ok(), count == N_SHELLS);
        }

        /// Whatever valid blocks are tabulated, the angle is non-decreasing in the cumulative
        /// probability for every incident energy, and stays inside the tabulated angle range.
        #[test]
        fn angle_is_a_monotone_function_of_cumulative_probability(
            b1 in block_strategy(),
            b2 in block_strategy(),
            frac in 0.0f64..1.0,
            mut ps in prop::collection::vec(0.0f64..=1.0, 2..30),
        ) {
            let mut text = String::new();
            for &(p, a) in &b1 { text += &format!("100 {p:e} {a:e}\n"); }
            for &(p, a) in &b2 { text += &format!("200 {p:e} {a:e}\n"); }
            let t = AngularCdfTable::from_text("prop", &text).unwrap();
            let e = Energy::from_ev(100.0 + 100.0 * frac);
            ps.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut prev = f64::NEG_INFINITY;
            for &p in &ps {
                let a = t.angle_deg(e, p).unwrap();
                prop_assert!(a >= prev - 1e-9, "angle decreased: {prev} -> {a} at p={p}");
                prop_assert!((-1e-9..=180.0 + 1e-9).contains(&a));
                prev = a;
            }
        }

        /// Parsing is independent of the line-ending and separator style.
        #[test]
        fn parsing_ignores_line_ending_and_separator(b1 in block_strategy(), b2 in block_strategy()) {
            let mut lf = String::new();
            let mut crlf = String::new();
            for (e, b) in [(100, &b1), (200, &b2)] {
                for &(p, a) in b {
                    lf += &format!("{e} {p:e} {a:e}\n");
                    crlf += &format!("{e}\t{p:e}  {a:e} \r\n");
                }
            }
            let x = AngularCdfTable::from_text("lf", &lf).unwrap();
            let y = AngularCdfTable::from_text("crlf", &crlf).unwrap();
            for p in [0.0, 0.3, 0.9, 1.0] {
                prop_assert_eq!(
                    x.angle_deg(Energy::from_ev(150.0), p).unwrap(),
                    y.angle_deg(Energy::from_ev(150.0), p).unwrap()
                );
            }
        }
    }
}
