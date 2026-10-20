//! Exact scalar terms shared by durable metadata and semantic type identity.

use num_bigint::BigInt;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Exact language integer, normalized to an immediate when it fits.
#[derive(Clone, Debug, Eq)]
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

    /// Creates a normalized exact integer with allocation budget tracking for heap allocation.
    pub fn new_with_allocation_budget(
        value: BigInt,
        allocations: &mut crate::allocation::AllocationBudget,
    ) -> Result<Self, crate::allocation::AllocationError> {
        match i64::try_from(&value).ok() {
            Some(small) => Ok(Self::Small(small)),
            None => {
                let arc = allocations.arc(value)?;
                Ok(Self::Large(arc))
            }
        }
    }

    /// Normalizes compatibility inputs that directly constructed `Large`.
    pub fn normalized(self) -> Self {
        match self {
            Self::Large(value) => match i64::try_from(value.as_ref()).ok() {
                Some(small) => Self::Small(small),
                None => Self::Large(value),
            },
            small => small,
        }
    }

    /// Converts to the exact arbitrary-precision representation.
    pub fn to_bigint(&self) -> BigInt {
        match self {
            Self::Small(value) => BigInt::from(*value),
            Self::Large(value) => value.as_ref().clone(),
        }
    }

    /// Converts at an addressability/resource boundary where `i64` is required.
    pub fn try_to_i64(&self) -> Option<i64> {
        match self {
            Self::Small(value) => Some(*value),
            Self::Large(value) => i64::try_from(value.as_ref()).ok(),
        }
    }
}

impl fmt::Display for StaticInt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Small(value) => value.fmt(formatter),
            Self::Large(value) => value.fmt(formatter),
        }
    }
}

impl PartialEq for StaticInt {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => a == b,
            (Self::Large(a), Self::Large(b)) => a == b,
            (Self::Small(small), large) | (large, Self::Small(small)) => large.try_to_i64() == Some(*small),
        }
    }
}

impl Hash for StaticInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Public compatibility variants cannot create a second equality/hash
        // universe: even a directly constructed Large(7) hashes as Small(7).
        match self.try_to_i64() {
            Some(small) => {
                0_u8.hash(state);
                small.hash(state);
            }
            None => {
                1_u8.hash(state);
                if let Self::Large(value) = self {
                    value.hash(state);
                }
            }
        }
    }
}

impl Serialize for StaticInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Decimal text is portable across JSON implementations and avoids
        // consumers silently rounding values outside their native number width.
        serializer.serialize_str(&self.to_string())
    }
}

struct StaticIntVisitor;

impl<'de> Visitor<'de> for StaticIntVisitor {
    type Value = StaticInt;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an exact integer encoded as decimal text")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        value.parse::<BigInt>().map(StaticInt::new).map_err(E::custom)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(&value)
    }

    // Accept legacy numeric JSON metadata while all newly emitted metadata
    // uses the portable decimal-string representation above.
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(StaticInt::from(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(StaticInt::new(BigInt::from(value)))
    }
}

impl<'de> Deserialize<'de> for StaticInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StaticIntVisitor)
    }
}

impl From<i64> for StaticInt {
    fn from(value: i64) -> Self {
        Self::Small(value)
    }
}

impl std::str::FromStr for StaticInt {
    type Err = num_bigint::ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bi: BigInt = s.parse()?;
        Ok(Self::new(bi))
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
            (Self::Small(a), Self::Large(b)) => match i64::try_from(b.as_ref()).ok() {
                Some(b_i64) => a.cmp(&b_i64),
                None => match b.sign() {
                    num_bigint::Sign::Minus => std::cmp::Ordering::Greater,
                    num_bigint::Sign::NoSign => std::cmp::Ordering::Equal,
                    num_bigint::Sign::Plus => std::cmp::Ordering::Less,
                },
            },
            (Self::Large(a), Self::Small(b)) => match i64::try_from(a.as_ref()).ok() {
                Some(a_i64) => a_i64.cmp(b),
                None => match a.sign() {
                    num_bigint::Sign::Minus => std::cmp::Ordering::Less,
                    num_bigint::Sign::NoSign => std::cmp::Ordering::Equal,
                    num_bigint::Sign::Plus => std::cmp::Ordering::Greater,
                },
            },
            (Self::Large(a), Self::Large(b)) => a.as_ref().cmp(b.as_ref()),
        }
    }
}

impl PartialOrd for StaticInt {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
