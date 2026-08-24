//! Statement type checking engine.

use super::context::CheckingContext;
use super::expected::ExpectedType;
use super::expression::{analyze_expression, synthesize_expr};
use super::policy::enforce_assignability;
use super::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::annotation::resolve_type_annotation;
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::{EvidenceAuthority, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::relation::{Assignability, check_assignability};
use crate::types::store::TypeData;
use phalcom_ast::ast::{Pattern, Statement};
use phalcom_common::selector::Selector;

/// Checks a single statement, updating context bindings and recording diagnostics.
pub fn check_statement(ctx: &mut CheckingContext<'_>, statement: &Statement) {
    match statement {
        Statement::Let(binding) => {
            let declared_k = binding.annotation.as_ref().map(|ann| {
                let mut diags = Vec::new();
                let k = resolve_type_annotation(ctx.store, ctx.declarations, &ctx.resolver, &ctx.current_module, ann, &mut diags);
                ctx.diagnostics.extend(diags);
                k
            });

            let expected_init = declared_k.as_ref().map(ExpectedType::from_knowledge).unwrap_or_default();
            let val_typed = if let Some(expr) = &binding.value {
                analyze_expression(ctx, expr, &expected_init)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            };

            let mut is_assignable = true;
            if let Some(ref decl_k) = declared_k {
                if binding.value.is_some() {
                    let assignability = check_assignability(ctx.store, &ctx.hierarchy, &val_typed.knowledge, decl_k);
                    if let Assignability::Refuted { .. } = assignability {
                        is_assignable = false;
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
                        ctx.diagnostics.push(diag);
                    }
                }
            }

            let effective_fact = if let Some(decl_k) = declared_k {
                let denotation = if is_assignable { val_typed.denotation } else { None };
                ValueSemanticFact { knowledge: decl_k, denotation }
            } else {
                val_typed.fact()
            };

            if let Pattern::Name { name, .. } = &binding.pattern {
                ctx.bind_local(name.clone(), effective_fact, binding.range);
            }
        }
        Statement::Return(ret) => {
            let expected_ret = ctx.expected_return.as_ref().map(ExpectedType::from_knowledge).unwrap_or_default();
            let val_typed = if let Some(expr) = &ret.value {
                analyze_expression(ctx, expr, &expected_ret)
            } else {
                TypedExpression::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax, ret.range)
            };

            if let Some(expected) = ctx.expected_return.clone() {
                enforce_assignability(
                    ctx.store,
                    &ctx.hierarchy,
                    &val_typed.knowledge,
                    &expected,
                    &ctx.current_module,
                    DiagnosticCode::ReturnMismatch,
                    "returned value is not assignable to method's declared return type",
                    ret.range,
                    &mut ctx.diagnostics,
                );
            }
        }
        Statement::Expr { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
        }
        Statement::Throw { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
        }
        Statement::Class(class_def) => {
            super::declaration::check_class(ctx, class_def);
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
                    ctx.bind_local(name.clone(), fact, *range);
                }
            }
            for s in &for_stmt.body {
                check_statement(ctx, s);
            }
            ctx.pop_scope();
        }
        _ => {}
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

    // 3. For Applied collections where origin declaration has generic parameters (e.g. List<T>, Set<T>),
    // if Applied has type arguments, derive element type if the origin has an iterator protocol or single type parameter
    if let TypeData::Applied { origin, arguments } = ctx.store.get(receiver_ty).clone() {
        if let TypeData::Nominal { declaration } = ctx.store.get(origin) {
            if let Some(sig) = ctx.declaration_generic_signature(declaration) {
                if !sig.parameters.is_empty() && !arguments.is_empty() {
                    return TypeKnowledge::known(arguments[0], EvidenceAuthority::Proven);
                }
            }
        }
    }

    // 4. Check iterate(_) protocol existence
    if let Ok(iterate_sel) = Selector::method("iterate", vec![phalcom_common::selector::SelectorSlot::Positional]) {
        let dispatch_res = ctx.resolve_dispatch(receiver_ty, &iterate_sel, crate::dispatch::DispatchLookup::Normal);
        if dispatch_res.is_found() {
            return TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration);
        }
    }

    TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend)
}
