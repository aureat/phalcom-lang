//! Canonical enum and variant semantic declaration products.

use crate::declaration_type::DeclaredTypeFact;
use crate::identity::{DeclarationId, SemanticSourceSpan, VariantConstructorId, VariantFamilyId, VariantFieldId, VariantId};
use crate::surface::MemberVisibility;
use crate::types::case_environment::CaseTypeEnvironment;
use crate::types::id::{TypeId, VariantTypeId};
use crate::types::parameter::GenericSignature;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use std::sync::Arc;

/// Structural shape of an enum variant: singleton value vs constructor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VariantShape {
    Singleton,
    Constructor,
}

/// Structural payload shape of an algebraic case.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AlgebraicCasePayloadKind {
    Singleton,
    Tuple,
    Record,
    ExplicitEmpty,
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
    /// Generic parameters declared by this constructor, owned by its
    /// canonical class-side variant callable identity.
    pub generic_signature: Option<GenericSignature>,
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
    pub payload_kind: AlgebraicCasePayloadKind,
    pub fields: Box<[VariantFieldSemantic]>,
    pub result_type_template: TypeId,
    pub exact_case_template: TypeId,
    pub case_environment: CaseTypeEnvironment,
    pub constructor: Option<VariantConstructorSignature>,
    pub visibility: VariantVisibility,
    pub source: Option<SemanticSourceSpan>,
}

impl VariantInfo {
    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        let mut fields = allocations.vector(self.fields.len())?;
        for field in self.fields.iter() {
            fields.push(VariantFieldSemantic {
                id: field.id.clone_with_allocation_budget(allocations)?,
                local_name: allocations.boxed_str(&field.local_name)?,
                external_label: field.external_label.as_deref().map(|label| allocations.boxed_str(label)).transpose()?,
                declared_type: field.declared_type.clone_with_allocation_budget(allocations)?,
                source: clone_source_span_with_allocation_budget(field.source.as_ref(), allocations)?,
            });
        }
        let constructor = self
            .constructor
            .as_ref()
            .map(|constructor| clone_constructor_with_allocation_budget(constructor, allocations))
            .transpose()?;
        let family = self
            .family
            .as_ref()
            .map(|family| {
                Ok(VariantFamilyId {
                    owner: family.owner.clone_with_allocation_budget(allocations)?,
                    base_name: allocations.boxed_str(&family.base_name)?,
                })
            })
            .transpose()?;
        Ok(Self {
            id: self.id.clone_with_allocation_budget(allocations)?,
            type_handle: self.type_handle,
            family,
            shape: self.shape,
            payload_kind: self.payload_kind,
            fields: allocations.boxed_slice(fields)?,
            result_type_template: self.result_type_template,
            exact_case_template: self.exact_case_template,
            case_environment: self.case_environment.clone_with_allocation_budget(allocations)?,
            constructor,
            visibility: VariantVisibility {
                name: self.visibility.name,
                construct: self.visibility.construct,
                payload: self.visibility.payload,
            },
            source: clone_source_span_with_allocation_budget(self.source.as_ref(), allocations)?,
        })
    }

    pub fn is_singleton(&self) -> bool {
        self.shape == VariantShape::Singleton
    }

    pub fn is_record(&self) -> bool {
        self.payload_kind == AlgebraicCasePayloadKind::Record
    }

    pub fn is_explicit_empty(&self) -> bool {
        self.payload_kind == AlgebraicCasePayloadKind::ExplicitEmpty
    }
}

impl EnumInfo {
    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        let mut variants = allocations.vector(self.variants.len())?;
        for variant in self.variants.iter() {
            variants.push(variant.clone_with_allocation_budget(allocations)?);
        }
        let mut variant_families = allocations.vector(self.variant_families.len())?;
        for family in self.variant_families.iter() {
            variant_families.push(VariantFamilyId {
                owner: family.owner.clone_with_allocation_budget(allocations)?,
                base_name: allocations.boxed_str(&family.base_name)?,
            });
        }
        let static_data = self
            .static_data
            .as_ref()
            .map(|static_data| {
                let mut constructors = allocations.vector(static_data.constructors.len())?;
                constructors.extend_from_slice(&static_data.constructors);
                Ok(crate::algebraic_semantics::StaticAlgebraicDeclarationInfo {
                    domain: static_data.domain,
                    constructors: allocations.boxed_slice(constructors)?,
                })
            })
            .transpose()?;
        Ok(Self {
            owner: self.owner.clone_with_allocation_budget(allocations)?,
            root_form: self.root_form,
            generic_signature: self
                .generic_signature
                .as_ref()
                .map(|signature| signature.clone_with_allocation_budget(allocations))
                .transpose()?,
            static_data,
            default_result_type: self.default_result_type,
            variants: allocations.boxed_slice(variants)?,
            variant_families: allocations.boxed_slice(variant_families)?,
            source: clone_source_span_with_allocation_budget(self.source.as_ref(), allocations)?,
        })
    }
}

fn clone_constructor_with_allocation_budget(
    constructor: &VariantConstructorSignature,
    allocations: &mut AllocationBudget,
) -> Result<VariantConstructorSignature, AllocationError> {
    let mut parameters = allocations.vector(constructor.parameters.len())?;
    for parameter in constructor.parameters.iter() {
        parameters.push(VariantConstructorParameter {
            field: parameter.field.clone_with_allocation_budget(allocations)?,
            external_label: parameter.external_label.as_deref().map(|label| allocations.boxed_str(label)).transpose()?,
            local_name: allocations.boxed_str(&parameter.local_name)?,
            declared_type: parameter.declared_type.clone_with_allocation_budget(allocations)?,
        });
    }
    Ok(VariantConstructorSignature {
        constructor: VariantConstructorId {
            variant: constructor.constructor.variant.clone_with_allocation_budget(allocations)?,
        },
        generic_signature: constructor
            .generic_signature
            .as_ref()
            .map(|signature| signature.clone_with_allocation_budget(allocations))
            .transpose()?,
        parameters: allocations.boxed_slice(parameters)?,
        result_type_template: constructor.result_type_template,
        exact_case_template: constructor.exact_case_template,
        source: clone_source_span_with_allocation_budget(constructor.source.as_ref(), allocations)?,
    })
}

fn clone_source_span_with_allocation_budget(
    source: Option<&SemanticSourceSpan>,
    allocations: &mut AllocationBudget,
) -> Result<Option<SemanticSourceSpan>, AllocationError> {
    source
        .map(|source| {
            Ok(SemanticSourceSpan {
                module: source.module.clone_with_allocation_budget(allocations)?,
                range: source.range,
            })
        })
        .transpose()
}

/// Complete structural semantic metadata for one declared enum root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumInfo {
    pub owner: DeclarationId,
    pub root_form: TypeId,
    pub generic_signature: Option<GenericSignature>,
    /// Canonical static-domain declaration product when this algebraic
    /// declaration is type-only.
    pub static_data: Option<crate::algebraic_semantics::StaticAlgebraicDeclarationInfo>,
    pub default_result_type: TypeId,
    pub variants: Box<[VariantId]>,
    pub variant_families: Box<[VariantFamilyId]>,
    pub source: Option<SemanticSourceSpan>,
}

/// Table of all published enum and variant declarations in a snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnumSemanticTable {
    pub enums: Arc<crate::parent_map::ParentMap<DeclarationId, Arc<EnumInfo>>>,
    pub variants: Arc<crate::parent_map::ParentMap<VariantId, Arc<VariantInfo>>>,
    module_enums: std::collections::HashMap<phalcom_modules::identity::ModuleId, std::collections::BTreeSet<DeclarationId>>,
    parent: Option<Arc<Self>>,
}

impl EnumSemanticTable {
    /// Shares canonical entries and starts with empty local contributions.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            enums: Arc::new(crate::parent_map::ParentMap::with_parent(parent.enums.clone())),
            variants: Arc::new(crate::parent_map::ParentMap::with_parent(parent.variants.clone())),
            module_enums: std::collections::HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn from_frozen_entries_with_allocation_budget(
        enums: Vec<(DeclarationId, Arc<EnumInfo>)>,
        variants: Vec<(VariantId, Arc<VariantInfo>)>,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, crate::parent_map::FrozenParentMapError> {
        let enums = crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(enums, allocations)?;
        let variants = crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(variants, allocations)?;
        let enums = allocations.arc(enums).map_err(crate::parent_map::FrozenParentMapError::Allocation)?;
        let variants = allocations.arc(variants).map_err(crate::parent_map::FrozenParentMapError::Allocation)?;
        Ok(Self {
            enums,
            variants,
            module_enums: std::collections::HashMap::new(),
            parent: None,
        })
    }

    pub fn insert_enum(&mut self, info: Arc<EnumInfo>) {
        self.module_enums.entry(info.owner.module.clone()).or_default().insert(info.owner.clone());
        Arc::make_mut(&mut self.enums).insert(info.owner.clone(), info);
    }

    pub fn insert_variant(&mut self, info: Arc<VariantInfo>) {
        Arc::make_mut(&mut self.variants).insert(info.id.clone(), info);
    }

    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        self.module_enums.remove(module);
        Arc::make_mut(&mut self.enums).retain(|owner, _| &owner.module != module);
        Arc::make_mut(&mut self.variants).retain(|variant, _| &variant.owner.module != module);
    }

    pub fn enum_info(&self, owner: &DeclarationId) -> Option<&EnumInfo> {
        self.enums.get(owner).map(|a| a.as_ref())
    }

    pub fn variant_info(&self, id: &VariantId) -> Option<&VariantInfo> {
        self.variants.get(id).map(|a| a.as_ref())
    }

    /// Iterates enum declarations belonging to the specified module without full-table scanning.
    pub fn iter_module<'a>(&'a self, module: &'a phalcom_modules::identity::ModuleId) -> Box<dyn Iterator<Item = (&'a DeclarationId, &'a Arc<EnumInfo>)> + 'a> {
        let local_iter = self
            .module_enums
            .get(module)
            .into_iter()
            .flat_map(move |ids| ids.iter().filter_map(move |id| self.enums.get(id).map(|info| (id, info))));
        let frozen_iter = self.enums.iter_frozen().filter(move |(owner, _)| &owner.module == module);
        match &self.parent {
            Some(parent) => Box::new(parent.iter_module(module).chain(frozen_iter).chain(local_iter)),
            None => Box::new(frozen_iter.chain(local_iter)),
        }
    }
}
