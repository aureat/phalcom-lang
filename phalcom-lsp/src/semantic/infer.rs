//! Deterministic syntax and local-flow inference.

#![allow(clippy::only_used_in_recursion, clippy::too_many_arguments)]

use std::collections::BTreeMap;
use std::sync::Arc;

use super::CallableSummary;
use super::callable::SolverResult;
use super::dispatch::{DispatchReceiver, DispatchResolver, ResolvedDispatch};
use super::facts::{FieldFacts, InferredValue, LocalFacts, MAX_SHAPE_UNION, ParameterFacts, ValueShape};
#[cfg(test)]
use super::ids::CORE_MODULE_URI;
use super::ids::{CallableId, ClassId, DispatchSide, ModuleId};
use super::module_graph::ModuleGraph;
use super::query::SemanticGeneration;
use super::surface::ModuleSurface;

/// Collects constructor-assigned field facts from one module's source surface.
pub fn field_facts_for_surface(
    program: &phalcom_ast::ast::Program,
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    _is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    contains_class: impl Fn(&ClassId) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    callable_effects: impl Fn(&CallableId) -> Option<super::SummaryEffects> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolver: DispatchResolver<'_>,
) -> FieldFacts {
    let field_value = |_: &ClassId, _: &str, _: DispatchSide| None;
    super::flow::analyze_surface(
        program,
        surface,
        module,
        &known_class,
        &contains_class,
        &callable_return,
        &callable_effects,
        &parameter_fact,
        &field_value,
        &|receiver, selector| resolver.resolve(receiver, selector),
        SemanticGeneration(0),
    )
    .field_facts
}

/// Collects parameter facts from unambiguous call sites in one source module.
pub fn parameter_facts_for_program(
    program: &phalcom_ast::ast::Program,
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    _is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    contains_class: impl Fn(&ClassId) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    callable_effects: impl Fn(&CallableId) -> Option<super::SummaryEffects> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
) -> ParameterFacts {
    let field_value = |_: &ClassId, _: &str, _: DispatchSide| None;
    super::flow::analyze_surface(
        program,
        surface,
        module,
        &known_class,
        &contains_class,
        &callable_return,
        &callable_effects,
        &parameter_fact,
        &field_value,
        &resolve_member,
        SemanticGeneration(0),
    )
    .parameter_facts
}

/// Solves source callable summaries and parameter facts without mutating the
/// published semantic database.
#[expect(dead_code, reason = "retained as a full-workspace solver reference for regression comparisons")]
pub(crate) fn solve_workspace_callables(
    inputs: &[(ModuleId, Arc<phalcom_ast::ast::Program>, ModuleSurface)],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
) -> SolverResult {
    let callable_count = inputs
        .iter()
        .map(|(_, _, surface)| surface.classes.values().map(|class| class.members_by_side.len()).sum::<usize>())
        .sum::<usize>();
    let slot_count = inputs
        .iter()
        .map(|(_, _, surface)| {
            surface
                .classes
                .values()
                .flat_map(|class| class.members_by_side.values())
                .map(|member| member.params.len())
                .sum::<usize>()
        })
        .sum::<usize>();
    let max_rounds = (callable_count + slot_count).max(1) * (MAX_SHAPE_UNION + 2);
    let mut summaries = BTreeMap::new();
    let mut parameter_facts = ParameterFacts::default();

    for _ in 0..max_rounds {
        crate::perf::COUNTERS
            .solver_rounds
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let previous_summaries = summaries.clone();
        let previous_parameters = parameter_facts.clone();
        let mut next_parameters = ParameterFacts::default();

        for (module, program, surface) in inputs {
            let known_class = |name: &str| super::resolve_named_class(classes, graph, module, name);
            let resolver = DispatchResolver::new(classes);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolver
                    .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                    .is_some_and(|resolved| resolved.member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| super::return_for_callable(classes, &previous_summaries, id);
            let callable_effects = |id: &CallableId| previous_summaries.get(id).map(|summary| summary.effects.clone());
            let parameter_fact = |id: &CallableId, name: &str| previous_parameters.get(id, name).cloned();
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
            let facts = parameter_facts_for_program(
                program,
                surface,
                module,
                known_class,
                is_constructor,
                |class| classes.contains_key(class),
                callable_return,
                callable_effects,
                parameter_fact,
                resolve_member,
            );
            next_parameters.merge_from(&facts);
        }

        let mut next_summaries = BTreeMap::new();
        for (module, program, surface) in inputs {
            let known_class = |name: &str| super::resolve_named_class(classes, graph, module, name);
            let resolver = DispatchResolver::new(classes);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolver
                    .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                    .is_some_and(|resolved| resolved.member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| super::return_for_callable(classes, &previous_summaries, id);
            let callable_effects = |id: &CallableId| previous_summaries.get(id).map(|summary| summary.effects.clone());
            let parameter_fact = |id: &CallableId, name: &str| next_parameters.get(id, name).cloned();
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
            for (summary, evidence) in summaries_for_surface_with_bottom(
                program,
                surface,
                module,
                known_class,
                is_constructor,
                |class| classes.contains_key(class),
                callable_return,
                callable_effects,
                parameter_fact,
                resolve_member,
                generation,
            ) {
                if evidence.is_some() {
                    next_summaries.insert(summary.callable.clone(), summary);
                }
            }
        }

        let summaries_changed = next_summaries != previous_summaries;
        let parameters_changed = next_parameters != previous_parameters;
        summaries = next_summaries;
        parameter_facts = next_parameters;
        if !summaries_changed && !parameters_changed {
            complete_missing_summaries(inputs, classes, graph, generation, &parameter_facts, &mut summaries);
            return SolverResult { summaries, parameter_facts };
        }
    }

    if cfg!(debug_assertions) {
        panic!("callable solver failed to converge within derived budget");
    }

    // Release builds must still publish a coherent state. Widen all facts and
    // summaries together so no caller observes a partial round.
    parameter_facts.widen_all();
    for summary in summaries.values_mut() {
        summary.returns = InferredValue::flow(ValueShape::Unknown, Default::default());
        for value in &mut summary.params {
            *value = InferredValue::flow(ValueShape::Unknown, Default::default());
        }
    }
    SolverResult { summaries, parameter_facts }
}

/// Re-solves only callable surfaces owned by `inputs` while treating the
/// supplied summaries and parameter facts as read-only boundary values.
pub(crate) fn solve_affected_callables(
    inputs: &[(ModuleId, Arc<phalcom_ast::ast::Program>, ModuleSurface)],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
    seed_summaries: BTreeMap<CallableId, CallableSummary>,
    base_parameters: ParameterFacts,
) -> SolverResult {
    if inputs.is_empty() {
        return SolverResult {
            summaries: seed_summaries,
            parameter_facts: base_parameters,
        };
    }

    let callable_count = inputs
        .iter()
        .map(|(_, _, surface)| surface.classes.values().map(|class| class.members_by_side.len()).sum::<usize>())
        .sum::<usize>();
    let slot_count = inputs
        .iter()
        .map(|(_, _, surface)| {
            surface
                .classes
                .values()
                .flat_map(|class| class.members_by_side.values())
                .map(|member| member.params.len())
                .sum::<usize>()
        })
        .sum::<usize>();
    let max_rounds = (callable_count + slot_count).max(1) * (MAX_SHAPE_UNION + 2);
    let mut summaries = seed_summaries;
    let mut parameter_facts = base_parameters.clone();

    for _ in 0..max_rounds {
        crate::perf::COUNTERS
            .solver_rounds
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let previous_summaries = summaries.clone();
        let previous_parameters = parameter_facts.clone();
        let mut next_parameters = base_parameters.clone();

        for (module, program, surface) in inputs {
            let known_class = |name: &str| super::resolve_named_class(classes, graph, module, name);
            let resolver = DispatchResolver::new(classes);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolver
                    .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                    .is_some_and(|resolved| resolved.member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| super::return_for_callable(classes, &previous_summaries, id);
            let callable_effects = |id: &CallableId| previous_summaries.get(id).map(|summary| summary.effects.clone());
            let parameter_fact = |id: &CallableId, name: &str| previous_parameters.get(id, name).cloned();
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
            let facts = parameter_facts_for_program(
                program,
                surface,
                module,
                known_class,
                is_constructor,
                |class| classes.contains_key(class),
                callable_return,
                callable_effects,
                parameter_fact,
                resolve_member,
            );
            next_parameters.merge_from(&facts);
        }

        let mut next_summaries = previous_summaries.clone();
        for (module, program, surface) in inputs {
            let known_class = |name: &str| super::resolve_named_class(classes, graph, module, name);
            let resolver = DispatchResolver::new(classes);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolver
                    .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                    .is_some_and(|resolved| resolved.member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| super::return_for_callable(classes, &previous_summaries, id);
            let callable_effects = |id: &CallableId| previous_summaries.get(id).map(|summary| summary.effects.clone());
            let parameter_fact = |id: &CallableId, name: &str| next_parameters.get(id, name).cloned();
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
            for (summary, evidence) in summaries_for_surface_with_bottom(
                program,
                surface,
                module,
                known_class,
                is_constructor,
                |class| classes.contains_key(class),
                callable_return,
                callable_effects,
                parameter_fact,
                resolve_member,
                generation,
            ) {
                if evidence.is_some() {
                    next_summaries.insert(summary.callable.clone(), summary);
                }
            }
        }

        let summaries_changed = next_summaries != previous_summaries;
        let parameters_changed = next_parameters != previous_parameters;
        summaries = next_summaries;
        parameter_facts = next_parameters;
        if !summaries_changed && !parameters_changed {
            complete_missing_summaries(inputs, classes, graph, generation, &parameter_facts, &mut summaries);
            return SolverResult { summaries, parameter_facts };
        }
    }

    if cfg!(debug_assertions) {
        panic!("affected callable solver failed to converge within derived budget");
    }

    parameter_facts.widen_all();
    for summary in summaries.values_mut() {
        if inputs.iter().any(|(module, _, _)| summary.callable.owner.module == *module) {
            summary.returns = InferredValue::flow(ValueShape::Unknown, Default::default());
            for value in &mut summary.params {
                *value = InferredValue::flow(ValueShape::Unknown, Default::default());
            }
        }
    }
    SolverResult { summaries, parameter_facts }
}

fn complete_missing_summaries(
    inputs: &[(ModuleId, Arc<phalcom_ast::ast::Program>, ModuleSurface)],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
    parameter_facts: &ParameterFacts,
    summaries: &mut BTreeMap<CallableId, CallableSummary>,
) {
    for (module, program, surface) in inputs {
        let known_class = |name: &str| super::resolve_named_class(classes, graph, module, name);
        let resolver = DispatchResolver::new(classes);
        let is_constructor = |class: &ClassId, selector: &str| {
            resolver
                .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                .is_some_and(|resolved| resolved.member.is_constructor)
                || (selector == "new()" && classes.contains_key(class))
        };
        let callable_return = |id: &CallableId| super::return_for_callable(classes, summaries, id);
        let callable_effects = |id: &CallableId| summaries.get(id).map(|summary| summary.effects.clone());
        let parameter_fact = |id: &CallableId, name: &str| parameter_facts.get(id, name).cloned();
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
        let generated = summaries_for_surface(
            program,
            surface,
            module,
            known_class,
            is_constructor,
            |class| classes.contains_key(class),
            callable_return,
            callable_effects,
            parameter_fact,
            resolve_member,
            generation,
        );
        for summary in generated {
            summaries.entry(summary.callable.clone()).or_insert(summary);
        }
    }
}

pub fn collect_local_facts_with_returns(
    program: &phalcom_ast::ast::Program,
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    _is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    contains_class: impl Fn(&ClassId) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    callable_effects: impl Fn(&CallableId) -> Option<super::SummaryEffects> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
) -> LocalFacts {
    let field_value = |_: &ClassId, _: &str, _: DispatchSide| None;
    super::flow::analyze_surface(
        program,
        surface,
        module,
        &known_class,
        &contains_class,
        &callable_return,
        &callable_effects,
        &parameter_fact,
        &field_value,
        &resolve_member,
        SemanticGeneration(0),
    )
    .local_facts
}

/// Computes one-pass callable return summaries for a source module surface.
pub fn summaries_for_surface(
    program: &phalcom_ast::ast::Program,
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    _is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    contains_class: impl Fn(&ClassId) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    callable_effects: impl Fn(&CallableId) -> Option<super::SummaryEffects> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
    revision: SemanticGeneration,
) -> Vec<CallableSummary> {
    let field_value = |_: &ClassId, _: &str, _: DispatchSide| None;
    super::flow::analyze_surface(
        program,
        surface,
        module,
        &known_class,
        &contains_class,
        &callable_return,
        &callable_effects,
        &parameter_fact,
        &field_value,
        &resolve_member,
        revision,
    )
    .summaries
    .into_iter()
    .filter_map(|(summary, evidence)| evidence.then_some(summary))
    .collect()
}

type SummaryEvidence = Option<InferredValue>;

fn summaries_for_surface_with_bottom(
    program: &phalcom_ast::ast::Program,
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    _is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    contains_class: impl Fn(&ClassId) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    callable_effects: impl Fn(&CallableId) -> Option<super::SummaryEffects> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch> + Copy,
    revision: SemanticGeneration,
) -> Vec<(CallableSummary, SummaryEvidence)> {
    let field_value = |_: &ClassId, _: &str, _: DispatchSide| None;
    super::flow::analyze_surface(
        program,
        surface,
        module,
        &known_class,
        &contains_class,
        &callable_return,
        &callable_effects,
        &parameter_fact,
        &field_value,
        &resolve_member,
        revision,
    )
    .summaries
    .into_iter()
    .map(|(summary, evidence)| {
        let returns = summary.returns.clone();
        (summary, evidence.then_some(returns))
    })
    .collect()
}

#[cfg(test)]
fn core_class(name: &str) -> ClassId {
    ClassId::new(ModuleId::new(CORE_MODULE_URI), name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;

    fn no_classes(_: &str) -> Option<ClassId> {
        None
    }

    #[test]
    fn literals_and_reassignment_are_queryable() {
        let program = parse("let value = 1\nvalue = \"ok\"\n", 0).program;
        let module = ModuleId::new("file:///main.ph");
        let surface = super::super::surface::build_module_surface(module.clone(), &program);
        let facts = collect_local_facts_with_returns(
            &program,
            &surface,
            &module,
            no_classes,
            |_, _| false,
            |_| false,
            |_| None,
            |_| None,
            |_, _| None,
            |_, _| None,
        );
        let scopes = super::super::scope::build_scope_graph(module, &program);
        let binding = match scopes.resolve(scopes.scope_at(10), "value", 10) {
            super::super::scope::NameResolution::Binding(binding) => binding,
            other => panic!("expected binding, got {other:?}"),
        };
        assert_eq!(facts.value_before(binding, 10).unwrap().shape, ValueShape::Instance(core_class("Int")));
        assert_eq!(facts.value_before(binding, 30).unwrap().shape, ValueShape::Instance(core_class("String")));
    }

    #[test]
    fn union_widens_after_nine_distinct_shapes() {
        let module = ModuleId::new("file:///main.ph");
        let mut shape = ValueShape::Instance(ClassId::new(module.clone(), "C0"));
        for index in 1..=7 {
            shape = shape.join(&ValueShape::Instance(ClassId::new(module.clone(), format!("C{index}"))));
        }
        assert!(matches!(shape, ValueShape::Union(_)));
        shape = shape.join(&ValueShape::Instance(ClassId::new(module, "C8")));
        assert_eq!(shape, ValueShape::Unknown);
    }
}
