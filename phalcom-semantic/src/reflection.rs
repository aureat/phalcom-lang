use crate::enum_semantics::{EnumInfo, VariantInfo, VariantShape, VariantVisibility};
use crate::identity::{CallableId, DeclarationId, SemanticSourceSpan, VariantFamilyId, VariantFieldId, VariantId};
use crate::types::id::{TypeId, TypeParameterId};
use crate::types::store::TypeStore;

/// Protocol-neutral reflection descriptor for an enum declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumReflection {
    pub declaration: DeclarationId,
    pub generic_parameters: Box<[TypeParameterId]>,
    pub variants: Box<[VariantId]>,
    pub families: Box<[VariantFamilyId]>,
    pub native: bool,
    pub source: Option<SemanticSourceSpan>,
}

/// Protocol-neutral reflection descriptor for a variant declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantReflection {
    pub variant: VariantId,
    pub family: Option<VariantFamilyId>,
    pub shape: VariantShape,
    pub fields: Box<[VariantFieldId]>,
    pub result_template: TypeId,
    pub exact_case_template: TypeId,
    pub visibility: VariantVisibility,
    pub case_behavior: Box<[CallableId]>,
}

/// Protocol-neutral reflection descriptor for a variant family.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantFamilyReflection {
    pub family: VariantFamilyId,
    pub members: Box<[VariantId]>,
}

/// Protocol-neutral reflection descriptor for a variant payload field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantFieldReflection {
    pub id: VariantFieldId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub declared_type: TypeId,
}

/// Protocol-neutral reflection descriptor for a specialized field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecializedVariantFieldReflection {
    pub id: VariantFieldId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub specialized_type: Option<TypeId>,
}

/// Protocol-neutral reflection descriptor for a canonical specialized exact-case type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactCaseTypeReflection {
    pub ty: TypeId,
    pub variant: VariantId,
    pub enum_type: TypeId,
    pub fields: Box<[SpecializedVariantFieldReflection]>,
    pub result_type: TypeId,
}

impl EnumReflection {
    pub fn from_enum_info(info: &EnumInfo, native: bool) -> Self {
        let generic_params = info
            .generic_signature
            .as_ref()
            .map(|sig| sig.parameters.clone())
            .unwrap_or_default();
        Self {
            declaration: info.owner.clone(),
            generic_parameters: generic_params,
            variants: info.variants.clone(),
            families: info.variant_families.clone(),
            native,
            source: info.source.clone(),
        }
    }
}

impl VariantReflection {
    pub fn from_variant_info(info: &VariantInfo) -> Self {
        let field_ids: Vec<VariantFieldId> = info.fields.iter().map(|f| f.id.clone()).collect();
        Self {
            variant: info.id.clone(),
            family: info.family.clone(),
            shape: info.shape,
            fields: field_ids.into_boxed_slice(),
            result_template: info.result_type_template,
            exact_case_template: info.exact_case_template,
            visibility: info.visibility.clone(),
            case_behavior: Box::new([]),
        }
    }
}

impl ExactCaseTypeReflection {
    pub fn from_exact_case(
        ty: TypeId,
        variant_info: &VariantInfo,
        enum_type: TypeId,
        _store: &TypeStore,
    ) -> Self {
        let mut fields = Vec::new();
        for f in variant_info.fields.iter() {
            fields.push(SpecializedVariantFieldReflection {
                id: f.id.clone(),
                local_name: f.local_name.clone(),
                external_label: f.external_label.clone(),
                specialized_type: f.declared_type.canonical_type(),
            });
        }
        Self {
            ty,
            variant: variant_info.id.clone(),
            enum_type,
            fields: fields.into_boxed_slice(),
            result_type: enum_type,
        }
    }
}
