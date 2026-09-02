//! Expression type synthesis, bidirectional checking, and inference engine (Spec 04.5 / Wave 3).

use super::call::{
    CallPremise, CallTargetAuthority, CallableApplicationTarget, StaticCallShape, UnresolvedApplicationReason, analyze_non_callable_invocation,
    analyze_unresolved_application, application_arguments, apply_resolved_callable, callable_value_target, static_call_shape,
};
use super::context::CheckingContext;
use super::expected::{ExpectationOrigin, ExpectedType};
use super::statement::check_statement;
use super::typed_expr::TypedExpression;
use crate::associated::AssociatedMemberId;
use crate::checker::analysis::AnalysisStatus;
use crate::checker::associated::{
    AssociatedResolution, AssociatedResolutionKind, BehavioralFamilySpec, FamilyApplicationCandidate, FamilyApplicationResolution, FamilyApplicationSelection,
    check_reification_underconstrained, resolve_associated_owner, resolve_bound_behavioral_family, resolve_effective_associated_family,
    specialize_associated_member,
};
use crate::checker::binding::{BindingConsistency, BindingWriteResult, reconcile_binding_relation};
use crate::checker::causal::CausalInvalidity;
use crate::checker::flow::FlowState;
use crate::diagnostic::DiagnosticCode;
use crate::dispatch::{CallableSignature, ResolvedDispatch, ResolvedDispatchResult};
use crate::identity::{DeclarationId, InvocationTargetId};
use crate::types::annotation::TypeResolver;
use crate::types::denotation::{AssociatedValueDenotation, CapturedBehavioralMember, SemanticDenotation, ValueSemanticFact};
use crate::types::environment::TypeView;
use crate::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::family::{FamilyMemberTypeKind, FamilyOperationShape};
use crate::types::id::{KindId, TypeId};
use crate::types::outcome::{BlockReason, DynamicBoundaryObligation, RelationOutcome};
use crate::types::relation::{TypeHierarchy, is_subtype};
use crate::types::store::{RecordTypeField, TupleTypeElement, TypeData};
use phalcom_ast::ast::{
    AssociatedInvokeExpr, AssociatedLookupExpr, AssociatedMemberSyntax, AssociatedNamedMode, AssociatedResidualSelectorSyntax, BinaryExpr, BinaryOp,
    ComparisonChainExpr, Expr, GetPropertyExpr, IndexExpr, IsMembershipExpr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, MembershipExpr,
    MethodCallExpr, PackItem, PackLabel, Pattern, ProductLabel, RecordLiteralEntry, RelationOp, SetIndexExpr, SetLiteralEntry, SetPropertyExpr, Statement,
    SymbolExpr, SymbolLiteralKind, TupleLiteralEntry, UnaryExpr, UnaryOp, UnqualifiedCallExpr,
};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorBase, SelectorPattern, SelectorSlot};

/// Central entry point for bidirectional expression analysis (Spec 04.5 / E4).
pub fn analyze_expression(ctx: &mut CheckingContext<'_>, expr: &Expr, expected: &ExpectedType) -> TypedExpression {
    let expr_id = ctx.alloc_expression_id();
    ctx.push_expression_owner(expr_id);
    let mut typed = analyze_expression_inner(ctx, expr, expected);
    if let Some(cause_id) = ctx.pop_expression_owner(expr_id) {
        typed.invalidate(cause_id);
    }

    if let (Some(expected_ty), Some(origin)) = (expected.ty(), expected.origin()) {
        let requirement = crate::explain::ExplanationStep::TypeRequirement {
            expected: expected_ty,
            origin,
            source: Some(crate::diagnostic::SemanticSourceSpan::new(ctx.current_module.clone(), expr.range())),
        };
        let requirement_id = ctx.record_derivation(
            requirement,
            crate::explain::DerivationRule::TypeRequirement,
            EvidenceStatus::Established,
            EvidenceOrigin::DeveloperAnnotation,
            vec![
                crate::explain::EvidenceRef::SourceSpan(expr.range()),
                crate::explain::EvidenceRef::TypeId(expected_ty),
            ],
            Vec::new(),
        );
        typed.explanation_parents.push(requirement_id);
    }

    let step = match &typed.knowledge {
        TypeKnowledge::Known(_) => {
            let ty = typed.knowledge.ty().expect("known expression has type");
            match expr {
                Expr::Int { .. } | Expr::Float { .. } | Expr::String { .. } | Expr::Boolean { .. } => {
                    crate::explain::ExplanationStep::Literal { expression: expr_id, ty }
                }
                Expr::Var { value, .. } => match ctx.lookup_binding_info(value) {
                    Some(info) => crate::explain::ExplanationStep::BindingRead {
                        expression: expr_id,
                        binding: info.id,
                        knowledge: typed.knowledge.clone(),
                    },
                    None => crate::explain::ExplanationStep::ExpressionResult {
                        expression: expr_id,
                        knowledge: typed.knowledge.clone(),
                    },
                },
                Expr::MethodCall(_) => match typed.callable.clone() {
                    Some(callable) => crate::explain::ExplanationStep::MethodCall {
                        call: expr_id,
                        callable,
                        return_ty: ty,
                    },
                    None => crate::explain::ExplanationStep::UnresolvedCall { call: expr_id, return_ty: ty },
                },
                Expr::ListLiteral(_) => crate::explain::ExplanationStep::CollectionSynthesis {
                    expression: expr_id,
                    kind: crate::explain::CollectionKind::List,
                    element_types: Box::new([]),
                    result: ty,
                },
                Expr::SetLiteral(_) => crate::explain::ExplanationStep::CollectionSynthesis {
                    expression: expr_id,
                    kind: crate::explain::CollectionKind::Set,
                    element_types: Box::new([]),
                    result: ty,
                },
                Expr::MapLiteral(_) => crate::explain::ExplanationStep::CollectionSynthesis {
                    expression: expr_id,
                    kind: crate::explain::CollectionKind::Map,
                    element_types: Box::new([]),
                    result: ty,
                },
                Expr::TupleLiteral(_) => crate::explain::ExplanationStep::CollectionSynthesis {
                    expression: expr_id,
                    kind: crate::explain::CollectionKind::Tuple,
                    element_types: Box::new([]),
                    result: ty,
                },
                Expr::RecordLiteral(_) => crate::explain::ExplanationStep::CollectionSynthesis {
                    expression: expr_id,
                    kind: crate::explain::CollectionKind::Record,
                    element_types: Box::new([]),
                    result: ty,
                },
                _ => crate::explain::ExplanationStep::ExpressionResult {
                    expression: expr_id,
                    knowledge: typed.knowledge.clone(),
                },
            }
        }
        TypeKnowledge::Unknown(reason) => crate::explain::ExplanationStep::UnknownBoundary {
            reason: reason.clone(),
            source: Some(expr.range()),
        },
        TypeKnowledge::Dynamic(reason) => crate::explain::ExplanationStep::DynamicBoundary {
            reason: reason.clone(),
            source: Some(expr.range()),
        },
    };
    let rule = step.derivation_rule();
    let mut ev = vec![crate::explain::EvidenceRef::SourceSpan(expr.range())];
    if let Some(ty) = typed.knowledge.ty() {
        ev.push(crate::explain::EvidenceRef::TypeId(ty));
    }
    let status = typed.knowledge.status().unwrap_or(EvidenceStatus::Assumed);
    let origin = typed.knowledge.origin().unwrap_or(EvidenceOrigin::Syntax);
    let explanation_id = Some(ctx.record_derivation(step, rule, status, origin, ev, typed.explanation_parents.clone()));
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
            let int_decl = ctx.core_ids.int.clone();
            if let Some(ty) = ctx.core_type(&int_decl) {
                TypedExpression::established(ty, EvidenceOrigin::Syntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause)
            }
        }
        Expr::Float { range, .. } => {
            let float_decl = ctx.core_ids.float.clone();
            if let Some(ty) = ctx.core_type(&float_decl) {
                TypedExpression::established(ty, EvidenceOrigin::Syntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause)
            }
        }
        Expr::String { range, .. } => {
            let string_decl = ctx.core_ids.string.clone();
            if let Some(ty) = ctx.core_type(&string_decl) {
                TypedExpression::established(ty, EvidenceOrigin::Syntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause)
            }
        }
        Expr::Boolean { range, .. } => {
            let bool_decl = ctx.core_ids.bool_.clone();
            if let Some(ty) = ctx.core_type(&bool_decl) {
                TypedExpression::established(ty, EvidenceOrigin::Syntax, *range)
            } else {
                TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause)
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
                        if let Some(explanation) = state.explanation {
                            typed.explanation_parents.push(explanation);
                        }
                    }
                }
                typed
            } else if let Some(decl) = ctx.resolve_type_name(value) {
                if let Some(info) = ctx.declaration_info(&decl) {
                    TypedExpression::established(info.class_object_type, EvidenceOrigin::DeclarationSemantics, *range)
                        .with_denotation(SemanticDenotation::TypeForm(info.form))
                } else if let Some(form) = ctx.resolver.resolve_alias_form(&decl) {
                    let Some(value_type) = type_form_descriptor_type(ctx, form) else {
                        return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
                    };
                    TypedExpression::established(value_type, EvidenceOrigin::DeclarationSemantics, *range).with_denotation(SemanticDenotation::TypeForm(form))
                } else {
                    TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause)
                }
            } else if let Some(param_ty) = ctx.resolve_type_parameter(value) {
                TypedExpression::established(param_ty, EvidenceOrigin::DeclarationSemantics, *range).with_denotation(SemanticDenotation::TypeForm(param_ty))
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
                        TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause)
                    }
                } else {
                    let Some(ty) = ctx.instance_type_of(&class_decl) else {
                        return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
                    };
                    TypedExpression::established(ty, EvidenceOrigin::Flow, *range)
                }
            } else {
                TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause)
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
                        TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause).with_dispatch_lookup(lookup)
                    }
                } else {
                    let Some(ty) = ctx.instance_type_of(&class_decl) else {
                        return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause).with_dispatch_lookup(lookup);
                    };
                    TypedExpression::established(ty, EvidenceOrigin::Flow, *range).with_dispatch_lookup(lookup)
                }
            } else {
                TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause)
            }
        }
        Expr::Field { value, range, .. } => {
            if let Some(class_decl) = ctx.current_class.clone() {
                if let Some((_, field_k, field_causal)) = ctx.resolve_current_field(&class_decl, ctx.current_side, value) {
                    let mut typed = TypedExpression::new(field_k.with_range(*range));
                    typed.causal_invalidity = field_causal;
                    return typed;
                }
            }
            TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause)
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
                        let reconciliation = super::field_lifecycle::reconcile_field_write(&field_k, &val_typed.knowledge, &application.outcome);
                        let write_causal = val_typed.causal_invalidity.join(result.causal_invalidity);
                        ctx.write_current_field(field_id, field_k, reconciliation.current, reconciliation.validity, write_causal);
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
            let outer_normal_returns = ctx.take_normal_return_exits();
            let outer_throw_exits = std::mem::take(&mut ctx.throw_exit_flows);
            ctx.push_scope();
            let (expected_params, expected_ret) = expected.callable_signature(ctx.store).unwrap_or_default();

            let object_decl = ctx.core_ids.object.clone();
            let Some(top) = ctx.core_type(&object_decl) else {
                ctx.pop_scope();
                ctx.normal_return_exits = outer_normal_returns;
                ctx.throw_exit_flows = outer_throw_exits;
                ctx.expected_return = outer_expected_return;
                return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
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
                    rest: phalcom_ast::ast::RestMode::None,
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
                    rest: phalcom_ast::ast::RestMode::Positional,
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
            let block_normal_returns = ctx.take_normal_return_exits();
            let _block_throw_exits = std::mem::take(&mut ctx.throw_exit_flows);
            ctx.normal_return_exits = outer_normal_returns;
            ctx.throw_exit_flows = outer_throw_exits;

            // Constructing a closure does not execute its body. Keep facts
            // produced while checking the captured body inside that body.
            ctx.flow = outer_flow;

            let mut block_return_values = block_normal_returns.into_iter().map(|f| f.knowledge).collect::<Vec<_>>();
            if tail_typed.knowledge.ty() != Some(ctx.store.never()) && (block_return_values.is_empty() || tail_typed.knowledge.ty() != Some(ctx.store.unit())) {
                block_return_values.push(tail_typed.knowledge.clone());
            }
            let closure_return_knowledge = if block_return_values.is_empty() {
                tail_typed.knowledge
            } else if block_return_values.len() == 1 {
                block_return_values.remove(0)
            } else {
                crate::types::evidence::join_type_knowledge(ctx.store, block_return_values)
            };

            let Some(return_type) = closure_return_knowledge.ty() else {
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
            let before = ctx.flow.clone();

            ctx.flow = before.clone();
            let pattern = &if_let.pattern;
            let fact = val_typed.fact();
            let causal = val_typed.causal_invalidity;
            let then_result = super::control::analyze_executable_region_with_prelude(ctx, &if_let.then_body.body, if_let.then_body.range, expected, |ctx| {
                bind_pattern(ctx, pattern, fact, causal);
            });

            ctx.flow = before.clone();
            let else_result = if let Some(ref else_body) = if_let.else_body {
                super::control::analyze_executable_region(ctx, &else_body.body, else_body.range, expected)
            } else {
                super::control::ExecutableRegionResult {
                    value: Some(TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Syntax, if_let.range)),
                    flow: before,
                    causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
                }
            };

            super::control::join_branch_results(ctx, val_typed.causal_invalidity, &then_result, &else_result, if_let.range)
        }
        Expr::WhileLet(while_let) => {
            let before = ctx.flow.clone();

            let evaluate_step = |step_ctx: &mut CheckingContext<'_>,
                                 current_header: &FlowState|
             -> (Option<FlowState>, FlowState, Vec<FlowState>, CausalInvalidity) {
                step_ctx.flow = current_header.clone();
                let val_typed = analyze_expression(step_ctx, &while_let.value, &ExpectedType::None);
                let exit_flow = step_ctx.flow.clone();
                let pattern = &while_let.pattern;
                let fact = val_typed.fact();
                let causal = val_typed.causal_invalidity;

                step_ctx.push_loop_frame();
                let body_res = super::control::analyze_executable_region_with_prelude(step_ctx, &while_let.body, while_let.range, &ExpectedType::None, |ctx| {
                    bind_pattern(ctx, pattern, fact, causal);
                });
                let body_flow = step_ctx.flow.clone();
                let loop_frame = step_ctx.pop_loop_frame();
                let normal_backedge = if body_res.completes_normally() { Some(body_flow) } else { None };
                let mut continues = loop_frame.continues;
                let breaks = loop_frame.breaks;
                continues.extend(normal_backedge);
                let backedge = if continues.is_empty() {
                    None
                } else if continues.len() == 1 {
                    continues.pop()
                } else {
                    step_ctx.join_flow_states(&continues).ok()
                };

                (backedge, exit_flow, breaks, val_typed.causal_invalidity.join(body_res.causal_invalidity))
            };

            let fixpoint = match super::loop_analysis::solve_loop_header(ctx, &before, |probe_ctx, current_header| {
                let (backedge, _exit_flow, breaks, _causal) = evaluate_step(probe_ctx, current_header);
                super::loop_analysis::LoopStepResult {
                    normal_backedge: backedge,
                    continues: Vec::new(),
                    breaks,
                }
            }) {
                Ok(fp) => fp,
                Err(failure) => {
                    let status = ctx.publish_flow_join_failure(failure, while_let.range);
                    let mut typed = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, while_let.range);
                    typed.status = status;
                    return typed;
                }
            };

            // Final real pass at stable header
            let (_backedge, exit_flow, breaks, causal) = evaluate_step(ctx, &fixpoint.header);

            let mut exit_states = Vec::new();
            if exit_flow.is_reachable() {
                exit_states.push(exit_flow);
            }
            for brk in breaks {
                if brk.is_reachable() {
                    exit_states.push(brk);
                }
            }

            let join_status = if exit_states.is_empty() {
                ctx.flow = FlowState::unreachable();
                None
            } else if exit_states.len() == 1 {
                ctx.flow = exit_states.pop().unwrap();
                None
            } else {
                match ctx.join_flow_states(&exit_states) {
                    Ok(flow) => {
                        ctx.flow = flow;
                        None
                    }
                    Err(failure) => Some(ctx.publish_flow_join_failure(failure, while_let.range)),
                }
            };

            let mut typed = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, while_let.range);
            if let Some(status) = join_status {
                typed.status = status;
            }
            typed.causal_invalidity = causal;
            typed
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

        // --- 7.5 Static Associated Lookup ---
        Expr::AssociatedLookup(lookup) => synthesize_associated_lookup(ctx, lookup, expected),
        Expr::AssociatedInvoke(invoke) => synthesize_associated_invoke(ctx, invoke, expected),

        // --- 7.6 Match Expression ---
        Expr::Match(match_expr) => synthesize_match_expr(ctx, match_expr, expected),

        // --- 8. Miscellaneous Expressions ---
        Expr::ComparisonChain(chain) => synthesize_comparison_chain(ctx, chain),
        Expr::Membership(m) => synthesize_membership_expr(ctx, m),
        Expr::IsMembership(m) => synthesize_is_membership_expr(ctx, m),
        Expr::Range(r) => {
            if let Some(ref lower) = r.lower {
                analyze_expression(ctx, lower, &ExpectedType::None);
            }
            if let Some(ref upper) = r.upper {
                analyze_expression(ctx, upper, &ExpectedType::None);
            }
            let object_decl = ctx.core_ids.object.clone();
            let Some(ty) = ctx.core_type(&object_decl) else {
                return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
            };
            TypedExpression::established(ty, EvidenceOrigin::Flow, r.range)
        }
        Expr::Ellipsis { .. } => TypedExpression::unknown(UnknownReason::UncheckedExpression),
        Expr::TypeForm(annotation) => {
            let resolver = ctx.resolver.inner();
            let site = if let Some(owner) = ctx.current_class.clone() {
                crate::types::annotation::TypeFormationSite::member(ctx.current_module.clone(), owner, ctx.current_side)
            } else {
                crate::types::annotation::TypeFormationSite::module(ctx.current_module.clone())
            };
            let (resolution, causal_invalidity) = ctx.resolve_type_form(resolver, &site, annotation);
            let mut typed = match resolution {
                crate::types::annotation::TypeFormResolution::Ready(form) => {
                    let denotation = SemanticDenotation::TypeForm(form);
                    match type_form_descriptor_type(ctx, form) {
                        Some(value_type) => {
                            TypedExpression::established(value_type, EvidenceOrigin::DeclarationSemantics, annotation.range).with_denotation(denotation)
                        }
                        None => TypedExpression::new(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)).with_denotation(denotation),
                    }
                }
                crate::types::annotation::TypeFormResolution::Dynamic => TypedExpression::new(TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)),
                crate::types::annotation::TypeFormResolution::Unresolved(reason) => TypedExpression::new(TypeKnowledge::Unknown(match reason {
                    crate::types::annotation::TypeFormationUnresolved::Name(name) => UnknownReason::UnresolvedName(name),
                    crate::types::annotation::TypeFormationUnresolved::SelfOutsideOwner => UnknownReason::UnresolvedName("Self".into()),
                })),
                crate::types::annotation::TypeFormResolution::Missing(_) | crate::types::annotation::TypeFormResolution::Invalid(_) => {
                    TypedExpression::new(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause))
                }
                crate::types::annotation::TypeFormResolution::Blocked(_) => TypedExpression::new(TypeKnowledge::Unknown(UnknownReason::InferenceBlocked)),
                crate::types::annotation::TypeFormResolution::Cancelled => TypedExpression::new(TypeKnowledge::Unknown(UnknownReason::InferenceCancelled)),
                crate::types::annotation::TypeFormResolution::BudgetExceeded(_) => {
                    TypedExpression::new(TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded))
                }
                crate::types::annotation::TypeFormResolution::InternalFailure(_) => {
                    TypedExpression::new(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause))
                }
            };
            typed.knowledge = typed.knowledge.clone().with_range(annotation.range);
            typed.causal_invalidity = causal_invalidity;
            typed
        }
        _ => TypedExpression::unknown(UnknownReason::UncheckedExpression),
    }
}

/// Returns ordinary runtime value type for a type-form value, without
/// confusing its denoted form with its descriptor.
fn type_form_descriptor_type(ctx: &mut CheckingContext<'_>, form: TypeId) -> Option<TypeId> {
    let mut current = form;
    let declaration = loop {
        match ctx.store.get(current) {
            TypeData::Nominal { declaration } => break Some(declaration.clone()),
            TypeData::Applied { origin, .. } => current = *origin,
            _ => break None,
        }
    };
    if let Some(declaration) = declaration {
        return ctx.declaration_info(&declaration).map(|info| info.class_object_type);
    }

    let class = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Class);
    ctx.declaration_info(&class).map(|info| info.form)
}

fn synthesize_associated_lookup(ctx: &mut CheckingContext<'_>, lookup: &AssociatedLookupExpr, expected: &ExpectedType) -> TypedExpression {
    let receiver = analyze_expression(ctx, &lookup.receiver, &ExpectedType::None);

    let is_getter_only = match &lookup.member {
        AssociatedMemberSyntax::Named(named) => matches!(
            named.mode,
            AssociatedNamedMode::Getter {
                explicit_separator_range: None
            }
        ),
        _ => false,
    };

    let owner = match receiver.denotation.as_ref() {
        Some(SemanticDenotation::TypeForm(_)) => match resolve_associated_owner(ctx, &receiver, lookup.range) {
            Ok(owner) => Some(owner),
            Err(_) => return TypedExpression::unknown(UnknownReason::UncheckedExpression),
        },
        _ => {
            if is_getter_only {
                let _ = resolve_associated_owner(ctx, &receiver, lookup.range);
                return TypedExpression::unknown(UnknownReason::UncheckedExpression);
            }
            None
        }
    };

    match &lookup.member {
        AssociatedMemberSyntax::Named(member) => {
            let base = SelectorBase::Named(member.base.clone());
            let Some(owner) = owner else {
                return synthesize_bound_behavioral_lookup(ctx, receiver, lookup, expected);
            };
            let has_associated_base = ctx
                .associated_surface(&owner.lookup_owner)
                .is_some_and(|surface| surface.families.contains_key(&base));
            if !has_associated_base {
                return synthesize_bound_behavioral_lookup(ctx, receiver, lookup, expected);
            }
            let Ok(family) = resolve_effective_associated_family(ctx, &owner, &base, lookup.range) else {
                return TypedExpression::unknown(UnknownReason::UncheckedExpression);
            };

            match &member.mode {
                AssociatedNamedMode::Getter { .. } => {
                    let Ok(selector) = Selector::getter(&member.base) else {
                        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                    };
                    let Some(member_id) = family
                        .members
                        .iter()
                        .find(|m| match m {
                            AssociatedMemberId::Variant(v) => v.selector == selector,
                        })
                        .cloned()
                    else {
                        ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::AssociatedMemberMissing,
                            format!("getter `{}` not found in associated family `{}`", selector, member.base),
                            lookup.range,
                        ));
                        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                    };

                    let Ok(specialized) = specialize_associated_member(ctx, &owner, &member_id, lookup.range) else {
                        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                    };

                    let Ok(value_type) = check_reification_underconstrained(ctx, specialized.value_type, expected, lookup.range) else {
                        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                    };

                    if let Some(expression) = ctx.current_expression_id() {
                        let kind = match specialized.target.clone() {
                            Some(target) => AssociatedResolutionKind::ExactCallable {
                                member: member_id.clone(),
                                target,
                                callable_type: value_type,
                            },
                            None => AssociatedResolutionKind::ExactValue {
                                member: member_id.clone(),
                                value_type,
                            },
                        };
                        ctx.record_associated_resolution(
                            expression,
                            AssociatedResolution {
                                owner_form: owner.owner_form,
                                lookup_owner: owner.lookup_owner.clone(),
                                family: Some(family.id.clone()),
                                kind,
                            },
                        );
                    }

                    TypedExpression::established(value_type, EvidenceOrigin::DeclarationSemantics, lookup.range).with_denotation(
                        SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::exact(
                            owner.owner_form,
                            owner.lookup_owner,
                            member_id,
                            specialized.target,
                        ))),
                    )
                }
                AssociatedNamedMode::Exact { residual, .. } => {
                    let selector = match residual {
                        AssociatedResidualSelectorSyntax::Method { slots, .. } => {
                            let slot_objs: Vec<SelectorSlot> = slots.iter().map(|s| s.slot.clone()).collect();
                            match Selector::method(&member.base, slot_objs) {
                                Ok(sel) => sel,
                                Err(_) => return TypedExpression::unknown(UnknownReason::UncheckedExpression),
                            }
                        }
                        AssociatedResidualSelectorSyntax::Setter { .. } => match Selector::setter(&member.base) {
                            Ok(sel) => sel,
                            Err(_) => return TypedExpression::unknown(UnknownReason::UncheckedExpression),
                        },
                    };

                    let Some(member_id) = family
                        .members
                        .iter()
                        .find(|m| match m {
                            AssociatedMemberId::Variant(v) => v.selector == selector,
                        })
                        .cloned()
                    else {
                        ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::AssociatedMemberMissing,
                            format!("exact member `{}` not found in associated family `{}`", selector, member.base),
                            lookup.range,
                        ));
                        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                    };

                    let Ok(specialized) = specialize_associated_member(ctx, &owner, &member_id, lookup.range) else {
                        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                    };

                    let Ok(value_type) = check_reification_underconstrained(ctx, specialized.value_type, expected, lookup.range) else {
                        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                    };

                    let target = specialized.target.unwrap_or_else(|| {
                        InvocationTargetId::Behavioral(crate::identity::CallableId::new(
                            owner.lookup_owner.clone(),
                            selector.clone(),
                            crate::identity::DispatchSide::Class,
                        ))
                    });

                    if let Some(expression) = ctx.current_expression_id() {
                        ctx.record_associated_resolution(
                            expression,
                            AssociatedResolution {
                                owner_form: owner.owner_form,
                                lookup_owner: owner.lookup_owner.clone(),
                                family: Some(family.id.clone()),
                                kind: AssociatedResolutionKind::ExactCallable {
                                    member: member_id.clone(),
                                    target: target.clone(),
                                    callable_type: value_type,
                                },
                            },
                        );
                    }

                    TypedExpression::established(value_type, EvidenceOrigin::DeclarationSemantics, lookup.range).with_denotation(
                        SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::exact(
                            owner.owner_form,
                            owner.lookup_owner,
                            member_id,
                            Some(target),
                        ))),
                    )
                }
                AssociatedNamedMode::Family { .. } => {
                    let mut specialized_members = Vec::new();
                    let mut family_member_types = Vec::new();
                    let mut captured_members = Vec::new();

                    for member_id in &family.members {
                        if let Ok(spec) = specialize_associated_member(ctx, &owner, member_id, lookup.range) {
                            let member_type = match spec.target {
                                Some(_) => crate::types::family::FamilyMemberType::callable(spec.operation.clone(), spec.value_type),
                                None => crate::types::family::FamilyMemberType::value(spec.operation.clone(), spec.value_type),
                            };
                            family_member_types.push(member_type);
                            captured_members.push(crate::types::denotation::CapturedAssociatedMember {
                                operation: spec.operation.clone(),
                                member: member_id.clone(),
                                target: spec.target.clone(),
                            });
                            specialized_members.push(spec);
                        }
                    }

                    let Ok(family_type) = ctx.store.family_type(family_member_types) else {
                        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                    };

                    let Ok(value_type) = check_reification_underconstrained(ctx, family_type, expected, lookup.range) else {
                        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                    };

                    if let Some(expression) = ctx.current_expression_id() {
                        ctx.record_associated_resolution(
                            expression,
                            AssociatedResolution {
                                owner_form: owner.owner_form,
                                lookup_owner: owner.lookup_owner.clone(),
                                family: Some(family.id.clone()),
                                kind: AssociatedResolutionKind::Family {
                                    family_type: value_type,
                                    members: specialized_members.into_boxed_slice(),
                                },
                            },
                        );
                    }

                    TypedExpression::established(value_type, EvidenceOrigin::DeclarationSemantics, lookup.range).with_denotation(
                        SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::family(
                            owner.owner_form,
                            owner.lookup_owner,
                            family.id,
                            captured_members,
                        ))),
                    )
                }
            }
        }
        _ => TypedExpression::unknown(UnknownReason::UncheckedExpression),
    }
}

fn behavioral_family_spec(member: &AssociatedMemberSyntax) -> Option<BehavioralFamilySpec> {
    let AssociatedMemberSyntax::Named(member) = member else {
        return None;
    };
    match &member.mode {
        AssociatedNamedMode::Getter { .. } => Selector::getter(&member.base).ok().map(BehavioralFamilySpec::Exact),
        AssociatedNamedMode::Exact { residual, .. } => {
            let selector = match residual {
                AssociatedResidualSelectorSyntax::Method { slots, .. } => {
                    Selector::method(&member.base, slots.iter().map(|slot| slot.slot.clone()).collect::<Vec<_>>()).ok()
                }
                AssociatedResidualSelectorSyntax::Setter { .. } => Selector::setter(&member.base).ok(),
            }?;
            Some(BehavioralFamilySpec::Exact(selector))
        }
        AssociatedNamedMode::Family { .. } => SelectorPattern::named(
            member.base.clone(),
            phalcom_common::selector::SelectorKindPattern::AnyNamed,
            Vec::<SelectorSlot>::new(),
            Vec::<SelectorSlot>::new(),
            true,
        )
        .ok()
        .map(BehavioralFamilySpec::Pattern),
    }
}

fn synthesize_bound_behavioral_lookup(
    ctx: &mut CheckingContext<'_>,
    receiver: TypedExpression,
    lookup: &AssociatedLookupExpr,
    expected: &ExpectedType,
) -> TypedExpression {
    let Some(receiver_type) = receiver.knowledge.ty() else {
        return TypedExpression::unknown(UnknownReason::DynamicMessageSend);
    };
    let Some(spec) = behavioral_family_spec(&lookup.member) else {
        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
    };
    let dispatch_lookup = receiver.dispatch_lookup.clone();
    let Ok((lookup_owner, family_type, members)) = resolve_bound_behavioral_family(ctx, receiver_type, dispatch_lookup, spec.clone(), lookup.range) else {
        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
    };
    let Ok(family_type) = check_reification_underconstrained(ctx, family_type, expected, lookup.range) else {
        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
    };
    let captured = members
        .iter()
        .map(|member| CapturedBehavioralMember {
            operation: member.operation.clone(),
            target: member.target.clone(),
        })
        .collect::<Vec<_>>();
    if let Some(expression) = ctx.current_expression_id() {
        ctx.record_associated_resolution(
            expression,
            AssociatedResolution {
                owner_form: receiver_type,
                lookup_owner: lookup_owner.clone(),
                family: None,
                kind: AssociatedResolutionKind::BoundBehavioralFamily {
                    family_type,
                    spec: spec.clone(),
                    members: members.into_boxed_slice(),
                },
            },
        );
    }
    TypedExpression::established(family_type, EvidenceOrigin::DeclarationSemantics, lookup.range).with_denotation(SemanticDenotation::AssociatedValue(
        Box::new(AssociatedValueDenotation::BehavioralFamily {
            receiver_type,
            spec,
            members: captured.into(),
        }),
    ))
}

/// Resolves a concrete variant constructor or behavioral method selected by an exact static call shape.
fn synthesize_associated_invoke(ctx: &mut CheckingContext<'_>, invoke: &AssociatedInvokeExpr, expected: &ExpectedType) -> TypedExpression {
    let receiver = analyze_expression(ctx, &invoke.receiver, &ExpectedType::None);

    let arguments = application_arguments(&invoke.args);
    let StaticCallShape::Exact(slots) = static_call_shape(&arguments) else {
        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
    };
    let Ok(selector) = Selector::method(&invoke.base, slots) else {
        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
    };
    let base = SelectorBase::Named(invoke.base.clone());
    let owner = match receiver.denotation.as_ref() {
        Some(SemanticDenotation::TypeForm(owner_form)) if ctx.store.applied_nominal_parts(*owner_form).is_some() => {
            resolve_associated_owner(ctx, &receiver, invoke.range).ok()
        }
        _ => None,
    };
    let Some(owner) = owner else {
        return synthesize_bound_behavioral_invoke(ctx, receiver, invoke, &arguments, selector, expected);
    };
    let has_associated_base = ctx
        .associated_surface(&owner.lookup_owner)
        .is_some_and(|surface| surface.families.contains_key(&base));
    if !has_associated_base {
        return synthesize_bound_behavioral_invoke(ctx, receiver, invoke, &arguments, selector, expected);
    }
    let Ok(family) = resolve_effective_associated_family(ctx, &owner, &base, invoke.range) else {
        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
    };

    let Some(member_id) = family
        .members
        .iter()
        .find(|m| match m {
            AssociatedMemberId::Variant(v) => v.selector == selector,
        })
        .cloned()
    else {
        ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AssociatedCallShapeMissing,
            format!("no associated member matching call shape `{}` on `{}`", selector, owner.lookup_owner.name),
            invoke.range,
        ));
        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
    };

    let target = match &member_id {
        AssociatedMemberId::Variant(variant) => {
            let Some(variant_info) = ctx.variant_info(variant).cloned() else {
                return TypedExpression::unknown(UnknownReason::UncheckedExpression);
            };
            let Some(constructor) = variant_info.constructor else {
                return TypedExpression::unknown(UnknownReason::UncheckedExpression);
            };

            let mut env = crate::types::environment::TypeEnvironment::new();
            let mut fixed_generics = Vec::with_capacity(owner.supplied_arguments.len());
            for (idx, &arg) in owner.supplied_arguments.iter().enumerate() {
                if let Some(param_id) = ctx.store.find_type_parameter_id(
                    &crate::types::parameter::TypeParameterOwner::Declaration(owner.lookup_owner.clone()),
                    idx as u32,
                ) {
                    env.bind_param(param_id, arg);
                    fixed_generics.push((param_id, arg));
                }
            }

            // GADT check if owner type arguments were explicitly supplied
            for (param_id, &constrained_ty) in &variant_info.case_environment.bindings {
                if let Some(supplied) = env.get_param(*param_id) {
                    let matches = is_subtype(ctx.store, ctx.hierarchy.inner(), supplied, constrained_ty)
                        && is_subtype(ctx.store, ctx.hierarchy.inner(), constrained_ty, supplied);

                    if !matches {
                        ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::AssociatedGadtOwnerConflict,
                            format!(
                                "GADT variant `{}` requires type parameter to be `{}` but owner specified `{}`",
                                variant.selector,
                                ctx.store.format_type(constrained_ty),
                                ctx.store.format_type(supplied)
                            ),
                            invoke.range,
                        ));
                        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
                    }
                }
            }

            let parameters = constructor
                .parameters
                .iter()
                .map(|parameter| {
                    let Some(p_ty) = parameter.declared_type.canonical_type() else {
                        return Err(());
                    };
                    let ty = TypeView::new(p_ty, env.clone()).materialize(ctx.store);
                    let mut callable_parameter = crate::dispatch::CallableParameter::new(
                        parameter.local_name.to_string(),
                        TypeKnowledge::established(ty, EvidenceOrigin::ConstructorSemantics),
                    );
                    if let Some(label) = &parameter.external_label {
                        callable_parameter = callable_parameter.with_label(label.to_string());
                    }
                    Ok(callable_parameter)
                })
                .collect::<Result<Vec<_>, ()>>();
            let Ok(parameters) = parameters else {
                ctx.record_call_status(AnalysisStatus::Blocked(BlockReason::InvalidAnnotation(DiagnosticCode::AssociatedMemberMissing)));
                return TypedExpression::unknown(UnknownReason::InferenceBlocked);
            };
            let constructor_result_type = TypeView::new(constructor.exact_case_template, env).materialize(ctx.store);
            let mut signature = CallableSignature::new(
                selector.clone(),
                parameters,
                TypeKnowledge::established(constructor_result_type, EvidenceOrigin::ConstructorSemantics),
            );
            if let Some(enum_signature) = ctx.enum_info(&owner.lookup_owner).and_then(|info| info.generic_signature.clone()) {
                let mut enum_signature = enum_signature;
                let mut constraints = enum_signature.constraints.to_vec();
                constraints.extend(variant_info.case_environment.equalities.iter().cloned());
                enum_signature.constraints = constraints.into_boxed_slice();
                signature = signature.with_generics(enum_signature);
            }
            CallableApplicationTarget::variant_constructor(variant.clone(), signature).with_fixed_generics(fixed_generics)
        }
    };

    let Some(invocation_target) = target.target.clone() else {
        return TypedExpression::unknown(UnknownReason::UncheckedExpression);
    };

    let premise = CallPremise::from_typed(ctx, &receiver);
    let result = apply_resolved_callable(ctx, &target, &premise, &arguments, expected, invoke.range);
    if let (Some(result_type), Some(expression)) = (result.knowledge.ty(), ctx.current_expression_id()) {
        ctx.record_associated_resolution(
            expression,
            AssociatedResolution {
                owner_form: owner.owner_form,
                lookup_owner: owner.lookup_owner,
                family: Some(family.id),
                kind: AssociatedResolutionKind::StaticInvoke {
                    member: member_id,
                    target: invocation_target,
                    result_type,
                },
            },
        );
    }
    result.into()
}

fn synthesize_bound_behavioral_invoke(
    ctx: &mut CheckingContext<'_>,
    receiver: TypedExpression,
    invoke: &AssociatedInvokeExpr,
    arguments: &[super::call::ApplicationArgument<'_>],
    selector: Selector,
    expected: &ExpectedType,
) -> TypedExpression {
    let premise = CallPremise::from_typed(ctx, &receiver);
    let Some(receiver_type) = receiver.knowledge.ty() else {
        let reason = match &receiver.knowledge {
            TypeKnowledge::Unknown(_) => {
                if matches!(receiver.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Suppressed(_)) {
                    UnresolvedApplicationReason::PremiseInvalidUnavailable
                } else {
                    UnresolvedApplicationReason::PremiseUnknown
                }
            }
            TypeKnowledge::Dynamic(reason) => UnresolvedApplicationReason::PremiseDynamic(reason.clone()),
            TypeKnowledge::Known(_) => unreachable!("known receiver has a type"),
        };
        return analyze_unresolved_application(ctx, &premise, arguments, reason).into();
    };

    match ctx.resolve_dispatch_target(receiver_type, &selector, receiver.dispatch_lookup.clone()) {
        ResolvedDispatchResult::Found(resolved) => {
            let callable = resolved.callable.clone();
            let lookup_owner = ctx
                .dispatch_owner_for_lookup(receiver_type, receiver.dispatch_lookup.clone())
                .map(|(owner, _)| owner)
                .unwrap_or_else(|| callable.owner.declaration().clone());
            let target = CallableApplicationTarget::from_dispatch(resolved);
            let result = apply_resolved_callable(ctx, &target, &premise, arguments, expected, invoke.range);
            if let Some(result_type) = result.knowledge.ty() {
                if let Some(expression) = ctx.current_expression_id() {
                    let is_inherited_type_form =
                        matches!(receiver.denotation, Some(SemanticDenotation::TypeForm(_))) && callable.owner.declaration() != &lookup_owner;
                    let kind = if is_inherited_type_form {
                        AssociatedResolutionKind::StaticInvoke {
                            member: AssociatedMemberId::Variant(crate::identity::VariantId::new(lookup_owner.clone(), selector)),
                            target: InvocationTargetId::Behavioral(callable),
                            result_type,
                        }
                    } else {
                        AssociatedResolutionKind::BoundBehavioralInvoke {
                            target: InvocationTargetId::Behavioral(callable),
                            result_type,
                        }
                    };
                    ctx.record_associated_resolution(
                        expression,
                        AssociatedResolution {
                            owner_form: receiver_type,
                            lookup_owner,
                            family: None,
                            kind,
                        },
                    );
                }
            }
            result.into()
        }
        ResolvedDispatchResult::Missing { .. } => analyze_unresolved_application(ctx, &premise, arguments, UnresolvedApplicationReason::DispatchMissing).into(),
        ResolvedDispatchResult::Ambiguous(_) => analyze_unresolved_application(ctx, &premise, arguments, UnresolvedApplicationReason::DispatchAmbiguous).into(),
        ResolvedDispatchResult::Dynamic => analyze_unresolved_application(
            ctx,
            &premise,
            arguments,
            UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
        )
        .into(),
    }
}

// ---------------------------------------------------------------------------
// Helpers for Complex Expressions
// ---------------------------------------------------------------------------

fn synthesize_symbol_expr(ctx: &mut CheckingContext<'_>, s: &SymbolExpr) -> TypedExpression {
    let symbol_decl = ctx.core_ids.symbol.clone();
    let string_decl = ctx.core_ids.string.clone();
    if let Some(ty) = ctx.core_type(&symbol_decl) {
        TypedExpression::established(ty, EvidenceOrigin::Syntax, s.range)
    } else if let Some(ty) = ctx.core_type(&string_decl) {
        TypedExpression::established(ty, EvidenceOrigin::Syntax, s.range)
    } else {
        TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause)
    }
}

fn synthesize_list_literal(ctx: &mut CheckingContext<'_>, list: &phalcom_ast::ast::ListLiteralExpr, expected: &ExpectedType) -> TypedExpression {
    let list_decl = ctx.core_ids.list.clone();
    let expected_elem = expected.collection_element_type(ctx.store);
    let list_form = {
        let kind = ctx.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        Some(ctx.store.nominal_form(list_decl.clone(), kind))
    };

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
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause));
                contributions.push(projected);
                operands.push(typed);
            }
        }
    }

    let knowledge = if list.elements.is_empty() {
        if let Some(expected_ty) = expected.ty() {
            if is_applied_core_collection(ctx.store, expected_ty, &list_decl) {
                expected
                    .contextual_knowledge(expected_ty)
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence))
            } else {
                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)
            }
        } else {
            TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)
        }
    } else if let Some(form) = list_form {
        crate::types::evidence::compose_required_knowledge(contributions, EvidenceOrigin::Syntax, |types| {
            if types.is_empty() {
                return Err(UnknownReason::NoTypeEvidence);
            }
            let element = ctx.store.union(types);
            ctx.store.list_of(form, element).map_err(|_| UnknownReason::UncheckedExpression)
        })
    } else {
        TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)
    };
    let mut result = TypedExpression::new(match knowledge {
        TypeKnowledge::Known(_) => knowledge.with_range(list.range),
        other => other,
    });
    crate::checker::composition::propagate_required_dependencies(&mut result, &operands);
    result
}

fn synthesize_set_literal(ctx: &mut CheckingContext<'_>, set: &phalcom_ast::ast::SetLiteralExpr, expected: &ExpectedType) -> TypedExpression {
    let set_decl = ctx.core_ids.set.clone();
    let expected_elem = expected.collection_element_type(ctx.store);
    let set_form = {
        let kind = ctx.store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        Some(ctx.store.nominal_form(set_decl.clone(), kind))
    };
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
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause));
                contributions.push(projected);
                operands.push(typed);
            }
        }
    }

    let knowledge = if set.entries.is_empty() {
        if let Some(expected_ty) = expected.ty() {
            if is_applied_core_collection(ctx.store, expected_ty, &set_decl) {
                expected
                    .contextual_knowledge(expected_ty)
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence))
            } else {
                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)
            }
        } else {
            TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)
        }
    } else if let Some(form) = set_form {
        crate::types::evidence::compose_required_knowledge(contributions, EvidenceOrigin::Syntax, |types| {
            if types.is_empty() {
                return Err(UnknownReason::NoTypeEvidence);
            }
            let element = ctx.store.union(types);
            ctx.store.set_of(form, element).map_err(|_| UnknownReason::UncheckedExpression)
        })
    } else {
        TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)
    };
    let mut result = TypedExpression::new(match knowledge {
        TypeKnowledge::Known(_) => knowledge.with_range(set.range),
        other => other,
    });
    crate::checker::composition::propagate_required_dependencies(&mut result, &operands);
    result
}

fn synthesize_map_literal(ctx: &mut CheckingContext<'_>, map: &phalcom_ast::ast::MapLiteralExpr, expected: &ExpectedType) -> TypedExpression {
    let map_decl = ctx.core_ids.map.clone();
    let symbol_decl = ctx.core_ids.symbol.clone();
    let (expected_key, expected_val) = expected.map_key_val_types(ctx.store);
    let map_form = {
        let kind = ctx.store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);
        Some(ctx.store.nominal_form(map_decl.clone(), kind))
    };
    let mut operands = Vec::new();
    let mut key_knowledge = Vec::new();
    let mut value_knowledge = Vec::new();

    for entry in &map.entries {
        match entry {
            MapLiteralEntry::Association { key, value, .. } => {
                match key {
                    MapLiteralKey::BareSymbol { .. } => {
                        let knowledge = ctx
                            .core_type(&symbol_decl)
                            .map(|ty| TypeKnowledge::established(ty, EvidenceOrigin::Syntax))
                            .unwrap_or(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause));
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
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause));
                let value = map_form
                    .map(|form| crate::checker::composition::project_applied_argument(ctx.store, &typed.knowledge, form, 1))
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause));
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
    let knowledge = if map.entries.is_empty() {
        if let Some(expected_ty) = expected.ty() {
            if is_applied_core_collection(ctx.store, expected_ty, &map_decl) {
                expected
                    .contextual_knowledge(expected_ty)
                    .unwrap_or(TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence))
            } else {
                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)
            }
        } else {
            TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)
        }
    } else if let Some(form) = map_form {
        crate::types::evidence::compose_required_knowledge([key_lane, value_lane], EvidenceOrigin::Syntax, |types| {
            let [key, value] = types else {
                return Err(UnknownReason::UncheckedExpression);
            };
            ctx.store.map_of(form, *key, *value).map_err(|_| UnknownReason::UncheckedExpression)
        })
    } else {
        TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)
    };
    let mut result = TypedExpression::new(match knowledge {
        TypeKnowledge::Known(_) => knowledge.with_range(map.range),
        other => other,
    });
    crate::checker::composition::propagate_required_dependencies(&mut result, &operands);
    result
}

fn is_applied_core_collection(store: &crate::types::store::TypeStore, applied_ty: TypeId, core_decl: &DeclarationId) -> bool {
    if let crate::types::store::TypeData::Applied { origin, .. } = store.get(applied_ty) {
        if let crate::types::store::TypeData::Nominal { declaration } = store.get(*origin) {
            return declaration == core_decl;
        }
    }
    false
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

fn captured_family_target(denotation: Option<&SemanticDenotation>, operation: &FamilyOperationShape) -> Option<InvocationTargetId> {
    let Some(SemanticDenotation::AssociatedValue(assoc)) = denotation else {
        return None;
    };
    let AssociatedValueDenotation::Family { members, .. } = &**assoc else {
        return None;
    };
    members
        .iter()
        .find(|member| &member.operation == operation)
        .and_then(|member| member.target.clone())
}

fn callable_return_type(ctx: &CheckingContext<'_>, callable_type: TypeId) -> Option<TypeId> {
    let TypeData::Callable(callable) = ctx.store.get(callable_type) else {
        return None;
    };
    Some(callable.return_type)
}

fn family_callable_application_target(
    ctx: &CheckingContext<'_>,
    callable_type: TypeId,
    target: Option<&InvocationTargetId>,
    authority: EvidenceStatus,
) -> Option<CallableApplicationTarget> {
    let mut application_target = callable_value_target(ctx.store, callable_type, authority)?;
    if let Some(target) = target {
        application_target.target = Some(target.clone());
        application_target.authority = CallTargetAuthority::ExactDispatch;
        application_target.callable = match target {
            InvocationTargetId::Behavioral(callable) => Some(callable.clone()),
            InvocationTargetId::VariantConstructor(_) => None,
        };
    }
    Some(application_target)
}

fn synthesize_family_value_call(
    ctx: &mut CheckingContext<'_>,
    family_type: TypeId,
    denotation: Option<&SemanticDenotation>,
    premise: &CallPremise,
    arguments: &[super::call::ApplicationArgument<'_>],
    expected: &ExpectedType,
    range: SourceRange,
) -> TypedExpression {
    let TypeData::Family(family_id) = ctx.store.get(family_type) else {
        return analyze_unresolved_application(ctx, premise, arguments, UnresolvedApplicationReason::DispatchMissing).into();
    };
    let members = ctx.store.get_family(*family_id).members.to_vec();

    match static_call_shape(arguments) {
        StaticCallShape::Exact(slots) => {
            let operation = FamilyOperationShape::method(slots);
            let Some(member) = members
                .iter()
                .find(|member| member.member_kind == FamilyMemberTypeKind::Callable && member.operation == operation)
            else {
                return analyze_unresolved_application(ctx, premise, arguments, UnresolvedApplicationReason::DispatchMissing).into();
            };

            let callable_type = member.ty;
            let target = captured_family_target(denotation, &operation);
            let authority = premise.knowledge.status().unwrap_or(EvidenceStatus::Assumed);
            let Some(application_target) = family_callable_application_target(ctx, callable_type, target.as_ref(), authority) else {
                return analyze_unresolved_application(ctx, premise, arguments, UnresolvedApplicationReason::DispatchMissing).into();
            };
            let fallback_result_type = callable_return_type(ctx, callable_type);
            let result = apply_resolved_callable(ctx, &application_target, premise, arguments, expected, range);
            let Some(result_type) = result.knowledge.ty().or(fallback_result_type) else {
                return result.into();
            };
            if matches!(
                denotation,
                Some(SemanticDenotation::AssociatedValue(assoc))
                    if matches!(&**assoc, AssociatedValueDenotation::Family { .. })
            ) && let Some(expression) = ctx.current_expression_id()
            {
                ctx.record_family_application(
                    expression,
                    FamilyApplicationResolution {
                        family_type,
                        selection: FamilyApplicationSelection::Static {
                            operation,
                            target,
                            callable_type,
                            result_type,
                        },
                    },
                );
            }
            result.into()
        }
        StaticCallShape::Dynamic(reason) => {
            let candidates = members
                .iter()
                .filter(|member| member.member_kind == FamilyMemberTypeKind::Callable)
                .map(|member| FamilyApplicationCandidate {
                    operation: member.operation.clone(),
                    target: captured_family_target(denotation, &member.operation),
                    callable_type: member.ty,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice();
            let result = analyze_unresolved_application(ctx, premise, arguments, UnresolvedApplicationReason::DynamicShape(reason));
            if matches!(
                denotation,
                Some(SemanticDenotation::AssociatedValue(assoc))
                    if matches!(&**assoc, AssociatedValueDenotation::Family { .. })
            ) && let Some(expression) = ctx.current_expression_id()
            {
                ctx.record_family_application(
                    expression,
                    FamilyApplicationResolution {
                        family_type,
                        selection: FamilyApplicationSelection::Dynamic { candidates, result_type: None },
                    },
                );
            }
            result.into()
        }
    }
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
            if matches!(ctx.store.get(receiver_ty), TypeData::Family(_)) {
                let arguments = application_arguments(&call.args);
                return synthesize_family_value_call(ctx, receiver_ty, recv_typed.denotation.as_ref(), &premise, &arguments, expected, call.range);
            }
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
        .core_type(&ctx.core_ids.bool_.clone())
        .zip(receiver_typed.knowledge.ty())
        .is_some_and(|(bool_ty, receiver_ty)| bool_ty == receiver_ty);
    let receiver_is_literal_block = matches!(&call.object, Expr::Block(_));

    match call.method.as_str() {
        "ifTrue" if receiver_is_bool => {
            let then_block = positional_block(0)?;
            let else_block = labeled_block("ifFalse");
            let branch_result = super::control::analyze_branch_pair(
                ctx,
                &call.object,
                receiver_typed,
                &then_block.body,
                then_block.range,
                else_block.map(|b| (b.body.as_slice(), b.range)),
                call.range,
                expected,
            );
            Some(branch_result.typed)
        }
        "whileTrue" if receiver_is_literal_block => {
            let body = positional_block(0)?;
            let condition_block = match &call.object {
                Expr::Block(b) => b.as_ref(),
                _ => return None,
            };
            let expected_bool = ctx
                .core_type(&ctx.core_ids.bool_.clone())
                .map(|ty| ExpectedType::proper_from(ty, crate::checker::expected::ExpectationOrigin::ExplicitCheck))
                .unwrap_or_default();

            let before = ctx.flow.clone();

            let condition_expr = condition_block.body.last().and_then(|s| match s {
                Statement::Expr { expr, .. } => Some(expr),
                _ => None,
            });

            let evaluate_step =
                |step_ctx: &mut CheckingContext<'_>, current_header: &FlowState| -> (Option<FlowState>, FlowState, Vec<FlowState>, CausalInvalidity) {
                    step_ctx.flow = current_header.clone();
                    let cond_res = super::control::analyze_executable_region(step_ctx, &condition_block.body, condition_block.range, &expected_bool);
                    if !cond_res.completes_normally() {
                        return (None, FlowState::unreachable(), Vec::new(), cond_res.causal_invalidity);
                    }

                    let (when_true, when_false) = if let (Some(cond_expr), Some(cond_typed)) = (condition_expr, &cond_res.value) {
                        let truth = super::control::condition_truth(cond_expr);
                        let when_true = match truth {
                            super::control::ConditionTruth::AlwaysFalse => FlowState::unreachable(),
                            _ => {
                                step_ctx.flow = cond_res.flow.clone();
                                if let Some(predicate) = crate::checker::flow::extract_trusted_predicate(step_ctx, cond_expr, cond_typed, true) {
                                    step_ctx.apply_flow_predicate(&predicate);
                                }
                                step_ctx.flow.clone()
                            }
                        };
                        let when_false = match truth {
                            super::control::ConditionTruth::AlwaysTrue => FlowState::unreachable(),
                            _ => {
                                step_ctx.flow = cond_res.flow.clone();
                                if let Some(predicate) = crate::checker::flow::extract_trusted_predicate(step_ctx, cond_expr, cond_typed, false) {
                                    step_ctx.apply_flow_predicate(&predicate);
                                }
                                step_ctx.flow.clone()
                            }
                        };
                        (when_true, when_false)
                    } else {
                        (cond_res.flow.clone(), cond_res.flow.clone())
                    };

                    if !when_true.is_reachable() {
                        return (None, when_false, Vec::new(), cond_res.causal_invalidity);
                    }

                    step_ctx.flow = when_true;
                    step_ctx.push_loop_frame();
                    let body_res = super::control::analyze_executable_region(step_ctx, &body.body, body.range, &ExpectedType::None);
                    let body_flow = step_ctx.flow.clone();
                    let loop_frame = step_ctx.pop_loop_frame();
                    let normal_backedge = if body_res.completes_normally() { Some(body_flow) } else { None };
                    let mut continues = loop_frame.continues;
                    let breaks = loop_frame.breaks;
                    continues.extend(normal_backedge);
                    let backedge = if continues.is_empty() {
                        None
                    } else if continues.len() == 1 {
                        continues.pop()
                    } else {
                        step_ctx.join_flow_states(&continues).ok()
                    };

                    (backedge, when_false, breaks, cond_res.causal_invalidity.join(body_res.causal_invalidity))
                };

            let fixpoint = match super::loop_analysis::solve_loop_header(ctx, &before, |probe_ctx, current_header| {
                let (backedge, _false_exit, breaks, _causal) = evaluate_step(probe_ctx, current_header);
                super::loop_analysis::LoopStepResult {
                    normal_backedge: backedge,
                    continues: Vec::new(),
                    breaks,
                }
            }) {
                Ok(fp) => fp,
                Err(failure) => {
                    let status = ctx.publish_flow_join_failure(failure, call.range);
                    let mut typed = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, call.range);
                    typed.status = status;
                    return Some(typed);
                }
            };

            // Final real pass at stable header
            let (_backedge, when_false, breaks, causal) = evaluate_step(ctx, &fixpoint.header);

            let mut exit_states = Vec::new();
            if when_false.is_reachable() {
                exit_states.push(when_false);
            }
            for brk in breaks {
                if brk.is_reachable() {
                    exit_states.push(brk);
                }
            }

            let join_status = if exit_states.is_empty() {
                ctx.flow = FlowState::unreachable();
                None
            } else if exit_states.len() == 1 {
                ctx.flow = exit_states.pop().unwrap();
                None
            } else {
                match ctx.join_flow_states(&exit_states) {
                    Ok(flow) => {
                        ctx.flow = flow;
                        None
                    }
                    Err(failure) => Some(ctx.publish_flow_join_failure(failure, call.range)),
                }
            };

            let mut typed = TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, call.range);
            if let Some(status) = join_status {
                typed.status = status;
            }
            typed.causal_invalidity = causal;
            Some(typed)
        }
        _ => None,
    }
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
            if matches!(ctx.store.get(ty), TypeData::Family(_)) {
                return synthesize_family_value_call(ctx, ty, fact.denotation.as_ref(), &premise, &arguments, expected, call.range);
            }

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

                let is_family_or_callable = match ctx.store.get(ty) {
                    TypeData::Nominal { declaration } => {
                        declaration.name.as_ref() == "Family" || declaration.name.as_ref() == "BoundMethod" || declaration.name.as_ref() == "Method"
                    }
                    _ => false,
                };
                if is_family_or_callable {
                    return analyze_unresolved_application(
                        ctx,
                        &premise,
                        &arguments,
                        UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
                    )
                    .into();
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
                Some(info.class_object_type)
            } else {
                ctx.nominal_type_of(class_decl)
            }
        } else {
            ctx.instance_type_of(class_decl)
        };
        let Some(class_ty) = class_ty else {
            return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
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
        let Some(ty) = ctx.nominal_type_of(&decl) else {
            return TypedExpression::unknown(UnknownReason::SuppressedByInvalidCause);
        };
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

pub(crate) fn apply_binary_operation_from_typed(
    ctx: &mut CheckingContext<'_>,
    left_expr: &Expr,
    left_typed: &TypedExpression,
    op: BinaryOp,
    right_expr: &Expr,
    right_typed: &TypedExpression,
    range: SourceRange,
) -> TypedExpression {
    let op_name = match op {
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
        let premise = CallPremise::from_typed(ctx, left_typed);
        let arguments = vec![super::call::ApplicationArgument::PreAnalyzed {
            label: None,
            typed: right_typed,
            range: right_expr.range(),
        }];
        return analyze_unresolved_application(ctx, &premise, &arguments, UnresolvedApplicationReason::DispatchMissing).into();
    };
    let direct = left_typed
        .knowledge
        .ty()
        .map(|left_ty| ctx.resolve_dispatch_target(left_ty, &selector, left_typed.dispatch_lookup.clone()));

    if let Some(right_ty) = right_typed.knowledge.ty()
        && let Some(reflected_selector) = reflected_binary_selector(&op)
        && let ResolvedDispatchResult::Found(reflected) = ctx.resolve_dispatch_target(right_ty, &reflected_selector, crate::dispatch::DispatchLookup::Normal)
        && should_use_reflected_binary_target(ctx, &left_typed.knowledge, right_ty, &right_typed.knowledge, direct.as_ref(), &reflected)
    {
        let premise = CallPremise::from_typed(ctx, right_typed);
        let target = CallableApplicationTarget::from_dispatch(reflected);
        let arguments = vec![super::call::ApplicationArgument::PreAnalyzed {
            label: if matches!(op, BinaryOp::Compare) { None } else { Some("from") },
            typed: left_typed,
            range: left_expr.range(),
        }];
        return apply_resolved_callable(ctx, &target, &premise, &arguments, &ExpectedType::None, range).into();
    }

    let premise = CallPremise::from_typed(ctx, left_typed);
    let arguments = vec![super::call::ApplicationArgument::PreAnalyzed {
        label: None,
        typed: right_typed,
        range: right_expr.range(),
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
            apply_resolved_callable(ctx, &target, &premise, &arguments, &ExpectedType::None, range).into()
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

fn synthesize_binary_expr(ctx: &mut CheckingContext<'_>, binary: &BinaryExpr) -> TypedExpression {
    let left_typed = analyze_expression(ctx, &binary.left, &ExpectedType::None);
    let right_typed = analyze_expression(ctx, &binary.right, &ExpectedType::None);
    apply_binary_operation_from_typed(ctx, &binary.left, &left_typed, binary.op.clone(), &binary.right, &right_typed, binary.range)
}

fn synthesize_comparison_chain(ctx: &mut CheckingContext<'_>, chain: &ComparisonChainExpr) -> TypedExpression {
    if chain.operands.is_empty() {
        return TypedExpression::unknown(UnknownReason::NoTypeEvidence);
    }
    let operands: Vec<TypedExpression> = chain.operands.iter().map(|expr| analyze_expression(ctx, expr, &ExpectedType::None)).collect();

    let bool_decl = ctx.core_ids.bool_.clone();
    let bool_ty = ctx.core_type(&bool_decl).unwrap_or(ctx.store.unit());

    let mut link_results = Vec::new();
    for (i, op) in chain.operators.iter().enumerate() {
        if i + 1 >= operands.len() {
            break;
        }
        let left_expr = &chain.operands[i];
        let left_typed = &operands[i];
        let right_expr = &chain.operands[i + 1];
        let right_typed = &operands[i + 1];
        let link_range = left_expr.range().merge(&right_expr.range());
        let link_result = match op {
            RelationOp::Binary(b_op) => apply_binary_operation_from_typed(ctx, left_expr, left_typed, b_op.clone(), right_expr, right_typed, link_range),
            RelationOp::Matches | RelationOp::Understands => TypedExpression::unknown(UnknownReason::UncheckedExpression),
        };
        link_results.push(link_result);
    }

    let knowledge = if link_results.is_empty() {
        TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)
    } else {
        crate::types::evidence::compose_required_knowledge(link_results.iter().map(|l| l.knowledge.clone()), EvidenceOrigin::Flow, |_types| Ok(bool_ty))
    };

    let mut causal = operands.iter().fold(CausalInvalidity::Clean, |acc, op| acc.join(op.causal_invalidity));
    for link in &link_results {
        causal = causal.join(link.causal_invalidity);
    }

    let mut result = TypedExpression::new(match knowledge {
        TypeKnowledge::Known(_) => knowledge.with_range(chain.range),
        other => other,
    });
    result.causal_invalidity = causal;
    crate::checker::composition::propagate_required_dependencies(&mut result, &operands);
    crate::checker::composition::propagate_required_dependencies(&mut result, &link_results);
    result
}

fn synthesize_membership_expr(ctx: &mut CheckingContext<'_>, m: &MembershipExpr) -> TypedExpression {
    let left = analyze_expression(ctx, &m.left, &ExpectedType::None);
    let right = analyze_expression(ctx, &m.right, &ExpectedType::None);
    let mut result = TypedExpression::unknown(UnknownReason::UncheckedExpression);
    result.causal_invalidity = left.causal_invalidity.join(right.causal_invalidity);
    crate::checker::composition::propagate_required_dependencies(&mut result, &[left, right]);
    result
}

fn synthesize_is_membership_expr(ctx: &mut CheckingContext<'_>, m: &IsMembershipExpr) -> TypedExpression {
    let left = analyze_expression(ctx, &m.left, &ExpectedType::None);
    let candidates = analyze_expression(ctx, &m.candidates, &ExpectedType::None);
    let mut result = TypedExpression::unknown(UnknownReason::UncheckedExpression);
    result.causal_invalidity = left.causal_invalidity.join(candidates.causal_invalidity);
    crate::checker::composition::propagate_required_dependencies(&mut result, &[left, candidates]);
    result
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
    right != left
        && ctx.hierarchy.is_subclass(&right, &left)
        && reflected.callable.declaration_owner() != &left
        && ctx.hierarchy.is_subclass(reflected.callable.declaration_owner(), &left)
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
    let field_read = match ctx.store.get(recv_ty).clone() {
        TypeData::ClassObject { declaration } => ctx.resolve_field_read(&declaration, crate::identity::DispatchSide::Class, &get.property),
        TypeData::Nominal { declaration } => ctx.resolve_field_read(&declaration, crate::identity::DispatchSide::Instance, &get.property),
        _ => None,
    };
    if let Some((_, field_k, field_causal)) = field_read {
        let mut typed = TypedExpression::new(field_k.with_range(get.range));
        typed.causal_invalidity = recv_typed.causal_invalidity.join(field_causal);
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

pub fn synthesize_match_expr(ctx: &mut CheckingContext<'_>, match_expr: &phalcom_ast::ast::MatchExpr, expected: &ExpectedType) -> TypedExpression {
    let expr_id = ctx.current_expression_id().unwrap_or_else(|| ctx.alloc_expression_id());
    let scrutinee_typed = analyze_expression(ctx, &match_expr.value, &ExpectedType::None);
    let scrutinee_ty = scrutinee_typed.knowledge.ty().unwrap_or_else(|| ctx.store.unit());
    let before_flow = ctx.flow.clone();
    let stable_scrutinee = match match_expr.value.as_ref() {
        Expr::Var { value, .. } => ctx.lookup_binding_info(value).map(|binding| binding.id),
        _ => None,
    };

    let initial_space = crate::checker::exhaustiveness::build_initial_pattern_space(ctx, scrutinee_ty);
    let mut remaining_space = initial_space.clone().normalize();
    let mut arm_resolutions = Vec::with_capacity(match_expr.arms.len());
    let mut normal_branch_types = Vec::new();
    let mut normal_branch_flows = Vec::new();

    for (arm_index, arm) in match_expr.arms.iter().enumerate() {
        ctx.flow = before_flow.clone();
        ctx.push_scope();
        let mut arm_bindings = Vec::new();
        let (pattern_res, arm_space) = crate::checker::pattern::resolve_pattern(ctx, &arm.pattern, scrutinee_ty, &initial_space, &mut arm_bindings);
        let (reachable, residual_after, usefulness) =
            crate::checker::exhaustiveness::evaluate_match_arm_usefulness(ctx, &initial_space, &remaining_space, &arm_space, arm.range);
        remaining_space = residual_after.clone();
        let proof = pattern_common_proof(&pattern_res);

        let analyzed_branch = if usefulness == crate::match_semantics::PatternUsefulness::Useful && !reachable.is_empty() {
            if let (Some(binding), Some(exact_case)) = (stable_scrutinee, pattern_exact_case_type(ctx, &pattern_res)) {
                ctx.apply_flow_predicate(&crate::checker::flow::FlowPredicate::IsInstance { binding, target: exact_case }.authoritative());
            }
            let branch_typed = analyze_expression(ctx, &arm.branch, expected);
            Some((branch_typed.knowledge, ctx.flow.clone()))
        } else {
            None
        };

        ctx.pop_scope();
        let branch_result = if let Some((branch_result, mut branch_flow)) = analyzed_branch {
            for pattern_binding in &arm_bindings {
                branch_flow.bindings.remove(&pattern_binding.binding);
                branch_flow.facts.invalidate_binding(pattern_binding.binding);
            }
            if branch_flow.is_reachable() {
                normal_branch_types.push(branch_result.clone());
                normal_branch_flows.push(branch_flow);
            }
            branch_result
        } else {
            TypeKnowledge::Unknown(UnknownReason::UncheckedExpression)
        };
        ctx.flow = before_flow.clone();

        arm_resolutions.push(crate::match_semantics::MatchArmResolution {
            arm_index: arm_index as u32,
            pattern: pattern_res,
            reachable_space: reachable.summarize(),
            residual_after: residual_after.summarize(),
            bindings: arm_bindings.into_boxed_slice(),
            proof,
            usefulness,
            branch_result,
        });
    }

    let exhaustiveness = crate::checker::exhaustiveness::finalize_match_exhaustiveness(ctx, &remaining_space, match_expr.range);

    ctx.flow = if normal_branch_flows.is_empty() {
        FlowState::unreachable()
    } else if normal_branch_flows.len() == 1 {
        match normal_branch_flows.pop() {
            Some(flow) => flow,
            None => FlowState::unreachable(),
        }
    } else {
        match ctx.join_flow_states(&normal_branch_flows) {
            Ok(flow) => flow,
            Err(failure) => {
                ctx.publish_flow_join_failure(failure, match_expr.range);
                before_flow
            }
        }
    };

    let unified_result = crate::checker::exhaustiveness::join_match_result_knowledge(ctx.store, normal_branch_types);

    let resolution = crate::match_semantics::MatchResolution {
        expression: expr_id,
        scrutinee: scrutinee_typed.knowledge.clone(),
        initial_space: initial_space.summarize(),
        arms: arm_resolutions.into_boxed_slice(),
        result: unified_result.clone(),
        exhaustiveness,
    };
    ctx.record_match_resolution(expr_id, resolution);

    TypedExpression::new(unified_result)
}

/// Returns the exact-case union established by a successful root pattern.
fn pattern_exact_case_type(ctx: &mut CheckingContext<'_>, resolution: &crate::match_semantics::PatternResolution) -> Option<TypeId> {
    match resolution {
        crate::match_semantics::PatternResolution::Variant(variant) => {
            let exact_cases = variant.candidates.iter().map(|candidate| candidate.exact_case).collect::<Vec<_>>();
            (!exact_cases.is_empty()).then(|| ctx.store.union(&exact_cases))
        }
        crate::match_semantics::PatternResolution::Or(or_pattern) => {
            let exact_cases = or_pattern
                .alternatives
                .iter()
                .filter_map(|alternative| pattern_exact_case_type(ctx, alternative))
                .collect::<Vec<_>>();
            (!exact_cases.is_empty()).then(|| ctx.store.union(&exact_cases))
        }
        _ => None,
    }
}

/// Keeps only GADT facts established by every reachable pattern alternative.
fn pattern_common_proof(resolution: &crate::match_semantics::PatternResolution) -> crate::match_semantics::BranchProofEnvironment {
    let alternatives = match resolution {
        crate::match_semantics::PatternResolution::Variant(variant) => variant.candidates.iter().map(|candidate| candidate.proof.clone()).collect::<Vec<_>>(),
        crate::match_semantics::PatternResolution::Or(or_pattern) => or_pattern.alternatives.iter().map(pattern_common_proof).collect::<Vec<_>>(),
        _ => return crate::match_semantics::BranchProofEnvironment::default(),
    };
    let Some(first) = alternatives.first() else {
        return crate::match_semantics::BranchProofEnvironment::default();
    };
    let mut proof = first.clone();
    proof
        .bindings
        .retain(|parameter, ty| alternatives.iter().all(|alternative| alternative.bindings.get(parameter) == Some(ty)));
    proof.equalities = proof
        .equalities
        .iter()
        .filter(|equality| alternatives.iter().all(|alternative| alternative.equalities.contains(equality)))
        .cloned()
        .collect::<Vec<_>>()
        .into_boxed_slice();
    proof
}
