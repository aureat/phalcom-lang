//! Mutable worker-only semantic analysis engine.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use phalcom_ast::ast::Program;
use tower_lsp::lsp_types::Url;

#[cfg(test)]
use super::RebuildTrace;
use super::callable::CallableSummary;
use super::core_source;
#[cfg(test)]
use super::facts::{FieldEvidenceKind, ValueShape};
use super::facts::{FileRevision, InferredValue, ParameterContributions, ParameterFacts, ParameterSlot};
#[cfg(test)]
use super::ids::DispatchSide;
use super::ids::{CORE_MODULE_URI, CallableId, ClassId, DocumentModuleMap, FieldId, ModuleId};
use super::invalidation::{SourceChangeKind, classify_source_delta};
use super::module_graph::ModuleGraph;
use super::query::SemanticGeneration;
use super::snapshot::{FileSourceSnapshot, SemanticSnapshot, StaticSemanticSnapshot};
use super::surface::{ClassSurface, build_module_surface};
use super::{DependencySet, FileSemanticSnapshot, infer};
use crate::perf::{PerfCounters, PerfCountersHandle};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RebuildTraceData {
    pub modules_recomputed: BTreeSet<ModuleId>,
    pub callables_changed: BTreeSet<CallableId>,
    pub callables_recomputed: BTreeSet<CallableId>,
}

#[derive(Clone, Default)]
pub(crate) struct SemanticState {
    pub generation: SemanticGeneration,
    pub files: Arc<BTreeMap<ModuleId, Arc<FileSemanticSnapshot>>>,
    pub classes: Arc<BTreeMap<ClassId, Arc<ClassSurface>>>,
    pub summaries: Arc<BTreeMap<CallableId, Arc<CallableSummary>>>,
    pub field_facts: Arc<BTreeMap<FieldId, InferredValue>>,
    pub parameter_facts: Arc<BTreeMap<(CallableId, String), InferredValue>>,
    pub parameter_contributions: Arc<BTreeMap<ModuleId, ParameterFacts>>,
    pub parameter_contribution_slots: Arc<ParameterContributions>,
    pub callable_dependencies: Arc<BTreeMap<CallableId, BTreeSet<CallableId>>>,
    pub callable_dependents: Arc<BTreeMap<CallableId, BTreeSet<CallableId>>>,
    pub graph: Arc<ModuleGraph>,
    pub documents: DocumentModuleMap,
    pub static_snapshot: Option<Arc<StaticSemanticSnapshot>>,
    #[cfg(test)]
    pub last_trace: Option<RebuildTrace>,
}

/// Mutable single-threaded semantic analysis worker engine.
pub struct SemanticEngine {
    state: SemanticState,
    counters: PerfCountersHandle,
}

impl Clone for SemanticEngine {
    fn clone(&self) -> Self {
        self.counters.semantic_candidate_state_clones.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            state: self.state.clone(),
            counters: self.counters.clone(),
        }
    }
}

impl Default for SemanticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticEngine {
    /// Creates a new empty semantic engine with zero state (generation 0).
    pub fn new() -> Self {
        Self::new_with_counters(Arc::new(PerfCounters::new()))
    }

    /// Creates an engine whose semantic passes report into `counters`.
    pub fn new_with_counters(counters: PerfCountersHandle) -> Self {
        Self {
            state: SemanticState::default(),
            counters,
        }
    }

    /// Creates an engine initialized with zero state.
    pub fn empty() -> Self {
        Self::new()
    }

    /// Returns the counter set owned by this engine.
    pub fn perf_counters(&self) -> PerfCountersHandle {
        self.counters.clone()
    }

    /// Returns the current semantic generation.
    pub fn generation(&self) -> SemanticGeneration {
        self.state.generation
    }

    /// Atomically publishes or clears static semantics and their URI identities.
    pub fn set_static_analysis(&mut self, analysis: Option<(Arc<StaticSemanticSnapshot>, DocumentModuleMap)>) {
        if let Some((snapshot, documents)) = analysis {
            self.state.static_snapshot = Some(snapshot);
            self.state.documents = documents;
        } else {
            self.state.static_snapshot = None;
        }
    }

    /// Produces an immutable published snapshot of current engine state.
    pub fn snapshot(&self) -> SemanticSnapshot {
        SemanticSnapshot {
            generation: self.state.generation,
            files: self.state.files.clone(),
            classes: self.state.classes.clone(),
            summaries: self.state.summaries.clone(),
            field_facts: self.state.field_facts.clone(),
            parameter_facts: self.state.parameter_facts.clone(),
            graph: self.state.graph.clone(),
            documents: Arc::new(self.state.documents.clone()),
            static_snapshot: self.state.static_snapshot.clone(),
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

    /// Updates one file while retaining its already-ingested source text for
    /// exact callable body comparison.
    pub fn update_file_with_source(&mut self, uri: &Url, revision: FileRevision, text: Arc<str>, program: &Program) -> SemanticGeneration {
        self.update_files_batch_with_source(vec![(uri.clone(), revision, text, program.clone())])
    }

    /// Updates several file contributions in a single semantic batch transaction.
    pub fn update_files_batch(&mut self, files: Vec<(Url, FileRevision, Program)>) -> SemanticGeneration {
        let files = files
            .into_iter()
            .map(|(uri, revision, program)| (uri, revision, Arc::from(""), program))
            .collect();
        self.update_files_batch_with_source(files)
    }

    /// Updates several files with source text retained by source ingestion.
    pub fn update_files_batch_with_source(&mut self, files: Vec<(Url, FileRevision, Arc<str>, Program)>) -> SemanticGeneration {
        self.update_files_batch_with_source_cancel(files, &|| false)
            .expect("uncancelled update must complete")
    }

    fn update_files_batch_inner(&mut self, files: Vec<(Url, FileRevision, Arc<str>, Program)>, cancelled: &dyn Fn() -> bool) -> Option<SemanticGeneration> {
        for (uri, _, _, _) in &files {
            self.state.documents.ensure_lsp_for_uri(uri);
        }
        let files = files
            .into_iter()
            .filter(|(uri, revision, _, _)| {
                let module = self.state.documents.lsp_for_uri(uri).expect("document mapping was seeded");
                self.state.files.get(module).is_none_or(|file| *revision > file.revision)
            })
            .collect::<Vec<_>>();
        if files.is_empty() {
            return Some(self.state.generation);
        }
        let next_generation = SemanticGeneration(self.state.generation.0 + 1);

        let mut affected = BTreeSet::new();
        let mut dirty_callables = BTreeSet::new();
        let mut precise_body_frontier = true;

        let mut change_kinds = Vec::new();
        for (uri, revision, text, program) in files {
            if cancelled() {
                return None;
            }
            let module = self.state.documents.lsp_for_uri(&uri).expect("document mapping was seeded").clone();
            let old_source = self.state.files.get(&module).map(|file| file.source.clone());
            let surface = if module.as_str() == CORE_MODULE_URI {
                core_source::build_core_surface(&program)
            } else {
                build_module_surface(module.clone(), &program)
            };
            let program = Arc::new(program);
            let scopes = super::scope::build_scope_graph(module.clone(), &program);
            let occurrences = super::occurrence::build_occurrence_index(module.clone(), &program, &surface, &scopes);

            let callables = surface.callable_index();
            let source = Arc::new(FileSourceSnapshot {
                module: module.clone(),
                text,
                program: program.clone(),
                surface,
                scopes,
                callables,
            });
            let delta = classify_source_delta(&module, old_source.as_deref(), Some(&source));
            if delta.kind != SourceChangeKind::BodyOnly {
                precise_body_frontier = false;
                Arc::make_mut(&mut self.state.classes).retain(|id, _| id.module != module);
                Arc::make_mut(&mut self.state.classes).extend(source.surface.classes.iter().map(|(id, class)| (id.clone(), Arc::new(class.clone()))));
            }
            affected.insert(module.clone());
            if delta.kind == SourceChangeKind::BodyOnly {
                dirty_callables.extend(delta.changed_callables.iter().cloned());
            } else {
                affected.extend(self.state.graph.dependent_closure(&module));
            }
            change_kinds.push((module.clone(), delta));
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
            Arc::make_mut(&mut self.state.files).insert(module.clone(), Arc::new(snapshot));
        }

        let available = self.state.files.keys().cloned().collect::<BTreeSet<_>>();
        for (module, delta) in &change_kinds {
            if delta.kind == SourceChangeKind::BodyOnly {
                continue;
            }
            if let Some(file) = self.state.files.get(module) {
                Arc::make_mut(&mut self.state.graph).update(file.module.clone(), &file.source.program, &available);
            }
        }
        for (module, delta) in &change_kinds {
            if delta.kind == SourceChangeKind::FileAddedRemoved {
                affected.extend(Arc::make_mut(&mut self.state.graph).repair_provider(module, &available));
            }
        }
        for (module, delta) in &change_kinds {
            if delta.kind != SourceChangeKind::BodyOnly {
                affected.extend(self.state.graph.dependent_closure(module));
            }
        }

        self.state.generation = next_generation;
        let trace = rebuild_affected_state(
            &mut self.state,
            next_generation,
            affected,
            if precise_body_frontier { Some(dirty_callables) } else { None },
            cancelled,
            &self.counters,
        )?;
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
        let files = files
            .into_iter()
            .map(|(uri, revision, program)| (uri, revision, Arc::from(""), program))
            .collect();
        self.update_files_batch_with_source_cancel(files, cancelled)
    }

    fn update_files_batch_with_source_cancel(
        &mut self,
        files: Vec<(Url, FileRevision, Arc<str>, Program)>,
        cancelled: &dyn Fn() -> bool,
    ) -> Option<SemanticGeneration> {
        if cancelled() {
            return None;
        }
        let mut candidate = self.clone();
        let generation = candidate.update_files_batch_inner(files, cancelled)?;
        if cancelled() {
            return None;
        }
        record_product_reuse(&self.state, &candidate.state, &self.counters);
        *self = candidate;
        Some(generation)
    }

    /// Applies removals, ordinary files, and one logical core replacement as
    /// one candidate transaction. Nothing reaches the live engine when
    /// cancellation observes a newer worker epoch.
    pub fn apply_mutations_with_cancel(
        &mut self,
        removals: Vec<Url>,
        files: Vec<(Url, FileRevision, Program)>,
        core_update: Option<(FileRevision, Program)>,
        cancelled: &dyn Fn() -> bool,
    ) -> Option<SemanticGeneration> {
        let files = files
            .into_iter()
            .map(|(uri, revision, program)| (uri, revision, Arc::from(""), program))
            .collect();
        let core_update = core_update.map(|(revision, program)| (revision, Arc::from(""), program));
        self.apply_mutations_with_source_cancel(removals, files, core_update, cancelled)
    }

    /// Applies one worker-ingested batch while retaining source text for exact
    /// callable invalidation.
    pub fn apply_mutations_with_source_cancel(
        &mut self,
        removals: Vec<Url>,
        files: Vec<(Url, FileRevision, Arc<str>, Program)>,
        core_update: Option<(FileRevision, Arc<str>, Program)>,
        cancelled: &dyn Fn() -> bool,
    ) -> Option<SemanticGeneration> {
        if cancelled() {
            return None;
        }
        let mut candidate = self.clone();
        for uri in removals {
            if cancelled() {
                return None;
            }
            candidate.remove_file_with_cancel(&uri, cancelled)?;
        }
        if !files.is_empty() {
            candidate.update_files_batch_inner(files, cancelled)?;
        }
        if let Some((revision, text, program)) = core_update {
            let uri = Url::parse(CORE_MODULE_URI).expect("core module URI must parse");
            candidate.update_files_batch_inner(vec![(uri, revision, text, program)], cancelled)?;
        }
        let generation = candidate.state.generation;
        if cancelled() {
            return None;
        }
        record_product_reuse(&self.state, &candidate.state, &self.counters);
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
        self.remove_file_with_cancel(uri, &|| false).expect("uncancelled removal must complete")
    }

    fn remove_file_with_cancel(&mut self, uri: &Url, cancelled: &dyn Fn() -> bool) -> Option<SemanticGeneration> {
        if cancelled() {
            return None;
        }
        let Some(module) = self.state.documents.lsp_for_uri(uri).cloned() else {
            return Some(self.state.generation);
        };
        if Arc::make_mut(&mut self.state.files).remove(&module).is_none() {
            return Some(self.state.generation);
        }
        self.state.documents.remove_uri(uri);

        let next_generation = SemanticGeneration(self.state.generation.0 + 1);
        let mut affected: BTreeSet<ModuleId> = self.state.graph.dependent_closure(&module).into_iter().collect();
        Arc::make_mut(&mut self.state.graph).remove(&module);

        let available = self.state.files.keys().cloned().collect::<BTreeSet<_>>();
        affected.extend(Arc::make_mut(&mut self.state.graph).repair_provider(&module, &available));

        let old_callables = self.state.summaries.keys().filter(|id| id.owner.module == module).cloned().collect::<Vec<_>>();
        for callable in old_callables {
            if let Some(dependents) = Arc::make_mut(&mut self.state.callable_dependents).remove(&callable) {
                affected.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
            }
            Arc::make_mut(&mut self.state.summaries).remove(&callable);
            if let Some(dependencies) = Arc::make_mut(&mut self.state.callable_dependencies).remove(&callable) {
                for dependency in dependencies {
                    if let Some(dependents) = Arc::make_mut(&mut self.state.callable_dependents).get_mut(&dependency) {
                        dependents.remove(&callable);
                    }
                }
            }
        }
        Arc::make_mut(&mut self.state.parameter_contributions).remove(&module);
        let deltas = Arc::make_mut(&mut self.state.parameter_contribution_slots).remove_module(&module);
        record_parameter_deltas(&self.counters, 0, &deltas);
        Arc::make_mut(&mut self.state.classes).retain(|id, _| id.module != module);
        Arc::make_mut(&mut self.state.summaries).retain(|id, _| id.owner.module != module);

        for dependents in Arc::make_mut(&mut self.state.callable_dependents).values_mut() {
            dependents.retain(|id| id.owner.module != module);
        }

        affected.retain(|id| self.state.files.contains_key(id));
        self.state.generation = next_generation;

        if !affected.is_empty() {
            let trace = rebuild_affected_state(&mut self.state, next_generation, affected, None, cancelled, &self.counters)?;
            #[cfg(test)]
            {
                self.state.last_trace = Some(trace.into());
            }
            #[cfg(not(test))]
            drop(trace);
        }
        Some(self.state.generation)
    }
}

fn record_product_reuse(previous: &SemanticState, next: &SemanticState, counters: &PerfCounters) {
    for (id, product) in previous.files.iter() {
        if next.files.get(id).is_some_and(|candidate| Arc::ptr_eq(product, candidate)) {
            counters.published_file_products_reused.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    for (id, product) in previous.classes.iter() {
        if next.classes.get(id).is_some_and(|candidate| Arc::ptr_eq(product, candidate)) {
            counters.published_class_products_reused.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    for (id, product) in previous.summaries.iter() {
        if next.summaries.get(id).is_some_and(|candidate| Arc::ptr_eq(product, candidate)) {
            counters.published_summary_products_reused.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

fn record_parameter_deltas(counters: &PerfCounters, touched_slots: usize, deltas: &[super::facts::ParameterFactDelta]) {
    counters.parameter_sources_replaced.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    counters
        .parameter_slots_touched
        .fetch_add(touched_slots as u64, std::sync::atomic::Ordering::Relaxed);
    counters
        .parameter_slots_changed
        .fetch_add(deltas.len() as u64, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn rebuild_affected_state(
    state: &mut SemanticState,
    generation: SemanticGeneration,
    mut affected: BTreeSet<ModuleId>,
    mut dirty_callables: Option<BTreeSet<CallableId>>,
    cancelled: &dyn Fn() -> bool,
    counters: &PerfCounters,
) -> Option<RebuildTraceData> {
    let previous_summaries = state.summaries.clone();
    let previous_parameters = state.parameter_facts.clone();
    let precise_frontier = dirty_callables.is_some();
    let mut trace = RebuildTraceData::default();
    let mut analysis_by_module = loop {
        if cancelled() {
            return None;
        }
        for module in &affected {
            Arc::make_mut(&mut state.parameter_contributions).remove(module);
            let deltas = Arc::make_mut(&mut state.parameter_contribution_slots).remove_module(module);
            record_parameter_deltas(counters, 0, &deltas);
        }

        let classes = state
            .classes
            .iter()
            .map(|(id, surface)| (id.clone(), (**surface).clone()))
            .collect::<BTreeMap<_, _>>();
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
            .map(|(id, summary)| (id.clone(), (**summary).clone()))
            .collect::<BTreeMap<_, _>>();
        let base_contributions = state.parameter_contribution_slots.as_ref().clone();
        let seeds = dirty_callables.as_ref();
        let solved = infer::solve_affected_callables_with_cancel(
            &inputs,
            &classes,
            graph.as_ref(),
            generation,
            seed_summaries,
            base_contributions,
            seeds,
            cancelled,
            counters,
        )?;
        let solved_source_analyses = solved.source_analyses;
        trace.callables_recomputed.extend(solved.callables_visited.iter().cloned());
        trace.callables_changed.extend(solved.callables_changed.iter().cloned());
        dirty_callables = precise_frontier.then(BTreeSet::new);

        // One unified source result owns local, field, parameter, and summary
        // products. Do not re-enter flow for individual fact families.
        for (module, analysis) in &solved_source_analyses {
            let facts = analysis.parameter_facts.clone();
            Arc::make_mut(&mut state.parameter_contributions).insert(module.clone(), facts.clone());
            for (source, contribution) in &analysis.parameter_contributions {
                let deltas = Arc::make_mut(&mut state.parameter_contribution_slots).replace_source(
                    source.clone(),
                    contribution.iter().map(|((callable, name), value)| {
                        (
                            ParameterSlot {
                                callable: callable.clone(),
                                name: name.clone(),
                            },
                            value.clone(),
                        )
                    }),
                );
                record_parameter_deltas(counters, contribution.iter().count(), &deltas);
            }
        }

        let aggregate_parameters = state
            .parameter_contribution_slots
            .joined_iter()
            .fold(ParameterFacts::default(), |mut facts, (slot, value)| {
                facts.record(slot.callable.clone(), slot.name.clone(), value.clone());
                facts
            });
        state.parameter_facts = Arc::new(aggregate_parameters.iter().map(|(key, value)| (key.clone(), value.clone())).collect());

        // Callable worklist summaries are solver inputs for the final source
        // pass, not an independent publication product. Publish summaries
        // emitted by that same source-backed analysis and retain only
        // unaffected summaries from the previous state.
        let mut published_summaries = state
            .summaries
            .iter()
            .map(|(id, summary)| (id.clone(), (**summary).clone()))
            .collect::<BTreeMap<_, _>>();
        published_summaries.retain(|id, _| !affected.contains(&id.owner.module));
        for analysis in solved_source_analyses.values() {
            for (summary, evidence) in &analysis.summaries {
                if *evidence {
                    published_summaries.insert(summary.callable.clone(), summary.clone());
                }
            }
        }
        infer::complete_missing_summaries(&inputs, &classes, generation, &aggregate_parameters, &mut published_summaries);
        state.summaries = Arc::new(
            published_summaries
                .into_iter()
                .map(|(id, summary)| {
                    let product = state
                        .summaries
                        .get(&id)
                        .filter(|previous| !infer::callable_summary_changed(Some(previous.as_ref()), Some(&summary)))
                        .cloned()
                        .unwrap_or_else(|| Arc::new(summary));
                    (id, product)
                })
                .collect(),
        );

        let mut additions = BTreeSet::new();
        let mut propagated_callables = BTreeSet::new();
        let summary_ids = previous_summaries.keys().chain(state.summaries.keys()).cloned().collect::<BTreeSet<_>>();
        for id in summary_ids {
            if infer::callable_summary_changed(previous_summaries.get(&id).map(Arc::as_ref), state.summaries.get(&id).map(Arc::as_ref)) {
                if let Some(dependents) = state.callable_dependents.get(&id) {
                    for dependent in dependents {
                        additions.insert(dependent.owner.module.clone());
                        propagated_callables.insert(dependent.clone());
                    }
                }
            }
        }
        for ((callable, name), before) in previous_parameters.iter() {
            if state.parameter_facts.get(&(callable.clone(), name.clone())) != Some(before) {
                additions.insert(callable.owner.module.clone());
                if let Some(dependents) = state.callable_dependents.get(callable) {
                    additions.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
                }
                additions.extend(
                    state
                        .parameter_contributions
                        .iter()
                        .filter_map(|(module, contribution)| contribution.get(callable, name).is_some().then_some(module.clone())),
                );
            }
        }
        for ((callable, name), after) in state.parameter_facts.iter() {
            if previous_parameters.get(&(callable.clone(), name.clone())) != Some(after) {
                additions.insert(callable.owner.module.clone());
                if let Some(dependents) = state.callable_dependents.get(callable) {
                    additions.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
                }
                additions.extend(
                    state
                        .parameter_contributions
                        .iter()
                        .filter_map(|(module, contribution)| contribution.get(callable, name).is_some().then_some(module.clone())),
                );
            }
        }
        additions.retain(|module| state.files.contains_key(module) && !affected.contains(module));
        if additions.is_empty() {
            trace.modules_recomputed = affected.clone();
            break solved_source_analyses;
        }
        if precise_frontier {
            let dirty = dirty_callables.as_mut().expect("precise dirty callable frontier initialized");
            dirty.extend(propagated_callables);
        } else {
            dirty_callables = None;
        }
        affected.extend(additions);
    };

    let existing_modules = affected
        .iter()
        .filter(|module| state.files.contains_key(*module))
        .cloned()
        .collect::<BTreeSet<_>>();
    Arc::make_mut(&mut state.field_facts).retain(|field, _| !affected.contains(&field.owner.module));
    for analysis in analysis_by_module.values() {
        Arc::make_mut(&mut state.field_facts).extend(analysis.field_facts.iter().map(|(key, value)| (key.clone(), value.clone())));
    }
    for (module, analysis) in &analysis_by_module {
        Arc::make_mut(&mut state.parameter_contributions).insert(module.clone(), analysis.parameter_facts.clone());
        for (source, contribution) in &analysis.parameter_contributions {
            let deltas = Arc::make_mut(&mut state.parameter_contribution_slots).replace_source(
                source.clone(),
                contribution.iter().map(|((callable, name), value)| {
                    (
                        ParameterSlot {
                            callable: callable.clone(),
                            name: name.clone(),
                        },
                        value.clone(),
                    )
                }),
            );
            record_parameter_deltas(counters, contribution.iter().count(), &deltas);
        }
    }
    update_callable_edges(state);
    for module in existing_modules {
        let Some(file) = Arc::make_mut(&mut state.files).get_mut(&module) else {
            continue;
        };
        let file = Arc::make_mut(file);
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
            if let Some(dependents) = Arc::make_mut(&mut state.callable_dependents).get_mut(dependency) {
                dependents.remove(&callable);
            }
        }
        for dependency in new.difference(&old) {
            Arc::make_mut(&mut state.callable_dependents)
                .entry(dependency.clone())
                .or_default()
                .insert(callable.clone());
        }
        if new.is_empty() {
            Arc::make_mut(&mut state.callable_dependencies).remove(&callable);
        } else {
            Arc::make_mut(&mut state.callable_dependencies).insert(callable, new);
        }
    }
    Arc::make_mut(&mut state.callable_dependents).retain(|_, dependents| !dependents.is_empty());
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
        let before_solver_steps = infer::test_solver_steps();
        let mut engine = SemanticEngine::empty();
        let uri = Url::parse("file:///one-pass-engine.ph").expect("fixture URI");
        engine.update_file(&uri, FileRevision(1), &parse(ONE_PASS_FIXTURE, 0).program);
        let after_flow = crate::semantic::flow::test_flow_passes();
        let flow_passes = after_flow.0 - before_flow.0;
        let source_passes = after_flow.1 - before_flow.1;
        let callable_passes = after_flow.2 - before_flow.2;
        let solver_rounds = infer::test_solver_rounds() - before_solver;
        let solver_steps = infer::test_solver_steps() - before_solver_steps;

        assert!(callable_passes > 0, "callable worklist did not enter analyze_callable");
        assert!(solver_rounds > 0, "callable solver did not enter a round");
        assert_ne!(solver_rounds, solver_steps, "solver rounds and callable steps must remain distinct");
        assert_eq!(callable_passes, solver_steps, "each callable step must account for one callable flow pass");
        let initial_source_passes = 1;
        let allowed_final_stabilized_passes = 1;
        assert!(
            source_passes <= initial_source_passes + allowed_final_stabilized_passes,
            "source flow passes exceeded one permitted stabilization pass: source_passes={source_passes}"
        );
        assert!(
            flow_passes <= solver_steps + initial_source_passes + allowed_final_stabilized_passes,
            "duplicate unified traversal: flow_passes={flow_passes}, solver_steps={solver_steps}, initial_source={initial_source_passes}, allowed_final={allowed_final_stabilized_passes}"
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

        let product = ClassId::new(module.clone(), "Product");
        let seed_evidence = file.field_facts.evidence(&product, "_seed", DispatchSide::Instance);
        assert!(
            seed_evidence.iter().any(|evidence| evidence.kind == FieldEvidenceKind::DeclarationInitializer),
            "Product._seed declaration initializer evidence missing"
        );
        assert!(
            seed_evidence
                .iter()
                .any(|evidence| evidence.kind == FieldEvidenceKind::ConstructorInitialization),
            "Product._seed constructor initialization evidence missing"
        );

        let service = ClassId::new(module.clone(), "Service");
        let consume = engine
            .state
            .classes
            .get(&service)
            .and_then(|class| class.member("consume(_)", DispatchSide::Class))
            .expect("Service.consume(_) member");
        let input = file
            .parameter_facts
            .get(&consume.callable, "input")
            .expect("Service.consume(_) input parameter fact");
        assert_eq!(input.shape, ValueShape::Instance(ClassId::new(ModuleId::new(CORE_MODULE_URI), "Int")));
        assert!(
            engine.state.summaries.values().any(|summary| summary.callable.owner.module == module),
            "summary product missing"
        );
    }

    #[test]
    fn body_edit_reuses_unrelated_published_products() {
        let mut engine = SemanticEngine::empty();
        let left = Url::parse("file:///sharing-left.ph").expect("left URI");
        let right = Url::parse("file:///sharing-right.ph").expect("right URI");
        engine.update_files_batch(vec![
            (left.clone(), FileRevision(1), parse("class Left { value() { 1 } }", 0).program),
            (right.clone(), FileRevision(1), parse("class Right { value() { 2 } }", 0).program),
        ]);
        let first = engine.snapshot();
        engine.update_file(&left, FileRevision(2), &parse("class Left { value() { 3 } }", 0).program);
        let second = engine.snapshot();

        let right_module = ModuleId::from_uri(&right);
        let right_class = ClassId::new(right_module.clone(), "Right");
        let right_callable = CallableId {
            owner: right_class.clone(),
            selector: "value()".to_string(),
            side: super::super::ids::DispatchSide::Instance,
        };
        assert!(Arc::ptr_eq(
            first.files.get(&right_module).expect("right file"),
            second.files.get(&right_module).expect("right file retained")
        ));
        assert!(Arc::ptr_eq(
            first.classes.get(&right_class).expect("right class"),
            second.classes.get(&right_class).expect("right class retained")
        ));
        assert!(Arc::ptr_eq(
            first.summaries.get(&right_callable).expect("right summary"),
            second.summaries.get(&right_callable).expect("right summary retained"),
        ));
    }

    #[test]
    fn exact_body_frontier_visits_only_changed_callable() {
        let mut engine = SemanticEngine::empty();
        let uri = Url::parse("file:///task17-frontier.ph").expect("fixture URI");
        let old = "class A {\n  untouched() { 1 }\n  changed() { 2 }\n}\n";
        let new = "class A {\n  // shifted\n  untouched() { 1 }\n  changed() { 3 }\n}\n";
        engine.update_file_with_source(&uri, FileRevision(1), Arc::from(old), &phalcom_ast::parser::parse(old, 0).program);
        engine.update_file_with_source(&uri, FileRevision(2), Arc::from(new), &phalcom_ast::parser::parse(new, 0).program);

        let trace = engine.last_rebuild_trace().expect("rebuild trace");
        assert!(trace.callables_recomputed.iter().any(|id| id.selector == "changed()"));
        assert!(!trace.callables_recomputed.iter().any(|id| id.selector == "untouched()"));
    }

    fn task17_workspace() -> (Url, Url, &'static str, &'static str) {
        let provider = Url::parse("file:///task17-provider.ph").expect("provider URI");
        let consumer = Url::parse("file:///task17-consumer.ph").expect("consumer URI");
        let provider_text = "class A {\n  @constructor new() { }\n  f() { 1 }\n  g() { 100 }\n}\n";
        let consumer_text = "import .task17_provider as Provider\nclass B {\n  h() { Provider.A.new().f() }\n}\n";
        (provider, consumer, provider_text, consumer_text)
    }

    #[test]
    fn unchanged_summary_stops_callable_propagation() {
        let mut engine = SemanticEngine::empty();
        let (provider, consumer, provider_text, consumer_text) = task17_workspace();
        engine.update_files_batch_with_source(vec![
            (
                provider.clone(),
                FileRevision(1),
                Arc::from(provider_text),
                phalcom_ast::parser::parse(provider_text, 0).program,
            ),
            (
                consumer.clone(),
                FileRevision(1),
                Arc::from(consumer_text),
                phalcom_ast::parser::parse(consumer_text, 0).program,
            ),
        ]);
        let changed = "class A {\n  @constructor new() { }\n  f() { 1  }\n  g() { 100 }\n}\n";
        engine.update_file_with_source(&provider, FileRevision(2), Arc::from(changed), &phalcom_ast::parser::parse(changed, 0).program);
        let trace = engine.last_rebuild_trace().expect("rebuild trace");
        assert!(trace.callables_recomputed.iter().any(|id| id.selector == "f()"));
        assert!(!trace.callables_recomputed.iter().any(|id| id.selector == "g()"));
        assert!(!trace.callables_recomputed.iter().any(|id| id.selector == "h()"));
        let _ = consumer;
    }

    #[test]
    fn changed_summary_propagates_to_callable_dependent() {
        let mut engine = SemanticEngine::empty();
        let (provider, consumer, provider_text, consumer_text) = task17_workspace();
        engine.update_files_batch_with_source(vec![
            (
                provider.clone(),
                FileRevision(1),
                Arc::from(provider_text),
                phalcom_ast::parser::parse(provider_text, 0).program,
            ),
            (
                consumer.clone(),
                FileRevision(1),
                Arc::from(consumer_text),
                phalcom_ast::parser::parse(consumer_text, 0).program,
            ),
        ]);
        let changed = "class A {\n  @constructor new() { }\n  f() { \"changed\" }\n  g() { 100 }\n}\n";
        engine.update_file_with_source(&provider, FileRevision(2), Arc::from(changed), &phalcom_ast::parser::parse(changed, 0).program);
        let trace = engine.last_rebuild_trace().expect("rebuild trace");
        assert!(trace.callables_recomputed.iter().any(|id| id.selector == "f()"));
        assert!(trace.callables_recomputed.iter().any(|id| id.selector == "h()"));
        assert!(!trace.callables_recomputed.iter().any(|id| id.selector == "g()"));
        let _ = consumer;
    }
}
