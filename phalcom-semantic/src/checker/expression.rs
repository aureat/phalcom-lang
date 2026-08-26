//! Expression type synthesis, bidirectional checking, and inference engine (Spec 04.5 / Wave 3).

use super::call::{exact_return_origin, promote_exact_return, resolve_call};
use super::context::CheckingContext;
use super::expected::{ExpectationOrigin, ExpectedType};
use super::statement::check_statement;
use super::typed_expr::TypedExpression;
use crate::checker::analysis::AnalysisStatus;
use crate::checker::binding::{BindingConsistency, BindingWriteResult, reconcile_binding_relation};
use crate::diagnostic::DiagnosticCode;
use crate::dispatch::{CallableParameter, CallableSignature, DispatchResult};
use crate::types::denotation::{SemanticDenotation, ValueSemanticFact};
use crate::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::id::KindId;
use crate::types::outcome::{DynamicBoundaryObligation, RelationOutcome};
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
    ctx.push_expression_owner(expr_id);
    let mut typed = analyze_expression_inner(ctx, expr, expected);
    let owned_cause = ctx.pop_expression_owner(expr_id);

    let status = if let Some(cause_id) = owned_cause {
        typed.causal_invalidity = typed.causal_invalidity.join(crate::checker::causal::CausalInvalidity::One(cause_id));
        AnalysisStatus::Invalid(cause_id)
    } else {
        typed.status.clone()
    };
    let explanation_id = if let Some(ty) = typed.knowledge.ty() {
        let step = match expr {
            Expr::Int { .. } | Expr::Float { .. } | Expr::String { .. } | Expr::Boolean { .. } => {
                crate::explain::ExplanationStep::Literal { expression: expr_id, ty }
            }
            Expr::MethodCall(_) => match typed.callable.clone() {
                Some(callable) => crate::explain::ExplanationStep::MethodCall {
                    call: expr_id,
                    callable,
                    return_ty: ty,
                },
                None => crate::explain::ExplanationStep::UnresolvedCall { call: expr_id, return_ty: ty },
            },
            _ => crate::explain::ExplanationStep::Literal { expression: expr_id, ty },
        };
        let rule = step.derivation_rule();
        let ev = vec![crate::explain::EvidenceRef::SourceSpan(expr.range()), crate::explain::EvidenceRef::TypeId(ty)];
        let status = typed.knowledge.status().unwrap_or(EvidenceStatus::Established);
        let origin = typed.knowledge.origin().unwrap_or(EvidenceOrigin::Syntax);
        Some(ctx.record_derivation(step, rule, status, origin, ev, typed.explanation_parents.clone()))
    } else {
        None
    };
    ctx.record_call_dependency(typed.causal_invalidity, explanation_id);

    let mut analysis = ctx.record_expression(
        expr_id,
        expr.range(),
        typed.knowledge.clone(),
        typed.callable.clone(),
        typed.denotation,
        status.clone(),
    );
    analysis.causal_invalidity = typed.causal_invalidity;
    ctx.expressions.insert(expr_id, analysis.clone());
    if let Some(eid) = explanation_id {
        analysis.explanation = Some(eid);
        ctx.expressions.insert(expr_id, analysis);
    }
    typed.expression_id = Some(expr_id);
    typed.status = status;
    typed
}

/// Bidirectionally checks an expression against an expected type.
pub fn check_expr(ctx: &mut CheckingContext<'_>, expr: &Expr, expected: &ExpectedType) -> TypeKnowledge {
    check_typed_expr(ctx, expr, expected).knowledge
}

/// Bidirectionally checks a typed expression against an expected type, recording a TypeMismatch diagnostic on refutation.
pub fn check_typed_expr(ctx: &mut CheckingContext<'_>, expr: &Expr, expected: &ExpectedType) -> TypedExpression {
    let mut typed = analyze_expression(ctx, expr, expected);
    if let Some(expected_ty) = expected.ty() {
        let application = ctx.apply_knowledge_against_type_owned(
            &typed.knowledge,
            expected_ty,
            DiagnosticCode::TypeMismatch,
            "expression does not match expected type",
            expr.range(),
            typed.expression_id.expect("analyzed expression has expression identity"),
        );
        if application.outcome.is_refuted() {
            if let Some(analysis) = typed.expression_id.and_then(|id| ctx.expressions.get(&id)) {
                typed.status = analysis.status.clone();
                typed.causal_invalidity = analysis.causal_invalidity;
            }
        }
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
                TypedExpression::established(ty, EvidenceOrigin::Syntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Float { range, .. } => {
            if let Some(decl) = ctx.resolve_type_name("Float") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::established(ty, EvidenceOrigin::Syntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::String { range, .. } => {
            if let Some(decl) = ctx.resolve_type_name("String") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::established(ty, EvidenceOrigin::Syntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Boolean { range, .. } => {
            if let Some(decl) = ctx.resolve_type_name("Bool") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::established(ty, EvidenceOrigin::Syntax, *range)
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
                if let Some(info) = ctx.lookup_binding_info(value) {
                    if let Some(state) = ctx.flow.get_binding(info.id) {
                        typed.causal_invalidity = state.causal_invalidity;
                    }
                }
                typed
            } else if let Some(decl) = ctx.resolve_type_name(value) {
                if let Some(info) = ctx.declaration_info(&decl) {
                    TypedExpression::established(info.class_object_type, EvidenceOrigin::DeclarationSemantics, *range)
                        .with_denotation(SemanticDenotation::TypeForm(info.form))
                } else {
                    let ty = ctx.nominal_type_of(&decl);
                    TypedExpression::established(ty, EvidenceOrigin::DeclarationSemantics, *range)
                }
            } else {
                TypedExpression::unknown(UnknownReason::UnresolvedName(value.as_str().into()))
            }
        }
        Expr::SelfVar { range } => {
            if let Some(class_decl) = ctx.current_class.clone() {
                if ctx.current_side == crate::identity::DispatchSide::Class {
                    if let Some(info) = ctx.declaration_info(&class_decl) {
                        TypedExpression::established(info.class_object_type, EvidenceOrigin::Flow, *range)
                            .with_denotation(SemanticDenotation::TypeForm(info.form))
                    } else {
                        let ty = ctx.nominal_type_of(&class_decl);
                        TypedExpression::established(ty, EvidenceOrigin::Flow, *range)
                    }
                } else {
                    let ty = ctx.nominal_type_of(&class_decl);
                    TypedExpression::established(ty, EvidenceOrigin::Flow, *range)
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
                        TypedExpression::established(info.class_object_type, EvidenceOrigin::Flow, *range)
                            .with_denotation(SemanticDenotation::TypeForm(info.form))
                            .with_dispatch_lookup(lookup)
                    } else {
                        let ty = ctx.nominal_type_of(&class_decl);
                        TypedExpression::established(ty, EvidenceOrigin::Flow, *range).with_dispatch_lookup(lookup)
                    }
                } else {
                    let ty = ctx.nominal_type_of(&class_decl);
                    TypedExpression::established(ty, EvidenceOrigin::Flow, *range).with_dispatch_lookup(lookup)
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
                    ctx.flow
                        .get_binding(info.id)
                        .and_then(|state| {
                            state
                                .contract
                                .as_ref()
                                .map(|contract| ExpectedType::proper_from(contract.ty, ExpectationOrigin::AssignmentContract))
                        })
                        .unwrap_or_default()
                } else {
                    ExpectedType::None
                }
            } else {
                ExpectedType::None
            };

            let val_typed = analyze_expression(ctx, &assign.value, &target_expected);
            if let Expr::Var { value: var_name, .. } = &*assign.name {
                let mut causal_invalidity = val_typed.causal_invalidity;
                let mut consistency = BindingConsistency::Unconstrained;
                if let Some(info) = ctx.lookup_binding_info(var_name).cloned() {
                    let state = ctx.flow.get_binding(info.id).cloned();
                    if let Some(state) = state {
                        if !state.mutable {
                            let cause = ctx
                                .emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                                    ctx.current_module.clone(),
                                    DiagnosticCode::AssignmentToImmutable,
                                    format!("Cannot reassign immutable `const` binding `{}`; declare it with `let` to allow mutation.", var_name),
                                    assign.range,
                                ))
                                .expect("error diagnostic has cause");
                            causal_invalidity = causal_invalidity.join(crate::checker::causal::CausalInvalidity::One(cause));
                        } else {
                            let relation = match state.contract.as_ref() {
                                None => RelationOutcome::proven(()),
                                Some(contract) => match &val_typed.knowledge {
                                    TypeKnowledge::Unknown(reason) => RelationOutcome::Blocked(crate::types::outcome::BlockReason::UnknownType(reason.clone())),
                                    TypeKnowledge::Dynamic(_) => RelationOutcome::DynamicBoundary(DynamicBoundaryObligation {
                                        reason: "assignment crosses dynamic boundary".into(),
                                    }),
                                    TypeKnowledge::Known(_) => ctx.check_knowledge_against_type(&val_typed.knowledge, contract.ty),
                                },
                            };
                            let reconciliation = reconcile_binding_relation(state.contract.as_ref(), &val_typed.knowledge, relation);
                            consistency = reconciliation.consistency;
                            if matches!(consistency, BindingConsistency::Refuted { .. }) {
                                let cause = ctx
                                    .emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                                        ctx.current_module.clone(),
                                        DiagnosticCode::AssignmentMismatch,
                                        format!("assigned value is not assignable to `{}`", var_name),
                                        assign.range,
                                    ))
                                    .expect("error diagnostic has cause");
                                causal_invalidity = causal_invalidity.join(crate::checker::causal::CausalInvalidity::One(cause));
                            }
                        }
                    }
                }
                let write = ctx.write_existing(var_name, val_typed.fact(), consistency, causal_invalidity);
                if matches!(write, BindingWriteResult::Immutable) {
                    // Diagnostic and state preservation are handled above; keep
                    // this branch explicit so future write paths cannot silently
                    // turn immutable assignment into recovery mutation.
                }
                let mut result = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, assign.range);
                result.causal_invalidity = causal_invalidity;
                return result;
            }
            TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, assign.range)
        }

        // --- 4. Collections and Product Types (with Bidirectional Propagation) ---
        Expr::ListLiteral(list) => synthesize_list_literal(ctx, list, expected),
        Expr::SetLiteral(set) => synthesize_set_literal(ctx, set, expected),
        Expr::MapLiteral(map) => synthesize_map_literal(ctx, map, expected),
        Expr::TupleLiteral(tup) => synthesize_tuple_literal(ctx, tup, expected),
        Expr::RecordLiteral(rec) => synthesize_record_literal(ctx, rec, expected),

        // --- 5. Blocks and Control Flow ---
        Expr::Block(block) => {
            // A block literal owns its own callable body analysis. In
            // particular, a non-local `return` inside an escaping block is
            // checked at runtime against its frame token; it must not be
            // rechecked against an inferred outer callable return merely
            // because the block expression appears in that callable.
            let outer_expected_return = ctx.expected_return.take();
            let outer_flow = ctx.flow.clone();
            ctx.push_scope();
            let (expected_params, expected_ret) = expected.callable_signature(ctx.store).unwrap_or_default();

            let Some(top) = ctx.resolve_type_name("Object").map(|d| ctx.nominal_type_of(&d)) else {
                ctx.pop_scope();
                ctx.expected_return = outer_expected_return;
                return TypedExpression::unknown(UnknownReason::UnannotatedDeclaration);
            };

            let mut params = Vec::new();
            let mut incomplete_signature = false;
            for (i, p) in block.params.fixed.iter().enumerate() {
                let p_ty = expected_params.get(i).and_then(|e| e.ty());
                if let Some(p_ty) = p_ty {
                    ctx.bind_contextual_block_parameter(p.name.clone(), p_ty, p.range);
                } else {
                    ctx.bind_untyped_block_parameter(p.name.clone(), p.range);
                    incomplete_signature = true;
                }
                params.push(crate::types::store::CallableParameterType {
                    label: None,
                    ty: p_ty.unwrap_or(top),
                    rest: false,
                });
            }
            if let Some(ref rest_p) = block.params.positional_rest {
                let rest_ty = expected_params.get(block.params.fixed.len()).and_then(|e| e.ty());
                if let Some(rest_ty) = rest_ty {
                    ctx.bind_contextual_block_parameter(rest_p.name.clone(), rest_ty, rest_p.range);
                } else {
                    ctx.bind_untyped_block_parameter(rest_p.name.clone(), rest_p.range);
                    incomplete_signature = true;
                }
                params.push(crate::types::store::CallableParameterType {
                    label: None,
                    ty: rest_ty.unwrap_or(top),
                    rest: true,
                });
            }

            let mut tail_typed = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, block.range);
            let len = block.body.len();
            for (i, stmt) in block.body.iter().enumerate() {
                if i == len - 1 {
                    match stmt {
                        Statement::Expr { expr, .. } => {
                            tail_typed = analyze_expression(ctx, expr, &expected_ret);
                        }
                        Statement::Throw { expr, .. } => {
                            analyze_expression(ctx, expr, &ExpectedType::None);
                            tail_typed = TypedExpression::established(ctx.store.never(), EvidenceOrigin::Syntax, block.range);
                        }
                        _ => {
                            check_statement(ctx, stmt);
                            tail_typed = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, block.range);
                        }
                    }
                } else {
                    check_statement(ctx, stmt);
                }
            }
            ctx.pop_scope();
            // Constructing a closure does not execute its body. Keep facts
            // produced while checking the captured body inside that body.
            ctx.flow = outer_flow;

            let Some(return_type) = tail_typed.knowledge.ty() else {
                ctx.expected_return = outer_expected_return;
                return TypedExpression::unknown(UnknownReason::UncheckedExpression);
            };
            if incomplete_signature {
                ctx.expected_return = outer_expected_return;
                return TypedExpression::unknown(UnknownReason::NoTypeEvidence);
            }
            let callable_ty = ctx.store.callable(crate::types::store::CallableType {
                parameters: params.into_boxed_slice(),
                return_type,
            });
            ctx.expected_return = outer_expected_return;
            TypedExpression::established(callable_ty, EvidenceOrigin::Syntax, block.range)
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
                    TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, if_let.range),
                    flow_before,
                )
            };

            let join_status = match ctx.join_flow_states(&[then_flow, else_flow]) {
                Ok(flow) => {
                    ctx.flow = flow;
                    None
                }
                Err(failure) => Some(ctx.publish_flow_join_failure(failure, if_let.range)),
            };

            let combined_knowledge = crate::types::evidence::join_type_knowledge(ctx.store, [then_typed.knowledge.clone(), else_typed.knowledge.clone()]);
            let merged_denotation = match (then_typed.denotation, else_typed.denotation) {
                (Some(d1), Some(d2)) if d1 == d2 => Some(d1),
                _ => None,
            };
            let mut res = TypedExpression::new(combined_knowledge);
            if let Some(status) = join_status {
                res.status = status;
            }
            res.denotation = merged_denotation;
            res.causal_invalidity = val_typed
                .causal_invalidity
                .join(then_typed.causal_invalidity)
                .join(else_typed.causal_invalidity);
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
            TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, while_let.range)
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
                TypedExpression::established(ty, EvidenceOrigin::Flow, chain.range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Membership(m) => {
            analyze_expression(ctx, &m.left, &ExpectedType::None);
            analyze_expression(ctx, &m.right, &ExpectedType::None);
            if let Some(decl) = ctx.resolve_type_name("Bool") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::established(ty, EvidenceOrigin::Flow, m.range)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::IsMembership(m) => {
            analyze_expression(ctx, &m.left, &ExpectedType::None);
            analyze_expression(ctx, &m.candidates, &ExpectedType::None);
            if let Some(decl) = ctx.resolve_type_name("Bool") {
                let ty = ctx.nominal_type_of(&decl);
                TypedExpression::established(ty, EvidenceOrigin::Flow, m.range)
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
            let Some(decl) = ctx.resolve_type_name("Object") else {
                return TypedExpression::unknown(UnknownReason::UnannotatedDeclaration);
            };
            let ty = ctx.nominal_type_of(&decl);
            TypedExpression::established(ty, EvidenceOrigin::Flow, r.range)
        }
        Expr::Ellipsis { .. } => TypedExpression::unknown(UnknownReason::UncheckedExpression),
        _ => TypedExpression::unknown(UnknownReason::UncheckedExpression),
    }
}

// ---------------------------------------------------------------------------
// Helpers for Complex Expressions
// ---------------------------------------------------------------------------

fn synthesize_symbol_expr(ctx: &mut CheckingContext<'_>, s: &SymbolExpr) -> TypedExpression {
    if let Some(decl) = ctx.resolve_type_name("Symbol") {
        let ty = ctx.nominal_type_of(&decl);
        TypedExpression::established(ty, EvidenceOrigin::Syntax, s.range)
    } else if let Some(decl) = ctx.resolve_type_name("String") {
        let ty = ctx.nominal_type_of(&decl);
        TypedExpression::established(ty, EvidenceOrigin::Syntax, s.range)
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
        return TypedExpression::unknown(UnknownReason::NoTypeEvidence);
    } else {
        ctx.store.union(&elem_tys)
    };

    if let Some(decl) = list_decl {
        let k = ctx.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        let form = ctx.store.nominal_form(decl, k);
        if let Ok(ty) = ctx.store.list_of(form, elem_ty) {
            TypedExpression::established(ty, EvidenceOrigin::Syntax, list.range)
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
        return TypedExpression::unknown(UnknownReason::NoTypeEvidence);
    } else {
        ctx.store.union(&elem_tys)
    };

    if let Some(decl) = set_decl {
        let k = ctx.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        let form = ctx.store.nominal_form(decl, k);
        if let Ok(ty) = ctx.store.set_of(form, elem_ty) {
            TypedExpression::established(ty, EvidenceOrigin::Syntax, set.range)
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
        return TypedExpression::unknown(UnknownReason::NoTypeEvidence);
    } else {
        ctx.store.union(&key_tys)
    };

    let val_ty = if val_tys.is_empty() {
        return TypedExpression::unknown(UnknownReason::NoTypeEvidence);
    } else {
        ctx.store.union(&val_tys)
    };

    if let Some(decl) = map_decl {
        let k = ctx.store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        let form = ctx.store.nominal_form(decl, k);
        if let Ok(ty) = ctx.store.map_of(form, key_ty, val_ty) {
            TypedExpression::established(ty, EvidenceOrigin::Syntax, map.range)
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
                let Some(ty) = k.knowledge.ty() else {
                    return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                };
                elements.push(TupleTypeElement { label: None, ty });
            }
            TupleLiteralEntry::Labeled { label, value, .. } => {
                let k = analyze_expression(ctx, value, &ExpectedType::None);
                let Some(ty) = k.knowledge.ty() else {
                    return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                };
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
    TypedExpression::established(ty, EvidenceOrigin::Syntax, tup.range)
}

fn synthesize_record_literal(ctx: &mut CheckingContext<'_>, rec: &phalcom_ast::ast::RecordLiteralExpr, _expected: &ExpectedType) -> TypedExpression {
    let mut fields = Vec::new();

    for entry in &rec.entries {
        match entry {
            RecordLiteralEntry::Field(f) => {
                let k = analyze_expression(ctx, &f.value, &ExpectedType::None);
                let Some(ty) = k.knowledge.ty() else {
                    return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                };
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
    TypedExpression::established(ty, EvidenceOrigin::Syntax, rec.range)
}

// ---------------------------------------------------------------------------
// Message Send and Invocation Synthesis (E5)
// ---------------------------------------------------------------------------

/// A receiver-dependent operation is suppressed only when its required
/// receiver premise is unavailable because the receiver itself is invalid or
/// already suppressed. Ready knowledge with upstream causal invalidity stays
/// analyzable and is handled by ordinary dispatch recovery.
fn suppress_required_receiver_premise(receiver: &TypedExpression) -> Option<TypedExpression> {
    if receiver.knowledge.ty().is_some() {
        return None;
    }
    let cause = match &receiver.status {
        AnalysisStatus::Invalid(cause) => Some(crate::checker::causal::SuppressionCause::One(*cause)),
        AnalysisStatus::Suppressed(cause) => Some(cause.clone()),
        _ => None,
    }?;
    let mut typed = TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
    typed.status = AnalysisStatus::Suppressed(cause);
    typed.causal_invalidity = receiver.causal_invalidity;
    Some(typed)
}

fn synthesize_method_call(ctx: &mut CheckingContext<'_>, call: &MethodCallExpr, expected: &ExpectedType) -> TypedExpression {
    let recv_typed = analyze_expression(ctx, &call.object, &ExpectedType::None);
    let recv_k = &recv_typed.knowledge;

    if let Some(suppressed) = suppress_required_receiver_premise(&recv_typed) {
        return suppressed;
    }

    if let Some(mut typed) = synthesize_control_method_call(ctx, call, expected, recv_k) {
        typed.causal_invalidity = typed.causal_invalidity.join(recv_typed.causal_invalidity);
        return typed;
    }

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
                let result = resolve_call(ctx, &sig, &call.args, expected, call.range);
                let mut typed = TypedExpression::new(result.knowledge);
                typed.status = result.status;
                typed.callable = result.callable;
                typed.explanation_parents = result.explanation_parents;
                if let Some(receiver) = recv_typed.expression_id.and_then(|id| ctx.explanation_for_expression(id)) {
                    if !typed.explanation_parents.contains(&receiver) {
                        typed.explanation_parents.push(receiver);
                    }
                }
                typed.causal_invalidity = recv_typed.causal_invalidity.join(result.causal_invalidity);
                return typed;
            }
            DispatchResult::Dynamic => {
                let mut typed = TypedExpression::dynamic(DynamicReason::RuntimeReflection);
                typed.causal_invalidity = recv_typed.causal_invalidity;
                return typed;
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

/// Analyzes parser-desugared control sends using their block bodies as
/// executable flow while preserving Phalcom's ordinary message-send semantics.
/// A paired `ifTrue(_:ifFalse:)` is structured only for a receiver proven to be
/// the canonical Bool type. `whileTrue(_)` is structured only for the literal
/// block receiver shape recognized by the compiler's sacred-selector inliner.
/// Other same-named sends fall through to normal dispatch.
fn synthesize_control_method_call(
    ctx: &mut CheckingContext<'_>,
    call: &MethodCallExpr,
    expected: &ExpectedType,
    receiver_knowledge: &TypeKnowledge,
) -> Option<TypedExpression> {
    let positional_block = |index: usize| -> Option<&phalcom_ast::ast::BlockExpr> {
        call.args
            .iter()
            .filter_map(|arg| match arg {
                PackItem::Positional { expr: Expr::Block(block), .. } => Some(block.as_ref()),
                _ => None,
            })
            .nth(index)
    };
    let labeled_block = |label: &str| -> Option<&phalcom_ast::ast::BlockExpr> {
        call.args.iter().find_map(|arg| match arg {
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                value: Expr::Block(block),
                ..
            } if text == label => Some(block.as_ref()),
            _ => None,
        })
    };

    let receiver_is_bool = ctx
        .resolve_type_name("Bool")
        .map(|declaration| ctx.nominal_type_of(&declaration))
        .zip(receiver_knowledge.ty())
        .is_some_and(|(bool_ty, receiver_ty)| bool_ty == receiver_ty);
    let receiver_is_literal_block = matches!(&call.object, Expr::Block(_));

    match call.method.as_str() {
        "ifTrue" if receiver_is_bool && labeled_block("ifFalse").is_some() => {
            let then_block = positional_block(0)?;
            let else_block = labeled_block("ifFalse")?;
            let before = ctx.flow.clone();

            let then_typed = analyze_control_block(ctx, then_block, expected);
            let then_flow = ctx.flow.clone();
            ctx.flow = before.clone();
            let else_typed = analyze_control_block(ctx, else_block, expected);
            let else_flow = ctx.flow.clone();

            let join_status = match ctx.join_flow_states(&[then_flow, else_flow]) {
                Ok(flow) => {
                    ctx.flow = flow;
                    None
                }
                Err(failure) => Some(ctx.publish_flow_join_failure(failure, call.range)),
            };
            let knowledge = crate::types::evidence::join_type_knowledge(ctx.store, [then_typed.knowledge.clone(), else_typed.knowledge.clone()]);
            let mut typed = TypedExpression::new(knowledge);
            if let Some(status) = join_status {
                typed.status = status;
            }
            typed.causal_invalidity = then_typed.causal_invalidity.join(else_typed.causal_invalidity);
            typed.explanation_parents.extend(then_typed.explanation_parents);
            typed.explanation_parents.extend(else_typed.explanation_parents);
            Some(typed)
        }
        "whileTrue" if receiver_is_literal_block => {
            let body = positional_block(0)?;
            let before = ctx.flow.clone();
            ctx.push_loop_frame();
            let body_typed = analyze_control_block(ctx, body, &ExpectedType::None);
            let body_flow = ctx.flow.clone();
            let loop_frame = ctx.pop_loop_frame();
            let mut loop_states = vec![before, body_flow];
            loop_states.extend(loop_frame.continues);
            loop_states.extend(loop_frame.breaks);
            let join_status = match ctx.join_flow_states(&loop_states) {
                Ok(flow) => {
                    ctx.flow = flow;
                    None
                }
                Err(failure) => Some(ctx.publish_flow_join_failure(failure, call.range)),
            };
            let mut typed = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, call.range);
            if let Some(status) = join_status {
                typed.status = status;
            }
            typed.causal_invalidity = body_typed.causal_invalidity;
            Some(typed)
        }
        _ => None,
    }
}

/// Checks one sacred control-flow block in its own lexical scope. `return`,
/// `throw`, `break`, and `continue` terminate that path so callers can exclude
/// it from subsequent reachable joins.
fn analyze_control_block(ctx: &mut CheckingContext<'_>, block: &phalcom_ast::ast::BlockExpr, expected: &ExpectedType) -> TypedExpression {
    ctx.push_scope();
    let mut result = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, block.range);
    for statement in &block.body {
        match statement {
            Statement::Expr { expr, .. } => result = analyze_expression(ctx, expr, expected),
            Statement::Return(_) => {
                result = check_statement(ctx, statement)
                    .map(TypedExpression::new)
                    .unwrap_or_else(|| TypedExpression::established(ctx.store.never(), EvidenceOrigin::Flow, block.range));
                ctx.flow.mark_unreachable();
                break;
            }
            Statement::Throw { .. } => {
                check_statement(ctx, statement);
                result = TypedExpression::established(ctx.store.never(), EvidenceOrigin::Flow, block.range);
                ctx.flow.mark_unreachable();
                break;
            }
            Statement::Break { .. } | Statement::Continue { .. } => {
                check_statement(ctx, statement);
                result = TypedExpression::established(ctx.store.never(), EvidenceOrigin::Flow, block.range);
                ctx.flow.mark_unreachable();
                break;
            }
            _ => {
                check_statement(ctx, statement);
                result = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, block.range);
            }
        }
    }
    ctx.pop_scope();
    result
}

fn synthesize_unqualified_call(ctx: &mut CheckingContext<'_>, call: &UnqualifiedCallExpr, expected: &ExpectedType) -> TypedExpression {
    // 1. Local callable variable lookup
    if let Some(fact) = ctx.lookup_local(&call.name) {
        let fact_causal_invalidity = ctx
            .lookup_binding_info(&call.name)
            .and_then(|info| ctx.flow.get_binding(info.id))
            .map(|state| state.causal_invalidity)
            .unwrap_or(crate::checker::causal::CausalInvalidity::Clean);
        if let Some(ty) = fact.knowledge.ty() {
            if let TypeData::Callable(c) = ctx.store.get(ty).clone() {
                let mut params = Vec::new();
                for p in c.parameters.iter() {
                    let mut param = CallableParameter::new("p", TypeKnowledge::assumed(p.ty, EvidenceOrigin::CallableSignature));
                    if let Some(ref l) = p.label {
                        param = param.with_label(l.to_string());
                    }
                    param = param.with_rest(p.rest);
                    params.push(param);
                }
                let sig = CallableSignature::new(
                    Selector::method(&call.name, vec![]).unwrap_or_else(|_| Selector::getter(&call.name).unwrap()),
                    params,
                    TypeKnowledge::established(c.return_type, EvidenceOrigin::Flow),
                );
                let result = resolve_call(ctx, &sig, &call.args, expected, call.range);
                let mut typed = TypedExpression::new(result.knowledge);
                typed.status = result.status;
                typed.explanation_parents = result.explanation_parents;
                typed.causal_invalidity = fact_causal_invalidity.join(result.causal_invalidity);
                return typed;
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
                let result = resolve_call(ctx, &sig, &call.args, expected, call.range);
                let mut typed = TypedExpression::new(result.knowledge);
                typed.status = result.status;
                typed.callable = result.callable;
                typed.explanation_parents = result.explanation_parents;
                typed.causal_invalidity = result.causal_invalidity;
                return typed;
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
                            let Some(t) = k.knowledge.ty() else {
                                return TypedExpression::unknown(UnknownReason::UnderconstrainedTypeVariable);
                            };
                            arg_tys.push(t);
                        }
                        PackItem::Labeled { value, .. } => {
                            let k = analyze_expression(ctx, value, &ExpectedType::None);
                            let Some(t) = k.knowledge.ty() else {
                                return TypedExpression::unknown(UnknownReason::UnderconstrainedTypeVariable);
                            };
                            arg_tys.push(t);
                        }
                        PackItem::Expand { expr, .. } => {
                            analyze_expression(ctx, expr, &ExpectedType::None);
                        }
                    }
                }
                if arg_tys.len() != sig.parameters.len() {
                    return TypedExpression::unknown(UnknownReason::UnderconstrainedTypeVariable);
                }
                if arg_tys.len() == sig.parameters.len() {
                    if let Ok(applied) = ctx.store.apply_type_form(ty, &arg_tys) {
                        return TypedExpression::established(applied, EvidenceOrigin::DeclarationSemantics, call.range);
                    }
                }
            }
        }
        return TypedExpression::established(ty, EvidenceOrigin::DeclarationSemantics, call.range);
    }

    TypedExpression::unknown(UnknownReason::UnresolvedName(call.name.as_str().into()))
}

fn synthesize_binary_expr(ctx: &mut CheckingContext<'_>, binary: &BinaryExpr) -> TypedExpression {
    let left_typed = analyze_expression(ctx, &binary.left, &ExpectedType::None);
    let left_k = &left_typed.knowledge;
    let right_typed = analyze_expression(ctx, &binary.right, &ExpectedType::None);
    let right_k = &right_typed.knowledge;

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
                let mut typed = TypedExpression::new(promote_exact_return(&sig.return_type, exact_return_origin(sig.kind), binary.range));
                typed.callable = ctx.resolved_callable_for_current_expression();
                typed.causal_invalidity = left_typed.causal_invalidity.join(right_typed.causal_invalidity);
                return typed;
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
                let mut typed = TypedExpression::new(promote_exact_return(&sig.return_type, exact_return_origin(sig.kind), unary.range));
                typed.callable = ctx.resolved_callable_for_current_expression();
                typed.causal_invalidity = operand_typed.causal_invalidity;
                return typed;
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

    if let Some(suppressed) = suppress_required_receiver_premise(&recv_typed) {
        return suppressed;
    }

    if let Some(recv_ty) = recv_k.ty() {
        // 1. Check Field on class surface
        let field_opt = match ctx.store.get(recv_ty).clone() {
            TypeData::ClassObject { declaration } => ctx.get_field(&declaration, crate::identity::DispatchSide::Class, &get.property),
            TypeData::Nominal { declaration } => ctx.get_field(&declaration, crate::identity::DispatchSide::Instance, &get.property),
            _ => None,
        };
        if let Some(field_k) = field_opt {
            let mut typed = TypedExpression::new(field_k.with_range(get.range));
            typed.causal_invalidity = recv_typed.causal_invalidity;
            return typed;
        }

        // 2. Check Getter selector
        if let Ok(sel) = Selector::getter(&get.property) {
            let dispatch_res = ctx.resolve_dispatch(recv_ty, &sel, recv_typed.dispatch_lookup);
            if let DispatchResult::Found(sig) = dispatch_res {
                let mut typed = TypedExpression::new(promote_exact_return(&sig.return_type, exact_return_origin(sig.kind), get.range));
                typed.callable = ctx.resolved_callable_for_current_expression();
                typed.causal_invalidity = recv_typed.causal_invalidity;
                return typed;
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
    if let Some(suppressed) = suppress_required_receiver_premise(&recv_typed) {
        return suppressed;
    }
    let val_typed = analyze_expression(ctx, &set.value, &ExpectedType::None);
    let val_k = val_typed.knowledge;

    if let Some(recv_ty) = recv_k.ty() {
        // 1. Check field
        let field_opt = match ctx.store.get(recv_ty).clone() {
            TypeData::ClassObject { declaration } => ctx.get_field(&declaration, crate::identity::DispatchSide::Class, &set.property),
            TypeData::Nominal { declaration } => ctx.get_field(&declaration, crate::identity::DispatchSide::Instance, &set.property),
            _ => None,
        };
        if let Some(field_k) = field_opt {
            ctx.apply_assignability(
                &val_k,
                &field_k,
                DiagnosticCode::FieldMismatch,
                format!("assigned value does not match field `{}` type", set.property),
                set.range,
            );
            return TypedExpression::new(val_k);
        }

        // 2. Check setter selector
        if let Ok(sel) = Selector::setter(&set.property) {
            let dispatch_res = ctx.resolve_dispatch(recv_ty, &sel, recv_typed.dispatch_lookup);
            if let DispatchResult::Found(sig) = dispatch_res {
                if let Some(param) = sig.parameters.first() {
                    ctx.apply_assignability(
                        &val_k,
                        &param.ty,
                        DiagnosticCode::AssignmentMismatch,
                        format!("assigned value does not match setter `{}=` parameter type", set.property),
                        set.range,
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

    if let Some(suppressed) = suppress_required_receiver_premise(&recv_typed) {
        return suppressed;
    }

    let mut causal_invalidity = recv_typed.causal_invalidity;
    for arg in &idx.args {
        match arg {
            PackItem::Positional { expr, .. } => {
                causal_invalidity = causal_invalidity.join(analyze_expression(ctx, expr, &ExpectedType::None).causal_invalidity);
            }
            PackItem::Labeled { value, .. } => {
                causal_invalidity = causal_invalidity.join(analyze_expression(ctx, value, &ExpectedType::None).causal_invalidity);
            }
            PackItem::Expand { expr, .. } => {
                causal_invalidity = causal_invalidity.join(analyze_expression(ctx, expr, &ExpectedType::None).causal_invalidity);
            }
        }
    }

    if let Some(recv_ty) = recv_k.ty() {
        // Direct generic List/Map indexing
        if let TypeData::Applied { origin, arguments } = ctx.store.get(recv_ty).clone() {
            if let TypeData::Nominal { declaration } = ctx.store.get(origin) {
                if declaration.name.as_ref() == "List" && arguments.len() == 1 {
                    let elem_ty = arguments[0];
                    let mut typed = TypedExpression::established(elem_ty, EvidenceOrigin::Flow, idx.range);
                    typed.causal_invalidity = causal_invalidity;
                    return typed;
                } else if declaration.name.as_ref() == "Map" && arguments.len() == 2 {
                    let val_ty = arguments[1];
                    let mut typed = TypedExpression::established(val_ty, EvidenceOrigin::Flow, idx.range);
                    typed.causal_invalidity = causal_invalidity;
                    return typed;
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
                let mut typed = TypedExpression::new(promote_exact_return(&sig.return_type, exact_return_origin(sig.kind), idx.range));
                typed.callable = ctx.resolved_callable_for_current_expression();
                typed.causal_invalidity = causal_invalidity;
                return typed;
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
    if let Some(suppressed) = suppress_required_receiver_premise(&recv_typed) {
        return suppressed;
    }
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
                    let elem_k = TypeKnowledge::assumed(arguments[0], EvidenceOrigin::DeclarationSemantics);
                    ctx.apply_assignability(
                        &val_k,
                        &elem_k,
                        DiagnosticCode::AssignmentMismatch,
                        "value assigned to List index does not match element type",
                        set_idx.range,
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
        ctx.bind_pattern_binding(name.clone(), fact, *range);
    }
}
