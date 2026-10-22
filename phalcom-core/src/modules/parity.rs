//! Normalized compiler product and diagnostic parity verification (R.T5 / R.G5).

use crate::modules::compile::{CompiledProgram, ProgramCompileError, ProgramSemanticDiagnostics};
use crate::modules::semantic_lowering::{
    AssociatedLoweringSpec, DataConstructionLoweringSpec, ExecutableFamilyCandidate, ExecutableFamilyDescriptor, ExecutableFamilyTarget,
    ExecutableInvocationTarget, ForIterationLoweringSpec, ForIterationRequirementSpec,
};
use crate::typing::{LoadedSemanticMetadata, MetadataPoolId, RuntimeTypeRecipe, RuntimeTypeRef, RuntimeTypingRegistry, TypingContextData};
use phalcom_common::range::SourceRange;
use phalcom_modules::{DeclarationId, ModuleId};
use phalcom_semantic::identity::ImplId;
use phalcom_semantic::signature::{ConstructorInitializationProduct, ConstructorInitializationTable};
use std::collections::{BTreeMap, BTreeSet};

fn format_set_difference<T: std::fmt::Debug + Ord>(src: &BTreeSet<T>, art: &BTreeSet<T>) -> String {
    format!(
        "source={}, artifact={}, common={}, source_only={:?}, artifact_only={:?}",
        src.len(),
        art.len(),
        src.intersection(art).count(),
        src.difference(art).collect::<Vec<_>>(),
        art.difference(src).collect::<Vec<_>>(),
    )
}

struct RuntimeTypeParity {
    registry: RuntimeTypingRegistry,
    context: TypingContextData,
    source_pool: MetadataPoolId,
    artifact_pool: MetadataPoolId,
}

impl RuntimeTypeParity {
    fn new(source: &CompiledProgram, artifact: &CompiledProgram) -> Option<Self> {
        let source_metadata = source.semantic_metadata.as_ref()?.clone();
        let artifact_metadata = artifact.semantic_metadata.as_ref()?.clone();
        let mut registry = RuntimeTypingRegistry::new();
        let source_pool = registry.register_pool(LoadedSemanticMetadata::new(MetadataPoolId(0), source_metadata));
        let artifact_pool = registry.register_pool(LoadedSemanticMetadata::new(MetadataPoolId(1), artifact_metadata));
        let context = TypingContextData::new(Box::new([source_pool, artifact_pool]));
        Some(Self {
            registry,
            context,
            source_pool,
            artifact_pool,
        })
    }

    fn remap_program_local(&self, ty: RuntimeTypeRef, pool: MetadataPoolId) -> Option<RuntimeTypeRef> {
        match ty {
            RuntimeTypeRef::Base {
                pool: MetadataPoolId(0),
                node,
            } => Some(RuntimeTypeRef::Base { pool, node }),
            // Compiled lowering recipes are persisted against the program's
            // metadata bundle and therefore must not contain VM-local overlay
            // handles or references to any pre-existing runtime pool.
            RuntimeTypeRef::Base { .. } | RuntimeTypeRef::Overlay(_) => None,
        }
    }

    fn type_ref(&self, source: RuntimeTypeRef, artifact: RuntimeTypeRef) -> bool {
        let Some(source) = self.remap_program_local(source, self.source_pool) else {
            return false;
        };
        let Some(artifact) = self.remap_program_local(artifact, self.artifact_pool) else {
            return false;
        };
        crate::typing::identity::equivalent(&self.context, &self.registry, source, artifact)
    }

    fn recipe(&self, source: RuntimeTypeRecipe, artifact: RuntimeTypeRecipe) -> bool {
        match (source, artifact) {
            (RuntimeTypeRecipe::Closed(source), RuntimeTypeRecipe::Closed(artifact))
            | (RuntimeTypeRecipe::Template(source), RuntimeTypeRecipe::Template(artifact)) => self.type_ref(source, artifact),
            _ => false,
        }
    }
}

fn data_construction_equivalent(
    source: &DataConstructionLoweringSpec,
    artifact: &DataConstructionLoweringSpec,
    types: Option<&RuntimeTypeParity>,
) -> bool {
    source.constructor == artifact.constructor
        && source.exact_type == artifact.exact_type
        && source.layout == artifact.layout
        && source.argument_to_component == artifact.argument_to_component
        && source.component_to_logical == artifact.component_to_logical
        && match types {
            Some(types) => types.recipe(source.type_recipe, artifact.type_recipe),
            None => source.type_recipe == artifact.type_recipe,
        }
}

fn invocation_target_equivalent(
    source: &ExecutableInvocationTarget,
    artifact: &ExecutableInvocationTarget,
    types: Option<&RuntimeTypeParity>,
) -> bool {
    match (source, artifact) {
        (
            ExecutableInvocationTarget::DataConstructor {
                constructor: source_constructor,
                construction: source_construction,
            },
            ExecutableInvocationTarget::DataConstructor {
                constructor: artifact_constructor,
                construction: artifact_construction,
            },
        ) => {
            source_constructor == artifact_constructor
                && data_construction_equivalent(source_construction, artifact_construction, types)
        }
        _ => source == artifact,
    }
}

fn family_target_equivalent(
    source: &ExecutableFamilyTarget,
    artifact: &ExecutableFamilyTarget,
    types: Option<&RuntimeTypeParity>,
) -> bool {
    match (source, artifact) {
        (
            ExecutableFamilyTarget::DataConstructor {
                constructor: source_constructor,
                construction: source_construction,
            },
            ExecutableFamilyTarget::DataConstructor {
                constructor: artifact_constructor,
                construction: artifact_construction,
            },
        ) => {
            source_constructor == artifact_constructor
                && data_construction_equivalent(source_construction, artifact_construction, types)
        }
        (
            ExecutableFamilyTarget::Behavioral { target: source },
            ExecutableFamilyTarget::Behavioral { target: artifact },
        ) => invocation_target_equivalent(source, artifact, types),
        _ => source == artifact,
    }
}

fn family_descriptor_equivalent(
    source: &ExecutableFamilyDescriptor,
    artifact: &ExecutableFamilyDescriptor,
    types: Option<&RuntimeTypeParity>,
) -> bool {
    source.entries.len() == artifact.entries.len()
        && source.entries.iter().zip(artifact.entries.iter()).all(|(source, artifact)| {
            source.operation == artifact.operation
                && source.member_kind == artifact.member_kind
                && family_target_equivalent(&source.target, &artifact.target, types)
        })
}

fn family_candidate_equivalent(
    source: &ExecutableFamilyCandidate,
    artifact: &ExecutableFamilyCandidate,
    types: Option<&RuntimeTypeParity>,
) -> bool {
    source.operation == artifact.operation
        && match (&source.target, &artifact.target) {
            (Some(source), Some(artifact)) => invocation_target_equivalent(source, artifact, types),
            (None, None) => true,
            _ => false,
        }
}

fn associated_lowering_equivalent(
    source: &AssociatedLoweringSpec,
    artifact: &AssociatedLoweringSpec,
    types: Option<&RuntimeTypeParity>,
) -> bool {
    match (source, artifact) {
        (
            AssociatedLoweringSpec::ConstructData {
                constructor: source_constructor,
                arity: source_arity,
                construction: source_construction,
            },
            AssociatedLoweringSpec::ConstructData {
                constructor: artifact_constructor,
                arity: artifact_arity,
                construction: artifact_construction,
            },
        ) => {
            source_constructor == artifact_constructor
                && source_arity == artifact_arity
                && data_construction_equivalent(source_construction, artifact_construction, types)
        }
        (
            AssociatedLoweringSpec::InvokeResolvedAssociated {
                target: source_target,
                arity: source_arity,
            },
            AssociatedLoweringSpec::InvokeResolvedAssociated {
                target: artifact_target,
                arity: artifact_arity,
            },
        ) => source_arity == artifact_arity && invocation_target_equivalent(source_target, artifact_target, types),
        (
            AssociatedLoweringSpec::MakeResolvedBoundMethod { target: source },
            AssociatedLoweringSpec::MakeResolvedBoundMethod { target: artifact },
        ) => invocation_target_equivalent(source, artifact, types),
        (
            AssociatedLoweringSpec::MakeDataConstructorThunk {
                constructor: source_constructor,
                operation: source_operation,
                construction: source_construction,
            },
            AssociatedLoweringSpec::MakeDataConstructorThunk {
                constructor: artifact_constructor,
                operation: artifact_operation,
                construction: artifact_construction,
            },
        ) => {
            source_constructor == artifact_constructor
                && source_operation == artifact_operation
                && data_construction_equivalent(source_construction, artifact_construction, types)
        }
        (
            AssociatedLoweringSpec::MakeAssociatedFamily { descriptor: source },
            AssociatedLoweringSpec::MakeAssociatedFamily { descriptor: artifact },
        ) => family_descriptor_equivalent(source, artifact, types),
        (
            AssociatedLoweringSpec::DynamicInvoke { candidates: source },
            AssociatedLoweringSpec::DynamicInvoke { candidates: artifact },
        ) => {
            source.len() == artifact.len()
                && source
                    .iter()
                    .zip(artifact.iter())
                    .all(|(source, artifact)| family_candidate_equivalent(source, artifact, types))
        }
        _ => source == artifact,
    }
}

#[derive(Default)]
struct ConstructorProductPartitions {
    user: BTreeMap<phalcom_semantic::identity::CallableId, ConstructorInitializationProduct>,
    canonical: BTreeMap<phalcom_semantic::identity::CallableId, ConstructorInitializationProduct>,
    external: BTreeMap<phalcom_semantic::identity::CallableId, ConstructorInitializationProduct>,
}

fn partition_constructor_products(table: &ConstructorInitializationTable, user_modules: &BTreeSet<ModuleId>) -> ConstructorProductPartitions {
    let mut partitions = ConstructorProductPartitions::default();
    for (callable, product) in table.iter() {
        let owner_module = callable.owner.module();
        let normalized_owner_module = normalize_module_id(owner_module);
        let target = if user_modules.contains(&normalized_owner_module) {
            &mut partitions.user
        } else if owner_module.project.is_universe() {
            &mut partitions.canonical
        } else {
            &mut partitions.external
        };
        target.insert(callable.clone(), product.clone());
    }
    partitions
}

fn differing_constructor_fields(src: &ConstructorInitializationProduct, art: &ConstructorInitializationProduct) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if src.factory != art.factory {
        fields.push("factory");
    }
    if src.initializer != art.initializer {
        fields.push("initializer");
    }
    if src.origin != art.origin {
        fields.push("origin");
    }
    if src.field_input_fingerprint != art.field_input_fingerprint {
        fields.push("field_input_fingerprint");
    }
    if src.default_fields != art.default_fields {
        fields.push("default_fields");
    }
    if src.parameter_fields != art.parameter_fields {
        fields.push("parameter_fields");
    }
    fields
}

fn constructor_partition_summary(
    label: &str,
    src: &BTreeMap<phalcom_semantic::identity::CallableId, ConstructorInitializationProduct>,
    art: &BTreeMap<phalcom_semantic::identity::CallableId, ConstructorInitializationProduct>,
) -> String {
    let src_keys: BTreeSet<_> = src.keys().cloned().collect();
    let art_keys: BTreeSet<_> = art.keys().cloned().collect();
    let differing_common = src_keys
        .intersection(&art_keys)
        .filter_map(|key| {
            let fields = differing_constructor_fields(&src[key], &art[key]);
            (!fields.is_empty()).then_some((key, fields))
        })
        .collect::<Vec<_>>();
    format!(
        "{label}: source={}, artifact={}, source_only={:?}, artifact_only={:?}, differing_common={:?}",
        src.len(),
        art.len(),
        src_keys.difference(&art_keys).collect::<Vec<_>>(),
        art_keys.difference(&src_keys).collect::<Vec<_>>(),
        differing_common,
    )
}

fn constructor_table_difference(
    module: &ModuleId,
    src: &ConstructorInitializationTable,
    art: &ConstructorInitializationTable,
    user_modules: &BTreeSet<ModuleId>,
    referenced_initializers: &BTreeSet<phalcom_semantic::identity::CallableId>,
) -> Option<String> {
    let src_partitions = partition_constructor_products(src, user_modules);
    let art_partitions = partition_constructor_products(art, user_modules);
    let source_required = |products: &BTreeMap<phalcom_semantic::identity::CallableId, ConstructorInitializationProduct>| {
        products
            .iter()
            .filter(|(_, product)| referenced_initializers.contains(&product.initializer))
            .map(|(key, product)| (key.clone(), product.clone()))
            .collect::<BTreeMap<_, _>>()
    };
    let src_canonical = source_required(&src_partitions.canonical);
    let art_canonical = source_required(&art_partitions.canonical);
    let src_external = source_required(&src_partitions.external);
    let art_external = source_required(&art_partitions.external);
    let referenced_products = |table: &ConstructorInitializationTable| {
        table
            .iter()
            .filter(|(_, product)| referenced_initializers.contains(&product.initializer))
            .map(|(key, product)| (key.clone(), product.clone()))
            .collect::<BTreeMap<_, _>>()
    };
    let source_products = referenced_products(src);
    let artifact_products = referenced_products(art);
    let source_missing = referenced_initializers
        .iter()
        .filter(|initializer| !source_products.values().any(|product| &product.initializer == *initializer))
        .cloned()
        .collect::<Vec<_>>();
    let artifact_missing = referenced_initializers
        .iter()
        .filter(|initializer| !artifact_products.values().any(|product| &product.initializer == *initializer))
        .cloned()
        .collect::<Vec<_>>();

    if src_partitions.user == art_partitions.user
        && src_canonical == art_canonical
        && src_external == art_external
        && source_missing.is_empty()
        && artifact_missing.is_empty()
    {
        return None;
    }

    Some(format!(
        "Constructor initialization parity mismatch for {:?}: source effective rows={} (local={}, parent={}), artifact effective rows={} (local={}, parent={}); {}; {}; {}; referenced initializers missing in source={:?}, artifact={:?}",
        module,
        src.len(),
        src.local_len(),
        src.parent_len(),
        art.len(),
        art.local_len(),
        art.parent_len(),
        constructor_partition_summary("user-owned", &src_partitions.user, &art_partitions.user),
        constructor_partition_summary("referenced canonical-owned", &src_canonical, &art_canonical),
        constructor_partition_summary("referenced other-external-owned", &src_external, &art_external),
        source_missing,
        artifact_missing,
    ))
}

/// Normalizes module identity across synthetic project boundaries so that
/// sequential project IDs in inline/eval compilations compare identically.
pub fn normalize_module_id(id: &ModuleId) -> ModuleId {
    match &id.project {
        phalcom_modules::identity::ProjectIdentity::Synthetic(_) => {
            static NORMALIZED_SYNTHETIC_PROJECT: std::sync::OnceLock<phalcom_modules::identity::SyntheticProjectId> = std::sync::OnceLock::new();
            let normalized_project = *NORMALIZED_SYNTHETIC_PROJECT.get_or_init(|| phalcom_modules::identity::SyntheticProjectIdAllocator.allocate());
            ModuleId::synthetic(normalized_project, id.path.clone())
        }
        _ => id.clone(),
    }
}

pub fn normalize_semantic_node_id(node: &phalcom_modules::graph::SemanticNodeId) -> phalcom_modules::graph::SemanticNodeId {
    match node {
        phalcom_modules::graph::SemanticNodeId::Module(m) => phalcom_modules::graph::SemanticNodeId::Module(normalize_module_id(m)),
        phalcom_modules::graph::SemanticNodeId::Declaration { module, name } => phalcom_modules::graph::SemanticNodeId::Declaration {
            module: normalize_module_id(module),
            name: name.clone(),
        },
    }
}

/// Detailed report of structural differences between two compiled programs.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ProgramParityReport {
    pub differences: Vec<String>,
}

impl ProgramParityReport {
    pub fn is_empty(&self) -> bool {
        self.differences.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NormalizedForIterationSpec {
    source: SourceRange,
    iterator: NormalizedRequirementSpec,
    next: NormalizedRequirementSpec,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NormalizedRequirementSpec {
    Selected {
        source_impl: ImplId,
        trait_declaration: DeclarationId,
        requirement_slot: u16,
    },
    Requirement {
        requirement: phalcom_semantic::traits::TraitRequirementId,
        requirement_slot: u16,
    },
}

fn normalize_requirement_spec(spec: &ForIterationRequirementSpec) -> NormalizedRequirementSpec {
    match spec {
        ForIterationRequirementSpec::Selected(s) => NormalizedRequirementSpec::Selected {
            source_impl: s.selection.source_impl.clone(),
            trait_declaration: s.selection.exact_trait_ref.declaration.clone(),
            requirement_slot: s.requirement_slot,
        },
        ForIterationRequirementSpec::Requirement(r) => NormalizedRequirementSpec::Requirement {
            requirement: r.requirement.clone(),
            requirement_slot: r.requirement_slot,
        },
    }
}

fn normalize_for_iteration_spec(spec: &ForIterationLoweringSpec) -> NormalizedForIterationSpec {
    NormalizedForIterationSpec {
        source: spec.source,
        iterator: normalize_requirement_spec(&spec.iterator),
        next: normalize_requirement_spec(&spec.next),
    }
}

/// Normalizes and compares stable compiler products between source-backed and
/// artifact-backed compilation plans without comparing process-local pointers or
/// raw store IDs.
pub fn compare_compiled_programs(src: &CompiledProgram, art: &CompiledProgram) -> ProgramParityReport {
    let mut report = ProgramParityReport::default();
    let runtime_type_parity = RuntimeTypeParity::new(src, art);

    // 1. Entry logical module identity
    if normalize_module_id(&src.entry) != normalize_module_id(&art.entry) {
        report
            .differences
            .push(format!("Entry module mismatch: src={:?}, art={:?}", src.entry, art.entry));
    }

    // 2. Initialization order
    let src_init_order: Vec<_> = src.initialization_order.iter().map(normalize_module_id).collect();
    let art_init_order: Vec<_> = art.initialization_order.iter().map(normalize_module_id).collect();
    if src_init_order != art_init_order {
        report
            .differences
            .push(format!("Initialization order mismatch: src={:?}, art={:?}", src_init_order, art_init_order));
    }

    // 3. Compiled module logical-ID set
    let src_modules: BTreeMap<ModuleId, &crate::modules::compile::CompiledModule> = src.modules.iter().map(|(k, v)| (normalize_module_id(k), v)).collect();
    let art_modules: BTreeMap<ModuleId, &crate::modules::compile::CompiledModule> = art.modules.iter().map(|(k, v)| (normalize_module_id(k), v)).collect();
    assert_eq!(src_modules.len(), src.modules.len(), "normalize_module_id collision in src");
    assert_eq!(art_modules.len(), art.modules.len(), "normalize_module_id collision in art");
    let src_module_ids: BTreeSet<_> = src_modules.keys().cloned().collect();
    let art_module_ids: BTreeSet<_> = art_modules.keys().cloned().collect();
    let user_module_ids = src_module_ids.union(&art_module_ids).cloned().collect::<BTreeSet<_>>();
    if src_module_ids != art_module_ids {
        report
            .differences
            .push(format!("Module set mismatch: src={:?}, art={:?}", src_module_ids, art_module_ids));
    }

    // 4. Linked module topology (reference, semantic, runtime graphs)
    if normalize_module_id(&src.linked.entry) != normalize_module_id(&art.linked.entry) {
        report
            .differences
            .push(format!("Linked entry mismatch: src={:?}, art={:?}", src.linked.entry, art.linked.entry));
    }
    let src_linked_order: Vec<_> = src.linked.initialization_order.iter().map(normalize_module_id).collect();
    let art_linked_order: Vec<_> = art.linked.initialization_order.iter().map(normalize_module_id).collect();
    if src_linked_order != art_linked_order {
        report.differences.push(format!(
            "Linked initialization order mismatch: src={:?}, art={:?}",
            src_linked_order, art_linked_order
        ));
    }

    // Reference graph edges
    let src_ref_nodes: BTreeSet<_> = src.linked.graphs.references.nodes().into_iter().map(|n| normalize_module_id(&n)).collect();
    let art_ref_nodes: BTreeSet<_> = art.linked.graphs.references.nodes().into_iter().map(|n| normalize_module_id(&n)).collect();
    if src_ref_nodes != art_ref_nodes {
        report.differences.push(format!(
            "Reference graph nodes mismatch: {}",
            format_set_difference(&src_ref_nodes, &art_ref_nodes)
        ));
    } else {
        let src_ref_map: BTreeMap<ModuleId, BTreeSet<(ModuleId, phalcom_modules::graph::ReferenceKind)>> = src
            .linked
            .graphs
            .references
            .nodes()
            .into_iter()
            .map(|n| {
                let edges = src
                    .linked
                    .graphs
                    .references
                    .edges_from(&n)
                    .iter()
                    .map(|e| (normalize_module_id(&e.to), e.kind))
                    .collect();
                (normalize_module_id(&n), edges)
            })
            .collect();
        let art_ref_map: BTreeMap<ModuleId, BTreeSet<(ModuleId, phalcom_modules::graph::ReferenceKind)>> = art
            .linked
            .graphs
            .references
            .nodes()
            .into_iter()
            .map(|n| {
                let edges = art
                    .linked
                    .graphs
                    .references
                    .edges_from(&n)
                    .iter()
                    .map(|e| (normalize_module_id(&e.to), e.kind))
                    .collect();
                (normalize_module_id(&n), edges)
            })
            .collect();
        if src_ref_map != art_ref_map {
            report
                .differences
                .push(format!("Reference edges mismatch: src={:?}, art={:?}", src_ref_map, art_ref_map));
        }
    }

    // Semantic graph edges
    let src_sem_nodes: BTreeSet<_> = src
        .linked
        .graphs
        .semantics
        .nodes()
        .into_iter()
        .map(|n| normalize_semantic_node_id(&n))
        .collect();
    let art_sem_nodes: BTreeSet<_> = art
        .linked
        .graphs
        .semantics
        .nodes()
        .into_iter()
        .map(|n| normalize_semantic_node_id(&n))
        .collect();
    if src_sem_nodes != art_sem_nodes {
        report.differences.push(format!(
            "Semantic graph nodes mismatch: {}",
            format_set_difference(&src_sem_nodes, &art_sem_nodes)
        ));
    } else {
        let src_sem_map: BTreeMap<
            phalcom_modules::graph::SemanticNodeId,
            BTreeSet<(phalcom_modules::graph::SemanticNodeId, phalcom_modules::graph::SemanticEdgeKind)>,
        > = src
            .linked
            .graphs
            .semantics
            .nodes()
            .into_iter()
            .map(|n| {
                let edges = src
                    .linked
                    .graphs
                    .semantics
                    .edges_from(&n)
                    .iter()
                    .map(|e| (normalize_semantic_node_id(&e.to), e.kind))
                    .collect();
                (normalize_semantic_node_id(&n), edges)
            })
            .collect();
        let art_sem_map: BTreeMap<
            phalcom_modules::graph::SemanticNodeId,
            BTreeSet<(phalcom_modules::graph::SemanticNodeId, phalcom_modules::graph::SemanticEdgeKind)>,
        > = art
            .linked
            .graphs
            .semantics
            .nodes()
            .into_iter()
            .map(|n| {
                let edges = art
                    .linked
                    .graphs
                    .semantics
                    .edges_from(&n)
                    .iter()
                    .map(|e| (normalize_semantic_node_id(&e.to), e.kind))
                    .collect();
                (normalize_semantic_node_id(&n), edges)
            })
            .collect();
        if src_sem_map != art_sem_map {
            report
                .differences
                .push(format!("Semantic edges mismatch: src={:?}, art={:?}", src_sem_map, art_sem_map));
        }
    }

    // Runtime dependency graph edges
    let src_rt_nodes: BTreeSet<_> = src.linked.graphs.runtime.nodes().into_iter().map(|n| normalize_module_id(&n)).collect();
    let art_rt_nodes: BTreeSet<_> = art.linked.graphs.runtime.nodes().into_iter().map(|n| normalize_module_id(&n)).collect();
    if src_rt_nodes != art_rt_nodes {
        report
            .differences
            .push(format!("Runtime graph nodes mismatch: {}", format_set_difference(&src_rt_nodes, &art_rt_nodes)));
    } else {
        let src_rt_map: BTreeMap<ModuleId, BTreeSet<(ModuleId, phalcom_modules::graph::RuntimeDependencyReason)>> = src
            .linked
            .graphs
            .runtime
            .nodes()
            .into_iter()
            .map(|n| {
                let edges = src
                    .linked
                    .graphs
                    .runtime
                    .edges_from(&n)
                    .iter()
                    .map(|e| (normalize_module_id(&e.dependency), e.reason))
                    .collect();
                (normalize_module_id(&n), edges)
            })
            .collect();
        let art_rt_map: BTreeMap<ModuleId, BTreeSet<(ModuleId, phalcom_modules::graph::RuntimeDependencyReason)>> = art
            .linked
            .graphs
            .runtime
            .nodes()
            .into_iter()
            .map(|n| {
                let edges = art
                    .linked
                    .graphs
                    .runtime
                    .edges_from(&n)
                    .iter()
                    .map(|e| (normalize_module_id(&e.dependency), e.reason))
                    .collect();
                (normalize_module_id(&n), edges)
            })
            .collect();
        if src_rt_map != art_rt_map {
            report
                .differences
                .push(format!("Runtime edges mismatch: src={:?}, art={:?}", src_rt_map, art_rt_map));
        }
    }

    let src_linked_modules: BTreeMap<ModuleId, _> = src.linked.modules.iter().map(|(k, v)| (normalize_module_id(k), v)).collect();
    let art_linked_modules: BTreeMap<ModuleId, _> = art.linked.modules.iter().map(|(k, v)| (normalize_module_id(k), v)).collect();
    assert_eq!(
        src_linked_modules.len(),
        src.linked.modules.len(),
        "normalize_module_id collision in src linked modules"
    );
    assert_eq!(
        art_linked_modules.len(),
        art.linked.modules.len(),
        "normalize_module_id collision in art linked modules"
    );

    // A1: Explicitly compare linked-module identity sets.
    let src_linked_ids: BTreeSet<_> = src_linked_modules.keys().cloned().collect();
    let art_linked_ids: BTreeSet<_> = art_linked_modules.keys().cloned().collect();
    if src_linked_ids != art_linked_ids {
        report.differences.push(format!(
            "Linked module set mismatch: {}",
            format_set_difference(&src_linked_ids, &art_linked_ids)
        ));
    }

    // 5. Per-module comparison
    for id in src_module_ids.intersection(&art_module_ids) {
        let src_mod = src_modules[id];
        let art_mod = art_modules[id];

        // Module kinds
        if src_mod.kind != art_mod.kind {
            report
                .differences
                .push(format!("Module kind mismatch for {:?}: src={:?}, art={:?}", id, src_mod.kind, art_mod.kind));
        }

        // Public exports and fingerprints
        let src_exports: BTreeMap<_, _> = src_mod.interface.exports.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let art_exports: BTreeMap<_, _> = art_mod.interface.exports.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        if src_exports != art_exports {
            report
                .differences
                .push(format!("Public exports mismatch for {:?}: src={:?}, art={:?}", id, src_exports, art_exports));
        }
        if src_mod.interface.fingerprint() != art_mod.interface.fingerprint() {
            report.differences.push(format!(
                "Interface fingerprint mismatch for {:?}: src={:?}, art={:?}",
                id,
                src_mod.interface.fingerprint(),
                art_mod.interface.fingerprint()
            ));
        }

        // Linked module binding layout and linked reads
        match (src_linked_modules.get(id), art_linked_modules.get(id)) {
            (Some(src_lm), Some(art_lm)) => {
                if src_lm.bindings != art_lm.bindings {
                    report.differences.push(format!(
                        "Binding layout mismatch for {:?}: src={:?}, art={:?}",
                        id, src_lm.bindings, art_lm.bindings
                    ));
                }
                if src_lm.linked_reads != art_lm.linked_reads {
                    report.differences.push(format!(
                        "Linked reads mismatch for {:?}: src={:?}, art={:?}",
                        id, src_lm.linked_reads, art_lm.linked_reads
                    ));
                }
                if src_lm.runtime_dependencies != art_lm.runtime_dependencies {
                    report.differences.push(format!(
                        "Runtime dependencies mismatch for {:?}: src={:?}, art={:?}",
                        id, src_lm.runtime_dependencies, art_lm.runtime_dependencies
                    ));
                }
            }
            (Some(_), None) => report.differences.push(format!("Module {:?} has linked entry in src but not art", id)),
            (None, Some(_)) => report.differences.push(format!("Module {:?} has linked entry in art but not src", id)),
            (None, None) => {} // both absent — acceptable for canonical modules
        }

        // Materialization plan (declarations and linked reads)
        if src_mod.plan.declarations != art_mod.plan.declarations {
            report.differences.push(format!(
                "Materialization declarations mismatch for {:?}: src={:?}, art={:?}",
                id, src_mod.plan.declarations, art_mod.plan.declarations
            ));
        }
        if src_mod.plan.linked_reads != art_mod.plan.linked_reads {
            report.differences.push(format!(
                "Materialization linked reads mismatch for {:?}: src={:?}, art={:?}",
                id, src_mod.plan.linked_reads, art_mod.plan.linked_reads
            ));
        }

        // Lowering semantics
        if src_mod.lowering.module != art_mod.lowering.module {
            report.differences.push(format!("Lowering module mismatch for {:?}", id));
        }
        if src_mod.lowering.static_constants != art_mod.lowering.static_constants {
            report.differences.push(format!("Lowering static constants mismatch for {:?}", id));
        }
        if src_mod.lowering.enums != art_mod.lowering.enums {
            report.differences.push(format!("Lowering enums mismatch for {:?}", id));
        }
        if src_mod.lowering.data_decls != art_mod.lowering.data_decls {
            report.differences.push(format!("Lowering data declarations mismatch for {:?}", id));
        }
        if src_mod.lowering.inherent_impls != art_mod.lowering.inherent_impls {
            report.differences.push(format!("Lowering inherent impls mismatch for {:?}", id));
        }
        if src_mod.lowering.sequence_literals != art_mod.lowering.sequence_literals {
            report.differences.push(format!("Lowering sequence literals mismatch for {:?}", id));
        }
        if src_mod.lowering.pack_constructions != art_mod.lowering.pack_constructions {
            report.differences.push(format!("Lowering pack constructions mismatch for {:?}", id));
        }
        if src_mod.lowering.matches != art_mod.lowering.matches {
            report.differences.push(format!("Lowering matches mismatch for {:?}", id));
        }
        if src_mod.lowering.pattern_sites != art_mod.lowering.pattern_sites {
            report.differences.push(format!("Lowering pattern sites mismatch for {:?}", id));
        }
        if src_mod.lowering.callable_references != art_mod.lowering.callable_references {
            report.differences.push(format!("Lowering callable references mismatch for {:?}", id));
        }
        if src_mod.lowering.detached_methods != art_mod.lowering.detached_methods {
            report.differences.push(format!("Lowering detached methods mismatch for {:?}", id));
        }
        if src_mod.lowering.constructor_super_initializers != art_mod.lowering.constructor_super_initializers {
            report
                .differences
                .push(format!("Lowering constructor super initializers mismatch for {:?}", id));
        }
        if src_mod.lowering.conditional_invocations != art_mod.lowering.conditional_invocations {
            report.differences.push(format!("Lowering conditional invocations mismatch for {:?}", id));
        }
        if src_mod.lowering.trait_requirement_invocations != art_mod.lowering.trait_requirement_invocations {
            report.differences.push(format!("Lowering trait requirement invocations mismatch for {:?}", id));
        }
        let source_associated_sites = src_mod.lowering.associated.keys().cloned().collect::<BTreeSet<_>>();
        let artifact_associated_sites = art_mod.lowering.associated.keys().cloned().collect::<BTreeSet<_>>();
        let changed_associated_sites = source_associated_sites
            .intersection(&artifact_associated_sites)
            .filter(|site| {
                !associated_lowering_equivalent(
                    &src_mod.lowering.associated[*site],
                    &art_mod.lowering.associated[*site],
                    runtime_type_parity.as_ref(),
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        if source_associated_sites != artifact_associated_sites || !changed_associated_sites.is_empty() {
            report.differences.push(format!(
                "Lowering associated mismatch for {:?}: sites={}, changed={:?}",
                id,
                format_set_difference(&source_associated_sites, &artifact_associated_sites),
                changed_associated_sites,
            ));
        }
        if src_mod.lowering.bindings != art_mod.lowering.bindings {
            report.differences.push(format!("Lowering bindings mismatch for {:?}", id));
        }
        if src_mod.lowering.binding_read_safeties != art_mod.lowering.binding_read_safeties {
            report.differences.push(format!("Lowering binding read safeties mismatch for {:?}", id));
        }

        // Compare semantic authority across immutable parent and mutable local
        // storage. Table placement is not constructor ownership.
        if let Some(difference) = constructor_table_difference(
            id,
            &src_mod.lowering.constructor_initializations,
            &art_mod.lowering.constructor_initializations,
            &user_module_ids,
            &src_mod
                .lowering
                .constructor_super_initializers
                .values()
                .chain(art_mod.lowering.constructor_super_initializers.values())
                .cloned()
                .collect(),
        ) {
            report.differences.push(difference);
        }

        // Normalized for iteration lanes (excluding process-local TypeId)
        let src_lanes: BTreeMap<_, _> = src_mod
            .lowering
            .for_iteration_lanes
            .iter()
            .map(|(k, v)| (*k, normalize_for_iteration_spec(v)))
            .collect();
        let art_lanes: BTreeMap<_, _> = art_mod
            .lowering
            .for_iteration_lanes
            .iter()
            .map(|(k, v)| (*k, normalize_for_iteration_spec(v)))
            .collect();
        if src_lanes != art_lanes {
            report.differences.push(format!("Lowering for iteration lanes mismatch for {:?}", id));
        }
    }

    report
}

/// Asserts that two compiled programs agree on all stable observable coordinates.
pub fn assert_compiled_program_parity(src: &CompiledProgram, art: &CompiledProgram) {
    let report = compare_compiled_programs(src, art);
    if !report.is_empty() {
        panic!(
            "Compiled program parity failed ({} differences):\n{}",
            report.differences.len(),
            report.differences.join("\n")
        );
    }
}

/// Asserts structured diagnostic parity between source-backed and artifact-backed compilation errors.
pub fn assert_semantic_diagnostics_parity(src: &ProgramSemanticDiagnostics, art: &ProgramSemanticDiagnostics) {
    let src_modules: BTreeMap<ModuleId, _> = src.by_module.iter().map(|(k, v)| (normalize_module_id(k), v)).collect();
    let art_modules: BTreeMap<ModuleId, _> = art.by_module.iter().map(|(k, v)| (normalize_module_id(k), v)).collect();
    let src_keys: BTreeSet<_> = src_modules.keys().cloned().collect();
    let art_keys: BTreeSet<_> = art_modules.keys().cloned().collect();
    assert_eq!(src_keys, art_keys, "Diagnostic module sets must match");

    for module in &src_keys {
        let src_diags = src_modules[module];
        let art_diags = art_modules[module];
        assert_eq!(
            src_diags.len(),
            art_diags.len(),
            "Diagnostic count mismatch for module {:?}; source diagnostics: {src_diags:#?}; artifact diagnostics: {art_diags:#?}",
            module
        );

        for (i, (sd, ad)) in src_diags.iter().zip(art_diags.iter()).enumerate() {
            assert_eq!(sd.code, ad.code, "Diagnostic [{i}] code mismatch for {:?}", module);
            assert_eq!(sd.severity, ad.severity, "Diagnostic [{i}] severity mismatch for {:?}", module);
            assert_eq!(sd.primary_range, ad.primary_range, "Diagnostic [{i}] primary_range mismatch for {:?}", module);
            assert_eq!(sd.message, ad.message, "Diagnostic [{i}] message mismatch for {:?}", module);
            assert_eq!(sd.labels.len(), ad.labels.len(), "Diagnostic [{i}] labels count mismatch for {:?}", module);
            for (j, (sl, al)) in sd.labels.iter().zip(ad.labels.iter()).enumerate() {
                assert_eq!(sl.span.range, al.span.range, "Diagnostic [{i}] label [{j}] range mismatch for {:?}", module);
                assert_eq!(sl.message, al.message, "Diagnostic [{i}] label [{j}] message mismatch for {:?}", module);
            }
        }
    }
}

/// Asserts compile error parity, verifying matching failure variants and structured diagnostics.
pub fn assert_compile_error_parity(src_err: &ProgramCompileError, art_err: &ProgramCompileError) {
    match (src_err, art_err) {
        (ProgramCompileError::Semantic(src_diags), ProgramCompileError::Semantic(art_diags)) => {
            assert_semantic_diagnostics_parity(src_diags, art_diags);
        }
        _ => {
            assert_eq!(
                std::mem::discriminant(src_err),
                std::mem::discriminant(art_err),
                "Compile error variant mismatch: src={:?}, art={:?}",
                src_err,
                art_err
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{constructor_table_difference, partition_constructor_products};
    use phalcom_common::selector::Selector;
    use phalcom_modules::ModuleId;
    use phalcom_modules::identity::{ModulePath, ResolvedProjectId};
    use phalcom_semantic::db::ProductFingerprint;
    use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
    use phalcom_semantic::signature::{ConstructorInitializationProduct, ConstructorInitializationTable, ConstructorOrigin};
    use std::collections::BTreeSet;
    use std::sync::Arc;

    fn product(module: &ModuleId, name: &str, fingerprint: u64) -> ConstructorInitializationProduct {
        let owner = DeclarationId::new(module.clone(), name.into());
        let factory = CallableId::new(owner.clone(), Selector::method("new", Vec::new()).unwrap(), DispatchSide::Class);
        let initializer = CallableId::new(owner, Selector::method("initialize", Vec::new()).unwrap(), DispatchSide::Instance);
        ConstructorInitializationProduct {
            factory,
            initializer,
            origin: ConstructorOrigin::Source,
            field_input_fingerprint: ProductFingerprint(fingerprint),
            default_fields: Vec::new().into_boxed_slice(),
            parameter_fields: Vec::new().into_boxed_slice(),
        }
    }

    #[test]
    fn constructor_parity_compares_effective_authority_by_exact_owner() {
        let canonical_module = ModuleId::universe(ModulePath::root());
        let user_module = ModuleId::resolved(ResolvedProjectId::from_raw(7), ModulePath::root());
        let user_modules = BTreeSet::from([user_module.clone()]);
        let canonical = product(&canonical_module, "Canonical", 11);
        let user = product(&user_module, "User", 22);
        let referenced_initializers = BTreeSet::from([canonical.initializer.clone()]);

        let mut source = ConstructorInitializationTable::default();
        source.insert(canonical.clone());
        source.insert(user.clone());

        let mut artifact_parent = ConstructorInitializationTable::default();
        artifact_parent.insert(canonical.clone());
        let mut artifact = ConstructorInitializationTable::with_parent(Arc::new(artifact_parent));
        artifact.insert(user.clone());

        assert_eq!(source.local_len(), 2);
        assert_eq!(source.parent_len(), 0);
        assert_eq!(artifact.local_len(), 1);
        assert_eq!(artifact.parent_len(), 1);
        assert!(constructor_table_difference(&user_module, &source, &artifact, &user_modules, &referenced_initializers).is_none());

        let source_partitions = partition_constructor_products(&source, &user_modules);
        let artifact_partitions = partition_constructor_products(&artifact, &user_modules);
        assert_eq!(source_partitions.user, artifact_partitions.user);
        assert_eq!(source_partitions.canonical, artifact_partitions.canonical);
        assert!(source_partitions.external.is_empty());
        assert!(artifact_partitions.external.is_empty());

        // Keep the same user product while proving absent canonical authority
        // remains visible as a parity failure.
        let missing_artifact = {
            let mut table = ConstructorInitializationTable::with_parent(Arc::new(ConstructorInitializationTable::default()));
            table.insert(user.clone());
            table
        };
        let missing = constructor_table_difference(&user_module, &source, &missing_artifact, &user_modules, &referenced_initializers)
            .expect("missing canonical authority must fail parity");
        assert!(missing.contains("referenced canonical-owned: source=1, artifact=0"), "{missing}");

        let mut changed_parent = ConstructorInitializationTable::default();
        changed_parent.insert(canonical);
        let mut changed_artifact = ConstructorInitializationTable::with_parent(Arc::new(changed_parent));
        changed_artifact.insert(product(&user_module, "User", 23));
        let changed = constructor_table_difference(&user_module, &source, &changed_artifact, &user_modules, &referenced_initializers)
            .expect("changed user product fingerprint must fail parity");
        assert!(changed.contains("field_input_fingerprint"), "{changed}");

        let missing_user_table = ConstructorInitializationTable::with_parent(Arc::new({
            let mut parent = ConstructorInitializationTable::default();
            parent.insert(product(&canonical_module, "Canonical", 11));
            parent
        }));
        let missing_user = constructor_table_difference(&user_module, &source, &missing_user_table, &user_modules, &referenced_initializers)
            .expect("missing user product must fail parity");
        assert!(missing_user.contains("user-owned: source=1, artifact=0"), "{missing_user}");

        let mut broader_parent = ConstructorInitializationTable::default();
        broader_parent.insert(product(&canonical_module, "Canonical", 11));
        broader_parent.insert(product(&canonical_module, "ImageOnly", 33));
        let mut broader_artifact = ConstructorInitializationTable::with_parent(Arc::new(broader_parent));
        broader_artifact.insert(user);
        assert!(
            constructor_table_difference(&user_module, &source, &broader_artifact, &user_modules, &referenced_initializers).is_none(),
            "unreferenced imported canonical authority may be a strict superset"
        );
    }
}
