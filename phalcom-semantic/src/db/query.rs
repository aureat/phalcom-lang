//! Semantic database high-level query execution and caching (Spec 04.5 / Wave 5).

use crate::checker::analysis::{CallableAnalysis, CallableAnalysisStatus};
use crate::checker::body::analyze_callable_body;
use crate::db::SemanticDb;
use crate::db::SemanticProduct;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::db::key::{InputFingerprint, ProductFingerprint, QueryKey};
use crate::db::state::{QueryOutcome, QueryState};
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::SemanticDiagnostic;
use crate::dispatch::SurfaceDispatchResolver;
use crate::hierarchy_product::HierarchyEdgeProduct;
use crate::identity::{CallableId, DeclarationId, ModuleId};
use crate::module_product::ResolvedImportsProduct;
use crate::signature::CallableSemanticSignature;
use crate::source::ParsedModuleUnit;
use crate::surface::DeclarationSurface;
use crate::types::annotation::TypeResolver;
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::Statement;
use phalcom_common::range::SourceRange;
use phalcom_modules::interface::{InterfaceBuilder, LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::linker::LinkedProgram;
use std::sync::Arc;

/// Evaluates or retrieves the cached `ParsedModuleUnit` for a given module.
pub fn query_parsed_module(
    db: &mut SemanticDb,
    module: ModuleId,
    unit: Arc<ParsedModuleUnit>,
) -> QueryOutcome<Arc<ParsedModuleUnit>> {
    let key = QueryKey::ParsedModule(module);
    let input_fingerprint = crate::db::fingerprint::parsed_module_input_fingerprint(&unit.id, unit.kind, &unit.text);
    if db.is_reusable(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_parsed_module()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();
    let product_fingerprint = ProductFingerprint::new(input_fingerprint.raw());
    let rev = db.revision();
    let _ = db.publish_product_ready(
        key,
        rev,
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::ParsedModule(unit.clone()),
        Vec::new(),
    );
    QueryOutcome::Ready(unit)
}

/// Evaluates or retrieves the cached `UnlinkedModuleInterface` for a given module.
pub fn query_unlinked_interface(
    db: &mut SemanticDb,
    module: ModuleId,
    unit: Arc<ParsedModuleUnit>,
) -> QueryOutcome<Arc<UnlinkedModuleInterface>> {
    let key = QueryKey::UnlinkedInterface(module.clone());
    let input_fingerprint = crate::db::fingerprint::parsed_module_input_fingerprint(&unit.id, unit.kind, &unit.text);
    if db.is_reusable(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_unlinked_interface()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();
    let parsed_outcome = query_parsed_module(db, module.clone(), unit.clone());
    let _ = match parsed_outcome {
        QueryOutcome::Ready(p) => p,
        other => return match other {
            QueryOutcome::Cancelled => QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(r) => QueryOutcome::BudgetExceeded(r),
            QueryOutcome::Blocked(b) => QueryOutcome::Blocked(b),
            QueryOutcome::Failed(f) => QueryOutcome::Failed(f),
            _ => unreachable!(),
        },
    };

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    let _ = db.record_dependency(&mut recorder, QueryKey::ParsedModule(module.clone()));

    match InterfaceBuilder::build(module.clone(), unit.kind, &unit.program) {
        Ok(unlinked) => {
            let unlinked_arc = Arc::new(unlinked);
            let product_fingerprint = crate::db::fingerprint::unlinked_interface_product_fingerprint(&unlinked_arc);
            let rev = db.revision();
            let deps = recorder.finish();
            let _ = db.publish_product_ready(
                key,
                rev,
                input_fingerprint,
                product_fingerprint,
                SemanticProduct::UnlinkedInterface(unlinked_arc.clone()),
                deps,
            );
            QueryOutcome::Ready(unlinked_arc)
        }
        Err(err) => {
            let query_err = format!("failed to build unlinked interface: {err:?}");
            db.set_state(key, QueryState::Failed { revision: db.revision(), failure: query_err.clone() });
            QueryOutcome::Failed(query_err)
        }
    }
}

/// Evaluates or retrieves the cached `ResolvedImportsProduct` for a given module.
pub fn query_resolved_imports<P: phalcom_modules::source::SourceProvider>(
    db: &mut SemanticDb,
    module: ModuleId,
    unlinked: Arc<UnlinkedModuleInterface>,
    resolver: &mut phalcom_modules::resolver::ModuleResolver<'_, P>,
) -> QueryOutcome<Arc<ResolvedImportsProduct>> {
    let key = QueryKey::ResolvedImports(module.clone());
    let input_fingerprint = InputFingerprint::new(crate::db::fingerprint::unlinked_interface_product_fingerprint(&unlinked).raw());
    if db.is_reusable(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_resolved_imports()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    let _ = db.record_dependency(&mut recorder, QueryKey::UnlinkedInterface(module.clone()));

    let mut imports = std::collections::BTreeMap::new();
    let mut unresolved_diagnostics = Vec::new();

    for import_surface in &unlinked.imports {
        let import_path = match import_surface {
            phalcom_modules::interface::ImportSurface::Module(m) => &m.path,
            phalcom_modules::interface::ImportSurface::Selective(s) => &s.path,
            phalcom_modules::interface::ImportSurface::ReExport(r) => &r.path,
        };
        let path_str = import_path.to_string();
        match resolver.resolve_import_with_trace(&module, import_path) {
            Ok(trace) => {
                for pkg_mod in trace.package_interfaces {
                    let _ = db.record_dependency(&mut recorder, QueryKey::UnlinkedInterface(pkg_mod));
                }
                imports.insert(path_str, trace.target.id);
            }
            Err(err) => {
                unresolved_diagnostics.push((format!("unresolved import `{path_str}`: {err:?}"), import_path.range));
            }
        }
    }

    let product = Arc::new(ResolvedImportsProduct::new(module, imports, unresolved_diagnostics));
    let product_fingerprint = crate::db::fingerprint::resolved_imports_product_fingerprint(&product);
    let rev = db.revision();
    let deps = recorder.finish();
    let _ = db.publish_product_ready(
        key,
        rev,
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::ResolvedImports(product.clone()),
        deps,
    );
    QueryOutcome::Ready(product)
}

/// Evaluates or retrieves the cached `LinkedProgram` for a semantic component entry.
pub fn query_semantic_component(
    db: &mut SemanticDb,
    entry: ModuleId,
    universe: Arc<phalcom_modules::project::ProjectUniverse>,
    interfaces: std::collections::BTreeMap<ModuleId, UnlinkedModuleInterface>,
    resolved: &std::collections::BTreeMap<(ModuleId, String), ModuleId>,
) -> QueryOutcome<Arc<LinkedProgram>> {
    let key = QueryKey::SemanticComponent(entry.clone());
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&entry, &mut hasher);
    for (mod_id, iface) in &interfaces {
        std::hash::Hash::hash(mod_id, &mut hasher);
        std::hash::Hash::hash(&crate::db::fingerprint::unlinked_interface_product_fingerprint(iface).raw(), &mut hasher);
    }
    let input_fingerprint = InputFingerprint::new(std::hash::Hasher::finish(&hasher));
    if db.is_reusable(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_semantic_component()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    for mod_id in interfaces.keys() {
        let _ = db.record_dependency(&mut recorder, QueryKey::UnlinkedInterface(mod_id.clone()));
        let _ = db.record_dependency(&mut recorder, QueryKey::ResolvedImports(mod_id.clone()));
    }

    let linker = phalcom_modules::linker::ModuleLinker::new(universe, interfaces);
    match linker.link(entry, resolved) {
        Ok(linked_program) => {
            let linked_arc = Arc::new(linked_program);
            let rev = db.revision();
            let deps = recorder.finish();
            let prod_fp = ProductFingerprint::new(input_fingerprint.raw());
            let _ = db.publish_product_ready(
                key,
                rev,
                input_fingerprint,
                prod_fp,
                SemanticProduct::SemanticComponent(linked_arc.clone()),
                deps,
            );
            for (mod_id, linked_mod) in &linked_arc.modules {
                let mod_iface_arc = Arc::new(linked_mod.interface.clone());
                let mod_fp = crate::db::fingerprint::linked_interface_product_fingerprint(&mod_iface_arc);
                let _ = db.publish_product_ready(
                    QueryKey::LinkedInterface(mod_id.clone()),
                    rev,
                    InputFingerprint::new(mod_fp.raw()),
                    mod_fp,
                    SemanticProduct::LinkedInterface(mod_iface_arc),
                    Vec::new(),
                );
            }
            QueryOutcome::Ready(linked_arc)
        }
        Err(err) => {
            let query_err = format!("linker error: {err:?}");
            db.set_state(key, QueryState::Failed { revision: db.revision(), failure: query_err.clone() });
            QueryOutcome::Failed(query_err)
        }
    }
}

/// Evaluates or retrieves the cached `HierarchyEdgeProduct` for a declaration.
pub fn query_hierarchy_edge(
    db: &mut SemanticDb,
    class_decl: DeclarationId,
    super_decl: Option<DeclarationId>,
) -> QueryOutcome<Arc<HierarchyEdgeProduct>> {
    let key = QueryKey::HierarchyEdge(class_decl.clone());
    let prod_fp = crate::db::fingerprint::hierarchy_edge_product_fingerprint(&class_decl, &super_decl);
    let input_fingerprint = InputFingerprint::new(prod_fp.raw());
    if db.is_reusable(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_hierarchy_edge()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();

    let product = Arc::new(HierarchyEdgeProduct::new(class_decl, super_decl));
    let rev = db.revision();
    let _ = db.publish_product_ready(
        key,
        rev,
        input_fingerprint,
        prod_fp,
        SemanticProduct::HierarchyEdge(product.clone()),
        Vec::new(),
    );
    QueryOutcome::Ready(product)
}

/// Evaluates or retrieves the cached `DeclarationSurface` for a declaration.
pub fn query_declaration_surface(
    db: &mut SemanticDb,
    decl_id: DeclarationId,
    surface: Arc<DeclarationSurface>,
) -> QueryOutcome<Arc<DeclarationSurface>> {
    let key = QueryKey::DeclarationSurface(decl_id);
    let prod_fp = crate::db::fingerprint::declaration_surface_product_fingerprint(&surface);
    let input_fingerprint = InputFingerprint::new(prod_fp.raw());
    if db.is_reusable(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_declaration_surface()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();

    let rev = db.revision();
    let _ = db.publish_product_ready(
        key,
        rev,
        input_fingerprint,
        prod_fp,
        SemanticProduct::DeclarationSurface(surface.clone()),
        Vec::new(),
    );
    QueryOutcome::Ready(surface)
}

/// Evaluates or retrieves the cached `CallableSemanticSignature` for a callable.
pub fn query_callable_signature(
    db: &mut SemanticDb,
    callable: CallableId,
    signature: Arc<CallableSemanticSignature>,
) -> QueryOutcome<Arc<CallableSemanticSignature>> {
    let key = QueryKey::CallableSignature(callable);
    let prod_fp = crate::db::fingerprint::callable_signature_product_fingerprint(&signature);
    let input_fingerprint = InputFingerprint::new(prod_fp.raw());
    if db.is_reusable(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_callable_signature()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();

    let rev = db.revision();
    let _ = db.publish_product_ready(
        key,
        rev,
        input_fingerprint,
        prod_fp,
        SemanticProduct::CallableSignature(signature.clone()),
        Vec::new(),
    );
    QueryOutcome::Ready(signature)
}

/// Evaluates or retrieves the cached `LinkedModuleInterface` for a module.
pub fn query_linked_interface(
    db: &mut SemanticDb,
    module: ModuleId,
    linked_interface: Arc<LinkedModuleInterface>,
) -> QueryOutcome<Arc<LinkedModuleInterface>> {
    let key = QueryKey::LinkedInterface(module);
    let prod_fp = crate::db::fingerprint::linked_interface_product_fingerprint(&linked_interface);
    let input_fingerprint = InputFingerprint::new(prod_fp.raw());
    if db.is_reusable(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_linked_interface()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();

    let rev = db.revision();
    let _ = db.publish_product_ready(
        key,
        rev,
        input_fingerprint,
        prod_fp,
        SemanticProduct::LinkedInterface(linked_interface.clone()),
        Vec::new(),
    );
    QueryOutcome::Ready(linked_interface)
}

/// Evaluates or retrieves the cached `ModuleDiagnostics` for a module.
pub fn query_module_diagnostics(
    db: &mut SemanticDb,
    module: ModuleId,
    diagnostics: Arc<[SemanticDiagnostic]>,
) -> QueryOutcome<Arc<[SemanticDiagnostic]>> {
    let key = QueryKey::ModuleDiagnostics(module.clone());
    let prod_fp = crate::db::fingerprint::module_diagnostics_product_fingerprint(&module, &diagnostics);
    let input_fingerprint = InputFingerprint::new(prod_fp.raw());
    if db.is_reusable(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_module_diagnostics()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();

    let rev = db.revision();
    let _ = db.publish_product_ready(
        key,
        rev,
        input_fingerprint,
        prod_fp,
        SemanticProduct::ModuleDiagnostics(diagnostics.clone()),
        Vec::new(),
    );
    QueryOutcome::Ready(diagnostics)
}


/// Evaluates or retrieves the cached `CallableAnalysis` for a given callable body.
pub fn query_callable_body(
    db: &mut SemanticDb,
    callable: CallableId,
    body: &[Statement],
    body_range: SourceRange,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
    dispatch: &SurfaceDispatchResolver,
    module: ModuleId,
    budget: QueryBudget,
    cancel: &CancellationToken,
) -> QueryOutcome<Arc<CallableAnalysis>> {
    let key = QueryKey::CallableBody(callable.clone());

    let input_fingerprint = crate::db::fingerprint::callable_body_input_fingerprint(&callable, body, body_range, store);

    // 1. Check if already computed and ready for the same callable input and dependency products.
    if db.is_reusable(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_callable_body()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }

    // A ready product with a different input, or a non-ready state from an
    // earlier attempt, cannot remain in the dependency index while this
    // generation recomputes it. Invalidation also clears dependents that
    // consumed the old callable result.
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();

    // 2. Perform analysis
    let analysis = analyze_callable_body(callable, body, body_range, store, hierarchy, resolver, declarations, dispatch, module, budget, cancel);

    let mut analysis = analysis;
    let product_fingerprint = crate::db::fingerprint::callable_body_product_fingerprint(&analysis);
    analysis.dependency_fingerprint = product_fingerprint;
    let arc_analysis = Arc::new(analysis);

    match arc_analysis.status {
        CallableAnalysisStatus::Cancelled => {
            db.metrics().record_cancellation();
            db.set_state(key, QueryState::Cancelled { revision: db.revision() });
            QueryOutcome::Cancelled
        }
        CallableAnalysisStatus::BudgetExceeded => {
            let report = crate::db::budget::BudgetReport::new(crate::db::budget::BudgetKind::Steps, budget.max_steps, budget.max_steps);
            db.metrics().record_budget_exhaustion();
            db.set_state(
                key,
                QueryState::BudgetExceeded {
                    revision: db.revision(),
                    report: report.clone(),
                },
            );
            QueryOutcome::BudgetExceeded(report)
        }
        CallableAnalysisStatus::Blocked => {
            let reason = crate::types::outcome::BlockReason::SuppressedDependency;
            db.set_state(
                key,
                QueryState::Blocked {
                    revision: db.revision(),
                    reason: reason.clone(),
                },
            );
            QueryOutcome::Blocked(reason)
        }
        CallableAnalysisStatus::Complete | CallableAnalysisStatus::Partial => {
            let rev = db.revision();
            let mut recorder = crate::db::DependencyRecorder::new(key.clone());
            for sem_dep in arc_analysis.semantic_dependencies.iter() {
                let qk = match sem_dep {
                    crate::checker::analysis::SemanticDependency::CallableSignature(cid) => {
                        QueryKey::CallableSignature(cid.clone())
                    }
                    crate::checker::analysis::SemanticDependency::DeclarationSurface(did) => {
                        QueryKey::DeclarationSurface(did.clone())
                    }
                    crate::checker::analysis::SemanticDependency::HierarchyEdge(did) => {
                        QueryKey::HierarchyEdge(did.clone())
                    }
                    crate::checker::analysis::SemanticDependency::LinkedInterface(mid) => {
                        QueryKey::LinkedInterface(mid.clone())
                    }
                };
                let _ = db.record_dependency(&mut recorder, qk);
            }
            let deps = recorder.finish();
            let _ = db.publish_product_ready(
                key,
                rev,
                input_fingerprint,
                product_fingerprint,
                SemanticProduct::CallableBody(arc_analysis.clone()),
                deps,
            );
            QueryOutcome::Ready(arc_analysis)
        }
    }
}
