//! Canonical semantic diagnostics shared between compiler and LSP.

use phalcom_common::range::SourceRange;

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
        }
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticLabel {
    pub range: SourceRange,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticDiagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub primary_range: SourceRange,
    pub labels: Vec<DiagnosticLabel>,
}

impl SemanticDiagnostic {
    pub fn error(code: DiagnosticCode, message: impl Into<String>, primary_range: SourceRange) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            primary_range,
            labels: Vec::new(),
        }
    }

    pub fn with_label(mut self, range: SourceRange, message: impl Into<String>) -> Self {
        self.labels.push(DiagnosticLabel {
            range,
            message: message.into(),
        });
        self
    }
}
