//! Pure scalar values shared by semantic products and static-value storage.

use num_bigint::BigInt;
use std::sync::Arc;

/// Exact language integer, normalized to an immediate when it fits.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StaticInt {
    /// Integer representable by the runtime's immediate form.
    Small(i64),
    /// Arbitrary-precision integer outside the immediate range.
    Large(Arc<BigInt>),
}

impl StaticInt {
    /// Creates a normalized exact integer.
    pub fn new(value: BigInt) -> Self {
        match i64::try_from(&value).ok() {
            Some(small) => Self::Small(small),
            None => Self::Large(Arc::new(value)),
        }
    }

    /// Converts to the exact arbitrary-precision representation.
    pub fn to_bigint(&self) -> BigInt {
        match self {
            Self::Small(value) => BigInt::from(*value),
            Self::Large(value) => value.as_ref().clone(),
        }
    }
}

impl From<i64> for StaticInt {
    fn from(value: i64) -> Self {
        Self::Small(value)
    }
}

impl From<BigInt> for StaticInt {
    fn from(value: BigInt) -> Self {
        Self::new(value)
    }
}

impl Ord for StaticInt {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => a.cmp(b),
            _ => self.to_bigint().cmp(&other.to_bigint()),
        }
    }
}

impl PartialOrd for StaticInt {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
