//! Transparent source type-alias products.

use crate::diagnostic::SemanticSourceSpan;
use crate::identity::DeclarationId;
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::GenericSignature;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeAliasInfo {
    pub declaration: DeclarationId,
    pub kind: KindId,
    pub kind_shape: Box<str>,
    pub generic_signature: Option<GenericSignature>,
    pub form: TypeId,
    pub structural_form: Box<str>,
    pub dependencies: Box<[DeclarationId]>,
    pub source: SemanticSourceSpan,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeAliasTable {
    aliases: BTreeMap<DeclarationId, TypeAliasInfo>,
}

impl TypeAliasTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, info: TypeAliasInfo) {
        self.aliases.insert(info.declaration.clone(), info);
    }

    pub fn get(&self, declaration: &DeclarationId) -> Option<&TypeAliasInfo> {
        self.aliases.get(declaration)
    }

    pub fn form(&self, declaration: &DeclarationId) -> Option<TypeId> {
        self.get(declaration).map(|info| info.form)
    }

    pub fn generic_signature(&self, declaration: &DeclarationId) -> Option<&GenericSignature> {
        self.get(declaration).and_then(|info| info.generic_signature.as_ref())
    }

    pub fn contains_key(&self, declaration: &DeclarationId) -> bool {
        self.aliases.contains_key(declaration)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DeclarationId, &TypeAliasInfo)> {
        self.aliases.iter()
    }
}
