//! Stable heterogeneous Record field key identity.

use crate::scalar::StaticInt;
use serde::{Deserialize, Serialize};

/// Canonical heterogeneous Record key in a generalized type projection and durable metadata.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum FieldKeyRef {
    Symbol(Box<str>),
    String(Box<str>),
    Int(StaticInt),
    Bool(bool),
}

impl FieldKeyRef {
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

    pub fn as_int(&self) -> Option<&StaticInt> {
        match self {
            Self::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl std::fmt::Display for FieldKeyRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Symbol(s) => write!(f, "#{s}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Int(i) => write!(f, "{i}"),
            Self::Bool(b) => write!(f, "{b}"),
        }
    }
}
