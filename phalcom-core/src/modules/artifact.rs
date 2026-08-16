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
/// closures belong to VM materialization/execution, not to the linked plan.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleMaterializationPlan {
    pub id: ModuleId,
    pub declarations: Vec<RuntimeDeclarationBlueprint>,
    pub interface: LinkedModuleInterface,
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

    /// Produces a conservative declaration plan directly from canonical linked
    /// global identities. Source-aware compilation replaces class entries with
    /// richer `ClassBlueprint`s, but this fallback still predeclares every
    /// linked module-owned global and is never empty scaffolding by design.
    pub fn from_linked(module: &phalcom_modules::LinkedModule) -> Self {
        let declarations = module
            .bindings
            .local_globals
            .keys()
            .map(|name| RuntimeDeclarationBlueprint::Global {
                symbol: SymbolId {
                    module: module.interface.module.clone(),
                    name: name.clone(),
                },
                mutable: true,
            })
            .collect();
        Self::new(module, declarations)
    }
}

/// Transitional source-compatible name. The payload is now a truthful
/// VM-independent materialization plan rather than a partly-materialized
/// runtime artifact.
pub type ModuleArtifact = ModuleMaterializationPlan;
