//! Type parameter identities and generic signatures.

use super::id::{KindId, TypeParameterId};
use crate::identity::{CallableId, DeclarationId};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeParameterOwner {
    Declaration(DeclarationId),
    Callable(CallableId),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeParameterData {
    pub owner: TypeParameterOwner,
    pub index: u16,
    pub name: Box<str>,
    pub kind: KindId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericSignature {
    pub owner: TypeParameterOwner,
    pub parameters: Box<[TypeParameterId]>,
}
