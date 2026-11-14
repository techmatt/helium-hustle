//! A small, stable hash (64-bit FNV-1a) for config hashes, content hashes,
//! fingerprints and RNG stream ids. Not cryptographic; stable across
//! platforms and Rust versions, which `std::hash` is not.

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// A hash as 16 lowercase hex digits.
pub fn hex(h: u64) -> String {
    format!("{h:016x}")
}

/// Serde helpers writing integers as decimal strings, so they survive
/// JavaScript's 53-bit numbers.
pub(crate) mod as_string {
    use serde::{de::Error, Deserialize, Deserializer, Serializer};
    use std::fmt::Display;
    use std::str::FromStr;

    pub fn serialize<T: Display, S: Serializer>(v: &T, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(v)
    }

    pub fn deserialize<'de, T: FromStr, D: Deserializer<'de>>(d: D) -> Result<T, D::Error>
    where
        T::Err: Display,
    {
        String::deserialize(d)?.parse().map_err(D::Error::custom)
    }
}
