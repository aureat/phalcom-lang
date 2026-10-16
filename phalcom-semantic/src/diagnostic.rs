//! Canonical semantic diagnostics shared between compiler and LSP.

use phalcom_common::range::SourceRange;
use phalcom_modules::identity::ModuleId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticCode {
    BindingInitializerMismatch,
    BindingRedeclared,
    ConstWithoutInitializer,
    UninitializedBindingRead,
    AssignmentToImmutable,
    StaticObjectMutation,
    AssignmentMismatch,
    ReturnMismatch,
    ExistentialEscape,
    ArgumentMismatch,
    GenericInferenceConflict,
    GenericInferenceUnderconstrained,
    GenericInferenceAmbiguous,
    GenericConstraintUnsatisfied,
    CallShapeMismatch,
    NotCallable,
    MemberInaccessible,
    NativeSignatureMismatch,
    IncompatibleOverride,
    FieldMismatch,
    FieldNotDeclared,
    ImmutableFieldWriteOutsideConstruction,
    ImmutableFieldAlreadyInitialized,
    ImmutableFieldNotInitialized,
    ImmutableClassFieldRequiresInitializer,
    ConstructorSuperRequired,
    ConstructorSuperInvalid,
    TypeMismatch,
    AnnotationUnresolved,
    AnnotationUnsupported,
    AnnotationUnsaturatedConstructor,
    KindExpectedType,
    ApplicationNotConstructor,
    ApplicationTooManyArguments,
    ApplicationArgumentKindMismatch,
    ProjectLoadFailed,
    ModuleInterfaceFailed,
    ModuleImportUnresolved,
    ModuleExposureRejected,
    ModuleExportMissing,
    ModuleRelativeInvalidRoot,
    ModuleLinkFailed,
    ModuleRuntimeCycle,
    AnalysisBlocked,
    AnalysisBudgetExceeded,
    AnalysisInternalFailure,
    TypeRelationCycle,
    TypeAliasCycle,
    TypeSignatureCycle,
    CallableResultCycle,
    StaticConstantCycle,
    TypeDynamicBoundary,
    TypeOnlyRuntimeValue,
    EnumVariantDuplicate,
    EnumVariantResultWrongOwner,
    EnumVariantResultUnsaturated,
    EnumVariantResultInvalid,
    EnumVariantGadtCyclicEquality,
    EnumVariantVisibilityInvalid,
    EnumCaseStaticBehaviorUnsupported,
    EnumCaseDeclarationOnlyBehavior,
    EnumFamilyCategoryConflict,
    EnumFamilyInheritedBehaviorConflict,
    EnumRequirementIncomplete,
    EnumRequirementMissing,
    EnumRequirementIncompatible,
    AssociatedOwnerUnresolved,
    AssociatedOwnerNotTypeForm,
    AssociatedOwnerNotDeclarationBacked,
    AssociatedFamilyMissing,
    AssociatedFamilyInaccessible,
    AssociatedMemberMissing,
    AssociatedMemberInaccessible,
    AssociatedMemberNotConstructible,
    AssociatedCallShapeMissing,
    AssociatedCallAmbiguous,
    AssociatedCallDynamicShape,
    AssociatedGenericUnderconstrained,
    AssociatedGenericOwnerConflict,
    AssociatedGadtOwnerConflict,
    AssociatedFamilyTypeInvalid,
    RecordKeyNotStatic,
    RecordInvalidFieldKey,
    RecordDuplicateField,
    RecordSpreadSourceUnsupported,
    RecordSpreadNonSymbolCallKey,
    RecordRowTailUnresolved,
    RecordRowTailKindMismatch,
    RecordRowLacksViolation,
    RecordRowOccursCheck,
    RecordRowInferenceUnderconstrained,
    RecordRowInferenceConflict,
    RecordRowLacksUnproven,
    MatchNonExhaustive,
    MatchArmRedundant,
    MatchPatternUnresolved,
    MatchPatternArityMismatch,
    MatchPatternFieldMismatch,
    MatchPatternContradictory,
    /// A single pattern alternative binds the same source name more than once.
    MatchPatternDuplicateBinding,
    /// Or-pattern alternatives do not establish one common binding-name set.
    MatchPatternOrBindingMismatch,
    /// A later or-pattern alternative contributes no value beyond earlier alternatives.
    MatchPatternOrRedundant,
    /// Pattern is outside the original scrutinee domain rather than merely covered earlier.
    MatchPatternImpossible,
    /// Match totality or reachability could not be proved because formal analysis was blocked.
    MatchAnalysisBlocked,
    /// A refutable pattern was used in a declaration binding (let/var).
    DeclarationPatternRefutable,
    DataDuplicateComponent,
    DataComponentImmutable,
    DataUsedAsSuperclass,
    ImplForeignTarget,
    ImplTargetNotNominal,
    ImplTargetTypeAlias,
    ImplSpecializedTargetUnsupported,
    ImplWhereClauseUnsupported,
    ImplMemberConflict,
    DelegationFieldNotFound,
    DelegationFieldTypeRequired,
    DelegationSetterRequiresMutableField,
    DelegationInConformanceUnsupported,
    AssociatedTypeDuplicate,
    AssociatedTypeUnknown,
    AssociatedTypeMissing,
    AssociatedTypeBindingInvalid,
    AssociatedTypeInherentUnsupported,
    ImplConstructorUnsupported,
    ImplBodylessMemberUnsupported,
    ImplUnusedTypeParameter,
    ImplEnumCaseTargetUnsupported,
    ImplConformanceOverlap,
    ImplConformanceIncomplete,
    TraitMemberConflict,
    TraitDispatchAmbiguous,
    DerivedConformanceUnguarded,
    DerivedConformanceUnauthorized,
    DerivedConformanceOverlap,
    ConformanceCompilerDerivedTrait,
    StaticIterationNotStructural,
    StaticIterationOpenTailNotFinite,
}

impl DiagnosticCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BindingInitializerMismatch => "type.binding.initializer_mismatch",
            Self::BindingRedeclared => "binding.redeclared",
            Self::ConstWithoutInitializer => "binding.const_requires_initializer",
            Self::UninitializedBindingRead => "binding.uninitialized_read",
            Self::AssignmentToImmutable => "binding.assign_to_immutable",
            Self::StaticObjectMutation => "binding.static_object_mutation",
            Self::AssignmentMismatch => "type.assignment.mismatch",
            Self::ReturnMismatch => "type.return.mismatch",
            Self::ExistentialEscape => "type.existential.escape",
            Self::ArgumentMismatch => "type.call.argument_mismatch",
            Self::GenericInferenceConflict => "type.generic.inference_conflict",
            Self::GenericInferenceUnderconstrained => "type.generic.underconstrained",
            Self::GenericInferenceAmbiguous => "type.generic.inference_ambiguous",
            Self::GenericConstraintUnsatisfied => "type.generic.constraint_unsatisfied",
            Self::CallShapeMismatch => "type.call.shape_mismatch",
            Self::NotCallable => "type.call.not_callable",
            Self::MemberInaccessible => "member.inaccessible",
            Self::NativeSignatureMismatch => "native.signature_mismatch",
            Self::IncompatibleOverride => "type.override.incompatible",
            Self::FieldMismatch => "type.field.mismatch",
            Self::FieldNotDeclared => "field.not_declared",
            Self::ImmutableFieldWriteOutsideConstruction => "field.immutable_write_outside_construction",
            Self::ImmutableFieldAlreadyInitialized => "field.immutable_already_initialized",
            Self::ImmutableFieldNotInitialized => "field.immutable_not_initialized",
            Self::ImmutableClassFieldRequiresInitializer => "field.immutable_class_requires_initializer",
            Self::ConstructorSuperRequired => "constructor.super_required",
            Self::ConstructorSuperInvalid => "constructor.super_invalid",
            Self::TypeMismatch => "type.mismatch",
            Self::AnnotationUnresolved => "type.annotation.unresolved",
            Self::AnnotationUnsupported => "type.annotation.unsupported",
            Self::AnnotationUnsaturatedConstructor => "type.annotation.unsaturated_constructor",
            Self::KindExpectedType => "type.kind.expected_type",
            Self::ApplicationNotConstructor => "type.application.not_constructor",
            Self::ApplicationTooManyArguments => "type.application.too_many_arguments",
            Self::ApplicationArgumentKindMismatch => "type.application.argument_kind_mismatch",
            Self::ProjectLoadFailed => "project.load.failed",
            Self::ModuleInterfaceFailed => "module.interface.failed",
            Self::ModuleImportUnresolved => "module.import.unresolved",
            Self::ModuleExposureRejected => "module.exposure.rejected",
            Self::ModuleExportMissing => "module.export.missing",
            Self::ModuleRelativeInvalidRoot => "module.relative.invalid_root",
            Self::ModuleLinkFailed => "module.link.failed",
            Self::ModuleRuntimeCycle => "module.runtime_cycle",
            Self::AnalysisBlocked => "analysis.blocked",
            Self::AnalysisBudgetExceeded => "analysis.budget_exceeded",
            Self::AnalysisInternalFailure => "analysis.internal_failure",
            Self::TypeRelationCycle => "type.relation.cycle",
            Self::TypeAliasCycle => "type.alias.cycle",
            Self::TypeSignatureCycle => "type.signature.cycle",
            Self::CallableResultCycle => "callable.result.cycle",
            Self::StaticConstantCycle => "static.constant.cycle",
            Self::TypeDynamicBoundary => "type.dynamic_boundary",
            Self::TypeOnlyRuntimeValue => "type.type_only_runtime_value",
            Self::EnumVariantDuplicate => "enum.variant.duplicate",
            Self::EnumVariantResultWrongOwner => "enum.variant.result_wrong_owner",
            Self::EnumVariantResultUnsaturated => "enum.variant.result_unsaturated",
            Self::EnumVariantResultInvalid => "enum.variant.result_invalid",
            Self::EnumVariantGadtCyclicEquality => "enum.variant.gadt_cyclic_equality",
            Self::EnumVariantVisibilityInvalid => "enum.variant.visibility_invalid",
            Self::EnumCaseStaticBehaviorUnsupported => "enum.case.static_behavior_unsupported",
            Self::EnumCaseDeclarationOnlyBehavior => "enum.case.declaration_only_behavior",
            Self::EnumFamilyCategoryConflict => "enum.family.category_conflict",
            Self::EnumFamilyInheritedBehaviorConflict => "enum.family.inherited_behavior_conflict",
            Self::EnumRequirementIncomplete => "enum.requirement.incomplete",
            Self::EnumRequirementMissing => "enum.requirement.missing",
            Self::EnumRequirementIncompatible => "enum.requirement.incompatible",
            Self::AssociatedOwnerUnresolved => "associated.owner.unresolved",
            Self::AssociatedOwnerNotTypeForm => "associated.owner.not_type_form",
            Self::AssociatedOwnerNotDeclarationBacked => "associated.owner.not_declaration_backed",
            Self::AssociatedFamilyMissing => "associated.family.missing",
            Self::AssociatedFamilyInaccessible => "associated.family.inaccessible",
            Self::AssociatedMemberMissing => "associated.member.missing",
            Self::AssociatedMemberInaccessible => "associated.member.inaccessible",
            Self::AssociatedMemberNotConstructible => "associated.member.not_constructible",
            Self::AssociatedCallShapeMissing => "associated.call.shape_missing",
            Self::AssociatedCallAmbiguous => "associated.call.ambiguous",
            Self::AssociatedCallDynamicShape => "associated.call.dynamic_shape",
            Self::AssociatedGenericUnderconstrained => "associated.generic.underconstrained",
            Self::AssociatedGenericOwnerConflict => "associated.generic.owner_conflict",
            Self::AssociatedGadtOwnerConflict => "associated.gadt.owner_conflict",
            Self::AssociatedFamilyTypeInvalid => "associated.family.type_invalid",
            Self::RecordKeyNotStatic => "type.record.key_not_static",
            Self::RecordInvalidFieldKey => "type.record.invalid_field_key",
            Self::RecordDuplicateField => "type.record.duplicate_field",
            Self::RecordSpreadSourceUnsupported => "type.record.spread_source_unsupported",
            Self::RecordSpreadNonSymbolCallKey => "record.spread_non_symbol_call_key",
            Self::RecordRowTailUnresolved => "type.record.row_tail_unresolved",
            Self::RecordRowTailKindMismatch => "type.record.row_tail_kind_mismatch",
            Self::RecordRowLacksViolation => "type.record.row_lacks_violation",
            Self::RecordRowOccursCheck => "type.record.row_occurs_check",
            Self::RecordRowInferenceUnderconstrained => "type.record.row_inference_underconstrained",
            Self::RecordRowInferenceConflict => "type.record.row_inference_conflict",
            Self::RecordRowLacksUnproven => "type.record.row_lacks_unproven",
            Self::MatchNonExhaustive => "match.non_exhaustive",
            Self::MatchArmRedundant => "match.arm_redundant",
            Self::MatchPatternUnresolved => "match.pattern_unresolved",
            Self::MatchPatternArityMismatch => "match.pattern_arity_mismatch",
            Self::MatchPatternFieldMismatch => "match.pattern_field_mismatch",
            Self::MatchPatternContradictory => "match.pattern_contradictory",
            Self::MatchPatternDuplicateBinding => "match.pattern.duplicate_binding",
            Self::MatchPatternOrBindingMismatch => "match.pattern.or_binding_mismatch",
            Self::MatchPatternOrRedundant => "match.pattern.or_redundant",
            Self::MatchPatternImpossible => "match.pattern.impossible",
            Self::MatchAnalysisBlocked => "match.analysis.blocked",
            Self::DeclarationPatternRefutable => "declaration.pattern_refutable",
            Self::DataDuplicateComponent => "data.duplicate_component",
            Self::DataComponentImmutable => "data.component_immutable",
            Self::DataUsedAsSuperclass => "data.used_as_superclass",
            Self::ImplForeignTarget => "impl.target.foreign",
            Self::ImplTargetNotNominal => "impl.target.not_nominal",
            Self::ImplTargetTypeAlias => "impl.target.type_alias",
            Self::ImplSpecializedTargetUnsupported => "impl.target.specialized_unsupported",
            Self::ImplWhereClauseUnsupported => "impl.where_clause_unsupported",
            Self::ImplMemberConflict => "impl.member.conflict",
            Self::DelegationFieldNotFound => "delegation.field_not_found",
            Self::DelegationFieldTypeRequired => "delegation.field_type_required",
            Self::DelegationSetterRequiresMutableField => "delegation.setter_requires_mutable_field",
            Self::DelegationInConformanceUnsupported => "delegation.conformance_unsupported",
            Self::AssociatedTypeDuplicate => "associated_type.duplicate",
            Self::AssociatedTypeUnknown => "associated_type.unknown",
            Self::AssociatedTypeMissing => "associated_type.missing",
            Self::AssociatedTypeBindingInvalid => "associated_type.binding_invalid",
            Self::AssociatedTypeInherentUnsupported => "associated_type.inherent_unsupported",
            Self::ImplConstructorUnsupported => "impl.member.constructor_unsupported",
            Self::ImplBodylessMemberUnsupported => "impl.member.bodyless_unsupported",
            Self::ImplUnusedTypeParameter => "impl.type_parameter.unused",
            Self::ImplEnumCaseTargetUnsupported => "impl.target.enum_case_unsupported",
            Self::ImplConformanceOverlap => "impl.conformance.overlap",
            Self::ImplConformanceIncomplete => "impl.conformance.incomplete",
            Self::TraitMemberConflict => "trait.member.conflict",
            Self::TraitDispatchAmbiguous => "trait.dispatch.ambiguous",
            Self::DerivedConformanceUnguarded => "impl.derived_conformance.unguarded",
            Self::DerivedConformanceUnauthorized => "impl.derived_conformance.unauthorized",
            Self::DerivedConformanceOverlap => "impl.derived_conformance.overlap",
            Self::ConformanceCompilerDerivedTrait => "impl.conformance.compiler_derived",
            Self::StaticIterationNotStructural => "static.iteration_not_structural",
            Self::StaticIterationOpenTailNotFinite => "static.iteration_open_tail_not_finite",
        }
    }

    /// Parses the stable diagnostic code used by semantic artifact records.
    pub fn from_stable_code(code: &str) -> Option<Self> {
        match code {
            "type.binding.initializer_mismatch" => Some(Self::BindingInitializerMismatch),
            "binding.redeclared" => Some(Self::BindingRedeclared),
            "binding.const_requires_initializer" => Some(Self::ConstWithoutInitializer),
            "binding.uninitialized_read" => Some(Self::UninitializedBindingRead),
            "binding.assign_to_immutable" => Some(Self::AssignmentToImmutable),
            "binding.static_object_mutation" => Some(Self::StaticObjectMutation),
            "type.assignment.mismatch" => Some(Self::AssignmentMismatch),
            "type.return.mismatch" => Some(Self::ReturnMismatch),
            "type.existential.escape" => Some(Self::ExistentialEscape),
            "type.call.argument_mismatch" => Some(Self::ArgumentMismatch),
            "type.generic.inference_conflict" => Some(Self::GenericInferenceConflict),
            "type.generic.underconstrained" => Some(Self::GenericInferenceUnderconstrained),
            "type.generic.inference_ambiguous" => Some(Self::GenericInferenceAmbiguous),
            "type.generic.constraint_unsatisfied" => Some(Self::GenericConstraintUnsatisfied),
            "type.call.shape_mismatch" => Some(Self::CallShapeMismatch),
            "type.call.not_callable" => Some(Self::NotCallable),
            "member.inaccessible" => Some(Self::MemberInaccessible),
            "native.signature_mismatch" => Some(Self::NativeSignatureMismatch),
            "type.override.incompatible" => Some(Self::IncompatibleOverride),
            "type.field.mismatch" => Some(Self::FieldMismatch),
            "field.not_declared" => Some(Self::FieldNotDeclared),
            "field.immutable_write_outside_construction" => Some(Self::ImmutableFieldWriteOutsideConstruction),
            "field.immutable_already_initialized" => Some(Self::ImmutableFieldAlreadyInitialized),
            "field.immutable_not_initialized" => Some(Self::ImmutableFieldNotInitialized),
            "field.immutable_class_requires_initializer" => Some(Self::ImmutableClassFieldRequiresInitializer),
            "constructor.super_required" => Some(Self::ConstructorSuperRequired),
            "constructor.super_invalid" => Some(Self::ConstructorSuperInvalid),
            "type.mismatch" => Some(Self::TypeMismatch),
            "type.annotation.unresolved" => Some(Self::AnnotationUnresolved),
            "type.annotation.unsupported" => Some(Self::AnnotationUnsupported),
            "type.annotation.unsaturated_constructor" => Some(Self::AnnotationUnsaturatedConstructor),
            "type.kind.expected_type" => Some(Self::KindExpectedType),
            "type.application.not_constructor" => Some(Self::ApplicationNotConstructor),
            "type.application.too_many_arguments" => Some(Self::ApplicationTooManyArguments),
            "type.application.argument_kind_mismatch" => Some(Self::ApplicationArgumentKindMismatch),
            "project.load.failed" => Some(Self::ProjectLoadFailed),
            "module.interface.failed" => Some(Self::ModuleInterfaceFailed),
            "module.import.unresolved" => Some(Self::ModuleImportUnresolved),
            "module.exposure.rejected" => Some(Self::ModuleExposureRejected),
            "module.export.missing" => Some(Self::ModuleExportMissing),
            "module.relative.invalid_root" => Some(Self::ModuleRelativeInvalidRoot),
            "module.link.failed" => Some(Self::ModuleLinkFailed),
            "module.runtime_cycle" => Some(Self::ModuleRuntimeCycle),
            "analysis.blocked" => Some(Self::AnalysisBlocked),
            "analysis.budget_exceeded" => Some(Self::AnalysisBudgetExceeded),
            "analysis.internal_failure" => Some(Self::AnalysisInternalFailure),
            "type.relation.cycle" => Some(Self::TypeRelationCycle),
            "type.alias.cycle" => Some(Self::TypeAliasCycle),
            "type.signature.cycle" => Some(Self::TypeSignatureCycle),
            "callable.result.cycle" => Some(Self::CallableResultCycle),
            "static.constant.cycle" => Some(Self::StaticConstantCycle),
            "type.dynamic_boundary" => Some(Self::TypeDynamicBoundary),
            "type.type_only_runtime_value" => Some(Self::TypeOnlyRuntimeValue),
            "enum.variant.duplicate" => Some(Self::EnumVariantDuplicate),
            "enum.variant.result_wrong_owner" => Some(Self::EnumVariantResultWrongOwner),
            "enum.variant.result_unsaturated" => Some(Self::EnumVariantResultUnsaturated),
            "enum.variant.result_invalid" => Some(Self::EnumVariantResultInvalid),
            "enum.variant.gadt_cyclic_equality" => Some(Self::EnumVariantGadtCyclicEquality),
            "enum.variant.visibility_invalid" => Some(Self::EnumVariantVisibilityInvalid),
            "enum.case.static_behavior_unsupported" => Some(Self::EnumCaseStaticBehaviorUnsupported),
            "enum.case.declaration_only_behavior" => Some(Self::EnumCaseDeclarationOnlyBehavior),
            "enum.family.category_conflict" => Some(Self::EnumFamilyCategoryConflict),
            "enum.family.inherited_behavior_conflict" => Some(Self::EnumFamilyInheritedBehaviorConflict),
            "enum.requirement.incomplete" => Some(Self::EnumRequirementIncomplete),
            "enum.requirement.missing" => Some(Self::EnumRequirementMissing),
            "enum.requirement.incompatible" => Some(Self::EnumRequirementIncompatible),
            "associated.owner.unresolved" => Some(Self::AssociatedOwnerUnresolved),
            "associated.owner.not_type_form" => Some(Self::AssociatedOwnerNotTypeForm),
            "associated.owner.not_declaration_backed" => Some(Self::AssociatedOwnerNotDeclarationBacked),
            "associated.family.missing" => Some(Self::AssociatedFamilyMissing),
            "associated.family.inaccessible" => Some(Self::AssociatedFamilyInaccessible),
            "associated.member.missing" => Some(Self::AssociatedMemberMissing),
            "associated.member.inaccessible" => Some(Self::AssociatedMemberInaccessible),
            "associated.member.not_constructible" => Some(Self::AssociatedMemberNotConstructible),
            "associated.call.shape_missing" => Some(Self::AssociatedCallShapeMissing),
            "associated.call.ambiguous" => Some(Self::AssociatedCallAmbiguous),
            "associated.call.dynamic_shape" => Some(Self::AssociatedCallDynamicShape),
            "associated.generic.underconstrained" => Some(Self::AssociatedGenericUnderconstrained),
            "associated.generic.owner_conflict" => Some(Self::AssociatedGenericOwnerConflict),
            "associated.gadt.owner_conflict" => Some(Self::AssociatedGadtOwnerConflict),
            "associated.family.type_invalid" => Some(Self::AssociatedFamilyTypeInvalid),
            "type.record.key_not_static" => Some(Self::RecordKeyNotStatic),
            "type.record.invalid_field_key" => Some(Self::RecordInvalidFieldKey),
            "type.record.duplicate_field" => Some(Self::RecordDuplicateField),
            "type.record.spread_source_unsupported" => Some(Self::RecordSpreadSourceUnsupported),
            "record.spread_non_symbol_call_key" => Some(Self::RecordSpreadNonSymbolCallKey),
            "type.record.row_tail_unresolved" => Some(Self::RecordRowTailUnresolved),
            "type.record.row_tail_kind_mismatch" => Some(Self::RecordRowTailKindMismatch),
            "type.record.row_lacks_violation" => Some(Self::RecordRowLacksViolation),
            "type.record.row_occurs_check" => Some(Self::RecordRowOccursCheck),
            "type.record.row_inference_underconstrained" => Some(Self::RecordRowInferenceUnderconstrained),
            "type.record.row_inference_conflict" => Some(Self::RecordRowInferenceConflict),
            "type.record.row_lacks_unproven" => Some(Self::RecordRowLacksUnproven),
            "match.non_exhaustive" => Some(Self::MatchNonExhaustive),
            "match.arm_redundant" => Some(Self::MatchArmRedundant),
            "match.pattern_unresolved" => Some(Self::MatchPatternUnresolved),
            "match.pattern_arity_mismatch" => Some(Self::MatchPatternArityMismatch),
            "match.pattern_field_mismatch" => Some(Self::MatchPatternFieldMismatch),
            "match.pattern_contradictory" => Some(Self::MatchPatternContradictory),
            "match.pattern.duplicate_binding" => Some(Self::MatchPatternDuplicateBinding),
            "match.pattern.or_binding_mismatch" => Some(Self::MatchPatternOrBindingMismatch),
            "match.pattern.or_redundant" => Some(Self::MatchPatternOrRedundant),
            "match.pattern.impossible" => Some(Self::MatchPatternImpossible),
            "match.analysis.blocked" => Some(Self::MatchAnalysisBlocked),
            "declaration.pattern_refutable" => Some(Self::DeclarationPatternRefutable),
            "data.duplicate_component" => Some(Self::DataDuplicateComponent),
            "data.component_immutable" => Some(Self::DataComponentImmutable),
            "data.used_as_superclass" => Some(Self::DataUsedAsSuperclass),
            "impl.target.foreign" => Some(Self::ImplForeignTarget),
            "impl.target.not_nominal" => Some(Self::ImplTargetNotNominal),
            "impl.target.type_alias" => Some(Self::ImplTargetTypeAlias),
            "impl.target.specialized_unsupported" => Some(Self::ImplSpecializedTargetUnsupported),
            "impl.where_clause_unsupported" => Some(Self::ImplWhereClauseUnsupported),
            "impl.member.conflict" => Some(Self::ImplMemberConflict),
            "delegation.field_not_found" => Some(Self::DelegationFieldNotFound),
            "delegation.field_type_required" => Some(Self::DelegationFieldTypeRequired),
            "delegation.setter_requires_mutable_field" => Some(Self::DelegationSetterRequiresMutableField),
            "delegation.conformance_unsupported" => Some(Self::DelegationInConformanceUnsupported),
            "associated_type.duplicate" => Some(Self::AssociatedTypeDuplicate),
            "associated_type.unknown" => Some(Self::AssociatedTypeUnknown),
            "associated_type.missing" => Some(Self::AssociatedTypeMissing),
            "associated_type.binding_invalid" => Some(Self::AssociatedTypeBindingInvalid),
            "associated_type.inherent_unsupported" => Some(Self::AssociatedTypeInherentUnsupported),
            "impl.member.constructor_unsupported" => Some(Self::ImplConstructorUnsupported),
            "impl.member.bodyless_unsupported" => Some(Self::ImplBodylessMemberUnsupported),
            "impl.type_parameter.unused" => Some(Self::ImplUnusedTypeParameter),
            "impl.target.enum_case_unsupported" => Some(Self::ImplEnumCaseTargetUnsupported),
            "impl.conformance.overlap" => Some(Self::ImplConformanceOverlap),
            "impl.conformance.incomplete" => Some(Self::ImplConformanceIncomplete),
            "trait.member.conflict" => Some(Self::TraitMemberConflict),
            "trait.dispatch.ambiguous" => Some(Self::TraitDispatchAmbiguous),
            "impl.derived_conformance.unguarded" => Some(Self::DerivedConformanceUnguarded),
            "impl.derived_conformance.unauthorized" => Some(Self::DerivedConformanceUnauthorized),
            "impl.derived_conformance.overlap" => Some(Self::DerivedConformanceOverlap),
            "impl.conformance.compiler_derived" => Some(Self::ConformanceCompilerDerivedTrait),
            "static.iteration_not_structural" => Some(Self::StaticIterationNotStructural),
            "static.iteration_open_tail_not_finite" => Some(Self::StaticIterationOpenTailNotFinite),
            _ => None,
        }
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Source-owned location spanning a specific module and byte range.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SemanticSourceSpan {
    pub module: ModuleId,
    pub range: SourceRange,
}

impl SemanticSourceSpan {
    pub fn new(module: ModuleId, range: SourceRange) -> Self {
        Self { module, range }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticLabel {
    pub span: SemanticSourceSpan,
    pub range: SourceRange,
    pub message: String,
}

impl DiagnosticLabel {
    pub fn new(span: SemanticSourceSpan, message: impl Into<String>) -> Self {
        let range = span.range;
        Self {
            span,
            range,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticFix {
    pub message: String,
    pub replacement: Option<(SourceRange, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagnosticGuidance {
    ChangeAnnotation {
        range: SourceRange,
        ty: crate::types::TypeId,
    },
    SupplyAssignableValue {
        expected: crate::types::TypeId,
    },
    UseCallableShape {
        callable: crate::identity::CallableId,
    },
    EstablishTypeEvidence {
        range: SourceRange,
        expected: Option<crate::types::TypeId>,
    },
    ResolveGenericParameter {
        parameter: crate::types::TypeParameterId,
    },
}

impl DiagnosticFix {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            replacement: None,
        }
    }

    pub fn replacement(message: impl Into<String>, range: SourceRange, text: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            replacement: Some((range, text.into())),
        }
    }
}

/// A diagnostic-owned reference into one callable-local explanation arena.
///
/// Explanation IDs are intentionally only meaningful within the callable that
/// allocated them. Qualifying the local ID prevents an aggregated snapshot
/// from accidentally resolving an explanation against the wrong arena.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExplanationRef {
    pub callable: crate::identity::CallableId,
    pub explanation: crate::identity::ExplanationId,
}

impl ExplanationRef {
    pub fn new(callable: crate::identity::CallableId, explanation: crate::identity::ExplanationId) -> Self {
        Self { callable, explanation }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticDiagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub primary: SemanticSourceSpan,
    pub primary_range: SourceRange,
    pub labels: Vec<DiagnosticLabel>,
    pub notes: Vec<String>,
    pub helps: Vec<String>,
    pub guidance: Vec<DiagnosticGuidance>,
    pub explanations: Vec<ExplanationRef>,
    pub fixes: Vec<DiagnosticFix>,
    pub root_cause: Option<crate::identity::DiagnosticCauseId>,
}

impl SemanticDiagnostic {
    /// Creates an error diagnostic owned by `module`.
    pub fn error_in(module: ModuleId, code: DiagnosticCode, message: impl Into<String>, primary_range: SourceRange) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            primary: SemanticSourceSpan { module, range: primary_range },
            primary_range,
            labels: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            guidance: Vec::new(),
            explanations: Vec::new(),
            fixes: Vec::new(),
            root_cause: None,
        }
    }

    /// Creates a warning diagnostic owned by `module`.
    pub fn warning_in(module: ModuleId, code: DiagnosticCode, message: impl Into<String>, primary_range: SourceRange) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            primary: SemanticSourceSpan { module, range: primary_range },
            primary_range,
            labels: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            guidance: Vec::new(),
            explanations: Vec::new(),
            fixes: Vec::new(),
            root_cause: None,
        }
    }

    pub fn data_used_as_superclass(data_decl: crate::identity::DeclarationId, range: SourceRange) -> Self {
        Self::error_in(
            data_decl.module.clone(),
            DiagnosticCode::DataUsedAsSuperclass,
            format!("data declaration `{}` cannot be used as a superclass", data_decl.name),
            range,
        )
    }

    pub fn info_in(module: ModuleId, code: DiagnosticCode, message: impl Into<String>, primary_range: SourceRange) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Information,
            message: message.into(),
            primary: SemanticSourceSpan { module, range: primary_range },
            primary_range,
            labels: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            guidance: Vec::new(),
            explanations: Vec::new(),
            fixes: Vec::new(),
            root_cause: None,
        }
    }

    pub fn with_label(mut self, range: SourceRange, message: impl Into<String>) -> Self {
        let module = self.primary.module.clone();
        self.labels.push(DiagnosticLabel {
            span: SemanticSourceSpan { module, range },
            range,
            message: message.into(),
        });
        self
    }

    pub fn with_label_in(mut self, module: ModuleId, range: SourceRange, message: impl Into<String>) -> Self {
        self.labels.push(DiagnosticLabel {
            span: SemanticSourceSpan { module, range },
            range,
            message: message.into(),
        });
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.helps.push(help.into());
        self
    }

    pub fn with_guidance(mut self, guidance: DiagnosticGuidance) -> Self {
        self.guidance.push(guidance);
        self
    }

    pub fn with_explanation(mut self, explanation: ExplanationRef) -> Self {
        self.explanations.push(explanation);
        self
    }

    pub fn with_fix(mut self, fix: DiagnosticFix) -> Self {
        self.fixes.push(fix);
        self
    }

    pub fn with_root_cause(mut self, cause: crate::identity::DiagnosticCauseId) -> Self {
        self.root_cause = Some(cause);
        self
    }

    /// Renders this diagnostic to a formatted ANSI snippet if source text is available.
    pub fn render(&self, source_text: Option<&str>, path: Option<&str>) -> String {
        use phalcom_diagnostics::snippet::{Label, LabelKind, Snippet};
        use phalcom_diagnostics::style::RenderConfig;

        if let Some(src) = source_text {
            let snippet = if let Some(p) = path { Snippet::with_file(p) } else { Snippet::new() };

            let mut labels = vec![Label {
                span: self.primary_range,
                text: &self.message,
                kind: LabelKind::Primary,
            }];

            for label in self.labels.iter().filter(|label| label.span.module == self.primary.module) {
                labels.push(Label {
                    span: label.range,
                    text: &label.message,
                    kind: LabelKind::Secondary,
                });
            }

            let config = RenderConfig::default();
            snippet.render(src, &labels, &config)
        } else {
            format!("[{}]: {}", self.code.as_str(), self.message)
        }
    }
}

impl crate::snapshot::SemanticSnapshot {
    /// Resolves a callable-qualified explanation reference within this exact
    /// immutable snapshot.
    pub fn explanation_node(&self, reference: &ExplanationRef) -> Option<&crate::explain::ExplanationNode> {
        self.callable_analyses.get(&reference.callable)?.explanations.get(reference.explanation)
    }

    /// Returns the callable-local explanation arena owned by this snapshot.
    pub fn explanation_arena(&self, callable: &crate::identity::CallableId) -> Option<&crate::explain::ExplanationArena> {
        Some(self.callable_analyses.get(callable)?.explanations.as_ref())
    }
}
