//! Provenance metadata attached to every output (docs/05 E.2, docs/10, AGENTS.md).
//!
//! Results are reproducible only for the same seed, toolchain, lockfile and data; this
//! struct records what is needed to say so. Bitwise reproducibility is claimed solely for an
//! identical platform/toolchain/lockfile (docs/reproducibility.md).

use crate::manifest::Manifest;
use serde::Serialize;

/// Geant4 version whose algorithms and data this project follows (docs/README.md).
pub const GEANT4_REFERENCE_VERSION: &str = "11.4.1";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Provenance {
    pub crate_version: String,
    /// Output of `rustc -V` for the compiler that built this crate.
    pub rustc: String,
    pub geant4_reference_version: String,
    pub dataset: String,
    pub dataset_version: String,
    /// SHA-256 of `data/MANIFEST.toml`.
    pub manifest_sha256: String,
    pub master_seed: u64,
    pub internal_units: String,
}

impl Provenance {
    pub fn new(manifest: &Manifest, master_seed: u64) -> Self {
        Self {
            crate_version: env!("CARGO_PKG_VERSION").to_owned(),
            rustc: env!("DNA_CORE_RUSTC_VERSION").to_owned(),
            geant4_reference_version: GEANT4_REFERENCE_VERSION.to_owned(),
            dataset: manifest.dataset().to_owned(),
            dataset_version: manifest.version().to_owned(),
            manifest_sha256: manifest.hash().to_owned(),
            master_seed,
            internal_units: "energy: eV, length: nm, time: ps".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;

    fn manifest() -> Manifest {
        Manifest::from_toml_str(&format!(
            "dataset = \"G4EMLOW\"\nversion = \"8.8\"\n\n[[file]]\npath = \"dna/a\"\ngroup = \"g\"\nbytes = 0\nsha256 = \"{}\"\n",
            "0".repeat(64)
        ))
        .unwrap()
    }

    #[test]
    fn records_data_reference_seed_and_toolchain() {
        let m = manifest();
        let p = Provenance::new(&m, 4242);
        assert_eq!(p.geant4_reference_version, "11.4.1");
        assert_eq!(p.dataset, "G4EMLOW");
        assert_eq!(p.dataset_version, "8.8");
        assert_eq!(p.manifest_sha256, m.hash());
        assert_eq!(p.master_seed, 4242);
        assert_eq!(p.crate_version, env!("CARGO_PKG_VERSION"));
        assert!(p.rustc.starts_with("rustc "), "{}", p.rustc);
        assert_eq!(p.internal_units, "energy: eV, length: nm, time: ps");
    }

    #[test]
    fn serialises_to_json_with_stable_field_names() {
        let json = serde_json::to_value(Provenance::new(&manifest(), 1)).unwrap();
        for key in [
            "crate_version",
            "rustc",
            "geant4_reference_version",
            "dataset",
            "dataset_version",
            "manifest_sha256",
            "master_seed",
            "internal_units",
        ] {
            assert!(json.get(key).is_some(), "missing {key}");
        }
    }
}
