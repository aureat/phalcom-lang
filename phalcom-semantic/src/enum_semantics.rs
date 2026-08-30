//! Canonical enum and variant semantic declaration products.

use crate::declaration_type::DeclaredTypeFact;
use crate::identity::{DeclarationId, SemanticSourceSpan, VariantConstructorId, VariantFamilyId, VariantFieldId, VariantId};
use crate::surface::MemberVisibility;
use crate::types::case_environment::CaseTypeEnvironment;
use crate::types::id::{TypeId, VariantTypeId};
use crate::types::parameter::GenericSignature;
use std::collections::HashMap;
use std::sync::Arc;

/// Structural shape of an enum variant: singleton value vs constructor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VariantShape {
    Singleton,
    Constructor,
}

/// Multi-axis variant visibility metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantVisibility {
    pub name: MemberVisibility,
    pub construct: MemberVisibility,
    pub payload: MemberVisibility,
}

impl Default for VariantVisibility {
    fn default() -> Self {
        Self {
            name: MemberVisibility::Public,
            construct: MemberVisibility::Public,
            payload: MemberVisibility::Public,
        }
    }
}

/// Semantic identity and declared type of one variant payload field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantFieldSemantic {
    pub id: VariantFieldId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub declared_type: DeclaredTypeFact,
    pub source: Option<SemanticSourceSpan>,
}

/// Formal constructor parameter corresponding to a variant payload field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantConstructorParameter {
    pub field: VariantFieldId,
    pub external_label: Option<Box<str>>,
    pub local_name: Box<str>,
    pub declared_type: DeclaredTypeFact,
}

/// Formal signature for a variant constructor callable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantConstructorSignature {
    pub constructor: VariantConstructorId,
    pub parameters: Box<[VariantConstructorParameter]>,
    pub result_type_template: TypeId,
    pub exact_case_template: TypeId,
    pub source: Option<SemanticSourceSpan>,
}

/// Complete structural semantic metadata for one declared enum variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantInfo {
    pub id: VariantId,
    pub type_handle: VariantTypeId,
    pub family: Option<VariantFamilyId>,
    pub shape: VariantShape,
    pub fields: Box<[VariantFieldSemantic]>,
    pub result_type_template: TypeId,
    pub exact_case_template: TypeId,
    pub case_environment: CaseTypeEnvironment,
    pub constructor: Option<VariantConstructorSignature>,
    pub visibility: VariantVisibility,
    pub source: Option<SemanticSourceSpan>,
}

/// Complete structural semantic metadata for one declared enum root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumInfo {
    pub owner: DeclarationId,
    pub root_form: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub default_result_type: TypeId,
    pub variants: Box<[VariantId]>,
    pub variant_families: Box<[VariantFamilyId]>,
    pub source: Option<SemanticSourceSpan>,
}

/// Table of all published enum and variant declarations in a snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnumSemanticTable {
    pub enums: HashMap<DeclarationId, Arc<EnumInfo>>,
    pub variants: HashMap<VariantId, Arc<VariantInfo>>,
}

impl EnumSemanticTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_enum(&mut self, info: Arc<EnumInfo>) {
        self.enums.insert(info.owner.clone(), info);
    }

    pub fn insert_variant(&mut self, info: Arc<VariantInfo>) {
        self.variants.insert(info.id.clone(), info);
    }

    pub fn enum_info(&self, owner: &DeclarationId) -> Option<&EnumInfo> {
        self.enums.get(owner).map(|a| a.as_ref())
    }

    pub fn variant_info(&self, id: &VariantId) -> Option<&VariantInfo> {
        self.variants.get(id).map(|a| a.as_ref())
    }
}
