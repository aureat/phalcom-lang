//! Statement type checking engine.

use super::context::CheckingContext;
use super::expression::{synthesize_expr, synthesize_typed_expr};
use super::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::annotation::resolve_type_annotation;
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::{EvidenceAuthority, TypeKnowledge, UnknownReason};
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
                let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
                ctx.diagnostics.extend(diags);
                k
            });

            let val_typed = if let Some(expr) = &binding.value {
                synthesize_typed_expr(ctx, expr)
            } else {
                TypedExpression::unknown(UnknownReason::UnannotatedDeclaration)
            };

            let mut is_assignable = true;
            if let Some(ref decl_k) = declared_k {
                if binding.value.is_some() {
                    let assignability = check_assignability(ctx.store, ctx.hierarchy, &val_typed.knowledge, decl_k);
                    if let Assignability::Refuted { .. } = assignability {
                        is_assignable = false;
                        let mut diag = SemanticDiagnostic::error(
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

            match &binding.pattern {
                Pattern::Name { name, .. } => {
                    ctx.bind_local(name.clone(), effective_fact);
                }
                _ => {}
            }
        }
        Statement::Return(ret) => {
            let val_k = if let Some(expr) = &ret.value {
                synthesize_expr(ctx, expr)
            } else {
                TypeKnowledge::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax)
            };

            if let Some(expected) = ctx.expected_return.clone() {
                let assignability = check_assignability(ctx.store, ctx.hierarchy, &val_k, &expected);
                if let Assignability::Refuted { .. } = assignability {
                    ctx.diagnostics.push(SemanticDiagnostic::error(
                        DiagnosticCode::ReturnMismatch,
                        "returned value is not assignable to method's declared return type",
                        ret.range,
                    ));
                }
            }
        }
        Statement::Expr { expr, .. } => {
            synthesize_expr(ctx, expr);
        }
        Statement::Throw { expr, .. } => {
            synthesize_expr(ctx, expr);
        }
        Statement::Class(class_def) => {
            super::declaration::check_class(ctx, class_def);
        }
        Statement::For(for_stmt) => {
            let mut lane_facts = Vec::new();
            for lane in &for_stmt.lanes {
                let iter_k = synthesize_expr(ctx, &lane.iter);
                let elem_knowledge = if let Some(iter_ty) = iter_k.ty() {
                    // 1. Direct collection element typing for List<T>, Set<T>, Map<K, V>
                    if let TypeData::Applied { origin, arguments } = ctx.store.get(iter_ty).clone() {
                        if let TypeData::Nominal { declaration } = ctx.store.get(origin) {
                            if declaration.name.as_ref() == "List" && arguments.len() == 1 {
                                TypeKnowledge::known(arguments[0], EvidenceAuthority::Proven)
                            } else if declaration.name.as_ref() == "Set" && arguments.len() == 1 {
                                TypeKnowledge::known(arguments[0], EvidenceAuthority::Proven)
                            } else {
                                TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
                            }
                        } else {
                            TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
                        }
                    } else {
                        // 2. Protocol dispatch: iteratorValue(cursor)
                        if let Ok(sel) = Selector::method("iteratorValue", vec![]) {
                            let dispatch_res = ctx.resolve_dispatch(iter_ty, &sel, crate::dispatch::DispatchLookup::Normal);
                            match dispatch_res {
                                crate::dispatch::DispatchResult::Found(sig) => sig.return_type,
                                crate::dispatch::DispatchResult::Dynamic => TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::RuntimeReflection),
                                _ => TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend),
                            }
                        } else {
                            TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend)
                        }
                    }
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
                match pat {
                    Pattern::Name { name, .. } => {
                        ctx.bind_local(name.clone(), fact);
                    }
                    _ => {}
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
