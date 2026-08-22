//! Canonical semantic diagnostics shared between compiler and LSP.

use phalcom_common::range::SourceRange;
use phalcom_modules::identity::ModuleId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticCode {
    BindingInitializerMismatch,
    AssignmentMismatch,
    ReturnMismatch,
    ArgumentMismatch,
    FieldMismatch,
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
            Self::AssignmentMismatch => "type.assignment.mismatch",
            Self::ReturnMismatch => "type.return.mismatch",
            Self::ArgumentMismatch => "type.call.argument_mismatch",
            Self::FieldMismatch => "type.field.mismatch",
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
pub struct SemanticDiagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub primary: SemanticSourceSpan,
    pub primary_range: SourceRange,
    pub labels: Vec<DiagnosticLabel>,
}

impl SemanticDiagnostic {
    pub fn error(code: DiagnosticCode, message: impl Into<String>, primary_range: SourceRange) -> Self {
        Self::error_in(ModuleId::core(), code, message, primary_range)
    }

    pub fn error_in(module: ModuleId, code: DiagnosticCode, message: impl Into<String>, primary_range: SourceRange) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            primary: SemanticSourceSpan { module, range: primary_range },
            primary_range,
            labels: Vec::new(),
        }
    }

    pub fn warning(code: DiagnosticCode, message: impl Into<String>, primary_range: SourceRange) -> Self {
        Self::warning_in(ModuleId::core(), code, message, primary_range)
    }

    pub fn warning_in(module: ModuleId, code: DiagnosticCode, message: impl Into<String>, primary_range: SourceRange) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            primary: SemanticSourceSpan { module, range: primary_range },
            primary_range,
            labels: Vec::new(),
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
}
