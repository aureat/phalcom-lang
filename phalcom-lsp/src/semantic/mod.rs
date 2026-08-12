//! VM-free live semantic database for LSP requests.

mod callable;
pub(crate) mod core_source;
mod facts;
mod flow;
mod ids;
mod infer;
mod invalidation;
mod module_graph;
mod query;
mod surface;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::RwLock;

use phalcom_ast::ast::{Expr, PackItem, PackLabel, Program};
use tower_lsp::lsp_types::Url;

pub use callable::{CallableSummary, SummaryEffects};
pub use core_source::NativeReturnKnowledge;
pub use facts::{Confidence, FactOrigin, FieldFacts, FileRevision, InferredValue, LocalFacts, ParameterFacts, ValueShape, MAX_SHAPE_UNION};
pub use flow::join_values;
pub use ids::{CallableId, ClassId, DispatchSide, ModuleId, CORE_MODULE_URI};
pub use invalidation::InvalidationQueue;
pub use module_graph::{ImportEdge, ModuleGraph};
pub use query::{SemanticGeneration, SnapshotStamp};
pub use surface::{build_module_surface, ClassSurface, FieldKind, FieldSurface, MemberKind, MemberSurface, MemberVisibility, ModuleSurface, ParamSurface};

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
    field_facts: BTreeMap<(ClassId, String), InferredValue>,
    parameter_facts: BTreeMap<(CallableId, String), InferredValue>,
    callable_dependents: BTreeMap<CallableId, std::collections::BTreeSet<CallableId>>,
    graph: ModuleGraph,
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
        let module = ModuleId::from_uri(uri);
        let mut state = self.state.write().expect("semantic database lock poisoned");
        let surface = if module.as_str() == CORE_MODULE_URI {
            core_source::build_core_surface(program)
        } else {
            build_module_surface(module.clone(), program)
        };
        let next_generation = SemanticGeneration(state.generation.0 + 1);
        state.graph.update(module.clone(), program);
        state.files.insert(
            module.clone(),
            FileSemanticSnapshot {
                revision,
                module: module.clone(),
                program: Arc::new(program.clone()),
                surface,
                local_facts: LocalFacts::default(),
                field_facts: FieldFacts::default(),
                parameter_facts: ParameterFacts::default(),
                dependencies: DependencySet::default(),
            },
        );
        state.graph.refresh_resolutions();
        let mut queue = InvalidationQueue::default();
        queue.push(module.clone());
        let changed_callables = state
            .summaries
            .values()
            .filter(|summary| summary.callable.owner.module == module)
            .map(|summary| summary.callable.clone())
            .collect::<Vec<_>>();
        for callable in changed_callables {
            if let Some(dependents) = state.callable_dependents.get(&callable) {
                for dependent in dependents {
                    queue.push(dependent.owner.module.clone());
                }
            }
        }
        for dependent in state.graph.dependent_closure(&module) {
            queue.push(dependent);
        }
        rebuild_state(&mut state, next_generation, &mut queue);
        state.generation = next_generation;
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
        state.files.remove(&module);
        state.classes.retain(|class, _| class.module != module);
        state.summaries.retain(|callable, _| callable.owner.module != module);
        state.field_facts.retain(|(class, _), _| class.module != module);
        state.parameter_facts.retain(|(callable, _), _| callable.owner.module != module);
        state.graph.remove(&module);
        state.graph.refresh_resolutions();
        let mut queue = InvalidationQueue::default();
        for dependent in state.graph.dependent_closure(&module) {
            queue.push(dependent);
        }
        let next_generation = SemanticGeneration(state.generation.0 + 1);
        rebuild_state(&mut state, next_generation, &mut queue);
        state.generation.0 += 1;
        state.generation
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
        let member = resolve_member_surface(&state.classes, class, selector)?;
        (member.side == side).then_some(member)
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
            for (selector, member) in &surface.members {
                if member.side == side && seen.insert(selector.clone()) {
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
            for (selector, member) in &surface.members {
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
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .summaries
            .get(id)
            .map(|summary| summary.returns.clone())
    }

    /// Returns one observed return summary for a canonical selector.
    pub fn return_for_selector(&self, selector: &str) -> Option<InferredValue> {
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .summaries
            .values()
            .find(|summary| summary.callable.selector == selector)
            .map(|summary| summary.returns.clone())
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
        let local = ClassId::new(module, name);
        if state.classes.contains_key(&local) {
            return Some(local);
        }
        let core = ClassId::new(ModuleId::new(CORE_MODULE_URI), name);
        state.classes.contains_key(&core).then_some(core)
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
            .flat_map(|class| class.members.values())
            .find(|member| member.source_range.contains(offset))
            .cloned()
    }

    /// Joins return summaries for a bounded set of receiver candidates.
    pub fn returns_for_callables(&self, ids: impl IntoIterator<Item = CallableId>) -> Option<InferredValue> {
        let state = self.state.read().expect("semantic database lock poisoned");
        ids.into_iter()
            .filter_map(|id| state.summaries.get(&id).map(|summary| summary.returns.clone()))
            .reduce(|left, right| left.join(&right))
    }

    /// Returns the fact visible for a local binding at a byte offset.
    pub fn binding_at(&self, uri: &Url, name: &str, offset: usize) -> Option<InferredValue> {
        let module = ModuleId::from_uri(uri);
        self.state
            .read()
            .expect("semantic database lock poisoned")
            .files
            .get(&module)?
            .local_facts
            .binding_at(name, offset)
            .cloned()
    }

    /// Infers a parsed receiver expression against the coherent current
    /// semantic generation.
    pub fn infer_expression(&self, uri: &Url, expr: &phalcom_ast::ast::Expr, offset: usize) -> InferredValue {
        let module = ModuleId::from_uri(uri);
        let state = self.state.read().expect("semantic database lock poisoned");
        if let Some(shape) = infer_imported_expression(&state, &module, expr) {
            return InferredValue::flow(shape, expr.range());
        }
        let mut environment = BTreeMap::new();
        if let Some(file) = state.files.get(&module) {
            collect_expression_environment(expr, &file.local_facts, offset, &mut environment);
        }
        let known_classes = |name: &str| resolve_named_class(&state.classes, &state.graph, &module, name);
        let is_constructor = |class: &ClassId, selector: &str| {
            resolve_member_surface(&state.classes, class, selector).is_some_and(|member| member.is_constructor)
                || (selector == "new()" && state.classes.contains_key(class))
        };
        let callable_return = |id: &CallableId| state.summaries.get(id).map(|summary| summary.returns.clone());
        let field_value = |class: &ClassId, name: &str| state.field_facts.get(&(class.clone(), name.to_string())).cloned();
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
                    .and_then(|class| class.members.values().find(|member| member.source_range.contains(offset)))
                {
                    for param in &member.params {
                        if let Some(value) = state.parameter_facts.get(&(member.callable.clone(), param.name.clone())) {
                            environment.insert(param.name.clone(), value.clone());
                        }
                    }
                }
            }
        }
        infer::infer_expr_with_fields(
            expr,
            &module,
            current_class.as_ref(),
            &environment,
            known_classes,
            is_constructor,
            callable_return,
            field_value,
        )
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

const MAX_SOLVER_ROUNDS: usize = 64;

fn rebuild_state(state: &mut SemanticState, generation: SemanticGeneration, queue: &mut InvalidationQueue) {
    // Consume the dependency queue as part of publication. The current
    // implementation recomputes the compact workspace model in one bounded
    // transaction; the queue still defines the affected frontier and keeps
    // dependency invalidation explicit for a later incremental extractor.
    let _affected = queue.drain().collect::<std::collections::BTreeSet<_>>();
    let inputs = state
        .files
        .values()
        .map(|file| (file.module.clone(), file.program.clone(), file.surface.clone()))
        .collect::<Vec<_>>();
    let mut classes = BTreeMap::new();
    for (_, _, surface) in &inputs {
        classes.extend(surface.classes.iter().map(|(id, class)| (id.clone(), class.clone())));
    }
    let graph = state.graph.clone();
    let mut summaries: BTreeMap<CallableId, CallableSummary> = BTreeMap::new();
    let mut parameter_facts: BTreeMap<(CallableId, String), InferredValue> = BTreeMap::new();

    for _ in 0..MAX_SOLVER_ROUNDS {
        let previous_summaries = summaries.clone();
        let previous_parameters = parameter_facts.clone();
        let mut next_parameters = BTreeMap::new();
        for (module, program, surface) in &inputs {
            let known_class = |name: &str| resolve_named_class(&classes, &graph, module, name);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolve_member_surface(&classes, class, selector).is_some_and(|member| member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| previous_summaries.get(id).map(|summary| summary.returns.clone());
            let resolve_member = |class: &ClassId, selector: &str| resolve_member_surface(&classes, class, selector);
            let facts = infer::parameter_facts_for_program(program, surface, module, known_class, is_constructor, callable_return, resolve_member);
            next_parameters.extend(facts.iter().map(|(key, value)| (key.clone(), value.clone())));
        }

        let mut next_summaries = BTreeMap::new();
        for (module, _, surface) in &inputs {
            let known_class = |name: &str| resolve_named_class(&classes, &graph, module, name);
            let is_constructor = |class: &ClassId, selector: &str| {
                resolve_member_surface(&classes, class, selector).is_some_and(|member| member.is_constructor)
                    || (selector == "new()" && classes.contains_key(class))
            };
            let callable_return = |id: &CallableId| previous_summaries.get(id).map(|summary| summary.returns.clone());
            let parameter_fact = |id: &CallableId, name: &str| next_parameters.get(&(id.clone(), name.to_string())).cloned();
            let resolve_member = |class: &ClassId, selector: &str| resolve_member_surface(&classes, class, selector);
            let summaries_for_module = infer::summaries_for_surface(
                surface,
                module,
                known_class,
                is_constructor,
                callable_return,
                parameter_fact,
                resolve_member,
                generation,
            );
            next_summaries.extend(summaries_for_module.into_iter().map(|summary| (summary.callable.clone(), summary)));
        }

        let summaries_changed = next_summaries != previous_summaries;
        let parameters_changed = next_parameters != previous_parameters;
        summaries = next_summaries;
        parameter_facts = next_parameters;
        if !summaries_changed && !parameters_changed {
            break;
        }
    }

    let mut local_by_module = BTreeMap::new();
    let mut fields_by_module = BTreeMap::new();
    for (module, program, surface) in &inputs {
        let known_class = |name: &str| resolve_named_class(&classes, &graph, module, name);
        let is_constructor = |class: &ClassId, selector: &str| {
            resolve_member_surface(&classes, class, selector).is_some_and(|member| member.is_constructor)
                || (selector == "new()" && classes.contains_key(class))
        };
        let callable_return = |id: &CallableId| summaries.get(id).map(|summary| summary.returns.clone());
        local_by_module.insert(
            module.clone(),
            infer::collect_local_facts_with_returns(program, module, known_class, is_constructor, callable_return),
        );
        fields_by_module.insert(
            module.clone(),
            infer::field_facts_for_surface(surface, module, known_class, is_constructor, callable_return),
        );
    }

    let mut fields = BTreeMap::new();
    for facts in fields_by_module.values() {
        fields.extend(facts.iter().map(|(key, value)| (key.clone(), value.clone())));
    }
    state.classes = classes;
    state.summaries = summaries;
    state.parameter_facts = parameter_facts;
    state.field_facts = fields;
    let mut callable_dependents: BTreeMap<CallableId, std::collections::BTreeSet<CallableId>> = BTreeMap::new();
    for summary in state.summaries.values() {
        for dependency in &summary.dependencies {
            callable_dependents.entry(dependency.clone()).or_default().insert(summary.callable.clone());
        }
    }
    state.callable_dependents = callable_dependents;
    for file in state.files.values_mut() {
        file.local_facts = local_by_module.remove(&file.module).unwrap_or_default();
        file.field_facts = fields_by_module.remove(&file.module).unwrap_or_default();
        file.parameter_facts = ParameterFacts::default();
        file.dependencies = DependencySet {
            imports: state.graph.imports(&file.module).iter().filter_map(|edge| edge.target.clone()).collect(),
        };
    }
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
    if classes.contains_key(&core) {
        return Some(core);
    }
    // Preserve legacy workspace-global class references only when identity is
    // unambiguous. Same-named classes still require module qualification.
    let mut matches = classes.keys().filter(|class| class.name == name);
    let first = matches.next()?.clone();
    matches.next().is_none().then_some(first)
}

fn resolve_member_surface(classes: &BTreeMap<ClassId, ClassSurface>, class: &ClassId, selector: &str) -> Option<MemberSurface> {
    let mut current = Some(class.clone());
    let mut visited = std::collections::BTreeSet::new();
    while let Some(id) = current {
        if !visited.insert(id.clone()) {
            return None;
        }
        let surface = classes.get(&id)?;
        if let Some(member) = surface.members.get(selector) {
            return Some(member.clone());
        }
        current = surface
            .superclass
            .clone()
            .or_else(|| (id.name != "Object").then(|| ClassId::new(ModuleId::new(CORE_MODULE_URI), "Object")));
    }
    None
}

fn infer_imported_expression(state: &SemanticState, module: &ModuleId, expr: &Expr) -> Option<ValueShape> {
    match expr {
        Expr::GetProperty(property) => {
            let Expr::Var { value: binding, .. } = &property.object else { return None };
            imported_class(state, module, binding, &property.property).map(ValueShape::ClassObject)
        }
        Expr::MethodCall(call) => {
            let ValueShape::ClassObject(class) = infer_imported_expression(state, module, &call.object)? else {
                return None;
            };
            let labels = call
                .args
                .iter()
                .map(|arg| match arg {
                    PackItem::Labeled {
                        label: PackLabel::Static { text, .. },
                        ..
                    } => Some(text.clone()),
                    PackItem::Positional { .. } | PackItem::Expand { .. } | PackItem::Labeled { .. } => None,
                })
                .collect::<Vec<_>>();
            let selector = crate::selectors::comma_form_from_labels(&call.method, &labels);
            let is_constructor = selector == "new()"
                || state
                    .classes
                    .get(&class)
                    .and_then(|surface| surface.members.get(&selector))
                    .is_some_and(|member| member.is_constructor);
            is_constructor.then_some(ValueShape::Instance(class))
        }
        _ => None,
    }
}

fn imported_class(state: &SemanticState, module: &ModuleId, binding: &str, name: &str) -> Option<ClassId> {
    let imported = state
        .graph
        .imports(module)
        .iter()
        .find(|edge| edge.binding == binding)
        .and_then(|edge| edge.target.as_ref())?;
    let class = ClassId::new(imported.clone(), name);
    state.classes.contains_key(&class).then_some(class)
}

fn collect_expression_environment(expr: &phalcom_ast::ast::Expr, facts: &LocalFacts, offset: usize, environment: &mut BTreeMap<String, InferredValue>) {
    match expr {
        phalcom_ast::ast::Expr::Var { value, .. } => {
            if let Some(fact) = facts.binding_at(value, offset) {
                environment.insert(value.clone(), fact.clone());
            }
        }
        phalcom_ast::ast::Expr::MethodCall(call) => {
            collect_expression_environment(&call.object, facts, offset, environment);
            for arg in &call.args {
                let expression = match arg {
                    phalcom_ast::ast::PackItem::Positional { expr, .. }
                    | phalcom_ast::ast::PackItem::Expand { expr, .. }
                    | phalcom_ast::ast::PackItem::Labeled { value: expr, .. } => expr,
                };
                collect_expression_environment(expression, facts, offset, environment);
            }
        }
        phalcom_ast::ast::Expr::GetProperty(property) => collect_expression_environment(&property.object, facts, offset, environment),
        _ => {}
    }
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
        assert!(db
            .completion_members(&string, DispatchSide::Instance)
            .iter()
            .any(|member| member.selector == "liveEditorMember()"));
        assert!(!db
            .completion_members(&string, DispatchSide::Instance)
            .iter()
            .any(|member| member.selector == "size"));
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
}
