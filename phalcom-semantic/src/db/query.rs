//! Semantic database high-level query execution and caching (Spec 04.5 / Wave 5).

use crate::checker::analysis::{CallableAnalysis, CallableAnalysisStatus};
use crate::checker::body::{analyze_callable_body, signature_consumed_by_body};
use crate::db::{DependencyEdge, SemanticDb, SemanticProduct};
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::db::key::{InputFingerprint, ProductFingerprint, QueryKey};
use crate::db::product::DeclarationSurfaceProduct;
use crate::db::state::{QueryOutcome, QueryState};
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable};
use crate::diagnostic::SemanticDiagnostic;
use crate::dispatch::{CallableSignature as SurfaceCallableSignature, SurfaceDispatchResolver};
use crate::hierarchy_product::HierarchyEdgeProduct;
use crate::identity::{CallableId, DeclarationId, ModuleId};
use crate::module_product::ResolvedImportsProduct;
use crate::signature::CallableSemanticSignature;
use crate::source::ParsedModuleUnit;
use crate::surface::DeclarationSurface;
use crate::types::annotation::TypeResolver;
use crate::types::evidence::UnknownReason;
use crate::types::outcome::BlockReason;
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::{ClassDef, RestMode, Statement};
use phalcom_common::range::SourceRange;
use phalcom_modules::interface::{InterfaceBuilder, LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::linker::LinkedProgram;
use std::collections::BTreeMap;
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
}

fn semantic_dependency_query_key(dependency: &crate::checker::analysis::SemanticDependency) -> QueryKey {
    match dependency {
        crate::checker::analysis::SemanticDependency::DeclarationShell(declaration) => {
            QueryKey::DeclarationShell(declaration.clone())
        }
        crate::checker::analysis::SemanticDependency::CallableSignature(callable) => {
            QueryKey::CallableSignature(callable.clone())
        }
        crate::checker::analysis::SemanticDependency::DeclarationSurface(declaration) => {
            QueryKey::DeclarationSurface(declaration.clone())
        }
        crate::checker::analysis::SemanticDependency::HierarchyEdge(declaration) => {
            QueryKey::HierarchyEdge(declaration.clone())
        }
        crate::checker::analysis::SemanticDependency::LinkedInterface(module) => {
            QueryKey::LinkedInterface(module.clone())
        }
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

pub(crate) fn semantic_signature_from_surface(
    callable: &CallableId,
    signature: &SurfaceCallableSignature,
) -> Option<CallableSemanticSignature> {
    if !signature.has_complete_types() {
        return None;
    }

    let parameters = signature
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            let ty = parameter.ty.ty().expect("complete signature parameter has canonical type");
            let mut semantic = crate::signature::CallableParameterSemantic::new(
                index as u32,
                parameter.local_name.clone(),
                ty.into(),
            );
            if let Some(label) = &parameter.external_label {
                semantic = semantic.with_label(label.clone());
            }
            if parameter.rest {
                semantic = semantic.with_rest(RestMode::Positional);
            }
            semantic
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let return_type = signature
        .return_type
        .ty()
        .expect("complete signature return has canonical type");

    Some(CallableSemanticSignature {
        callable: callable.clone(),
        owner: callable.owner.clone(),
        side: callable.side,
        selector: callable.selector.clone(),
        generics: signature.generics.clone(),
        parameters,
        return_type: return_type.into(),
        source: None,
        implementation: phalcom_native_meta::ImplementationKind::Source,
        native_id: None,
        effects: phalcom_native_meta::EffectSpec::Unknown,
        raises: phalcom_native_meta::RaisesSpec::Unknown,
        flow: phalcom_native_meta::ReturnFlowSpec::Value,
        lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    })
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
    db.publish_product_ready(
        key,
        revision,
        input_fingerprint,
        product_fingerprint,
        product,
        dependencies,
    )
    .map_err(|error| {
        format!(
            "stale semantic query publication: expected revision {:?}, attempted revision {:?}",
            error.expected_revision(),
            error.actual_revision()
        )
    })
}

/// Evaluates or retrieves the cached `ParsedModuleUnit` for a given module.
pub fn query_parsed_module(
    db: &mut SemanticDb,
    module: ModuleId,
    unit: Arc<ParsedModuleUnit>,
) -> QueryOutcome<Arc<ParsedModuleUnit>> {
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
        for dependency in [
            QueryKey::UnlinkedInterface(mod_id.clone()),
            QueryKey::ResolvedImports(mod_id.clone()),
        ] {
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
            db.set_state(key, QueryState::Failed { revision: db.revision(), failure: query_err.clone() });
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
        return query_failure(
            db,
            key,
            format!("hierarchy query inputs do not belong to declaration {class_decl:?}"),
        );
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
    let input_fingerprint = crate::db::fingerprint::hierarchy_edge_input_fingerprint(
        &class_decl,
        superclass_source(&unit, class_def),
        &super_decl,
    );

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
    if let Err(error) = db.record_dependency(
        &mut recorder,
        QueryKey::LinkedInterface(class_decl.module.clone()),
    ) {
        return query_failure(db, key, error);
    }

    let product = Arc::new(HierarchyEdgeProduct::new(class_decl.clone(), super_decl));
    let product_fingerprint =
        crate::db::fingerprint::hierarchy_edge_product_fingerprint(&class_decl, &product.super_decl);
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
pub fn query_declaration_shell(
    db: &mut SemanticDb,
    info: Arc<DeclarationTypeInfo>,
) -> QueryOutcome<Arc<DeclarationTypeInfo>> {
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

/// Evaluates or computes the source member surface for one declaration.
pub fn query_declaration_surface(
    db: &mut SemanticDb,
    decl_id: DeclarationId,
    unit: Arc<ParsedModuleUnit>,
    linked_interface: Arc<LinkedModuleInterface>,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
) -> QueryOutcome<Arc<DeclarationSurface>> {
    let key = QueryKey::DeclarationSurface(decl_id.clone());
    if unit.id != decl_id.module || linked_interface.module != decl_id.module {
        return query_failure(
            db,
            key,
            format!("declaration-surface query inputs do not belong to declaration {decl_id:?}"),
        );
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
        let mut context = crate::checker::context::CheckingContext::new(
            store,
            hierarchy,
            resolver,
            declarations,
            decl_id.module.clone(),
        );
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

/// Evaluates or projects the canonical semantic signature for one callable.
pub fn query_callable_signature(
    db: &mut SemanticDb,
    callable: CallableId,
) -> QueryOutcome<Arc<CallableSemanticSignature>> {
    let key = QueryKey::CallableSignature(callable.clone());
    let surface_key = QueryKey::DeclarationSurface(callable.owner.clone());

    if db
        .query_state(&surface_key)
        .and_then(QueryState::validated_revision)
        != Some(db.revision())
    {
        return query_failure(
            db,
            key,
            format!("callable-signature prerequisite {surface_key:?} is not validated for the current revision"),
        );
    }
    let Some(surface) = db.product(&surface_key).and_then(|product| product.as_declaration_surface()).cloned() else {
        return query_failure(db, key, format!("callable-signature prerequisite {surface_key:?} has no typed product"));
    };
    let Some(source_signature) = surface.get_callable(callable.side, &callable.selector) else {
        if db.query_state(&key).is_some() {
            db.discard_for_recompute(&key);
        }
        return query_failure(db, key, format!("callable {:?} is absent from its declaration surface", callable));
    };
    let Some(signature) = semantic_signature_from_surface(&callable, source_signature) else {
        if db.query_state(&key).is_some() {
            db.discard_for_recompute(&key);
        }
        return query_blocked(
            db,
            key,
            BlockReason::UnknownType(UnknownReason::UnannotatedDeclaration),
        );
    };
    let signature = Arc::new(signature);
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

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    if let Err(error) = db.record_dependency(&mut recorder, surface_key) {
        return query_failure(db, key, error);
    }

    let product_fingerprint = crate::db::fingerprint::callable_signature_product_fingerprint(&signature);
    let dependencies = recorder.finish();
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::CallableSignature(signature.clone()),
        dependencies,
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(signature)
}

fn ensure_declaration_shell(
    db: &mut SemanticDb,
    declaration: &DeclarationId,
    declarations: &DeclarationTypeTable,
) -> QueryOutcome<Arc<DeclarationTypeInfo>> {
    let Some(info) = declarations.get(declaration).cloned() else {
        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
    };
    query_declaration_shell(db, Arc::new(info))
}

fn ensure_linked_interface(
    db: &mut SemanticDb,
    module: &ModuleId,
    linked: &LinkedProgram,
) -> QueryOutcome<Arc<LinkedModuleInterface>> {
    let Some(linked_module) = linked.modules.get(module) else {
        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
    };
    query_linked_interface(db, module.clone(), Arc::new(linked_module.interface.clone()))
}

fn ensure_declaration_surface(
    db: &mut SemanticDb,
    declaration: &DeclarationId,
    formal_inputs: &FormalQueryInputs<'_>,
    store: &mut TypeStore,
) -> QueryOutcome<Arc<DeclarationSurface>> {
    let Some(unit) = formal_inputs.sources.get(&declaration.module).cloned() else {
        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
    };
    let Some(linked_module) = formal_inputs.linked.modules.get(&declaration.module) else {
        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
    };
    query_declaration_surface(
        db,
        declaration.clone(),
        unit,
        Arc::new(linked_module.interface.clone()),
        store,
        formal_inputs.hierarchy,
        formal_inputs.base_resolver,
        formal_inputs.declarations,
    )
}

fn ensure_callable_signature(
    db: &mut SemanticDb,
    callable: &CallableId,
    formal_inputs: &FormalQueryInputs<'_>,
    store: &mut TypeStore,
) -> QueryOutcome<Arc<CallableSemanticSignature>> {
    match ensure_declaration_shell(db, &callable.owner, formal_inputs.declarations) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }
    match ensure_linked_interface(db, &callable.owner.module, formal_inputs.linked) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }
    match ensure_declaration_surface(db, &callable.owner, formal_inputs, store) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }
    query_callable_signature(db, callable.clone())
}

/// Evaluates or retrieves the cached `LinkedModuleInterface` for a module.
pub fn query_linked_interface(
    db: &mut SemanticDb,
    module: ModuleId,
    linked_interface: Arc<LinkedModuleInterface>,
) -> QueryOutcome<Arc<LinkedModuleInterface>> {
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
pub fn query_module_diagnostics(
    db: &mut SemanticDb,
    module: ModuleId,
    diagnostics: Arc<[SemanticDiagnostic]>,
) -> QueryOutcome<Arc<[SemanticDiagnostic]>> {
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
    query_callable_body_with_formal_inputs(
        db,
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
        None,
    )
}

/// Evaluates a callable body while allowing missing formal prerequisites to be
/// evaluated from borrowed current workspace inputs.
pub fn query_callable_body_with_formal_inputs(
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
    formal_inputs: Option<&FormalQueryInputs<'_>>,
) -> QueryOutcome<Arc<CallableAnalysis>> {
    let key = QueryKey::CallableBody(callable.clone());

    let input_fingerprint = crate::db::fingerprint::callable_body_input_fingerprint(&callable, body, body_range, store);

    // Complete source signatures must be requested from their canonical query
    // product before body analysis can publish a result. Incomplete source
    // signatures intentionally remain surface-only until inference completes.
    if let Some((signature_id, signature)) = signature_consumed_by_body(dispatch, &callable) {
        if signature.has_complete_types() {
            let signature_outcome = match formal_inputs {
                Some(formal_inputs) => ensure_callable_signature(db, &signature_id, formal_inputs, store),
                None => query_callable_signature(db, signature_id),
            };
            match signature_outcome {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
                QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
                QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
            }
        }
    }

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
            let mut recorder = crate::db::DependencyRecorder::new(key.clone());
            for sem_dep in arc_analysis.semantic_dependencies.iter() {
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
