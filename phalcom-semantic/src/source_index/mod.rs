//! Compiler-owned source identity and source-site indexing.
//!
//! This module owns source locations, lexical source identity, occurrences,
//! and their attachment to canonical semantic products. It intentionally does
//! not depend on LSP identity or protocol types.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::associated::AssociatedMemberId;
use crate::checker::CallableAnalysis;
use crate::checker::associated::AssociatedResolutionKind;
use crate::db::ProductFingerprint;
use crate::identity::{
    BindingId, CallableId, CallableParameterId, DeclarationId, ExpressionId, FieldId, ModuleId, SemanticTargetId, SourceOwner, SourceSiteId,
};
use crate::source_index::interval::{RangeEntry, RangeIndex};

pub mod builder;
pub mod interval;
pub mod occurrence;
pub mod reference;
pub mod scope;
pub mod site;
pub mod symbol;

pub use builder::{
    SourceIndexContext, TypeParameterSourceTargets, build_source_scope_index, resolve_type_member_projection_targets, resolve_type_parameter_source_targets,
    resolve_type_reference_targets,
};
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
    MissingBindingOrigin { callable: CallableId, binding: BindingId },
    InvalidBindingOrigin { callable: CallableId, binding: BindingId },
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
    pub fn from_analysis(
        callable: CallableId,
        source_callable: &CallableId,
        scopes: &SourceScopeIndex,
        analysis: &CallableAnalysis,
    ) -> Result<Self, Box<SourceAttachmentError>> {
        let (attachment, incidents) = Self::from_analysis_with_incidents(callable, source_callable, scopes, analysis);
        incidents.into_iter().next().map_or(Ok(attachment), |error| Err(Box::new(error)))
    }

    /// Builds all exact expression attachments while retaining binding failures
    /// as non-fatal incidents. A missing binding must not erase independently
    /// attachable call/expression products for the same callable.
    pub fn from_analysis_with_incidents(
        callable: CallableId,
        source_callable: &CallableId,
        scopes: &SourceScopeIndex,
        analysis: &CallableAnalysis,
    ) -> (Self, Vec<SourceAttachmentError>) {
        Self::from_analysis_with_source_owner(callable, &SourceOwner::Callable(source_callable.clone()), scopes, analysis)
    }

    fn from_analysis_with_source_owner(
        callable: CallableId,
        source_owner: &SourceOwner,
        scopes: &SourceScopeIndex,
        analysis: &CallableAnalysis,
    ) -> (Self, Vec<SourceAttachmentError>) {
        let mut formal_bindings = BTreeMap::new();
        let mut incidents = Vec::new();
        let mut current_bindings = scopes
            .bindings
            .values()
            .filter(|binding| &binding.declaration_site.owner == source_owner)
            .collect::<Vec<_>>();
        current_bindings.sort_by_key(|binding| binding.declaration_site.local);
        let mut states = Vec::new();
        for state in analysis.bindings.values() {
            match analysis.binding_origins.get(&state.binding) {
                Some(crate::checker::analysis::CallableBindingOrigin::BodyLocal) => states.push(state),
                Some(crate::checker::analysis::CallableBindingOrigin::ModuleValue { .. }) if state.parameter.is_none() => {}
                Some(crate::checker::analysis::CallableBindingOrigin::ModuleValue { .. }) => {
                    incidents.push(SourceAttachmentError::InvalidBindingOrigin {
                        callable: callable.clone(),
                        binding: state.binding,
                    });
                }
                None => incidents.push(SourceAttachmentError::MissingBindingOrigin {
                    callable: callable.clone(),
                    binding: state.binding,
                }),
            }
        }
        if let Some(binding) = analysis.binding_origins.keys().find(|binding| !analysis.bindings.contains_key(binding)) {
            incidents.push(SourceAttachmentError::InvalidBindingOrigin {
                callable: callable.clone(),
                binding: *binding,
            });
        }
        states.sort_by_key(|state| state.binding);
        let rebase_ranges = match source_owner {
            SourceOwner::Callable(source_callable) => scopes
                .callable_body_ranges
                .get(source_callable)
                .is_some_and(|current| *current != analysis.body_range),
            SourceOwner::Module(_) => false,
        };
        // The historical range-rebase shortcut is valid only within a
        // callable source shard. Module shards also contain imports, top-level
        // declarations, and nested lexical scopes, so their site count/order
        // cannot identify a checker binding.
        let current_bindings =
            (matches!(source_owner, SourceOwner::Callable(_)) && rebase_ranges && current_bindings.len() == states.len()).then_some(current_bindings);
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
            if let SourceOwner::Module(_) = source_owner {
                let candidates = scopes
                    .bindings
                    .values()
                    .filter(|binding| {
                        &binding.declaration_site.owner == source_owner && binding.declaration_range == state.range && binding.name.as_ref() == state.name
                    })
                    .map(|binding| binding.declaration_site.clone())
                    .collect::<Vec<_>>();
                match candidates.as_slice() {
                    [site] => {
                        formal_bindings.insert(state.binding, site.clone());
                    }
                    [] => incidents.push(SourceAttachmentError::MissingBinding {
                        callable: callable.clone(),
                        binding: state.binding,
                    }),
                    _ => incidents.push(SourceAttachmentError::AmbiguousBinding {
                        callable: callable.clone(),
                        binding: state.binding,
                    }),
                }
                continue;
            }
            let candidates = scopes
                .bindings
                .values()
                .filter(|binding| {
                    binding.name.as_ref() == state.name && binding.declaration_range == state.range && binding.declaration_site.owner == *source_owner
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
            if let Some(resolution) = analysis.associated_resolutions.get(&expression.id) {
                let target = match &resolution.kind {
                    AssociatedResolutionKind::ExactValue {
                        member: AssociatedMemberId::DataComponent(component),
                        ..
                    } => Some(SemanticTargetId::DataComponent(component.clone())),
                    AssociatedResolutionKind::StaticDomain {
                        member: AssociatedMemberId::StaticDomain(declaration),
                        ..
                    } => Some(SemanticTargetId::StaticDomain(crate::identity::StaticDomainDeclarationId::Associated(
                        declaration.clone(),
                    ))),
                    AssociatedResolutionKind::StaticTerm { case, .. } => Some(SemanticTargetId::StaticCase(case.clone())),
                    _ => None,
                };
                if let Some(target) = target {
                    exact_targets.insert(site.id.clone(), target);
                }
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
    modules: Option<im::OrdMap<ModuleId, Arc<ModuleSourceIndex>>>,
    references: Arc<ReferenceIndex>,
    workspace_symbols: Arc<WorkspaceSymbolIndex>,
    /// Non-fatal attachment incidents grouped by their owning module.
    incidents_by_module: Option<im::OrdMap<ModuleId, Arc<[SourceAttachmentError]>>>,
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
            let default_owner = SourceOwner::Callable(analysis.callable.clone());
            let source_owner = context
                .and_then(|context| context.body_source_owners.get(&analysis.callable))
                .unwrap_or(&default_owner);
            let (attachment, mut attachment_incidents) =
                CallableSourceAttachment::from_analysis_with_source_owner(analysis.callable.clone(), source_owner, &structure, analysis);
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
            if matches!(target, SemanticTargetId::DataComponent(_)) {
                let candidates = all
                    .iter()
                    .filter(|occurrence| {
                        occurrence.kind == OccurrenceKind::Field
                            && occurrence.role == OccurrenceRole::Read
                            && formal_source.range.start <= occurrence.range.start
                            && occurrence.range.end == formal_source.range.end
                            && occurrence.range != formal_source.range
                    })
                    .collect::<Vec<_>>();
                if let [occurrence] = candidates.as_slice() {
                    exact_targets.insert(occurrence.site.clone(), target.clone());
                }
            } else if let Some(name) = match target {
                SemanticTargetId::StaticDomain(crate::identity::StaticDomainDeclarationId::Associated(declaration)) => Some(declaration.name.as_ref()),
                SemanticTargetId::StaticCase(case) => Some(case.name()),
                _ => None,
            } {
                for occurrence in &all {
                    if occurrence.kind == OccurrenceKind::Member
                        && formal_source.range.start <= occurrence.range.start
                        && occurrence.range.end <= formal_source.range.end
                        && matches!(&occurrence.hint, Some(crate::source_index::occurrence::OccurrenceHint::Name(occurrence_name)) if occurrence_name.as_ref() == name)
                    {
                        exact_targets.insert(occurrence.site.clone(), target.clone());
                    }
                }
            } else {
                for occurrence in &all {
                    if occurrence.role == OccurrenceRole::Call
                        && formal_source.range.start <= occurrence.range.start
                        && occurrence.range.end <= formal_source.range.end
                    {
                        exact_targets.insert(occurrence.site.clone(), target.clone());
                    }
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
        let target = structure
            .target_for(&decl.declaration_site)
            .cloned()
            .unwrap_or_else(|| SemanticTargetId::Declaration(decl.id.clone()));
        symbols.push(WorkspaceSymbolEntry {
            id: WorkspaceSymbolId {
                target: target.clone(),
                site: decl.declaration_site.clone(),
            },
            name: decl.name.clone(),
            normalized_name: decl.name.to_lowercase().into_boxed_str(),
            target,
            declaration_site: decl.declaration_site.clone(),
            kind: match decl.kind {
                SourceDeclarationKind::Class => EditorSymbolKind::Class,
                SourceDeclarationKind::Data => EditorSymbolKind::Data,
                SourceDeclarationKind::TypeData => EditorSymbolKind::TypeData,
                SourceDeclarationKind::Trait => EditorSymbolKind::Trait,
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
            container_name: callable.id.try_declaration_owner().map(|owner| owner.name.clone()),
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
            SourceSiteKind::StaticCase(id) => {
                let name = id.selector.encode().into_boxed_str();
                symbols.push(WorkspaceSymbolEntry {
                    id: WorkspaceSymbolId {
                        target: SemanticTargetId::StaticCase(id.clone()),
                        site: site.id.clone(),
                    },
                    name: name.clone(),
                    normalized_name: name.to_lowercase().into_boxed_str(),
                    target: SemanticTargetId::StaticCase(id.clone()),
                    declaration_site: site.id.clone(),
                    kind: EditorSymbolKind::Variant,
                    container_name: Some(static_domain_name(&id.domain).into()),
                });
            }
            SourceSiteKind::StaticDomain(id) if matches!(id, crate::identity::StaticDomainDeclarationId::Associated(_)) => {
                let name = static_domain_name(id).to_owned().into_boxed_str();
                symbols.push(WorkspaceSymbolEntry {
                    id: WorkspaceSymbolId {
                        target: SemanticTargetId::StaticDomain(id.clone()),
                        site: site.id.clone(),
                    },
                    name: name.clone(),
                    normalized_name: name.to_lowercase().into_boxed_str(),
                    target: SemanticTargetId::StaticDomain(id.clone()),
                    declaration_site: site.id.clone(),
                    kind: EditorSymbolKind::TypeData,
                    container_name: match id {
                        crate::identity::StaticDomainDeclarationId::Associated(declaration) => Some(declaration.owner.name.clone()),
                        crate::identity::StaticDomainDeclarationId::Nominal(_) => None,
                    },
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
            SourceSiteKind::DataComponent(id) => {
                let name = format!("{}:{}", id.owner.name, id.index).into_boxed_str();
                symbols.push(WorkspaceSymbolEntry {
                    id: WorkspaceSymbolId {
                        target: SemanticTargetId::DataComponent(id.clone()),
                        site: site.id.clone(),
                    },
                    name: name.clone(),
                    normalized_name: name.to_lowercase().into_boxed_str(),
                    target: SemanticTargetId::DataComponent(id.clone()),
                    declaration_site: site.id.clone(),
                    kind: EditorSymbolKind::DataComponent,
                    container_name: Some(id.owner.name.clone()),
                });
            }
            SourceSiteKind::AssociatedType(id) => {
                let name = format!("type {}", id.index).into_boxed_str();
                symbols.push(WorkspaceSymbolEntry {
                    id: WorkspaceSymbolId {
                        target: SemanticTargetId::AssociatedType(id.clone()),
                        site: site.id.clone(),
                    },
                    name: name.clone(),
                    normalized_name: name.to_lowercase().into_boxed_str(),
                    target: SemanticTargetId::AssociatedType(id.clone()),
                    declaration_site: site.id.clone(),
                    kind: EditorSymbolKind::Callable,
                    container_name: Some(id.owner.name.clone()),
                });
            }
            _ => {}
        }
    }
    symbols.sort_by(|a, b| a.id.cmp(&b.id));
    Arc::from(symbols.into_boxed_slice())
}

fn static_domain_name(domain: &crate::identity::StaticDomainDeclarationId) -> &str {
    match domain {
        crate::identity::StaticDomainDeclarationId::Nominal(declaration) => &declaration.name,
        crate::identity::StaticDomainDeclarationId::Associated(declaration) => &declaration.name,
    }
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
            modules: None,
            references: Arc::new(ReferenceIndex::new()),
            workspace_symbols: Arc::new(WorkspaceSymbolIndex::new()),
            incidents_by_module: None,
            stats: SourceIndexUpdateStats::default(),
        }
    }

    /// Creates an empty source index using the caller's allocation budget for Arc overhead.
    pub fn empty_with_allocation_budget(
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, phalcom_type_meta::allocation::AllocationError> {
        Ok(Self {
            modules: None,
            references: allocations.arc(ReferenceIndex::default())?,
            workspace_symbols: allocations.arc(WorkspaceSymbolIndex::default())?,
            incidents_by_module: None,
            stats: SourceIndexUpdateStats::default(),
        })
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
        let old_shard = self.modules.as_ref().and_then(|m| m.get(&module)).cloned();
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

        let mut modules = self.modules.clone().unwrap_or_default();
        modules.insert(module, new_shard);
        self.modules = Some(modules);
        self.stats.source_modules_rebuilt += 1;
    }

    /// Retires one module source shard and removes its contributions from references and workspace symbols.
    pub fn retire_module_shard(&mut self, module: &ModuleId) {
        if let Some(mut modules) = self.modules.clone() {
            if let Some(old_shard) = modules.get(module).cloned() {
                let old_contrib = Some(old_shard.reference_contribution.as_ref());
                let new_references = self.references.replace_module_contribution(module, old_contrib, None, &mut self.stats);
                self.references = Arc::new(new_references);

                let old_symbols = &old_shard.workspace_symbols;
                let new_sym_index = self.workspace_symbols.replace_module(module, old_symbols, &[], &mut self.stats);
                self.workspace_symbols = Arc::new(new_sym_index);

                if let Some(mut incidents) = self.incidents_by_module.clone() {
                    incidents.remove(module);
                    self.incidents_by_module = if incidents.is_empty() { None } else { Some(incidents) };
                }

                modules.remove(module);
                self.modules = if modules.is_empty() { None } else { Some(modules) };
                self.stats.source_modules_retired += 1;
            }
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
            if let Some(mut map) = self.incidents_by_module.clone() {
                map.remove(&module);
                self.incidents_by_module = if map.is_empty() { None } else { Some(map) };
            }
        } else {
            let mut map = self.incidents_by_module.clone().unwrap_or_default();
            map.insert(module, Arc::from(incidents.into_boxed_slice()));
            self.incidents_by_module = Some(map);
        }
    }

    /// Returns source attachment incidents without turning them into formal
    /// diagnostics or discarding valid source/formal products.
    pub fn incidents(&self) -> Vec<SourceAttachmentError> {
        self.incidents_by_module
            .as_ref()
            .into_iter()
            .flat_map(|m| m.values())
            .flat_map(|incidents| incidents.iter().cloned())
            .collect()
    }

    pub fn fingerprints(&self) -> SourceIndexFingerprints {
        let mut semantic = DefaultHasher::new();
        let mut presentation = DefaultHasher::new();
        if let Some(modules) = &self.modules {
            for (module, shard) in modules {
                module.hash(&mut semantic);
                module.hash(&mut presentation);
                let fingerprints = shard.fingerprints();
                fingerprints.semantic.hash(&mut semantic);
                fingerprints.presentation.hash(&mut presentation);
            }
        }
        SourceIndexFingerprints {
            semantic: ProductFingerprint::new(semantic.finish()),
            presentation: ProductFingerprint::new(presentation.finish()),
        }
    }

    /// Attaches one formal callable product to its exact source sites.
    pub fn attach_formal_analysis(
        &mut self,
        module: &ModuleId,
        analysis: &CallableAnalysis,
        source_callable: &CallableId,
    ) -> Result<(), Box<SourceAttachmentError>> {
        self.attach_formal_analysis_with_source_owner(module, analysis, &SourceOwner::Callable(source_callable.clone()))
    }

    /// Attaches one formal callable product through an explicit source-owner
    /// relation. Module-body analyses use this when their lexical declaration
    /// sites live in the module namespace rather than a callable shard.
    pub fn attach_formal_analysis_with_source_owner(
        &mut self,
        module: &ModuleId,
        analysis: &CallableAnalysis,
        source_owner: &SourceOwner,
    ) -> Result<(), Box<SourceAttachmentError>> {
        let Some(module_index) = self.modules.as_ref().and_then(|m| m.get(module)).cloned() else {
            let error = SourceAttachmentError::MissingModule(module.clone());
            self.replace_module_incidents(module.clone(), [error.clone()]);
            return Err(Box::new(error));
        };
        let (attachment, incidents) =
            CallableSourceAttachment::from_analysis_with_source_owner(analysis.callable.clone(), source_owner, &module_index.structure, analysis);
        let mut attachments = module_index.attachments.clone();
        attachments.insert(analysis.callable.clone(), Arc::new(attachment));
        let updated_shard = Arc::new(ModuleSourceIndex::from_structure_and_attachments(
            (*module_index.structure).clone(),
            (*module_index.baseline_occurrences).clone(),
            attachments,
        ));
        self.replace_module_shard(module.clone(), updated_shard);

        let mut module_incidents = self
            .incidents_by_module
            .as_ref()
            .and_then(|m| m.get(module))
            .map_or_else(Vec::new, |retained| retained.to_vec());
        module_incidents.extend(incidents.iter().cloned());
        self.replace_module_incidents(module.clone(), module_incidents);
        incidents.into_iter().next().map_or(Ok(()), |error| Err(Box::new(error)))
    }

    /// Returns one module source shard.
    pub fn module(&self, module: &ModuleId) -> Option<&ModuleSourceIndex> {
        self.modules.as_ref().and_then(|m| m.get(module)).map(AsRef::as_ref)
    }

    /// Returns the immutable module shard for typed DB publication.
    pub fn module_arc(&self, module: &ModuleId) -> Option<Arc<ModuleSourceIndex>> {
        self.modules.as_ref().and_then(|m| m.get(module)).cloned()
    }

    /// Returns iterator over module shards.
    pub fn modules(&self) -> impl Iterator<Item = (&ModuleId, &Arc<ModuleSourceIndex>)> {
        self.modules.as_ref().into_iter().flat_map(|m| m.iter())
    }

    /// Returns iterator over module identities.
    pub fn module_ids(&self) -> impl Iterator<Item = &ModuleId> {
        self.modules.as_ref().into_iter().flat_map(|m| m.keys())
    }

    /// Number of indexed module shards.
    pub fn len(&self) -> usize {
        self.modules.as_ref().map_or(0, |m| m.len())
    }

    /// Whether the index has no modules.
    pub fn is_empty(&self) -> bool {
        self.modules.as_ref().map_or(true, |m| m.is_empty())
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
        self.modules.as_ref().and_then(|m| m.get(&id.module))?.structure.declaration_sources.get(id)
    }

    /// Returns canonical source metadata for one callable identity.
    pub fn callable_source(&self, id: &CallableId) -> Option<&CallableSourceInfo> {
        self.modules.as_ref().and_then(|m| m.get(id.module()))?.structure.callable_sources.get(id)
    }

    /// Returns canonical source metadata for one field identity.
    pub fn field_source(&self, id: &FieldId) -> Option<&FieldSourceInfo> {
        self.modules.as_ref().and_then(|m| m.get(&id.owner.module))?.structure.field_sources.get(id)
    }

    /// Returns source site selected at a byte offset.
    pub fn source_site_at(&self, module: &ModuleId, offset: usize) -> Option<&SourceSite> {
        let module_index = self.modules.as_ref().and_then(|m| m.get(module))?;
        let occurrence = module_index.occurrences.occurrence_at(offset)?;
        self.source_site(&occurrence.occurrence.site)
    }

    /// Returns the innermost AST expression site selected by the bounded
    /// expression interval index.
    pub fn expression_site_at(&self, module: &ModuleId, offset: usize) -> Option<&SourceSite> {
        self.modules.as_ref().and_then(|m| m.get(module))?.expression_site_at(offset)
    }

    /// Returns the exact formal attachment for one callable.
    pub fn formal_attachment(&self, callable: &CallableId) -> Option<&CallableSourceAttachment> {
        let module = self.modules.as_ref().and_then(|m| m.get(callable.module()))?;
        module.attachments.get(callable).map(AsRef::as_ref)
    }

    /// Reports whether the current source contains a declaration site for a
    /// formal parameter identity. This distinguishes source parameters from
    /// synthetic semantic bindings such as an implicit receiver.
    pub fn has_parameter_source_site(&self, source_callable: &CallableId, parameter: &CallableParameterId) -> bool {
        self.modules
            .as_ref()
            .and_then(|m| m.get(source_callable.module()))
            .and_then(|module| module.structure.callable_sources.get(source_callable))
            .is_some_and(|source| source.parameter_sites.contains_key(parameter))
    }

    /// Returns whether a source-owned callable record has a declaration site
    /// for `parameter`. `None` means the source owner itself is unavailable;
    /// callers must not treat that as evidence of a generated parameter.
    pub fn parameter_source_site_is_indexed(&self, source_callable: &CallableId, parameter: &CallableParameterId) -> Option<bool> {
        let module = self.modules.as_ref().and_then(|m| m.get(source_callable.module()))?;
        let source = module.structure.callable_sources.get(source_callable)?;
        Some(source.parameter_sites.contains_key(parameter))
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
        let module_index = self.modules.as_ref().and_then(|m| m.get(module))?;
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
        self.modules.as_ref().and_then(|m| m.get(module))?.occurrences.occurrence_at(offset)
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
    use super::{SourceIndexUpdateStats, *};
    use crate::checker::analysis::BindingState;
    use crate::checker::analysis::{BindingAnalysisIndex, CallableBindingOriginIndex};
    use crate::checker::analysis::{CallableAnalysis, CallableAnalysisStatus, FlowStateSummary};
    use crate::checker::analysis_map::AnalysisMap;
    use crate::checker::flow::graph::FlowGraph;
    use crate::explain::ExplanationArena;
    use crate::identity::{CallableId, DeclarationId, DispatchSide, SourceOwner};
    use crate::types::evidence::{TypeKnowledge, UnknownReason};
    use crate::{ModuleId, ReturnContractValidation};
    use phalcom_ast::parser::parse;
    use phalcom_common::range::SourceRange;
    use phalcom_common::selector::Selector;
    use phalcom_modules::source::ModuleKind;
    use std::collections::{BTreeMap, HashMap};
    use std::sync::Arc;

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

    #[test]
    fn same_local_binding_ordinal_keeps_distinct_module_lambda_sites_and_source_free_projection() {
        let source: Arc<str> = Arc::from("import universe.numbers\nclass Packet {}\nlet first = |text| text\nlet second = |text| text\n");
        let parsed = parse(&source, 0);
        assert!(parsed.errors.is_empty(), "{:#?}", parsed.errors);
        let module = ModuleId::universe_root();
        let context = SourceIndexContext::default();
        let scopes = build_source_scope_index(module.clone(), &parsed.program, &context);
        let unit = Arc::new(crate::source::ParsedModuleUnit::new(
            module.clone(),
            ModuleKind::Module,
            None,
            source.clone(),
            Arc::new(parsed.program),
        ));
        let programs = BTreeMap::from([(module.clone(), unit)]);
        let mut source_index =
            SourceSemanticIndex::from_scope_indices_with_programs_and_context(BTreeMap::from([(module.clone(), scopes)]), &programs, Some(&context));
        let starts = [source.find("|text|").unwrap() + 1, source.rfind("|text|").unwrap() + 1];
        let ranges = starts.map(|start| SourceRange::new(start, start + "text".len()));

        let make_analysis = |name: &str, range: SourceRange| {
            let callable = CallableId::new(
                DeclarationId::new(module.clone(), name.into()),
                Selector::getter("invoke").unwrap(),
                DispatchSide::Instance,
            );
            let binding = crate::identity::BindingId(0);
            let mut bindings = BindingAnalysisIndex::new();
            bindings
                .insert_source(
                    binding,
                    BindingState::new(binding, "text", range, None, TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence), false),
                )
                .unwrap();
            let mut binding_origins = CallableBindingOriginIndex::new();
            binding_origins
                .insert_source(binding, crate::checker::analysis::CallableBindingOrigin::BodyLocal)
                .unwrap();
            CallableAnalysis {
                callable,
                body_range: SourceRange::new(0, source.len()),
                body_source_owner: Some(SourceOwner::Module(module.clone())),
                expressions: AnalysisMap::new(),
                bindings,
                binding_origins,
                for_iteration_lanes: Arc::new(AnalysisMap::new()),
                positional_expansions: Arc::new(AnalysisMap::new()),
                known_invalid_positional_expansions: Arc::new(AnalysisMap::new()),
                sequence_literals: Arc::new(AnalysisMap::new()),
                associated_resolutions: Arc::new(AnalysisMap::new()),
                callable_reference_resolutions: Arc::new(AnalysisMap::new()),
                family_applications: Arc::new(AnalysisMap::new()),
                match_resolutions: Arc::new(AnalysisMap::new()),
                pattern_site_resolutions: Arc::new(AnalysisMap::new()),
                local_proof_graph: Arc::new(Default::default()),
                flow_graph: Arc::new(FlowGraph::default()),
                entry_flow: FlowStateSummary::default(),
                exits: Default::default(),
                return_validation: ReturnContractValidation::NotApplicable,
                diagnostics: Arc::from([]),
                internal_incidents: Arc::from([]),
                explanations: Arc::new(ExplanationArena::default()),
                return_explanation: None,
                dependencies: Arc::from([]),
                semantic_dependencies: Arc::from([]),
                constructor_super_initializer: None,
                dependency_fingerprint: crate::db::ProductFingerprint::new(1),
                status: CallableAnalysisStatus::Complete,
            }
        };

        let first = make_analysis("FirstLambda", ranges[0]);
        let second = make_analysis("SecondLambda", ranges[1]);
        assert_eq!(first.bindings.keys().next(), second.bindings.keys().next(), "binding ordinals are owner-local");
        for analysis in [&first, &second] {
            source_index
                .attach_formal_analysis_with_source_owner(&module, analysis, &SourceOwner::Module(module.clone()))
                .expect("each lambda binding attaches to its exact module-owned source site");
        }
        let first_site = source_index
            .source_site_for_binding(&first.callable, crate::identity::BindingId(0))
            .unwrap()
            .id
            .clone();
        let second_site = source_index
            .source_site_for_binding(&second.callable, crate::identity::BindingId(0))
            .unwrap()
            .id
            .clone();
        assert_ne!(first_site, second_site, "same local ordinal and name must not alias distinct source sites");
        assert_eq!(source_index.source_site(&first_site).unwrap().range, ranges[0]);
        assert_eq!(source_index.source_site(&second_site).unwrap().range, ranges[1]);

        let analyses = HashMap::from([
            (first.callable.clone(), Arc::new(first.clone())),
            (second.callable.clone(), Arc::new(second.clone())),
        ]);
        let binding_ranges = BTreeMap::from([
            (first.callable.clone(), BTreeMap::from([(crate::identity::BindingId(0), ranges[0])])),
            (second.callable.clone(), BTreeMap::from([(crate::identity::BindingId(0), ranges[1])])),
        ]);
        let source_free = crate::presentation::FormalSemanticProjection::from_callable_analyses_with_binding_ranges(&analyses, &binding_ranges);
        assert_eq!(
            source_free.binding_attachment_range(&first.callable, crate::identity::BindingId(0)),
            Some(ranges[0])
        );
        assert_eq!(
            source_free.binding_attachment_range(&second.callable, crate::identity::BindingId(0)),
            Some(ranges[1])
        );
    }
}
