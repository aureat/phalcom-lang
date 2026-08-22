//! 128-bit structural fingerprint implementation for deterministic hashing.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Deterministic 128-bit hash used for structural type/signature equivalence and cache validation.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Fingerprint128(pub [u8; 16]);

impl Fingerprint128 {
    pub const ZERO: Self = Self([0u8; 16]);

    pub fn from_u128(val: u128) -> Self {
        Self(val.to_be_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for Fingerprint128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fingerprint128(")?;
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for Fingerprint128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

/// Simple deterministic streaming hasher producing a 128-bit fingerprint.
#[derive(Clone, Debug)]
pub struct FingerprintBuilder {
    state: u128,
}

impl Default for FingerprintBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FingerprintBuilder {
    pub fn new() -> Self {
        Self {
            // FNV-1a 128-bit offset basis
            state: 0x6c62272e07bb014262b821756295c58d,
        }
    }

    pub fn write_u8(&mut self, b: u8) {
        self.state ^= b as u128;
        self.state = self.state.wrapping_mul(0x1000000000000000000013b);
    }

    pub fn write_u32(&mut self, val: u32) {
        for b in val.to_be_bytes() {
            self.write_u8(b);
        }
    }

    pub fn write_u64(&mut self, val: u64) {
        for b in val.to_be_bytes() {
            self.write_u8(b);
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.write_u32(s.len() as u32);
        for b in s.as_bytes() {
            self.write_u8(*b);
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.write_u32(bytes.len() as u32);
        for &b in bytes {
            self.write_u8(b);
        }
    }

    pub fn write_fingerprint(&mut self, fp: Fingerprint128) {
        for b in fp.0 {
            self.write_u8(b);
        }
    }

    pub fn finish(self) -> Fingerprint128 {
        Fingerprint128(self.state.to_be_bytes())
    }
}
