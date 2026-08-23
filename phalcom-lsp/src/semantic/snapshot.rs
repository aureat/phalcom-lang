//! Immutable published semantic snapshot for zero-blocking LSP queries.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::Url;

use phalcom_semantic::{FormalPresentation, TypePresenter};

use super::analyzer::{AnalysisContext, analyze_expr};
use super::callable::{CallableSignature, CallableSummary, ParameterSignature};
use super::dispatch::{DispatchReceiver, DispatchResolver};
use super::facts::InferredValue;
use super::ids::{CORE_MODULE_URI, CallableId, ClassId, DispatchSide, DocumentModuleMap, FieldId, ModuleId};
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

/// Advisory runtime-shape semantic snapshot used by existing editor queries.
pub type AdvisorySemanticSnapshot = SemanticSnapshot;
/// Static type/kind semantic snapshot produced by `phalcom-semantic`.
pub type StaticSemanticSnapshot = phalcom_semantic::SemanticSnapshot;

/// Compiler-owned callable signature projected for editor presentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormalCallablePresentation {
    /// Formal parameter states in declaration order.
    pub parameters: Vec<FormalPresentation>,
    /// Formal return state.
    pub return_type: FormalPresentation,
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
    /// Published document-to-module mapping used at request boundaries.
    pub documents: Arc<DocumentModuleMap>,
    /// Whole-workspace static semantic snapshot published by the semantic tower.
    pub static_snapshot: Option<Arc<StaticSemanticSnapshot>>,
}

impl SemanticSnapshot {
    /// Returns the current semantic generation.
    pub fn generation(&self) -> SemanticGeneration {
        self.generation
    }

    /// Resolves an editor URI to the published module identity without doing
    /// filesystem work on the request path.
    pub fn module_for_uri(&self, uri: &Url) -> Option<&ModuleId> {
        self.documents.lsp_for_uri(uri).filter(|module| self.files.contains_key(*module))
    }

    /// Returns one published file by its resolved module identity.
    pub fn file(&self, module: &ModuleId) -> Option<&FileSemanticSnapshot> {
        self.files.get(module).map(Arc::as_ref)
    }

    /// Returns an immutable clone of one file's semantic snapshot.
    pub fn file_snapshot(&self, uri: &Url) -> Option<FileSemanticSnapshot> {
        let module = self.module_for_uri(uri)?;
        self.files.get(module).map(|f| (**f).clone())
    }

    /// Returns the file revision stamp for one module if present.
    pub fn file_revision(&self, module: &ModuleId) -> Option<super::facts::FileRevision> {
        self.files.get(module).map(|f| f.revision)
    }

    /// Returns the exact semantic occurrence covering one source offset.
    pub fn occurrence_at(&self, uri: &Url, offset: usize) -> Option<SemanticOccurrence> {
        let module = self.module_for_uri(uri)?;
        self.files.get(module).and_then(|file| file.occurrences.occurrence_at(offset).cloned())
    }

    /// Returns all references to a `SemanticTarget` in the workspace.
    pub fn references_for_target(&self, uri: &Url, target: &SemanticTarget) -> Vec<(Url, SourceRange, OccurrenceRole)> {
        match target {
            SemanticTarget::Binding(_) => {
                let Some(module) = self.module_for_uri(uri) else { return Vec::new() };
                self.files
                    .get(module)
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
                    if let Some(file_uri) = self.documents.uri_for_lsp(module_id) {
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
        let Some(module) = self.module_for_uri(uri) else { return Vec::new() };
        self.files
            .get(module)
            .map(|file| file.source.scopes.visible_bindings_at(offset))
            .unwrap_or_default()
    }

    /// Returns one binding's declaration metadata from a file-local identity.
    pub fn binding_info(&self, uri: &Url, binding: BindingId) -> Option<BindingInfo> {
        let module = self.module_for_uri(uri)?;
        self.files.get(module).and_then(|file| file.source.scopes.bindings.get(&binding).cloned())
    }

    /// Returns one class surface by module-qualified identity.
    pub fn class_surface(&self, id: &ClassId) -> Option<&ClassSurface> {
        self.classes.get(id).map(Arc::as_ref)
    }

    /// Returns one member surface by its complete callable identity.
    pub fn member_surface(&self, callable: &CallableId) -> Option<&MemberSurface> {
        self.classes.get(&callable.owner).and_then(|surface| surface.member_by_id(callable))
    }

    /// Resolves one receiver-qualified member, including inherited members.
    pub fn receiver_member(&self, class: &ClassId, selector: &str, side: DispatchSide) -> Option<MemberSurface> {
        let receiver = match side {
            DispatchSide::Instance => DispatchReceiver::Instance(class.clone()),
            DispatchSide::Class => DispatchReceiver::ClassObject(class.clone()),
        };
        let resolver = DispatchResolver::new(self.classes.as_ref());
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

    /// Returns the static module identity for a document URI if analyzed.
    pub fn formal_static_module(&self, uri: &Url) -> Option<&phalcom_modules::ModuleId> {
        self.documents.get_by_uri(uri)
    }

    /// Returns the whole-workspace static semantic snapshot if available.
    pub fn formal_static_snapshot(&self) -> Option<&Arc<StaticSemanticSnapshot>> {
        self.static_snapshot.as_ref()
    }

    /// Returns a formal callable analysis for a callable identity if present.
    pub fn formal_callable_analysis(&self, callable: &phalcom_semantic::identity::CallableId) -> Option<&Arc<phalcom_semantic::checker::CallableAnalysis>> {
        self.static_snapshot.as_ref()?.callable_analyses.get(callable)
    }

    /// Looks up formal binding knowledge, preserving non-ready states.
    pub fn formal_binding_presentation_at(&self, uri: &Url, name: &str, offset: usize) -> Option<FormalPresentation> {
        let static_snap = self.static_snapshot.as_ref()?;
        let static_mod = self.formal_static_module(uri)?;
        let presenter = TypePresenter::new(&static_snap.store);

        for (callable_id, analysis) in static_snap.callable_analyses.iter() {
            if &callable_id.owner.module == static_mod && analysis.body_range.contains(offset) {
                for state in analysis.bindings.values() {
                    if state.name == name && (state.range.contains(offset) || state.range.end == offset || state.range.start <= offset) {
                        return Some(presenter.present_knowledge(&state.current));
                    }
                }
                for expr in analysis.expressions.values() {
                    if expr.range.contains(offset) {
                        return Some(presenter.present_expression(expr));
                    }
                }
            }
        }
        None
    }

    /// Looks up a known formal binding type for compatibility with receiver resolution.
    pub fn formal_binding_type_at(&self, uri: &Url, name: &str, offset: usize) -> Option<String> {
        match self.formal_binding_presentation_at(uri, name, offset)? {
            FormalPresentation::Known(ty) => Some(ty),
            _ => None,
        }
    }

    /// Looks up formal expression knowledge, preserving non-ready states.
    pub fn formal_expression_presentation_at(&self, uri: &Url, offset: usize) -> Option<FormalPresentation> {
        let static_snap = self.static_snapshot.as_ref()?;
        let static_mod = self.formal_static_module(uri)?;
        let presenter = TypePresenter::new(&static_snap.store);

        for (callable_id, analysis) in static_snap.callable_analyses.iter() {
            if &callable_id.owner.module == static_mod && analysis.body_range.contains(offset) {
                for expr in analysis.expressions.values() {
                    if expr.range.contains(offset) {
                        return Some(presenter.present_expression(expr));
                    }
                }
            }
        }
        None
    }

    /// Looks up a known formal expression type for compatibility with receiver resolution.
    pub fn formal_expression_type_at(&self, uri: &Url, offset: usize) -> Option<String> {
        match self.formal_expression_presentation_at(uri, offset)? {
            FormalPresentation::Known(ty) => Some(ty),
            _ => None,
        }
    }

    /// Looks up the compiler-owned formal return type of one LSP callable identity.
    pub fn formal_callable_return_presentation(&self, callable: &CallableId) -> Option<FormalPresentation> {
        Some(self.formal_callable_presentation(callable)?.return_type)
    }

    /// Looks up the compiler-owned formal parameter and return states of one callable.
    pub fn formal_callable_presentation(&self, callable: &CallableId) -> Option<FormalCallablePresentation> {
        let static_snap = self.static_snapshot.as_ref()?;
        let uri = self.documents.uri_for_lsp(&callable.owner.module)?;
        let module = self.formal_static_module(uri)?;
        let side = match callable.side {
            DispatchSide::Instance => phalcom_semantic::DispatchSide::Instance,
            DispatchSide::Class => phalcom_semantic::DispatchSide::Class,
        };
        let signature = static_snap.callable_signatures.iter().find_map(|(_, signature)| {
            (signature.owner.module == *module
                && signature.owner.name.as_ref() == callable.owner.name
                && signature.selector.encode() == callable.selector
                && signature.side == side)
                .then_some(signature)
        })?;
        let presenter = TypePresenter::new(&static_snap.store);
        let parameters = signature
            .parameters
            .iter()
            .map(|parameter| match &parameter.ty {
                phalcom_semantic::types::TypeTerm::Canonical(ty) => FormalPresentation::Known(presenter.present_type(*ty)),
                phalcom_semantic::types::TypeTerm::SelfType(_) | phalcom_semantic::types::TypeTerm::Infer(_) => FormalPresentation::Unknown,
            })
            .collect();
        let return_type = match &signature.return_type {
            phalcom_semantic::types::TypeTerm::Canonical(ty) => FormalPresentation::Known(presenter.present_type(*ty)),
            phalcom_semantic::types::TypeTerm::SelfType(_) | phalcom_semantic::types::TypeTerm::Infer(_) => FormalPresentation::Unknown,
        };
        Some(FormalCallablePresentation { parameters, return_type })
    }

    /// Returns a source callable summary from the current semantic generation.
    pub fn callable_summary(&self, id: &CallableId) -> Option<&CallableSummary> {
        self.summaries.get(id).map(Arc::as_ref)
    }

    /// Returns a callable's target-specific return summary.
    pub fn return_for_callable(&self, id: &CallableId) -> Option<InferredValue> {
        return_for_callable(self.classes.as_ref(), self.summaries.as_ref(), id)
    }

    /// Returns a callable's structured signature with resolved parameters and return value.
    pub fn callable_signature(&self, id: &CallableId) -> Option<CallableSignature> {
        let surface = self.member_surface(id)?;
        let returns = self.return_for_callable(id).unwrap_or_else(InferredValue::unknown);
        let summary = self.callable_summary(id);
        let parameters = surface
            .params
            .iter()
            .enumerate()
            .map(|(idx, param)| {
                let value = self
                    .parameter_at(id, &param.name)
                    .or_else(|| summary.and_then(|s| s.params.get(idx).cloned()))
                    .unwrap_or_else(InferredValue::unknown);
                ParameterSignature {
                    name: param.name.clone(),
                    label: param.label.clone(),
                    rest_mode: param.rest_mode,
                    value,
                }
            })
            .collect();
        Some(CallableSignature {
            callable: id.clone(),
            parameters,
            returns,
        })
    }

    /// Resolves an inferred field fact for a class and side.
    pub fn field_value(&self, class: &ClassId, name: &str, side: DispatchSide) -> Option<InferredValue> {
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
    }

    /// Returns the joined call-site fact observed for one callable parameter.
    pub fn parameter_at(&self, id: &CallableId, name: &str) -> Option<InferredValue> {
        self.parameter_facts.get(&(id.clone(), name.to_string())).cloned()
    }

    /// Resolves a class name in its module, with the stable core namespace as a fallback.
    pub fn class_for_name(&self, uri: &Url, name: &str) -> Option<ClassId> {
        let module = self.module_for_uri(uri)?;
        resolve_named_class(self.classes.as_ref(), &self.graph, module, name)
    }

    /// Returns the class whose declaration contains a byte offset in `uri`.
    pub fn class_at(&self, uri: &Url, offset: usize) -> Option<ClassId> {
        let module = self.module_for_uri(uri)?;
        self.files
            .get(module)?
            .source
            .surface
            .classes
            .values()
            .find(|class| class.source_range.contains(offset))
            .map(|class| class.id.clone())
    }

    /// Returns the source-authored class whose name range contains `offset`.
    pub fn class_name_at(&self, uri: &Url, offset: usize) -> Option<ClassSurface> {
        let module = self.module_for_uri(uri)?;
        self.files
            .get(module)?
            .source
            .surface
            .classes
            .values()
            .find(|class| class.name_range.contains(offset))
            .cloned()
    }

    /// Returns the declared callable enclosing a source offset.
    pub fn member_at(&self, uri: &Url, offset: usize) -> Option<MemberSurface> {
        let module = self.module_for_uri(uri)?;
        self.files
            .get(module)?
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
        ids.into_iter()
            .filter_map(|id| return_for_callable(self.classes.as_ref(), self.summaries.as_ref(), &id))
            .reduce(|left, right| left.join(&right))
    }

    /// Returns the fact visible for a local binding at a byte offset.
    pub fn binding_at(&self, uri: &Url, name: &str, offset: usize) -> Option<InferredValue> {
        let module = self.module_for_uri(uri)?;
        let file = self.files.get(module)?;
        let binding = match file.source.scopes.resolve(file.source.scopes.scope_at(offset), name, offset) {
            NameResolution::Binding(binding) => binding,
            _ => return None,
        };
        file.local_facts.value_before(binding, offset).cloned()
    }

    /// Infers a parsed receiver expression against the coherent current semantic snapshot.
    pub fn infer_expression(&self, uri: &Url, expr: &phalcom_ast::ast::Expr, offset: usize) -> InferredValue {
        let fallback_module = ModuleId::new(uri.to_string());
        let module = self.module_for_uri(uri).unwrap_or(&fallback_module);

        let mut environment = BTreeMap::new();
        let known_classes = |name: &str| resolve_named_class(self.classes.as_ref(), &self.graph, module, name);
        let callable_return = |id: &CallableId| return_for_callable(self.classes.as_ref(), self.summaries.as_ref(), id);
        let field_value = |class: &ClassId, name: &str, side: DispatchSide| self.field_value(class, name, side);
        let current_class = self
            .files
            .get(module)
            .and_then(|file| file.source.surface.classes.values().find(|class| class.source_range.contains(offset)))
            .map(|class| class.id.clone());
        if let Some(class) = current_class.as_ref() {
            if let Some(file) = self.files.get(module) {
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
                .get(module)
                .and_then(|file| file.source.surface.classes.get(class))
                .and_then(|surface| surface.all_members().find(|member| member.source_range.contains(offset)))
                .map(|member| member.side)
        });
        let local_facts = self.files.get(module).map(|file| &file.local_facts);
        let scopes = self.files.get(module).map(|file| &file.source.scopes);
        let resolver = DispatchResolver::new(self.classes.as_ref());
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
        let family_resolver =
            |receiver: &DispatchReceiver, pattern: &phalcom_common::selector::SelectorPattern| resolver.capture_method_family(receiver, pattern);
        let member_surface = |id: &CallableId| resolver.member(id).cloned();
        let contains_class = |class: &ClassId| resolver.contains_class(class);
        let is_same_or_subclass = |child: &ClassId, ancestor: &ClassId| super::is_same_or_subclass(self.classes.as_ref(), child, ancestor);
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
            family_resolver: &family_resolver,
            member_surface: &member_surface,
            contains_class: &contains_class,
            is_same_or_subclass: &is_same_or_subclass,
        };
        analyze_expr(expr, &context)
    }

    /// Returns current import edges for one module.
    pub fn imports(&self, uri: &Url) -> Vec<ImportEdge> {
        let Some(module) = self.module_for_uri(uri) else { return Vec::new() };
        self.graph.imports(module).to_vec()
    }

    /// Returns a coherent revision/generation stamp for one file.
    pub fn stamp(&self, uri: &Url) -> Option<SnapshotStamp> {
        let module = self.module_for_uri(uri)?;
        Some(SnapshotStamp {
            revision: self.files.get(module)?.revision,
            generation: self.generation,
        })
    }
}
