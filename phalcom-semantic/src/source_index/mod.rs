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
use crate::identity::{BindingId, CallableId, ExpressionId, ModuleId, SemanticTargetId, SourceSiteId};

pub mod builder;
pub mod interval;
pub mod occurrence;
pub mod scope;
pub mod site;

pub use builder::{SourceIndexContext, build_source_scope_index};
pub use occurrence::{OccurrenceHint, OccurrenceIndex, OccurrenceKind, OccurrenceRole, OccurrenceView, SemanticOccurrence};
pub use scope::{SourceBindingInfo, SourceBindingKind, SourceNameResolution, SourceScope, SourceScopeId, SourceScopeIndex};
pub use site::{SourceSite, SourceSiteKind};

/// Formal source-site attachment failure. Construction fails closed instead of
/// selecting an arbitrary same-name or same-range candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceAttachmentError {
    MissingModule(ModuleId),
    AmbiguousBinding { callable: CallableId, binding: BindingId },
    MissingBinding { callable: CallableId, binding: BindingId },
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
    pub fn from_analysis(callable: CallableId, scopes: &SourceScopeIndex, analysis: &CallableAnalysis) -> Result<Self, SourceAttachmentError> {
        let (attachment, incidents) = Self::from_analysis_with_incidents(callable, scopes, analysis);
        incidents.into_iter().next().map_or(Ok(attachment), Err)
    }

    /// Builds all exact expression attachments while retaining binding failures
    /// as non-fatal incidents. A missing binding must not erase independently
    /// attachable call/expression products for the same callable.
    pub fn from_analysis_with_incidents(callable: CallableId, scopes: &SourceScopeIndex, analysis: &CallableAnalysis) -> (Self, Vec<SourceAttachmentError>) {
        let mut formal_bindings = BTreeMap::new();
        let mut incidents = Vec::new();
        for state in analysis.bindings.values() {
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
        expressions.sort_by_key(|expression| (expression.range.start, expression.range.end, expression.id));
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
                expression.range,
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
    pub modules: BTreeMap<ModuleId, Arc<ModuleSourceIndex>>,
    pub target_occurrences: BTreeMap<SemanticTargetId, Arc<[SourceSiteId]>>,
    /// Non-fatal attachment incidents retained with this source product.
    pub incidents: Arc<[SourceAttachmentError]>,
}

/// Source structure, exact occurrences, and formal attachments for one module.
#[derive(Clone, Debug)]
pub struct ModuleSourceIndex {
    pub structure: Arc<SourceScopeIndex>,
    pub occurrences: Arc<OccurrenceIndex>,
    pub attachments: BTreeMap<CallableId, Arc<CallableSourceAttachment>>,
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
    /// Creates a source index from compiler-owned lexical source structures.
    pub fn from_scope_indices(scopes: BTreeMap<ModuleId, SourceScopeIndex>) -> Self {
        let modules = scopes
            .into_iter()
            .map(|(module, structure)| {
                let occurrences = OccurrenceIndex::from_scope_index(&structure);
                (
                    module,
                    Arc::new(ModuleSourceIndex {
                        structure: Arc::new(structure),
                        occurrences: Arc::new(occurrences),
                        attachments: BTreeMap::new(),
                    }),
                )
            })
            .collect();
        let mut index = Self {
            modules,
            target_occurrences: BTreeMap::new(),
            incidents: Arc::from([]),
        };
        index.rebuild_target_occurrences();
        index
    }

    /// Creates source shards while collecting all AST-owned occurrences before
    /// formal expression attachments allocate their owner-local identities.
    pub fn from_scope_indices_with_programs(
        scopes: BTreeMap<ModuleId, SourceScopeIndex>,
        programs: &BTreeMap<ModuleId, Arc<crate::source::ParsedModuleUnit>>,
    ) -> Self {
        let modules = scopes
            .into_iter()
            .map(|(module, mut structure)| {
                let occurrences = if let Some(source) = programs.get(&module) {
                    OccurrenceIndex::from_program(&mut structure, &source.program)
                } else {
                    OccurrenceIndex::from_scope_index(&structure)
                };
                (
                    module,
                    Arc::new(ModuleSourceIndex {
                        structure: Arc::new(structure),
                        occurrences: Arc::new(occurrences),
                        attachments: BTreeMap::new(),
                    }),
                )
            })
            .collect();
        let mut index = Self {
            modules,
            target_occurrences: BTreeMap::new(),
            incidents: Arc::from([]),
        };
        index.rebuild_target_occurrences();
        index
    }

    /// Returns source attachment incidents without turning them into formal
    /// diagnostics or discarding valid source/formal products.
    pub fn incidents(&self) -> &[SourceAttachmentError] {
        &self.incidents
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
    pub fn attach_formal_analysis(&mut self, module: &ModuleId, analysis: &CallableAnalysis) -> Result<(), SourceAttachmentError> {
        let Some(module_index) = self.modules.get_mut(module) else {
            return Err(SourceAttachmentError::MissingModule(module.clone()));
        };
        let (attachment, incidents) = CallableSourceAttachment::from_analysis_with_incidents(analysis.callable.clone(), &module_index.structure, analysis);
        let module_index = Arc::make_mut(module_index);
        module_index.attachments.insert(analysis.callable.clone(), Arc::new(attachment));
        let mut all = module_index.occurrences.all().to_vec();
        let mut exact_targets = module_index.structure.targets.clone();
        for occurrence in &all {
            if let Some(target) = module_index.occurrences.target_for(&occurrence.site) {
                exact_targets.insert(occurrence.site.clone(), target.clone());
            }
        }
        for attachment in module_index.attachments.values() {
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
        // Formal call resolution covers the full expression range while the
        // editor cursor normally lands on its selector token. Project exact
        // callable targets onto contained call occurrences without rerunning
        // dispatch at presentation time.
        for (formal_site, target) in module_index.attachments.values().flat_map(|attachment| attachment.exact_targets.iter()) {
            let Some(formal_source) = module_index
                .attachments
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
        module_index.occurrences = Arc::new(occurrences);
        self.rebuild_target_occurrences();
        incidents.into_iter().next().map_or(Ok(()), Err)
    }

    /// Returns one module source shard.
    pub fn module(&self, module: &ModuleId) -> Option<&ModuleSourceIndex> {
        self.modules.get(module).map(AsRef::as_ref)
    }

    /// Returns the immutable module shard for typed DB publication.
    pub fn module_arc(&self, module: &ModuleId) -> Option<Arc<ModuleSourceIndex>> {
        self.modules.get(module).cloned()
    }

    /// Returns source site selected at a byte offset.
    pub fn source_site_at(&self, module: &ModuleId, offset: usize) -> Option<&SourceSite> {
        let module_index = self.modules.get(module)?;
        let occurrence = module_index.occurrences.occurrence_at(offset)?;
        self.source_site(&occurrence.occurrence.site)
    }

    /// Returns the exact formal attachment for one callable.
    pub fn formal_attachment(&self, callable: &CallableId) -> Option<&CallableSourceAttachment> {
        let module = self.modules.get(&callable.owner.module)?;
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
            crate::identity::SourceOwner::Module(module)
            | crate::identity::SourceOwner::Callable(crate::identity::CallableId {
                owner: crate::identity::DeclarationId { module, .. },
                ..
            }) => module,
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
        self.target_occurrences.get(target).map(AsRef::as_ref)
    }

    pub(crate) fn rebuild_target_occurrences(&mut self) {
        let mut reverse = BTreeMap::<SemanticTargetId, Vec<SourceSiteId>>::new();
        for module in self.modules.values() {
            for occurrence in module.occurrences.all() {
                if let Some(target) = module.occurrences.target_for(&occurrence.site) {
                    reverse.entry(target.clone()).or_default().push(occurrence.site.clone());
                }
            }
            for attachment in module.attachments.values() {
                for (site, target) in &attachment.exact_targets {
                    reverse.entry(target.clone()).or_default().push(site.clone());
                }
            }
        }
        self.target_occurrences = reverse
            .into_iter()
            .map(|(target, mut sites)| {
                sites.sort();
                sites.dedup();
                (target, Arc::from(sites))
            })
            .collect();
    }
}
