//! Provenance metadata attached to every output (docs/05 E.2, docs/10, AGENTS.md).
//!
//! Results are reproducible only for the same seed, toolchain, lockfile and data; this
//! struct records what is needed to say so. Bitwise reproducibility is claimed solely for an
//! identical platform/toolchain/lockfile (docs/reproducibility.md).

use crate::manifest::Manifest;
use crate::sha256::{Sha256, hex};
use serde::Serialize;

/// Geant4 version whose algorithms and data this project follows (docs/README.md).
pub const GEANT4_REFERENCE_VERSION: &str = "11.4.1";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Provenance {
    pub crate_version: String,
    /// Output of `rustc -V` for the compiler that built this crate.
    pub rustc: Option<String>,
    /// Cargo's target triple for this compiled crate, not the caller's host triple.
    pub target: Option<String>,
    pub platform: Platform,
    pub source: SourceState,
    pub cargo_lock: LockfileIdentity,
    pub geant4_reference_version: String,
    pub dataset: String,
    pub dataset_version: String,
    /// SHA-256 of supplied manifest text; `inputs.verification` states whether its files
    /// were actually captured and checked.
    pub manifest_sha256: String,
    pub master_seed: u64,
    pub internal_units: String,
    pub units: Units,
    pub numerical_policy: NumericalPolicy,
    pub model_policy: ModelPolicy,
    pub inputs: InputIdentity,
    pub chemistry: Option<ChemistryIdentity>,
}

/// Build-time source evidence. `None` means the corresponding Git query was unavailable;
/// `dirty = false` is emitted only after a successful status query.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SourceState {
    pub revision: Option<String>,
    pub dirty: Option<bool>,
}

/// SHA-256 of the bytes of the workspace `Cargo.lock`, or an explicit unavailable value.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LockfileIdentity {
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Platform {
    pub arch: Option<String>,
    pub os: Option<String>,
    pub env: Option<String>,
    pub family: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Units {
    pub energy: &'static str,
    pub length: &'static str,
    pub time: &'static str,
}

/// Stable policy identifiers; change an identifier whenever its implementation changes
/// in a way that can alter a numerical result or random draw sequence.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NumericalPolicy {
    pub floating_point: &'static str,
    pub rng: &'static str,
    pub history_streams: &'static str,
    pub reduction: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelPolicy {
    pub transport: &'static str,
    pub physicochemistry: Option<&'static str>,
    pub chemistry: Option<&'static str>,
}

/// Identity of the actual input boundary. A supplied manifest by itself is not verification.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "verification", rename_all = "snake_case")]
pub enum InputIdentity {
    Verified {
        manifest_sha256: String,
        files: Vec<LoadedFileIdentity>,
    },
    Unverified {
        supplied_manifest_sha256: String,
        reason: &'static str,
    },
}

/// A logical file name and verified byte digest, in manifest order. No data-root path is kept.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LoadedFileIdentity {
    pub relative_path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChemistryIdentity {
    pub semantic_sha256: String,
    /// Present only when the engine was constructed from the embedded option3 source files.
    pub source_file_sha256: Option<String>,
}

fn available(value: &'static str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}

fn hash_part(hash: &mut Sha256, value: &str) {
    hash.update(&(value.len() as u64).to_le_bytes());
    hash.update(value.as_bytes());
}

impl Provenance {
    /// Metadata for a supplied manifest; this alone makes no verification claim.
    pub fn new(manifest: &Manifest, master_seed: u64) -> Self {
        let mut provenance = Self::unverified(master_seed);
        provenance.dataset = manifest.dataset().to_owned();
        provenance.dataset_version = manifest.version().to_owned();
        provenance.manifest_sha256 = manifest.hash().to_owned();
        provenance.inputs = InputIdentity::Unverified {
            supplied_manifest_sha256: manifest.hash().to_owned(),
            reason: "a supplied manifest does not prove loaded bytes",
        };
        provenance
    }

    /// Synthetic test metadata with no manifest claim. Empty identity strings mean no
    /// dataset or manifest was supplied; the explicit verification marker is authoritative.
    pub fn unverified(master_seed: u64) -> Self {
        Self {
            crate_version: env!("CARGO_PKG_VERSION").to_owned(),
            rustc: available(env!("DNA_CORE_RUSTC_VERSION")),
            target: available(env!("DNA_CORE_TARGET")),
            platform: Platform {
                arch: available(env!("DNA_CORE_TARGET_ARCH")),
                os: available(env!("DNA_CORE_TARGET_OS")),
                env: available(env!("DNA_CORE_TARGET_ENV")),
                family: available(env!("DNA_CORE_TARGET_FAMILY")),
            },
            source: SourceState {
                revision: available(env!("DNA_CORE_GIT_REVISION")),
                dirty: match env!("DNA_CORE_GIT_DIRTY") {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                },
            },
            cargo_lock: LockfileIdentity {
                sha256: available(env!("DNA_CORE_LOCKFILE_SHA256")),
            },
            geant4_reference_version: GEANT4_REFERENCE_VERSION.to_owned(),
            dataset: String::new(),
            dataset_version: String::new(),
            manifest_sha256: String::new(),
            master_seed,
            internal_units: "energy: eV, length: nm, time: ps".to_owned(),
            units: Units {
                energy: "eV",
                length: "nm",
                time: "ps",
            },
            numerical_policy: NumericalPolicy {
                floating_point: "ieee754-binary64-v1",
                rng: "chacha12-splitmix64-v1",
                history_streams: "history-stage-v1",
                reduction: "history-order-v1",
            },
            model_policy: ModelPolicy {
                transport: "g4dna-option2-electron-v1",
                physicochemistry: None,
                chemistry: None,
            },
            inputs: InputIdentity::Unverified {
                supplied_manifest_sha256: String::new(),
                reason: "synthetic test input; no verified dataset identity",
            },
            chemistry: None,
        }
    }

    /// Attach identity from a completed verified byte snapshot. The caller must obtain
    /// `manifest` from the snapshot rather than from an unrelated input file.
    pub fn with_verified_input(mut self, manifest: &Manifest) -> Self {
        self.inputs = InputIdentity::Verified {
            manifest_sha256: manifest.hash().to_owned(),
            files: manifest
                .files()
                .iter()
                .map(|f| LoadedFileIdentity {
                    relative_path: f.path.clone(),
                    bytes: f.bytes,
                    sha256: f.sha256.clone(),
                })
                .collect(),
        };
        self
    }

    /// Attach the constructed chemistry engine's semantic identity and optional source hash.
    pub fn with_chemistry(
        mut self,
        semantic_sha256: &str,
        source_file_sha256: Option<&str>,
    ) -> Self {
        self.chemistry = Some(ChemistryIdentity {
            semantic_sha256: semantic_sha256.to_owned(),
            source_file_sha256: source_file_sha256.map(str::to_owned),
        });
        self.model_policy.physicochemistry = Some("g4dna-water-physchem-v1");
        self.model_policy.chemistry = Some("g4dna-option3-constructed-v1");
        self
    }

    /// Fingerprint the known lockfile and policy identifiers. No machine path participates.
    /// Returns `None` when lockfile bytes could not be read at build time.
    pub fn policy_lock_sha256(&self) -> Option<String> {
        let lock = self.cargo_lock.sha256.as_deref()?;
        let mut hash = Sha256::new();
        for value in [
            "geant4-rs/policy-lock/v1",
            lock,
            self.numerical_policy.floating_point,
            self.numerical_policy.rng,
            self.numerical_policy.history_streams,
            self.numerical_policy.reduction,
            self.model_policy.transport,
            self.model_policy.physicochemistry.unwrap_or(""),
            self.model_policy.chemistry.unwrap_or(""),
        ] {
            hash_part(&mut hash, value);
        }
        Some(hex(&hash.finalize()))
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
    fn synthetic_metadata_has_no_manifest_claim() {
        let json = serde_json::to_value(Provenance::unverified(17)).unwrap();
        assert_eq!(json["master_seed"], 17);
        assert_eq!(json["dataset"], "");
        assert_eq!(json["dataset_version"], "");
        assert_eq!(json["manifest_sha256"], "");
        assert_eq!(json["inputs"]["verification"], "unverified");
        assert_eq!(json["inputs"]["supplied_manifest_sha256"], "");
        assert!(json["inputs"].get("files").is_none());
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
        assert!(p.rustc.as_deref().is_some_and(|v| v.starts_with("rustc ")));
        assert_eq!(p.internal_units, "energy: eV, length: nm, time: ps");
        assert_eq!(p.target.as_deref(), Some(env!("DNA_CORE_TARGET")));
        assert!(p.cargo_lock.sha256.as_ref().is_some_and(|h| h.len() == 64));
        assert!(matches!(p.inputs, InputIdentity::Unverified { .. }));
    }

    #[test]
    fn serialises_to_json_with_stable_field_names() {
        let json = serde_json::to_value(Provenance::new(&manifest(), 1)).unwrap();
        for key in [
            "crate_version",
            "rustc",
            "target",
            "platform",
            "source",
            "cargo_lock",
            "geant4_reference_version",
            "dataset",
            "dataset_version",
            "manifest_sha256",
            "master_seed",
            "internal_units",
            "units",
            "numerical_policy",
            "model_policy",
            "inputs",
            "chemistry",
        ] {
            assert!(json.get(key).is_some(), "missing {key}");
        }
        assert_eq!(json["inputs"]["verification"], "unverified");
        assert!(json["source"].get("dirty").is_some());
        assert!(json["cargo_lock"].get("sha256").is_some());
    }

    #[test]
    fn verified_input_records_loaded_file_hashes_without_data_root() {
        let m = manifest();
        let p = Provenance::new(&m, 1).with_verified_input(&m);
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"verification\":\"verified\""));
        assert!(json.contains(&m.files()[0].sha256));
        assert!(!json.contains("C:\\\\"));
        assert!(!json.contains("/tmp/"));
    }

    #[test]
    fn lockfile_and_policy_changes_change_identity() {
        let mut p = Provenance::new(&manifest(), 1);
        p.cargo_lock.sha256 = Some("a".repeat(64));
        let original = p.policy_lock_sha256().unwrap();
        p.cargo_lock.sha256 = Some("b".repeat(64));
        assert_ne!(original, p.policy_lock_sha256().unwrap());
        p.cargo_lock.sha256 = Some("a".repeat(64));
        p.model_policy.transport = "g4dna-option2-electron-v2";
        assert_ne!(original, p.policy_lock_sha256().unwrap());
        p.cargo_lock.sha256 = None;
        assert_eq!(p.policy_lock_sha256(), None);
    }

    #[test]
    fn custom_chemistry_has_its_own_semantic_identity() {
        let p = Provenance::new(&manifest(), 1).with_chemistry("custom", None);
        let json = serde_json::to_value(p).unwrap();
        assert_eq!(json["chemistry"]["semantic_sha256"], "custom");
        assert!(json["chemistry"]["source_file_sha256"].is_null());
    }

    #[test]
    fn unavailable_build_evidence_serializes_as_null() {
        let mut p = Provenance::new(&manifest(), 1);
        p.rustc = None;
        p.source = SourceState {
            revision: None,
            dirty: None,
        };
        p.cargo_lock.sha256 = None;
        let json = serde_json::to_value(p).unwrap();
        assert!(json["rustc"].is_null());
        assert!(json["source"]["revision"].is_null());
        assert!(json["source"]["dirty"].is_null());
        assert!(json["cargo_lock"]["sha256"].is_null());
    }
}
