//! Source-style finite-horizon search, owned by the serial history engine.
//!
//! Geant4 11.4.1 `source/processes/electromagnetic/dna/models/src/G4DNAIRT.cc`:
//! constructor 75-76 fixes R from the absolute end time; `SpaceBinning` 128-158
//! fixes initial bounds and cell counts; `Sampling` 189-231 visits whole cells
//! intersecting the scheduling track's +/-R box BEFORE lazy partner diffusion;
//! `FindBin` 344-358 clamps to those initial bounds. `MakeReaction` 497-504
//! inserts products into the same grid. The local 11.4.3 file is identical.
//!
//! Preserve these cell semantics, including the source's extrema seeds (1e9 nm
//! and zero), no radial rejection, and out-of-bounds products in edge cells.
//! Sparse ordered storage replaces the nested source maps. Initial pairs use
//! the newer identity as query center, but are emitted in canonical (older,
//! newer) order; this keeps the established Rust tie/RNG ordering. The fixed
//! cutoff is a heuristic: omitted pairs can react with nonzero probability.
//! Multiplier 1 is a candidate default, not evidence of R05/G2 convergence.

use std::collections::BTreeMap;
use std::mem::size_of;

use super::{HistoryMolecule, MAX_END_PS, START_PS, validate_time};
use crate::{ChemistryError, SpeciesInstance};
use g4_core::units::{Length, Time};
use g4_core::vec3::Vec3;

/// Candidate enumeration; both paths share identical reaction/event semantics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CandidateSearch {
    /// Source-style fixed bins. The multiplier scales the complete source cutoff.
    /// Finite, strictly positive values are accepted; convergence is not implied.
    Indexed { cutoff_multiplier: f64 },
    /// Untruncated quadratic enumeration for small-system reference comparisons.
    AllPairs,
}

impl Default for CandidateSearch {
    fn default() -> Self {
        Self::Indexed {
            cutoff_multiplier: 1.0,
        }
    }
}

/// Geant4's `m * (1.45 nm + 2 sqrt(8 * 0.00946 nm²/ps * end_ps))`.
///
/// Uses the absolute final time, not time remaining or end minus the 1 ps start.
/// The complete expression is multiplied by `multiplier`. This is a candidate
/// search heuristic, not a reaction radius, probability bound, or zero-tail law.
pub fn source_cutoff(end_time: Time, multiplier: f64) -> Result<Length, ChemistryError> {
    validate_time("end time", end_time.ps(), START_PS, MAX_END_PS)?;
    let cutoff = multiplier * (1.45 + 2.0 * (8.0 * 0.00946 * end_time.ps()).sqrt());
    if !multiplier.is_finite() || multiplier <= 0.0 || !cutoff.is_finite() || cutoff <= 0.0 {
        return Err(ChemistryError::invalid(
            "history spatial search",
            "cutoff multiplier and resulting cutoff must be finite and positive",
        ));
    }
    Ok(Length::from_nm(cutoff))
}

#[derive(Debug, Clone, Copy)]
struct Axis {
    min: f64,
    max: f64,
    bins: i32,
}

impl Axis {
    fn new(min: f64, max: f64, cutoff: f64) -> Result<Self, ChemistryError> {
        let span = max - min;
        let bins = (span / cutoff).floor().max(1.0);
        // C++ uses G4int. Reject an unrepresentable source grid instead of
        // overflowing or silently merging cells and changing candidate sets.
        if !span.is_finite() || bins > f64::from(i32::MAX) || !(bins * span).is_finite() {
            return Err(ChemistryError::invalid(
                "history spatial search",
                "initial extent or cell count exceeds the finite source grid domain",
            ));
        }
        Ok(Self {
            min,
            max,
            bins: bins as i32,
        })
    }

    fn bin(self, value: f64) -> i32 {
        if value <= self.min {
            0
        } else if value >= self.max {
            self.bins - 1
        } else {
            // Keep the source's operation order; construction checks overflow.
            (f64::from(self.bins) * (value - self.min) / (self.max - self.min)) as i32
        }
        .clamp(0, self.bins - 1)
    }
}

type Cell = [i32; 3];

#[derive(Debug)]
pub(super) struct SpatialIndex {
    axes: [Axis; 3],
    cutoff: Length,
    cells: BTreeMap<Cell, Vec<usize>>,
    entries: usize,
    payload_bytes: usize,
}

impl SpatialIndex {
    pub(super) fn new(records: &[SpeciesInstance], cutoff: Length) -> Result<Self, ChemistryError> {
        let mut min = [1e9_f64; 3];
        let mut max = [0.0_f64; 3];
        for record in records {
            for (axis, coordinate) in coordinates(record.position).into_iter().enumerate() {
                min[axis] = min[axis].min(coordinate);
                max[axis] = max[axis].max(coordinate);
            }
        }
        // An empty input has no grid extent. Future products cannot appear.
        if records.is_empty() {
            min = [0.0; 3];
        }
        let axes = [
            Axis::new(min[0], max[0], cutoff.nm())?,
            Axis::new(min[1], max[1], cutoff.nm())?,
            Axis::new(min[2], max[2], cutoff.nm())?,
        ];
        let mut index = Self {
            axes,
            cutoff,
            cells: BTreeMap::new(),
            entries: 0,
            payload_bytes: 0,
        };
        for (id, record) in records.iter().enumerate() {
            index.insert(id, record.position);
        }
        Ok(index)
    }

    pub(super) fn insert(&mut self, id: usize, position: Vec3) {
        let key = self.key(position);
        let cell = self.cells.entry(key).or_insert_with(|| {
            self.payload_bytes += size_of::<(Cell, Vec<usize>)>();
            Vec::new()
        });
        let before = cell.capacity();
        cell.push(id);
        self.payload_bytes += (cell.capacity() - before) * size_of::<usize>();
        self.entries += 1;
    }

    fn key(&self, position: Vec3) -> Cell {
        let xyz = coordinates(position);
        std::array::from_fn(|i| self.axes[i].bin(xyz[i]))
    }

    fn range(&self, position: Vec3) -> [Cell; 2] {
        let xyz = coordinates(position);
        [-1.0, 1.0]
            .map(|sign| std::array::from_fn(|i| self.axes[i].bin(xyz[i] + sign * self.cutoff.nm())))
    }

    /// One bounded row; initial rows are reverse queries to preserve pair order.
    pub(super) fn candidates(
        &self,
        id: usize,
        initial: bool,
        molecules: &[HistoryMolecule],
        out: &mut Vec<usize>,
    ) {
        out.clear();
        let position = molecules[id].instance().position;
        let key = self.key(position);
        let [low, high] = if initial {
            // Cell width >= R, so a newer center can reach this cell only
            // from an adjacent cell. One extra cell covers edge roundoff;
            // the exact source query below rejects every extra candidate.
            [
                key.map(|bin| bin.saturating_sub(2).max(0)),
                std::array::from_fn(|i| key[i].saturating_add(2).min(self.axes[i].bins - 1)),
            ]
        } else {
            self.range(position)
        };
        for x in low[0]..=high[0] {
            for y in low[1]..=high[1] {
                for z in low[2]..=high[2] {
                    let Some(cell) = self.cells.get(&[x, y, z]) else {
                        continue;
                    };
                    for &other in cell {
                        if !molecules[other].is_alive()
                            || (initial && other <= id)
                            || (!initial && other >= id)
                        {
                            continue;
                        }
                        if initial {
                            let [lo, hi] = self.range(molecules[other].instance().position);
                            if !(0..3).all(|i| lo[i] <= key[i] && key[i] <= hi[i]) {
                                continue;
                            }
                        }
                        out.push(other);
                    }
                }
            }
        }
        // IDs occur in exactly one cell; no deduplication may hide double insertion.
        out.sort_unstable();
        debug_assert!(out.windows(2).all(|pair| pair[0] < pair[1]));
    }

    pub(super) fn cutoff(&self) -> Length {
        self.cutoff
    }
    pub(super) fn entries(&self) -> usize {
        self.entries
    }

    // Lower bound: map keys, Vec headers and ID capacities. Excludes BTreeMap
    // node links/spare slots, allocator metadata/alignment and stack storage.
    pub(super) fn payload_bytes(&self) -> usize {
        self.payload_bytes
    }
}

fn coordinates(position: Vec3) -> [f64; 3] {
    [position.x, position.y, position.z]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Species;
    use crate::molecule::{Molecule, MoleculeId};

    fn molecule(id: usize, position: Vec3) -> HistoryMolecule {
        HistoryMolecule {
            molecule: Molecule::new(
                MoleculeId(id as u64),
                SpeciesInstance {
                    species: Species::H,
                    position,
                    time: Time::from_ps(1.0),
                },
            ),
            alive: true,
            generation: 0,
        }
    }

    #[test]
    fn canonical_reverse_queries_match_exhaustive_three_dimensional_cell_oracle() {
        // Independent exhaustive predicate for fixed [-20,20]^3 bounds:
        // floor(40 / 2.000199963649581) = 19 bins per axis. Includes negative
        // sites, exact planes, corners, cell boundaries and shuffled identity order.
        let mut molecules = vec![
            molecule(0, Vec3::new(-20.0, -20.0, -20.0)),
            molecule(1, Vec3::new(20.0, 20.0, 20.0)),
        ];
        for i in 0..200 {
            let coordinate = |factor: usize| -20.0 + ((i * factor) % 201) as f64 * 0.2;
            molecules.push(molecule(
                molecules.len(),
                Vec3::new(coordinate(37), coordinate(71), coordinate(113)),
            ));
        }
        for x in [-20.0, -20.0 + 40.0 / 19.0, 0.0, 20.0] {
            molecules.push(molecule(molecules.len(), Vec3::new(x, x, x)));
        }
        let records: Vec<_> = molecules.iter().map(|m| *m.instance()).collect();
        let index = SpatialIndex::new(&records, Length::from_nm(2.000199963649581)).unwrap();
        let source_bin = |x: f64| ((19.0 * (x + 20.0) / 40.0).floor() as i32).clamp(0, 18);
        let mut actual = Vec::new();
        for first in 0..molecules.len() {
            let first_xyz = coordinates(molecules[first].instance().position);
            let expected: Vec<_> = (first + 1..molecules.len())
                .filter(|&second| {
                    coordinates(molecules[second].instance().position)
                        .into_iter()
                        .enumerate()
                        .all(|(axis, center)| {
                            source_bin(center - 2.000199963649581) <= source_bin(first_xyz[axis])
                                && source_bin(first_xyz[axis])
                                    <= source_bin(center + 2.000199963649581)
                        })
                })
                .collect();
            index.candidates(first, true, &molecules, &mut actual);
            assert_eq!(actual, expected, "canonical row {first}");
        }
    }

    #[test]
    fn products_use_fixed_edge_cells_and_exclude_dead_self_and_future_siblings() {
        let mut molecules = vec![
            molecule(0, Vec3::ZERO),
            molecule(1, Vec3::new(10.0, 0.0, 0.0)),
        ];
        let records: Vec<_> = molecules.iter().map(|m| *m.instance()).collect();
        let mut index = SpatialIndex::new(&records, Length::from_nm(2.000199963649581)).unwrap();
        // Products far outside the initial box still share its edge cells.
        molecules.push(molecule(2, Vec3::new(-100.0, 0.0, 0.0)));
        molecules.push(molecule(3, Vec3::new(100.0, 0.0, 0.0)));
        molecules.push(molecule(4, Vec3::new(-101.0, 0.0, 0.0)));
        molecules[1].alive = false;
        let mut actual = Vec::new();
        for (id, expected) in [(2, vec![0]), (3, vec![]), (4, vec![0, 2])] {
            index.insert(id, molecules[id].instance().position);
            index.candidates(id, false, &molecules, &mut actual);
            assert_eq!(actual, expected, "product {id}");
        }
        assert_eq!(index.entries(), 5);
    }
}
