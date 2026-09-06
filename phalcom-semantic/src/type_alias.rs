//! Transparent source type-alias products.

use crate::diagnostic::SemanticSourceSpan;
use crate::identity::DeclarationId;
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::GenericSignature;
use phalcom_modules::identity::ModuleId;
use std::collections::{BTreeMap, BTreeSet};

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
    module_aliases: BTreeMap<ModuleId, BTreeSet<DeclarationId>>,
}

impl TypeAliasTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, info: TypeAliasInfo) {
        self.module_aliases
            .entry(info.declaration.module.clone())
            .or_default()
            .insert(info.declaration.clone());
        self.aliases.insert(info.declaration.clone(), info);
    }

    /// Removes aliases contributed by one source module.
    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        if let Some(aliases) = self.module_aliases.remove(module) {
            for declaration in aliases {
                self.aliases.remove(&declaration);
            }
        }
    }

    /// Removes one alias contribution while retaining the module index for its
    /// other declarations.
    pub fn remove(&mut self, declaration: &DeclarationId) {
        let Some(info) = self.aliases.remove(declaration) else {
            return;
        };
        if let Some(aliases) = self.module_aliases.get_mut(&info.declaration.module) {
            aliases.remove(declaration);
            if aliases.is_empty() {
                self.module_aliases.remove(&info.declaration.module);
            }
        }
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

    pub fn declarations_for_module(&self, module: &ModuleId) -> impl Iterator<Item = &DeclarationId> {
        self.module_aliases.get(module).into_iter().flat_map(|declarations| declarations.iter())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DeclarationId, &TypeAliasInfo)> {
        self.aliases.iter()
    }
}
