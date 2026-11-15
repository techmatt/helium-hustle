//! Seeded, per-purpose random streams held in the game state.
//!
//! Every purpose (a name such as `"speculators"`) gets its own ChaCha8 stream,
//! keyed by the game seed and the FNV-1a hash of the purpose name, so a new
//! draw in one system never shifts the numbers another sees. Each variant
//! declares its purposes (`VariantDef::rng`); none is shared.

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::fx::Fx;
use crate::hash::{as_string, fnv1a64};

/// A ChaCha8 stream; serializes as
/// `{"seed": "<64 hex>", "stream": "<u64>", "word_pos": "<u128>"}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stream(ChaCha8Rng);

impl Stream {
    pub fn new(seed: u64, purpose: &str) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        rng.set_stream(fnv1a64(purpose.as_bytes()));
        Stream(rng)
    }

    /// Uniform in `0..n`, `n >= 1`. Drawn on `u32` words only, never `usize`
    /// (rand samples `usize` differently on 64-bit and wasm32), and by
    /// rejection, so every value is equally likely and independent of rand's
    /// sampling code.
    pub fn below(&mut self, n: u32) -> u32 {
        assert!(n > 0, "below(0)");
        let threshold = n.wrapping_neg() % n;
        loop {
            let x = self.0.next_u32();
            if x >= threshold {
                return x % n;
            }
        }
    }

    /// Uniform whole number in `lo..=hi`.
    pub fn int_in(&mut self, lo: u32, hi: u32) -> u32 {
        assert!(lo <= hi, "int_in({lo}, {hi})");
        match (hi - lo).checked_add(1) {
            Some(n) => lo + self.below(n),
            None => self.0.next_u32(),
        }
    }

    /// Uniform in `lo..=hi` in steps of `Fx::EPSILON` (a span of more than
    /// 2^32 - 1 steps is refused).
    pub fn fx_in(&mut self, lo: Fx, hi: Fx) -> Fx {
        assert!(lo <= hi, "fx_in({lo}, {hi})");
        let span = u32::try_from((hi - lo).steps()).ok().and_then(|s| s.checked_add(1));
        let span = span.unwrap_or_else(|| panic!("fx_in({lo}, {hi}): span too wide"));
        lo.plus_steps(self.below(span) as i64)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StreamJson {
    seed: String,
    #[serde(with = "as_string")]
    stream: u64,
    #[serde(with = "as_string")]
    word_pos: u128,
}

impl Serialize for Stream {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let seed: String = self.0.get_seed().iter().map(|b| format!("{b:02x}")).collect();
        StreamJson {
            seed,
            stream: self.0.get_stream(),
            word_pos: self.0.get_word_pos(),
        }
        .serialize(s)
    }
}

impl<'de> Deserialize<'de> for Stream {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let j = StreamJson::deserialize(d)?;
        if j.seed.len() != 64 || !j.seed.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(D::Error::custom("rng seed: expected 64 hex digits"));
        }
        let mut seed = [0u8; 32];
        for (i, b) in seed.iter_mut().enumerate() {
            *b = u8::from_str_radix(&j.seed[2 * i..2 * i + 2], 16)
                .map_err(|_| D::Error::custom("rng seed: bad hex"))?;
        }
        let mut rng = ChaCha8Rng::from_seed(seed);
        rng.set_stream(j.stream);
        rng.set_word_pos(j.word_pos);
        Ok(Stream(rng))
    }
}
