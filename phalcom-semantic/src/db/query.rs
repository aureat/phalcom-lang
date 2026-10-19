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
use crate::identity::{CallableId, DeclarationId, FieldId, ModuleId, TypeId};
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
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Borrowed formal inputs used when a query must evaluate a missing prerequisite.
///
/// This view owns no workspace state and constructs no parallel resolver, linker,
/// or type store. It only exposes the current session inputs to prerequisite
/// helpers.
#[derive(Clone, Copy)]
pub struct FormalQueryInputs<'a> {
    pub sources: &'a BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub source_resolution_input: InputFingerprint,
    pub linked_component_product: ProductFingerprint,
    pub linked: &'a LinkedProgram,
    pub topology: &'a phalcom_modules::topology::ModuleTopology,
    pub import_products: &'a BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub base_resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,
    pub type_aliases: &'a crate::type_alias::TypeAliasTable,
    pub field_signatures: Option<&'a crate::signature::FieldSignatureTable>,
    pub field_lifecycle: Option<&'a crate::checker::field_lifecycle::FieldLifecycleTable>,
    pub constructor_initializations: Option<&'a crate::signature::ConstructorInitializationTable>,
    pub callable_signatures: Option<&'a crate::signature::CallableSignatureTable>,
    pub data_semantics: Option<&'a crate::data_semantics::DataSemanticTable>,
    pub enum_semantics: Option<&'a crate::enum_semantics::EnumSemanticTable>,
    pub associated_families: Option<&'a crate::associated::AssociatedFamilyTable>,
    pub type_member_declarations:
        Option<&'a BTreeMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>>,
    pub conformance_semantics: Option<&'a crate::trait_dispatch::ConformanceSemanticView<'a>>,
    /// Checked module value facts used by the ordinary callable body checker.
    pub module_values: Option<crate::module_bindings::ModuleValueEnvironment<'a>>,
    pub semantic_linked: Option<&'a phalcom_modules::linker::SemanticLinkedProgram>,
}

pub struct HierarchyEdgeQueryInputs<'a> {
    pub linked_interface: Arc<LinkedModuleInterface>,
    pub resolver: &'a dyn TypeResolver,
    pub linked: &'a LinkedProgram,
    pub declarations: &'a DeclarationTypeTable,
    pub import_products: &'a BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>,
}

pub struct SignatureQueryInputs<'a> {
    pub store: &'a mut TypeStore,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
    /// Owner-relative computed Record-key facts prepared by the session demand
    /// driver before this signature query forms any canonical rows.
    pub record_key_projections: Option<&'a crate::static_key::ProjectedRecordKeySet<'a>>,
    pub declarations: &'a DeclarationTypeTable,
    pub linked: Option<&'a LinkedProgram>,
    pub type_aliases: Option<&'a crate::type_alias::TypeAliasTable>,
    pub import_products: Option<&'a BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
}

fn signature_projection_resolver<'a>(
    resolver: &'a dyn TypeResolver,
    projections: Option<&'a crate::static_key::ProjectedRecordKeySet<'a>>,
    owner: &QueryKey,
) -> Result<Option<crate::types::annotation::ProjectedRecordKeyResolver<'a>>, String> {
    let Some(projections) = projections else {
        return Ok(None);
    };
    if projections.owner() != owner {
        return Err(format!(
            "computed Record-key projections belong to {:?}, expected {owner:?}",
            projections.owner()
        ));
    }
    Ok(Some(crate::types::annotation::ProjectedRecordKeyResolver { parent: resolver, projections }))
}

fn require_current_projection_dependencies(db: &SemanticDb, projections: &crate::static_key::ProjectedRecordKeySet<'_>) -> QueryOutcome<()> {
    for dependency in projections.dependencies() {
        let key = semantic_dependency_query_key(&dependency);
        if db.query_state(&key).and_then(QueryState::validated_revision) != Some(db.revision()) {
            return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
        }
    }
    QueryOutcome::Ready(())
}

fn signature_projection_input_fingerprint(
    fingerprint: crate::db::InputFingerprint,
    projections: Option<&crate::static_key::ProjectedRecordKeySet<'_>>,
) -> crate::db::InputFingerprint {
    let Some(projections) = projections else {
        return fingerprint;
    };
    let mut hasher = DefaultHasher::new();
    fingerprint.hash(&mut hasher);
    projections.owner().hash(&mut hasher);
    for fact in projections.facts() {
        fact.use_id.hash(&mut hasher);
        format!("{:?}", fact.checked.root).hash(&mut hasher);
        format!("{:?}", fact.projection).hash(&mut hasher);
        fact.checked.static_constant_reads.hash(&mut hasher);
        fact.checked.dependencies.hash(&mut hasher);
    }
    crate::db::InputFingerprint::new(hasher.finish())
}

fn semantic_dependency_query_key(dependency: &crate::checker::analysis::SemanticDependency) -> QueryKey {
    match dependency {
        crate::checker::analysis::SemanticDependency::DeclarationShell(declaration) => QueryKey::DeclarationShell(declaration.clone()),
        crate::checker::analysis::SemanticDependency::TraitHeader(declaration) => QueryKey::TraitHeader(declaration.clone()),
        crate::checker::analysis::SemanticDependency::CallableSignature(callable) => QueryKey::CallableSignature(callable.clone()),
        crate::checker::analysis::SemanticDependency::FieldSignature(field) => QueryKey::FieldSignature(field.clone()),
        crate::checker::analysis::SemanticDependency::ConstructorInitialization(owner) => QueryKey::ConstructorInitialization(owner.clone()),
        crate::checker::analysis::SemanticDependency::DeclarationSurface(declaration) => QueryKey::DeclarationSurface(declaration.clone()),
        crate::checker::analysis::SemanticDependency::TraitSurface(declaration) => QueryKey::TraitSurface(declaration.clone()),
        crate::checker::analysis::SemanticDependency::ConformanceDispatch(impl_id) => QueryKey::ConformanceDispatch(impl_id.clone()),
        crate::checker::analysis::SemanticDependency::TraitDispatchBucket(bucket) => QueryKey::TraitDispatchBucket(bucket.clone()),
        crate::checker::analysis::SemanticDependency::GenericConformance(key) => QueryKey::GenericConformance(key.clone()),
        crate::checker::analysis::SemanticDependency::HierarchyEdge(declaration) => QueryKey::HierarchyEdge(declaration.clone()),
        crate::checker::analysis::SemanticDependency::LinkedInterface(module) => QueryKey::LinkedInterface(module.clone()),
        crate::checker::analysis::SemanticDependency::DataDeclaration(declaration) => QueryKey::DataDeclaration(declaration.clone()),
        crate::checker::analysis::SemanticDependency::EnumDeclaration(declaration) => QueryKey::EnumDeclaration(declaration.clone()),
        crate::checker::analysis::SemanticDependency::AssociatedSurface(declaration) => QueryKey::AssociatedSurface(declaration.clone()),
        crate::checker::analysis::SemanticDependency::TypeMemberDeclaration(declaration) => QueryKey::TypeMemberDeclaration(declaration.clone()),
        crate::checker::analysis::SemanticDependency::StaticDomainCaseCatalog(domain) => QueryKey::StaticDomainCaseCatalog(domain.clone()),
        crate::checker::analysis::SemanticDependency::StaticCaseSignature(case) => QueryKey::StaticCaseSignature(case.clone()),
        crate::checker::analysis::SemanticDependency::StaticDefinition(declaration) => QueryKey::StaticDefinition(declaration.clone()),
        crate::checker::analysis::SemanticDependency::ResolvedImport(site) => QueryKey::ResolvedImport(site.clone()),
        crate::checker::analysis::SemanticDependency::LinkedName(module, name) => QueryKey::LinkedName(module.clone(), name.clone()),
        crate::checker::analysis::SemanticDependency::PublicExport(module, name) => QueryKey::PublicExport(module.clone(), name.clone()),
        crate::checker::analysis::SemanticDependency::ModuleStaticBindingCatalog(module) => QueryKey::ModuleStaticBindingCatalog(module.clone()),
        crate::checker::analysis::SemanticDependency::ModuleStaticBindings(module) => QueryKey::ModuleStaticBindings(module.clone()),
        crate::checker::analysis::SemanticDependency::StaticConstant(id) => QueryKey::StaticConstant(id.clone()),
    }
}

fn semantic_dependency_from_query_key(key: &QueryKey) -> Option<crate::checker::analysis::SemanticDependency> {
    match key {
        QueryKey::DeclarationShell(declaration) => Some(crate::checker::analysis::SemanticDependency::DeclarationShell(declaration.clone())),
        QueryKey::TraitHeader(declaration) => Some(crate::checker::analysis::SemanticDependency::TraitHeader(declaration.clone())),
        QueryKey::CallableSignature(callable) => Some(crate::checker::analysis::SemanticDependency::CallableSignature(callable.clone())),
        QueryKey::FieldSignature(field) => Some(crate::checker::analysis::SemanticDependency::FieldSignature(field.clone())),
        QueryKey::ConstructorInitialization(owner) => Some(crate::checker::analysis::SemanticDependency::ConstructorInitialization(owner.clone())),
        QueryKey::DeclarationSurface(declaration) => Some(crate::checker::analysis::SemanticDependency::DeclarationSurface(declaration.clone())),
        QueryKey::TraitSurface(declaration) => Some(crate::checker::analysis::SemanticDependency::TraitSurface(declaration.clone())),
        QueryKey::ConformanceDispatch(impl_id) => Some(crate::checker::analysis::SemanticDependency::ConformanceDispatch(impl_id.clone())),
        QueryKey::TraitDispatchBucket(bucket) => Some(crate::checker::analysis::SemanticDependency::TraitDispatchBucket(bucket.clone())),
        QueryKey::GenericConformance(key) => Some(crate::checker::analysis::SemanticDependency::GenericConformance(key.clone())),
        QueryKey::HierarchyEdge(declaration) => Some(crate::checker::analysis::SemanticDependency::HierarchyEdge(declaration.clone())),
        QueryKey::LinkedInterface(module) => Some(crate::checker::analysis::SemanticDependency::LinkedInterface(module.clone())),
        QueryKey::DataDeclaration(declaration) => Some(crate::checker::analysis::SemanticDependency::DataDeclaration(declaration.clone())),
        QueryKey::EnumDeclaration(declaration) => Some(crate::checker::analysis::SemanticDependency::EnumDeclaration(declaration.clone())),
        QueryKey::AssociatedSurface(declaration) => Some(crate::checker::analysis::SemanticDependency::AssociatedSurface(declaration.clone())),
        QueryKey::TypeMemberDeclaration(declaration) => Some(crate::checker::analysis::SemanticDependency::TypeMemberDeclaration(declaration.clone())),
        QueryKey::StaticDomainCaseCatalog(domain) => Some(crate::checker::analysis::SemanticDependency::StaticDomainCaseCatalog(domain.clone())),
        QueryKey::StaticCaseSignature(case) => Some(crate::checker::analysis::SemanticDependency::StaticCaseSignature(case.clone())),
        QueryKey::StaticDefinition(declaration) => Some(crate::checker::analysis::SemanticDependency::StaticDefinition(declaration.clone())),
        QueryKey::ResolvedImport(site) => Some(crate::checker::analysis::SemanticDependency::ResolvedImport(site.clone())),
        QueryKey::LinkedName(module, name) => Some(crate::checker::analysis::SemanticDependency::LinkedName(module.clone(), name.clone())),
        QueryKey::PublicExport(module, name) => Some(crate::checker::analysis::SemanticDependency::PublicExport(module.clone(), name.clone())),
        QueryKey::ModuleStaticBindingCatalog(module) => Some(crate::checker::analysis::SemanticDependency::ModuleStaticBindingCatalog(module.clone())),
        QueryKey::ModuleStaticBindings(module) => Some(crate::checker::analysis::SemanticDependency::ModuleStaticBindings(module.clone())),
        QueryKey::StaticConstant(id) => Some(crate::checker::analysis::SemanticDependency::StaticConstant(id.clone())),
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

fn callable_source_has_computed_record_key(
    unit: &ParsedModuleUnit,
    callable: &CallableId,
    effective_definition: Option<&crate::impls::EffectiveCallableDefinition>,
) -> bool {
    if let Some(declaration_owner) = callable.try_declaration_owner()
        && let Some(class_def) = class_definition_for(unit, declaration_owner)
        && class_def.members.iter().any(|member| {
            crate::checker::declaration_signature::callable_id_for_member(declaration_owner, member).as_ref() == Some(callable)
                && crate::checker::declaration_signature::callable_type_annotations(member)
                    .into_iter()
                    .any(crate::checker::declaration_signature::annotation_has_computed_record_key)
        })
    {
        return true;
    }

    let Some(effective_definition) = effective_definition else {
        return false;
    };
    let crate::impls::CallableDefinitionOrigin::InherentImpl(impl_id) = &effective_definition.origin else {
        return false;
    };
    if impl_id.module != unit.id {
        return false;
    }
    let Some(Statement::Impl(impl_def)) = unit.program.statements.get(impl_id.local.0 as usize) else {
        return false;
    };
    let Some(member) = impl_def
        .members
        .get(effective_definition.source_member_index)
        .and_then(phalcom_ast::ast::ImplMember::behavior)
    else {
        return false;
    };
    let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(member);
    crate::checker::declaration_signature::callable_id_for_syntax(&callable.owner, syntax, callable.side).as_ref() == Some(callable)
        && crate::checker::declaration_signature::callable_syntax_type_annotations(syntax)
            .into_iter()
            .any(crate::checker::declaration_signature::annotation_has_computed_record_key)
}

fn enum_definition_for<'a>(unit: &'a ParsedModuleUnit, declaration: &DeclarationId) -> Option<&'a phalcom_ast::ast::EnumDef> {
    if unit.id != declaration.module {
        return None;
    }
    unit.program.statements.iter().find_map(|statement| match statement {
        Statement::Data(data_def) if data_def.is_sum() && data_def.name == declaration.name.as_ref() => Some(data_def),
        _ => None,
    })
}

fn data_definition_for<'a>(unit: &'a ParsedModuleUnit, declaration: &DeclarationId) -> Option<&'a phalcom_ast::ast::DataDef> {
    if unit.id != declaration.module {
        return None;
    }
    unit.program.statements.iter().find_map(|statement| match statement {
        Statement::Data(data_def) if data_def.is_product() && data_def.name == declaration.name.as_ref() => Some(data_def),
        _ => None,
    })
}

fn is_data_declaration(declaration: &DeclarationId, unit: &ParsedModuleUnit, _linked: &LinkedProgram) -> bool {
    if unit.id == declaration.module {
        data_definition_for(unit, declaration).is_some()
    } else {
        false
    }
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
    source_structure: Arc<ModuleSourceIndex>,
    callables: impl IntoIterator<Item = CallableId>,
) -> QueryOutcome<Arc<AdvisoryModuleProduct>> {
    let key = QueryKey::AdvisoryModule(product.module.clone());
    match query_source_structure(db, product.module.clone(), source_structure) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => {
            return QueryOutcome::Blocked(reason);
        }
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }
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
        QueryOutcome::Blocked(reason) => {
            return QueryOutcome::Blocked(reason);
        }
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
    inputs: HierarchyEdgeQueryInputs<'_>,
) -> QueryOutcome<Arc<HierarchyEdgeProduct>> {
    let HierarchyEdgeQueryInputs {
        linked_interface,
        resolver,
        linked,
        declarations,
        import_products,
    } = inputs;
    let key = QueryKey::HierarchyEdge(class_decl.clone());
    if unit.id != class_decl.module || linked_interface.module != class_decl.module {
        return query_failure(db, key, format!("hierarchy query inputs do not belong to declaration {class_decl:?}"));
    }

    match query_linked_interface(db, linked_interface.module.clone(), linked_interface) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => {
            return QueryOutcome::Blocked(reason);
        }
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let class_def = class_definition_for(&unit, &class_decl);
    if class_def.is_none() && enum_definition_for(&unit, &class_decl).is_none() && data_definition_for(&unit, &class_decl).is_none() {
        return query_failure(db, key, format!("source declaration {class_decl:?} was not found in its parsed module"));
    }
    let superclass_syntax = class_def.and_then(|class_def| superclass_source(&unit, class_def));
    let (super_decl, captured_dependencies) = if let Some(class_def) = class_def {
        crate::checker::context::resolve_source_class_superclass(&class_decl, class_def, resolver)
    } else {
        (
            Some(crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object)),
            BTreeSet::new(),
        )
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
    for dependency in captured_dependencies {
        match ensure_semantic_dependency_current(db, &dependency, Some(linked), declarations, None, Some(import_products)) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => {
                return QueryOutcome::Blocked(reason);
            }
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
        if let Err(error) = db.record_dependency(&mut recorder, semantic_dependency_query_key(&dependency)) {
            return query_failure(db, key, error);
        }
    }

    let is_rejected_data = if let Some(ref id) = super_decl {
        is_data_declaration(id, &unit, linked)
    } else {
        false
    };

    let (effective_super_decl, diagnostics) = if is_rejected_data {
        let id = super_decl.as_ref().unwrap();
        let range = class_def
            .and_then(|c| c.superclass.as_ref())
            .map(|sc| sc.range)
            .unwrap_or(SourceRange::new(0, 0));
        let diag = SemanticDiagnostic::data_used_as_superclass(id.clone(), range);
        (None, vec![diag])
    } else {
        (super_decl, Vec::new())
    };

    let product = Arc::new(if is_rejected_data {
        HierarchyEdgeProduct::new_rejected(class_decl.clone(), diagnostics.into_iter().next().unwrap())
    } else {
        HierarchyEdgeProduct::new(class_decl.clone(), effective_super_decl)
    });
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

/// Publishes one canonical trait header product. Trait headers are semantic
/// contract metadata and deliberately have no nominal declaration-shell edge.
pub fn query_bootstrap_trait_header(db: &mut SemanticDb, header: Arc<crate::traits::TraitHeader>) -> QueryOutcome<Arc<crate::traits::TraitHeader>> {
    let key = QueryKey::TraitHeader(header.declaration.clone());
    let input_fingerprint = crate::db::fingerprint::trait_header_input_fingerprint(&header);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|product| product.as_trait_header()) {
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
        crate::db::fingerprint::trait_header_product_fingerprint(&header),
        SemanticProduct::TraitHeader(header.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(header)
}

/// Publishes one complete body-independent trait surface and records its
/// header prerequisite. Default bodies are separate `CallableBody` products.
pub fn query_bootstrap_trait_surface(db: &mut SemanticDb, surface: Arc<crate::traits::TraitSurface>) -> QueryOutcome<Arc<crate::traits::TraitSurface>> {
    let key = QueryKey::TraitSurface(surface.declaration.clone());
    let header_key = QueryKey::TraitHeader(surface.declaration.clone());
    if db.query_state(&header_key).and_then(QueryState::validated_revision) != Some(db.revision()) {
        return query_failure(db, key, format!("trait surface prerequisite {header_key:?} is not current"));
    }
    let input_fingerprint = crate::db::fingerprint::trait_surface_input_fingerprint(&surface);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|product| product.as_trait_surface()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    if let Err(error) = db.record_dependency(&mut recorder, header_key) {
        return query_failure(db, key, error);
    }
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        crate::db::fingerprint::trait_surface_product_fingerprint(&surface),
        SemanticProduct::TraitSurface(surface.clone()),
        recorder.finish(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(surface)
}

pub fn query_data_declaration(
    db: &mut SemanticDb,
    product: Arc<crate::db::product::DataDeclarationProduct>,
) -> QueryOutcome<Arc<crate::db::product::DataDeclarationProduct>> {
    let key = QueryKey::DataDeclaration(product.info.owner.clone());
    let input_fingerprint = crate::db::fingerprint::data_declaration_input_fingerprint(&product);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|p| p.as_data_declaration()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::data_declaration_product_fingerprint(&product);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::DataDeclaration(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
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

/// Publishes one checked source type-member signature under its stable declaration identity.
pub fn query_type_member_declaration(
    db: &mut SemanticDb,
    unit: Option<&ParsedModuleUnit>,
    product: Arc<crate::db::product::TypeMemberDeclarationProduct>,
) -> QueryOutcome<Arc<crate::db::product::TypeMemberDeclarationProduct>> {
    let key = QueryKey::TypeMemberDeclaration(product.id.clone());
    let input_fingerprint = crate::db::fingerprint::type_member_declaration_input_fingerprint(unit, &product);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_type_member_declaration()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::type_member_declaration_product_fingerprint(&product);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::TypeMemberDeclaration(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Publishes the ordinary module binding handoff for static-body consumers.
pub fn query_module_static_binding_catalog(
    db: &mut SemanticDb,
    product: Arc<crate::module_bindings::ModuleStaticBindingCatalog>,
) -> QueryOutcome<Arc<crate::module_bindings::ModuleStaticBindingCatalog>> {
    let key = QueryKey::ModuleStaticBindingCatalog(product.module().clone());
    let input_fingerprint = product.input_fingerprint();
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_module_static_binding_catalog()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = product.product_fingerprint();
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::ModuleStaticBindingCatalog(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Publishes the ordinary module binding handoff for static-body consumers.
pub fn query_module_static_bindings(
    db: &mut SemanticDb,
    product: Arc<crate::module_bindings::ModuleStaticBindingIndex>,
) -> QueryOutcome<Arc<crate::module_bindings::ModuleStaticBindingIndex>> {
    let key = QueryKey::ModuleStaticBindings(product.module().clone());
    let input_fingerprint = product.input_fingerprint();
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_module_static_bindings()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = product.product_fingerprint();
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::ModuleStaticBindings(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Publishes the sole cached owner of ordinary checker facts for one static body.
/// Every recorded semantic dependency and the module binding handoff must be
/// current before the body product can be reused or published.
pub fn query_checked_static_body(
    db: &mut SemanticDb,
    product: Arc<crate::static_definition::CheckedStaticBody>,
    inputs: &FormalQueryInputs<'_>,
    store: &mut TypeStore,
) -> QueryOutcome<Arc<crate::static_definition::CheckedStaticBody>> {
    let key = QueryKey::CheckedStaticBody(product.owner.clone());
    let input_fingerprint = crate::db::fingerprint::checked_static_body_input_fingerprint(&product);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_checked_static_body()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let mut semantic_dependencies = product.dependencies.iter().cloned().collect::<BTreeSet<_>>();
    semantic_dependencies.insert(crate::checker::analysis::SemanticDependency::ModuleStaticBindings(
        product.owner.0.module.clone(),
    ));
    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    for semantic_dependency in semantic_dependencies {
        match ensure_formal_semantic_dependency_current(db, &semantic_dependency, inputs, store) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
        let dependency_key = semantic_dependency_query_key(&semantic_dependency);
        if dependency_key == key {
            return query_failure(db, key, "checked static body captured a self-dependency");
        }
        if let crate::checker::analysis::SemanticDependency::ConformanceDispatch(impl_id) = &semantic_dependency {
            let Some(fingerprint) = inputs.conformance_semantics.and_then(|view| view.fingerprint_for(impl_id)) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            recorder.record(dependency_key, fingerprint);
        } else if let crate::checker::analysis::SemanticDependency::TraitDispatchBucket(bucket_key) = &semantic_dependency {
            let Some(fingerprint) = inputs.conformance_semantics.map(|view| view.bucket_fingerprint(bucket_key)) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            recorder.record(dependency_key, fingerprint);
        } else if let Err(error) = db.record_dependency(&mut recorder, dependency_key) {
            return query_failure(db, key, error);
        }
    }
    let dependencies = recorder.finish();
    let product_fingerprint = crate::db::fingerprint::checked_static_body_product_fingerprint(&product);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::CheckedStaticBody(product.clone()),
        dependencies,
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Returns a current checked-body product when its exact source and dependency
/// fingerprints remain valid. Session builders call this before constructing a
/// checker context so valid bodies avoid being analyzed again.
pub fn cached_checked_static_body(
    db: &mut SemanticDb,
    owner: &crate::static_definition::StaticBodyId,
    input_fingerprint: InputFingerprint,
) -> Option<Arc<crate::static_definition::CheckedStaticBody>> {
    let key = QueryKey::CheckedStaticBody(owner.clone());
    if !db.validate_reuse(&key, input_fingerprint) {
        return None;
    }
    let product = db.product(&key).and_then(|value| value.as_checked_static_body()).cloned();
    if product.is_some() {
        db.metrics().record_hit();
    }
    product
}

/// Returns a current static constant product when its exact source and dependency
/// fingerprints remain valid.
pub fn cached_static_constant(
    db: &mut SemanticDb,
    id: &crate::static_constant::StaticConstantId,
    input_fingerprint: InputFingerprint,
) -> Option<Arc<crate::static_constant::StaticConstantProduct>> {
    let key = QueryKey::StaticConstant(id.clone());
    if !db.validate_reuse(&key, input_fingerprint) {
        return None;
    }
    let product = db.product(&key).and_then(|value| value.as_static_constant()).cloned();
    if product.is_some() {
        db.metrics().record_hit();
    }
    product
}

/// Publishes one checked static constant product under its stable identity.
/// Every recorded semantic dependency must be current before the constant product
/// can be reused or published.
pub fn query_static_constant(
    db: &mut SemanticDb,
    product: Arc<crate::static_constant::StaticConstantProduct>,
    value_store: &crate::static_value::StaticValueStore,
    inputs: &FormalQueryInputs<'_>,
    type_store: &mut TypeStore,
) -> QueryOutcome<Arc<crate::static_constant::StaticConstantProduct>> {
    match &product.outcome {
        crate::static_eval::StaticEvalOutcome::Cancelled => return QueryOutcome::Cancelled,
        crate::static_eval::StaticEvalOutcome::BudgetExceeded(report) => {
            return QueryOutcome::BudgetExceeded(report.clone());
        }
        _ => {}
    }
    let key = QueryKey::StaticConstant(product.id.clone());
    let input_fingerprint = product.checked.as_ref().map_or(crate::db::key::InputFingerprint(0), |c| c.input_fingerprint);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_static_constant()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    if !matches!(product.outcome, crate::static_eval::StaticEvalOutcome::Recursive(_)) {
        if let Some(checked) = &product.checked {
            for semantic_dependency in checked.dependencies.iter() {
                match ensure_formal_semantic_dependency_current(db, semantic_dependency, inputs, type_store) {
                    QueryOutcome::Ready(()) => {}
                    QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
                    QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
                    QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
                    QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
                }
                let dependency_key = semantic_dependency_query_key(semantic_dependency);
                if dependency_key == key {
                    return query_failure(db, key, "static constant captured a self-dependency");
                }
                if let crate::checker::analysis::SemanticDependency::ConformanceDispatch(impl_id) = semantic_dependency {
                    let Some(fingerprint) = inputs.conformance_semantics.and_then(|view| view.fingerprint_for(impl_id)) else {
                        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
                    };
                    recorder.record(dependency_key, fingerprint);
                } else if let crate::checker::analysis::SemanticDependency::TraitDispatchBucket(bucket_key) = semantic_dependency {
                    let Some(fingerprint) = inputs.conformance_semantics.map(|view| view.bucket_fingerprint(bucket_key)) else {
                        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
                    };
                    recorder.record(dependency_key, fingerprint);
                } else if let Err(error) = db.record_dependency(&mut recorder, dependency_key) {
                    return query_failure(db, key, error);
                }
            }
        }
    } else {
        if let Some(checked) = &product.checked {
            for semantic_dependency in checked.dependencies.iter() {
                let dependency_key = semantic_dependency_query_key(semantic_dependency);
                if dependency_key == key {
                    continue;
                }
                if db.query_state(&dependency_key).and_then(QueryState::validated_revision) == Some(db.revision()) {
                    let _ = db.record_dependency(&mut recorder, dependency_key);
                }
            }
        }
    }
    let dependencies = recorder.finish();
    let product_fingerprint = match crate::db::fingerprint::static_constant_product_fingerprint(&product, value_store, type_store, &inputs.linked.universe) {
        Ok(fingerprint) => fingerprint,
        Err(error) => return query_failure(db, key, error),
    };
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::StaticConstant(product.clone()),
        dependencies,
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Publishes one checked static definition product under its declaration identity.
pub fn query_static_definition(
    db: &mut SemanticDb,
    product: Arc<crate::static_definition::StaticDefinitionProduct>,
) -> QueryOutcome<Arc<crate::static_definition::StaticDefinitionProduct>> {
    let key = QueryKey::StaticDefinition(product.definition.id.clone());
    let input_fingerprint = crate::db::fingerprint::static_definition_input_fingerprint(&product);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_static_definition()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::static_definition_product_fingerprint(&product);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::StaticDefinition(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Publishes a type-alias static definition formed with owner-relative
/// computed Record-key facts and records the exact semantic products selected
/// while checking those keys.
pub fn query_static_definition_with_record_key_projections(
    db: &mut SemanticDb,
    product: Arc<crate::static_definition::StaticDefinitionProduct>,
    projections: &crate::static_key::ProjectedRecordKeySet<'_>,
    alias_dependencies: &[crate::identity::DeclarationId],
    conformance_semantics: Option<&crate::trait_dispatch::ConformanceSemanticView<'_>>,
) -> QueryOutcome<Arc<crate::static_definition::StaticDefinitionProduct>> {
    let key = QueryKey::StaticDefinition(product.definition.id.clone());
    if projections.owner() != &key {
        return query_failure(
            db,
            key.clone(),
            format!("computed Record-key projections belong to {:?}, expected {key:?}", projections.owner()),
        );
    }
    for declaration in alias_dependencies {
        let dependency_key = QueryKey::StaticDefinition(declaration.clone());
        if dependency_key == key {
            return query_failure(db, key, "static definition projection captured a self-dependency");
        }
        if db.query_state(&dependency_key).and_then(QueryState::validated_revision) != Some(db.revision()) {
            return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
        }
    }
    for dependency in projections.dependencies() {
        let dependency_key = semantic_dependency_query_key(&dependency);
        if dependency_key == key {
            return query_failure(db, key, "static definition key projection captured a self-dependency");
        }
        if let crate::checker::analysis::SemanticDependency::ConformanceDispatch(impl_id) = dependency {
            if conformance_semantics.and_then(|view| view.fingerprint_for(&impl_id)).is_none() {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            }
        } else if let crate::checker::analysis::SemanticDependency::TraitDispatchBucket(_) = dependency {
            if conformance_semantics.is_none() {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            }
        } else if db.query_state(&dependency_key).and_then(QueryState::validated_revision) != Some(db.revision()) {
            return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
        }
    }

    let input_fingerprint = signature_projection_input_fingerprint(crate::db::fingerprint::static_definition_input_fingerprint(&product), Some(projections));
    if db.validate_reuse(&key, input_fingerprint)
        && let Some(cached) = db.product(&key).and_then(|value| value.as_static_definition())
    {
        db.metrics().record_hit();
        return QueryOutcome::Ready(cached.clone());
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    for declaration in alias_dependencies {
        if let Err(error) = db.record_dependency(&mut recorder, QueryKey::StaticDefinition(declaration.clone())) {
            return query_failure(db, key, error);
        }
    }
    for dependency in projections.dependencies() {
        let dependency_key = semantic_dependency_query_key(&dependency);
        if let crate::checker::analysis::SemanticDependency::ConformanceDispatch(impl_id) = dependency {
            let Some(fingerprint) = conformance_semantics.and_then(|view| view.fingerprint_for(&impl_id)) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            recorder.record(dependency_key, fingerprint);
        } else if let crate::checker::analysis::SemanticDependency::TraitDispatchBucket(bucket_key) = dependency {
            let Some(fingerprint) = conformance_semantics.map(|view| view.bucket_fingerprint(&bucket_key)) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            recorder.record(dependency_key, fingerprint);
        } else if let Err(error) = db.record_dependency(&mut recorder, dependency_key) {
            return query_failure(db, key, error);
        }
    }
    let product_fingerprint = crate::db::fingerprint::static_definition_product_fingerprint(&product);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::StaticDefinition(product.clone()),
        recorder.finish(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Publishes source-structural case membership before payload signature checks.
pub fn query_static_domain_case_catalog(
    db: &mut SemanticDb,
    product: Arc<crate::static_case::StaticDomainCaseCatalogProduct>,
) -> QueryOutcome<Arc<crate::static_case::StaticDomainCaseCatalogProduct>> {
    let key = QueryKey::StaticDomainCaseCatalog(product.domain.clone());
    let input_fingerprint = crate::db::fingerprint::static_domain_case_catalog_input_fingerprint(&product);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_static_domain_case_catalog()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::static_domain_case_catalog_product_fingerprint(&product);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::StaticDomainCaseCatalog(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Publishes one checked static-case constructor contract.
pub fn query_static_case_signature(
    db: &mut SemanticDb,
    unit: &ParsedModuleUnit,
    product: Arc<crate::static_case::StaticCaseSignatureProduct>,
) -> QueryOutcome<Arc<crate::static_case::StaticCaseSignatureProduct>> {
    let key = QueryKey::StaticCaseSignature(product.id.clone());
    let input_fingerprint = crate::db::fingerprint::static_case_signature_input_fingerprint(unit, &product);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_static_case_signature()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::static_case_signature_product_fingerprint(&product);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::StaticCaseSignature(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

/// Publishes the checked case set for one associated static-domain declaration.
pub fn query_associated_static_domain_declaration(
    db: &mut SemanticDb,
    unit: &ParsedModuleUnit,
    product: Arc<crate::db::product::AssociatedStaticDomainDeclarationProduct>,
    store: &TypeStore,
) -> QueryOutcome<Arc<crate::db::product::AssociatedStaticDomainDeclarationProduct>> {
    let key = QueryKey::AssociatedStaticDomainDeclaration(product.id.clone());
    let input_fingerprint = crate::db::fingerprint::associated_static_domain_declaration_input_fingerprint(unit, &product, store);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|value| value.as_associated_static_domain_declaration()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::associated_static_domain_declaration_product_fingerprint(&product, store);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::AssociatedStaticDomainDeclaration(product.clone()),
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
    let input_fingerprint = crate::db::fingerprint::associated_surface_input_fingerprint(&surface);
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
    let recorder = crate::db::DependencyRecorder::new(key.clone());
    let product_fingerprint = crate::db::fingerprint::associated_surface_product_fingerprint(&surface);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::AssociatedSurface(surface.clone()),
        recorder.finish(),
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
    pub field_signatures: Option<&'a crate::signature::FieldSignatureTable>,
    pub callable_signatures: Option<&'a crate::signature::CallableSignatureTable>,
    pub type_aliases: Option<&'a crate::type_alias::TypeAliasTable>,
    pub linked: Option<&'a LinkedProgram>,
    pub import_products: Option<&'a BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
}

fn unavailable_computed_signature_owner(
    db: &SemanticDb,
    declaration: &DeclarationId,
    class_def: &ClassDef,
    field_signatures: Option<&crate::signature::FieldSignatureTable>,
    callable_signatures: Option<&crate::signature::CallableSignatureTable>,
) -> Option<QueryKey> {
    for member in &class_def.members {
        let owner = match member {
            phalcom_ast::ast::ClassMember::Field(field)
                if field
                    .annotation
                    .as_ref()
                    .is_some_and(crate::checker::declaration_signature::annotation_has_computed_record_key) =>
            {
                let field = crate::checker::declaration_signature::field_id_for_member(declaration, member)?;
                let key = QueryKey::FieldSignature(field.clone());
                let published = field_signatures.and_then(|signatures| signatures.get(&field)).is_some();
                Some((published, key))
            }
            _ if crate::checker::declaration_signature::callable_id_for_member(declaration, member).is_some()
                && crate::checker::declaration_signature::callable_type_annotations(member)
                    .into_iter()
                    .any(crate::checker::declaration_signature::annotation_has_computed_record_key) =>
            {
                let callable = crate::checker::declaration_signature::callable_id_for_member(declaration, member)?;
                let published = callable_signatures.and_then(|signatures| signatures.get(&callable)).is_some();
                Some((published, QueryKey::CallableSignature(callable)))
            }
            _ => None,
        };
        let Some((present_in_table, key)) = owner else {
            continue;
        };
        let current_in_db = db.query_state(&key).and_then(QueryState::validated_revision) == Some(db.revision());
        if !present_in_table || !current_in_db {
            return Some(key);
        }
    }
    None
}

fn declaration_signature_owner_diagnostics(
    db: &SemanticDb,
    declaration: &DeclarationId,
    class_def: &ClassDef,
    field_signatures: Option<&crate::signature::FieldSignatureTable>,
    callable_signatures: Option<&crate::signature::CallableSignatureTable>,
) -> Result<Vec<SemanticDiagnostic>, QueryKey> {
    let mut diagnostics = Vec::new();
    for member in &class_def.members {
        let owner = match member {
            phalcom_ast::ast::ClassMember::Field(_) => field_signatures.and_then(|signatures| {
                crate::checker::declaration_signature::field_id_for_member(declaration, member)
                    .filter(|field| signatures.get(field).is_some())
                    .map(|field| QueryKey::FieldSignature(field.clone()))
            }),
            _ => callable_signatures.and_then(|signatures| {
                crate::checker::declaration_signature::callable_id_for_member(declaration, member)
                    .filter(|callable| signatures.get(callable).is_some())
                    .map(QueryKey::CallableSignature)
            }),
        };
        let Some(key) = owner else {
            continue;
        };
        if db.query_state(&key).and_then(QueryState::validated_revision) != Some(db.revision()) {
            return Err(key);
        }
        let owner_diagnostics = db.product(&key).and_then(|product| match &key {
            QueryKey::FieldSignature(_) => product.as_field_signature_diagnostics(),
            QueryKey::CallableSignature(_) => product.as_callable_signature_diagnostics(),
            _ => None,
        });
        let Some(owner_diagnostics) = owner_diagnostics else {
            return Err(key);
        };
        for diagnostic in owner_diagnostics.iter() {
            if !diagnostics.contains(diagnostic) {
                diagnostics.push(diagnostic.clone());
            }
        }
    }
    Ok(diagnostics)
}

fn ensure_semantic_dependency_current(
    db: &mut SemanticDb,
    dependency: &crate::checker::analysis::SemanticDependency,
    linked: Option<&LinkedProgram>,
    declarations: &DeclarationTypeTable,
    type_aliases: Option<&crate::type_alias::TypeAliasTable>,
    import_products: Option<&BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
) -> QueryOutcome<()> {
    match dependency {
        crate::checker::analysis::SemanticDependency::DeclarationShell(declaration) => {
            let key = QueryKey::DeclarationShell(declaration.clone());
            if db.query_state(&key).and_then(QueryState::validated_revision) == Some(db.revision()) {
                return QueryOutcome::Ready(());
            }
            let shell = declarations
                .get(declaration)
                .cloned()
                .map(TypeDeclarationShell::Nominal)
                .or_else(|| type_aliases.and_then(|aliases| aliases.get(declaration).cloned().map(TypeDeclarationShell::Alias)));
            let Some(shell) = shell else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            match query_declaration_shell(db, Arc::new(shell)) {
                QueryOutcome::Ready(_) => QueryOutcome::Ready(()),
                QueryOutcome::Cancelled => QueryOutcome::Cancelled,
                QueryOutcome::BudgetExceeded(report) => QueryOutcome::BudgetExceeded(report),
                QueryOutcome::Blocked(reason) => QueryOutcome::Blocked(reason),
                QueryOutcome::Failed(failure) => QueryOutcome::Failed(failure),
            }
        }
        crate::checker::analysis::SemanticDependency::TraitHeader(declaration) => {
            let key = QueryKey::TraitHeader(declaration.clone());
            if db.query_state(&key).and_then(QueryState::validated_revision) == Some(db.revision())
                && db.product(&key).and_then(|product| product.as_trait_header()).is_some()
            {
                QueryOutcome::Ready(())
            } else {
                QueryOutcome::Blocked(BlockReason::SuppressedDependency)
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
            match query_linked_name(db, mod_id.clone(), name.clone(), linked, declarations, &prelude) {
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
        crate::checker::analysis::SemanticDependency::ConstructorInitialization(_) => QueryOutcome::Blocked(BlockReason::SuppressedDependency),
        _ => QueryOutcome::Ready(()),
    }
}

fn unit_outcome<T>(outcome: QueryOutcome<T>) -> QueryOutcome<()> {
    match outcome {
        QueryOutcome::Ready(_) => QueryOutcome::Ready(()),
        QueryOutcome::Cancelled => QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => QueryOutcome::Failed(failure),
    }
}

fn ensure_dependency_state_current(db: &SemanticDb, key: &QueryKey) -> QueryOutcome<()> {
    if db.query_state(key).and_then(QueryState::validated_revision) == Some(db.revision()) {
        QueryOutcome::Ready(())
    } else {
        QueryOutcome::Failed(format!(
            "semantic dependency {key:?} was not current after replay in revision {:?}",
            db.revision()
        ))
    }
}

/// Replays one formal semantic dependency using the complete current workspace
/// inputs. Every typed dependency has one owner query here, so body refresh,
/// cached-body validation, and publication all share the same currentness rule.
pub(crate) fn ensure_formal_semantic_dependency_current(
    db: &mut SemanticDb,
    dependency: &crate::checker::analysis::SemanticDependency,
    inputs: &FormalQueryInputs<'_>,
    store: &mut TypeStore,
) -> QueryOutcome<()> {
    let key = semantic_dependency_query_key(dependency);
    if db.query_state(&key).and_then(QueryState::validated_revision) == Some(db.revision()) {
        if let crate::checker::analysis::SemanticDependency::TraitDispatchBucket(bucket_key) = dependency {
            let Some(view) = inputs.conformance_semantics else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            if db.ready_product_fingerprint(&key) == Some(view.bucket_fingerprint(bucket_key)) {
                return QueryOutcome::Ready(());
            }
        } else {
            return QueryOutcome::Ready(());
        }
    }

    let outcome = match dependency {
        crate::checker::analysis::SemanticDependency::DeclarationShell(_) => unit_outcome(ensure_semantic_dependency_current(
            db,
            dependency,
            Some(inputs.linked),
            inputs.declarations,
            Some(inputs.type_aliases),
            Some(inputs.import_products),
        )),
        crate::checker::analysis::SemanticDependency::CallableSignature(callable) => {
            if !inputs.sources.contains_key(callable.module()) && callable.module().project == phalcom_modules::ProjectIdentity::Universe {
                return QueryOutcome::Ready(());
            }
            let Some(unit) = inputs.sources.get(callable.module()).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            unit_outcome(query_callable_signature_with_inputs(
                db,
                callable.clone(),
                unit,
                SignatureQueryInputs {
                    store,
                    hierarchy: inputs.hierarchy,
                    resolver: inputs.base_resolver,
                    record_key_projections: None,
                    declarations: inputs.declarations,
                    linked: Some(inputs.linked),
                    type_aliases: Some(inputs.type_aliases),
                    import_products: Some(inputs.import_products),
                },
            ))
        }
        crate::checker::analysis::SemanticDependency::FieldSignature(field) => {
            if !inputs.sources.contains_key(&field.owner.module) && field.owner.module.project == phalcom_modules::ProjectIdentity::Universe {
                return QueryOutcome::Ready(());
            }
            let Some(unit) = inputs.sources.get(&field.owner.module).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            unit_outcome(query_field_signature_with_inputs(
                db,
                field.clone(),
                unit,
                SignatureQueryInputs {
                    store,
                    hierarchy: inputs.hierarchy,
                    resolver: inputs.base_resolver,
                    record_key_projections: None,
                    declarations: inputs.declarations,
                    linked: Some(inputs.linked),
                    type_aliases: Some(inputs.type_aliases),
                    import_products: Some(inputs.import_products),
                },
            ))
        }
        crate::checker::analysis::SemanticDependency::ConstructorInitialization(owner) => {
            let Some(table) = inputs.constructor_initializations else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let paths = table
                .iter()
                .filter(|(factory, _)| factory.owner == *owner)
                .map(|(_, product)| product.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice();
            unit_outcome(query_constructor_initialization_owner(
                db,
                Arc::new(crate::db::product::ConstructorInitializationOwnerProduct { owner: owner.clone(), paths }),
            ))
        }
        crate::checker::analysis::SemanticDependency::DeclarationSurface(declaration) => {
            if !inputs.sources.contains_key(&declaration.module) && declaration.module.project == phalcom_modules::ProjectIdentity::Universe {
                return QueryOutcome::Ready(());
            }
            let Some(unit) = inputs.sources.get(&declaration.module).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let Some(linked_module) = inputs.linked.modules.get(&declaration.module) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            unit_outcome(query_declaration_surface(
                db,
                DeclarationSurfaceQuery {
                    decl_id: declaration.clone(),
                    unit,
                    linked_interface: Arc::new(linked_module.interface.clone()),
                    store,
                    hierarchy: inputs.hierarchy,
                    resolver: inputs.base_resolver,
                    declarations: inputs.declarations,
                    field_signatures: None,
                    callable_signatures: inputs.callable_signatures,
                    type_aliases: Some(inputs.type_aliases),
                    linked: Some(inputs.linked),
                    import_products: Some(inputs.import_products),
                },
            ))
        }
        crate::checker::analysis::SemanticDependency::TraitHeader(_declaration) => {
            if db.product(&key).and_then(|product| product.as_trait_header()).is_some() {
                QueryOutcome::Ready(())
            } else {
                QueryOutcome::Blocked(BlockReason::SuppressedDependency)
            }
        }
        crate::checker::analysis::SemanticDependency::TraitSurface(_) => {
            if db.product(&key).and_then(|product| product.as_trait_surface()).is_some() {
                QueryOutcome::Ready(())
            } else {
                QueryOutcome::Blocked(BlockReason::SuppressedDependency)
            }
        }
        crate::checker::analysis::SemanticDependency::ConformanceDispatch(impl_id) => {
            let Some(view) = inputs.conformance_semantics else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let Some(fingerprint) = view.fingerprint_for(impl_id) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            if db.query_state(&key).is_some() {
                if db.ready_product_fingerprint(&key) == Some(fingerprint) && db.validate_ready(&key) {
                    QueryOutcome::Ready(())
                } else {
                    QueryOutcome::Blocked(BlockReason::SuppressedDependency)
                }
            } else if view.derived_conformance_rules.get_rule(impl_id).is_some() {
                // Guarded-rule views still use their existing non-query
                // publication path. Source witness plans are DB-visible
                // derived products and must have a current query row.
                QueryOutcome::Ready(())
            } else {
                QueryOutcome::Blocked(BlockReason::SuppressedDependency)
            }
        }
        crate::checker::analysis::SemanticDependency::TraitDispatchBucket(bucket_key) => {
            let Some(view) = inputs.conformance_semantics else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            refresh_trait_dispatch_bucket_root(db, bucket_key, view.bucket_fingerprint(bucket_key))
        }
        crate::checker::analysis::SemanticDependency::GenericConformance(conformance_key) => {
            let Some(view) = inputs.conformance_semantics else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            refresh_generic_conformance_root(db, conformance_key, view, inputs.declarations, inputs.hierarchy, store)
        }
        crate::checker::analysis::SemanticDependency::HierarchyEdge(declaration) => {
            if !inputs.sources.contains_key(&declaration.module) && declaration.module.project == phalcom_modules::ProjectIdentity::Universe {
                return QueryOutcome::Ready(());
            }
            let Some(unit) = inputs.sources.get(&declaration.module).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let Some(linked_module) = inputs.linked.modules.get(&declaration.module) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            unit_outcome(query_hierarchy_edge(
                db,
                declaration.clone(),
                unit,
                HierarchyEdgeQueryInputs {
                    linked_interface: Arc::new(linked_module.interface.clone()),
                    resolver: inputs.base_resolver,
                    linked: inputs.linked,
                    declarations: inputs.declarations,
                    import_products: inputs.import_products,
                },
            ))
        }
        crate::checker::analysis::SemanticDependency::LinkedInterface(_)
        | crate::checker::analysis::SemanticDependency::LinkedName(_, _)
        | crate::checker::analysis::SemanticDependency::PublicExport(_, _)
        | crate::checker::analysis::SemanticDependency::ResolvedImport(_) => unit_outcome(ensure_semantic_dependency_current(
            db,
            dependency,
            Some(inputs.linked),
            inputs.declarations,
            Some(inputs.type_aliases),
            Some(inputs.import_products),
        )),
        crate::checker::analysis::SemanticDependency::DataDeclaration(declaration) => {
            let Some(table) = inputs.data_semantics else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let Some(info) = table.get(declaration).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            unit_outcome(query_data_declaration(
                db,
                Arc::new(crate::db::product::DataDeclarationProduct {
                    info,
                    diagnostics: Arc::from(Vec::<SemanticDiagnostic>::new().into_boxed_slice()),
                }),
            ))
        }
        crate::checker::analysis::SemanticDependency::EnumDeclaration(declaration) => {
            let Some(table) = inputs.enum_semantics else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let Some(info) = table.enums.get(declaration).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let variants = info
                .variants
                .iter()
                .filter_map(|variant| table.variants.get(variant).cloned())
                .collect::<Vec<_>>();
            unit_outcome(query_enum_declaration(
                db,
                Arc::new(crate::db::product::EnumDeclarationProduct {
                    info,
                    variants: Arc::from(variants.into_boxed_slice()),
                    diagnostics: Arc::from(Vec::<SemanticDiagnostic>::new().into_boxed_slice()),
                }),
            ))
        }
        crate::checker::analysis::SemanticDependency::AssociatedSurface(declaration) => {
            let Some(table) = inputs.associated_families else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let Some(surface) = table.surfaces.get(declaration).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            unit_outcome(query_associated_surface(db, surface))
        }
        crate::checker::analysis::SemanticDependency::TypeMemberDeclaration(declaration) => {
            if !inputs.sources.contains_key(&declaration.owner.module) && declaration.owner.module.project == phalcom_modules::ProjectIdentity::Universe {
                return QueryOutcome::Ready(());
            }
            let Some(table) = inputs.type_member_declarations else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let Some(product) = table.get(declaration).cloned() else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let Some(unit) = inputs.sources.get(&declaration.owner.module) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            unit_outcome(query_type_member_declaration(db, Some(unit), product))
        }
        crate::checker::analysis::SemanticDependency::StaticDomainCaseCatalog(domain) => {
            let Some(product) = inputs.base_resolver.resolve_static_case_catalog(domain) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            unit_outcome(query_static_domain_case_catalog(db, product))
        }
        crate::checker::analysis::SemanticDependency::StaticCaseSignature(case) => {
            let Some(product) = inputs.base_resolver.resolve_static_case_signature(case) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            let module = match &case.domain {
                crate::identity::StaticDomainDeclarationId::Nominal(declaration) => &declaration.module,
                crate::identity::StaticDomainDeclarationId::Associated(declaration) => &declaration.owner.module,
            };
            let Some(unit) = inputs.sources.get(module) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            unit_outcome(query_static_case_signature(db, unit, product))
        }
        crate::checker::analysis::SemanticDependency::StaticDefinition(declaration) => {
            let Some(product) = inputs.base_resolver.resolve_static_definition(declaration) else {
                return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
            };
            unit_outcome(query_static_definition(db, product))
        }
        crate::checker::analysis::SemanticDependency::ModuleStaticBindingCatalog(module) => {
            let key = QueryKey::ModuleStaticBindingCatalog(module.clone());
            if db.product(&key).and_then(|product| product.as_module_static_binding_catalog()).is_some() && db.validate_ready(&key) {
                QueryOutcome::Ready(())
            } else {
                QueryOutcome::Blocked(BlockReason::SuppressedDependency)
            }
        }
        crate::checker::analysis::SemanticDependency::ModuleStaticBindings(module) => {
            let key = QueryKey::ModuleStaticBindings(module.clone());
            if db.product(&key).and_then(|product| product.as_module_static_bindings()).is_some() && db.validate_ready(&key) {
                QueryOutcome::Ready(())
            } else {
                QueryOutcome::Blocked(BlockReason::SuppressedDependency)
            }
        }
        crate::checker::analysis::SemanticDependency::StaticConstant(id) => {
            let key = QueryKey::StaticConstant(id.clone());
            if db.product(&key).and_then(|product| product.as_static_constant()).is_some() && db.validate_ready(&key) {
                QueryOutcome::Ready(())
            } else {
                QueryOutcome::Blocked(BlockReason::SuppressedDependency)
            }
        }
    };

    // Guarded-rule ConformanceDispatch remains a borrowed semantic view; source
    // witness-plan dispatch has a DB query-state row published by the session.
    if matches!(dependency, crate::checker::analysis::SemanticDependency::ConformanceDispatch(_)) {
        if db.query_state(&key).is_some() {
            return ensure_dependency_state_current(db, &key);
        }
        return outcome;
    }

    match outcome {
        QueryOutcome::Ready(()) => ensure_dependency_state_current(db, &key),
        other => other,
    }
}

/// Publishes or revalidates the canonical fingerprint owner for one dispatch
/// bucket. Publishing a changed root preserves incoming client edges, allowing
/// each dependent body to compare the fingerprint it observed and recompute
/// only when that exact bucket changed.
pub(crate) fn refresh_trait_dispatch_bucket_root(
    db: &mut SemanticDb,
    bucket_key: &crate::trait_dispatch::TraitDispatchBucketKey,
    fingerprint: crate::db::ProductFingerprint,
) -> QueryOutcome<()> {
    let key = QueryKey::TraitDispatchBucket(bucket_key.clone());
    if db.ready_product_fingerprint(&key) == Some(fingerprint) && db.validate_ready(&key) {
        return QueryOutcome::Ready(());
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bucket_key.hash(&mut hasher);
    let input_fingerprint = InputFingerprint::new(hasher.finish());
    if let Err(error) = db.publish_ready(
        key.clone(),
        db.revision(),
        input_fingerprint,
        fingerprint,
        crate::db::state::QueryValue::from_bytes([]),
        Vec::new(),
    ) {
        return query_failure(db, key, format!("{error:?}"));
    }
    QueryOutcome::Ready(())
}

fn generic_conformance_input_fingerprint(
    key: &crate::generic_evidence::GenericConformanceKey,
    view: &crate::trait_dispatch::ConformanceSemanticView<'_>,
    store: &mut TypeStore,
) -> InputFingerprint {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);

    match view.trait_surfaces.get(&key.trait_ref.declaration) {
        Some(surface) => {
            1u8.hash(&mut hasher);
            crate::db::fingerprint::trait_surface_product_fingerprint(surface).raw().hash(&mut hasher);
        }
        None => 0u8.hash(&mut hasher),
    }

    // Hash the exact matching source-head slice, including each instantiated
    // target/TraitRef and impl substitution. This makes a newly added exact
    // candidate visible even when the previous proof was negative and had no
    // selected ConformanceDispatch edge.
    if let crate::types::parameter::TypeTerm::Canonical(target) = key.subject {
        let matches = view.conformance_index.query_exact(store, target, &key.trait_ref);
        matches.len().hash(&mut hasher);
        for head in matches {
            head.impl_id.hash(&mut hasher);
            head.exact_target.hash(&mut hasher);
            head.exact_trait_ref.hash(&mut hasher);
            let mut bindings = head.impl_bindings.into_iter().collect::<Vec<_>>();
            bindings.sort_unstable_by_key(|(parameter, _)| *parameter);
            bindings.hash(&mut hasher);
            view.fingerprint_for(&head.impl_id).hash(&mut hasher);
            if let Some(contribution) = view.conformance_index.get(&head.impl_id) {
                contribution.target_head.hash(&mut hasher);
                contribution.trait_ref.hash(&mut hasher);
                contribution.authorized.hash(&mut hasher);
                contribution.eligible.hash(&mut hasher);
            }
        }
    } else {
        0usize.hash(&mut hasher);
    }

    // The proof engine tests only rules for this trait, then substitutes the
    // exact target into each candidate. Hashing this trait-local candidate
    // superset is conservative and also covers a negative-to-positive change.
    let derived_rules = view.derived_conformance_rules.rules_for_trait(&key.trait_ref.declaration);
    derived_rules.len().hash(&mut hasher);
    for rule in derived_rules {
        rule.fingerprint.hash(&mut hasher);
    }

    InputFingerprint::new(hasher.finish())
}

fn generic_conformance_proof_fingerprint(
    key: &crate::generic_evidence::GenericConformanceKey,
    proof: &crate::generic_evidence::GenericConformanceProof,
) -> ProductFingerprint {
    use crate::generic_evidence::GenericConformanceProof as Proof;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    match proof {
        Proof::Proven(evidence) => {
            0u8.hash(&mut hasher);
            crate::db::fingerprint::hash_generic_conformance_evidence(evidence, &mut hasher);
        }
        Proof::Missing => 1u8.hash(&mut hasher),
        Proof::Ambiguous(evidences) => {
            2u8.hash(&mut hasher);
            evidences.len().hash(&mut hasher);
            for evidence in evidences.iter() {
                crate::db::fingerprint::hash_generic_conformance_evidence(evidence, &mut hasher);
            }
        }
        Proof::Incomplete(impl_id) => {
            3u8.hash(&mut hasher);
            impl_id.hash(&mut hasher);
        }
        Proof::CoherenceConflict(impl_ids) => {
            4u8.hash(&mut hasher);
            impl_ids.hash(&mut hasher);
        }
        Proof::Unknown(reason) => {
            5u8.hash(&mut hasher);
            format!("{reason:?}").hash(&mut hasher);
        }
        Proof::Blocked(reason) => {
            6u8.hash(&mut hasher);
            format!("{reason:?}").hash(&mut hasher);
        }
        Proof::Dynamic(obligation) => {
            7u8.hash(&mut hasher);
            format!("{obligation:?}").hash(&mut hasher);
        }
        Proof::Cancelled => 8u8.hash(&mut hasher),
        Proof::BudgetExceeded(report) => {
            9u8.hash(&mut hasher);
            format!("{report:?}").hash(&mut hasher);
        }
        Proof::InternalFailure(message) => {
            10u8.hash(&mut hasher);
            message.hash(&mut hasher);
        }
    }
    ProductFingerprint::new(hasher.finish())
}

/// Re-evaluates the globally cached, context-independent proof for one exact
/// proposition against current session inputs. Incoming client edges remain
/// attached when the proof row changes so clients compare result fingerprints
/// and only recompute when their observed proof actually changed.
pub(crate) fn refresh_generic_conformance_root(
    db: &mut SemanticDb,
    key: &crate::generic_evidence::GenericConformanceKey,
    view: &crate::trait_dispatch::ConformanceSemanticView<'_>,
    declarations: &DeclarationTypeTable,
    hierarchy: &dyn TypeHierarchy,
    store: &mut TypeStore,
) -> QueryOutcome<()> {
    let query_key = QueryKey::GenericConformance(key.clone());
    let input_fingerprint = generic_conformance_input_fingerprint(key, view, store);
    let mut proof_context = crate::generic_evidence::ConformanceProofContext::new(
        None,
        Some(view.conformance_index),
        Some(view.witness_plans),
        Some(view.trait_surfaces),
        Some(declarations),
        Some(hierarchy),
        Some(store),
    )
    .with_derived_rules(Some(view.derived_conformance_rules));
    let proof = crate::generic_evidence::prove_conformance_with_context(key, &mut proof_context);
    let proof_fingerprint = generic_conformance_proof_fingerprint(key, &proof);

    if db.validate_reuse(&query_key, input_fingerprint) && db.ready_product_fingerprint(&query_key) == Some(proof_fingerprint) {
        return QueryOutcome::Ready(());
    }

    let product = Arc::new(crate::db::product::GenericConformanceQueryProduct { key: key.clone(), proof });
    if let Err(error) = publish_current_product(
        db,
        query_key.clone(),
        input_fingerprint,
        proof_fingerprint,
        SemanticProduct::GenericConformance(product),
        Vec::new(),
    ) {
        return query_failure(db, query_key, format!("{error:?}"));
    }
    QueryOutcome::Ready(())
}

/// Replays every semantic dependency recorded by a cached formal query.
pub(crate) fn ensure_cached_formal_semantic_dependencies_current(
    db: &mut SemanticDb,
    key: &QueryKey,
    inputs: &FormalQueryInputs<'_>,
    store: &mut TypeStore,
) -> QueryOutcome<()> {
    let dependencies = db.index().dependencies_of(key).map(|edges| edges.to_vec()).unwrap_or_default();
    for edge in dependencies {
        let Some(dependency) = semantic_dependency_from_query_key(&edge.dependency) else {
            continue;
        };
        match ensure_formal_semantic_dependency_current(db, &dependency, inputs, store) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
    }
    QueryOutcome::Ready(())
}

fn ensure_cached_semantic_dependencies_current(
    db: &mut SemanticDb,
    key: &QueryKey,
    linked: Option<&LinkedProgram>,
    declarations: &DeclarationTypeTable,
    type_aliases: Option<&crate::type_alias::TypeAliasTable>,
    import_products: Option<&BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>>,
) -> QueryOutcome<()> {
    let dependencies = db.index().dependencies_of(key).map(|edges| edges.to_vec()).unwrap_or_default();
    for edge in dependencies {
        let Some(dependency) = semantic_dependency_from_query_key(&edge.dependency) else {
            continue;
        };
        match ensure_semantic_dependency_current(db, &dependency, linked, declarations, type_aliases, import_products) {
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
        field_signatures,
        callable_signatures,
        type_aliases,
        linked,
        import_products,
    } = query;
    let key = QueryKey::DeclarationSurface(decl_id.clone());
    if unit.id != decl_id.module || linked_interface.module != decl_id.module {
        return query_failure(db, key, format!("declaration-surface query inputs do not belong to declaration {decl_id:?}"));
    }

    let class_def = class_definition_for(&unit, &decl_id);
    let enum_def = enum_definition_for(&unit, &decl_id);
    let data_def = data_definition_for(&unit, &decl_id);
    if class_def.is_none() && enum_def.is_none() && data_def.is_none() {
        return query_failure(db, key, format!("source declaration {decl_id:?} was not found in its parsed module"));
    }

    let Some(declaration_info) = declarations.get(&decl_id).cloned() else {
        return query_failure(db, key, format!("declaration metadata was not found for {decl_id:?}"));
    };
    if let Some(class_def) = class_def
        && let Some(owner) = unavailable_computed_signature_owner(db, &decl_id, class_def, field_signatures, callable_signatures)
    {
        return QueryOutcome::Blocked(BlockReason::UnavailableSemanticOwner(Box::new(owner)));
    }
    let (input_fingerprint, signature_diagnostics) = if let Some(class_def) = class_def {
        let signature_diagnostics = match declaration_signature_owner_diagnostics(db, &decl_id, class_def, field_signatures, callable_signatures) {
            Ok(diagnostics) => diagnostics,
            Err(owner) => return QueryOutcome::Blocked(BlockReason::UnavailableSemanticOwner(Box::new(owner))),
        };
        let input_fingerprint = crate::db::fingerprint::declaration_surface_source_input_fingerprint(&unit, &decl_id, class_def);
        (
            crate::db::fingerprint::input_fingerprint_with_diagnostics(input_fingerprint, &signature_diagnostics),
            signature_diagnostics,
        )
    } else if let Some(enum_def) = enum_def {
        (
            crate::db::fingerprint::declaration_surface_enum_input_fingerprint(&unit, &decl_id, enum_def),
            Vec::new(),
        )
    } else {
        let data_def = data_def.expect("data declaration checked above");
        let mut hasher = DefaultHasher::new();
        2_u8.hash(&mut hasher);
        decl_id.hash(&mut hasher);
        data_def.range.start.hash(&mut hasher);
        data_def.range.end.hash(&mut hasher);
        unit.text.get(data_def.range.start..data_def.range.end).hash(&mut hasher);
        (InputFingerprint::new(hasher.finish()), Vec::new())
    };

    match query_declaration_shell(db, Arc::new(TypeDeclarationShell::Nominal(declaration_info))) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => {
            return QueryOutcome::Blocked(reason);
        }
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    match query_linked_interface(db, linked_interface.module.clone(), linked_interface) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    match ensure_cached_semantic_dependencies_current(db, &key, linked, declarations, type_aliases, import_products) {
        QueryOutcome::Ready(()) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => {
            return QueryOutcome::Blocked(reason);
        }
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
    let (computed_surface, mut computed_diagnostics, captured_dependencies) = {
        let mut context = crate::checker::context::CheckingContext::new(store, hierarchy, resolver, declarations, decl_id.module.clone());
        if let Some(field_signatures) = field_signatures {
            context.attach_field_signatures(field_signatures);
        }
        if let Some(callable_signatures) = callable_signatures {
            context.attach_callable_signatures(callable_signatures);
        }
        if let Some(class_def) = class_def {
            crate::checker::declaration::register_class_surface(&mut context, class_def);
        } else {
            // Enum and data roots own no class-style primary behavior surface
            // here. Enum behavior and inherent impl behavior are composed by
            // their dedicated semantic passes, while data components remain
            // first-class data witnesses rather than fabricated getters.
            context.register_surface(decl_id.clone(), DeclarationSurface::new(Some(decl_id.clone())));
        }
        let computed_surface = context.dispatch_ref().get_surface(&decl_id).cloned();
        let captured_dependencies = context.semantic_dependencies_snapshot();
        let diagnostics = context.diagnostics;
        (computed_surface, diagnostics, captured_dependencies)
    };
    let Some(computed_surface) = computed_surface else {
        return query_failure(db, key, format!("declaration-surface query did not publish {decl_id:?}"));
    };
    for diagnostic in signature_diagnostics {
        if computed_diagnostics.contains(&diagnostic) {
            continue;
        }
        let insertion = computed_diagnostics
            .iter()
            .position(|existing| existing.primary_range.start > diagnostic.primary_range.start)
            .unwrap_or(computed_diagnostics.len());
        computed_diagnostics.insert(insertion, diagnostic);
    }
    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    let mut semantic_dependencies = std::collections::BTreeSet::new();
    semantic_dependencies.insert(crate::checker::analysis::SemanticDependency::DeclarationShell(decl_id.clone()));
    semantic_dependencies.extend(captured_dependencies);
    for dependency in semantic_dependencies {
        match ensure_semantic_dependency_current(db, &dependency, linked, declarations, type_aliases, import_products) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => {
                return QueryOutcome::Blocked(reason);
            }
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
    let product = Arc::new(DeclarationSurfaceProduct::new(
        surface.clone(),
        Arc::from(computed_diagnostics.into_boxed_slice()),
    ));
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
        SemanticProduct::CallableSignature(Arc::new(crate::db::product::CallableSignatureProduct::new(signature.clone(), Arc::from([])))),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(signature)
}

/// Publishes the single conflict-admitted source definition for a callable.
pub fn query_callable_definition(
    db: &mut SemanticDb,
    definition: Arc<crate::impls::EffectiveCallableDefinition>,
) -> QueryOutcome<Arc<crate::impls::EffectiveCallableDefinition>> {
    let key = QueryKey::CallableDefinition(definition.callable.clone());
    let signature_fingerprint = crate::db::fingerprint::callable_signature_product_fingerprint(&definition.signature);
    let origin_tag = match &definition.origin {
        crate::impls::CallableDefinitionOrigin::PrimaryDeclaration => 0_u64,
        crate::impls::CallableDefinitionOrigin::InherentImpl(impl_id) => {
            let mut hasher = DefaultHasher::new();
            impl_id.hash(&mut hasher);
            hasher.finish()
        }
        crate::impls::CallableDefinitionOrigin::ConformanceWitness(impl_id) => {
            let mut hasher = DefaultHasher::new();
            impl_id.hash(&mut hasher);
            hasher.finish().rotate_left(7)
        }
    };
    let body_context_tag = definition
        .inherent_body_context
        .as_ref()
        .map_or(0, crate::db::fingerprint::inherent_body_context_fingerprint);
    let body_provenance_tag = crate::db::fingerprint::callable_definition_body_provenance_fingerprint(&definition.body_provenance);
    let value = signature_fingerprint.raw()
        ^ origin_tag.rotate_left(17)
        ^ (definition.source_member_index as u64).rotate_left(33)
        ^ body_context_tag.rotate_left(11)
        ^ body_provenance_tag.rotate_left(23);
    let input_fingerprint = crate::db::InputFingerprint::new(value);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(product) = db.product(&key).and_then(|product| product.as_callable_definition()) {
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
        crate::db::ProductFingerprint::new(value),
        SemanticProduct::CallableDefinition(definition.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(definition)
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
    query_callable_signature_with_inputs(
        db,
        callable,
        unit,
        SignatureQueryInputs {
            store,
            hierarchy,
            resolver,
            record_key_projections: None,
            declarations,
            linked: None,
            type_aliases: None,
            import_products: None,
        },
    )
}

pub fn query_callable_signature_with_inputs(
    db: &mut SemanticDb,
    callable: CallableId,
    unit: Arc<ParsedModuleUnit>,
    inputs: SignatureQueryInputs<'_>,
) -> QueryOutcome<Arc<CallableSemanticSignature>> {
    let SignatureQueryInputs {
        store,
        hierarchy,
        resolver,
        record_key_projections,
        declarations,
        linked,
        type_aliases,
        import_products,
    } = inputs;
    let key = QueryKey::CallableSignature(callable.clone());
    let effective_definition = db
        .product(&QueryKey::CallableDefinition(callable.clone()))
        .and_then(|product| product.as_callable_definition())
        .cloned();
    if db.query_state(&key).and_then(QueryState::validated_revision) != Some(db.revision())
        && record_key_projections.is_none()
        && callable_source_has_computed_record_key(&unit, &callable, effective_definition.as_deref())
    {
        return QueryOutcome::Blocked(BlockReason::UnavailableSemanticOwner(Box::new(key)));
    }
    let projected_resolver = match signature_projection_resolver(resolver, record_key_projections, &key) {
        Ok(projected_resolver) => projected_resolver,
        Err(error) => return query_failure(db, key, error),
    };
    if let Some(projections) = record_key_projections {
        match require_current_projection_dependencies(db, projections) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Failed(error) => return QueryOutcome::Failed(error),
        }
    }
    let resolver = projected_resolver.as_ref().map_or(resolver, |projected| projected as &dyn TypeResolver);
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
        QueryOutcome::Blocked(reason) => {
            return QueryOutcome::Blocked(reason);
        }
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let linked_key = QueryKey::LinkedInterface(callable.module().clone());
    if let Some(linked) = linked {
        match ensure_linked_interface(db, callable.module(), linked) {
            QueryOutcome::Ready(_) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => {
                return QueryOutcome::Blocked(reason);
            }
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
    }
    if db.query_state(&linked_key).and_then(QueryState::validated_revision) != Some(db.revision()) {
        return query_failure(db, key, format!("CallableSignature prerequisite {linked_key:?} is not current"));
    }

    match ensure_cached_semantic_dependencies_current(db, &key, linked, declarations, type_aliases, import_products) {
        QueryOutcome::Ready(()) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let uses_effective_definition = effective_definition.is_some();
    let (signature, mut captured_dependencies, signature_diagnostics) = if let Some(definition) = effective_definition {
        (Arc::new(definition.signature.clone()), BTreeSet::new(), Arc::from([]))
    } else if let Some(class_def) = class_definition_for(&unit, callable.declaration_owner()) {
        if let Some(member) = class_def.members.iter().find(|member| {
            crate::checker::declaration_signature::callable_id_for_member(callable.declaration_owner(), member).is_some_and(|candidate| candidate == callable)
        }) {
            let mut context = crate::checker::CheckingContext::new(store, hierarchy, resolver, declarations, callable.module().clone());
            let Some(signature) = crate::checker::declaration_signature::semantic_signature_for_member(&mut context, callable.declaration_owner(), member)
            else {
                return query_failure(db, key, format!("source member cannot publish callable signature {callable:?}"));
            };
            let dependencies = context.semantic_dependencies_snapshot();
            let diagnostics = Arc::<[SemanticDiagnostic]>::from(context.diagnostics.into_boxed_slice());
            (Arc::new(signature), dependencies, diagnostics)
        } else {
            return query_failure(db, key, format!("missing effective definition for callable {callable:?}"));
        }
    } else {
        return query_failure(db, key, format!("missing effective definition for callable {callable:?}"));
    };

    if let Some(projections) = record_key_projections {
        captured_dependencies.extend(projections.dependencies());
    }

    let input_fingerprint = crate::db::fingerprint::input_fingerprint_with_diagnostics(
        signature_projection_input_fingerprint(crate::db::fingerprint::callable_signature_input_fingerprint(&signature), record_key_projections),
        &signature_diagnostics,
    );
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
        match ensure_semantic_dependency_current(db, dependency, linked, declarations, type_aliases, import_products) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => {
                return QueryOutcome::Blocked(reason);
            }
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
    }

    let mut dependency_keys = BTreeSet::from([QueryKey::DeclarationShell(callable.declaration_owner().clone())]);
    if uses_effective_definition {
        dependency_keys.insert(QueryKey::CallableDefinition(callable.clone()));
    }
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
        SemanticProduct::CallableSignature(Arc::new(crate::db::product::CallableSignatureProduct::new(
            signature.clone(),
            signature_diagnostics,
        ))),
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
    query_field_signature_with_inputs(
        db,
        field,
        unit,
        SignatureQueryInputs {
            store,
            hierarchy,
            resolver,
            record_key_projections: None,
            declarations,
            linked: None,
            type_aliases: None,
            import_products: None,
        },
    )
}

/// Publishes the constructor paths contributed by one class owner.
///
/// Empty path sets are meaningful: they revoke a previously published
/// lifecycle and invalidate dependent explicit-super resolutions.
pub fn query_constructor_initialization_owner(
    db: &mut SemanticDb,
    product: Arc<crate::db::product::ConstructorInitializationOwnerProduct>,
) -> QueryOutcome<Arc<crate::db::product::ConstructorInitializationOwnerProduct>> {
    let key = QueryKey::ConstructorInitialization(product.owner.clone());
    let input_fingerprint = crate::db::fingerprint::constructor_initialization_owner_input_fingerprint(&product);
    if db.validate_reuse(&key, input_fingerprint) {
        if let Some(cached) = db.product(&key).and_then(|product| product.as_constructor_initialization()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(cached.clone());
        }
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();
    let product_fingerprint = crate::db::fingerprint::constructor_initialization_owner_product_fingerprint(&product);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::ConstructorInitialization(product.clone()),
        Vec::new(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(product)
}

pub fn query_field_signature_with_inputs(
    db: &mut SemanticDb,
    field: FieldId,
    unit: Arc<ParsedModuleUnit>,
    inputs: SignatureQueryInputs<'_>,
) -> QueryOutcome<Arc<FieldSemanticSignature>> {
    let SignatureQueryInputs {
        store,
        hierarchy,
        resolver,
        record_key_projections,
        declarations,
        linked,
        type_aliases,
        import_products,
    } = inputs;
    let key = QueryKey::FieldSignature(field.clone());
    let projected_resolver = match signature_projection_resolver(resolver, record_key_projections, &key) {
        Ok(projected_resolver) => projected_resolver,
        Err(error) => return query_failure(db, key, error),
    };
    if let Some(projections) = record_key_projections {
        match require_current_projection_dependencies(db, projections) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Failed(error) => return QueryOutcome::Failed(error),
        }
    }
    let resolver = projected_resolver.as_ref().map_or(resolver, |projected| projected as &dyn TypeResolver);
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

    match ensure_cached_semantic_dependencies_current(db, &key, linked, declarations, type_aliases, import_products) {
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
        .find(|member| crate::checker::declaration_signature::field_id_for_member(&field.owner, member).is_some_and(|candidate| candidate == field))
    else {
        return query_failure(db, key, format!("missing source declaration for field {field:?}"));
    };
    if db.query_state(&key).and_then(QueryState::validated_revision) != Some(db.revision())
        && record_key_projections.is_none()
        && let phalcom_ast::ast::ClassMember::Field(field_member) = member
        && field_member
            .annotation
            .as_ref()
            .is_some_and(crate::checker::declaration_signature::annotation_has_computed_record_key)
    {
        return QueryOutcome::Blocked(BlockReason::UnavailableSemanticOwner(Box::new(key)));
    }

    let mut context = crate::checker::CheckingContext::new(store, hierarchy, resolver, declarations, field.owner.module.clone());
    let Some(signature) = crate::checker::declaration_signature::semantic_field_signature_for_member(&mut context, &field.owner, member) else {
        return query_failure(db, key, format!("source member cannot publish field signature {field:?}"));
    };

    let mut captured_dependencies = context.semantic_dependencies_snapshot();
    let signature_diagnostics = Arc::<[SemanticDiagnostic]>::from(context.diagnostics.into_boxed_slice());
    let input_fingerprint = crate::db::fingerprint::input_fingerprint_with_diagnostics(
        signature_projection_input_fingerprint(crate::db::fingerprint::field_signature_input_fingerprint(&signature), record_key_projections),
        &signature_diagnostics,
    );
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

    if let Some(projections) = record_key_projections {
        captured_dependencies.extend(projections.dependencies());
    }
    for dependency in &captured_dependencies {
        match ensure_semantic_dependency_current(db, dependency, linked, declarations, type_aliases, import_products) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
    }

    let mut dependency_keys = BTreeSet::from([QueryKey::DeclarationShell(field.owner.clone())]);
    dependency_keys.extend(captured_dependencies.iter().map(semantic_dependency_query_key));
    dependency_keys.remove(&key);

    let mut recorder = crate::db::DependencyRecorder::new(key.clone());
    for dependency in dependency_keys {
        if let Err(error) = db.record_dependency(&mut recorder, dependency) {
            return query_failure(db, key, error);
        }
    }

    let product_fingerprint = crate::db::fingerprint::field_signature_product_fingerprint(&signature);
    let signature = Arc::new(signature);
    if let Err(error) = publish_current_product(
        db,
        key.clone(),
        input_fingerprint,
        product_fingerprint,
        SemanticProduct::FieldSignature(Arc::new(crate::db::product::FieldSignatureProduct::new(
            signature.clone(),
            signature_diagnostics,
        ))),
        recorder.finish(),
    ) {
        return query_failure(db, key, error);
    }
    QueryOutcome::Ready(signature)
}

pub(crate) fn declaration_signature_id_for_body(db: &SemanticDb, callable: &CallableId, unit: &ParsedModuleUnit) -> Option<CallableId> {
    if db
        .product(&QueryKey::CallableDefinition(callable.clone()))
        .and_then(|product| product.as_callable_definition())
        .is_some()
    {
        return Some(callable.clone());
    }
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
    let key = QueryKey::CallableSignature(callable.clone());
    if db.validate_ready(&key) {
        if let Some(signature) = db.product(&key).and_then(|product| product.as_callable_signature()) {
            return QueryOutcome::Ready(signature.clone());
        }
    }
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
        SignatureQueryInputs {
            store,
            hierarchy: formal_inputs.hierarchy,
            resolver: formal_inputs.base_resolver,
            record_key_projections: None,
            declarations: formal_inputs.declarations,
            linked: Some(formal_inputs.linked),
            type_aliases: Some(formal_inputs.type_aliases),
            import_products: Some(formal_inputs.import_products),
        },
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
pub fn compute_public_export_fact(module: &ModuleId, name: &str, linked: &LinkedProgram) -> crate::db::product::PublicExportFact {
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
    /// Optional canonical signature supplied by a non-nominal owner such as
    /// a trait surface. Nominal bodies resolve this from source products.
    pub declared_signature: Option<(&'a CallableId, &'a CallableSemanticSignature)>,
    /// Optional explicit owner generic metadata for non-nominal callables.
    pub owner_generic_signature: Option<&'a crate::types::parameter::GenericSignature>,
    /// Optional concrete receiver used by conformance witness body checking.
    pub receiver_value_override: Option<TypeId>,
    /// Optional type-level `Self` specialization that leaves the runtime
    /// receiver value unchanged.
    pub instance_self_override: Option<TypeId>,
    /// Optional abstract trait contract used for trait default body analysis.
    pub trait_surface: Option<&'a crate::traits::TraitSurface>,
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

/// Evaluates or retrieves one context-aware executable attribute argument.
/// The dedicated key prevents attribute evidence from replacing an owner's
/// ordinary body analysis.
pub fn query_attribute_argument(
    db: &mut SemanticDb,
    id: crate::attribute_analysis::AttributeArgumentId,
    query: CallableBodyQuery<'_>,
    attribute_context: crate::attribute_analysis::AttributeAnalysisContext,
) -> QueryOutcome<Arc<crate::attribute_analysis::AttributeArgumentAnalysis>> {
    let CallableBodyQuery {
        callable,
        body,
        body_range,
        declared_signature,
        owner_generic_signature,
        receiver_value_override,
        instance_self_override,
        trait_surface,
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
    let key = QueryKey::AttributeArgument(id.clone());
    let Some(inputs) = formal_inputs else {
        return query_failure(db, key, "attribute argument analysis requires current formal inputs");
    };
    let body_input = crate::db::fingerprint::callable_body_input_fingerprint_with_formal_inputs(
        &callable,
        body,
        body_range,
        store,
        crate::db::fingerprint::CallableBodyFormalInputFingerprint {
            sources: inputs.sources,
            source_resolution_input: inputs.source_resolution_input,
            linked_component_product: inputs.linked_component_product,
            lifecycle: inputs.field_lifecycle,
            owner_generic_signature,
            declared_signature: declared_signature.map(|(_, signature)| signature),
            instance_self_override,
            trait_surface,
            conformance_semantics: inputs.conformance_semantics,
            has_module_value_environment: inputs.module_values.is_some(),
        },
    );
    let mut input_hasher = DefaultHasher::new();
    id.hash(&mut input_hasher);
    attribute_context.hash(&mut input_hasher);
    body_input.hash(&mut input_hasher);
    let input_fingerprint = InputFingerprint::new(input_hasher.finish());

    match ensure_cached_formal_semantic_dependencies_current(db, &key, inputs, store) {
        QueryOutcome::Ready(()) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }
    if db.validate_reuse(&key, input_fingerprint)
        && let Some(cached) = db.product(&key).and_then(|product| product.as_attribute_argument())
    {
        db.metrics().record_hit();
        return QueryOutcome::Ready(cached.clone());
    }
    if db.query_state(&key).is_some() {
        db.discard_for_recompute(&key);
    }
    db.metrics().record_miss();

    let analysis = crate::checker::body::analyze_attribute_argument(
        crate::checker::body::BodyAnalysisContext {
            store,
            hierarchy,
            resolver,
            declarations,
            dispatch,
            trait_surface,
            conformance_semantics: inputs.conformance_semantics,
            module_values: inputs.module_values,
            linked_module: inputs.linked.modules.get(&module),
            semantic_linked_module: inputs.semantic_linked.and_then(|semantic| semantic.modules.get(&module)),
            module: module.clone(),
            caller_package: Some(inputs.topology.owning_package(&module)),
            access_topology: Some(inputs.topology),
        },
        crate::checker::body::CallableBodyRequest {
            callable,
            body,
            body_range,
            owner_generic_signature,
            receiver_value_override,
            instance_self_override,
            declared_signature,
            budget,
            cancel,
            field_signatures: inputs.field_signatures,
            field_lifecycle: inputs.field_lifecycle,
            constructor_initializations: inputs.constructor_initializations,
            callable_signatures: inputs.callable_signatures,
            enum_semantics: inputs.enum_semantics,
            data_semantics: inputs.data_semantics,
            associated_families: inputs.associated_families,
            conformance_semantics: inputs.conformance_semantics,
        },
        attribute_context,
    );
    let product_fingerprint = crate::db::fingerprint::callable_body_product_fingerprint(&analysis);
    let mut full_product_hasher = DefaultHasher::new();
    id.hash(&mut full_product_hasher);
    product_fingerprint.hash(&mut full_product_hasher);
    let product_fingerprint = ProductFingerprint::new(full_product_hasher.finish());
    let product = Arc::new(crate::attribute_analysis::AttributeArgumentAnalysis {
        id: id.clone(),
        analysis: Arc::new(analysis),
    });

    match product.analysis.status {
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
            let reason = BlockReason::SuppressedDependency;
            db.set_state(
                key,
                QueryState::Blocked {
                    revision: db.revision(),
                    reason: reason.clone(),
                },
            );
            QueryOutcome::Blocked(reason)
        }
        CallableAnalysisStatus::Complete | CallableAnalysisStatus::Partial | CallableAnalysisStatus::InternalFailure(_) => {
            let mut recorder = crate::db::DependencyRecorder::new(key.clone());
            for semantic_dependency in product.analysis.semantic_dependencies.iter() {
                match ensure_formal_semantic_dependency_current(db, semantic_dependency, inputs, store) {
                    QueryOutcome::Ready(()) => {}
                    QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
                    QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
                    QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
                    QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
                }
                let dependency = semantic_dependency_query_key(semantic_dependency);
                if dependency == key {
                    return query_failure(db, key, "attribute argument captured a self-dependency");
                }
                if let crate::checker::analysis::SemanticDependency::ConformanceDispatch(impl_id) = semantic_dependency {
                    let Some(fingerprint) = inputs.conformance_semantics.and_then(|view| view.fingerprint_for(impl_id)) else {
                        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
                    };
                    recorder.record(dependency, fingerprint);
                } else if let crate::checker::analysis::SemanticDependency::TraitDispatchBucket(bucket) = semantic_dependency {
                    let Some(fingerprint) = inputs.conformance_semantics.map(|view| view.bucket_fingerprint(bucket)) else {
                        return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
                    };
                    recorder.record(dependency, fingerprint);
                } else if let Err(error) = db.record_dependency(&mut recorder, dependency) {
                    return query_failure(db, key, error);
                }
            }
            if let Err(error) = publish_current_product(
                db,
                key.clone(),
                input_fingerprint,
                product_fingerprint,
                SemanticProduct::AttributeArgument(product.clone()),
                recorder.finish(),
            ) {
                return query_failure(db, key, error);
            }
            QueryOutcome::Ready(product)
        }
    }
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
        declared_signature: explicit_declared_signature,
        owner_generic_signature,
        receiver_value_override,
        instance_self_override,
        trait_surface,
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

    if std::env::var_os("PHALCOM_TRACE_SEMANTIC_BOOTSTRAP").is_some() {
        eprintln!("[semantic bootstrap trace] callable-body module={module} callable={callable:?} range={body_range:?}");
    }

    if let Some(inputs) = formal_inputs {
        match ensure_cached_formal_semantic_dependencies_current(db, &key, inputs, store) {
            QueryOutcome::Ready(()) => {}
            QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
            QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
            QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
        }
    }

    // Every source callable declaration has a canonical signature product,
    // including partially-known signatures. Constructor body identities remain
    // instance-side while consuming their class-side constructor declaration.
    let declared_signature = match explicit_declared_signature {
        Some((signature_id, signature)) => Some((signature_id.clone(), Arc::new(signature.clone()))),
        None => match formal_inputs {
            Some(inputs) => {
                let Some(unit) = inputs.sources.get(callable.module()).cloned() else {
                    return QueryOutcome::Blocked(BlockReason::SuppressedDependency);
                };
                let Some(signature_id) = declaration_signature_id_for_body(db, &callable, &unit) else {
                    return query_failure(db, key.clone(), format!("missing declaration signature identity for body {callable:?}"));
                };
                let sig_res = ensure_callable_signature_with_inputs(db, &signature_id, inputs, store);
                match sig_res {
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
        },
    };

    let declared_signature_for_fingerprint = explicit_declared_signature.map(|(_, signature)| signature);
    let input_fingerprint = match formal_inputs {
        Some(inputs) => crate::db::fingerprint::callable_body_input_fingerprint_with_formal_inputs(
            &callable,
            body,
            body_range,
            store,
            crate::db::fingerprint::CallableBodyFormalInputFingerprint {
                sources: inputs.sources,
                source_resolution_input: inputs.source_resolution_input,
                linked_component_product: inputs.linked_component_product,
                lifecycle: inputs.field_lifecycle,
                owner_generic_signature,
                declared_signature: declared_signature_for_fingerprint,
                instance_self_override,
                trait_surface,
                conformance_semantics: formal_inputs.and_then(|inputs| inputs.conformance_semantics),
                has_module_value_environment: formal_inputs.is_some_and(|inputs| inputs.module_values.is_some()),
            },
        ),
        None => crate::db::fingerprint::callable_body_input_fingerprint_with_owner_generics(&callable, body, body_range, store, owner_generic_signature),
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
    let caller_package = formal_inputs.map(|inputs| inputs.topology.owning_package(&module));
    let analysis = crate::checker::body::analyze_callable_body(
        crate::checker::body::BodyAnalysisContext {
            store,
            hierarchy,
            resolver,
            declarations,
            dispatch,
            trait_surface,
            conformance_semantics: formal_inputs.and_then(|inputs| inputs.conformance_semantics),
            module_values: formal_inputs.and_then(|inputs| inputs.module_values),
            linked_module: formal_inputs.and_then(|inputs| inputs.linked.modules.get(&module)),
            semantic_linked_module: formal_inputs.and_then(|inputs| inputs.semantic_linked.and_then(|sl| sl.modules.get(&module))),
            module,
            caller_package,
            access_topology: formal_inputs.map(|inputs| inputs.topology),
        },
        crate::checker::body::CallableBodyRequest {
            callable: callable.clone(),
            body,
            body_range,
            owner_generic_signature,
            receiver_value_override,
            instance_self_override,
            declared_signature: declared_signature.as_ref().map(|(signature_id, signature)| (signature_id, signature.as_ref())),
            budget,
            cancel,
            field_signatures: formal_inputs.and_then(|inputs| inputs.field_signatures),
            field_lifecycle: formal_inputs.and_then(|inputs| inputs.field_lifecycle),
            constructor_initializations: formal_inputs.and_then(|inputs| inputs.constructor_initializations),
            callable_signatures: formal_inputs.and_then(|inputs| inputs.callable_signatures),
            enum_semantics: formal_inputs.and_then(|inputs| inputs.enum_semantics),
            data_semantics: formal_inputs.and_then(|inputs| inputs.data_semantics),
            associated_families: formal_inputs.and_then(|inputs| inputs.associated_families),
            conformance_semantics: formal_inputs.and_then(|inputs| inputs.conformance_semantics),
        },
    );

    let mut analysis = analysis;
    if let Some(surface) = trait_surface {
        let dependency = crate::checker::analysis::SemanticDependency::TraitSurface(surface.declaration.clone());
        if !analysis.semantic_dependencies.iter().any(|existing| existing == &dependency) {
            let mut dependencies = analysis.semantic_dependencies.to_vec();
            dependencies.push(dependency);
            analysis.semantic_dependencies = Arc::from(dependencies.into_boxed_slice());
        }
    }
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
            if db.product(&QueryKey::CallableDefinition(callable.clone())).is_some()
                && let Err(error) = db.record_dependency(&mut recorder, QueryKey::CallableDefinition(callable.clone()))
            {
                return query_failure(db, key, error);
            }
            for sem_dep in arc_analysis.semantic_dependencies.iter() {
                let outcome = if let Some(inputs) = formal_inputs {
                    ensure_formal_semantic_dependency_current(db, sem_dep, inputs, store)
                } else {
                    ensure_semantic_dependency_current(db, sem_dep, None, declarations, None, None)
                };
                match &outcome {
                    QueryOutcome::Ready(()) => {}
                    QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
                    QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report.clone()),
                    QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason.clone()),
                    QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure.clone()),
                }
                let dependency = semantic_dependency_query_key(sem_dep);
                if let crate::checker::analysis::SemanticDependency::ConformanceDispatch(impl_id) = sem_dep {
                    if let Some(inputs) = formal_inputs {
                        if let Some(view) = inputs.conformance_semantics {
                            let Some(fingerprint) = view.fingerprint_for(impl_id) else {
                                return query_failure(db, key, format!("missing current conformance view for {dependency:?}"));
                            };
                            recorder.record(dependency, fingerprint);
                            continue;
                        }
                    }
                }
                if let crate::checker::analysis::SemanticDependency::TraitDispatchBucket(bucket_key) = sem_dep {
                    if let Some(inputs) = formal_inputs {
                        if let Some(view) = inputs.conformance_semantics {
                            let fingerprint = view.bucket_fingerprint(bucket_key);
                            recorder.record(dependency, fingerprint);
                            continue;
                        }
                    }
                }
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
    use phalcom_modules::error::ModuleResolutionError;
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
        assert!(
            db.index()
                .dependencies_of(&key)
                .is_some_and(|edges| { edges.iter().any(|edge| edge.dependency == QueryKey::LinkedInterface(provider.clone())) })
        );

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

        assert!(matches!(
            query_public_export(&mut db, provider.clone(), "Foo".into(), &first),
            QueryOutcome::Ready(_)
        ));
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
        assert!(
            db.index()
                .dependencies_of(&key)
                .is_some_and(|edges| { edges.iter().any(|edge| edge.dependency == QueryKey::LinkedInterface(provider.clone())) })
        );

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

    #[test]
    fn resolved_import_product_ignores_topology_evidence_when_result_is_unchanged() {
        let importer = module("consumer");
        let site = phalcom_modules::identity::ImportSiteId::new(importer, phalcom_modules::identity::ImportSiteLocalId::new(0));
        let target = module("target");
        let first = ImportResolutionProduct::new(
            site.clone(),
            ImportPathIdentity {
                written: "dep".into(),
                is_relative: true,
            },
            Arc::from(Vec::new()),
            Ok(target.clone()),
            ResolutionTopologyDependencies::default(),
        );
        let mut changed_dependencies = ResolutionTopologyDependencies::default();
        changed_dependencies.consulted_packages.insert(module("evidence"));
        let second = ImportResolutionProduct::new(
            site,
            ImportPathIdentity {
                written: "dep".into(),
                is_relative: true,
            },
            Arc::from(Vec::new()),
            Ok(target),
            changed_dependencies,
        );

        assert_ne!(
            crate::db::fingerprint::resolved_import_input_fingerprint(&first),
            crate::db::fingerprint::resolved_import_input_fingerprint(&second)
        );
        assert_eq!(
            crate::db::fingerprint::resolved_import_product_fingerprint(&first),
            crate::db::fingerprint::resolved_import_product_fingerprint(&second)
        );
    }

    #[test]
    fn resolved_import_failure_product_ignores_diagnostic_text() {
        let importer = module("consumer");
        let site = phalcom_modules::identity::ImportSiteId::new(importer, phalcom_modules::identity::ImportSiteLocalId::new(0));
        let first = ImportResolutionProduct::new(
            site.clone(),
            ImportPathIdentity {
                written: "dep".into(),
                is_relative: true,
            },
            Arc::from(Vec::new()),
            Err(ModuleResolutionError::ModuleNotFound("first spelling".into())),
            ResolutionTopologyDependencies::default(),
        );
        let second = ImportResolutionProduct::new(
            site,
            ImportPathIdentity {
                written: "dep".into(),
                is_relative: true,
            },
            Arc::from(Vec::new()),
            Err(ModuleResolutionError::ModuleNotFound("second spelling".into())),
            ResolutionTopologyDependencies::default(),
        );

        assert_ne!(
            crate::db::fingerprint::resolved_import_input_fingerprint(&first),
            crate::db::fingerprint::resolved_import_input_fingerprint(&second)
        );
        assert_eq!(
            crate::db::fingerprint::resolved_import_product_fingerprint(&first),
            crate::db::fingerprint::resolved_import_product_fingerprint(&second)
        );
    }
}
