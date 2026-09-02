//! Canonical record-row domain and representation.

use super::id::{KindId, TypeId, TypeParameterId};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RecordRowField {
    pub name: Box<str>,
    pub ty: TypeId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RecordRowTail {
    Closed,
    Parameter(TypeParameterId), // must have kind RecordRow
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordRowData {
    pub fields: Box<[RecordRowField]>, // sorted by name, unique
    pub tail: RecordRowTail,
}

impl RecordRowData {
    /// Creates a canonical closed row from unsorted fields.
    /// Sorts fields by name and checks for duplicates.
    pub fn new_closed(mut fields: Vec<RecordRowField>) -> Result<Self, DuplicateFieldError> {
        fields.sort_by(|a, b| a.name.cmp(&b.name));
        for i in 1..fields.len() {
            if fields[i - 1].name == fields[i].name {
                return Err(DuplicateFieldError(fields[i].name.clone()));
            }
        }
        Ok(Self {
            fields: fields.into_boxed_slice(),
            tail: RecordRowTail::Closed,
        })
    }

    /// Creates a canonical row with a tail.
    pub fn new_with_tail(mut fields: Vec<RecordRowField>, tail: RecordRowTail) -> Result<Self, DuplicateFieldError> {
        fields.sort_by(|a, b| a.name.cmp(&b.name));
        for i in 1..fields.len() {
            if fields[i - 1].name == fields[i].name {
                return Err(DuplicateFieldError(fields[i].name.clone()));
            }
        }
        Ok(Self {
            fields: fields.into_boxed_slice(),
            tail,
        })
    }

    pub fn find_field(&self, name: &str) -> Option<TypeId> {
        self.fields.binary_search_by(|f| f.name.as_ref().cmp(name)).ok().map(|idx| self.fields[idx].ty)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("duplicate record field: {0}")]
pub struct DuplicateFieldError(pub Box<str>);

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RecordRowFormationError {
    #[error("duplicate record field: {0}")]
    DuplicateField(Box<str>),

    #[error("record field `{field}` is not a proper type")]
    FieldNotProperType { field: Box<str>, ty: TypeId },

    #[error("record row tail parameter is missing")]
    TailParameterMissing(TypeParameterId),

    #[error("record row tail parameter must have kind RecordRow")]
    TailParameterWrongKind { parameter: TypeParameterId, actual: KindId },
}
