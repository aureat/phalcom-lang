//! Compiler binding metadata derived from a linked module.

use crate::heap::ObjRef;
use phalcom_modules::{ImportBindingId, LinkedModule, LinkedReadSpec, SymbolId};
use std::collections::BTreeMap;

/// Runtime reference to one mutable module global slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingRef {
    /// Source module object.
    pub module: ObjRef,
    /// Stable append-only global slot.
    pub slot: u16,
}

/// VM materialization of one symbolic linked read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinkedRead {
    /// Materialized module object.
    Module(ObjRef),
    /// Live source binding slot.
    Binding(BindingRef),
}

/// Top-level binding kind visible to the compiler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TopLevelBindingKind {
    /// Mutable source global.
    MutableGlobal,
    /// Immutable source global or class declaration.
    ImmutableGlobal,
    /// Class declaration.
    Class,
    /// Immutable indexed linked import.
    Import(ImportBindingId),
}

/// One compiler-visible top-level binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopLevelBindingInfo {
    /// Binding classification.
    pub kind: TopLevelBindingKind,
    /// Optional source range supplied by a higher compiler layer.
    pub declared_at: Option<phalcom_common::range::SourceRange>,
}

/// Resolved imported binding and its canonical target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedImportInfo {
    /// Local source name.
    pub local_name: Box<str>,
    /// Indexed linked read.
    pub binding: ImportBindingId,
    /// Canonical target, if this is a selected value.
    pub target: LinkedReadSpec,
    /// Canonical value identity for selected imports.
    pub symbol: Option<SymbolId>,
}

/// Unified namespace table seeded before module body compilation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CompileBindings {
    /// All module-scope names.
    pub entries: BTreeMap<Box<str>, TopLevelBindingInfo>,
    /// Detailed imported reads.
    pub imports: BTreeMap<Box<str>, LinkedImportInfo>,
}

impl CompileBindings {
    /// Constructs a compiler namespace from a linked module layout.
    pub fn from_linked_module(module: &LinkedModule) -> Self {
        let mut bindings = Self::default();
        for name in module.bindings.local_globals.keys() {
            bindings.entries.insert(
                name.clone(),
                TopLevelBindingInfo {
                    kind: TopLevelBindingKind::MutableGlobal,
                    declared_at: None,
                },
            );
        }
        for (name, &binding) in &module.bindings.imports {
            let target = module.linked_reads[binding.0 as usize].clone();
            let symbol = match &target {
                LinkedReadSpec::Binding(symbol) => Some(symbol.clone()),
                LinkedReadSpec::Module(_) => None,
            };
            bindings.entries.insert(
                name.clone(),
                TopLevelBindingInfo {
                    kind: TopLevelBindingKind::Import(binding),
                    declared_at: None,
                },
            );
            bindings.imports.insert(
                name.clone(),
                LinkedImportInfo {
                    local_name: name.clone(),
                    binding,
                    target,
                    symbol,
                },
            );
        }
        bindings
    }

    /// Returns an immutable linked import by local name.
    pub fn import(&self, name: &str) -> Option<&LinkedImportInfo> {
        self.imports.get(name)
    }
}
