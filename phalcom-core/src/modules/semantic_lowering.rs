//! Semantic-to-Codegen Lowering Projection (Part 4).
//!
//! Bridges the formal `SemanticSnapshot` products to compact, immutable,
//! backend-facing lowering specifications attached by `LoweringSite` keys.

use phalcom_common::range::SourceRange;
use phalcom_modules::{DeclarationId, ModuleId, SourceId};
use phalcom_semantic::associated::AssociatedMemberId;
use phalcom_semantic::checker::associated::{
    AssociatedResolution, AssociatedResolutionKind, BehavioralFamilySpec, CallableReferenceResolution, CallableReferenceResolutionKind, FamilyApplicationKind,
    FamilyApplicationResolution, FamilyApplicationSelection,
};
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{
    BindingId, CallableId, DataComponentId, DataConstructorId, ExpressionId, ImplId, InvocationTargetId, SemanticTargetId, VariantFieldId, VariantId,
};
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_semantic::types::denotation::{AssociatedValueDenotation, SemanticDenotation};
use phalcom_semantic::types::family::{FamilyMemberTypeKind, FamilyOperationShape};
use phalcom_semantic::types::id::TypeId;
use phalcom_semantic::types::store::TypeData;
use crate::typing::{RuntimeTypeRecipe, RuntimeTypeRef};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;
use thiserror::Error;

/// Classification of an AST expression site for lowering attachment.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LoweringSiteKind {
    AssociatedLookup,
    AssociatedInvoke,
    CallableReference,
    FamilyApplication,
    Match,
    ConditionalInvoke,
}

/// Compiler-facing lowering attachment key.
/// Keyed by source identity, source range, and site kind (no semantic IDs in AST).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LoweringSite {
    pub source: SourceId,
    pub range: SourceRange,
    pub kind: LoweringSiteKind,
}

impl LoweringSite {
    pub fn new(source: SourceId, range: SourceRange, kind: LoweringSiteKind) -> Self {
        Self { source, range, kind }
    }
}

/// Executable rest lane handling mode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExecutableRestMode {
    None,
    Positional,
    Labeled,
    Complete,
}

/// Exact resolved invocation target specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutableInvocationTarget {
    Behavioral {
        lookup_owner: DeclarationId,
        callable: CallableId,
        operation: FamilyOperationShape,
        rest_mode: ExecutableRestMode,
    },
    VariantConstructor {
        variant: VariantId,
    },
    DataConstructor {
        constructor: DataConstructorId,
        construction: Arc<DataConstructionLoweringSpec>,
    },
}

/// Target of a member entry in an executable family descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutableFamilyTarget {
    Singleton {
        variant: VariantId,
    },
    Behavioral {
        target: ExecutableInvocationTarget,
    },
    VariantConstructor {
        variant: VariantId,
    },
    DataConstructor {
        constructor: DataConstructorId,
        construction: Arc<DataConstructionLoweringSpec>,
    },
}

/// One executable member entry in a frozen family descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableFamilyEntry {
    pub operation: FamilyOperationShape,
    pub member_kind: FamilyMemberTypeKind,
    pub target: ExecutableFamilyTarget,
}

/// Frozen executable family descriptor.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutableFamilyDescriptor {
    pub entries: Box<[ExecutableFamilyEntry]>,
}

/// Candidate for dynamic family pack invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableFamilyCandidate {
    pub operation: FamilyOperationShape,
    pub target: Option<ExecutableInvocationTarget>,
}

/// Candidate set for dynamic family pack invocation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutableFamilyCandidateSet {
    pub candidates: Box<[ExecutableFamilyCandidate]>,
}

/// Semantic selection evidence for one conditional member retained by a
/// receiver-bound behavior family. The compiler turns the canonical callable
/// into a fallback method handle; it does not re-solve the impl domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableConditionalFamilyEntry {
    pub operation: FamilyOperationShape,
    pub impl_id: ImplId,
    pub callable: CallableId,
    pub declaring_owner: DeclarationId,
    pub side: phalcom_semantic::identity::DispatchSide,
}

/// Lowering specification for an associated expression site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssociatedLoweringSpec {
    /// Canonical singleton variant load (immediate value).
    SingletonLoad { variant: VariantId },
    /// Fresh constructor case allocation.
    ConstructVariant { variant: VariantId, arity: u8 },
    /// Data object construction.
    ConstructData {
        constructor: DataConstructorId,
        arity: u8,
        construction: Arc<DataConstructionLoweringSpec>,
    },
    /// Data component projection.
    GetDataComponent { component: DataComponentId, logical_index: u32 },
    /// Direct resolved behavioral call (no hierarchy walk, no dNU).
    InvokeResolvedAssociated { target: ExecutableInvocationTarget, arity: u8 },
    /// Exact behavioral member reification as BoundMethod.
    MakeResolvedBoundMethod { target: ExecutableInvocationTarget },
    /// Exact variant constructor reification as closure thunk.
    MakeVariantConstructorThunk { variant: VariantId, operation: FamilyOperationShape },
    /// Exact data constructor reification as closure thunk.
    MakeDataConstructorThunk {
        constructor: DataConstructorId,
        operation: FamilyOperationShape,
        construction: Arc<DataConstructionLoweringSpec>,
    },
    /// Frozen whole-family capture.
    MakeAssociatedFamily { descriptor: Arc<ExecutableFamilyDescriptor> },
    /// Dynamic associated invocation over frozen candidate set.
    DynamicInvoke { candidates: Box<[ExecutableFamilyCandidate]> },
}

/// Canonical data component lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataComponentLoweringSpec {
    pub id: DataComponentId,
    pub local_name: Box<str>,
    pub logical_index: u32,
}

/// Data declaration lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataDeclarationLoweringSpec {
    pub owner: DeclarationId,
    pub layout: crate::product::ProductLayoutSpec,
    pub components: Box<[DataComponentLoweringSpec]>,
}

/// Data construction lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataConstructionLoweringSpec {
    pub constructor: DataConstructorId,
    pub exact_type: Arc<phalcom_type_meta::SemanticMetadataBundle>,
    pub layout: crate::product::ProductLayoutSpec,
    pub argument_to_component: Box<[u32]>,
}

/// Lowering specification for an accepted member defined in an inherent `impl` block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplMemberLowering {
    pub callable: CallableId,
    pub source_member_index: usize,
}

/// Target entity of an inherent `impl` block.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InherentImplLoweringTarget {
    Declaration(DeclarationId),
    ExactEnumCase(VariantId),
}

/// Inherent `impl` declaration lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplLoweringSpec {
    pub id: ImplId,
    pub target: InherentImplLoweringTarget,
    pub members: Box<[InherentImplMemberLowering]>,
    pub is_conditional: bool,
}

/// Kind of an anonymous product construction whose shape is statically known.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnonymousProductConstructionKind {
    Tuple {
        positional_len: u32,
        labels: Box<[Box<str>]>,
    },
    Record {
        presentation_labels: Box<[Box<str>]>,
        logical_labels: Box<[Box<str>]>,
        source_to_logical: Box<[u32]>,
    },
}

/// Compiler-executable specification for a statically closed anonymous product.
///
/// Labels remain source-owned strings at this boundary. The VM interns them
/// exactly once while materializing the shared runtime shape; labels are never
/// pushed as component values for this construction path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousProductConstructionLoweringSpec {
    pub kind: AnonymousProductConstructionKind,
    pub layout: crate::product::ProductLayoutSpec,
    pub type_recipe: RuntimeTypeRecipe,
}

/// Lowering specification for a prefix-`&` callable reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallableReferenceLoweringSpec {
    /// Ordinary receiver-bound family capture. Runtime retains the captured
    /// receiver and performs live behavioral dispatch on future invocation.
    MakeBoundFamily {
        spec: BehavioralFamilySpec,
        conditional_members: Box<[ExecutableConditionalFamilyEntry]>,
    },
    /// Exact associated behavioral member reification as a bound method.
    MakeResolvedBoundMethod { target: ExecutableInvocationTarget },
    /// Exact associated variant constructor reification as a closure thunk.
    MakeVariantConstructorThunk { variant: VariantId, operation: FamilyOperationShape },
    /// Exact associated data constructor reification as a closure thunk.
    MakeDataConstructorThunk {
        constructor: DataConstructorId,
        operation: FamilyOperationShape,
        construction: Arc<DataConstructionLoweringSpec>,
    },
    /// Frozen associated family capture.
    MakeAssociatedFamily { descriptor: Arc<ExecutableFamilyDescriptor> },
}

/// Lowering specification for an application on a first-class family value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum FamilyApplicationLoweringSpec {
    /// Statically known operation invocation on family.
    Static {
        kind: FamilyApplicationKind,
        operation: FamilyOperationShape,
        target: Option<ExecutableInvocationTarget>,
        arity: u8,
    },
    /// Dynamic pack invocation restricted to frozen candidates.
    DynamicPack {
        kind: FamilyApplicationKind,
        candidates: Box<[ExecutableFamilyCandidate]>,
    },
}

/// Canonical payload field lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantFieldLoweringSpec {
    pub id: VariantFieldId,
    pub local_name: Box<str>,
    pub slot: u16,
}

/// Variant lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantLoweringSpec {
    pub id: VariantId,
    pub shape: VariantShape,
    pub payload_fields: Box<[VariantFieldLoweringSpec]>,
    pub layout: Option<crate::product::ProductLayoutSpec>,
}

/// Enum declaration lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumLoweringSpec {
    pub owner: DeclarationId,
    pub representation: crate::adt::RuntimeAdtRepresentation,
    pub variants: Box<[VariantLoweringSpec]>,
}

/// Executable binding specification for an arm or pattern context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableBindingSpec {
    pub binding: phalcom_semantic::identity::BindingId,
    pub name: Box<str>,
    pub range: SourceRange,
}

/// Field projection for an exact variant candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableFieldProjection {
    pub field_id: VariantFieldId,
    pub slot: u16,
    pub child: ExecutablePattern,
}

/// Exact resolved variant candidate in an executable pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableVariantCandidate {
    pub variant: VariantId,
    pub fields: Box<[ExecutableFieldProjection]>,
}

/// Backend executable pattern structure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutablePattern {
    Wildcard,
    Binding {
        binding_index: u32,
        name: Box<str>,
    },
    Variant {
        candidates: Box<[ExecutableVariantCandidate]>,
    },
    Or {
        alternatives: Box<[ExecutablePattern]>,
    },
    Tuple {
        elements: Box<[ExecutablePattern]>,
    },
    List {
        elements: Box<[ExecutablePattern]>,
        rest: Option<Box<ExecutablePattern>>,
    },
    Record {
        entries: Box<[(Box<str>, ExecutablePattern)]>,
    },
    Map {
        entries: Box<[(phalcom_ast::ast::MapPatternKey, ExecutablePattern)]>,
    },
}

/// Executable match arm lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableMatchArm {
    pub arm_index: u32,
    pub pattern: ExecutablePattern,
    pub bindings: Box<[ExecutableBindingSpec]>,
}

/// Match expression lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchLoweringSpec {
    pub arms: Box<[ExecutableMatchArm]>,
}

/// Conditional inherent method invocation lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConditionalInvocationSpec {
    pub callable: CallableId,
    /// Canonical semantic owner whose runtime behavior participates in the
    /// override check. Keep the complete declaration identity: a leaf name is
    /// not sufficient in a module-scoped class namespace.
    pub declaring_owner: DeclarationId,
    pub side: phalcom_semantic::identity::DispatchSide,
}

/// Complete compiled lowering semantics for a single module.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleLoweringSemantics {
    pub module: ModuleId,
    /// Canonical local-binding attachments keyed by their current source
    /// ranges. The compiler may use these facts to classify AST uses, but it
    /// never resolves names or scopes independently.
    pub bindings: BTreeMap<SourceRange, BindingId>,
    pub anonymous_products: BTreeMap<SourceRange, Arc<AnonymousProductConstructionLoweringSpec>>,
    pub enums: Box<[EnumLoweringSpec]>,
    pub data_decls: Box<[DataDeclarationLoweringSpec]>,
    pub inherent_impls: Box<[InherentImplLoweringSpec]>,
    pub associated: BTreeMap<LoweringSite, AssociatedLoweringSpec>,
    pub callable_references: BTreeMap<LoweringSite, CallableReferenceLoweringSpec>,
    pub family_values: BTreeSet<LoweringSite>,
    pub family_application_sites: BTreeSet<LoweringSite>,
    pub family_applications: BTreeMap<LoweringSite, FamilyApplicationLoweringSpec>,
    pub matches: BTreeMap<LoweringSite, MatchLoweringSpec>,
    pub conditional_invocations: BTreeMap<LoweringSite, ConditionalInvocationSpec>,
}

impl ModuleLoweringSemantics {
    pub fn new(module: ModuleId) -> Self {
        Self {
            module,
            bindings: BTreeMap::new(),
            anonymous_products: BTreeMap::new(),
            enums: Box::new([]),
            data_decls: Box::new([]),
            inherent_impls: Box::new([]),
            associated: BTreeMap::new(),
            callable_references: BTreeMap::new(),
            family_values: BTreeSet::new(),
            family_application_sites: BTreeSet::new(),
            family_applications: BTreeMap::new(),
            matches: BTreeMap::new(),
            conditional_invocations: BTreeMap::new(),
        }
    }
}

/// Errors occurring during semantic-to-lowering projection.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum ProjectionError {
    #[error("data type metadata projection failed: {0}")]
    DataTypeMetadata(String),
    #[error("ambiguous lowering site attachment at {0:?}")]
    AmbiguousLoweringSiteAttachment(LoweringSite),
    #[error("missing source range for expression {0:?}")]
    MissingSourceRange(ExpressionId),
    #[error("missing variant metadata for {0:?}")]
    MissingVariantMetadata(VariantId),
    #[error("missing field layout for field {0:?} in variant {1:?}")]
    MissingFieldLayout(Box<VariantFieldId>, Box<VariantId>),
    #[error("missing pattern binding for {0:?}")]
    MissingPatternBinding(phalcom_semantic::identity::BindingId),
    #[error("pattern binding index overflow for {0}")]
    PatternBindingIndexOverflow(usize),
    #[error("arity overflow for length {0}")]
    ArityOverflow(usize),
    #[error("slot overflow for index {0}")]
    SlotOverflow(usize),
    #[error("non-proven match reached executable lowering for expression {0:?}")]
    NonProvenMatch(ExpressionId),
    #[error("callable reference carried an invalid lowering specification")]
    InvalidCallableReferenceSpec,
    #[error("missing constructor metadata for variant {0:?}")]
    MissingConstructorMetadata(VariantId),
    #[error("missing data metadata for {0:?}")]
    MissingDataMetadata(DeclarationId),
    #[error("invalid anonymous product construction specification")]
    InvalidAnonymousProductSpec,
    #[error("open or unrepresentable data construction type {result_type:?} for {constructor:?}")]
    OpenDataConstructionType { constructor: DataConstructorId, result_type: TypeId },
}

fn contains_exact_case(store: &phalcom_semantic::types::TypeStore, ty: TypeId) -> bool {
    use phalcom_semantic::types::store::TypeData;
    match store.get(ty) {
        TypeData::ExactCase { .. } => true,
        TypeData::Applied { origin, arguments } => contains_exact_case(store, *origin) || arguments.iter().any(|&a| contains_exact_case(store, a)),
        TypeData::Union(members) => members.iter().any(|&m| contains_exact_case(store, m)),
        TypeData::Tuple(elems) => elems.iter().any(|e| contains_exact_case(store, e.ty)),
        TypeData::Record(row_id) => {
            let row = store.record_row(*row_id);
            row.fields.iter().any(|f| contains_exact_case(store, f.ty))
        }
        TypeData::Callable(call) => call.parameters.iter().any(|p| contains_exact_case(store, p.ty)) || contains_exact_case(store, call.return_type),
        TypeData::Lambda(lambda_id) => {
            let mut free_types = Vec::new();
            let lambda = store.arena().get_lambda(*lambda_id);
            store.arena().collect_free_types(lambda.body, &mut free_types);
            free_types.into_iter().any(|free_type| contains_exact_case(store, free_type))
        }
        _ => false,
    }
}

fn build_data_construction_spec(
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    constructor: &DataConstructorId,
    result_type: TypeId,
    arity: u8,
    argument_mapping: Option<Box<[u32]>>,
) -> Result<Arc<DataConstructionLoweringSpec>, ProjectionError> {
    if phalcom_semantic::checker::associated::contains_any_type_parameter(&snapshot.store, result_type) || contains_exact_case(&snapshot.store, result_type) {
        return Err(ProjectionError::OpenDataConstructionType {
            constructor: constructor.clone(),
            result_type,
        });
    }

    let info = snapshot
        .data_semantics
        .data_info(&constructor.owner)
        .ok_or_else(|| ProjectionError::MissingDataMetadata(constructor.owner.clone()))?;
    let exporter =
        phalcom_semantic::metadata::MetadataExporter::new(&snapshot.store, None, None, None, phalcom_type_meta::header::MetadataProfile::RuntimePublic)
            .with_project_universe(projects);
    let metadata = exporter
        .build_bundle(&[(&constructor.owner.module, "data", result_type)])
        .map_err(|error| ProjectionError::DataTypeMetadata(error.to_string()))?;
    let argument_to_component = argument_mapping.unwrap_or_else(|| (0..u32::from(arity)).collect());
    Ok(Arc::new(DataConstructionLoweringSpec {
        constructor: constructor.clone(),
        exact_type: Arc::new(metadata),
        layout: crate::product::ProductLayoutSpec::new(
            info.components
                .iter()
                .enumerate()
                .map(|(i, component)| crate::product::ProductComponentSpec {
                    logical_index: i as u32,
                    repr: project_slot_repr(&component.declared_type, snapshot),
                })
                .collect(),
        ),
        argument_to_component,
    }))
}

fn project_data_constructor_target(
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    constructor: &DataConstructorId,
    callable_type: TypeId,
    argument_mapping: Option<Box<[u32]>>,
) -> Result<ExecutableInvocationTarget, ProjectionError> {
    use phalcom_semantic::types::store::TypeData;
    let (result_type, arity) = match snapshot.store.get(callable_type) {
        TypeData::Callable(call) => (call.return_type, call.parameters.len()),
        _ => {
            let info = snapshot
                .data_semantics
                .data_info(&constructor.owner)
                .ok_or_else(|| ProjectionError::MissingDataMetadata(constructor.owner.clone()))?;
            (info.constructor.result_type_template, info.components.len())
        }
    };
    let arity_u8 = u8::try_from(arity).map_err(|_| ProjectionError::ArityOverflow(arity))?;
    let construction = build_data_construction_spec(snapshot, projects, constructor, result_type, arity_u8, argument_mapping)?;
    Ok(ExecutableInvocationTarget::DataConstructor {
        constructor: constructor.clone(),
        construction,
    })
}

/// Projects formal snapshot products into an immutable `ModuleLoweringSemantics` bundle.
pub fn build_module_lowering_semantics(
    module: &ModuleId,
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
) -> Result<ModuleLoweringSemantics, ProjectionError> {
    build_module_lowering_semantics_with_runtime_types(module, snapshot, projects, &BTreeMap::new())
}

pub fn build_module_lowering_semantics_with_runtime_types(
    module: &ModuleId,
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    runtime_type_roots: &BTreeMap<SourceRange, RuntimeTypeRef>,
) -> Result<ModuleLoweringSemantics, ProjectionError> {
    let source_id = if let Some(parsed_unit) = snapshot.sources.get(module) {
        parsed_unit
            .source
            .as_ref()
            .map(|s| s.source_id.clone())
            .unwrap_or_else(|| SourceId(module.to_string().into_boxed_str()))
    } else {
        SourceId(module.to_string().into_boxed_str())
    };

    // Project canonical binding identity once at the semantic/codegen
    // boundary. Declaration sites come from the formal callable attachment;
    // variable occurrences come from the source-index target attachments.
    // Both paths are keyed by source-site identity, so this projection does
    // not recreate lexical name resolution in the optimizer.
    let mut bindings = BTreeMap::new();
    if let Some(module_index) = snapshot.source_index().module(module) {
        for callable_id in snapshot.callable_analyses.keys().filter(|callable| callable.owner.module() == module) {
            let Some(attachment) = snapshot.source_index().formal_attachment(callable_id) else {
                continue;
            };
            let binding_by_site = attachment
                .formal_bindings
                .iter()
                .map(|(binding, site)| (site, *binding))
                .collect::<BTreeMap<_, _>>();

            for (binding, site) in &attachment.formal_bindings {
                if let Some(source_site) = snapshot.source_index().source_site(site) {
                    bindings.insert(source_site.range, *binding);
                }
            }

            for occurrence in module_index.occurrences.all() {
                let Some(SemanticTargetId::Binding(site)) = module_index.occurrences.target_for(&occurrence.site) else {
                    continue;
                };
                if let Some(binding) = binding_by_site.get(site) {
                    bindings.insert(occurrence.range, *binding);
                }
            }
        }
    }

    // 1. Project Enums in this module
    let core_ids = phalcom_semantic::core_surface::CoreDeclarationIds::default();
    let mut enums = Vec::new();
    for (owner, enum_info) in &snapshot.enum_semantics.enums {
        if owner.module != *module {
            continue;
        }
        let mut variants = Vec::new();
        for variant_id in enum_info.variants.iter() {
            let vinfo = snapshot
                .enum_semantics
                .variant_info(variant_id)
                .ok_or_else(|| ProjectionError::MissingVariantMetadata(variant_id.clone()))?;
            let shape = vinfo.shape;
            let mut payload_fields = Vec::new();
            for (idx, field) in vinfo.fields.iter().enumerate() {
                let slot = u16::try_from(idx).map_err(|_| ProjectionError::SlotOverflow(idx))?;
                payload_fields.push(VariantFieldLoweringSpec {
                    id: field.id.clone(),
                    local_name: field.local_name.clone(),
                    slot,
                });
            }
            variants.push(VariantLoweringSpec {
                id: variant_id.clone(),
                shape,
                layout: (!core_ids.is_option(owner)).then(|| {
                    crate::product::ProductLayoutSpec::new(
                        vinfo
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(index, field)| crate::product::ProductComponentSpec {
                                logical_index: index as u32,
                                repr: project_slot_repr(&field.declared_type, snapshot),
                            })
                            .collect(),
                    )
                }),
                payload_fields: payload_fields.into_boxed_slice(),
            });
        }
        let representation = if core_ids.is_option(owner) {
            crate::adt::RuntimeAdtRepresentation::NativeOption
        } else {
            crate::adt::RuntimeAdtRepresentation::General
        };
        enums.push(EnumLoweringSpec {
            owner: owner.clone(),
            representation,
            variants: variants.into_boxed_slice(),
        });
    }
    enums.sort_by(|a, b| a.owner.cmp(&b.owner));

    // 2. Project Data Declarations in this module
    let mut data_decls = Vec::new();
    for (owner, data_info) in &snapshot.data_semantics.data_decls {
        if owner.module != *module {
            continue;
        }
        let mut components = Vec::new();
        let mut component_specs = Vec::new();
        for (idx, comp) in data_info.components.iter().enumerate() {
            let logical_index = u32::try_from(idx).map_err(|_| ProjectionError::SlotOverflow(idx))?;
            components.push(DataComponentLoweringSpec {
                id: comp.id.clone(),
                local_name: comp.local_name.clone(),
                logical_index,
            });
            let repr = project_slot_repr(&comp.declared_type, snapshot);
            component_specs.push(crate::product::ProductComponentSpec { logical_index, repr });
        }
        let layout = crate::product::ProductLayoutSpec::new(component_specs);
        data_decls.push(DataDeclarationLoweringSpec {
            owner: owner.clone(),
            layout,
            components: components.into_boxed_slice(),
        });
    }
    data_decls.sort_by(|a, b| a.owner.cmp(&b.owner));

    // 2b. Project the semantic owner's exact accepted inherent definitions.
    // Source locations locate bodies; they never establish authorization.
    let mut inherent_impls_by_id = BTreeMap::<ImplId, (InherentImplLoweringTarget, Vec<InherentImplMemberLowering>)>::new();
    for definition in snapshot.callable_definitions.values() {
        let phalcom_semantic::impls::CallableDefinitionOrigin::InherentImpl(impl_id) = &definition.origin else {
            continue;
        };
        if impl_id.module != *module {
            continue;
        }
        if !snapshot.callable_analyses.contains_key(&definition.callable) {
            continue;
        }
        let target = match &definition.callable.owner {
            phalcom_semantic::identity::CallableOwnerId::Declaration(decl) => InherentImplLoweringTarget::Declaration(decl.clone()),
            phalcom_semantic::identity::CallableOwnerId::Variant(var) => InherentImplLoweringTarget::ExactEnumCase(var.clone()),
        };
        let entry = inherent_impls_by_id.entry(impl_id.clone()).or_insert_with(|| (target.clone(), Vec::new()));
        debug_assert_eq!(entry.0, target, "one inherent impl cannot contribute to multiple targets");
        entry.1.push(InherentImplMemberLowering {
            callable: definition.callable.clone(),
            source_member_index: definition.source_member_index,
        });
    }
    let mut inherent_impls = inherent_impls_by_id
        .into_iter()
        .map(|(id, (target, mut members))| {
            members.sort_by_key(|member| member.source_member_index);
            let is_conditional = match &target {
                InherentImplLoweringTarget::Declaration(decl) => snapshot
                    .dispatch
                    .get_conditional_members(&phalcom_semantic::impls::InherentImplTarget::Declaration(decl.clone()))
                    .map_or(false, |set| set.members.iter().any(|m| m.impl_id == id)),
                InherentImplLoweringTarget::ExactEnumCase(var) => snapshot
                    .dispatch
                    .get_conditional_members(&phalcom_semantic::impls::InherentImplTarget::ExactEnumCase(var.clone()))
                    .map_or(false, |set| set.members.iter().any(|m| m.impl_id == id)),
            };
            InherentImplLoweringSpec {
                id,
                target,
                members: members.into_boxed_slice(),
                is_conditional,
            }
        })
        .collect::<Vec<_>>();
    inherent_impls.sort_by(|a, b| a.id.cmp(&b.id));

    // 3. Project Associated Expressions & Family Applications
    let mut associated = BTreeMap::new();
    let mut callable_references = BTreeMap::new();
    let mut family_values = BTreeSet::new();
    let mut family_application_sites = BTreeSet::new();
    let mut family_applications = BTreeMap::new();
    let mut matches = BTreeMap::new();
    let mut conditional_invocations = BTreeMap::new();

    for (callable_id, analysis) in snapshot.callable_analyses.iter() {
        if callable_id.owner.module() != module {
            continue;
        }

        // Associated resolutions
        for (expr_id, resolution) in analysis.associated_resolutions.iter() {
            let expr_analysis = analysis.expressions.get(expr_id);
            let range = match expr_analysis {
                Some(ea) => ea.range,
                None => return Err(ProjectionError::MissingSourceRange(*expr_id)),
            };

            let (kind, spec) = project_associated_resolution(resolution, snapshot, projects)?;
            let site = LoweringSite::new(source_id.clone(), range, kind);

            if associated.contains_key(&site) {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
            }
            associated.insert(site, spec);
        }

        // Prefix-`&` callable references have a separate semantic product and
        // lowering lane. Their receiver expression is compiled by the
        // expression visitor at the attached source range.
        for (expr_id, resolution) in analysis.callable_reference_resolutions.iter() {
            let expr_analysis = analysis.expressions.get(expr_id);
            let range = match expr_analysis {
                Some(ea) => ea.range,
                None => return Err(ProjectionError::MissingSourceRange(*expr_id)),
            };
            let spec = project_callable_reference_resolution(resolution, snapshot, projects)?;
            let site = LoweringSite::new(source_id.clone(), range, LoweringSiteKind::CallableReference);
            if callable_references.contains_key(&site) {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
            }
            callable_references.insert(site, spec);
        }

        // Family-valued expressions
        for expression in analysis.expressions.values() {
            let is_associated_family = matches!(
                expression.denotation.as_ref(),
                Some(SemanticDenotation::AssociatedValue(assoc))
                    if matches!(&**assoc, AssociatedValueDenotation::Family { .. })
            ) || matches!(expression.denotation.as_ref(), Some(SemanticDenotation::BehavioralFamily(_)));
            if is_associated_family
                && let Some(ty) = expression.knowledge.ty()
                && matches!(snapshot.store.get(ty), TypeData::Family(_))
            {
                family_values.insert(LoweringSite::new(source_id.clone(), expression.range, LoweringSiteKind::FamilyApplication));
            }
        }

        // Family applications
        for (expr_id, fam_app) in analysis.family_applications.iter() {
            let expr_analysis = analysis.expressions.get(expr_id);
            let range = match expr_analysis {
                Some(ea) => ea.range,
                None => return Err(ProjectionError::MissingSourceRange(*expr_id)),
            };

            let spec = project_family_application(snapshot, projects, fam_app)?;
            let site = LoweringSite::new(source_id.clone(), range, LoweringSiteKind::FamilyApplication);
            family_application_sites.insert(site.clone());

            if family_applications.contains_key(&site) {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
            }
            family_applications.insert(site, spec);
        }

        // 3. Project Match expressions
        for (expr_id, match_resolution) in analysis.match_resolutions.iter() {
            let expr_analysis = analysis.expressions.get(expr_id);
            let range = match expr_analysis {
                Some(ea) => ea.range,
                None => return Err(ProjectionError::MissingSourceRange(*expr_id)),
            };

            let spec = project_match_resolution(match_resolution, snapshot)?;
            let site = LoweringSite::new(source_id.clone(), range, LoweringSiteKind::Match);

            if matches.contains_key(&site) {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
            }
            matches.insert(site, spec);
        }

        // 4. Project Conditional Inherent Invocations
        for expression in analysis.expressions.values() {
            if let Some(selection) = &expression.conditional_dispatch {
                let site = LoweringSite::new(source_id.clone(), expression.range, LoweringSiteKind::ConditionalInvoke);
                conditional_invocations.insert(
                    site,
                    ConditionalInvocationSpec {
                        callable: selection.callable.clone(),
                        declaring_owner: selection.declaring_owner.clone(),
                        side: selection.side,
                    },
                );
            }
        }
    }

    let anonymous_products = snapshot
        .sources
        .get(module)
        .map(|source| project_anonymous_products(module, snapshot, &source.program, runtime_type_roots))
        .unwrap_or_default();

    Ok(ModuleLoweringSemantics {
        module: module.clone(),
        bindings,
        anonymous_products,
        enums: enums.into_boxed_slice(),
        data_decls: data_decls.into_boxed_slice(),
        inherent_impls: inherent_impls.into_boxed_slice(),
        associated,
        callable_references,
        family_values,
        family_application_sites,
        family_applications,
        matches,
        conditional_invocations,
    })
}

fn project_slot_repr(declared_type: &phalcom_semantic::DeclaredTypeFact, snapshot: &SemanticSnapshot) -> crate::product::ProductSlotRepr {
    let core_ids = phalcom_semantic::core_surface::CoreDeclarationIds::default();
    if let Some(ty) = declared_type.canonical_type() {
        if let TypeData::Nominal { declaration } = snapshot.store.get(ty) {
            if declaration == &core_ids.int {
                // Int includes heap-backed arbitrary-precision values; only a range proof permits Int64.
                return crate::product::ProductSlotRepr::Value;
            } else if declaration == &core_ids.float {
                return crate::product::ProductSlotRepr::Float64;
            } else if declaration == &core_ids.bool_ {
                return crate::product::ProductSlotRepr::Bool;
            } else if declaration == &core_ids.symbol {
                return crate::product::ProductSlotRepr::Symbol;
            }
        }
    }
    crate::product::ProductSlotRepr::Value
}

#[derive(Clone, Debug)]
enum AnonymousProductSource {
    Tuple {
        range: SourceRange,
        positional_len: u32,
        labels: Vec<Box<str>>,
        component_ranges: Vec<SourceRange>,
    },
    Record {
        range: SourceRange,
        labels: Vec<Box<str>>,
        component_ranges: Vec<SourceRange>,
    },
}

fn static_product_label(label: &phalcom_ast::ast::ProductLabel) -> Option<Box<str>> {
    match label {
        phalcom_ast::ast::ProductLabel::Static {
            symbol: phalcom_ast::ast::SymbolLiteralKind::Name(name),
            ..
        } => Some(name.clone().into_boxed_str()),
        _ => None,
    }
}

fn collect_anonymous_product_expr(expr: &phalcom_ast::ast::Expr, out: &mut Vec<AnonymousProductSource>) {
    use phalcom_ast::ast::{Expr, ListLiteralElement, MapLiteralEntry, RecordLiteralEntry, SetLiteralEntry, TupleLiteralEntry};
    match expr {
        Expr::TupleLiteral(tuple) => {
            let mut positional_len = 0u32;
            let mut labels = Vec::new();
            let mut component_ranges = Vec::new();
            let mut eligible = true;
            for entry in &tuple.entries {
                match entry {
                    TupleLiteralEntry::Positional { expr, range } => {
                        positional_len = positional_len.saturating_add(1);
                        let _ = range;
                        component_ranges.push(expr.range());
                        collect_anonymous_product_expr(expr, out);
                    }
                    TupleLiteralEntry::Labeled { label, value, range } => {
                        let Some(label) = static_product_label(label) else {
                            eligible = false;
                            continue;
                        };
                        labels.push(label);
                        let _ = range;
                        component_ranges.push(value.range());
                        collect_anonymous_product_expr(value, out);
                    }
                    TupleLiteralEntry::Expand { expr, .. } => {
                        eligible = false;
                        collect_anonymous_product_expr(expr, out);
                    }
                }
            }
            if eligible {
                out.push(AnonymousProductSource::Tuple {
                    range: tuple.range,
                    positional_len,
                    labels,
                    component_ranges,
                });
            }
        }
        Expr::RecordLiteral(record) => {
            let mut labels = Vec::new();
            let mut component_ranges = Vec::new();
            let mut eligible = true;
            for entry in &record.entries {
                match entry {
                    RecordLiteralEntry::Field(field) => {
                        let Some(label) = static_product_label(&field.label) else {
                            eligible = false;
                            continue;
                        };
                        labels.push(label);
                        component_ranges.push(field.value.range());
                        collect_anonymous_product_expr(&field.value, out);
                    }
                    RecordLiteralEntry::Expansion { expr, .. } => {
                        eligible = false;
                        collect_anonymous_product_expr(expr, out);
                    }
                }
            }
            if eligible {
                out.push(AnonymousProductSource::Record {
                    range: record.range,
                    labels,
                    component_ranges,
                });
            }
        }
        Expr::Assignment(e) => {
            collect_anonymous_product_expr(&e.name, out);
            collect_anonymous_product_expr(&e.value, out);
        }
        Expr::Range(e) => {
            if let Some(lower) = &e.lower {
                collect_anonymous_product_expr(lower, out);
            }
            if let Some(upper) = &e.upper {
                collect_anonymous_product_expr(upper, out);
            }
        }
        Expr::Unary(e) => collect_anonymous_product_expr(&e.expr, out),
        Expr::Binary(e) => {
            collect_anonymous_product_expr(&e.left, out);
            collect_anonymous_product_expr(&e.right, out);
        }
        Expr::ComparisonChain(e) => e.operands.iter().for_each(|operand| collect_anonymous_product_expr(operand, out)),
        Expr::IfLet(e) => {
            collect_anonymous_product_expr(&e.value, out);
            collect_anonymous_product_statements(&e.then_body.body, out);
            if let Some(body) = &e.else_body {
                collect_anonymous_product_statements(&body.body, out);
            }
        }
        Expr::WhileLet(e) => {
            collect_anonymous_product_expr(&e.value, out);
            collect_anonymous_product_statements(&e.body, out);
        }
        Expr::UnqualifiedCall(e) => collect_anonymous_product_pack(&e.args, out),
        Expr::MethodCall(e) => {
            collect_anonymous_product_expr(&e.object, out);
            collect_anonymous_product_pack(&e.args, out);
        }
        Expr::GetProperty(e) => collect_anonymous_product_expr(&e.object, out),
        Expr::SetProperty(e) => {
            collect_anonymous_product_expr(&e.object, out);
            collect_anonymous_product_expr(&e.value, out);
        }
        Expr::Index(e) => {
            collect_anonymous_product_expr(&e.object, out);
            collect_anonymous_product_pack(&e.args, out);
        }
        Expr::SetIndex(e) => {
            collect_anonymous_product_expr(&e.object, out);
            collect_anonymous_product_pack(&e.args, out);
            collect_anonymous_product_expr(&e.value, out);
        }
        Expr::Block(e) => collect_anonymous_product_statements(&e.body, out),
        Expr::AssociatedLookup(e) => collect_anonymous_product_expr(&e.receiver, out),
        Expr::AssociatedInvoke(e) => {
            collect_anonymous_product_expr(&e.receiver, out);
            collect_anonymous_product_pack(&e.args, out);
        }
        Expr::CallableReference(e) => match &e.target {
            phalcom_ast::ast::CallableReferenceTarget::Bound { receiver, .. } | phalcom_ast::ast::CallableReferenceTarget::Associated { receiver, .. } => {
                collect_anonymous_product_expr(receiver, out)
            }
        },
        Expr::RecordConstruction(e) => e.entries.iter().for_each(|entry| collect_anonymous_product_expr(&entry.value, out)),
        Expr::MapLiteral(e) => e.entries.iter().for_each(|entry| match entry {
            MapLiteralEntry::Association { key, value, .. } => {
                if let phalcom_ast::ast::MapLiteralKey::Computed { expr, .. } = key {
                    collect_anonymous_product_expr(expr, out);
                }
                collect_anonymous_product_expr(value, out);
            }
            MapLiteralEntry::Expansion { expr, .. } => collect_anonymous_product_expr(expr, out),
        }),
        Expr::SetLiteral(e) => e.entries.iter().for_each(|entry| match entry {
            SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => collect_anonymous_product_expr(expr, out),
        }),
        Expr::ListLiteral(e) => e.elements.iter().for_each(|entry| match entry {
            ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => collect_anonymous_product_expr(expr, out),
        }),
        Expr::Match(e) => {
            collect_anonymous_product_expr(&e.value, out);
            e.arms.iter().for_each(|arm| collect_anonymous_product_expr(&arm.branch, out));
        }
        Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::String { .. }
        | Expr::Boolean { .. }
        | Expr::Var { .. }
        | Expr::Field { .. }
        | Expr::SelfVar { .. }
        | Expr::SuperVar { .. }
        | Expr::Ellipsis { .. }
        | Expr::ImplementationSelector { .. }
        | Expr::Symbol(_)
        | Expr::TypeForm(_)
        | Expr::Membership(_)
        | Expr::IsMembership(_) => {}
    }
}

fn collect_anonymous_product_pack(items: &[phalcom_ast::ast::PackItem], out: &mut Vec<AnonymousProductSource>) {
    for item in items {
        match item {
            phalcom_ast::ast::PackItem::Positional { expr, .. }
            | phalcom_ast::ast::PackItem::Expand { expr, .. }
            | phalcom_ast::ast::PackItem::Labeled { value: expr, .. } => collect_anonymous_product_expr(expr, out),
        }
    }
}

fn collect_anonymous_product_statements(statements: &[phalcom_ast::ast::Statement], out: &mut Vec<AnonymousProductSource>) {
    use phalcom_ast::ast::{ClassMember, MemberBody, Statement};
    for statement in statements {
        match statement {
            Statement::Let(binding) => {
                if let Some(value) = &binding.value {
                    collect_anonymous_product_expr(value, out);
                }
            }
            Statement::Return(return_statement) => {
                if let Some(value) = &return_statement.value {
                    collect_anonymous_product_expr(value, out);
                }
            }
            Statement::Expr { expr, .. } | Statement::Throw { expr, .. } => collect_anonymous_product_expr(expr, out),
            Statement::For(for_statement) => {
                for lane in &for_statement.lanes {
                    collect_anonymous_product_expr(&lane.iter, out);
                }
                collect_anonymous_product_statements(&for_statement.body, out);
            }
            Statement::Class(class) => {
                for invariant in &class.invariants {
                    collect_anonymous_product_expr(&invariant.0, out);
                }
                for member in &class.members {
                    match member {
                        ClassMember::Method(method) => {
                            if let MemberBody::Block(body) = &method.body {
                                collect_anonymous_product_statements(body, out);
                            }
                        }
                        ClassMember::Getter(getter) => {
                            if let MemberBody::Block(body) = &getter.body {
                                collect_anonymous_product_statements(body, out);
                            }
                        }
                        ClassMember::Setter(setter) => {
                            if let MemberBody::Block(body) = &setter.body {
                                collect_anonymous_product_statements(body, out);
                            }
                        }
                        ClassMember::Index(index) => collect_anonymous_product_statements(&index.body, out),
                        ClassMember::Field(field) => {
                            if let Some(default) = &field.default {
                                collect_anonymous_product_expr(default, out);
                            }
                        }
                        ClassMember::Variant(variant) => {
                            for attribute in &variant.attributes {
                                for argument in &attribute.args {
                                    collect_anonymous_product_expr(argument, out);
                                }
                            }
                        }
                    }
                }
            }
            Statement::Impl(impl_def) => {
                for member in &impl_def.members {
                    match member {
                        phalcom_ast::ast::BehaviorMember::Method(method) => {
                            if let MemberBody::Block(body) = &method.body {
                                collect_anonymous_product_statements(body, out);
                            }
                        }
                        phalcom_ast::ast::BehaviorMember::Getter(getter) => {
                            if let MemberBody::Block(body) = &getter.body {
                                collect_anonymous_product_statements(body, out);
                            }
                        }
                        phalcom_ast::ast::BehaviorMember::Setter(setter) => {
                            if let MemberBody::Block(body) = &setter.body {
                                collect_anonymous_product_statements(body, out);
                            }
                        }
                        phalcom_ast::ast::BehaviorMember::Index(index) => collect_anonymous_product_statements(&index.body, out),
                    }
                }
            }
            Statement::Enum(_) | Statement::TypeAlias(_) | Statement::Data(_) | Statement::Break { .. } | Statement::Continue { .. } | Statement::Export(_) => {
            }
        }
    }
}

/// Collects only direct module-body product expressions. Callable bodies are
/// analyzed separately and are intentionally left on the conservative dynamic
/// path until their callable-local expression attachment is available here.
#[allow(dead_code)]
fn collect_anonymous_product_module_roots(program: &phalcom_ast::ast::Program) -> Vec<AnonymousProductSource> {
    let mut out = Vec::new();
    for statement in &program.statements {
        let expression = match statement {
            phalcom_ast::ast::Statement::Let(binding) => binding.value.as_ref(),
            phalcom_ast::ast::Statement::Return(return_statement) => return_statement.value.as_ref(),
            phalcom_ast::ast::Statement::Expr { expr, .. } | phalcom_ast::ast::Statement::Throw { expr, .. } => Some(expr),
            _ => None,
        };
        let Some(expression) = expression else { continue };
        match expression {
            phalcom_ast::ast::Expr::TupleLiteral(tuple) if tuple.entries.iter().all(|entry| {
                matches!(entry, phalcom_ast::ast::TupleLiteralEntry::Positional { .. } | phalcom_ast::ast::TupleLiteralEntry::Labeled {
                    label: phalcom_ast::ast::ProductLabel::Static { .. }, ..
                })
            }) => {
                let positional_len = tuple.entries.iter().filter(|entry| matches!(entry, phalcom_ast::ast::TupleLiteralEntry::Positional { .. })).count() as u32;
                let labels = tuple
                    .entries
                    .iter()
                    .filter_map(|entry| match entry {
                        phalcom_ast::ast::TupleLiteralEntry::Labeled { label, .. } => static_product_label(label),
                        _ => None,
                    })
                    .collect();
                let component_ranges = tuple
                    .entries
                    .iter()
                    .map(|entry| match entry {
                        phalcom_ast::ast::TupleLiteralEntry::Positional { expr, .. } => expr.range(),
                        phalcom_ast::ast::TupleLiteralEntry::Labeled { value, .. } => value.range(),
                        phalcom_ast::ast::TupleLiteralEntry::Expand { .. } => unreachable!(),
                    })
                    .collect();
                out.push(AnonymousProductSource::Tuple { range: tuple.range, positional_len, labels, component_ranges });
            }
            phalcom_ast::ast::Expr::RecordLiteral(record) if record.entries.iter().all(|entry| {
                matches!(entry, phalcom_ast::ast::RecordLiteralEntry::Field(field) if matches!(field.label, phalcom_ast::ast::ProductLabel::Static { .. }))
            }) => {
                let mut labels = Vec::new();
                let mut component_ranges = Vec::new();
                for entry in &record.entries {
                    let phalcom_ast::ast::RecordLiteralEntry::Field(field) = entry else { unreachable!() };
                    let Some(label) = static_product_label(&field.label) else { unreachable!() };
                    labels.push(label);
                    component_ranges.push(field.value.range());
                }
                out.push(AnonymousProductSource::Record { range: record.range, labels, component_ranges });
            }
            _ => {}
        }
    }
    out
}

fn project_anonymous_products(
    module: &ModuleId,
    snapshot: &SemanticSnapshot,
    program: &phalcom_ast::ast::Program,
    runtime_type_roots: &BTreeMap<SourceRange, RuntimeTypeRef>,
) -> BTreeMap<SourceRange, Arc<AnonymousProductConstructionLoweringSpec>> {
    let mut sources = Vec::new();
    collect_anonymous_product_statements(&program.statements, &mut sources);
    let mut expression_facts = BTreeMap::<SourceRange, Option<TypeId>>::new();
    for (callable, analysis) in snapshot.callable_analyses.iter().filter(|(callable, _)| callable.owner.module() == module) {
        let _ = callable;
        for expression in analysis.expressions.values() {
            expression_facts
                .entry(expression.range)
                .and_modify(|fact| {
                    *fact = (*fact).or_else(|| expression.knowledge.ty());
                })
                .or_insert_with(|| expression.knowledge.ty());
        }
    }
    let mut projected = BTreeMap::new();

    for source in sources {
        let (range, component_ranges, kind, type_id) = match &source {
            AnonymousProductSource::Tuple { range, component_ranges, .. } => {
                let Some(type_id) = expression_facts.get(range).copied().flatten() else {
                    continue;
                };
                (*range, component_ranges.clone(), 0u8, type_id)
            }
            AnonymousProductSource::Record { range, component_ranges, .. } => {
                let Some(type_id) = expression_facts.get(range).copied().flatten() else {
                    continue;
                };
                (*range, component_ranges.clone(), 1u8, type_id)
            }
        };
        let Some(runtime_type) = runtime_type_roots.get(&range).copied() else {
            continue;
        };
        let type_recipe = if phalcom_semantic::checker::associated::contains_any_type_parameter(&snapshot.store, type_id) {
            RuntimeTypeRecipe::Template(runtime_type)
        } else {
            RuntimeTypeRecipe::Closed(runtime_type)
        };
        let spec = if kind == 0 {
            let AnonymousProductSource::Tuple { positional_len, labels, .. } = source else {
                unreachable!()
            };
            let phalcom_semantic::types::store::TypeData::Tuple(elements) = snapshot.store.get(type_id) else {
                continue;
            };
            if elements.len() != component_ranges.len() || elements.len() != (positional_len as usize + labels.len()) {
                continue;
            }
            AnonymousProductConstructionLoweringSpec {
                kind: AnonymousProductConstructionKind::Tuple {
                    positional_len,
                    labels: labels.into_boxed_slice(),
                },
                layout: crate::product::ProductLayoutSpec::new(
                    elements
                        .iter()
                        .enumerate()
                        .map(|(index, element)| crate::product::ProductComponentSpec {
                            logical_index: index as u32,
                            repr: project_slot_repr_type(element.ty, snapshot),
                        })
                        .collect(),
                ),
                type_recipe,
            }
        } else {
            let AnonymousProductSource::Record { labels, .. } = source else {
                unreachable!()
            };
            let phalcom_semantic::types::store::TypeData::Record(row_id) = snapshot.store.get(type_id) else {
                continue;
            };
            let row = snapshot.store.record_row(*row_id);
            if !matches!(row.tail, phalcom_semantic::types::row::RecordRowTail::Closed) || row.fields.len() != labels.len() {
                continue;
            }
            let mut source_to_logical = Vec::with_capacity(labels.len());
            for label in &labels {
                let Ok(index) = row.fields.binary_search_by(|field| field.name.as_ref().cmp(label.as_ref())) else {
                    source_to_logical.clear();
                    break;
                };
                source_to_logical.push(index as u32);
            }
            if source_to_logical.len() != labels.len() || {
                let mut seen = BTreeSet::new();
                source_to_logical.iter().any(|index| !seen.insert(*index))
            } {
                continue;
            }
            AnonymousProductConstructionLoweringSpec {
                kind: AnonymousProductConstructionKind::Record {
                    presentation_labels: labels.into_boxed_slice(),
                    logical_labels: row.fields.iter().map(|field| field.name.clone()).collect(),
                    source_to_logical: source_to_logical.into_boxed_slice(),
                },
                layout: crate::product::ProductLayoutSpec::new(
                    row.fields
                        .iter()
                        .enumerate()
                        .map(|(index, field)| crate::product::ProductComponentSpec {
                            logical_index: index as u32,
                            repr: project_slot_repr_type(field.ty, snapshot),
                        })
                        .collect(),
                ),
                type_recipe,
            }
        };
        projected.insert(range, Arc::new(spec));
    }
    projected
}

/// Returns the semantic type roots needed by runtime product descriptors.
/// The compiler uses these roots to build metadata-backed runtime references;
/// it does not reconstruct product types from AST payloads.
pub fn anonymous_product_type_roots(
    module: &ModuleId,
    snapshot: &SemanticSnapshot,
    program: &phalcom_ast::ast::Program,
) -> Vec<(SourceRange, TypeId)> {
    let mut sources = Vec::new();
    collect_anonymous_product_statements(&program.statements, &mut sources);
    let mut expression_facts = BTreeMap::<SourceRange, Option<TypeId>>::new();
    for (callable, analysis) in snapshot.callable_analyses.iter().filter(|(callable, _)| callable.owner.module() == module) {
        let _ = callable;
        for expression in analysis.expressions.values() {
            expression_facts
                .entry(expression.range)
                .and_modify(|fact| *fact = (*fact).or_else(|| expression.knowledge.ty()))
                .or_insert_with(|| expression.knowledge.ty());
        }
    }
    sources
        .into_iter()
        .filter_map(|source| {
            let (range, component_count) = match &source {
                AnonymousProductSource::Tuple { range, component_ranges, .. }
                | AnonymousProductSource::Record { range, component_ranges, .. } => (*range, component_ranges.len()),
            };
            let ty = expression_facts.get(&range).copied().flatten()?;
            match snapshot.store.get(ty) {
                TypeData::Tuple(elements) if elements.len() == component_count => Some((range, ty)),
                TypeData::Record(row_id) => {
                    let row = snapshot.store.record_row(*row_id);
                    if matches!(row.tail, phalcom_semantic::types::row::RecordRowTail::Closed) && row.fields.len() == component_count {
                        Some((range, ty))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .collect()
}

fn project_slot_repr_type(ty: TypeId, snapshot: &SemanticSnapshot) -> crate::product::ProductSlotRepr {
    let core_ids = phalcom_semantic::core_surface::CoreDeclarationIds::default();
    match snapshot.store.get(ty) {
        TypeData::Nominal { declaration } if declaration == &core_ids.float => crate::product::ProductSlotRepr::Float64,
        TypeData::Nominal { declaration } if declaration == &core_ids.bool_ => crate::product::ProductSlotRepr::Bool,
        TypeData::Nominal { declaration } if declaration == &core_ids.symbol => crate::product::ProductSlotRepr::Symbol,
        _ => crate::product::ProductSlotRepr::Value,
    }
}

fn project_match_resolution(
    resolution: &phalcom_semantic::match_semantics::MatchResolution,
    snapshot: &SemanticSnapshot,
) -> Result<MatchLoweringSpec, ProjectionError> {
    if !matches!(resolution.exhaustiveness, phalcom_semantic::match_semantics::ExhaustivenessResult::Proven) {
        return Err(ProjectionError::NonProvenMatch(resolution.expression));
    }

    let mut arms = Vec::with_capacity(resolution.arms.len());
    for arm in resolution.arms.iter() {
        let bindings = arm
            .bindings
            .iter()
            .map(|b| ExecutableBindingSpec {
                binding: b.binding,
                name: b.name.clone(),
                range: b.source,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let pattern = project_pattern_resolution(&arm.pattern, &arm.bindings, snapshot)?;

        arms.push(ExecutableMatchArm {
            arm_index: arm.arm_index,
            pattern,
            bindings,
        });
    }

    Ok(MatchLoweringSpec { arms: arms.into_boxed_slice() })
}

fn project_pattern_resolution(
    pattern: &phalcom_semantic::match_semantics::PatternResolution,
    bindings: &[phalcom_semantic::match_semantics::PatternBindingResolution],
    snapshot: &SemanticSnapshot,
) -> Result<ExecutablePattern, ProjectionError> {
    match pattern {
        phalcom_semantic::match_semantics::PatternResolution::Wildcard => Ok(ExecutablePattern::Wildcard),
        phalcom_semantic::match_semantics::PatternResolution::Binding { binding, name, .. } => {
            let index = bindings
                .iter()
                .position(|b| b.binding == *binding)
                .ok_or(ProjectionError::MissingPatternBinding(*binding))?;
            let binding_index = u32::try_from(index).map_err(|_| ProjectionError::PatternBindingIndexOverflow(index))?;
            Ok(ExecutablePattern::Binding {
                binding_index,
                name: name.clone(),
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::Variant(var_pat) => {
            let mut candidates = Vec::with_capacity(var_pat.candidates.len());
            for candidate in var_pat.candidates.iter() {
                let vinfo = snapshot
                    .enum_semantics
                    .variant_info(&candidate.variant)
                    .ok_or_else(|| ProjectionError::MissingVariantMetadata(candidate.variant.clone()))?;
                let mut field_projections = Vec::with_capacity(candidate.fields.len());
                for field in candidate.fields.iter() {
                    let idx = vinfo
                        .fields
                        .iter()
                        .position(|f| f.id == field.field)
                        .ok_or_else(|| ProjectionError::MissingFieldLayout(Box::new(field.field.clone()), Box::new(candidate.variant.clone())))?;
                    let slot = u16::try_from(idx).map_err(|_| ProjectionError::SlotOverflow(idx))?;

                    let child = project_pattern_resolution(&field.child, bindings, snapshot)?;
                    field_projections.push(ExecutableFieldProjection {
                        field_id: field.field.clone(),
                        slot,
                        child,
                    });
                }

                candidates.push(ExecutableVariantCandidate {
                    variant: candidate.variant.clone(),
                    fields: field_projections.into_boxed_slice(),
                });
            }

            Ok(ExecutablePattern::Variant {
                candidates: candidates.into_boxed_slice(),
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::Or(or_pat) => {
            let mut alts = Vec::with_capacity(or_pat.alternatives.len());
            for alt in or_pat.alternatives.iter() {
                alts.push(project_pattern_resolution(alt, bindings, snapshot)?);
            }
            Ok(ExecutablePattern::Or {
                alternatives: alts.into_boxed_slice(),
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::Tuple(elements) => {
            let mut elems = Vec::with_capacity(elements.len());
            for elem in elements.iter() {
                elems.push(project_pattern_resolution(elem, bindings, snapshot)?);
            }
            Ok(ExecutablePattern::Tuple {
                elements: elems.into_boxed_slice(),
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::List(list_pat) => {
            let mut elems = Vec::with_capacity(list_pat.prefix.len());
            for elem in list_pat.prefix.iter() {
                elems.push(project_pattern_resolution(elem, bindings, snapshot)?);
            }
            let rest = if let Some(r) = &list_pat.rest {
                Some(Box::new(project_pattern_resolution(r, bindings, snapshot)?))
            } else {
                None
            };
            Ok(ExecutablePattern::List {
                elements: elems.into_boxed_slice(),
                rest,
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::Record(fields) => {
            let mut entries = Vec::with_capacity(fields.len());
            for f in fields.iter() {
                let child = project_pattern_resolution(&f.child, bindings, snapshot)?;
                entries.push((f.label.clone(), child));
            }
            Ok(ExecutablePattern::Record {
                entries: entries.into_boxed_slice(),
            })
        }
        phalcom_semantic::match_semantics::PatternResolution::Map(entries_pat) => {
            let mut entries = Vec::with_capacity(entries_pat.len());
            for e in entries_pat.iter() {
                let child = project_pattern_resolution(&e.child, bindings, snapshot)?;
                entries.push((e.key.clone(), child));
            }
            Ok(ExecutablePattern::Map {
                entries: entries.into_boxed_slice(),
            })
        }
    }
}

fn project_associated_resolution(
    resolution: &AssociatedResolution,
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
) -> Result<(LoweringSiteKind, AssociatedLoweringSpec), ProjectionError> {
    match &resolution.kind {
        AssociatedResolutionKind::ExactValue { member, value_type } => {
            let spec = match member {
                AssociatedMemberId::Variant(v) => AssociatedLoweringSpec::SingletonLoad { variant: v.clone() },
                AssociatedMemberId::DataComponent(dc) => AssociatedLoweringSpec::GetDataComponent {
                    component: dc.clone(),
                    logical_index: dc.index,
                },
                AssociatedMemberId::DataConstructor(dc) => {
                    let dinfo = snapshot
                        .data_semantics
                        .data_info(&dc.owner)
                        .ok_or_else(|| ProjectionError::MissingDataMetadata(dc.owner.clone()))?;
                    let arity = u8::try_from(dinfo.components.len()).map_err(|_| ProjectionError::ArityOverflow(dinfo.components.len()))?;
                    let construction = build_data_construction_spec(snapshot, projects, dc, *value_type, arity, None)?;
                    AssociatedLoweringSpec::ConstructData {
                        constructor: dc.clone(),
                        arity,
                        construction,
                    }
                }
            };
            Ok((LoweringSiteKind::AssociatedLookup, spec))
        }
        AssociatedResolutionKind::ExactCallable { target, callable_type, .. } => {
            let spec = match target {
                InvocationTargetId::Behavioral(c) => AssociatedLoweringSpec::MakeResolvedBoundMethod {
                    target: ExecutableInvocationTarget::Behavioral {
                        lookup_owner: resolution.lookup_owner.clone(),
                        callable: c.clone(),
                        operation: behavioral_operation(c),
                        rest_mode: executable_rest_mode(snapshot, c),
                    },
                },
                InvocationTargetId::VariantConstructor(vc) => AssociatedLoweringSpec::MakeVariantConstructorThunk {
                    variant: vc.variant.clone(),
                    operation: variant_constructor_operation(snapshot, &vc.variant)?,
                },
                InvocationTargetId::DataConstructor(dc) => {
                    let target = project_data_constructor_target(snapshot, projects, dc, *callable_type, None)?;
                    let construction = match target {
                        ExecutableInvocationTarget::DataConstructor { construction, .. } => construction,
                        _ => unreachable!(),
                    };
                    AssociatedLoweringSpec::MakeDataConstructorThunk {
                        constructor: dc.clone(),
                        operation: data_constructor_operation(snapshot, dc)?,
                        construction,
                    }
                }
            };
            Ok((LoweringSiteKind::AssociatedLookup, spec))
        }
        AssociatedResolutionKind::Family { members, .. } => {
            let mut entries = Vec::new();
            for member in members.iter() {
                let (target, member_kind) = match (&member.member, &member.target) {
                    (AssociatedMemberId::Variant(variant), None) => {
                        (ExecutableFamilyTarget::Singleton { variant: variant.clone() }, FamilyMemberTypeKind::Value)
                    }
                    (AssociatedMemberId::DataConstructor(dc), _) | (_, Some(InvocationTargetId::DataConstructor(dc))) => {
                        let target = project_data_constructor_target(snapshot, projects, dc, member.value_type, None)?;
                        let construction = match target {
                            ExecutableInvocationTarget::DataConstructor { construction, .. } => construction,
                            _ => unreachable!(),
                        };
                        (
                            ExecutableFamilyTarget::DataConstructor {
                                constructor: dc.clone(),
                                construction,
                            },
                            FamilyMemberTypeKind::Callable,
                        )
                    }
                    (AssociatedMemberId::DataComponent(_), None) => {
                        continue;
                    }
                    (_, Some(InvocationTargetId::VariantConstructor(vc))) => (
                        ExecutableFamilyTarget::VariantConstructor { variant: vc.variant.clone() },
                        FamilyMemberTypeKind::Callable,
                    ),
                    (_, Some(InvocationTargetId::Behavioral(c))) => (
                        ExecutableFamilyTarget::Behavioral {
                            target: ExecutableInvocationTarget::Behavioral {
                                lookup_owner: resolution.lookup_owner.clone(),
                                callable: c.clone(),
                                operation: member.operation.clone(),
                                rest_mode: executable_rest_mode(snapshot, c),
                            },
                        },
                        FamilyMemberTypeKind::Callable,
                    ),
                };
                entries.push(ExecutableFamilyEntry {
                    operation: member.operation.clone(),
                    member_kind,
                    target,
                });
            }
            let desc = ExecutableFamilyDescriptor {
                entries: entries.into_boxed_slice(),
            };
            Ok((
                LoweringSiteKind::AssociatedLookup,
                AssociatedLoweringSpec::MakeAssociatedFamily { descriptor: Arc::new(desc) },
            ))
        }
        AssociatedResolutionKind::StaticInvoke {
            target,
            result_type,
            argument_mapping,
            ..
        } => {
            let spec = match target {
                InvocationTargetId::VariantConstructor(vc) => {
                    let vinfo = snapshot
                        .enum_semantics
                        .variant_info(&vc.variant)
                        .ok_or_else(|| ProjectionError::MissingVariantMetadata(vc.variant.clone()))?;
                    let arity = u8::try_from(vinfo.fields.len()).map_err(|_| ProjectionError::ArityOverflow(vinfo.fields.len()))?;
                    AssociatedLoweringSpec::ConstructVariant {
                        variant: vc.variant.clone(),
                        arity,
                    }
                }
                InvocationTargetId::DataConstructor(dc) => {
                    let dinfo = snapshot
                        .data_semantics
                        .data_info(&dc.owner)
                        .ok_or_else(|| ProjectionError::MissingDataMetadata(dc.owner.clone()))?;
                    let arity = u8::try_from(dinfo.components.len()).map_err(|_| ProjectionError::ArityOverflow(dinfo.components.len()))?;
                    let construction = build_data_construction_spec(snapshot, projects, dc, *result_type, arity, argument_mapping.clone())?;
                    AssociatedLoweringSpec::ConstructData {
                        constructor: dc.clone(),
                        arity,
                        construction,
                    }
                }
                InvocationTargetId::Behavioral(c) => {
                    let arity = u8::try_from(c.selector.slots.len()).map_err(|_| ProjectionError::ArityOverflow(c.selector.slots.len()))?;
                    AssociatedLoweringSpec::InvokeResolvedAssociated {
                        target: ExecutableInvocationTarget::Behavioral {
                            lookup_owner: resolution.lookup_owner.clone(),
                            callable: c.clone(),
                            operation: behavioral_operation(c),
                            rest_mode: executable_rest_mode(snapshot, c),
                        },
                        arity,
                    }
                }
            };
            Ok((LoweringSiteKind::AssociatedInvoke, spec))
        }
        AssociatedResolutionKind::DynamicInvoke { candidates, .. } => {
            let mut exec_candidates = Vec::new();
            for c in candidates.iter() {
                let target = match &c.target {
                    Some(InvocationTargetId::Behavioral(cid)) => Some(ExecutableInvocationTarget::Behavioral {
                        lookup_owner: resolution.lookup_owner.clone(),
                        callable: cid.clone(),
                        operation: c.operation.clone(),
                        rest_mode: executable_rest_mode(snapshot, cid),
                    }),
                    Some(InvocationTargetId::VariantConstructor(vc)) => Some(ExecutableInvocationTarget::VariantConstructor { variant: vc.variant.clone() }),
                    Some(InvocationTargetId::DataConstructor(dc)) => Some(project_data_constructor_target(snapshot, projects, dc, c.value_type, None)?),
                    None => None,
                };
                exec_candidates.push(ExecutableFamilyCandidate {
                    operation: c.operation.clone(),
                    target,
                });
            }
            let spec = AssociatedLoweringSpec::DynamicInvoke {
                candidates: exec_candidates.into_boxed_slice(),
            };
            Ok((LoweringSiteKind::AssociatedInvoke, spec))
        }
    }
}

fn project_callable_reference_resolution(
    resolution: &CallableReferenceResolution,
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
) -> Result<CallableReferenceLoweringSpec, ProjectionError> {
    match &resolution.kind {
        CallableReferenceResolutionKind::BoundFamily { spec, members, .. } => {
            let conditional_members = members
                .iter()
                .filter_map(|member| {
                    let selection = member.conditional.as_ref()?;
                    Some(ExecutableConditionalFamilyEntry {
                        operation: member.operation.clone(),
                        impl_id: selection.impl_id.clone(),
                        callable: selection.callable.clone(),
                        declaring_owner: selection.declaring_owner.clone(),
                        side: selection.side,
                    })
                })
                .collect::<Vec<_>>()
                .into_boxed_slice();
            Ok(CallableReferenceLoweringSpec::MakeBoundFamily {
                spec: spec.clone(),
                conditional_members,
            })
        }
        CallableReferenceResolutionKind::Associated(associated) => {
            let (_, spec) = project_associated_resolution(associated, snapshot, projects)?;
            match spec {
                AssociatedLoweringSpec::MakeResolvedBoundMethod { target } => Ok(CallableReferenceLoweringSpec::MakeResolvedBoundMethod { target }),
                AssociatedLoweringSpec::MakeVariantConstructorThunk { variant, operation } => {
                    Ok(CallableReferenceLoweringSpec::MakeVariantConstructorThunk { variant, operation })
                }
                AssociatedLoweringSpec::MakeDataConstructorThunk {
                    constructor,
                    operation,
                    construction,
                } => Ok(CallableReferenceLoweringSpec::MakeDataConstructorThunk {
                    constructor,
                    operation,
                    construction,
                }),
                AssociatedLoweringSpec::MakeAssociatedFamily { descriptor } => Ok(CallableReferenceLoweringSpec::MakeAssociatedFamily { descriptor }),
                _ => Err(ProjectionError::InvalidCallableReferenceSpec),
            }
        }
    }
}

fn behavioral_operation(callable: &CallableId) -> FamilyOperationShape {
    FamilyOperationShape::new(callable.selector.kind, callable.selector.slots.clone())
}

fn variant_constructor_operation(snapshot: &SemanticSnapshot, variant: &VariantId) -> Result<FamilyOperationShape, ProjectionError> {
    let info = snapshot
        .enum_semantics
        .variant_info(variant)
        .ok_or_else(|| ProjectionError::MissingVariantMetadata(variant.clone()))?;
    let constructor = info
        .constructor
        .as_ref()
        .ok_or_else(|| ProjectionError::MissingConstructorMetadata(variant.clone()))?;
    let slots = constructor
        .parameters
        .iter()
        .map(|parameter| match &parameter.external_label {
            Some(label) => phalcom_common::selector::SelectorSlot::Label(label.to_string()),
            None => phalcom_common::selector::SelectorSlot::Positional,
        })
        .collect::<Vec<_>>();
    Ok(FamilyOperationShape::method(slots.into_boxed_slice()))
}

fn executable_rest_mode(snapshot: &SemanticSnapshot, callable: &CallableId) -> ExecutableRestMode {
    snapshot
        .callable_signatures
        .get(callable)
        .and_then(|signature| signature.parameters.iter().find(|parameter| parameter.rest != phalcom_ast::ast::RestMode::None))
        .map(|parameter| match parameter.rest {
            phalcom_ast::ast::RestMode::None => ExecutableRestMode::None,
            phalcom_ast::ast::RestMode::Positional => ExecutableRestMode::Positional,
            phalcom_ast::ast::RestMode::Labeled => ExecutableRestMode::Labeled,
            phalcom_ast::ast::RestMode::Complete => ExecutableRestMode::Complete,
        })
        .unwrap_or(ExecutableRestMode::None)
}

fn project_family_application(
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    fam_app: &FamilyApplicationResolution,
) -> Result<FamilyApplicationLoweringSpec, ProjectionError> {
    match &fam_app.selection {
        FamilyApplicationSelection::Static {
            operation,
            target,
            callable_type,
            ..
        } => {
            let exec_target = match target {
                Some(target) => Some(executable_invocation_target(snapshot, projects, target, operation, *callable_type)?),
                None => None,
            };
            let arity = u8::try_from(operation.slots.len()).map_err(|_| ProjectionError::ArityOverflow(operation.slots.len()))?;
            Ok(FamilyApplicationLoweringSpec::Static {
                kind: fam_app.kind,
                operation: operation.clone(),
                target: exec_target,
                arity,
            })
        }
        FamilyApplicationSelection::Dynamic { candidates, .. } => {
            let exec_candidates = candidates
                .iter()
                .map(|candidate| {
                    let target = match &candidate.target {
                        Some(target) => Some(executable_invocation_target(
                            snapshot,
                            projects,
                            target,
                            &candidate.operation,
                            candidate.callable_type,
                        )?),
                        None => None,
                    };
                    Ok(ExecutableFamilyCandidate {
                        operation: candidate.operation.clone(),
                        target,
                    })
                })
                .collect::<Result<Vec<_>, ProjectionError>>()?
                .into_boxed_slice();
            Ok(FamilyApplicationLoweringSpec::DynamicPack {
                kind: fam_app.kind,
                candidates: exec_candidates,
            })
        }
    }
}

fn executable_invocation_target(
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    target: &InvocationTargetId,
    operation: &FamilyOperationShape,
    callable_type: TypeId,
) -> Result<ExecutableInvocationTarget, ProjectionError> {
    match target {
        InvocationTargetId::Behavioral(c) => Ok(ExecutableInvocationTarget::Behavioral {
            lookup_owner: c.owner.declaration().clone(),
            callable: c.clone(),
            operation: operation.clone(),
            rest_mode: executable_rest_mode(snapshot, c),
        }),
        InvocationTargetId::VariantConstructor(vc) => Ok(ExecutableInvocationTarget::VariantConstructor { variant: vc.variant.clone() }),
        InvocationTargetId::DataConstructor(dc) => project_data_constructor_target(snapshot, projects, dc, callable_type, None),
    }
}

fn data_constructor_operation(snapshot: &SemanticSnapshot, constructor: &DataConstructorId) -> Result<FamilyOperationShape, ProjectionError> {
    let dinfo = snapshot
        .data_semantics
        .data_info(&constructor.owner)
        .ok_or_else(|| ProjectionError::MissingDataMetadata(constructor.owner.clone()))?;
    let slots = dinfo
        .constructor
        .parameters
        .iter()
        .map(|parameter| match &parameter.external_label {
            Some(label) => phalcom_common::selector::SelectorSlot::Label(label.to_string()),
            None => phalcom_common::selector::SelectorSlot::Positional,
        })
        .collect::<Vec<_>>();
    Ok(FamilyOperationShape::method(slots.into_boxed_slice()))
}

#[cfg(test)]
mod tests {
    use super::contains_exact_case;
    use phalcom_common::selector::Selector;
    use phalcom_modules::identity::ModuleId;
    use phalcom_semantic::identity::{DeclarationId, VariantId};
    use phalcom_semantic::types::{KindId, ScopedTypeData, TypeStore};

    #[test]
    fn exact_case_detection_descends_into_type_lambda_free_types() {
        let mut store = TypeStore::new();
        let owner = DeclarationId::new(ModuleId::universe_root(), "Choice".into());
        let choice = store.nominal_type(owner.clone());
        let variant = VariantId::new(owner, Selector::method("A", []).expect("variant selector"));
        let exact_case = store.exact_case_type(&variant, choice).expect("exact case");
        let body = store.arena_mut().intern_scoped(ScopedTypeData::Free(exact_case));
        let lambda = store.arena_mut().intern_lambda(Box::new([KindId::TYPE]), body, KindId::TYPE, None);
        let lambda = store.type_lambda(lambda);

        assert!(contains_exact_case(&store, lambda));
    }
}
