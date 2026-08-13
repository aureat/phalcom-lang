//! Mutable worker-only semantic analysis engine.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use phalcom_ast::ast::Program;
use tower_lsp::lsp_types::Url;

#[cfg(test)]
use super::RebuildTrace;
use super::callable::CallableSummary;
use super::core_source;
use super::facts::{ContributionSource, FileRevision, InferredValue, ParameterContributions, ParameterFacts, ParameterSlot};
use super::ids::{CORE_MODULE_URI, CallableId, ClassId, FieldId, ModuleId};
use super::module_graph::ModuleGraph;
use super::query::SemanticGeneration;
use super::snapshot::{FileSourceSnapshot, SemanticSnapshot};
use super::surface::{ClassSurface, build_module_surface};
use super::{DependencySet, FileSemanticSnapshot, SourceChangeKind, classify_source_change, infer};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RebuildTraceData {
    pub modules_recomputed: BTreeSet<ModuleId>,
    pub callables_recomputed: BTreeSet<CallableId>,
}

#[derive(Clone, Default)]
pub(crate) struct SemanticState {
    pub generation: SemanticGeneration,
    pub files: BTreeMap<ModuleId, FileSemanticSnapshot>,
    pub classes: BTreeMap<ClassId, ClassSurface>,
    pub summaries: BTreeMap<CallableId, CallableSummary>,
    pub field_facts: BTreeMap<FieldId, InferredValue>,
    pub parameter_facts: BTreeMap<(CallableId, String), InferredValue>,
    pub parameter_contributions: BTreeMap<ModuleId, ParameterFacts>,
    pub parameter_contribution_slots: ParameterContributions,
    pub callable_dependencies: BTreeMap<CallableId, BTreeSet<CallableId>>,
    pub callable_dependents: BTreeMap<CallableId, BTreeSet<CallableId>>,
    pub graph: ModuleGraph,
    #[cfg(test)]
    pub last_trace: Option<RebuildTrace>,
}

/// Mutable single-threaded semantic analysis worker engine.
#[derive(Clone, Default)]
pub struct SemanticEngine {
    state: SemanticState,
}

impl SemanticEngine {
    /// Creates a new empty semantic engine with zero state (generation 0).
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an engine initialized with zero state.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns the current semantic generation.
    pub fn generation(&self) -> SemanticGeneration {
        self.state.generation
    }

    /// Produces an immutable published snapshot of current engine state.
    pub fn snapshot(&self) -> SemanticSnapshot {
        SemanticSnapshot {
            generation: self.state.generation,
            files: self.state.files.iter().map(|(k, v)| (k.clone(), Arc::new(v.clone()))).collect(),
            classes: self.state.classes.iter().map(|(k, v)| (k.clone(), Arc::new(v.clone()))).collect(),
            summaries: self.state.summaries.iter().map(|(k, v)| (k.clone(), Arc::new(v.clone()))).collect(),
            field_facts: self.state.field_facts.clone(),
            parameter_facts: self.state.parameter_facts.clone(),
            graph: self.state.graph.clone(),
        }
    }

    #[cfg(test)]
    /// Returns the trace of the last rebuild.
    pub fn last_rebuild_trace(&self) -> Option<RebuildTrace> {
        self.state.last_trace.clone()
    }

    /// Updates one single file's contribution.
    pub fn update_file(&mut self, uri: &Url, revision: FileRevision, program: &Program) -> SemanticGeneration {
        self.update_files_batch(vec![(uri.clone(), revision, program.clone())])
    }

    /// Updates several file contributions in a single semantic batch transaction.
    pub fn update_files_batch(&mut self, files: Vec<(Url, FileRevision, Program)>) -> SemanticGeneration {
        self.update_files_batch_with_cancel(files, &|| false).expect("uncancelled update must complete")
    }

    fn update_files_batch_inner(&mut self, files: Vec<(Url, FileRevision, Program)>, cancelled: &dyn Fn() -> bool) -> Option<SemanticGeneration> {
        if files.is_empty() {
            return Some(self.state.generation);
        }
        let next_generation = SemanticGeneration(self.state.generation.0 + 1);

        let mut affected = BTreeSet::new();
        for (uri, _, _) in &files {
            let module = ModuleId::from_uri(uri);
            affected.insert(module.clone());
            affected.extend(self.state.graph.dependent_closure(&module));
            let old_callables = self.state.summaries.keys().filter(|id| id.owner.module == module).cloned().collect::<Vec<_>>();
            for callable in old_callables {
                if let Some(dependents) = self.state.callable_dependents.get(&callable) {
                    affected.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
                }
            }
        }

        let updated_modules = files.iter().map(|(uri, _, _)| ModuleId::from_uri(uri)).collect::<Vec<_>>();
        let mut change_kinds = Vec::new();
        for (uri, revision, program) in files {
            if cancelled() {
                return None;
            }
            let module = ModuleId::from_uri(&uri);
            let old_source = self.state.files.get(&module).map(|file| file.source.clone());
            let surface = if module.as_str() == CORE_MODULE_URI {
                core_source::build_core_surface(&program)
            } else {
                build_module_surface(module.clone(), &program)
            };
            let program = Arc::new(program);
            let scopes = super::scope::build_scope_graph(module.clone(), &program);
            let occurrences = super::occurrence::build_occurrence_index(module.clone(), &program, &surface, &scopes);

            let source = Arc::new(FileSourceSnapshot {
                module: module.clone(),
                program: program.clone(),
                surface,
                scopes,
            });
            change_kinds.push(classify_source_change(&module, old_source.as_deref(), Some(&source)));
            self.state.classes.retain(|id, _| id.module != module);
            self.state
                .classes
                .extend(source.surface.classes.iter().map(|(id, class)| (id.clone(), class.clone())));
            let snapshot = FileSemanticSnapshot {
                revision,
                module: module.clone(),
                source,
                occurrences,
                local_facts: super::facts::LocalFacts::default(),
                field_facts: super::facts::FieldFacts::default(),
                parameter_facts: super::facts::ParameterFacts::default(),
                dependencies: super::DependencySet::default(),
            };
            self.state.files.insert(module.clone(), snapshot);
        }

        let available = self.state.files.keys().cloned().collect::<BTreeSet<_>>();
        for module in &updated_modules {
            if let Some(file) = self.state.files.get(module) {
                self.state.graph.update(file.module.clone(), &file.source.program, &available);
            }
        }
        if change_kinds.iter().any(|kind| *kind != SourceChangeKind::BodyOnly) {
            self.state.graph.refresh_resolutions(&available);
        }

        for module in updated_modules {
            affected.extend(self.state.graph.dependent_closure(&module));
        }

        self.state.generation = next_generation;
        let trace = rebuild_affected_state(&mut self.state, next_generation, affected, cancelled)?;
        #[cfg(test)]
        {
            self.state.last_trace = Some(trace.into());
        }
        #[cfg(not(test))]
        drop(trace);

        Some(self.state.generation)
    }

    /// Applies one batch on a temporary engine state and abandons it when the
    /// caller's epoch becomes stale. This keeps publication atomic while the
    /// callable solver cooperatively stops between work items.
    pub fn update_files_batch_with_cancel(&mut self, files: Vec<(Url, FileRevision, Program)>, cancelled: &dyn Fn() -> bool) -> Option<SemanticGeneration> {
        if cancelled() {
            return None;
        }
        let mut candidate = self.clone();
        let generation = candidate.update_files_batch_inner(files, cancelled)?;
        *self = candidate;
        Some(generation)
    }

    /// Replaces the active core library module.
    pub fn update_core(&mut self, revision: FileRevision, program: &Program) -> SemanticGeneration {
        let uri = Url::parse(CORE_MODULE_URI).expect("core module URI must parse");
        self.update_file(&uri, revision, program)
    }

    /// Removes one source file from the active universe.
    pub fn remove_file(&mut self, uri: &Url) -> SemanticGeneration {
        let module = ModuleId::from_uri(uri);
        if self.state.files.remove(&module).is_none() {
            return self.state.generation;
        }

        let next_generation = SemanticGeneration(self.state.generation.0 + 1);
        let mut affected: BTreeSet<ModuleId> = self.state.graph.dependent_closure(&module).into_iter().collect();
        self.state.graph.remove(&module);

        let available = self.state.files.keys().cloned().collect::<BTreeSet<_>>();
        self.state.graph.refresh_resolutions(&available);

        let old_callables = self.state.summaries.keys().filter(|id| id.owner.module == module).cloned().collect::<Vec<_>>();
        for callable in old_callables {
            if let Some(dependents) = self.state.callable_dependents.remove(&callable) {
                affected.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
            }
            self.state.summaries.remove(&callable);
            if let Some(dependencies) = self.state.callable_dependencies.remove(&callable) {
                for dependency in dependencies {
                    if let Some(dependents) = self.state.callable_dependents.get_mut(&dependency) {
                        dependents.remove(&callable);
                    }
                }
            }
        }
        self.state.parameter_contributions.remove(&module);
        self.state.parameter_contribution_slots.replace_source(
            ContributionSource::TopLevel(module.clone()),
            std::iter::empty::<(ParameterSlot, InferredValue)>(),
        );
        self.state.classes.retain(|id, _| id.module != module);
        self.state.summaries.retain(|id, _| id.owner.module != module);

        for dependents in self.state.callable_dependents.values_mut() {
            dependents.retain(|id| id.owner.module != module);
        }

        affected.retain(|id| self.state.files.contains_key(id));
        self.state.generation = next_generation;

        if !affected.is_empty() {
            let trace = rebuild_affected_state(&mut self.state, next_generation, affected, &|| false).expect("uncancelled rebuild must complete");
            #[cfg(test)]
            {
                self.state.last_trace = Some(trace.into());
            }
            #[cfg(not(test))]
            drop(trace);
        }
        self.state.generation
    }
}

pub(crate) fn rebuild_affected_state(
    state: &mut SemanticState,
    generation: SemanticGeneration,
    mut affected: BTreeSet<ModuleId>,
    cancelled: &dyn Fn() -> bool,
) -> Option<RebuildTraceData> {
    let previous_summaries = state.summaries.clone();
    let previous_parameters = state.parameter_facts.clone();
    let mut trace = RebuildTraceData::default();
    let mut analysis_by_module = loop {
        if cancelled() {
            return None;
        }
        for module in &affected {
            state.parameter_contributions.remove(module);
            state.parameter_contribution_slots.replace_source(
                ContributionSource::TopLevel(module.clone()),
                std::iter::empty::<(ParameterSlot, InferredValue)>(),
            );
        }

        let classes = state.classes.clone();
        let graph = state.graph.clone();

        let inputs = state
            .files
            .values()
            .filter(|file| affected.contains(&file.module))
            .map(|file| file.source.clone())
            .collect::<Vec<_>>();
        let seed_summaries = state
            .summaries
            .iter()
            .filter(|(id, _)| !affected.contains(&id.owner.module))
            .map(|(id, summary)| (id.clone(), summary.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut base_parameters = ParameterFacts::default();
        for (module, contribution) in &state.parameter_contributions {
            if !affected.contains(module) {
                base_parameters.merge_from(contribution);
            }
        }
        let solved = infer::solve_affected_callables_with_cancel(&inputs, &classes, &graph, generation, seed_summaries, base_parameters, cancelled)?;
        let solved_source_analyses = solved.source_analyses;

        // One unified source result owns local, field, parameter, and summary
        // products. Do not re-enter flow for individual fact families.
        for (module, analysis) in &solved_source_analyses {
            let facts = analysis.parameter_facts.clone();
            state.parameter_contributions.insert(module.clone(), facts.clone());
            state.parameter_contribution_slots.replace_source(
                ContributionSource::TopLevel(module.clone()),
                facts.iter().map(|((callable, name), value)| {
                    (
                        ParameterSlot {
                            callable: callable.clone(),
                            name: name.clone(),
                        },
                        value.clone(),
                    )
                }),
            );
        }

        let mut aggregate_parameters = ParameterFacts::default();
        for contribution in state.parameter_contributions.values() {
            aggregate_parameters.merge_from(contribution);
        }
        state.parameter_facts = aggregate_parameters.iter().map(|(key, value)| (key.clone(), value.clone())).collect();

        // Callable worklist summaries are solver inputs for the final source
        // pass, not an independent publication product. Publish summaries
        // emitted by that same source-backed analysis and retain only
        // unaffected summaries from the previous state.
        let mut published_summaries = state.summaries.clone();
        published_summaries.retain(|id, _| !affected.contains(&id.owner.module));
        for analysis in solved_source_analyses.values() {
            for (summary, evidence) in &analysis.summaries {
                if *evidence {
                    published_summaries.insert(summary.callable.clone(), summary.clone());
                }
            }
        }
        infer::complete_missing_summaries(&inputs, &classes, generation, &aggregate_parameters, &mut published_summaries);
        state.summaries = published_summaries;

        let mut additions = BTreeSet::new();
        for id in previous_summaries.keys().chain(state.summaries.keys()) {
            if previous_summaries.get(id) != state.summaries.get(id) {
                trace.callables_recomputed.insert(id.clone());
                if let Some(dependents) = state.callable_dependents.get(id) {
                    additions.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
                }
            }
        }
        for ((callable, name), before) in previous_parameters.iter() {
            if state.parameter_facts.get(&(callable.clone(), name.clone())) != Some(before) {
                additions.insert(callable.owner.module.clone());
                if let Some(dependents) = state.callable_dependents.get(callable) {
                    additions.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
                }
            }
        }
        for ((callable, name), after) in state.parameter_facts.iter() {
            if previous_parameters.get(&(callable.clone(), name.clone())) != Some(after) {
                additions.insert(callable.owner.module.clone());
                if let Some(dependents) = state.callable_dependents.get(callable) {
                    additions.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
                }
            }
        }
        additions.retain(|module| state.files.contains_key(module) && !affected.contains(module));
        if additions.is_empty() {
            trace.modules_recomputed = affected.clone();
            break solved_source_analyses;
        }
        affected.extend(additions);
    };

    let existing_modules = affected
        .iter()
        .filter(|module| state.files.contains_key(*module))
        .cloned()
        .collect::<BTreeSet<_>>();
    state.field_facts.retain(|field, _| !affected.contains(&field.owner.module));
    for analysis in analysis_by_module.values() {
        state
            .field_facts
            .extend(analysis.field_facts.iter().map(|(key, value)| (key.clone(), value.clone())));
    }
    for (module, analysis) in &analysis_by_module {
        state.parameter_contributions.insert(module.clone(), analysis.parameter_facts.clone());
        state.parameter_contribution_slots.replace_source(
            ContributionSource::TopLevel(module.clone()),
            analysis.parameter_facts.iter().map(|((callable, name), value)| {
                (
                    ParameterSlot {
                        callable: callable.clone(),
                        name: name.clone(),
                    },
                    value.clone(),
                )
            }),
        );
    }
    update_callable_edges(state);
    for module in existing_modules {
        let Some(file) = state.files.get_mut(&module) else { continue };
        if let Some(analysis) = analysis_by_module.remove(&module) {
            file.local_facts = analysis.local_facts.clone();
            file.field_facts = analysis.field_facts.clone();
            file.parameter_facts = analysis.parameter_facts.clone();
        }
        file.dependencies = DependencySet {
            imports: state.graph.imports(&module).iter().filter_map(|edge| edge.target.clone()).collect(),
        };
    }
    Some(trace)
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

    #[test]
    fn engine_uses_one_unified_surface_flow_result() {
        let before_flow = crate::semantic::flow::test_flow_passes();
        let before_solver = infer::test_solver_rounds();
        let mut engine = SemanticEngine::empty();
        let uri = Url::parse("file:///one-pass-engine.ph").expect("fixture URI");
        engine.update_file(&uri, FileRevision(1), &parse(ONE_PASS_FIXTURE, 0).program);
        let after_flow = crate::semantic::flow::test_flow_passes();
        let flow_passes = after_flow.0 - before_flow.0;
        let source_passes = after_flow.1 - before_flow.1;
        let callable_passes = after_flow.2 - before_flow.2;
        let solver_rounds = infer::test_solver_rounds() - before_solver;

        assert!(callable_passes > 0, "callable worklist did not enter analyze_callable");
        assert_eq!(callable_passes, solver_rounds, "each solver step must account for one callable flow pass");
        let allowed_final_stabilized_passes = 1;
        assert_eq!(source_passes, allowed_final_stabilized_passes);
        assert!(
            flow_passes <= solver_rounds + allowed_final_stabilized_passes,
            "duplicate unified traversal: flow_passes={flow_passes}, solver_rounds={solver_rounds}, allowed_final={allowed_final_stabilized_passes}"
        );

        let module = ModuleId::from_uri(&uri);
        let file = engine.state.files.get(&module).expect("fixture file published");
        let local_binding = file
            .source
            .scopes
            .bindings
            .values()
            .find(|binding| binding.name == "local")
            .expect("fixture local binding")
            .id;
        assert!(file.local_facts.facts_for(local_binding).next().is_some(), "local product missing");
        assert!(file.field_facts.iter().next().is_some(), "field product missing");
        assert!(file.parameter_facts.iter().next().is_some(), "parameter product missing");
        assert!(
            engine.state.summaries.values().any(|summary| summary.callable.owner.module == module),
            "summary product missing"
        );
    }
}

/// Diffs callable dependency edges against the currently published working
/// graph. Unchanged callables retain their reverse edges untouched.
fn update_callable_edges(state: &mut SemanticState) {
    let mut identities = state.callable_dependencies.keys().cloned().collect::<BTreeSet<_>>();
    identities.extend(state.summaries.keys().cloned());
    for callable in identities {
        let old = state.callable_dependencies.get(&callable).cloned().unwrap_or_default();
        let new = state
            .summaries
            .get(&callable)
            .map(|summary| summary.dependencies.iter().cloned().collect::<BTreeSet<_>>())
            .unwrap_or_default();
        for dependency in old.difference(&new) {
            if let Some(dependents) = state.callable_dependents.get_mut(dependency) {
                dependents.remove(&callable);
            }
        }
        for dependency in new.difference(&old) {
            state.callable_dependents.entry(dependency.clone()).or_default().insert(callable.clone());
        }
        if new.is_empty() {
            state.callable_dependencies.remove(&callable);
        } else {
            state.callable_dependencies.insert(callable, new);
        }
    }
    state.callable_dependents.retain(|_, dependents| !dependents.is_empty());
}
