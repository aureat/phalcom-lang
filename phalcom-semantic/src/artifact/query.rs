//! Query image validation, reverse-pruning, and reconstruction (`PERF003.C2.P7.T9`).

use crate::canonical_image::{CanonicalImageError, CanonicalQueryEntry, CanonicalSemanticQueryImage, CanonicalSemanticWorldId};
use crate::db::key::QueryKey;
use crate::db::product::{CallableSignatureProduct, DeclarationSurfaceProduct, SemanticProduct};
use crate::db::{DependencyEdge, InputFingerprint, ProductFingerprint};
use crate::metadata::stable_identity::{from_stable_callable, from_stable_declaration, from_stable_field};
use crate::snapshot::SemanticSnapshot;
use phalcom_compiler_artifact::query::{
    PortableQueryKeyV1, PortableQueryKindV1, QueryImageV1, QueryProfileSupport,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::identity::{StableCallableRef, StableDeclarationRef, StableFieldRef};
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

fn incomplete(message: impl Into<String>) -> CanonicalImageError {
    CanonicalImageError::IncompletePublication(message.into())
}

/// Validates that `QueryImageV1` contains no source-dependent roots, has closed
/// acyclic dependencies, and separates aggregate dependency evidence (`P7.T9`).
pub fn validate_query_image_closure(image: &QueryImageV1) -> Result<BTreeSet<PortableQueryKeyV1>, ArtifactError> {
    let mut retained = BTreeSet::new();
    let len = image.supported_roots.len();
    for (idx, root) in image.supported_roots.iter().enumerate() {
        if root.key.kind.profile_support() != QueryProfileSupport::SupportedReusableRoot {
            return Err(invalid("unsupported or source-dependent query root in QueryImageV1"));
        }
        if !retained.insert(root.key.clone()) {
            return Err(invalid("duplicate query root key in QueryImageV1"));
        }
        for &dep in &root.dependency_indices {
            if (dep as usize) >= len || (dep as usize) == idx {
                return Err(invalid("out-of-bounds or self-referential query dependency index"));
            }
        }
    }
    let mut pending: BTreeSet<usize> = (0..len).collect();
    loop {
        let leaves = pending
            .iter()
            .copied()
            .filter(|&i| image.supported_roots[i].dependency_indices.iter().all(|&d| !pending.contains(&(d as usize))))
            .collect::<Vec<_>>();
        if leaves.is_empty() {
            break;
        }
        for leaf in leaves {
            pending.remove(&leaf);
        }
    }
    if !pending.is_empty() {
        return Err(invalid("cyclic dependency in QueryImageV1"));
    }
    for evidence in &image.dependency_evidence {
        if evidence.key.kind.profile_support() == QueryProfileSupport::OmittedSourceDependent {
            return Err(invalid("ParsedModule cannot appear in dependency evidence"));
        }
    }
    Ok(retained)
}

fn decode_owner<T: DeserializeOwned>(key: &PortableQueryKeyV1, tag: &str) -> Result<T, CanonicalImageError> {
    let prefix = format!("{tag}:");
    let payload = key
        .stable_owner_key
        .strip_prefix(&prefix)
        .ok_or_else(|| incomplete(format!("query owner key has wrong type tag for {:?}", key.kind)))?;
    serde_json::from_str(payload).map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))
}

fn resolve_query_key(key: &PortableQueryKeyV1) -> Result<QueryKey, CanonicalImageError> {
    Ok(match key.kind {
        PortableQueryKindV1::UnlinkedInterface => QueryKey::UnlinkedInterface(
            crate::metadata::stable_identity::from_stable_module(&decode_owner(key, "module")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::ResolvedImports => QueryKey::ResolvedImports(
            crate::metadata::stable_identity::from_stable_module(&decode_owner(key, "module")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::LinkedInterface => QueryKey::LinkedInterface(
            crate::metadata::stable_identity::from_stable_module(&decode_owner(key, "module")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::DeclarationShell => QueryKey::DeclarationShell(
            from_stable_declaration(&decode_owner::<StableDeclarationRef>(key, "declaration")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::CallableDefinition => QueryKey::CallableDefinition(
            from_stable_callable(&decode_owner::<StableCallableRef>(key, "callable")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::CallableSignature => QueryKey::CallableSignature(
            from_stable_callable(&decode_owner::<StableCallableRef>(key, "callable")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::FieldSignature => QueryKey::FieldSignature(
            from_stable_field(&decode_owner::<StableFieldRef>(key, "field")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::DeclarationSurface => QueryKey::DeclarationSurface(
            from_stable_declaration(&decode_owner::<StableDeclarationRef>(key, "declaration")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::TraitHeader => QueryKey::TraitHeader(
            from_stable_declaration(&decode_owner::<StableDeclarationRef>(key, "declaration")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::TraitSurface => QueryKey::TraitSurface(
            from_stable_declaration(&decode_owner::<StableDeclarationRef>(key, "declaration")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::DataDeclaration => QueryKey::DataDeclaration(
            from_stable_declaration(&decode_owner::<StableDeclarationRef>(key, "declaration")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::EnumDeclaration => QueryKey::EnumDeclaration(
            from_stable_declaration(&decode_owner::<StableDeclarationRef>(key, "declaration")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::AssociatedSurface => QueryKey::AssociatedSurface(
            from_stable_declaration(&decode_owner::<StableDeclarationRef>(key, "declaration")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::TypeMemberDeclaration => {
            let (owner, (selector_tag, selector_name)): (StableDeclarationRef, (u8, String)) = decode_owner(key, "type-member")?;
            let selector = match selector_tag {
                0 if !selector_name.is_empty() => crate::types::type_member::TypeMemberSelector::Named(selector_name.into_boxed_str()),
                1 if selector_name.is_empty() => crate::types::type_member::TypeMemberSelector::Subscript,
                _ => return Err(incomplete("invalid type-member selector in portable query owner")),
            };
            QueryKey::TypeMemberDeclaration(crate::types::type_member::TypeMemberDeclarationId::new(
                from_stable_declaration(&owner).map_err(incomplete)?,
                selector,
            ))
        }
        PortableQueryKindV1::ModuleStaticBindingCatalog => QueryKey::ModuleStaticBindingCatalog(
            crate::metadata::stable_identity::from_stable_module(&decode_owner(key, "module")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::ModuleStaticBindings => QueryKey::ModuleStaticBindings(
            crate::metadata::stable_identity::from_stable_module(&decode_owner(key, "module")?).map_err(incomplete)?,
        ),
        PortableQueryKindV1::ParsedModule | PortableQueryKindV1::HierarchyEdge => {
            return Err(incomplete("non-reusable query kind cannot be reconstructed as a root"));
        }
    })
}

fn callable_definition_fingerprint(definition: &crate::impls::EffectiveCallableDefinition) -> ProductFingerprint {
    use std::hash::{Hash, Hasher};
    let signature_fingerprint = crate::db::fingerprint::callable_signature_product_fingerprint(&definition.signature);
    let origin_tag = match &definition.origin {
        crate::impls::CallableDefinitionOrigin::PrimaryDeclaration => 0_u64,
        crate::impls::CallableDefinitionOrigin::InherentImpl(impl_id) => {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            impl_id.hash(&mut hasher);
            hasher.finish()
        }
        crate::impls::CallableDefinitionOrigin::ConformanceWitness(impl_id) => {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            impl_id.hash(&mut hasher);
            hasher.finish().rotate_left(7)
        }
    };
    let body_context_tag = definition
        .inherent_body_context
        .as_ref()
        .map_or(0, crate::db::fingerprint::inherent_body_context_fingerprint);
    let value = signature_fingerprint.raw()
        ^ origin_tag.rotate_left(17)
        ^ (definition.source_member_index as u64).rotate_left(33)
        ^ body_context_tag.rotate_left(11);
    ProductFingerprint::new(value)
}

fn product_and_fingerprints(
    snapshot: &SemanticSnapshot,
    key: &QueryKey,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
) -> Result<(Arc<SemanticProduct>, ProductFingerprint, InputFingerprint), CanonicalImageError> {
    use crate::db::fingerprint as fp;
    let empty_diagnostics: Arc<[crate::diagnostic::SemanticDiagnostic]> = Arc::from([]);
    let result = match key {
        QueryKey::UnlinkedInterface(module) => {
            let interface = modules.interfaces().get(module).cloned()
                .ok_or_else(|| incomplete("missing unlinked interface product for portable owner"))?;
            let product_fingerprint = fp::unlinked_interface_product_fingerprint(&interface);
            let input_fingerprint = fp::unlinked_interface_input_fingerprint(&interface);
            (SemanticProduct::UnlinkedInterface(interface), product_fingerprint, input_fingerprint)
        }
        QueryKey::LinkedInterface(module) => {
            let linked = modules.linked().modules.get(module)
                .ok_or_else(|| incomplete("missing linked interface product for portable owner"))?;
            let interface = Arc::new(linked.interface.clone());
            let product_fingerprint = fp::linked_interface_product_fingerprint(&interface);
            let input_fingerprint = fp::linked_interface_input_fingerprint(&interface);
            (SemanticProduct::LinkedInterface(interface), product_fingerprint, input_fingerprint)
        }
        QueryKey::ResolvedImports(module) => {
            let interface = modules.interfaces().get(module)
                .ok_or_else(|| incomplete("missing import source interface for portable owner"))?;
            let references = modules.linked().graphs.references.edges_from(module);
            let mut imports = std::collections::BTreeMap::new();
            for import in &interface.imports {
                let edge = references.iter().find(|edge| edge.range == import.range())
                    .ok_or_else(|| incomplete("resolved import has no matching owner edge"))?;
                let path = import.path().to_string();
                if imports.insert(path, edge.to.clone()).is_some() {
                    return Err(incomplete("duplicate resolved import path in portable owner"));
                }
            }
            let product = Arc::new(crate::module_product::ResolvedImportsProduct::new(module.clone(), imports, Vec::new()));
            let product_fingerprint = fp::resolved_imports_product_fingerprint(&product);
            let input_fingerprint = fp::unlinked_interface_input_fingerprint(interface);
            (SemanticProduct::ResolvedImports(product), product_fingerprint, input_fingerprint)
        }
        QueryKey::DeclarationShell(declaration) => {
            let shell = if let Some(info) = snapshot.declarations.get(declaration) {
                crate::declarations::TypeDeclarationShell::Nominal(info.clone())
            } else if let Some(alias) = snapshot.type_aliases.get(declaration) {
                crate::declarations::TypeDeclarationShell::Alias(alias.clone())
            } else {
                return Err(incomplete("missing declaration shell product for portable owner"));
            };
            let product_fingerprint = fp::declaration_shell_product_fingerprint(&shell);
            let input_fingerprint = fp::declaration_shell_input_fingerprint(&shell);
            (SemanticProduct::DeclarationShell(Arc::new(shell)), product_fingerprint, input_fingerprint)
        }
        QueryKey::DeclarationSurface(declaration) => {
            let surface = snapshot.dispatch.get_surface(declaration).cloned()
                .ok_or_else(|| incomplete("missing declaration surface product for portable owner"))?;
            let product_fingerprint = fp::declaration_surface_product_fingerprint(&surface);
            let input_fingerprint = fp::declaration_surface_query_input_fingerprint(&surface, &[]);
            let product = DeclarationSurfaceProduct { surface: Arc::new(surface), diagnostics: empty_diagnostics };
            (SemanticProduct::DeclarationSurface(Arc::new(product)), product_fingerprint, input_fingerprint)
        }
        QueryKey::CallableSignature(callable) => {
            let signature = snapshot.callable_signatures.get(callable).cloned()
                .ok_or_else(|| incomplete("missing callable signature product for portable owner"))?;
            let product_fingerprint = fp::callable_signature_product_fingerprint(&signature);
            let input_fingerprint = fp::callable_signature_input_fingerprint(&signature);
            let product = CallableSignatureProduct::new(Arc::new(signature), empty_diagnostics);
            (SemanticProduct::CallableSignature(Arc::new(product)), product_fingerprint, input_fingerprint)
        }
        QueryKey::CallableDefinition(callable) => {
            let definition = snapshot.callable_definitions.get(callable).cloned()
                .ok_or_else(|| incomplete("missing callable definition product for portable owner"))?;
            let fingerprint = callable_definition_fingerprint(&definition);
            (SemanticProduct::CallableDefinition(Arc::new(definition)), fingerprint, InputFingerprint::new(fingerprint.raw()))
        }
        QueryKey::FieldSignature(field) => {
            let signature = snapshot.field_signatures.get(field).cloned()
                .ok_or_else(|| incomplete("missing field signature product for portable owner"))?;
            let product_fingerprint = fp::field_signature_product_fingerprint(&signature);
            let input_fingerprint = fp::field_signature_input_fingerprint(&signature);
            let product = crate::db::product::FieldSignatureProduct::new(Arc::new(signature), empty_diagnostics);
            (SemanticProduct::FieldSignature(Arc::new(product)), product_fingerprint, input_fingerprint)
        }
        QueryKey::TraitHeader(declaration) => {
            let header = snapshot.trait_headers.get(declaration).cloned()
                .ok_or_else(|| incomplete("missing trait header product for portable owner"))?;
            let product_fingerprint = fp::trait_header_product_fingerprint(&header);
            let input_fingerprint = fp::trait_header_input_fingerprint(&header);
            (SemanticProduct::TraitHeader(header), product_fingerprint, input_fingerprint)
        }
        QueryKey::TraitSurface(declaration) => {
            let surface = snapshot.trait_surfaces.get(declaration).cloned()
                .ok_or_else(|| incomplete("missing trait surface product for portable owner"))?;
            let product_fingerprint = fp::trait_surface_product_fingerprint(&surface);
            let input_fingerprint = fp::trait_surface_input_fingerprint(&surface);
            (SemanticProduct::TraitSurface(surface), product_fingerprint, input_fingerprint)
        }
        QueryKey::DataDeclaration(declaration) => {
            let info = snapshot.data_semantics.data_decls.get(declaration).cloned()
                .ok_or_else(|| incomplete("missing data declaration product for portable owner"))?;
            let product = crate::db::product::DataDeclarationProduct { info, diagnostics: empty_diagnostics };
            let product_fingerprint = fp::data_declaration_product_fingerprint(&product);
            let input_fingerprint = fp::data_declaration_input_fingerprint(&product);
            (SemanticProduct::DataDeclaration(Arc::new(product)), product_fingerprint, input_fingerprint)
        }
        QueryKey::EnumDeclaration(declaration) => {
            let info = snapshot.enum_semantics.enums.get(declaration).cloned()
                .ok_or_else(|| incomplete("missing enum declaration product for portable owner"))?;
            let variants = info.variants.iter().map(|variant| {
                snapshot.enum_semantics.variants.get(variant).map(|value| (**value).clone())
                    .ok_or_else(|| incomplete("missing enum variant product for portable owner"))
            }).collect::<Result<Vec<_>, _>>()?;
            let product = crate::db::product::EnumDeclarationProduct {
                info,
                variants: Arc::from(variants),
                diagnostics: empty_diagnostics,
            };
            let product_fingerprint = fp::enum_declaration_product_fingerprint(&product);
            let input_fingerprint = fp::enum_declaration_input_fingerprint(&product);
            (SemanticProduct::EnumDeclaration(Arc::new(product)), product_fingerprint, input_fingerprint)
        }
        QueryKey::AssociatedSurface(declaration) => {
            let surface = snapshot.associated_surfaces.surfaces.get(declaration).cloned()
                .ok_or_else(|| incomplete("missing associated surface product for portable owner"))?;
            let product_fingerprint = fp::associated_surface_product_fingerprint(&surface);
            let input_fingerprint = fp::associated_surface_input_fingerprint(&surface);
            (SemanticProduct::AssociatedSurface(surface), product_fingerprint, input_fingerprint)
        }
        QueryKey::TypeMemberDeclaration(id) => {
            let product = snapshot.type_member_declarations.get(id).cloned()
                .ok_or_else(|| incomplete("missing type-member declaration product for portable owner"))?;
            let product_fingerprint = fp::type_member_declaration_product_fingerprint(&product);
            let input_fingerprint = fp::type_member_declaration_input_fingerprint(None, &product);
            (SemanticProduct::TypeMemberDeclaration(product), product_fingerprint, input_fingerprint)
        }
        _ => return Err(incomplete("query kind has no loaded semantic owner adapter")),
    };
    Ok((Arc::new(result.0), result.1, result.2))
}

/// Reconstructs an immutable query image. Every wire root and edge must map to
/// a typed live owner and a current product; no row is skipped or assigned a
/// synthetic zero fingerprint.
pub fn reconstruct_query_image(
    image: &QueryImageV1,
    snapshot: &Arc<SemanticSnapshot>,
    world: &CanonicalSemanticWorldId,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
) -> Result<(Arc<CanonicalSemanticQueryImage>, BTreeMap<QueryKey, ProductFingerprint>), CanonicalImageError> {
    validate_query_image_closure(image).map_err(|error| incomplete(error.to_string()))?;

    let mut roots = Vec::with_capacity(image.supported_roots.len());
    for root in &image.supported_roots {
        let query_key = resolve_query_key(&root.key)?;
        let (product, product_fingerprint, input_fingerprint) = product_and_fingerprints(snapshot, &query_key, modules)?;
        roots.push((query_key, product, product_fingerprint, input_fingerprint));
    }

    let mut entries = BTreeMap::new();
    let mut fingerprints = BTreeMap::new();
    for (idx, root) in image.supported_roots.iter().enumerate() {
        let (query_key, product, product_fingerprint, input_fingerprint) = &roots[idx];
        let mut edges = Vec::with_capacity(root.dependency_indices.len());
        for &dep_idx in &root.dependency_indices {
            let (dependency, _, observed_fingerprint, _) = roots
                .get(dep_idx as usize)
                .ok_or_else(|| incomplete("query dependency references an absent root"))?;
            edges.push(DependencyEdge {
                dependent: query_key.clone(),
                dependency: dependency.clone(),
                observed_fingerprint: *observed_fingerprint,
            });
        }
        if entries.insert(
            query_key.clone(),
            CanonicalQueryEntry {
                input_fingerprint: *input_fingerprint,
                product_fingerprint: *product_fingerprint,
                dependencies: Arc::from(edges.into_boxed_slice()),
                product: product.clone(),
            },
        ).is_some() {
            return Err(incomplete("distinct portable query roots resolve to the same semantic owner"));
        }
        fingerprints.insert(query_key.clone(), *product_fingerprint);
    }

    let query_image = CanonicalSemanticQueryImage::from_loaded_artifact_entries(
        world.clone(),
        entries,
        snapshot.store.clone(),
        snapshot.static_values.clone(),
    )?;
    Ok((Arc::new(query_image), fingerprints))
}
