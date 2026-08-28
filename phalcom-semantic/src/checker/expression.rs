//! Expression type synthesis, bidirectional checking, and inference engine (Spec 04.5 / Wave 3).

use super::call::{
    CallPremise, CallableApplicationTarget, StaticCallShape, UnresolvedApplicationReason, analyze_non_callable_invocation, analyze_unresolved_application,
    application_arguments, apply_resolved_callable,
};
use super::context::CheckingContext;
use super::expected::{ExpectationOrigin, ExpectedType};
use super::statement::check_statement;
use super::typed_expr::TypedExpression;
use crate::checker::analysis::AnalysisStatus;
use crate::checker::binding::{BindingConsistency, BindingWriteResult, reconcile_binding_relation};
use crate::diagnostic::DiagnosticCode;
use crate::dispatch::{CallableSignature, ResolvedDispatch, ResolvedDispatchResult};
use crate::identity::DeclarationId;
use crate::types::denotation::{SemanticDenotation, ValueSemanticFact};
use crate::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::id::{KindId, TypeId};
use crate::types::outcome::{DynamicBoundaryObligation, RelationOutcome};
use crate::types::relation::TypeHierarchy;
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
    if let Some(cause_id) = ctx.pop_expression_owner(expr_id) {
        typed.invalidate(cause_id);
    }
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

    typed.expression_id = Some(expr_id);
    ctx.publish_expression_analysis(expr_id, expr.range(), &typed, explanation_id);
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
                if let Some((_, field_k)) = ctx.resolve_current_field(&class_decl, ctx.current_side, value) {
                    return TypedExpression::new(field_k.with_range(*range));
                }
            }
            TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
        }

        // --- 3. Assignments ---
        Expr::Assignment(assign) => {
            let target_expected = match &*assign.name {
                Expr::Var { value: var_name, .. } => ctx
                    .lookup_binding_info(var_name)
                    .and_then(|info| ctx.flow.get_binding(info.id))
                    .and_then(|state| {
                        state
                            .contract
                            .as_ref()
                            .map(|contract| ExpectedType::proper_from(contract.ty, ExpectationOrigin::AssignmentContract))
                    })
                    .unwrap_or_default(),
                Expr::Field { value: field_name, .. } => ctx
                    .current_class
                    .clone()
                    .and_then(|class_decl| ctx.get_field(&class_decl, ctx.current_side, field_name))
                    .and_then(|field| field.ty().map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::AssignmentContract)))
                    .unwrap_or_default(),
                _ => ExpectedType::None,
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
                                    format!(
                                        "Cannot reassign immutable `const` binding `{}`; declare it with `let` to allow mutation.",
                                        var_name
                                    ),
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
            if let Expr::Field { value: field_name, .. } = &*assign.name {
                if let Some(class_decl) = ctx.current_class.clone() {
                    if let Some((field_id, field_k)) = ctx.resolve_field_contract(&class_decl, ctx.current_side, field_name) {
                        let application = ctx.apply_assignability(
                            &val_typed.knowledge,
                            &field_k,
                            DiagnosticCode::FieldMismatch,
                            "assigned value does not match field type".to_string(),
                            assign.range,
                        );
                        let mut result = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, assign.range);
                        result.causal_invalidity = val_typed.causal_invalidity;
                        apply_relation_application_to_typed(&mut result, &application);
                        ctx.write_current_field(field_id, field_k, val_typed.knowledge);
                        return result;
                    }
                }
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
            bind_pattern(ctx, &if_let.pattern, val_typed.fact(), val_typed.causal_invalidity);
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
            bind_pattern(ctx, &while_let.pattern, val_typed.fact(), val_typed.causal_invalidity);
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
    let list_form = list_decl.as_ref().map(|decl| {
        let kind = ctx.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        ctx.store.nominal_form(decl.clone(), kind)
    });
    let mut contributions = Vec::new();
    let mut operands = Vec::new();

    for el in &list.elements {
        match el {
            ListLiteralElement::Element { expr, .. } => {
                let typed = analyze_expression(ctx, expr, &expected_elem);
                contributions.push(typed.knowledge.clone());
                operands.push(typed);
            }
            ListLiteralElement::Expansion { expr, .. } => {
                let typed = analyze_expression(ctx, expr, &ExpectedType::None);
                let projected = list_form
                    .map(|form| crate::checker::composition::project_applied_argument(ctx.store, &typed.knowledge, form, 0))
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));
                contributions.push(projected);
                operands.push(typed);
            }
        }
    }

    let knowledge = if list.elements.is_empty() {
        TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)
    } else if let Some(form) = list_form {
        crate::types::evidence::compose_required_knowledge(contributions, EvidenceOrigin::Syntax, |types| {
            if types.is_empty() {
                return Err(UnknownReason::NoTypeEvidence);
            }
            let element = ctx.store.union(types);
            ctx.store.list_of(form, element).map_err(|_| UnknownReason::UncheckedExpression)
        })
    } else {
        TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
    };
    let mut result = TypedExpression::new(match knowledge {
        TypeKnowledge::Known(_) => knowledge.with_range(list.range),
        other => other,
    });
    crate::checker::composition::propagate_required_dependencies(&mut result, &operands);
    result
}

fn synthesize_set_literal(ctx: &mut CheckingContext<'_>, set: &phalcom_ast::ast::SetLiteralExpr, expected: &ExpectedType) -> TypedExpression {
    let set_decl = ctx.resolve_type_name("Set");
    let expected_elem = expected.collection_element_type(ctx.store);
    let set_form = set_decl.as_ref().map(|decl| {
        let kind = ctx.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        ctx.store.nominal_form(decl.clone(), kind)
    });
    let mut contributions = Vec::new();
    let mut operands = Vec::new();

    for el in &set.entries {
        match el {
            SetLiteralEntry::Element { expr, .. } => {
                let typed = analyze_expression(ctx, expr, &expected_elem);
                contributions.push(typed.knowledge.clone());
                operands.push(typed);
            }
            SetLiteralEntry::Expansion { expr, .. } => {
                let typed = analyze_expression(ctx, expr, &ExpectedType::None);
                let projected = set_form
                    .map(|form| crate::checker::composition::project_applied_argument(ctx.store, &typed.knowledge, form, 0))
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));
                contributions.push(projected);
                operands.push(typed);
            }
        }
    }

    let knowledge = if set.entries.is_empty() {
        TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)
    } else if let Some(form) = set_form {
        crate::types::evidence::compose_required_knowledge(contributions, EvidenceOrigin::Syntax, |types| {
            if types.is_empty() {
                return Err(UnknownReason::NoTypeEvidence);
            }
            let element = ctx.store.union(types);
            ctx.store.set_of(form, element).map_err(|_| UnknownReason::UncheckedExpression)
        })
    } else {
        TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
    };
    let mut result = TypedExpression::new(match knowledge {
        TypeKnowledge::Known(_) => knowledge.with_range(set.range),
        other => other,
    });
    crate::checker::composition::propagate_required_dependencies(&mut result, &operands);
    result
}

fn synthesize_map_literal(ctx: &mut CheckingContext<'_>, map: &phalcom_ast::ast::MapLiteralExpr, expected: &ExpectedType) -> TypedExpression {
    let map_decl = ctx.resolve_type_name("Map");
    let symbol_decl = ctx.resolve_type_name("Symbol");
    let (expected_key, expected_val) = expected.map_key_val_types(ctx.store);
    let map_form = map_decl.as_ref().map(|decl| {
        let kind = ctx.store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        ctx.store.nominal_form(decl.clone(), kind)
    });
    let mut operands = Vec::new();
    let mut key_knowledge = Vec::new();
    let mut value_knowledge = Vec::new();

    for entry in &map.entries {
        match entry {
            MapLiteralEntry::Association { key, value, .. } => {
                match key {
                    MapLiteralKey::BareSymbol { .. } => {
                        let knowledge = symbol_decl
                            .as_ref()
                            .map(|decl| TypeKnowledge::established(ctx.nominal_type_of(decl), EvidenceOrigin::Syntax))
                            .unwrap_or(TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));
                        key_knowledge.push(knowledge);
                    }
                    MapLiteralKey::Computed { expr, .. } => {
                        let typed = analyze_expression(ctx, expr, &expected_key);
                        key_knowledge.push(typed.knowledge.clone());
                        operands.push(typed);
                    }
                }
                let typed = analyze_expression(ctx, value, &expected_val);
                value_knowledge.push(typed.knowledge.clone());
                operands.push(typed);
            }
            MapLiteralEntry::Expansion { expr, .. } => {
                let typed = analyze_expression(ctx, expr, &ExpectedType::None);
                let key = map_form
                    .map(|form| crate::checker::composition::project_applied_argument(ctx.store, &typed.knowledge, form, 0))
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));
                let value = map_form
                    .map(|form| crate::checker::composition::project_applied_argument(ctx.store, &typed.knowledge, form, 1))
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));
                key_knowledge.push(key);
                value_knowledge.push(value);
                operands.push(typed);
            }
        }
    }

    let mut lane = |knowledge: Vec<TypeKnowledge>| {
        crate::types::evidence::compose_required_knowledge(knowledge, EvidenceOrigin::Syntax, |types| {
            if types.is_empty() {
                return Err(UnknownReason::NoTypeEvidence);
            }
            Ok(ctx.store.union(types))
        })
    };
    let key_lane = lane(key_knowledge);
    let value_lane = lane(value_knowledge);
    let knowledge = if let Some(form) = map_form {
        crate::types::evidence::compose_required_knowledge([key_lane, value_lane], EvidenceOrigin::Syntax, |types| {
            let [key, value] = types else {
                return Err(UnknownReason::UncheckedExpression);
            };
            ctx.store.map_of(form, *key, *value).map_err(|_| UnknownReason::UncheckedExpression)
        })
    } else {
        TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
    };
    let mut result = TypedExpression::new(match knowledge {
        TypeKnowledge::Known(_) => knowledge.with_range(map.range),
        other => other,
    });
    crate::checker::composition::propagate_required_dependencies(&mut result, &operands);
    result
}

fn synthesize_tuple_literal(ctx: &mut CheckingContext<'_>, tup: &phalcom_ast::ast::TupleLiteralExpr, _expected: &ExpectedType) -> TypedExpression {
    let mut labels = Vec::new();
    let mut knowledge = Vec::new();
    let mut operands = Vec::new();

    for entry in &tup.entries {
        match entry {
            TupleLiteralEntry::Positional { expr, .. } => {
                let typed = analyze_expression(ctx, expr, &ExpectedType::None);
                labels.push(None);
                knowledge.push(typed.knowledge.clone());
                operands.push(typed);
            }
            TupleLiteralEntry::Labeled { label, value, .. } => {
                let typed = analyze_expression(ctx, value, &ExpectedType::None);
                let label_name = match label {
                    ProductLabel::Static { symbol, .. } => match symbol {
                        SymbolLiteralKind::Name(n) => Some(n.clone().into_boxed_str()),
                        SymbolLiteralKind::Selector { name, .. } => Some(name.clone().into_boxed_str()),
                        _ => None,
                    },
                    _ => None,
                };
                labels.push(label_name);
                knowledge.push(typed.knowledge.clone());
                operands.push(typed);
            }
            TupleLiteralEntry::Expand { expr, .. } => {
                let typed = analyze_expression(ctx, expr, &ExpectedType::None);
                match crate::checker::composition::project_tuple_elements(ctx.store, &typed.knowledge) {
                    Ok(projected) => {
                        let source_labels = typed.knowledge.ty().and_then(|ty| match ctx.store.get(ty) {
                            TypeData::Tuple(elements) => Some(elements.iter().map(|element| element.label.clone()).collect::<Vec<_>>()),
                            _ => None,
                        });
                        for (index, component) in projected.into_iter().enumerate() {
                            labels.push(source_labels.as_ref().and_then(|labels| labels.get(index).cloned()).flatten());
                            knowledge.push(component);
                        }
                    }
                    Err(blocker) => {
                        labels.push(None);
                        knowledge.push(blocker);
                    }
                }
                operands.push(typed);
            }
        }
    }

    let knowledge = crate::types::evidence::compose_required_knowledge(knowledge, EvidenceOrigin::Syntax, |types| {
        if types.len() != labels.len() {
            return Err(UnknownReason::UncheckedExpression);
        }
        let elements = labels
            .iter()
            .cloned()
            .zip(types.iter().copied())
            .map(|(label, ty)| TupleTypeElement { label, ty })
            .collect::<Vec<_>>();
        Ok(ctx.store.tuple(elements.into_boxed_slice()))
    });
    let mut result = TypedExpression::new(match knowledge {
        TypeKnowledge::Known(_) => knowledge.with_range(tup.range),
        other => other,
    });
    crate::checker::composition::propagate_required_dependencies(&mut result, &operands);
    result
}

fn synthesize_record_literal(ctx: &mut CheckingContext<'_>, rec: &phalcom_ast::ast::RecordLiteralExpr, _expected: &ExpectedType) -> TypedExpression {
    let mut names = Vec::new();
    let mut knowledge = Vec::new();
    let mut operands = Vec::new();

    for entry in &rec.entries {
        match entry {
            RecordLiteralEntry::Field(f) => {
                let typed = analyze_expression(ctx, &f.value, &ExpectedType::None);
                let name = match &f.label {
                    ProductLabel::Static { symbol, .. } => match symbol {
                        SymbolLiteralKind::Name(n) => n.clone(),
                        SymbolLiteralKind::Selector { name, .. } => name.clone(),
                        _ => "field".into(),
                    },
                    _ => "field".into(),
                };
                names.push(name.into_boxed_str());
                knowledge.push(typed.knowledge.clone());
                operands.push(typed);
            }
            RecordLiteralEntry::Expansion { expr, .. } => {
                let typed = analyze_expression(ctx, expr, &ExpectedType::None);
                match crate::checker::composition::project_record_fields(ctx.store, &typed.knowledge) {
                    Ok(fields) => {
                        for (name, field_knowledge) in fields {
                            names.push(name);
                            knowledge.push(field_knowledge);
                        }
                    }
                    Err(blocker) => {
                        names.push("field".into());
                        knowledge.push(blocker);
                    }
                }
                operands.push(typed);
            }
        }
    }

    let knowledge = crate::types::evidence::compose_required_knowledge(knowledge, EvidenceOrigin::Syntax, |types| {
        if types.len() != names.len() {
            return Err(UnknownReason::UncheckedExpression);
        }
        let fields = names
            .iter()
            .cloned()
            .zip(types.iter().copied())
            .map(|(name, ty)| RecordTypeField { name, ty })
            .collect::<Vec<_>>();
        Ok(ctx.store.record(fields.into_boxed_slice()))
    });
    let mut result = TypedExpression::new(match knowledge {
        TypeKnowledge::Known(_) => knowledge.with_range(rec.range),
        other => other,
    });
    crate::checker::composition::propagate_required_dependencies(&mut result, &operands);
    result
}

// ---------------------------------------------------------------------------
// Message Send and Invocation Synthesis (E5)
// ---------------------------------------------------------------------------

fn apply_relation_application_to_typed(typed: &mut TypedExpression, application: &super::context::RelationApplication) {
    if let Some(status) = &application.status {
        match status {
            AnalysisStatus::Invalid(cause) => typed.invalidate(*cause),
            other => typed.status = other.clone(),
        }
    }
    typed.debug_assert_coherent();
}

fn synthesize_method_call(ctx: &mut CheckingContext<'_>, call: &MethodCallExpr, expected: &ExpectedType) -> TypedExpression {
    let recv_typed = analyze_expression(ctx, &call.object, &ExpectedType::None);
    let premise = CallPremise::from_typed(ctx, &recv_typed);

    if let Some(mut typed) = synthesize_control_method_call(ctx, call, expected, &recv_typed) {
        typed.causal_invalidity = typed.causal_invalidity.join(recv_typed.causal_invalidity);
        return typed;
    }

    if call.method == "call" {
        if let Some(receiver_ty) = recv_typed.knowledge.ty() {
            if let Some(target) = super::call::callable_value_target(ctx.store, receiver_ty, recv_typed.knowledge.status().unwrap_or(EvidenceStatus::Assumed)) {
                let arguments = application_arguments(&call.args);
                return apply_resolved_callable(ctx, &target, &premise, &arguments, expected, call.range).into();
            }
        }
    }

    let arguments = application_arguments(&call.args);
    let Some(receiver_ty) = recv_typed.knowledge.ty() else {
        let reason = match &recv_typed.knowledge {
            TypeKnowledge::Unknown(_) => {
                if matches!(recv_typed.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Suppressed(_)) {
                    UnresolvedApplicationReason::PremiseInvalidUnavailable
                } else {
                    UnresolvedApplicationReason::PremiseUnknown
                }
            }
            TypeKnowledge::Dynamic(reason) => UnresolvedApplicationReason::PremiseDynamic(reason.clone()),
            TypeKnowledge::Known(_) => unreachable!("known receiver has a type"),
        };
        return analyze_unresolved_application(ctx, &premise, &arguments, reason).into();
    };

    let slots = match super::call::static_call_shape(&arguments) {
        StaticCallShape::Exact(slots) => slots,
        StaticCallShape::Dynamic(reason) => {
            return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DynamicShape(reason)).into();
        }
    };
    let Ok(selector) = Selector::method(&call.method, slots) else {
        return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchMissing).into();
    };
    match ctx.resolve_dispatch_target(receiver_ty, &selector, recv_typed.dispatch_lookup.clone()) {
        ResolvedDispatchResult::Found(resolved) => {
            let target = CallableApplicationTarget::from_dispatch(resolved);
            apply_resolved_callable(ctx, &target, &premise, &arguments, expected, call.range).into()
        }
        ResolvedDispatchResult::Missing { .. } => {
            analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchMissing).into()
        }
        ResolvedDispatchResult::Ambiguous(_) => {
            analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchAmbiguous).into()
        }
        ResolvedDispatchResult::Dynamic => analyze_unresolved_application(
            ctx,
            &premise,
            &arguments,
            UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
        )
        .into(),
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
    receiver_typed: &TypedExpression,
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
        .zip(receiver_typed.knowledge.ty())
        .is_some_and(|(bool_ty, receiver_ty)| bool_ty == receiver_ty);
    let receiver_is_literal_block = matches!(&call.object, Expr::Block(_));

    match call.method.as_str() {
        "ifTrue" if receiver_is_bool && labeled_block("ifFalse").is_some() => {
            let then_block = positional_block(0)?;
            let else_block = labeled_block("ifFalse")?;
            let before = ctx.flow.clone();

            ctx.flow = before.clone();
            if let Some(predicate) = crate::checker::flow::extract_trusted_predicate(ctx, &call.object, receiver_typed, true) {
                let hierarchy = &ctx.hierarchy;
                crate::checker::flow::apply_predicate(&mut ctx.flow, &predicate, ctx.store, hierarchy);
            }
            let then_typed = analyze_control_block(ctx, then_block, expected);
            let then_flow = ctx.flow.clone();
            ctx.flow = before.clone();
            if let Some(predicate) = crate::checker::flow::extract_trusted_predicate(ctx, &call.object, receiver_typed, false) {
                let hierarchy = &ctx.hierarchy;
                crate::checker::flow::apply_predicate(&mut ctx.flow, &predicate, ctx.store, hierarchy);
            }
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
        let binding_state = ctx.lookup_binding_info(&call.name).and_then(|info| ctx.flow.get_binding(info.id)).cloned();
        let premise = CallPremise {
            knowledge: fact.knowledge.clone(),
            status: AnalysisStatus::Ready,
            causal_invalidity: binding_state.as_ref().map(|state| state.causal_invalidity).unwrap_or_default(),
            explanation: binding_state.as_ref().and_then(|state| state.explanation),
        };
        let arguments = application_arguments(&call.args);
        if let Some(ty) = fact.knowledge.ty() {
            if let Some(target) = super::call::callable_value_target(ctx.store, ty, fact.knowledge.status().unwrap_or(EvidenceStatus::Assumed)) {
                match super::call::static_call_shape(&arguments) {
                    StaticCallShape::Exact(_) => {
                        return apply_resolved_callable(ctx, &target, &premise, &arguments, expected, call.range).into();
                    }
                    StaticCallShape::Dynamic(reason) => {
                        return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DynamicShape(reason)).into();
                    }
                }
            }

            // Ordinary nominal objects are structurally callable when their
            // runtime class dispatches the matching `call(...)` selector.
            // Direct invocation (`value(...)`) and explicit `value.call(...)`
            // therefore share the same canonical dispatch surface; nominal
            // knowledge is evidence for dispatch, not evidence of non-callability.
            let slots = match super::call::static_call_shape(&arguments) {
                StaticCallShape::Exact(slots) => slots,
                StaticCallShape::Dynamic(reason) => {
                    return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DynamicShape(reason)).into();
                }
            };
            if let Ok(selector) = Selector::method("call", slots) {
                match ctx.resolve_dispatch_target(ty, &selector, crate::dispatch::DispatchLookup::Normal) {
                    ResolvedDispatchResult::Found(resolved) => {
                        let target = CallableApplicationTarget::from_dispatch(resolved);
                        return apply_resolved_callable(ctx, &target, &premise, &arguments, expected, call.range).into();
                    }
                    ResolvedDispatchResult::Ambiguous(_) => {
                        return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchAmbiguous).into();
                    }
                    ResolvedDispatchResult::Dynamic => {
                        return analyze_unresolved_application(
                            ctx,
                            &premise,
                            &arguments,
                            UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
                        )
                        .into();
                    }
                    ResolvedDispatchResult::Missing { .. } => {}
                }
            }
            return analyze_non_callable_invocation(ctx, &premise, &call.args, call.range).into();
        }
        let reason = match &fact.knowledge {
            TypeKnowledge::Unknown(_) => UnresolvedApplicationReason::PremiseUnknown,
            TypeKnowledge::Dynamic(reason) => UnresolvedApplicationReason::PremiseDynamic(reason.clone()),
            TypeKnowledge::Known(_) => unreachable!("known lexical value has a type"),
        };
        return analyze_unresolved_application(ctx, &premise, &arguments, reason).into();
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
        let premise = CallPremise::established(TypeKnowledge::established(class_ty, EvidenceOrigin::DeclarationSemantics));
        let arguments = application_arguments(&call.args);
        let slots = match super::call::static_call_shape(&arguments) {
            StaticCallShape::Exact(slots) => slots,
            StaticCallShape::Dynamic(reason) => {
                return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DynamicShape(reason)).into();
            }
        };
        if let Ok(selector) = Selector::method(&call.name, slots) {
            match ctx.resolve_dispatch_target(class_ty, &selector, crate::dispatch::DispatchLookup::Normal) {
                ResolvedDispatchResult::Found(resolved) => {
                    let target = CallableApplicationTarget::from_dispatch(resolved);
                    return apply_resolved_callable(ctx, &target, &premise, &arguments, expected, call.range).into();
                }
                ResolvedDispatchResult::Missing { .. } => {}
                ResolvedDispatchResult::Ambiguous(_) => {
                    return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchAmbiguous).into();
                }
                ResolvedDispatchResult::Dynamic => {
                    return analyze_unresolved_application(
                        ctx,
                        &premise,
                        &arguments,
                        UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
                    )
                    .into();
                }
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

    let Ok(selector) = Selector::method(op_name, vec![SelectorSlot::Positional]) else {
        let premise = CallPremise::from_typed(ctx, &left_typed);
        let arguments = vec![super::call::ApplicationArgument::Positional {
            expression: &binary.right,
            range: binary.right.range(),
        }];
        return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchMissing).into();
    };
    let direct = left_typed
        .knowledge
        .ty()
        .map(|left_ty| ctx.resolve_dispatch_target(left_ty, &selector, left_typed.dispatch_lookup.clone()));

    if let Some(right_knowledge) = static_binary_operand_knowledge(ctx, &binary.right)
        && let Some(right_ty) = right_knowledge.ty()
        && let Some(reflected_selector) = reflected_binary_selector(&binary.op)
        && let ResolvedDispatchResult::Found(reflected) = ctx.resolve_dispatch_target(right_ty, &reflected_selector, crate::dispatch::DispatchLookup::Normal)
        && should_use_reflected_binary_target(ctx, &left_typed.knowledge, right_ty, &right_knowledge, direct.as_ref(), &reflected)
    {
        let right_typed = analyze_expression(ctx, &binary.right, &ExpectedType::None);
        let premise = CallPremise::from_typed(ctx, &right_typed);
        let target = CallableApplicationTarget::from_dispatch(reflected);
        let arguments = vec![super::call::ApplicationArgument::PreAnalyzed {
            label: if matches!(binary.op, BinaryOp::Compare) { None } else { Some("from") },
            typed: &left_typed,
            range: binary.left.range(),
        }];
        return apply_resolved_callable(ctx, &target, &premise, &arguments, &ExpectedType::None, binary.range).into();
    }

    let premise = CallPremise::from_typed(ctx, &left_typed);
    let arguments = vec![super::call::ApplicationArgument::Positional {
        expression: &binary.right,
        range: binary.right.range(),
    }];
    let Some(direct) = direct else {
        let reason = match &left_typed.knowledge {
            TypeKnowledge::Unknown(_) => UnresolvedApplicationReason::PremiseUnknown,
            TypeKnowledge::Dynamic(reason) => UnresolvedApplicationReason::PremiseDynamic(reason.clone()),
            TypeKnowledge::Known(_) => unreachable!("known binary receiver has a type"),
        };
        return analyze_unresolved_application(ctx, &premise, &arguments, reason).into();
    };
    match direct {
        ResolvedDispatchResult::Found(resolved) => {
            let target = CallableApplicationTarget::from_dispatch(resolved);
            apply_resolved_callable(ctx, &target, &premise, &arguments, &ExpectedType::None, binary.range).into()
        }
        ResolvedDispatchResult::Missing { .. } => {
            analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchMissing).into()
        }
        ResolvedDispatchResult::Ambiguous(_) => {
            analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchAmbiguous).into()
        }
        ResolvedDispatchResult::Dynamic => analyze_unresolved_application(
            ctx,
            &premise,
            &arguments,
            UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
        )
        .into(),
    }
}

fn static_binary_operand_knowledge(ctx: &mut CheckingContext<'_>, expr: &Expr) -> Option<TypeKnowledge> {
    match expr {
        Expr::Var { value, .. } => ctx.lookup_local_knowledge(value),
        Expr::Int { .. } => static_nominal_knowledge(ctx, "Int"),
        Expr::Float { .. } => static_nominal_knowledge(ctx, "Float"),
        Expr::String { .. } => static_nominal_knowledge(ctx, "String"),
        Expr::Boolean { .. } => static_nominal_knowledge(ctx, "Bool"),
        Expr::Symbol(_) => static_nominal_knowledge(ctx, "Symbol"),
        _ => None,
    }
}

fn static_nominal_knowledge(ctx: &mut CheckingContext<'_>, name: &str) -> Option<TypeKnowledge> {
    let declaration = ctx.resolve_type_name(name)?;
    Some(TypeKnowledge::established(ctx.nominal_type_of(&declaration), EvidenceOrigin::Syntax))
}

fn should_use_reflected_binary_target(
    ctx: &mut CheckingContext<'_>,
    left: &TypeKnowledge,
    right_ty: TypeId,
    right: &TypeKnowledge,
    direct: Option<&ResolvedDispatchResult>,
    reflected: &ResolvedDispatch,
) -> bool {
    if !binary_target_may_accept(ctx, left, &reflected.signature) {
        return false;
    }

    match direct {
        None | Some(ResolvedDispatchResult::Missing { .. }) => true,
        Some(ResolvedDispatchResult::Found(direct)) => {
            reflected_target_has_runtime_priority(ctx, left.ty(), right_ty, reflected) || binary_target_refutes(ctx, right, &direct.signature)
        }
        Some(ResolvedDispatchResult::Ambiguous(_) | ResolvedDispatchResult::Dynamic) => false,
    }
}

fn binary_target_may_accept(ctx: &mut CheckingContext<'_>, actual: &TypeKnowledge, signature: &CallableSignature) -> bool {
    if signature.generics.as_ref().is_some_and(|generics| !generics.parameters.is_empty()) {
        return true;
    }
    let Some(parameter) = signature.parameters.first() else {
        return false;
    };
    let Some(expected) = parameter.ty.ty() else {
        return true;
    };
    !matches!(ctx.check_knowledge_against_type(actual, expected), RelationOutcome::Refuted(_))
}

fn binary_target_refutes(ctx: &mut CheckingContext<'_>, actual: &TypeKnowledge, signature: &CallableSignature) -> bool {
    if signature.generics.as_ref().is_some_and(|generics| !generics.parameters.is_empty()) {
        return false;
    }
    let Some(expected) = signature.parameters.first().and_then(|parameter| parameter.ty.ty()) else {
        return false;
    };
    matches!(ctx.check_knowledge_against_type(actual, expected), RelationOutcome::Refuted(_))
}

fn reflected_target_has_runtime_priority(ctx: &CheckingContext<'_>, left_ty: Option<TypeId>, right_ty: TypeId, reflected: &ResolvedDispatch) -> bool {
    let Some(left) = left_ty.and_then(|ty| nominal_instance_declaration(ctx, ty)) else {
        return false;
    };
    let Some(right) = nominal_instance_declaration(ctx, right_ty) else {
        return false;
    };
    right != left && ctx.hierarchy.is_subclass(&right, &left) && reflected.callable.owner != left && ctx.hierarchy.is_subclass(&reflected.callable.owner, &left)
}

fn nominal_instance_declaration(ctx: &CheckingContext<'_>, mut ty: TypeId) -> Option<DeclarationId> {
    loop {
        match ctx.store.get(ty) {
            TypeData::Nominal { declaration } => return Some(declaration.clone()),
            TypeData::Applied { origin, .. } => ty = *origin,
            _ => return None,
        }
    }
}

fn reflected_binary_selector(op: &BinaryOp) -> Option<Selector> {
    let (name, compare) = match op {
        BinaryOp::Add => ("+", false),
        BinaryOp::Subtract => ("-", false),
        BinaryOp::Multiply => ("*", false),
        BinaryOp::Divide => ("/", false),
        BinaryOp::IntegerDivide => ("//", false),
        BinaryOp::Power => ("**", false),
        BinaryOp::Modulo => ("%", false),
        BinaryOp::ShiftLeft => ("<<", false),
        BinaryOp::ShiftRight => (">>", false),
        BinaryOp::BitAnd => ("&", false),
        BinaryOp::BitXor => ("^", false),
        BinaryOp::BitOr => ("|", false),
        BinaryOp::Compare => ("compare", true),
        _ => return None,
    };
    if compare {
        Selector::method(name, vec![SelectorSlot::Positional]).ok()
    } else {
        Selector::method(name, vec![SelectorSlot::Label("from".to_string())]).ok()
    }
}

fn synthesize_unary_expr(ctx: &mut CheckingContext<'_>, unary: &UnaryExpr) -> TypedExpression {
    let operand_typed = analyze_expression(ctx, &unary.expr, &ExpectedType::None);
    let premise = CallPremise::from_typed(ctx, &operand_typed);

    let op_name = match unary.op {
        UnaryOp::Plus => "+",
        UnaryOp::Minus => "-",
        UnaryOp::Not => "not",
        UnaryOp::BitNot => "~",
    };

    let Some(operand_ty) = operand_typed.knowledge.ty() else {
        let reason = match &operand_typed.knowledge {
            TypeKnowledge::Unknown(_) => UnresolvedApplicationReason::PremiseUnknown,
            TypeKnowledge::Dynamic(reason) => UnresolvedApplicationReason::PremiseDynamic(reason.clone()),
            TypeKnowledge::Known(_) => unreachable!("known unary receiver has a type"),
        };
        return analyze_unresolved_application(ctx, &premise, &[], reason).into();
    };
    let Ok(selector) = Selector::getter(op_name) else {
        return analyze_unresolved_application(ctx, &premise, &[], UnresolvedApplicationReason::DispatchMissing).into();
    };
    match ctx.resolve_dispatch_target(operand_ty, &selector, operand_typed.dispatch_lookup.clone()) {
        ResolvedDispatchResult::Found(resolved) => {
            let target = CallableApplicationTarget::from_dispatch(resolved);
            apply_resolved_callable(ctx, &target, &premise, &[], &ExpectedType::None, unary.range).into()
        }
        ResolvedDispatchResult::Missing { .. } => analyze_unresolved_application(ctx, &premise, &[], UnresolvedApplicationReason::DispatchMissing).into(),
        ResolvedDispatchResult::Ambiguous(_) => analyze_unresolved_application(ctx, &premise, &[], UnresolvedApplicationReason::DispatchAmbiguous).into(),
        ResolvedDispatchResult::Dynamic => analyze_unresolved_application(
            ctx,
            &premise,
            &[],
            UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
        )
        .into(),
    }
}

fn synthesize_get_property(ctx: &mut CheckingContext<'_>, get: &GetPropertyExpr) -> TypedExpression {
    let recv_typed = analyze_expression(ctx, &get.object, &ExpectedType::None);
    let premise = CallPremise::from_typed(ctx, &recv_typed);
    let Some(recv_ty) = recv_typed.knowledge.ty() else {
        let reason = match &recv_typed.knowledge {
            TypeKnowledge::Unknown(_) => {
                if matches!(recv_typed.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Suppressed(_)) {
                    UnresolvedApplicationReason::PremiseInvalidUnavailable
                } else {
                    UnresolvedApplicationReason::PremiseUnknown
                }
            }
            TypeKnowledge::Dynamic(reason) => UnresolvedApplicationReason::PremiseDynamic(reason.clone()),
            TypeKnowledge::Known(_) => unreachable!("known property receiver has a type"),
        };
        return analyze_unresolved_application(ctx, &premise, &[], reason).into();
    };

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
        match ctx.resolve_dispatch_target(recv_ty, &sel, recv_typed.dispatch_lookup.clone()) {
            ResolvedDispatchResult::Found(resolved) => {
                let target = CallableApplicationTarget::from_dispatch(resolved);
                return apply_resolved_callable(ctx, &target, &premise, &[], &ExpectedType::None, get.range).into();
            }
            ResolvedDispatchResult::Missing { .. } => {}
            ResolvedDispatchResult::Ambiguous(_) => {
                return analyze_unresolved_application(ctx, &premise, &[], UnresolvedApplicationReason::DispatchAmbiguous).into();
            }
            ResolvedDispatchResult::Dynamic => {
                return analyze_unresolved_application(
                    ctx,
                    &premise,
                    &[],
                    UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
                )
                .into();
            }
        }
    }
    analyze_unresolved_application(ctx, &premise, &[], UnresolvedApplicationReason::DispatchMissing).into()
}

fn synthesize_set_property(ctx: &mut CheckingContext<'_>, set: &SetPropertyExpr) -> TypedExpression {
    let recv_typed = analyze_expression(ctx, &set.object, &ExpectedType::None);
    let premise = CallPremise::from_typed(ctx, &recv_typed);
    let Some(recv_ty) = recv_typed.knowledge.ty() else {
        let operation = analyze_unresolved_application(
            ctx,
            &premise,
            &[super::call::ApplicationArgument::Positional {
                expression: &set.value,
                range: set.value.range(),
            }],
            match &recv_typed.knowledge {
                TypeKnowledge::Unknown(_) => {
                    if matches!(recv_typed.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Suppressed(_)) {
                        UnresolvedApplicationReason::PremiseInvalidUnavailable
                    } else {
                        UnresolvedApplicationReason::PremiseUnknown
                    }
                }
                TypeKnowledge::Dynamic(reason) => UnresolvedApplicationReason::PremiseDynamic(reason.clone()),
                TypeKnowledge::Known(_) => unreachable!("known property receiver has a type"),
            },
        );
        return super::call::assignment_result_from_call(ctx, operation, set.range);
    };

    // 1. Check field
    let field_opt = match ctx.store.get(recv_ty).clone() {
        TypeData::ClassObject { declaration } => ctx.get_field(&declaration, crate::identity::DispatchSide::Class, &set.property),
        TypeData::Nominal { declaration } => ctx.get_field(&declaration, crate::identity::DispatchSide::Instance, &set.property),
        _ => None,
    };
    if let Some(field_k) = field_opt {
        let expected_value = field_k
            .ty()
            .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::AssignmentContract))
            .unwrap_or_default();
        let value_typed = analyze_expression(ctx, &set.value, &expected_value);
        let application = ctx.apply_assignability(
            &value_typed.knowledge,
            &field_k,
            DiagnosticCode::FieldMismatch,
            format!("assigned value does not match field `{}` type", set.property),
            set.range,
        );
        let mut result = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, set.range);
        crate::checker::composition::propagate_required_dependencies(&mut result, &[recv_typed, value_typed]);
        apply_relation_application_to_typed(&mut result, &application);
        return result;
    }

    // 2. Check setter selector
    if let Ok(sel) = Selector::setter(&set.property) {
        match ctx.resolve_dispatch_target(recv_ty, &sel, recv_typed.dispatch_lookup.clone()) {
            ResolvedDispatchResult::Found(resolved) => {
                let arguments = vec![super::call::ApplicationArgument::Positional {
                    expression: &set.value,
                    range: set.value.range(),
                }];
                let target = CallableApplicationTarget::from_dispatch(resolved);
                let operation = apply_resolved_callable(ctx, &target, &premise, &arguments, &ExpectedType::None, set.range);
                return super::call::assignment_result_from_call(ctx, operation, set.range);
            }
            ResolvedDispatchResult::Missing { .. } => {}
            ResolvedDispatchResult::Ambiguous(_) => {
                let operation = analyze_unresolved_application(
                    ctx,
                    &premise,
                    &[super::call::ApplicationArgument::Positional {
                        expression: &set.value,
                        range: set.value.range(),
                    }],
                    UnresolvedApplicationReason::DispatchAmbiguous,
                );
                return super::call::assignment_result_from_call(ctx, operation, set.range);
            }
            ResolvedDispatchResult::Dynamic => {
                let operation = analyze_unresolved_application(
                    ctx,
                    &premise,
                    &[super::call::ApplicationArgument::Positional {
                        expression: &set.value,
                        range: set.value.range(),
                    }],
                    UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
                );
                return super::call::assignment_result_from_call(ctx, operation, set.range);
            }
        }
    }
    let operation = analyze_unresolved_application(
        ctx,
        &premise,
        &[super::call::ApplicationArgument::Positional {
            expression: &set.value,
            range: set.value.range(),
        }],
        UnresolvedApplicationReason::DispatchMissing,
    );
    super::call::assignment_result_from_call(ctx, operation, set.range)
}

fn synthesize_index_expr(ctx: &mut CheckingContext<'_>, idx: &IndexExpr) -> TypedExpression {
    let recv_typed = analyze_expression(ctx, &idx.object, &ExpectedType::None);
    let premise = CallPremise::from_typed(ctx, &recv_typed);
    let arguments = application_arguments(&idx.args);
    let Some(recv_ty) = recv_typed.knowledge.ty() else {
        let reason = match &recv_typed.knowledge {
            TypeKnowledge::Unknown(_) => {
                if matches!(recv_typed.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Suppressed(_)) {
                    UnresolvedApplicationReason::PremiseInvalidUnavailable
                } else {
                    UnresolvedApplicationReason::PremiseUnknown
                }
            }
            TypeKnowledge::Dynamic(reason) => UnresolvedApplicationReason::PremiseDynamic(reason.clone()),
            TypeKnowledge::Known(_) => unreachable!("known subscript receiver has a type"),
        };
        return analyze_unresolved_application(ctx, &premise, &arguments, reason).into();
    };
    let slots = match super::call::static_call_shape(&arguments) {
        StaticCallShape::Exact(slots) => slots,
        StaticCallShape::Dynamic(reason) => {
            return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DynamicShape(reason)).into();
        }
    };
    let Ok(selector) = Selector::subscript_get(slots) else {
        return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchMissing).into();
    };
    match ctx.resolve_dispatch_target(recv_ty, &selector, recv_typed.dispatch_lookup.clone()) {
        ResolvedDispatchResult::Found(resolved) => {
            let target = CallableApplicationTarget::from_dispatch(resolved);
            return apply_resolved_callable(ctx, &target, &premise, &arguments, &ExpectedType::None, idx.range).into();
        }
        ResolvedDispatchResult::Missing { .. } => {}
        ResolvedDispatchResult::Ambiguous(_) => {
            return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchAmbiguous).into();
        }
        ResolvedDispatchResult::Dynamic => {
            return analyze_unresolved_application(
                ctx,
                &premise,
                &arguments,
                UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
            )
            .into();
        }
    }
    if let Some(target) = super::call::structural_list_index_get_target(ctx, recv_ty).or_else(|| super::call::structural_map_index_get_target(ctx, recv_ty)) {
        return apply_resolved_callable(ctx, &target, &premise, &arguments, &ExpectedType::None, idx.range).into();
    }
    analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchMissing).into()
}

fn synthesize_set_index_expr(ctx: &mut CheckingContext<'_>, set_idx: &SetIndexExpr) -> TypedExpression {
    let recv_typed = analyze_expression(ctx, &set_idx.object, &ExpectedType::None);
    let premise = CallPremise::from_typed(ctx, &recv_typed);
    let index_arguments = application_arguments(&set_idx.args);
    let mut all_arguments = index_arguments.clone();
    all_arguments.push(super::call::ApplicationArgument::Positional {
        expression: &set_idx.value,
        range: set_idx.value.range(),
    });
    let Some(recv_ty) = recv_typed.knowledge.ty() else {
        let reason = match &recv_typed.knowledge {
            TypeKnowledge::Unknown(_) => {
                if matches!(recv_typed.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Suppressed(_)) {
                    UnresolvedApplicationReason::PremiseInvalidUnavailable
                } else {
                    UnresolvedApplicationReason::PremiseUnknown
                }
            }
            TypeKnowledge::Dynamic(reason) => UnresolvedApplicationReason::PremiseDynamic(reason.clone()),
            TypeKnowledge::Known(_) => unreachable!("known subscript receiver has a type"),
        };
        let operation = analyze_unresolved_application(ctx, &premise, &all_arguments, reason);
        return super::call::assignment_result_from_call(ctx, operation, set_idx.range);
    };
    let slots = match super::call::static_call_shape(&index_arguments) {
        StaticCallShape::Exact(slots) => slots,
        StaticCallShape::Dynamic(reason) => {
            let operation = analyze_unresolved_application(ctx, &premise, &all_arguments, UnresolvedApplicationReason::DynamicShape(reason));
            return super::call::assignment_result_from_call(ctx, operation, set_idx.range);
        }
    };
    let Ok(selector) = Selector::subscript_set(slots) else {
        let operation = analyze_unresolved_application(ctx, &premise, &all_arguments, UnresolvedApplicationReason::DispatchMissing);
        return super::call::assignment_result_from_call(ctx, operation, set_idx.range);
    };
    match ctx.resolve_dispatch_target(recv_ty, &selector, recv_typed.dispatch_lookup.clone()) {
        ResolvedDispatchResult::Found(resolved) => {
            let target = CallableApplicationTarget::from_dispatch(resolved);
            let operation = apply_resolved_callable(ctx, &target, &premise, &all_arguments, &ExpectedType::None, set_idx.range);
            return super::call::assignment_result_from_call(ctx, operation, set_idx.range);
        }
        ResolvedDispatchResult::Missing { .. } => {}
        ResolvedDispatchResult::Ambiguous(_) => {
            let operation = analyze_unresolved_application(ctx, &premise, &all_arguments, UnresolvedApplicationReason::DispatchAmbiguous);
            return super::call::assignment_result_from_call(ctx, operation, set_idx.range);
        }
        ResolvedDispatchResult::Dynamic => {
            let operation = analyze_unresolved_application(
                ctx,
                &premise,
                &all_arguments,
                UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
            );
            return super::call::assignment_result_from_call(ctx, operation, set_idx.range);
        }
    }
    if let Some(target) = super::call::structural_list_index_set_target(ctx, recv_ty) {
        let operation = apply_resolved_callable(ctx, &target, &premise, &all_arguments, &ExpectedType::None, set_idx.range);
        return super::call::assignment_result_from_call(ctx, operation, set_idx.range);
    }
    let operation = analyze_unresolved_application(ctx, &premise, &all_arguments, UnresolvedApplicationReason::DispatchMissing);
    super::call::assignment_result_from_call(ctx, operation, set_idx.range)
}

fn bind_pattern(ctx: &mut CheckingContext<'_>, pattern: &Pattern, fact: ValueSemanticFact, causal_invalidity: crate::checker::causal::CausalInvalidity) {
    match pattern {
        Pattern::Name { name, range, .. } => {
            ctx.bind_pattern_binding_with_causal(name.clone(), fact, *range, causal_invalidity);
        }
        Pattern::Tuple { elements, .. } => {
            for (index, element) in elements.iter().enumerate() {
                let component = ValueSemanticFact::new(crate::checker::composition::decompose_tuple_component(
                    ctx.store,
                    &fact.knowledge,
                    index,
                    elements.len(),
                ));
                bind_pattern(ctx, element, component, causal_invalidity);
            }
        }
        _ => {}
    }
}
