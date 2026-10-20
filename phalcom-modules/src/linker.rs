//! Static symbol/import linking for a resolved module universe.

use crate::authority::ModuleAuthorityMap;
use crate::error::ModuleGraphError;
use crate::graph::{
    DependencyPhase, ModuleGraphs, ReferenceEdge, ReferenceKind, RuntimeDependencyEdge, RuntimeDependencyReason, RuntimeRequirementProduct,
    RuntimeRequirementProvenance, RuntimeRequirementTarget, RuntimeRequirementTiming, SemanticEdge, SemanticEdgeKind, SemanticNodeId,
};
use crate::identity::{ImportBindingLocalId, ModuleId, ModulePath, SourceImportBindingId};
use crate::interface::{ImportSurface, LinkedExport, LinkedModuleInterface, UnlinkedExportTarget, UnlinkedModuleInterface};
use crate::project::ProjectUniverse;
use phalcom_ast::ast::{ImportPath, StaticSymbolRef};
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use thiserror::Error;

/// Semantic identity of a module-owned global declaration.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SymbolId {
    /// Module that owns the declaration.
    pub module: ModuleId,
    /// Declaration name in that module.
    pub name: Box<str>,
}

/// A target used by a linked read.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LinkedReadSpec {
    /// Read the materialized module object.
    Module(ModuleId),
    /// Read a mutable global slot owned by the canonical symbol.
    Binding(SymbolId),
}

/// Executable requirement transported to the finalizer.
///
/// Every requirement contributes a runtime graph edge. Requirements that also
/// need a canonical external read carry its modules-owned target here so the
/// finalizer can allocate the read slot from the same authority as the graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDependencyRequirement {
    pub importer: ModuleId,
    pub dependency: ModuleId,
    pub range: SourceRange,
    pub reason: RuntimeDependencyReason,
    pub read_target: Option<LinkedReadSpec>,
    /// Exact producer, target, product, and timing for this edge.
    pub provenance: RuntimeRequirementProvenance,
}

impl RuntimeDependencyRequirement {
    /// Orders the complete record, including read effect and observation
    /// provenance, independently of producer traversal order.
    pub fn canonical_cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            &self.importer,
            &self.dependency,
            self.range,
            self.reason,
            &self.read_target,
            &self.provenance.target,
            self.provenance.product,
            self.provenance.timing,
            &self.provenance.consumer,
            self.provenance.producer,
        )
            .cmp(&(
                &other.importer,
                &other.dependency,
                other.range,
                other.reason,
                &other.read_target,
                &other.provenance.target,
                other.provenance.product,
                other.provenance.timing,
                &other.provenance.consumer,
                other.provenance.producer,
            ))
    }

    pub fn edge(&self) -> RuntimeDependencyEdge {
        RuntimeDependencyEdge {
            importer: self.importer.clone(),
            dependency: self.dependency.clone(),
            range: self.range,
            reason: self.reason,
            provenance: Some(self.provenance.clone()),
        }
    }
}

/// Authority that guarantees an exact product is installed by a given stage.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AvailabilityAuthority {
    /// Exact module identity was established by semantic linking.
    SemanticIdentityCatalog,
    /// Native binding was written by the Universe binding installation catalog.
    NativeBindingCatalog,
    /// Canonical module object was created by native Universe materialization.
    NativeModuleMaterialization,
    /// Descriptor was installed from the immutable semantic metadata bundle.
    SemanticMetadataBundle,
    /// The canonical source module initializer produced this value.
    CanonicalSourceInitializer,
    /// The full canonical Universe bootstrap completed before user execution.
    FullyInitializedUniverse,
}

/// One exact availability proof. It contains no VM or heap handles.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AvailabilityProof {
    /// Exact module or binding whose product is guaranteed.
    pub target: RuntimeRequirementTarget,
    /// Product guaranteed for the target.
    pub product: RuntimeRequirementProduct,
    /// Installation or initialization authority.
    pub authority: AvailabilityAuthority,
    /// Latest stage at which the product is guaranteed available.
    pub available_at: RuntimeRequirementTiming,
}

/// Immutable exact availability facts accepted during executable finalization.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AvailabilityContract {
    proofs: BTreeSet<AvailabilityProof>,
}

impl AvailabilityContract {
    /// Builds a contract from immutable exact proofs.
    pub fn from_proofs(proofs: impl IntoIterator<Item = AvailabilityProof>) -> Self {
        Self {
            proofs: proofs.into_iter().collect(),
        }
    }

    /// Returns the exact proofs carried by this contract.
    pub fn proofs(&self) -> impl Iterator<Item = &AvailabilityProof> {
        self.proofs.iter()
    }

    fn satisfies(&self, provenance: &RuntimeRequirementProvenance) -> bool {
        self.proofs.iter().any(|proof| {
            let available_for_consumer = proof.available_at <= provenance.timing
                || (proof.authority == AvailabilityAuthority::FullyInitializedUniverse
                    && provenance.timing == RuntimeRequirementTiming::DuringModuleInitializer);
            proof.target == provenance.target && proof.product == provenance.product && available_for_consumer
        })
    }
}

/// Canonical semantic target, independent of runtime read allocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SymbolicImportTarget {
    Module(ModuleId),
    Binding(SymbolId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolicImportBinding {
    pub id: SourceImportBindingId,
    pub local_name: Box<str>,
    pub target: SymbolicImportTarget,
    /// Authored provider, which may differ from the canonical declaration owner.
    pub provider: ModuleId,
    pub kind: ReferenceKind,
    pub range: SourceRange,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SymbolicImportTable {
    pub bindings: BTreeMap<SourceImportBindingId, SymbolicImportBinding>,
    pub by_local_name: BTreeMap<Box<str>, SourceImportBindingId>,
}

impl SymbolicImportTable {
    pub fn get(&self, name: &str) -> Option<&SymbolicImportBinding> {
        self.by_local_name.get(name).and_then(|id| self.bindings.get(id))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticLinkedModule {
    pub interface: LinkedModuleInterface,
    pub local_globals: BTreeMap<Box<str>, GlobalBindingId>,
    pub imports: SymbolicImportTable,
}

/// Symbolic linkage has no runtime slots, graph or initialization order.
/// Executable consumers accept `LinkedProgram`, never this provisional type.
#[derive(Clone, Debug)]
pub struct SemanticLinkedProgram {
    pub universe: Arc<ProjectUniverse>,
    pub modules: BTreeMap<ModuleId, SemanticLinkedModule>,
    pub references: crate::graph::ReferenceGraph,
    pub semantics: crate::graph::SemanticGraph,
    pub entry: ModuleId,
}

impl SemanticLinkedProgram {
    /// Projects every symbolic source import into an executable-shaped program
    /// without claiming that its runtime graph has been validated.
    ///
    /// This compatibility projection is useful while a caller is migrating to
    /// semantic phase evidence. Executable compilation must use
    /// [`ModuleLinker::finalize_executable`] instead.
    pub fn to_unfinalized_program(&self) -> Arc<LinkedProgram> {
        let mut graphs = ModuleGraphs {
            references: self.references.clone(),
            semantics: self.semantics.clone(),
            runtime: crate::graph::RuntimeDependencyGraph::default(),
        };
        let mut modules = BTreeMap::new();

        for (module, symbolic) in &self.modules {
            graphs.runtime.add_node(module.clone());
            let mut bindings = ModuleBindingLayout {
                local_globals: symbolic.local_globals.clone().into(),
                imports: BTreeMap::new().into(),
            };
            let mut linked_reads = Vec::new();

            for import in symbolic.imports.bindings.values() {
                let target_module = symbolic_import_target_module(&import.target);
                let read = symbolic_import_read(&import.target);
                let slot = ImportBindingId(linked_reads.len() as u32);
                linked_reads.push(read);
                bindings.imports.insert(import.local_name.clone(), slot);

                graphs.runtime.add(RuntimeDependencyEdge {
                    importer: module.clone(),
                    dependency: target_module,
                    range: import.range,
                    reason: runtime_reason(import.kind),
                    provenance: None,
                });
            }

            let mut runtime_dependencies = graphs.runtime.dependencies(module);
            runtime_dependencies.sort();
            modules.insert(
                module.clone(),
                LinkedModule {
                    interface: symbolic.interface.clone(),
                    bindings,
                    linked_reads,
                    runtime_dependencies,
                },
            );
        }

        Arc::new(LinkedProgram {
            universe: self.universe.clone(),
            modules: modules.into(),
            graphs,
            entry: self.entry.clone(),
            initialization_order: Vec::new(),
        })
    }
}

fn symbolic_import_target_module(target: &SymbolicImportTarget) -> ModuleId {
    match target {
        SymbolicImportTarget::Module(module) => module.clone(),
        SymbolicImportTarget::Binding(symbol) => symbol.module.clone(),
    }
}

fn symbolic_import_read(target: &SymbolicImportTarget) -> LinkedReadSpec {
    match target {
        SymbolicImportTarget::Module(module) => LinkedReadSpec::Module(module.clone()),
        SymbolicImportTarget::Binding(symbol) => LinkedReadSpec::Binding(symbol.clone()),
    }
}

fn linked_read_module(target: &LinkedReadSpec) -> &ModuleId {
    match target {
        LinkedReadSpec::Module(module) => module,
        LinkedReadSpec::Binding(symbol) => &symbol.module,
    }
}

fn requirement_target_matches(read: &LinkedReadSpec, target: &RuntimeRequirementTarget) -> bool {
    match (read, target) {
        (LinkedReadSpec::Module(read), RuntimeRequirementTarget::Module(target)) => read == target,
        (LinkedReadSpec::Binding(read), RuntimeRequirementTarget::Binding { module, name }) => read.module == *module && read.name == *name,
        _ => false,
    }
}

fn requirement_target_from_read(read: &LinkedReadSpec) -> RuntimeRequirementTarget {
    match read {
        LinkedReadSpec::Module(module) => RuntimeRequirementTarget::Module(module.clone()),
        LinkedReadSpec::Binding(symbol) => RuntimeRequirementTarget::Binding {
            module: symbol.module.clone(),
            name: symbol.name.clone(),
        },
    }
}

fn runtime_reason(kind: ReferenceKind) -> RuntimeDependencyReason {
    match kind {
        ReferenceKind::WholeModuleImport => RuntimeDependencyReason::WholeModuleImport,
        ReferenceKind::ReExport => RuntimeDependencyReason::ReExport,
        _ => RuntimeDependencyReason::SelectiveValueImport,
    }
}

/// Whether semantic analysis covered every phase-relevant owner of a module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhaseEvidenceCoverage {
    Complete,
    Incomplete,
}

/// Conservative phase evidence for one source import binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportPhaseSummary {
    pub phase: DependencyPhase,
    pub observed: bool,
}

/// Semantic phase product consumed by executable finalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleImportPhaseProduct {
    pub module: ModuleId,
    pub bindings: crate::authority::ModuleAuthorityMap<SourceImportBindingId, ImportPhaseSummary>,
    pub coverage: PhaseEvidenceCoverage,
}

/// Read-only lookup accepted by executable finalization for source and frozen phase products.
pub trait ModuleImportPhaseProductLookup {
    fn get(&self, module: &ModuleId) -> Option<&ModuleImportPhaseProduct>;
}

impl ModuleImportPhaseProductLookup for BTreeMap<ModuleId, ModuleImportPhaseProduct> {
    fn get(&self, module: &ModuleId) -> Option<&ModuleImportPhaseProduct> {
        BTreeMap::get(self, module)
    }
}

impl ModuleImportPhaseProductLookup for crate::authority::ModuleAuthorityMap<ModuleId, ModuleImportPhaseProduct> {
    fn get(&self, module: &ModuleId) -> Option<&ModuleImportPhaseProduct> {
        crate::authority::ModuleAuthorityMap::get(self, module)
    }
}

impl<T: ModuleImportPhaseProductLookup> ModuleImportPhaseProductLookup for std::sync::Arc<T> {
    fn get(&self, module: &ModuleId) -> Option<&ModuleImportPhaseProduct> {
        (**self).get(module)
    }
}

/// The only linked product accepted by executable compilation/materialization.
#[derive(Clone, Debug)]
pub struct ExecutableLinkedProgram {
    program: LinkedProgram,
    canonical_runtime_reads: BTreeMap<(ModuleId, LinkedReadSpec), ImportBindingId>,
    runtime_requirements: Vec<RuntimeDependencyRequirement>,
}

impl ExecutableLinkedProgram {
    pub fn as_program(&self) -> &LinkedProgram {
        &self.program
    }

    /// Returns the immutable slot assigned to one canonical runtime target.
    pub fn runtime_read(&self, importer: &ModuleId, target: &LinkedReadSpec) -> Option<ImportBindingId> {
        self.canonical_runtime_reads.get(&(importer.clone(), target.clone())).copied()
    }

    /// Returns the canonical runtime read map for one importing module.
    pub fn runtime_reads_for(&self, importer: &ModuleId) -> impl Iterator<Item = (&LinkedReadSpec, ImportBindingId)> {
        self.canonical_runtime_reads
            .iter()
            .filter_map(move |((module, target), slot)| (module == importer).then_some((target, *slot)))
    }

    /// Validates a legacy conservative link product before wrapping it in the
    /// executable stage. Unlike phase-aware finalization, this path retains all
    /// existing reads and runtime edges.
    pub fn from_conservative_link(mut program: LinkedProgram) -> Result<Self, LinkError> {
        let known_modules = program.modules.keys().cloned().collect::<BTreeSet<_>>();
        for module in program.graphs.runtime.nodes() {
            if !known_modules.contains(&module) {
                return Err(LinkError::MissingModule { module });
            }
        }

        let mut canonical_runtime_reads = BTreeMap::new();
        for (module_id, module) in &program.modules {
            for (slot, read) in module.linked_reads.iter().enumerate().map(|(slot, read)| (slot as u32, read)) {
                let dependency = linked_read_module(read);
                if !program.graphs.runtime.edges_from(module_id).iter().any(|edge| &edge.dependency == dependency) {
                    return Err(LinkError::UncoveredRuntimeRead {
                        importer: module_id.clone(),
                        dependency: dependency.clone(),
                    });
                }
                canonical_runtime_reads
                    .entry((module_id.clone(), read.clone()))
                    .or_insert(ImportBindingId(slot));
            }
            for slot in module.bindings.imports.values() {
                if slot.0 as usize >= module.linked_reads.len() {
                    return Err(LinkError::InvalidImportSlot {
                        module: module_id.clone(),
                        slot: slot.0,
                    });
                }
            }
        }

        let initialization_order = program.graphs.runtime.initialization_order().map_err(LinkError::RuntimeCycle)?;
        for (module_id, module) in program
            .modules
            .as_source_mut()
            .expect("source linker output must remain mutable until finalization")
            .iter_mut()
        {
            module.runtime_dependencies = program.graphs.runtime.dependencies(module_id);
        }
        program.initialization_order = initialization_order;
        Ok(Self {
            program,
            canonical_runtime_reads,
            runtime_requirements: Vec::new(),
        })
    }

    /// Returns the exact generated requirements retained during finalization.
    pub fn runtime_requirements(&self) -> &[RuntimeDependencyRequirement] {
        &self.runtime_requirements
    }
}

impl std::ops::Deref for ExecutableLinkedProgram {
    type Target = LinkedProgram;

    fn deref(&self) -> &Self::Target {
        &self.program
    }
}

/// Linked local binding layout. IDs are symbolic and VM-independent.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModuleBindingLayout {
    /// Module-owned globals.
    pub local_globals: ModuleAuthorityMap<Box<str>, GlobalBindingId>,
    /// Immutable imported bindings.
    pub imports: ModuleAuthorityMap<Box<str>, ImportBindingId>,
}

/// Index of a local global in a linked module layout.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GlobalBindingId(pub u32);

/// Index of an imported linked read in a linked module layout.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ImportBindingId(pub u32);

/// A linked module and its VM-independent reads/dependencies.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkedModule {
    /// Canonical public interface.
    pub interface: LinkedModuleInterface,
    /// Local/import binding names.
    pub bindings: ModuleBindingLayout,
    /// Indexed reads emitted by a compiler.
    pub linked_reads: Vec<LinkedReadSpec>,
    /// Runtime dependencies in initialization order-independent form.
    pub runtime_dependencies: Vec<ModuleId>,
}

/// A closed, statically linked program plan.
#[derive(Clone, Debug)]
pub struct LinkedProgram {
    /// Resolved project/package universe.
    pub universe: Arc<ProjectUniverse>,
    /// Every linked reachable module.
    pub modules: ModuleAuthorityMap<ModuleId, LinkedModule>,
    /// Reference, semantic, and runtime graphs.
    pub graphs: ModuleGraphs,
    /// Selected entry module.
    pub entry: ModuleId,
    /// Deterministic runtime initialization order.
    pub initialization_order: Vec<ModuleId>,
}

/// Linker failure with source ownership retained for diagnostics.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum LinkError {
    /// A path used by an import was not resolved before linking.
    #[error("unresolved import '{path}' from module {module}")]
    UnresolvedImport { module: ModuleId, path: String, range: SourceRange },
    /// A selected name is absent from a target interface.
    ///
    /// `module` identifies the provider being queried; `importer` owns the
    /// source range and therefore owns the resulting diagnostic.
    #[error("module {module} does not export '{name}'")]
    MissingExport {
        module: ModuleId,
        importer: ModuleId,
        name: String,
        range: SourceRange,
    },
    /// A local name is not declared or imported.
    #[error("module {module} has no binding '{name}'")]
    MissingBinding { module: ModuleId, name: String, range: SourceRange },
    /// A linked name collides with another top-level binding.
    #[error("module {module} has colliding binding '{name}'")]
    BindingCollision { module: ModuleId, name: String, range: SourceRange },
    /// The selected entry or a statically reached module is absent from the
    /// interface universe supplied to the linker.
    #[error("module {module} is absent from the link universe")]
    MissingModule { module: ModuleId },
    /// An import slot points outside its module's finalized read table.
    #[error("module {module} has invalid finalized import slot {slot}")]
    InvalidImportSlot { module: ModuleId, slot: u32 },
    /// A linked read is not represented in the runtime initialization graph.
    #[error("runtime read from {importer} to {dependency} is not covered by the runtime graph")]
    UncoveredRuntimeRead { importer: ModuleId, dependency: ModuleId },
    /// A generated read target and its declared graph dependency disagree.
    #[error("runtime requirement from {importer} declares {dependency} but targets {target_module}")]
    RuntimeReadTargetMismatch {
        importer: ModuleId,
        dependency: ModuleId,
        target_module: ModuleId,
    },
    /// Generated requirement provenance does not describe its actual canonical target.
    #[error("runtime requirement from {importer} has provenance inconsistent with dependency {dependency}")]
    RuntimeRequirementProvenanceMismatch { importer: ModuleId, dependency: ModuleId },
    /// A re-export loop cannot produce a canonical symbol.
    #[error("cyclic re-export involving module {module} and '{name}'")]
    CyclicReExport { module: ModuleId, name: String },
    /// Runtime graph validation failed.
    #[error(transparent)]
    RuntimeCycle(#[from] ModuleGraphError),
}

impl LinkError {
    pub fn module(&self) -> Option<&ModuleId> {
        match self {
            Self::UnresolvedImport { module, .. }
            | Self::MissingExport { module, .. }
            | Self::MissingBinding { module, .. }
            | Self::BindingCollision { module, .. }
            | Self::MissingModule { module }
            | Self::InvalidImportSlot { module, .. }
            | Self::CyclicReExport { module, .. } => Some(module),
            Self::UncoveredRuntimeRead { importer, .. }
            | Self::RuntimeReadTargetMismatch { importer, .. }
            | Self::RuntimeRequirementProvenanceMismatch { importer, .. } => Some(importer),
            Self::RuntimeCycle(ModuleGraphError::RuntimeCycle { cycle, .. }) => cycle.first(),
        }
    }

    pub fn range(&self) -> SourceRange {
        match self {
            Self::UnresolvedImport { range, .. }
            | Self::MissingExport { range, .. }
            | Self::MissingBinding { range, .. }
            | Self::BindingCollision { range, .. } => *range,
            _ => SourceRange::default(),
        }
    }
}

/// Result of linking reachable components in workspace/tolerant mode.
#[derive(Clone, Debug)]
pub struct TolerantLinkResult {
    /// Linked program containing all successfully linked modules.
    pub program: LinkedProgram,
    /// Diagnostics accumulated during linking.
    pub diagnostics: Vec<LinkError>,
    /// Modules whose interfaces or bindings failed to link and could not produce valid products.
    pub blocked_modules: BTreeSet<ModuleId>,
}

/// Static linker over source-local interfaces and pre-resolved import targets.
pub struct ModuleLinker {
    universe: Arc<ProjectUniverse>,
    interfaces: BTreeMap<ModuleId, UnlinkedModuleInterface>,
    external: Option<Arc<crate::canonical_products::CanonicalExternalModuleWorld>>,
}

impl ModuleLinker {
    /// Creates a linker. The interface map is the complete source universe for
    /// this linking generation; no source is parsed during linking.
    pub fn new(universe: Arc<ProjectUniverse>, interfaces: BTreeMap<ModuleId, UnlinkedModuleInterface>) -> Self {
        Self {
            universe,
            interfaces,
            external: None,
        }
    }

    /// Resolves remote exports against completed immutable module products.
    /// External modules never become local linking or initialization work.
    pub fn with_external_modules(mut self, external: Arc<crate::canonical_products::CanonicalExternalModuleWorld>) -> Self {
        self.external = Some(external);
        self
    }

    fn is_external(&self, module: &ModuleId) -> bool {
        self.external.as_ref().is_some_and(|image| image.linked().modules.contains_key(module))
    }

    /// Links all interfaces using `(importer, logical path)` resolutions.
    ///
    /// Resolution remains a separate phase so filesystem/package policy cannot
    /// accidentally leak into symbol linking.
    pub fn link(&self, entry: ModuleId, resolved: &BTreeMap<(ModuleId, String), ModuleId>) -> Result<LinkedProgram, LinkError> {
        self.link_inner(entry, resolved, false)
    }

    /// Links the currently available source universe while retaining modules
    /// whose imports are temporarily unresolved. This is used by persistent
    /// workspace updates so a deleted dependency can publish the importer and
    /// its semantic diagnostics instead of retaining the removed module in the
    /// last-known-good snapshot.
    pub fn link_with_unresolved_imports(&self, entry: ModuleId, resolved: &BTreeMap<(ModuleId, String), ModuleId>) -> Result<LinkedProgram, LinkError> {
        self.link_inner(entry, resolved, true)
    }

    /// Links the reachable component starting at `entry` in tolerant workspace mode,
    /// accumulating diagnostics and marking affected modules as blocked without
    /// failing unaffected canonical module products.
    pub fn link_component_tolerant(&self, entry: ModuleId, resolved: &BTreeMap<(ModuleId, String), ModuleId>) -> TolerantLinkResult {
        let reachable = match self.reachable_interfaces(&entry, resolved, true) {
            Ok(reachable) => reachable,
            Err(err) => {
                let mut blocked = BTreeSet::new();
                blocked.insert(entry.clone());
                return TolerantLinkResult {
                    program: LinkedProgram {
                        universe: self.universe.clone(),
                        modules: ModuleAuthorityMap::new(),
                        graphs: ModuleGraphs::default(),
                        entry,
                        initialization_order: Vec::new(),
                    },
                    diagnostics: vec![err],
                    blocked_modules: blocked,
                };
            }
        };
        let reachable_interfaces = self
            .interfaces
            .iter()
            .filter(|(module, _)| reachable.contains(*module))
            .map(|(module, interface)| (module.clone(), interface.clone()))
            .collect();
        let mut reachable_linker = ModuleLinker::new(self.universe.clone(), reachable_interfaces);
        reachable_linker.external = self.external.clone();
        let mut context = LinkContext::new(&reachable_linker, resolved, true, true);
        match context.build() {
            Ok((modules, graphs, initialization_order)) => TolerantLinkResult {
                program: LinkedProgram {
                    universe: self.universe.clone(),
                    modules: modules.into(),
                    graphs,
                    entry,
                    initialization_order,
                },
                diagnostics: context.diagnostics,
                blocked_modules: context.blocked_modules,
            },
            Err(err) => {
                let mut blocked = context.blocked_modules;
                blocked.insert(entry.clone());
                let mut diags = context.diagnostics;
                diags.push(err);
                TolerantLinkResult {
                    program: LinkedProgram {
                        universe: self.universe.clone(),
                        modules: ModuleAuthorityMap::new(),
                        graphs: ModuleGraphs::default(),
                        entry,
                        initialization_order: Vec::new(),
                    },
                    diagnostics: diags,
                    blocked_modules: blocked,
                }
            }
        }
    }

    /// Links the exact interfaces stored in this linker in tolerant workspace mode.
    pub fn link_component_interfaces_tolerant(&self, entry: ModuleId, resolved: &BTreeMap<(ModuleId, String), ModuleId>) -> TolerantLinkResult {
        let mut context = LinkContext::new(self, resolved, true, true);
        match context.build() {
            Ok((modules, graphs, initialization_order)) => TolerantLinkResult {
                program: LinkedProgram {
                    universe: self.universe.clone(),
                    modules: modules.into(),
                    graphs,
                    entry,
                    initialization_order,
                },
                diagnostics: context.diagnostics,
                blocked_modules: context.blocked_modules,
            },
            Err(err) => {
                let mut blocked = context.blocked_modules;
                blocked.insert(entry.clone());
                let mut diags = context.diagnostics;
                diags.push(err);
                TolerantLinkResult {
                    program: LinkedProgram {
                        universe: self.universe.clone(),
                        modules: ModuleAuthorityMap::new(),
                        graphs: ModuleGraphs::default(),
                        entry,
                        initialization_order: Vec::new(),
                    },
                    diagnostics: diags,
                    blocked_modules: blocked,
                }
            }
        }
    }

    /// Links every interface supplied to this linker, retaining the complete
    /// source universe for workspace-wide semantic analysis.
    pub fn link_all(&self, entry: ModuleId, resolved: &BTreeMap<(ModuleId, String), ModuleId>) -> Result<LinkedProgram, LinkError> {
        let mut context = LinkContext::new(self, resolved, false, false);
        context.build().map(|(modules, graphs, initialization_order)| LinkedProgram {
            universe: self.universe.clone(),
            modules: modules.into(),
            graphs,
            entry,
            initialization_order,
        })
    }

    /// Links all canonical names without runtime cycle validation or slot
    /// allocation. Semantic analysis can therefore resolve interface cycles.
    pub fn link_semantic(&self, entry: ModuleId, resolved: &BTreeMap<(ModuleId, String), ModuleId>) -> Result<SemanticLinkedProgram, LinkError> {
        if !self.interfaces.contains_key(&entry) {
            return Err(LinkError::MissingModule { module: entry });
        }
        let mut context = LinkContext::new(self, resolved, false, false);
        context.collect_imports_and_graphs()?;
        let mut modules = BTreeMap::new();
        for (module, interface) in &self.interfaces {
            let mut imports = SymbolicImportTable::default();
            for (site, import) in interface.import_sites() {
                let provider = context.target(module, import.path(), import.range())?;
                let items: Vec<(String, SourceRange, ReferenceKind)> = match import {
                    ImportSurface::Module(decl) => vec![(
                        decl.alias
                            .as_ref()
                            .map(|alias| alias.name.clone())
                            .or_else(|| default_module_binding(&decl.path).map(str::to_owned))
                            .unwrap_or_else(|| provider.path.to_string()),
                        decl.range,
                        ReferenceKind::WholeModuleImport,
                    )],
                    ImportSurface::Selective(decl) => decl
                        .items
                        .iter()
                        .map(|item| {
                            (
                                item.alias.as_ref().map(|alias| alias.name.clone()).unwrap_or_else(|| item.name.clone()),
                                item.range,
                                ReferenceKind::SelectiveImport,
                            )
                        })
                        .collect(),
                    ImportSurface::ReExport(decl) => decl
                        .items
                        .iter()
                        .map(|item| (item.local_or_remote_name.clone(), item.range, ReferenceKind::ReExport))
                        .collect(),
                };
                for (ordinal, (local, range, kind)) in items.into_iter().enumerate() {
                    let target = match &context.import_targets[&(module.clone(), local.clone())] {
                        LinkedReadSpec::Module(target) => SymbolicImportTarget::Module(target.clone()),
                        LinkedReadSpec::Binding(target) => SymbolicImportTarget::Binding(target.clone()),
                    };
                    let id = SourceImportBindingId {
                        site: site.clone(),
                        local: ImportBindingLocalId(ordinal as u32),
                    };
                    imports.by_local_name.insert(local.clone().into_boxed_str(), id.clone());
                    imports.bindings.insert(
                        id.clone(),
                        SymbolicImportBinding {
                            id,
                            local_name: local.into_boxed_str(),
                            target,
                            provider: provider.clone(),
                            kind,
                            range,
                        },
                    );
                }
            }
            let local_globals = interface
                .declarations
                .keys()
                .enumerate()
                .map(|(index, name)| (name.clone().into_boxed_str(), GlobalBindingId(index as u32)))
                .collect();
            modules.insert(
                module.clone(),
                SemanticLinkedModule {
                    interface: context.linked_interface(module)?,
                    local_globals,
                    imports,
                },
            );
        }
        Ok(SemanticLinkedProgram {
            universe: self.universe.clone(),
            modules,
            references: context.graphs.references,
            semantics: context.graphs.semantics,
            entry,
        })
    }

    /// Finalizes runtime slots and the strict initialization DAG from the
    /// symbolic product and semantic phase evidence. Missing or incomplete
    /// evidence remains runtime by construction.
    pub fn finalize_executable(
        &self,
        semantic: SemanticLinkedProgram,
        phase_products: &impl ModuleImportPhaseProductLookup,
    ) -> Result<ExecutableLinkedProgram, LinkError> {
        self.finalize_executable_with_requirements(semantic, phase_products, &[])
    }

    /// Finalizes runtime slots and the strict initialization DAG from
    /// symbolic imports, semantic phase evidence, and generated runtime
    /// declaration requirements. Missing or incomplete source evidence stays
    /// runtime by construction.
    pub fn finalize_executable_with_requirements(
        &self,
        semantic: SemanticLinkedProgram,
        phase_products: &impl ModuleImportPhaseProductLookup,
        requirements: &[RuntimeDependencyRequirement],
    ) -> Result<ExecutableLinkedProgram, LinkError> {
        self.finalize_executable_with_availability(semantic, phase_products, requirements, &AvailabilityContract::default())
    }

    /// Finalizes executable reads and strict runtime ordering using exact
    /// availability proofs. Read slots are allocated even when a proof removes
    /// the corresponding eager initialization edge.
    pub fn finalize_executable_with_availability(
        &self,
        semantic: SemanticLinkedProgram,
        phase_products: &impl ModuleImportPhaseProductLookup,
        requirements: &[RuntimeDependencyRequirement],
        availability: &AvailabilityContract,
    ) -> Result<ExecutableLinkedProgram, LinkError> {
        let SemanticLinkedProgram {
            universe,
            modules: semantic_modules,
            references,
            semantics,
            entry,
        } = semantic;
        let mut modules = BTreeMap::new();
        let mut graphs = ModuleGraphs {
            references,
            semantics,
            runtime: crate::graph::RuntimeDependencyGraph::default(),
        };
        let mut canonical_runtime_reads = BTreeMap::new();
        for (module, symbolic) in semantic_modules {
            graphs.runtime.add_node(module.clone());
            let evidence = phase_products.get(&module);
            let mut bindings = ModuleBindingLayout {
                local_globals: symbolic.local_globals.clone().into(),
                imports: BTreeMap::new().into(),
            };
            let mut linked_reads = Vec::new();
            for (id, import) in symbolic.imports.bindings {
                let target_module = symbolic_import_target_module(&import.target);
                let runtime = match import.kind {
                    ReferenceKind::WholeModuleImport | ReferenceKind::ReExport => true,
                    ReferenceKind::SelectiveImport => {
                        evidence.is_none_or(|product| product.module != module || product.coverage != PhaseEvidenceCoverage::Complete)
                            || evidence
                                .and_then(|product| product.bindings.get(&id))
                                .is_none_or(|summary| !summary.observed || summary.phase == DependencyPhase::Runtime)
                    }
                    // This is not an authored import kind. If one reaches the
                    // executable projection without complete selective-import
                    // evidence, retain it conservatively.
                    ReferenceKind::InterfaceOnly => true,
                };
                if !runtime {
                    continue;
                }
                let read = symbolic_import_read(&import.target);
                let slot = ImportBindingId(linked_reads.len() as u32);
                linked_reads.push(read);
                canonical_runtime_reads
                    .entry((module.clone(), linked_reads[slot.0 as usize].clone()))
                    .or_insert(slot);
                bindings.imports.insert(import.local_name.clone(), slot);
                let read = &linked_reads[slot.0 as usize];
                let target = requirement_target_from_read(read);
                let default_provenance = RuntimeRequirementProvenance {
                    consumer: module.clone(),
                    producer: crate::graph::RuntimeRequirementProducer::ImportRead,
                    product: target.conservative_product(),
                    target,
                    timing: RuntimeRequirementTiming::DuringModuleInitializer,
                };
                let mut import_provenances = requirements
                    .iter()
                    .filter(|requirement| {
                        requirement.importer == module
                            && requirement.provenance.producer == crate::graph::RuntimeRequirementProducer::ImportRead
                            && requirement.read_target.as_ref() == Some(read)
                    })
                    .map(|requirement| requirement.provenance.clone())
                    .collect::<Vec<_>>();
                import_provenances.sort();
                import_provenances.dedup();
                let unsatisfied_provenance = import_provenances.iter().find(|provenance| !availability.satisfies(provenance)).cloned();
                let unsatisfied_default = import_provenances
                    .is_empty()
                    .then(|| (!availability.satisfies(&default_provenance)).then_some(default_provenance));
                if let Some(provenance) = unsatisfied_provenance.or_else(|| unsatisfied_default.flatten()) {
                    graphs.runtime.add(RuntimeDependencyEdge {
                        importer: module.clone(),
                        dependency: target_module,
                        range: import.range,
                        reason: runtime_reason(import.kind),
                        provenance: Some(provenance),
                    });
                }
            }
            modules.insert(
                module,
                LinkedModule {
                    interface: symbolic.interface,
                    bindings,
                    linked_reads,
                    runtime_dependencies: Vec::new(),
                },
            );
        }

        let known_modules = modules.keys().cloned().collect::<BTreeSet<_>>();
        let mut ordered_requirements = requirements.to_vec();
        ordered_requirements.sort_by(RuntimeDependencyRequirement::canonical_cmp);
        for requirement in &ordered_requirements {
            if !known_modules.contains(&requirement.importer) {
                return Err(LinkError::MissingModule {
                    module: requirement.importer.clone(),
                });
            }
            // Canonical Universe modules are preinitialized by the VM and
            // may own source-backed bindings that the user program did not
            // import explicitly. They still participate in strict DAG checks
            // and receive finalized read slots, but are not compiled as part
            // of this linked source closure.
            if !known_modules.contains(&requirement.dependency) && !requirement.dependency.project.is_universe() {
                return Err(LinkError::MissingModule {
                    module: requirement.dependency.clone(),
                });
            }
            if let Some(target) = &requirement.read_target {
                let target_module = linked_read_module(target);
                if target_module != &requirement.dependency {
                    return Err(LinkError::RuntimeReadTargetMismatch {
                        importer: requirement.importer.clone(),
                        dependency: requirement.dependency.clone(),
                        target_module: target_module.clone(),
                    });
                }
                if let std::collections::btree_map::Entry::Vacant(entry) = canonical_runtime_reads.entry((requirement.importer.clone(), target.clone())) {
                    let linked = modules
                        .get_mut(&requirement.importer)
                        .expect("known runtime requirement importer has a linked module");
                    let slot = ImportBindingId(linked.linked_reads.len() as u32);
                    linked.linked_reads.push(target.clone());
                    entry.insert(slot);
                }
            }
            if requirement.provenance.consumer != requirement.importer
                || requirement.provenance.target.module() != &requirement.dependency
                || requirement
                    .read_target
                    .as_ref()
                    .is_some_and(|target| !requirement_target_matches(target, &requirement.provenance.target))
            {
                return Err(LinkError::RuntimeRequirementProvenanceMismatch {
                    importer: requirement.importer.clone(),
                    dependency: requirement.dependency.clone(),
                });
            }
            if !availability.satisfies(&requirement.provenance) {
                graphs.runtime.add(requirement.edge());
            }
        }

        for (module, linked) in &mut modules {
            linked.runtime_dependencies = graphs.runtime.dependencies(module);
        }
        let initialization_order = graphs.runtime.initialization_order().map_err(LinkError::RuntimeCycle)?;
        Ok(ExecutableLinkedProgram {
            program: LinkedProgram {
                universe,
                modules: modules.into(),
                graphs,
                entry,
                initialization_order,
            },
            canonical_runtime_reads,
            runtime_requirements: ordered_requirements,
        })
    }

    fn link_inner(
        &self,
        entry: ModuleId,
        resolved: &BTreeMap<(ModuleId, String), ModuleId>,
        allow_unresolved_imports: bool,
    ) -> Result<LinkedProgram, LinkError> {
        let reachable = self.reachable_interfaces(&entry, resolved, allow_unresolved_imports)?;
        let reachable_interfaces = self
            .interfaces
            .iter()
            .filter(|(module, _)| reachable.contains(*module))
            .map(|(module, interface)| (module.clone(), interface.clone()))
            .collect();
        let mut reachable_linker = ModuleLinker::new(self.universe.clone(), reachable_interfaces);
        reachable_linker.external = self.external.clone();
        let mut context = LinkContext::new(&reachable_linker, resolved, allow_unresolved_imports, false);
        context.build().map(|(modules, graphs, initialization_order)| LinkedProgram {
            universe: self.universe.clone(),
            modules: modules.into(),
            graphs,
            entry,
            initialization_order,
        })
    }

    pub(crate) fn reachable_interfaces(
        &self,
        entry: &ModuleId,
        resolved: &BTreeMap<(ModuleId, String), ModuleId>,
        allow_unresolved_imports: bool,
    ) -> Result<BTreeSet<ModuleId>, LinkError> {
        if !self.interfaces.contains_key(entry) {
            return Err(LinkError::MissingModule { module: entry.clone() });
        }
        let mut reachable = BTreeSet::from([entry.clone()]);
        let mut pending = vec![entry.clone()];

        if let Some(project_id) = entry.project.as_resolved() {
            let root_id = ModuleId::resolved(project_id, ModulePath::root());
            if self.interfaces.contains_key(&root_id) && reachable.insert(root_id.clone()) {
                pending.push(root_id);
            }
        } else if let Some(sid) = entry.project.as_synthetic() {
            let root_id = ModuleId::synthetic(sid, ModulePath::root());
            if self.interfaces.contains_key(&root_id) && reachable.insert(root_id.clone()) {
                pending.push(root_id);
            }
        }

        while let Some(module) = pending.pop() {
            let interface = self.interfaces.get(&module).expect("reachable interface exists");
            for import in &interface.imports {
                let path = match import {
                    ImportSurface::Module(decl) => (&decl.path, decl.range),
                    ImportSurface::Selective(decl) => (&decl.path, decl.range),
                    ImportSurface::ReExport(decl) => (&decl.path, decl.range),
                };
                let Some(target) = resolved
                    .get(&(module.clone(), path.0.to_string()))
                    .cloned()
                    .filter(|target| self.interfaces.contains_key(target) || self.is_external(target))
                else {
                    if allow_unresolved_imports {
                        continue;
                    }
                    return Err(LinkError::UnresolvedImport {
                        module: module.clone(),
                        path: path.0.to_string(),
                        range: path.1,
                    });
                };
                if self.is_external(&target) {
                    continue;
                }
                if reachable.insert(target.clone()) {
                    pending.push(target.clone());
                }
                if let Some(project_id) = target.project.as_resolved() {
                    let mut curr_path = target.path.parent();
                    while let Some(parent) = curr_path {
                        let pkg_id = ModuleId::resolved(project_id, parent.clone());
                        if self.interfaces.contains_key(&pkg_id) && reachable.insert(pkg_id.clone()) {
                            pending.push(pkg_id);
                        }
                        curr_path = parent.parent();
                    }
                    let root_id = ModuleId::resolved(project_id, ModulePath::root());
                    if self.interfaces.contains_key(&root_id) && reachable.insert(root_id.clone()) {
                        pending.push(root_id);
                    }
                }
            }
            for export in interface.exports.values() {
                let UnlinkedExportTarget::CanonicalDeclaration { module: target, .. } = &export.target else {
                    continue;
                };
                if self.is_external(target) {
                    continue;
                }
                if !self.interfaces.contains_key(target) {
                    return Err(LinkError::MissingModule { module: target.clone() });
                }
                if reachable.insert(target.clone()) {
                    pending.push(target.clone());
                }
            }
        }
        Ok(reachable)
    }

    /// Resolves a declaration-only static symbol through the current module's
    /// local declarations or a whole-module import alias.
    pub fn resolve_static_symbol(
        &self,
        module: &ModuleId,
        reference: &StaticSymbolRef,
        resolved: &BTreeMap<(ModuleId, String), ModuleId>,
    ) -> Result<SymbolId, LinkError> {
        let interface = self.interfaces.get(module).ok_or_else(|| LinkError::MissingBinding {
            module: module.clone(),
            name: reference.root.clone(),
            range: reference.range,
        })?;
        if reference.is_bare() {
            if interface.declarations.contains_key(&reference.root) {
                return Ok(SymbolId {
                    module: module.clone(),
                    name: reference.root.clone().into_boxed_str(),
                });
            }
            let mut context = LinkContext::new(self, resolved, false, false);
            context.collect_imports_and_graphs()?;
            for import in &interface.imports {
                let Some((path, remote, range)) = (match import {
                    ImportSurface::Selective(decl) => decl.items.iter().find_map(|item| {
                        let local = item.alias.as_ref().map(|alias| alias.name.as_str()).unwrap_or(item.name.as_str());
                        (local == reference.root).then_some((&decl.path, item.name.as_str(), item.range))
                    }),
                    ImportSurface::ReExport(decl) => decl.items.iter().find_map(|item| {
                        let local = item.local_or_remote_name.as_str();
                        (local == reference.root).then_some((&decl.path, item.local_or_remote_name.as_str(), item.range))
                    }),
                    ImportSurface::Module(_) => None,
                }) else {
                    continue;
                };
                let target = context.target(module, path, range)?;
                let export = context.resolve_export(&target, remote, range, module)?;
                let symbol = export.symbol().cloned().ok_or_else(|| LinkError::MissingBinding {
                    module: target.clone(),
                    name: remote.to_string(),
                    range,
                })?;
                return Ok(symbol);
            }
            return Err(LinkError::MissingBinding {
                module: module.clone(),
                name: reference.root.clone(),
                range: reference.root_range,
            });
        }

        let import = interface
            .imports
            .iter()
            .find_map(|import| {
                let ImportSurface::Module(decl) = import else { return None };
                let binding = decl
                    .alias
                    .as_ref()
                    .map(|alias| alias.name.as_str())
                    .or_else(|| default_module_binding(&decl.path))?;
                (binding == reference.root).then_some(decl)
            })
            .ok_or_else(|| LinkError::MissingBinding {
                module: module.clone(),
                name: reference.root.clone(),
                range: reference.root_range,
            })?;
        let target = resolved
            .get(&(module.clone(), import.path.to_string()))
            .cloned()
            .ok_or_else(|| LinkError::UnresolvedImport {
                module: module.clone(),
                path: import.path.to_string(),
                range: import.range,
            })?;
        if reference.members.len() != 1 {
            return Err(LinkError::MissingBinding {
                module: target,
                name: reference.leaf_name().to_string(),
                range: reference.range,
            });
        }
        let name = reference.leaf_name();
        let mut context = LinkContext::new(self, resolved, false, false);
        context.collect_imports_and_graphs()?;
        let export = context.resolve_export(&target, name, reference.range, module)?;
        let symbol = export.symbol().cloned().ok_or_else(|| LinkError::MissingBinding {
            module: target,
            name: name.to_string(),
            range: reference.range,
        })?;
        Ok(symbol)
    }
}

struct LinkContext<'a> {
    linker: &'a ModuleLinker,
    resolved: &'a BTreeMap<(ModuleId, String), ModuleId>,
    allow_unresolved_imports: bool,
    tolerant: bool,
    import_targets: BTreeMap<(ModuleId, String), LinkedReadSpec>,
    import_symbols: BTreeMap<(ModuleId, String), Option<SymbolId>>,
    linked_exports: BTreeMap<(ModuleId, String), LinkedExport>,
    resolving_exports: BTreeSet<(ModuleId, String)>,
    graphs: ModuleGraphs,
    pub diagnostics: Vec<LinkError>,
    pub blocked_modules: BTreeSet<ModuleId>,
}

type LinkBuild = (BTreeMap<ModuleId, LinkedModule>, ModuleGraphs, Vec<ModuleId>);

impl<'a> LinkContext<'a> {
    fn new(linker: &'a ModuleLinker, resolved: &'a BTreeMap<(ModuleId, String), ModuleId>, allow_unresolved_imports: bool, tolerant: bool) -> Self {
        Self {
            linker,
            resolved,
            allow_unresolved_imports,
            tolerant,
            import_targets: BTreeMap::new(),
            import_symbols: BTreeMap::new(),
            linked_exports: BTreeMap::new(),
            resolving_exports: BTreeSet::new(),
            graphs: ModuleGraphs::default(),
            diagnostics: Vec::new(),
            blocked_modules: BTreeSet::new(),
        }
    }

    fn build(&mut self) -> Result<LinkBuild, LinkError> {
        self.collect_imports_and_graphs()?;
        let module_ids = self.linker.interfaces.keys().cloned().collect::<Vec<_>>();
        for module in &module_ids {
            let names = self
                .linker
                .interfaces
                .get(module)
                .map(|interface| {
                    interface
                        .exports
                        .iter()
                        .map(|(name, surface)| (name.clone(), surface.range))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for (name, range) in names {
                match self.resolve_export(module, &name, range, module) {
                    Ok(_) => {}
                    Err(err) => {
                        if self.tolerant {
                            self.diagnostics.push(err);
                            self.blocked_modules.insert(module.clone());
                        } else {
                            return Err(err);
                        }
                    }
                }
            }
        }

        // Transitive blocked cascade: if any dependency of `module` is blocked,
        // `module` is also blocked.
        if self.tolerant && !self.blocked_modules.is_empty() {
            let mut changed = true;
            while changed {
                changed = false;
                for module in &module_ids {
                    if self.blocked_modules.contains(module) {
                        continue;
                    }
                    let has_blocked_dep = self
                        .graphs
                        .references
                        .edges_from(module)
                        .iter()
                        .any(|edge| self.blocked_modules.contains(&edge.to));
                    if has_blocked_dep {
                        self.blocked_modules.insert(module.clone());
                        changed = true;
                    }
                }
            }
        }

        // Runtime cycle detection: in tolerant mode, if a cycle occurs among valid modules,
        // capture diagnostic, block cycling modules, and compute a real topological order
        // for surviving unblocked modules.
        let initialization_order = match self.graphs.runtime.initialization_order() {
            Ok(order) => order,
            Err(err) => {
                if self.tolerant {
                    let crate::error::ModuleGraphError::RuntimeCycle { ref cycle, ref components, .. } = err;
                    for component in components {
                        self.blocked_modules.extend(component.iter().cloned());
                    }
                    if components.is_empty() {
                        self.blocked_modules.extend(cycle.iter().cloned());
                    }
                    self.diagnostics.push(LinkError::RuntimeCycle(err));

                    // Cascade blocked: if module depends on a blocked module at runtime
                    let mut changed = true;
                    while changed {
                        changed = false;
                        for module in &module_ids {
                            if self.blocked_modules.contains(module) {
                                continue;
                            }
                            let has_blocked_dep = self
                                .graphs
                                .runtime
                                .edges_from(module)
                                .iter()
                                .any(|edge| self.blocked_modules.contains(&edge.dependency));
                            if has_blocked_dep {
                                self.blocked_modules.insert(module.clone());
                                changed = true;
                            }
                        }
                    }

                    // Filter unblocked modules for real topological order on surviving subgraph
                    let unblocked_nodes: BTreeSet<ModuleId> = self.graphs.runtime.nodes().into_iter().filter(|m| !self.blocked_modules.contains(m)).collect();
                    let surviving_runtime = self.graphs.runtime.filtered_subgraph(&unblocked_nodes);
                    let order = surviving_runtime
                        .initialization_order()
                        .unwrap_or_else(|_| unblocked_nodes.into_iter().collect());
                    self.graphs.runtime = surviving_runtime;
                    order
                } else {
                    return Err(LinkError::RuntimeCycle(err));
                }
            }
        };

        let mut modules = BTreeMap::new();
        for module in module_ids {
            if self.tolerant && self.blocked_modules.contains(&module) {
                continue;
            }
            let interface = match self.linked_interface(&module) {
                Ok(iface) => iface,
                Err(err) => {
                    if self.tolerant {
                        self.diagnostics.push(err);
                        self.blocked_modules.insert(module);
                        continue;
                    } else {
                        return Err(err);
                    }
                }
            };
            let mut bindings = ModuleBindingLayout::default();
            let mut linked_reads = Vec::new();
            if let Some(unlinked) = self.linker.interfaces.get(&module) {
                for name in unlinked.declarations.keys() {
                    let next = bindings.local_globals.len() as u32;
                    bindings.local_globals.insert(name.clone().into_boxed_str(), GlobalBindingId(next));
                }
                for (local, read) in self
                    .import_targets
                    .iter()
                    .filter_map(|((owner, local), read)| (owner == &module).then_some((local, read)))
                {
                    let index = linked_reads.len() as u32;
                    linked_reads.push(read.clone());
                    bindings.imports.insert(local.clone().into_boxed_str(), ImportBindingId(index));
                }
            }
            let mut runtime_dependencies = self
                .graphs
                .runtime
                .edges_from(&module)
                .iter()
                .map(|edge| edge.dependency.clone())
                .collect::<Vec<_>>();
            runtime_dependencies.sort();
            runtime_dependencies.dedup();
            modules.insert(
                module,
                LinkedModule {
                    interface,
                    bindings,
                    linked_reads,
                    runtime_dependencies,
                },
            );
        }
        Ok((modules, std::mem::take(&mut self.graphs), initialization_order))
    }

    fn collect_imports_and_graphs(&mut self) -> Result<(), LinkError> {
        for (module, interface) in &self.linker.interfaces {
            self.graphs.references.add_node(module.clone());
            self.graphs.runtime.add_node(module.clone());
            let mut reference_edges = Vec::new();
            for import in &interface.imports {
                match import {
                    ImportSurface::Module(decl) => {
                        let Some(target) = self.target_if_present(module, &decl.path, decl.range)? else {
                            continue;
                        };
                        reference_edges.push(ReferenceEdge {
                            from: module.clone(),
                            to: target.clone(),
                            kind: ReferenceKind::WholeModuleImport,
                            range: decl.range,
                        });
                        self.graphs.semantics.add(SemanticEdge {
                            from: SemanticNodeId::Module(module.clone()),
                            to: SemanticNodeId::Module(target.clone()),
                            kind: SemanticEdgeKind::ModuleInterface,
                            range: decl.range,
                        });
                        self.graphs.runtime.add(RuntimeDependencyEdge {
                            importer: module.clone(),
                            dependency: target.clone(),
                            range: decl.range,
                            reason: RuntimeDependencyReason::WholeModuleImport,
                            provenance: None,
                        });
                        let local = decl
                            .alias
                            .as_ref()
                            .map(|alias| alias.name.clone())
                            .or_else(|| default_module_binding(&decl.path).map(str::to_owned))
                            .unwrap_or_else(|| target.path.to_string());
                        self.add_import(module, local, LinkedReadSpec::Module(target), None, decl.range)?;
                    }
                    ImportSurface::Selective(decl) => {
                        let Some(target) = self.target_if_present(module, &decl.path, decl.range)? else {
                            continue;
                        };
                        for item in &decl.items {
                            reference_edges.push(ReferenceEdge {
                                from: module.clone(),
                                to: target.clone(),
                                kind: ReferenceKind::SelectiveImport,
                                range: item.range,
                            });
                            self.graphs.semantics.add(SemanticEdge {
                                from: SemanticNodeId::Module(module.clone()),
                                to: SemanticNodeId::Module(target.clone()),
                                kind: SemanticEdgeKind::ModuleInterface,
                                range: item.range,
                            });
                            self.graphs.runtime.add(RuntimeDependencyEdge {
                                importer: module.clone(),
                                dependency: target.clone(),
                                range: item.range,
                                reason: RuntimeDependencyReason::SelectiveValueImport,
                                provenance: None,
                            });
                            let local = item.alias.as_ref().map(|alias| alias.name.clone()).unwrap_or_else(|| item.name.clone());
                            let provisional = SymbolId {
                                module: target.clone(),
                                name: item.name.clone().into_boxed_str(),
                            };
                            self.add_import(module, local, LinkedReadSpec::Binding(provisional.clone()), Some(provisional), item.range)?;
                        }
                    }
                    ImportSurface::ReExport(decl) => {
                        let Some(target) = self.target_if_present(module, &decl.path, decl.range)? else {
                            continue;
                        };
                        for item in &decl.items {
                            reference_edges.push(ReferenceEdge {
                                from: module.clone(),
                                to: target.clone(),
                                kind: ReferenceKind::ReExport,
                                range: item.range,
                            });
                            self.graphs.semantics.add(SemanticEdge {
                                from: SemanticNodeId::Module(module.clone()),
                                to: SemanticNodeId::Module(target.clone()),
                                kind: SemanticEdgeKind::ModuleInterface,
                                range: item.range,
                            });
                            self.graphs.runtime.add(RuntimeDependencyEdge {
                                importer: module.clone(),
                                dependency: target.clone(),
                                range: item.range,
                                reason: RuntimeDependencyReason::ReExport,
                                provenance: None,
                            });
                            let local = item.local_or_remote_name.clone();
                            let provisional = SymbolId {
                                module: target.clone(),
                                name: item.local_or_remote_name.clone().into_boxed_str(),
                            };
                            self.add_import(module, local, LinkedReadSpec::Binding(provisional.clone()), Some(provisional), item.range)?;
                        }
                    }
                }
            }
            self.graphs.references.replace(module.clone(), reference_edges);
        }
        // All local import names now exist, so canonicalize selected imports
        // and re-exports in a second pass. This keeps interface-map iteration
        // order from affecting linked symbol identity.
        for (module, interface) in &self.linker.interfaces {
            for import in &interface.imports {
                match import {
                    ImportSurface::Selective(decl) => {
                        let Some(target) = self.target_if_present(module, &decl.path, decl.range)? else {
                            continue;
                        };
                        for item in &decl.items {
                            let local = item.alias.as_ref().map(|alias| alias.name.clone()).unwrap_or_else(|| item.name.clone());
                            let linked_exp = match self.resolve_export(&target, &item.name, item.range, module) {
                                Ok(exp) => exp,
                                Err(err) => {
                                    if self.tolerant {
                                        self.diagnostics.push(err);
                                        self.blocked_modules.insert(module.clone());
                                        continue;
                                    } else {
                                        return Err(err);
                                    }
                                }
                            };
                            // Reads address the canonical export owner, which can differ
                            // from the authored import target after a re-export chain.
                            // Project that owner before validating initialization order.
                            let owner = match &linked_exp.target {
                                crate::interface::LinkedExportTarget::Binding(symbol) => symbol.module.clone(),
                                crate::interface::LinkedExportTarget::Module(module) => module.clone(),
                            };
                            self.graphs.runtime.add(RuntimeDependencyEdge {
                                importer: module.clone(),
                                dependency: owner,
                                range: item.range,
                                reason: RuntimeDependencyReason::SelectiveValueImport,
                                provenance: None,
                            });
                            match &linked_exp.target {
                                crate::interface::LinkedExportTarget::Binding(symbol) => {
                                    self.import_targets
                                        .insert((module.clone(), local.clone()), LinkedReadSpec::Binding(symbol.clone()));
                                    self.import_symbols.insert((module.clone(), local), Some(symbol.clone()));
                                }
                                crate::interface::LinkedExportTarget::Module(mod_id) => {
                                    self.import_targets
                                        .insert((module.clone(), local.clone()), LinkedReadSpec::Module(mod_id.clone()));
                                    self.import_symbols.insert((module.clone(), local), None);
                                }
                            }
                        }
                    }
                    ImportSurface::ReExport(decl) => {
                        let Some(target) = self.target_if_present(module, &decl.path, decl.range)? else {
                            continue;
                        };
                        for item in &decl.items {
                            let local = item.local_or_remote_name.clone();
                            let linked_exp = match self.resolve_export(&target, &item.local_or_remote_name, item.range, module) {
                                Ok(exp) => exp,
                                Err(err) => {
                                    if self.tolerant {
                                        self.diagnostics.push(err);
                                        self.blocked_modules.insert(module.clone());
                                        continue;
                                    } else {
                                        return Err(err);
                                    }
                                }
                            };
                            // Reads address the canonical export owner, which can differ
                            // from the authored import target after a re-export chain.
                            // Project that owner before validating initialization order.
                            let owner = match &linked_exp.target {
                                crate::interface::LinkedExportTarget::Binding(symbol) => symbol.module.clone(),
                                crate::interface::LinkedExportTarget::Module(module) => module.clone(),
                            };
                            self.graphs.runtime.add(RuntimeDependencyEdge {
                                importer: module.clone(),
                                dependency: owner,
                                range: item.range,
                                reason: RuntimeDependencyReason::ReExport,
                                provenance: None,
                            });
                            match &linked_exp.target {
                                crate::interface::LinkedExportTarget::Binding(symbol) => {
                                    self.import_targets
                                        .insert((module.clone(), local.clone()), LinkedReadSpec::Binding(symbol.clone()));
                                    self.import_symbols.insert((module.clone(), local), Some(symbol.clone()));
                                }
                                crate::interface::LinkedExportTarget::Module(mod_id) => {
                                    self.import_targets
                                        .insert((module.clone(), local.clone()), LinkedReadSpec::Module(mod_id.clone()));
                                    self.import_symbols.insert((module.clone(), local), None);
                                }
                            }
                        }
                    }
                    ImportSurface::Module(_) => {}
                }
            }
        }
        Ok(())
    }

    fn add_import(&mut self, module: &ModuleId, local: String, target: LinkedReadSpec, symbol: Option<SymbolId>, range: SourceRange) -> Result<(), LinkError> {
        let key = (module.clone(), local.clone());
        if self.import_targets.contains_key(&key) {
            return Err(LinkError::BindingCollision {
                module: module.clone(),
                name: local,
                range,
            });
        }
        self.import_targets.insert(key.clone(), target);
        self.import_symbols.insert(key, symbol);
        Ok(())
    }

    fn target(&self, module: &ModuleId, path: &ImportPath, range: SourceRange) -> Result<ModuleId, LinkError> {
        self.resolved
            .get(&(module.clone(), path.to_string()))
            .cloned()
            .filter(|target| self.linker.interfaces.contains_key(target) || self.linker.is_external(target))
            .ok_or_else(|| LinkError::UnresolvedImport {
                module: module.clone(),
                path: path.to_string(),
                range,
            })
    }

    fn target_if_present(&self, module: &ModuleId, path: &ImportPath, range: SourceRange) -> Result<Option<ModuleId>, LinkError> {
        match self.target(module, path, range) {
            Ok(target) => Ok(Some(target)),
            Err(LinkError::UnresolvedImport { .. }) if self.allow_unresolved_imports => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn resolve_export(&mut self, module: &ModuleId, name: &str, range: SourceRange, importer: &ModuleId) -> Result<LinkedExport, LinkError> {
        if let Some(remote) = self.linker.external.as_ref().and_then(|image| image.linked().modules.get(module)) {
            return remote.interface.exports.get(name).cloned().ok_or_else(|| LinkError::MissingExport {
                module: module.clone(),
                importer: importer.clone(),
                name: name.to_owned(),
                range,
            });
        }
        let key = (module.clone(), name.to_string());
        if let Some(export) = self.linked_exports.get(&key) {
            return Ok(export.clone());
        }
        if !self.resolving_exports.insert(key.clone()) {
            return Err(LinkError::CyclicReExport {
                module: module.clone(),
                name: name.to_string(),
            });
        }
        let interface = self
            .linker
            .interfaces
            .get(module)
            .ok_or_else(|| LinkError::MissingModule { module: module.clone() })?;
        let surface = interface.exports.get(name).ok_or_else(|| LinkError::MissingExport {
            module: module.clone(),
            importer: importer.clone(),
            name: name.to_string(),
            range,
        })?;
        let (target, export_range) = match &surface.target {
            UnlinkedExportTarget::Local(local) => {
                if let Some(declaration) = interface.declarations.get(local) {
                    (
                        crate::interface::LinkedExportTarget::Binding(SymbolId {
                            module: module.clone(),
                            name: local.clone().into_boxed_str(),
                        }),
                        declaration.range,
                    )
                } else if let Some(Some(symbol)) = self.import_symbols.get(&(module.clone(), local.clone())) {
                    (crate::interface::LinkedExportTarget::Binding(symbol.clone()), surface.range)
                } else if let Some(LinkedReadSpec::Module(target_mod)) = self.import_targets.get(&(module.clone(), local.clone())) {
                    (crate::interface::LinkedExportTarget::Module(target_mod.clone()), surface.range)
                } else {
                    return Err(LinkError::MissingBinding {
                        module: module.clone(),
                        name: local.clone(),
                        range: surface.range,
                    });
                }
            }
            UnlinkedExportTarget::ReExport { path, remote } => {
                let target_mod = self.target(module, path, surface.range)?;
                let linked = self.resolve_export(&target_mod, remote, surface.range, module)?;
                (linked.target, surface.range)
            }
            UnlinkedExportTarget::CanonicalDeclaration { module: target_module, name } => {
                let target_interface = self
                    .linker
                    .interfaces
                    .get(target_module)
                    .ok_or_else(|| LinkError::MissingModule { module: target_module.clone() })?;
                if !target_interface.declarations.contains_key(name) {
                    return Err(LinkError::MissingBinding {
                        module: target_module.clone(),
                        name: name.clone(),
                        range: surface.range,
                    });
                }
                (
                    crate::interface::LinkedExportTarget::Binding(SymbolId {
                        module: target_module.clone(),
                        name: name.clone().into_boxed_str(),
                    }),
                    surface.range,
                )
            }
        };
        self.resolving_exports.remove(&key);
        let linked = LinkedExport {
            public_name: name.to_owned().into_boxed_str(),
            target,
            range: export_range,
        };
        self.linked_exports.insert(key, linked.clone());
        Ok(linked)
    }

    fn linked_interface(&mut self, module: &ModuleId) -> Result<LinkedModuleInterface, LinkError> {
        let unlinked = self
            .linker
            .interfaces
            .get(module)
            .ok_or_else(|| LinkError::MissingModule { module: module.clone() })?;
        let mut exports = BTreeMap::new();
        for (name, surface) in &unlinked.exports {
            let linked = self.resolve_export(module, name, surface.range, module)?;
            exports.insert(name.clone().into_boxed_str(), linked);
        }
        Ok(LinkedModuleInterface {
            module: module.clone(),
            kind: unlinked.kind,
            exports: exports.into(),
            metadata: unlinked.metadata.clone(),
        })
    }
}

/// Classifies a source reference into the strongest dependency phase needed by
/// its current v1 runtime representation.
pub fn dependency_phase(kind: ReferenceKind) -> DependencyPhase {
    match kind {
        ReferenceKind::InterfaceOnly => DependencyPhase::InterfaceOnly,
        ReferenceKind::WholeModuleImport | ReferenceKind::SelectiveImport | ReferenceKind::ReExport => DependencyPhase::Runtime,
    }
}

/// Helper for constructing a project-relative path resolution table.
pub fn resolution_key(module: &ModuleId, path: &ImportPath) -> (ModuleId, String) {
    (module.clone(), path.to_string())
}

/// Returns a human-readable module path for diagnostics that do not have a
/// source provider available.
pub fn module_path(module: &ModuleId) -> String {
    ModulePath::from_components(module.path.components().to_vec()).to_string()
}

fn default_module_binding(path: &ImportPath) -> Option<&str> {
    path.segments.last().map(|segment| segment.name.as_str()).or(match &path.root {
        phalcom_ast::ast::ImportRoot::Absolute(segment) => Some(segment.name.as_str()),
        phalcom_ast::ast::ImportRoot::Relative { .. } => None,
    })
}
