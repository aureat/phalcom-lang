//! Expression type synthesis, bidirectional checking, and inference engine (Spec 04.5 / Wave 3).

use super::call::resolve_call;
use super::context::CheckingContext;
use super::expected::ExpectedType;
use super::flow::FlowState;
use super::policy::enforce_assignability;
use super::statement::check_statement;
use super::typed_expr::TypedExpression;
use crate::checker::analysis::AnalysisStatus;
use crate::diagnostic::DiagnosticCode;
use crate::dispatch::{CallableParameter, CallableSignature, DispatchResult};
use crate::types::denotation::{SemanticDenotation, ValueSemanticFact};
use crate::types::evidence::{DynamicReason, EvidenceAuthority, TypeKnowledge, UnknownReason};
use crate::types::id::KindId;
use crate::types::store::{RecordTypeField, TupleTypeElement, TypeData};
use phalcom_ast::ast::{
    BinaryExpr, BinaryOp, Expr, GetPropertyExpr, IndexExpr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, MethodCallExpr, PackItem, PackLabel, Pattern,
    ProductLabel, RecordLiteralEntry, SetIndexExpr, SetLiteralEntry, SetPropertyExpr, Statement, SymbolExpr, SymbolLiteralKind, TupleLiteralEntry, UnaryExpr,
    UnaryOp, UnqualifiedCallExpr,
};
use phalcom_common::selector::{Selector, SelectorSlot};

/// Central entry point for bidirectional expression analysis (Spec 04.5 / E4).
pub fn analyze_expression(ctx: &mut CheckingContext<'_>, expr: &Expr, expected: &ExpectedType) -> TypedExpression {
    let expr_id = ctx.alloc_expression_id();
    let typed = analyze_expression_inner(ctx, expr, expected);

    let status = if let Some(_diag) = ctx
        .diagnostics
        .iter()
        .find(|d| d.primary.range == expr.range() && d.severity == crate::diagnostic::DiagnosticSeverity::Error)
    {
        let cause_id = crate::identity::DiagnosticCauseId(expr_id.local.0);
        ctx.mark_suppressed(expr_id, cause_id);
        AnalysisStatus::Invalid(cause_id)
    } else if typed.knowledge.is_dynamic() {
        AnalysisStatus::DynamicBoundary(DynamicReason::RuntimeReflection)
    } else {
        AnalysisStatus::Ready
    };

    let explanation_id = if let Some(ty) = typed.knowledge.ty() {
        let step = match expr {
            Expr::Int { .. } | Expr::Float { .. } | Expr::String { .. } | Expr::Boolean { .. } => {
                crate::explain::ExplanationStep::Literal { expression: expr_id, ty }
            }
            Expr::MethodCall(call) => {
                let sel = Selector::getter(&call.method)
                    .or_else(|_| Selector::method(&call.method, vec![]))
                    .unwrap_or_else(|_| Selector::getter("unknown").unwrap());
                let callable = crate::identity::CallableId::new(
                    ctx.current_class
                        .clone()
                        .unwrap_or_else(|| crate::identity::DeclarationId::new(ctx.current_module.clone(), "Unknown".into())),
                    sel,
                    ctx.current_side,
                );
                crate::explain::ExplanationStep::MethodCall {
                    call: expr_id,
                    callable,
                    return_ty: ty,
                }
            }
            _ => crate::explain::ExplanationStep::Literal { expression: expr_id, ty },
        };
        let rule = step.derivation_rule();
        let ev = vec![crate::explain::EvidenceRef::SourceSpan(expr.range()), crate::explain::EvidenceRef::TypeId(ty)];
        Some(ctx.record_derivation(step, rule, crate::types::evidence::EvidenceAuthority::Proven, ev, Vec::new()))
    } else {
        None
    };

    let mut analysis = ctx.record_expression(expr_id, expr.range(), typed.knowledge.clone(), typed.denotation, status);
    if let Some(eid) = explanation_id {
        analysis.explanation = Some(eid);
        ctx.expressions.insert(expr_id, analysis);
    }
    typed
}

/// Bidirectionally checks an expression against an expected type.
pub fn check_expr(ctx: &mut CheckingContext<'_>, expr: &Expr, expected: &ExpectedType) -> TypeKnowledge {
    check_typed_expr(ctx, expr, expected).knowledge
}

/// Bidirectionally checks a typed expression against an expected type, recording a TypeMismatch diagnostic on refutation.
pub fn check_typed_expr(ctx: &mut CheckingContext<'_>, expr: &Expr, expected: &ExpectedType) -> TypedExpression {
    let typed = analyze_expression(ctx, expr, expected);
    if let Some(expected_ty) = expected.ty() {
        let expected_k = TypeKnowledge::known(expected_ty, EvidenceAuthority::Declared);
        let _ = enforce_assignability(
            ctx.store,
            &ctx.hierarchy,
            &typed.knowledge,
            &expected_k,
            &ctx.current_module,
            DiagnosticCode::TypeMismatch,
            "expression does not match expected type",
            expr.range(),
            &mut ctx.diagnostics,
        );
    }
    typed
}

/// Synthesizes epistemic type knowledge for an expression.
pub fn synthesize_expr(ctx: &mut CheckingContext<'_>, expr: &Expr) -> TypeKnowledge {
    analyze_expression(ctx, expr, &ExpectedType::None).knowledge
}

/// Synthesizes a full [`TypedExpression`] with type knowledge, constraints, and provenance.
pub fn synthesize_typed_expr(ctx: &mut CheckingContext<'_>, expr: &Expr) -> TypedExpression {
    analyze_expression(ctx, expr, &ExpectedType::None)
}

fn analyze_expression_inner(ctx: &mut CheckingContext<'_>, expr: &Expr, expected: &ExpectedType) -> TypedExpression {
    match expr {
        // --- 1. Primitive Literals ---
        Expr::Int { range, .. } => {
            if let Some(decl) = ctx.resolve_type_name("Int") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Float { range, .. } => {
            if let Some(decl) = ctx.resolve_type_name("Float") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::String { range, .. } => {
            if let Some(decl) = ctx.resolve_type_name("String") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Boolean { range, .. } => {
            if let Some(decl) = ctx.resolve_type_name("Bool") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Symbol(s) => synthesize_symbol_expr(ctx, s),

        // --- 2. Variables and Identifiers ---
        Expr::Var { value, range } => {
            if let Some(fact) = ctx.lookup_local(value) {
                let mut typed = TypedExpression::new(fact.knowledge.clone().with_range(*range));
                typed.denotation = fact.denotation;
                typed
            } else if let Some(decl) = ctx.resolve_type_name(value) {
                if let Some(info) = ctx.declaration_info(&decl) {
                    TypedExpression::known(info.class_object_type, EvidenceAuthority::Declared, *range).with_denotation(SemanticDenotation::TypeForm(info.form))
                } else {
                    let ty = ctx.nominal_type_of(&decl);
                    TypedExpression::known(ty, EvidenceAuthority::Declared, *range)
                }
            } else {
                TypedExpression::unknown(UnknownReason::UnresolvedName(value.as_str().into()))
            }
        }
        Expr::SelfVar { range } => {
            if let Some(class_decl) = ctx.current_class.clone() {
                if ctx.current_side == crate::identity::DispatchSide::Class {
                    if let Some(info) = ctx.declaration_info(&class_decl) {
                        TypedExpression::known(info.class_object_type, EvidenceAuthority::Proven, *range)
                            .with_denotation(SemanticDenotation::TypeForm(info.form))
                    } else {
                        let ty = ctx.nominal_type_of(&class_decl);
                        TypedExpression::known(ty, EvidenceAuthority::Proven, *range)
                    }
                } else {
                    let ty = ctx.nominal_type_of(&class_decl);
                    TypedExpression::known(ty, EvidenceAuthority::Proven, *range)
                }
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::SuperVar { range } => {
            if let Some(class_decl) = ctx.current_class.clone() {
                let side = ctx.current_side;
                let lookup = crate::dispatch::DispatchLookup::Super {
                    defining_class: class_decl.clone(),
                    side,
                };
                if side == crate::identity::DispatchSide::Class {
                    if let Some(info) = ctx.declaration_info(&class_decl) {
                        TypedExpression::known(info.class_object_type, EvidenceAuthority::Proven, *range)
                            .with_denotation(SemanticDenotation::TypeForm(info.form))
                            .with_dispatch_lookup(lookup)
                    } else {
                        let ty = ctx.nominal_type_of(&class_decl);
                        TypedExpression::known(ty, EvidenceAuthority::Proven, *range).with_dispatch_lookup(lookup)
                    }
                } else {
                    let ty = ctx.nominal_type_of(&class_decl);
                    TypedExpression::known(ty, EvidenceAuthority::Proven, *range).with_dispatch_lookup(lookup)
                }
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Field { value, range, .. } => {
            if let Some(class_decl) = ctx.current_class.clone() {
                if let Some(field_k) = ctx.get_field(&class_decl, ctx.current_side, value) {
                    return TypedExpression::new(field_k.with_range(*range));
                }
            }
            TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
        }

        // --- 3. Assignments ---
        Expr::Assignment(assign) => {
            let target_expected = if let Expr::Var { value: var_name, .. } = &*assign.name {
                if let Some(info) = ctx.lookup_binding_info(var_name) {
                    info.declared
                        .map(ExpectedType::Proper)
                        .or_else(|| ctx.flow.get_current_type(info.id).and_then(|k| k.ty()).map(ExpectedType::Proper))
                        .unwrap_or_default()
                } else {
                    ExpectedType::None
                }
            } else {
                ExpectedType::None
            };

            let val_typed = analyze_expression(ctx, &assign.value, &target_expected);
            let val_k = &val_typed.knowledge;
            if let Expr::Var { value: var_name, .. } = &*assign.name {
                if let Some(target_fact) = ctx.lookup_local(var_name).cloned() {
                    enforce_assignability(
                        ctx.store,
                        &ctx.hierarchy,
                        val_k,
                        &target_fact.knowledge,
                        &ctx.current_module,
                        DiagnosticCode::AssignmentMismatch,
                        format!("assigned value is not assignable to `{}`", var_name),
                        assign.range,
                        &mut ctx.diagnostics,
                    );
                }
                ctx.assign_existing(var_name, val_typed.fact());
            }
            TypedExpression::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax, assign.range)
        }

        // --- 4. Collections and Product Types (with Bidirectional Propagation) ---
        Expr::ListLiteral(list) => synthesize_list_literal(ctx, list, expected),
        Expr::SetLiteral(set) => synthesize_set_literal(ctx, set, expected),
        Expr::MapLiteral(map) => synthesize_map_literal(ctx, map, expected),
        Expr::TupleLiteral(tup) => synthesize_tuple_literal(ctx, tup, expected),
        Expr::RecordLiteral(rec) => synthesize_record_literal(ctx, rec, expected),

        // --- 5. Blocks and Control Flow ---
        Expr::Block(block) => {
            ctx.push_scope();
            let (expected_params, expected_ret) = expected.callable_signature(ctx.store).unwrap_or_default();

            let top = ctx
                .resolve_type_name("Object")
                .map(|d| ctx.nominal_type_of(&d))
                .unwrap_or_else(|| ctx.store.unit());

            let mut params = Vec::new();
            for (i, p) in block.params.fixed.iter().enumerate() {
                let p_ty = expected_params.get(i).and_then(|e| e.ty()).unwrap_or(top);
                let p_k = TypeKnowledge::known(p_ty, EvidenceAuthority::ExactSyntax);
                ctx.bind_local(p.name.clone(), ValueSemanticFact::new(p_k), p.range);
                params.push(crate::types::store::CallableParameterType {
                    label: None,
                    ty: p_ty,
                    rest: false,
                });
            }
            if let Some(ref rest_p) = block.params.positional_rest {
                let rest_k = TypeKnowledge::known(top, EvidenceAuthority::ExactSyntax);
                ctx.bind_local(rest_p.name.clone(), ValueSemanticFact::new(rest_k), rest_p.range);
                params.push(crate::types::store::CallableParameterType {
                    label: None,
                    ty: top,
                    rest: true,
                });
            }

            let mut tail_typed = TypedExpression::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax, block.range);
            let len = block.body.len();
            for (i, stmt) in block.body.iter().enumerate() {
                if i == len - 1 {
                    match stmt {
                        Statement::Expr { expr, .. } => {
                            tail_typed = analyze_expression(ctx, expr, &expected_ret);
                        }
                        Statement::Throw { expr, .. } => {
                            analyze_expression(ctx, expr, &ExpectedType::None);
                            tail_typed = TypedExpression::known(ctx.store.never(), EvidenceAuthority::ExactSyntax, block.range);
                        }
                        _ => {
                            check_statement(ctx, stmt);
                            tail_typed = TypedExpression::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax, block.range);
                        }
                    }
                } else {
                    check_statement(ctx, stmt);
                }
            }
            ctx.pop_scope();

            let return_type = tail_typed.knowledge.ty().unwrap_or_else(|| ctx.store.unit());
            let callable_ty = ctx.store.callable(crate::types::store::CallableType {
                parameters: params.into_boxed_slice(),
                return_type,
            });
            TypedExpression::known(callable_ty, EvidenceAuthority::ExactSyntax, block.range)
        }
        Expr::IfLet(if_let) => {
            let val_typed = analyze_expression(ctx, &if_let.value, &ExpectedType::None);
            let flow_before = ctx.flow.clone();

            ctx.push_scope();
            bind_pattern(ctx, &if_let.pattern, val_typed.fact());
            let then_typed = analyze_expression(ctx, &Expr::Block(Box::new(if_let.then_body.clone())), expected);
            let then_flow = ctx.flow.clone();
            ctx.pop_scope();

            let (else_typed, else_flow) = if let Some(ref else_body) = if_let.else_body {
                ctx.flow = flow_before.clone();
                ctx.push_scope();
                let typed = analyze_expression(ctx, &Expr::Block(Box::new(else_body.clone())), expected);
                let f = ctx.flow.clone();
                ctx.pop_scope();
                (typed, f)
            } else {
                (
                    TypedExpression::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax, if_let.range),
                    flow_before,
                )
            };

            ctx.flow = FlowState::join(&[then_flow, else_flow], ctx.store);

            let combined_ty = match (then_typed.knowledge.ty(), else_typed.knowledge.ty()) {
                (Some(t1), Some(t2)) => ctx.store.union(&[t1, t2]),
                (Some(t1), None) => t1,
                (None, Some(t2)) => t2,
                _ => ctx.store.unit(),
            };
            let merged_denotation = match (then_typed.denotation, else_typed.denotation) {
                (Some(d1), Some(d2)) if d1 == d2 => Some(d1),
                _ => None,
            };
            let mut res = TypedExpression::known(combined_ty, EvidenceAuthority::Proven, if_let.range);
            res.denotation = merged_denotation;
            res
        }
        Expr::WhileLet(while_let) => {
            let val_typed = analyze_expression(ctx, &while_let.value, &ExpectedType::None);
            ctx.push_scope();
            bind_pattern(ctx, &while_let.pattern, val_typed.fact());
            for stmt in &while_let.body {
                check_statement(ctx, stmt);
            }
            ctx.pop_scope();
            TypedExpression::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax, while_let.range)
        }

        // --- 6. Message Sends and Invocations (Canonical Resolution E5) ---
        Expr::MethodCall(call) => synthesize_method_call(ctx, call, expected),
        Expr::UnqualifiedCall(call) => synthesize_unqualified_call(ctx, call, expected),
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
                analyze_expression(ctx, op, &ExpectedType::None);
            }
            if let Some(decl) = ctx.resolve_type_name("Bool") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::known(ty, EvidenceAuthority::Proven, chain.range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Membership(m) => {
            analyze_expression(ctx, &m.left, &ExpectedType::None);
            analyze_expression(ctx, &m.right, &ExpectedType::None);
            if let Some(decl) = ctx.resolve_type_name("Bool") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::known(ty, EvidenceAuthority::Proven, m.range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::IsMembership(m) => {
            analyze_expression(ctx, &m.left, &ExpectedType::None);
            analyze_expression(ctx, &m.candidates, &ExpectedType::None);
            if let Some(decl) = ctx.resolve_type_name("Bool") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::known(ty, EvidenceAuthority::Proven, m.range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Range(r) => {
            if let Some(ref lower) = r.lower {
                analyze_expression(ctx, lower, &ExpectedType::None);
            }
            if let Some(ref upper) = r.upper {
                analyze_expression(ctx, upper, &ExpectedType::None);
            }
            if let Some(decl) = ctx.resolve_type_name("Object") {
                let ty = ctx.nominal_type_of(&decl);
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
    if let Some(decl) = ctx.resolve_type_name("Symbol") {
        let ty = ctx.nominal_type_of(&decl);
        TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, s.range)
    } else if let Some(decl) = ctx.resolve_type_name("String") {
        let ty = ctx.nominal_type_of(&decl);
        TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, s.range)
    } else {
        TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
    }
}

fn synthesize_list_literal(ctx: &mut CheckingContext<'_>, list: &phalcom_ast::ast::ListLiteralExpr, expected: &ExpectedType) -> TypedExpression {
    let list_decl = ctx.resolve_type_name("List");
    let expected_elem = expected.collection_element_type(ctx.store);
    let mut elem_tys = Vec::new();

    for el in &list.elements {
        match el {
            ListLiteralElement::Element { expr, .. } => {
                let k = analyze_expression(ctx, expr, &expected_elem);
                if let Some(ty) = k.knowledge.ty() {
                    elem_tys.push(ty);
                }
            }
            ListLiteralElement::Expansion { expr, .. } => {
                analyze_expression(ctx, expr, &ExpectedType::None);
            }
        }
    }

    let elem_ty = if elem_tys.is_empty() {
        expected_elem.ty().unwrap_or_else(|| ctx.store.never())
    } else {
        ctx.store.union(&elem_tys)
    };

    if let Some(decl) = list_decl {
        let k = ctx.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        let form = ctx.store.nominal_form(decl, k);
        if let Ok(ty) = ctx.store.list_of(form, elem_ty) {
            TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, list.range)
        } else {
            TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
        }
    } else {
        TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
    }
}

fn synthesize_set_literal(ctx: &mut CheckingContext<'_>, set: &phalcom_ast::ast::SetLiteralExpr, expected: &ExpectedType) -> TypedExpression {
    let set_decl = ctx.resolve_type_name("Set");
    let expected_elem = expected.collection_element_type(ctx.store);
    let mut elem_tys = Vec::new();

    for el in &set.entries {
        match el {
            SetLiteralEntry::Element { expr, .. } => {
                let k = analyze_expression(ctx, expr, &expected_elem);
                if let Some(ty) = k.knowledge.ty() {
                    elem_tys.push(ty);
                }
            }
            SetLiteralEntry::Expansion { expr, .. } => {
                analyze_expression(ctx, expr, &ExpectedType::None);
            }
        }
    }

    let elem_ty = if elem_tys.is_empty() {
        expected_elem.ty().unwrap_or_else(|| ctx.store.never())
    } else {
        ctx.store.union(&elem_tys)
    };

    if let Some(decl) = set_decl {
        let k = ctx.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        let form = ctx.store.nominal_form(decl, k);
        if let Ok(ty) = ctx.store.set_of(form, elem_ty) {
            TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, set.range)
        } else {
            TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
        }
    } else {
        TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
    }
}

fn synthesize_map_literal(ctx: &mut CheckingContext<'_>, map: &phalcom_ast::ast::MapLiteralExpr, expected: &ExpectedType) -> TypedExpression {
    let map_decl = ctx.resolve_type_name("Map");
    let string_decl = ctx.resolve_type_name("String");
    let (expected_key, expected_val) = expected.map_key_val_types(ctx.store);
    let mut key_tys = Vec::new();
    let mut val_tys = Vec::new();

    for entry in &map.entries {
        match entry {
            MapLiteralEntry::Association { key, value, .. } => {
                let key_ty = match key {
                    MapLiteralKey::BareSymbol { .. } => string_decl.as_ref().map(|d| ctx.nominal_type_of(d)),
                    MapLiteralKey::Computed { expr, .. } => analyze_expression(ctx, expr, &expected_key).knowledge.ty(),
                };
                if let Some(kt) = key_ty {
                    key_tys.push(kt);
                }
                let val_k = analyze_expression(ctx, value, &expected_val);
                if let Some(vt) = val_k.knowledge.ty() {
                    val_tys.push(vt);
                }
            }
            MapLiteralEntry::Expansion { expr, .. } => {
                analyze_expression(ctx, expr, &ExpectedType::None);
            }
        }
    }

    let key_ty = if key_tys.is_empty() {
        expected_key.ty().unwrap_or_else(|| ctx.store.never())
    } else {
        ctx.store.union(&key_tys)
    };

    let val_ty = if val_tys.is_empty() {
        expected_val.ty().unwrap_or_else(|| ctx.store.never())
    } else {
        ctx.store.union(&val_tys)
    };

    if let Some(decl) = map_decl {
        let k = ctx.store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        let form = ctx.store.nominal_form(decl, k);
        if let Ok(ty) = ctx.store.map_of(form, key_ty, val_ty) {
            TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, map.range)
        } else {
            TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
        }
    } else {
        TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
    }
}

fn synthesize_tuple_literal(ctx: &mut CheckingContext<'_>, tup: &phalcom_ast::ast::TupleLiteralExpr, _expected: &ExpectedType) -> TypedExpression {
    let mut elements = Vec::new();

    for entry in &tup.entries {
        match entry {
            TupleLiteralEntry::Positional { expr, .. } => {
                let k = analyze_expression(ctx, expr, &ExpectedType::None);
                let ty = k.knowledge.ty().unwrap_or_else(|| ctx.store.unit());
                elements.push(TupleTypeElement { label: None, ty });
            }
            TupleLiteralEntry::Labeled { label, value, .. } => {
                let k = analyze_expression(ctx, value, &ExpectedType::None);
                let ty = k.knowledge.ty().unwrap_or_else(|| ctx.store.unit());
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
                analyze_expression(ctx, expr, &ExpectedType::None);
            }
        }
    }

    let ty = ctx.store.tuple(elements.into_boxed_slice());
    TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, tup.range)
}

fn synthesize_record_literal(ctx: &mut CheckingContext<'_>, rec: &phalcom_ast::ast::RecordLiteralExpr, _expected: &ExpectedType) -> TypedExpression {
    let mut fields = Vec::new();

    for entry in &rec.entries {
        match entry {
            RecordLiteralEntry::Field(f) => {
                let k = analyze_expression(ctx, &f.value, &ExpectedType::None);
                let ty = k.knowledge.ty().unwrap_or_else(|| ctx.store.unit());
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
                analyze_expression(ctx, expr, &ExpectedType::None);
            }
        }
    }

    let ty = ctx.store.record(fields.into_boxed_slice());
    TypedExpression::known(ty, EvidenceAuthority::ExactSyntax, rec.range)
}

// ---------------------------------------------------------------------------
// Message Send and Invocation Synthesis (E5)
// ---------------------------------------------------------------------------

fn synthesize_method_call(ctx: &mut CheckingContext<'_>, call: &MethodCallExpr, expected: &ExpectedType) -> TypedExpression {
    let recv_typed = analyze_expression(ctx, &call.object, &ExpectedType::None);
    let recv_k = &recv_typed.knowledge;

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
        let dispatch_res = ctx.resolve_dispatch(recv_ty, &sel, recv_typed.dispatch_lookup);
        match dispatch_res {
            DispatchResult::Found(sig) => {
                let ret_k = resolve_call(ctx, &sig, &call.args, expected, call.range);
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

fn synthesize_unqualified_call(ctx: &mut CheckingContext<'_>, call: &UnqualifiedCallExpr, expected: &ExpectedType) -> TypedExpression {
    // 1. Local callable variable lookup
    if let Some(fact) = ctx.lookup_local(&call.name).cloned() {
        if let Some(ty) = fact.knowledge.ty() {
            if let TypeData::Callable(c) = ctx.store.get(ty).clone() {
                let mut params = Vec::new();
                for p in c.parameters.iter() {
                    let mut param = CallableParameter::new("p", TypeKnowledge::known(p.ty, EvidenceAuthority::Declared));
                    if let Some(ref l) = p.label {
                        param = param.with_label(l.to_string());
                    }
                    param = param.with_rest(p.rest);
                    params.push(param);
                }
                let sig = CallableSignature::new(
                    Selector::method(&call.name, vec![]).unwrap_or_else(|_| Selector::getter(&call.name).unwrap()),
                    params,
                    TypeKnowledge::known(c.return_type, EvidenceAuthority::Proven),
                );
                let ret_k = resolve_call(ctx, &sig, &call.args, expected, call.range);
                return TypedExpression::new(ret_k);
            }
        }
        let mut typed = TypedExpression::new(fact.knowledge);
        typed.denotation = fact.denotation;
        return typed;
    }

    // 2. Dispatch send on `self` if inside a class
    if let Some(ref class_decl) = ctx.current_class.clone() {
        let class_ty = if ctx.current_side == crate::identity::DispatchSide::Class {
            if let Some(info) = ctx.declaration_info(class_decl) {
                info.class_object_type
            } else {
                ctx.nominal_type_of(class_decl)
            }
        } else {
            ctx.nominal_type_of(class_decl)
        };
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
            let dispatch_res = ctx.resolve_dispatch(class_ty, &sel, crate::dispatch::DispatchLookup::Normal);
            if let DispatchResult::Found(sig) = dispatch_res {
                let ret_k = resolve_call(ctx, &sig, &call.args, expected, call.range);
                return TypedExpression::new(ret_k);
            }
        }
    }

    // 3. Constructor or nominal reference
    if let Some(decl) = ctx.resolve_type_name(&call.name) {
        let ty = ctx.nominal_type_of(&decl);
        if let Some(sig) = ctx.declaration_generic_signature(&decl) {
            if !sig.parameters.is_empty() {
                let mut arg_tys = Vec::new();
                for arg in &call.args {
                    match arg {
                        PackItem::Positional { expr, .. } => {
                            let k = analyze_expression(ctx, expr, &ExpectedType::None);
                            let t = k.knowledge.ty().unwrap_or_else(|| ctx.store.never());
                            arg_tys.push(t);
                        }
                        PackItem::Labeled { value, .. } => {
                            let k = analyze_expression(ctx, value, &ExpectedType::None);
                            let t = k.knowledge.ty().unwrap_or_else(|| ctx.store.never());
                            arg_tys.push(t);
                        }
                        PackItem::Expand { expr, .. } => {
                            analyze_expression(ctx, expr, &ExpectedType::None);
                        }
                    }
                }
                while arg_tys.len() < sig.parameters.len() {
                    arg_tys.push(ctx.store.never());
                }
                if arg_tys.len() == sig.parameters.len() {
                    if let Ok(applied) = ctx.store.apply_type_form(ty, &arg_tys) {
                        return TypedExpression::known(applied, EvidenceAuthority::Declared, call.range);
                    }
                }
            }
        }
        return TypedExpression::known(ty, EvidenceAuthority::Declared, call.range);
    }

    TypedExpression::unknown(UnknownReason::UnresolvedName(call.name.as_str().into()))
}

fn synthesize_binary_expr(ctx: &mut CheckingContext<'_>, binary: &BinaryExpr) -> TypedExpression {
    let left_typed = analyze_expression(ctx, &binary.left, &ExpectedType::None);
    let left_k = &left_typed.knowledge;
    let right_k = analyze_expression(ctx, &binary.right, &ExpectedType::None).knowledge;

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
            let dispatch_res = ctx.resolve_dispatch(left_ty, &sel, left_typed.dispatch_lookup);
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
    let operand_typed = analyze_expression(ctx, &unary.expr, &ExpectedType::None);
    let operand_k = &operand_typed.knowledge;

    let op_name = match unary.op {
        UnaryOp::Plus => "+",
        UnaryOp::Minus => "-",
        UnaryOp::Not => "not",
        UnaryOp::BitNot => "~",
    };

    if let Ok(sel) = Selector::getter(op_name) {
        if let Some(operand_ty) = operand_k.ty() {
            let dispatch_res = ctx.resolve_dispatch(operand_ty, &sel, operand_typed.dispatch_lookup);
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
    let recv_typed = analyze_expression(ctx, &get.object, &ExpectedType::None);
    let recv_k = &recv_typed.knowledge;

    if let Some(recv_ty) = recv_k.ty() {
        // 1. Check Field on class surface
        let field_opt = match ctx.store.get(recv_ty).clone() {
            TypeData::ClassObject { declaration } => {
                ctx.get_field(&declaration, crate::identity::DispatchSide::Class, &get.property)
            }
            TypeData::Nominal { declaration } => {
                ctx.get_field(&declaration, crate::identity::DispatchSide::Instance, &get.property)
            }
            _ => None,
        };
        if let Some(field_k) = field_opt {
            return TypedExpression::new(field_k.with_range(get.range));
        }

        // 2. Check Getter selector
        if let Ok(sel) = Selector::getter(&get.property) {
            let dispatch_res = ctx.resolve_dispatch(recv_ty, &sel, recv_typed.dispatch_lookup);
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
    let recv_typed = analyze_expression(ctx, &set.object, &ExpectedType::None);
    let recv_k = &recv_typed.knowledge;
    let val_typed = analyze_expression(ctx, &set.value, &ExpectedType::None);
    let val_k = val_typed.knowledge;

    if let Some(recv_ty) = recv_k.ty() {
        // 1. Check field
        let field_opt = match ctx.store.get(recv_ty).clone() {
            TypeData::ClassObject { declaration } => {
                ctx.get_field(&declaration, crate::identity::DispatchSide::Class, &set.property)
            }
            TypeData::Nominal { declaration } => {
                ctx.get_field(&declaration, crate::identity::DispatchSide::Instance, &set.property)
            }
            _ => None,
        };
        if let Some(field_k) = field_opt {
            enforce_assignability(
                ctx.store,
                &ctx.hierarchy,
                &val_k,
                &field_k,
                &ctx.current_module,
                DiagnosticCode::FieldMismatch,
                format!("assigned value does not match field `{}` type", set.property),
                set.range,
                &mut ctx.diagnostics,
            );
            return TypedExpression::new(val_k);
        }

        // 2. Check setter selector
        if let Ok(sel) = Selector::setter(&set.property) {
            let dispatch_res = ctx.resolve_dispatch(recv_ty, &sel, recv_typed.dispatch_lookup);
            if let DispatchResult::Found(sig) = dispatch_res {
                if let Some(param) = sig.parameters.first() {
                    enforce_assignability(
                        ctx.store,
                        &ctx.hierarchy,
                        &val_k,
                        &param.ty,
                        &ctx.current_module,
                        DiagnosticCode::AssignmentMismatch,
                        format!("assigned value does not match setter `{}=` parameter type", set.property),
                        set.range,
                        &mut ctx.diagnostics,
                    );
                }
                return TypedExpression::new(val_k);
            }
        }
    }

    TypedExpression::new(val_k)
}

fn synthesize_index_expr(ctx: &mut CheckingContext<'_>, idx: &IndexExpr) -> TypedExpression {
    let recv_typed = analyze_expression(ctx, &idx.object, &ExpectedType::None);
    let recv_k = &recv_typed.knowledge;

    for arg in &idx.args {
        match arg {
            PackItem::Positional { expr, .. } => {
                analyze_expression(ctx, expr, &ExpectedType::None);
            }
            PackItem::Labeled { value, .. } => {
                analyze_expression(ctx, value, &ExpectedType::None);
            }
            PackItem::Expand { expr, .. } => {
                analyze_expression(ctx, expr, &ExpectedType::None);
            }
        }
    }

    if let Some(recv_ty) = recv_k.ty() {
        // Direct generic List/Map indexing
        if let TypeData::Applied { origin, arguments } = ctx.store.get(recv_ty).clone() {
            if let TypeData::Nominal { declaration } = ctx.store.get(origin) {
                if declaration.name.as_ref() == "List" && arguments.len() == 1 {
                    let elem_ty = arguments[0];
                    return TypedExpression::known(elem_ty, EvidenceAuthority::Proven, idx.range);
                } else if declaration.name.as_ref() == "Map" && arguments.len() == 2 {
                    let val_ty = arguments[1];
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
            let dispatch_res = ctx.resolve_dispatch(recv_ty, &sel, recv_typed.dispatch_lookup);
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
    let recv_typed = analyze_expression(ctx, &set_idx.object, &ExpectedType::None);
    let recv_k = &recv_typed.knowledge;
    let val_typed = analyze_expression(ctx, &set_idx.value, &ExpectedType::None);
    let val_k = val_typed.knowledge;

    for arg in &set_idx.args {
        match arg {
            PackItem::Positional { expr, .. } => {
                analyze_expression(ctx, expr, &ExpectedType::None);
            }
            PackItem::Labeled { value, .. } => {
                analyze_expression(ctx, value, &ExpectedType::None);
            }
            PackItem::Expand { expr, .. } => {
                analyze_expression(ctx, expr, &ExpectedType::None);
            }
        }
    }

    if let Some(recv_ty) = recv_k.ty() {
        if let TypeData::Applied { origin, arguments } = ctx.store.get(recv_ty).clone() {
            if let Some(list_decl) = ctx.resolve_type_name("List") {
                if origin == ctx.nominal_type_of(&list_decl) && arguments.len() == 1 {
                    let elem_k = TypeKnowledge::known(arguments[0], EvidenceAuthority::Declared);
                    enforce_assignability(
                        ctx.store,
                        &ctx.hierarchy,
                        &val_k,
                        &elem_k,
                        &ctx.current_module,
                        DiagnosticCode::AssignmentMismatch,
                        "value assigned to List index does not match element type",
                        set_idx.range,
                        &mut ctx.diagnostics,
                    );
                    return TypedExpression::new(val_k);
                }
            }
        }
    }

    TypedExpression::new(val_k)
}

fn bind_pattern(ctx: &mut CheckingContext<'_>, pattern: &Pattern, fact: ValueSemanticFact) {
    if let Pattern::Name { name, range, .. } = pattern {
        ctx.bind_local(name.clone(), fact, *range);
    }
}
