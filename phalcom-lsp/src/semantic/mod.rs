//! VM-free live semantic database for LSP requests.

mod analyzer;
mod callable;
pub(crate) mod core_source;
mod dispatch;
pub(crate) mod engine;
mod facts;
mod flow;
mod ids;
mod infer;
mod invalidation;
mod module_graph;
mod occurrence;
mod query;
mod scope;
pub(crate) mod snapshot;
pub(crate) mod source;
mod surface;

use std::collections::BTreeSet;
#[cfg(test)]
use std::sync::Mutex;
use std::sync::{Arc, RwLock};

pub use engine::SemanticEngine;
pub use snapshot::{FileSourceSnapshot, SemanticSnapshot};

#[cfg(test)]
use phalcom_ast::ast::Program;
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::Url;

use crate::perf::{PerfCounters, PerfCountersHandle};

pub use callable::{CallableSummary, SummaryEffects};
pub use core_source::NativeReturnShape;
use dispatch::{ClassTable, SummaryTable};
pub use facts::{
    Confidence, ContributionSource, FactOrigin, FieldEvidence, FieldEvidenceKind, FieldFacts, FileRevision, InferredValue, LocalFacts, MAX_SHAPE_UNION,
    ParameterContributions, ParameterFacts, ParameterSlot, ValueShape,
};
pub use flow::join_values;
pub use ids::{CORE_MODULE_URI, CallableId, ClassId, DispatchSide, DocumentModuleMap, FieldId, ModuleId};
pub use invalidation::{InvalidationQueue, SourceChangeKind, classify_source_change};
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
        ValueShape::Selector(sel) => format!("#{sel}"),
        ValueShape::SelectorPattern(pat) => format!("#{pat}"),
        ValueShape::Family { spec, .. } => match spec {
            phalcom_ast::ast::NormalizedSelectorSpec::Exact(sel) => format!("Family<#{sel}>"),
            phalcom_ast::ast::NormalizedSelectorSpec::Pattern(pat) => format!("Family<#{pat}>"),
        },
        ValueShape::Method(callable) => format!("Method<{}>", callable.selector),
        ValueShape::MethodFamily(family) => format!("MethodFamily<#{}>", family.pattern),
        ValueShape::BoundMethod { method, .. } => format!("BoundMethod<{}>", method.selector),
        ValueShape::BoundMethodFamily { family, .. } => format!("BoundMethodFamily<#{}>", family.pattern),
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
    /// Immutable source products shared by semantic passes and read queries.
    pub source: Arc<FileSourceSnapshot>,
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

#[cfg(test)]
pub(crate) use engine::RebuildTraceData;

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

/// Thread-safe published semantic database wrapper owned by [`crate::backend::Backend`].
pub struct SemanticDb {
    current: RwLock<Arc<SemanticSnapshot>>,
    #[cfg(test)]
    engine: Mutex<SemanticEngine>,
    counters: PerfCountersHandle,
}

impl Default for SemanticDb {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticDb {
    /// Creates a new empty semantic database at generation zero.
    pub fn new() -> Self {
        Self::with_counters(Arc::new(PerfCounters::new()))
    }

    /// Creates a semantic database with a caller-owned counter set.
    pub(crate) fn with_counters(counters: PerfCountersHandle) -> Self {
        #[cfg(test)]
        let engine = SemanticEngine::new_with_counters(counters.clone());
        Self {
            current: RwLock::new(Arc::new(SemanticSnapshot::default())),
            #[cfg(test)]
            engine: Mutex::new(engine),
            counters,
        }
    }

    /// Creates an empty semantic database without running core analysis (generation 0).
    pub fn empty() -> Self {
        Self::new()
    }

    /// Returns the counter set owned by this database and its analysis service.
    pub fn perf_counters(&self) -> PerfCountersHandle {
        self.counters.clone()
    }

    /// Returns the latest immutable published semantic snapshot.
    pub fn snapshot(&self) -> Arc<SemanticSnapshot> {
        self.current.read().expect("semantic publication lock poisoned").clone()
    }

    /// Atomically publishes a new immutable snapshot.
    pub(crate) fn publish(&self, snapshot: Arc<SemanticSnapshot>) {
        *self.current.write().expect("semantic publication lock poisoned") = snapshot;
    }

    /// Replaces one file contribution and publishes one coherent generation.
    #[cfg(test)]
    pub fn update_file(&self, uri: &Url, revision: FileRevision, program: &Program) -> SemanticGeneration {
        self.update_files_batch(vec![(uri.clone(), revision, program.clone())])
    }

    /// Replaces several file contributions and publishes one coherent generation.
    #[cfg(test)]
    pub fn update_files_batch(&self, files: Vec<(Url, FileRevision, Program)>) -> SemanticGeneration {
        let mut engine = self.engine.lock().expect("semantic engine lock poisoned");
        let generation = engine.update_files_batch(files);
        self.publish(Arc::new(engine.snapshot()));
        generation
    }

    /// Replaces the active core library module.
    #[cfg(test)]
    pub fn update_core(&self, revision: FileRevision, program: &Program) -> SemanticGeneration {
        let mut engine = self.engine.lock().expect("semantic engine lock poisoned");
        let generation = engine.update_core(revision, program);
        self.publish(Arc::new(engine.snapshot()));
        generation
    }

    /// Removes one source file from the active universe.
    #[cfg(test)]
    pub fn remove_file(&self, uri: &Url) -> SemanticGeneration {
        let mut engine = self.engine.lock().expect("semantic engine lock poisoned");
        let generation = engine.remove_file(uri);
        self.publish(Arc::new(engine.snapshot()));
        generation
    }

    #[cfg(test)]
    /// Returns the last semantic rebuild trace.
    pub fn last_rebuild_trace(&self) -> Option<RebuildTrace> {
        self.engine.lock().expect("semantic engine lock poisoned").last_rebuild_trace()
    }

    /// Returns the current semantic generation.
    pub fn generation(&self) -> SemanticGeneration {
        self.snapshot().generation()
    }

    /// Returns an immutable clone of one file's semantic snapshot.
    pub fn file_snapshot(&self, uri: &Url) -> Option<FileSemanticSnapshot> {
        self.snapshot().file_snapshot(uri)
    }

    /// Returns the exact semantic occurrence covering one source offset.
    pub fn occurrence_at(&self, uri: &Url, offset: usize) -> Option<SemanticOccurrence> {
        self.snapshot().occurrence_at(uri, offset)
    }

    /// Returns all references to a SemanticTarget in the workspace.
    pub fn references_for_target(&self, uri: &Url, target: &SemanticTarget) -> Vec<(Url, SourceRange, OccurrenceRole)> {
        self.snapshot().references_for_target(uri, target)
    }

    /// Returns lexical bindings visible at one source offset, nearest scope first.
    pub fn visible_bindings_at(&self, uri: &Url, offset: usize) -> Vec<BindingInfo> {
        self.snapshot().visible_bindings_at(uri, offset)
    }

    /// Returns one binding's declaration metadata from a file-local identity.
    pub fn binding_info(&self, uri: &Url, binding: BindingId) -> Option<BindingInfo> {
        self.snapshot().binding_info(uri, binding)
    }

    /// Returns one class surface by module-qualified identity.
    pub fn class_surface(&self, id: &ClassId) -> Option<ClassSurface> {
        self.snapshot().class_surface(id).cloned()
    }

    /// Returns one member surface by its complete callable identity.
    pub fn member_surface(&self, callable: &CallableId) -> Option<MemberSurface> {
        self.snapshot().member_surface(callable).cloned()
    }

    /// Resolves one receiver-qualified member, including inherited members.
    pub fn receiver_member(&self, class: &ClassId, selector: &str, side: DispatchSide) -> Option<MemberSurface> {
        self.snapshot().receiver_member(class, selector, side)
    }

    /// Returns inherited, de-duplicated members for one live class surface.
    pub fn completion_members(&self, class: &ClassId, side: DispatchSide) -> Vec<CompletionMember> {
        self.snapshot().completion_members(class, side)
    }

    /// Returns every declared live member, de-duplicated by selector.
    pub fn all_completion_members(&self) -> Vec<CompletionMember> {
        self.snapshot().all_completion_members()
    }

    /// Tests module-qualified class ancestry for visibility filtering.
    pub fn is_same_or_subclass(&self, child: &ClassId, ancestor: &ClassId) -> bool {
        self.snapshot().is_same_or_subclass(child, ancestor)
    }

    /// Returns a source callable summary from the current semantic generation.
    pub fn callable_summary(&self, id: &CallableId) -> Option<CallableSummary> {
        self.snapshot().callable_summary(id).cloned()
    }

    /// Returns a callable's target-specific return summary.
    pub fn return_for_callable(&self, id: &CallableId) -> Option<InferredValue> {
        self.snapshot().return_for_callable(id)
    }

    /// Returns the joined call-site fact observed for one callable parameter.
    pub fn parameter_at(&self, id: &CallableId, name: &str) -> Option<InferredValue> {
        self.snapshot().parameter_at(id, name)
    }

    /// Resolves a class name in its module, with the stable core namespace as
    /// a fallback for primitive/runtime classes.
    pub fn class_for_name(&self, uri: &Url, name: &str) -> Option<ClassId> {
        self.snapshot().class_for_name(uri, name)
    }

    /// Returns the class whose declaration contains a byte offset in `uri`.
    pub fn class_at(&self, uri: &Url, offset: usize) -> Option<ClassId> {
        self.snapshot().class_at(uri, offset)
    }

    /// Returns the source-authored class whose name range contains `offset`.
    pub fn class_name_at(&self, uri: &Url, offset: usize) -> Option<ClassSurface> {
        self.snapshot().class_name_at(uri, offset)
    }

    /// Returns the declared callable enclosing a source offset.
    pub fn member_at(&self, uri: &Url, offset: usize) -> Option<MemberSurface> {
        self.snapshot().member_at(uri, offset)
    }

    /// Joins return summaries for a bounded set of receiver candidates.
    pub fn returns_for_callables(&self, ids: impl IntoIterator<Item = CallableId>) -> Option<InferredValue> {
        self.snapshot().returns_for_callables(ids)
    }

    /// Returns the fact visible for a local binding at a byte offset.
    pub fn binding_at(&self, uri: &Url, name: &str, offset: usize) -> Option<InferredValue> {
        self.snapshot().binding_at(uri, name, offset)
    }

    /// Infers a parsed receiver expression against the coherent current
    /// semantic generation.
    pub fn infer_expression(&self, uri: &Url, expr: &phalcom_ast::ast::Expr, offset: usize) -> InferredValue {
        self.snapshot().infer_expression(uri, expr, offset)
    }

    /// Returns current import edges for one module.
    pub fn imports(&self, uri: &Url) -> Vec<ImportEdge> {
        self.snapshot().imports(uri)
    }

    /// Returns a coherent revision/generation stamp for one file.
    pub fn stamp(&self, uri: &Url) -> Option<SnapshotStamp> {
        self.snapshot().stamp(uri)
    }
}

fn resolve_named_class<C: ClassTable + ?Sized>(classes: &C, graph: &ModuleGraph, module: &ModuleId, name: &str) -> Option<ClassId> {
    if let Some((binding, class_name)) = name.split_once('.') {
        let imported = graph
            .imports(module)
            .iter()
            .find(|edge| edge.binding == binding)
            .and_then(|edge| edge.target.as_ref())?;
        let class = ClassId::new(imported.clone(), class_name);
        return classes.contains_class(&class).then_some(class);
    }
    let local = ClassId::new(module.clone(), name);
    if classes.contains_class(&local) {
        return Some(local);
    }
    let core = ClassId::new(ModuleId::new(CORE_MODULE_URI), name);
    classes.contains_class(&core).then_some(core)
}

fn is_same_or_subclass<C: ClassTable + ?Sized>(classes: &C, child: &ClassId, ancestor: &ClassId) -> bool {
    let mut current = Some(child.clone());
    let mut visited = BTreeSet::new();
    while let Some(id) = current.take() {
        if !visited.insert(id.clone()) {
            return false;
        }
        if &id == ancestor {
            return true;
        }
        let Some(surface) = classes.class(&id) else { return false };
        current = surface
            .superclass
            .clone()
            .or_else(|| (id.name != "Object").then(|| ClassId::new(ModuleId::new(CORE_MODULE_URI), "Object")));
    }
    false
}

fn return_for_callable<C: ClassTable + ?Sized, S: SummaryTable + ?Sized>(classes: &C, summaries: &S, id: &CallableId) -> Option<InferredValue> {
    let class = classes.class(&id.owner)?;
    let member = class.member(&id.selector, id.side);
    if id.side == DispatchSide::Class && (id.selector == "new()" || member.is_some_and(|member| member.is_constructor)) {
        return Some(InferredValue::flow(ValueShape::Instance(id.owner.clone()), Default::default()));
    }
    if let Some(summary) = summaries.summary(id) {
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
    fn empty_semantic_db_has_generation_zero_and_no_files() {
        let db = SemanticDb::empty();
        assert_eq!(db.generation(), SemanticGeneration(0));
        assert!(db.snapshot().files.is_empty());
        assert!(db.snapshot().classes.is_empty());
    }

    #[test]
    fn update_publishes_revisioned_local_facts() {
        let db = SemanticDb::new();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let uri = uri("file:///main.ph");
        let parse = parse("let text = \"hello\"\n", 0);
        let generation = db.update_file(&uri, FileRevision(7), &parse.program);
        assert_eq!(generation.0, 2);
        assert_eq!(db.file_snapshot(&uri).unwrap().revision, FileRevision(7));
        assert!(matches!(db.binding_at(&uri, "text", 20).unwrap().shape, ValueShape::Instance(ClassId { name, .. }) if name == "String"));
    }

    #[test]
    fn same_named_classes_are_isolated() {
        let db = SemanticDb::new();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let string = ClassId::new(ModuleId::new(CORE_MODULE_URI), "String");
        let members = db.completion_members(&string, DispatchSide::Instance);
        assert!(members.iter().any(|member| member.selector == "size"));
        assert!(members.iter().any(|member| member.selector == "hash"));
    }

    #[test]
    fn live_core_replacement_updates_semantic_surface() {
        let db = SemanticDb::new();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let uri = uri("file:///family.ph");
        let source = "class Box { @constructor new() { } value() { 1 } }\nlet family = Box.new()::value(...)\n";
        let parsed = parse(source, 0);
        db.update_file(&uri, FileRevision(1), &parsed.program);
        let family_binding = db
            .binding_at(&uri, "family", source.find("family").expect("family binding offset") + 1)
            .expect("family binding");
        assert_eq!(
            family_binding.shape,
            ValueShape::Family {
                receiver: Box::new(ValueShape::Instance(ClassId::new(ModuleId::from_uri(&uri), "Box"))),
                spec: phalcom_ast::ast::NormalizedSelectorSpec::Pattern(
                    phalcom_common::selector::SelectorPattern::named_method("value", [], [], true).unwrap(),
                ),
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
    fn trusted_is_guards_narrow_sheetcalc_cell_value_returns() {
        let db = SemanticDb::new();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let uri = uri("file:///cell_value.ph");
        let source = format!(
            "{}\nlet x = CellNum.of(10)\nlet y = CellNum.of(5)\nlet z = CellEmpty.new()\nlet result1 = x.minus(y)\nlet result2 = z.minus(x)\n",
            include_str!("../../../examples/sheetcalc/src/value/cell_value.ph")
        );
        let parsed = parse(&source, 0);
        assert!(parsed.errors.is_empty(), "CellValue fixture parse errors: {:?}", parsed.errors);
        db.update_file(&uri, FileRevision(1), &parsed.program);

        let result1 = db
            .binding_at(&uri, "result1", source.find("let result2").expect("result2 declaration"))
            .expect("result1 binding fact");
        let result2 = db.binding_at(&uri, "result2", source.len()).expect("result2 binding fact");

        assert!(
            matches!(result1.shape, ValueShape::Instance(ClassId { ref name, .. }) if name == "CellNum"),
            "result1: {:?}",
            result1.shape
        );
        assert!(
            matches!(result2.shape, ValueShape::Instance(ClassId { ref name, .. }) if name == "ErrorVal"),
            "result2: {:?}",
            result2.shape
        );
    }

    #[test]
    fn invoked_literal_block_contributes_nonlocal_return() {
        let db = SemanticDb::new();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let consumer_text = "import .provider as Provider\nlet product = Provider.Factory.new().make()\nlet consumed = Provider.Service.new().consume(Provider.Product.new())\n";
        std::fs::write(&provider_path, provider_text).unwrap();
        std::fs::write(&consumer_path, consumer_text).unwrap();
        let provider_uri = Url::from_file_path(&provider_path).unwrap();
        let consumer_uri = Url::from_file_path(&consumer_path).unwrap();
        let db = SemanticDb::new();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let provider = uri("file:///provider.ph");
        let consumer = uri("file:///consumer.ph");
        db.update_files_batch(vec![
            (provider.clone(), FileRevision(1), parse("class Product { old() { } }", 0).program),
            (
                consumer.clone(),
                FileRevision(1),
                parse("import .provider as Provider\nlet product = Provider.Product.new()\n", 0).program,
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let provider = uri("file:///created-provider.ph");
        let consumer = uri("file:///created-consumer.ph");
        db.update_file(&consumer, FileRevision(1), &parse("import .created_provider as Provider\n", 0).program);
        db.update_file(&provider, FileRevision(1), &parse("class Product { }", 0).program);
        assert!(db.imports(&consumer)[0].target.is_some());
        assert!(db.last_rebuild_trace().unwrap().modules_recomputed.contains(&ModuleId::from_uri(&consumer)));
    }

    #[test]
    fn provider_removal_invalidates_existing_importer() {
        let db = SemanticDb::new();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let provider = uri("file:///removed-provider.ph");
        let consumer = uri("file:///removed-consumer.ph");
        db.update_files_batch(vec![
            (provider.clone(), FileRevision(1), parse("class Product { }", 0).program),
            (consumer.clone(), FileRevision(1), parse("import .removed_provider as Provider\n", 0).program),
        ]);
        db.remove_file(&provider);
        assert!(db.imports(&consumer)[0].target.is_none());
        assert!(db.last_rebuild_trace().unwrap().modules_recomputed.contains(&ModuleId::from_uri(&consumer)));
    }

    #[test]
    fn caller_edit_removes_stale_parameter_contribution() {
        let db = SemanticDb::new();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let provider = uri("file:///parameter-provider.ph");
        let caller = uri("file:///parameter-caller.ph");
        let provider_text = "class Cat { catOnly() { } }\nclass Dog { dogOnly() { } }\nclass Service { consume(_ value) { value } }\n";
        db.update_file(&provider, FileRevision(1), &parse(provider_text, 0).program);
        let cat_call = "import .parameter_provider as Provider\nProvider.Service.new().consume(Provider.Cat.new())\n";
        db.update_file(&caller, FileRevision(1), &parse(cat_call, 0).program);
        let service = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&provider), "Service"),
            selector: "consume(_)".to_string(),
            side: DispatchSide::Instance,
        };
        assert!(matches!(db.parameter_at(&service, "value").unwrap().shape, ValueShape::Instance(ClassId { name, .. }) if name == "Cat"));
        let dog_call = "import .parameter_provider as Provider\nProvider.Service.new().consume(Provider.Dog.new())\n";
        db.update_file(&caller, FileRevision(2), &parse(dog_call, 0).program);
        assert!(matches!(db.parameter_at(&service, "value").unwrap().shape, ValueShape::Instance(ClassId { name, .. }) if name == "Dog"));
    }

    #[test]
    fn parameter_facts_from_two_consumers_join_across_sequential_updates() {
        let db = SemanticDb::new();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let provider = uri("file:///join-provider.ph");
        let first = uri("file:///join-first.ph");
        let second = uri("file:///join-second.ph");
        db.update_file(&provider, FileRevision(1), &parse("class Service { consume(_ value) { value } }\n", 0).program);
        db.update_file(
            &first,
            FileRevision(1),
            &parse(
                "import .join_provider as Provider\nclass Cat { catOnly() { } }\nProvider.Service.new().consume(Cat.new())\n",
                0,
            )
            .program,
        );
        db.update_file(
            &second,
            FileRevision(1),
            &parse(
                "import .join_provider as Provider\nclass Dog { dogOnly() { } }\nProvider.Service.new().consume(Dog.new())\n",
                0,
            )
            .program,
        );
        let service = CallableId {
            owner: ClassId::new(ModuleId::from_uri(&provider), "Service"),
            selector: "consume(_)".to_string(),
            side: DispatchSide::Instance,
        };
        let shape = db.parameter_at(&service, "value").expect("joined parameter fact").shape;
        assert!(
            matches!(shape, ValueShape::Union(ref shapes) if shapes.iter().any(|shape| matches!(shape, ValueShape::Instance(ClassId { name, .. }) if name == "Cat")))
        );
        assert!(
            matches!(shape, ValueShape::Union(ref shapes) if shapes.iter().any(|shape| matches!(shape, ValueShape::Instance(ClassId { name, .. }) if name == "Dog")))
        );
    }

    #[test]
    fn unimported_unique_workspace_class_does_not_resolve() {
        let db = SemanticDb::new();
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let first = uri("file:///first-user.ph");
        let second = uri("file:///second-user.ph");
        let consumer = uri("file:///qualified-consumer.ph");
        db.update_files_batch(vec![
            (first.clone(), FileRevision(1), parse("class User { firstOnly() { } }", 0).program),
            (second.clone(), FileRevision(1), parse("class User { secondOnly() { } }", 0).program),
            (
                consumer.clone(),
                FileRevision(1),
                parse("import .first_user as First\nimport .second_user as Second\n", 0).program,
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
        let bundled = core_source::bundled_parse();
        db.update_core(FileRevision(1), &bundled.program);
        let first = uri("file:///cycle-a.ph");
        let second = uri("file:///cycle-b.ph");
        db.update_files_batch(vec![
            (first.clone(), FileRevision(1), parse("import .cycle_b as B\n", 0).program),
            (second.clone(), FileRevision(1), parse("import .cycle_a as A\n", 0).program),
        ]);
        assert!(db.imports(&first)[0].target.is_some());
        assert!(db.imports(&second)[0].target.is_some());
    }
}
