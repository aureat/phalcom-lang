//! Query image validation, reverse-pruning, and reconstruction (`PERF003.C2.P7.T9`).

use crate::canonical_image::{CanonicalImageError, CanonicalQueryEntry, CanonicalSemanticQueryImage, CanonicalSemanticWorldId};
use crate::db::key::QueryKey;
use crate::db::product::{CallableSignatureProduct, DeclarationSurfaceProduct, SemanticProduct};
use crate::db::{DependencyEdge, InputFingerprint, ProductFingerprint};
use crate::metadata::stable_identity::from_stable_callable;
use crate::snapshot::SemanticSnapshot;
use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::error::QueryImageError;
use phalcom_compiler_artifact::query::{PortableQueryKeyV1, PortableQueryKindV1, QueryImageV1, QueryProfileSupport};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::budgeted::BudgetDecode;
use phalcom_type_meta::identity::{StableCallableRef, StableDeclarationRef, StableFieldRef};
use std::collections::BTreeMap;
use std::sync::Arc;

fn incomplete(message: impl Into<String>) -> CanonicalImageError {
    CanonicalImageError::IncompletePublication(message.into())
}

/// Validates that `QueryImageV1` contains no source-dependent roots, has closed
/// acyclic dependencies, and separates aggregate dependency evidence (`P7.T9`).
/// Scratch is charged against the caller's load transaction before allocation.
pub fn validate_query_image_closure_with_allocation_budget(
    image: &QueryImageV1,
    limits: &phalcom_compiler_artifact::ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<(), ArtifactError> {
    let len = image.supported_roots.len();
    let root_limit = limits.max_semantic_entries_per_family.min(limits.max_query_roots);
    if len > root_limit {
        return Err(QueryImageError::CountLimitExceeded.into());
    }

    for (idx, root) in image.supported_roots.iter().enumerate() {
        if root.key.kind.profile_support() != QueryProfileSupport::SupportedReusableRoot {
            return Err(QueryImageError::UnsupportedRoot.into());
        }
        if root.dependency_indices.len() > limits.max_query_dependencies_per_root {
            return Err(QueryImageError::CountLimitExceeded.into());
        }
        for &dep in &root.dependency_indices {
            let dep = usize::try_from(dep).map_err(|_| QueryImageError::InvalidDependencyIndex)?;
            if dep >= len || dep == idx {
                return Err(QueryImageError::InvalidDependencyIndex.into());
            }
        }
    }

    for evidence in &image.dependency_evidence {
        if evidence.key.kind.profile_support() == QueryProfileSupport::OmittedSourceDependent {
            return Err(QueryImageError::SourceDependentEvidence.into());
        }
    }

    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| AllocationError::Overflow)?;
    let mut scratch = transaction.child(section_limit);
    let mut ordered_keys = scratch.vector::<&PortableQueryKeyV1>(len).map_err(ArtifactError::from)?;
    for root in &image.supported_roots {
        ordered_keys.push(&root.key);
    }

    ordered_keys.sort_unstable();
    if ordered_keys.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(QueryImageError::DuplicateRoot.into());
    }

    let mut colors = scratch.vector::<u8>(len).map_err(ArtifactError::from)?;
    colors.resize(len, 0);
    let mut stack = scratch.vector::<(usize, usize)>(len).map_err(ArtifactError::from)?;

    // Iterative three-color DFS is O(V + E), bounded by the root count, and
    // avoids recursion and repeated whole-graph leaf scans for untrusted input.
    for start in 0..len {
        if colors[start] != 0 {
            continue;
        }
        colors[start] = 1;
        stack.push((start, 0));

        while let Some(&(node, next_dependency)) = stack.last() {
            let dependencies = &image.supported_roots[node].dependency_indices;
            if next_dependency == dependencies.len() {
                colors[node] = 2;
                stack.pop();
                continue;
            }

            let dependency = usize::try_from(dependencies[next_dependency]).map_err(|_| QueryImageError::InvalidDependencyIndex)?;
            let Some(frame) = stack.last_mut() else {
                break;
            };
            frame.1 = next_dependency.checked_add(1).ok_or(AllocationError::Overflow)?;

            match colors[dependency] {
                1 => return Err(QueryImageError::DependencyCycle.into()),
                2 => {}
                _ => {
                    colors[dependency] = 1;
                    stack.push((dependency, 0));
                }
            }
        }
    }

    transaction.commit_child(scratch).map_err(ArtifactError::from)?;
    Ok(())
}

/// Validates the query closure using a local bounded allocation budget.
pub fn validate_query_image_closure(image: &QueryImageV1) -> Result<(), ArtifactError> {
    let limits = phalcom_compiler_artifact::ValidationLimits::default();
    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| AllocationError::Overflow)?;
    let mut budget = AllocationBudget::new(section_limit);
    validate_query_image_closure_with_allocation_budget(image, &limits, &mut budget)
}

fn decode_owner<T: phalcom_type_meta::budgeted::BudgetDecode>(
    key: &PortableQueryKeyV1,
    tag: &str,
    allocations: &mut AllocationBudget,
) -> Result<T, CanonicalImageError> {
    let prefix_len = tag.len() + 1;
    let prefix = key.stable_owner_key.get(..prefix_len).ok_or_else(|| incomplete("query owner key has wrong type tag"))?;
    if prefix.strip_suffix(':') != Some(tag) {
        return Err(incomplete("query owner key has wrong type tag"));
    }
    let payload = key
        .stable_owner_key
        .get(prefix_len..)
        .ok_or_else(|| incomplete(format!("query owner key has wrong type tag for {:?}", key.kind)))?;
    let mut decoder = phalcom_type_meta::budgeted::Decoder::new(payload, allocations, 256)
        .map_err(|error| incomplete(format!("invalid portable query owner key for {:?}: {error}: {payload}", key.kind)))?;
    let decoded = T::decode(&mut decoder).map_err(|error| match error {
        phalcom_type_meta::budgeted::DecodeError::Allocation(error) => allocation_error(error),
        error => incomplete(format!("invalid portable query owner key for {:?}: {error}: {payload}", key.kind)),
    })?;
    decoder
        .cursor
        .finish()
        .map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
    Ok(decoded)
}

fn consume_stable_project(project: phalcom_type_meta::identity::StableProjectRef) -> Result<phalcom_modules::ProjectIdentity, CanonicalImageError> {
    use phalcom_type_meta::identity::StableProjectRef;
    Ok(match project {
        StableProjectRef::Builtin { namespace, .. } if namespace.as_ref() == "universe" => phalcom_modules::ProjectIdentity::Universe,
        StableProjectRef::Session { session_fingerprint } => {
            let bytes = session_fingerprint.as_bytes();
            let raw = u64::from_be_bytes([
                bytes[8], bytes[9], bytes[10], bytes[11],
                bytes[12], bytes[13], bytes[14], bytes[15],
            ]);
            phalcom_modules::ProjectIdentity::Synthetic(phalcom_modules::SyntheticProjectId::from_raw(raw))
        }
        _ => return Err(incomplete("unsupported stable project ref for direct conversion")),
    })
}

fn consume_stable_module(
    module: phalcom_type_meta::identity::StableModuleRef,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_modules::ModuleId, CanonicalImageError> {
    let project = consume_stable_project(module.project)?;
    let mut components = allocations.vector::<phalcom_modules::ModuleComponent>(module.path.len()).map_err(allocation_error)?;
    for spelling in Vec::from(module.path) {
        let component = phalcom_modules::ModuleComponent::from_owned_identifier(spelling)
            .map_err(|_| incomplete("invalid module component in stable query owner"))?;
        components.push(component);
    }
    let components = allocations.boxed_slice(components).map_err(allocation_error)?;
    Ok(phalcom_modules::ModuleId {
        project,
        path: phalcom_modules::ModulePath::from_components(components),
    })
}

fn consume_stable_declaration(
    declaration: StableDeclarationRef,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::DeclarationId, CanonicalImageError> {
    let module = consume_stable_module(declaration.module, allocations)?;
    let name = Vec::from(declaration.path)
        .into_iter()
        .next()
        .ok_or_else(|| incomplete("empty declaration path in stable query owner"))?;
    Ok(crate::identity::DeclarationId::new(module, name))
}

fn consume_stable_field(
    field: StableFieldRef,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::FieldId, CanonicalImageError> {
    let owner = consume_stable_declaration(field.owner, allocations)?;
    Ok(crate::identity::FieldId {
        owner,
        name: field.name,
        side: crate::metadata::stable_identity::from_stable_dispatch_side(field.side),
    })
}

fn resolve_query_key(key: &PortableQueryKeyV1, allocations: &mut AllocationBudget) -> Result<QueryKey, CanonicalImageError> {
    Ok(match key.kind {
        PortableQueryKindV1::UnlinkedInterface => {
            QueryKey::UnlinkedInterface(consume_stable_module(decode_owner(key, "module", allocations)?, allocations)?)
        }
        PortableQueryKindV1::ResolvedImports => {
            QueryKey::ResolvedImports(consume_stable_module(decode_owner(key, "module", allocations)?, allocations)?)
        }
        PortableQueryKindV1::LinkedInterface => {
            QueryKey::LinkedInterface(consume_stable_module(decode_owner(key, "module", allocations)?, allocations)?)
        }
        PortableQueryKindV1::DeclarationShell => {
            QueryKey::DeclarationShell(consume_stable_declaration(decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?, allocations)?)
        }
        PortableQueryKindV1::CallableDefinition => {
            QueryKey::CallableDefinition(from_stable_callable(&decode_owner::<StableCallableRef>(key, "callable", allocations)?).map_err(incomplete)?)
        }
        PortableQueryKindV1::CallableSignature => {
            QueryKey::CallableSignature(from_stable_callable(&decode_owner::<StableCallableRef>(key, "callable", allocations)?).map_err(incomplete)?)
        }
        PortableQueryKindV1::FieldSignature => {
            QueryKey::FieldSignature(consume_stable_field(decode_owner::<StableFieldRef>(key, "field", allocations)?, allocations)?)
        }
        PortableQueryKindV1::DeclarationSurface => {
            QueryKey::DeclarationSurface(consume_stable_declaration(decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?, allocations)?)
        }
        PortableQueryKindV1::TraitHeader => {
            QueryKey::TraitHeader(consume_stable_declaration(decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?, allocations)?)
        }
        PortableQueryKindV1::TraitSurface => {
            QueryKey::TraitSurface(consume_stable_declaration(decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?, allocations)?)
        }
        PortableQueryKindV1::DataDeclaration => {
            QueryKey::DataDeclaration(consume_stable_declaration(decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?, allocations)?)
        }
        PortableQueryKindV1::EnumDeclaration => {
            QueryKey::EnumDeclaration(consume_stable_declaration(decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?, allocations)?)
        }
        PortableQueryKindV1::AssociatedSurface => {
            QueryKey::AssociatedSurface(consume_stable_declaration(decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?, allocations)?)
        }
        PortableQueryKindV1::TypeMemberDeclaration => {
            let prefix_len = "type-member:".len();
            let payload = key.stable_owner_key.get(prefix_len..).ok_or_else(|| incomplete("query owner key has wrong type tag"))?;
            if key.stable_owner_key.get(..prefix_len) != Some("type-member:") {
                return Err(incomplete("query owner key has wrong type tag"));
            }
            let mut decoder = phalcom_type_meta::budgeted::Decoder::new(payload, allocations, 256)
                .map_err(|error| incomplete(format!("invalid portable query owner key for type-member: {error}: {payload}")))?;
            decoder.cursor.expect(b'[').map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            let owner = StableDeclarationRef::decode(&mut decoder).map_err(|error| match error {
                phalcom_type_meta::budgeted::DecodeError::Allocation(error) => allocation_error(error),
                error => incomplete(format!("invalid type-member stable declaration: {error}: {payload}")),
            })?;
            decoder.cursor.expect(b',').map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            decoder.cursor.expect(b'[').map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            let selector_tag = u8::decode(&mut decoder).map_err(|error| match error {
                phalcom_type_meta::budgeted::DecodeError::Allocation(error) => allocation_error(error),
                error => incomplete(format!("invalid type-member selector tag: {error}: {payload}")),
            })?;
            decoder.cursor.expect(b',').map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            let selector_name = String::decode(&mut decoder).map_err(|error| match error {
                phalcom_type_meta::budgeted::DecodeError::Allocation(error) => allocation_error(error),
                error => incomplete(format!("invalid portable query owner key: {error}")),
            })?;
            decoder.cursor.expect(b']').map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            decoder.cursor.expect(b']').map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            decoder.cursor.finish().map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            let selector = match selector_tag {
                0 if !selector_name.is_empty() => crate::types::type_member::TypeMemberSelector::Named(selector_name.into_boxed_str()),
                1 if selector_name.is_empty() => crate::types::type_member::TypeMemberSelector::Subscript,
                _ => return Err(incomplete("invalid type-member selector in portable query owner")),
            };
            QueryKey::TypeMemberDeclaration(crate::types::type_member::TypeMemberDeclarationId::new(
                consume_stable_declaration(owner, allocations)?,
                selector,
            ))
        }
        PortableQueryKindV1::ModuleStaticBindingCatalog => {
            QueryKey::ModuleStaticBindingCatalog(consume_stable_module(decode_owner(key, "module", allocations)?, allocations)?)
        }
        PortableQueryKindV1::ModuleStaticBindings => {
            QueryKey::ModuleStaticBindings(consume_stable_module(decode_owner(key, "module", allocations)?, allocations)?)
        }
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
    let body_provenance_tag = crate::db::fingerprint::callable_definition_body_provenance_fingerprint(&definition.body_provenance);
    let value = signature_fingerprint.raw()
        ^ origin_tag.rotate_left(17)
        ^ (definition.source_member_index as u64).rotate_left(33)
        ^ body_context_tag.rotate_left(11)
        ^ body_provenance_tag.rotate_left(23);
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
            let interface = modules
                .interfaces()
                .get(module)
                .cloned()
                .ok_or_else(|| incomplete("missing unlinked interface product for portable owner"))?;
            let product_fingerprint = fp::unlinked_interface_product_fingerprint(&interface);
            let input_fingerprint = fp::unlinked_interface_input_fingerprint(&interface);
            (SemanticProduct::UnlinkedInterface(interface), product_fingerprint, input_fingerprint)
        }
        QueryKey::LinkedInterface(module) => {
            let linked = modules
                .linked()
                .modules
                .get(module)
                .ok_or_else(|| incomplete("missing linked interface product for portable owner"))?;
            let interface = Arc::new(linked.interface.clone());
            let product_fingerprint = fp::linked_interface_product_fingerprint(&interface);
            let input_fingerprint = fp::linked_interface_input_fingerprint(&interface);
            (SemanticProduct::LinkedInterface(interface), product_fingerprint, input_fingerprint)
        }
        QueryKey::ResolvedImports(module) => {
            let interface = modules
                .interfaces()
                .get(module)
                .ok_or_else(|| incomplete("missing import source interface for portable owner"))?;
            let references = modules.linked().graphs.references.edges_from(module);
            let mut imports = std::collections::BTreeMap::new();
            for import in &interface.imports {
                let edge = references
                    .iter()
                    .find(|edge| edge.range == import.range())
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
            let surface = snapshot
                .dispatch
                .get_surface(declaration)
                .cloned()
                .ok_or_else(|| incomplete("missing declaration surface product for portable owner"))?;
            let product_fingerprint = fp::declaration_surface_product_fingerprint(&surface);
            let input_fingerprint = fp::declaration_surface_query_input_fingerprint(&surface, &[]);
            let product = DeclarationSurfaceProduct {
                surface: Arc::new(surface),
                diagnostics: empty_diagnostics,
            };
            (SemanticProduct::DeclarationSurface(Arc::new(product)), product_fingerprint, input_fingerprint)
        }
        QueryKey::CallableSignature(callable) => {
            let signature = snapshot
                .callable_signatures
                .get(callable)
                .cloned()
                .ok_or_else(|| incomplete("missing callable signature product for portable owner"))?;
            let product_fingerprint = fp::callable_signature_product_fingerprint(&signature);
            let input_fingerprint = fp::callable_signature_input_fingerprint(&signature);
            let product = CallableSignatureProduct::new(Arc::new(signature), empty_diagnostics);
            (SemanticProduct::CallableSignature(Arc::new(product)), product_fingerprint, input_fingerprint)
        }
        QueryKey::CallableDefinition(callable) => {
            let definition = snapshot
                .callable_definitions
                .get(callable)
                .cloned()
                .ok_or_else(|| incomplete("missing callable definition product for portable owner"))?;
            let fingerprint = callable_definition_fingerprint(&definition);
            (
                SemanticProduct::CallableDefinition(Arc::new(definition)),
                fingerprint,
                InputFingerprint::new(fingerprint.raw()),
            )
        }
        QueryKey::FieldSignature(field) => {
            let signature = snapshot
                .field_signatures
                .get(field)
                .cloned()
                .ok_or_else(|| incomplete("missing field signature product for portable owner"))?;
            let product_fingerprint = fp::field_signature_product_fingerprint(&signature);
            let input_fingerprint = fp::field_signature_input_fingerprint(&signature);
            let product = crate::db::product::FieldSignatureProduct::new(Arc::new(signature), empty_diagnostics);
            (SemanticProduct::FieldSignature(Arc::new(product)), product_fingerprint, input_fingerprint)
        }
        QueryKey::TraitHeader(declaration) => {
            let header = snapshot
                .trait_headers
                .get(declaration)
                .cloned()
                .ok_or_else(|| incomplete("missing trait header product for portable owner"))?;
            let product_fingerprint = fp::trait_header_product_fingerprint(&header);
            let input_fingerprint = fp::trait_header_input_fingerprint(&header);
            (SemanticProduct::TraitHeader(header), product_fingerprint, input_fingerprint)
        }
        QueryKey::TraitSurface(declaration) => {
            let surface = snapshot
                .trait_surfaces
                .get(declaration)
                .cloned()
                .ok_or_else(|| incomplete("missing trait surface product for portable owner"))?;
            let product_fingerprint = fp::trait_surface_product_fingerprint(&surface);
            let input_fingerprint = fp::trait_surface_input_fingerprint(&surface);
            (SemanticProduct::TraitSurface(surface), product_fingerprint, input_fingerprint)
        }
        QueryKey::DataDeclaration(declaration) => {
            let info = snapshot
                .data_semantics
                .data_decls
                .get(declaration)
                .cloned()
                .ok_or_else(|| incomplete("missing data declaration product for portable owner"))?;
            let product = crate::db::product::DataDeclarationProduct {
                info,
                diagnostics: empty_diagnostics,
            };
            let product_fingerprint = fp::data_declaration_product_fingerprint(&product);
            let input_fingerprint = fp::data_declaration_input_fingerprint(&product);
            (SemanticProduct::DataDeclaration(Arc::new(product)), product_fingerprint, input_fingerprint)
        }
        QueryKey::EnumDeclaration(declaration) => {
            let info = snapshot
                .enum_semantics
                .enums
                .get(declaration)
                .cloned()
                .ok_or_else(|| incomplete("missing enum declaration product for portable owner"))?;
            let variants = info
                .variants
                .iter()
                .map(|variant| {
                    snapshot
                        .enum_semantics
                        .variants
                        .get(variant)
                        .map(|value| (**value).clone())
                        .ok_or_else(|| incomplete("missing enum variant product for portable owner"))
                })
                .collect::<Result<Vec<_>, _>>()?;
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
            let surface = snapshot
                .associated_surfaces
                .surfaces
                .get(declaration)
                .cloned()
                .ok_or_else(|| incomplete("missing associated surface product for portable owner"))?;
            let product_fingerprint = fp::associated_surface_product_fingerprint(&surface);
            let input_fingerprint = fp::associated_surface_input_fingerprint(&surface);
            (SemanticProduct::AssociatedSurface(surface), product_fingerprint, input_fingerprint)
        }
        QueryKey::TypeMemberDeclaration(id) => {
            let product = snapshot
                .type_member_declarations
                .get(id)
                .cloned()
                .ok_or_else(|| incomplete("missing type-member declaration product for portable owner"))?;
            let product_fingerprint = fp::type_member_declaration_product_fingerprint(&product);
            let input_fingerprint = fp::type_member_declaration_input_fingerprint(None, &product);
            (SemanticProduct::TypeMemberDeclaration(product), product_fingerprint, input_fingerprint)
        }
        QueryKey::ModuleStaticBindingCatalog(module) => {
            let catalog = if let Some(index) = snapshot.module_static_bindings.get(module) {
                Arc::new(crate::module_bindings::ModuleStaticBindingCatalog::from_index(index))
            } else {
                Arc::new(crate::module_bindings::ModuleStaticBindingCatalog::empty(module.clone()))
            };
            let product_fingerprint = catalog.product_fingerprint();
            let input_fingerprint = catalog.input_fingerprint();
            (SemanticProduct::ModuleStaticBindingCatalog(catalog), product_fingerprint, input_fingerprint)
        }
        QueryKey::ModuleStaticBindings(module) => {
            let product = snapshot
                .module_static_bindings
                .get(module)
                .cloned()
                .ok_or_else(|| incomplete("missing module static bindings product for portable owner"))?;
            let product_fingerprint = product.product_fingerprint();
            let input_fingerprint = product.input_fingerprint();
            (SemanticProduct::ModuleStaticBindings(product), product_fingerprint, input_fingerprint)
        }
        QueryKey::HierarchyEdge(class_decl) => {
            let super_decl = snapshot.hierarchy.superclass(class_decl).cloned();
            let product = Arc::new(crate::hierarchy_product::HierarchyEdgeProduct::new(class_decl.clone(), super_decl.clone()));
            let product_fingerprint = crate::db::fingerprint::hierarchy_edge_product_fingerprint(class_decl, &super_decl, None);
            let input_fingerprint = crate::db::fingerprint::hierarchy_edge_input_fingerprint(class_decl, None, &super_decl);
            (SemanticProduct::HierarchyEdge(product), product_fingerprint, input_fingerprint)
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
    let limits = phalcom_compiler_artifact::ValidationLimits::default();
    let budget_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = AllocationBudget::new(budget_limit);
    reconstruct_query_image_with_allocation_budget(image, snapshot, world, modules, &limits, &mut allocations)
}

/// Reconstructs query authority using the caller's cumulative load transaction.
pub fn reconstruct_query_image_with_allocation_budget(
    image: &QueryImageV1,
    snapshot: &Arc<SemanticSnapshot>,
    world: &CanonicalSemanticWorldId,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
    limits: &phalcom_compiler_artifact::ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<(Arc<CanonicalSemanticQueryImage>, BTreeMap<QueryKey, ProductFingerprint>), CanonicalImageError> {
    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = transaction.child(section_limit);
    let reconstructed = reconstruct_query_image_inner(image, snapshot, world, modules, limits, &mut allocations)?;
    transaction.commit_child(allocations).map_err(allocation_error)?;
    Ok(reconstructed)
}

fn allocation_error(error: AllocationError) -> CanonicalImageError {
    CanonicalImageError::Artifact(ArtifactError::Allocation(error))
}

fn reconstruct_query_image_inner(
    image: &QueryImageV1,
    snapshot: &Arc<SemanticSnapshot>,
    world: &CanonicalSemanticWorldId,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
    limits: &phalcom_compiler_artifact::ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<(Arc<CanonicalSemanticQueryImage>, BTreeMap<QueryKey, ProductFingerprint>), CanonicalImageError> {
    validate_query_image_closure_with_allocation_budget(image, limits, allocations)?;

    let mut roots = allocations
        .vector::<(QueryKey, Arc<SemanticProduct>, ProductFingerprint, InputFingerprint)>(image.supported_roots.len())
        .map_err(allocation_error)?;

    for root in &image.supported_roots {
        let query_key = resolve_query_key(&root.key, allocations)?;

        let (product, product_fingerprint, input_fingerprint) = product_and_fingerprints(snapshot, &query_key, modules)?;
        roots.push((query_key, product, product_fingerprint, input_fingerprint));
    }

    let mut entries = BTreeMap::new();
    let mut fingerprints = BTreeMap::new();
    for (idx, root) in image.supported_roots.iter().enumerate() {
        let (query_key, product, product_fingerprint, input_fingerprint) = &roots[idx];
        let mut edges = allocations.vector::<DependencyEdge>(root.dependency_indices.len()).map_err(allocation_error)?;
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
        if entries
            .insert(
                query_key.clone(),
                CanonicalQueryEntry {
                    input_fingerprint: *input_fingerprint,
                    product_fingerprint: *product_fingerprint,
                    dependencies: Arc::from(edges.into_boxed_slice()),
                    product: product.clone(),
                },
            )
            .is_some()
        {
            return Err(incomplete("distinct portable query roots resolve to the same semantic owner"));
        }
        fingerprints.insert(query_key.clone(), *product_fingerprint);
    }

    let query_image =
        CanonicalSemanticQueryImage::from_loaded_artifact_entries(world.clone(), entries, snapshot.store.clone(), snapshot.static_values.clone())?;
    let query_image = allocations.arc(query_image).map_err(allocation_error)?;
    Ok((query_image, fingerprints))
}
