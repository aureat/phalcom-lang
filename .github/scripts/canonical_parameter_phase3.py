from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old in text:
        return text.replace(old, new, 1)
    if new in text:
        return text
    raise SystemExit(f"{label} shape changed")


flow_path = Path("phalcom-semantic/src/advisory/flow.rs")
flow = flow_path.read_text()
flow = replace_once(
    flow,
    "use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId, SourceSiteId};",
    "use crate::identity::{CallableId, CallableParameterId, DeclarationId, DispatchSide, FieldId, SourceSiteId};",
    "flow identity import",
)
flow = flow.replace(
    "call.transfer_target\n                .selector\n                .slots",
    "call.target\n                .selector\n                .slots",
)
flow = flow.replace(
    "call\n                .transfer_target\n                .selector\n                .slots",
    "call\n                .target\n                .selector\n                .slots",
)
flow = replace_once(
    flow,
    "let slot = AdvisoryParameterSlot::new(call.transfer_target.clone(), index as u32);",
    "let slot = CallableParameterId::new(call.target.clone(), index as u32);",
    "call contribution slot construction",
)
if "transfer_target\n                .selector\n                .slots" in flow:
    raise SystemExit("transfer-target selector reconstruction remains")
flow_path.write_text(flow)

session_path = Path("phalcom-semantic/src/session.rs")
session = session_path.read_text()

session = replace_once(
    session,
    '''                let mut seed_bindings = BTreeMap::new();
                for binding in callable_parameter_bindings(scope_index, &analysis.callable) {
                    let index = seed_bindings.len() as u32;
                    let slot = AdvisoryParameterSlot::new(analysis.callable.clone(), index);
                    let fact = parameter_facts
                        .get(&slot)
                        .cloned()
                        .unwrap_or_else(|| AdvisoryFact::unknown().derive(AdvisoryConfidence::Flow, AdvisoryOrigin::Binding(binding.declaration_site.clone())));
                    seed_bindings.insert(binding.declaration_site.clone(), fact);
                }
''',
    '''                let mut seed_bindings = BTreeMap::new();
                for (parameter, binding) in callable_parameter_bindings(scope_index, analysis) {
                    let fact = parameter_facts
                        .get(parameter)
                        .cloned()
                        .unwrap_or_else(|| AdvisoryFact::unknown().derive(AdvisoryConfidence::Flow, AdvisoryOrigin::Binding(binding.declaration_site.clone())));
                    seed_bindings.insert(binding.declaration_site.clone(), fact);
                }
''',
    "seed parameter reconstruction",
)

session = replace_once(
    session,
    '''                let mut summary_parameters = Vec::new();
                for (index, binding) in callable_parameter_bindings(scope_index, &analysis.callable).into_iter().enumerate() {
                    let slot = AdvisoryParameterSlot::new(analysis.callable.clone(), index as u32);
                    let fact = parameter_facts
                        .get(&slot)
                        .cloned()
                        .or_else(|| bindings.get(&binding.declaration_site).cloned())
                        .unwrap_or_else(AdvisoryFact::unknown);
                    parameters.insert(slot.clone(), fact.clone());
                    summary_parameters.push((slot, fact));
                }
''',
    '''                let mut summary_parameters = Vec::new();
                for (parameter, binding) in callable_parameter_bindings(scope_index, analysis) {
                    let fact = parameter_facts
                        .get(parameter)
                        .cloned()
                        .or_else(|| bindings.get(&binding.declaration_site).cloned())
                        .unwrap_or_else(AdvisoryFact::unknown);
                    parameters.insert(parameter.clone(), fact.clone());
                    summary_parameters.push((parameter.clone(), fact));
                }
''',
    "summary parameter reconstruction",
)

session = replace_once(
    session,
    '''            let own_parameters = next_parameter_facts
                .iter()
                .filter(|(slot, _)| slot.callable == *callable)
                .map(|(slot, fact)| (slot.clone(), fact.clone()))
                .collect::<BTreeMap<_, _>>();
''',
    '''            let own_parameter_ids = summary.parameters.iter().map(|(parameter, _)| parameter.clone()).collect::<BTreeSet<_>>();
            let own_parameters = next_parameter_facts
                .iter()
                .filter(|(parameter, _)| own_parameter_ids.contains(*parameter))
                .map(|(parameter, fact)| (parameter.clone(), fact.clone()))
                .collect::<BTreeMap<_, _>>();
''',
    "solver parameter ownership",
)

session = replace_once(
    session,
    '''fn callable_parameter_bindings<'a>(
    scope_index: &'a crate::source_index::SourceScopeIndex,
    callable: &CallableId,
) -> Vec<&'a crate::source_index::SourceBindingInfo> {
    let mut bindings = scope_index
        .bindings
        .values()
        .filter(|binding| {
            binding.declaration_site.owner == SourceOwner::Callable(callable.clone())
                && matches!(
                    binding.kind,
                    crate::source_index::SourceBindingKind::MethodParameter
                        | crate::source_index::SourceBindingKind::SetterParameter
                        | crate::source_index::SourceBindingKind::IndexParameter
                )
        })
        .collect::<Vec<_>>();
    bindings.sort_by_key(|binding| (binding.declaration_range.start, binding.declaration_range.end));
    bindings
}
''',
    '''fn callable_parameter_bindings<'a>(
    scope_index: &'a crate::source_index::SourceScopeIndex,
    analysis: &'a crate::checker::CallableAnalysis,
) -> Vec<(&'a crate::identity::CallableParameterId, &'a crate::source_index::SourceBindingInfo)> {
    let mut bindings = analysis
        .bindings
        .values()
        .filter_map(|binding| {
            let parameter = binding.parameter.as_ref()?;
            let callable_source = scope_index.callable_sources.get(&parameter.callable)?;
            let site = callable_source.parameter_sites.get(parameter)?;
            let source_binding = scope_index.bindings.get(site)?;
            Some((parameter, source_binding))
        })
        .collect::<Vec<_>>();
    bindings.sort_by_key(|(parameter, _)| parameter.index);
    bindings
}
''',
    "callable parameter helper",
)

for forbidden in (
    "callable_parameter_bindings(scope_index, &analysis.callable)",
    "bindings.sort_by_key(|binding| (binding.declaration_range.start, binding.declaration_range.end))",
    "AdvisoryParameterSlot::new(analysis.callable.clone()",
):
    if forbidden in session:
        raise SystemExit(f"transitional parameter reconstruction remains: {forbidden}")

session_path.write_text(session)
