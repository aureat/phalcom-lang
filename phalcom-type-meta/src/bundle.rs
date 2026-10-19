//! Module roots, runtime type roots, bundle structure, and occurrences.

use crate::declaration::{
    AliasRecordId, CallableRecordId, CallableSemanticRecord, DeclarationRecordId, DeclarationTypeRecord, FieldRecordId, FieldSemanticRecord, TypeAliasRecord,
};
use crate::fingerprint::Fingerprint128;
use crate::generic::{GenericSignatureRecord, TypeParameterRecord};
use crate::header::{FeatureSectionId, SemanticMetadataHeader};
use crate::identity::{SourceSpanRef, StableDeclarationRef, StableModuleRef};
use crate::kind::KindNodeEntry;
use crate::scoped_type::ScopedTypeNodeEntry;
use crate::type_node::{TypeNodeEntry, TypeNodeId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct ModuleMetadataRoot {
    pub module: StableModuleRef,
    pub declarations: Box<[DeclarationRecordId]>,
    pub aliases: Box<[AliasRecordId]>,
    pub callables: Box<[CallableRecordId]>,
    pub fields: Box<[FieldRecordId]>,
    pub interface_fingerprint: Fingerprint128,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct RuntimeTypeFormKey(pub Box<str>);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct RuntimeTypeFormRoot {
    pub module: StableModuleRef,
    pub local_key: RuntimeTypeFormKey,
    pub form: TypeNodeId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct MetadataExtensionSection {
    pub feature: FeatureSectionId,
    pub schema_version: u32,
    pub required: bool,
    pub semantic_fingerprint: Fingerprint128,
    pub payload: Box<[u8]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum TypeUseRoleRef {
    Parameter,
    Return,
    Field,
    Superclass,
    TypeArgument,
    TypeConstant,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub enum TypeUseStatusRef {
    Known(TypeNodeId),
    InternalClassObject(StableDeclarationRef),
    Dynamic(crate::declaration::DynamicReasonRef),
    Missing,
    Unknown(crate::declaration::UnknownReasonRef),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct TypeUseRecord {
    pub role: TypeUseRoleRef,
    pub status: TypeUseStatusRef,
    pub written: Option<Box<str>>,
    pub source: Option<SourceSpanRef>,
}

/// The complete immutable metadata bundle for a compiled artifact / program.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, phalcom_type_meta_derive::BudgetDecode)]
pub struct SemanticMetadataBundle {
    pub header: SemanticMetadataHeader,
    pub kinds: Box<[KindNodeEntry]>,
    pub types: Box<[TypeNodeEntry]>,
    pub scoped_types: Box<[ScopedTypeNodeEntry]>,
    pub parameters: Box<[TypeParameterRecord]>,
    pub generic_signatures: Box<[GenericSignatureRecord]>,
    pub declarations: Box<[DeclarationTypeRecord]>,
    /// Stable declaration/case catalog required by schema v8. Older schemas
    /// decode with an empty catalog and retain their original validation rules.
    #[serde(default)]
    pub static_domains: Box<[crate::declaration::StaticDomainDeclarationRecord]>,
    pub aliases: Box<[TypeAliasRecord]>,
    pub callables: Box<[CallableSemanticRecord]>,
    pub fields: Box<[FieldSemanticRecord]>,
    pub module_roots: Box<[ModuleMetadataRoot]>,
    pub runtime_roots: Box<[RuntimeTypeFormRoot]>,
    pub occurrences: Box<[TypeUseRecord]>,
    pub extensions: Box<[MetadataExtensionSection]>,
}
