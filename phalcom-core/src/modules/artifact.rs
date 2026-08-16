//! VM-independent compiled module metadata.

use phalcom_modules::{LinkedModuleInterface, LinkedReadSpec, ModuleId, SymbolId};

/// Declaration metadata materialized before ordinary module initialization.
#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeDeclarationBlueprint {
    /// A class declaration and its linked superclass identity.
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

/// Output of compiling one linked module.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleMaterializationPlan {
    /// Module identity.
    pub id: ModuleId,
    /// Declaration metadata.
    pub declarations: Vec<RuntimeDeclarationBlueprint>,
    /// Linked interface used by runtime materialization.
    pub interface: LinkedModuleInterface,
    /// Symbolic reads consumed by `GetLinked`.
    pub linked_reads: Vec<LinkedReadSpec>,
}

impl ModuleMaterializationPlan {
    /// Creates an empty artifact for a linked module.
    pub fn empty(module: &phalcom_modules::LinkedModule) -> Self {
        Self {
            id: module.interface.module.clone(),
            declarations: Vec::new(),
            interface: module.interface.clone(),
            linked_reads: module.linked_reads.clone(),
        }
    }
}
