//! Artifact digests and deterministic domain-separated hashing.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const DOMAIN_SOURCE: &str = "telos.telosc.v1.source";
pub const DOMAIN_MODULE_WORLD: &str = "telos.telosc.v1.module-world";
pub const DOMAIN_SEMANTIC_WORLD: &str = "telos.telosc.v1.semantic-world";
pub const DOMAIN_QUERY_IMAGE: &str = "telos.telosc.v1.query-image";
pub const DOMAIN_METADATA: &str = "telos.telosc.v1.metadata";
pub const DOMAIN_ARTIFACT: &str = "telos.telosc.v1.artifact";
pub const DOMAIN_SECTION: &str = "telos.telosc.v1.section";

/// 256-bit cryptographic / integrity digest for artifacts and sections.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ArtifactDigest(pub [u8; 32]);

impl ArtifactDigest {
    pub const ZERO: Self = Self([0u8; 32]);

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn digest(data: &[u8]) -> Self {
        let mut hasher = ArtifactHasher::new();
        hasher.write(data);
        hasher.finalize()
    }

    pub fn digest_domain(domain: &str, data: &[u8]) -> Self {
        let mut hasher = ArtifactHasher::new_domain(domain);
        hasher.write(data);
        hasher.finalize()
    }
}

impl fmt::Debug for ArtifactDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArtifactDigest(")?;
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for ArtifactDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

/// BLAKE3-256 streaming integrity digest. Domain framing is part of wire v1.
#[derive(Clone, Debug)]
pub struct ArtifactHasher {
    inner: blake3::Hasher,
}

impl Default for ArtifactHasher {
    fn default() -> Self {
        Self::new()
    }
}
impl ArtifactHasher {
    pub fn new() -> Self {
        Self { inner: blake3::Hasher::new() }
    }
    pub fn new_domain(domain: &str) -> Self {
        let mut hasher = Self::new();
        hasher.write_u32(u32::try_from(domain.len()).expect("digest domain fits u32"));
        hasher.write(domain.as_bytes());
        hasher.write_u8(0xff);
        hasher
    }
    pub fn write_u8(&mut self, value: u8) {
        self.write(&[value]);
    }
    pub fn write_u16(&mut self, value: u16) {
        self.write(&value.to_le_bytes());
    }
    pub fn write_u32(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }
    pub fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }
    pub fn write(&mut self, data: &[u8]) {
        self.inner.update(data);
    }
    pub fn finalize(self) -> ArtifactDigest {
        ArtifactDigest(*self.inner.finalize().as_bytes())
    }
}
