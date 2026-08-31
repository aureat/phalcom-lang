//! Semantic database high-level query execution and caching (Spec 04.5 / Wave 5).

use crate::advisory::{AdvisoryCallableSummary, AdvisoryModuleProduct};
use crate::checker::analysis::{CallableAnalysis, CallableAnalysisStatus};
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::db::key::{InputFingerprint, ProductFingerprint, QueryKey};
use crate::db::product::DeclarationSurfaceProduct;
use crate::db::state::{QueryOutcome, QueryState};
use crate::db::{DependencyEdge, SemanticDb, SemanticProduct};
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable};
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
    for dependency in [QueryKey::SourceStructure(product.module.clone())] {
        if let Err(error) = db.record_dependency(&mut recorder, dependency) {
            return query_failure(db, key, error);
        }
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
pub fn query_unlinked_interface(db: &mut SemanticDb, module: ModuleId, unit: Arc<ParsedModuleUnit>) -> QueryOutcome<Arc<UnlinkedModuleInterface>> {
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

    match InterfaceBuilder::build(module.clone(), unit.kind, &unit.program) {
        Ok(unlinked) => {
            let unlinked_arc = Arc::new(unlinked);
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
        Err(err) => {
            let query_err = format!("failed to build unlinked interface: {err:?}");
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

    let Some(class_def) = class_definition_for(&unit, &class_decl) else {
        return query_failure(db, key, format!("source declaration {class_decl:?} was not found in its parsed module"));
    };
    let super_decl = if let Some(super_ref) = class_def.superclass_ref() {
        let members = super_ref.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
        resolver.resolve_type_name(&class_decl.module, &super_ref.root, &members)
    } else {
        let object = DeclarationId::new(ModuleId::core(), "Object".into());
        (class_decl != object).then_some(object)
    };
    let input_fingerprint = crate::db::fingerprint::hierarchy_edge_input_fingerprint(&class_decl, superclass_source(&unit, class_def), &super_decl);

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
    let product_fingerprint = crate::db::fingerprint::hierarchy_edge_product_fingerprint(&class_decl, &product.super_decl);
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

/// Evaluates or retrieves canonical declaration type metadata for one declaration.
pub fn query_declaration_shell(db: &mut SemanticDb, info: Arc<DeclarationTypeInfo>) -> QueryOutcome<Arc<DeclarationTypeInfo>> {
    let key = QueryKey::DeclarationShell(info.declaration.clone());
    let input_fingerprint = crate::db::fingerprint::declaration_shell_input_fingerprint(&info);
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

    let product_fingerprint = crate::db::fingerprint::declaration_shell_product_fingerprint(&info);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::DeclarationShell(info.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(info)
}

pub fn query_enum_declaration(
    db: &mut SemanticDb,
    product: Arc<crate::db::product::EnumDeclarationProduct>,
) -> QueryOutcome<Arc<crate::db::product::EnumDeclarationProduct>> {
    let key = QueryKey::EnumDeclaration(product.info.owner.clone());
    let input_fingerprint = crate::db::fingerprint::enum_declaration_input_fingerprint(&product.info);
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
    } = query;
    let key = QueryKey::DeclarationSurface(decl_id.clone());
    if unit.id != decl_id.module || linked_interface.module != decl_id.module {
        return query_failure(db, key, format!("declaration-surface query inputs do not belong to declaration {decl_id:?}"));
    }

    let Some(class_def) = class_definition_for(&unit, &decl_id) else {
        return query_failure(db, key, format!("source declaration {decl_id:?} was not found in its parsed module"));
    };

    let Some(declaration_info) = declarations.get(&decl_id).cloned() else {
        return query_failure(db, key, format!("declaration metadata was not found for {decl_id:?}"));
    };
    let input_fingerprint = crate::db::fingerprint::declaration_surface_source_input_fingerprint(&unit, &decl_id, class_def);

    match query_declaration_shell(db, Arc::new(declaration_info)) {
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
        crate::checker::declaration::register_class_surface(&mut context, class_def);
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
    let key = QueryKey::CallableSignature(callable.clone());
    if unit.id != *callable.module() {
        return query_failure(db, key, format!("source unit does not own callable {callable:?}"));
    }

    let Some(declaration_info) = declarations.get(callable.declaration_owner()).cloned() else {
        return query_failure(db, key, format!("missing declaration metadata for {:?}", callable.owner));
    };
    match query_declaration_shell(db, Arc::new(declaration_info)) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let linked_key = QueryKey::LinkedInterface(callable.module().clone());
    if db.query_state(&linked_key).and_then(QueryState::validated_revision) != Some(db.revision()) {
        return query_failure(db, key, format!("CallableSignature prerequisite {linked_key:?} is not current"));
    }

    let Some(class_def) = class_definition_for(&unit, callable.declaration_owner()) else {
        return query_failure(db, key, format!("missing class declaration for {:?}", callable.owner));
    };
    let Some(member) = class_def.members.iter().find(|member| {
        crate::checker::declaration_signature::callable_id_for_member(callable.declaration_owner(), member).is_some_and(|candidate| candidate == callable)
    }) else {
        return query_failure(db, key, format!("missing source declaration for callable {callable:?}"));
    };

    let (signature, captured_dependencies) = {
        let mut context = crate::checker::CheckingContext::new(store, hierarchy, resolver, declarations, callable.module().clone());
        let Some(signature) = crate::checker::declaration_signature::semantic_signature_for_member(&mut context, callable.declaration_owner(), member) else {
            return query_failure(db, key, format!("source member cannot publish callable signature {callable:?}"));
        };
        (Arc::new(signature), context.semantic_dependencies_snapshot())
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
    let key = QueryKey::FieldSignature(field.clone());
    if unit.id != field.owner.module {
        return query_failure(db, key, format!("source unit does not own field {field:?}"));
    }

    let Some(declaration_info) = declarations.get(&field.owner).cloned() else {
        return query_failure(db, key, format!("missing declaration metadata for {:?}", field.owner));
    };
    match query_declaration_shell(db, Arc::new(declaration_info)) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let linked_key = QueryKey::LinkedInterface(field.owner.module.clone());
    if db.query_state(&linked_key).and_then(QueryState::validated_revision) != Some(db.revision()) {
        return query_failure(db, key, format!("FieldSignature prerequisite {linked_key:?} is not current"));
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
    let class_def = class_definition_for(unit, callable.declaration_owner())?;
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
    None
}

fn ensure_declaration_shell(db: &mut SemanticDb, declaration: &DeclarationId, declarations: &DeclarationTypeTable) -> QueryOutcome<Arc<DeclarationTypeInfo>> {
    let Some(info) = declarations.get(declaration).cloned() else {
        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
    };
    query_declaration_shell(db, Arc::new(info))
}

fn ensure_linked_interface(db: &mut SemanticDb, module: &ModuleId, linked: &LinkedProgram) -> QueryOutcome<Arc<LinkedModuleInterface>> {
    let Some(linked_module) = linked.modules.get(module) else {
        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
    };
    query_linked_interface(db, module.clone(), Arc::new(linked_module.interface.clone()))
}

fn ensure_callable_signature(
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
    query_callable_signature(
        db,
        callable.clone(),
        unit,
        store,
        formal_inputs.hierarchy,
        formal_inputs.base_resolver,
        formal_inputs.declarations,
    )
}

fn ensure_field_signature(
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
    query_field_signature(
        db,
        field.clone(),
        unit,
        store,
        formal_inputs.hierarchy,
        formal_inputs.base_resolver,
        formal_inputs.declarations,
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

    let input_fingerprint = match formal_inputs {
        Some(inputs) => crate::db::fingerprint::callable_body_input_fingerprint_with_formal_inputs(
            &callable,
            body,
            body_range,
            store,
            inputs.sources,
            inputs.linked,
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
            match ensure_callable_signature(db, &signature_id, inputs, store) {
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
                if let (crate::checker::analysis::SemanticDependency::FieldSignature(field), Some(inputs)) = (sem_dep, formal_inputs) {
                    match ensure_field_signature(db, field, inputs, store) {
                        QueryOutcome::Ready(_) => {}
                        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
                        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
                        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
                        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
                    }
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
