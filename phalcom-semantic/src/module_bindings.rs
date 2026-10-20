//! Stable handoff of ordinary checker facts for module-level bindings.
//!
//! Static bodies use this product to distinguish a module `const` from an
//! immutable runtime `let` without copying body-local identities or resolving
//! names and types a second time.

use crate::checker::analysis::{BindingState, CallableAnalysis};
use crate::db::key::{InputFingerprint, ProductFingerprint};
use crate::identity::{BindingId, CallableId, ModuleId};
use phalcom_ast::ast::{BindingKind, LetBinding, Pattern, Program, Statement};
use phalcom_common::range::SourceRange;
use phalcom_modules::linker::SymbolId;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Canonical checked module values available while analyzing callable bodies.
///
/// The linker supplies public-name and import identity; these indexes supply
/// the ordinary checker's published value facts.
#[derive(Clone, Copy)]
pub struct ModuleValueEnvironment<'a> {
    pub bindings: &'a crate::parent_map::ParentMap<ModuleId, Arc<ModuleStaticBindingIndex>>,
    pub exports: &'a BTreeMap<(ModuleId, String), SymbolId>,
}

/// Type-free source identity for a module-owned binding before checked
/// `BindingState` products are available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleStaticBindingSeed {
    pub symbol: SymbolId,
    pub phase: ModuleBindingPhase,
    /// Source span of the simple-name binding pattern, used to recover the
    /// corresponding ordinary checker binding fact.
    pub binding_range: SourceRange,
    pub declaration_range: SourceRange,
    pub initializer_range: Option<SourceRange>,
}

/// Pre-type catalog of source module bindings. This product carries source
/// identity and phase only; it is not a name/type lookup authority for the
/// ordinary checker.
#[derive(Clone, Debug)]
pub struct ModuleStaticBindingCatalog {
    module: ModuleId,
    bindings: crate::ordered_map::OrderedMap<SymbolId, ModuleStaticBindingSeed>,
    visible_names: crate::ordered_map::OrderedMap<Box<str>, SymbolId>,
}

impl ModuleStaticBindingCatalog {
    /// Builds local top-level simple-name seeds without checking annotations or
    /// inferring types.
    pub fn from_source(module: &ModuleId, program: &Program) -> Self {
        let mut bindings = BTreeMap::new();
        for statement in &program.statements {
            let Statement::Let(binding) = statement else { continue };
            let Pattern::Name { name, range } = &binding.pattern else { continue };
            let phase = match binding.kind {
                BindingKind::Const => ModuleBindingPhase::StaticConstant,
                BindingKind::Let | BindingKind::LetMut => ModuleBindingPhase::Runtime,
            };
            let symbol = SymbolId {
                module: module.clone(),
                name: name.clone().into_boxed_str(),
            };
            let seed = ModuleStaticBindingSeed {
                symbol: symbol.clone(),
                phase,
                binding_range: *range,
                declaration_range: binding.range,
                initializer_range: binding.value.as_ref().map(|value| value.range()),
            };
            bindings.entry(symbol.clone()).or_insert(seed);
        }
        let visible_names: BTreeMap<Box<str>, SymbolId> = bindings.keys().map(|symbol| (symbol.name.clone(), symbol.clone())).collect();
        Self {
            module: module.clone(),
            bindings: bindings.into(),
            visible_names: visible_names.into(),
        }
    }

    /// Reconstructs a catalog from an authoritative `ModuleStaticBindingIndex`.
    pub fn from_index(index: &ModuleStaticBindingIndex) -> Self {
        let mut bindings = BTreeMap::new();
        for (symbol, fact) in index.iter() {
            let seed = ModuleStaticBindingSeed {
                symbol: symbol.clone(),
                phase: fact.phase,
                binding_range: fact.state.range,
                declaration_range: fact.declaration_range,
                initializer_range: fact.initializer_range,
            };
            bindings.insert(symbol.clone(), seed);
        }
        let visible_names = index
            .visible_symbol_bindings()
            .map(|(name, symbol)| (name.into(), symbol.clone()))
            .collect::<BTreeMap<_, _>>();
        Self {
            module: index.module().clone(),
            bindings: bindings.into(),
            visible_names: visible_names.into(),
        }
    }

    /// Reconstructs a frozen catalog from an authoritative index while
    /// budgeting every retained key, identity, map buffer, and Arc owner.
    pub(crate) fn from_index_with_allocation_budget(
        index: &ModuleStaticBindingIndex,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, ModuleStaticBindingCatalogBuildError> {
        let mut bindings = allocations
            .vector::<(SymbolId, ModuleStaticBindingSeed)>(index.len())
            .map_err(ModuleStaticBindingCatalogBuildError::Allocation)?;
        for (symbol, fact) in index.iter() {
            let key = clone_symbol_with_allocation_budget(symbol, allocations).map_err(ModuleStaticBindingCatalogBuildError::Allocation)?;
            let seed_symbol = clone_symbol_with_allocation_budget(&fact.symbol, allocations).map_err(ModuleStaticBindingCatalogBuildError::Allocation)?;
            bindings.push((
                key,
                ModuleStaticBindingSeed {
                    symbol: seed_symbol,
                    phase: fact.phase,
                    binding_range: fact.state.range,
                    declaration_range: fact.declaration_range,
                    initializer_range: fact.initializer_range,
                },
            ));
        }
        let bindings = crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(bindings, allocations).map_err(map_catalog_map_error)?;

        let visible_count = index.visible_symbol_bindings().count();
        let mut visible_names = allocations
            .vector::<(Box<str>, SymbolId)>(visible_count)
            .map_err(ModuleStaticBindingCatalogBuildError::Allocation)?;
        for (name, symbol) in index.visible_symbol_bindings() {
            visible_names.push((
                allocations.boxed_str(name).map_err(ModuleStaticBindingCatalogBuildError::Allocation)?,
                clone_symbol_with_allocation_budget(symbol, allocations).map_err(ModuleStaticBindingCatalogBuildError::Allocation)?,
            ));
        }
        let visible_names =
            crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(visible_names, allocations).map_err(map_catalog_map_error)?;

        Ok(Self {
            module: index
                .module()
                .clone_with_allocation_budget(allocations)
                .map_err(ModuleStaticBindingCatalogBuildError::Allocation)?,
            bindings,
            visible_names,
        })
    }

    /// Creates a budgeted empty catalog for a module with no checker index.
    pub(crate) fn empty_with_allocation_budget(module: &ModuleId, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        Ok(Self {
            module: module.clone_with_allocation_budget(allocations)?,
            bindings: crate::ordered_map::OrderedMap::default(),
            visible_names: crate::ordered_map::OrderedMap::default(),
        })
    }

    /// Creates an empty catalog for a module with no top-level bindings.
    pub fn empty(module: ModuleId) -> Self {
        Self {
            module,
            bindings: crate::ordered_map::OrderedMap::default(),
            visible_names: crate::ordered_map::OrderedMap::default(),
        }
    }

    /// Adds a linked import using its canonical remote symbol identity.
    pub fn insert_imported_binding(&mut self, local_name: &str, seed: ModuleStaticBindingSeed) {
        let symbol = seed.symbol.clone();
        if !self.bindings.contains_key(&symbol) {
            self.bindings.insert(symbol.clone(), seed);
        }
        let local_name: Box<str> = local_name.into();
        if !self.visible_names.contains_key(&local_name) {
            self.visible_names.insert(local_name, symbol);
        }
    }

    pub fn module(&self) -> &ModuleId {
        &self.module
    }

    pub fn get(&self, symbol: &SymbolId) -> Option<&ModuleStaticBindingSeed> {
        self.bindings.get(symbol)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SymbolId, &ModuleStaticBindingSeed)> {
        self.bindings.iter()
    }

    pub fn visible_bindings(&self) -> impl Iterator<Item = (&str, &ModuleStaticBindingSeed)> {
        self.visible_names
            .iter()
            .filter_map(|(name, symbol)| self.bindings.get(symbol).map(|seed| (name.as_ref(), seed)))
    }

    pub fn input_fingerprint(&self) -> InputFingerprint {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.module.hash(&mut hasher);
        for (name, seed) in self.visible_bindings() {
            name.hash(&mut hasher);
            seed.symbol.hash(&mut hasher);
            seed.phase.hash(&mut hasher);
            format!("{:?}{:?}{:?}", seed.binding_range, seed.declaration_range, seed.initializer_range).hash(&mut hasher);
        }
        InputFingerprint::new(hasher.finish())
    }

    /// Computes the source-compatible fingerprint while budgeting its exact
    /// temporary formatted-range buffer.
    pub(crate) fn input_fingerprint_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<InputFingerprint, AllocationError> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.module.hash(&mut hasher);
        for (name, seed) in self.visible_bindings() {
            name.hash(&mut hasher);
            seed.symbol.hash(&mut hasher);
            seed.phase.hash(&mut hasher);
            format_ranges_with_allocation_budget(seed, allocations)?.hash(&mut hasher);
        }
        Ok(InputFingerprint::new(hasher.finish()))
    }

    pub fn product_fingerprint(&self) -> ProductFingerprint {
        ProductFingerprint::new(self.input_fingerprint().raw())
    }
}

#[derive(Debug)]
pub(crate) enum ModuleStaticBindingCatalogBuildError {
    Allocation(AllocationError),
    DuplicateKey,
}

fn map_catalog_map_error(error: crate::ordered_map::FrozenOrderedMapError) -> ModuleStaticBindingCatalogBuildError {
    match error {
        crate::ordered_map::FrozenOrderedMapError::Allocation(error) => ModuleStaticBindingCatalogBuildError::Allocation(error),
        crate::ordered_map::FrozenOrderedMapError::DuplicateKey => ModuleStaticBindingCatalogBuildError::DuplicateKey,
    }
}

fn clone_symbol_with_allocation_budget(symbol: &SymbolId, allocations: &mut AllocationBudget) -> Result<SymbolId, AllocationError> {
    Ok(SymbolId {
        module: symbol.module.clone_with_allocation_budget(allocations)?,
        name: allocations.boxed_str(&symbol.name)?,
    })
}

fn format_ranges_with_allocation_budget(seed: &ModuleStaticBindingSeed, allocations: &mut AllocationBudget) -> Result<String, AllocationError> {
    struct CountWriter(usize);
    impl std::fmt::Write for CountWriter {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0 = self.0.checked_add(text.len()).ok_or(std::fmt::Error)?;
            Ok(())
        }
    }

    struct ByteWriter<'a>(&'a mut Vec<u8>);
    impl std::fmt::Write for ByteWriter<'_> {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0.extend_from_slice(text.as_bytes());
            Ok(())
        }
    }

    let mut count = CountWriter(0);
    write!(&mut count, "{:?}{:?}{:?}", seed.binding_range, seed.declaration_range, seed.initializer_range).map_err(|_| AllocationError::Overflow)?;
    let mut bytes = allocations.vector::<u8>(count.0)?;
    write!(
        &mut ByteWriter(&mut bytes),
        "{:?}{:?}{:?}",
        seed.binding_range,
        seed.declaration_range,
        seed.initializer_range
    )
    .map_err(|_| AllocationError::Failed)?;
    String::from_utf8(bytes).map_err(|_| AllocationError::Failed)
}

/// Declaration-time phase category for a module-owned binding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ModuleBindingPhase {
    StaticConstant,
    Runtime,
}

/// Checked source binding facts paired with their stable module identity.
#[derive(Clone, Debug)]
pub struct ModuleBindingFact {
    pub symbol: SymbolId,
    /// The body that owns `source_binding`; `BindingId` alone is body-local.
    pub source_body: CallableId,
    pub source_binding: BindingId,
    pub phase: ModuleBindingPhase,
    pub state: BindingState,
    pub declaration_range: SourceRange,
    pub initializer_range: Option<SourceRange>,
}

/// Ordinary-checker facts for simple-name module bindings in one source unit.
#[derive(Clone, Debug)]
pub struct ModuleStaticBindingIndex {
    module: ModuleId,
    bindings: crate::ordered_map::OrderedMap<SymbolId, ModuleBindingFact>,
    visible_names: crate::ordered_map::OrderedMap<Box<str>, SymbolId>,
}

impl ModuleStaticBindingIndex {
    pub(crate) fn from_frozen_artifact_parts(
        module: ModuleId,
        bindings: crate::ordered_map::OrderedMap<SymbolId, ModuleBindingFact>,
        visible_names: crate::ordered_map::OrderedMap<Box<str>, SymbolId>,
    ) -> Result<Self, &'static str> {
        if bindings.iter().any(|(symbol, fact)| symbol != &fact.symbol) || visible_names.values().any(|symbol| !bindings.contains_key(symbol)) {
            return Err("module binding artifact parts contain a mismatched owner or dangling visible name");
        }
        Ok(Self {
            module,
            bindings,
            visible_names,
        })
    }

    /// Builds the stable module handoff from the final ordinary `<main>`
    /// checker result and the source declarations that established phase.
    ///
    /// Only top-level, simple-name bindings are module namespace entries.
    /// Their type, contract, and flow state come from `CallableAnalysis`; the
    /// AST contributes only stable module name and declaration-time phase.
    pub fn from_module_analysis(module: &ModuleId, program: &Program, analysis: &CallableAnalysis) -> Self {
        let mut bindings = BTreeMap::new();
        for statement in &program.statements {
            let Statement::Let(binding) = statement else { continue };
            let Pattern::Name { name, range } = &binding.pattern else { continue };
            let phase = match binding.kind {
                BindingKind::Const => ModuleBindingPhase::StaticConstant,
                BindingKind::Let | BindingKind::LetMut => ModuleBindingPhase::Runtime,
            };
            let Some(state) = analysis
                .bindings
                .values()
                .find(|state| state.name.as_str() == name.as_str() && state.range == *range)
                .cloned()
            else {
                continue;
            };
            let symbol = SymbolId {
                module: module.clone(),
                name: name.clone().into_boxed_str(),
            };
            let initializer_range = binding.value.as_ref().map(|v| v.range());
            let fact = ModuleBindingFact {
                symbol: symbol.clone(),
                source_body: analysis.callable.clone(),
                source_binding: state.binding,
                phase,
                state,
                declaration_range: binding.range,
                initializer_range,
            };
            bindings.entry(symbol.clone()).or_insert(fact);
        }
        let visible_names: BTreeMap<Box<str>, SymbolId> = bindings.keys().map(|symbol| (symbol.name.clone(), symbol.clone())).collect();
        Self {
            module: module.clone(),
            bindings: bindings.into(),
            visible_names: visible_names.into(),
        }
    }

    pub fn get(&self, symbol: &SymbolId) -> Option<&ModuleBindingFact> {
        self.bindings.get(symbol)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SymbolId, &ModuleBindingFact)> {
        self.bindings.iter()
    }

    /// Returns visible source names and their stable binding identities.
    pub fn visible_bindings(&self) -> impl Iterator<Item = (&str, &ModuleBindingFact)> {
        self.visible_symbol_bindings()
            .filter_map(|(name, symbol)| self.bindings.get(symbol).map(|fact| (name, fact)))
    }

    /// Returns the exact local-name to canonical-symbol mapping.
    pub fn visible_symbol_bindings(&self) -> impl Iterator<Item = (&str, &SymbolId)> {
        self.visible_names.iter().map(|(name, symbol)| (name.as_ref(), symbol))
    }

    /// Adds a linked imported binding under its local source name.
    ///
    /// The linked target identity remains canonical; the local alias is only
    /// the key used to seed the ordinary checker scope for this module.
    pub fn insert_imported_binding(&mut self, local_name: &str, fact: ModuleBindingFact) {
        let symbol = fact.symbol.clone();
        if !self.bindings.contains_key(&symbol) {
            self.bindings.insert(symbol.clone(), fact);
        }
        let local_name: Box<str> = local_name.into();
        if !self.visible_names.contains_key(&local_name) {
            self.visible_names.insert(local_name, symbol);
        }
    }

    pub fn module(&self) -> &ModuleId {
        &self.module
    }

    /// Content fingerprint for checked binding and linked-name inputs.
    pub fn input_fingerprint(&self) -> InputFingerprint {
        crate::db::fingerprint::module_static_binding_input_fingerprint(self)
    }

    /// Content fingerprint for the facts consumers observe.
    pub fn product_fingerprint(&self) -> ProductFingerprint {
        crate::db::fingerprint::module_static_binding_product_fingerprint(self)
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

/// Locates the parsed syntax of a top-level constant binding by its retained
/// source site and stable symbol identity, without performing a fresh name lookup.
pub fn find_constant_declaration<'a>(
    program: &'a Program,
    symbol: &SymbolId,
    declaration_range: SourceRange,
    initializer_range: Option<SourceRange>,
) -> Option<&'a LetBinding> {
    for statement in &program.statements {
        let Statement::Let(binding) = statement else { continue };
        if binding.kind != BindingKind::Const {
            continue;
        }
        let Pattern::Name { name, .. } = &binding.pattern else { continue };
        if name.as_str() != symbol.name.as_ref() {
            continue;
        }
        if binding.range == declaration_range {
            return Some(binding);
        }
        if let Some(init_range) = initializer_range {
            if binding.value.as_ref().map(|v| v.range()) == Some(init_range) {
                return Some(binding);
            }
        }
    }
    // Fallback matching symbol name and kind
    for statement in &program.statements {
        let Statement::Let(binding) = statement else { continue };
        if binding.kind == BindingKind::Const {
            if let Pattern::Name { name, .. } = &binding.pattern {
                if name.as_str() == symbol.name.as_ref() {
                    return Some(binding);
                }
            }
        }
    }
    None
}
