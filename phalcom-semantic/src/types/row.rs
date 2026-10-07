//! Canonical record-row domain and representation.

use super::id::{KindId, TypeId, TypeParameterId};
use crate::field_key::FieldKey;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StructuralMemberPresence {
    Required,
    Optional,
}

impl StructuralMemberPresence {
    pub fn is_optional(self) -> bool {
        self == Self::Optional
    }

    pub fn is_required(self) -> bool {
        self == Self::Required
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AnonymousRecordRemainder<T = TypeId> {
    Unconstrained { key_type: T },
    Typed { key_type: T, value_type: T },
}

impl<T: Copy> AnonymousRecordRemainder<T> {
    pub fn key_type(self) -> T {
        match self {
            Self::Unconstrained { key_type } | Self::Typed { key_type, .. } => key_type,
        }
    }

    pub fn value_type(self) -> Option<T> {
        match self {
            Self::Unconstrained { .. } => None,
            Self::Typed { value_type, .. } => Some(value_type),
        }
    }
}

impl<T> AnonymousRecordRemainder<T> {
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> AnonymousRecordRemainder<U> {
        match self {
            Self::Unconstrained { key_type } => AnonymousRecordRemainder::Unconstrained { key_type: f(key_type) },
            Self::Typed { key_type, value_type } => AnonymousRecordRemainder::Typed {
                key_type: f(key_type),
                value_type: f(value_type),
            },
        }
    }

    pub fn try_map<U, E>(self, mut f: impl FnMut(T) -> Result<U, E>) -> Result<AnonymousRecordRemainder<U>, E> {
        Ok(match self {
            Self::Unconstrained { key_type } => AnonymousRecordRemainder::Unconstrained { key_type: f(key_type)? },
            Self::Typed { key_type, value_type } => AnonymousRecordRemainder::Typed {
                key_type: f(key_type)?,
                value_type: f(value_type)?,
            },
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RecordRowField<T = TypeId> {
    pub key: FieldKey,
    pub ty: T,
    pub presence: StructuralMemberPresence,
}

impl<T> RecordRowField<T> {
    pub fn new(key: FieldKey, ty: T, presence: StructuralMemberPresence) -> Self {
        Self { key, ty, presence }
    }

    pub fn required(key: FieldKey, ty: T) -> Self {
        Self::new(key, ty, StructuralMemberPresence::Required)
    }

    pub fn optional(key: FieldKey, ty: T) -> Self {
        Self::new(key, ty, StructuralMemberPresence::Optional)
    }

    pub fn symbol_required(name: impl Into<std::sync::Arc<str>>, ty: T) -> Self {
        Self::required(FieldKey::symbol(name), ty)
    }

    pub fn symbol_optional(name: impl Into<std::sync::Arc<str>>, ty: T) -> Self {
        Self::optional(FieldKey::symbol(name), ty)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RecordRowTail {
    Closed,
    Anonymous(AnonymousRecordRemainder),
    Parameter(TypeParameterId), // must have kind RecordRow
    Projection(TypeId),         // must have kind RecordRow
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordRowData {
    pub fields: Box<[RecordRowField]>, // sorted by canonical_cmp, unique
    pub tail: RecordRowTail,
}

impl RecordRowData {
    /// Creates a canonical closed row from unsorted fields.
    /// Sorts fields by canonical FieldKey order and checks for duplicates.
    pub fn new_closed(mut fields: Vec<RecordRowField>) -> Result<Self, DuplicateFieldError> {
        fields.sort_by(|a, b| a.key.canonical_cmp(&b.key));
        for i in 1..fields.len() {
            if fields[i - 1].key == fields[i].key {
                return Err(DuplicateFieldError(fields[i].key.clone()));
            }
        }
        Ok(Self {
            fields: fields.into_boxed_slice(),
            tail: RecordRowTail::Closed,
        })
    }

    /// Creates a canonical row with a tail.
    pub fn new_with_tail(mut fields: Vec<RecordRowField>, tail: RecordRowTail) -> Result<Self, DuplicateFieldError> {
        fields.sort_by(|a, b| a.key.canonical_cmp(&b.key));
        for i in 1..fields.len() {
            if fields[i - 1].key == fields[i].key {
                return Err(DuplicateFieldError(fields[i].key.clone()));
            }
        }
        Ok(Self {
            fields: fields.into_boxed_slice(),
            tail,
        })
    }

    pub fn find_field(&self, key: &FieldKey) -> Option<TypeId> {
        self.fields.binary_search_by(|f| f.key.canonical_cmp(key)).ok().map(|idx| self.fields[idx].ty)
    }

    pub fn find_field_entry(&self, key: &FieldKey) -> Option<&RecordRowField> {
        self.fields.binary_search_by(|f| f.key.canonical_cmp(key)).ok().map(|idx| &self.fields[idx])
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("duplicate record field: {0}")]
pub struct DuplicateFieldError(pub FieldKey);

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RecordRowFormationError {
    #[error("duplicate record field: {0}")]
    DuplicateField(FieldKey),

    #[error("record field `{field}` is not a proper type")]
    FieldNotProperType { field: FieldKey, ty: TypeId },

    #[error("record row tail key domain is not a proper type: {0:?}")]
    TailKeyDomainInvalid(TypeId),

    #[error("record row tail value domain is not a proper type: {0:?}")]
    TailValueNotProperType(TypeId),

    #[error("record row tail parameter is missing")]
    TailParameterMissing(TypeParameterId),

    #[error("record row tail parameter must have kind RecordFields")]
    TailParameterWrongKind { parameter: TypeParameterId, actual: KindId },
}
