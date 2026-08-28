from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text()


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text)


def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    if old not in text:
        raise SystemExit(f"anchor missing in {path}: {old[:120]!r}")
    write(path, text.replace(old, new, 1))


def sub_once(path: str, pattern: str, replacement: str) -> None:
    text = read(path)
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"expected one regex match in {path}, found {count}: {pattern[:120]!r}")
    write(path, updated)


# Fix the source builder's borrowed index accessor match.
replace_once(
    "phalcom-semantic/src/checker/declaration_signature.rs",
    "            let selector = match index.accessor {\n",
    "            let selector = match &index.accessor {\n",
)

# DeclarationSurface becomes a projection of canonical declaration signatures.
path = "phalcom-semantic/src/checker/declaration.rs"
text = read(path)
text = text.replace("use crate::dispatch::{CallableParameter, CallableSemanticKind, CallableSignature};\n", "")
text = text.replace("use phalcom_common::selector::{Selector, SelectorSlot};\n", "")
write(path, text)
sub_once(
    path,
    r"pub fn register_class_surface\(ctx: &mut CheckingContext<'_>, class_def: &ClassDef\) \{.*?\n\}\n\nfn member_visibility",
    '''pub fn register_class_surface(ctx: &mut CheckingContext<'_>, class_def: &ClassDef) {
    let decl_id = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    let mut surface = DeclarationSurface::new(Some(decl_id.clone()));
    let class_ty = ctx.nominal_type_of(&decl_id);
    ctx.dispatch.make_mut().register_type(class_ty, decl_id.clone());

    // Fields are still projected directly in this phase. Callable members go
    // through the declaration-owned semantic signature builder first.
    let type_params_map = if let Some(sig) = ctx.declaration_generic_signature(&decl_id) {
        sig.parameters
            .iter()
            .map(|&param_id| {
                let name = ctx.store.type_parameter(param_id).name.to_string();
                let param_form = ctx.store.parameter_form(param_id);
                (name, param_form)
            })
            .collect()
    } else {
        std::collections::HashMap::new()
    };
    let parent_resolver = ctx.resolver.clone();
    let field_resolver = crate::types::annotation::ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: type_params_map,
    };

    for member in &class_def.members {
        let visibility = member_visibility(member);
        match member {
            ClassMember::Field(field) => {
                let side = member_side(member);
                let declared = field
                    .annotation
                    .as_ref()
                    .map(|annotation| ctx.resolve_type_annotation(&field_resolver, annotation).0)
                    .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));
                surface.add_field_with_visibility(side, &field.name, declared, visibility);
            }
            ClassMember::Method(_) | ClassMember::Getter(_) | ClassMember::Setter(_) | ClassMember::Index(_) => {
                let Some(signature) = super::declaration_signature::semantic_signature_for_member(ctx, &decl_id, member) else {
                    continue;
                };
                let side = signature.side;
                let projection = super::declaration_signature::project_semantic_signature(&signature);
                surface.add_callable_with_visibility(side, projection, visibility);
            }
            ClassMember::Variant(_) => {}
        }
    }

    ctx.register_surface(decl_id, surface);
}

fn member_visibility''',
)

# DB CallableSignature is now computed from source declaration facts, not surfaces.
path = "phalcom-semantic/src/db/query.rs"
text = read(path)
text = text.replace("use crate::checker::body::signature_consumed_by_body;\n", "")
text = text.replace(
    "use crate::dispatch::{CallableSignature as SurfaceCallableSignature, SurfaceDispatchResolver};\n",
    "use crate::dispatch::SurfaceDispatchResolver;\n",
)
text = text.replace("use phalcom_ast::ast::{ClassDef, RestMode, Statement};\n", "use phalcom_ast::ast::{ClassDef, Statement};\n")
text = text.replace("use std::collections::BTreeMap;\n", "use std::collections::{BTreeMap, BTreeSet};\n")
write(path, text)
sub_once(
    path,
    r"pub\(crate\) fn semantic_signature_from_surface\(.*?\n\}\n\nfn publish_current_product",
    "fn publish_current_product",
)
sub_once(
    path,
    r"/// Evaluates or projects the canonical semantic signature for one callable\..*?\nfn ensure_declaration_shell",
    '''/// Evaluates or retrieves the canonical semantic signature for one source callable.
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
    if unit.id != callable.owner.module {
        return query_failure(db, key, format!("source unit does not own callable {callable:?}"));
    }

    let Some(declaration_info) = declarations.get(&callable.owner).cloned() else {
        return query_failure(db, key, format!("missing declaration metadata for {:?}", callable.owner));
    };
    match query_declaration_shell(db, Arc::new(declaration_info)) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }

    let linked_key = QueryKey::LinkedInterface(callable.owner.module.clone());
    if db.query_state(&linked_key).and_then(QueryState::validated_revision) != Some(db.revision()) {
        return query_failure(db, key, format!("CallableSignature prerequisite {linked_key:?} is not current"));
    }

    let Some(class_def) = class_definition_for(&unit, &callable.owner) else {
        return query_failure(db, key, format!("missing class declaration for {:?}", callable.owner));
    };
    let Some(member) = class_def.members.iter().find(|member| {
        crate::checker::declaration_signature::callable_id_for_member(&callable.owner, member)
            .is_some_and(|candidate| candidate == callable)
    }) else {
        return query_failure(db, key, format!("missing source declaration for callable {callable:?}"));
    };

    let (signature, captured_dependencies) = {
        let mut context = crate::checker::CheckingContext::new(
            store,
            hierarchy,
            resolver,
            declarations,
            callable.owner.module.clone(),
        );
        let Some(signature) = crate::checker::declaration_signature::semantic_signature_for_member(
            &mut context,
            &callable.owner,
            member,
        ) else {
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

    let mut dependency_keys = BTreeSet::from([
        QueryKey::DeclarationShell(callable.owner.clone()),
        linked_key,
    ]);
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

fn declaration_signature_id_for_body(callable: &CallableId, unit: &ParsedModuleUnit) -> Option<CallableId> {
    let class_def = class_definition_for(unit, &callable.owner)?;
    if class_def.members.iter().any(|member| {
        crate::checker::declaration_signature::callable_id_for_member(&callable.owner, member)
            .as_ref()
            == Some(callable)
    }) {
        return Some(callable.clone());
    }

    if callable.side == crate::identity::DispatchSide::Instance {
        let class_side = CallableId::new(
            callable.owner.clone(),
            callable.selector.clone(),
            crate::identity::DispatchSide::Class,
        );
        if class_def.members.iter().any(|member| {
            crate::checker::declaration_signature::callable_id_for_member(&callable.owner, member)
                .as_ref()
                == Some(&class_side)
        }) {
            return Some(class_side);
        }
    }
    None
}

fn ensure_declaration_shell''',
)
sub_once(
    path,
    r"fn ensure_declaration_surface\(.*?\n\}\n\nfn ensure_callable_signature\(.*?\n\}\n\n/// Evaluates or retrieves the cached",
    '''fn ensure_callable_signature(
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
    match ensure_linked_interface(db, &callable.owner.module, formal_inputs) {
        QueryOutcome::Ready(_) => {}
        QueryOutcome::Cancelled => return QueryOutcome::Cancelled,
        QueryOutcome::BudgetExceeded(report) => return QueryOutcome::BudgetExceeded(report),
        QueryOutcome::Blocked(reason) => return QueryOutcome::Blocked(reason),
        QueryOutcome::Failed(failure) => return QueryOutcome::Failed(failure),
    }
    let Some(unit) = formal_inputs.sources.get(&callable.owner.module).cloned() else {
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

/// Evaluates or retrieves the cached''',
)
sub_once(
    path,
    r"    // Complete source signatures must be requested from their canonical query.*?\n    // 1\. Check if already computed",
    '''    // Every source callable declaration has a canonical signature product,
    // including partially-known signatures. Constructor body identities remain
    // instance-side while consuming their class-side constructor declaration.
    let declared_signature = match formal_inputs {
        Some(inputs) => {
            let Some(unit) = inputs.sources.get(&callable.owner.module).cloned() else {
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
            let direct = db
                .product(&QueryKey::CallableSignature(callable.clone()))
                .and_then(|product| product.as_callable_signature())
                .cloned()
                .map(|signature| (callable.clone(), signature));
            direct.or_else(|| {
                (callable.side == crate::identity::DispatchSide::Instance).then(|| {
                    let signature_id = CallableId::new(
                        callable.owner.clone(),
                        callable.selector.clone(),
                        crate::identity::DispatchSide::Class,
                    );
                    db.product(&QueryKey::CallableSignature(signature_id.clone()))
                        .and_then(|product| product.as_callable_signature())
                        .cloned()
                        .map(|signature| (signature_id, signature))
                })
                .flatten()
            })
        }
    };

    // 1. Check if already computed''',
)
replace_once(
    path,
    '''        declarations,
        dispatch,
        module,
        budget,
        cancel,
        formal_inputs.and_then(|inputs| inputs.field_lifecycle),
''',
    '''        declarations,
        dispatch,
        declared_signature
            .as_ref()
            .map(|(signature_id, signature)| (signature_id, signature.as_ref())),
        module,
        budget,
        cancel,
        formal_inputs.and_then(|inputs| inputs.field_lifecycle),
''',
)

# Body checking consumes the canonical semantic signature product directly.
path = "phalcom-semantic/src/checker/body.rs"
text = read(path)
text = text.replace("use crate::dispatch::{CallableSignature, SurfaceDispatchResolver};\n", "use crate::dispatch::SurfaceDispatchResolver;\n")
write(path, text)
sub_once(
    path,
    r"/// Returns the declaration signature consumed by a body.*?\n/// Context holding canonical published semantic inputs",
    "/// Context holding canonical published semantic inputs",
)
replace_once(
    path,
    '''        declarations,
        dispatch,
        module,
        budget,
        cancel,
        None,
''',
    '''        declarations,
        dispatch,
        None,
        module,
        budget,
        cancel,
        None,
''',
)
replace_once(
    path,
    '''    declarations: &DeclarationTypeTable,
    dispatch: &SurfaceDispatchResolver,
    module: ModuleId,
''',
    '''    declarations: &DeclarationTypeTable,
    dispatch: &SurfaceDispatchResolver,
    declared_signature: Option<(&CallableId, &crate::signature::CallableSemanticSignature)>,
    module: ModuleId,
''',
)
sub_once(
    path,
    r"    // Bind parameters and expected return from the exact published signature consumed by this body\..*?\n    // 2\. Check each statement",
    '''    // Bind parameters and the constraining return requirement from the exact
    // canonical declaration signature. `inferred_return` is deliberately not
    // consulted here; a body-derived result can never become its own premise.
    let constructor_body = declared_signature.is_some_and(|(signature_id, _)| {
        callable.side == crate::identity::DispatchSide::Instance
            && signature_id.side == crate::identity::DispatchSide::Class
    });
    let setter_body = matches!(
        callable.selector.kind,
        phalcom_common::selector::SelectorKind::Setter | phalcom_common::selector::SelectorKind::SubscriptSet
    );
    if let Some(field_lifecycle) = field_lifecycle {
        field_lifecycle.seed_flow_for_owner(&mut ctx.flow, &callable.owner, constructor_body);
    }

    if let Some((signature_id, signature)) = declared_signature {
        ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::CallableSignature(signature_id.clone()));
        ctx.push_scope();
        for parameter in &signature.parameters {
            ctx.bind_callable_parameter(
                parameter.local_name.to_string(),
                parameter.declared_type.to_knowledge(),
                body_range,
            );
        }
        let declared_return = signature.declared_return.to_knowledge();
        if let Some(ret_ty) = declared_return.ty() {
            ctx.expected_return = Some(CallableReturnContract {
                ty: ret_ty,
                origin: crate::types::evidence::EvidenceOrigin::CallableSignature,
                source: None,
            });
        }
    }

    // 2. Check each statement''',
)

# Dispatch reads still record dispatch-structure ownership, but type contract
# dependency is always the canonical CallableSignature product, partial or not.
path = "phalcom-semantic/src/checker/context.rs"
sub_once(
    path,
    r"    /// Records the canonical dependency for a callable signature consumed from a declaration surface\..*?\n    pub\(crate\) fn record_consumed_callable_signature\(.*?\n    \}\n",
    '''    /// Records a dispatch lookup's structural and callable-type dependencies.
    ///
    /// `DeclarationSurface` owns selector/visibility/hierarchy projection only;
    /// every query-owned callable's type contract is represented by its
    /// canonical `CallableSignature` product, including partial declarations.
    pub(crate) fn record_consumed_callable_signature(&self, callable: &CallableId, _signature: &crate::dispatch::CallableSignature) {
        if !is_query_owned_module(&callable.owner.module) {
            return;
        }
        record_declaration_surface_dependency(&self.semantic_dependencies, &callable.owner);
        self.record_semantic_dependency(SemanticDependency::CallableSignature(callable.clone()));
    }
''',
)

# Session publishes signatures from class members before dispatch surfaces, and
# inference augments the canonical table rather than reconstructing from dispatch.
path = "phalcom-semantic/src/session.rs"
text = read(path)
text = text.replace(", query_source_structure, query_unlinked_interface, semantic_signature_from_surface,\n", ", query_source_structure, query_unlinked_interface,\n")
write(path, text)
sub_once(
    path,
    r"                let surface = match query_declaration_surface\(.*?\n                \}\n            \}\n        \}\n\n        // 7\. Check field defaults",
    '''                // Publish declaration-owned callable signatures first. Dispatch
                // surfaces are compatibility projections of these facts.
                for member in &class_def.members {
                    let Some(callable_id) = crate::checker::declaration_signature::callable_id_for_member(&decl_id, member) else {
                        continue;
                    };
                    match query_callable_signature(
                        &mut self.db,
                        callable_id,
                        parsed_unit.clone(),
                        &mut self.store,
                        &hierarchy,
                        &resolver,
                        &declarations,
                    ) {
                        QueryOutcome::Ready(signature) => callable_signatures.insert((*signature).clone()),
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    }
                }

                let surface = match query_declaration_surface(
                    &mut self.db,
                    decl_id.clone(),
                    parsed_unit.clone(),
                    linked_interface.clone(),
                    &mut self.store,
                    &hierarchy,
                    &resolver,
                    &declarations,
                ) {
                    QueryOutcome::Ready(surface) => surface,
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                    QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                };
                if let Some(diagnostics) = self
                    .db
                    .product(&QueryKey::DeclarationSurface(decl_id.clone()))
                    .and_then(|product| product.as_declaration_surface_diagnostics())
                {
                    diags_by_module.entry(module_id.clone()).or_default().extend(diagnostics.iter().cloned());
                }

                dispatch.register_surface(decl_id.clone(), (*surface).clone());
                if let Some(ty) = declarations.form(&decl_id) {
                    dispatch.register_type(ty, decl_id.clone());
                }
            }
        }

        // 7. Check field defaults''',
)
replace_once(
    path,
    '''            if let Some(surface) = dispatch.surfaces().get(&callable.owner)
                && let Some(signature) = surface.get_callable(callable.side, &callable.selector)
            {
                let mut semantic_signature = semantic_signature_from_surface(&callable, signature);
                semantic_signature.inferred_return = Some(summary.clone());
                callable_signatures.insert(semantic_signature);
            }
''',
    '''            let signature_id = if callable_signatures.get(&callable).is_some() {
                Some(callable.clone())
            } else if callable.side == DispatchSide::Instance {
                let class_side = CallableId::new(callable.owner.clone(), callable.selector.clone(), DispatchSide::Class);
                callable_signatures.get(&class_side).is_some().then_some(class_side)
            } else {
                None
            };
            if let Some(signature_id) = signature_id
                && let Some(signature) = callable_signatures.get_mut(&signature_id)
            {
                signature.inferred_return = Some(summary.clone());
            }
''',
)

# The legacy dispatch completeness helper is no longer a publication gate.
path = "phalcom-semantic/src/dispatch.rs"
text = read(path)
text = text.replace(
    '''    /// Returns whether every parameter and the return value have canonical known types.
    ///
    /// Only complete source contracts can be projected into a
    /// [`CallableSemanticSignature`](crate::signature::CallableSemanticSignature).
    /// Partial source signatures stay represented by their declaration surface until
    /// inference publishes a richer canonical signature product.
''',
    '''    /// Returns whether every slot in this compatibility dispatch projection
    /// currently has a known type. Canonical signature publication does not
    /// depend on this predicate; partial declarations are first-class products.
''',
)
write(path, text)

# Flip ownership regressions to the new architecture.
path = "phalcom-semantic/tests/semantic/incremental/query_ownership.rs"
text = read(path)
text = text.replace(
    "    assert_eq!(dependency_keys(&session, &signature_key), vec![surface_key]);\n",
    '''    let signature_dependencies = dependency_keys(&session, &signature_key);
    assert!(
        !signature_dependencies.contains(&surface_key),
        "callable signature is declaration-owned and must not depend on its dispatch projection"
    );
''',
)
text = text.replace(
    "fn body_query_ensures_complete_signature_without_signature_prewarm()",
    "fn body_query_ensures_canonical_signature_without_signature_prewarm()",
)
write(path, text)
sub_once(
    path,
    r"#\[test\]\nfn partial_source_signature_stays_surface_backed_without_truncated_callable_product\(\) \{.*?\n\}\s*$",
    '''#[test]
fn partial_source_signature_is_canonical_and_body_depends_on_it() {
    let module = module_id();
    let source = r#"
class Api {
  @class
  value(_ input) -> Int { 1 }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), source, 1));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let owner = DeclarationId::new(module.clone(), "Api".into());
    let selector = Selector::method("value", [SelectorSlot::Positional]).unwrap();
    let body_callable = CallableId::new(owner.clone(), selector, DispatchSide::Class);
    let signature_key = QueryKey::CallableSignature(body_callable.clone());
    let body_key = QueryKey::CallableBody(body_callable);

    let signature = session
        .db()
        .product(&signature_key)
        .and_then(|product| product.as_callable_signature())
        .expect("partial declaration must publish a canonical callable signature");
    assert_eq!(signature.parameter_count(), 1);
    assert!(signature.parameters[0].declared_type.is_unknown());

    let body_dependencies = dependency_keys(&session, &body_key);
    assert!(body_dependencies.contains(&signature_key));
    assert!(
        !body_dependencies.contains(&QueryKey::DeclarationSurface(owner)),
        "body declaration constraints come from CallableSignature, not dispatch surface"
    );
}
''',
)

print("phase B declaration authority patch applied")
