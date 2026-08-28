from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old in text:
        return text.replace(old, new, 1)
    if new in text:
        return text
    raise SystemExit(f"{label} shape changed")


# Query identity.
path = Path("phalcom-semantic/src/db/key.rs")
text = path.read_text()
text = replace_once(
    text,
    "use crate::identity::{CallableId, DeclarationId, ModuleId};\n",
    "use crate::identity::{CallableId, DeclarationId, FieldId, ModuleId};\n",
    "query key FieldId import",
)
text = replace_once(
    text,
    "    HierarchyEdge(DeclarationId),\n    CallableSignature(CallableId),\n",
    "    HierarchyEdge(DeclarationId),\n    FieldSignature(FieldId),\n    CallableSignature(CallableId),\n",
    "FieldSignature query key",
)
path.write_text(text)

# Body dependency identity.
path = Path("phalcom-semantic/src/checker/analysis.rs")
text = path.read_text()
text = replace_once(
    text,
    "    DeclarationShell(DeclarationId),\n    CallableSignature(CallableId),\n    DeclarationSurface(DeclarationId),\n",
    "    DeclarationShell(DeclarationId),\n    CallableSignature(CallableId),\n    FieldSignature(FieldId),\n    DeclarationSurface(DeclarationId),\n",
    "FieldSignature semantic dependency",
)
path.write_text(text)

# Typed DB product.
path = Path("phalcom-semantic/src/db/product.rs")
text = path.read_text()
text = replace_once(
    text,
    "use crate::signature::CallableSemanticSignature;\n",
    "use crate::signature::{CallableSemanticSignature, FieldSemanticSignature};\n",
    "field product import",
)
text = replace_once(
    text,
    "    HierarchyEdge(Arc<HierarchyEdgeProduct>),\n    CallableSignature(Arc<CallableSemanticSignature>),\n    CallableBody(Arc<CallableAnalysis>),\n",
    "    HierarchyEdge(Arc<HierarchyEdgeProduct>),\n    CallableSignature(Arc<CallableSemanticSignature>),\n    FieldSignature(Arc<FieldSemanticSignature>),\n    CallableBody(Arc<CallableAnalysis>),\n",
    "field product variant",
)
anchor = '''    pub fn as_callable_signature(&self) -> Option<&Arc<CallableSemanticSignature>> {
        match self {
            Self::CallableSignature(sig) => Some(sig),
            _ => None,
        }
    }

'''
insert = anchor + '''    pub fn as_field_signature(&self) -> Option<&Arc<FieldSemanticSignature>> {
        match self {
            Self::FieldSignature(sig) => Some(sig),
            _ => None,
        }
    }

'''
text = replace_once(text, anchor, insert, "field product accessor")
text = replace_once(
    text,
    "            Self::CallableSignature(_) => b\"callable-signature\".as_slice(),\n            Self::CallableBody(_) => b\"callable-body\".as_slice(),\n",
    "            Self::CallableSignature(_) => b\"callable-signature\".as_slice(),\n            Self::FieldSignature(_) => b\"field-signature\".as_slice(),\n            Self::CallableBody(_) => b\"callable-body\".as_slice(),\n",
    "field product erased kind",
)
path.write_text(text)

# Central field identity construction.
path = Path("phalcom-semantic/src/checker/declaration_signature.rs")
text = path.read_text()
anchor = '''pub(crate) fn semantic_field_signature_for_member(
    ctx: &mut CheckingContext<'_>,
    owner: &DeclarationId,
    member: &ClassMember,
) -> Option<FieldSemanticSignature> {
'''
insert = '''pub(crate) fn field_id_for_member(owner: &DeclarationId, member: &ClassMember) -> Option<FieldId> {
    let ClassMember::Field(field) = member else {
        return None;
    };
    Some(FieldId::new(owner.clone(), field.name.clone(), super::declaration::member_side(member)))
}

''' + anchor
text = replace_once(text, anchor, insert, "field identity helper")
text = replace_once(
    text,
    "    let side = super::declaration::member_side(member);\n",
    "    let field_id = field_id_for_member(owner, member)?;\n    let side = field_id.side;\n",
    "field semantic identity source",
)
text = replace_once(
    text,
    "    let field_id = FieldId::new(owner.clone(), field.name.clone(), side);\n    Some(FieldSemanticSignature {\n",
    "    Some(FieldSemanticSignature {\n",
    "remove duplicate field identity",
)
path.write_text(text)

# Range-free semantic field fingerprints.
path = Path("phalcom-semantic/src/db/fingerprint.rs")
text = path.read_text()
text = replace_once(
    text,
    "use crate::signature::CallableSemanticSignature;\n",
    "use crate::signature::{CallableSemanticSignature, FieldSemanticSignature};\n",
    "field fingerprint import",
)
anchor = '''fn hash_budget_report(report: &BudgetReport, hasher: &mut impl Hasher) {
'''
helper = '''fn hash_field_semantic_signature(signature: &FieldSemanticSignature, include_source: bool, hasher: &mut impl Hasher) {
    signature.field.hash(hasher);
    signature.owner.hash(hasher);
    signature.side.hash(hasher);
    signature.name.hash(hasher);
    signature.mutable.hash(hasher);
    signature.declared_type.hash(hasher);
    if include_source {
        match &signature.source {
            Some(source) => {
                1u8.hash(hasher);
                hash_source_span(source, hasher);
            }
            None => 0u8.hash(hasher),
        }
    }
}

''' + anchor
text = replace_once(text, anchor, helper, "field signature hash helper")
anchor = '''/// Computes direct input identity for a hierarchy-edge query.
'''
funcs = '''/// Computes the source-sensitive input identity of a canonical field signature.
pub fn field_signature_input_fingerprint(signature: &FieldSemanticSignature) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_field_semantic_signature(signature, true, &mut hasher);
    finish_input(hasher)
}

/// Computes the range-free semantic product identity of a canonical field signature.
pub fn field_signature_product_fingerprint(signature: &FieldSemanticSignature) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_field_semantic_signature(signature, false, &mut hasher);
    finish_product(hasher)
}

''' + anchor
text = replace_once(text, anchor, funcs, "field signature fingerprint API")
path.write_text(text)

# Exact formal reads record FieldSignature(field), not DeclarationSurface(owner).
path = Path("phalcom-semantic/src/checker/context.rs")
text = path.read_text()
old = '''    pub fn get_field(&self, decl: &DeclarationId, side: DispatchSide, name: &str) -> Option<TypeKnowledge> {
        // Structural invalidation is intentionally retained until FieldSignature
        // becomes its own query product. Type authority already belongs solely
        // to canonical declaration knowledge.
        record_declaration_surface_dependency(&self.semantic_dependencies, decl);
        let field = crate::identity::FieldId::new(decl.clone(), name, side);
        let signature = self.field_signatures?.get(&field)?;
        Some(signature.declared_type.to_knowledge())
    }

    pub(crate) fn resolve_field_contract(&self, owner: &DeclarationId, side: DispatchSide, name: &str) -> Option<(crate::identity::FieldId, TypeKnowledge)> {
        record_declaration_surface_dependency(&self.semantic_dependencies, owner);
        let field = crate::identity::FieldId::new(owner.clone(), name, side);
        let signature = self.field_signatures?.get(&field)?;
        Some((field, signature.declared_type.to_knowledge()))
    }
'''
new = '''    pub fn get_field(&self, decl: &DeclarationId, side: DispatchSide, name: &str) -> Option<TypeKnowledge> {
        let field = crate::identity::FieldId::new(decl.clone(), name, side);
        let signature = self.field_signatures?.get(&field)?;
        if is_query_owned_module(&field.owner.module) {
            self.record_semantic_dependency(SemanticDependency::FieldSignature(field));
        }
        Some(signature.declared_type.to_knowledge())
    }

    pub(crate) fn resolve_field_contract(&self, owner: &DeclarationId, side: DispatchSide, name: &str) -> Option<(crate::identity::FieldId, TypeKnowledge)> {
        let field = crate::identity::FieldId::new(owner.clone(), name, side);
        let signature = self.field_signatures?.get(&field)?;
        if is_query_owned_module(&field.owner.module) {
            self.record_semantic_dependency(SemanticDependency::FieldSignature(field.clone()));
        }
        Some((field, signature.declared_type.to_knowledge()))
    }
'''
text = replace_once(text, old, new, "exact field semantic dependency")
path.write_text(text)

# DB query ownership and prerequisite materialization.
path = Path("phalcom-semantic/src/db/query.rs")
text = path.read_text()
text = replace_once(
    text,
    "use crate::identity::{CallableId, DeclarationId, ModuleId};\n",
    "use crate::identity::{CallableId, DeclarationId, FieldId, ModuleId};\n",
    "query FieldId import",
)
text = replace_once(
    text,
    "use crate::signature::CallableSemanticSignature;\n",
    "use crate::signature::{CallableSemanticSignature, FieldSemanticSignature};\n",
    "query field signature import",
)
text = replace_once(
    text,
    "        crate::checker::analysis::SemanticDependency::CallableSignature(callable) => QueryKey::CallableSignature(callable.clone()),\n        crate::checker::analysis::SemanticDependency::DeclarationSurface(declaration) => QueryKey::DeclarationSurface(declaration.clone()),\n",
    "        crate::checker::analysis::SemanticDependency::CallableSignature(callable) => QueryKey::CallableSignature(callable.clone()),\n        crate::checker::analysis::SemanticDependency::FieldSignature(field) => QueryKey::FieldSignature(field.clone()),\n        crate::checker::analysis::SemanticDependency::DeclarationSurface(declaration) => QueryKey::DeclarationSurface(declaration.clone()),\n",
    "field dependency query mapping",
)
anchor = '''fn declaration_signature_id_for_body(callable: &CallableId, unit: &ParsedModuleUnit) -> Option<CallableId> {
'''
query_field = '''/// Evaluates or retrieves canonical declaration knowledge for one source field.
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
    let Some(member) = class_def.members.iter().find(|member| {
        crate::checker::declaration_signature::field_id_for_member(&field.owner, member).as_ref() == Some(&field)
    }) else {
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

''' + anchor
text = replace_once(text, anchor, query_field, "field signature query")
anchor = '''/// Evaluates or retrieves the cached `LinkedModuleInterface` for a module.
'''
ensure_field = '''fn ensure_field_signature(
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

''' + anchor
text = replace_once(text, anchor, ensure_field, "ensure field signature")
old = '''            for sem_dep in arc_analysis.semantic_dependencies.iter() {
                let dependency = semantic_dependency_query_key(sem_dep);
                if let Err(error) = db.record_dependency(&mut recorder, dependency) {
                    return query_failure(db, key, error);
                }
            }
'''
new = '''            for sem_dep in arc_analysis.semantic_dependencies.iter() {
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
'''
text = replace_once(text, old, new, "body field prerequisite ownership")
path.write_text(text)

# Workspace materializes the compatibility table exclusively from DB field products.
path = Path("phalcom-semantic/src/session.rs")
text = path.read_text()
text = replace_once(
    text,
    "    query_callable_signature, query_declaration_shell, query_declaration_surface, query_hierarchy_edge, query_linked_interface, query_source_formal_attachment,\n",
    "    query_callable_signature, query_declaration_shell, query_declaration_surface, query_field_signature, query_hierarchy_edge, query_linked_interface,\n    query_source_formal_attachment,\n",
    "session field query import",
)
old = '''                {
                    let mut context = CheckingContext::new(&mut self.store, &hierarchy, &resolver, &declarations, module_id.clone());
                    for member in &class_def.members {
                        if let Some(signature) = crate::checker::declaration_signature::semantic_field_signature_for_member(&mut context, &decl_id, member) {
                            field_signatures.insert(signature);
                        }
                    }
                    if !context.diagnostics.is_empty() {
                        diags_by_module.entry(module_id.clone()).or_default().extend(context.diagnostics);
                    }
                }
                // Publish declaration-owned callable signatures first. Dispatch
'''
new = '''                for member in &class_def.members {
                    let Some(field_id) = crate::checker::declaration_signature::field_id_for_member(&decl_id, member) else {
                        continue;
                    };
                    match query_field_signature(
                        &mut self.db,
                        field_id,
                        parsed_unit.clone(),
                        &mut self.store,
                        &hierarchy,
                        &resolver,
                        &declarations,
                    ) {
                        QueryOutcome::Ready(signature) => field_signatures.insert((*signature).clone()),
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    }
                }
                // Publish declaration-owned callable signatures first. Dispatch
'''
text = replace_once(text, old, new, "session DB-owned field materialization")
path.write_text(text)

# Static architecture sanity checks before invoking Rust.
key = Path("phalcom-semantic/src/db/key.rs").read_text()
product = Path("phalcom-semantic/src/db/product.rs").read_text()
query = Path("phalcom-semantic/src/db/query.rs").read_text()
analysis = Path("phalcom-semantic/src/checker/analysis.rs").read_text()
context = Path("phalcom-semantic/src/checker/context.rs").read_text()
assert "FieldSignature(FieldId)" in key
assert "FieldSignature(Arc<FieldSemanticSignature>)" in product
assert "pub fn query_field_signature" in query
assert "FieldSignature(FieldId)" in analysis
assert "record_semantic_dependency(SemanticDependency::FieldSignature" in context
