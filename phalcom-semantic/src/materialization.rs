//! Compile-time materialization descriptors for static constants (LANG007.C4.P2).
//!
//! Bridges canonical semantic static values into closed, self-contained descriptors
//! consumed by backend codegen / lowering without retaining VM or heap pointers.

use crate::static_value::{StaticFloat, StaticInt, StaticValueData, StaticValueRef, StaticValueStore};
use std::sync::Arc;

/// Materialization capability classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MaterializationCapability {
    /// Primitive value materializable in P2 directly to bytecode constants.
    P2Primitive,
    /// Structural or advanced value whose runtime materialization is deferred to P8.
    DeferredStructural,
    /// Non-materializable runtime value (e.g. Type / StaticTerm without runtime term form).
    Unsupported,
}

/// Category of structural value deferred to P8.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DeferredStructuralKind {
    Tuple,
    Record,
    Data,
    Variant,
    Pack,
}

/// Standalone materialization descriptor for one evaluated static constant value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaterializationDescriptor {
    Unit,
    Bool(bool),
    Int(StaticInt),
    Float(StaticFloat),
    String(Arc<str>),
    Symbol(Arc<str>),
    DeferredStructural(DeferredStructuralKind),
    Unsupported(String),
}

impl MaterializationDescriptor {
    pub fn capability(&self) -> MaterializationCapability {
        match self {
            Self::Unit | Self::Bool(_) | Self::Int(_) | Self::Float(_) | Self::String(_) | Self::Symbol(_) => MaterializationCapability::P2Primitive,
            Self::DeferredStructural(_) => MaterializationCapability::DeferredStructural,
            Self::Unsupported(_) => MaterializationCapability::Unsupported,
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self.capability(), MaterializationCapability::P2Primitive)
    }

    /// Converts a static value in the store into a self-contained materialization descriptor.
    pub fn from_static_value(store: &StaticValueStore, val_ref: StaticValueRef) -> Option<Self> {
        let data = store.get(val_ref).ok()?;
        let descriptor = match data {
            StaticValueData::Unit => Self::Unit,
            StaticValueData::Bool(b) => Self::Bool(*b),
            StaticValueData::Int(i) => Self::Int(i.clone()),
            StaticValueData::Float(f) => Self::Float(*f),
            StaticValueData::String(str_id) => {
                let s = store.get_string(*str_id)?;
                Self::String(Arc::from(s))
            }
            StaticValueData::Symbol(sym_id) => {
                let s = store.get_symbol(*sym_id)?;
                Self::Symbol(Arc::from(s))
            }
            StaticValueData::Tuple(_) => Self::DeferredStructural(DeferredStructuralKind::Tuple),
            StaticValueData::Record(_) => Self::DeferredStructural(DeferredStructuralKind::Record),
            StaticValueData::Data(_) => Self::DeferredStructural(DeferredStructuralKind::Data),
            StaticValueData::Variant(_) => Self::DeferredStructural(DeferredStructuralKind::Variant),
            StaticValueData::Pack(_) => Self::DeferredStructural(DeferredStructuralKind::Pack),
            StaticValueData::Type(ty) => Self::Unsupported(format!("type value {ty:?} cannot be materialized at runtime in P2")),
            StaticValueData::StaticTerm(ty) => Self::Unsupported(format!("static term {ty:?} cannot be materialized at runtime in P2")),
        };
        Some(descriptor)
    }
}
