//! Discrete selection among partial cross sections, in the order Geant4 uses.

/// `RandomSelect` of `G4DNABornExcitationModel1.cc:232-267` and
/// `G4DNABornIonisationModel1.cc:926-981`: the buffer is summed from the **last** component down,
/// scaled by the uniform variate `u`, and the first component (again from the last) whose value
/// exceeds the running remainder is chosen; if none does (only through rounding, or when every
/// value is zero) component 0 is returned.
///
/// The selection probability of component `i` is `values[i] / sum(values)`; the downward order
/// only matters for which uniform value selects which component (i.e. for the random stream).
pub fn random_select(values: &[f64], u: f64) -> usize {
    let mut remainder: f64 = values.iter().rev().sum();
    remainder *= u;
    for i in (0..values.len()).rev() {
        if values[i] > remainder {
            return i;
        }
        remainder -= values[i];
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_walks_from_the_last_component() {
        // total 10: u in [0, 0.5) -> last (5), then [0.5, 0.8) -> middle (3), then first (2)
        let v = [2.0, 3.0, 5.0];
        assert_eq!(random_select(&v, 0.0), 2);
        assert_eq!(random_select(&v, 0.49), 2);
        assert_eq!(random_select(&v, 0.51), 1);
        assert_eq!(random_select(&v, 0.79), 1);
        assert_eq!(random_select(&v, 0.81), 0);
        assert_eq!(random_select(&v, 0.999_999), 0);
    }

    #[test]
    fn zero_components_are_never_selected_and_all_zero_falls_back_to_zero() {
        let v = [0.0, 4.0, 0.0, 6.0];
        for k in 0..1000 {
            let u = (k as f64 + 0.5) / 1000.0;
            let i = random_select(&v, u);
            assert!(i == 1 || i == 3, "u = {u} selected {i}");
        }
        assert_eq!(random_select(&[0.0, 0.0, 0.0], 0.3), 0);
    }

    #[test]
    fn selection_frequencies_equal_partial_over_total() {
        let v = [1.0, 2.0, 3.0, 4.0];
        let n = 100_000;
        let mut count = [0usize; 4];
        for k in 0..n {
            count[random_select(&v, (k as f64 + 0.5) / n as f64)] += 1;
        }
        for (i, c) in count.iter().enumerate() {
            let expect = v[i] / 10.0;
            assert!((*c as f64 / n as f64 - expect).abs() < 1e-4, "{i}: {c}");
        }
    }
}
