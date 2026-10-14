//! Canonical semantic identity for heterogeneous Record field keys.

use crate::scalar::StaticInt;
use num_bigint::Sign;
use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;

/// The closed kind domain accepted for unordered Record fields.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FieldKeyKind {
    Symbol,
    String,
    Int,
    Bool,
}

/// A content-owned Record key. Arc identity is never part of its semantics.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FieldKey {
    Symbol(Arc<str>),
    String(Arc<str>),
    Int(StaticInt),
    Bool(bool),
}

impl FieldKey {
    pub fn symbol(value: impl Into<Arc<str>>) -> Self {
        Self::Symbol(value.into())
    }

    pub fn string(value: impl Into<Arc<str>>) -> Self {
        Self::String(value.into())
    }

    pub const fn int(value: StaticInt) -> Self {
        Self::Int(value)
    }

    pub const fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    pub const fn kind(&self) -> FieldKeyKind {
        match self {
            Self::Symbol(_) => FieldKeyKind::Symbol,
            Self::String(_) => FieldKeyKind::String,
            Self::Int(_) => FieldKeyKind::Int,
            Self::Bool(_) => FieldKeyKind::Bool,
        }
    }

    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Self::Symbol(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    /// Compares by the language-defined kind order and then by exact content.
    pub fn canonical_cmp(&self, other: &Self) -> Ordering {
        let kind_order = self.kind_rank().cmp(&other.kind_rank());
        if kind_order != Ordering::Equal {
            return kind_order;
        }

        match (self, other) {
            (Self::Symbol(left), Self::Symbol(right)) | (Self::String(left), Self::String(right)) => left.as_ref().cmp(right.as_ref()),
            (Self::Int(left), Self::Int(right)) => left.cmp(right),
            (Self::Bool(left), Self::Bool(right)) => left.cmp(right),
            _ => Ordering::Equal,
        }
    }

    /// Encodes a stable, tagged key representation for durable fingerprints.
    pub fn stable_fingerprint_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Self::Symbol(value) => encode_text(&mut bytes, 0, value),
            Self::String(value) => encode_text(&mut bytes, 1, value),
            Self::Int(value) => {
                bytes.push(2);
                let integer = value.to_bigint();
                let (sign, magnitude) = integer.to_bytes_be();
                bytes.push(u8::from(sign == Sign::Minus));
                encode_len(&mut bytes, magnitude.len());
                bytes.extend_from_slice(&magnitude);
            }
            Self::Bool(value) => bytes.extend_from_slice(&[3, u8::from(*value)]),
        }
        bytes
    }

    fn kind_rank(&self) -> u8 {
        match self {
            Self::Symbol(_) => 0,
            Self::String(_) => 1,
            Self::Int(_) => 2,
            Self::Bool(_) => 3,
        }
    }
}

fn encode_text(bytes: &mut Vec<u8>, tag: u8, value: &str) {
    bytes.push(tag);
    encode_len(bytes, value.len());
    bytes.extend_from_slice(value.as_bytes());
}

fn encode_len(bytes: &mut Vec<u8>, len: usize) {
    bytes.extend_from_slice(&(len as u64).to_be_bytes());
}

impl fmt::Display for FieldKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symbol(value) => write!(f, "#{value}"),
            Self::String(value) => write!(f, "{value:?}"),
            Self::Int(value) => write!(f, "{}", value.to_bigint()),
            Self::Bool(value) => write!(f, "{value}"),
        }
    }
}

impl Ord for FieldKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.canonical_cmp(other)
    }
}

impl PartialOrd for FieldKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<StaticInt> for FieldKey {
    fn from(value: StaticInt) -> Self {
        Self::Int(value)
    }
}

impl From<bool> for FieldKey {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for FieldKey {
    fn from(value: i64) -> Self {
        Self::Int(StaticInt::Small(value))
    }
}

impl From<&phalcom_type_meta::type_node::RecordKeyRef> for FieldKey {
    fn from(key: &phalcom_type_meta::type_node::RecordKeyRef) -> Self {
        use phalcom_type_meta::type_node::RecordKeyRef;
        match key {
            RecordKeyRef::Symbol(value) => Self::symbol(value.as_ref()),
            RecordKeyRef::String(value) => Self::string(value.as_ref()),
            RecordKeyRef::Int(value) => Self::int(value.clone()),
            RecordKeyRef::Bool(value) => Self::bool(*value),
        }
    }
}

impl From<&FieldKey> for phalcom_type_meta::type_node::RecordKeyRef {
    fn from(key: &FieldKey) -> Self {
        match key {
            FieldKey::Symbol(value) => Self::Symbol(value.as_ref().into()),
            FieldKey::String(value) => Self::String(value.as_ref().into()),
            FieldKey::Int(value) => Self::Int(value.clone()),
            FieldKey::Bool(value) => Self::Bool(*value),
        }
    }
}
