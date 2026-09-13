//! Canonical data declaration semantic products.

use crate::declaration_type::DeclaredTypeFact;
use crate::identity::{DataComponentId, DataConstructorId, DeclarationId, SemanticSourceSpan};
use crate::types::id::TypeId;
use crate::types::parameter::GenericSignature;
use std::collections::HashMap;
use std::sync::Arc;

/// Structural shape of a data declaration: Tuple vs Record.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DataShape {
    Tuple,
    Record,
}

/// Semantic identity and declared type of one data component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataComponentSemantic {
    pub id: DataComponentId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub declared_type: DeclaredTypeFact,
    pub source: Option<SemanticSourceSpan>,
}

/// Formal constructor parameter corresponding to a data component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataConstructorParameter {
    pub component: DataComponentId,
    pub external_label: Option<Box<str>>,
    pub local_name: Box<str>,
    pub declared_type: DeclaredTypeFact,
}

/// Formal signature for a data constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataConstructorSignature {
    pub constructor: DataConstructorId,
    pub parameters: Box<[DataConstructorParameter]>,
    pub result_type_template: TypeId,
    pub source: Option<SemanticSourceSpan>,
}

/// Complete structural semantic metadata for one declared data root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataInfo {
    pub owner: DeclarationId,
    pub root_form: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub shape: DataShape,
    pub components: Box<[DataComponentSemantic]>,
    pub constructor: DataConstructorSignature,
    pub source: Option<SemanticSourceSpan>,
}

impl DataInfo {
    pub fn find_component(&self, name: &str) -> Option<&DataComponentSemantic> {
        self.components
            .iter()
            .find(|c| c.local_name.as_ref() == name || c.external_label.as_deref() == Some(name))
    }
}

/// Table of all published data declarations in a snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DataSemanticTable {
    pub data_decls: HashMap<DeclarationId, Arc<DataInfo>>,
}

impl DataSemanticTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_data(&mut self, info: Arc<DataInfo>) {
        self.data_decls.insert(info.owner.clone(), info);
    }

    pub fn get(&self, owner: &DeclarationId) -> Option<&Arc<DataInfo>> {
        self.data_decls.get(owner)
    }

    pub fn data_info(&self, owner: &DeclarationId) -> Option<&Arc<DataInfo>> {
        self.get(owner)
    }

    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        self.data_decls.retain(|owner, _| &owner.module != module);
    }
}
