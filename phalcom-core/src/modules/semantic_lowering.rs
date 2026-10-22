//! Semantic-to-Codegen Lowering Projection (Part 4).
//!
//! Bridges the formal `SemanticSnapshot` products to compact, immutable,
//! backend-facing lowering specifications attached by `LoweringSite` keys.

use crate::typing::{RuntimeCallEnvironmentRecipe, RuntimeTypeRecipe, RuntimeTypeRef};
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
use phalcom_semantic::types::store::{TypeData, TypeStore};
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
    TraitInvoke,
    TraitRequirementInvoke,
    IfLetPattern,
    WhileLetPattern,
    ForLanePattern,
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

/// Semantic executable projection of a type-form expression. The generic
/// arguments remain part of the semantic call specialization; the value of a
/// declaration-backed nominal type form is its ordinary class object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeTypeFormValueSpec {
    ClassObject { declaration: DeclarationId },
    TypeParameter { parameter: phalcom_semantic::types::id::TypeParameterId },
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
    /// Semantic-owned exact environment recipe for the captured callable.
    pub call_environment_recipe: Option<Arc<RuntimeCallEnvironmentRecipe>>,
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
    /// Exact projection of a specialization-only product slot. This has no
    /// declaration component identity and bypasses declaration-slot mapping.
    GetDataProductSlot { logical_index: u32 },
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
    /// Fixed non-generic layouts may be registered at declaration time.
    /// Shape-generic declarations defer every concrete layout to construction.
    pub layout: Option<crate::product::ProductLayoutSpec>,
    pub components: Box<[DataComponentLoweringSpec]>,
}

/// Canonical source declaration bound to an existing runtime structural owner.
/// The compiler uses this association to reuse the preinstalled Universe class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntrinsicRuntimeFacadeLoweringSpec {
    pub declaration: DeclarationId,
    pub owner: phalcom_semantic::declarations::IntrinsicRuntimeOwner,
}

/// Data construction lowering specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataConstructionLoweringSpec {
    pub constructor: DataConstructorId,
    /// Source-compilation metadata used to establish the exact runtime root.
    /// Validated executable artifacts relocate the canonical `type_recipe`
    /// directly and therefore carry no duplicate metadata bundle here.
    pub exact_type: Option<Arc<phalcom_type_meta::SemanticMetadataBundle>>,
    /// Closed constructions can cache their descriptor. Generic source
    /// methods carry a template which is instantiated against the active
    /// runtime type environment at each construction site.
    pub type_recipe: RuntimeTypeRecipe,
    pub layout: crate::product::ProductLayoutSpec,
    pub argument_to_component: Box<[u32]>,
    /// Stable fixed-component ordinal to exact logical slot. Expanded fields
    /// have no stable declaration ordinal and are omitted from this map.
    pub component_to_logical: Box<[u32]>,
}

/// Lowering specification for an accepted member defined in an inherent `impl` block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplMemberLowering {
    pub callable: CallableId,
    pub source_member_index: usize,
    /// Canonical target field for a generated direct-field accessor. This is
    /// present only for accepted `via` members; the compiler uses it as the
    /// authorization/projection fact rather than rebuilding it from source.
    pub delegation_target_field: Option<Box<str>>,
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
        logical_keys: Box<[phalcom_semantic::field_key::FieldKey]>,
        source_to_logical: Box<[u32]>,
    },
    Pack {
        positional_len: u32,
        labels: Box<[Box<str>]>,
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
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallableReferenceLoweringSpec {
    /// Ordinary receiver-bound family capture. Runtime retains the captured
    /// receiver and performs live behavioral dispatch on future invocation.
    MakeBoundFamily {
        spec: BehavioralFamilySpec,
        conditional_members: Box<[ExecutableConditionalFamilyEntry]>,
    },
    /// Exact receiver-bound trait selection. The runtime captures the
    /// selected method and conformance environment at reference creation.
    MakeTraitBoundMethod { invocation: TraitInvocationSpec },
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
    Char(char),
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

/// Frozen semantic projection for an executable pattern site outside `match`
/// (`if let`, `while let`, `for` lane).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternSiteLoweringSpec {
    pub pattern: ExecutablePattern,
    pub bindings: Box<[ExecutableBindingSpec]>,
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
    /// Semantic-owned exact environment recipe for this selected call edge.
    pub call_environment_recipe: Option<Arc<RuntimeCallEnvironmentRecipe>>,
}

/// Exact semantic trait selection attached to an ordinary call/reference site.
/// The compiler projects this product; it never performs trait discovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraitInvocationSpec {
    pub selection: phalcom_semantic::trait_dispatch::TraitDispatchSelection,
    pub requirement_slot: u16,
    pub requirement_slots: Box<[(phalcom_semantic::traits::TraitRequirementId, u16)]>,
    pub antecedent_slots: Box<[(u16, phalcom_semantic::impls::RequirementSelectionTemplate, phalcom_common::selector::Selector)]>,
    /// Runtime bindings owned by the exact selected conformance/witness.
    /// These are projected from semantic evidence; the VM never reconstructs
    /// them from the receiver or from the selected method's class.
    pub type_bindings: Box<[(phalcom_type_meta::StableTypeParameterRef, RuntimeTypeRecipe)]>,
    /// Method-local type arguments and checked conformance obligations for this
    /// call edge. Kept separate from the selected impl's owner environment.
    pub call_environment_recipe: Option<Arc<RuntimeCallEnvironmentRecipe>>,
}

/// Compiler-facing projection of one checked contextual algebraic
/// realization. The selected class-side capabilities remain ordinary
/// semantic trait invocations; this record only supplies their canonical
/// receivers and the source-owned basis constructor for suffix literals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextualRealizationLoweringSpec {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub target_receiver: ContextualClassReceiver,
    pub realization: ForIterationRequirementSpec,
    pub operation: ContextualRealizationOperationLowering,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualRealizationOperationLowering {
    Basis {
        basis_construction: Arc<DataConstructionLoweringSpec>,
        coefficient: Box<phalcom_semantic::checker::analysis::RealizationCoefficientSource>,
        evaluation: phalcom_semantic::checker::analysis::RealizationEvaluationPlan,
        coefficient_receiver: Option<ContextualClassReceiver>,
        coefficient_one: Option<Box<ForIterationRequirementSpec>>,
    },
    RealEmbedding {
        value: ExpressionId,
        scalar_type: TypeId,
    },
}

/// The class-side receiver for a contextual coefficient capability. Generic
/// receivers are projected from the active runtime type environment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualClassReceiver {
    Declaration(DeclarationId),
    TypeParameter(phalcom_type_meta::StableTypeParameterRef),
}

pub type RuntimeTraitEnvironmentKey = (SourceRange, ImplId, phalcom_semantic::traits::TraitRequirementId);
pub type RuntimeTraitEnvironmentBindings = BTreeMap<RuntimeTraitEnvironmentKey, Box<[(phalcom_type_meta::StableTypeParameterRef, RuntimeTypeRecipe)]>>;

/// Abstract requirement call inside a trait default. The active runtime
/// conformance environment supplies the selected target for this slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraitRequirementInvocationSpec {
    pub requirement: phalcom_semantic::traits::TraitRequirementId,
    pub requirement_slot: u16,
}

/// One executable requirement used by the canonical `for` lowering.
/// Exact/derived evidence is lowered through the selected-trait path; an
/// assumed generic obligation is lowered through the active conformance slot.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForIterationRequirementSpec {
    Selected(TraitInvocationSpec),
    Requirement(TraitRequirementInvocationSpec),
}

/// Frozen semantic projection for one canonical iteration source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForIterationLoweringSpec {
    pub source: SourceRange,
    pub item_type: TypeId,
    pub iterator_type: TypeId,
    pub iterator: ForIterationRequirementSpec,
    pub next: ForIterationRequirementSpec,
}

/// Frozen semantic projection for one canonical positional expansion source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PositionalExpansionLoweringSpec {
    pub source: SourceRange,
    pub item_type: TypeId,
    pub iterator_type: TypeId,
    pub iterator: ForIterationRequirementSpec,
    pub next: ForIterationRequirementSpec,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DetachedMethodSource {
    ConformanceWitness {
        impl_id: ImplId,
        source_member_index: usize,
    },
    TraitDefault {
        trait_declaration: DeclarationId,
        source_member_index: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetachedMethodSpec {
    pub callable: CallableId,
    pub source: DetachedMethodSource,
}

/// Materialization specification for one evaluated compile-time static constant.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticConstantLoweringSpec {
    pub id: phalcom_semantic::static_constant::StaticConstantId,
    pub descriptor: phalcom_semantic::materialization::MaterializationDescriptor,
}

/// Complete compiled lowering semantics for a single module.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleLoweringSemantics {
    pub module: ModuleId,
    /// Evaluated compile-time static constants projected for runtime materialization.
    pub static_constants: BTreeMap<String, StaticConstantLoweringSpec>,
    /// Canonical callable signatures used to materialize compiler-only
    /// constructor projections without rebuilding signature authority.
    pub callable_signatures: Arc<phalcom_semantic::signature::CallableSignatureTable>,
    /// Canonical constructor/initializer identities shared by semantic checking and code generation.
    pub constructor_initializations: Arc<phalcom_semantic::signature::ConstructorInitializationTable>,
    /// Semantic direct-parent initializer selections keyed by their exact
    /// explicit `super` call sites.
    pub constructor_super_initializers: BTreeMap<SourceRange, CallableId>,
    /// Canonical local-binding attachments keyed by their current source
    /// ranges. The compiler may use these facts to classify AST uses, but it
    /// never resolves names or scopes independently.
    pub bindings: BTreeMap<SourceRange, BindingId>,
    /// Read safety classifications for binding reads published by static flow analysis.
    pub binding_read_safeties: BTreeMap<SourceRange, phalcom_semantic::checker::flow::BindingReadSafety>,
    pub type_form_values: BTreeMap<SourceRange, RuntimeTypeFormValueSpec>,
    pub anonymous_products: BTreeMap<SourceRange, Arc<AnonymousProductConstructionLoweringSpec>>,
    /// Canonical contextual algebraic realization facts keyed by their AST
    /// source range. The compiler consumes these without repeating semantic
    /// capability discovery.
    pub contextual_realizations: BTreeMap<SourceRange, Arc<ContextualRealizationLoweringSpec>>,
    /// Checked calls whose constructor belongs to the canonical intrinsic Pack.
    pub pack_constructions: BTreeSet<SourceRange>,
    pub call_environment_recipes: BTreeMap<SourceRange, Arc<RuntimeCallEnvironmentRecipe>>,
    /// Conditional selected-call recipes keyed by source edge and exact
    /// callable identity; one bound family source may capture several entries.
    pub conditional_call_environment_recipes: BTreeMap<(SourceRange, phalcom_semantic::identity::CallableId), Arc<RuntimeCallEnvironmentRecipe>>,
    pub enums: Box<[EnumLoweringSpec]>,
    pub data_decls: Box<[DataDeclarationLoweringSpec]>,
    pub intrinsic_runtime_facades: Box<[IntrinsicRuntimeFacadeLoweringSpec]>,
    pub inherent_impls: Box<[InherentImplLoweringSpec]>,
    pub associated: BTreeMap<LoweringSite, AssociatedLoweringSpec>,
    pub callable_references: BTreeMap<LoweringSite, CallableReferenceLoweringSpec>,
    pub family_values: BTreeSet<LoweringSite>,
    pub family_application_sites: BTreeSet<LoweringSite>,
    pub family_applications: BTreeMap<LoweringSite, FamilyApplicationLoweringSpec>,
    pub matches: BTreeMap<LoweringSite, MatchLoweringSpec>,
    pub pattern_sites: BTreeMap<LoweringSite, PatternSiteLoweringSpec>,
    pub conditional_invocations: BTreeMap<LoweringSite, ConditionalInvocationSpec>,
    pub trait_invocations: BTreeMap<LoweringSite, TraitInvocationSpec>,
    pub trait_requirement_invocations: BTreeMap<LoweringSite, TraitRequirementInvocationSpec>,
    pub for_iteration_lanes: BTreeMap<SourceRange, ForIterationLoweringSpec>,
    pub positional_expansions: BTreeMap<SourceRange, PositionalExpansionLoweringSpec>,
    pub known_invalid_positional_expansions: BTreeMap<SourceRange, TypeId>,
    pub sequence_literals: BTreeMap<SourceRange, SequenceLiteralLoweringSpec>,
    pub detached_methods: BTreeMap<CallableId, DetachedMethodSpec>,
}

/// Checked sequence literal lowering specification (COLL006.C1.P1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceLiteralLoweringSpec {
    pub family: phalcom_semantic::checker::SequenceLiteralFamily,
    pub construction: phalcom_semantic::checker::SequenceConstructionPlan,
    pub extent: Option<TypeId>,
    pub exact_expansions: std::collections::BTreeMap<SourceRange, phalcom_semantic::checker::SequenceExpansionResolution>,
    /// Checked host encoding of canonical extent for construction bytecodes.
    pub expected_extent: Option<u32>,
    pub exact_expansion_counts: BTreeMap<SourceRange, u32>,
}

pub(crate) fn attach_runtime_trait_environments(lowering: &mut ModuleLoweringSemantics, environments: &RuntimeTraitEnvironmentBindings) {
    for (site, spec) in lowering.trait_invocations.iter_mut() {
        let key = (site.range, spec.selection.source_impl.clone(), spec.selection.requirement.clone());
        if let Some(bindings) = environments.get(&key) {
            spec.type_bindings = bindings.clone();
        }
    }

    fn attach_requirement(requirement: &mut ForIterationRequirementSpec, range: SourceRange, environments: &RuntimeTraitEnvironmentBindings) {
        let ForIterationRequirementSpec::Selected(spec) = requirement else { return };
        let key = (range, spec.selection.source_impl.clone(), spec.selection.requirement.clone());
        if let Some(bindings) = environments.get(&key) {
            spec.type_bindings = bindings.clone();
        }
    }

    for spec in lowering.for_iteration_lanes.values_mut() {
        let range = spec.source;
        attach_requirement(&mut spec.iterator, range, environments);
        attach_requirement(&mut spec.next, range, environments);
    }

    for spec in lowering.positional_expansions.values_mut() {
        let range = spec.source;
        attach_requirement(&mut spec.iterator, range, environments);
        attach_requirement(&mut spec.next, range, environments);
    }

    for realization in lowering.contextual_realizations.values_mut() {
        let spec = Arc::make_mut(realization);
        attach_requirement(&mut spec.realization, spec.source_range, environments);
        if let ContextualRealizationOperationLowering::Basis {
            coefficient_one: Some(invocation),
            ..
        } = &mut spec.operation
        {
            attach_requirement(invocation, spec.source_range, environments);
        }
    }
}

impl ModuleLoweringSemantics {
    pub fn new(module: ModuleId) -> Self {
        Self {
            module,
            static_constants: BTreeMap::new(),
            callable_signatures: Arc::new(phalcom_semantic::signature::CallableSignatureTable::default()),
            constructor_initializations: Arc::new(phalcom_semantic::signature::ConstructorInitializationTable::default()),
            constructor_super_initializers: BTreeMap::new(),
            bindings: BTreeMap::new(),
            binding_read_safeties: BTreeMap::new(),
            type_form_values: BTreeMap::new(),
            anonymous_products: BTreeMap::new(),
            contextual_realizations: BTreeMap::new(),
            pack_constructions: BTreeSet::new(),
            call_environment_recipes: BTreeMap::new(),
            conditional_call_environment_recipes: BTreeMap::new(),
            enums: Box::new([]),
            data_decls: Box::new([]),
            intrinsic_runtime_facades: Box::new([]),
            inherent_impls: Box::new([]),
            associated: BTreeMap::new(),
            callable_references: BTreeMap::new(),
            family_values: BTreeSet::new(),
            family_application_sites: BTreeSet::new(),
            family_applications: BTreeMap::new(),
            matches: BTreeMap::new(),
            pattern_sites: BTreeMap::new(),
            conditional_invocations: BTreeMap::new(),
            trait_invocations: BTreeMap::new(),
            trait_requirement_invocations: BTreeMap::new(),
            for_iteration_lanes: BTreeMap::new(),
            positional_expansions: BTreeMap::new(),
            known_invalid_positional_expansions: BTreeMap::new(),
            sequence_literals: BTreeMap::new(),
            detached_methods: BTreeMap::new(),
        }
    }
}

/// Errors occurring during semantic-to-lowering projection.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum ProjectionError {
    #[error("generic evidence layout error: {0}")]
    GenericEvidenceLayout(String),
    #[error("contextual realization cannot lower this generic evidence shape: {0}")]
    UnsupportedContextualRealizationEvidence(&'static str),
    #[error("data type metadata projection failed: {0}")]
    DataTypeMetadata(String),
    #[error("ambiguous lowering site attachment at {0:?}")]
    AmbiguousLoweringSiteAttachment(LoweringSite),
    #[error("ambiguous constructor super-initializer attachment at {0:?}")]
    AmbiguousConstructorSuperInitializerAttachment(SourceRange),
    #[error("ambiguous contextual algebraic realization at {0:?}")]
    AmbiguousContextualRealization(SourceRange),
    #[error("contextual algebraic realization {0:?} requires runtime reification for an assumed class-side receiver")]
    AssumedContextualRealizationReceiver(ExpressionId),
    #[error("contextual algebraic realization {0:?} has no canonical type-parameter receiver")]
    MissingContextualRealizationTypeParameter(ExpressionId),
    #[error("contextual algebraic realization {0:?} has no canonical nominal class receiver")]
    MissingContextualRealizationClass(ExpressionId),
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
    #[error("data constructor {constructor:?} did not materialize an exact product shape")]
    SymbolicDataProductShape { constructor: DataConstructorId },
    #[error("data constructor {constructor:?} is missing a complete argument mapping")]
    MissingDataArgumentMapping { constructor: DataConstructorId },
    #[error("missing executable slot for trait requirement {0:?}")]
    MissingTraitRequirementSlot(phalcom_semantic::traits::TraitRequirementId),
    #[error("missing conformance metadata for callable owner {0:?}")]
    MissingConformanceMetadata(ImplId),
    #[error("static associated result cannot be lowered as a runtime value")]
    StaticAssociatedResultInRuntimeContext,
}

fn contains_exact_case(store: &phalcom_semantic::types::TypeStore, ty: TypeId) -> bool {
    use phalcom_semantic::types::store::TypeData;
    match store.get(ty) {
        TypeData::ExactCase { .. } => true,
        TypeData::Applied { origin, arguments } => {
            contains_exact_case(store, *origin)
                || arguments.iter().any(|argument| match argument {
                    phalcom_semantic::types::store::StaticArgument::Term(argument) => contains_exact_case(store, *argument),
                    phalcom_semantic::types::store::StaticArgument::RecordRow(row) => contains_exact_case_in_row(store, *row),
                    phalcom_semantic::types::store::StaticArgument::DynamicType => false,
                })
        }
        TypeData::Union(members) => members.iter().any(|&m| contains_exact_case(store, m)),
        TypeData::Tuple(elems) => elems.elements.iter().any(|e| contains_exact_case(store, e.ty)),
        TypeData::Record(row_id) => contains_exact_case_in_row(store, *row_id),
        TypeData::Callable(call) => call.parameters.iter().any(|p| contains_exact_case(store, p.ty)) || contains_exact_case(store, call.return_type),
        TypeData::Lambda(lambda_id) => {
            let mut free_types = Vec::new();
            let lambda = store.arena().get_lambda(*lambda_id);
            store.arena().collect_free_types(lambda.body, &mut free_types);
            free_types.into_iter().any(|free_type| contains_exact_case(store, free_type))
        }
        TypeData::TypeMemberProjection(projection) => {
            let subject_contains_exact_case = match projection.subject {
                phalcom_semantic::types::type_member::ProjectionSubject::Type(subject) => contains_exact_case(store, subject),
                phalcom_semantic::types::type_member::ProjectionSubject::RecordRow(row) => contains_exact_case_in_row(store, row),
            };
            subject_contains_exact_case
                || projection.arguments.iter().any(|argument| match argument {
                    phalcom_semantic::types::type_member::TypeMemberArgument::Type(argument) => contains_exact_case(store, *argument),
                    phalcom_semantic::types::type_member::TypeMemberArgument::RecordRow(row) => contains_exact_case_in_row(store, *row),
                    phalcom_semantic::types::type_member::TypeMemberArgument::Const(_) => false,
                })
        }
        _ => false,
    }
}

fn contains_exact_case_in_row(store: &phalcom_semantic::types::TypeStore, row_id: phalcom_semantic::types::RecordRowId) -> bool {
    let row = store.record_row(row_id);
    row.fields.iter().any(|field| contains_exact_case(store, field.ty))
        || match row.tail {
            phalcom_semantic::types::RecordRowTail::Projection(projection) => contains_exact_case(store, projection),
            phalcom_semantic::types::RecordRowTail::Closed | phalcom_semantic::types::RecordRowTail::Parameter(_) => false,
            phalcom_semantic::types::RecordRowTail::Anonymous(phalcom_semantic::types::row::AnonymousRecordRemainder::Unconstrained { key_type }) => {
                contains_exact_case(store, key_type)
            }
            phalcom_semantic::types::RecordRowTail::Anonymous(phalcom_semantic::types::row::AnonymousRecordRemainder::Typed { key_type, value_type }) => {
                contains_exact_case(store, key_type) || contains_exact_case(store, value_type)
            }
        }
}

fn build_data_construction_spec(
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    constructor: &DataConstructorId,
    result_type: TypeId,
    arity: u8,
    argument_mapping: Option<Box<[u32]>>,
    portable_runtime_type: Option<RuntimeTypeRef>,
) -> Result<Arc<DataConstructionLoweringSpec>, ProjectionError> {
    if contains_exact_case(&snapshot.store, result_type) {
        return Err(ProjectionError::OpenDataConstructionType {
            constructor: constructor.clone(),
            result_type,
        });
    }

    let info = snapshot
        .data_semantics
        .data_info(&constructor.owner)
        .ok_or_else(|| ProjectionError::MissingDataMetadata(constructor.owner.clone()))?;
    let mut shape_store = (*snapshot.store).clone();
    let materialized_shape = phalcom_semantic::data_semantics::materialize_data_product_shape(&mut shape_store, info, result_type)
        .map_err(|error| ProjectionError::DataTypeMetadata(error.to_string()))?;
    if materialized_shape.state != phalcom_semantic::data_semantics::DataProductShapeState::Concrete
        || materialized_shape.components.iter().any(|component| component.ty.is_none())
    {
        return Err(ProjectionError::SymbolicDataProductShape {
            constructor: constructor.clone(),
        });
    }
    let exporter =
        phalcom_semantic::metadata::MetadataExporter::new(&snapshot.store, None, None, None, phalcom_type_meta::header::MetadataProfile::RuntimePublic)
            .with_project_universe(projects)
            .with_algebraic_semantics(&snapshot.data_semantics, &snapshot.enum_semantics)
            .with_associated_static_domains(snapshot.associated_static_domains.as_ref());
    let metadata = exporter
        .build_bundle(&[(&constructor.owner.module, "data", result_type)])
        .map_err(|error| ProjectionError::DataTypeMetadata(error.to_string()))?;
    let argument_to_component = match argument_mapping {
        Some(mapping) => mapping,
        None if !info.shape_template.has_expansions() => (0..u32::from(arity)).collect(),
        None => {
            return Err(ProjectionError::MissingDataArgumentMapping {
                constructor: constructor.clone(),
            });
        }
    };
    if argument_to_component.len() != usize::from(arity) {
        return Err(ProjectionError::MissingDataArgumentMapping {
            constructor: constructor.clone(),
        });
    }
    let mut component_to_logical = vec![u32::MAX; info.components.len()];
    for (logical_index, component) in materialized_shape.components.iter().enumerate() {
        if let Some(component_id) = component.component.as_ref() {
            let component_index = component_id.index as usize;
            let Some(slot) = component_to_logical.get_mut(component_index) else {
                return Err(ProjectionError::SymbolicDataProductShape {
                    constructor: constructor.clone(),
                });
            };
            *slot = u32::try_from(logical_index).map_err(|_| ProjectionError::SlotOverflow(logical_index))?;
        }
    }
    if component_to_logical.contains(&u32::MAX) {
        return Err(ProjectionError::SymbolicDataProductShape {
            constructor: constructor.clone(),
        });
    }
    let runtime_root = metadata
        .runtime_roots
        .first()
        .ok_or_else(|| ProjectionError::DataTypeMetadata("missing runtime data type root".to_string()))?;
    // Pool 0 is reserved here for roots owned by the program's persisted
    // RuntimePublic metadata bundle. A source-only exact bundle is deliberately
    // marked with an impossible placeholder so materialization cannot
    // accidentally reinterpret its node index as a P7 node.
    let runtime_type = portable_runtime_type.unwrap_or(RuntimeTypeRef::Base {
        pool: crate::typing::MetadataPoolId(u32::MAX),
        node: runtime_root.form,
    });
    let type_recipe = if phalcom_semantic::checker::associated::contains_any_type_parameter(&snapshot.store, result_type) {
        RuntimeTypeRecipe::Template(runtime_type)
    } else {
        RuntimeTypeRecipe::Closed(runtime_type)
    };

    Ok(Arc::new(DataConstructionLoweringSpec {
        constructor: constructor.clone(),
        exact_type: Some(Arc::new(metadata)),
        type_recipe,
        layout: crate::product::ProductLayoutSpec::new(
            materialized_shape
                .components
                .iter()
                .enumerate()
                .map(|(logical_index, component)| crate::product::ProductComponentSpec {
                    logical_index: logical_index as u32,
                    repr: project_slot_repr_type(component.ty.expect("shape completeness checked above"), &shape_store),
                })
                .collect(),
        ),
        argument_to_component,
        component_to_logical: component_to_logical.into_boxed_slice(),
    }))
}

fn project_data_constructor_target(
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    constructor: &DataConstructorId,
    callable_type: TypeId,
    argument_mapping: Option<Box<[u32]>>,
    runtime_data_type_roots: &BTreeMap<TypeId, RuntimeTypeRef>,
) -> Result<ExecutableInvocationTarget, ProjectionError> {
    use phalcom_semantic::types::store::TypeData;
    let (result_type, signature_arity) = match snapshot.store.get(callable_type) {
        TypeData::Callable(call) => (call.return_type, call.parameters.len()),
        _ => {
            let info = snapshot
                .data_semantics
                .data_info(&constructor.owner)
                .ok_or_else(|| ProjectionError::MissingDataMetadata(constructor.owner.clone()))?;
            (info.constructor.result_type_template, info.components.len())
        }
    };
    let arity = argument_mapping.as_ref().map_or(signature_arity, |mapping| mapping.len());
    let arity_u8 = u8::try_from(arity).map_err(|_| ProjectionError::ArityOverflow(arity))?;
    let construction = build_data_construction_spec(
        snapshot,
        projects,
        constructor,
        result_type,
        arity_u8,
        argument_mapping,
        runtime_data_type_roots.get(&result_type).copied(),
    )?;
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
    build_module_lowering_semantics_with_runtime_types_and_calls(module, snapshot, projects, &BTreeMap::new(), &BTreeMap::new())
}

pub fn build_module_lowering_semantics_with_runtime_types(
    module: &ModuleId,
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    runtime_type_roots: &BTreeMap<SourceRange, RuntimeTypeRef>,
) -> Result<ModuleLoweringSemantics, ProjectionError> {
    build_module_lowering_semantics_with_runtime_types_and_calls(module, snapshot, projects, runtime_type_roots, &BTreeMap::new())
}

pub fn build_module_lowering_semantics_with_runtime_types_and_calls(
    module: &ModuleId,
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    runtime_type_roots: &BTreeMap<SourceRange, RuntimeTypeRef>,
    runtime_call_environment_recipes: &BTreeMap<SourceRange, Arc<RuntimeCallEnvironmentRecipe>>,
) -> Result<ModuleLoweringSemantics, ProjectionError> {
    build_module_lowering_semantics_with_runtime_types_calls_and_conditional_calls(
        module,
        snapshot,
        projects,
        runtime_type_roots,
        &BTreeMap::new(),
        runtime_call_environment_recipes,
        &BTreeMap::new(),
    )
}

pub fn build_module_lowering_semantics_with_runtime_types_calls_and_conditional_calls(
    module: &ModuleId,
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    runtime_type_roots: &BTreeMap<SourceRange, RuntimeTypeRef>,
    runtime_data_type_roots: &BTreeMap<TypeId, RuntimeTypeRef>,
    runtime_call_environment_recipes: &BTreeMap<SourceRange, Arc<RuntimeCallEnvironmentRecipe>>,
    conditional_call_environment_recipes: &BTreeMap<(SourceRange, phalcom_semantic::identity::CallableId), Arc<RuntimeCallEnvironmentRecipe>>,
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

    let mut constructor_super_initializers = BTreeMap::new();
    for (callable, analysis) in snapshot.callable_analyses.iter() {
        if *callable.owner.module() != *module {
            continue;
        }
        let Some(resolution) = analysis.constructor_super_initializer.as_ref() else {
            continue;
        };
        if constructor_super_initializers
            .insert(resolution.call_range, resolution.initializer.clone())
            .is_some()
        {
            return Err(ProjectionError::AmbiguousConstructorSuperInitializerAttachment(resolution.call_range));
        }
    }

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

    let mut binding_read_safeties = BTreeMap::new();
    for (callable_id, analysis) in snapshot.callable_analyses.iter() {
        if *callable_id.owner.module() != *module {
            continue;
        }
        for expr in analysis.expressions.values() {
            if let Some(safety) = expr.binding_read_safety {
                binding_read_safeties.insert(expr.range, safety);
            }
        }
    }

    // 1. Project Enums in this module
    let core_ids = phalcom_semantic::core_surface::CoreDeclarationIds::default();
    let mut enums = Vec::new();
    for (owner, enum_info) in snapshot.enum_semantics.enums.iter() {
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
    for (owner, data_info) in snapshot.data_semantics.data_decls.iter() {
        if owner.module != *module {
            continue;
        }
        let mut components = Vec::new();
        let mut component_specs = Vec::new();
        for (idx, comp) in data_info.components.iter().enumerate() {
            let logical_index = comp.id.index;
            components.push(DataComponentLoweringSpec {
                id: comp.id.clone(),
                local_name: comp.local_name.clone(),
                logical_index,
            });
            let repr = project_slot_repr(&comp.declared_type, snapshot);
            component_specs.push(crate::product::ProductComponentSpec { logical_index, repr });
        }
        let layout = (!data_info.shape_template.has_expansions() && data_info.generic_signature.is_none())
            .then(|| crate::product::ProductLayoutSpec::new(component_specs));
        data_decls.push(DataDeclarationLoweringSpec {
            owner: owner.clone(),
            layout,
            components: components.into_boxed_slice(),
        });
    }
    data_decls.sort_by(|a, b| a.owner.cmp(&b.owner));

    let mut intrinsic_runtime_facades = snapshot
        .declarations
        .intrinsic_runtime_declarations()
        .filter(|(declaration, _)| &declaration.module == module)
        .map(|(declaration, owner)| IntrinsicRuntimeFacadeLoweringSpec {
            declaration: declaration.clone(),
            owner,
        })
        .collect::<Vec<_>>();
    intrinsic_runtime_facades.sort_by(|a, b| a.declaration.cmp(&b.declaration));

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
        let is_generated_delegation = snapshot
            .sources
            .get(&impl_id.module)
            .and_then(|source| source.program.statements.get(impl_id.local.0 as usize))
            .and_then(|statement| match statement {
                phalcom_ast::ast::Statement::Impl(impl_def) => impl_def.members.get(definition.source_member_index),
                _ => None,
            })
            .is_some_and(|member| matches!(member, phalcom_ast::ast::ImplMember::Delegation(_)));
        if !snapshot.callable_analyses.contains_key(&definition.callable) && !is_generated_delegation {
            continue;
        }
        let target = match &definition.callable.owner {
            phalcom_semantic::identity::CallableOwnerId::Declaration(decl) => InherentImplLoweringTarget::Declaration(decl.clone()),
            phalcom_semantic::identity::CallableOwnerId::Variant(var) => InherentImplLoweringTarget::ExactEnumCase(var.clone()),
            // Conformance-local witnesses are detached semantic behavior. They
            // are projected by the trait-dispatch lane, never installed as
            // target-owned inherent members.
            phalcom_semantic::identity::CallableOwnerId::Conformance(_) => continue,
        };
        let entry = inherent_impls_by_id.entry(impl_id.clone()).or_insert_with(|| (target.clone(), Vec::new()));
        debug_assert_eq!(entry.0, target, "one inherent impl cannot contribute to multiple targets");
        entry.1.push(InherentImplMemberLowering {
            callable: definition.callable.clone(),
            source_member_index: definition.source_member_index,
            delegation_target_field: snapshot
                .sources
                .get(&impl_id.module)
                .and_then(|source| source.program.statements.get(impl_id.local.0 as usize))
                .and_then(|statement| match statement {
                    phalcom_ast::ast::Statement::Impl(impl_def) => impl_def.members.get(definition.source_member_index),
                    _ => None,
                })
                .and_then(|member| match member {
                    phalcom_ast::ast::ImplMember::Delegation(delegation) => Some(delegation.target_field.clone().into_boxed_str()),
                    _ => None,
                }),
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
                    .is_some_and(|set| set.members.iter().any(|m| m.impl_id == id && m.is_conditionally_applicable)),
                InherentImplLoweringTarget::ExactEnumCase(var) => snapshot
                    .dispatch
                    .get_conditional_members(&phalcom_semantic::impls::InherentImplTarget::ExactEnumCase(var.clone()))
                    .is_some_and(|set| set.members.iter().any(|m| m.impl_id == id && m.is_conditionally_applicable)),
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
    let mut pack_constructions = BTreeSet::new();
    let mut callable_references = BTreeMap::new();
    let mut family_values = BTreeSet::new();
    let mut family_application_sites = BTreeSet::new();
    let mut family_applications = BTreeMap::new();
    let mut matches = BTreeMap::new();
    let mut pattern_sites = BTreeMap::new();
    let mut conditional_invocations = BTreeMap::new();
    let mut trait_invocations = BTreeMap::new();
    let mut trait_requirement_invocations = BTreeMap::new();
    let mut contextual_realizations = BTreeMap::new();
    let mut for_iteration_lanes = BTreeMap::new();
    let mut positional_expansions = BTreeMap::new();
    let mut known_invalid_positional_expansions = BTreeMap::new();
    let mut sequence_literals = BTreeMap::new();
    let mut detached_methods = BTreeMap::new();

    for definition in snapshot
        .callable_definitions
        .values()
        .filter(|definition| definition.callable.module() == module)
    {
        if let phalcom_semantic::impls::CallableDefinitionOrigin::ConformanceWitness(impl_id) = &definition.origin {
            detached_methods.insert(
                definition.callable.clone(),
                DetachedMethodSpec {
                    callable: definition.callable.clone(),
                    source: DetachedMethodSource::ConformanceWitness {
                        impl_id: impl_id.clone(),
                        source_member_index: definition.source_member_index,
                    },
                },
            );
        }
    }
    if let Some(source) = snapshot.sources.get(module) {
        for statement in &source.program.statements {
            let phalcom_ast::ast::Statement::Trait(trait_def) = statement else { continue };
            let trait_declaration = DeclarationId::new(module.clone(), trait_def.name.clone().into());
            let Some(surface) = snapshot.trait_surfaces.get(&trait_declaration) else {
                continue;
            };
            for (source_member_index, trait_member) in trait_def.members.iter().enumerate() {
                let Some(member) = trait_member.behavior() else {
                    continue;
                };
                let member_range = match member {
                    phalcom_ast::ast::BehaviorMember::Method(member) => member.range,
                    phalcom_ast::ast::BehaviorMember::Getter(member) => member.range,
                    phalcom_ast::ast::BehaviorMember::Setter(member) => member.range,
                    phalcom_ast::ast::BehaviorMember::Index(member) => member.range,
                };
                let Some((_, surface_member)) = surface.iter().find(|(_, member)| member.source.range == member_range) else {
                    continue;
                };
                if surface_member.default_present {
                    detached_methods.insert(
                        surface_member.callable.clone(),
                        DetachedMethodSpec {
                            callable: surface_member.callable.clone(),
                            source: DetachedMethodSource::TraitDefault {
                                trait_declaration: trait_declaration.clone(),
                                source_member_index,
                            },
                        },
                    );
                }
            }
        }
    }

    let requirement_slots = trait_default_requirement_slots(snapshot)?;

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

            if let AssociatedResolutionKind::StaticInvoke {
                target: InvocationTargetId::DataConstructor(constructor),
                ..
            } = &resolution.kind
                && snapshot.declarations.intrinsic_runtime_owner(&constructor.owner) == Some(phalcom_semantic::declarations::IntrinsicRuntimeOwner::Pack)
            {
                pack_constructions.insert(range);
                continue;
            }
            let (kind, spec) = project_associated_resolution(resolution, snapshot, projects, runtime_data_type_roots)?;
            let site = LoweringSite::new(source_id.clone(), range, kind);

            if let Some(previous) = associated.get(&site) {
                if previous != &spec {
                    return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
                }
                // Multiple semantic expression ids can refer to the same
                // source span after contextual analysis. Coalesce only when
                // projection produces exactly the same executable meaning.
                continue;
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
            let spec = project_callable_reference_resolution(
                resolution,
                snapshot,
                projects,
                range,
                runtime_data_type_roots,
                runtime_call_environment_recipes,
                conditional_call_environment_recipes,
            )?;
            let site = LoweringSite::new(source_id.clone(), range, LoweringSiteKind::CallableReference);
            if callable_references.contains_key(&site) {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
            }
            callable_references.insert(site, spec);
        }

        let owner_sig = match &callable_id.owner {
            phalcom_semantic::identity::CallableOwnerId::Conformance(impl_id) => snapshot
                .conformance_witness_plans
                .get(impl_id)
                .and_then(|plan| plan.generic_signature.as_ref())
                .or_else(|| {
                    snapshot
                        .derived_conformance_rules
                        .iter()
                        .find(|r| &r.impl_id == impl_id)
                        .map(|r| &r.generic_signature)
                }),
            _ if callable_id.side == phalcom_semantic::identity::DispatchSide::Instance => callable_id.try_declaration_owner().and_then(|owner| {
                snapshot
                    .declarations
                    .generic_signature(owner)
                    .or_else(|| snapshot.trait_headers.get(owner).and_then(|h| h.generic_signature.as_ref()))
            }),
            _ => None,
        };
        let mut generic_evidence = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::empty();
        if let Some(owner_sig) = owner_sig {
            let owner_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                &owner_sig.constraints,
                owner_sig.owner.clone(),
                &snapshot.trait_surfaces,
            )
            .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
            generic_evidence = generic_evidence
                .try_compose(&owner_env, &snapshot.trait_surfaces)
                .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
        }
        if callable_id.side == phalcom_semantic::identity::DispatchSide::Instance
            && let phalcom_semantic::identity::CallableOwnerId::Declaration(declaration) = &callable_id.owner
            && let Some(surface) = snapshot.trait_surfaces.get(declaration)
        {
            let receiver_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_for_trait_default(
                surface,
                snapshot.store.as_ref(),
                &snapshot.trait_surfaces,
            )
            .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
            generic_evidence = generic_evidence
                .try_compose(&receiver_env, &snapshot.trait_surfaces)
                .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
        }
        if let Some(phalcom_semantic::impls::EffectiveCallableDefinition {
            origin: phalcom_semantic::impls::CallableDefinitionOrigin::InherentImpl(source_impl),
            ..
        }) = snapshot.callable_definitions.get(callable_id)
        {
            let impl_signature = snapshot
                .dispatch
                .conditional_members()
                .values()
                .flat_map(|members| members.members.iter())
                .find(|member| &member.impl_id == source_impl)
                .and_then(|member| member.domain.generic_signature.as_ref());
            if let Some(impl_signature) = impl_signature {
                let impl_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                    &impl_signature.constraints,
                    impl_signature.owner.clone(),
                    &snapshot.trait_surfaces,
                )
                .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
                generic_evidence = generic_evidence
                    .try_compose(&impl_env, &snapshot.trait_surfaces)
                    .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
            }
        }
        if let Some(sig) = snapshot.callable_signatures.get(callable_id) {
            if let Some(generics) = &sig.generics {
                let callable_env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                    &generics.constraints,
                    generics.owner.clone(),
                    &snapshot.trait_surfaces,
                )
                .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
                generic_evidence = generic_evidence
                    .try_compose(&callable_env, &snapshot.trait_surfaces)
                    .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
            }
        }

        for lane in analysis.for_iteration_lanes.values() {
            let iterator = project_for_iteration_requirement(&lane.iterator, &generic_evidence, &requirement_slots, snapshot)?;
            let next = project_for_iteration_requirement(&lane.next, &generic_evidence, &requirement_slots, snapshot)?;
            if for_iteration_lanes
                .insert(
                    lane.source_range,
                    ForIterationLoweringSpec {
                        source: lane.source_range,
                        item_type: lane.item_type,
                        iterator_type: lane.iterator_type,
                        iterator,
                        next,
                    },
                )
                .is_some()
            {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(LoweringSite::new(
                    source_id.clone(),
                    lane.source_range,
                    LoweringSiteKind::TraitInvoke,
                )));
            }
        }
        for expansion in analysis.positional_expansions.values() {
            let iterator = project_for_iteration_requirement(&expansion.iterator, &generic_evidence, &requirement_slots, snapshot)?;
            let next = project_for_iteration_requirement(&expansion.next, &generic_evidence, &requirement_slots, snapshot)?;
            if positional_expansions
                .insert(
                    expansion.source_range,
                    PositionalExpansionLoweringSpec {
                        source: expansion.source_range,
                        item_type: expansion.item_type,
                        iterator_type: expansion.iterator_type,
                        iterator,
                        next,
                    },
                )
                .is_some()
            {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(LoweringSite::new(
                    source_id.clone(),
                    expansion.source_range,
                    LoweringSiteKind::TraitInvoke,
                )));
            }
        }
        for expansion in analysis.known_invalid_positional_expansions.values() {
            known_invalid_positional_expansions.insert(expansion.source_range, expansion.source_type);
        }
        for literal in analysis.sequence_literals.values() {
            sequence_literals.insert(
                literal.source_range,
                SequenceLiteralLoweringSpec {
                    family: literal.family,
                    construction: literal.construction,
                    extent: literal.extent,
                    exact_expansions: literal.exact_expansions.iter().map(|(range, expansion)| (*range, expansion.clone())).collect(),
                    expected_extent: literal.extent.and_then(|extent| match snapshot.store.get(extent) {
                        TypeData::ConstInt(value) => u32::try_from(value.to_bigint()).ok(),
                        _ => None,
                    }),
                    exact_expansion_counts: literal
                        .exact_expansions
                        .iter()
                        .filter_map(|(range, expansion)| {
                            let extent = match expansion {
                                phalcom_semantic::checker::SequenceExpansionResolution::TuplePositionals { extent }
                                | phalcom_semantic::checker::SequenceExpansionResolution::VectorStorage { extent } => *extent,
                            };
                            match snapshot.store.get(extent) {
                                TypeData::ConstInt(value) => u32::try_from(value.to_bigint()).ok().map(|count| (*range, count)),
                                _ => None,
                            }
                        })
                        .collect(),
                },
            );
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

            if let Some(realization) = expression.contextual_realization.as_ref() {
                let spec = project_contextual_realization(realization, &generic_evidence, snapshot, projects, runtime_data_type_roots)?;
                if contextual_realizations.insert(realization.source_range, Arc::new(spec)).is_some() {
                    return Err(ProjectionError::AmbiguousContextualRealization(realization.source_range));
                }
            }

            match expression.trait_dispatch.as_ref() {
                Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::Evidenced(selection)) => {
                    let Some(&raw_slot) = requirement_slots.get(&selection.requirement) else {
                        return Err(ProjectionError::MissingTraitRequirementSlot(selection.requirement.clone()));
                    };
                    let call_environment_recipe = runtime_call_environment_recipes.get(&expression.range).cloned();
                    let callee_generic_evidence_len = if let Some(recipe) = &call_environment_recipe {
                        u16::try_from(recipe.conformance_slots.len()).map_err(|_| ProjectionError::SlotOverflow(recipe.conformance_slots.len()))?
                    } else if let phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable } = &selection.selection {
                        let layout = trait_default_body_evidence_layout(callable, snapshot)?;
                        u16::try_from(layout.len()).map_err(|_| ProjectionError::SlotOverflow(layout.len()))?
                    } else {
                        snapshot
                            .trait_headers
                            .get(&selection.requirement.owner)
                            .and_then(|h| h.generic_signature.as_ref())
                            .and_then(|sig| {
                                phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                                    &sig.constraints,
                                    sig.owner.clone(),
                                    &snapshot.trait_surfaces,
                                )
                                .ok()
                            })
                            .map_or(0, |env| env.len() as u16)
                    };
                    let requirement_slot = callee_generic_evidence_len + raw_slot;
                    let req_slots = requirement_slots
                        .iter()
                        .filter(|(requirement, _)| requirement.owner == selection.requirement.owner)
                        .map(|(requirement, slot)| (requirement.clone(), callee_generic_evidence_len + *slot))
                        .collect();
                    // A checked call-entry recipe already supplies the complete
                    // selected callable generic prefix (owner and method bounds).
                    // Project trait-header antecedents only for unspecialized
                    // edges, so they cannot overwrite method-local proof slots.
                    let antecedent_slots = if call_environment_recipe.is_some() {
                        Box::new([])
                    } else {
                        project_antecedent_slots(&selection.evidence, snapshot)?
                    };
                    let site = LoweringSite::new(source_id.clone(), expression.range, LoweringSiteKind::TraitInvoke);
                    trait_invocations.insert(
                        site,
                        TraitInvocationSpec {
                            selection: selection.clone(),
                            requirement_slot,
                            requirement_slots: req_slots,
                            antecedent_slots,
                            type_bindings: Box::new([]),
                            call_environment_recipe,
                        },
                    );
                }
                Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::AbstractRequirement { requirement }) => {
                    let Some(&raw_slot) = requirement_slots.get(requirement) else {
                        return Err(ProjectionError::MissingTraitRequirementSlot(requirement.clone()));
                    };
                    let requirement_slot = (generic_evidence.len() as u16) + raw_slot;
                    let site = LoweringSite::new(source_id.clone(), expression.range, LoweringSiteKind::TraitRequirementInvoke);
                    trait_requirement_invocations.insert(
                        site,
                        TraitRequirementInvocationSpec {
                            requirement: requirement.clone(),
                            requirement_slot,
                        },
                    );
                }
                Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::Assumed(selection)) => {
                    let slot_key =
                        phalcom_semantic::generic_evidence::GenericEvidenceSlotKey::new(selection.conformance.clone(), selection.requirement.clone());
                    let requirement_slot = generic_evidence
                        .slot_for(&slot_key)
                        .ok_or_else(|| ProjectionError::MissingTraitRequirementSlot(selection.requirement.clone()))?;
                    let site = LoweringSite::new(source_id.clone(), expression.range, LoweringSiteKind::TraitRequirementInvoke);
                    trait_requirement_invocations.insert(
                        site,
                        TraitRequirementInvocationSpec {
                            requirement: selection.requirement.clone(),
                            requirement_slot,
                        },
                    );
                }
                None => {}
            }
        }

        // Family applications
        for (expr_id, fam_app) in analysis.family_applications.iter() {
            let expr_analysis = analysis.expressions.get(expr_id);
            let range = match expr_analysis {
                Some(ea) => ea.range,
                None => return Err(ProjectionError::MissingSourceRange(*expr_id)),
            };

            let spec = project_family_application(snapshot, projects, fam_app, runtime_data_type_roots)?;
            let site = LoweringSite::new(source_id.clone(), range, LoweringSiteKind::FamilyApplication);
            family_application_sites.insert(site.clone());

            if let Some(previous) = family_applications.get(&site) {
                if previous != &spec {
                    return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
                }
                // Distinct semantic expression ids can share one source range
                // when the same family call is observed through more than one
                // analysis path. Identical projections have one unambiguous
                // lowering meaning, so coalesce them at the source site.
                continue;
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

        // 3b. Project standalone executable pattern sites (if let, while let, for lane)
        for (site_key, site_res) in analysis.pattern_site_resolutions.iter() {
            let kind = match site_key.kind {
                phalcom_semantic::match_semantics::PatternSiteKind::IfLet => LoweringSiteKind::IfLetPattern,
                phalcom_semantic::match_semantics::PatternSiteKind::WhileLet => LoweringSiteKind::WhileLetPattern,
                phalcom_semantic::match_semantics::PatternSiteKind::ForLane => LoweringSiteKind::ForLanePattern,
            };
            let site = LoweringSite::new(source_id.clone(), site_key.range, kind);
            let spec = project_pattern_site_resolution(site_res, snapshot)?;
            if pattern_sites.contains_key(&site) {
                return Err(ProjectionError::AmbiguousLoweringSiteAttachment(site));
            }
            pattern_sites.insert(site, spec);
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
                        call_environment_recipe: conditional_call_environment_recipes
                            .get(&(expression.range, selection.callable.clone()))
                            .cloned()
                            .or_else(|| runtime_call_environment_recipes.get(&expression.range).cloned()),
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
    let type_form_values = project_type_form_values(module, snapshot);
    let static_constants = project_static_constants(module, snapshot);

    Ok(ModuleLoweringSemantics {
        module: module.clone(),
        static_constants,
        callable_signatures: snapshot.callable_signatures.clone(),
        constructor_initializations: snapshot.constructor_initializations.clone(),
        constructor_super_initializers,
        bindings,
        binding_read_safeties,
        type_form_values,
        anonymous_products,
        contextual_realizations,
        pack_constructions,
        call_environment_recipes: runtime_call_environment_recipes.clone(),
        conditional_call_environment_recipes: conditional_call_environment_recipes.clone(),
        enums: enums.into_boxed_slice(),
        data_decls: data_decls.into_boxed_slice(),
        intrinsic_runtime_facades: intrinsic_runtime_facades.into_boxed_slice(),
        inherent_impls: inherent_impls.into_boxed_slice(),
        associated,
        callable_references,
        family_values,
        family_application_sites,
        family_applications,
        matches,
        pattern_sites,
        conditional_invocations,
        trait_invocations,
        trait_requirement_invocations,
        for_iteration_lanes,
        positional_expansions,
        known_invalid_positional_expansions,
        sequence_literals,
        detached_methods,
    })
}

fn project_static_constants(module: &ModuleId, snapshot: &SemanticSnapshot) -> BTreeMap<String, StaticConstantLoweringSpec> {
    let mut projected = BTreeMap::new();
    let static_constants_table = snapshot.static_constants();
    let store = snapshot.static_values();
    for product in static_constants_table.module_constants(module) {
        if product.is_successful() {
            if let Some(descriptor) = product.materialization_descriptor(store) {
                projected.insert(
                    product.id.name().to_string(),
                    StaticConstantLoweringSpec {
                        id: product.id.clone(),
                        descriptor,
                    },
                );
            }
        }
    }
    projected
}

fn project_type_form_values(module: &ModuleId, snapshot: &SemanticSnapshot) -> BTreeMap<SourceRange, RuntimeTypeFormValueSpec> {
    let mut projected = BTreeMap::new();
    for (callable, analysis) in snapshot.callable_analyses.iter().filter(|(callable, _)| callable.owner.module() == module) {
        let _ = callable;
        for expression in analysis.expressions.values() {
            let Some(SemanticDenotation::TypeForm(form)) = expression.denotation.as_ref() else {
                continue;
            };
            if let Some(declaration) = snapshot.store.nominal_origin_declaration(*form).cloned() {
                projected.insert(expression.range, RuntimeTypeFormValueSpec::ClassObject { declaration });
            } else if let phalcom_semantic::types::store::TypeData::Parameter(parameter) = snapshot.store.get(*form) {
                projected.insert(expression.range, RuntimeTypeFormValueSpec::TypeParameter { parameter: *parameter });
            }
        }
    }
    projected
}

fn project_for_iteration_requirement(
    requirement: &phalcom_semantic::checker::ForIterationRequirementResolution,
    generic_evidence: &phalcom_semantic::generic_evidence::GenericEvidenceEnvironment,
    requirement_slots: &BTreeMap<phalcom_semantic::traits::TraitRequirementId, u16>,
    snapshot: &SemanticSnapshot,
) -> Result<ForIterationRequirementSpec, ProjectionError> {
    match requirement {
        phalcom_semantic::checker::ForIterationRequirementResolution::AbstractRequirement { requirement, .. } => {
            let raw_slot = requirement_slots
                .get(requirement)
                .copied()
                .ok_or_else(|| ProjectionError::MissingTraitRequirementSlot(requirement.clone()))?;
            let requirement_slot = (generic_evidence.len() as u16) + raw_slot;
            Ok(ForIterationRequirementSpec::Requirement(TraitRequirementInvocationSpec {
                requirement: requirement.clone(),
                requirement_slot,
            }))
        }
        phalcom_semantic::checker::ForIterationRequirementResolution::Evidence {
            conformance,
            evidence,
            requirement,
            signature,
        } => match evidence {
            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Assumed(_) => {
                let key = phalcom_semantic::generic_evidence::GenericEvidenceSlotKey::new(conformance.clone(), requirement.clone());
                let requirement_slot = generic_evidence
                    .slot_for(&key)
                    .ok_or_else(|| ProjectionError::MissingTraitRequirementSlot(requirement.clone()))?;
                Ok(ForIterationRequirementSpec::Requirement(TraitRequirementInvocationSpec {
                    requirement: requirement.clone(),
                    requirement_slot,
                }))
            }
            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(exact) => {
                let selection = exact
                    .requirements
                    .get(requirement)
                    .cloned()
                    .ok_or_else(|| ProjectionError::MissingTraitRequirementSlot(requirement.clone()))?;
                let selection = phalcom_semantic::trait_dispatch::TraitDispatchSelection {
                    source_impl: exact.source_impl.clone(),
                    exact_target: exact.exact_target,
                    exact_trait_ref: exact.exact_trait_ref.clone(),
                    requirement: requirement.clone(),
                    callable: Some(signature.callable.clone()),
                    signature: signature.clone(),
                    evidence_fingerprint: exact.fingerprint,
                    evidence: evidence.clone(),
                    selection,
                    requirement_targets: exact.requirements.clone(),
                };
                let raw_slot = requirement_slots
                    .get(requirement)
                    .copied()
                    .ok_or_else(|| ProjectionError::MissingTraitRequirementSlot(requirement.clone()))?;
                let callee_generic_evidence_len = match &selection.selection {
                    phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable } => {
                        let layout = trait_default_body_evidence_layout(callable, snapshot)?;
                        u16::try_from(layout.len()).map_err(|_| ProjectionError::SlotOverflow(layout.len()))?
                    }
                    _ => 0,
                };
                let requirement_slot = callee_generic_evidence_len + raw_slot;
                let req_slots = requirement_slots
                    .iter()
                    .filter(|(id, _)| id.owner == requirement.owner)
                    .map(|(id, slot)| (id.clone(), callee_generic_evidence_len + *slot))
                    .collect();
                let antecedent_slots = project_antecedent_slots(evidence, snapshot)?;
                Ok(ForIterationRequirementSpec::Selected(TraitInvocationSpec {
                    selection,
                    requirement_slot,
                    requirement_slots: req_slots,
                    antecedent_slots,
                    type_bindings: Box::new([]),
                    call_environment_recipe: None,
                }))
            }
            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(derived) => {
                let (source_impl, exact_target, exact_trait_ref, evidence_fingerprint) = match &derived.origin {
                    phalcom_semantic::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => (
                        exact_source.source_impl.clone(),
                        exact_source.exact_target,
                        exact_source.exact_trait_ref.clone(),
                        exact_source.fingerprint,
                    ),
                    phalcom_semantic::generic_evidence::DerivedConformanceOrigin::GuardedRule {
                        rule,
                        exact_subject,
                        exact_trait_ref,
                    } => (rule.impl_id.clone(), *exact_subject, exact_trait_ref.clone(), rule.fingerprint),
                };
                let selection = derived
                    .requirements()
                    .get(requirement)
                    .cloned()
                    .ok_or_else(|| ProjectionError::MissingTraitRequirementSlot(requirement.clone()))?;
                let selection = phalcom_semantic::trait_dispatch::TraitDispatchSelection {
                    source_impl,
                    exact_target,
                    exact_trait_ref,
                    requirement: requirement.clone(),
                    callable: Some(signature.callable.clone()),
                    signature: signature.clone(),
                    evidence_fingerprint,
                    evidence: evidence.clone(),
                    selection,
                    requirement_targets: derived.requirements().clone(),
                };
                let raw_slot = requirement_slots
                    .get(requirement)
                    .copied()
                    .ok_or_else(|| ProjectionError::MissingTraitRequirementSlot(requirement.clone()))?;
                let callee_generic_evidence_len = match &selection.selection {
                    phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable } => {
                        let layout = trait_default_body_evidence_layout(callable, snapshot)?;
                        u16::try_from(layout.len()).map_err(|_| ProjectionError::SlotOverflow(layout.len()))?
                    }
                    _ => 0,
                };
                let requirement_slot = callee_generic_evidence_len + raw_slot;
                let req_slots = requirement_slots
                    .iter()
                    .filter(|(id, _)| id.owner == requirement.owner)
                    .map(|(id, slot)| (id.clone(), callee_generic_evidence_len + *slot))
                    .collect();
                let antecedent_slots = project_antecedent_slots(evidence, snapshot)?;
                Ok(ForIterationRequirementSpec::Selected(TraitInvocationSpec {
                    selection,
                    requirement_slot,
                    requirement_slots: req_slots,
                    antecedent_slots,
                    type_bindings: Box::new([]),
                    call_environment_recipe: None,
                }))
            }
            phalcom_semantic::generic_evidence::GenericConformanceEvidence::CompilerDerived(_) => {
                Err(ProjectionError::MissingTraitRequirementSlot(requirement.clone()))
            }
        },
    }
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
    Pack {
        range: SourceRange,
        positional_len: u32,
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
    use phalcom_ast::ast::{BracketLiteralElement, Expr, MapLiteralEntry, RecordLiteralEntry, SetLiteralEntry, TupleLiteralEntry};
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
        Expr::If(e) => {
            collect_anonymous_product_expr(&e.condition, out);
            collect_anonymous_product_statements(&e.then_body.body, out);
            if let Some(body) = &e.else_body {
                match body {
                    phalcom_ast::ast::IfElseBody::Block(block) => collect_anonymous_product_statements(&block.body, out),
                    phalcom_ast::ast::IfElseBody::ElseIf(expr) => collect_anonymous_product_expr(expr, out),
                }
            }
        }
        Expr::WhileLet(e) => {
            collect_anonymous_product_expr(&e.value, out);
            collect_anonymous_product_statements(&e.body, out);
        }
        Expr::While(e) => {
            collect_anonymous_product_expr(&e.condition, out);
            collect_anonymous_product_statements(&e.body.body, out);
        }
        Expr::UnqualifiedCall(e) => {
            let mut positional_len = 0u32;
            let mut labels = Vec::new();
            let mut component_ranges = Vec::new();
            let mut eligible = true;
            for arg in &e.args {
                match arg {
                    phalcom_ast::ast::PackItem::Positional { expr, .. } => {
                        positional_len = positional_len.saturating_add(1);
                        component_ranges.push(expr.range());
                    }
                    phalcom_ast::ast::PackItem::Labeled { label, value, .. } => {
                        let label_str = match label {
                            phalcom_ast::ast::PackLabel::Static { text, .. } => Some(text.clone().into_boxed_str()),
                            phalcom_ast::ast::PackLabel::Computed { .. } => None,
                        };
                        let Some(label_str) = label_str else {
                            eligible = false;
                            continue;
                        };
                        labels.push(label_str);
                        component_ranges.push(value.range());
                    }
                    phalcom_ast::ast::PackItem::Expand { .. } => {
                        eligible = false;
                        break;
                    }
                }
            }
            if eligible {
                out.push(AnonymousProductSource::Pack {
                    range: e.range,
                    positional_len,
                    labels,
                    component_ranges,
                });
            }
            collect_anonymous_product_pack(&e.args, out);
        }
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
            phalcom_ast::ast::CallableReferenceTarget::PrimaryDataConstructor { receiver, .. } => collect_anonymous_product_expr(receiver, out),
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
        Expr::BracketLiteral(e) => e.elements.iter().for_each(|entry| match entry {
            BracketLiteralElement::Element { expr, .. } | BracketLiteralElement::Expansion { expr, .. } => collect_anonymous_product_expr(expr, out),
        }),
        Expr::Match(e) => {
            collect_anonymous_product_expr(&e.value, out);
            e.arms.iter().for_each(|arm| match &arm.branch {
                phalcom_ast::ast::MatchArmBody::Expression(expr) => collect_anonymous_product_expr(expr, out),
                phalcom_ast::ast::MatchArmBody::Block(block) => collect_anonymous_product_statements(&block.body, out),
            });
        }
        Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::Imaginary { .. }
        | Expr::String { .. }
        | Expr::Char { .. }
        | Expr::Boolean { .. }
        | Expr::Var { .. }
        | Expr::Field { .. }
        | Expr::SelfVar { .. }
        | Expr::SuperVar { .. }
        | Expr::Ellipsis { .. }
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

#[allow(deprecated)]
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
                        ClassMember::Index(index) => {
                            if let MemberBody::Block(body) = &index.body {
                                collect_anonymous_product_statements(body, out);
                            }
                        }
                        ClassMember::Field(field) => {
                            if let Some(default) = &field.default {
                                collect_anonymous_product_expr(default, out);
                            }
                        }
                        ClassMember::Delegation(_) => {}
                        ClassMember::TypeMember(_) => {}
                    }
                }
            }
            Statement::Impl(impl_def) => {
                for impl_member in &impl_def.members {
                    let Some(member) = impl_member.behavior() else {
                        continue;
                    };
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
                        phalcom_ast::ast::BehaviorMember::Index(index) => {
                            if let MemberBody::Block(body) = &index.body {
                                collect_anonymous_product_statements(body, out);
                            }
                        }
                    }
                }
            }
            Statement::TypeAlias(_)
            | Statement::TypeDefinition(_)
            | Statement::Data(_)
            | Statement::Trait(_)
            | Statement::Break { .. }
            | Statement::Continue { .. }
            | Statement::Export(_) => {}
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

fn canonical_pack_construction_ranges(module: &ModuleId, snapshot: &SemanticSnapshot) -> BTreeSet<SourceRange> {
    snapshot
        .callable_analyses
        .iter()
        .filter(|(callable, _)| callable.owner.module() == module)
        .flat_map(|(_, analysis)| {
            analysis.associated_resolutions.iter().filter_map(|(expression, resolution)| {
                let AssociatedResolutionKind::StaticInvoke {
                    target: InvocationTargetId::DataConstructor(constructor),
                    ..
                } = &resolution.kind
                else {
                    return None;
                };
                (snapshot.declarations.intrinsic_runtime_owner(&constructor.owner) == Some(phalcom_semantic::declarations::IntrinsicRuntimeOwner::Pack))
                    .then(|| analysis.expressions.get(expression).map(|expression| expression.range))
                    .flatten()
            })
        })
        .collect()
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
    let pack_sites = canonical_pack_construction_ranges(module, snapshot);
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
            AnonymousProductSource::Pack { range, component_ranges, .. } => {
                if !pack_sites.contains(range) {
                    continue;
                }
                let Some(type_id) = expression_facts.get(range).copied().flatten() else {
                    continue;
                };
                (*range, component_ranges.clone(), 2u8, type_id)
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
            let phalcom_semantic::types::store::TypeData::Tuple(tuple) = snapshot.store.get(type_id) else {
                continue;
            };
            if tuple.elements.len() != component_ranges.len() || tuple.elements.len() != (positional_len as usize + labels.len()) {
                continue;
            }
            AnonymousProductConstructionLoweringSpec {
                kind: AnonymousProductConstructionKind::Tuple {
                    positional_len,
                    labels: labels.into_boxed_slice(),
                },
                layout: crate::product::ProductLayoutSpec::new(
                    tuple
                        .elements
                        .iter()
                        .enumerate()
                        .map(|(index, element)| crate::product::ProductComponentSpec {
                            logical_index: index as u32,
                            repr: project_slot_repr_type(element.ty, &snapshot.store),
                        })
                        .collect(),
                ),
                type_recipe,
            }
        } else if kind == 1 {
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
                let key = phalcom_semantic::field_key::FieldKey::symbol(label.as_ref());
                let Ok(index) = row.fields.binary_search_by(|field| field.key.canonical_cmp(&key)) else {
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
            let logical_keys = row.fields.iter().map(|field| field.key.clone()).collect::<Vec<_>>();
            AnonymousProductConstructionLoweringSpec {
                kind: AnonymousProductConstructionKind::Record {
                    logical_keys: logical_keys.into_boxed_slice(),
                    source_to_logical: source_to_logical.into_boxed_slice(),
                },
                layout: crate::product::ProductLayoutSpec::new(
                    row.fields
                        .iter()
                        .enumerate()
                        .map(|(index, field)| crate::product::ProductComponentSpec {
                            logical_index: index as u32,
                            repr: project_slot_repr_type(field.ty, &snapshot.store),
                        })
                        .collect(),
                ),
                type_recipe,
            }
        } else {
            let AnonymousProductSource::Pack { positional_len, labels, .. } = source else {
                unreachable!()
            };
            let phalcom_semantic::types::store::TypeData::Pack(pack) = snapshot.store.get(type_id) else {
                continue;
            };
            if pack.elements.len() != component_ranges.len() || pack.elements.len() != (positional_len as usize + labels.len()) {
                continue;
            }
            AnonymousProductConstructionLoweringSpec {
                kind: AnonymousProductConstructionKind::Pack {
                    positional_len,
                    labels: labels.into_boxed_slice(),
                },
                layout: crate::product::ProductLayoutSpec::new(
                    pack.elements
                        .iter()
                        .enumerate()
                        .map(|(index, element)| crate::product::ProductComponentSpec {
                            logical_index: index as u32,
                            repr: project_slot_repr_type(element.ty, &snapshot.store),
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
pub fn anonymous_product_type_roots(module: &ModuleId, snapshot: &SemanticSnapshot, program: &phalcom_ast::ast::Program) -> Vec<(SourceRange, TypeId)> {
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
    let pack_sites = canonical_pack_construction_ranges(module, snapshot);
    sources
        .into_iter()
        .filter_map(|source| {
            if let AnonymousProductSource::Pack { range, .. } = &source {
                if !pack_sites.contains(range) {
                    return None;
                }
            }
            let (range, component_count) = match &source {
                AnonymousProductSource::Tuple { range, component_ranges, .. }
                | AnonymousProductSource::Record { range, component_ranges, .. }
                | AnonymousProductSource::Pack { range, component_ranges, .. } => (*range, component_ranges.len()),
            };
            let ty = expression_facts.get(&range).copied().flatten()?;
            match snapshot.store.get(ty) {
                TypeData::Tuple(tuple) if tuple.elements.len() == component_count => Some((range, ty)),
                TypeData::Record(row_id) => {
                    let row = snapshot.store.record_row(*row_id);
                    if matches!(row.tail, phalcom_semantic::types::row::RecordRowTail::Closed) && row.fields.len() == component_count {
                        Some((range, ty))
                    } else {
                        None
                    }
                }
                TypeData::Pack(pack) if pack.elements.len() == component_count => Some((range, ty)),
                _ => None,
            }
        })
        .collect()
}


fn data_constructor_result_type(
    snapshot: &SemanticSnapshot,
    constructor: &DataConstructorId,
    callable_type: TypeId,
) -> Result<TypeId, ProjectionError> {
    match snapshot.store.get(callable_type) {
        TypeData::Callable(call) => Ok(call.return_type),
        _ => snapshot
            .data_semantics
            .data_info(&constructor.owner)
            .map(|info| info.constructor.result_type_template)
            .ok_or_else(|| ProjectionError::MissingDataMetadata(constructor.owner.clone())),
    }
}

fn collect_associated_data_construction_roots(
    resolution: &AssociatedResolution,
    snapshot: &SemanticSnapshot,
    roots: &mut BTreeSet<TypeId>,
) -> Result<(), ProjectionError> {
    match &resolution.kind {
        AssociatedResolutionKind::ExactValue {
            member: AssociatedMemberId::DataConstructor(_),
            value_type,
        } => {
            roots.insert(*value_type);
        }
        AssociatedResolutionKind::ExactCallable {
            target: InvocationTargetId::DataConstructor(constructor),
            callable_type,
            ..
        } => {
            roots.insert(data_constructor_result_type(snapshot, constructor, *callable_type)?);
        }
        AssociatedResolutionKind::Family { members, .. } => {
            for member in members.iter() {
                let constructor = match (&member.member, &member.target) {
                    (AssociatedMemberId::DataConstructor(constructor), _) => Some(constructor),
                    (_, Some(InvocationTargetId::DataConstructor(constructor))) => Some(constructor),
                    _ => None,
                };
                if let Some(constructor) = constructor {
                    roots.insert(data_constructor_result_type(snapshot, constructor, member.value_type)?);
                }
            }
        }
        AssociatedResolutionKind::StaticInvoke {
            target: InvocationTargetId::DataConstructor(_),
            result_type,
            ..
        } => {
            roots.insert(*result_type);
        }
        AssociatedResolutionKind::DynamicInvoke { candidates, .. } => {
            for candidate in candidates.iter() {
                if let Some(InvocationTargetId::DataConstructor(constructor)) = &candidate.target {
                    roots.insert(data_constructor_result_type(snapshot, constructor, candidate.value_type)?);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Returns every exact data-construction result type that executable lowering
/// may materialize in this module. These roots are exported into the program's
/// single RuntimePublic metadata bundle before lowering, so executable
/// Canonical recipes never depend on source-only per-construction metadata.
pub fn data_construction_type_roots(module: &ModuleId, snapshot: &SemanticSnapshot) -> Result<Vec<TypeId>, ProjectionError> {
    let mut roots = BTreeSet::new();
    for (callable, analysis) in snapshot.callable_analyses.iter().filter(|(callable, _)| callable.owner.module() == module) {
        let _ = callable;
        for resolution in analysis.associated_resolutions.values() {
            collect_associated_data_construction_roots(resolution, snapshot, &mut roots)?;
        }
        for resolution in analysis.callable_reference_resolutions.values() {
            if let CallableReferenceResolutionKind::Associated(associated) = &resolution.kind {
                collect_associated_data_construction_roots(associated, snapshot, &mut roots)?;
            }
        }
        for family in analysis.family_applications.values() {
            match &family.selection {
                FamilyApplicationSelection::Static {
                    target: Some(InvocationTargetId::DataConstructor(constructor)),
                    callable_type,
                    ..
                } => {
                    roots.insert(data_constructor_result_type(snapshot, constructor, *callable_type)?);
                }
                FamilyApplicationSelection::Dynamic { candidates, .. } => {
                    for candidate in candidates.iter() {
                        if let Some(InvocationTargetId::DataConstructor(constructor)) = &candidate.target {
                            roots.insert(data_constructor_result_type(snapshot, constructor, candidate.callable_type)?);
                        }
                    }
                }
                _ => {}
            }
        }
        for expression in analysis.expressions.values() {
            if let Some(realization) = expression.contextual_realization.as_ref()
                && let phalcom_semantic::checker::analysis::ContextualRealizationOperation::Basis { basis_type, .. } = &realization.operation
            {
                roots.insert(*basis_type);
            }
        }
    }
    Ok(roots.into_iter().collect())
}

fn project_slot_repr_type(ty: TypeId, store: &TypeStore) -> crate::product::ProductSlotRepr {
    let core_ids = phalcom_semantic::core_surface::CoreDeclarationIds::default();
    match store.get(ty) {
        TypeData::Nominal { declaration } if declaration == &core_ids.float => crate::product::ProductSlotRepr::Float64,
        TypeData::Nominal { declaration } if declaration == &core_ids.bool_ => crate::product::ProductSlotRepr::Bool,
        TypeData::Nominal { declaration } if declaration == &core_ids.symbol => crate::product::ProductSlotRepr::Symbol,
        _ => crate::product::ProductSlotRepr::Value,
    }
}

fn project_match_resolution<R, S, K>(
    resolution: &phalcom_semantic::match_semantics::MatchResolution<R, S, K>,
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

fn project_pattern_site_resolution<R, S, K>(
    site: &phalcom_semantic::match_semantics::PatternSiteResolution<R, S, K>,
    snapshot: &SemanticSnapshot,
) -> Result<PatternSiteLoweringSpec, ProjectionError> {
    let bindings = site
        .bindings
        .iter()
        .map(|b| ExecutableBindingSpec {
            binding: b.binding,
            name: b.name.clone(),
            range: b.source,
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let pattern = project_pattern_resolution(&site.pattern, &site.bindings, snapshot)?;

    Ok(PatternSiteLoweringSpec { pattern, bindings })
}

fn project_pattern_resolution<R, S, K>(
    pattern: &phalcom_semantic::match_semantics::PatternResolution<R, S, K>,
    bindings: &[phalcom_semantic::match_semantics::PatternBindingResolution<R, K>],
    snapshot: &SemanticSnapshot,
) -> Result<ExecutablePattern, ProjectionError> {
    match pattern {
        phalcom_semantic::match_semantics::PatternResolution::Wildcard => Ok(ExecutablePattern::Wildcard),
        phalcom_semantic::match_semantics::PatternResolution::Char(value) => Ok(ExecutablePattern::Char(*value)),
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
    runtime_data_type_roots: &BTreeMap<TypeId, RuntimeTypeRef>,
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
                    let construction = build_data_construction_spec(snapshot, projects, dc, *value_type, arity, None, runtime_data_type_roots.get(value_type).copied())?;
                    AssociatedLoweringSpec::ConstructData {
                        constructor: dc.clone(),
                        arity,
                        construction,
                    }
                }
                AssociatedMemberId::StaticDomain(_) | AssociatedMemberId::TypeMember(_) => {
                    return Err(ProjectionError::StaticAssociatedResultInRuntimeContext);
                }
            };
            Ok((LoweringSiteKind::AssociatedLookup, spec))
        }
        AssociatedResolutionKind::DataExpansionField { logical_index, .. } => Ok((
            LoweringSiteKind::AssociatedLookup,
            AssociatedLoweringSpec::GetDataProductSlot { logical_index: *logical_index },
        )),
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
                    let target = project_data_constructor_target(snapshot, projects, dc, *callable_type, None, runtime_data_type_roots)?;
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
                        let target = project_data_constructor_target(snapshot, projects, dc, member.value_type, None, runtime_data_type_roots)?;
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
                    (AssociatedMemberId::StaticDomain(_) | AssociatedMemberId::TypeMember(_), _) => {
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
                    let arity = u8::try_from(argument_mapping.as_ref().map_or(dinfo.components.len(), |mapping| mapping.len()))
                        .map_err(|_| ProjectionError::ArityOverflow(argument_mapping.as_ref().map_or(dinfo.components.len(), |mapping| mapping.len())))?;
                    let construction = build_data_construction_spec(snapshot, projects, dc, *result_type, arity, argument_mapping.clone(), runtime_data_type_roots.get(result_type).copied())?;
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
                    Some(InvocationTargetId::DataConstructor(dc)) => Some(project_data_constructor_target(snapshot, projects, dc, c.value_type, None, runtime_data_type_roots)?),
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
        AssociatedResolutionKind::StaticDomain { .. } | AssociatedResolutionKind::StaticTerm { .. } => {
            Err(ProjectionError::StaticAssociatedResultInRuntimeContext)
        }
    }
}

fn project_callable_reference_resolution(
    resolution: &CallableReferenceResolution,
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    range: SourceRange,
    runtime_data_type_roots: &BTreeMap<TypeId, RuntimeTypeRef>,
    runtime_call_environment_recipes: &BTreeMap<SourceRange, Arc<RuntimeCallEnvironmentRecipe>>,
    conditional_call_environment_recipes: &BTreeMap<(SourceRange, phalcom_semantic::identity::CallableId), Arc<RuntimeCallEnvironmentRecipe>>,
) -> Result<CallableReferenceLoweringSpec, ProjectionError> {
    match &resolution.kind {
        CallableReferenceResolutionKind::BoundFamily { spec, members, .. } => {
            if matches!(spec, BehavioralFamilySpec::Exact(_))
                && members.len() == 1
                && let Some(selection) = members[0].trait_dispatch.as_ref()
            {
                return Ok(CallableReferenceLoweringSpec::MakeTraitBoundMethod {
                    invocation: project_trait_invocation_spec(selection, snapshot)?,
                });
            }
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
                        call_environment_recipe: conditional_call_environment_recipes
                            .get(&(range, selection.callable.clone()))
                            .cloned()
                            .or_else(|| runtime_call_environment_recipes.get(&range).cloned()),
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
            let (_, spec) = project_associated_resolution(associated, snapshot, projects, runtime_data_type_roots)?;
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

type ProjectedAntecedentSlot = (u16, phalcom_semantic::impls::RequirementSelectionTemplate, phalcom_common::selector::Selector);

fn exact_prerequisite_has_flat_runtime_environment(exact: &phalcom_semantic::impls::ConformanceEvidence) -> bool {
    !exact
        .trait_header_contract
        .iter()
        .any(|constraint| matches!(constraint, phalcom_semantic::types::parameter::GenericConstraint::Conforms { .. }))
        && exact.trait_header_conformance_proofs.as_ref().is_some_and(|proofs| proofs.is_empty())
}

fn project_antecedent_slots(
    evidence: &phalcom_semantic::generic_evidence::GenericConformanceEvidence,
    snapshot: &SemanticSnapshot,
) -> Result<Box<[ProjectedAntecedentSlot]>, ProjectionError> {
    match evidence {
        phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(derived) => {
            let nested_signature = match &derived.origin {
                phalcom_semantic::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => snapshot
                    .conformance_witness_plans
                    .get(&exact_source.source_impl)
                    .and_then(|plan| plan.generic_signature.as_ref()),
                phalcom_semantic::generic_evidence::DerivedConformanceOrigin::GuardedRule { rule, .. } => Some(&rule.generic_signature),
            };
            let Some(nested_sig) = nested_signature else {
                return Ok(Box::new([]));
            };
            let nested_layout = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                &nested_sig.constraints,
                nested_sig.owner.clone(),
                &snapshot.trait_surfaces,
            )
            .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;

            let mut antecedent_slots = Vec::new();
            for (constraint, nested_item) in nested_sig.constraints.iter().zip(derived.nested.iter()) {
                if let (
                    phalcom_semantic::types::parameter::GenericConstraint::Conforms { subject, trait_ref },
                    phalcom_semantic::generic_evidence::NestedGenericEvidence::Conformance(nested_conformance),
                ) = (constraint, nested_item)
                {
                    let conformance_key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(subject.clone(), trait_ref.clone());
                    for (slot_key, target_slot) in nested_layout.layout().slots.iter() {
                        if slot_key.conformance != conformance_key {
                            continue;
                        }
                        let selection = match nested_conformance {
                            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(exact_ev) => {
                                exact_ev.requirements.get(&slot_key.requirement).cloned()
                            }
                            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(derived_ev) => {
                                derived_ev.requirements().get(&slot_key.requirement).cloned()
                            }
                            _ => None,
                        };
                        if let Some(selection) = selection {
                            antecedent_slots.push((*target_slot, selection, slot_key.requirement.selector.clone()));
                        }
                    }
                }
            }
            Ok(antecedent_slots.into_boxed_slice())
        }
        phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(exact) => {
            let trait_sig = snapshot
                .trait_headers
                .get(&exact.exact_trait_ref.declaration)
                .and_then(|h| h.generic_signature.as_ref());
            let Some(trait_sig) = trait_sig else {
                if exact.trait_header_contract.is_empty() {
                    return Ok(Box::new([]));
                }
                return Err(ProjectionError::GenericEvidenceLayout(
                    "proof-complete exact evidence has trait-header obligations but no current trait signature".into(),
                ));
            };
            let Some(prerequisite_proofs) = exact.trait_header_conformance_proofs.as_ref() else {
                return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                    "exact trait-header evidence was not proof-complete",
                ));
            };
            if trait_sig.constraints.len() != exact.trait_header_contract.len() {
                return Err(ProjectionError::GenericEvidenceLayout(
                    "normalized exact trait-header contract does not align with its declared signature".into(),
                ));
            }
            let nested_layout = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
                &trait_sig.constraints,
                trait_sig.owner.clone(),
                &snapshot.trait_surfaces,
            )
            .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;

            let expected_proof_keys = exact
                .trait_header_contract
                .iter()
                .filter_map(|constraint| match constraint {
                    phalcom_semantic::types::parameter::GenericConstraint::Conforms { subject, trait_ref } => Some(
                        phalcom_semantic::generic_evidence::GenericConformanceKey::new(subject.clone(), trait_ref.clone()),
                    ),
                    _ => None,
                })
                .collect::<std::collections::BTreeSet<_>>();
            let published_proof_keys = prerequisite_proofs.keys().cloned().collect::<std::collections::BTreeSet<_>>();
            if expected_proof_keys != published_proof_keys {
                return Err(ProjectionError::GenericEvidenceLayout(
                    "published exact trait-header proofs do not match the normalized obligations".into(),
                ));
            }

            let mut antecedent_slots = Vec::new();
            for (declared, normalized) in trait_sig.constraints.iter().zip(exact.trait_header_contract.iter()) {
                match (declared, normalized) {
                    (
                        phalcom_semantic::types::parameter::GenericConstraint::Conforms { subject, trait_ref },
                        phalcom_semantic::types::parameter::GenericConstraint::Conforms {
                            subject: normalized_subject,
                            trait_ref: normalized_trait_ref,
                        },
                    ) => {
                        let layout_key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(subject.clone(), trait_ref.clone());
                        let proof_key =
                            phalcom_semantic::generic_evidence::GenericConformanceKey::new(normalized_subject.clone(), normalized_trait_ref.clone());
                        let prerequisite = prerequisite_proofs
                            .get(&proof_key)
                            .ok_or_else(|| ProjectionError::GenericEvidenceLayout(format!("missing published exact proof for {proof_key:?}")))?;
                        let phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(prerequisite) = prerequisite else {
                            return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                                "exact trait-header projection requires direct exact prerequisite proofs",
                            ));
                        };
                        if !exact_prerequisite_has_flat_runtime_environment(prerequisite) {
                            return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                                "recursive exact trait-header runtime environments are not supported",
                            ));
                        }
                        for (slot_key, target_slot) in nested_layout.layout().slots.iter().filter(|(slot_key, _)| slot_key.conformance == layout_key) {
                            let selection = prerequisite
                                .requirements
                                .get(&slot_key.requirement)
                                .cloned()
                                .ok_or_else(|| ProjectionError::MissingTraitRequirementSlot(slot_key.requirement.clone()))?;
                            antecedent_slots.push((*target_slot, selection, slot_key.requirement.selector.clone()));
                        }
                    }
                    (
                        phalcom_semantic::types::parameter::GenericConstraint::Equivalent { .. },
                        phalcom_semantic::types::parameter::GenericConstraint::Equivalent { .. },
                    )
                    | (
                        phalcom_semantic::types::parameter::GenericConstraint::Subtype { .. },
                        phalcom_semantic::types::parameter::GenericConstraint::Subtype { .. },
                    ) => {}
                    _ => {
                        return Err(ProjectionError::GenericEvidenceLayout(
                            "normalized exact trait-header obligations changed constraint kind".into(),
                        ));
                    }
                }
            }
            Ok(antecedent_slots.into_boxed_slice())
        }
        _ => Ok(Box::new([])),
    }
}

/// Default bodies and their callers may be compiled in different import
/// closures. Requirement slots must depend only on the owning trait surface,
/// never on unrelated traits present in the surrounding snapshot.
pub(crate) fn trait_default_requirement_slots(
    snapshot: &SemanticSnapshot,
) -> Result<BTreeMap<phalcom_semantic::traits::TraitRequirementId, u16>, ProjectionError> {
    let mut slots = BTreeMap::new();
    for (_, surface) in snapshot.trait_surfaces.iter() {
        for (slot, (requirement, _)) in surface.iter().enumerate() {
            slots.insert(requirement.clone(), u16::try_from(slot).map_err(|_| ProjectionError::SlotOverflow(slot))?);
        }
    }
    Ok(slots)
}

/// Returns the canonical evidence layout used while compiling a trait-default
/// callable body. Abstract requirement slots are offset by this complete
/// layout, including owner bounds, the implicit receiver conformance, and
/// callable-local bounds.
fn trait_default_body_evidence_layout(
    callable: &phalcom_semantic::identity::CallableId,
    snapshot: &SemanticSnapshot,
) -> Result<phalcom_semantic::generic_evidence::GenericEvidenceEnvironment, ProjectionError> {
    use phalcom_semantic::generic_evidence::GenericEvidenceEnvironment;
    use phalcom_semantic::identity::{CallableOwnerId, DispatchSide};

    let CallableOwnerId::Declaration(owner) = &callable.owner else {
        return Err(ProjectionError::GenericEvidenceLayout(format!(
            "trait-default callable {callable:?} has no declaration owner"
        )));
    };
    let mut layout = GenericEvidenceEnvironment::empty();
    if let Some(signature) = snapshot
        .declarations
        .generic_signature(owner)
        .or_else(|| snapshot.trait_headers.get(owner).and_then(|header| header.generic_signature.as_ref()))
    {
        let owner_layout = GenericEvidenceEnvironment::try_from_constraints(&signature.constraints, signature.owner.clone(), &snapshot.trait_surfaces)
            .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
        layout = layout
            .try_compose(&owner_layout, &snapshot.trait_surfaces)
            .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
    }
    if callable.side == DispatchSide::Instance
        && let Some(surface) = snapshot.trait_surfaces.get(owner)
    {
        let receiver_layout = GenericEvidenceEnvironment::try_for_trait_default(surface, snapshot.store.as_ref(), &snapshot.trait_surfaces)
            .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
        layout = layout
            .try_compose(&receiver_layout, &snapshot.trait_surfaces)
            .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
    }
    if let Some(signature) = snapshot.callable_signatures.get(callable)
        && let Some(generics) = &signature.generics
    {
        let callable_layout = GenericEvidenceEnvironment::try_from_constraints(&generics.constraints, generics.owner.clone(), &snapshot.trait_surfaces)
            .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
        layout = layout
            .try_compose(&callable_layout, &snapshot.trait_surfaces)
            .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
    }
    Ok(layout)
}

fn project_trait_invocation_spec(
    selection: &phalcom_semantic::trait_dispatch::TraitDispatchSelection,
    snapshot: &SemanticSnapshot,
) -> Result<TraitInvocationSpec, ProjectionError> {
    let default_requirement_slots = trait_default_requirement_slots(snapshot)?;
    let Some(&raw_slot) = default_requirement_slots.get(&selection.requirement) else {
        return Err(ProjectionError::MissingTraitRequirementSlot(selection.requirement.clone()));
    };
    let callee_generic_evidence_len = match &selection.selection {
        phalcom_semantic::impls::RequirementSelectionTemplate::TraitDefault { callable } => {
            let layout = trait_default_body_evidence_layout(callable, snapshot)?;
            u16::try_from(layout.len()).map_err(|_| ProjectionError::SlotOverflow(layout.len()))?
        }
        _ => 0,
    };
    let requirement_slot = callee_generic_evidence_len + raw_slot;
    let requirement_slots = default_requirement_slots
        .into_iter()
        .filter(|(req, _)| req.owner == selection.requirement.owner)
        .map(|(req, slot)| (req, callee_generic_evidence_len + slot))
        .collect();
    let antecedent_slots = project_antecedent_slots(&selection.evidence, snapshot)?;
    Ok(TraitInvocationSpec {
        selection: selection.clone(),
        requirement_slot,
        requirement_slots,
        antecedent_slots,
        type_bindings: Box::new([]),
        call_environment_recipe: None,
    })
}

fn project_contextual_realization(
    realization: &phalcom_semantic::checker::analysis::ContextualRealization,
    generic_evidence: &phalcom_semantic::generic_evidence::GenericEvidenceEnvironment,
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
    runtime_data_type_roots: &BTreeMap<TypeId, RuntimeTypeRef>,
) -> Result<ContextualRealizationLoweringSpec, ProjectionError> {
    use phalcom_semantic::checker::analysis::{ContextualRealizationOperation, RealizationCapabilitySelection, RealizationCoefficientSource};

    fn project_capability(
        expression: ExpressionId,
        receiver_type: TypeId,
        selection: &RealizationCapabilitySelection,
        generic_evidence: &phalcom_semantic::generic_evidence::GenericEvidenceEnvironment,
        snapshot: &SemanticSnapshot,
        projects: &phalcom_modules::ProjectUniverse,
    ) -> Result<(ContextualClassReceiver, ForIterationRequirementSpec), ProjectionError> {
        match selection {
            RealizationCapabilitySelection::Evidenced(selection) => {
                let target_class = snapshot
                    .store
                    .nominal_origin_declaration(selection.exact_target)
                    .cloned()
                    .ok_or(ProjectionError::MissingContextualRealizationClass(expression))?;
                // Validate the evidence shape before ordinary invocation projection. That
                // projection still has a legacy partial antecedent path, which cannot
                // represent every nested generic proof environment.
                let call_environment_recipe = project_realization_prerequisite_recipe(selection, generic_evidence, snapshot, projects)?;
                let mut invocation = project_trait_invocation_spec(selection, snapshot)?;
                invocation.call_environment_recipe = call_environment_recipe;
                if invocation.call_environment_recipe.is_some() {
                    // The recipe supplies the selected impl's complete evidence
                    // layout, including forwarded assumptions. Do not overwrite
                    // those slots with the legacy concrete antecedent projection.
                    invocation.antecedent_slots = Box::new([]);
                }
                Ok((
                    ContextualClassReceiver::Declaration(target_class),
                    ForIterationRequirementSpec::Selected(invocation),
                ))
            }
            RealizationCapabilitySelection::Assumed(selection) => {
                let TypeData::Parameter(parameter) = snapshot.store.get(receiver_type) else {
                    return Err(ProjectionError::MissingContextualRealizationTypeParameter(expression));
                };
                let slot_key = phalcom_semantic::generic_evidence::GenericEvidenceSlotKey::new(selection.conformance.clone(), selection.requirement.clone());
                let requirement_slot = generic_evidence
                    .slot_for(&slot_key)
                    .ok_or_else(|| ProjectionError::MissingTraitRequirementSlot(selection.requirement.clone()))?;
                let stable_parameter = phalcom_semantic::metadata::MetadataExporter::new(
                    &snapshot.store,
                    None,
                    None,
                    None,
                    phalcom_type_meta::header::MetadataProfile::RuntimePublic,
                )
                .with_project_universe(projects)
                .export_type_parameter(*parameter)
                .map_err(|error| ProjectionError::DataTypeMetadata(error.to_string()))?;
                Ok((
                    ContextualClassReceiver::TypeParameter(stable_parameter),
                    ForIterationRequirementSpec::Requirement(TraitRequirementInvocationSpec {
                        requirement: selection.requirement.clone(),
                        requirement_slot,
                    }),
                ))
            }
        }
    }

    let (target_receiver, realization_invocation) = project_capability(
        realization.source_expression,
        realization.target_type,
        &realization.realization,
        generic_evidence,
        snapshot,
        projects,
    )?;
    let operation = match &realization.operation {
        ContextualRealizationOperation::Basis {
            basis_type,
            coefficient,
            evaluation,
            ..
        } => {
            let basis_class = snapshot
                .store
                .nominal_origin_declaration(*basis_type)
                .cloned()
                .ok_or(ProjectionError::MissingContextualRealizationClass(realization.source_expression))?;
            let basis_construction =
                build_data_construction_spec(
                    snapshot,
                    projects,
                    &DataConstructorId::new(basis_class),
                    *basis_type,
                    0,
                    Some(Box::new([])),
                    runtime_data_type_roots.get(basis_type).copied(),
                )?;
            let (coefficient_receiver, coefficient_one) = match coefficient {
                RealizationCoefficientSource::One { scalar_type, one } => {
                    let (receiver, invocation) = project_capability(realization.source_expression, *scalar_type, one, generic_evidence, snapshot, projects)?;
                    (Some(receiver), Some(invocation))
                }
                RealizationCoefficientSource::IntegerLiteral { .. }
                | RealizationCoefficientSource::FloatLiteral { .. }
                | RealizationCoefficientSource::Expression { .. } => (None, None),
            };
            ContextualRealizationOperationLowering::Basis {
                basis_construction,
                coefficient: Box::new(coefficient.clone()),
                evaluation: evaluation.clone(),
                coefficient_receiver,
                coefficient_one: coefficient_one.map(Box::new),
            }
        }
        ContextualRealizationOperation::RealEmbedding { value, scalar_type } => ContextualRealizationOperationLowering::RealEmbedding {
            value: *value,
            scalar_type: *scalar_type,
        },
    };

    Ok(ContextualRealizationLoweringSpec {
        source_expression: realization.source_expression,
        source_range: realization.source_range,
        target_receiver,
        realization: realization_invocation,
        operation,
    })
}

fn classify_realization_prerequisite_projection(
    selection: &phalcom_semantic::impls::RequirementSelectionTemplate,
    evidence: &phalcom_semantic::generic_evidence::GenericConformanceEvidence,
    trait_header_has_conformance_constraints: bool,
) -> Result<bool, ProjectionError> {
    use phalcom_semantic::generic_evidence::{GenericConformanceEvidence, NestedGenericEvidence};
    use phalcom_semantic::impls::RequirementSelectionTemplate;

    if trait_header_has_conformance_constraints {
        let GenericConformanceEvidence::Exact(exact) = evidence else {
            return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                "trait-header conformance prerequisites require exact canonical proof slots",
            ));
        };
        let Some(proofs) = exact.trait_header_conformance_proofs.as_ref() else {
            return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                "exact trait-header evidence was not proof-complete",
            ));
        };
        let expected = exact
            .trait_header_contract
            .iter()
            .filter(|constraint| matches!(constraint, phalcom_semantic::types::parameter::GenericConstraint::Conforms { .. }))
            .count();
        if expected == 0 || expected != proofs.len() {
            return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                "exact trait-header proof slots do not match the declared contract",
            ));
        }
        for proof in proofs.values() {
            let GenericConformanceEvidence::Exact(prerequisite) = proof else {
                return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                    "exact trait-header projection requires direct exact prerequisite proofs",
                ));
            };
            if !exact_prerequisite_has_flat_runtime_environment(prerequisite) {
                return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                    "recursive exact trait-header runtime environments are not supported",
                ));
            }
        }
    }

    match evidence {
        GenericConformanceEvidence::Exact(_) => Ok(false),
        GenericConformanceEvidence::Assumed(_) => Err(ProjectionError::UnsupportedContextualRealizationEvidence(
            "assumed evidence must use the canonical assumed-capability projection",
        )),
        GenericConformanceEvidence::CompilerDerived(_) => Err(ProjectionError::UnsupportedContextualRealizationEvidence(
            "compiler-derived structural evidence has no runtime witness table",
        )),
        GenericConformanceEvidence::Derived(derived) => {
            if !matches!(selection, RequirementSelectionTemplate::ConformanceCallable { .. }) {
                return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                    "derived evidence requires a selected conformance callable",
                ));
            }

            let mut has_assumed_prerequisite = false;
            for nested in derived.nested.iter() {
                match nested {
                    NestedGenericEvidence::Conformance(GenericConformanceEvidence::Assumed(_)) => {
                        has_assumed_prerequisite = true;
                    }
                    NestedGenericEvidence::Conformance(
                        GenericConformanceEvidence::Exact(_) | GenericConformanceEvidence::Derived(_) | GenericConformanceEvidence::CompilerDerived(_),
                    ) => {
                        return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                            "only caller-assumed prerequisites can be forwarded safely",
                        ));
                    }
                    NestedGenericEvidence::Equivalent | NestedGenericEvidence::Subtype => {}
                }
            }

            if !has_assumed_prerequisite {
                return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                    "derived evidence has no caller-assumed prerequisite to forward",
                ));
            }
            Ok(true)
        }
    }
}

fn project_realization_prerequisite_recipe(
    selection: &phalcom_semantic::trait_dispatch::TraitDispatchSelection,
    caller_evidence: &phalcom_semantic::generic_evidence::GenericEvidenceEnvironment,
    snapshot: &SemanticSnapshot,
    projects: &phalcom_modules::ProjectUniverse,
) -> Result<Option<Arc<RuntimeCallEnvironmentRecipe>>, ProjectionError> {
    use crate::typing::RuntimeConformanceSlotRecipe;
    use phalcom_semantic::generic_evidence::{
        DerivedConformanceOrigin, GenericConformanceEvidence, GenericConformanceKey, GenericEvidenceEnvironment, GenericEvidenceSlotKey, NestedGenericEvidence,
    };
    use phalcom_semantic::impls::RequirementSelectionTemplate;
    use phalcom_semantic::types::parameter::GenericConstraint;

    let trait_header_has_conformance_constraints = snapshot
        .trait_headers
        .get(&selection.exact_trait_ref.declaration)
        .and_then(|header| header.generic_signature.as_ref())
        .is_some_and(|signature| {
            signature
                .constraints
                .iter()
                .any(|constraint| matches!(constraint, GenericConstraint::Conforms { .. }))
        });
    let needs_recipe = classify_realization_prerequisite_projection(&selection.selection, &selection.evidence, trait_header_has_conformance_constraints)?;
    if !needs_recipe {
        return Ok(None);
    }
    let RequirementSelectionTemplate::ConformanceCallable { callable } = &selection.selection else {
        return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
            "derived evidence requires a selected conformance callable",
        ));
    };
    let GenericConformanceEvidence::Derived(evidence) = &selection.evidence else {
        return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
            "forwarded prerequisites require derived evidence",
        ));
    };
    let signature = match &evidence.origin {
        DerivedConformanceOrigin::ConditionalSource { exact_source } => snapshot
            .conformance_witness_plans
            .get(&exact_source.source_impl)
            .and_then(|plan| plan.generic_signature.as_ref()),
        DerivedConformanceOrigin::GuardedRule { rule, .. } => Some(&rule.generic_signature),
    }
    .ok_or_else(|| ProjectionError::GenericEvidenceLayout("selected realization lacks its prerequisite signature".into()))?;
    if signature.constraints.len() != evidence.nested.len() {
        return Err(ProjectionError::GenericEvidenceLayout(
            "realization prerequisite proofs do not match source constraints".into(),
        ));
    }
    let layout = GenericEvidenceEnvironment::try_from_constraints(&signature.constraints, signature.owner.clone(), &snapshot.trait_surfaces)
        .map_err(|error| ProjectionError::GenericEvidenceLayout(format!("{error:?}")))?;
    let mut slots = vec![None; layout.len()];
    for (constraint, nested) in signature.constraints.iter().zip(evidence.nested.iter()) {
        match (constraint, nested) {
            (GenericConstraint::Conforms { subject, trait_ref }, NestedGenericEvidence::Conformance(proof)) => {
                let conformance = GenericConformanceKey::new(subject.clone(), trait_ref.clone());
                for (key, target_slot) in layout.layout().slots.iter().filter(|(key, _)| key.conformance == conformance) {
                    let recipe = match proof {
                        GenericConformanceEvidence::Assumed(assumed) => {
                            let caller_key = GenericEvidenceSlotKey::new(assumed.key.clone(), key.requirement.clone());
                            let caller_slot = caller_evidence.slot_for(&caller_key).ok_or_else(|| {
                                ProjectionError::GenericEvidenceLayout(format!("realization prerequisite has no caller slot: {caller_key:?}"))
                            })?;
                            RuntimeConformanceSlotRecipe::Forwarded(caller_slot)
                        }
                        GenericConformanceEvidence::Exact(_) => {
                            return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                                "exact nested prerequisites require their complete canonical runtime environment",
                            ));
                        }
                        GenericConformanceEvidence::Derived(_) => {
                            return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                                "nested derived prerequisites require recursive runtime materialization",
                            ));
                        }
                        GenericConformanceEvidence::CompilerDerived(_) => {
                            return Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                                "compiler-derived structural evidence has no runtime witness table",
                            ));
                        }
                    };
                    slots[*target_slot as usize] = Some(recipe);
                }
            }
            (GenericConstraint::Equivalent { .. }, NestedGenericEvidence::Equivalent) | (GenericConstraint::Subtype { .. }, NestedGenericEvidence::Subtype) => {
            }
            _ => {
                return Err(ProjectionError::GenericEvidenceLayout(
                    "realization prerequisite proof does not align with its source constraint".into(),
                ));
            }
        }
    }
    let conformance_slots = slots
        .into_iter()
        .enumerate()
        .map(|(slot, recipe)| recipe.ok_or_else(|| ProjectionError::GenericEvidenceLayout(format!("missing realization prerequisite recipe for slot {slot}"))))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    Ok(Some(Arc::new(RuntimeCallEnvironmentRecipe {
        callable: phalcom_semantic::metadata::stable_identity::to_stable_callable_with_context(
            callable,
            &phalcom_semantic::metadata::stable_identity::StableIdentityContext::new(projects),
        ),
        bindings: Box::new([]),
        conformance_slots,
    })))
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
    runtime_data_type_roots: &BTreeMap<TypeId, RuntimeTypeRef>,
) -> Result<FamilyApplicationLoweringSpec, ProjectionError> {
    match &fam_app.selection {
        FamilyApplicationSelection::Static {
            operation,
            target,
            callable_type,
            ..
        } => {
            let exec_target = match target {
                Some(target) => Some(executable_invocation_target(snapshot, projects, target, operation, *callable_type, runtime_data_type_roots)?),
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
                            runtime_data_type_roots,
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
    runtime_data_type_roots: &BTreeMap<TypeId, RuntimeTypeRef>,
) -> Result<ExecutableInvocationTarget, ProjectionError> {
    match target {
        InvocationTargetId::Behavioral(c) => {
            let lookup_owner = c
                .try_declaration_owner()
                .cloned()
                .or_else(|| {
                    c.conformance_owner()
                        .and_then(|impl_id| snapshot.conformance_index.get(impl_id))
                        .map(|contribution| contribution.target.declaration().clone())
                })
                .ok_or_else(|| {
                    c.conformance_owner()
                        .cloned()
                        .map_or_else(|| ProjectionError::InvalidCallableReferenceSpec, ProjectionError::MissingConformanceMetadata)
                })?;
            Ok(ExecutableInvocationTarget::Behavioral {
                lookup_owner,
                callable: c.clone(),
                operation: operation.clone(),
                rest_mode: executable_rest_mode(snapshot, c),
            })
        }
        InvocationTargetId::VariantConstructor(vc) => Ok(ExecutableInvocationTarget::VariantConstructor { variant: vc.variant.clone() }),
        InvocationTargetId::DataConstructor(dc) => project_data_constructor_target(snapshot, projects, dc, callable_type, None, runtime_data_type_roots),
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
    use super::{ProjectionError, classify_realization_prerequisite_projection, contains_exact_case};
    use phalcom_common::selector::Selector;
    use phalcom_modules::identity::ModuleId;
    use phalcom_semantic::generic_evidence::{
        AssumedConformanceEvidence, DerivedConformanceEvidence, DerivedConformanceOrigin, GenericConformanceEvidence, GenericConformanceKey,
        NestedGenericEvidence,
    };
    use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, ImplId, ImplLocalId, VariantId};
    use phalcom_semantic::impls::{ConformanceEvidence, RequirementSelectionTemplate};
    use phalcom_semantic::traits::TraitRef;
    use phalcom_semantic::types::environment::TypeEnvironment;
    use phalcom_semantic::types::id::TypeId;
    use phalcom_semantic::types::parameter::{TypeParameterOwner, TypeTerm};
    use phalcom_semantic::types::row::{RecordRowField, RecordRowTail, StructuralMemberPresence};
    use phalcom_semantic::types::store::StaticArgument;
    use phalcom_semantic::types::type_member::{ProjectionSubject, TypeLevelProjection, TypeMemberSelector};
    use phalcom_semantic::types::{KindId, ScopedTypeData, TypeStore};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn exact_conformance_evidence() -> Arc<ConformanceEvidence> {
        let module = ModuleId::universe_root();
        let trait_declaration = DeclarationId::new(module.clone(), "Prerequisite".into());
        Arc::new(ConformanceEvidence {
            source_impl: ImplId::new(module, ImplLocalId(0)),
            exact_target: TypeId::DUMMY,
            exact_trait_ref: TraitRef::new(trait_declaration, Box::<[TypeId]>::default()),
            impl_environment: TypeEnvironment::default(),
            requirement_views: BTreeMap::new().into(),
            requirements: BTreeMap::new().into(),
            associated_types: BTreeMap::new().into(),
            trait_header_contract: Box::default(),
            trait_header_conformance_proofs: None,
            fingerprint: Default::default(),
        })
    }

    fn assumed_prerequisite() -> NestedGenericEvidence {
        let module = ModuleId::universe_root();
        let owner = DeclarationId::new(module.clone(), "Caller".into());
        let trait_declaration = DeclarationId::new(module, "Prerequisite".into());
        let key = GenericConformanceKey::new(TypeTerm::Canonical(TypeId::DUMMY), TraitRef::new(trait_declaration, Box::<[TypeId]>::default()));
        NestedGenericEvidence::Conformance(GenericConformanceEvidence::Assumed(AssumedConformanceEvidence {
            key,
            owner: TypeParameterOwner::Declaration(owner),
        }))
    }

    fn derived_evidence(nested: Vec<NestedGenericEvidence>) -> GenericConformanceEvidence {
        GenericConformanceEvidence::Derived(Arc::new(DerivedConformanceEvidence {
            origin: DerivedConformanceOrigin::ConditionalSource {
                exact_source: exact_conformance_evidence(),
            },
            impl_bindings: Default::default(),
            nested: nested.into_boxed_slice(),
        }))
    }

    fn conformance_callable() -> RequirementSelectionTemplate {
        let owner = DeclarationId::new(ModuleId::universe_root(), "Realizable".into());
        let callable = CallableId::method(owner, Selector::getter("basis").expect("selector"), DispatchSide::Class);
        RequirementSelectionTemplate::ConformanceCallable { callable }
    }

    #[test]
    fn contextual_realization_prerequisite_projection_fails_closed_outside_assumed_forwarding() {
        let selection = conformance_callable();
        assert_eq!(
            classify_realization_prerequisite_projection(
                &selection,
                &derived_evidence(vec![
                    assumed_prerequisite(),
                    NestedGenericEvidence::Equivalent,
                    assumed_prerequisite(),
                    NestedGenericEvidence::Subtype,
                ]),
                false,
            ),
            Ok(true),
            "all conformance prerequisites may be forwarded alongside static equality/subtype proofs"
        );
        for nested in [
            vec![
                assumed_prerequisite(),
                NestedGenericEvidence::Conformance(GenericConformanceEvidence::Exact(exact_conformance_evidence())),
            ],
            vec![NestedGenericEvidence::Conformance(GenericConformanceEvidence::Exact(
                exact_conformance_evidence(),
            ))],
            vec![NestedGenericEvidence::Conformance(derived_evidence(vec![assumed_prerequisite()]))],
            vec![assumed_prerequisite(), NestedGenericEvidence::Conformance(derived_evidence(vec![]))],
        ] {
            assert_eq!(
                classify_realization_prerequisite_projection(&selection, &derived_evidence(nested), false),
                Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                    "only caller-assumed prerequisites can be forwarded safely"
                )),
                "mixed and nested exact/derived proofs must not reach partial legacy projection"
            );
        }
        for nested in [vec![], vec![NestedGenericEvidence::Equivalent, NestedGenericEvidence::Subtype]] {
            assert_eq!(
                classify_realization_prerequisite_projection(&selection, &derived_evidence(nested), false),
                Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                    "derived evidence has no caller-assumed prerequisite to forward"
                )),
                "derived evidence without a forwardable prerequisite must not use the legacy path"
            );
        }

        let RequirementSelectionTemplate::ConformanceCallable { callable } = &selection else {
            unreachable!();
        };
        for unsupported_selection in [
            RequirementSelectionTemplate::TraitDefault { callable: callable.clone() },
            RequirementSelectionTemplate::InherentCallable {
                callable: callable.clone(),
                conditional_impl: None,
                applicability: None,
            },
            RequirementSelectionTemplate::DataComponent {
                component: phalcom_semantic::identity::DataComponentId::new(DeclarationId::new(ModuleId::universe_root(), "Realizable".into()), 0),
                specialized_type: TypeId::DUMMY,
            },
        ] {
            assert_eq!(
                classify_realization_prerequisite_projection(&unsupported_selection, &derived_evidence(vec![assumed_prerequisite()]), false),
                Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                    "derived evidence requires a selected conformance callable"
                )),
                "unsupported selection forms must fail before invocation projection"
            );
        }

        let exact = GenericConformanceEvidence::Exact(exact_conformance_evidence());
        assert_eq!(classify_realization_prerequisite_projection(&selection, &exact, false), Ok(false));
        assert_eq!(
            classify_realization_prerequisite_projection(&selection, &exact, true),
            Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                "exact trait-header evidence was not proof-complete"
            )),
            "a raw exact witness must not stand in for its trait-header proofs"
        );
        assert_eq!(
            classify_realization_prerequisite_projection(&selection, &derived_evidence(vec![assumed_prerequisite()]), true),
            Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                "trait-header conformance prerequisites require exact canonical proof slots"
            )),
            "derived parent evidence does not use the exact trait-header projection path"
        );

        let gate_ref = TraitRef::new(DeclarationId::new(ModuleId::universe_root(), "Gate".into()), Box::<[TypeId]>::default());
        let gate_key = GenericConformanceKey::new(TypeTerm::Canonical(TypeId::DUMMY), gate_ref);
        let mut completed_prerequisite = (*exact_conformance_evidence()).clone();
        completed_prerequisite.trait_header_conformance_proofs = Some(BTreeMap::new().into());
        let mut completed_parent = (*exact_conformance_evidence()).clone();
        completed_parent.trait_header_contract = Box::new([phalcom_semantic::types::parameter::GenericConstraint::Conforms {
            subject: TypeTerm::Canonical(TypeId::DUMMY),
            trait_ref: gate_key.trait_ref.clone(),
        }]);
        completed_parent.trait_header_conformance_proofs =
            Some(BTreeMap::from([(gate_key, GenericConformanceEvidence::Exact(Arc::new(completed_prerequisite)))]).into());
        assert_eq!(
            classify_realization_prerequisite_projection(&selection, &GenericConformanceEvidence::Exact(Arc::new(completed_parent)), true),
            Ok(false),
            "one-level direct exact prerequisites with a complete empty nested environment are projectable"
        );

        let mut recursive_prerequisite = (*exact_conformance_evidence()).clone();
        recursive_prerequisite.trait_header_conformance_proofs = Some(
            BTreeMap::from([(
                GenericConformanceKey::new(
                    TypeTerm::Canonical(TypeId::DUMMY),
                    TraitRef::new(DeclarationId::new(ModuleId::universe_root(), "Nested".into()), Box::<[TypeId]>::default()),
                ),
                GenericConformanceEvidence::Exact(exact_conformance_evidence()),
            )])
            .into(),
        );
        let mut recursive_parent = (*exact_conformance_evidence()).clone();
        recursive_parent.trait_header_contract = Box::new([phalcom_semantic::types::parameter::GenericConstraint::Conforms {
            subject: TypeTerm::Canonical(TypeId::DUMMY),
            trait_ref: TraitRef::new(DeclarationId::new(ModuleId::universe_root(), "Gate".into()), Box::<[TypeId]>::default()),
        }]);
        recursive_parent.trait_header_conformance_proofs = Some(
            BTreeMap::from([(
                GenericConformanceKey::new(
                    TypeTerm::Canonical(TypeId::DUMMY),
                    TraitRef::new(DeclarationId::new(ModuleId::universe_root(), "Gate".into()), Box::<[TypeId]>::default()),
                ),
                GenericConformanceEvidence::Exact(Arc::new(recursive_prerequisite)),
            )])
            .into(),
        );
        assert_eq!(
            classify_realization_prerequisite_projection(&selection, &GenericConformanceEvidence::Exact(Arc::new(recursive_parent)), true),
            Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                "recursive exact trait-header runtime environments are not supported"
            )),
            "recursive exact environments remain fail-closed"
        );
        let NestedGenericEvidence::Conformance(assumed) = assumed_prerequisite() else {
            unreachable!();
        };
        assert_eq!(
            classify_realization_prerequisite_projection(&selection, &assumed, false),
            Err(ProjectionError::UnsupportedContextualRealizationEvidence(
                "assumed evidence must use the canonical assumed-capability projection"
            )),
            "top-level assumptions must use their established capability path"
        );
    }

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

    #[test]
    fn exact_case_detection_descends_into_applied_record_rows_and_projection_tails() {
        let mut store = TypeStore::new();
        let choice_owner = DeclarationId::new(ModuleId::universe_root(), "Choice".into());
        let choice = store.nominal_type(choice_owner.clone());
        let variant = VariantId::new(choice_owner, Selector::method("A", []).expect("variant selector"));
        let exact_case = store.exact_case_type(&variant, choice).expect("exact case");

        let applied_owner = DeclarationId::new(ModuleId::universe_root(), "RowBox".into());
        let row_box_kind = store.arrow_kind(Box::new([KindId::RECORD_ROW]), KindId::TYPE);
        let row_box = store.nominal_form(applied_owner, row_box_kind);
        let field_row = store
            .record_row_checked(
                vec![RecordRowField {
                    key: phalcom_semantic::FieldKey::symbol("payload"),
                    ty: exact_case,
                    presence: StructuralMemberPresence::Required,
                }],
                RecordRowTail::Closed,
            )
            .expect("row with exact case field");
        let applied_field = store
            .apply_static_form(row_box, &[StaticArgument::RecordRow(field_row)])
            .expect("applied row field");
        assert!(contains_exact_case(&store, applied_field));

        let projected_row = store.type_member_projection(
            TypeLevelProjection {
                subject: ProjectionSubject::Type(exact_case),
                selector: TypeMemberSelector::Named("Fields".into()),
                arguments: Box::new([]),
                source_declaration: None,
            },
            KindId::RECORD_ROW,
        );
        let projected_tail = store
            .record_row_checked(Vec::new(), RecordRowTail::Projection(projected_row))
            .expect("projected row tail");
        let applied_tail = store
            .apply_static_form(row_box, &[StaticArgument::RecordRow(projected_tail)])
            .expect("applied projected tail");
        assert!(contains_exact_case(&store, applied_tail));
    }
}
