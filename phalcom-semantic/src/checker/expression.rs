//! Expression type synthesis and message-based inference engine.

use super::call::match_callable_arguments;
use super::context::CheckingContext;
use super::statement::check_statement;
use super::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::DispatchResult;
use crate::types::evidence::{DynamicReason, EvidenceAuthority, TypeKnowledge, UnknownReason};
use crate::types::relation::{Assignability, check_assignability};
use crate::types::store::{RecordTypeField, TupleTypeElement, TypeData};
use phalcom_ast::ast::{
    BinaryExpr, BinaryOp, Expr, GetPropertyExpr, IndexExpr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, MethodCallExpr, PackItem, PackLabel, Pattern,
    ProductLabel, RecordLiteralEntry, SetIndexExpr, SetLiteralEntry, SetPropertyExpr, Statement, SymbolExpr, SymbolLiteralKind, TupleLiteralEntry, UnaryExpr,
    UnaryOp, UnqualifiedCallExpr,
};
use phalcom_common::selector::{Selector, SelectorSlot};

/// Synthesizes epistemic type knowledge for an expression.
pub fn synthesize_expr(ctx: &mut CheckingContext<'_>, expr: &Expr) -> TypeKnowledge {
    synthesize_typed_expr(ctx, expr).knowledge
}

/// Synthesizes a full [`TypedExpression`] with type knowledge, constraints, and provenance.
pub fn synthesize_typed_expr(ctx: &mut CheckingContext<'_>, expr: &Expr) -> TypedExpression {
    match expr {
        // --- 1. Primitive Literals ---
        Expr::Int { range, .. } => {
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Int", &[]) {
                let ty = ctx.store.nominal(decl);
                TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Float { range, .. } => {
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Float", &[]) {
                let ty = ctx.store.nominal(decl);
                TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::String { range, .. } => {
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "String", &[]) {
                let ty = ctx.store.nominal(decl);
                TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Boolean { range, .. } => {
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Bool", &[]) {
                let ty = ctx.store.nominal(decl);
                TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Symbol(s) => synthesize_symbol_expr(ctx, s),

        // --- 2. Variables and Identifiers ---
        Expr::Var { value, range } => {
            if let Some(k) = ctx.lookup_local(value) {
                TypedExpression::new(k.clone().with_range(*range))
            } else if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, value, &[]) {
                let ty = ctx.store.nominal(decl);
                TypedExpression::known(ty, EvidenceAuthority::Declared, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnresolvedName(value.as_str().into()))
            }
        }
        Expr::SelfVar { range } => {
            if let Some(ref class_decl) = ctx.current_class {
                let ty = ctx.store.nominal(class_decl.clone());
                TypedExpression::known(ty, EvidenceAuthority::Proven, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::SuperVar { range } => {
            if let Some(ref class_decl) = ctx.current_class {
                if let Some(super_decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Object", &[]) {
                    let ty = ctx.store.nominal(super_decl);
                    TypedExpression::known(ty, EvidenceAuthority::Proven, *range)
                } else {
                    let ty = ctx.store.nominal(class_decl.clone());
                    TypedExpression::known(ty, EvidenceAuthority::Proven, *range)
                }
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Field { value, range, .. } => {
            if let Some(ref class_decl) = ctx.current_class {
                if let Some(surface) = ctx.dispatch.get_surface(class_decl) {
                    if let Some(field_k) = surface.get_field(value) {
                        return TypedExpression::new(field_k.clone().with_range(*range));
                    }
                }
            }
            TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
        }

        // --- 3. Assignments ---
        Expr::Assignment(assign) => {
            let val_k = synthesize_expr(ctx, &assign.value);
            if let Expr::Var { value: var_name, .. } = &*assign.name {
                if let Some(target_k) = ctx.lookup_local(var_name).cloned() {
                    let assignability = check_assignability(ctx.store, ctx.hierarchy, &val_k, &target_k);
                    if let Assignability::Refuted { .. } = assignability {
                        ctx.diagnostics.push(SemanticDiagnostic::error(
                            DiagnosticCode::AssignmentMismatch,
                            format!("assigned value is not assignable to `{}`", var_name),
                            assign.range,
                        ));
                    }
                }
            }
            TypedExpression::new(val_k)
        }

        // --- 4. Collections and Product Types ---
        Expr::ListLiteral(list) => synthesize_list_literal(ctx, list),
        Expr::SetLiteral(set) => synthesize_set_literal(ctx, set),
        Expr::MapLiteral(map) => synthesize_map_literal(ctx, map),
        Expr::TupleLiteral(tup) => synthesize_tuple_literal(ctx, tup),
        Expr::RecordLiteral(rec) => synthesize_record_literal(ctx, rec),

        // --- 5. Blocks and Control Flow ---
        Expr::Block(block) => {
            ctx.push_scope();
            let mut tail_k = TypeKnowledge::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax);
            let len = block.body.len();
            for (i, stmt) in block.body.iter().enumerate() {
                if i == len - 1 {
                    match stmt {
                        Statement::Expr { expr, .. } => {
                            tail_k = synthesize_expr(ctx, expr);
                        }
                        Statement::Throw { expr, .. } => {
                            synthesize_expr(ctx, expr);
                            tail_k = TypeKnowledge::known(ctx.store.never(), EvidenceAuthority::ExactSyntax);
                        }
                        _ => {
                            check_statement(ctx, stmt);
                        }
                    }
                } else {
                    check_statement(ctx, stmt);
                }
            }
            ctx.pop_scope();
            TypedExpression::new(tail_k.with_range(block.range))
        }
        Expr::IfLet(if_let) => {
            let val_k = synthesize_expr(ctx, &if_let.value);
            ctx.push_scope();
            bind_pattern(ctx, &if_let.pattern, val_k.clone());
            let then_k = synthesize_expr(ctx, &Expr::Block(Box::new(if_let.then_body.clone())));
            ctx.pop_scope();

            let else_k = if let Some(ref else_body) = if_let.else_body {
                ctx.push_scope();
                let k = synthesize_expr(ctx, &Expr::Block(Box::new(else_body.clone())));
                ctx.pop_scope();
                k
            } else {
                TypeKnowledge::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax)
            };

            let combined_ty = match (then_k.ty(), else_k.ty()) {
                (Some(t1), Some(t2)) => ctx.store.union(&[t1, t2]),
                (Some(t1), None) => t1,
                (None, Some(t2)) => t2,
                _ => ctx.store.unit(),
            };
            TypedExpression::known(combined_ty, EvidenceAuthority::Proven, if_let.range)
        }
        Expr::WhileLet(while_let) => {
            let val_k = synthesize_expr(ctx, &while_let.value);
            ctx.push_scope();
            bind_pattern(ctx, &while_let.pattern, val_k);
            for stmt in &while_let.body {
                check_statement(ctx, stmt);
            }
            ctx.pop_scope();
            TypedExpression::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax, while_let.range)
        }

        // --- 6. Message Sends and Invocations ---
        Expr::MethodCall(call) => synthesize_method_call(ctx, call),
        Expr::UnqualifiedCall(call) => synthesize_unqualified_call(ctx, call),
        Expr::Binary(binary) => synthesize_binary_expr(ctx, binary),
        Expr::Unary(unary) => synthesize_unary_expr(ctx, unary),

        // --- 7. Member and Subscript Access ---
        Expr::GetProperty(get) => synthesize_get_property(ctx, get),
        Expr::SetProperty(set) => synthesize_set_property(ctx, set),
        Expr::Index(idx) => synthesize_index_expr(ctx, idx),
        Expr::SetIndex(set_idx) => synthesize_set_index_expr(ctx, set_idx),

        // --- 8. Miscellaneous Expressions ---
        Expr::ComparisonChain(chain) => {
            for op in &chain.operands {
                synthesize_expr(ctx, op);
            }
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Bool", &[]) {
                let ty = ctx.store.nominal(decl);
                TypedExpression::known(ty, EvidenceAuthority::Proven, chain.range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Membership(m) => {
            synthesize_expr(ctx, &m.left);
            synthesize_expr(ctx, &m.right);
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Bool", &[]) {
                let ty = ctx.store.nominal(decl);
                TypedExpression::known(ty, EvidenceAuthority::Proven, m.range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::IsMembership(m) => {
            synthesize_expr(ctx, &m.left);
            synthesize_expr(ctx, &m.candidates);
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Bool", &[]) {
                let ty = ctx.store.nominal(decl);
                TypedExpression::known(ty, EvidenceAuthority::Proven, m.range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Range(r) => {
            if let Some(ref lower) = r.lower {
                synthesize_expr(ctx, lower);
            }
            if let Some(ref upper) = r.upper {
                synthesize_expr(ctx, upper);
            }
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Object", &[]) {
                let ty = ctx.store.nominal(decl);
                TypedExpression::known(ty, EvidenceAuthority::Proven, r.range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Ellipsis { range } => TypedExpression::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax, *range),
        _ => TypedExpression::unknown(UnknownReason::UncheckedExpression),
    }
}

// ---------------------------------------------------------------------------
// Helpers for Complex Expressions
// ---------------------------------------------------------------------------

fn synthesize_symbol_expr(ctx: &mut CheckingContext<'_>, s: &SymbolExpr) -> TypedExpression {
    if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Symbol", &[]) {
        let ty = ctx.store.nominal(decl);
        TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, s.range)
    } else if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "String", &[]) {
        let ty = ctx.store.nominal(decl);
        TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, s.range)
    } else {
        TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
    }
}

fn synthesize_list_literal(ctx: &mut CheckingContext<'_>, list: &phalcom_ast::ast::ListLiteralExpr) -> TypedExpression {
    let list_decl = ctx.resolver.resolve_type_name(&ctx.current_module, "List", &[]);
    let mut elem_tys = Vec::new();

    for el in &list.elements {
        match el {
            ListLiteralElement::Element { expr, .. } => {
                let k = synthesize_expr(ctx, expr);
                if let Some(ty) = k.ty() {
                    elem_tys.push(ty);
                }
            }
            ListLiteralElement::Expansion { expr, .. } => {
                synthesize_expr(ctx, expr);
            }
        }
    }

    let elem_ty = if elem_tys.is_empty() {
        let (_var, infer_ty) = ctx.solver.fresh_var(ctx.store);
        infer_ty
    } else {
        ctx.store.union(&elem_tys)
    };

    if let Some(decl) = list_decl {
        let ty = ctx.store.list_of(decl, elem_ty);
        TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, list.range)
    } else {
        TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
    }
}

fn synthesize_set_literal(ctx: &mut CheckingContext<'_>, set: &phalcom_ast::ast::SetLiteralExpr) -> TypedExpression {
    let set_decl = ctx.resolver.resolve_type_name(&ctx.current_module, "Set", &[]);
    let mut elem_tys = Vec::new();

    for el in &set.entries {
        match el {
            SetLiteralEntry::Element { expr, .. } => {
                let k = synthesize_expr(ctx, expr);
                if let Some(ty) = k.ty() {
                    elem_tys.push(ty);
                }
            }
            SetLiteralEntry::Expansion { expr, .. } => {
                synthesize_expr(ctx, expr);
            }
        }
    }

    let elem_ty = if elem_tys.is_empty() {
        let (_var, infer_ty) = ctx.solver.fresh_var(ctx.store);
        infer_ty
    } else {
        ctx.store.union(&elem_tys)
    };

    if let Some(decl) = set_decl {
        let ty = ctx.store.set_of(decl, elem_ty);
        TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, set.range)
    } else {
        TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
    }
}

fn synthesize_map_literal(ctx: &mut CheckingContext<'_>, map: &phalcom_ast::ast::MapLiteralExpr) -> TypedExpression {
    let map_decl = ctx.resolver.resolve_type_name(&ctx.current_module, "Map", &[]);
    let string_decl = ctx.resolver.resolve_type_name(&ctx.current_module, "String", &[]);
    let mut key_tys = Vec::new();
    let mut val_tys = Vec::new();

    for entry in &map.entries {
        match entry {
            MapLiteralEntry::Association { key, value, .. } => {
                let key_ty = match key {
                    MapLiteralKey::BareSymbol { .. } => string_decl.as_ref().map(|d| ctx.store.nominal(d.clone())),
                    MapLiteralKey::Computed { expr, .. } => synthesize_expr(ctx, expr).ty(),
                };
                if let Some(kt) = key_ty {
                    key_tys.push(kt);
                }
                let val_k = synthesize_expr(ctx, value);
                if let Some(vt) = val_k.ty() {
                    val_tys.push(vt);
                }
            }
            MapLiteralEntry::Expansion { expr, .. } => {
                synthesize_expr(ctx, expr);
            }
        }
    }

    let key_ty = if key_tys.is_empty() {
        let (_var, infer_ty) = ctx.solver.fresh_var(ctx.store);
        infer_ty
    } else {
        ctx.store.union(&key_tys)
    };

    let val_ty = if val_tys.is_empty() {
        let (_var, infer_ty) = ctx.solver.fresh_var(ctx.store);
        infer_ty
    } else {
        ctx.store.union(&val_tys)
    };

    if let Some(decl) = map_decl {
        let ty = ctx.store.map_of(decl, key_ty, val_ty);
        TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, map.range)
    } else {
        TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
    }
}

fn synthesize_tuple_literal(ctx: &mut CheckingContext<'_>, tup: &phalcom_ast::ast::TupleLiteralExpr) -> TypedExpression {
    let mut elements = Vec::new();

    for entry in &tup.entries {
        match entry {
            TupleLiteralEntry::Positional { expr, .. } => {
                let k = synthesize_expr(ctx, expr);
                let ty = k.ty().unwrap_or_else(|| ctx.store.unit());
                elements.push(TupleTypeElement { label: None, ty });
            }
            TupleLiteralEntry::Labeled { label, value, .. } => {
                let k = synthesize_expr(ctx, value);
                let ty = k.ty().unwrap_or_else(|| ctx.store.unit());
                let label_name = match label {
                    ProductLabel::Static { symbol, .. } => match symbol {
                        SymbolLiteralKind::Name(n) => Some(n.clone().into_boxed_str()),
                        SymbolLiteralKind::Selector { name, .. } => Some(name.clone().into_boxed_str()),
                        _ => None,
                    },
                    _ => None,
                };
                elements.push(TupleTypeElement { label: label_name, ty });
            }
            TupleLiteralEntry::Expand { expr, .. } => {
                synthesize_expr(ctx, expr);
            }
        }
    }

    let ty = ctx.store.tuple(elements.into_boxed_slice());
    TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, tup.range)
}

fn synthesize_record_literal(ctx: &mut CheckingContext<'_>, rec: &phalcom_ast::ast::RecordLiteralExpr) -> TypedExpression {
    let mut fields = Vec::new();

    for entry in &rec.entries {
        match entry {
            RecordLiteralEntry::Field(f) => {
                let k = synthesize_expr(ctx, &f.value);
                let ty = k.ty().unwrap_or_else(|| ctx.store.unit());
                let name = match &f.label {
                    ProductLabel::Static { symbol, .. } => match symbol {
                        SymbolLiteralKind::Name(n) => n.clone(),
                        SymbolLiteralKind::Selector { name, .. } => name.clone(),
                        _ => "field".into(),
                    },
                    _ => "field".into(),
                };
                fields.push(RecordTypeField {
                    name: name.into_boxed_str(),
                    ty,
                });
            }
            RecordLiteralEntry::Expansion { expr, .. } => {
                synthesize_expr(ctx, expr);
            }
        }
    }

    let ty = ctx.store.record(fields.into_boxed_slice());
    TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, rec.range)
}

// ---------------------------------------------------------------------------
// Message Send and Invocation Synthesis
// ---------------------------------------------------------------------------

fn synthesize_method_call(ctx: &mut CheckingContext<'_>, call: &MethodCallExpr) -> TypedExpression {
    let recv_k = synthesize_expr(ctx, &call.object);

    // Build selector from method name + pack items
    let mut slots = Vec::new();
    for arg in &call.args {
        match arg {
            PackItem::Positional { .. } => slots.push(SelectorSlot::Positional),
            PackItem::Labeled { label, .. } => {
                if let PackLabel::Static { text, .. } = label {
                    slots.push(SelectorSlot::Label(text.clone()));
                } else {
                    slots.push(SelectorSlot::Positional);
                }
            }
            PackItem::Expand { .. } => slots.push(SelectorSlot::Positional),
        }
    }

    let Ok(sel) = Selector::method(&call.method, slots) else {
        return TypedExpression::unknown(UnknownReason::DynamicMessageSend);
    };

    if let Some(recv_ty) = recv_k.ty() {
        let dispatch_res = ctx.resolve_dispatch(recv_ty, &sel);
        match dispatch_res {
            DispatchResult::Found(sig) => {
                let ret_k = match_callable_arguments(ctx, &call.args, &sig, call.range);

                // Check generic collection constraint inference (e.g. List<T>.add(x))
                if call.method == "add" && call.args.len() == 1 {
                    if let TypeData::Applied { origin, arguments } = ctx.store.get(recv_ty).clone() {
                        if let Some(list_decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "List", &[]) {
                            if origin == ctx.store.nominal(list_decl) && arguments.len() == 1 {
                                let elem_var = arguments[0];
                                let maybe_var = if let TypeData::Infer(var) = ctx.store.get(elem_var) {
                                    Some(*var)
                                } else {
                                    None
                                };
                                if let Some(var) = maybe_var {
                                    if let PackItem::Positional { expr: arg_expr, .. } = &call.args[0] {
                                        let arg_k = synthesize_expr(ctx, arg_expr);
                                        if let Some(arg_ty) = arg_k.ty() {
                                            ctx.solver.bind(var, arg_ty);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                return TypedExpression::new(ret_k);
            }
            DispatchResult::Dynamic => {
                return TypedExpression::dynamic(DynamicReason::RuntimeReflection);
            }
            _ => {}
        }
    }

    if recv_k.is_dynamic() {
        TypedExpression::dynamic(DynamicReason::RuntimeReflection)
    } else {
        TypedExpression::unknown(UnknownReason::DynamicMessageSend)
    }
}

fn synthesize_unqualified_call(ctx: &mut CheckingContext<'_>, call: &UnqualifiedCallExpr) -> TypedExpression {
    // 1. Local callable variable lookup
    if let Some(local_k) = ctx.lookup_local(&call.name).cloned() {
        if let Some(ty) = local_k.ty() {
            if let TypeData::Callable(c) = ctx.store.get(ty).clone() {
                return TypedExpression::known(c.return_type, EvidenceAuthority::Proven, call.range);
            }
        }
        return TypedExpression::new(local_k);
    }

    // 2. Dispatch send on `self` if inside a class
    if let Some(ref class_decl) = ctx.current_class.clone() {
        let class_ty = ctx.store.nominal(class_decl.clone());
        let mut slots = Vec::new();
        for arg in &call.args {
            match arg {
                PackItem::Positional { .. } => slots.push(SelectorSlot::Positional),
                PackItem::Labeled { label, .. } => {
                    if let PackLabel::Static { text, .. } = label {
                        slots.push(SelectorSlot::Label(text.clone()));
                    } else {
                        slots.push(SelectorSlot::Positional);
                    }
                }
                PackItem::Expand { .. } => slots.push(SelectorSlot::Positional),
            }
        }
        if let Ok(sel) = Selector::method(&call.name, slots) {
            let dispatch_res = ctx.resolve_dispatch(class_ty, &sel);
            if let DispatchResult::Found(sig) = dispatch_res {
                let ret_k = match_callable_arguments(ctx, &call.args, &sig, call.range);
                return TypedExpression::new(ret_k);
            }
        }
    }

    // 3. Constructor or nominal reference
    if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, &call.name, &[]) {
        let ty = ctx.store.nominal(decl);
        return TypedExpression::known(ty, EvidenceAuthority::Declared, call.range);
    }

    TypedExpression::unknown(UnknownReason::UnresolvedName(call.name.as_str().into()))
}

fn synthesize_binary_expr(ctx: &mut CheckingContext<'_>, binary: &BinaryExpr) -> TypedExpression {
    let left_k = synthesize_expr(ctx, &binary.left);
    let right_k = synthesize_expr(ctx, &binary.right);

    let op_name = match binary.op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::IntegerDivide => "//",
        BinaryOp::Modulo => "%",
        BinaryOp::Power => "**",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::Equal => "==",
        BinaryOp::Same => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanOrEqual => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanOrEqual => ">=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::Compare => "<=>",
    };

    if let Ok(sel) = Selector::method(op_name, vec![SelectorSlot::Positional]) {
        if let Some(left_ty) = left_k.ty() {
            let dispatch_res = ctx.resolve_dispatch(left_ty, &sel);
            if let DispatchResult::Found(sig) = dispatch_res {
                return TypedExpression::new(sig.return_type.with_range(binary.range));
            }
        }
    }

    if left_k.is_dynamic() || right_k.is_dynamic() {
        TypedExpression::dynamic(DynamicReason::RuntimeReflection)
    } else {
        TypedExpression::unknown(UnknownReason::DynamicMessageSend)
    }
}

fn synthesize_unary_expr(ctx: &mut CheckingContext<'_>, unary: &UnaryExpr) -> TypedExpression {
    let operand_k = synthesize_expr(ctx, &unary.expr);

    let op_name = match unary.op {
        UnaryOp::Plus => "+",
        UnaryOp::Minus => "-",
        UnaryOp::Not => "not",
        UnaryOp::BitNot => "~",
    };

    if let Ok(sel) = Selector::getter(op_name) {
        if let Some(operand_ty) = operand_k.ty() {
            let dispatch_res = ctx.resolve_dispatch(operand_ty, &sel);
            if let DispatchResult::Found(sig) = dispatch_res {
                return TypedExpression::new(sig.return_type.with_range(unary.range));
            }
        }
    }

    if operand_k.is_dynamic() {
        TypedExpression::dynamic(DynamicReason::RuntimeReflection)
    } else {
        TypedExpression::unknown(UnknownReason::DynamicMessageSend)
    }
}

fn synthesize_get_property(ctx: &mut CheckingContext<'_>, get: &GetPropertyExpr) -> TypedExpression {
    let recv_k = synthesize_expr(ctx, &get.object);

    if let Some(recv_ty) = recv_k.ty() {
        // 1. Check Field on class surface
        if let TypeData::Nominal { ref declaration } = ctx.store.get(recv_ty).clone() {
            if let Some(surface) = ctx.dispatch.get_surface(declaration) {
                if let Some(field_k) = surface.get_field(&get.property) {
                    return TypedExpression::new(field_k.clone().with_range(get.range));
                }
            }
        }

        // 2. Check Getter selector
        if let Ok(sel) = Selector::getter(&get.property) {
            let dispatch_res = ctx.resolve_dispatch(recv_ty, &sel);
            if let DispatchResult::Found(sig) = dispatch_res {
                return TypedExpression::new(sig.return_type.with_range(get.range));
            }
        }
    }

    if recv_k.is_dynamic() {
        TypedExpression::dynamic(DynamicReason::RuntimeReflection)
    } else {
        TypedExpression::unknown(UnknownReason::DynamicMessageSend)
    }
}

fn synthesize_set_property(ctx: &mut CheckingContext<'_>, set: &SetPropertyExpr) -> TypedExpression {
    let recv_k = synthesize_expr(ctx, &set.object);
    let val_k = synthesize_expr(ctx, &set.value);

    if let Some(recv_ty) = recv_k.ty() {
        // 1. Check field
        if let TypeData::Nominal { ref declaration } = ctx.store.get(recv_ty).clone() {
            if let Some(surface) = ctx.dispatch.get_surface(declaration) {
                if let Some(field_k) = surface.get_field(&set.property) {
                    let assignability = check_assignability(ctx.store, ctx.hierarchy, &val_k, field_k);
                    if let Assignability::Refuted { .. } = assignability {
                        ctx.diagnostics.push(SemanticDiagnostic::error(
                            DiagnosticCode::FieldMismatch,
                            format!("assigned value does not match field `{}` type", set.property),
                            set.range,
                        ));
                    }
                    return TypedExpression::new(val_k);
                }
            }
        }

        // 2. Check setter selector
        if let Ok(sel) = Selector::setter(&set.property) {
            let dispatch_res = ctx.resolve_dispatch(recv_ty, &sel);
            if let DispatchResult::Found(sig) = dispatch_res {
                if let Some(param) = sig.parameters.first() {
                    let assignability = check_assignability(ctx.store, ctx.hierarchy, &val_k, &param.ty);
                    if let Assignability::Refuted { .. } = assignability {
                        ctx.diagnostics.push(SemanticDiagnostic::error(
                            DiagnosticCode::AssignmentMismatch,
                            format!("assigned value does not match setter `{}=` parameter type", set.property),
                            set.range,
                        ));
                    }
                }
                return TypedExpression::new(val_k);
            }
        }
    }

    TypedExpression::new(val_k)
}

fn synthesize_index_expr(ctx: &mut CheckingContext<'_>, idx: &IndexExpr) -> TypedExpression {
    let recv_k = synthesize_expr(ctx, &idx.object);

    for arg in &idx.args {
        match arg {
            PackItem::Positional { expr, .. } => {
                synthesize_expr(ctx, expr);
            }
            PackItem::Labeled { value, .. } => {
                synthesize_expr(ctx, value);
            }
            PackItem::Expand { expr, .. } => {
                synthesize_expr(ctx, expr);
            }
        }
    }

    if let Some(recv_ty) = recv_k.ty() {
        // Direct generic List/Map indexing
        if let TypeData::Applied { origin, arguments } = ctx.store.get(recv_ty).clone() {
            if let Some(list_decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "List", &[]) {
                if origin == ctx.store.nominal(list_decl) && arguments.len() == 1 {
                    let elem_ty = ctx.solver.substitute_type(arguments[0], ctx.store);
                    return TypedExpression::known(elem_ty, EvidenceAuthority::Proven, idx.range);
                }
            }
            if let Some(map_decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Map", &[]) {
                if origin == ctx.store.nominal(map_decl) && arguments.len() == 2 {
                    let val_ty = ctx.solver.substitute_type(arguments[1], ctx.store);
                    return TypedExpression::known(val_ty, EvidenceAuthority::Proven, idx.range);
                }
            }
        }

        // Subscript selector dispatch
        let mut slots = Vec::new();
        for arg in &idx.args {
            match arg {
                PackItem::Positional { .. } => slots.push(SelectorSlot::Positional),
                PackItem::Labeled { label, .. } => {
                    if let PackLabel::Static { text, .. } = label {
                        slots.push(SelectorSlot::Label(text.clone()));
                    } else {
                        slots.push(SelectorSlot::Positional);
                    }
                }
                PackItem::Expand { .. } => slots.push(SelectorSlot::Positional),
            }
        }
        if let Ok(sel) = Selector::subscript_get(slots) {
            let dispatch_res = ctx.resolve_dispatch(recv_ty, &sel);
            if let DispatchResult::Found(sig) = dispatch_res {
                return TypedExpression::new(sig.return_type.with_range(idx.range));
            }
        }
    }

    if recv_k.is_dynamic() {
        TypedExpression::dynamic(DynamicReason::RuntimeReflection)
    } else {
        TypedExpression::unknown(UnknownReason::DynamicMessageSend)
    }
}

fn synthesize_set_index_expr(ctx: &mut CheckingContext<'_>, set_idx: &SetIndexExpr) -> TypedExpression {
    let recv_k = synthesize_expr(ctx, &set_idx.object);
    let val_k = synthesize_expr(ctx, &set_idx.value);

    for arg in &set_idx.args {
        match arg {
            PackItem::Positional { expr, .. } => {
                synthesize_expr(ctx, expr);
            }
            PackItem::Labeled { value, .. } => {
                synthesize_expr(ctx, value);
            }
            PackItem::Expand { expr, .. } => {
                synthesize_expr(ctx, expr);
            }
        }
    }

    if let Some(recv_ty) = recv_k.ty() {
        if let TypeData::Applied { origin, arguments } = ctx.store.get(recv_ty).clone() {
            if let Some(list_decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "List", &[]) {
                if origin == ctx.store.nominal(list_decl) && arguments.len() == 1 {
                    let elem_k = TypeKnowledge::known(arguments[0], EvidenceAuthority::Declared);
                    let assignability = check_assignability(ctx.store, ctx.hierarchy, &val_k, &elem_k);
                    if let Assignability::Refuted { .. } = assignability {
                        ctx.diagnostics.push(SemanticDiagnostic::error(
                            DiagnosticCode::AssignmentMismatch,
                            "value assigned to List index does not match element type",
                            set_idx.range,
                        ));
                    }
                    return TypedExpression::new(val_k);
                }
            }
        }
    }

    TypedExpression::new(val_k)
}

fn bind_pattern(ctx: &mut CheckingContext<'_>, pattern: &Pattern, knowledge: TypeKnowledge) {
    match pattern {
        Pattern::Name { name, .. } => {
            ctx.bind_local(name.clone(), knowledge);
        }
        _ => {}
    }
}
