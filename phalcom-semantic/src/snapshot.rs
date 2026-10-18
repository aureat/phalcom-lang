use crate::advisory::AdvisoryWorkspace;
use crate::checker::incident::InternalSemanticIncident;
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticSeverity, SemanticDiagnostic};
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::{DeclarationId, ModuleId, SemanticRevision, SnapshotId, SourceSiteId, SourceSiteRef, TypeId, WorkspaceId};
use crate::impls::{ConformanceEvidence, ConformanceIndex, ConformanceResolution, EffectiveCallableDefinition};
use crate::module_bindings::ModuleStaticBindingIndex;
use crate::presentation::{FormalFactRef, FormalFactSite, FormalSemanticProjection, SemanticSiteView};
use crate::semantic_shard::ModuleSemanticStructureShard;
use crate::signature::{CallableSignatureTable, ConstructorInitializationTable, FieldSignatureTable};
use crate::source::ParsedModuleUnit;
use crate::source_index::{OccurrenceView, SourceSemanticIndex, SourceSite};
use crate::static_constant::{StaticConstantId, StaticConstantProduct, StaticConstantTable};
use crate::static_definition::{StaticDefinitionProduct, StaticDefinitionTable};
use crate::static_value::StaticValueStore;
use crate::surface::DeclarationSurface;
use crate::trait_dispatch::{TraitDispatchIndex, TraitDispatchResolution};
use crate::traits::{TraitHeaderTable, TraitRef, TraitSurfaceTable};
use crate::type_alias::TypeAliasTable;
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_common::selector::Selector;
use phalcom_modules::graph::SemanticGraph;
use phalcom_modules::authority::{ModuleAuthorityMapRef, ModuleAuthoritySet, ModuleAuthoritySetRef, ReverseImportsRef};
use phalcom_modules::canonical_products::CanonicalExternalModuleWorld;
use phalcom_modules::identity::{SourceId, SourceLocation};
use phalcom_modules::interface::{LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::query::ModuleQueryFacade;
use phalcom_modules::stabilization::ResolverGeneration;
use phalcom_modules::topology::ModuleTopology;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::Index;
use std::path::PathBuf;
use std::sync::Arc;

fn collect_internal_incidents(callable_analyses: &crate::checker::CallableAnalysisTable) -> Arc<[InternalSemanticIncident]> {
    let mut incidents = callable_analyses
        .values()
        .flat_map(|analysis| analysis.internal_incidents.iter().cloned())
        .collect::<Vec<_>>();
    incidents.sort_by_key(|incident| (incident.callable.clone(), incident.id));
    Arc::from(incidents.into_boxed_slice())
}

/// Source-owned or budget-frozen resolved-import index used by module queries.
#[derive(Clone, Debug)]
pub enum ModuleQueryResolvedImports {
    Source(Arc<BTreeMap<(ModuleId, String), ModuleId>>),
    Frozen(Arc<phalcom_type_meta::allocation::BudgetedSortedMap<(ModuleId, String), ModuleId>>),
}

pub enum ModuleQueryResolvedImportsIter<'a> {
    Source(std::collections::btree_map::Iter<'a, (ModuleId, String), ModuleId>),
    Frozen(std::slice::Iter<'a, ((ModuleId, String), ModuleId)>),
}

impl<'a> Iterator for ModuleQueryResolvedImportsIter<'a> {
    type Item = (&'a (ModuleId, String), &'a ModuleId);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(key, value)| (key, value)),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        };
        (len, Some(len))
    }
}

impl ExactSizeIterator for ModuleQueryResolvedImportsIter<'_> {}

impl ModuleQueryResolvedImports {
    pub fn get(&self, key: &(ModuleId, String)) -> Option<&ModuleId> {
        match self {
            Self::Source(map) => map.get(key),
            Self::Frozen(map) => map.get(key),
        }
    }
    pub fn contains_key(&self, key: &(ModuleId, String)) -> bool {
        self.get(key).is_some()
    }
    pub fn iter(&self) -> ModuleQueryResolvedImportsIter<'_> {
        match self {
            Self::Source(map) => ModuleQueryResolvedImportsIter::Source(map.iter()),
            Self::Frozen(map) => ModuleQueryResolvedImportsIter::Frozen(map.as_entries().iter()),
        }
    }
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &(ModuleId, String)> {
        self.iter().map(|(key, _)| key)
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Source(map) => map.len(),
            Self::Frozen(map) => map.len(),
        }
    }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    fn authority_ref(&self) -> ModuleAuthorityMapRef<'_, (ModuleId, String), ModuleId> {
        match self {
            Self::Source(map) => ModuleAuthorityMapRef::from(map),
            Self::Frozen(map) => ModuleAuthorityMapRef::from(map),
        }
    }
    fn from_frozen_entries(
        entries: Vec<((ModuleId, String), ModuleId)>,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, ModuleQueryProductsBuildError> {
        let frozen = phalcom_type_meta::allocation::BudgetedSortedMap::from_sorted_entries(entries)
            .map_err(|_| ModuleQueryProductsBuildError::DuplicateKey)?;
        let frozen = allocations.arc(frozen).map_err(ModuleQueryProductsBuildError::Allocation)?;
        Ok(Self::Frozen(frozen))
    }
}

impl PartialEq for ModuleQueryResolvedImports {
    fn eq(&self, other: &Self) -> bool { self.len() == other.len() && self.iter().eq(other.iter()) }
}
impl Eq for ModuleQueryResolvedImports {}
impl Index<&(ModuleId, String)> for ModuleQueryResolvedImports {
    type Output = ModuleId;
    fn index(&self, key: &(ModuleId, String)) -> &Self::Output { self.get(key).expect("key present in resolved import index") }
}

/// Module interface view that either retains source-built interfaces or reads
/// them directly from the canonical loaded module world.
#[derive(Clone, Debug)]
pub enum ModuleQueryUnlinkedInterfaces {
    Source(Arc<BTreeMap<ModuleId, UnlinkedModuleInterface>>),
    Canonical(Arc<CanonicalExternalModuleWorld>),
}

pub enum ModuleQueryUnlinkedIter<'a> {
    Source(std::collections::btree_map::Iter<'a, ModuleId, UnlinkedModuleInterface>),
    Canonical(phalcom_modules::authority::ModuleAuthorityMapIter<'a, ModuleId, Arc<UnlinkedModuleInterface>>),
}

impl<'a> Iterator for ModuleQueryUnlinkedIter<'a> {
    type Item = (&'a ModuleId, &'a UnlinkedModuleInterface);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Canonical(iter) => iter.next().map(|(module, interface)| (module, interface.as_ref())),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self { Self::Source(iter) => iter.len(), Self::Canonical(iter) => iter.len() };
        (len, Some(len))
    }
}
impl ExactSizeIterator for ModuleQueryUnlinkedIter<'_> {}

impl ModuleQueryUnlinkedInterfaces {
    pub fn get(&self, module: &ModuleId) -> Option<&UnlinkedModuleInterface> {
        match self {
            Self::Source(map) => map.get(module),
            Self::Canonical(world) => world.interfaces().get(module).map(Arc::as_ref),
        }
    }
    pub fn iter(&self) -> ModuleQueryUnlinkedIter<'_> {
        match self {
            Self::Source(map) => ModuleQueryUnlinkedIter::Source(map.iter()),
            Self::Canonical(world) => ModuleQueryUnlinkedIter::Canonical(world.interfaces().iter()),
        }
    }
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &ModuleId> { self.iter().map(|(module, _)| module) }
    pub fn len(&self) -> usize {
        match self { Self::Source(map) => map.len(), Self::Canonical(world) => world.interfaces().len() }
    }
    pub fn contains_key(&self, module: &ModuleId) -> bool { self.get(module).is_some() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    fn source_authority_ref(&self) -> Option<ModuleAuthorityMapRef<'_, ModuleId, UnlinkedModuleInterface>> {
        match self { Self::Source(map) => Some(ModuleAuthorityMapRef::from(map)), Self::Canonical(_) => None }
    }
    fn canonical_world(&self) -> Option<&Arc<CanonicalExternalModuleWorld>> {
        match self { Self::Source(_) => None, Self::Canonical(world) => Some(world) }
    }
}

impl PartialEq for ModuleQueryUnlinkedInterfaces {
    fn eq(&self, other: &Self) -> bool { self.len() == other.len() && self.iter().eq(other.iter()) }
}
impl Eq for ModuleQueryUnlinkedInterfaces {}
impl Index<&ModuleId> for ModuleQueryUnlinkedInterfaces {
    type Output = UnlinkedModuleInterface;
    fn index(&self, key: &ModuleId) -> &Self::Output { self.get(key).expect("module interface present") }
}

/// Linked interface view that shares the loaded canonical `LinkedProgram`.
#[derive(Clone, Debug)]
pub enum ModuleQueryLinkedInterfaces {
    Source(Arc<BTreeMap<ModuleId, LinkedModuleInterface>>),
    Canonical(Arc<CanonicalExternalModuleWorld>),
}

pub enum ModuleQueryLinkedIter<'a> {
    Source(std::collections::btree_map::Iter<'a, ModuleId, LinkedModuleInterface>),
    Canonical(phalcom_modules::authority::ModuleAuthorityMapIter<'a, ModuleId, phalcom_modules::LinkedModule>),
}

impl<'a> Iterator for ModuleQueryLinkedIter<'a> {
    type Item = (&'a ModuleId, &'a LinkedModuleInterface);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Canonical(iter) => iter.next().map(|(module, linked)| (module, &linked.interface)),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self { Self::Source(iter) => iter.len(), Self::Canonical(iter) => iter.len() };
        (len, Some(len))
    }
}
impl ExactSizeIterator for ModuleQueryLinkedIter<'_> {}

impl ModuleQueryLinkedInterfaces {
    pub fn get(&self, module: &ModuleId) -> Option<&LinkedModuleInterface> {
        match self {
            Self::Source(map) => map.get(module),
            Self::Canonical(world) => world.linked().modules.get(module).map(|linked| &linked.interface),
        }
    }
    pub fn iter(&self) -> ModuleQueryLinkedIter<'_> {
        match self {
            Self::Source(map) => ModuleQueryLinkedIter::Source(map.iter()),
            Self::Canonical(world) => ModuleQueryLinkedIter::Canonical(world.linked().modules.iter()),
        }
    }
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &ModuleId> { self.iter().map(|(module, _)| module) }
    pub fn len(&self) -> usize {
        match self { Self::Source(map) => map.len(), Self::Canonical(world) => world.linked().modules.len() }
    }
    pub fn contains_key(&self, module: &ModuleId) -> bool { self.get(module).is_some() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    fn source_authority_ref(&self) -> Option<ModuleAuthorityMapRef<'_, ModuleId, LinkedModuleInterface>> {
        match self { Self::Source(map) => Some(ModuleAuthorityMapRef::from(map)), Self::Canonical(_) => None }
    }
}

impl PartialEq for ModuleQueryLinkedInterfaces {
    fn eq(&self, other: &Self) -> bool { self.len() == other.len() && self.iter().eq(other.iter()) }
}
impl Eq for ModuleQueryLinkedInterfaces {}
impl Index<&ModuleId> for ModuleQueryLinkedInterfaces {
    type Output = LinkedModuleInterface;
    fn index(&self, key: &ModuleId) -> &Self::Output { self.get(key).expect("linked module interface present") }
}

/// Reverse-import set view reuses the module authority read adapter.
pub type ModuleQuerySetRef<'a> = ModuleAuthoritySetRef<'a, ModuleId>;

#[derive(Clone, Debug)]
pub enum ModuleQueryReverseImports {
    Source(Arc<BTreeMap<ModuleId, BTreeSet<ModuleId>>>),
    Frozen(Arc<phalcom_type_meta::allocation::BudgetedSortedMap<ModuleId, ModuleAuthoritySet<ModuleId>>>),
}

pub enum ModuleQueryReverseIter<'a> {
    Source(std::collections::btree_map::Iter<'a, ModuleId, BTreeSet<ModuleId>>),
    Frozen(std::slice::Iter<'a, (ModuleId, ModuleAuthoritySet<ModuleId>)>),
}

impl<'a> Iterator for ModuleQueryReverseIter<'a> {
    type Item = (&'a ModuleId, ModuleQuerySetRef<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next().map(|(module, imports)| (module, imports.into())),
            Self::Frozen(iter) => iter.next().map(|(module, imports)| (module, imports.into())),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self { Self::Source(iter) => iter.len(), Self::Frozen(iter) => iter.len() };
        (len, Some(len))
    }
}
impl ExactSizeIterator for ModuleQueryReverseIter<'_> {}

impl ModuleQueryReverseImports {
    pub fn get(&self, module: &ModuleId) -> Option<ModuleQuerySetRef<'_>> {
        match self {
            Self::Source(map) => map.get(module).map(Into::into),
            Self::Frozen(map) => map.get(module).map(Into::into),
        }
    }
    pub fn iter(&self) -> ModuleQueryReverseIter<'_> {
        match self {
            Self::Source(map) => ModuleQueryReverseIter::Source(map.iter()),
            Self::Frozen(map) => ModuleQueryReverseIter::Frozen(map.as_entries().iter()),
        }
    }
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &ModuleId> { self.iter().map(|(module, _)| module) }
    pub fn len(&self) -> usize {
        match self { Self::Source(map) => map.len(), Self::Frozen(map) => map.len() }
    }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    fn reverse_ref(&self) -> ReverseImportsRef<'_> {
        match self {
            Self::Source(map) => ReverseImportsRef::from(map),
            Self::Frozen(map) => ReverseImportsRef::from(ModuleAuthorityMapRef::from(map)),
        }
    }
}

/// Retained module query products for pure read queries over immutable snapshot.
#[derive(Clone, Debug)]
pub struct ModuleQueryProducts {
    pub universe: Arc<ProjectUniverse>,
    pub unlinked: ModuleQueryUnlinkedInterfaces,
    pub linked: ModuleQueryLinkedInterfaces,
    /// Exact canonical import-resolution products retained for semantic and
    /// integration queries. The flattened target map below is a convenience
    /// projection and must not replace these products as evidence.
    pub import_products: Arc<BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
    pub resolved_imports: ModuleQueryResolvedImports,
    pub sources: Arc<BTreeMap<ModuleId, SourceLocation>>,
    pub source_modules: Arc<BTreeMap<SourceId, ModuleId>>,
    pub display_path_modules: Arc<BTreeMap<PathBuf, ModuleId>>,
    pub topology: Arc<ModuleTopology>,
    pub reverse_imports: ModuleQueryReverseImports,
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleQueryProductsBuildError {
    #[error("module query product allocation failed: {0}")]
    Allocation(#[from] phalcom_type_meta::allocation::AllocationError),
    #[error("duplicate key in frozen module query index")]
    DuplicateKey,
}

pub struct ModuleQueryProductsInputs {
    pub universe: Arc<ProjectUniverse>,
    pub unlinked: Arc<BTreeMap<ModuleId, UnlinkedModuleInterface>>,
    pub linked: Arc<BTreeMap<ModuleId, LinkedModuleInterface>>,
    pub import_products: Arc<BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
    pub resolved_imports: Arc<BTreeMap<(ModuleId, String), ModuleId>>,
    pub sources: Arc<BTreeMap<ModuleId, SourceLocation>>,
    pub topology: Arc<ModuleTopology>,
    pub reverse_imports: Arc<BTreeMap<ModuleId, BTreeSet<ModuleId>>>,
}

impl ModuleQueryProducts {
    pub fn new(inputs: ModuleQueryProductsInputs) -> Self {
        let ModuleQueryProductsInputs {
            universe,
            unlinked,
            linked,
            import_products,
            resolved_imports,
            sources,
            topology,
            reverse_imports,
        } = inputs;
        let source_modules = Arc::new(sources.iter().map(|(module, location)| (location.source_id.clone(), module.clone())).collect());
        let display_path_modules = Arc::new(
            sources
                .iter()
                .map(|(module, location)| (location.display_path.clone(), module.clone()))
                .collect(),
        );
        Self {
            universe,
            unlinked: ModuleQueryUnlinkedInterfaces::Source(unlinked),
            linked: ModuleQueryLinkedInterfaces::Source(linked),
            import_products,
            resolved_imports: ModuleQueryResolvedImports::Source(resolved_imports),
            sources,
            source_modules,
            display_path_modules,
            topology,
            reverse_imports: ModuleQueryReverseImports::Source(reverse_imports),
        }
    }

    pub fn empty() -> Self {
        let universe = Arc::new(ProjectUniverse::new());
        let topology = Arc::new(ModuleTopology::from_parts(ResolverGeneration(0), &universe, &BTreeMap::new(), &BTreeMap::new()));
        Self {
            universe,
            unlinked: ModuleQueryUnlinkedInterfaces::Source(Arc::new(BTreeMap::new())),
            linked: ModuleQueryLinkedInterfaces::Source(Arc::new(BTreeMap::new())),
            import_products: Arc::new(BTreeMap::new()),
            resolved_imports: ModuleQueryResolvedImports::Source(Arc::new(BTreeMap::new())),
            sources: Arc::new(BTreeMap::new()),
            source_modules: Arc::new(BTreeMap::new()),
            display_path_modules: Arc::new(BTreeMap::new()),
            topology,
            reverse_imports: ModuleQueryReverseImports::Source(Arc::new(BTreeMap::new())),
        }
    }

    pub fn empty_with_allocation_budget(
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, phalcom_type_meta::allocation::AllocationError> {
        let universe = allocations.arc(ProjectUniverse::new())?;
        let topology = allocations.arc(ModuleTopology::from_parts(ResolverGeneration(0), &universe, &BTreeMap::new(), &BTreeMap::new()))?;
        Ok(Self {
            universe,
            unlinked: ModuleQueryUnlinkedInterfaces::Source(allocations.arc(BTreeMap::new())?),
            linked: ModuleQueryLinkedInterfaces::Source(allocations.arc(BTreeMap::new())?),
            import_products: allocations.arc(BTreeMap::new())?,
            resolved_imports: ModuleQueryResolvedImports::Source(allocations.arc(BTreeMap::new())?),
            sources: allocations.arc(BTreeMap::new())?,
            source_modules: allocations.arc(BTreeMap::new())?,
            display_path_modules: allocations.arc(BTreeMap::new())?,
            topology,
            reverse_imports: ModuleQueryReverseImports::Source(allocations.arc(BTreeMap::new())?),
        })
    }

    pub fn from_external_modules_with_allocation_budget(
        modules: Arc<CanonicalExternalModuleWorld>,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, ModuleQueryProductsBuildError> {
        let references = &modules.linked().graphs.references;
        let import_count = modules
            .interfaces()
            .iter()
            .try_fold(0usize, |count, (_, interface)| count.checked_add(interface.imports.len()))
            .ok_or(phalcom_type_meta::allocation::AllocationError::Overflow)?;
        let mut resolved_entries = allocations.vector::<((ModuleId, String), ModuleId)>(import_count)?;
        for (module, interface) in modules.interfaces() {
            let edges = references.edges_from(module);
            for import in &interface.imports {
                if let Some(edge) = edges.iter().find(|edge| edge.range == import.range()) {
                    resolved_entries.push((
                        (module.clone_with_allocation_budget(allocations)?, allocations.string(import.path())?),
                        edge.to.clone_with_allocation_budget(allocations)?,
                    ));
                }
            }
        }
        resolved_entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let resolved_imports = ModuleQueryResolvedImports::from_frozen_entries(resolved_entries, allocations)?;

        let edge_count = modules
            .interfaces()
            .keys()
            .try_fold(0usize, |count, module| count.checked_add(references.edges_from(module).len()))
            .ok_or(phalcom_type_meta::allocation::AllocationError::Overflow)?;
        let mut reverse_pairs = allocations.vector::<(ModuleId, ModuleId)>(edge_count)?;
        for module in modules.interfaces().keys() {
            for edge in references.edges_from(module) {
                reverse_pairs.push((
                    edge.to.clone_with_allocation_budget(allocations)?,
                    edge.from.clone_with_allocation_budget(allocations)?,
                ));
            }
        }
        reverse_pairs.sort_unstable_by(|left, right| left.cmp(right));
        reverse_pairs.dedup();
        let mut reverse_entries = allocations.vector::<(ModuleId, ModuleAuthoritySet<ModuleId>)>(0)?;
        let mut pairs = reverse_pairs.into_iter().peekable();
        while let Some((target, importer)) = pairs.next() {
            let mut importers = allocations.vector::<ModuleId>(1)?;
            importers.push(importer);
            while pairs.peek().is_some_and(|(next_target, _)| *next_target == target) {
                let (_, importer) = pairs.next().expect("peeked reverse import pair");
                allocations.grow_vector_exact(&mut importers, 1)?;
                importers.push(importer);
            }
            let importers = ModuleAuthoritySet::from_budgeted_sorted_entries(importers, allocations)
                .map_err(|error| match error {
                    phalcom_modules::authority::ModuleAuthorityBuildError::Allocation(error) => ModuleQueryProductsBuildError::Allocation(error),
                    phalcom_modules::authority::ModuleAuthorityBuildError::Entries(_)
                    | phalcom_modules::authority::ModuleAuthorityBuildError::SetEntries(_) => ModuleQueryProductsBuildError::DuplicateKey,
                })?;
            allocations.grow_vector_exact(&mut reverse_entries, 1)?;
            reverse_entries.push((target, importers));
        }
        let frozen_reverse = phalcom_type_meta::allocation::BudgetedSortedMap::from_sorted_entries(reverse_entries)
            .map_err(|_| ModuleQueryProductsBuildError::DuplicateKey)?;
        let reverse_imports = ModuleQueryReverseImports::Frozen(allocations.arc(frozen_reverse)?);

        Ok(Self {
            universe: modules.linked().universe.clone(),
            unlinked: ModuleQueryUnlinkedInterfaces::Canonical(modules.clone()),
            linked: ModuleQueryLinkedInterfaces::Canonical(modules.clone()),
            import_products: allocations.arc(BTreeMap::new())?,
            resolved_imports,
            sources: allocations.arc(BTreeMap::new())?,
            source_modules: allocations.arc(BTreeMap::new())?,
            display_path_modules: allocations.arc(BTreeMap::new())?,
            topology: modules.topology().clone(),
            reverse_imports,
        })
    }
}

impl Default for ModuleQueryProducts {
    fn default() -> Self {
        Self::empty()
    }
}

/// Semantic completeness status of a snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotStatus {
    Complete,
    Partial { blocked_modules: u32 },
}

impl SnapshotStatus {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }

    pub fn is_partial(&self) -> bool {
        matches!(self, Self::Partial { .. })
    }
}

use crate::associated::AssociatedFamilyTable;
use crate::data_semantics::DataSemanticTable;
use crate::enum_requirements::EnumRequirementTable;
use crate::enum_semantics::EnumSemanticTable;

/// Immutable semantic snapshot representing a consistent view of the workspace.
#[derive(Clone, Debug)]
pub struct SemanticSnapshot {
    pub id: SnapshotId,
    pub generation: u64,
    pub store: Arc<TypeStore>,
    pub sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,
    /// Read-only compiler-generated source documents used solely for semantic
    /// provenance and editor presentation. They are never analysis inputs.
    pub presentation_sources: Arc<BTreeMap<ModuleId, Arc<str>>>,
    pub surfaces: Arc<crate::parent_map::ParentMap<DeclarationId, DeclarationSurface>>,
    pub dispatch: Arc<SurfaceDispatchResolver>,
    pub callable_signatures: Arc<CallableSignatureTable>,
    /// Canonical class-side factory to instance-initializer construction paths.
    pub constructor_initializations: Arc<ConstructorInitializationTable>,
    /// Canonical accepted callable-definition provenance, keyed by the
    /// target-owned callable identity. Backend lowering consumes this map
    /// directly and must not reconstruct acceptance from source locations.
    pub callable_definitions: Arc<crate::parent_map::ParentMap<crate::identity::CallableId, EffectiveCallableDefinition>>,
    /// Derived lookup index; definition values remain owned by `callable_definitions`.
    pub callable_definitions_by_module: Arc<crate::ordered_map::OrderedMap<ModuleId, Box<[crate::identity::CallableId]>>>,
    /// Explicit conformance heads published for this exact semantic snapshot.
    /// The index contains source-head matches only, never witness evidence.
    pub conformance_index: Arc<ConformanceIndex>,
    /// Source witness/default selections, keyed by the declaring conformance
    /// identity. Exact evidence is derived only from this canonical plan.
    pub conformance_witness_plans: Arc<crate::impls::ConformanceWitnessPlans>,
    /// Guarded derived conformance rules published for this semantic snapshot.
    pub derived_conformance_rules: Arc<crate::derived_conformance::DerivedConformanceRuleIndex>,
    /// Bounded ordinary receiver/selector discovery for trait evidence.
    pub trait_dispatch_index: Arc<TraitDispatchIndex>,
    pub field_signatures: Arc<FieldSignatureTable>,
    pub declarations: Arc<DeclarationTypeTable>,
    pub type_aliases: Arc<TypeAliasTable>,
    /// Generic contract headers for first-class traits. Traits deliberately
    /// do not appear in `declarations` as nominal/class-object type entries.
    pub trait_headers: Arc<TraitHeaderTable>,
    /// Complete body-independent behavioral contract surfaces for traits.
    pub trait_surfaces: Arc<TraitSurfaceTable>,
    pub hierarchy: Arc<MapTypeHierarchy>,
    pub diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
    pub semantic_graph: Arc<SemanticGraph>,
    pub callable_analyses: Arc<crate::checker::CallableAnalysisTable>,
    /// Internal analyzer incidents, kept separate from source diagnostics.
    pub internal_incidents: Arc<[InternalSemanticIncident]>,
    pub formal_projection: Arc<FormalSemanticProjection>,
    pub source_index: Arc<SourceSemanticIndex>,
    /// Immutable advisory runtime-shape products for this exact snapshot.
    pub advisory: Arc<AdvisoryWorkspace>,
    pub module_products: Arc<ModuleQueryProducts>,
    pub data_semantics: Arc<DataSemanticTable>,
    pub enum_semantics: Arc<EnumSemanticTable>,
    pub enum_requirements: Arc<EnumRequirementTable>,
    pub associated_surfaces: Arc<AssociatedFamilyTable>,
    /// Checked source type-member contracts keyed by stable declaration identity.
    /// Associated namespace selection remains exclusively in `associated_surfaces`.
    pub type_member_declarations:
        Arc<crate::parent_map::ParentMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>>,
    /// Checked associated static-domain declarations keyed by stable owner/name identity.
    pub associated_static_domains:
        Arc<crate::ordered_map::OrderedMap<crate::identity::AssociatedStaticDomainDeclarationId, Arc<crate::db::product::AssociatedStaticDomainDeclarationProduct>>>,
    /// Source-structural static case membership, independent of payload validity.
    pub static_case_catalogs: Arc<crate::ordered_map::OrderedMap<crate::identity::StaticDomainDeclarationId, Arc<crate::static_case::StaticDomainCaseCatalogProduct>>>,
    /// Checked constructor contracts keyed by stable static-case identity.
    pub static_case_signatures: Arc<crate::ordered_map::OrderedMap<crate::identity::StaticCaseId, Arc<crate::static_case::StaticCaseSignatureProduct>>>,
    /// Checked static definitions keyed by declaration identity.
    pub static_definitions: Arc<StaticDefinitionTable>,
    /// Checked compile-time constants keyed by canonical static constant identity.
    pub static_constants: Arc<StaticConstantTable>,
    /// Immutable static-value arena shared by semantic products in this store lineage.
    pub static_values: Arc<StaticValueStore>,
    /// Ordinary-checker facts keyed by stable module binding identity.
    pub module_static_bindings: Arc<crate::parent_map::ParentMap<ModuleId, Arc<ModuleStaticBindingIndex>>>,
    /// Retained source-local semantic structure shards for incremental reuse.
    pub semantic_structure_shards: Arc<BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>>,
    pub status: SnapshotStatus,
}

impl SemanticSnapshot {
    // Snapshot construction intentionally mirrors its immutable field layout.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workspace: WorkspaceId,
        revision: SemanticRevision,
        generation: u64,
        store: Arc<TypeStore>,
        sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,
        surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
        dispatch: Arc<SurfaceDispatchResolver>,
        callable_signatures: Arc<CallableSignatureTable>,
        declarations: Arc<DeclarationTypeTable>,
        hierarchy: Arc<MapTypeHierarchy>,
        diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
        semantic_graph: Arc<SemanticGraph>,
    ) -> Self {
        let store_id = store.id();
        let id = SnapshotId::new(workspace, revision, store_id);
        Self {
            id,
            generation,
            store,
            sources,
            presentation_sources: Arc::new(BTreeMap::new()),
            surfaces: Arc::new(crate::parent_map::ParentMap::from(surfaces.as_ref())),
            dispatch,
            callable_signatures,
            constructor_initializations: Arc::new(ConstructorInitializationTable::default()),
            callable_definitions: Arc::new(crate::parent_map::ParentMap::default()),
            callable_definitions_by_module: Arc::new(crate::ordered_map::OrderedMap::default()),
            conformance_index: Arc::new(ConformanceIndex::new()),
            conformance_witness_plans: Arc::new(crate::impls::ConformanceWitnessPlans::default()),
            derived_conformance_rules: Arc::new(crate::derived_conformance::DerivedConformanceRuleIndex::new()),
            trait_dispatch_index: Arc::new(TraitDispatchIndex::default()),
            field_signatures: Arc::new(FieldSignatureTable::new()),
            declarations,
            type_aliases: Arc::new(TypeAliasTable::new()),
            trait_headers: Arc::new(TraitHeaderTable::new()),
            trait_surfaces: Arc::new(TraitSurfaceTable::new()),
            hierarchy,
            diagnostics,
            semantic_graph,
            callable_analyses: Arc::new(crate::checker::CallableAnalysisTable::new()),
            internal_incidents: Arc::from([]),
            formal_projection: Arc::new(FormalSemanticProjection::default()),
            source_index: Arc::new(SourceSemanticIndex::default()),
            advisory: Arc::new(AdvisoryWorkspace::default()),
            module_products: Arc::new(ModuleQueryProducts::empty()),
            data_semantics: Arc::new(DataSemanticTable::new()),
            enum_semantics: Arc::new(EnumSemanticTable::new()),
            enum_requirements: Arc::new(EnumRequirementTable::new()),
            associated_surfaces: Arc::new(AssociatedFamilyTable::new()),
            type_member_declarations: Arc::new(crate::parent_map::ParentMap::default()),
            associated_static_domains: Arc::new(crate::ordered_map::OrderedMap::default()),
            static_case_catalogs: Arc::new(crate::ordered_map::OrderedMap::default()),
            static_case_signatures: Arc::new(crate::ordered_map::OrderedMap::default()),
            static_definitions: Arc::new(StaticDefinitionTable::new()),
            static_constants: Arc::new(StaticConstantTable::new()),
            static_values: Arc::new(StaticValueStore::new()),
            module_static_bindings: Arc::new(crate::parent_map::ParentMap::default()),
            semantic_structure_shards: Arc::new(BTreeMap::new()),
            status: SnapshotStatus::Complete,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_callable_analyses(
        workspace: WorkspaceId,
        revision: SemanticRevision,
        generation: u64,
        store: Arc<TypeStore>,
        sources: Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,
        surfaces: Arc<HashMap<DeclarationId, DeclarationSurface>>,
        dispatch: Arc<SurfaceDispatchResolver>,
        callable_signatures: Arc<CallableSignatureTable>,
        declarations: Arc<DeclarationTypeTable>,
        hierarchy: Arc<MapTypeHierarchy>,
        diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
        semantic_graph: Arc<SemanticGraph>,
        callable_analyses: Arc<crate::checker::CallableAnalysisTable>,
        formal_projection: Arc<FormalSemanticProjection>,
    ) -> Self {
        let store_id = store.id();
        let id = SnapshotId::new(workspace, revision, store_id);
        let internal_incidents = collect_internal_incidents(&callable_analyses);
        Self {
            id,
            generation,
            store,
            sources,
            presentation_sources: Arc::new(BTreeMap::new()),
            surfaces: Arc::new(crate::parent_map::ParentMap::from(surfaces.as_ref())),
            dispatch,
            callable_signatures,
            constructor_initializations: Arc::new(ConstructorInitializationTable::default()),
            callable_definitions: Arc::new(crate::parent_map::ParentMap::default()),
            callable_definitions_by_module: Arc::new(crate::ordered_map::OrderedMap::default()),
            conformance_index: Arc::new(ConformanceIndex::new()),
            conformance_witness_plans: Arc::new(crate::impls::ConformanceWitnessPlans::default()),
            derived_conformance_rules: Arc::new(crate::derived_conformance::DerivedConformanceRuleIndex::new()),
            trait_dispatch_index: Arc::new(TraitDispatchIndex::default()),
            field_signatures: Arc::new(FieldSignatureTable::new()),
            declarations,
            type_aliases: Arc::new(TypeAliasTable::new()),
            trait_headers: Arc::new(TraitHeaderTable::new()),
            trait_surfaces: Arc::new(TraitSurfaceTable::new()),
            hierarchy,
            diagnostics,
            semantic_graph,
            callable_analyses,
            internal_incidents,
            formal_projection,
            source_index: Arc::new(SourceSemanticIndex::default()),
            advisory: Arc::new(AdvisoryWorkspace::default()),
            module_products: Arc::new(ModuleQueryProducts::empty()),
            data_semantics: Arc::new(DataSemanticTable::new()),
            enum_semantics: Arc::new(EnumSemanticTable::new()),
            enum_requirements: Arc::new(EnumRequirementTable::new()),
            associated_surfaces: Arc::new(AssociatedFamilyTable::new()),
            type_member_declarations: Arc::new(crate::parent_map::ParentMap::default()),
            associated_static_domains: Arc::new(crate::ordered_map::OrderedMap::default()),
            static_case_catalogs: Arc::new(crate::ordered_map::OrderedMap::default()),
            static_case_signatures: Arc::new(crate::ordered_map::OrderedMap::default()),
            static_definitions: Arc::new(StaticDefinitionTable::new()),
            static_constants: Arc::new(StaticConstantTable::new()),
            static_values: Arc::new(StaticValueStore::new()),
            module_static_bindings: Arc::new(crate::parent_map::ParentMap::default()),
            semantic_structure_shards: Arc::new(BTreeMap::new()),
            status: SnapshotStatus::Complete,
        }
    }

    pub fn with_status(mut self, status: SnapshotStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_data_semantics(mut self, data_semantics: Arc<DataSemanticTable>) -> Self {
        self.data_semantics = data_semantics;
        self
    }

    pub fn with_enum_semantics(mut self, enum_semantics: Arc<EnumSemanticTable>) -> Self {
        self.enum_semantics = enum_semantics;
        self
    }

    pub fn with_enum_requirements(mut self, enum_requirements: Arc<EnumRequirementTable>) -> Self {
        self.enum_requirements = enum_requirements;
        self
    }

    pub fn with_associated_surfaces(mut self, associated_surfaces: Arc<AssociatedFamilyTable>) -> Self {
        self.associated_surfaces = associated_surfaces;
        self
    }

    pub fn with_type_member_declarations(
        mut self,
        declarations: Arc<
            crate::parent_map::ParentMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>,
        >,
    ) -> Self {
        self.type_member_declarations = declarations;
        self
    }

    pub fn with_associated_static_domains(
        mut self,
        domains: Arc<BTreeMap<crate::identity::AssociatedStaticDomainDeclarationId, Arc<crate::db::product::AssociatedStaticDomainDeclarationProduct>>>,
    ) -> Self {
        self.associated_static_domains = Arc::new(Arc::unwrap_or_clone(domains).into());
        self
    }

    pub fn with_static_case_catalogs(
        mut self,
        catalogs: Arc<BTreeMap<crate::identity::StaticDomainDeclarationId, Arc<crate::static_case::StaticDomainCaseCatalogProduct>>>,
    ) -> Self {
        self.static_case_catalogs = Arc::new(Arc::unwrap_or_clone(catalogs).into());
        self
    }

    pub fn with_static_case_signatures(
        mut self,
        signatures: Arc<BTreeMap<crate::identity::StaticCaseId, Arc<crate::static_case::StaticCaseSignatureProduct>>>,
    ) -> Self {
        self.static_case_signatures = Arc::new(Arc::unwrap_or_clone(signatures).into());
        self
    }

    pub fn with_static_definitions(mut self, static_definitions: Arc<StaticDefinitionTable>) -> Self {
        self.static_definitions = static_definitions;
        self
    }

    pub fn with_static_constants(mut self, static_constants: Arc<StaticConstantTable>) -> Self {
        self.static_constants = static_constants;
        self
    }

    /// Publishes the session-owned static-value arena for this snapshot.
    pub fn with_static_values(mut self, static_values: Arc<StaticValueStore>) -> Self {
        self.static_values = static_values;
        self
    }

    /// Publishes ordinary checker facts for stable module binding identities.
    pub fn with_module_static_bindings(mut self, bindings: Arc<crate::parent_map::ParentMap<ModuleId, Arc<ModuleStaticBindingIndex>>>) -> Self {
        self.module_static_bindings = bindings;
        self
    }

    /// Attaches retained source-local semantic structure shards.
    pub fn with_semantic_structure_shards(mut self, shards: Arc<BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>>) -> Self {
        self.semantic_structure_shards = shards;
        self
    }

    pub fn with_type_aliases(mut self, aliases: Arc<TypeAliasTable>) -> Self {
        self.type_aliases = aliases;
        self
    }

    pub fn with_trait_headers(mut self, trait_headers: Arc<TraitHeaderTable>) -> Self {
        self.trait_headers = trait_headers;
        self
    }

    pub fn with_trait_surfaces(mut self, trait_surfaces: Arc<TraitSurfaceTable>) -> Self {
        self.trait_surfaces = trait_surfaces;
        self
    }

    pub fn with_field_signatures(mut self, field_signatures: Arc<FieldSignatureTable>) -> Self {
        self.field_signatures = field_signatures;
        self
    }

    /// Replaces the snapshot's canonical constructor/initializer products.
    pub fn with_constructor_initializations(mut self, products: Arc<ConstructorInitializationTable>) -> Self {
        self.constructor_initializations = products;
        self
    }

    /// Attaches the accepted callable definitions for this exact snapshot.
    pub fn with_callable_definitions(
        mut self,
        definitions: Arc<crate::parent_map::ParentMap<crate::identity::CallableId, EffectiveCallableDefinition>>,
    ) -> Self {
        let mut by_module = BTreeMap::<ModuleId, Vec<crate::identity::CallableId>>::new();
        for (callable, _) in definitions.iter() {
            by_module
                .entry(callable.owner.module().clone())
                .or_default()
                .push(callable.clone());
        }
        self.callable_definitions_by_module = Arc::new(
            by_module
                .into_iter()
                .map(|(module, callables)| (module, callables.into_boxed_slice()))
                .collect::<BTreeMap<_, _>>().into(),
        );
        self.callable_definitions = definitions;
        self
    }

    /// Attaches the immutable explicit-conformance head index for this snapshot.
    pub fn with_conformance_index(mut self, conformance_index: Arc<ConformanceIndex>) -> Self {
        self.conformance_index = conformance_index;
        self
    }

    pub fn with_conformance_witness_plans(mut self, plans: Arc<crate::impls::ConformanceWitnessPlans>) -> Self {
        self.conformance_witness_plans = plans;
        self
    }

    pub fn with_derived_conformance_rules(mut self, rules: Arc<crate::derived_conformance::DerivedConformanceRuleIndex>) -> Self {
        self.derived_conformance_rules = rules;
        self
    }

    pub fn with_trait_dispatch_index(mut self, index: Arc<TraitDispatchIndex>) -> Self {
        self.trait_dispatch_index = index;
        self
    }

    /// Resolves a receiver/selector lookup through exact P1/P2 trait evidence
    /// after ordinary inherent lookup has missed.
    pub fn resolve_trait_evidenced_candidates(&self, receiver: TypeId, selector: &Selector, side: crate::identity::DispatchSide) -> TraitDispatchResolution {
        let mut store = (*self.store).clone();
        crate::trait_dispatch::resolve_trait_evidenced_candidates(
            &self.trait_dispatch_index,
            &self.conformance_index,
            &self.conformance_witness_plans,
            &self.trait_surfaces,
            &self.derived_conformance_rules,
            None,
            &self.declarations,
            &mut store,
            self.hierarchy.as_ref(),
            receiver,
            selector,
            side,
        )
    }

    /// Resolves exact conformance evidence from the immutable source plan.
    /// This query is intentionally pure: it clones the persistent type store
    /// view and cannot mutate the published snapshot or reselect witnesses.
    pub fn resolve_conformance_evidence(&self, target: TypeId, trait_ref: &TraitRef) -> ConformanceResolution {
        let mut store = (*self.store).clone();
        if !self.trait_surfaces.contains(&trait_ref.declaration) {
            return ConformanceResolution::InternalFailure("trait surface is unavailable for conformance evidence".into());
        }
        crate::impls::resolve_conformance_evidence_with_surfaces(
            &self.conformance_index,
            &self.conformance_witness_plans,
            &self.trait_surfaces,
            &self.declarations,
            &mut store,
            self.hierarchy.as_ref(),
            target,
            trait_ref,
        )
    }

    pub fn conformance_evidence_for(&self, target: TypeId, trait_ref: &TraitRef) -> Option<Arc<ConformanceEvidence>> {
        match self.resolve_conformance_evidence(target, trait_ref) {
            ConformanceResolution::Proven(evidence) => Some(evidence),
            _ => None,
        }
    }

    /// Normalizes a canonical type through exact published conformance
    /// evidence using the same semantic projection normalizer as all other
    /// consumers.
    pub fn normalize_projection_type(&self, ty: TypeId) -> crate::types::ProjectionNormalizationResult {
        let mut store = (*self.store).clone();
        let mut budget = crate::types::QueryBudget::default();
        let cancel = crate::types::CancellationToken::new();
        let mut context = crate::types::ProjectionNormalizationContext::new(
            crate::types::ProjectionNormalizationMode::Exact {
                conformance_index: &self.conformance_index,
                witness_plans: &self.conformance_witness_plans,
                trait_surfaces: &self.trait_surfaces,
                derived_rules: Some(&self.derived_conformance_rules),
                declarations: &self.declarations,
                hierarchy: self.hierarchy.as_ref(),
            },
            &mut budget,
            &cancel,
        );
        crate::types::normalize_type(&mut store, ty, &mut context)
    }

    pub fn with_callable_analyses(mut self, callable_analyses: Arc<crate::checker::CallableAnalysisTable>) -> Self {
        self.internal_incidents = collect_internal_incidents(&callable_analyses);
        self.callable_analyses = callable_analyses;
        self
    }

    pub fn with_formal_projection(mut self, formal_projection: Arc<FormalSemanticProjection>) -> Self {
        self.formal_projection = formal_projection;
        self
    }

    /// Attaches read-only compiler presentation sources. These documents
    /// provide source coordinates for canonical declaration provenance but do
    /// not participate in linking or semantic analysis.
    pub fn with_presentation_sources(mut self, sources: Arc<BTreeMap<ModuleId, Arc<str>>>) -> Self {
        self.presentation_sources = sources;
        self
    }

    /// Returns exact compiler-owned presentation text for one virtual source.
    pub fn presentation_source(&self, module: &ModuleId) -> Option<&str> {
        self.presentation_sources.get(module).map(AsRef::as_ref)
    }

    /// Attaches one immutable compiler-owned source semantic index.
    pub fn with_source_index(mut self, source_index: Arc<SourceSemanticIndex>) -> Self {
        self.source_index = source_index;
        self
    }

    /// Attaches immutable advisory products built from this snapshot's source
    /// and formal inputs.
    pub fn with_advisory(mut self, advisory: Arc<AdvisoryWorkspace>) -> Self {
        self.advisory = advisory;
        self
    }

    pub fn with_module_products(mut self, module_products: Arc<ModuleQueryProducts>) -> Self {
        self.module_products = module_products;
        self
    }

    pub fn module_queries(&self) -> ModuleQueryFacade<'_> {
        let products = &self.module_products;
        let facade = if let (Some(unlinked), Some(linked)) = (
            products.unlinked.source_authority_ref(),
            products.linked.source_authority_ref(),
        ) {
            ModuleQueryFacade::new(
                &products.universe,
                unlinked,
                linked,
                products.resolved_imports.authority_ref(),
                &products.sources,
                &products.source_modules,
                &products.display_path_modules,
            )
        } else if let Some(world) = products.unlinked.canonical_world() {
            ModuleQueryFacade::new_with_external_modules(
                &products.universe,
                world,
                products.resolved_imports.authority_ref(),
                &products.sources,
                &products.source_modules,
                &products.display_path_modules,
            )
        } else {
            unreachable!("module query interface backings must be source or canonical")
        };
        facade
            .with_topology(&products.topology)
            .with_reverse_imports(products.reverse_imports.reverse_ref())
    }

    /// Returns true if this snapshot's module query facade is backed by precomputed indexed products.
    pub fn is_module_queries_indexed(&self) -> bool {
        self.module_queries().is_fully_indexed()
    }

    /// Returns the protocol-neutral editor query facade for this snapshot.
    pub fn editor(&self) -> crate::editor::EditorSemanticQuery<'_> {
        crate::editor::EditorSemanticQuery::new(self)
    }

    /// Returns the canonical module associated with a source-provider identity.
    pub fn module_for_source(&self, source: &SourceId) -> Option<&ModuleId> {
        self.module_products.topology.module_for_source(source)
    }

    /// Returns the canonical module associated with an already-produced display path.
    /// This query performs no filesystem work or path canonicalization.
    pub fn module_for_display_path(&self, path: &std::path::Path) -> Option<&ModuleId> {
        self.module_products.display_path_modules.get(path)
    }

    pub fn id(&self) -> SnapshotId {
        self.id
    }

    pub fn status(&self) -> &SnapshotStatus {
        &self.status
    }

    pub fn store(&self) -> &Arc<TypeStore> {
        &self.store
    }

    pub fn sources(&self) -> &Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>> {
        &self.sources
    }

    pub fn surfaces(&self) -> &Arc<crate::parent_map::ParentMap<DeclarationId, DeclarationSurface>> {
        &self.surfaces
    }

    pub fn dispatch(&self) -> &Arc<SurfaceDispatchResolver> {
        &self.dispatch
    }

    pub fn callable_signatures(&self) -> &Arc<CallableSignatureTable> {
        &self.callable_signatures
    }

    /// Returns canonical source and generated constructor/initializer paths.
    pub fn constructor_initializations(&self) -> &Arc<ConstructorInitializationTable> {
        &self.constructor_initializations
    }

    pub fn declarations(&self) -> &Arc<DeclarationTypeTable> {
        &self.declarations
    }

    pub fn static_definitions(&self) -> &Arc<StaticDefinitionTable> {
        &self.static_definitions
    }

    pub fn static_definition(&self, declaration: &DeclarationId) -> Option<&Arc<StaticDefinitionProduct>> {
        self.static_definitions.get(declaration)
    }

    pub fn static_values(&self) -> &Arc<StaticValueStore> {
        &self.static_values
    }

    pub fn static_constants(&self) -> &Arc<StaticConstantTable> {
        &self.static_constants
    }

    pub fn static_constant(&self, id: &StaticConstantId) -> Option<&Arc<StaticConstantProduct>> {
        self.static_constants.get(id)
    }

    pub fn static_constant_by_symbol(&self, symbol: &phalcom_modules::linker::SymbolId) -> Option<&Arc<StaticConstantProduct>> {
        self.static_constants.get_by_symbol(symbol)
    }

    pub fn module_static_binding(&self, symbol: &phalcom_modules::linker::SymbolId) -> Option<&crate::module_bindings::ModuleBindingFact> {
        self.module_static_bindings.get(&symbol.module)?.get(symbol)
    }

    pub fn hierarchy(&self) -> &Arc<MapTypeHierarchy> {
        &self.hierarchy
    }

    pub fn diagnostics(&self) -> &Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>> {
        &self.diagnostics
    }

    pub fn semantic_graph(&self) -> &Arc<SemanticGraph> {
        &self.semantic_graph
    }

    pub fn callable_analyses(&self) -> &Arc<crate::checker::CallableAnalysisTable> {
        &self.callable_analyses
    }

    /// Iterates only the callable analyses belonging to the specified module.
    /// Provides O(callables in module) traversal instead of whole-snapshot filtering.
    pub fn callable_analyses_in_module<'a>(
        &'a self,
        module: &'a ModuleId,
    ) -> impl Iterator<Item = (&'a crate::identity::CallableId, &'a Arc<crate::checker::CallableAnalysis>)> {
        self.callable_analyses.iter_module(module)
    }

    /// Iterates only the accepted callable definitions belonging to the specified module.
    pub fn callable_definitions_in_module<'a>(
        &'a self,
        module: &'a ModuleId,
    ) -> impl Iterator<Item = (&'a crate::identity::CallableId, &'a crate::impls::EffectiveCallableDefinition)> {
        self.callable_definitions_by_module
            .get(module)
            .into_iter()
            .flat_map(|callables| callables.iter())
            .filter_map(|callable| self.callable_definitions.get(callable).map(|definition| (callable, definition)))
    }

    /// Returns the derived module-to-callable-definitions index (`P7.T8`).
    pub fn callable_definitions_by_module(&self) -> &Arc<crate::ordered_map::OrderedMap<ModuleId, Box<[crate::identity::CallableId]>>> {
        &self.callable_definitions_by_module
    }

    /// Checks structural completeness for a source-free loaded artifact snapshot (`P7.T7.B/T8`).
    pub fn is_source_free_artifact_complete(&self) -> bool {
        self.status.is_complete() && self.has_source_free_artifact_contents()
    }

    /// Checks required source-free artifact families without asserting that
    /// the snapshot has already been promoted to `Complete`.
    pub fn has_source_free_artifact_contents(&self) -> bool {
        let has_callable_body_facts = self.callable_analyses.values().any(|analysis| {
            !analysis.expressions.is_empty()
                || !analysis.bindings.is_empty()
                || !analysis.associated_resolutions.is_empty()
                || !analysis.callable_reference_resolutions.is_empty()
                || !analysis.family_applications.is_empty()
                || !analysis.match_resolutions.is_empty()
                || !analysis.pattern_site_resolutions.is_empty()
        });
        let callable_analyses_are_complete = self.callable_analyses.values().all(|analysis| analysis.status.is_complete());
        self.sources.is_empty()
            && self.presentation_sources.is_empty()
            && !self.callable_analyses.is_empty()
            && callable_analyses_are_complete
            && has_callable_body_facts
            && !self.callable_signatures.is_empty()
            && !self.callable_definitions.is_empty()
            && self.constructor_initializations.len() > 0
            && !self.field_signatures.is_empty()
            && self.trait_surfaces.iter().next().is_some()
            && self.trait_headers.iter().next().is_some()
            && self.declarations.iter().next().is_some()
            && !self.hierarchy.is_empty()
            && !self.dispatch.surfaces().is_empty()
            && !self.data_semantics.data_decls.is_empty()
            && !self.enum_semantics.enums.is_empty()
            && !self.enum_requirements.requirements.is_empty()
            && self.type_aliases.iter().next().is_some()
            && self.conformance_index.iter().next().is_some()
            && !self.conformance_witness_plans.is_empty()
            && !self.derived_conformance_rules.is_empty()
            && !self.trait_dispatch_index.is_empty()
            && self.associated_surfaces.surfaces.iter().next().is_some()
            && self.type_member_declarations.iter().next().is_some()
            && self.static_definitions.iter().next().is_some()
            && !self.static_constants.is_empty()
            && self.module_static_bindings.iter().next().is_some()
            && !self.module_products.unlinked.is_empty()
            && !self.module_products.linked.is_empty()
    }

    /// Returns source semantic products published with this snapshot.
    pub fn source_index(&self) -> &Arc<SourceSemanticIndex> {
        &self.source_index
    }

    /// Returns machine-readable formal source projection.
    pub fn formal_projection(&self) -> &Arc<FormalSemanticProjection> {
        &self.formal_projection
    }

    /// Returns compiler-owned advisory products for this snapshot.
    pub fn advisory(&self) -> &Arc<AdvisoryWorkspace> {
        &self.advisory
    }

    /// Returns the read-only advisory query facade for this snapshot.
    pub fn advisory_queries(&self) -> crate::advisory::AdvisoryQuery<'_> {
        crate::advisory::AdvisoryQuery::new(&self.advisory)
    }

    /// Retrieves formal expression product by exact callable/expression key.
    pub fn formal_expression(
        &self,
        callable: &crate::identity::CallableId,
        expression: crate::identity::ExpressionId,
    ) -> Option<&crate::checker::ExpressionAnalysis> {
        self.callable_analyses.get(callable)?.expressions.get(&expression)
    }

    /// Retrieves formal binding product by exact callable/binding key.
    pub fn formal_binding(&self, callable: &crate::identity::CallableId, binding: crate::identity::BindingId) -> Option<&crate::checker::BindingState> {
        self.callable_analyses.get(callable)?.bindings.get(&binding)
    }

    /// Retrieves machine-readable formal fact attachment at a source position.
    pub fn formal_fact_at(&self, module: &ModuleId, offset: usize) -> Option<&FormalFactSite> {
        self.formal_projection.fact_at(module, offset)
    }

    /// Retrieves a machine-readable formal site by canonical fact identity.
    pub fn formal_fact(&self, fact: &FormalFactRef) -> Option<&FormalFactSite> {
        self.formal_projection.get(fact)
    }

    /// Resolves a source-site handle only against its owning snapshot.
    pub fn resolve_site_ref(&self, site: &SourceSiteRef) -> Option<&SourceSite> {
        site.resolve_for(self.id).and_then(|site| self.source_site(site))
    }

    /// Returns an immutable source site by canonical snapshot-local identity.
    pub fn source_site(&self, site: &SourceSiteId) -> Option<&SourceSite> {
        self.source_index.source_site(site)
    }

    /// Returns source site selected by indexed byte position.
    pub fn source_site_at(&self, module: &ModuleId, offset: usize) -> Option<&SourceSite> {
        self.source_index.source_site_at(module, offset)
    }

    /// Returns occurrence and exact target selected by indexed byte position.
    pub fn occurrence_at(&self, module: &ModuleId, offset: usize) -> Option<OccurrenceView<'_>> {
        self.source_index.occurrence_at(module, offset)
    }

    /// Returns exact source sites for one canonical target.
    pub fn occurrences_for_target(&self, target: &crate::identity::SemanticTargetId) -> Option<&[SourceSiteId]> {
        self.source_index.occurrences_for_target(target)
    }

    /// Returns formal expression product attached to one source site.
    pub fn formal_expression_at(&self, site: &SourceSiteId) -> Option<&crate::checker::ExpressionAnalysis> {
        self.formal_fact_for_site(site).as_ref().and_then(|fact| match fact {
            FormalFactRef::Expression { callable, expression } => self.formal_expression(callable, *expression),
            _ => None,
        })
    }

    /// Returns formal binding product attached to one source site.
    pub fn formal_binding_at(&self, site: &SourceSiteId) -> Option<&crate::checker::BindingState> {
        self.formal_fact_for_site(site).as_ref().and_then(|fact| match fact {
            FormalFactRef::Binding { callable, binding } => self.formal_binding(callable, *binding),
            _ => None,
        })
    }

    /// Returns advisory expression/binding fact attached to one source site.
    pub fn advisory_fact(&self, site: &SourceSiteId) -> Option<&crate::advisory::AdvisoryFact> {
        self.advisory.expression(site).or_else(|| self.advisory.binding(site))
    }

    /// Returns advisory parameter fact for one canonical callable slot.
    pub fn advisory_binding(&self, slot: &crate::advisory::AdvisoryParameterSlot) -> Option<&crate::advisory::AdvisoryFact> {
        self.advisory.parameter(slot)
    }

    /// Returns advisory callable summary for one canonical callable.
    pub fn advisory_callable(&self, callable: &crate::identity::CallableId) -> Option<&crate::advisory::AdvisoryCallableSummary> {
        self.advisory.callable(callable)
    }

    /// Composes indexed source, formal, and advisory products without
    /// triggering analysis or scanning callable bodies.
    pub fn semantic_site_at(&self, module: &ModuleId, offset: usize) -> SemanticSiteView<'_> {
        let source_site = self.source_index.occurrence_at(module, offset).map(|view| view.occurrence.site.clone());
        let formal = self.formal_projection.fact_at(module, offset);
        let advisory = source_site
            .as_ref()
            .and_then(|site| self.advisory.expression(site).or_else(|| self.advisory.binding(site)));
        let target = source_site.as_ref().and_then(|site| self.advisory.target(site));
        SemanticSiteView {
            source_site,
            formal,
            advisory,
            target,
        }
    }

    pub(crate) fn formal_fact_for_site(&self, site: &SourceSiteId) -> Option<FormalFactRef> {
        let module = self.source_index.module_for_site(site)?;
        for attachment in module.attachments.values() {
            if attachment.formal_expressions.values().any(|candidate| candidate == site) {
                let expression = attachment
                    .formal_expressions
                    .iter()
                    .find_map(|(expression, candidate)| (candidate == site).then_some(expression))?;
                return Some(FormalFactRef::Expression {
                    callable: attachment.callable.clone(),
                    expression: *expression,
                });
            }
            if let Some(binding) = attachment
                .formal_bindings
                .iter()
                .find_map(|(binding, candidate)| (candidate == site).then_some(binding))
            {
                return Some(FormalFactRef::Binding {
                    callable: attachment.callable.clone(),
                    binding: *binding,
                });
            }
        }
        None
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .values()
            .any(|diags| diags.iter().any(|d| d.severity == DiagnosticSeverity::Error))
    }

    pub fn all_diagnostics(&self) -> impl Iterator<Item = &SemanticDiagnostic> {
        self.diagnostics.values().flat_map(|diags| diags.iter())
    }

    pub fn diagnostics_for(&self, module: &ModuleId) -> Option<&[SemanticDiagnostic]> {
        self.diagnostics.get(module).map(|d| d.as_ref())
    }
}
