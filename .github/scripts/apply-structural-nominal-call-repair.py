from pathlib import Path

path = Path("phalcom-semantic/src/checker/expression.rs")
text = path.read_text()
old = '''        if let Some(ty) = fact.knowledge.ty() {
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
            return analyze_non_callable_invocation(ctx, &premise, &call.args, call.range).into();
        }
'''
new = '''        if let Some(ty) = fact.knowledge.ty() {
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
'''
if old not in text:
    raise SystemExit("structural nominal call anchor not found")
path.write_text(text.replace(old, new, 1))
