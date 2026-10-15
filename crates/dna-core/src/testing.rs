//! Test utilities (public so that the integration tests of this and other crates can use them).

use rand::TryRng;
use std::convert::Infallible;

/// A "random" generator replaying a fixed list of uniform variates, for checking that a model
/// consumes the draws it documents and maps them as documented.
///
/// Each value `u` in (0,1) is encoded so that [`crate::sampling::uniform`] returns it to within
/// 2^-52. After the script is exhausted the generator returns 0.5.
#[derive(Debug, Clone)]
pub struct ScriptedRng {
    words: Vec<u64>,
    next: usize,
}

impl ScriptedRng {
    pub fn new(uniforms: &[f64]) -> Self {
        let words = uniforms
            .iter()
            .map(|&u| {
                assert!(u > 0.0 && u < 1.0, "scripted uniforms must lie in (0, 1)");
                let k = (u * 4_503_599_627_370_496.0 - 0.5).round().max(0.0) as u64;
                k << 12
            })
            .collect();
        Self { words, next: 0 }
    }

    /// Total number of 64-bit words drawn so far, including any beyond the script (which return
    /// 0.5): a model that draws more than documented is caught by comparing with the script
    /// length.
    pub fn consumed(&self) -> usize {
        self.next
    }
}

impl TryRng for ScriptedRng {
    type Error = Infallible;
    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        Ok((self.try_next_u64()? >> 32) as u32)
    }
    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        let w = self
            .words
            .get(self.next)
            .copied()
            .unwrap_or(2_251_799_813_685_248u64 << 12);
        self.next += 1;
        Ok(w)
    }
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        for chunk in dst.chunks_mut(8) {
            let w = self.try_next_u64()?.to_le_bytes();
            chunk.copy_from_slice(&w[..chunk.len()]);
        }
        Ok(())
    }
}
