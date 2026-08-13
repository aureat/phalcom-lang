//! AST borrowing helpers for surface references.

use super::snapshot::FileSourceSnapshot;
use super::surface::MemberAstRef;
use phalcom_ast::ast::{ClassMember, Expr, Statement};

/// Returns the raw AST member for a given surface reference.
pub fn member_ast(source: &FileSourceSnapshot, ast_ref: MemberAstRef) -> Option<&ClassMember> {
    let Statement::Class(class) = source.program.statements.get(ast_ref.class_stmt_idx)? else {
        return None;
    };
    class.members.get(ast_ref.member_idx)
}

/// Returns the body statements for a member, if it has a body.
pub fn member_body(source: &FileSourceSnapshot, ast_ref: MemberAstRef) -> &[Statement] {
    match member_ast(source, ast_ref) {
        None => &[],
        Some(ClassMember::Method(m)) => &m.body,
        Some(ClassMember::Getter(g)) => &g.body,
        Some(ClassMember::Setter(s)) => &s.body,
        Some(ClassMember::Index(i)) => &i.body,
        Some(_) => &[],
    }
}

/// Returns the field initializer expression, if any.
pub fn field_initializer(source: &FileSourceSnapshot, ast_ref: MemberAstRef) -> Option<&Expr> {
    match member_ast(source, ast_ref) {
        Some(ClassMember::Field(f)) => f.default.as_ref(),
        _ => None,
    }
}
