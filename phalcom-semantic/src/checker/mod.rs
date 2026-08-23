pub mod analysis;
pub mod body;
pub mod call;
pub mod context;
pub mod declaration;
pub mod expected;
pub mod expression;
pub mod flow;
pub mod inference;
pub mod policy;
pub mod result;
pub mod statement;
pub mod typed_expr;

pub use analysis::{
    AnalysisStatus, BindingAnalysisIndex, BindingState, BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, ExpressionAnalysis, ExpressionAnalysisIndex,
    FlowStateSummary,
};
pub use body::analyze_callable_body;
pub use call::{check_arguments, match_callable_arguments, resolve_call};
pub use context::CheckingContext;
pub use declaration::{check_class, check_class_bodies, register_class_surface};
pub use expected::ExpectedType;
pub use expression::{analyze_expression, check_expr, check_typed_expr, synthesize_expr, synthesize_typed_expr};
pub use policy::{enforce_assignability, handle_relation_outcome};

pub use flow::FlowState;
pub use inference::{InferenceOutcome, InferenceSession, InferenceTerm};
pub use result::TypeCheckReport;

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
