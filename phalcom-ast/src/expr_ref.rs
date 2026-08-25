//! Borrowing compatibility for AST expressions.
//!
//! `Expr` is a value enum rather than a boxed node at several AST boundaries.
//! Implementing `AsRef<Expr>` lets generic borrowing code inspect an expression
//! without depending on whether its owner stores it directly or behind a box.

use crate::ast::Expr;

impl AsRef<Expr> for Expr {
    #[inline]
    fn as_ref(&self) -> &Expr {
        self
    }
}
