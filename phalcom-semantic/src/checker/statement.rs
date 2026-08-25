//! Statement type checking engine.

use super::binding::{BindingContract, BindingContractOrigin, BindingSeed, reconcile_binding_contract};
use super::context::CheckingContext;
use super::expected::{ExpectationOrigin, ExpectedType};
use super::expression::{analyze_expression, synthesize_expr};
use super::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use phalcom_ast::ast::{BindingKind, Pattern, Statement};
use phalcom_common::selector::Selector;

/// Checks a single statement, updating context bindings and recording
/// diagnostics. A direct `return` reports its typed normal-return value to
/// callable-body analysis; all other statement forms return `None`.
pub fn check_statement(ctx: &mut CheckingContext<'_>, statement: &Statement) -> Option<TypeKnowledge> {
    match statement {
        Statement::Let(binding) => {
            let (declared_k, annotation_invalidity) = binding
                .annotation
                .as_ref()
                .map(|annotation| {
                    let resolver = ctx.resolver.clone();
                    let (knowledge, causal_invalidity) = ctx.resolve_type_annotation(&resolver, annotation);
                    (Some(knowledge), causal_invalidity)
                })
                .unwrap_or((None, crate::checker::causal::CausalInvalidity::Clean));

            let expected_init = declared_k
                .as_ref()
                .and_then(TypeKnowledge::ty)
                .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::DeclarationContract))
                .unwrap_or_default();
            let val_typed = if let Some(expr) = &binding.value {
                analyze_expression(ctx, expr, &expected_init)
            } else {
                if binding.kind == BindingKind::Const {
                    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ConstWithoutInitializer,
                        "const binding requires an initializer",
                        binding.range,
                    ));
                }
                TypedExpression::unknown(UnknownReason::MissingInitializer)
            };

            let contract = if let Some(declared_k) = declared_k.as_ref().and_then(TypeKnowledge::ty) {
                Some(BindingContract {
                    ty: declared_k,
                    origin: BindingContractOrigin::SourceAnnotation,
                    source: binding.annotation.as_ref().map(|annotation| annotation.range),
                })
            } else {
                val_typed.knowledge.ty().map(|ty| BindingContract {
                    ty,
                    origin: BindingContractOrigin::InferredInitializer,
                    source: binding.value.as_ref().map(|value| value.range()),
                })
            };
            let reconciliation = reconcile_binding_contract(ctx.store, &ctx.hierarchy, contract.as_ref(), &val_typed.knowledge);
            let mut causal_invalidity = val_typed.causal_invalidity.join(annotation_invalidity);
            if matches!(reconciliation.consistency, crate::checker::binding::BindingConsistency::Refuted { .. }) {
                let mut diag = SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::BindingInitializerMismatch,
                    "initializer expression is not assignable to declared type",
                    binding.range,
                );
                if let Some(ann) = &binding.annotation {
                    diag = diag.with_label(ann.range, "declared type");
                }
                if let Some(val) = &binding.value {
                    diag = diag.with_label(val.range(), "inferred type");
                }
                let cause = ctx.emit_diagnostic(diag).expect("error diagnostic has cause");
                causal_invalidity = causal_invalidity.join(crate::checker::causal::CausalInvalidity::One(cause));
            }

            if let Pattern::Name { name, .. } = &binding.pattern {
                ctx.declare_binding(BindingSeed {
                    name: name.clone(),
                    range: binding.range,
                    contract,
                    current: reconciliation.current,
                    denotation: val_typed.denotation,
                    causal_invalidity,
                    mutable: binding.kind == BindingKind::Let,
                });
            }
            None
        }
        Statement::Return(ret) => {
            let expected_ret = ctx
                .expected_return
                .as_ref()
                .and_then(TypeKnowledge::ty)
                .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::ReturnContract))
                .unwrap_or_default();
            let val_typed = if let Some(expr) = &ret.value {
                analyze_expression(ctx, expr, &expected_ret)
            } else {
                TypedExpression::established(ctx.store.unit(), EvidenceOrigin::DeclarationSemantics, ret.range)
            };

            if let Some(expected) = ctx.expected_return.clone() {
                ctx.enforce_assignability(
                    &val_typed.knowledge,
                    &expected,
                    DiagnosticCode::ReturnMismatch,
                    "returned value is not assignable to method's declared return type",
                    ret.range,
                );
            }
            Some(val_typed.knowledge)
        }
        Statement::Expr { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
            None
        }
        Statement::Throw { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
            None
        }
        Statement::Class(class_def) => {
            super::declaration::check_class(ctx, class_def);
            None
        }
        Statement::For(for_stmt) => {
            let mut lane_facts = Vec::new();
            for lane in &for_stmt.lanes {
                let iter_k = synthesize_expr(ctx, &lane.iter);
                let elem_knowledge = if let Some(iter_ty) = iter_k.ty() {
                    resolve_iteration_element(ctx, iter_ty)
                } else if iter_k.is_dynamic() {
                    TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::RuntimeReflection)
                } else {
                    TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
                };

                let elem_fact = ValueSemanticFact::new(elem_knowledge);
                lane_facts.push((&lane.pattern, elem_fact));
            }
            ctx.push_scope();
            for (pat, fact) in lane_facts {
                if let Pattern::Name { name, range, .. } = pat {
                    ctx.bind_pattern_binding(name.clone(), fact, *range);
                }
            }
            for s in &for_stmt.body {
                check_statement(ctx, s);
            }
            ctx.pop_scope();
            None
        }
        _ => None,
    }
}

/// Derives the iteration element type from the receiver via the `iterate(_)` / `iteratorValue(_)` protocol (F5 / DEC-IMPL-FOR-PROTOCOL-ONLY).
pub fn resolve_iteration_element(ctx: &mut CheckingContext<'_>, receiver_ty: TypeId) -> TypeKnowledge {
    // 1. Try 1-argument protocol selector `iteratorValue(_)`
    if let Ok(sel_1) = Selector::method("iteratorValue", vec![phalcom_common::selector::SelectorSlot::Positional]) {
        let dispatch_res = ctx.resolve_dispatch(receiver_ty, &sel_1, crate::dispatch::DispatchLookup::Normal);
        match dispatch_res {
            crate::dispatch::DispatchResult::Found(sig) => {
                if !sig.return_type.is_unknown() {
                    return sig.return_type;
                }
            }
            crate::dispatch::DispatchResult::Dynamic => {
                return TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::RuntimeReflection);
            }
            _ => {}
        }
    }

    // 2. Try 0-argument selector / getter `iteratorValue`
    if let Ok(sel_0) = Selector::method("iteratorValue", vec![]) {
        let dispatch_res = ctx.resolve_dispatch(receiver_ty, &sel_0, crate::dispatch::DispatchLookup::Normal);
        match dispatch_res {
            crate::dispatch::DispatchResult::Found(sig) => {
                if !sig.return_type.is_unknown() {
                    return sig.return_type;
                }
            }
            crate::dispatch::DispatchResult::Dynamic => {
                return TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::RuntimeReflection);
            }
            _ => {}
        }
    }

    // 3. Check iterate(_) protocol existence. Presence without a resolved
    // element result is not enough to invent one from generic arguments.
    if let Ok(iterate_sel) = Selector::method("iterate", vec![phalcom_common::selector::SelectorSlot::Positional]) {
        let dispatch_res = ctx.resolve_dispatch(receiver_ty, &iterate_sel, crate::dispatch::DispatchLookup::Normal);
        if dispatch_res.is_found() {
            return TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration);
        }
    }

    TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend)
}
