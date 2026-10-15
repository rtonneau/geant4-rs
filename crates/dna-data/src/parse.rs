//! Tolerant reader for Geant4 `.dat` numeric tables.
//!
//! Mirrors the line handling of `G4DNACrossSectionDataSet::LoadData`
//! (`source/processes/electromagnetic/dna/utils/src/G4DNACrossSectionDataSet.cc:83-240`):
//! `#` starts a comment, `\r` and `\n` both end a line (CRLF and LF files are equivalent),
//! blank lines are skipped, spaces and tabs are interchangeable separators. Unlike the C++
//! (where a bad token silently ends the row) a malformed token, a non-finite number or a
//! ragged row is a [`DataError::Parse`] with file and line.

use crate::error::DataError;

/// A rectangular table of finite numbers, stored row-major.
#[derive(Debug, Clone, PartialEq)]
pub struct RawTable {
    n_cols: usize,
    values: Vec<f64>,
}

impl RawTable {
    /// Parse `text`; `name` is only used in error messages.
    pub fn parse(name: &str, text: &str) -> Result<Self, DataError> {
        let mut n_cols = 0usize;
        let mut values = Vec::new();
        for (index, line) in text.split('\n').enumerate() {
            // A lone '\r' also ends a line in the C++ reader; it keeps the '\n' line number.
            for segment in line.split('\r') {
                let content = segment.split('#').next().unwrap_or("");
                let before = values.len();
                for token in content.split_ascii_whitespace() {
                    let v: f64 = token.parse().map_err(|_| DataError::Parse {
                        file: name.to_string(),
                        line: index + 1,
                        message: format!("cannot parse {token:?} as a number"),
                    })?;
                    if !v.is_finite() {
                        return Err(DataError::Parse {
                            file: name.to_string(),
                            line: index + 1,
                            message: format!("non-finite value {token:?}"),
                        });
                    }
                    values.push(v);
                }
                let row_len = values.len() - before;
                if row_len == 0 {
                    continue;
                }
                if n_cols == 0 {
                    n_cols = row_len;
                } else if row_len != n_cols {
                    return Err(DataError::Parse {
                        file: name.to_string(),
                        line: index + 1,
                        message: format!("row has {row_len} columns, expected {n_cols}"),
                    });
                }
            }
        }
        if n_cols == 0 {
            return Err(DataError::InvalidTable {
                file: name.to_string(),
                message: "no data rows".to_string(),
            });
        }
        Ok(Self { n_cols, values })
    }

    pub fn n_cols(&self) -> usize {
        self.n_cols
    }

    pub fn n_rows(&self) -> usize {
        self.values.len() / self.n_cols
    }

    /// Row `i` (panics if `i >= n_rows()`).
    pub fn row(&self, i: usize) -> &[f64] {
        &self.values[i * self.n_cols..(i + 1) * self.n_cols]
    }

    pub fn rows(&self) -> impl Iterator<Item = &[f64]> {
        self.values.chunks_exact(self.n_cols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LF: &str = "1.0 2.0 3.0\n4.0 5.0 6.0\n";

    #[test]
    fn parses_lf_crlf_and_lone_cr_identically() {
        let lf = RawTable::parse("t", LF).unwrap();
        let crlf = RawTable::parse("t", &LF.replace('\n', "\r\n")).unwrap();
        let cr = RawTable::parse("t", &LF.replace('\n', "\r")).unwrap();
        assert_eq!(lf, crlf);
        assert_eq!(lf, cr);
        assert_eq!((lf.n_rows(), lf.n_cols()), (2, 3));
        assert_eq!(lf.row(1), &[4.0, 5.0, 6.0]);
    }

    #[test]
    fn tabs_spaces_mixed_and_no_trailing_newline() {
        let t = RawTable::parse("t", "  1.0\t2.0   \n\t3.0 \t 4.0").unwrap();
        assert_eq!(t.rows().collect::<Vec<_>>(), vec![&[1.0, 2.0], &[3.0, 4.0]]);
    }

    #[test]
    fn comments_and_blank_lines_are_skipped() {
        let t = RawTable::parse("t", "# header\n\n1 2 # trailing\n   \n3 4\n").unwrap();
        assert_eq!(t.n_rows(), 2);
    }

    #[test]
    fn geant4_number_formats() {
        let t = RawTable::parse("t", "1.00000E+02 0. 7.40000000000000 5e-3 .5 -1.5E+1\n").unwrap();
        assert_eq!(t.row(0), &[100.0, 0.0, 7.4, 0.005, 0.5, -15.0]);
    }

    #[test]
    fn errors_report_file_and_line() {
        let e = RawTable::parse("f.dat", "1 2\n3 x\n").unwrap_err();
        assert!(matches!(&e, DataError::Parse { file, line: 2, .. } if file == "f.dat"), "{e:?}");
        let e = RawTable::parse("f.dat", "1 2\n3\n").unwrap_err();
        assert!(matches!(e, DataError::Parse { line: 2, .. }), "{e:?}");
        let e = RawTable::parse("f.dat", "1 NaN\n").unwrap_err();
        assert!(matches!(e, DataError::Parse { line: 1, .. }), "{e:?}");
        let e = RawTable::parse("f.dat", "1 inf\n").unwrap_err();
        assert!(matches!(e, DataError::Parse { line: 1, .. }), "{e:?}");
    }

    #[test]
    fn empty_input_is_an_error() {
        assert!(matches!(
            RawTable::parse("f.dat", "# only a comment\n\n"),
            Err(DataError::InvalidTable { .. })
        ));
    }
}
