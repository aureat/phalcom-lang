//! Immutable advisory workspace products.
//!
//! These products are compiler-owned runtime-shape observations. They share
//! source and semantic identities with formal products, but their status and
//! uncertainty never become formal checker state.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::db::ProductFingerprint;
use crate::identity::{CallableId, FieldId, ModuleId, SemanticTargetId, SourceSiteId};

use super::{AdvisoryCallableSummary, AdvisoryConfidence, AdvisoryFact, AdvisoryOrigin, AdvisoryParameterSlot, AdvisoryProductStatus};

/// Advisory resolution of one compiler-owned source target.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AdvisoryTargetResolution {
    /// Canonical target selected by source/formal analysis.
    pub target: SemanticTargetId,
    /// Advisory confidence for exposing the target to consumers.
    pub confidence: AdvisoryConfidence,
    /// Bounded causal evidence for the target resolution.
    pub provenance: Vec<AdvisoryOrigin>,
}

/// Immutable advisory products belonging to one module shard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryModuleProduct {
    /// Canonical module owning this shard.
    pub module: ModuleId,
    /// Advisory expression facts published for this module.
    pub expressions: Arc<BTreeMap<SourceSiteId, AdvisoryFact>>,
    /// Advisory binding facts published for this module.
    pub bindings: Arc<BTreeMap<SourceSiteId, AdvisoryFact>>,
    /// Advisory field facts owned by declarations in this module.
    pub fields: Arc<BTreeMap<FieldId, AdvisoryFact>>,
    /// Advisory parameter facts owned by callables in this module.
    pub parameters: Arc<BTreeMap<AdvisoryParameterSlot, AdvisoryFact>>,
    /// Exact canonical targets exposed by this module's source index.
    pub targets: Arc<BTreeMap<SourceSiteId, AdvisoryTargetResolution>>,
    /// Explicit outcome of this shard's advisory analysis.
    pub status: AdvisoryProductStatus,
    /// Deterministic fingerprint of semantically observable shard content.
    pub fingerprint: ProductFingerprint,
}

impl AdvisoryModuleProduct {
    /// Creates one canonical, deterministically fingerprinted module shard.
    pub fn new(
        module: ModuleId,
        expressions: BTreeMap<SourceSiteId, AdvisoryFact>,
        bindings: BTreeMap<SourceSiteId, AdvisoryFact>,
        fields: BTreeMap<FieldId, AdvisoryFact>,
        parameters: BTreeMap<AdvisoryParameterSlot, AdvisoryFact>,
        targets: BTreeMap<SourceSiteId, AdvisoryTargetResolution>,
        status: AdvisoryProductStatus,
    ) -> Self {
        let fingerprint = fingerprint_module(&module, &expressions, &bindings, &fields, &parameters, &targets, &status);
        Self {
            module,
            expressions: Arc::new(expressions),
            bindings: Arc::new(bindings),
            fields: Arc::new(fields),
            parameters: Arc::new(parameters),
            targets: Arc::new(targets),
            status,
            fingerprint,
        }
    }
}

/// Immutable advisory products for one semantic snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryWorkspace {
    /// Reusable per-module advisory shards.
    pub modules: Arc<BTreeMap<ModuleId, Arc<AdvisoryModuleProduct>>>,
    /// Advisory expression facts keyed by snapshot-local source sites.
    pub expressions: Arc<BTreeMap<SourceSiteId, AdvisoryFact>>,
    /// Advisory binding facts keyed by snapshot-local declaration sites.
    pub bindings: Arc<BTreeMap<SourceSiteId, AdvisoryFact>>,
    /// Advisory field facts keyed by canonical field identity.
    pub fields: Arc<BTreeMap<FieldId, AdvisoryFact>>,
    /// Advisory parameter facts keyed by canonical callable/index slots.
    pub parameters: Arc<BTreeMap<AdvisoryParameterSlot, AdvisoryFact>>,
    /// Advisory callable summaries keyed by canonical callable identity.
    pub callables: Arc<BTreeMap<CallableId, Arc<AdvisoryCallableSummary>>>,
    /// Advisory target resolutions keyed by source site.
    pub targets: Arc<BTreeMap<SourceSiteId, AdvisoryTargetResolution>>,
    /// Explicit outcome of workspace advisory publication.
    pub status: AdvisoryProductStatus,
    /// Deterministic fingerprint of the complete advisory workspace product.
    pub fingerprint: ProductFingerprint,
}

impl Default for AdvisoryWorkspace {
    fn default() -> Self {
        Self::from_parts(BTreeMap::new(), BTreeMap::new(), AdvisoryProductStatus::Complete)
    }
}

impl AdvisoryWorkspace {
    /// Combines immutable module shards and callable summaries into one facade.
    pub fn from_parts(
        modules: BTreeMap<ModuleId, Arc<AdvisoryModuleProduct>>,
        callables: BTreeMap<CallableId, Arc<AdvisoryCallableSummary>>,
        status: AdvisoryProductStatus,
    ) -> Self {
        let mut expressions = BTreeMap::new();
        let mut bindings = BTreeMap::new();
        let mut fields = BTreeMap::new();
        let mut parameters = BTreeMap::new();
        let mut targets = BTreeMap::new();
        for shard in modules.values() {
            expressions.extend(shard.expressions.iter().map(|(site, fact)| (site.clone(), fact.clone())));
            bindings.extend(shard.bindings.iter().map(|(site, fact)| (site.clone(), fact.clone())));
            fields.extend(shard.fields.iter().map(|(field, fact)| (field.clone(), fact.clone())));
            parameters.extend(shard.parameters.iter().map(|(slot, fact)| (slot.clone(), fact.clone())));
            targets.extend(shard.targets.iter().map(|(site, target)| (site.clone(), target.clone())));
        }
        let fingerprint = fingerprint_workspace(&modules, &expressions, &bindings, &fields, &parameters, &callables, &targets, &status);
        Self {
            modules: Arc::new(modules),
            expressions: Arc::new(expressions),
            bindings: Arc::new(bindings),
            fields: Arc::new(fields),
            parameters: Arc::new(parameters),
            callables: Arc::new(callables),
            targets: Arc::new(targets),
            status,
            fingerprint,
        }
    }

    /// Returns module shard, if advisory coverage was published for it.
    pub fn module(&self, module: &ModuleId) -> Option<&Arc<AdvisoryModuleProduct>> {
        self.modules.get(module)
    }

    /// Returns expression fact only when a product was published for `site`.
    pub fn expression(&self, site: &SourceSiteId) -> Option<&AdvisoryFact> {
        self.expressions.get(site)
    }

    /// Returns binding fact only when a product was published for `site`.
    pub fn binding(&self, site: &SourceSiteId) -> Option<&AdvisoryFact> {
        self.bindings.get(site)
    }

    /// Returns field fact only when a product was published for `field`.
    pub fn field(&self, field: &FieldId) -> Option<&AdvisoryFact> {
        self.fields.get(field)
    }

    /// Returns parameter fact only when a product was published for `slot`.
    pub fn parameter(&self, slot: &AdvisoryParameterSlot) -> Option<&AdvisoryFact> {
        self.parameters.get(slot)
    }

    /// Returns callable summary only when a product was published for `callable`.
    pub fn callable(&self, callable: &CallableId) -> Option<&AdvisoryCallableSummary> {
        self.callables.get(callable).map(AsRef::as_ref)
    }

    /// Returns target resolution only when a product was published for `site`.
    pub fn target(&self, site: &SourceSiteId) -> Option<&AdvisoryTargetResolution> {
        self.targets.get(site)
    }

    /// Returns an explicit unknown fact for callers that opt into that view.
    /// Missing coverage remains observable through [`Self::expression`] and
    /// the corresponding other query methods.
    pub fn expression_or_unknown(&self, site: &SourceSiteId) -> AdvisoryFact {
        self.expression(site).cloned().unwrap_or_else(AdvisoryFact::unknown)
    }

    /// Returns whether workspace advisory publication completed.
    pub fn is_complete(&self) -> bool {
        matches!(self.status, AdvisoryProductStatus::Complete)
    }

    /// Rebuilds immutable workspace facade with an orthogonal publication
    /// status while preserving all already-computed advisory facts.
    pub fn with_status(&self, status: AdvisoryProductStatus) -> Self {
        Self::from_parts(self.modules.as_ref().clone(), self.callables.as_ref().clone(), status)
    }
}

fn fingerprint_module(
    module: &ModuleId,
    expressions: &BTreeMap<SourceSiteId, AdvisoryFact>,
    bindings: &BTreeMap<SourceSiteId, AdvisoryFact>,
    fields: &BTreeMap<FieldId, AdvisoryFact>,
    parameters: &BTreeMap<AdvisoryParameterSlot, AdvisoryFact>,
    targets: &BTreeMap<SourceSiteId, AdvisoryTargetResolution>,
    status: &AdvisoryProductStatus,
) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    module.hash(&mut hasher);
    hash_facts(&mut hasher, expressions);
    hash_facts(&mut hasher, bindings);
    hash_facts(&mut hasher, fields);
    hash_facts(&mut hasher, parameters);
    targets.hash(&mut hasher);
    status.hash(&mut hasher);
    ProductFingerprint::new(hasher.finish())
}

fn fingerprint_workspace(
    modules: &BTreeMap<ModuleId, Arc<AdvisoryModuleProduct>>,
    expressions: &BTreeMap<SourceSiteId, AdvisoryFact>,
    bindings: &BTreeMap<SourceSiteId, AdvisoryFact>,
    fields: &BTreeMap<FieldId, AdvisoryFact>,
    parameters: &BTreeMap<AdvisoryParameterSlot, AdvisoryFact>,
    callables: &BTreeMap<CallableId, Arc<AdvisoryCallableSummary>>,
    targets: &BTreeMap<SourceSiteId, AdvisoryTargetResolution>,
    status: &AdvisoryProductStatus,
) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    for (module, shard) in modules {
        module.hash(&mut hasher);
        shard.fingerprint.hash(&mut hasher);
    }
    hash_facts(&mut hasher, expressions);
    hash_facts(&mut hasher, bindings);
    hash_facts(&mut hasher, fields);
    hash_facts(&mut hasher, parameters);
    for (callable, summary) in callables {
        callable.hash(&mut hasher);
        summary.fingerprint.hash(&mut hasher);
    }
    targets.hash(&mut hasher);
    status.hash(&mut hasher);
    ProductFingerprint::new(hasher.finish())
}

fn hash_facts<K: Hash, H: Hasher>(hasher: &mut H, facts: &BTreeMap<K, AdvisoryFact>) {
    for (key, fact) in facts {
        key.hash(hasher);
        fact.fingerprint().hash(hasher);
    }
}
