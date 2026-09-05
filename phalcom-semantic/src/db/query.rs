//! Semantic database high-level query execution and caching (Spec 04.5 / Wave 5).

use crate::advisory::{AdvisoryCallableSummary, AdvisoryModuleProduct};
use crate::checker::analysis::{CallableAnalysis, CallableAnalysisStatus};
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::db::key::{InputFingerprint, ProductFingerprint, QueryKey};
use crate::db::product::DeclarationSurfaceProduct;
use crate::db::state::{QueryOutcome, QueryState};
use crate::db::{DependencyEdge, SemanticDb, SemanticProduct};
use crate::declarations::{DeclarationTypeTable, TypeDeclarationShell};
use crate::diagnostic::SemanticDiagnostic;
use crate::dispatch::SurfaceDispatchResolver;
use crate::hierarchy_product::HierarchyEdgeProduct;
use crate::identity::{CallableId, DeclarationId, FieldId, ModuleId};
use crate::module_product::ResolvedImportsProduct;
use crate::signature::{CallableSemanticSignature, FieldSemanticSignature};
use crate::source::ParsedModuleUnit;
use crate::source_index::{CallableSourceAttachment, ModuleSourceIndex};
use crate::surface::DeclarationSurface;
use crate::types::annotation::TypeResolver;
use crate::types::outcome::BlockReason;
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::{ClassDef, Statement};
use phalcom_common::range::SourceRange;
use phalcom_modules::interface::{InterfaceBuilder, LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::linker::LinkedProgram;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Borrowed formal inputs used when a query must evaluate a missing prerequisite.
///
/// This view owns no workspace state and constructs no parallel resolver, linker,
/// or type store. It only exposes the current session inputs to prerequisite
/// helpers.
pub struct FormalQueryInputs<'a> {
    pub sources: &'a BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub linked: &'a LinkedProgram,
    pub import_products: &'a BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub base_resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,
    pub field_signatures: Option<&'a crate::signature::FieldSignatureTable>,
    pub field_lifecycle: Option<&'a crate::checker::field_lifecycle::FieldLifecycleTable>,
    pub enum_semantics: Option<&'a crate::enum_semantics::EnumSemanticTable>,
    pub associated_families: Option<&'a crate::associated::AssociatedFamilyTable>,
}

fn semantic_dependency_query_key(dependency: &crate::checker::analysis::SemanticDependency) -> QueryKey {
    match dependency {
        crate::checker::analysis::SemanticDependency::DeclarationShell(declaration) => QueryKey::DeclarationShell(declaration.clone()),
        crate::checker::analysis::SemanticDependency::CallableSignature(callable) => QueryKey::CallableSignature(callable.clone()),
        crate::checker::analysis::SemanticDependency::FieldSignature(field) => QueryKey::FieldSignature(field.clone()),
        crate::checker::analysis::SemanticDependency::DeclarationSurface(declaration) => QueryKey::DeclarationSurface(declaration.clone()),
        crate::checker::analysis::SemanticDependency::HierarchyEdge(declaration) => QueryKey::HierarchyEdge(declaration.clone()),
        crate::checker::analysis::SemanticDependency::LinkedInterface(module) => QueryKey::LinkedInterface(module.clone()),
        crate::checker::analysis::SemanticDependency::EnumDeclaration(declaration) => QueryKey::EnumDeclaration(declaration.clone()),
        crate::checker::analysis::SemanticDependency::AssociatedSurface(declaration) => QueryKey::AssociatedSurface(declaration.clone()),
        crate::checker::analysis::SemanticDependency::ResolvedImport(site) => QueryKey::ResolvedImport(site.clone()),
        crate::checker::analysis::SemanticDependency::LinkedName(module, name) => QueryKey::LinkedName(module.clone(), name.clone()),
        crate::checker::analysis::SemanticDependency::PublicExport(module, name) => QueryKey::PublicExport(module.clone(), name.clone()),
    }
}

fn semantic_dependency_from_query_key(key: &QueryKey) -> Option<crate::checker::analysis::SemanticDependency> {
    match key {
        QueryKey::LinkedInterface(module) => Some(crate::checker::analysis::SemanticDependency::LinkedInterface(module.clone())),
        QueryKey::ResolvedImport(site) => Some(crate::checker::analysis::SemanticDependency::ResolvedImport(site.clone())),
        QueryKey::LinkedName(module, name) => Some(crate::checker::analysis::SemanticDependency::LinkedName(module.clone(), name.clone())),
        QueryKey::PublicExport(module, name) => Some(crate::checker::analysis::SemanticDependency::PublicExport(module.clone(), name.clone())),
        _ => None,
    }
}

fn query_failure<T>(db: &mut SemanticDb, key: QueryKey, failure: impl Into<String>) -> QueryOutcome<T> {
    let failure = failure.into();
    let revision = db.revision();
    db.set_state(
        key,
        QueryState::Failed {
            revision,
            failure: failure.clone(),
        },
    );
    QueryOutcome::Failed(failure)
}

#[allow(dead_code)]
fn query_blocked<T>(db: &mut SemanticDb, key: QueryKey, reason: BlockReason) -> QueryOutcome<T> {
    let revision = db.revision();
    db.set_state(
        key,
        QueryState::Blocked {
            revision,
            reason: reason.clone(),
        },
    );
    QueryOutcome::Blocked(reason)
}

fn class_definition_for<'a>(unit: &'a ParsedModuleUnit, declaration: &DeclarationId) -> Option<&'a ClassDef> {
    if unit.id != declaration.module {
        return None;
    }
    unit.program.statements.iter().find_map(|statement| match statement {
        Statement::Class(class_def) if class_def.name == declaration.name.as_ref() => Some(class_def),
        _ => None,
    })
}

fn enum_definition_for<'a>(unit: &'a ParsedModuleUnit, declaration: &DeclarationId) -> Option<&'a phalcom_ast::ast::EnumDef> {
    if unit.id != declaration.module {
        return None;
    }
    unit.program.statements.iter().find_map(|statement| match statement {
        Statement::Enum(enum_def) if enum_def.name == declaration.name.as_ref() => Some(enum_def),
        _ => None,
    })
}

fn superclass_source<'a>(unit: &'a ParsedModuleUnit, class_def: &ClassDef) -> Option<&'a str> {
    let range = class_def.superclass.as_ref()?.range;
    unit.text.get(range.start..range.end)
}

fn publish_current_product(
    db: &mut SemanticDb,
    key: QueryKey,
    input_fingerprint: InputFingerprint,
    product_fingerprint: ProductFingerprint,
    product: SemanticProduct,
    dependencies: Vec<DependencyEdge>,
) -> Result<(), String> {
    let revision = db.revision();
    db.publish_product_ready(key, revision, input_fingerprint, product_fingerprint, product, dependencies)
        .map_err(|error| {
            format!(
                "stale semantic query publication: expected revision {:?}, attempted revision {:?}",
                error.expected_revision(),
                error.actual_revision()
            )
        })
}

/// Publishes compiler-owned source structure as a typed incremental product.
pub fn query_source_structure(db: &mut SemanticDb, module: ModuleId, product: Arc<ModuleSourceIndex>) -> QueryOutcome<Arc<ModuleSourceIndex>> {
    let key = QueryKey::SourceStructure(module);
    let input_fingerprint = crate::db::fingerprint::source_structure_input_fingerprint(&product);
    let product_fingerprint = crate::db::fingerprint::source_structure_product_fingerprint(&product);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_source_structure()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::SourceStructure(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Publishes exact formal-to-source attachment with an explicit structure edge.
pub fn query_source_formal_attachment(
    db: &mut SemanticDb,
    callable: CallableId,
    attachment: Arc<CallableSourceAttachment>,
) -> QueryOutcome<Arc<CallableSourceAttachment>> {
    let key = QueryKey::SourceFormalAttachment(callable.clone());
    let structure_key = QueryKey::SourceStructure(callable.module().clone());
    let product_fingerprint = crate::db::fingerprint::source_formal_attachment_fingerprint(&attachment);
    let input_fingerprint = InputFingerprint::new(attachment_fingerprint(&attachment));
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_source_formal_attachment()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    if let Err(error) = db.record_dependency(&mut recorder, structure_key) {
        return query_failure(db, key, error);
    }
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::SourceFormalAttachment(attachment.clone()),
        recorder.finish(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(attachment)
}

/// Publishes one advisory callable summary and records canonical callable
/// dependencies. SCC-internal edges may be omitted during bootstrap and are
/// added by callers once all members have a current product.
pub fn query_advisory_callable(db: &mut SemanticDb, summary: Arc<AdvisoryCallableSummary>) -> QueryOutcome<Arc<AdvisoryCallableSummary>> {
    let key = QueryKey::AdvisoryCallable(summary.callable.clone());
    let input_fingerprint = crate::db::fingerprint::advisory_callable_input_fingerprint(&summary);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_advisory_callable()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    for dependency in summary.dependencies.iter().filter(|dependency| **dependency != summary.callable) {
        let dependency_key = QueryKey::AdvisoryCallable(dependency.clone());
        if let Err(error) = db.record_dependency(&mut recorder, dependency_key) {
            return query_failure(db, key, error);
        }
    }
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        crate::db::fingerprint::advisory_callable_product_fingerprint(&summary),
        SemanticProduct::AdvisoryCallable(summary.clone()),
        recorder.finish(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(summary)
}

/// Seeds one advisory callable product without dependency edges. Callers use
/// this only to bootstrap an SCC, then refresh through
/// [`query_advisory_callable`] once every SCC member is current.
pub fn bootstrap_advisory_callable(db: &mut SemanticDb, summary: Arc<AdvisoryCallableSummary>) -> QueryOutcome<Arc<AdvisoryCallableSummary>> {
    let key = QueryKey::AdvisoryCallable(summary.callable.clone());
    let input_fingerprint = crate::db::fingerprint::advisory_callable_input_fingerprint(&summary);
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        crate::db::fingerprint::advisory_callable_product_fingerprint(&summary),
        SemanticProduct::AdvisoryCallable(summary.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(summary)
}

/// Publishes one advisory module shard after source and callable products.
pub fn query_advisory_module(
    db: &mut SemanticDb,
    product: Arc<AdvisoryModuleProduct>,
    callables: impl IntoIterator<Item = CallableId>,
) -> QueryOutcome<Arc<AdvisoryModuleProduct>> {
    let key = QueryKey::AdvisoryModule(product.module.clone());
    let input_fingerprint = crate::db::fingerprint::advisory_module_input_fingerprint(&product);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_advisory_module()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    let dependency = QueryKey::SourceStructure(product.module.clone());
    if let Err(error) = db.record_dependency(&mut recorder, dependency) {
        return query_failure(db, key, error);
    }
    for callable in callables {
        if let Err(error) = db.record_dependency(&mut recorder, QueryKey::AdvisoryCallable(callable)) {
            return query_failure(db, key, error);
        }
    }
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        crate::db::fingerprint::advisory_module_product_fingerprint(&product),
        SemanticProduct::AdvisoryModule(product.clone()),
        recorder.finish(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

fn attachment_fingerprint(attachment: &CallableSourceAttachment) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    attachment.callable.hash(&mut hasher);
    attachment.expression_sites.hash(&mut hasher);
    attachment.formal_bindings.hash(&mut hasher);
    attachment.formal_expressions.hash(&mut hasher);
    attachment.exact_targets.hash(&mut hasher);
    hasher.finish()
}

/// Evaluates or retrieves the cached `ParsedModuleUnit` for a given module.
pub fn query_parsed_module(db: &mut SemanticDb, module: ModuleId, unit: Arc<ParsedModuleUnit>) -> QueryOutcome<Arc<ParsedModuleUnit>> {
    let key = QueryKey::ParsedModule(module);
    let input_fingerprint = crate::db::fingerprint::parsed_module_input_fingerprint(&unit.id, unit.kind, &unit.text);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_parsed_module()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = ProductFingerprint::new(input_fingerprint.raw());
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::ParsedModule(unit.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(unit)
}

/// Evaluates or retrieves the cached `UnlinkedModuleInterface` for a given module.
pub fn query_unlinked_interface(
    db: &mut SemanticDb,
    module: ModuleId,
    unit: Arc<ParsedModuleUnit>,
    precomputed: Option<Arc<UnlinkedModuleInterface>>,
) -> QueryOutcome<Arc<UnlinkedModuleInterface>> {
    let key = QueryKey::UnlinkedInterface(module.clone());
    let input_fingerprint = crate::db::fingerprint::parsed_module_input_fingerprint(&unit.id, unit.kind, &unit.text);

    // A dependent may only validate after its prerequisite is current. This
    // prevents an old Ready dependency from making transitive reuse appear safe.
    match query_parsed_module(db, module.clone(), unit.clone()) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_unlinked_interface()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    if let Err(error) = db.record_dependency(&mut recorder, QueryKey::ParsedModule(module.clone())) {
        return query_failure(db, key, error);
    }

    let unlinked_arc = match precomputed {
        Some(iface) => iface,
        None => match InterfaceBuilder::build(module.clone(), unit.kind, &unit.program) {
            Ok(unlinked) => Arc::new(unlinked),
            Err(err) => {
                let query_err = format!("failed to build unlinked interface: {err:?}");
                db.set_state(
                    key,
                    QueryState::Failed {
                        revision: db.revision(),
                        failure: query_err.clone(),
                    },
                );
                return QueryOutcome::Failed(query_err);
            }
        },
    };

    let product_fingerprint = crate::db::fingerprint::unlinked_interface_product_fingerprint(&unlinked_arc);
    let deps = recorder.finish();
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::UnlinkedInterface(unlinked_arc.clone()),
        deps,
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(unlinked_arc)
}

/// Evaluates or retrieves the cached `ResolvedImportsProduct` for a given module.
pub fn query_resolved_imports<P: phalcom_modules::source::SourceProvider>(
    db: &mut SemanticDb,
    module: ModuleId,
    unlinked: Arc<UnlinkedModuleInterface>,
    resolver: &mut phalcom_modules::resolver::ModuleResolver<'_, P>,
) -> QueryOutcome<Arc<ResolvedImportsProduct>> {
    let key = QueryKey::ResolvedImports(module.clone());
    let input_fingerprint = crate::db::fingerprint::unlinked_interface_input_fingerprint(&unlinked);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_resolved_imports()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    if let Err(error) = db.record_dependency(&mut recorder, QueryKey::UnlinkedInterface(module.clone())) {
        return query_failure(db, key, error);
    }

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
                    if let Err(error) = db.record_dependency(&mut recorder, QueryKey::UnlinkedInterface(pkg_mod)) {
                        return query_failure(db, key, error);
                    }
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
    let deps = recorder.finish();
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::ResolvedImports(product.clone()),
        deps,
    ) {
        return query_failure(db, key, error);
    }
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
    let input_fingerprint = crate::db::fingerprint::semantic_component_input_fingerprint(&entry, &universe, &interfaces, resolved);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_semantic_component()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    for mod_id in interfaces.keys() {
        for dependency in [QueryKey::UnlinkedInterface(mod_id.clone()), QueryKey::ResolvedImports(mod_id.clone())] {
            if let Err(error) = db.record_dependency(&mut recorder, dependency) {
                return query_failure(db, key, error);
            }
        }
    }

    let linker = phalcom_modules::linker::ModuleLinker::new(universe, interfaces);
    match linker.link(entry, resolved) {
        Ok(linked_program) => {
            let linked_arc = Arc::new(linked_program);
            let deps = recorder.finish();
            let prod_fp = crate::db::fingerprint::semantic_component_product_fingerprint(&linked_arc);
            if let Err(error) = publish_current_product(
                db,
                key.clone(),
                input_fingerprint,
                prod_fp,
                SemanticProduct::SemanticComponent(linked_arc.clone()),
                deps,
            ) {
                return query_failure(db, key, error);
            }
            for (mod_id, linked_mod) in &linked_arc.modules {
                let projection_key = QueryKey::LinkedInterface(mod_id.clone());
                let mod_iface_arc = Arc::new(linked_mod.interface.clone());
                let mod_fp = crate::db::fingerprint::linked_interface_product_fingerprint(&mod_iface_arc);
                if let Err(error) = publish_current_product(
                    db,
                    projection_key.clone(),
                    crate::db::fingerprint::linked_interface_input_fingerprint(&mod_iface_arc),
                    mod_fp,
                    SemanticProduct::LinkedInterface(mod_iface_arc),
                    Vec::new(),
                ) {
                    return query_failure(db, projection_key, error);
                }
            }
            QueryOutcome::Ready(linked_arc)
        }
        Err(err) => {
            let query_err = format!("linker error: {err:?}");
            db.set_state(
                key,
                QueryState::Failed {
                    revision: db.revision(),
                    failure: query_err.clone(),
                },
            );
            QueryOutcome::Failed(query_err)
        }
    }
}

/// Evaluates or computes the direct superclass edge for a source declaration.
pub fn query_hierarchy_edge(
    db: &mut SemanticDb,
    class_decl: DeclarationId,
    unit: Arc<ParsedModuleUnit>,
    linked_interface: Arc<LinkedModuleInterface>,
    resolver: &dyn TypeResolver,
) -> QueryOutcome<Arc<HierarchyEdgeProduct>> {
    let key = QueryKey::HierarchyEdge(class_decl.clone());
    if unit.id != class_decl.module || linked_interface.module != class_decl.module {
        return query_failure(db, key, format!("hierarchy query inputs do not belong to declaration {class_decl:?}"));
    }

    match query_linked_interface(db, linked_interface.module.clone(), linked_interface) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let class_def = class_definition_for(&unit, &class_decl);
    if class_def.is_none() && enum_definition_for(&unit, &class_decl).is_none() {
        return query_failure(db, key, format!("source declaration {class_decl:?} was not found in its parsed module"));
    }
    let superclass_syntax = class_def.and_then(|class_def| superclass_source(&unit, class_def));
    let super_decl = if let Some(class_def) = class_def {
        if let Some(super_ref) = class_def.superclass_ref() {
            let members = super_ref.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
            resolver
                .resolve_type_name(&class_decl.module, &super_ref.root, &members)
                .or_else(|| match (super_ref.root.as_str(), members.is_empty()) {
                    ("Some", true) => Some(crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Some)),
                    ("None", true) => Some(crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::None)),
                    _ => None,
                })
        } else {
            let object = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object);
            (class_decl != object).then_some(object)
        }
    } else {
        Some(crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object))
    };
    let input_fingerprint = if let Some(class_def) = class_def {
        crate::db::fingerprint::hierarchy_edge_input_fingerprint(&class_decl, superclass_source(&unit, class_def), &super_decl)
    } else {
        crate::db::fingerprint::hierarchy_edge_input_fingerprint(&class_decl, Some("Object"), &super_decl)
    };

    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_hierarchy_edge()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    if let Err(error) = db.record_dependency(&mut recorder, QueryKey::LinkedInterface(class_decl.module.clone())) {
        return query_failure(db, key, error);
    }

    let product = Arc::new(HierarchyEdgeProduct::new(class_decl.clone(), super_decl));
    let product_fingerprint = crate::db::fingerprint::hierarchy_edge_product_fingerprint(&class_decl, &product.super_decl, superclass_syntax);
    let dependencies = recorder.finish();
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::HierarchyEdge(product.clone()),
        dependencies,
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Publishes one canonical bootstrap hierarchy edge before source-owned
/// hierarchy queries consume it as a dependency.
pub fn query_bootstrap_hierarchy_edge(
    db: &mut SemanticDb,
    class_decl: DeclarationId,
    super_decl: Option<DeclarationId>,
) -> QueryOutcome<Arc<HierarchyEdgeProduct>> {
    let key = QueryKey::HierarchyEdge(class_decl.clone());
    let input_fingerprint = crate::db::fingerprint::hierarchy_edge_input_fingerprint(&class_decl, None, &super_decl);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_hierarchy_edge()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let product = Arc::new(HierarchyEdgeProduct::new(class_decl.clone(), super_decl.clone()));
    let product_fingerprint = crate::db::fingerprint::hierarchy_edge_product_fingerprint(&class_decl, &super_decl, None);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::HierarchyEdge(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Evaluates or retrieves canonical declaration type metadata for one declaration.
pub fn query_declaration_shell(db: &mut SemanticDb, shell: Arc<TypeDeclarationShell>) -> QueryOutcome<Arc<TypeDeclarationShell>> {
    let key = QueryKey::DeclarationShell(shell.declaration().clone());
    let input_fingerprint = crate::db::fingerprint::declaration_shell_input_fingerprint(shell.as_ref());
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_declaration_shell()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let product_fingerprint = crate::db::fingerprint::declaration_shell_product_fingerprint(shell.as_ref());
    let dependencies = match shell.as_ref() {
        TypeDeclarationShell::Nominal(_) => Vec::new(),
        TypeDeclarationShell::Alias(info) => {
            let mut edges = Vec::with_capacity(info.dependencies.len());
            for dependency in &info.dependencies {
                let dependency_key = QueryKey::DeclarationShell(dependency.clone());
                let Some(observed_fingerprint) = db.ready_product_fingerprint(&dependency_key) else {
                    return query_failure(db, key, format!("alias dependency {dependency:?} is not published"));
                };
                edges.push(DependencyEdge {
                    dependent: key.clone(),
                    observed_fingerprint,
                    dependency: dependency_key,
                });
            }
            edges
        }
    };
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::DeclarationShell(shell.clone()),
        dependencies,
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(shell)
}

/// Publishes one canonical bootstrap declaration surface before source-owned
/// surface queries consume it as a dependency.
pub fn query_bootstrap_declaration_surface(
    db: &mut SemanticDb,
    declaration: DeclarationId,
    surface: Arc<crate::surface::DeclarationSurface>,
) -> QueryOutcome<Arc<crate::surface::DeclarationSurface>> {
    let key = QueryKey::DeclarationSurface(declaration);
    let diagnostics = Arc::<[SemanticDiagnostic]>::from(Vec::new());
    let input_fingerprint = crate::db::fingerprint::declaration_surface_query_input_fingerprint(&surface, &diagnostics);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_declaration_surface()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let product_fingerprint = crate::db::fingerprint::declaration_surface_product_fingerprint(&surface);
    let product = Arc::new(DeclarationSurfaceProduct::new(surface.clone(), diagnostics));
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::DeclarationSurface(product),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(surface)
}

pub fn query_enum_declaration(
    db: &mut SemanticDb,
    product: Arc<crate::db::product::EnumDeclarationProduct>,
) -> QueryOutcome<Arc<crate::db::product::EnumDeclarationProduct>> {
    let key = QueryKey::EnumDeclaration(product.info.owner.clone());
    let input_fingerprint = crate::db::fingerprint::enum_declaration_input_fingerprint(&product);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|p| p.as_enum_declaration()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::enum_declaration_product_fingerprint(&product);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::EnumDeclaration(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

pub fn query_enum_requirements(
    db: &mut SemanticDb,
    owner: DeclarationId,
    product: Arc<crate::db::product::EnumRequirementsProduct>,
) -> QueryOutcome<Arc<crate::db::product::EnumRequirementsProduct>> {
    let key = QueryKey::EnumRequirements(owner.clone());
    let input_fingerprint = crate::db::fingerprint::enum_requirements_input_fingerprint(&owner);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|p| p.as_enum_requirements()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::enum_requirements_product_fingerprint(&product);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::EnumRequirements(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

pub fn query_associated_surface(
    db: &mut SemanticDb,
    surface: Arc<crate::associated::AssociatedSurface>,
) -> QueryOutcome<Arc<crate::associated::AssociatedSurface>> {
    let key = QueryKey::AssociatedSurface(surface.owner.clone());
    let input_fingerprint = crate::db::fingerprint::associated_surface_input_fingerprint(&surface.owner);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|p| p.as_associated_surface()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::associated_surface_product_fingerprint(&surface);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::AssociatedSurface(surface.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(surface)
}

/// Inputs for one declaration-surface query.
pub struct DeclarationSurfaceQuery<'a> {
    pub decl_id: DeclarationId,
    pub unit: Arc<ParsedModuleUnit>,
    pub linked_interface: Arc<LinkedModuleInterface>,
    pub store: &'a mut TypeStore,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,
    pub linked: Option<&'a LinkedProgram>,
    pub import_products: Option<&'a BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
}

fn ensure_semantic_dependency_current(
    db: &mut SemanticDb,
    dependency: &crate::checker::analysis::SemanticDependency,
    linked: Option<&LinkedProgram>,
    declarations: &DeclarationTypeTable,
    import_products: Option<&BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
) -> QueryOutcome<()> {
    match dependency {
        crate::checker::analysis::SemanticDependency::DeclarationShell(declaration) => {
            let key = QueryKey::DeclarationShell(declaration.clone());
            if db.query_state(&key).and_then(QueryState::validated_revision) == Some(db.revision()) {
                return QueryOutcome::Ready(());
            }
            let Some(info) = declarations.get(declaration).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            match query_declaration_shell(db, Arc::new(TypeDeclarationShell::Nominal(info))) {
                QueryOutcome::Ready(_) => QueryOutcome::Ready(()),
                QueryOutcome::Cancelled => QueryOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => QueryOutcome::BudgetExceeded(report),
                QueryOutcome::Blocked(reason) => QueryOutcome::Blocked(reason),
                QueryOutcome::Failed(failure) => QueryOutcome::Failed(failure),
            }
        }
        crate::checker::analysis::SemanticDependency::LinkedInterface(module) => {
            let key = QueryKey::LinkedInterface(module.clone());
            if db.query_state(&key).and_then(QueryState::validated_revision) == Some(db.revision()) {
                return QueryOutcome::Ready(());
            }
            let Some(linked) = linked else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            match ensure_linked_interface(db, module, linked) {
                QueryOutcome::Ready(_) => QueryOutcome::Ready(()),
                QueryOutcome::Cancelled => QueryOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => QueryOutcome::BudgetExceeded(report),
                QueryOutcome::Blocked(reason) => QueryOutcome::Blocked(reason),
                QueryOutcome::Failed(failure) => QueryOutcome::Failed(failure),
            }
        }
        crate::checker::analysis::SemanticDependency::LinkedName(mod_id, name) => {
            let key = QueryKey::LinkedName(mod_id.clone(), name.clone());
            if db.query_state(&key).and_then(QueryState::validated_revision) == Some(db.revision()) {
                return QueryOutcome::Ready(());
            }
            let Some(linked) = linked else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let prelude = crate::prelude::PreludeTypeMap::shared_canonical_universe();
            match query_linked_name(
                db,
                mod_id.clone(),
                name.clone(),
                linked,
                declarations,
                &prelude,
            ) {
                QueryOutcome::Ready(_) => QueryOutcome::Ready(()),
                QueryOutcome::Cancelled => QueryOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => QueryOutcome::BudgetExceeded(report),
                QueryOutcome::Blocked(reason) => QueryOutcome::Blocked(reason),
                QueryOutcome::Failed(failure) => QueryOutcome::Failed(failure),
            }
        }
        crate::checker::analysis::SemanticDependency::PublicExport(mod_id, name) => {
            let key = QueryKey::PublicExport(mod_id.clone(), name.clone());
            if db.query_state(&key).and_then(QueryState::validated_revision) == Some(db.revision()) {
                return QueryOutcome::Ready(());
            }
            let Some(linked) = linked else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            match query_public_export(db, mod_id.clone(), name.clone(), linked) {
                QueryOutcome::Ready(_) => QueryOutcome::Ready(()),
                QueryOutcome::Cancelled => QueryOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => QueryOutcome::BudgetExceeded(report),
                QueryOutcome::Blocked(reason) => QueryOutcome::Blocked(reason),
                QueryOutcome::Failed(failure) => QueryOutcome::Failed(failure),
            }
        }
        crate::checker::analysis::SemanticDependency::ResolvedImport(site) => {
            let key = QueryKey::ResolvedImport(site.clone());
            if db.query_state(&key).and_then(QueryState::validated_revision) == Some(db.revision()) {
                return QueryOutcome::Ready(());
            }
            let Some(import_products) = import_products else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let Some(product) = import_products.get(site).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            match query_resolved_import(db, site.clone(), product) {
                QueryOutcome::Ready(_) => QueryOutcome::Ready(()),
                QueryOutcome::Cancelled => QueryOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => QueryOutcome::BudgetExceeded(report),
                QueryOutcome::Blocked(reason) => QueryOutcome::Blocked(reason),
                QueryOutcome::Failed(failure) => QueryOutcome::Failed(failure),
            }
        }
        _ => QueryOutcome::Ready(()),
    }
}

fn ensure_cached_semantic_dependencies_current(
    db: &mut SemanticDb,
    key: &QueryKey,
    linked: Option<&LinkedProgram>,
    declarations: &DeclarationTypeTable,
    import_products: Option<&BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
) -> QueryOutcome<()> {
    let dependencies = db.index().dependencies_of(key).map(|edges| edges.to_vec()).unwrap_or_default();
    for edge in dependencies {
        let Some(dependency) = semantic_dependency_from_query_key(&edge.dependency) else {
            continue;
        };
        match ensure_semantic_dependency_current(db, &dependency, linked, declarations, import_products) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
    }
    QueryOutcome::Ready(())
}

/// Evaluates or computes the source member surface for one declaration.
pub fn query_declaration_surface(db: &mut SemanticDb, query: DeclarationSurfaceQuery<'_>) -> QueryOutcome<Arc<DeclarationSurface>> {
    let DeclarationSurfaceQuery {
        decl_id,
        unit,
        linked_interface,
        store,
        hierarchy,
        resolver,
        declarations,
        linked,
        import_products,
    } = query;
    let key = QueryKey::DeclarationSurface(decl_id.clone());
    if unit.id != decl_id.module || linked_interface.module != decl_id.module {
        return query_failure(db, key, format!("declaration-surface query inputs do not belong to declaration {decl_id:?}"));
    }

    let class_def = class_definition_for(&unit, &decl_id);
    if class_def.is_none() && enum_definition_for(&unit, &decl_id).is_none() {
        return query_failure(db, key, format!("source declaration {decl_id:?} was not found in its parsed module"));
    }

    let Some(declaration_info) = declarations.get(&decl_id).cloned() else {
        return query_failure(db, key, format!("declaration metadata was not found for {decl_id:?}"));
    };
    let input_fingerprint = if let Some(class_def) = class_def {
        crate::db::fingerprint::declaration_surface_source_input_fingerprint(&unit, &decl_id, class_def)
    } else {
        crate::db::fingerprint::declaration_surface_enum_input_fingerprint(&unit, &decl_id, enum_definition_for(&unit, &decl_id).unwrap())
    };

    match query_declaration_shell(db, Arc::new(TypeDeclarationShell::Nominal(declaration_info))) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    match query_linked_interface(db, linked_interface.module.clone(), linked_interface) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    match ensure_cached_semantic_dependencies_current(db, &key, linked, declarations, import_products) {
        QueryOutcome::Ready(()) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_declaration_surface()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    // Semantic resolution is query-owned and only runs after the source-contract
    // cache lookup misses. Body-only source edits therefore avoid this branch.
    let (computed_surface, computed_diagnostics, captured_dependencies) = {
        let mut context = crate::checker::context::CheckingContext::new(store, hierarchy, resolver, declarations, decl_id.module.clone());
        if let Some(class_def) = class_def {
            crate::checker::declaration::register_class_surface(&mut context, class_def);
        } else {
            // Enum root surfaces are assembled by the enum semantic pass. This
            // product anchors dependency readiness; body queries consume the
            // already-published dispatch surface from the workspace session.
            context.register_surface(decl_id.clone(), DeclarationSurface::new(Some(decl_id.clone())));
        }
        let computed_surface = context.dispatch_ref().get_surface(&decl_id).cloned();
        let captured_dependencies = context.semantic_dependencies_snapshot();
        let diagnostics = Arc::<[crate::diagnostic::SemanticDiagnostic]>::from(context.diagnostics.into_boxed_slice());
        (computed_surface, diagnostics, captured_dependencies)
    };
    let Some(computed_surface) = computed_surface else {
        return query_failure(db, key, format!("declaration-surface query did not publish {decl_id:?}"));
    };
    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    let mut semantic_dependencies = std::collections::BTreeSet::new();
    semantic_dependencies.insert(crate::checker::analysis::SemanticDependency::DeclarationShell(decl_id.clone()));
    semantic_dependencies.insert(crate::checker::analysis::SemanticDependency::LinkedInterface(decl_id.module.clone()));
    semantic_dependencies.extend(captured_dependencies);
    for dependency in semantic_dependencies {
        match ensure_semantic_dependency_current(db, &dependency, linked, declarations, import_products) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
        let dependency_key = semantic_dependency_query_key(&dependency);
        if dependency_key == key {
            return query_failure(db, key, "declaration surface captured a self-surface dependency");
        }
        if let Err(error) = db.record_dependency(&mut recorder, dependency_key) {
            return query_failure(db, key, error);
        }
    }

    let surface = Arc::new(computed_surface);
    let product_fingerprint = crate::db::fingerprint::declaration_surface_product_fingerprint(&surface);
    let product = Arc::new(DeclarationSurfaceProduct::new(surface.clone(), computed_diagnostics));
    let dependencies = recorder.finish();
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::DeclarationSurface(product),
        dependencies,
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(surface)
}

/// Publishes one canonical bootstrap callable signature before source-owned
/// body queries consume it as a dependency.
pub fn query_bootstrap_callable_signature(db: &mut SemanticDb, signature: Arc<CallableSemanticSignature>) -> QueryOutcome<Arc<CallableSemanticSignature>> {
    let key = QueryKey::CallableSignature(signature.callable.clone());
    let input_fingerprint = crate::db::fingerprint::callable_signature_input_fingerprint(&signature);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_callable_signature()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::callable_signature_product_fingerprint(&signature);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::CallableSignature(signature.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(signature)
}

/// Evaluates or retrieves the canonical semantic signature for one source callable.
///
/// Declaration syntax and declaration/type-resolution prerequisites are the
/// authority. `DeclarationSurface` is intentionally absent from this query's
/// dependency set because dispatch is a projection of this product.
pub fn query_callable_signature(
    db: &mut SemanticDb,
    callable: CallableId,
    unit: Arc<ParsedModuleUnit>,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
) -> QueryOutcome<Arc<CallableSemanticSignature>> {
    query_callable_signature_with_inputs(db, callable, unit, store, hierarchy, resolver, declarations, None, None)
}

pub fn query_callable_signature_with_inputs(
    db: &mut SemanticDb,
    callable: CallableId,
    unit: Arc<ParsedModuleUnit>,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
    linked: Option<&LinkedProgram>,
    import_products: Option<&BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
) -> QueryOutcome<Arc<CallableSemanticSignature>> {
    let key = QueryKey::CallableSignature(callable.clone());
    if unit.id != *callable.module() {
        return query_failure(db, key, format!("source unit does not own callable {callable:?}"));
    }

    let Some(declaration_info) = declarations.get(callable.declaration_owner()).cloned() else {
        return query_failure(db, key, format!("missing declaration metadata for {:?}", callable.owner));
    };
    match query_declaration_shell(db, Arc::new(TypeDeclarationShell::Nominal(declaration_info))) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let linked_key = QueryKey::LinkedInterface(callable.module().clone());
    if let Some(linked) = linked {
        match ensure_linked_interface(db, callable.module(), linked) {
            QueryOutcome::Ready(_) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
    }
    if db.query_state(&linked_key).and_then(QueryState::validated_revision) != Some(db.revision()) {
        return query_failure(db, key, format!("CallableSignature prerequisite {linked_key:?} is not current"));
    }

    match ensure_cached_semantic_dependencies_current(db, &key, linked, declarations, import_products) {
        QueryOutcome::Ready(()) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let (signature, captured_dependencies) = if let Some(class_def) = class_definition_for(&unit, callable.declaration_owner()) {
        let Some(member) = class_def.members.iter().find(|member| {
            crate::checker::declaration_signature::callable_id_for_member(callable.declaration_owner(), member).is_some_and(|candidate| candidate == callable)
        }) else {
            return query_failure(db, key, format!("missing source declaration for callable {callable:?}"));
        };

        let mut context = crate::checker::CheckingContext::new(store, hierarchy, resolver, declarations, callable.module().clone());
        let Some(signature) = crate::checker::declaration_signature::semantic_signature_for_member(&mut context, callable.declaration_owner(), member) else {
            return query_failure(db, key, format!("source member cannot publish callable signature {callable:?}"));
        };
        (Arc::new(signature), context.semantic_dependencies_snapshot())
    } else if let Some(enum_def) = enum_definition_for(&unit, callable.declaration_owner()) {
        let mut context = crate::checker::CheckingContext::new(store, hierarchy, resolver, declarations, callable.module().clone());
        let signature = match &callable.owner {
            crate::identity::CallableOwnerId::Declaration(_) => {
                let Some(sig) = enum_def.members.iter().find_map(|m| match m {
                    phalcom_ast::ast::EnumMember::Behavior(b) => {
                        let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(b);
                        let is_class_side = syntax.attributes().iter().any(|attr| attr.name == "class")
                            || match b {
                                phalcom_ast::ast::EnumBehaviorMember::Method(m) => m.is_static,
                                phalcom_ast::ast::EnumBehaviorMember::Getter(g) => g.is_static,
                                phalcom_ast::ast::EnumBehaviorMember::Setter(s) => s.is_static,
                                phalcom_ast::ast::EnumBehaviorMember::Index(_) => false,
                            };
                        let side = if is_class_side {
                            crate::identity::DispatchSide::Class
                        } else {
                            crate::identity::DispatchSide::Instance
                        };
                        if crate::checker::declaration_signature::callable_id_for_syntax(&callable.owner, syntax, side).as_ref() == Some(&callable) {
                            crate::checker::declaration_signature::semantic_signature_for_syntax(&mut context, &callable.owner, syntax, side)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }) else {
                    return query_failure(db, key, format!("missing root enum behavior for {callable:?}"));
                };
                sig
            }
            crate::identity::CallableOwnerId::Variant(var_id) => {
                let Some(sig) = enum_def.members.iter().find_map(|m| match m {
                    phalcom_ast::ast::EnumMember::Variant(v) => {
                        let sel = phalcom_ast::selector::selector_from_variant(v);
                        if sel == var_id.selector {
                            v.body.as_ref().and_then(|body| {
                                body.members.iter().find_map(|case_member| {
                                    let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(case_member);
                                    if crate::checker::declaration_signature::callable_id_for_syntax(
                                        &callable.owner,
                                        syntax,
                                        crate::identity::DispatchSide::Instance,
                                    )
                                    .as_ref()
                                        == Some(&callable)
                                    {
                                        crate::checker::declaration_signature::semantic_signature_for_syntax(
                                            &mut context,
                                            &callable.owner,
                                            syntax,
                                            crate::identity::DispatchSide::Instance,
                                        )
                                    } else {
                                        None
                                    }
                                })
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                }) else {
                    return query_failure(db, key, format!("missing case enum behavior for {callable:?}"));
                };
                sig
            }
        };
        (Arc::new(signature), context.semantic_dependencies_snapshot())
    } else {
        return query_failure(db, key, format!("missing class/enum declaration for {:?}", callable.owner));
    };

    let input_fingerprint = crate::db::fingerprint::callable_signature_input_fingerprint(&signature);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_callable_signature()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    for dependency in &captured_dependencies {
        match ensure_semantic_dependency_current(db, dependency, linked, declarations, import_products) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
    }

    let mut dependency_keys = BTreeSet::from([QueryKey::DeclarationShell(callable.declaration_owner().clone()), linked_key]);
    dependency_keys.extend(captured_dependencies.iter().map(semantic_dependency_query_key));
    dependency_keys.remove(&key);

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    for dependency in dependency_keys {
        if let Err(error) = db.record_dependency(&mut recorder, dependency) {
            return query_failure(db, key, error);
        }
    }

    let product_fingerprint = crate::db::fingerprint::callable_signature_product_fingerprint(&signature);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::CallableSignature(signature.clone()),
        recorder.finish(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(signature)
}

/// Evaluates or retrieves canonical declaration knowledge for one source field.
///
/// Source declaration syntax and type-resolution prerequisites are authoritative;
/// `DeclarationSurface` is deliberately not an input because it is a projection
/// of this product.
pub fn query_field_signature(
    db: &mut SemanticDb,
    field: FieldId,
    unit: Arc<ParsedModuleUnit>,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
) -> QueryOutcome<Arc<FieldSemanticSignature>> {
    query_field_signature_with_inputs(db, field, unit, store, hierarchy, resolver, declarations, None, None)
}

pub fn query_field_signature_with_inputs(
    db: &mut SemanticDb,
    field: FieldId,
    unit: Arc<ParsedModuleUnit>,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
    linked: Option<&LinkedProgram>,
    import_products: Option<&BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
) -> QueryOutcome<Arc<FieldSemanticSignature>> {
    let key = QueryKey::FieldSignature(field.clone());
    if unit.id != field.owner.module {
        return query_failure(db, key, format!("source unit does not own field {field:?}"));
    }

    let Some(declaration_info) = declarations.get(&field.owner).cloned() else {
        return query_failure(db, key, format!("missing declaration metadata for {:?}", field.owner));
    };
    match query_declaration_shell(db, Arc::new(TypeDeclarationShell::Nominal(declaration_info))) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let linked_key = QueryKey::LinkedInterface(field.owner.module.clone());
    if let Some(linked) = linked {
        match ensure_linked_interface(db, &field.owner.module, linked) {
            QueryOutcome::Ready(_) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
    }
    if db.query_state(&linked_key).and_then(QueryState::validated_revision) != Some(db.revision()) {
        return query_failure(db, key, format!("FieldSignature prerequisite {linked_key:?} is not current"));
    }

    match ensure_cached_semantic_dependencies_current(db, &key, linked, declarations, import_products) {
        QueryOutcome::Ready(()) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let Some(class_def) = class_definition_for(&unit, &field.owner) else {
        return query_failure(db, key, format!("missing class declaration for {:?}", field.owner));
    };
    let Some(member) = class_def
        .members
        .iter()
        .find(|member| crate::checker::declaration_signature::field_id_for_member(&field.owner, member).as_ref() == Some(&field))
    else {
        return query_failure(db, key, format!("missing source declaration for field {field:?}"));
    };

    let (signature, captured_dependencies) = {
        let mut context = crate::checker::CheckingContext::new(store, hierarchy, resolver, declarations, field.owner.module.clone());
        let Some(signature) = crate::checker::declaration_signature::semantic_field_signature_for_member(&mut context, &field.owner, member) else {
            return query_failure(db, key, format!("source member cannot publish field signature {field:?}"));
        };
        (Arc::new(signature), context.semantic_dependencies_snapshot())
    };

    let input_fingerprint = crate::db::fingerprint::field_signature_input_fingerprint(&signature);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_field_signature()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    for dependency in &captured_dependencies {
        match ensure_semantic_dependency_current(db, dependency, linked, declarations, import_products) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
    }

    let mut dependency_keys = BTreeSet::from([QueryKey::DeclarationShell(field.owner.clone()), linked_key]);
    dependency_keys.extend(captured_dependencies.iter().map(semantic_dependency_query_key));
    dependency_keys.remove(&key);

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    for dependency in dependency_keys {
        if let Err(error) = db.record_dependency(&mut recorder, dependency) {
            return query_failure(db, key, error);
        }
    }

    let product_fingerprint = crate::db::fingerprint::field_signature_product_fingerprint(&signature);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::FieldSignature(signature.clone()),
        recorder.finish(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(signature)
}

fn declaration_signature_id_for_body(callable: &CallableId, unit: &ParsedModuleUnit) -> Option<CallableId> {
    if let Some(class_def) = class_definition_for(unit, callable.declaration_owner()) {
        if class_def
            .members
            .iter()
            .any(|member| crate::checker::declaration_signature::callable_id_for_member(callable.declaration_owner(), member).as_ref() == Some(callable))
        {
            return Some(callable.clone());
        }

        if callable.side == crate::identity::DispatchSide::Instance {
            let class_side = CallableId::new(callable.owner.clone(), callable.selector.clone(), crate::identity::DispatchSide::Class);
            if class_def
                .members
                .iter()
                .any(|member| crate::checker::declaration_signature::callable_id_for_member(callable.declaration_owner(), member).as_ref() == Some(&class_side))
            {
                return Some(class_side);
            }
        }
        return None;
    }

    if let Some(enum_def) = enum_definition_for(unit, callable.declaration_owner()) {
        match &callable.owner {
            crate::identity::CallableOwnerId::Declaration(_) => {
                for member in &enum_def.members {
                    if let phalcom_ast::ast::EnumMember::Behavior(b) = member {
                        let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(b);
                        let is_class_side = syntax.attributes().iter().any(|attr| attr.name == "class")
                            || match b {
                                phalcom_ast::ast::EnumBehaviorMember::Method(m) => m.is_static,
                                phalcom_ast::ast::EnumBehaviorMember::Getter(g) => g.is_static,
                                phalcom_ast::ast::EnumBehaviorMember::Setter(s) => s.is_static,
                                phalcom_ast::ast::EnumBehaviorMember::Index(_) => false,
                            };
                        let side = if is_class_side {
                            crate::identity::DispatchSide::Class
                        } else {
                            crate::identity::DispatchSide::Instance
                        };
                        if crate::checker::declaration_signature::callable_id_for_syntax(&callable.owner, syntax, side).as_ref() == Some(callable) {
                            return Some(callable.clone());
                        }
                    }
                }
            }
            crate::identity::CallableOwnerId::Variant(var_id) => {
                for member in &enum_def.members {
                    if let phalcom_ast::ast::EnumMember::Variant(v) = member {
                        let sel = phalcom_ast::selector::selector_from_variant(v);
                        if sel == var_id.selector {
                            if let Some(ref body) = v.body {
                                for case_member in &body.members {
                                    let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(case_member);
                                    if crate::checker::declaration_signature::callable_id_for_syntax(
                                        &callable.owner,
                                        syntax,
                                        crate::identity::DispatchSide::Instance,
                                    )
                                    .as_ref()
                                        == Some(callable)
                                    {
                                        return Some(callable.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn ensure_declaration_shell(db: &mut SemanticDb, declaration: &DeclarationId, declarations: &DeclarationTypeTable) -> QueryOutcome<Arc<TypeDeclarationShell>> {
    let Some(info) = declarations.get(declaration).cloned() else {
        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
    };
    query_declaration_shell(db, Arc::new(TypeDeclarationShell::Nominal(info)))
}

fn ensure_linked_interface(db: &mut SemanticDb, module: &ModuleId, linked: &LinkedProgram) -> QueryOutcome<Arc<LinkedModuleInterface>> {
    let Some(linked_module) = linked.modules.get(module) else {
        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
    };
    query_linked_interface(db, module.clone(), Arc::new(linked_module.interface.clone()))
}

fn ensure_callable_signature_with_inputs(
    db: &mut SemanticDb,
    callable: &CallableId,
    formal_inputs: &FormalQueryInputs<'_>,
    store: &mut TypeStore,
) -> QueryOutcome<Arc<CallableSemanticSignature>> {
    match ensure_declaration_shell(db, callable.declaration_owner(), formal_inputs.declarations) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }
    match ensure_linked_interface(db, callable.module(), formal_inputs.linked) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }
    let Some(unit) = formal_inputs.sources.get(callable.module()).cloned() else {
        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
    };
    query_callable_signature_with_inputs(
        db,
        callable.clone(),
        unit,
        store,
        formal_inputs.hierarchy,
        formal_inputs.base_resolver,
        formal_inputs.declarations,
        Some(formal_inputs.linked),
        Some(formal_inputs.import_products),
    )
}

fn ensure_field_signature_with_inputs(
    db: &mut SemanticDb,
    field: &FieldId,
    formal_inputs: &FormalQueryInputs<'_>,
    store: &mut TypeStore,
) -> QueryOutcome<Arc<FieldSemanticSignature>> {
    match ensure_declaration_shell(db, &field.owner, formal_inputs.declarations) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }
    match ensure_linked_interface(db, &field.owner.module, formal_inputs.linked) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }
    let Some(unit) = formal_inputs.sources.get(&field.owner.module).cloned() else {
        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
    };
    query_field_signature_with_inputs(
        db,
        field.clone(),
        unit,
        store,
        formal_inputs.hierarchy,
        formal_inputs.base_resolver,
        formal_inputs.declarations,
        Some(formal_inputs.linked),
        Some(formal_inputs.import_products),
    )
}

/// Evaluates or retrieves the cached `LinkedModuleInterface` for a module.
pub fn query_linked_interface(db: &mut SemanticDb, module: ModuleId, linked_interface: Arc<LinkedModuleInterface>) -> QueryOutcome<Arc<LinkedModuleInterface>> {
    let key = QueryKey::LinkedInterface(module);
    let prod_fp = crate::db::fingerprint::linked_interface_product_fingerprint(&linked_interface);
    let input_fingerprint = crate::db::fingerprint::linked_interface_input_fingerprint(&linked_interface);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_linked_interface()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        prod_fp,
        SemanticProduct::LinkedInterface(linked_interface.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(linked_interface)
}

/// Evaluates or retrieves the cached `ModuleDiagnostics` for a module.
pub fn query_module_diagnostics(db: &mut SemanticDb, module: ModuleId, diagnostics: Arc<[SemanticDiagnostic]>) -> QueryOutcome<Arc<[SemanticDiagnostic]>> {
    let key = QueryKey::ModuleDiagnostics(module.clone());
    let prod_fp = crate::db::fingerprint::module_diagnostics_product_fingerprint(&module, &diagnostics);
    let input_fingerprint = InputFingerprint::new(prod_fp.raw());
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|p| p.as_module_diagnostics()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        prod_fp,
        SemanticProduct::ModuleDiagnostics(diagnostics.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(diagnostics)
}

/// Computes the exact `LinkedNameFact` for a module and name.
pub fn compute_linked_name_fact(
    module: &ModuleId,
    name: &str,
    linked: &LinkedProgram,
    declarations: &DeclarationTypeTable,
    prelude_types: &crate::prelude::PreludeTypeMap,
) -> crate::db::product::LinkedNameFact {
    let decl_candidate = DeclarationId::new(module.clone(), name.into());
    if declarations.contains(&decl_candidate) {
        return crate::db::product::LinkedNameFact::Local(decl_candidate);
    }

    if let Some(linked_mod) = linked.modules.get(module) {
        if let Some(&import_id) = linked_mod.bindings.imports.get::<str>(name) {
            if let Some(spec) = linked_mod.linked_reads.get(import_id.0 as usize) {
                match spec {
                    phalcom_modules::linker::LinkedReadSpec::Binding(sym) => {
                        let decl = DeclarationId::new(sym.module.clone(), sym.name.clone());
                        return crate::db::product::LinkedNameFact::ImportedBinding(decl);
                    }
                    phalcom_modules::linker::LinkedReadSpec::Module(target_mod) => {
                        return crate::db::product::LinkedNameFact::ImportedModule(target_mod.clone());
                    }
                }
            }
        }

        if let Some(export) = linked_mod.interface.exports.get::<str>(name) {
            match &export.target {
                phalcom_modules::interface::LinkedExportTarget::Binding(sym) => {
                    let decl = DeclarationId::new(sym.module.clone(), sym.name.clone());
                    return crate::db::product::LinkedNameFact::ImportedBinding(decl);
                }
                phalcom_modules::interface::LinkedExportTarget::Module(target_mod) => {
                    return crate::db::product::LinkedNameFact::ImportedModule(target_mod.clone());
                }
            }
        }
    }

    if let Some(decl) = prelude_types.get(name) {
        if declarations.contains(decl) {
            return crate::db::product::LinkedNameFact::ImportedBinding(decl.clone());
        }
    }

    crate::db::product::LinkedNameFact::Absent
}

/// Computes the exact `PublicExportFact` for a module and export name.
pub fn compute_public_export_fact(
    module: &ModuleId,
    name: &str,
    linked: &LinkedProgram,
) -> crate::db::product::PublicExportFact {
    if let Some(linked_mod) = linked.modules.get(module) {
        if let Some(export) = linked_mod.interface.exports.get::<str>(name) {
            return crate::db::product::PublicExportFact::Present(export.clone());
        }
    }
    crate::db::product::PublicExportFact::Absent
}

/// Evaluates or retrieves the cached `LinkedNameProduct` for a module name.
pub fn query_linked_name(
    db: &mut SemanticDb,
    module: ModuleId,
    name: String,
    linked: &LinkedProgram,
    declarations: &DeclarationTypeTable,
    prelude_types: &crate::prelude::PreludeTypeMap,
) -> QueryOutcome<Arc<crate::db::product::LinkedNameProduct>> {
    let key = QueryKey::LinkedName(module.clone(), name.clone());

    // Exact name facts are invalidated through the current linked interface.
    // Materialize that prerequisite before attempting reuse; otherwise a
    // cached name fact could validate against an older interface revision.
    let has_linked_interface = match linked.modules.get(&module) {
        Some(linked_mod) => match query_linked_interface(db, module.clone(), Arc::new(linked_mod.interface.clone())) {
            QueryOutcome::Ready(_) => true,
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        },
        None => false,
    };

    let fact = compute_linked_name_fact(&module, &name, linked, declarations, prelude_types);
    let product = Arc::new(crate::db::product::LinkedNameProduct {
        module: module.clone(),
        name,
        fact,
    });
    let input_fingerprint = crate::db::fingerprint::linked_name_input_fingerprint_with_module(&product, linked.modules.get(&module));
    let product_fingerprint = crate::db::fingerprint::linked_name_product_fingerprint(&product);

    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_linked_name()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    if has_linked_interface {
        if let Err(error) = db.record_dependency(&mut recorder, QueryKey::LinkedInterface(module.clone())) {
            return query_failure(db, key, error);
        }
    }

    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::LinkedName(product.clone()),
        recorder.finish(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Evaluates or retrieves the cached `PublicExportProduct` for a public export.
pub fn query_public_export(
    db: &mut SemanticDb,
    module: ModuleId,
    name: String,
    linked: &LinkedProgram,
) -> QueryOutcome<Arc<crate::db::product::PublicExportProduct>> {
    let key = QueryKey::PublicExport(module.clone(), name.clone());

    // Public export facts are exact projections of one module's linked
    // interface. Ensure that canonical input is current before validating the
    // cached projection.
    let has_linked_interface = match linked.modules.get(&module) {
        Some(linked_mod) => match query_linked_interface(db, module.clone(), Arc::new(linked_mod.interface.clone())) {
            QueryOutcome::Ready(_) => true,
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        },
        None => false,
    };

    let fact = compute_public_export_fact(&module, &name, linked);
    let product = Arc::new(crate::db::product::PublicExportProduct {
        module: module.clone(),
        name,
        fact,
    });
    let input_fingerprint = crate::db::fingerprint::public_export_input_fingerprint_with_interface(
        &product,
        linked.modules.get(&module).map(|linked_module| &linked_module.interface),
    );
    let product_fingerprint = crate::db::fingerprint::public_export_product_fingerprint(&product);

    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_public_export()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    if has_linked_interface {
        if let Err(error) = db.record_dependency(&mut recorder, QueryKey::LinkedInterface(module.clone())) {
            return query_failure(db, key, error);
        }
    }

    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::PublicExport(product.clone()),
        recorder.finish(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Evaluates or retrieves the cached `ImportResolutionProduct` for an import site.
pub fn query_resolved_import(
    db: &mut SemanticDb,
    site: phalcom_modules::identity::ImportSiteId,
    product: Arc<phalcom_modules::resolver::ImportResolutionProduct>,
) -> QueryOutcome<Arc<phalcom_modules::resolver::ImportResolutionProduct>> {
    let key = QueryKey::ResolvedImport(site);
    let input_fingerprint = crate::db::fingerprint::resolved_import_input_fingerprint(&product);
    let product_fingerprint = crate::db::fingerprint::resolved_import_product_fingerprint(&product);

    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_resolved_import()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::ResolvedImport(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CallableBodySignatureRequirement {
    Required,
    SignaturelessSynthetic,
}

/// Inputs for one callable-body query.
pub struct CallableBodyQuery<'a> {
    pub callable: CallableId,
    pub body: &'a [Statement],
    pub body_range: SourceRange,
    pub store: &'a mut TypeStore,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,
    pub dispatch: &'a SurfaceDispatchResolver,
    pub module: ModuleId,
    pub budget: QueryBudget,
    pub cancel: &'a CancellationToken,
    pub formal_inputs: Option<&'a FormalQueryInputs<'a>>,
}

/// Evaluates or retrieves the cached `CallableAnalysis` for a declared callable body.
///
/// Declared bodies fail closed unless their canonical `CallableSignature` product is
/// current. Tests that intentionally exercise a body without a declaration must use
/// [`query_signatureless_callable_body`] explicitly.
pub fn query_callable_body(db: &mut SemanticDb, query: CallableBodyQuery<'_>) -> QueryOutcome<Arc<CallableAnalysis>> {
    query_callable_body_with_requirement(db, query, CallableBodySignatureRequirement::Required)
}

/// Low-level query entry for synthetic DB fixtures that deliberately have no
/// source declaration and therefore no canonical callable-signature product.
pub fn query_signatureless_callable_body(db: &mut SemanticDb, query: CallableBodyQuery<'_>) -> QueryOutcome<Arc<CallableAnalysis>> {
    query_callable_body_with_requirement(db, query, CallableBodySignatureRequirement::SignaturelessSynthetic)
}

/// Evaluates a declared callable body while allowing missing formal prerequisites
/// to be evaluated from borrowed current workspace inputs.
pub fn query_callable_body_with_formal_inputs(db: &mut SemanticDb, query: CallableBodyQuery<'_>) -> QueryOutcome<Arc<CallableAnalysis>> {
    query_callable_body_with_requirement(db, query, CallableBodySignatureRequirement::Required)
}

fn query_callable_body_with_requirement(
    db: &mut SemanticDb,
    query: CallableBodyQuery<'_>,
    signature_requirement: CallableBodySignatureRequirement,
) -> QueryOutcome<Arc<CallableAnalysis>> {
    let CallableBodyQuery {
        callable,
        body,
        body_range,
        store,
        hierarchy,
        resolver,
        declarations,
        dispatch,
        module,
        budget,
        cancel,
        formal_inputs,
    } = query;
    let key = QueryKey::CallableBody(callable.clone());

    if let Some(inputs) = formal_inputs {
        let cached_dependencies = db.index().dependencies_of(&key).map(|edges| edges.to_vec()).unwrap_or_default();
        for edge in cached_dependencies {
            let Some(dependency) = semantic_dependency_from_query_key(&edge.dependency) else {
                continue;
            };
            match ensure_semantic_dependency_current(
                db,
                &dependency,
                Some(inputs.linked),
                inputs.declarations,
                Some(inputs.import_products),
            ) {
                QueryOutcome::Ready(()) => {}
                QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
                QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
                QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
            }
        }
    }

    let input_fingerprint = match formal_inputs {
        Some(inputs) => crate::db::fingerprint::callable_body_input_fingerprint_with_formal_inputs(
            &callable,
            body,
            body_range,
            store,
            inputs.sources,
            inputs.field_lifecycle,
        ),
        None => crate::db::fingerprint::callable_body_input_fingerprint(&callable, body, body_range, store),
    };

    // Every source callable declaration has a canonical signature product,
    // including partially-known signatures. Constructor body identities remain
    // instance-side while consuming their class-side constructor declaration.
    let declared_signature = match formal_inputs {
        Some(inputs) => {
            let Some(unit) = inputs.sources.get(callable.module()).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let Some(signature_id) = declaration_signature_id_for_body(&callable, &unit) else {
                return query_failure(db, key.clone(), format!("missing declaration signature identity for body {callable:?}"));
            };
            match ensure_callable_signature_with_inputs(db, &signature_id, inputs, store) {
                QueryOutcome::Ready(signature) => Some((signature_id, signature)),
                QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
                QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
                QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
            }
        }
        None => {
            let current_signature = |db: &SemanticDb, signature_id: &CallableId| {
                let signature_key = QueryKey::CallableSignature(signature_id.clone());
                (db.query_state(&signature_key).and_then(QueryState::validated_revision) == Some(db.revision()))
                    .then(|| db.product(&signature_key).and_then(|product| product.as_callable_signature()).cloned())
                    .flatten()
            };
            let direct = current_signature(db, &callable).map(|signature| (callable.clone(), signature));
            let found = direct.or_else(|| {
                (callable.side == crate::identity::DispatchSide::Instance)
                    .then(|| {
                        let signature_id = CallableId::new(callable.owner.clone(), callable.selector.clone(), crate::identity::DispatchSide::Class);
                        current_signature(db, &signature_id).map(|signature| (signature_id, signature))
                    })
                    .flatten()
            });
            match (found, signature_requirement) {
                (Some(signature), _) => Some(signature),
                (None, CallableBodySignatureRequirement::SignaturelessSynthetic) => None,
                (None, CallableBodySignatureRequirement::Required) => {
                    return query_failure(
                        db,
                        key.clone(),
                        format!("missing current canonical CallableSignature prerequisite for body {callable:?}"),
                    );
                }
            }
        }
    };

    // 1. Check if already computed and ready for the same callable input and dependency products.
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_callable_body()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }

    // A ready product with a different input, or a non-ready state from an
    // earlier attempt, cannot remain current while this generation recomputes
    // it. Preserve incoming dependents: their observed product fingerprints
    // decide lazily whether they can revalidate after this body republishes.
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    // 2. Perform analysis
    let analysis = crate::checker::body::analyze_callable_body(
        crate::checker::body::BodyAnalysisContext {
            store,
            hierarchy,
            resolver,
            declarations,
            dispatch,
            module,
        },
        crate::checker::body::CallableBodyRequest {
            callable,
            body,
            body_range,
            declared_signature: declared_signature.as_ref().map(|(signature_id, signature)| (signature_id, signature.as_ref())),
            budget,
            cancel,
            field_signatures: formal_inputs.and_then(|inputs| inputs.field_signatures),
            field_lifecycle: formal_inputs.and_then(|inputs| inputs.field_lifecycle),
            enum_semantics: formal_inputs.and_then(|inputs| inputs.enum_semantics),
            associated_families: formal_inputs.and_then(|inputs| inputs.associated_families),
        },
    );

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
        // Internal failures are already contained at callable scope. Publish
        // the structured product so release/LSP queries remain operational;
        // test fixtures enforce fail-fast policy by asserting the incident
        // collection is empty.
        CallableAnalysisStatus::Complete | CallableAnalysisStatus::Partial | CallableAnalysisStatus::InternalFailure(_) => {
            let mut recorder = crate::db::DependencyRecorder::new(key.clone());
            for sem_dep in arc_analysis.semantic_dependencies.iter() {
                match (sem_dep, formal_inputs) {
                    (crate::checker::analysis::SemanticDependency::FieldSignature(field), Some(inputs)) => {
                        match ensure_field_signature_with_inputs(db, field, inputs, store) {
                            QueryOutcome::Ready(_) => {}
                            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
                            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
                            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
                            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
                        }
                    }
                    _ => {}
                }
                let dependency_inputs = formal_inputs.map(|inputs| (Some(inputs.linked), Some(inputs.import_products)));
                let (linked, import_products) = dependency_inputs.unwrap_or((None, None));
                match ensure_semantic_dependency_current(db, sem_dep, linked, declarations, import_products) {
                    QueryOutcome::Ready(()) => {}
                    QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
                    QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
                    QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
                    QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
                }
                let dependency = semantic_dependency_query_key(sem_dep);
                if let Err(error) = db.record_dependency(&mut recorder, dependency) {
                    return query_failure(db, key, error);
                }
            }
            let deps = recorder.finish();
            if let Err(error) = publish_current_product(
                db,
                key.clone(),
                input_fingerprint,
                product_fingerprint,
                SemanticProduct::CallableBody(arc_analysis.clone()),
                deps,
            ) {
                return query_failure(db, key, error);
            }
            QueryOutcome::Ready(arc_analysis)
        }
    }
}

#[cfg(test)]
mod exact_module_fact_tests {
    use super::*;
    use crate::db::key::QueryKey;
    use crate::db::product::{LinkedNameFact, PublicExportFact};
    use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
    use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout, SymbolId};
    use phalcom_modules::metadata::ModuleMetadata;
    use phalcom_modules::project::ProjectUniverse;
    use phalcom_modules::resolver::{ImportPathIdentity, ImportResolutionProduct, ResolutionTopologyDependencies};
    use phalcom_modules::{ModuleComponent, ModuleId, ModuleKind, ModulePath, ResolvedProjectId};

    fn module(name: &str) -> ModuleId {
        ModuleId::resolved(
            ResolvedProjectId::from_raw(1),
            ModulePath::from_components(vec![ModuleComponent::from_identifier(name).expect("test module name")]),
        )
    }

    fn linked_program(module_id: ModuleId, exports: impl IntoIterator<Item = LinkedExport>) -> LinkedProgram {
        let exports = exports.into_iter().map(|export| (export.public_name.clone(), export)).collect();
        let interface = LinkedModuleInterface {
            module: module_id.clone(),
            kind: ModuleKind::Module,
            exports,
            metadata: ModuleMetadata::default(),
        };
        let linked_module = LinkedModule {
            interface,
            bindings: ModuleBindingLayout::default(),
            linked_reads: Vec::new(),
            runtime_dependencies: Vec::new(),
        };
        LinkedProgram {
            universe: Arc::new(ProjectUniverse::new()),
            modules: BTreeMap::from([(module_id.clone(), linked_module)]),
            graphs: Default::default(),
            entry: module_id.clone(),
            initialization_order: vec![module_id],
        }
    }

    fn binding_export(public_name: &str, target_module: ModuleId, target_name: &str) -> LinkedExport {
        LinkedExport {
            public_name: public_name.into(),
            target: LinkedExportTarget::Binding(SymbolId {
                module: target_module,
                name: target_name.into(),
            }),
            range: SourceRange::default(),
        }
    }

    fn resolved_import(site: phalcom_modules::identity::ImportSiteId, target: ModuleId) -> Arc<phalcom_modules::resolver::ImportResolutionProduct> {
        Arc::new(ImportResolutionProduct::new(
            site,
            ImportPathIdentity {
                written: "dep".into(),
                is_relative: true,
            },
            Arc::from(Vec::new()),
            Ok(target),
            ResolutionTopologyDependencies::default(),
        ))
    }

    #[test]
    fn public_export_refreshes_when_current_linked_interface_changes() {
        let provider = module("provider");
        let first = linked_program(provider.clone(), [binding_export("Foo", module("a1"), "Foo")]);
        let second = linked_program(provider.clone(), [binding_export("Foo", module("a2"), "Foo")]);
        let mut db = SemanticDb::new();

        let first_product = match query_public_export(&mut db, provider.clone(), "Foo".into(), &first) {
            QueryOutcome::Ready(product) => product,
            outcome => panic!("first export query was not ready: {outcome:?}"),
        };
        let key = QueryKey::PublicExport(provider.clone(), "Foo".into());
        let first_fingerprint = db.ready_product_fingerprint(&key).expect("first fingerprint");
        assert!(matches!(first_product.fact, PublicExportFact::Present(_)));
        assert!(db.index().dependencies_of(&key).is_some_and(|edges| {
            edges.iter().any(|edge| edge.dependency == QueryKey::LinkedInterface(provider.clone()))
        }));

        db.begin_revision();
        let second_product = match query_public_export(&mut db, provider, "Foo".into(), &second) {
            QueryOutcome::Ready(product) => product,
            outcome => panic!("second export query was not ready: {outcome:?}"),
        };
        let second_fingerprint = db.ready_product_fingerprint(&key).expect("second fingerprint");

        assert_ne!(first_fingerprint, second_fingerprint);
        assert!(matches!(second_product.fact, PublicExportFact::Present(ref export) if export.symbol().is_some_and(|symbol| symbol.module == module("a2"))));
    }

    #[test]
    fn public_export_product_stays_stable_for_unrelated_export() {
        let provider = module("provider");
        let first = linked_program(provider.clone(), [binding_export("Foo", module("a1"), "Foo")]);
        let second = linked_program(
            provider.clone(),
            [binding_export("Foo", module("a1"), "Foo"), binding_export("Bar", module("a1"), "Bar")],
        );
        let mut db = SemanticDb::new();

        assert!(matches!(query_public_export(&mut db, provider.clone(), "Foo".into(), &first), QueryOutcome::Ready(_)));
        let key = QueryKey::PublicExport(provider.clone(), "Foo".into());
        let first_fingerprint = db.ready_product_fingerprint(&key).expect("first fingerprint");
        db.begin_revision();
        assert!(matches!(query_public_export(&mut db, provider, "Foo".into(), &second), QueryOutcome::Ready(_)));
        let second_fingerprint = db.ready_product_fingerprint(&key).expect("second fingerprint");

        assert_eq!(first_fingerprint, second_fingerprint);
    }

    #[test]
    fn linked_name_refreshes_absent_fact_when_interface_adds_name() {
        let provider = module("provider");
        let first = linked_program(provider.clone(), []);
        let second = linked_program(provider.clone(), [binding_export("Missing", provider.clone(), "Missing")]);
        let declarations = DeclarationTypeTable::new();
        let prelude = crate::prelude::PreludeTypeMap::shared_canonical_universe();
        let mut db = SemanticDb::new();

        let first_product = match query_linked_name(&mut db, provider.clone(), "Missing".into(), &first, &declarations, &prelude) {
            QueryOutcome::Ready(product) => product,
            outcome => panic!("first name query was not ready: {outcome:?}"),
        };
        let key = QueryKey::LinkedName(provider.clone(), "Missing".into());
        let first_fingerprint = db.ready_product_fingerprint(&key).expect("first fingerprint");
        assert_eq!(first_product.fact, LinkedNameFact::Absent);
        assert!(db.index().dependencies_of(&key).is_some_and(|edges| {
            edges.iter().any(|edge| edge.dependency == QueryKey::LinkedInterface(provider.clone()))
        }));

        db.begin_revision();
        let second_product = match query_linked_name(&mut db, provider, "Missing".into(), &second, &declarations, &prelude) {
            QueryOutcome::Ready(product) => product,
            outcome => panic!("second name query was not ready: {outcome:?}"),
        };
        let second_fingerprint = db.ready_product_fingerprint(&key).expect("second fingerprint");

        assert_ne!(first_fingerprint, second_fingerprint);
        assert!(matches!(second_product.fact, LinkedNameFact::ImportedBinding(_)));
    }

    #[test]
    fn resolved_import_input_tracks_canonical_resolution_product() {
        let importer = module("consumer");
        let site = phalcom_modules::identity::ImportSiteId::new(importer, phalcom_modules::identity::ImportSiteLocalId::new(0));
        let first = resolved_import(site.clone(), module("a1"));
        let second = resolved_import(site.clone(), module("a2"));
        let mut db = SemanticDb::new();

        assert!(matches!(query_resolved_import(&mut db, site.clone(), first), QueryOutcome::Ready(_)));
        let key = QueryKey::ResolvedImport(site.clone());
        let first_fingerprint = db.ready_product_fingerprint(&key).expect("first fingerprint");
        db.begin_revision();
        assert!(matches!(query_resolved_import(&mut db, site, second), QueryOutcome::Ready(_)));
        let second_fingerprint = db.ready_product_fingerprint(&key).expect("second fingerprint");

        assert_ne!(first_fingerprint, second_fingerprint);
    }
}
