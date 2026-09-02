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
    AssignmentToImmutable,
    AssignmentMismatch,
    ReturnMismatch,
    ArgumentMismatch,
    GenericInferenceConflict,
    GenericInferenceUnderconstrained,
    GenericInferenceAmbiguous,
    GenericConstraintUnsatisfied,
    CallShapeMismatch,
    NotCallable,
    FieldMismatch,
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
    ModuleLinkFailed,
    ModuleRuntimeCycle,
    AnalysisBlocked,
    AnalysisBudgetExceeded,
    AnalysisInternalFailure,
    TypeRelationCycle,
    TypeAliasCycle,
    TypeDynamicBoundary,
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
    RecordDuplicateField,
    RecordRowTailUnresolved,
    RecordRowTailKindMismatch,
    RecordRowLacksViolation,
    RecordRowOccursCheck,
    RecordRowInferenceUnderconstrained,
    RecordRowInferenceConflict,
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
}

impl DiagnosticCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BindingInitializerMismatch => "type.binding.initializer_mismatch",
            Self::BindingRedeclared => "binding.redeclared",
            Self::ConstWithoutInitializer => "binding.const_requires_initializer",
            Self::AssignmentToImmutable => "binding.assign_to_immutable",
            Self::AssignmentMismatch => "type.assignment.mismatch",
            Self::ReturnMismatch => "type.return.mismatch",
            Self::ArgumentMismatch => "type.call.argument_mismatch",
            Self::GenericInferenceConflict => "type.generic.inference_conflict",
            Self::GenericInferenceUnderconstrained => "type.generic.underconstrained",
            Self::GenericInferenceAmbiguous => "type.generic.inference_ambiguous",
            Self::GenericConstraintUnsatisfied => "type.generic.constraint_unsatisfied",
            Self::CallShapeMismatch => "type.call.shape_mismatch",
            Self::NotCallable => "type.call.not_callable",
            Self::FieldMismatch => "type.field.mismatch",
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
            Self::ModuleLinkFailed => "module.link.failed",
            Self::ModuleRuntimeCycle => "module.runtime_cycle",
            Self::AnalysisBlocked => "analysis.blocked",
            Self::AnalysisBudgetExceeded => "analysis.budget_exceeded",
            Self::AnalysisInternalFailure => "analysis.internal_failure",
            Self::TypeRelationCycle => "type.relation.cycle",
            Self::TypeAliasCycle => "type.alias.cycle",
            Self::TypeDynamicBoundary => "type.dynamic_boundary",
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
            Self::RecordDuplicateField => "type.record.duplicate_field",
            Self::RecordRowTailUnresolved => "type.record.row_tail_unresolved",
            Self::RecordRowTailKindMismatch => "type.record.row_tail_kind_mismatch",
            Self::RecordRowLacksViolation => "type.record.row_lacks_violation",
            Self::RecordRowOccursCheck => "type.record.row_occurs_check",
            Self::RecordRowInferenceUnderconstrained => "type.record.row_inference_underconstrained",
            Self::RecordRowInferenceConflict => "type.record.row_inference_conflict",
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
