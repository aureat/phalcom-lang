//! Normalized compiler product and diagnostic parity verification (R.T5 / R.G5).

use crate::modules::compile::{CompiledProgram, ProgramCompileError, ProgramSemanticDiagnostics};
use crate::modules::semantic_lowering::{ForIterationLoweringSpec, ForIterationRequirementSpec};
use phalcom_common::range::SourceRange;
use phalcom_modules::{DeclarationId, ModuleId};
use phalcom_semantic::identity::ImplId;
use std::collections::{BTreeMap, BTreeSet};

/// Normalizes module identity across synthetic project boundaries so that
/// sequential project IDs in inline/eval compilations compare identically.
pub fn normalize_module_id(id: &ModuleId) -> ModuleId {
    match &id.project {
        phalcom_modules::identity::ProjectIdentity::Synthetic(_) => {
            ModuleId::synthetic(phalcom_modules::identity::SyntheticProjectId::from_raw(0), id.path.clone())
        }
        _ => id.clone(),
    }
}

pub fn normalize_semantic_node_id(node: &phalcom_modules::graph::SemanticNodeId) -> phalcom_modules::graph::SemanticNodeId {
    match node {
        phalcom_modules::graph::SemanticNodeId::Module(m) => phalcom_modules::graph::SemanticNodeId::Module(normalize_module_id(m)),
        phalcom_modules::graph::SemanticNodeId::Declaration { module, name } => {
            phalcom_modules::graph::SemanticNodeId::Declaration {
                module: normalize_module_id(module),
                name: name.clone(),
            }
        }
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

    // 1. Entry logical module identity
    if normalize_module_id(&src.entry) != normalize_module_id(&art.entry) {
        report.differences.push(format!("Entry module mismatch: src={:?}, art={:?}", src.entry, art.entry));
    }

    // 2. Initialization order
    let src_init_order: Vec<_> = src.initialization_order.iter().map(normalize_module_id).collect();
    let art_init_order: Vec<_> = art.initialization_order.iter().map(normalize_module_id).collect();
    if src_init_order != art_init_order {
        report.differences.push(format!(
            "Initialization order mismatch: src={:?}, art={:?}",
            src_init_order, art_init_order
        ));
    }

    // 3. Compiled module logical-ID set
    let src_modules: BTreeMap<ModuleId, &crate::modules::compile::CompiledModule> = src.modules.iter().map(|(k, v)| (normalize_module_id(k), v)).collect();
    let art_modules: BTreeMap<ModuleId, &crate::modules::compile::CompiledModule> = art.modules.iter().map(|(k, v)| (normalize_module_id(k), v)).collect();
    let src_module_ids: BTreeSet<_> = src_modules.keys().cloned().collect();
    let art_module_ids: BTreeSet<_> = art_modules.keys().cloned().collect();
    if src_module_ids != art_module_ids {
        report.differences.push(format!(
            "Module set mismatch: src={:?}, art={:?}",
            src_module_ids, art_module_ids
        ));
    }

    // 4. Linked module topology (reference, semantic, runtime graphs)
    if normalize_module_id(&src.linked.entry) != normalize_module_id(&art.linked.entry) {
        report.differences.push(format!("Linked entry mismatch: src={:?}, art={:?}", src.linked.entry, art.linked.entry));
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
        report.differences.push(format!("Reference graph nodes mismatch: src={:?}, art={:?}", src_ref_nodes, art_ref_nodes));
    } else {
        let src_ref_map: BTreeMap<ModuleId, BTreeSet<(ModuleId, phalcom_modules::graph::ReferenceKind)>> = src.linked.graphs.references.nodes().into_iter().map(|n| {
            let edges = src.linked.graphs.references.edges_from(&n).iter().map(|e| (normalize_module_id(&e.to), e.kind)).collect();
            (normalize_module_id(&n), edges)
        }).collect();
        let art_ref_map: BTreeMap<ModuleId, BTreeSet<(ModuleId, phalcom_modules::graph::ReferenceKind)>> = art.linked.graphs.references.nodes().into_iter().map(|n| {
            let edges = art.linked.graphs.references.edges_from(&n).iter().map(|e| (normalize_module_id(&e.to), e.kind)).collect();
            (normalize_module_id(&n), edges)
        }).collect();
        if src_ref_map != art_ref_map {
            report.differences.push(format!("Reference edges mismatch: src={:?}, art={:?}", src_ref_map, art_ref_map));
        }
    }

    // Semantic graph edges
    let src_sem_nodes: BTreeSet<_> = src.linked.graphs.semantics.nodes().into_iter().map(|n| normalize_semantic_node_id(&n)).collect();
    let art_sem_nodes: BTreeSet<_> = art.linked.graphs.semantics.nodes().into_iter().map(|n| normalize_semantic_node_id(&n)).collect();
    if src_sem_nodes != art_sem_nodes {
        report.differences.push(format!("Semantic graph nodes mismatch: src={:?}, art={:?}", src_sem_nodes, art_sem_nodes));
    } else {
        let src_sem_map: BTreeMap<phalcom_modules::graph::SemanticNodeId, BTreeSet<(phalcom_modules::graph::SemanticNodeId, phalcom_modules::graph::SemanticEdgeKind)>> = src.linked.graphs.semantics.nodes().into_iter().map(|n| {
            let edges = src.linked.graphs.semantics.edges_from(&n).iter().map(|e| (normalize_semantic_node_id(&e.to), e.kind)).collect();
            (normalize_semantic_node_id(&n), edges)
        }).collect();
        let art_sem_map: BTreeMap<phalcom_modules::graph::SemanticNodeId, BTreeSet<(phalcom_modules::graph::SemanticNodeId, phalcom_modules::graph::SemanticEdgeKind)>> = art.linked.graphs.semantics.nodes().into_iter().map(|n| {
            let edges = art.linked.graphs.semantics.edges_from(&n).iter().map(|e| (normalize_semantic_node_id(&e.to), e.kind)).collect();
            (normalize_semantic_node_id(&n), edges)
        }).collect();
        if src_sem_map != art_sem_map {
            report.differences.push(format!("Semantic edges mismatch: src={:?}, art={:?}", src_sem_map, art_sem_map));
        }
    }

    // Runtime dependency graph edges
    let src_rt_nodes: BTreeSet<_> = src.linked.graphs.runtime.nodes().into_iter().map(|n| normalize_module_id(&n)).collect();
    let art_rt_nodes: BTreeSet<_> = art.linked.graphs.runtime.nodes().into_iter().map(|n| normalize_module_id(&n)).collect();
    if src_rt_nodes != art_rt_nodes {
        report.differences.push(format!("Runtime graph nodes mismatch: src={:?}, art={:?}", src_rt_nodes, art_rt_nodes));
    } else {
        let src_rt_map: BTreeMap<ModuleId, BTreeSet<(ModuleId, phalcom_modules::graph::RuntimeDependencyReason)>> = src.linked.graphs.runtime.nodes().into_iter().map(|n| {
            let edges = src.linked.graphs.runtime.edges_from(&n).iter().map(|e| (normalize_module_id(&e.dependency), e.reason)).collect();
            (normalize_module_id(&n), edges)
        }).collect();
        let art_rt_map: BTreeMap<ModuleId, BTreeSet<(ModuleId, phalcom_modules::graph::RuntimeDependencyReason)>> = art.linked.graphs.runtime.nodes().into_iter().map(|n| {
            let edges = art.linked.graphs.runtime.edges_from(&n).iter().map(|e| (normalize_module_id(&e.dependency), e.reason)).collect();
            (normalize_module_id(&n), edges)
        }).collect();
        if src_rt_map != art_rt_map {
            report.differences.push(format!("Runtime edges mismatch: src={:?}, art={:?}", src_rt_map, art_rt_map));
        }
    }

    let src_linked_modules: BTreeMap<ModuleId, _> = src.linked.modules.iter().map(|(k, v)| (normalize_module_id(k), v)).collect();
    let art_linked_modules: BTreeMap<ModuleId, _> = art.linked.modules.iter().map(|(k, v)| (normalize_module_id(k), v)).collect();

    // 5. Per-module comparison
    for id in src_module_ids.intersection(&art_module_ids) {
        let src_mod = src_modules[id];
        let art_mod = art_modules[id];

        // Module kinds
        if src_mod.kind != art_mod.kind {
            report.differences.push(format!("Module kind mismatch for {:?}: src={:?}, art={:?}", id, src_mod.kind, art_mod.kind));
        }

        // Public exports and fingerprints
        let src_exports: BTreeMap<_, _> = src_mod.interface.exports.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let art_exports: BTreeMap<_, _> = art_mod.interface.exports.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        if src_exports != art_exports {
            report.differences.push(format!("Public exports mismatch for {:?}: src={:?}, art={:?}", id, src_exports, art_exports));
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
        if let (Some(src_lm), Some(art_lm)) = (src_linked_modules.get(id), art_linked_modules.get(id)) {
            if src_lm.bindings != art_lm.bindings {
                report.differences.push(format!("Binding layout mismatch for {:?}: src={:?}, art={:?}", id, src_lm.bindings, art_lm.bindings));
            }
            if src_lm.linked_reads != art_lm.linked_reads {
                report.differences.push(format!("Linked reads mismatch for {:?}: src={:?}, art={:?}", id, src_lm.linked_reads, art_lm.linked_reads));
            }
            if src_lm.runtime_dependencies != art_lm.runtime_dependencies {
                report.differences.push(format!("Runtime dependencies mismatch for {:?}: src={:?}, art={:?}", id, src_lm.runtime_dependencies, art_lm.runtime_dependencies));
            }
        }

        // Materialization plan (declarations and linked reads)
        if src_mod.plan.declarations != art_mod.plan.declarations {
            report.differences.push(format!("Materialization declarations mismatch for {:?}: src={:?}, art={:?}", id, src_mod.plan.declarations, art_mod.plan.declarations));
        }
        if src_mod.plan.linked_reads != art_mod.plan.linked_reads {
            report.differences.push(format!("Materialization linked reads mismatch for {:?}: src={:?}, art={:?}", id, src_mod.plan.linked_reads, art_mod.plan.linked_reads));
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
            report.differences.push(format!("Lowering constructor super initializers mismatch for {:?}", id));
        }
        if src_mod.lowering.conditional_invocations != art_mod.lowering.conditional_invocations {
            report.differences.push(format!("Lowering conditional invocations mismatch for {:?}", id));
        }
        if src_mod.lowering.trait_requirement_invocations != art_mod.lowering.trait_requirement_invocations {
            report.differences.push(format!("Lowering trait requirement invocations mismatch for {:?}", id));
        }
        if src_mod.lowering.associated != art_mod.lowering.associated {
            report.differences.push(format!("Lowering associated mismatch for {:?}", id));
        }
        if src_mod.lowering.bindings != art_mod.lowering.bindings {
            report.differences.push(format!("Lowering bindings mismatch for {:?}", id));
        }
        if src_mod.lowering.binding_read_safeties != art_mod.lowering.binding_read_safeties {
            report.differences.push(format!("Lowering binding read safeties mismatch for {:?}", id));
        }

        // Local constructor initializations
        let src_ci: BTreeMap<_, _> = src_mod.lowering.constructor_initializations.iter_local().map(|(k, v)| (k.clone(), v.clone())).collect();
        let art_ci: BTreeMap<_, _> = art_mod.lowering.constructor_initializations.iter_local().map(|(k, v)| (k.clone(), v.clone())).collect();
        if src_ci != art_ci {
            report.differences.push(format!("Lowering constructor initializations mismatch for {:?}", id));
        }

        // Normalized for iteration lanes (excluding process-local TypeId)
        let src_lanes: BTreeMap<_, _> = src_mod.lowering.for_iteration_lanes.iter().map(|(k, v)| (*k, normalize_for_iteration_spec(v))).collect();
        let art_lanes: BTreeMap<_, _> = art_mod.lowering.for_iteration_lanes.iter().map(|(k, v)| (*k, normalize_for_iteration_spec(v))).collect();
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
pub fn assert_semantic_diagnostics_parity(
    src: &ProgramSemanticDiagnostics,
    art: &ProgramSemanticDiagnostics,
) {
    let src_modules: BTreeMap<ModuleId, _> = src
        .by_module
        .iter()
        .map(|(k, v)| (normalize_module_id(k), v))
        .collect();
    let art_modules: BTreeMap<ModuleId, _> = art
        .by_module
        .iter()
        .map(|(k, v)| (normalize_module_id(k), v))
        .collect();
    let src_keys: BTreeSet<_> = src_modules.keys().cloned().collect();
    let art_keys: BTreeSet<_> = art_modules.keys().cloned().collect();
    assert_eq!(src_keys, art_keys, "Diagnostic module sets must match");

    for module in &src_keys {
        let src_diags = src_modules[module];
        let art_diags = art_modules[module];
        assert_eq!(
            src_diags.len(),
            art_diags.len(),
            "Diagnostic count mismatch for module {:?}",
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
