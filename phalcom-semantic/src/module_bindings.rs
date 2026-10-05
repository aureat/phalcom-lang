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
use std::collections::BTreeMap;

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
    bindings: BTreeMap<SymbolId, ModuleBindingFact>,
    visible_names: BTreeMap<Box<str>, SymbolId>,
}

impl ModuleStaticBindingIndex {
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
        let visible_names = bindings.keys().map(|symbol| (symbol.name.clone(), symbol.clone())).collect();
        Self {
            module: module.clone(),
            bindings,
            visible_names,
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
        self.visible_names
            .iter()
            .filter_map(|(name, symbol)| self.bindings.get(symbol).map(|fact| (name.as_ref(), fact)))
    }

    /// Adds a linked imported binding under its local source name.
    ///
    /// The linked target identity remains canonical; the local alias is only
    /// the key used to seed the ordinary checker scope for this module.
    pub fn insert_imported_binding(&mut self, local_name: &str, fact: ModuleBindingFact) {
        let symbol = fact.symbol.clone();
        self.bindings.entry(symbol.clone()).or_insert(fact);
        self.visible_names.entry(local_name.into()).or_insert(symbol);
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
