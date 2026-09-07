//! Compiler-owned source identity and source-site indexing.
//!
//! This module owns source locations, lexical source identity, occurrences,
//! and their attachment to canonical semantic products. It intentionally does
//! not depend on LSP identity or protocol types.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::checker::CallableAnalysis;
use crate::db::ProductFingerprint;
use crate::identity::{BindingId, CallableId, DeclarationId, ExpressionId, FieldId, ModuleId, SemanticTargetId, SourceSiteId};
use crate::source_index::interval::{RangeEntry, RangeIndex};

pub mod builder;
pub mod interval;
pub mod occurrence;
pub mod reference;
pub mod scope;
pub mod site;
pub mod symbol;

pub use builder::{SourceIndexContext, build_source_scope_index, resolve_type_reference_targets};
pub use occurrence::{OccurrenceHint, OccurrenceIndex, OccurrenceKind, OccurrenceRole, OccurrenceView, SemanticOccurrence};
pub use reference::{ModuleReferenceContribution, ReferenceIndex, TargetReferenceSet};
pub use scope::{
    CallableSourceInfo, DeclarationSourceInfo, FieldSourceInfo, ImportBindingOrigin, SourceBindingInfo, SourceBindingKind, SourceCallableKind,
    SourceDeclarationKind, SourceNameResolution, SourceReceiverKind, SourceScope, SourceScopeId, SourceScopeIndex,
};
pub use site::{SourceSite, SourceSiteKind};
pub use symbol::{EditorSymbolKind, WorkspaceSymbolEntry, WorkspaceSymbolId, WorkspaceSymbolIndex};

/// Formal source-site attachment failure. Construction fails closed instead of
/// selecting an arbitrary same-name or same-range candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceAttachmentError {
    MissingModule(ModuleId),
    AmbiguousBinding { callable: CallableId, binding: BindingId },
    MissingBinding { callable: CallableId, binding: BindingId },
}

/// Deterministic performance and change statistics published by the source index.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SourceIndexUpdateStats {
    pub source_modules_rebuilt: usize,
    pub source_modules_reused: usize,
    pub source_modules_retired: usize,
    pub presentation_only_modules: usize,

    pub reference_contributions_replaced: usize,
    pub reference_targets_touched: usize,
    pub reference_sites_added: usize,
    pub reference_sites_removed: usize,

    pub formal_modules_rebuilt: usize,
    pub formal_modules_reused: usize,
    pub formal_modules_retired: usize,

    pub workspace_symbol_contributions_replaced: usize,
    pub workspace_symbol_entries_added: usize,
    pub workspace_symbol_entries_removed: usize,

    /// Retained source/import units inspected by broad delta discovery.
    pub source_workspace_scan_units: usize,
    /// Reverse-reference units inspected by a broad traversal instead of exact targets.
    pub reference_workspace_scan_units: usize,
    /// Callable/formal products inspected to discover one module's projection.
    pub formal_workspace_scan_units: usize,
}

impl SourceIndexUpdateStats {
    /// Records broad source discovery work without performing any discovery.
    #[doc(hidden)]
    pub fn record_source_workspace_scan(&mut self, units: usize) {
        self.source_workspace_scan_units = self.source_workspace_scan_units.saturating_add(units);
    }

    /// Records broad reference discovery work without performing any discovery.
    #[doc(hidden)]
    pub fn record_reference_workspace_scan(&mut self, units: usize) {
        self.reference_workspace_scan_units = self.reference_workspace_scan_units.saturating_add(units);
    }

    /// Records broad formal discovery work without performing any discovery.
    #[doc(hidden)]
    pub fn record_formal_workspace_scan(&mut self, units: usize) {
        self.formal_workspace_scan_units = self.formal_workspace_scan_units.saturating_add(units);
    }
}

/// Formal products attached to one canonical callable identity.
#[derive(Clone, Debug)]
pub struct CallableSourceAttachment {
    pub callable: CallableId,
    pub expression_sites: Arc<[SourceSite]>,
    pub formal_bindings: BTreeMap<BindingId, SourceSiteId>,
    pub formal_expressions: BTreeMap<ExpressionId, SourceSiteId>,
    pub exact_targets: BTreeMap<SourceSiteId, SemanticTargetId>,
}

impl CallableSourceAttachment {
    /// Attaches checker products to unique compiler-owned declaration sites.
    pub fn from_analysis(callable: CallableId, scopes: &SourceScopeIndex, analysis: &CallableAnalysis) -> Result<Self, Box<SourceAttachmentError>> {
        let (attachment, incidents) = Self::from_analysis_with_incidents(callable, scopes, analysis);
        incidents.into_iter().next().map_or(Ok(attachment), |error| Err(Box::new(error)))
    }

    /// Builds all exact expression attachments while retaining binding failures
    /// as non-fatal incidents. A missing binding must not erase independently
    /// attachable call/expression products for the same callable.
    pub fn from_analysis_with_incidents(callable: CallableId, scopes: &SourceScopeIndex, analysis: &CallableAnalysis) -> (Self, Vec<SourceAttachmentError>) {
        let mut formal_bindings = BTreeMap::new();
        let mut incidents = Vec::new();
        let mut current_bindings = scopes
            .bindings
            .values()
            .filter(|binding| binding.declaration_site.owner == crate::identity::SourceOwner::Callable(callable.clone()))
            .collect::<Vec<_>>();
        current_bindings.sort_by_key(|binding| binding.declaration_site.local);
        let mut states = analysis.bindings.values().collect::<Vec<_>>();
        states.sort_by_key(|state| state.binding);
        let rebase_ranges = scopes
            .callable_body_ranges
            .get(&callable)
            .is_some_and(|current| *current != analysis.body_range);
        let current_bindings = (rebase_ranges && current_bindings.len() == states.len()).then_some(current_bindings);
        for (offset, state) in states.into_iter().enumerate() {
            if let Some(parameter) = &state.parameter {
                if let Some(site) = scopes
                    .callable_sources
                    .get(&parameter.callable)
                    .and_then(|source| source.parameter_sites.get(parameter))
                    .cloned()
                {
                    formal_bindings.insert(state.binding, site);
                    continue;
                }
                incidents.push(SourceAttachmentError::MissingBinding {
                    callable: callable.clone(),
                    binding: state.binding,
                });
                continue;
            }
            if let Some(binding) = current_bindings.as_ref().and_then(|bindings| bindings.get(offset)) {
                formal_bindings.insert(state.binding, binding.declaration_site.clone());
                continue;
            }
            let candidates = scopes
                .bindings
                .values()
                .filter(|binding| {
                    binding.name.as_ref() == state.name
                        && binding.declaration_range == state.range
                        && binding.declaration_site.owner == crate::identity::SourceOwner::Callable(callable.clone())
                })
                .map(|binding| binding.declaration_site.clone())
                .collect::<Vec<_>>();
            let site = match candidates.as_slice() {
                [site] => site.clone(),
                [] => {
                    incidents.push(SourceAttachmentError::MissingBinding {
                        callable: callable.clone(),
                        binding: state.binding,
                    });
                    continue;
                }
                _ => {
                    incidents.push(SourceAttachmentError::AmbiguousBinding {
                        callable: callable.clone(),
                        binding: state.binding,
                    });
                    continue;
                }
            };
            formal_bindings.insert(state.binding, site.clone());
        }

        let mut expressions = analysis.expressions.values().collect::<Vec<_>>();
        expressions.sort_by_key(|expression| expression.id);
        let current_expression_ranges = rebase_ranges
            .then(|| {
                scopes
                    .callable_expression_ranges
                    .get(&callable)
                    .filter(|ranges| ranges.len() == expressions.len())
            })
            .flatten();
        let next_local = scopes
            .sites
            .keys()
            .filter_map(|site| match &site.owner {
                crate::identity::SourceOwner::Callable(owner) if owner == &callable => Some(site.local.0),
                _ => None,
            })
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let mut expression_sites = Vec::with_capacity(expressions.len());
        let mut formal_expressions = BTreeMap::new();
        let mut exact_targets = BTreeMap::new();
        for (offset, expression) in expressions.into_iter().enumerate() {
            let site = SourceSite::new(
                crate::identity::SourceOwner::Callable(callable.clone()),
                crate::identity::SourceSiteLocalId(next_local.saturating_add(offset as u32)),
                current_expression_ranges
                    .and_then(|ranges| ranges.get(offset).copied())
                    .unwrap_or(expression.range),
                SourceSiteKind::Expression,
            );
            if let Some(target) = &expression.callable {
                exact_targets.insert(site.id.clone(), SemanticTargetId::Callable(target.clone()));
            }
            formal_expressions.insert(expression.id, site.id.clone());
            expression_sites.push(site);
        }
        for site in formal_bindings.values() {
            exact_targets.insert(site.clone(), SemanticTargetId::Binding(site.clone()));
        }
        (
            Self {
                callable,
                expression_sites: Arc::from(expression_sites),
                formal_bindings,
                formal_expressions,
                exact_targets,
            },
            incidents,
        )
    }

    /// Finds source site for one formal binding.
    pub fn source_site_for_binding(&self, binding: BindingId) -> Option<&SourceSiteId> {
        self.formal_bindings.get(&binding)
    }

    /// Finds source site for one formal expression.
    pub fn source_site_for_expression(&self, expression: ExpressionId) -> Option<&SourceSiteId> {
        self.formal_expressions.get(&expression)
    }
}

/// All source-owned semantic products for one immutable compiler snapshot.
#[derive(Clone, Debug, Default)]
pub struct SourceSemanticIndex {
    modules: im::OrdMap<ModuleId, Arc<ModuleSourceIndex>>,
    references: Arc<ReferenceIndex>,
    workspace_symbols: Arc<WorkspaceSymbolIndex>,
    /// Non-fatal attachment incidents grouped by their owning module.
    incidents_by_module: im::OrdMap<ModuleId, Arc<[SourceAttachmentError]>>,
    stats: SourceIndexUpdateStats,
}

/// Source structure, exact occurrences, and formal attachments for one module.
#[derive(Clone, Debug)]
pub struct ModuleSourceIndex {
    pub structure: Arc<SourceScopeIndex>,
    pub occurrences: Arc<OccurrenceIndex>,
    baseline_occurrences: Arc<OccurrenceIndex>,
    /// AST expression sites owned by this source shard. These are separate
    /// from token occurrences so chained top-level expressions remain
    /// queryable without redispatch or request-time AST analysis.
    pub expression_sites: Arc<[SourceSite]>,
    expression_intervals: RangeIndex<usize>,
    pub attachments: BTreeMap<CallableId, Arc<CallableSourceAttachment>>,
    reference_contribution: Arc<ModuleReferenceContribution>,
    workspace_symbols: Arc<[WorkspaceSymbolEntry]>,
}

impl ModuleSourceIndex {
    /// Builds one final source shard after attaching every formal callable
    /// product owned by the module. Publication must call this once per
    /// rebuilt module; attaching callables one at a time republishes the
    /// module's reverse-reference and symbol contributions repeatedly.
    pub(crate) fn from_scope_index_with_formal(
        mut structure: SourceScopeIndex,
        program: &crate::source::ParsedModuleUnit,
        context: Option<&crate::source_index::builder::SourceIndexContext>,
        analyses: &[&CallableAnalysis],
    ) -> (Self, Vec<SourceAttachmentError>) {
        let occurrences = OccurrenceIndex::from_program_with_context(&mut structure, &program.program, context);
        let mut attachments = BTreeMap::new();
        let mut incidents = Vec::new();
        for analysis in analyses {
            let (attachment, mut attachment_incidents) =
                CallableSourceAttachment::from_analysis_with_incidents(analysis.callable.clone(), &structure, analysis);
            attachments.insert(analysis.callable.clone(), Arc::new(attachment));
            incidents.append(&mut attachment_incidents);
        }
        let shard = Self::from_structure_and_attachments(structure, occurrences, attachments);
        (shard, incidents)
    }

    fn from_structure_and_attachments(
        structure: SourceScopeIndex,
        baseline_occurrences: OccurrenceIndex,
        attachments: BTreeMap<CallableId, Arc<CallableSourceAttachment>>,
    ) -> Self {
        let mut all = baseline_occurrences.all().to_vec();
        let mut exact_targets = structure.targets.clone();
        for occurrence in &all {
            if let Some(target) = baseline_occurrences.target_for(&occurrence.site) {
                exact_targets.insert(occurrence.site.clone(), target.clone());
            }
        }
        for attachment in attachments.values() {
            for site in attachment.expression_sites.iter() {
                let (kind, role) = if attachment.exact_targets.contains_key(&site.id) {
                    (OccurrenceKind::Member, OccurrenceRole::Call)
                } else {
                    (OccurrenceKind::Operator, OccurrenceRole::Reference)
                };
                all.push(SemanticOccurrence {
                    site: site.id.clone(),
                    range: site.range,
                    kind,
                    role,
                    owner: site.id.owner.clone(),
                    hint: None,
                });
            }
            exact_targets.extend(attachment.exact_targets.clone());
        }
        for (formal_site, target) in attachments.values().flat_map(|attachment| attachment.exact_targets.iter()) {
            let Some(formal_source) = attachments
                .values()
                .flat_map(|attachment| attachment.expression_sites.iter())
                .find(|site| &site.id == formal_site)
            else {
                continue;
            };
            for occurrence in &all {
                if occurrence.role == OccurrenceRole::Call
                    && formal_source.range.start <= occurrence.range.start
                    && occurrence.range.end <= formal_source.range.end
                {
                    exact_targets.insert(occurrence.site.clone(), target.clone());
                }
            }
        }
        let occurrences = OccurrenceIndex::new(all, exact_targets);
        let (expression_sites, expression_intervals) = expression_products(&structure, &attachments);
        let reference_contribution = Arc::new(ModuleReferenceContribution::from_module_index(
            &structure,
            occurrences.all(),
            &|site| occurrences.target_for(site).cloned(),
            &attachments,
        ));
        let workspace_symbols = build_module_workspace_symbols(&structure);
        Self {
            structure: Arc::new(structure),
            baseline_occurrences: Arc::new(baseline_occurrences),
            occurrences: Arc::new(occurrences),
            expression_sites,
            expression_intervals,
            attachments,
            reference_contribution,
            workspace_symbols,
        }
    }

    pub fn new(structure: SourceScopeIndex, occurrences: OccurrenceIndex, attachments: BTreeMap<CallableId, Arc<CallableSourceAttachment>>) -> Self {
        let (expression_sites, expression_intervals) = expression_products(&structure, &attachments);
        let reference_contribution = Arc::new(ModuleReferenceContribution::from_module_index(
            &structure,
            occurrences.all(),
            &|site| occurrences.target_for(site).cloned(),
            &attachments,
        ));
        let workspace_symbols = build_module_workspace_symbols(&structure);
        let occurrences = Arc::new(occurrences);
        Self {
            structure: Arc::new(structure),
            baseline_occurrences: occurrences.clone(),
            occurrences,
            expression_sites,
            expression_intervals,
            attachments,
            reference_contribution,
            workspace_symbols,
        }
    }

    /// Returns the innermost compiler-owned AST expression site at `offset`.
    pub fn expression_site_at(&self, offset: usize) -> Option<&SourceSite> {
        self.expression_intervals.value_at(offset).and_then(|index| self.expression_sites.get(index))
    }

    /// Returns this module's reference contribution.
    pub fn reference_contribution(&self) -> &Arc<ModuleReferenceContribution> {
        &self.reference_contribution
    }

    /// Returns this module's workspace symbol entries.
    pub fn workspace_symbols(&self) -> &[WorkspaceSymbolEntry] {
        &self.workspace_symbols
    }
}

fn build_module_workspace_symbols(structure: &SourceScopeIndex) -> Arc<[WorkspaceSymbolEntry]> {
    let mut symbols = Vec::new();
    for decl in structure.declaration_sources.values() {
        symbols.push(WorkspaceSymbolEntry {
            id: WorkspaceSymbolId {
                target: SemanticTargetId::Declaration(decl.id.clone()),
                site: decl.declaration_site.clone(),
            },
            name: decl.name.clone(),
            normalized_name: decl.name.to_lowercase().into_boxed_str(),
            target: SemanticTargetId::Declaration(decl.id.clone()),
            declaration_site: decl.declaration_site.clone(),
            kind: match decl.kind {
                SourceDeclarationKind::Class => EditorSymbolKind::Class,
                SourceDeclarationKind::Enum => EditorSymbolKind::Enum,
                SourceDeclarationKind::TypeAlias => EditorSymbolKind::TypeAlias,
            },
            container_name: None,
        });
    }
    for callable in structure.callable_sources.values() {
        let name = callable.id.selector.encode().into_boxed_str();
        symbols.push(WorkspaceSymbolEntry {
            id: WorkspaceSymbolId {
                target: SemanticTargetId::Callable(callable.id.clone()),
                site: callable.declaration_site.clone(),
            },
            name: name.clone(),
            normalized_name: name.to_lowercase().into_boxed_str(),
            target: SemanticTargetId::Callable(callable.id.clone()),
            declaration_site: callable.declaration_site.clone(),
            kind: EditorSymbolKind::Callable,
            container_name: Some(callable.id.owner.name.clone()),
        });
    }
    for field in structure.field_sources.values() {
        let name = field.id.name.clone();
        symbols.push(WorkspaceSymbolEntry {
            id: WorkspaceSymbolId {
                target: SemanticTargetId::Field(field.id.clone()),
                site: field.declaration_site.clone(),
            },
            name: name.clone(),
            normalized_name: name.to_lowercase().into_boxed_str(),
            target: SemanticTargetId::Field(field.id.clone()),
            declaration_site: field.declaration_site.clone(),
            kind: EditorSymbolKind::Field,
            container_name: Some(field.id.owner.name.clone()),
        });
    }
    for site in structure.sites.values() {
        match &site.kind {
            SourceSiteKind::Variant(id) => {
                let name = id.selector.encode().into_boxed_str();
                symbols.push(WorkspaceSymbolEntry {
                    id: WorkspaceSymbolId {
                        target: SemanticTargetId::Variant(id.clone()),
                        site: site.id.clone(),
                    },
                    name: name.clone(),
                    normalized_name: name.to_lowercase().into_boxed_str(),
                    target: SemanticTargetId::Variant(id.clone()),
                    declaration_site: site.id.clone(),
                    kind: EditorSymbolKind::Variant,
                    container_name: Some(id.owner.name.clone()),
                });
            }
            SourceSiteKind::VariantField(id) => {
                let name = format!("{}:{}", id.variant.selector.encode(), id.index).into_boxed_str();
                symbols.push(WorkspaceSymbolEntry {
                    id: WorkspaceSymbolId {
                        target: SemanticTargetId::VariantField(id.clone()),
                        site: site.id.clone(),
                    },
                    name: name.clone(),
                    normalized_name: name.to_lowercase().into_boxed_str(),
                    target: SemanticTargetId::VariantField(id.clone()),
                    declaration_site: site.id.clone(),
                    kind: EditorSymbolKind::VariantField,
                    container_name: Some(id.variant.owner.name.clone()),
                });
            }
            _ => {}
        }
    }
    symbols.sort_by(|a, b| a.id.cmp(&b.id));
    Arc::from(symbols.into_boxed_slice())
}

fn expression_products(
    structure: &SourceScopeIndex,
    attachments: &BTreeMap<CallableId, Arc<CallableSourceAttachment>>,
) -> (Arc<[SourceSite]>, RangeIndex<usize>) {
    let mut sites = structure
        .sites
        .values()
        .filter(|site| matches!(site.kind, SourceSiteKind::Expression))
        .cloned()
        .collect::<Vec<_>>();
    sites.extend(attachments.values().flat_map(|attachment| attachment.expression_sites.iter().cloned()));
    sites.sort_by_key(|site| (site.range.start, site.range.len(), site.id.clone()));
    let intervals = RangeIndex::new(sites.iter().enumerate().map(|(index, site)| RangeEntry::new(site.range, index, 0)));
    (Arc::from(sites), intervals)
}

/// Separate source-index identities for semantic reuse and editor position
/// publication. Trivia/range movement belongs only to presentation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceIndexFingerprints {
    pub semantic: ProductFingerprint,
    pub presentation: ProductFingerprint,
}

impl ModuleSourceIndex {
    /// Stable semantic fingerprint for source structure, occurrences, and
    /// formal attachments. Range movement remains part of source input while
    /// target and binding identity remain explicit in the hash.
    pub fn fingerprint(&self) -> ProductFingerprint {
        let mut hasher = DefaultHasher::new();
        self.structure.module.hash(&mut hasher);
        for site in self.structure.sites.values() {
            site.id.hash(&mut hasher);
            site.range.start.hash(&mut hasher);
            site.range.end.hash(&mut hasher);
            site.kind.hash(&mut hasher);
        }
        for binding in self.structure.bindings.values() {
            binding.declaration_site.hash(&mut hasher);
            binding.scope.hash(&mut hasher);
            binding.name.hash(&mut hasher);
            binding.kind.hash(&mut hasher);
            binding.declaration_range.start.hash(&mut hasher);
            binding.declaration_range.end.hash(&mut hasher);
            binding.mutable.hash(&mut hasher);
            binding.redeclaration_of.hash(&mut hasher);
        }
        self.structure.receiver_kinds.hash(&mut hasher);
        self.structure.import_origins.hash(&mut hasher);
        for occurrence in self.occurrences.all() {
            occurrence.hash(&mut hasher);
        }
        for (callable, attachment) in &self.attachments {
            callable.hash(&mut hasher);
            for site in attachment.expression_sites.iter() {
                site.hash(&mut hasher);
            }
            attachment.formal_bindings.hash(&mut hasher);
            attachment.formal_expressions.hash(&mut hasher);
            attachment.exact_targets.hash(&mut hasher);
        }
        ProductFingerprint::new(hasher.finish())
    }

    pub fn fingerprints(&self) -> SourceIndexFingerprints {
        let presentation = self.fingerprint();
        let mut hasher = DefaultHasher::new();
        self.structure.module.hash(&mut hasher);
        for binding in self.structure.bindings.values() {
            binding.name.hash(&mut hasher);
            binding.kind.hash(&mut hasher);
            binding.mutable.hash(&mut hasher);
            binding.redeclaration_of.is_some().hash(&mut hasher);
        }
        self.structure.receiver_kinds.hash(&mut hasher);
        self.structure.import_origins.hash(&mut hasher);
        for (index, occurrence) in self.occurrences.all().iter().enumerate() {
            index.hash(&mut hasher);
            occurrence.kind.hash(&mut hasher);
            occurrence.role.hash(&mut hasher);
            occurrence.hint.hash(&mut hasher);
            self.occurrences.target_for(&occurrence.site).hash(&mut hasher);
        }
        for (callable, attachment) in &self.attachments {
            callable.hash(&mut hasher);
            for binding in attachment.formal_bindings.keys() {
                binding.hash(&mut hasher);
            }
            for expression in attachment.formal_expressions.keys() {
                expression.hash(&mut hasher);
            }
            for target in attachment.exact_targets.values() {
                target.hash(&mut hasher);
            }
        }
        SourceIndexFingerprints {
            semantic: ProductFingerprint::new(hasher.finish()),
            presentation,
        }
    }
}

impl SourceSemanticIndex {
    /// Creates an empty source index.
    pub fn empty() -> Self {
        Self {
            modules: im::OrdMap::new(),
            references: Arc::new(ReferenceIndex::new()),
            workspace_symbols: Arc::new(WorkspaceSymbolIndex::new()),
            incidents_by_module: im::OrdMap::new(),
            stats: SourceIndexUpdateStats::default(),
        }
    }

    /// Creates a source index from compiler-owned lexical source structures.
    pub fn from_scope_indices(scopes: BTreeMap<ModuleId, SourceScopeIndex>) -> Self {
        let mut index = Self::empty();
        for (module, structure) in scopes {
            let occurrences = OccurrenceIndex::from_scope_index(&structure);
            let shard = Arc::new(ModuleSourceIndex::new(structure, occurrences, BTreeMap::new()));
            index.replace_module_shard(module, shard);
        }
        index
    }

    /// Creates source shards while collecting all AST-owned occurrences before
    /// formal expression attachments allocate their owner-local identities.
    pub fn from_scope_indices_with_programs(
        scopes: BTreeMap<ModuleId, SourceScopeIndex>,
        programs: &BTreeMap<ModuleId, Arc<crate::source::ParsedModuleUnit>>,
    ) -> Self {
        Self::from_scope_indices_with_programs_and_context(scopes, programs, None)
    }

    /// Creates source shards with compiler-owned qualified-member targets.
    pub fn from_scope_indices_with_programs_and_context(
        scopes: BTreeMap<ModuleId, SourceScopeIndex>,
        programs: &BTreeMap<ModuleId, Arc<crate::source::ParsedModuleUnit>>,
        context: Option<&crate::source_index::builder::SourceIndexContext>,
    ) -> Self {
        let mut index = Self::empty();
        for (module, mut structure) in scopes {
            let occurrences = if let Some(source) = programs.get(&module) {
                OccurrenceIndex::from_program_with_context(&mut structure, &source.program, context)
            } else {
                OccurrenceIndex::from_scope_index(&structure)
            };
            let shard = Arc::new(ModuleSourceIndex::new(structure, occurrences, BTreeMap::new()));
            index.replace_module_shard(module, shard);
        }
        index
    }

    /// Replaces one module source shard and incrementally delta-maintains references and workspace symbols.
    pub fn replace_module_shard(&mut self, module: ModuleId, new_shard: Arc<ModuleSourceIndex>) {
        let old_shard = self.modules.get(&module).cloned();
        let mut new_shard = new_shard;
        let references_changed = old_shard
            .as_ref()
            .is_none_or(|old| old.reference_contribution.as_ref() != new_shard.reference_contribution.as_ref());
        let symbols_changed = old_shard
            .as_ref()
            .is_none_or(|old| old.workspace_symbols.as_ref() != new_shard.workspace_symbols.as_ref());

        if let Some(old) = old_shard.as_ref() {
            if !references_changed || !symbols_changed {
                let shard = Arc::make_mut(&mut new_shard);
                if !references_changed {
                    shard.reference_contribution = old.reference_contribution.clone();
                }
                if !symbols_changed {
                    shard.workspace_symbols = old.workspace_symbols.clone();
                }
            }
        }

        if references_changed {
            let old_contrib = old_shard.as_ref().map(|s| s.reference_contribution.as_ref());
            let new_contrib = Some(new_shard.reference_contribution.as_ref());
            let new_references = self.references.replace_module_contribution(&module, old_contrib, new_contrib, &mut self.stats);
            self.references = Arc::new(new_references);
        }

        if symbols_changed {
            let old_symbols = old_shard.as_ref().map_or(&[][..], |s| &s.workspace_symbols);
            let new_symbols = &new_shard.workspace_symbols;
            let new_sym_index = self.workspace_symbols.replace_module(&module, old_symbols, new_symbols, &mut self.stats);
            self.workspace_symbols = Arc::new(new_sym_index);
        }

        self.modules.insert(module, new_shard);
        self.stats.source_modules_rebuilt += 1;
    }

    /// Retires one module source shard and removes its contributions from references and workspace symbols.
    pub fn retire_module_shard(&mut self, module: &ModuleId) {
        if let Some(old_shard) = self.modules.get(module).cloned() {
            let old_contrib = Some(old_shard.reference_contribution.as_ref());
            let new_references = self.references.replace_module_contribution(module, old_contrib, None, &mut self.stats);
            self.references = Arc::new(new_references);

            let old_symbols = &old_shard.workspace_symbols;
            let new_sym_index = self.workspace_symbols.replace_module(module, old_symbols, &[], &mut self.stats);
            self.workspace_symbols = Arc::new(new_sym_index);

            self.incidents_by_module.remove(module);

            self.modules.remove(module);
            self.stats.source_modules_retired += 1;
        }
    }

    /// Returns persistent reference index.
    pub fn references(&self) -> &ReferenceIndex {
        &self.references
    }

    /// Returns persistent reference index Arc.
    pub fn references_arc(&self) -> &Arc<ReferenceIndex> {
        &self.references
    }

    /// Returns persistent workspace symbol index.
    pub fn workspace_symbols(&self) -> &WorkspaceSymbolIndex {
        &self.workspace_symbols
    }

    /// Returns persistent workspace symbol index Arc.
    pub fn workspace_symbols_arc(&self) -> &Arc<WorkspaceSymbolIndex> {
        &self.workspace_symbols
    }

    /// Returns publication update stats.
    pub fn stats(&self) -> SourceIndexUpdateStats {
        self.stats
    }

    /// Sets publication update stats.
    pub fn set_stats(&mut self, stats: SourceIndexUpdateStats) {
        self.stats = stats;
    }

    pub(crate) fn replace_module_incidents(&mut self, module: ModuleId, incidents: impl IntoIterator<Item = SourceAttachmentError>) {
        let incidents = incidents.into_iter().collect::<Vec<_>>();
        if incidents.is_empty() {
            self.incidents_by_module.remove(&module);
        } else {
            self.incidents_by_module.insert(module, Arc::from(incidents.into_boxed_slice()));
        }
    }

    /// Returns source attachment incidents without turning them into formal
    /// diagnostics or discarding valid source/formal products.
    pub fn incidents(&self) -> Vec<SourceAttachmentError> {
        self.incidents_by_module.values().flat_map(|incidents| incidents.iter().cloned()).collect()
    }

    pub fn fingerprints(&self) -> SourceIndexFingerprints {
        let mut semantic = DefaultHasher::new();
        let mut presentation = DefaultHasher::new();
        for (module, shard) in &self.modules {
            module.hash(&mut semantic);
            module.hash(&mut presentation);
            let fingerprints = shard.fingerprints();
            fingerprints.semantic.hash(&mut semantic);
            fingerprints.presentation.hash(&mut presentation);
        }
        SourceIndexFingerprints {
            semantic: ProductFingerprint::new(semantic.finish()),
            presentation: ProductFingerprint::new(presentation.finish()),
        }
    }

    /// Attaches one formal callable product to its exact source sites.
    pub fn attach_formal_analysis(&mut self, module: &ModuleId, analysis: &CallableAnalysis) -> Result<(), Box<SourceAttachmentError>> {
        let Some(module_index) = self.modules.get(module).cloned() else {
            let error = SourceAttachmentError::MissingModule(module.clone());
            self.replace_module_incidents(module.clone(), [error.clone()]);
            return Err(Box::new(error));
        };
        let (attachment, incidents) = CallableSourceAttachment::from_analysis_with_incidents(analysis.callable.clone(), &module_index.structure, analysis);
        let mut attachments = module_index.attachments.clone();
        attachments.insert(analysis.callable.clone(), Arc::new(attachment));
        let updated_shard = Arc::new(ModuleSourceIndex::from_structure_and_attachments(
            (*module_index.structure).clone(),
            (*module_index.baseline_occurrences).clone(),
            attachments,
        ));
        self.replace_module_shard(module.clone(), updated_shard);

        let mut module_incidents = self.incidents_by_module.get(module).map_or_else(Vec::new, |retained| retained.to_vec());
        module_incidents.extend(incidents.iter().cloned());
        self.replace_module_incidents(module.clone(), module_incidents);
        incidents.into_iter().next().map_or(Ok(()), |error| Err(Box::new(error)))
    }

    /// Returns one module source shard.
    pub fn module(&self, module: &ModuleId) -> Option<&ModuleSourceIndex> {
        self.modules.get(module).map(AsRef::as_ref)
    }

    /// Returns the immutable module shard for typed DB publication.
    pub fn module_arc(&self, module: &ModuleId) -> Option<Arc<ModuleSourceIndex>> {
        self.modules.get(module).cloned()
    }

    /// Returns iterator over module shards.
    pub fn modules(&self) -> impl Iterator<Item = (&ModuleId, &Arc<ModuleSourceIndex>)> {
        self.modules.iter()
    }

    /// Returns iterator over module identities.
    pub fn module_ids(&self) -> impl Iterator<Item = &ModuleId> {
        self.modules.keys()
    }

    /// Number of indexed module shards.
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Whether the index has no modules.
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// Returns module shard owning one snapshot-local source site.
    pub fn module_for_site(&self, site: &SourceSiteId) -> Option<&ModuleSourceIndex> {
        let module = match &site.owner {
            crate::identity::SourceOwner::Module(module) => module,
            crate::identity::SourceOwner::Callable(callable) => callable.module(),
        };
        self.module(module)
    }

    /// Returns exact canonical target attached to one source site.
    pub fn target_for(&self, site: &SourceSiteId) -> Option<&SemanticTargetId> {
        let module = self.module_for_site(site)?;
        module.occurrences.target_for(site).or_else(|| module.structure.target_for(site))
    }

    /// Returns canonical source metadata for one declaration identity.
    pub fn declaration_source(&self, id: &DeclarationId) -> Option<&DeclarationSourceInfo> {
        self.modules.get(&id.module)?.structure.declaration_sources.get(id)
    }

    /// Returns canonical source metadata for one callable identity.
    pub fn callable_source(&self, id: &CallableId) -> Option<&CallableSourceInfo> {
        self.modules.get(id.module())?.structure.callable_sources.get(id)
    }

    /// Returns canonical source metadata for one field identity.
    pub fn field_source(&self, id: &FieldId) -> Option<&FieldSourceInfo> {
        self.modules.get(&id.owner.module)?.structure.field_sources.get(id)
    }

    /// Returns source site selected at a byte offset.
    pub fn source_site_at(&self, module: &ModuleId, offset: usize) -> Option<&SourceSite> {
        let module_index = self.modules.get(module)?;
        let occurrence = module_index.occurrences.occurrence_at(offset)?;
        self.source_site(&occurrence.occurrence.site)
    }

    /// Returns the innermost AST expression site selected by the bounded
    /// expression interval index.
    pub fn expression_site_at(&self, module: &ModuleId, offset: usize) -> Option<&SourceSite> {
        self.modules.get(module)?.expression_site_at(offset)
    }

    /// Returns the exact formal attachment for one callable.
    pub fn formal_attachment(&self, callable: &CallableId) -> Option<&CallableSourceAttachment> {
        let module = self.modules.get(callable.module())?;
        module.attachments.get(callable).map(AsRef::as_ref)
    }

    /// Returns the canonical source site attached to one formal expression.
    pub fn source_site_for_expression(&self, callable: &CallableId, expression: ExpressionId) -> Option<&SourceSite> {
        let attachment = self.formal_attachment(callable)?;
        let site = attachment.source_site_for_expression(expression)?;
        self.source_site(site)
    }

    /// Returns the canonical source site attached to one formal binding.
    pub fn source_site_for_binding(&self, callable: &CallableId, binding: BindingId) -> Option<&SourceSite> {
        let attachment = self.formal_attachment(callable)?;
        let site = attachment.source_site_for_binding(binding)?;
        self.source_site(site)
    }

    /// Returns one source site by owner-qualified snapshot-local identity.
    pub fn source_site(&self, site: &SourceSiteId) -> Option<&SourceSite> {
        let module = match &site.owner {
            crate::identity::SourceOwner::Module(module) => module,
            crate::identity::SourceOwner::Callable(callable) => callable.module(),
        };
        let module_index = self.modules.get(module)?;
        if let Some(site) = module_index.structure.site(site) {
            return Some(site);
        }
        module_index
            .attachments
            .values()
            .find_map(|attachment| attachment.expression_sites.iter().find(|candidate| &candidate.id == site))
    }

    /// Returns occurrence selected at byte offset without analysis or scanning
    /// all source sites.
    pub fn occurrence_at(&self, module: &ModuleId, offset: usize) -> Option<OccurrenceView<'_>> {
        self.modules.get(module)?.occurrences.occurrence_at(offset)
    }

    /// Returns exact source sites attached to one canonical semantic target.
    pub fn occurrences_for_target(&self, target: &SemanticTargetId) -> Option<&[SourceSiteId]> {
        let set = self.references.target_set(target)?;
        if !set.semantic_references.is_empty() {
            Some(&set.semantic_references)
        } else if !set.lexical_references.is_empty() {
            Some(&set.lexical_references)
        } else if !set.definitions.is_empty() {
            Some(&set.definitions)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SourceIndexUpdateStats;

    #[test]
    fn scan_counter_recorders_are_observable_without_scanning() {
        let mut stats = SourceIndexUpdateStats::default();
        stats.record_source_workspace_scan(3);
        stats.record_reference_workspace_scan(5);
        stats.record_formal_workspace_scan(7);
        assert_eq!(stats.source_workspace_scan_units, 3);
        assert_eq!(stats.reference_workspace_scan_units, 5);
        assert_eq!(stats.formal_workspace_scan_units, 7);
    }
}
