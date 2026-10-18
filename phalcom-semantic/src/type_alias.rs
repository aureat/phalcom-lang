//! Transparent source type-alias products.

use crate::diagnostic::SemanticSourceSpan;
use crate::identity::DeclarationId;
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::GenericSignature;
use phalcom_modules::identity::ModuleId;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedMap, SortedMapEntriesError};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

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

impl TypeAliasInfo {
    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        let mut dependencies = allocations.vector(self.dependencies.len())?;
        for dependency in &self.dependencies {
            dependencies.push(dependency.clone_with_allocation_budget(allocations)?);
        }
        Ok(Self {
            declaration: self.declaration.clone_with_allocation_budget(allocations)?,
            kind: self.kind,
            kind_shape: allocations.boxed_str(&self.kind_shape)?,
            generic_signature: self.generic_signature.as_ref().map(|signature| signature.clone_with_allocation_budget(allocations)).transpose()?,
            form: self.form,
            structural_form: allocations.boxed_str(&self.structural_form)?,
            dependencies: allocations.boxed_slice(dependencies)?,
            source: SemanticSourceSpan { module: self.source.module.clone_with_allocation_budget(allocations)?, range: self.source.range },
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeAliasTable {
    parent: Option<Arc<Self>>,
    frozen: Option<Arc<BudgetedSortedMap<DeclarationId, TypeAliasInfo>>>,
    aliases: BTreeMap<DeclarationId, TypeAliasInfo>,
    module_aliases: BTreeMap<ModuleId, BTreeSet<DeclarationId>>,
}

impl TypeAliasTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Attaches a read-only canonical alias table without copying its entries.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            parent: Some(parent),
            ..Self::default()
        }
    }

    pub(crate) fn from_frozen_entries_with_allocation_budget(
        mut entries: Vec<(DeclarationId, TypeAliasInfo)>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, FrozenTypeAliasTableError> {
        if entries.is_empty() {
            return Ok(Self::new());
        }
        entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let frozen = BudgetedSortedMap::from_sorted_entries(entries).map_err(|error| match error {
            SortedMapEntriesError::DuplicateKey => FrozenTypeAliasTableError::DuplicateDeclaration,
            SortedMapEntriesError::OutOfOrder => unreachable!("entries were sorted before frozen construction"),
        })?;
        let frozen = allocations.arc(frozen).map_err(FrozenTypeAliasTableError::Allocation)?;
        Ok(Self {
            frozen: Some(frozen),
            ..Self::default()
        })
    }

    /// Number of aliases owned by this workspace, excluding the parent.
    pub fn local_len(&self) -> usize {
        self.aliases.len() + self.frozen.as_ref().map_or(0, |aliases| aliases.len())
    }

    pub fn insert(&mut self, info: TypeAliasInfo) {
        assert!(
            !self.parent.as_ref().is_some_and(|parent| parent.contains_key(&info.declaration))
                && !self.frozen.as_ref().is_some_and(|frozen| frozen.contains_key(&info.declaration)),
            "cannot overwrite a canonical alias"
        );
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
        self.aliases
            .get(declaration)
            .or_else(|| self.frozen.as_ref().and_then(|frozen| frozen.get(declaration)))
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.get(declaration)))
    }

    pub fn form(&self, declaration: &DeclarationId) -> Option<TypeId> {
        self.get(declaration).map(|info| info.form)
    }

    pub fn generic_signature(&self, declaration: &DeclarationId) -> Option<&GenericSignature> {
        self.get(declaration).and_then(|info| info.generic_signature.as_ref())
    }

    pub fn contains_key(&self, declaration: &DeclarationId) -> bool {
        self.get(declaration).is_some()
    }

    pub fn declarations_for_module<'a>(&'a self, module: &'a ModuleId) -> Box<dyn Iterator<Item = &'a DeclarationId> + 'a> {
        Box::new(self.iter().filter_map(move |(declaration, _)| (&declaration.module == module).then_some(declaration)))
    }

    /// Iterates canonical and local aliases in stable parent-then-local order.
    pub fn iter(&self) -> Box<dyn Iterator<Item = (&DeclarationId, &TypeAliasInfo)> + '_> {
        let frozen = self.frozen.iter().flat_map(|entries| entries.iter());
        match &self.parent {
            Some(parent) => Box::new(parent.iter().chain(frozen).chain(self.aliases.iter())),
            None => Box::new(frozen.chain(self.aliases.iter())),
        }
    }
}

#[derive(Debug)]
pub(crate) enum FrozenTypeAliasTableError {
    Allocation(AllocationError),
    DuplicateDeclaration,
}
