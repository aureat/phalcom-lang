//! VM-independent module materialization plans.

use phalcom_modules::{LinkedModuleInterface, LinkedReadSpec, ModuleId, SymbolId};

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeDeclarationBlueprint {
    Class(ClassBlueprint),
    Global { symbol: SymbolId, mutable: bool },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClassBlueprint {
    pub symbol: SymbolId,
    pub superclass: Option<SymbolId>,
    pub fields: Vec<Box<str>>,
    pub methods: Vec<Box<str>>,
}

/// Immutable, VM-independent plan for materializing one linked module.
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

    pub fn from_linked(module: &phalcom_modules::LinkedModule) -> Self {
        let declarations = module
            .bindings
            .local_globals
            .keys()
            .map(|name| RuntimeDeclarationBlueprint::Global {
                symbol: SymbolId { module: module.interface.module.clone(), name: name.clone() },
                mutable: true,
            })
            .collect();
        Self::new(module, declarations)
    }

    /// Temporary source-compatibility shim for the pre-repair call site. It no
    /// longer creates an empty artifact; it constructs the active declaration
    /// materialization plan.
    #[deprecated(note = "use ModuleMaterializationPlan::from_linked")]
    pub fn empty(module: &phalcom_modules::LinkedModule) -> Self {
        Self::from_linked(module)
    }
}

pub type ModuleArtifact = ModuleMaterializationPlan;
