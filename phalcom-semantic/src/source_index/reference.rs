//! Compiler-owned module reference contributions and persistent reverse index.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::identity::{ModuleId, SemanticTargetId, SourceOwner, SourceSiteId};
use crate::source_index::occurrence::OccurrenceRole;
use crate::source_index::scope::{SourceBindingKind, SourceScopeIndex};
use crate::source_index::{CallableSourceAttachment, SourceIndexUpdateStats};

/// Immutable set of declaration sites and reference sites for one canonical semantic target.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TargetReferenceSet {
    pub definitions: Arc<[SourceSiteId]>,
    pub lexical_references: Arc<[SourceSiteId]>,
    pub semantic_references: Arc<[SourceSiteId]>,
}

impl TargetReferenceSet {
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty() && self.lexical_references.is_empty() && self.semantic_references.is_empty()
    }
}

/// Module-owned contribution to workspace definitions and reverse references.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModuleReferenceContribution {
    pub definitions: BTreeMap<SemanticTargetId, Arc<[SourceSiteId]>>,
    pub lexical_references: BTreeMap<SemanticTargetId, Arc<[SourceSiteId]>>,
    pub semantic_references: BTreeMap<SemanticTargetId, Arc<[SourceSiteId]>>,
}

impl ModuleReferenceContribution {
    /// Builds one module reference contribution from structure, occurrences, and formal attachments.
    pub fn from_module_index(
        structure: &SourceScopeIndex,
        occurrences: &[crate::source_index::occurrence::SemanticOccurrence],
        target_map: &impl Fn(&SourceSiteId) -> Option<SemanticTargetId>,
        attachments: &BTreeMap<crate::identity::CallableId, Arc<CallableSourceAttachment>>,
    ) -> Self {
        let mut defs = BTreeMap::<SemanticTargetId, Vec<SourceSiteId>>::new();
        let mut lexical = BTreeMap::<SemanticTargetId, Vec<SourceSiteId>>::new();
        let mut semantic = BTreeMap::<SemanticTargetId, Vec<SourceSiteId>>::new();

        // 1. Declarations / definitions from source structure
        for decl in structure.declaration_sources.values() {
            defs.entry(SemanticTargetId::Declaration(decl.id.clone()))
                .or_default()
                .push(decl.declaration_site.clone());
        }
        for callable in structure.callable_sources.values() {
            defs.entry(SemanticTargetId::Callable(callable.id.clone()))
                .or_default()
                .push(callable.declaration_site.clone());
        }
        for field in structure.field_sources.values() {
            defs.entry(SemanticTargetId::Field(field.id.clone()))
                .or_default()
                .push(field.declaration_site.clone());
        }
        for binding in structure.bindings.values() {
            defs.entry(SemanticTargetId::Binding(binding.declaration_site.clone()))
                .or_default()
                .push(binding.declaration_site.clone());
            if matches!(binding.kind, SourceBindingKind::TopLevelLet | SourceBindingKind::TopLevelConst) {
                if let Some(target) = structure.target_for(&binding.declaration_site) {
                    if matches!(target, SemanticTargetId::ModuleBinding(_)) {
                        defs.entry(target.clone()).or_default().push(binding.declaration_site.clone());
                    }
                }
            }
        }
        for site in structure.sites.values() {
            match &site.kind {
                crate::source_index::SourceSiteKind::Variant(id) => {
                    defs.entry(SemanticTargetId::Variant(id.clone())).or_default().push(site.id.clone());
                }
                crate::source_index::SourceSiteKind::VariantFamily(id) => {
                    defs.entry(SemanticTargetId::VariantFamily(id.clone())).or_default().push(site.id.clone());
                }
                crate::source_index::SourceSiteKind::VariantField(id) => {
                    defs.entry(SemanticTargetId::VariantField(id.clone())).or_default().push(site.id.clone());
                }
                crate::source_index::SourceSiteKind::DataComponent(id) => {
                    defs.entry(SemanticTargetId::DataComponent(id.clone())).or_default().push(site.id.clone());
                }
                crate::source_index::SourceSiteKind::Module => {
                    if let SourceOwner::Module(mod_id) = &site.id.owner {
                        defs.entry(SemanticTargetId::Module(mod_id.clone())).or_default().push(site.id.clone());
                    }
                }
                _ => {}
            }
        }

        // 2. Token occurrences
        for occurrence in occurrences {
            if occurrence.role == OccurrenceRole::Declaration {
                continue;
            }
            let site = &occurrence.site;
            let target = target_map(site).or_else(|| structure.target_for(site).cloned());
            let Some(target) = target else { continue };

            match &target {
                SemanticTargetId::Binding(binding_site) => {
                    lexical.entry(target.clone()).or_default().push(site.clone());
                    if let Some(origin) = structure.import_origin(binding_site) {
                        semantic.entry(origin.remote_target.clone()).or_default().push(site.clone());
                    } else {
                        semantic.entry(target.clone()).or_default().push(site.clone());
                    }
                }
                _ => {
                    lexical.entry(target.clone()).or_default().push(site.clone());
                    semantic.entry(target).or_default().push(site.clone());
                }
            }
        }

        // 3. Attachments exact expression targets
        for attachment in attachments.values() {
            for (expr_site, target) in &attachment.exact_targets {
                if structure.sites.contains_key(expr_site) {
                    continue;
                }
                match target {
                    SemanticTargetId::Binding(binding_site) => {
                        lexical.entry(target.clone()).or_default().push(expr_site.clone());
                        if let Some(origin) = structure.import_origin(binding_site) {
                            semantic.entry(origin.remote_target.clone()).or_default().push(expr_site.clone());
                        } else {
                            semantic.entry(target.clone()).or_default().push(expr_site.clone());
                        }
                    }
                    _ => {
                        lexical.entry(target.clone()).or_default().push(expr_site.clone());
                        semantic.entry(target.clone()).or_default().push(expr_site.clone());
                    }
                }
            }
        }

        let convert = |map: BTreeMap<SemanticTargetId, Vec<SourceSiteId>>| {
            map.into_iter()
                .map(|(target, mut sites)| {
                    sites.sort();
                    sites.dedup();
                    (target, Arc::from(sites.into_boxed_slice()))
                })
                .collect()
        };

        Self {
            definitions: convert(defs),
            lexical_references: convert(lexical),
            semantic_references: convert(semantic),
        }
    }

    /// All semantic target keys touched by this contribution.
    pub fn all_targets(&self) -> impl Iterator<Item = &SemanticTargetId> {
        self.definitions
            .keys()
            .chain(self.lexical_references.keys())
            .chain(self.semantic_references.keys())
    }
}

/// Persistent workspace-wide reference index.
#[derive(Clone, Debug, Default)]
pub struct ReferenceIndex {
    by_target: im::OrdMap<SemanticTargetId, Arc<TargetReferenceSet>>,
}

impl ReferenceIndex {
    /// Creates a reference index with empty target sets.
    pub fn new() -> Self {
        Self { by_target: im::OrdMap::new() }
    }

    /// Returns definitions for a target.
    pub fn definitions(&self, target: &SemanticTargetId) -> &[SourceSiteId] {
        self.by_target.get(target).map_or(&[], |set| &set.definitions)
    }

    /// Returns lexical references for a target.
    pub fn lexical_references(&self, target: &SemanticTargetId) -> &[SourceSiteId] {
        self.by_target.get(target).map_or(&[], |set| &set.lexical_references)
    }

    /// Returns semantic references for a target.
    pub fn semantic_references(&self, target: &SemanticTargetId) -> &[SourceSiteId] {
        self.by_target.get(target).map_or(&[], |set| &set.semantic_references)
    }

    /// Returns target reference set for one target, if any.
    pub fn target_set(&self, target: &SemanticTargetId) -> Option<&Arc<TargetReferenceSet>> {
        self.by_target.get(target)
    }

    /// Returns all targets in the index.
    pub fn targets(&self) -> impl Iterator<Item = &SemanticTargetId> {
        self.by_target.keys()
    }

    /// Replaces or retires a module contribution incrementally.
    pub fn replace_module_contribution(
        &self,
        module: &ModuleId,
        old: Option<&ModuleReferenceContribution>,
        new: Option<&ModuleReferenceContribution>,
        stats: &mut SourceIndexUpdateStats,
    ) -> Self {
        let mut by_target = self.by_target.clone();

        let mut touched_targets = std::collections::BTreeSet::new();
        if let Some(old) = old {
            for target in old.all_targets() {
                touched_targets.insert(target.clone());
            }
        }
        if let Some(new) = new {
            for target in new.all_targets() {
                touched_targets.insert(target.clone());
            }
        }

        stats.reference_contributions_replaced += 1;
        stats.reference_targets_touched += touched_targets.len();

        let site_belongs_to_module = |site: &SourceSiteId| match &site.owner {
            SourceOwner::Module(m) => m == module,
            SourceOwner::Callable(c) => c.module() == module,
        };

        for target in touched_targets {
            let current_set = by_target.get(&target).cloned().unwrap_or_default();

            let update_slice = |current: &[SourceSiteId], new_contrib_slice: Option<&Arc<[SourceSiteId]>>| -> Arc<[SourceSiteId]> {
                let mut kept = current.iter().filter(|site| !site_belongs_to_module(site)).cloned().collect::<Vec<_>>();
                if let Some(new_slice) = new_contrib_slice {
                    kept.extend(new_slice.iter().cloned());
                }
                kept.sort();
                kept.dedup();
                Arc::from(kept.into_boxed_slice())
            };

            let new_defs = update_slice(&current_set.definitions, new.and_then(|n| n.definitions.get(&target)));
            let new_lexical = update_slice(&current_set.lexical_references, new.and_then(|n| n.lexical_references.get(&target)));
            let new_semantic = update_slice(&current_set.semantic_references, new.and_then(|n| n.semantic_references.get(&target)));

            let (added, removed) = slice_delta(&current_set.definitions, &new_defs);
            stats.reference_sites_added += added;
            stats.reference_sites_removed += removed;
            let (added, removed) = slice_delta(&current_set.lexical_references, &new_lexical);
            stats.reference_sites_added += added;
            stats.reference_sites_removed += removed;
            let (added, removed) = slice_delta(&current_set.semantic_references, &new_semantic);
            stats.reference_sites_added += added;
            stats.reference_sites_removed += removed;

            if new_defs == current_set.definitions && new_lexical == current_set.lexical_references && new_semantic == current_set.semantic_references {
                continue;
            }

            if new_defs.is_empty() && new_lexical.is_empty() && new_semantic.is_empty() {
                by_target.remove(&target);
            } else {
                by_target.insert(
                    target,
                    Arc::new(TargetReferenceSet {
                        definitions: new_defs,
                        lexical_references: new_lexical,
                        semantic_references: new_semantic,
                    }),
                );
            }
        }

        Self { by_target }
    }
}

fn slice_delta(old: &[SourceSiteId], new: &[SourceSiteId]) -> (usize, usize) {
    let mut added = 0;
    let mut removed = 0;
    let mut old_index = 0;
    let mut new_index = 0;
    while old_index < old.len() || new_index < new.len() {
        match (old.get(old_index), new.get(new_index)) {
            (Some(old_site), Some(new_site)) if old_site == new_site => {
                old_index += 1;
                new_index += 1;
            }
            (Some(old_site), Some(new_site)) if old_site < new_site => {
                removed += 1;
                old_index += 1;
            }
            (Some(_), Some(_)) => {
                added += 1;
                new_index += 1;
            }
            (Some(_), None) => {
                removed += 1;
                old_index += 1;
            }
            (None, Some(_)) => {
                added += 1;
                new_index += 1;
            }
            (None, None) => break,
        }
    }
    (added, removed)
}
