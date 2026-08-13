//! Mutable worker-only semantic analysis engine.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use phalcom_ast::ast::Program;
use tower_lsp::lsp_types::Url;

use super::callable::CallableSummary;
use super::core_source;
use super::dispatch::{DispatchReceiver, DispatchResolver};
use super::facts::{FileRevision, InferredValue, ParameterFacts};
use super::ids::{CORE_MODULE_URI, CallableId, ClassId, FieldId, ModuleId};
use super::module_graph::ModuleGraph;
use super::query::SemanticGeneration;
use super::snapshot::SemanticSnapshot;
use super::surface::{ClassSurface, build_module_surface};
#[cfg(test)]
use super::RebuildTrace;
use super::{DependencySet, FileSemanticSnapshot, infer, return_for_callable, resolve_named_class};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RebuildTraceData {
    pub modules_recomputed: BTreeSet<ModuleId>,
    pub callables_recomputed: BTreeSet<CallableId>,
}

#[derive(Default)]
pub(crate) struct SemanticState {
    pub generation: SemanticGeneration,
    pub files: BTreeMap<ModuleId, FileSemanticSnapshot>,
    pub classes: BTreeMap<ClassId, ClassSurface>,
    pub summaries: BTreeMap<CallableId, CallableSummary>,
    pub field_facts: BTreeMap<FieldId, InferredValue>,
    pub parameter_facts: BTreeMap<(CallableId, String), InferredValue>,
    pub parameter_contributions: BTreeMap<ModuleId, ParameterFacts>,
    pub callable_dependents: BTreeMap<CallableId, BTreeSet<CallableId>>,
    pub graph: ModuleGraph,
    #[cfg(test)]
    pub last_trace: Option<RebuildTrace>,
}

/// Mutable single-threaded semantic analysis worker engine.
#[derive(Default)]
pub struct SemanticEngine {
    state: SemanticState,
}

impl SemanticEngine {
    /// Creates a new empty semantic engine.
    pub fn new() -> Self {
        let mut engine = Self::default();
        let bundled = core_source::bundled_parse();
        engine.update_core(FileRevision(1), &bundled.program);
        engine.state.generation = SemanticGeneration(0);
        engine
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
            files: self
                .state
                .files
                .iter()
                .map(|(k, v)| (k.clone(), Arc::new(v.clone())))
                .collect(),
            classes: self
                .state
                .classes
                .iter()
                .map(|(k, v)| (k.clone(), Arc::new(v.clone())))
                .collect(),
            summaries: self
                .state
                .summaries
                .iter()
                .map(|(k, v)| (k.clone(), Arc::new(v.clone())))
                .collect(),
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
        if files.is_empty() {
            return self.state.generation;
        }
        let next_generation = SemanticGeneration(self.state.generation.0 + 1);

        let mut affected = BTreeSet::new();
        for (uri, _, _) in &files {
            let module = ModuleId::from_uri(uri);
            affected.insert(module.clone());
            affected.extend(self.state.graph.dependent_closure(&module));
            let old_callables = self
                .state
                .summaries
                .keys()
                .filter(|id| id.owner.module == module)
                .cloned()
                .collect::<Vec<_>>();
            for callable in old_callables {
                if let Some(dependents) = self.state.callable_dependents.get(&callable) {
                    affected.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
                }
            }
        }

        let updated_modules = files.iter().map(|(uri, _, _)| ModuleId::from_uri(uri)).collect::<Vec<_>>();
        for (uri, revision, program) in files {
            let module = ModuleId::from_uri(&uri);
            let surface = if module.as_str() == CORE_MODULE_URI {
                core_source::build_core_surface(&program)
            } else {
                build_module_surface(module.clone(), &program)
            };
            let program = Arc::new(program);
            let scopes = super::scope::build_scope_graph(module.clone(), &program);
            let occurrences = super::occurrence::build_occurrence_index(module.clone(), &program, &surface, &scopes);

            let snapshot = FileSemanticSnapshot {
                revision,
                module: module.clone(),
                program: program.clone(),
                surface: surface.clone(),
                scopes,
                occurrences,
                local_facts: super::facts::LocalFacts::default(),
                field_facts: super::facts::FieldFacts::default(),
                parameter_facts: super::facts::ParameterFacts::default(),
                dependencies: super::DependencySet::default(),
            };
            self.state.files.insert(module.clone(), snapshot);
        }

        let available = self.state.files.keys().cloned().collect::<BTreeSet<_>>();
        for file in self.state.files.values() {
            self.state.graph.update(file.module.clone(), &file.program, &available);
        }
        self.state.graph.refresh_resolutions(&available);

        for module in updated_modules {
            affected.extend(self.state.graph.dependent_closure(&module));
        }

        self.state.generation = next_generation;
        let trace = rebuild_affected_state(&mut self.state, next_generation, affected);
        #[cfg(test)]
        {
            self.state.last_trace = Some(trace.into());
        }
        #[cfg(not(test))]
        drop(trace);

        self.state.generation
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

        let old_callables = self
            .state
            .summaries
            .keys()
            .filter(|id| id.owner.module == module)
            .cloned()
            .collect::<Vec<_>>();
        for callable in old_callables {
            if let Some(dependents) = self.state.callable_dependents.remove(&callable) {
                affected.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
            }
            self.state.summaries.remove(&callable);
        }
        self.state.parameter_contributions.remove(&module);
        self.state.classes.retain(|id, _| id.module != module);
        self.state.summaries.retain(|id, _| id.owner.module != module);

        for dependents in self.state.callable_dependents.values_mut() {
            dependents.retain(|id| id.owner.module != module);
        }

        affected.retain(|id| self.state.files.contains_key(id));
        self.state.generation = next_generation;

        if !affected.is_empty() {
            let trace = rebuild_affected_state(&mut self.state, next_generation, affected);
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

pub(crate) fn rebuild_affected_state(state: &mut SemanticState, generation: SemanticGeneration, mut affected: BTreeSet<ModuleId>) -> RebuildTraceData {
    let previous_summaries = state.summaries.clone();
    let previous_parameters = state.parameter_facts.clone();
    let previous_dependents = state.callable_dependents.clone();
    let mut trace = RebuildTraceData::default();

    loop {
        for module in &affected {
            state.parameter_contributions.remove(module);
        }

        let mut classes = BTreeMap::new();
        for file in state.files.values() {
            classes.extend(file.surface.classes.iter().map(|(id, class)| (id.clone(), class.clone())));
        }
        let graph = state.graph.clone();
        state.classes = classes.clone();

        let inputs = state
            .files
            .values()
            .filter(|file| affected.contains(&file.module))
            .map(|file| (file.module.clone(), file.program.clone(), file.surface.clone()))
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
        let solved = infer::solve_affected_callables(&inputs, &classes, &graph, generation, seed_summaries, base_parameters);
        state.summaries = solved.summaries;

        let solved_parameters = solved.parameter_facts;
        for (module, program, surface) in &inputs {
            let known_class = |name: &str| resolve_named_class(&classes, &graph, module, name);
            let contains_class = |class: &ClassId| classes.contains_key(class);
            let resolver = DispatchResolver::new(&classes);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolver
                    .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                    .is_some_and(|resolved| resolved.member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| return_for_callable(&classes, &state.summaries, id);
            let callable_effects = |id: &CallableId| state.summaries.get(id).map(|summary| summary.effects.clone());
            let parameter_fact = |id: &CallableId, name: &str| solved_parameters.get(id, name).cloned();
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
            let contribution = infer::parameter_facts_for_program(
                program,
                surface,
                module,
                known_class,
                is_constructor,
                contains_class,
                callable_return,
                callable_effects,
                parameter_fact,
                resolve_member,
            );
            state.parameter_contributions.insert(module.clone(), contribution);
        }

        let mut aggregate_parameters = ParameterFacts::default();
        for contribution in state.parameter_contributions.values() {
            aggregate_parameters.merge_from(contribution);
        }
        state.parameter_facts = aggregate_parameters.iter().map(|(key, value)| (key.clone(), value.clone())).collect();

        let mut additions = BTreeSet::new();
        for id in previous_summaries.keys().chain(state.summaries.keys()) {
            if previous_summaries.get(id) != state.summaries.get(id) {
                trace.callables_recomputed.insert(id.clone());
                if let Some(dependents) = previous_dependents.get(id) {
                    additions.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
                }
            }
        }
        for ((callable, name), before) in previous_parameters.iter() {
            if state.parameter_facts.get(&(callable.clone(), name.clone())) != Some(before) {
                additions.insert(callable.owner.module.clone());
                if let Some(dependents) = previous_dependents.get(callable) {
                    additions.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
                }
            }
        }
        for ((callable, name), after) in state.parameter_facts.iter() {
            if previous_parameters.get(&(callable.clone(), name.clone())) != Some(after) {
                additions.insert(callable.owner.module.clone());
                if let Some(dependents) = previous_dependents.get(callable) {
                    additions.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
                }
            }
        }
        additions.retain(|module| state.files.contains_key(module) && !affected.contains(module));
        if additions.is_empty() {
            trace.modules_recomputed = affected.clone();
            break;
        }
        affected.extend(additions);
    }

    let classes = state.classes.clone();
    let graph = state.graph.clone();
    let summaries = state.summaries.clone();
    let existing_modules = affected
        .iter()
        .filter(|module| state.files.contains_key(*module))
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut local_by_module = BTreeMap::new();
    let mut fields_by_module = BTreeMap::new();
    for module in &existing_modules {
        let Some(file) = state.files.get(module) else { continue };
        let known_class = |name: &str| resolve_named_class(&classes, &graph, module, name);
        let contains_class = |class: &ClassId| classes.contains_key(class);
        let resolver = DispatchResolver::new(&classes);
        let is_constructor = |class: &ClassId, selector: &str| {
            resolver
                .resolve(&DispatchReceiver::ClassObject(class.clone()), selector)
                .is_some_and(|resolved| resolved.member.is_constructor)
                || (selector == "new()" && classes.contains_key(class))
        };
        let callable_return = |id: &CallableId| return_for_callable(&classes, &summaries, id);
        let callable_effects = |id: &CallableId| summaries.get(id).map(|summary| summary.effects.clone());
        let parameter_fact = |id: &CallableId, name: &str| state.parameter_facts.get(&(id.clone(), name.to_string())).cloned();
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
        local_by_module.insert(
            module.clone(),
            infer::collect_local_facts_with_returns(
                &file.program,
                &file.surface,
                module,
                known_class,
                is_constructor,
                contains_class,
                callable_return,
                callable_effects,
                parameter_fact,
                resolve_member,
            ),
        );
        fields_by_module.insert(
            module.clone(),
            infer::field_facts_for_surface(
                &file.program,
                &file.surface,
                module,
                known_class,
                is_constructor,
                contains_class,
                callable_return,
                callable_effects,
                parameter_fact,
                resolver,
            ),
        );
    }

    state.field_facts.retain(|field, _| !affected.contains(&field.owner.module));
    for facts in fields_by_module.values() {
        state.field_facts.extend(facts.iter().map(|(key, value)| (key.clone(), value.clone())));
    }
    state.classes = classes;
    let mut callable_dependents: BTreeMap<CallableId, BTreeSet<CallableId>> = BTreeMap::new();
    for summary in state.summaries.values() {
        for dependency in &summary.dependencies {
            callable_dependents.entry(dependency.clone()).or_default().insert(summary.callable.clone());
        }
    }
    state.callable_dependents = callable_dependents;
    for module in existing_modules {
        let Some(file) = state.files.get_mut(&module) else { continue };
        file.local_facts = local_by_module.remove(&module).unwrap_or_default();
        file.field_facts = fields_by_module.remove(&module).unwrap_or_default();
        file.parameter_facts = state.parameter_contributions.get(&module).cloned().unwrap_or_default();
        file.dependencies = DependencySet {
            imports: state.graph.imports(&module).iter().filter_map(|edge| edge.target.clone()).collect(),
        };
    }
    trace
}
