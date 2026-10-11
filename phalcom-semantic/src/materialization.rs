//! Compile-time materialization descriptors for static constants (LANG007.C4).
//!
//! Bridges canonical semantic static values into closed, self-contained descriptors
//! consumed by backend codegen / lowering without retaining VM or heap pointers.

use crate::static_value::{StaticFloat, StaticInt, StaticSequenceFamily, StaticValueData, StaticValueRef, StaticValueStore};
use std::sync::Arc;

/// Materialization capability classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MaterializationCapability {
    /// Primitive value materializable in P2 directly to bytecode constants.
    P2Primitive,
    /// Structural or advanced value whose runtime materialization is deferred to its owning plan.
    DeferredStructural,
    /// Immutable sequence recursively materializable by the collections runtime path.
    Sequence,
    /// Non-materializable runtime value (e.g. Type / StaticTerm without runtime term form).
    Unsupported,
}

/// Category of structural value not yet projected into a runtime descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DeferredStructuralKind {
    Tuple,
    Sequence,
    Record,
    Data,
    Variant,
    Pack,
}

/// Runtime representation chosen from the canonical checked sequence family.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SequenceMaterializationKind {
    Vector,
    FrozenArray,
}

/// Standalone materialization descriptor for one evaluated static constant value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaterializationDescriptor {
    Unit,
    Bool(bool),
    Char(char),
    Int(StaticInt),
    Float(StaticFloat),
    String(Arc<str>),
    Symbol(Arc<str>),
    Sequence {
        kind: SequenceMaterializationKind,
        elements: Arc<[MaterializationDescriptor]>,
    },
    DeferredStructural(DeferredStructuralKind),
    Unsupported(String),
}

impl MaterializationDescriptor {
    pub fn capability(&self) -> MaterializationCapability {
        match self {
            Self::Unit | Self::Bool(_) | Self::Char(_) | Self::Int(_) | Self::Float(_) | Self::String(_) | Self::Symbol(_) => {
                MaterializationCapability::P2Primitive
            }
            Self::Sequence { .. } => MaterializationCapability::Sequence,
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
            StaticValueData::Char(c) => Self::Char(*c),
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
            StaticValueData::Sequence(sequence_id) => {
                let sequence = store.get_sequence(*sequence_id).ok()?;
                let kind = match sequence.family() {
                    StaticSequenceFamily::Vector => SequenceMaterializationKind::Vector,
                    StaticSequenceFamily::Array => SequenceMaterializationKind::FrozenArray,
                };
                let mut elements = Vec::with_capacity(sequence.elements().len());
                for child in sequence.elements() {
                    let descriptor = Self::from_static_value(store, *child)?;
                    if matches!(
                        descriptor.capability(),
                        MaterializationCapability::DeferredStructural | MaterializationCapability::Unsupported
                    ) {
                        return Some(Self::DeferredStructural(DeferredStructuralKind::Sequence));
                    }
                    elements.push(descriptor);
                }
                Self::Sequence {
                    kind,
                    elements: Arc::from(elements),
                }
            }
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
