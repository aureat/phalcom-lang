pub mod analysis;
pub mod binding;
pub mod body;
pub mod call;
pub mod causal;
pub(crate) mod composition;
pub mod context;
pub(crate) mod control;
pub mod declaration;
pub(crate) mod declaration_signature;
pub mod enum_declaration;
pub mod expected;
pub mod expression;
pub mod field_lifecycle;
pub mod flow;
pub mod incident;
pub mod inference;
pub(crate) mod loop_analysis;
pub mod result;
pub mod statement;
pub mod typed_expr;

pub use analysis::{
    AnalysisStatus, BindingAnalysisIndex, BindingState, BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, ExpressionAnalysisIndex,
    FlowStateSummary,
};
pub use binding::{
    AssumptionBasis, BindingConsistency, BindingContract, BindingContractOrigin, BindingDeclarationResult, BindingReconciliation, BindingSeed,
    BindingWriteResult, reconcile_binding_contract,
};
pub use body::analyze_callable_body;
pub use call::CallCheckResult;
pub use causal::{CausalInvalidity, SuppressionCause};
pub use context::CheckingContext;
pub use declaration::{check_class, check_class_bodies, register_class_surface};
pub use expected::{ExpectationOrigin, ExpectedType};
pub use expression::{analyze_expression, check_expr, check_typed_expr, synthesize_expr, synthesize_typed_expr};
pub use incident::{BindingContractSummary, InternalFailurePolicy, InternalSemanticIncident, InternalSemanticIncidentDetails, InternalSemanticIncidentKind};

pub use flow::FlowState;
pub use inference::{InferenceOutcome, InferenceSession, InferenceSupport, InferenceTerm};
pub use result::TypeCheckReport;

pub use control::StatementControl;
pub use statement::check_statement;
pub use typed_expr::TypedExpression;

use crate::declarations::DeclarationTypeTable;
use crate::identity::ModuleId;
use crate::types::annotation::TypeResolver;
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::Program;

/// Runs semantic type checking on a parsed program.
pub fn check_program(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
    module: ModuleId,
    program: &Program,
) -> TypeCheckReport {
    let mut ctx = CheckingContext::new(store, hierarchy, resolver, declarations, module);

    // Pre-pass: register top-level class surfaces
    for stmt in &program.statements {
        if let phalcom_ast::ast::Statement::Class(class_def) = stmt {
            declaration::register_class_surface(&mut ctx, class_def);
        }
    }

    for stmt in &program.statements {
        check_statement(&mut ctx, stmt);
    }

    TypeCheckReport { diagnostics: ctx.diagnostics }
}
