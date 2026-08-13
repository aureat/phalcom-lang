//! Deterministic syntax and local-flow inference.

#![allow(clippy::only_used_in_recursion, clippy::too_many_arguments)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::CallableSummary;
use super::callable::{CallableWorklist, SolverResult};
use super::dispatch::{DispatchReceiver, DispatchResolver};
use super::facts::{InferredValue, MAX_SHAPE_UNION, ParameterFacts, ValueShape};
#[cfg(test)]
use super::ids::CORE_MODULE_URI;
use super::ids::{CallableId, ClassId, DispatchSide, ModuleId};
use super::module_graph::ModuleGraph;
use super::query::SemanticGeneration;

use super::flow::{SolverContext, SurfaceFlowAnalysis};
use super::snapshot::FileSourceSnapshot;

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

fn record_solver_round() {
    #[cfg(test)]
    TEST_SOLVER_ROUNDS.with(|count| count.set(count.get() + 1));
    crate::perf::COUNTERS.solver_rounds.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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

/// Coherent products from one bounded callable solve and its final source flow
/// pass. Engine publication consumes these analyses directly.
#[derive(Clone, Debug, Default)]
pub(crate) struct FlowSolveResult {
    /// One unified product per affected source module.
    pub source_analyses: BTreeMap<ModuleId, SurfaceFlowAnalysis>,
}

/// Builds the immutable solver context for one source-backed flow pass.
pub(crate) fn analyze_source(
    source: &FileSourceSnapshot,
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    summaries: &BTreeMap<CallableId, CallableSummary>,
    parameters: &ParameterFacts,
    generation: SemanticGeneration,
) -> SurfaceFlowAnalysis {
    let known_class = |name: &str| super::resolve_named_class(classes, graph, &source.module, name);
    let contains_class = |class: &ClassId| classes.contains_key(class);
    let callable_return = |id: &CallableId| super::return_for_callable(classes, summaries, id);
    let callable_effects = |id: &CallableId| summaries.get(id).map(|summary| summary.effects.clone());
    let parameter_fact = |id: &CallableId, name: &str| parameters.get(id, name).cloned();
    let field_value = |_: &ClassId, _: &str, _: DispatchSide| None;
    let resolver = DispatchResolver::new(classes);
    let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
    let member_surface = |id: &CallableId| {
        classes
            .get(&id.owner)
            .and_then(|class| class.members_by_side.get(&(id.selector.clone(), id.side)).cloned())
    };
    let context = SolverContext {
        known_class: &known_class,
        contains_class: &contains_class,
        callable_return: &callable_return,
        callable_effects: &callable_effects,
        parameter_fact: &parameter_fact,
        field_value: &field_value,
        resolve_member: &resolve_member,
        member_surface: &member_surface,
    };
    super::flow::analyze_surface(source, &context, generation)
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
) -> SurfaceFlowAnalysis {
    let known_class = |name: &str| super::resolve_named_class(classes, graph, &source.module, name);
    let contains_class = |class: &ClassId| classes.contains_key(class);
    let callable_return = |id: &CallableId| super::return_for_callable(classes, summaries, id);
    let callable_effects = |id: &CallableId| summaries.get(id).map(|summary| summary.effects.clone());
    let parameter_fact = |id: &CallableId, name: &str| parameters.get(id, name).cloned();
    let field_value = |_: &ClassId, _: &str, _: DispatchSide| None;
    let resolver = DispatchResolver::new(classes);
    let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
    let member_surface = |id: &CallableId| {
        classes
            .get(&id.owner)
            .and_then(|class| class.members_by_side.get(&(id.selector.clone(), id.side)).cloned())
    };
    let context = SolverContext {
        known_class: &known_class,
        contains_class: &contains_class,
        callable_return: &callable_return,
        callable_effects: &callable_effects,
        parameter_fact: &parameter_fact,
        field_value: &field_value,
        resolve_member: &resolve_member,
        member_surface: &member_surface,
    };
    super::flow::analyze_callable(source, &context, generation, callable)
}

/// Solves source callable summaries and parameter facts without mutating the
/// published semantic database.
#[expect(dead_code, reason = "retained as a full-workspace solver reference for regression comparisons")]
pub(crate) fn solve_workspace_callables(
    inputs: &[Arc<FileSourceSnapshot>],
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
) -> SolverResult {
    let callable_count = inputs
        .iter()
        .map(|source| source.surface.classes.values().map(|class| class.members_by_side.len()).sum::<usize>())
        .sum::<usize>();
    let slot_count = inputs
        .iter()
        .map(|source| {
            source
                .surface
                .classes
                .values()
                .flat_map(|class| class.members_by_side.values())
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
            for member in class.members_by_side.values() {
                worklist.push(member.callable.clone());
            }
        }
    }

    let mut rounds = 0;
    while rounds < max_rounds && (!worklist.is_empty() || (callable_count == 0 && rounds == 0)) {
        record_solver_round();
        while worklist.pop().is_some() {}
        rounds += 1;
        let previous_summaries = summaries.clone();
        let previous_parameters = parameter_facts.clone();
        let mut next_parameters = ParameterFacts::default();

        let mut next_summaries = BTreeMap::new();
        for source in inputs {
            let analysis = analyze_source(source, classes, graph, &previous_summaries, &previous_parameters, generation);
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
                    for member in class.members_by_side.values() {
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
    solve_affected_callables_with_cancel(inputs, classes, graph, generation, seed_summaries, base_parameters, &|| false)
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
    base_parameters: ParameterFacts,
    cancelled: &dyn Fn() -> bool,
) -> Option<FlowSolveResult> {
    if inputs.is_empty() {
        return Some(FlowSolveResult {
            source_analyses: BTreeMap::new(),
        });
    }

    let mut callable_sources = BTreeMap::new();
    for source in inputs {
        for class in source.surface.classes.values() {
            for member in class.members_by_side.values() {
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
                .flat_map(|class| class.members_by_side.values())
                .map(|member| member.params.len())
                .sum::<usize>()
        })
        .sum::<usize>();
    let max_steps = solver_budget(callable_count, slot_count);
    let mut summaries = seed_summaries;
    let mut parameter_facts = base_parameters.clone();
    let mut source_facts = BTreeMap::<CallableId, ParameterFacts>::new();
    let mut dependents = BTreeMap::<CallableId, BTreeSet<CallableId>>::new();
    for summary in summaries.values() {
        for dependency in &summary.dependencies {
            dependents.entry(dependency.clone()).or_default().insert(summary.callable.clone());
        }
    }
    let mut worklist = CallableWorklist::default();
    for callable in callable_sources.keys() {
        worklist.push(callable.clone());
    }

    let mut steps = 0;
    let mut exceeded_budget = false;
    'rounds: while !worklist.is_empty() {
        if cancelled() {
            return None;
        }
        record_solver_round();
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
        let previous_summary = summaries.get(&callable).cloned();
        let analysis = analyze_callable_source(source, &callable, classes, graph, &summaries, &parameter_facts, generation);
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

        source_facts.insert(callable.clone(), analysis.parameter_facts);
        let previous_parameters = parameter_facts.clone();
        parameter_facts = base_parameters.clone();
        for facts in source_facts.values() {
            parameter_facts.merge_from(facts);
        }
        apply_parameter_facts_to_summaries(&mut summaries, classes, &parameter_facts);

        if previous_summary != summaries.get(&callable).cloned() || previous_parameters != parameter_facts {
            if let Some(edges) = dependents.get(&callable) {
                for dependent in edges {
                    if callable_sources.contains_key(dependent) {
                        worklist.push(dependent.clone());
                    }
                }
            }
            for (slot, value) in parameter_facts.iter() {
                if previous_parameters.get(&slot.0, &slot.1) != Some(value) && callable_sources.contains_key(&slot.0) {
                    worklist.push(slot.0.clone());
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
    let mut source_analyses = BTreeMap::new();
    for source in inputs {
        if cancelled() {
            return None;
        }
        let analysis = analyze_source(source, classes, graph, &summaries, &parameter_facts, generation);
        source_analyses.insert(source.module.clone(), analysis);
    }
    Some(FlowSolveResult { source_analyses })
}

fn apply_parameter_facts_to_summaries(
    summaries: &mut BTreeMap<CallableId, CallableSummary>,
    classes: &BTreeMap<ClassId, super::surface::ClassSurface>,
    parameters: &ParameterFacts,
) {
    for summary in summaries.values_mut() {
        let Some(class) = classes.get(&summary.callable.owner) else { continue };
        let Some(member) = class.members_by_side.get(&(summary.callable.selector.clone(), summary.callable.side)) else {
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
            for member in class.members_by_side.values() {
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
        FileSourceSnapshot {
            module,
            program,
            surface,
            scopes,
        }
    }

    #[test]
    fn one_surface_flow_analysis_contains_all_products() {
        let source = one_pass_source();
        let classes = source.surface.classes.clone();
        let before = crate::semantic::flow::test_flow_passes();
        let analysis = analyze_source(
            &source,
            &classes,
            &ModuleGraph::default(),
            &BTreeMap::new(),
            &ParameterFacts::default(),
            SemanticGeneration(1),
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
            program: Arc::new(program),
            scopes: super::super::scope::build_scope_graph(module.clone(), &parse("let value = 1\nvalue = \"ok\"\n", 0).program),
            surface,
        };
        let facts = analyze_source(
            &source,
            &BTreeMap::new(),
            &ModuleGraph::default(),
            &BTreeMap::new(),
            &ParameterFacts::default(),
            SemanticGeneration(0),
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
}
