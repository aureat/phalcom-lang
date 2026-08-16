//! VM-independent module materialization plans.

use phalcom_modules::{LinkedModuleInterface, LinkedReadSpec, ModuleId, SymbolId};

/// Declaration metadata materialized before ordinary module initialization.
#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeDeclarationBlueprint {
    /// A class declaration and its statically linked superclass identity.
    Class(ClassBlueprint),
    /// A top-level global slot declaration.
    Global { symbol: SymbolId, mutable: bool },
}

/// VM-independent class declaration metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct ClassBlueprint {
    /// Canonical class symbol.
    pub symbol: SymbolId,
    /// Canonical superclass symbol, if explicit.
    pub superclass: Option<SymbolId>,
    /// Source field names in declaration order.
    pub fields: Vec<Box<str>>,
    /// Canonical method selectors.
    pub methods: Vec<Box<str>>,
}

/// Immutable, VM-independent plan for materializing one linked module.
///
/// Runtime heap handles deliberately do not appear here. Heap objects and
/// closures belong to VM materialization/execution, not to the compiled plan.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleMaterializationPlan {
    /// Module identity.
    pub id: ModuleId,
    /// Declaration shells/global slots to allocate before execution.
    pub declarations: Vec<RuntimeDeclarationBlueprint>,
    /// Linked interface used by runtime materialization.
    pub interface: LinkedModuleInterface,
    /// Symbolic reads consumed by `GetLinked`.
    pub linked_reads: Vec<LinkedReadSpec>,
}

impl ModuleMaterializationPlan {
    pub fn new(module: &phalcom_modules::LinkedModule, declarations: Vec<RuntimeDeclarationBlueprint>) -> Self {
        Self {
            id: module.interface.module.clone(),
            declarations,
            interface: module.interface.clone(),
            linked_reads: module.linked_reads.clone(),
        }
    }

    /// Compatibility constructor for callers that have no declaration AST.
    /// It is intentionally named as a fallback; production compilation uses
    /// `new` with a populated declaration plan.
    pub fn without_declarations(module: &phalcom_modules::LinkedModule) -> Self {
        Self::new(module, Vec::new())
    }
}

/// Transitional source-compatible name. The payload is now a truthful
/// VM-independent materialization plan rather than a partly-materialized
/// runtime artifact.
pub type ModuleArtifact = ModuleMaterializationPlan;
