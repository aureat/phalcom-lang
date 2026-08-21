//! Deterministic syntax and local-flow inference.

#![allow(clippy::only_used_in_recursion, clippy::too_many_arguments)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::CallableSummary;
use super::callable::{CallableWorklist, SolverResult};
use super::dispatch::{DispatchReceiver, DispatchResolver};
use super::facts::{ContributionSource, InferredValue, MAX_SHAPE_UNION, ParameterContributions, ParameterFacts, ValueShape};
#[cfg(test)]
use super::ids::CORE_MODULE_URI;
use super::ids::{CallableId, ClassId, DispatchSide, ModuleId};
use super::module_graph::ModuleGraph;
use super::query::SemanticGeneration;

use super::flow::{SolverContext, SurfaceFlowAnalysis};
use super::snapshot::FileSourceSnapshot;
use crate::perf::PerfCounters;

#[cfg(test)]
thread_local! {
    static TEST_SOLVER_ROUNDS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static TEST_SOLVER_STEPS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn test_solver_rounds() -> u64 {
    TEST_SOLVER_ROUNDS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(crate) fn test_solver_steps() -> u64 {
    TEST_SOLVER_STEPS.with(std::cell::Cell::get)
}

fn record_solver_round(counters: &PerfCounters) {
    #[cfg(test)]
    TEST_SOLVER_ROUNDS.with(|count| count.set(count.get() + 1));
    counters.solver_rounds.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

fn record_solver_step() {
    #[cfg(test)]
    TEST_SOLVER_STEPS.with(|count| count.set(count.get() + 1));
}

fn solver_budget(callable_count: usize, slot_count: usize) -> usize {
    let possible_dependency_edges = callable_count.saturating_mul(callable_count);
    callable_count
        .saturating_add(slot_count)
        .saturating_add(possible_dependency_edges)
        .max(1)
        .saturating_mul(MAX_SHAPE_UNION + 2)
}

fn joined_parameter_facts(contributions: &ParameterContributions) -> ParameterFacts {
    contributions.joined_iter().fold(ParameterFacts::default(), |mut facts, (slot, value)| {
        facts.record(slot.callable.clone(), slot.name.clone(), value.clone());
        facts
    })
}

fn record_parameter_replacement(counters: &PerfCounters, replacement_slots: usize, deltas: &[super::facts::ParameterFactDelta]) {
    counters.parameter_sources_replaced.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    counters
        .parameter_slots_touched
        .fetch_add(replacement_slots as u64, std::sync::atomic::Ordering::Relaxed);
    counters
        .parameter_slots_changed
        .fetch_add(deltas.len() as u64, std::sync::atomic::Ordering::Relaxed);
}

/// Coherent products from one bounded callable solve and its final source flow
/// pass. Engine publication consumes these analyses directly.
#[derive(Clone, Debug, Default)]
pub(crate) struct FlowSolveResult {
    /// One unified product per affected source module.
    pub source_analyses: BTreeMap<ModuleId, SurfaceFlowAnalysis>,
    /// Callable bodies actually visited by the incremental worklist.
    pub callables_visited: BTreeSet<CallableId>,
    /// Visited callables whose semantic summary changed.
    pub callables_changed: BTreeSet<CallableId>,
}

/// Builds the immutable solver context for one source-backed flow pass.
pub(crate) fn analyze_source(
    source: &FileSourceSnapshot,
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    summaries: &BTreeMap<CallableId, CallableSummary>,
    parameters: &ParameterFacts,
    generation: SemanticGeneration,
    counters: &PerfCounters,
) -> SurfaceFlowAnalysis {
    let known_class = |name: &str| super::resolve_named_class(classes, graph, &source.module, name);
    let contains_class = |class: &ClassId| classes.contains_key(class);
    let callable_return = |id: &CallableId| super::return_for_callable(classes, summaries, id);
    let callable_effects = |id: &CallableId| summaries.get(id).map(|summary| summary.effects.clone());
    let parameter_fact = |id: &CallableId, name: &str| parameters.get(id, name).cloned();
    let field_value = |_: &ClassId, _: &str, _: DispatchSide| None;
    let resolver = DispatchResolver::new(classes);
    let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
    let family_resolver = |receiver: &DispatchReceiver, pattern: &phalcom_common::selector::SelectorPattern| resolver.capture_method_family(receiver, pattern);
    let is_same_or_subclass = |child: &ClassId, ancestor: &ClassId| super::is_same_or_subclass(classes, child, ancestor);
    let member_surface = |id: &CallableId| classes.get(&id.owner).and_then(|class| class.member_by_id(id).cloned());
    let context = SolverContext {
        known_class: &known_class,
        contains_class: &contains_class,
        callable_return: &callable_return,
        callable_effects: &callable_effects,
        parameter_fact: &parameter_fact,
        field_value: &field_value,
        resolve_member: &resolve_member,
        family_resolver: &family_resolver,
        member_surface: &member_surface,
        is_same_or_subclass: &is_same_or_subclass,
    };
    super::flow::analyze_surface(source, &context, generation, counters)
}

/// Analyzes one callable body against the current immutable solver context.
pub(crate) fn analyze_callable_source(
    source: &FileSourceSnapshot,
    callable: &CallableId,
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    summaries: &BTreeMap<CallableId, CallableSummary>,
    parameters: &ParameterFacts,
    generation: SemanticGeneration,
    include_top_level: bool,
    counters: &PerfCounters,
) -> SurfaceFlowAnalysis {
    let known_class = |name: &str| super::resolve_named_class(classes, graph, &source.module, name);
    let contains_class = |class: &ClassId| classes.contains_key(class);
    let callable_return = |id: &CallableId| super::return_for_callable(classes, summaries, id);
    let callable_effects = |id: &CallableId| summaries.get(id).map(|summary| summary.effects.clone());
    let parameter_fact = |id: &CallableId, name: &str| parameters.get(id, name).cloned();
    let field_value = |_: &ClassId, _: &str, _: DispatchSide| None;
    let resolver = DispatchResolver::new(classes);
    let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
    let family_resolver = |receiver: &DispatchReceiver, pattern: &phalcom_common::selector::SelectorPattern| resolver.capture_method_family(receiver, pattern);
    let is_same_or_subclass = |child: &ClassId, ancestor: &ClassId| super::is_same_or_subclass(classes, child, ancestor);
    let member_surface = |id: &CallableId| classes.get(&id.owner).and_then(|class| class.member_by_id(id).cloned());
    let context = SolverContext {
        known_class: &known_class,
        contains_class: &contains_class,
        callable_return: &callable_return,
        callable_effects: &callable_effects,
        parameter_fact: &parameter_fact,
        field_value: &field_value,
        resolve_member: &resolve_member,
        family_resolver: &family_resolver,
        member_surface: &member_surface,
        is_same_or_subclass: &is_same_or_subclass,
    };
    super::flow::analyze_callable(source, &context, generation, callable, include_top_level, counters)
}

/// Solves source callable summaries and parameter facts without mutating the
/// published semantic database.
#[expect(dead_code, reason = "retained as a full-workspace solver reference for regression comparisons")]
pub(crate) fn solve_workspace_callables(
    inputs: &[Arc<FileSourceSnapshot>],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
    counters: &PerfCounters,
) -> SolverResult {
    let callable_count = inputs
        .iter()
        .map(|source| source.surface.classes.values().map(|class| class.all_members().count()).sum::<usize>())
        .sum::<usize>();
    let slot_count = inputs
        .iter()
        .map(|source| {
            source
                .surface
                .classes
                .values()
                .flat_map(|class| class.all_members())
                .map(|member| member.params.len())
                .sum::<usize>()
        })
        .sum::<usize>();
    let max_rounds = solver_budget(callable_count, slot_count);
    let mut summaries = BTreeMap::new();
    let mut parameter_facts = ParameterFacts::default();
    let mut worklist = CallableWorklist::default();
    for source in inputs {
        for class in source.surface.classes.values() {
            for member in class.all_members() {
                worklist.push(member.callable.clone());
            }
        }
    }

    let mut rounds = 0;
    while rounds < max_rounds && (!worklist.is_empty() || (callable_count == 0 && rounds == 0)) {
        record_solver_round(counters);
        while worklist.pop().is_some() {}
        rounds += 1;
        let previous_summaries = summaries.clone();
        let previous_parameters = parameter_facts.clone();
        let mut next_parameters = ParameterFacts::default();

        let mut next_summaries = BTreeMap::new();
        for source in inputs {
            let analysis = analyze_source(source, classes, graph, &previous_summaries, &previous_parameters, generation, counters);
            next_parameters.merge_from(&analysis.parameter_facts);
            for (summary, evidence) in analysis.summaries {
                if evidence {
                    next_summaries.insert(summary.callable.clone(), summary);
                }
            }
        }
        apply_parameter_facts_to_summaries(&mut next_summaries, classes, &next_parameters);

        let summaries_changed = next_summaries != previous_summaries;
        let parameters_changed = next_parameters != previous_parameters;
        summaries = next_summaries;
        parameter_facts = next_parameters;
        if !summaries_changed && !parameters_changed {
            complete_missing_summaries(inputs, classes, generation, &parameter_facts, &mut summaries);
            return SolverResult { summaries, parameter_facts };
        }
        if summaries_changed || parameters_changed {
            for source in inputs {
                for class in source.surface.classes.values() {
                    for member in class.all_members() {
                        worklist.push(member.callable.clone());
                    }
                }
            }
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
#[allow(dead_code)]
pub(crate) fn solve_affected_callables(
    inputs: &[Arc<FileSourceSnapshot>],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
    seed_summaries: BTreeMap<CallableId, CallableSummary>,
    base_parameters: ParameterFacts,
) -> FlowSolveResult {
    let counters = PerfCounters::new();
    let mut base_contributions = ParameterContributions::default();
    base_contributions.replace_source(
        ContributionSource::TopLevel(ModuleId::new("memory://base-parameters")),
        base_parameters.iter().map(|((callable, name), value)| {
            (
                super::facts::ParameterSlot {
                    callable: callable.clone(),
                    name: name.clone(),
                },
                value.clone(),
            )
        }),
    );
    solve_affected_callables_with_cancel(
        inputs,
        classes,
        graph,
        generation,
        seed_summaries,
        base_contributions,
        None,
        &|| false,
        &counters,
    )
    .expect("uncancelled callable solve must complete")
}

/// Incremental callable solve with cooperative cancellation between worklist
/// items. No partial result is returned when the caller's epoch is stale.
pub(crate) fn solve_affected_callables_with_cancel(
    inputs: &[Arc<FileSourceSnapshot>],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
    seed_summaries: BTreeMap<CallableId, CallableSummary>,
    base_contributions: ParameterContributions,
    dirty_callables: Option<&BTreeSet<CallableId>>,
    cancelled: &dyn Fn() -> bool,
    counters: &PerfCounters,
) -> Option<FlowSolveResult> {
    if inputs.is_empty() {
        return Some(FlowSolveResult {
            source_analyses: BTreeMap::new(),
            callables_visited: BTreeSet::new(),
            callables_changed: BTreeSet::new(),
        });
    }

    let mut callable_sources = BTreeMap::new();
    for source in inputs {
        for class in source.surface.classes.values() {
            for member in class.all_members() {
                callable_sources.insert(member.callable.clone(), source.clone());
            }
        }
    }
    let callable_count = callable_sources.len();
    let slot_count = inputs
        .iter()
        .map(|source| {
            source
                .surface
                .classes
                .values()
                .flat_map(|class| class.all_members())
                .map(|member| member.params.len())
                .sum::<usize>()
        })
        .sum::<usize>();
    let max_steps = solver_budget(callable_count, slot_count);
    let mut summaries = seed_summaries;
    let mut contributions = base_contributions;
    let mut parameter_facts = joined_parameter_facts(&contributions);
    let mut top_level_sources = BTreeSet::new();
    let mut dependents = BTreeMap::<CallableId, BTreeSet<CallableId>>::new();
    for summary in summaries.values() {
        for dependency in &summary.dependencies {
            dependents.entry(dependency.clone()).or_default().insert(summary.callable.clone());
        }
    }
    let mut worklist = CallableWorklist::default();
    match dirty_callables {
        Some(dirty) => {
            for callable in dirty {
                if callable_sources.contains_key(callable) {
                    worklist.push(callable.clone());
                    counters.dirty_callables_seeded.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
        None => {
            for callable in callable_sources.keys() {
                worklist.push(callable.clone());
            }
        }
    }

    let mut steps = 0;
    let mut exceeded_budget = false;
    let mut callables_visited = BTreeSet::new();
    let mut callables_changed = BTreeSet::new();
    'rounds: while !worklist.is_empty() {
        if cancelled() {
            return None;
        }
        record_solver_round(counters);
        while let Some(callable) = worklist.pop() {
            if cancelled() {
                return None;
            }
            steps += 1;
            if steps > max_steps {
                exceeded_budget = true;
                break 'rounds;
            }
            record_solver_step();
            let Some(source) = callable_sources.get(&callable) else { continue };
            callables_visited.insert(callable.clone());
            counters.solver_callables_visited.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let include_top_level = top_level_sources.insert(source.module.clone());
            let previous_summary = summaries.get(&callable).cloned();
            let analysis = analyze_callable_source(
                source,
                &callable,
                classes,
                graph,
                &summaries,
                &parameter_facts,
                generation,
                include_top_level,
                counters,
            );
            if cancelled() {
                return None;
            }
            let candidate = analysis
                .summaries
                .into_iter()
                .find(|(summary, _)| summary.callable == callable)
                .and_then(|(summary, evidence)| evidence.then_some(summary));
            if let Some(summary) = candidate {
                summaries.insert(callable.clone(), summary.clone());
                let old_dependencies = previous_summary
                    .as_ref()
                    .map(|summary| summary.dependencies.iter().cloned().collect::<BTreeSet<_>>())
                    .unwrap_or_default();
                let new_dependencies = summary.dependencies.iter().cloned().collect::<BTreeSet<_>>();
                for dependency in old_dependencies.difference(&new_dependencies) {
                    if let Some(edges) = dependents.get_mut(dependency) {
                        edges.remove(&callable);
                    }
                }
                for dependency in new_dependencies.difference(&old_dependencies) {
                    dependents.entry(dependency.clone()).or_default().insert(callable.clone());
                }
            } else {
                summaries.remove(&callable);
            }

            let emitted_contributions = analysis.parameter_contributions;
            let mut parameter_deltas = Vec::new();
            for (source, facts) in emitted_contributions {
                let before_slots = facts.iter().count();
                let deltas = contributions.replace_source(
                    source,
                    facts.iter().map(|((target, name), value)| {
                        (
                            super::facts::ParameterSlot {
                                callable: target.clone(),
                                name: name.clone(),
                            },
                            value.clone(),
                        )
                    }),
                );
                record_parameter_replacement(counters, before_slots, &deltas);
                parameter_deltas.extend(deltas);
            }
            parameter_facts = joined_parameter_facts(&contributions);
            apply_parameter_facts_to_summaries(&mut summaries, classes, &parameter_facts);

            let summary_changed = callable_summary_changed(previous_summary.as_ref(), summaries.get(&callable));
            if summary_changed {
                callables_changed.insert(callable.clone());
                counters.solver_callables_changed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            let changed_parameter_callables = parameter_deltas
                .iter()
                .filter_map(|delta| (delta.before != delta.after).then_some(delta.slot.callable.clone()))
                .collect::<BTreeSet<_>>();
            if summary_changed || !changed_parameter_callables.is_empty() {
                if let Some(edges) = dependents.get(&callable) {
                    for dependent in edges {
                        if callable_sources.contains_key(dependent) {
                            worklist.push(dependent.clone());
                        }
                    }
                }
                for changed_callable in changed_parameter_callables {
                    if callable_sources.contains_key(&changed_callable) {
                        worklist.push(changed_callable);
                    }
                }
            }
        }
    }

    if exceeded_budget {
        parameter_facts.widen_all();
        for summary in summaries.values_mut() {
            if callable_sources.contains_key(&summary.callable) {
                summary.returns = InferredValue::flow(ValueShape::Unknown, Default::default());
                for value in &mut summary.params {
                    *value = InferredValue::flow(ValueShape::Unknown, Default::default());
                }
            }
        }
    }
    complete_missing_summaries(inputs, classes, generation, &parameter_facts, &mut summaries);
    if cancelled() {
        return None;
    }
    // Source flow can discover more precise argument facts once callable
    // summaries resolve (for example, a factory call becomes a concrete
    // instance). Allow one bounded feedback cycle for those facts and one
    // final source pass so binding facts and callable facts share one view.
    let mut final_summaries = summaries;
    let mut source_analyses = BTreeMap::new();
    for _ in 0..3 {
        source_analyses.clear();
        for source in inputs {
            if cancelled() {
                return None;
            }
            let analysis = analyze_source(source, classes, graph, &final_summaries, &parameter_facts, generation, counters);
            source_analyses.insert(source.module.clone(), analysis);
        }
        let mut next_summaries = final_summaries.clone();
        for analysis in source_analyses.values() {
            for (summary, evidence) in &analysis.summaries {
                if *evidence {
                    next_summaries.insert(summary.callable.clone(), summary.clone());
                }
            }
            for (source, facts) in &analysis.parameter_contributions {
                contributions.replace_source(
                    source.clone(),
                    facts.iter().map(|((callable, name), value)| {
                        (
                            super::facts::ParameterSlot {
                                callable: callable.clone(),
                                name: name.clone(),
                            },
                            value.clone(),
                        )
                    }),
                );
            }
        }
        parameter_facts = joined_parameter_facts(&contributions);
        apply_parameter_facts_to_summaries(&mut next_summaries, classes, &parameter_facts);
        if summaries_semantically_equal(&next_summaries, &final_summaries) {
            break;
        }
        final_summaries = next_summaries;
    }
    Some(FlowSolveResult {
        source_analyses,
        callables_visited,
        callables_changed,
    })
}

/// Compares summaries as semantic products. Publication generation is provenance,
/// not an invalidation input.
pub(crate) fn callable_summary_changed(previous: Option<&CallableSummary>, current: Option<&CallableSummary>) -> bool {
    match (previous, current) {
        (Some(previous), Some(current)) => {
            previous.callable != current.callable
                || !values_semantically_equal(&previous.params, &current.params)
                || !value_semantically_equal(&previous.returns, &current.returns)
                || previous.dependencies != current.dependencies
                || previous.effects != current.effects
        }
        (None, None) => false,
        (Some(_), None) | (None, Some(_)) => true,
    }
}

fn summaries_semantically_equal(left: &BTreeMap<CallableId, CallableSummary>, right: &BTreeMap<CallableId, CallableSummary>) -> bool {
    left.len() == right.len()
        && left.iter().all(|(id, summary)| {
            right.get(id).is_some_and(|other| {
                summary.callable == other.callable
                    && values_semantically_equal(&summary.params, &other.params)
                    && value_semantically_equal(&summary.returns, &other.returns)
                    && summary.dependencies == other.dependencies
                    && summary.effects == other.effects
            })
        })
}

fn values_semantically_equal(left: &[InferredValue], right: &[InferredValue]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(left, right)| value_semantically_equal(left, right))
}

fn value_semantically_equal(left: &InferredValue, right: &InferredValue) -> bool {
    left.shape == right.shape && left.known_boolean == right.known_boolean && left.confidence == right.confidence
}

fn apply_parameter_facts_to_summaries(
    summaries: &mut BTreeMap<CallableId, CallableSummary>,
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    parameters: &ParameterFacts,
) {
    for summary in summaries.values_mut() {
        let Some(class) = classes.get(&summary.callable.owner) else { continue };
        let Some(member) = class.member_by_id(&summary.callable) else {
            continue;
        };
        summary.params = member
            .params
            .iter()
            .map(|param| {
                parameters
                    .get(&summary.callable, &param.name)
                    .cloned()
                    .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, param.source_range))
            })
            .collect();
    }
}

pub(crate) fn complete_missing_summaries(
    inputs: &[Arc<FileSourceSnapshot>],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    generation: SemanticGeneration,
    parameter_facts: &ParameterFacts,
    summaries: &mut BTreeMap<CallableId, CallableSummary>,
) {
    for source in inputs {
        for class in source.surface.classes.values() {
            for member in class.all_members() {
                if summaries.contains_key(&member.callable) {
                    continue;
                }
                let params = member
                    .params
                    .iter()
                    .map(|param| {
                        parameter_facts
                            .get(&member.callable, &param.name)
                            .cloned()
                            .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, param.source_range))
                    })
                    .collect();
                let returns = super::return_for_callable(classes, summaries, &member.callable)
                    .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, member.source_range));
                summaries.insert(
                    member.callable.clone(),
                    CallableSummary {
                        callable: member.callable.clone(),
                        params,
                        returns,
                        dependencies: Vec::new(),
                        effects: Default::default(),
                        revision: generation,
                    },
                );
            }
        }
    }
}

#[cfg(test)]
fn core_class(name: &str) -> ClassId {
    ClassId::new(ModuleId::new(CORE_MODULE_URI), name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;

    const ONE_PASS_FIXTURE: &str = r#"
let local = 1

class Product {
  const _seed = 1

  @constructor
  new(_ value) { _seed = value }

  result() { _seed }
}

class Service {
  @class
  consume(_ input) { input }

  call() { Service.consume(1) }
}
"#;

    fn one_pass_source() -> FileSourceSnapshot {
        let module = ModuleId::new("file:///one-pass.ph");
        let program = Arc::new(parse(ONE_PASS_FIXTURE, 0).program);
        let surface = super::super::surface::build_module_surface(module.clone(), &program);
        let scopes = super::super::scope::build_scope_graph(module.clone(), &program);
        let callables = surface.callable_index();
        FileSourceSnapshot {
            module,
            text: Arc::from(ONE_PASS_FIXTURE),
            program,
            surface,
            scopes,
            callables,
        }
    }

    #[test]
    fn one_surface_flow_analysis_contains_all_products() {
        let source = one_pass_source();
        let classes = source.surface.classes.clone();
        let counters = PerfCounters::new();
        let before = crate::semantic::flow::test_flow_passes();
        let analysis = analyze_source(
            &source,
            &classes,
            &ModuleGraph::default(),
            &BTreeMap::new(),
            &ParameterFacts::default(),
            SemanticGeneration(1),
            &counters,
        );
        let after = crate::semantic::flow::test_flow_passes();

        assert_eq!(after.0 - before.0, 1, "one source analysis must run one flow pass");
        let local_binding = source
            .scopes
            .bindings
            .values()
            .find(|binding| binding.name == "local")
            .expect("fixture local binding")
            .id;
        assert!(!analysis.local_facts.facts_for(local_binding).next().is_none());
        assert!(analysis.field_facts.iter().next().is_some(), "field initializer/write product missing");
        assert!(analysis.parameter_facts.iter().next().is_some(), "parameter call product missing");
        assert!(analysis.summaries.iter().any(|(_, evidence)| *evidence), "callable summary product missing");
    }

    #[test]
    fn literals_and_reassignment_are_queryable() {
        let program = parse("let value = 1\nvalue = \"ok\"\n", 0).program;
        let module = ModuleId::new("file:///main.ph");
        let surface = super::super::surface::build_module_surface(module.clone(), &program);
        let source = FileSourceSnapshot {
            module: module.clone(),
            text: Arc::from("let value = 1\nvalue = \"ok\"\n"),
            program: Arc::new(program),
            scopes: super::super::scope::build_scope_graph(module.clone(), &parse("let value = 1\nvalue = \"ok\"\n", 0).program),
            callables: surface.callable_index(),
            surface,
        };
        let counters = PerfCounters::new();
        let facts = analyze_source(
            &source,
            &BTreeMap::new(),
            &ModuleGraph::default(),
            &BTreeMap::new(),
            &ParameterFacts::default(),
            SemanticGeneration(0),
            &counters,
        )
        .local_facts;
        let scopes = &source.scopes;
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

    #[test]
    fn exact_list_shapes_drive_nested_destructuring_and_rest() {
        let text = "let [first, second, *tail] = [1, 2, 3]\nlet [(nx, ny), last] = [(1, 2), 3]\n";
        let module = ModuleId::new("file:///destructure.ph");
        let parsed = parse(text, 0).program;
        let surface = super::super::surface::build_module_surface(module.clone(), &parsed);
        let source = FileSourceSnapshot {
            module: module.clone(),
            text: Arc::from(text),
            program: Arc::new(parsed),
            scopes: super::super::scope::build_scope_graph(module.clone(), &parse(text, 0).program),
            callables: surface.callable_index(),
            surface,
        };
        let facts = analyze_source(
            &source,
            &BTreeMap::new(),
            &ModuleGraph::default(),
            &BTreeMap::new(),
            &ParameterFacts::default(),
            SemanticGeneration(0),
            &PerfCounters::new(),
        )
        .local_facts;
        for name in ["first", "second", "nx", "ny", "last"] {
            let binding = source.scopes.bindings.values().find(|binding| binding.name == name).expect("binding").id;
            assert_eq!(
                facts.value_before(binding, text.len()).unwrap().shape,
                ValueShape::Instance(core_class("Int")),
                "{name}"
            );
        }
        let tail_binding = source.scopes.bindings.values().find(|binding| binding.name == "tail").expect("tail binding").id;
        assert_eq!(
            facts.value_before(tail_binding, text.len()).unwrap().shape,
            ValueShape::List(Box::new(ValueShape::Instance(core_class("Int"))))
        );
    }

    #[test]
    fn control_expression_values_transfer_through_let_and_return() {
        let text = "let cond = true\nlet x = if (cond) { true } else { false }\nx = if (cond) { false } else { true }\nclass Factory { choose(_ cond) { return if (cond) { 1 } else { 2 } } }\n";
        let module = ModuleId::new("file:///control-expressions.ph");
        let parsed = parse(text, 0).program;
        let surface = super::super::surface::build_module_surface(module.clone(), &parsed);
        let source = FileSourceSnapshot {
            module: module.clone(),
            text: Arc::from(text),
            program: Arc::new(parsed),
            scopes: super::super::scope::build_scope_graph(module.clone(), &parse(text, 0).program),
            callables: surface.callable_index(),
            surface,
        };
        let analysis = analyze_source(
            &source,
            &source.surface.classes,
            &ModuleGraph::default(),
            &BTreeMap::new(),
            &ParameterFacts::default(),
            SemanticGeneration(0),
            &PerfCounters::new(),
        );
        let x_binding = source.scopes.bindings.values().find(|binding| binding.name == "x").expect("x binding").id;
        assert_eq!(
            analysis.local_facts.value_before(x_binding, text.len()).unwrap().shape,
            ValueShape::Instance(core_class("Bool"))
        );
        let factory = ClassId::new(module, "Factory");
        let summary = &analysis
            .summaries
            .iter()
            .find(|(summary, _)| summary.callable.owner == factory && summary.callable.selector == "choose(_)")
            .expect("choose summary")
            .0;
        assert_eq!(summary.returns.shape, ValueShape::Instance(core_class("Int")));
    }
}
