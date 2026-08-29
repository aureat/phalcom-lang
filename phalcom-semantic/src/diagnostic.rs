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
    TypeDynamicBoundary,
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
            Self::TypeDynamicBoundary => "type.dynamic_boundary",
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
