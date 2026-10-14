//! Compiler binding metadata derived from a linked module.

use crate::heap::ObjRef;
use crate::interner::Symbol;
use phalcom_modules::{ExecutableLinkedProgram, ImportBindingId, LinkedModule, LinkedReadSpec, ModuleId, SymbolId};
use std::collections::BTreeMap;

/// Runtime reference to one mutable module global slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingRef {
    /// Source module object.
    pub module: ObjRef,
    /// Stable append-only global slot.
    pub slot: u16,
}

/// Canonical VM-local reference to one module-owned static constant.
///
/// The owner and source name preserve the semantic identity through imports
/// and re-exports without copying the materialized value into each descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeStaticConstantRef {
    /// Module object that owns the source declaration.
    pub module: ObjRef,
    /// Interned canonical source name, independent of any export alias.
    pub name: Symbol,
}

/// VM materialization of one symbolic linked read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinkedRead {
    /// Materialized module object.
    Module(ObjRef),
    /// Live source binding slot.
    Binding(BindingRef),
    /// Canonical source-owned static constant.
    StaticConstant(RuntimeStaticConstantRef),
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
    /// Static compile-time constant.
    StaticConstant,
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
    /// Compiler-visible prelude names mapped to runtime linked-read slots.
    pub prelude: BTreeMap<Box<str>, ImportBindingId>,
    /// Canonical read targets allocated by executable finalization.
    runtime_reads: BTreeMap<LinkedReadSpec, ImportBindingId>,
    /// Whether this namespace is backed by an immutable executable link.
    finalized: bool,
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
                    target: target.clone(),
                    symbol,
                },
            );
            bindings.runtime_reads.entry(target).or_insert(binding);
        }
        bindings
    }

    /// Constructs a finalized compiler namespace with every canonical runtime
    /// read mapped to the immutable slot allocated by the executable linker.
    pub fn from_executable_linked_program(program: &ExecutableLinkedProgram, module: &ModuleId) -> Option<Self> {
        let linked_module = program.modules.get(module)?;
        let mut bindings = Self::from_linked_module(linked_module);
        bindings.finalized = true;
        for (target, slot) in program.runtime_reads_for(module) {
            bindings.runtime_reads.insert(target.clone(), slot);
        }
        Some(bindings)
    }

    /// Returns an immutable linked import by local name.
    pub fn import(&self, name: &str) -> Option<&LinkedImportInfo> {
        self.imports.get(name)
    }

    /// Returns the finalized or legacy slot for a canonical runtime target.
    pub fn runtime_binding(&self, target: &LinkedReadSpec) -> Option<ImportBindingId> {
        self.runtime_reads.get(target).copied()
    }

    /// Whether this namespace must use finalized slots and fail closed on
    /// missing runtime reads.
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Adds one compiler-visible canonical prelude read.
    pub fn add_prelude(&mut self, name: impl Into<Box<str>>, binding: ImportBindingId) {
        self.prelude.insert(name.into(), binding);
    }
}
