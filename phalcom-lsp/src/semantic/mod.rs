//! VM-free live semantic database for LSP requests.

mod analyzer;
mod callable;
pub(crate) mod core_source;
mod dispatch;
mod facts;
mod flow;
mod ids;
mod infer;
mod invalidation;
mod module_graph;
mod occurrence;
mod query;
mod scope;
mod surface;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::RwLock;

use phalcom_ast::ast::Program;
use tower_lsp::lsp_types::Url;
use phalcom_common::range::SourceRange;

pub(crate) use analyzer::{AnalysisContext, analyze_expr};
pub use callable::{CallableSummary, SummaryEffects};
pub use core_source::NativeReturnShape;
pub(crate) use dispatch::{DispatchReceiver, DispatchResolver};
pub use facts::{
    Confidence, FactOrigin, FieldEvidence, FieldEvidenceKind, FieldFacts, FileRevision, InferredValue, LocalFacts, MAX_SHAPE_UNION, ParameterFacts, ValueShape,
};
pub use flow::join_values;
pub use ids::{CORE_MODULE_URI, CallableId, ClassId, DispatchSide, FieldId, ModuleId};
pub use invalidation::InvalidationQueue;
pub use module_graph::{ImportEdge, ModuleGraph};
pub use occurrence::{OccurrenceIndex, OccurrenceRole, SemanticOccurrence, SemanticOccurrenceKind, SemanticTarget};
pub use query::{SemanticGeneration, SnapshotStamp};
pub use scope::{BindingId, BindingInfo, NameResolution, ScopeGraph, ScopeId, ScopeInfo, SemanticBindingKind};
pub use surface::{ClassSurface, FieldKind, FieldSurface, MemberKind, MemberSurface, MemberVisibility, ModuleSurface, ParamSurface, build_module_surface};

/// Renders one advisory runtime shape for editor surfaces.
pub fn render_value_shape(shape: &ValueShape) -> String {
    match shape {
        ValueShape::Unknown => "?".to_string(),
        ValueShape::Instance(class) => class.name.clone(),
        ValueShape::ClassObject(class) => format!("{} class", class.name),
        ValueShape::Module(module) => module.to_string(),
        ValueShape::Tuple(elements) => format!("({})", elements.iter().map(render_value_shape).collect::<Vec<_>>().join(", ")),
        ValueShape::Record(fields) => format!(
            "#{{{}}}",
            fields
                .iter()
                .map(|(label, value)| format!("{label}: {}", render_value_shape(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ValueShape::List(element) => format!("List<{}>", render_value_shape(element)),
        ValueShape::Set(element) => format!("Set<{}>", render_value_shape(element)),
        ValueShape::Map { key, value } => format!("Map<{}, {}>", render_value_shape(key), render_value_shape(value)),
        ValueShape::Range(element) => format!("Range<{}>", render_value_shape(element)),
        ValueShape::Callable(_) => "Callable".to_string(),
        ValueShape::Family { base, .. } => format!("Family<{base}>"),
        ValueShape::Union(alternatives) => alternatives.iter().map(render_value_shape).collect::<Vec<_>>().join(" | "),
    }
}

/// Renders confidence as stable editor prose.
pub fn confidence_name(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::Exact => "exact",
        Confidence::Flow => "flow",
        Confidence::Interprocedural => "interprocedural",
        Confidence::Heuristic => "heuristic",
    }
}

/// One member candidate returned by the live semantic surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionMember {
    /// Canonical comma-form selector.
    pub selector: String,
    /// Source member category.
    pub kind: MemberKind,
    /// Defining class identity.
    pub owner: ClassId,
    /// Source/runtime visibility.
    pub visibility: MemberVisibility,
    /// Dispatch side.
    pub side: DispatchSide,
}

/// A complete semantic contribution from one source file.
#[derive(Clone, Debug)]
pub struct FileSemanticSnapshot {
    /// Monotonic file revision.
    pub revision: FileRevision,
    /// Module identity.
    pub module: ModuleId,
    /// Recovered source program retained for dependent recomputation.
    pub program: Arc<Program>,
    /// Source-authored class/member surface.
    pub surface: ModuleSurface,
    /// Lexical binding and scope identities.
    pub scopes: ScopeGraph,
    /// Exact source semantic occurrences.
    pub occurrences: OccurrenceIndex,
    /// Exact and local-flow facts.
    pub local_facts: LocalFacts,
    /// Constructor-assigned field facts.
    pub field_facts: FieldFacts,
    /// Call-site facts observed for source callable parameters.
    pub parameter_facts: ParameterFacts,
    /// Resolved module dependencies.
    pub dependencies: DependencySet,
}

/// Dependencies extracted from one module's imports.
#[derive(Clone, Debug, Default)]
pub struct DependencySet {
    /// Resolved imported modules. Unresolved imports are retained in the graph
    /// but absent from this resolved dependency list.
    pub imports: Vec<ModuleId>,
}

#[derive(Default)]
struct SemanticState {
    generation: SemanticGeneration,
    files: BTreeMap<ModuleId, FileSemanticSnapshot>,
    classes: BTreeMap<ClassId, ClassSurface>,
    summaries: BTreeMap<CallableId, CallableSummary>,
    field_facts: BTreeMap<FieldId, InferredValue>,
    parameter_facts: BTreeMap<(CallableId, String), InferredValue>,
    parameter_contributions: BTreeMap<ModuleId, ParameterFacts>,
    callable_dependents: BTreeMap<CallableId, std::collections::BTreeSet<CallableId>>,
    graph: ModuleGraph,
    #[cfg(test)]
    last_trace: Option<RebuildTrace>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RebuildTraceData {
    modules_recomputed: BTreeSet<ModuleId>,
    callables_recomputed: BTreeSet<CallableId>,
}

#[cfg(test)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// Test-visible record of the semantic frontier processed by the last update.
pub struct RebuildTrace {
    /// Modules whose source facts were recomputed.
    pub modules_recomputed: BTreeSet<ModuleId>,
    /// Callable identities visited while recomputing the frontier.
    pub callables_recomputed: BTreeSet<CallableId>,
}

#[cfg(test)]
impl From<RebuildTraceData> for RebuildTrace {
    fn from(trace: RebuildTraceData) -> Self {
        Self {
            modules_recomputed: trace.modules_recomputed,
            callables_recomputed: trace.callables_recomputed,
        }
    }
}

/// Thread-safe semantic state owned by [`crate::backend::Backend`].
#[derive(Default)]
pub struct SemanticDb {
    state: RwLock<SemanticState>,
}

impl SemanticDb {
    /// Creates an empty semantic database.
    pub fn new() -> Self {
        let db = Self::default();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        // Bundled core is the initial baseline, not a workspace mutation.
        db.state.write().expect("semantic database lock poisoned").generation = SemanticGeneration(0);
        db
    }

    /// Replaces one file contribution and publishes one coherent generation.
    pub fn update_file(&self, uri: &Url, revision: FileRevision, program: &Program) -> SemanticGeneration {
        self.update_files_batch(vec![(uri.clone(), revision, program.clone())])
    }

    /// Replaces several file contributions and publishes one coherent generation.
    pub fn update_files_batch(&self, files: Vec<(Url, FileRevision, Program)>) -> SemanticGeneration {
        if files.is_empty() {
            return self.generation();
        }
        let mut state = self.state.write().expect("semantic database lock poisoned");
        let next_generation = SemanticGeneration(state.generation.0 + 1);

        let mut affected = BTreeSet::new();
        for (uri, _, _) in &files {
            let module = ModuleId::from_uri(uri);
            affected.insert(module.clone());
            affected.extend(state.graph.dependent_closure(&module));
            let old_callables = state.summaries.keys().filter(|id| id.owner.module == module).cloned().collect::<Vec<_>>();
            for callable in old_callables {
                if let Some(dependents) = state.callable_dependents.get(&callable) {
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
            let scopes = scope::build_scope_graph(module.clone(), &program);
            let occurrences = occurrence::build_occurrence_index(module.clone(), &program, &surface, &scopes);
            state.parameter_contributions.remove(&module);
            state.files.insert(
                module.clone(),
                FileSemanticSnapshot {
                    revision,
                    module: module.clone(),
                    program: Arc::new(program),
                    surface,
                    scopes,
                    occurrences,
                    local_facts: LocalFacts::default(),
                    field_facts: FieldFacts::default(),
                    parameter_facts: ParameterFacts::default(),
                    dependencies: DependencySet::default(),
                },
            );
        }

        let available = state.files.keys().cloned().collect::<BTreeSet<_>>();
        for module in updated_modules {
            if let Some(program) = state.files.get(&module).map(|file| file.program.clone()) {
                state.graph.update(module, &program, &available);
            }
        }
        let changed_importers = state.graph.refresh_resolutions(&available);
        affected.extend(changed_importers);
        let current = affected.clone();
        for module in current {
            affected.extend(state.graph.dependent_closure(&module));
        }

        let trace = rebuild_affected_state(&mut state, next_generation, affected);
        state.generation = next_generation;
        #[cfg(test)]
        {
            state.last_trace = Some(trace.into());
        }
        #[cfg(not(test))]
        drop(trace);
        state.generation
    }

    /// Replaces the live source-authored core module while keeping its stable
    /// semantic identity. Workspace/open-buffer callers use this instead of
    /// publishing a file-qualified duplicate core namespace.
    pub fn update_core(&self, revision: FileRevision, program: &Program) -> SemanticGeneration {
        let uri = Url::parse(CORE_MODULE_URI).expect("core semantic URI must be valid");
        self.update_file(&uri, revision, program)
    }

    /// Removes one file contribution and publishes one coherent generation.
    pub fn remove_file(&self, uri: &Url) -> SemanticGeneration {
        let module = ModuleId::from_uri(uri);
        let mut state = self.state.write().expect("semantic database lock poisoned");
        let mut affected = BTreeSet::from([module.clone()]);
        affected.extend(state.graph.dependent_closure(&module));
        let old_callables = state.summaries.keys().filter(|id| id.owner.module == module).cloned().collect::<Vec<_>>();
        for callable in old_callables {
            if let Some(dependents) = state.callable_dependents.get(&callable) {
                affected.extend(dependents.iter().map(|dependent| dependent.owner.module.clone()));
            }
        }
        state.files.remove(&module);
        state.parameter_contributions.remove(&module);
        state.field_facts.retain(|field, _| field.owner.module != module);
        state.graph.remove(&module);
        let available = state.files.keys().cloned().collect::<BTreeSet<_>>();
        let changed_importers = state.graph.refresh_resolutions(&available);
        affected.extend(changed_importers);
        let current = affected.clone();
        for changed in current {
            affected.extend(state.graph.dependent_closure(&changed));
        }
        let next_generation = SemanticGeneration(state.generation.0 + 1);
        let trace = rebuild_affected_state(&mut state, next_generation, affected);
        state.generation = next_generation;
        #[cfg(test)]
        {
            state.last_trace = Some(trace.into());
        }
        #[cfg(not(test))]
        drop(trace);
        state.generation
    }

    #[cfg(test)]
    /// Returns the last semantic rebuild frontier.
    pub fn last_rebuild_trace(&self) -> Option<RebuildTrace> {
        self.state.read().expect("semantic database lock poisoned").last_trace.clone()
    }

    /// Returns the current semantic generation.
    pub fn generation(&self) -> SemanticGeneration {
        self.state.read().expect("semantic database lock poisoned").generation
    }

    /// Returns an immutable clone of one file's semantic snapshot.
    pub fn file_snapshot(&self, uri: &Url) -> Option<FileSemanticSnapshot> {
        let module = ModuleId::from_uri(uri);
        self.state.read().expect("semantic database lock poisoned").files.get(&module).cloned()
    }

    /// Returns the exact semantic occurrence covering one source offset.
    pub fn occurrence_at(&self, uri: &Url, offset: usize) -> Option<SemanticOccurrence> {
        let module = ModuleId::from_uri(uri);
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .files
            .get(&module)
            .and_then(|file| file.occurrences.occurrence_at(offset).cloned())
    }

    /// Returns all references to a SemanticTarget in the workspace.
    /// If the target is a file-local binding, only searches the given file.
    pub fn references_for_target(&self, uri: &Url, target: &SemanticTarget) -> Vec<(Url, SourceRange, OccurrenceRole)> {
        let state = self.state.read().expect("semantic database lock poisoned");
        match target {
            SemanticTarget::Binding(_) => {
                let module = ModuleId::from_uri(uri);
                state.files
                    .get(&module)
                    .map(|file| {
                        file.occurrences
                            .all()
                            .iter()
                            .filter(|occ| &occ.target == target)
                            .map(|occ| (uri.clone(), occ.range, occ.role))
                            .collect()
                    })
                    .unwrap_or_default()
            }
            _ => {
                let mut results = Vec::new();
                for (module_id, file) in &state.files {
                    if let Ok(file_uri) = Url::parse(module_id.as_str()) {
                        for occ in file.occurrences.all() {
                            if &occ.target == target {
                                results.push((file_uri.clone(), occ.range, occ.role));
                            }
                        }
                    }
                }
                results
            }
        }
    }

    /// Returns lexical bindings visible at one source offset, nearest scope first.
    pub fn visible_bindings_at(&self, uri: &Url, offset: usize) -> Vec<BindingInfo> {
        let module = ModuleId::from_uri(uri);
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .files
            .get(&module)
            .map(|file| file.scopes.visible_bindings_at(offset))
            .unwrap_or_default()
    }

    /// Returns one binding's declaration metadata from a file-local identity.
    pub fn binding_info(&self, uri: &Url, binding: BindingId) -> Option<BindingInfo> {
        let module = ModuleId::from_uri(uri);
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .files
            .get(&module)
            .and_then(|file| file.scopes.bindings.get(&binding).cloned())
    }

    /// Returns one class surface by module-qualified identity.
    pub fn class_surface(&self, id: &ClassId) -> Option<ClassSurface> {
        self.state.read().expect("semantic database lock poisoned").classes.get(id).cloned()
    }

    /// Returns one module-qualified member surface by canonical selector.
    pub fn member_surface(&self, class: &ClassId, selector: &str) -> Option<MemberSurface> {
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .classes
            .get(class)
            .and_then(|surface| surface.members.get(selector))
            .cloned()
    }

    /// Resolves one receiver-qualified member, including inherited members.
    pub fn receiver_member(&self, class: &ClassId, selector: &str, side: DispatchSide) -> Option<MemberSurface> {
        let state = self.state.read().expect("semantic database lock poisoned");
        let receiver = match side {
            DispatchSide::Instance => DispatchReceiver::Instance(class.clone()),
            DispatchSide::Class => DispatchReceiver::ClassObject(class.clone()),
        };
        DispatchResolver::new(&state.classes)
            .resolve(&receiver, selector)
            .map(|resolved| resolved.member)
    }

    /// Returns inherited, de-duplicated members for one live class surface.
    pub fn completion_members(&self, class: &ClassId, side: DispatchSide) -> Vec<CompletionMember> {
        let state = self.state.read().expect("semantic database lock poisoned");
        let mut current = Some(class.clone());
        let mut seen = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        let mut members = Vec::new();
        while let Some(id) = current.take() {
            if !visited.insert(id.clone()) {
                break;
            }
            let Some(surface) = state.classes.get(&id) else { break };
            for ((selector, member_side), member) in &surface.members_by_side {
                if *member_side == side && seen.insert(selector.clone()) {
                    members.push(CompletionMember {
                        selector: selector.clone(),
                        kind: member.kind,
                        owner: id.clone(),
                        visibility: member.visibility,
                        side,
                    });
                }
            }
            current = surface
                .superclass
                .clone()
                .or_else(|| (id.name != "Object").then(|| ClassId::new(ModuleId::new(CORE_MODULE_URI), "Object")));
        }
        members.sort_by(|left, right| left.selector.cmp(&right.selector));
        members
    }

    /// Returns every declared live member, de-duplicated by selector.
    pub fn all_completion_members(&self) -> Vec<CompletionMember> {
        let state = self.state.read().expect("semantic database lock poisoned");
        let mut members = BTreeMap::new();
        for (class_id, surface) in &state.classes {
            for (selector, member) in &surface.members_by_side {
                let selector = &selector.0;
                members.entry(selector.clone()).or_insert_with(|| CompletionMember {
                    selector: selector.clone(),
                    kind: member.kind,
                    owner: class_id.clone(),
                    visibility: member.visibility,
                    side: member.side,
                });
            }
        }
        members.into_values().collect()
    }

    /// Tests module-qualified class ancestry for visibility filtering.
    pub fn is_same_or_subclass(&self, child: &ClassId, ancestor: &ClassId) -> bool {
        let state = self.state.read().expect("semantic database lock poisoned");
        let mut current = Some(child.clone());
        let mut visited = std::collections::BTreeSet::new();
        while let Some(id) = current.take() {
            if !visited.insert(id.clone()) {
                return false;
            }
            if &id == ancestor {
                return true;
            }
            let Some(surface) = state.classes.get(&id) else { return false };
            current = surface
                .superclass
                .clone()
                .or_else(|| (id.name != "Object").then(|| ClassId::new(ModuleId::new(CORE_MODULE_URI), "Object")));
        }
        false
    }

    /// Returns a source callable summary from the current semantic generation.
    pub fn callable_summary(&self, id: &CallableId) -> Option<CallableSummary> {
        self.state.read().expect("semantic database lock poisoned").summaries.get(id).cloned()
    }

    /// Returns a callable's target-specific return summary.
    pub fn return_for_callable(&self, id: &CallableId) -> Option<InferredValue> {
        let state = self.state.read().expect("semantic database lock poisoned");
        return_for_callable(&state.classes, &state.summaries, id)
    }

    /// Returns the joined call-site fact observed for one callable parameter.
    pub fn parameter_at(&self, id: &CallableId, name: &str) -> Option<InferredValue> {
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .parameter_facts
            .get(&(id.clone(), name.to_string()))
            .cloned()
    }

    /// Resolves a class name in its module, with the stable core namespace as
    /// a fallback for primitive/runtime classes.
    pub fn class_for_name(&self, uri: &Url, name: &str) -> Option<ClassId> {
        let module = ModuleId::from_uri(uri);
        let state = self.state.read().expect("semantic database lock poisoned");
        resolve_named_class(&state.classes, &state.graph, &module, name)
    }

    /// Returns the class whose declaration contains a byte offset in `uri`.
    pub fn class_at(&self, uri: &Url, offset: usize) -> Option<ClassId> {
        let module = ModuleId::from_uri(uri);
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .files
            .get(&module)?
            .surface
            .classes
            .values()
            .find(|class| class.source_range.contains(offset))
            .map(|class| class.id.clone())
    }

    /// Returns the source-authored class whose name range contains `offset`.
    pub fn class_name_at(&self, uri: &Url, offset: usize) -> Option<ClassSurface> {
        let module = ModuleId::from_uri(uri);
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .files
            .get(&module)?
            .surface
            .classes
            .values()
            .find(|class| class.name_range.contains(offset))
            .cloned()
    }

    /// Returns the declared callable enclosing a source offset.
    pub fn member_at(&self, uri: &Url, offset: usize) -> Option<MemberSurface> {
        let module = ModuleId::from_uri(uri);
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .files
            .get(&module)?
            .surface
            .classes
            .values()
            .flat_map(|class| class.members_by_side.values())
            .find(|member| member.source_range.contains(offset))
            .cloned()
    }

    /// Joins return summaries for a bounded set of receiver candidates.
    pub fn returns_for_callables(&self, ids: impl IntoIterator<Item = CallableId>) -> Option<InferredValue> {
        let state = self.state.read().expect("semantic database lock poisoned");
        ids.into_iter()
            .filter_map(|id| return_for_callable(&state.classes, &state.summaries, &id))
            .reduce(|left, right| left.join(&right))
    }

    /// Returns the fact visible for a local binding at a byte offset.
    pub fn binding_at(&self, uri: &Url, name: &str, offset: usize) -> Option<InferredValue> {
        let module = ModuleId::from_uri(uri);
        let state = self.state.read().expect("semantic database lock poisoned");
        let file = state.files.get(&module)?;
        let binding = match file.scopes.resolve(file.scopes.scope_at(offset), name, offset) {
            NameResolution::Binding(binding) => binding,
            _ => return None,
        };
        file.local_facts.value_before(binding, offset).cloned()
    }

    /// Infers a parsed receiver expression against the coherent current
    /// semantic generation.
    pub fn infer_expression(&self, uri: &Url, expr: &phalcom_ast::ast::Expr, offset: usize) -> InferredValue {
        let module = ModuleId::from_uri(uri);
        let state = self.state.read().expect("semantic database lock poisoned");
        let mut environment = BTreeMap::new();
        let known_classes = |name: &str| resolve_named_class(&state.classes, &state.graph, &module, name);
        let callable_return = |id: &CallableId| return_for_callable(&state.classes, &state.summaries, id);
        let field_value = |class: &ClassId, name: &str, side: DispatchSide| {
            let mut current = Some(class.clone());
            let mut seen = BTreeSet::new();
            while let Some(owner) = current {
                if !seen.insert(owner.clone()) {
                    break;
                }
                if let Some(value) = state.field_facts.get(&FieldId {
                    owner: owner.clone(),
                    name: name.to_string(),
                    side,
                }) {
                    return Some(value.clone());
                }
                current = state.classes.get(&owner).and_then(|surface| surface.superclass.clone());
            }
            None
        };
        let current_class = state
            .files
            .get(&module)
            .and_then(|file| file.surface.classes.values().find(|class| class.source_range.contains(offset)))
            .map(|class| class.id.clone());
        if let Some(class) = current_class.as_ref() {
            if let Some(file) = state.files.get(&module) {
                if let Some(member) = file
                    .surface
                    .classes
                    .get(class)
                    .and_then(|class| class.members_by_side.values().find(|member| member.source_range.contains(offset)))
                {
                    for param in &member.params {
                        if let Some(value) = state.parameter_facts.get(&(member.callable.clone(), param.name.clone())) {
                            environment.insert(param.name.clone(), value.clone());
                        }
                    }
                }
            }
        }
        let dispatch_side = current_class.as_ref().and_then(|class| {
            state
                .files
                .get(&module)
                .and_then(|file| file.surface.classes.get(class))
                .and_then(|surface| surface.members_by_side.values().find(|member| member.source_range.contains(offset)))
                .map(|member| member.side)
        });
        let local_facts = state.files.get(&module).map(|file| &file.local_facts);
        let scopes = state.files.get(&module).map(|file| &file.scopes);
        let resolver = DispatchResolver::new(&state.classes);
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
        let contains_class = |class: &ClassId| resolver.contains_class(class);
        let context = AnalysisContext {
            current_class: current_class.as_ref(),
            dispatch_side,
            query_offset: offset,
            environment: &environment,
            local_facts,
            binding_values: None,
            scopes,
            known_class: &known_classes,
            callable_return: &callable_return,
            field_value: &field_value,
            resolver: &resolve_member,
            contains_class: &contains_class,
        };
        let inferred = analyze_expr(expr, &context);
        inferred
    }

    /// Returns current import edges for one module.
    pub fn imports(&self, uri: &Url) -> Vec<ImportEdge> {
        let module = ModuleId::from_uri(uri);
        self.state.read().expect("semantic database lock poisoned").graph.imports(&module).to_vec()
    }

    /// Returns a coherent revision/generation stamp for one file.
    pub fn stamp(&self, uri: &Url) -> Option<SnapshotStamp> {
        let module = ModuleId::from_uri(uri);
        let state = self.state.read().expect("semantic database lock poisoned");
        Some(SnapshotStamp {
            revision: state.files.get(&module)?.revision,
            generation: state.generation,
        })
    }
}

fn rebuild_affected_state(state: &mut SemanticState, generation: SemanticGeneration, mut affected: BTreeSet<ModuleId>) -> RebuildTraceData {
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

fn resolve_named_class(classes: &BTreeMap<ClassId, ClassSurface>, graph: &ModuleGraph, module: &ModuleId, name: &str) -> Option<ClassId> {
    if let Some((binding, class_name)) = name.split_once('.') {
        let imported = graph
            .imports(module)
            .iter()
            .find(|edge| edge.binding == binding)
            .and_then(|edge| edge.target.as_ref())?;
        let class = ClassId::new(imported.clone(), class_name);
        return classes.contains_key(&class).then_some(class);
    }
    let local = ClassId::new(module.clone(), name);
    if classes.contains_key(&local) {
        return Some(local);
    }
    let core = ClassId::new(ModuleId::new(CORE_MODULE_URI), name);
    classes.contains_key(&core).then_some(core)
}

fn return_for_callable(classes: &BTreeMap<ClassId, ClassSurface>, summaries: &BTreeMap<CallableId, CallableSummary>, id: &CallableId) -> Option<InferredValue> {
    let class = classes.get(&id.owner)?;
    let member = class.members_by_side.get(&(id.selector.clone(), id.side));
    if id.side == DispatchSide::Class && (id.selector == "new()" || member.is_some_and(|member| member.is_constructor)) {
        return Some(InferredValue::flow(ValueShape::Instance(id.owner.clone()), Default::default()));
    }
    if let Some(summary) = summaries.get(id) {
        return Some(summary.returns.clone());
    }
    let member = member?;
    let shape = match member.native_return? {
        NativeReturnShape::Unknown | NativeReturnShape::Argument(_) => ValueShape::Unknown,
        NativeReturnShape::Instance(name) => ValueShape::Instance(ClassId::new(ModuleId::new(CORE_MODULE_URI), name)),
        NativeReturnShape::ClassObject(name) => ValueShape::ClassObject(ClassId::new(ModuleId::new(CORE_MODULE_URI), name)),
        NativeReturnShape::Receiver => match id.side {
            DispatchSide::Instance => ValueShape::Instance(id.owner.clone()),
            DispatchSide::Class => ValueShape::ClassObject(id.owner.clone()),
        },
    };
    Some(InferredValue::flow(shape, Default::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;

    fn uri(value: &str) -> Url {
        Url::parse(value).unwrap()
    }

    #[test]
    fn update_publishes_revisioned_local_facts() {
        let db = SemanticDb::new();
        let uri = uri("file:///main.ph");
        let parse = parse("let text = \"hello\"\n", 0);
        let generation = db.update_file(&uri, FileRevision(7), &parse.program);
        assert_eq!(generation.0, 1);
        assert_eq!(db.file_snapshot(&uri).unwrap().revision, FileRevision(7));
        assert!(matches!(db.binding_at(&uri, "text", 20).unwrap().shape, ValueShape::Instance(ClassId { name, .. }) if name == "String"));
    }

    #[test]
    fn same_named_classes_are_isolated() {
        let db = SemanticDb::new();
        let one = uri("file:///one.ph");
        let two = uri("file:///two.ph");
        let parse = parse("class Point { move() { } }", 0);
        db.update_file(&one, FileRevision(1), &parse.program);
        db.update_file(&two, FileRevision(1), &parse.program);
        assert!(db.class_surface(&ClassId::new(ModuleId::from_uri(&one), "Point")).is_some());
        assert!(db.class_surface(&ClassId::new(ModuleId::from_uri(&two), "Point")).is_some());
        assert_ne!(ModuleId::from_uri(&one), ModuleId::from_uri(&two));
    }

    #[test]
    fn callable_summary_tracks_constructor_return() {
        let db = SemanticDb::new();
        let uri = uri("file:///factory.ph");
        let parse = parse("class Point { @constructor new() { } }\nclass Factory { make() { Point.new() } }\n", 0);
        db.update_file(&uri, FileRevision(1), &parse.program);
        let summary = db
            .callable_summary(&CallableId {
                owner: ClassId::new(ModuleId::from_uri(&uri), "Factory"),
                selector: "make()".to_string(),
                side: DispatchSide::Instance,
            })
            .unwrap();
        assert!(matches!(summary.returns.shape, ValueShape::Instance(ClassId { name, .. }) if name == "Point"));
    }

    #[test]
    fn bundled_core_source_is_queryable_without_core_table() {
        let db = SemanticDb::new();
        let string = ClassId::new(ModuleId::new(CORE_MODULE_URI), "String");
        let members = db.completion_members(&string, DispatchSide::Instance);
        assert!(members.iter().any(|member| member.selector == "size"));
        assert!(members.iter().any(|member| member.selector == "hash"));
    }

    #[test]
    fn live_core_replacement_updates_semantic_surface() {
        let db = SemanticDb::new();
        let parse = parse("class String { liveEditorMember() { } }", 0);
        db.update_core(FileRevision(2), &parse.program);
        let string = ClassId::new(ModuleId::new(CORE_MODULE_URI), "String");
        assert!(
            db.completion_members(&string, DispatchSide::Instance)
                .iter()
                .any(|member| member.selector == "liveEditorMember()")
        );
        assert!(
            !db.completion_members(&string, DispatchSide::Instance)
                .iter()
                .any(|member| member.selector == "size")
        );
    }

    #[test]
    fn explicit_receiver_expression_uses_callable_return_summary() {
        let db = SemanticDb::new();
        let uri = uri("file:///factory.ph");
        let parsed = parse(
            "class Point { @constructor new() { } }\nclass Factory { @constructor new() { } make() { Point.new() } }\nlet factory = Factory.new()\n",
            0,
        );
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let receiver = parse("factory.make()", 0)
            .program
            .statements
            .into_iter()
            .find_map(|statement| match statement {
                phalcom_ast::ast::Statement::Expr { expr, .. } => Some(expr),
                _ => None,
            })
            .unwrap();
        let value = db.infer_expression(&uri, &receiver, 200);
        assert!(matches!(value.shape, ValueShape::Instance(ClassId { name, .. }) if name == "Point"));
    }

    #[test]
    fn open_method_reference_invokes_against_call_site_selector() {
        let db = SemanticDb::new();
        let uri = uri("file:///family.ph");
        let source = "class Box { @constructor new() { } value() { 1 } }\nlet family = Box.new()::value\n";
        let parsed = parse(source, 0);
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let family_binding = db
            .binding_at(&uri, "family", source.find("family").expect("family binding offset") + 1)
            .expect("family binding");
        assert_eq!(
            family_binding.shape,
            ValueShape::Family {
                receiver: Box::new(ValueShape::Instance(ClassId::new(ModuleId::from_uri(&uri), "Box"))),
                base: "value".to_string(),
            }
        );
        let value_callable = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&uri), "Box"),
            selector: "value()".to_string(),
            side: DispatchSide::Instance,
        };
        assert_eq!(
            db.return_for_callable(&value_callable).map(|value| value.shape),
            Some(ValueShape::Instance(ClassId::new(ModuleId::new(CORE_MODULE_URI), "Int")))
        );
        let expression = parse("family()", 0)
            .program
            .statements
            .into_iter()
            .find_map(|statement| match statement {
                phalcom_ast::ast::Statement::Expr { expr, .. } => Some(expr),
                _ => None,
            })
            .expect("expression statement");
        let value = db.infer_expression(&uri, &expression, source.len());
        assert_eq!(value.shape, ValueShape::Instance(ClassId::new(ModuleId::new(CORE_MODULE_URI), "Int")));
    }

    #[test]
    fn direct_expression_inference_uses_native_return_contracts() {
        let db = SemanticDb::new();
        let uri = uri("file:///native-contract.ph");
        let expression = parse("1 < 2", 0)
            .program
            .statements
            .into_iter()
            .find_map(|statement| match statement {
                phalcom_ast::ast::Statement::Expr { expr, .. } => Some(expr),
                _ => None,
            })
            .expect("expression statement");

        let value = db.infer_expression(&uri, &expression, 100);
        assert_eq!(value.shape, ValueShape::Instance(ClassId::new(ModuleId::new(CORE_MODULE_URI), "Bool")));
    }

    #[test]
    fn field_expression_uses_constructor_assignment_fact() {
        let db = SemanticDb::new();
        let uri = uri("file:///service.ph");
        let parsed = parse(
            "class Client { send() { } }\nclass Service { @constructor new() { _client = Client.new() } run() { _client } }\n",
            0,
        );
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let field = parse("_client", 0)
            .program
            .statements
            .into_iter()
            .find_map(|statement| match statement {
                phalcom_ast::ast::Statement::Expr { expr, .. } => Some(expr),
                _ => None,
            })
            .unwrap();
        let value = db.infer_expression(&uri, &field, 100);
        assert!(matches!(value.shape, ValueShape::Instance(ClassId { name, .. }) if name == "Client"));
    }

    #[test]
    fn inherited_field_read_uses_defining_field_fact() {
        let db = SemanticDb::new();
        let uri = uri("file:///inherited-field.ph");
        let source = "class Client { send() { } }
class Base { const _client = Client.new() }
class Child is Base { run() { _client } }
";
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "field parse errors: {:?}", parsed.errors);
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let field = parse("_client", 0)
            .program
            .statements
            .into_iter()
            .find_map(|statement| match statement {
                phalcom_ast::ast::Statement::Expr { expr, .. } => Some(expr),
                _ => None,
            })
            .unwrap();
        let value = db.infer_expression(&uri, &field, source.rfind("_client").unwrap());
        assert!(matches!(value.shape, ValueShape::Instance(ClassId { name, .. }) if name == "Client"));
    }

    #[test]
    fn parameter_expression_uses_resolved_call_site_fact() {
        let db = SemanticDb::new();
        let uri = uri("file:///canvas.ph");
        let parsed = parse(
            "class Circle { stroke() { } }\nclass Canvas { draw(_ shape) { shape } }\ndraw(Circle.new())\n",
            0,
        );
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let callable = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&uri), "Canvas"),
            selector: "draw(_)".to_string(),
            side: DispatchSide::Instance,
        };
        assert!(matches!(db.parameter_at(&callable, "shape").unwrap().shape, ValueShape::Instance(ClassId { name, .. }) if name == "Circle"));
        let expression = parse("shape", 0)
            .program
            .statements
            .into_iter()
            .find_map(|statement| match statement {
                phalcom_ast::ast::Statement::Expr { expr, .. } => Some(expr),
                _ => None,
            })
            .unwrap();
        let value = db.infer_expression(&uri, &expression, 55);
        assert!(matches!(value.shape, ValueShape::Instance(ClassId { name, .. }) if name == "Circle"));
    }

    #[test]
    fn recursive_callable_summaries_terminate_at_unknown() {
        let db = SemanticDb::new();
        let uri = uri("file:///recursive.ph");
        let parsed = parse("class Loop { loop() { loop() } }", 0);
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let callable = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&uri), "Loop"),
            selector: "loop()".to_string(),
            side: DispatchSide::Instance,
        };
        assert_eq!(db.return_for_callable(&callable).unwrap().shape, ValueShape::Unknown);
    }

    #[test]
    fn mutually_recursive_callable_summaries_terminate_at_unknown() {
        let db = SemanticDb::new();
        let uri = uri("file:///mutual-recursive.ph");
        let parsed = parse("class Loop { first() { second() } second() { first() } }", 0);
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let callable = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&uri), "Loop"),
            selector: "first()".to_string(),
            side: DispatchSide::Instance,
        };
        assert_eq!(db.return_for_callable(&callable).unwrap().shape, ValueShape::Unknown);
    }

    #[test]
    fn explicit_multiple_returns_join_into_a_bounded_union() {
        let db = SemanticDb::new();
        let uri = uri("file:///returns.ph");
        let parsed = parse(
            "class A { @constructor new() { } }\nclass B { @constructor new() { } }\nclass Factory { choose() { return A.new()\nreturn B.new() } }",
            0,
        );
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let callable = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&uri), "Factory"),
            selector: "choose()".to_string(),
            side: DispatchSide::Instance,
        };
        assert!(matches!(db.return_for_callable(&callable).unwrap().shape, ValueShape::Union(_)));
    }

    #[test]
    fn invoked_literal_block_contributes_nonlocal_return() {
        let db = SemanticDb::new();
        let uri = uri("file:///block-flow.ph");
        let parsed = parse(
            "class Product { @constructor new() { } }
class Factory { choose() { true.ifTrue() || { return Product.new() } } escaped() { self.consume() || { return Product.new() } } }
",
            0,
        );
        assert!(parsed.errors.is_empty(), "block parse errors: {:?}", parsed.errors);
        db.update_file(&uri, FileRevision(1), &parsed.program);

        let factory = ClassId::new(ModuleId::from_uri(&uri), "Factory");
        let choose = db
            .return_for_callable(&CallableId {
                owner: factory.clone(),
                selector: "choose()".to_string(),
                side: DispatchSide::Instance,
            })
            .unwrap();
        assert!(matches!(choose.shape, ValueShape::Instance(ClassId { name, .. }) if name == "Product"));

        let escaped = db
            .return_for_callable(&CallableId {
                owner: factory,
                selector: "escaped()".to_string(),
                side: DispatchSide::Instance,
            })
            .unwrap();
        assert_eq!(escaped.shape, ValueShape::Unknown);
    }

    #[test]
    fn arbitrary_higher_order_call_propagates_literal_block_effects() {
        let db = SemanticDb::new();
        let uri = uri("file:///higher-order-flow.ph");
        let parsed = parse(
            "class Product { @constructor new() { } }\nclass Factory { consume(_ block) { block() } forward(_ block) { self.consume(block) } choose() { self.forward { return Product.new() } } }\n",
            0,
        );
        assert!(parsed.errors.is_empty(), "higher-order parse errors: {:?}", parsed.errors);
        db.update_file(&uri, FileRevision(1), &parsed.program);

        let summary = db
            .return_for_callable(&CallableId {
                owner: ClassId::new(ModuleId::from_uri(&uri), "Factory"),
                selector: "choose()".to_string(),
                side: DispatchSide::Instance,
            })
            .expect("higher-order summary");
        assert!(matches!(summary.shape, ValueShape::Instance(ClassId { name, .. }) if name == "Product"));
    }

    #[test]
    fn escaped_block_effects_do_not_change_outer_flow() {
        let db = SemanticDb::new();
        let uri = uri("file:///escaped-block-flow.ph");
        let source = "class Product { @constructor new() { } }\nclass Factory { store(_ block) { 1 } choose() {\nlet result = 1\nself.store { result = Product.new() }\nresult\n} }\n";
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "escaped block parse errors: {:?}", parsed.errors);
        db.update_file(&uri, FileRevision(1), &parsed.program);

        let factory = ClassId::new(ModuleId::from_uri(&uri), "Factory");
        let summary = db
            .return_for_callable(&CallableId {
                owner: factory,
                selector: "choose()".to_string(),
                side: DispatchSide::Instance,
            })
            .expect("escaped block summary");
        assert!(matches!(summary.shape, ValueShape::Instance(ClassId { module, name }) if module == ModuleId::new(CORE_MODULE_URI) && name == "Int"));
        let result = db
            .binding_at(&uri, "result", source.rfind("result").expect("result use"))
            .expect("escaped block binding fact");
        assert!(matches!(result.shape, ValueShape::Instance(ClassId { module, name }) if module == ModuleId::new(CORE_MODULE_URI) && name == "Int"));
    }

    #[test]
    fn loop_fixpoint_propagates_continue_carried_writes() {
        let db = SemanticDb::new();
        let uri = uri("file:///loop-flow.ph");
        let source = "class Product { @constructor new() { } }\nclass Factory { choose(_ values) {\nlet result = 1\nfor (item in values) {\nresult = Product.new()\ncontinue\n}\nresult\n} }\n";
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "loop parse errors: {:?}", parsed.errors);
        db.update_file(&uri, FileRevision(1), &parsed.program);

        let summary = db
            .return_for_callable(&CallableId {
                owner: ClassId::new(ModuleId::from_uri(&uri), "Factory"),
                selector: "choose(_)".to_string(),
                side: DispatchSide::Instance,
            })
            .expect("loop summary");
        assert!(matches!(summary.shape, ValueShape::Union(_)), "shape: {:?}", summary.shape);
    }

    #[test]
    fn while_fixpoint_propagates_continue_carried_writes() {
        let db = SemanticDb::new();
        let uri = uri("file:///while-flow.ph");
        let parsed = parse(
            "class Product { @constructor new() { } }\nclass Factory { choose() {\nlet result = 1\nlet i = 0\n|| { i < 1 }.whileTrue || {\nresult = Product.new()\ni = i + 1\ncontinue\n}\nresult\n} }\n",
            0,
        );
        assert!(parsed.errors.is_empty(), "while parse errors: {:?}", parsed.errors);
        db.update_file(&uri, FileRevision(1), &parsed.program);

        let summary = db
            .return_for_callable(&CallableId {
                owner: ClassId::new(ModuleId::from_uri(&uri), "Factory"),
                selector: "choose()".to_string(),
                side: DispatchSide::Instance,
            })
            .expect("while summary");
        assert!(matches!(summary.shape, ValueShape::Union(_)), "shape: {:?}", summary.shape);
    }

    #[test]
    fn three_step_return_forwarding_converges() {
        let db = SemanticDb::new();
        let uri = uri("file:///chain.ph");
        let parsed = parse(
            "class Product { @constructor new() { } }\nclass Chain { a() { b() } b() { c() } c() { Product.new() } }",
            0,
        );
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let callable = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&uri), "Chain"),
            selector: "a()".to_string(),
            side: DispatchSide::Instance,
        };
        let shape = db.return_for_callable(&callable).unwrap().shape;
        assert!(
            matches!(&shape, ValueShape::Instance(ClassId { name, .. }) if name == "Product"),
            "shape: {shape:?}"
        );
    }

    #[test]
    fn recursive_scc_with_concrete_evidence_converges() {
        let db = SemanticDb::new();
        let uri = uri("file:///recursive-concrete.ph");
        let parsed = parse(
            "class Product { @constructor new() { } }\nclass Loop { run() { return run()\nreturn Product.new() } }",
            0,
        );
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let callable = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&uri), "Loop"),
            selector: "run()".to_string(),
            side: DispatchSide::Instance,
        };
        let shape = db.return_for_callable(&callable).unwrap().shape;
        assert!(
            matches!(&shape, ValueShape::Instance(ClassId { name, .. }) if name == "Product"),
            "shape: {shape:?}"
        );
    }

    #[test]
    fn nine_incompatible_return_shapes_widen_to_unknown() {
        let db = SemanticDb::new();
        let uri = uri("file:///wide.ph");
        let mut source = String::new();
        for name in ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I'] {
            source.push_str(&format!("class {name} {{ @constructor new() {{ }} }}\n"));
        }
        source.push_str("class Factory { choose() { return A.new()\nreturn B.new()\nreturn C.new()\nreturn D.new()\nreturn E.new()\nreturn F.new()\nreturn G.new()\nreturn H.new()\nreturn I.new() } }");
        let parsed = parse(&source, 0);
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let callable = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&uri), "Factory"),
            selector: "choose()".to_string(),
            side: DispatchSide::Instance,
        };
        assert_eq!(db.return_for_callable(&callable).unwrap().shape, ValueShape::Unknown);
    }

    #[test]
    fn same_selector_different_classes_have_independent_summaries() {
        let db = SemanticDb::new();
        let uri = uri("file:///same-selector.ph");
        let parsed = parse(
            "class AValue { @constructor new() { } }\nclass BValue { @constructor new() { } }\nclass A { value() { AValue.new() } }\nclass B { value() { BValue.new() } }",
            0,
        );
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let module = ModuleId::from_uri(&uri);
        let a = db
            .return_for_callable(&CallableId {
                owner: ClassId::new(module.clone(), "A"),
                selector: "value()".to_string(),
                side: DispatchSide::Instance,
            })
            .unwrap();
        let b = db
            .return_for_callable(&CallableId {
                owner: ClassId::new(module, "B"),
                selector: "value()".to_string(),
                side: DispatchSide::Instance,
            })
            .unwrap();
        assert!(matches!(a.shape, ValueShape::Instance(ClassId { name, .. }) if name == "AValue"));
        assert!(matches!(b.shape, ValueShape::Instance(ClassId { name, .. }) if name == "BValue"));
    }

    #[test]
    fn imported_callable_returns_and_parameters_propagate_across_modules() {
        let root = std::env::temp_dir().join(format!("phalcom-lsp-semantic-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&root);
        let provider_path = root.join("provider.ph");
        let consumer_path = root.join("consumer.ph");
        let provider_text = "class Product { run() { } }\nclass Factory { make() { Product.new() } }\nclass Service { consume(_ value) { value } }\n";
        let consumer_text = "import \"./provider\" as Provider\nlet product = Provider.Factory.new().make()\nlet consumed = Provider.Service.new().consume(Provider.Product.new())\n";
        std::fs::write(&provider_path, provider_text).unwrap();
        std::fs::write(&consumer_path, consumer_text).unwrap();
        let provider_uri = Url::from_file_path(&provider_path).unwrap();
        let consumer_uri = Url::from_file_path(&consumer_path).unwrap();
        let db = SemanticDb::new();
        db.update_file(&provider_uri, FileRevision(1), &parse(provider_text, 0).program);
        db.update_file(&consumer_uri, FileRevision(1), &parse(consumer_text, 0).program);

        let product = db.binding_at(&consumer_uri, "product", consumer_text.len()).unwrap();
        assert!(matches!(product.shape, ValueShape::Instance(ClassId { name, .. }) if name == "Product"));
        let service = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&provider_uri), "Service"),
            selector: "consume(_)".to_string(),
            side: DispatchSide::Instance,
        };
        assert!(matches!(db.parameter_at(&service, "value").unwrap().shape, ValueShape::Instance(ClassId { name, .. }) if name == "Product"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn leaf_edit_does_not_recompute_unrelated_module() {
        let db = SemanticDb::new();
        let left = uri("file:///left.ph");
        let right = uri("file:///right.ph");
        db.update_files_batch(vec![
            (left.clone(), FileRevision(1), parse("class Left { ping() { } }", 0).program),
            (right.clone(), FileRevision(1), parse("class Right { pong() { } }", 0).program),
        ]);
        db.update_file(&left, FileRevision(2), &parse("class Left { changed() { } }", 0).program);
        assert_eq!(db.last_rebuild_trace().unwrap().modules_recomputed, BTreeSet::from([ModuleId::from_uri(&left)]));
        assert!(!db.last_rebuild_trace().unwrap().modules_recomputed.contains(&ModuleId::from_uri(&right)));
    }

    #[test]
    fn provider_edit_recomputes_transitive_consumers() {
        let db = SemanticDb::new();
        let provider = uri("file:///provider.ph");
        let consumer = uri("file:///consumer.ph");
        db.update_files_batch(vec![
            (provider.clone(), FileRevision(1), parse("class Product { old() { } }", 0).program),
            (
                consumer.clone(),
                FileRevision(1),
                parse("import \"./provider\" as Provider\nlet product = Provider.Product.new()\n", 0).program,
            ),
        ]);
        db.update_file(&provider, FileRevision(2), &parse("class Product { newMethod() { } }", 0).program);
        let modules = db.last_rebuild_trace().unwrap().modules_recomputed;
        assert!(modules.contains(&ModuleId::from_uri(&provider)));
        assert!(modules.contains(&ModuleId::from_uri(&consumer)));
    }

    #[test]
    fn provider_creation_repairs_previously_unresolved_import() {
        let db = SemanticDb::new();
        let provider = uri("file:///created-provider.ph");
        let consumer = uri("file:///created-consumer.ph");
        db.update_file(&consumer, FileRevision(1), &parse("import \"./created-provider\" as Provider\n", 0).program);
        db.update_file(&provider, FileRevision(1), &parse("class Product { }", 0).program);
        assert!(db.imports(&consumer)[0].target.is_some());
        assert!(db.last_rebuild_trace().unwrap().modules_recomputed.contains(&ModuleId::from_uri(&consumer)));
    }

    #[test]
    fn provider_removal_invalidates_existing_importer() {
        let db = SemanticDb::new();
        let provider = uri("file:///removed-provider.ph");
        let consumer = uri("file:///removed-consumer.ph");
        db.update_files_batch(vec![
            (provider.clone(), FileRevision(1), parse("class Product { }", 0).program),
            (
                consumer.clone(),
                FileRevision(1),
                parse("import \"./removed-provider\" as Provider\n", 0).program,
            ),
        ]);
        db.remove_file(&provider);
        assert!(db.imports(&consumer)[0].target.is_none());
        assert!(db.last_rebuild_trace().unwrap().modules_recomputed.contains(&ModuleId::from_uri(&consumer)));
    }

    #[test]
    fn caller_edit_removes_stale_parameter_contribution() {
        let db = SemanticDb::new();
        let provider = uri("file:///parameter-provider.ph");
        let caller = uri("file:///parameter-caller.ph");
        let provider_text = "class Cat { catOnly() { } }\nclass Dog { dogOnly() { } }\nclass Service { consume(_ value) { value } }\n";
        db.update_file(&provider, FileRevision(1), &parse(provider_text, 0).program);
        let cat_call = "import \"./parameter-provider\" as Provider\nProvider.Service.new().consume(Provider.Cat.new())\n";
        db.update_file(&caller, FileRevision(1), &parse(cat_call, 0).program);
        let service = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&provider), "Service"),
            selector: "consume(_)".to_string(),
            side: DispatchSide::Instance,
        };
        assert!(matches!(db.parameter_at(&service, "value").unwrap().shape, ValueShape::Instance(ClassId { name, .. }) if name == "Cat"));
        let dog_call = "import \"./parameter-provider\" as Provider\nProvider.Service.new().consume(Provider.Dog.new())\n";
        db.update_file(&caller, FileRevision(2), &parse(dog_call, 0).program);
        assert!(matches!(db.parameter_at(&service, "value").unwrap().shape, ValueShape::Instance(ClassId { name, .. }) if name == "Dog"));
    }

    #[test]
    fn unimported_unique_workspace_class_does_not_resolve() {
        let db = SemanticDb::new();
        let provider = uri("file:///unique-provider.ph");
        let consumer = uri("file:///unique-consumer.ph");
        db.update_file(&provider, FileRevision(1), &parse("class Product { }", 0).program);
        db.update_file(&consumer, FileRevision(1), &parse("class Factory { make() { Product.new() } }", 0).program);
        let id = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&consumer), "Factory"),
            selector: "make()".to_string(),
            side: DispatchSide::Instance,
        };
        assert_eq!(db.return_for_callable(&id).unwrap().shape, ValueShape::Unknown);
    }

    #[test]
    fn same_named_imported_classes_remain_module_qualified() {
        let db = SemanticDb::new();
        let first = uri("file:///first-user.ph");
        let second = uri("file:///second-user.ph");
        let consumer = uri("file:///qualified-consumer.ph");
        db.update_files_batch(vec![
            (first.clone(), FileRevision(1), parse("class User { firstOnly() { } }", 0).program),
            (second.clone(), FileRevision(1), parse("class User { secondOnly() { } }", 0).program),
            (
                consumer.clone(),
                FileRevision(1),
                parse("import \"./first-user\" as First\nimport \"./second-user\" as Second\n", 0).program,
            ),
        ]);
        let imports = db.imports(&consumer);
        assert_eq!(imports[0].target, Some(ModuleId::from_uri(&first)));
        assert_eq!(imports[1].target, Some(ModuleId::from_uri(&second)));
        assert_ne!(
            ClassId::new(imports[0].target.clone().unwrap(), "User"),
            ClassId::new(imports[1].target.clone().unwrap(), "User")
        );
    }

    #[test]
    fn cyclic_import_graph_terminates_without_panic() {
        let db = SemanticDb::new();
        let first = uri("file:///cycle-a.ph");
        let second = uri("file:///cycle-b.ph");
        db.update_files_batch(vec![
            (first.clone(), FileRevision(1), parse("import \"./cycle-b\" as B\n", 0).program),
            (second.clone(), FileRevision(1), parse("import \"./cycle-a\" as A\n", 0).program),
        ]);
        assert!(db.imports(&first)[0].target.is_some());
        assert!(db.imports(&second)[0].target.is_some());
    }
}
