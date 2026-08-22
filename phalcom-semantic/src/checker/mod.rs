//! Semantic type checking pipeline for Phalcom programs.

pub mod call;
pub mod context;
pub mod declaration;
pub mod expression;
pub mod result;
pub mod statement;

pub use call::check_arguments;
pub use context::CheckingContext;
pub use declaration::check_class;
pub use expression::synthesize_expr;
pub use result::TypeCheckReport;
pub use statement::check_statement;

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
    module: ModuleId,
    program: &Program,
) -> TypeCheckReport {
    let mut ctx = CheckingContext::new(store, hierarchy, resolver, module);

    for stmt in &program.statements {
        check_statement(&mut ctx, stmt);
    }

    TypeCheckReport {
        diagnostics: ctx.diagnostics,
    }
}
