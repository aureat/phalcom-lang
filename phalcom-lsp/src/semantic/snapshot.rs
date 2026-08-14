//! Immutable published semantic snapshot for zero-blocking LSP queries.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::Url;

use super::analyzer::{AnalysisContext, analyze_expr};
use super::callable::CallableSummary;
use super::dispatch::{DispatchReceiver, DispatchResolver};
use super::facts::InferredValue;
use super::ids::{CORE_MODULE_URI, CallableId, ClassId, DispatchSide, FieldId, ModuleId};
use super::module_graph::{ImportEdge, ModuleGraph};
use super::occurrence::{OccurrenceRole, SemanticOccurrence, SemanticTarget};
use super::query::{SemanticGeneration, SnapshotStamp};
use super::scope::{BindingId, BindingInfo, NameResolution};
use super::surface::{ClassSurface, MemberAstRef, MemberSurface};
use super::{CompletionMember, FileSemanticSnapshot, resolve_named_class, return_for_callable};

/// Immutable source products shared by every semantic pass for one file.
///
/// The worker builds this once when a file changes. Flow analysis borrows these
/// products instead of rebuilding lexical scopes or cloning the parsed AST for
/// each fact family.
#[derive(Clone, Debug)]
pub struct FileSourceSnapshot {
    /// Module identity.
    pub module: ModuleId,
    /// Source text retained from ingestion for exact body-delta comparison.
    pub text: Arc<str>,
    /// Parsed source retained for AST-backed member lookup.
    pub program: Arc<phalcom_ast::ast::Program>,
    /// Source-authored semantic surface.
    pub surface: super::surface::ModuleSurface,
    /// Lexical scope graph built for this parsed source.
    pub scopes: super::scope::ScopeGraph,
    /// Direct callable identity to AST-member lookup.
    pub callables: BTreeMap<CallableId, MemberAstRef>,
}

impl FileSourceSnapshot {
    /// Returns one source member through the immutable callable index.
    pub fn member_by_id(&self, callable: &CallableId) -> Option<&MemberSurface> {
        self.surface.member_by_id(callable)
    }
}

/// Immutable published generation of all workspace semantic facts.
#[derive(Clone, Debug, Default)]
pub struct SemanticSnapshot {
    /// Monotonic publication generation.
    pub generation: SemanticGeneration,
    /// Per-module source and semantic analysis facts.
    pub files: Arc<BTreeMap<ModuleId, Arc<FileSemanticSnapshot>>>,
    /// Module-qualified class surfaces.
    pub classes: Arc<BTreeMap<ClassId, Arc<ClassSurface>>>,
    /// Solved callable summaries.
    pub summaries: Arc<BTreeMap<CallableId, Arc<CallableSummary>>>,
    /// Solved field values.
    pub field_facts: Arc<BTreeMap<FieldId, InferredValue>>,
    /// Solved parameter values.
    pub parameter_facts: Arc<BTreeMap<(CallableId, String), InferredValue>>,
    /// Module import and dependency graph.
    pub graph: Arc<ModuleGraph>,
}

impl SemanticSnapshot {
    /// Returns the current semantic generation.
    pub fn generation(&self) -> SemanticGeneration {
        self.generation
    }

    /// Resolves an editor URI to the published module identity without doing
    /// filesystem work on the request path.
    pub fn module_for_uri(&self, uri: &Url) -> Option<&ModuleId> {
        let module = ModuleId::from_uri(uri);
        self.files.get_key_value(&module).map(|(module, _)| module)
    }

    /// Returns one published file by its resolved module identity.
    pub fn file(&self, module: &ModuleId) -> Option<&FileSemanticSnapshot> {
        self.files.get(module).map(Arc::as_ref)
    }

    /// Returns an immutable clone of one file's semantic snapshot.
    pub fn file_snapshot(&self, uri: &Url) -> Option<FileSemanticSnapshot> {
        let module = ModuleId::from_uri(uri);
        self.files.get(&module).map(|f| (**f).clone())
    }

    /// Returns the file revision stamp for one module if present.
    pub fn file_revision(&self, module: &ModuleId) -> Option<super::facts::FileRevision> {
        self.files.get(module).map(|f| f.revision)
    }

    /// Returns the exact semantic occurrence covering one source offset.
    pub fn occurrence_at(&self, uri: &Url, offset: usize) -> Option<SemanticOccurrence> {
        let module = ModuleId::from_uri(uri);
        self.files.get(&module).and_then(|file| file.occurrences.occurrence_at(offset).cloned())
    }

    /// Returns all references to a `SemanticTarget` in the workspace.
    pub fn references_for_target(&self, uri: &Url, target: &SemanticTarget) -> Vec<(Url, SourceRange, OccurrenceRole)> {
        match target {
            SemanticTarget::Binding(_) => {
                let module = ModuleId::from_uri(uri);
                self.files
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
                for (module_id, file) in self.files.iter() {
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
        self.files
            .get(&module)
            .map(|file| file.source.scopes.visible_bindings_at(offset))
            .unwrap_or_default()
    }

    /// Returns one binding's declaration metadata from a file-local identity.
    pub fn binding_info(&self, uri: &Url, binding: BindingId) -> Option<BindingInfo> {
        let module = ModuleId::from_uri(uri);
        self.files.get(&module).and_then(|file| file.source.scopes.bindings.get(&binding).cloned())
    }

    /// Returns one class surface by module-qualified identity.
    pub fn class_surface(&self, id: &ClassId) -> Option<ClassSurface> {
        self.classes.get(id).map(|c| (**c).clone())
    }

    /// Returns one member surface by its complete callable identity.
    pub fn member_surface(&self, callable: &CallableId) -> Option<MemberSurface> {
        self.classes.get(&callable.owner).and_then(|surface| surface.member_by_id(callable)).cloned()
    }

    /// Resolves one receiver-qualified member, including inherited members.
    pub fn receiver_member(&self, class: &ClassId, selector: &str, side: DispatchSide) -> Option<MemberSurface> {
        let receiver = match side {
            DispatchSide::Instance => DispatchReceiver::Instance(class.clone()),
            DispatchSide::Class => DispatchReceiver::ClassObject(class.clone()),
        };
        let classes = self
            .classes
            .iter()
            .map(|(id, surface)| (id.clone(), (**surface).clone()))
            .collect::<BTreeMap<_, _>>();
        let resolver = DispatchResolver::new(&classes);
        resolver
            .resolve(&receiver, selector)
            .and_then(|resolved| resolver.member(&resolved.callable).cloned())
    }

    /// Returns inherited, de-duplicated members for one live class surface.
    pub fn completion_members(&self, class: &ClassId, side: DispatchSide) -> Vec<CompletionMember> {
        let mut current = Some(class.clone());
        let mut seen = BTreeSet::new();
        let mut visited = BTreeSet::new();
        let mut members = Vec::new();
        while let Some(id) = current.take() {
            if !visited.insert(id.clone()) {
                break;
            }
            let Some(surface) = self.classes.get(&id) else { break };
            for member in surface.members_on(side) {
                if seen.insert(member.callable.selector.clone()) {
                    members.push(CompletionMember {
                        selector: member.callable.selector.clone(),
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
        let mut members = BTreeMap::new();
        for (class_id, surface) in self.classes.iter() {
            for member in surface.all_members() {
                let selector = &member.callable.selector;
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
        let mut current = Some(child.clone());
        let mut visited = BTreeSet::new();
        while let Some(id) = current.take() {
            if !visited.insert(id.clone()) {
                return false;
            }
            if &id == ancestor {
                return true;
            }
            let Some(surface) = self.classes.get(&id) else { return false };
            current = surface
                .superclass
                .clone()
                .or_else(|| (id.name != "Object").then(|| ClassId::new(ModuleId::new(CORE_MODULE_URI), "Object")));
        }
        false
    }

    /// Returns a source callable summary from the current semantic generation.
    pub fn callable_summary(&self, id: &CallableId) -> Option<CallableSummary> {
        self.summaries.get(id).map(|s| (**s).clone())
    }

    /// Returns a callable's target-specific return summary.
    pub fn return_for_callable(&self, id: &CallableId) -> Option<InferredValue> {
        let classes = self.classes.iter().map(|(k, v)| (k.clone(), (**v).clone())).collect::<BTreeMap<_, _>>();
        let summaries = self.summaries.iter().map(|(k, v)| (k.clone(), (**v).clone())).collect::<BTreeMap<_, _>>();
        return_for_callable(&classes, &summaries, id)
    }

    /// Returns the joined call-site fact observed for one callable parameter.
    pub fn parameter_at(&self, id: &CallableId, name: &str) -> Option<InferredValue> {
        self.parameter_facts.get(&(id.clone(), name.to_string())).cloned()
    }

    /// Resolves a class name in its module, with the stable core namespace as a fallback.
    pub fn class_for_name(&self, uri: &Url, name: &str) -> Option<ClassId> {
        let module = ModuleId::from_uri(uri);
        let classes = self.classes.iter().map(|(k, v)| (k.clone(), (**v).clone())).collect::<BTreeMap<_, _>>();
        resolve_named_class(&classes, &self.graph, &module, name)
    }

    /// Returns the class whose declaration contains a byte offset in `uri`.
    pub fn class_at(&self, uri: &Url, offset: usize) -> Option<ClassId> {
        let module = ModuleId::from_uri(uri);
        self.files
            .get(&module)?
            .source
            .surface
            .classes
            .values()
            .find(|class| class.source_range.contains(offset))
            .map(|class| class.id.clone())
    }

    /// Returns the source-authored class whose name range contains `offset`.
    pub fn class_name_at(&self, uri: &Url, offset: usize) -> Option<ClassSurface> {
        let module = ModuleId::from_uri(uri);
        self.files
            .get(&module)?
            .source
            .surface
            .classes
            .values()
            .find(|class| class.name_range.contains(offset))
            .cloned()
    }

    /// Returns the declared callable enclosing a source offset.
    pub fn member_at(&self, uri: &Url, offset: usize) -> Option<MemberSurface> {
        let module = ModuleId::from_uri(uri);
        self.files
            .get(&module)?
            .source
            .surface
            .classes
            .values()
            .flat_map(|class| class.all_members())
            .find(|member| member.source_range.contains(offset))
            .cloned()
    }

    /// Joins return summaries for a bounded set of receiver candidates.
    pub fn returns_for_callables(&self, ids: impl IntoIterator<Item = CallableId>) -> Option<InferredValue> {
        let classes = self.classes.iter().map(|(k, v)| (k.clone(), (**v).clone())).collect::<BTreeMap<_, _>>();
        let summaries = self.summaries.iter().map(|(k, v)| (k.clone(), (**v).clone())).collect::<BTreeMap<_, _>>();
        ids.into_iter()
            .filter_map(|id| return_for_callable(&classes, &summaries, &id))
            .reduce(|left, right| left.join(&right))
    }

    /// Returns the fact visible for a local binding at a byte offset.
    pub fn binding_at(&self, uri: &Url, name: &str, offset: usize) -> Option<InferredValue> {
        let module = ModuleId::from_uri(uri);
        let file = self.files.get(&module)?;
        let binding = match file.source.scopes.resolve(file.source.scopes.scope_at(offset), name, offset) {
            NameResolution::Binding(binding) => binding,
            _ => return None,
        };
        file.local_facts.value_before(binding, offset).cloned()
    }

    /// Infers a parsed receiver expression against the coherent current semantic snapshot.
    pub fn infer_expression(&self, uri: &Url, expr: &phalcom_ast::ast::Expr, offset: usize) -> InferredValue {
        let module = ModuleId::from_uri(uri);
        let classes = self.classes.iter().map(|(k, v)| (k.clone(), (**v).clone())).collect::<BTreeMap<_, _>>();
        let summaries = self.summaries.iter().map(|(k, v)| (k.clone(), (**v).clone())).collect::<BTreeMap<_, _>>();

        let mut environment = BTreeMap::new();
        let known_classes = |name: &str| resolve_named_class(&classes, &self.graph, &module, name);
        let callable_return = |id: &CallableId| return_for_callable(&classes, &summaries, id);
        let field_value = |class: &ClassId, name: &str, side: DispatchSide| {
            let mut current = Some(class.clone());
            let mut seen = BTreeSet::new();
            while let Some(owner) = current {
                if !seen.insert(owner.clone()) {
                    break;
                }
                if let Some(value) = self.field_facts.get(&FieldId {
                    owner: owner.clone(),
                    name: name.to_string(),
                    side,
                }) {
                    return Some(value.clone());
                }
                current = self.classes.get(&owner).and_then(|surface| surface.superclass.clone());
            }
            None
        };
        let current_class = self
            .files
            .get(&module)
            .and_then(|file| file.source.surface.classes.values().find(|class| class.source_range.contains(offset)))
            .map(|class| class.id.clone());
        if let Some(class) = current_class.as_ref() {
            if let Some(file) = self.files.get(&module) {
                if let Some(member) = file
                    .source
                    .surface
                    .classes
                    .get(class)
                    .and_then(|class| class.all_members().find(|member| member.source_range.contains(offset)))
                {
                    for param in &member.params {
                        if let Some(value) = self.parameter_facts.get(&(member.callable.clone(), param.name.clone())) {
                            environment.insert(param.name.clone(), value.clone());
                        }
                    }
                }
            }
        }
        let dispatch_side = current_class.as_ref().and_then(|class| {
            self.files
                .get(&module)
                .and_then(|file| file.source.surface.classes.get(class))
                .and_then(|surface| surface.all_members().find(|member| member.source_range.contains(offset)))
                .map(|member| member.side)
        });
        let local_facts = self.files.get(&module).map(|file| &file.local_facts);
        let scopes = self.files.get(&module).map(|file| &file.source.scopes);
        let resolver = DispatchResolver::new(&classes);
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
        let member_surface = |id: &CallableId| classes.get(&id.owner).and_then(|class| class.member_by_id(id).cloned());
        let contains_class = |class: &ClassId| resolver.contains_class(class);
        let is_same_or_subclass = |child: &ClassId, ancestor: &ClassId| super::is_same_or_subclass(&classes, child, ancestor);
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
            member_surface: &member_surface,
            contains_class: &contains_class,
            is_same_or_subclass: &is_same_or_subclass,
        };
        analyze_expr(expr, &context)
    }

    /// Returns current import edges for one module.
    pub fn imports(&self, uri: &Url) -> Vec<ImportEdge> {
        let module = ModuleId::from_uri(uri);
        self.graph.imports(&module).to_vec()
    }

    /// Returns a coherent revision/generation stamp for one file.
    pub fn stamp(&self, uri: &Url) -> Option<SnapshotStamp> {
        let module = ModuleId::from_uri(uri);
        Some(SnapshotStamp {
            revision: self.files.get(&module)?.revision,
            generation: self.generation,
        })
    }
}
