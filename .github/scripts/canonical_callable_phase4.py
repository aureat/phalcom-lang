from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old in text:
        return text.replace(old, new, 1)
    if new in text:
        return text
    raise SystemExit(f"{label} shape changed")


path = Path("phalcom-semantic/src/session.rs")
text = path.read_text()

text = replace_once(
    text,
    "    AdvisoryParameterSlot, AdvisoryProductStatus, AdvisorySolver, AdvisorySolverBudget, AdvisorySolverNode, AdvisoryTargetResolution, AdvisoryWorkspace,\n",
    "    AdvisoryProductStatus, AdvisorySolver, AdvisorySolverBudget, AdvisorySolverNode, AdvisoryTargetResolution, AdvisoryWorkspace,\n",
    "obsolete advisory parameter import",
)

text = replace_once(
    text,
    '''                                                    .filter(|analysis| {
                                                        dispatch
                                                            .get_surface(&decl_id)
                                                            .and_then(|surface| surface.get_callable(DispatchSide::Class, &analysis.callable.selector))
                                                            .is_some_and(|signature| signature.kind == crate::dispatch::CallableSemanticKind::Constructor)
                                                    })
''',
    '''                                                    .filter(|analysis| {
                                                        callable_signatures
                                                            .get_for_body(&analysis.callable)
                                                            .is_some_and(|signature| signature.is_constructor())
                                                    })
''',
    "field lifecycle constructor authority",
)

text = replace_once(
    text,
    '''            &declarations,
            &dispatch,
            &hierarchy,
''',
    '''            &declarations,
            &callable_signatures,
            &dispatch,
            &hierarchy,
''',
    "advisory workspace call signature table",
)

text = replace_once(
    text,
    '''    store: &TypeStore,
    declarations: &DeclarationTypeTable,
    dispatch: &SurfaceDispatchResolver,
''',
    '''    store: &TypeStore,
    declarations: &DeclarationTypeTable,
    callable_signatures: &CallableSignatureTable,
    dispatch: &SurfaceDispatchResolver,
''',
    "advisory workspace parameter signature table",
)

text = replace_once(
    text,
    '''    let resolve_formal_call_result = |callable: &CallableId, receiver: Option<&crate::advisory::ValueShape>| {
        let signature = dispatch.get_surface(&callable.owner)?.get_callable(callable.side, &callable.selector)?;
        let shape = receiver.map_or_else(
            || advisory_shape_from_formal(store, &signature.return_type),
            |receiver| advisory_shape_from_formal_for_receiver(store, &signature.return_type, receiver),
        );
        (!matches!(shape, crate::advisory::ValueShape::Unknown)).then(|| {
            AdvisoryFact::new(shape, AdvisoryConfidence::Interprocedural)
                .derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::Callable(callable.clone()))
        })
    };
    let advisory_transfer_target = |callable: &CallableId| {
        let is_constructor = dispatch
            .get_surface(&callable.owner)
            .and_then(|surface| surface.get_callable(callable.side, &callable.selector))
            .is_some_and(|signature| signature.kind == crate::dispatch::CallableSemanticKind::Constructor);
        if is_constructor && callable.side == DispatchSide::Class {
            CallableId::new(callable.owner.clone(), callable.selector.clone(), DispatchSide::Instance)
        } else {
            callable.clone()
        }
    };
''',
    '''    let resolve_formal_call_result = |callable: &CallableId, receiver: Option<&crate::advisory::ValueShape>| {
        let signature = callable_signatures.get(callable)?;
        let return_knowledge = signature.published_return_knowledge();
        let shape = receiver.map_or_else(
            || advisory_shape_from_formal(store, &return_knowledge),
            |receiver| advisory_shape_from_formal_for_receiver(store, &return_knowledge, receiver),
        );
        (!matches!(shape, crate::advisory::ValueShape::Unknown)).then(|| {
            AdvisoryFact::new(shape, AdvisoryConfidence::Interprocedural)
                .derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::Callable(signature.callable.clone()))
        })
    };
    let advisory_transfer_target = |callable: &CallableId| {
        let is_constructor = callable_signatures.get(callable).is_some_and(|signature| signature.is_constructor());
        if is_constructor && callable.side == DispatchSide::Class {
            CallableId::new(callable.owner.clone(), callable.selector.clone(), DispatchSide::Instance)
        } else {
            callable.clone()
        }
    };
''',
    "advisory callable authority closures",
)

text = replace_once(
    text,
    '''    let mut formal_returns = BTreeMap::new();
    let mut ordered_analyses = callable_analyses.values().cloned().collect::<Vec<_>>();
    ordered_analyses.sort_by(|left, right| left.callable.cmp(&right.callable));
    for analysis in &ordered_analyses {
        formal_returns.insert(analysis.callable.clone(), advisory_return_fact(store, analysis));
    }
''',
    '''    let mut formal_returns = BTreeMap::new();
    let mut ordered_analyses = callable_analyses.values().cloned().collect::<Vec<_>>();
    ordered_analyses.sort_by(|left, right| left.callable.cmp(&right.callable));
    for analysis in &ordered_analyses {
        let fact = callable_signatures
            .get_for_body(&analysis.callable)
            .map(|signature| {
                let return_knowledge = signature.published_return_knowledge();
                let shape = if signature.is_constructor() {
                    let receiver = crate::advisory::ValueShape::ClassObject(signature.owner.clone());
                    advisory_shape_from_formal_for_receiver(store, &return_knowledge, &receiver)
                } else {
                    advisory_shape_from_formal(store, &return_knowledge)
                };
                AdvisoryFact::new(shape, AdvisoryConfidence::Interprocedural).derive(
                    AdvisoryConfidence::Interprocedural,
                    AdvisoryOrigin::Callable(signature.callable.clone()),
                )
            })
            .unwrap_or_else(AdvisoryFact::unknown);
        formal_returns.insert(analysis.callable.clone(), fact);
    }
''',
    "formal return canonical seeding",
)

text = replace_once(
    text,
    '''fn advisory_return_fact(store: &TypeStore, analysis: &crate::checker::CallableAnalysis) -> AdvisoryFact {
    if analysis.exits.normal_return_values.is_empty() {
        return AdvisoryFact::unknown();
    }
    let mut result = None;
    for knowledge in &analysis.exits.normal_return_values {
        let fact = match knowledge {
            TypeKnowledge::Known(_) => advisory_fact_from_formal(store, knowledge, AdvisoryOrigin::Callable(analysis.callable.clone())),
            TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => AdvisoryFact::unknown(),
        };
        result = Some(result.map_or(fact.clone(), |current: AdvisoryFact| current.join(&fact)));
    }
    result.unwrap_or_else(AdvisoryFact::unknown)
}

''',
    "",
    "obsolete body-derived advisory return helper",
)

text = replace_once(
    text,
    '''        let candidates = callable_analyses
            .iter()
            .filter_map(|(callable, analysis)| {
                let surface = dispatch.surfaces().get(&callable.owner)?;
                let signature = surface.get_callable(callable.side, &callable.selector).or_else(|| {
                    (callable.side == crate::identity::DispatchSide::Instance)
                        .then(|| surface.get_callable(crate::identity::DispatchSide::Class, &callable.selector))
                        .flatten()
                })?;
                if !signature.return_type.is_unknown() {
                    return None;
                }
                Some((callable.clone(), analysis.exits.normal_return_values.clone()))
            })
            .collect::<Vec<_>>();

        let mut changed_callables = HashSet::new();
        for (callable, values) in candidates {
            let summary = normal_return_summary(store, &values);
            if !summary.is_known() {
                continue;
            }
            if !dispatch.update_callable_return_type(&callable, summary.clone()) {
                continue;
            }
            changed_callables.insert(callable.clone());

            let signature_id = if callable_signatures.get(&callable).is_some() {
                Some(callable.clone())
            } else if callable.side == DispatchSide::Instance {
                let class_side = CallableId::new(callable.owner.clone(), callable.selector.clone(), DispatchSide::Class);
                callable_signatures.get(&class_side).is_some().then_some(class_side)
            } else {
                None
            };
            if let Some(signature_id) = signature_id
                && let Some(signature) = callable_signatures.get_mut(&signature_id)
            {
                signature.inferred_return = Some(summary.clone());
            }
        }
''',
    '''        let candidates = callable_analyses
            .iter()
            .filter_map(|(callable, analysis)| {
                let signature = callable_signatures.get_for_body(callable)?;
                if !signature.published_return_knowledge().is_unknown() {
                    return None;
                }
                Some((
                    callable.clone(),
                    signature.callable.clone(),
                    analysis.exits.normal_return_values.clone(),
                ))
            })
            .collect::<Vec<_>>();

        let mut changed_callables = HashSet::new();
        for (callable, signature_id, values) in candidates {
            let summary = normal_return_summary(store, &values);
            if !summary.is_known() {
                continue;
            }
            let Some(signature) = callable_signatures.get_mut(&signature_id) else {
                continue;
            };
            if signature.inferred_return.as_ref() == Some(&summary) {
                continue;
            }
            signature.inferred_return = Some(summary.clone());
            changed_callables.insert(callable.clone());

            // Dispatch is a derived lookup projection. Failure to update that
            // projection must never suppress canonical semantic publication.
            let _ = dispatch.update_callable_return_type(&signature_id, summary);
        }
''',
    "inferred return canonical ownership",
)

text = replace_once(
    text,
    '''                    let signature_id = if callable_signatures.get(&callable).is_some() {
                        Some(callable.clone())
                    } else if callable.side == DispatchSide::Instance {
                        let class_side = CallableId::new(callable.owner.clone(), callable.selector.clone(), DispatchSide::Class);
                        callable_signatures.get(&class_side).is_some().then_some(class_side)
                    } else {
                        None
                    };
                    let declared_signature = signature_id
                        .as_ref()
                        .and_then(|signature_id| callable_signatures.get(signature_id).map(|signature| (signature_id, signature)));
''',
    '''                    let declared_signature = callable_signatures
                        .get_for_body(&callable)
                        .map(|signature| (&signature.callable, signature));
''',
    "body recheck canonical signature lookup",
)

text = text.replace(
    "/// Propagates body-derived return summaries through source dispatch. Source\n/// declaration surfaces are intentionally built before body checking, so this\n/// small fixed-point pass is required for calls such as `Probe.run ->\n/// Factory.of -> CellNum.new`.\n",
    "/// Publishes body-derived return summaries into canonical callable signatures,\n/// then refreshes dispatch as a derived lookup projection. The fixed-point pass\n/// is required for calls such as `Probe.run -> Factory.of -> CellNum.new`.\n",
)

for forbidden in (
    "dispatch.get_surface(&callable.owner)?.get_callable(callable.side, &callable.selector)?",
    "let surface = dispatch.surfaces().get(&callable.owner)?;",
    "signature.kind == crate::dispatch::CallableSemanticKind::Constructor",
    "advisory_return_fact(store, analysis)",
):
    if forbidden in text:
        raise SystemExit(f"reverse callable authority remains: {forbidden}")

required_after = {
    "field lifecycle constructor authority": "callable_signatures\n                                                            .get_for_body(&analysis.callable)",
    "advisory formal return authority": "let fact = callable_signatures\n            .get_for_body(&analysis.callable)",
    "inferred return authority": "let signature = callable_signatures.get_for_body(callable)?;",
    "body recheck authority": "let declared_signature = callable_signatures\n                        .get_for_body(&callable)",
}
for label, snippet in required_after.items():
    if snippet not in text:
        raise SystemExit(f"missing canonical authority site: {label}")

path.write_text(text)
