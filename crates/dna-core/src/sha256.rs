//! SHA-256 (FIPS 180-4) for integrity checks of the committed Geant4 data files and the
//! manifest hash. Thin wrapper over the approved `sha2` crate that keeps the small API used
//! by `manifest` and `provenance`. Not used for security purposes.

use sha2::{Digest, Sha256 as Inner};

/// Incremental SHA-256 hasher.
#[derive(Debug, Clone, Default)]
pub struct Sha256(Inner);

impl Sha256 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn digest(data: &[u8]) -> [u8; 32] {
        Inner::digest(data).into()
    }

    pub fn update(&mut self, data: &[u8]) {
        Digest::update(&mut self.0, data);
    }

    pub fn finalize(self) -> [u8; 32] {
        self.0.finalize().into()
    }
}

/// Lowercase hexadecimal encoding.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_message() {
        assert_eq!(
            hex(&Sha256::digest(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn abc() {
        assert_eq!(
            hex(&Sha256::digest(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn two_block_message() {
        assert_eq!(
            hex(&Sha256::digest(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn one_million_a_fed_in_odd_chunks() {
        let mut h = Sha256::new();
        let chunk = [b'a'; 997];
        let mut left = 1_000_000;
        while left > 0 {
            let n = left.min(chunk.len());
            h.update(&chunk[..n]);
            left -= n;
        }
        assert_eq!(
            hex(&h.finalize()),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn padding_boundaries_55_56_64_bytes() {
        // Reference digests computed with coreutils `sha256sum` (printf 'a' x N).
        for (n, want) in [
            (
                55,
                "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
            ),
            (
                56,
                "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
            ),
            (
                64,
                "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
            ),
        ] {
            assert_eq!(hex(&Sha256::digest(&vec![b'a'; n])), want, "n = {n}");
        }
    }
}
