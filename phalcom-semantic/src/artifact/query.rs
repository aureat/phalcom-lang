//! Query image validation, reverse-pruning, and reconstruction (`PERF003.C2.P7.T9`).

use crate::artifact::import::import_stable_callable_with_allocation_budget;
use crate::canonical_image::{
    CanonicalDependencyFingerprints, CanonicalImageError, CanonicalQueryEntries, CanonicalQueryEntry, CanonicalSemanticQueryImage, CanonicalSemanticWorldId,
};
use crate::db::key::QueryKey;
use crate::db::product::{CallableSignatureProduct, DeclarationSurfaceProduct, SemanticProduct};
use crate::db::{DependencyEdge, InputFingerprint, ProductFingerprint, QueryKind};
use crate::snapshot::SemanticSnapshot;
use phalcom_compiler_artifact::ArtifactError;
use phalcom_compiler_artifact::error::QueryImageError;
use phalcom_compiler_artifact::query::{
    LegacyPortableQueryKeyV1, PortableQueryKeyV1, PortableQueryKindV1, PortableQueryOwnerKeyV1, QueryImageV2, QueryImageV4, QueryProfileSupport,
};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::budgeted::BudgetDecode;
use phalcom_type_meta::identity::{StableCallableRef, StableDeclarationRef, StableFieldRef, StableModuleRef, StableProjectRef};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const QUERY_RECONSTRUCTION_CENSUS_PHASES: usize = 6;
const QUERY_KIND_CENSUS_KINDS: [QueryKind; QueryKind::ALL.len()] = QueryKind::ALL;
static QUERY_RECONSTRUCTION_CENSUS_ACTIVE: AtomicBool = AtomicBool::new(false);
static QUERY_RECONSTRUCTION_CENSUS_OBSERVED: [AtomicUsize; QUERY_RECONSTRUCTION_CENSUS_PHASES] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
static QUERY_RECONSTRUCTION_CENSUS_LEDGER: [AtomicUsize; QUERY_RECONSTRUCTION_CENSUS_PHASES] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
static QUERY_RECONSTRUCTION_CENSUS_ALLOCATOR_BEFORE: [AtomicUsize; QUERY_RECONSTRUCTION_CENSUS_PHASES] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
static QUERY_RECONSTRUCTION_CENSUS_ALLOCATOR_AFTER: [AtomicUsize; QUERY_RECONSTRUCTION_CENSUS_PHASES] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
static QUERY_RECONSTRUCTION_CENSUS_LEDGER_BEFORE: [AtomicUsize; QUERY_RECONSTRUCTION_CENSUS_PHASES] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
static QUERY_RECONSTRUCTION_CENSUS_LEDGER_AFTER: [AtomicUsize; QUERY_RECONSTRUCTION_CENSUS_PHASES] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
static QUERY_PRODUCT_KIND_OBSERVED: [AtomicUsize; QUERY_KIND_CENSUS_KINDS.len()] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
static QUERY_PRODUCT_KIND_LEDGER: [AtomicUsize; QUERY_KIND_CENSUS_KINDS.len()] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
static QUERY_PRODUCT_BUFFER_OBSERVED: AtomicUsize = AtomicUsize::new(0);
static QUERY_PRODUCT_BUFFER_LEDGER: AtomicUsize = AtomicUsize::new(0);

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QueryReconstructionCensusRow {
    pub allocator_before: usize,
    pub allocator_after: usize,
    pub observed_bytes: usize,
    pub ledger_before: usize,
    pub ledger_after: usize,
    pub ledger_bytes: usize,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryProductKindCensusRow {
    pub kind: QueryKind,
    pub observed_bytes: usize,
    pub ledger_bytes: usize,
}

#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryReconstructionCensus {
    pub closure_and_roots: QueryReconstructionCensusRow,
    pub owner_key_resolution: QueryReconstructionCensusRow,
    pub product_reconstruction: QueryReconstructionCensusRow,
    pub entry_assembly: QueryReconstructionCensusRow,
    pub loaded_validation: QueryReconstructionCensusRow,
    pub query_image_publication: QueryReconstructionCensusRow,
    pub product_by_kind: Vec<QueryProductKindCensusRow>,
    pub product_buffer_observed: usize,
    pub product_buffer_ledger: usize,
}

#[doc(hidden)]
pub fn reset_query_reconstruction_census() {
    QUERY_RECONSTRUCTION_CENSUS_ACTIVE.store(false, Ordering::Relaxed);
    for row in QUERY_RECONSTRUCTION_CENSUS_OBSERVED
        .iter()
        .chain(QUERY_RECONSTRUCTION_CENSUS_LEDGER.iter())
        .chain(QUERY_RECONSTRUCTION_CENSUS_ALLOCATOR_BEFORE.iter())
        .chain(QUERY_RECONSTRUCTION_CENSUS_ALLOCATOR_AFTER.iter())
        .chain(QUERY_RECONSTRUCTION_CENSUS_LEDGER_BEFORE.iter())
        .chain(QUERY_RECONSTRUCTION_CENSUS_LEDGER_AFTER.iter())
    {
        row.store(0, Ordering::Relaxed);
    }
    for row in QUERY_PRODUCT_KIND_OBSERVED.iter().chain(QUERY_PRODUCT_KIND_LEDGER.iter()) {
        row.store(0, Ordering::Relaxed);
    }
    QUERY_PRODUCT_BUFFER_OBSERVED.store(0, Ordering::Relaxed);
    QUERY_PRODUCT_BUFFER_LEDGER.store(0, Ordering::Relaxed);
    QUERY_RECONSTRUCTION_CENSUS_ACTIVE.store(true, Ordering::Relaxed);
}

#[doc(hidden)]
pub fn take_query_reconstruction_census() -> QueryReconstructionCensus {
    QUERY_RECONSTRUCTION_CENSUS_ACTIVE.store(false, Ordering::Relaxed);
    let row = |index: usize| QueryReconstructionCensusRow {
        allocator_before: QUERY_RECONSTRUCTION_CENSUS_ALLOCATOR_BEFORE[index].load(Ordering::Relaxed),
        allocator_after: QUERY_RECONSTRUCTION_CENSUS_ALLOCATOR_AFTER[index].load(Ordering::Relaxed),
        observed_bytes: QUERY_RECONSTRUCTION_CENSUS_OBSERVED[index].load(Ordering::Relaxed),
        ledger_before: QUERY_RECONSTRUCTION_CENSUS_LEDGER_BEFORE[index].load(Ordering::Relaxed),
        ledger_after: QUERY_RECONSTRUCTION_CENSUS_LEDGER_AFTER[index].load(Ordering::Relaxed),
        ledger_bytes: QUERY_RECONSTRUCTION_CENSUS_LEDGER[index].load(Ordering::Relaxed),
    };
    let product_by_kind = QUERY_KIND_CENSUS_KINDS
        .iter()
        .enumerate()
        .filter_map(|(index, kind)| {
            let observed_bytes = QUERY_PRODUCT_KIND_OBSERVED[index].load(Ordering::Relaxed);
            let ledger_bytes = QUERY_PRODUCT_KIND_LEDGER[index].load(Ordering::Relaxed);
            (observed_bytes != 0 || ledger_bytes != 0).then_some(QueryProductKindCensusRow {
                kind: *kind,
                observed_bytes,
                ledger_bytes,
            })
        })
        .collect();
    QueryReconstructionCensus {
        closure_and_roots: row(0),
        owner_key_resolution: row(1),
        product_reconstruction: row(2),
        entry_assembly: row(3),
        loaded_validation: row(4),
        query_image_publication: row(5),
        product_by_kind,
        product_buffer_observed: QUERY_PRODUCT_BUFFER_OBSERVED.load(Ordering::Relaxed),
        product_buffer_ledger: QUERY_PRODUCT_BUFFER_LEDGER.load(Ordering::Relaxed),
    }
}

fn census_checkpoint(allocations: &AllocationBudget) -> (Option<usize>, usize) {
    if QUERY_RECONSTRUCTION_CENSUS_ACTIVE.load(Ordering::Relaxed) {
        (phalcom_type_meta::allocation::current_observed_bytes(), allocations.requested())
    } else {
        (None, allocations.requested())
    }
}

fn record_census_phase(phase: usize, before: (Option<usize>, usize), allocations: &AllocationBudget) {
    if !QUERY_RECONSTRUCTION_CENSUS_ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    let observed_after = phalcom_type_meta::allocation::current_observed_bytes();
    if let (Some(start), Some(end)) = (before.0, observed_after) {
        QUERY_RECONSTRUCTION_CENSUS_OBSERVED[phase].fetch_add(end - start, Ordering::Relaxed);
        QUERY_RECONSTRUCTION_CENSUS_ALLOCATOR_BEFORE[phase].store(start, Ordering::Relaxed);
        QUERY_RECONSTRUCTION_CENSUS_ALLOCATOR_AFTER[phase].store(end, Ordering::Relaxed);
    }
    let ledger_after = allocations.requested();
    QUERY_RECONSTRUCTION_CENSUS_LEDGER[phase].fetch_add(ledger_after - before.1, Ordering::Relaxed);
    QUERY_RECONSTRUCTION_CENSUS_LEDGER_BEFORE[phase].store(before.1, Ordering::Relaxed);
    QUERY_RECONSTRUCTION_CENSUS_LEDGER_AFTER[phase].store(ledger_after, Ordering::Relaxed);
}

fn record_product_kind_census(kind: QueryKind, before: (Option<usize>, usize), allocations: &AllocationBudget) {
    if !QUERY_RECONSTRUCTION_CENSUS_ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    let Some(index) = QUERY_KIND_CENSUS_KINDS.iter().position(|candidate| *candidate == kind) else {
        return;
    };
    if let (Some(start), Some(end)) = (before.0, phalcom_type_meta::allocation::current_observed_bytes()) {
        QUERY_PRODUCT_KIND_OBSERVED[index].fetch_add(end - start, Ordering::Relaxed);
    }
    QUERY_PRODUCT_KIND_LEDGER[index].fetch_add(allocations.requested() - before.1, Ordering::Relaxed);
}

fn incomplete(message: impl Into<String>) -> CanonicalImageError {
    CanonicalImageError::IncompletePublication(message.into())
}

/// Validates that the legacy query image contains no source-dependent roots, has closed
/// acyclic dependencies, and separates aggregate dependency evidence (`P7.T9`).
/// Scratch is charged against the caller's load transaction before allocation.
pub fn validate_query_image_closure_with_allocation_budget(
    image: &QueryImageV2,
    limits: &phalcom_compiler_artifact::ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<(), ArtifactError> {
    let len = image.supported_roots.len();
    let root_limit = limits.max_semantic_entries_per_family.min(limits.max_query_roots);
    if len > root_limit {
        return Err(QueryImageError::CountLimitExceeded.into());
    }

    for (idx, root) in image.supported_roots.iter().enumerate() {
        if !crate::artifact::query_policy::query_root_is_reusable(crate::artifact::query_policy::query_kind_from_portable(root.key.kind)) {
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
    let mut ordered_keys = scratch.vector::<&LegacyPortableQueryKeyV1>(len).map_err(ArtifactError::from)?;
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
pub fn validate_query_image_closure(image: &QueryImageV2) -> Result<(), ArtifactError> {
    let limits = phalcom_compiler_artifact::ValidationLimits::default();
    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| AllocationError::Overflow)?;
    let mut budget = AllocationBudget::new(section_limit);
    validate_query_image_closure_with_allocation_budget(image, &limits, &mut budget)
}

fn decode_owner<T: phalcom_type_meta::budgeted::BudgetDecode>(
    key: &LegacyPortableQueryKeyV1,
    tag: &str,
    allocations: &mut AllocationBudget,
) -> Result<T, CanonicalImageError> {
    let prefix_len = tag.len() + 1;
    let prefix = key
        .stable_owner_key
        .get(..prefix_len)
        .ok_or_else(|| incomplete("query owner key has wrong type tag"))?;
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
        StableProjectRef::Builtin { namespace, version } if namespace.as_ref() == "universe" && version.as_ref() == "0.1.0" => {
            phalcom_modules::ProjectIdentity::Universe
        }
        StableProjectRef::Session { .. } => return Err(incomplete("session-owned query roots require an exact StableOwnerCatalog binding")),
        _ => return Err(incomplete("unsupported stable project ref for direct conversion")),
    })
}

fn consume_stable_module(
    module: phalcom_type_meta::identity::StableModuleRef,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_modules::ModuleId, CanonicalImageError> {
    let project = consume_stable_project(module.project)?;
    let mut components = allocations
        .vector::<phalcom_modules::ModuleComponent>(module.path.len())
        .map_err(allocation_error)?;
    for spelling in Vec::from(module.path) {
        let component =
            phalcom_modules::ModuleComponent::from_owned_identifier(spelling).map_err(|_| incomplete("invalid module component in stable query owner"))?;
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

fn consume_stable_field(field: StableFieldRef, allocations: &mut AllocationBudget) -> Result<crate::identity::FieldId, CanonicalImageError> {
    let owner = consume_stable_declaration(field.owner, allocations)?;
    Ok(crate::identity::FieldId {
        owner,
        name: field.name,
        side: crate::metadata::stable_identity::from_stable_dispatch_side(field.side),
    })
}

fn resolve_query_key(key: &LegacyPortableQueryKeyV1, allocations: &mut AllocationBudget) -> Result<QueryKey, CanonicalImageError> {
    Ok(match key.kind {
        PortableQueryKindV1::UnlinkedInterface => QueryKey::UnlinkedInterface(consume_stable_module(decode_owner(key, "module", allocations)?, allocations)?),
        PortableQueryKindV1::ResolvedImports => QueryKey::ResolvedImports(consume_stable_module(decode_owner(key, "module", allocations)?, allocations)?),
        PortableQueryKindV1::LinkedInterface => QueryKey::LinkedInterface(consume_stable_module(decode_owner(key, "module", allocations)?, allocations)?),
        PortableQueryKindV1::DeclarationShell => QueryKey::DeclarationShell(consume_stable_declaration(
            decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?,
            allocations,
        )?),
        PortableQueryKindV1::CallableDefinition => QueryKey::CallableDefinition(import_stable_callable_with_allocation_budget(
            &decode_owner::<StableCallableRef>(key, "callable", allocations)?,
            allocations,
        )?),
        PortableQueryKindV1::CallableSignature => QueryKey::CallableSignature(import_stable_callable_with_allocation_budget(
            &decode_owner::<StableCallableRef>(key, "callable", allocations)?,
            allocations,
        )?),
        PortableQueryKindV1::FieldSignature => {
            QueryKey::FieldSignature(consume_stable_field(decode_owner::<StableFieldRef>(key, "field", allocations)?, allocations)?)
        }
        PortableQueryKindV1::DeclarationSurface => QueryKey::DeclarationSurface(consume_stable_declaration(
            decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?,
            allocations,
        )?),
        PortableQueryKindV1::TraitHeader => QueryKey::TraitHeader(consume_stable_declaration(
            decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?,
            allocations,
        )?),
        PortableQueryKindV1::TraitSurface => QueryKey::TraitSurface(consume_stable_declaration(
            decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?,
            allocations,
        )?),
        PortableQueryKindV1::DataDeclaration => QueryKey::DataDeclaration(consume_stable_declaration(
            decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?,
            allocations,
        )?),
        PortableQueryKindV1::EnumDeclaration => QueryKey::EnumDeclaration(consume_stable_declaration(
            decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?,
            allocations,
        )?),
        PortableQueryKindV1::AssociatedSurface => QueryKey::AssociatedSurface(consume_stable_declaration(
            decode_owner::<StableDeclarationRef>(key, "declaration", allocations)?,
            allocations,
        )?),
        PortableQueryKindV1::TypeMemberDeclaration => {
            let prefix_len = "type-member:".len();
            let payload = key
                .stable_owner_key
                .get(prefix_len..)
                .ok_or_else(|| incomplete("query owner key has wrong type tag"))?;
            if key.stable_owner_key.get(..prefix_len) != Some("type-member:") {
                return Err(incomplete("query owner key has wrong type tag"));
            }
            let mut decoder = phalcom_type_meta::budgeted::Decoder::new(payload, allocations, 256)
                .map_err(|error| incomplete(format!("invalid portable query owner key for type-member: {error}: {payload}")))?;
            decoder
                .cursor
                .expect(b'[')
                .map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            let owner = StableDeclarationRef::decode(&mut decoder).map_err(|error| match error {
                phalcom_type_meta::budgeted::DecodeError::Allocation(error) => allocation_error(error),
                error => incomplete(format!("invalid type-member stable declaration: {error}: {payload}")),
            })?;
            decoder
                .cursor
                .expect(b',')
                .map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            decoder
                .cursor
                .expect(b'[')
                .map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            let selector_tag = u8::decode(&mut decoder).map_err(|error| match error {
                phalcom_type_meta::budgeted::DecodeError::Allocation(error) => allocation_error(error),
                error => incomplete(format!("invalid type-member selector tag: {error}: {payload}")),
            })?;
            decoder
                .cursor
                .expect(b',')
                .map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            let selector_name = String::decode(&mut decoder).map_err(|error| match error {
                phalcom_type_meta::budgeted::DecodeError::Allocation(error) => allocation_error(error),
                error => incomplete(format!("invalid portable query owner key: {error}")),
            })?;
            decoder
                .cursor
                .expect(b']')
                .map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            decoder
                .cursor
                .expect(b']')
                .map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
            decoder
                .cursor
                .finish()
                .map_err(|error| incomplete(format!("invalid portable query owner key: {error}")))?;
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
        _ => return Err(incomplete("legacy query image does not support this current query kind")),
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

fn format_import_path_with_allocation_budget(path: &phalcom_ast::ast::ImportPath, allocations: &mut AllocationBudget) -> Result<String, CanonicalImageError> {
    use phalcom_ast::ast::ImportRoot;

    let mut len = match &path.root {
        ImportRoot::Absolute(segment) => segment.name.len(),
        ImportRoot::Relative { dots, .. } => usize::from(*dots),
    };
    for (index, segment) in path.segments.iter().enumerate() {
        if index > 0 || matches!(path.root, ImportRoot::Absolute(_)) {
            len = len.checked_add(1).ok_or_else(|| allocation_error(AllocationError::Overflow))?;
        }
        len = len.checked_add(segment.name.len()).ok_or_else(|| allocation_error(AllocationError::Overflow))?;
    }

    let mut bytes = allocations.vector::<u8>(len).map_err(allocation_error)?;
    match &path.root {
        ImportRoot::Absolute(segment) => bytes.extend_from_slice(segment.name.as_bytes()),
        ImportRoot::Relative { dots, .. } => bytes.extend(std::iter::repeat_n(b'.', usize::from(*dots))),
    }
    for (index, segment) in path.segments.iter().enumerate() {
        if index > 0 || matches!(path.root, ImportRoot::Absolute(_)) {
            bytes.push(b'.');
        }
        bytes.extend_from_slice(segment.name.as_bytes());
    }
    String::from_utf8(bytes).map_err(|_| allocation_error(AllocationError::Failed))
}

fn product_and_fingerprints(
    snapshot: &SemanticSnapshot,
    key: &QueryKey,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
    allocations: &mut AllocationBudget,
) -> Result<(Arc<SemanticProduct>, ProductFingerprint, InputFingerprint), CanonicalImageError> {
    use crate::db::fingerprint as fp;
    let empty_diagnostics = allocations
        .arc_slice(Vec::<crate::diagnostic::SemanticDiagnostic>::new())
        .map_err(allocation_error)?;
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
            let interface = linked.interface.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
            let interface = allocations.arc(interface).map_err(allocation_error)?;
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
            let mut import_entries = allocations
                .vector::<(String, crate::identity::ModuleId)>(interface.imports.len())
                .map_err(allocation_error)?;
            for import in &interface.imports {
                let edge = references
                    .iter()
                    .find(|edge| edge.range == import.range())
                    .ok_or_else(|| incomplete("resolved import has no matching owner edge"))?;
                let path = format_import_path_with_allocation_budget(import.path(), allocations)?;
                import_entries.push((path, edge.to.clone_with_allocation_budget(allocations).map_err(allocation_error)?));
            }
            let imports = phalcom_modules::authority::ModuleAuthorityMap::from_budgeted_entries(import_entries, allocations).map_err(|error| match error {
                phalcom_modules::authority::ModuleAuthorityBuildError::Allocation(error) => allocation_error(error),
                phalcom_modules::authority::ModuleAuthorityBuildError::Entries(_) | phalcom_modules::authority::ModuleAuthorityBuildError::SetEntries(_) => {
                    incomplete("duplicate resolved import path in portable owner")
                }
            })?;
            let product = crate::module_product::ResolvedImportsProduct::new(
                module.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                imports,
                Vec::new(),
            );
            let product = allocations.arc(product).map_err(allocation_error)?;
            let product_fingerprint = fp::resolved_imports_product_fingerprint(&product);
            let input_fingerprint = fp::unlinked_interface_input_fingerprint(interface);
            (SemanticProduct::ResolvedImports(product), product_fingerprint, input_fingerprint)
        }
        QueryKey::DeclarationShell(declaration) => {
            let shell = if let Some(info) = snapshot.declarations.get(declaration) {
                crate::declarations::TypeDeclarationShell::Nominal(info.clone_with_allocation_budget(allocations).map_err(allocation_error)?)
            } else if let Some(alias) = snapshot.type_aliases.get(declaration) {
                crate::declarations::TypeDeclarationShell::Alias(alias.clone_with_allocation_budget(allocations).map_err(allocation_error)?)
            } else {
                return Err(incomplete("missing declaration shell product for portable owner"));
            };
            let product_fingerprint = fp::declaration_shell_product_fingerprint(&shell);
            let input_fingerprint = fp::declaration_shell_input_fingerprint(&shell);
            (
                SemanticProduct::DeclarationShell(allocations.arc(shell).map_err(allocation_error)?),
                product_fingerprint,
                input_fingerprint,
            )
        }
        QueryKey::DeclarationSurface(declaration) => {
            let surface = snapshot
                .dispatch
                .get_surface(declaration)
                .ok_or_else(|| incomplete("missing declaration surface product for portable owner"))?;
            let surface = surface.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
            let product_fingerprint = fp::declaration_surface_product_fingerprint(&surface);
            let input_fingerprint = fp::declaration_surface_query_input_fingerprint(&surface, &[]);
            let product = DeclarationSurfaceProduct {
                surface: allocations.arc(surface).map_err(allocation_error)?,
                diagnostics: empty_diagnostics,
            };
            (
                SemanticProduct::DeclarationSurface(allocations.arc(product).map_err(allocation_error)?),
                product_fingerprint,
                input_fingerprint,
            )
        }
        QueryKey::CallableSignature(callable) => {
            let signature = snapshot
                .callable_signatures
                .get(callable)
                .ok_or_else(|| incomplete("missing callable signature product for portable owner"))?;
            let signature = signature.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
            let product_fingerprint = fp::callable_signature_product_fingerprint(&signature);
            let input_fingerprint = fp::callable_signature_input_fingerprint(&signature);
            let signature = allocations.arc(signature).map_err(allocation_error)?;
            let product = CallableSignatureProduct::new(signature, empty_diagnostics);
            (
                SemanticProduct::CallableSignature(allocations.arc(product).map_err(allocation_error)?),
                product_fingerprint,
                input_fingerprint,
            )
        }
        QueryKey::CallableDefinition(callable) => {
            let definition = snapshot
                .callable_definitions
                .get(callable)
                .ok_or_else(|| incomplete("missing callable definition product for portable owner"))?;
            let definition = definition.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
            let fingerprint = callable_definition_fingerprint(&definition);
            (
                SemanticProduct::CallableDefinition(allocations.arc(definition).map_err(allocation_error)?),
                fingerprint,
                InputFingerprint::new(fingerprint.raw()),
            )
        }
        QueryKey::FieldSignature(field) => {
            let signature = snapshot
                .field_signatures
                .get(field)
                .ok_or_else(|| incomplete("missing field signature product for portable owner"))?;
            let signature = signature.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
            let product_fingerprint = fp::field_signature_product_fingerprint(&signature);
            let input_fingerprint = fp::field_signature_input_fingerprint(&signature);
            let signature = allocations.arc(signature).map_err(allocation_error)?;
            let product = crate::db::product::FieldSignatureProduct::new(signature, empty_diagnostics);
            (
                SemanticProduct::FieldSignature(allocations.arc(product).map_err(allocation_error)?),
                product_fingerprint,
                input_fingerprint,
            )
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
            (
                SemanticProduct::DataDeclaration(allocations.arc(product).map_err(allocation_error)?),
                product_fingerprint,
                input_fingerprint,
            )
        }
        QueryKey::EnumDeclaration(declaration) => {
            let info = snapshot
                .enum_semantics
                .enums
                .get(declaration)
                .map(|info| info.clone_with_allocation_budget(allocations))
                .transpose()
                .map_err(allocation_error)?
                .ok_or_else(|| incomplete("missing enum declaration product for portable owner"))?;
            let info = allocations.arc(info).map_err(allocation_error)?;
            let mut variants = allocations.vector(info.variants.len()).map_err(allocation_error)?;
            for variant in info.variants.iter() {
                let value = snapshot
                    .enum_semantics
                    .variants
                    .get(variant)
                    .ok_or_else(|| incomplete("missing enum variant product for portable owner"))?;
                let value = value.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
                variants.push(allocations.arc(value).map_err(allocation_error)?);
            }
            let variants = allocations.arc_slice(variants).map_err(allocation_error)?;
            let product = crate::db::product::EnumDeclarationProduct {
                info,
                variants,
                diagnostics: empty_diagnostics,
            };
            let product_fingerprint = fp::enum_declaration_product_fingerprint(&product);
            let input_fingerprint = fp::enum_declaration_input_fingerprint(&product);
            (
                SemanticProduct::EnumDeclaration(allocations.arc(product).map_err(allocation_error)?),
                product_fingerprint,
                input_fingerprint,
            )
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
                crate::module_bindings::ModuleStaticBindingCatalog::from_index_with_allocation_budget(index, allocations).map_err(|error| match error {
                    crate::module_bindings::ModuleStaticBindingCatalogBuildError::Allocation(error) => allocation_error(error),
                    crate::module_bindings::ModuleStaticBindingCatalogBuildError::DuplicateKey => incomplete("duplicate module static binding catalog entry"),
                })?
            } else {
                crate::module_bindings::ModuleStaticBindingCatalog::empty_with_allocation_budget(module, allocations).map_err(allocation_error)?
            };
            let input_fingerprint = catalog.input_fingerprint_with_allocation_budget(allocations).map_err(allocation_error)?;
            let product_fingerprint = ProductFingerprint::new(input_fingerprint.raw());
            let catalog = allocations.arc(catalog).map_err(allocation_error)?;
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
            let product_fingerprint = crate::db::fingerprint::hierarchy_edge_product_fingerprint(class_decl, super_decl.as_ref(), None);
            let input_fingerprint = crate::db::fingerprint::hierarchy_edge_input_fingerprint(class_decl, None, &super_decl);
            (SemanticProduct::HierarchyEdge(product), product_fingerprint, input_fingerprint)
        }
        _ => return Err(incomplete("query kind has no loaded semantic owner adapter")),
    };
    Ok((allocations.arc(result.0).map_err(allocation_error)?, result.1, result.2))
}

/// Reconstructs an immutable query image. Every wire root and edge must map to
/// a typed live owner and a current product; no row is skipped or assigned a
/// synthetic zero fingerprint.
pub fn reconstruct_query_image(
    image: &QueryImageV2,
    snapshot: &Arc<SemanticSnapshot>,
    world: &CanonicalSemanticWorldId,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
) -> Result<(Arc<CanonicalSemanticQueryImage>, CanonicalDependencyFingerprints), CanonicalImageError> {
    let limits = phalcom_compiler_artifact::ValidationLimits::default();
    let budget_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = AllocationBudget::new(budget_limit);
    reconstruct_query_image_with_allocation_budget(image, snapshot, world, modules, &limits, &mut allocations)
}

/// Reconstructs query authority using the caller's cumulative load transaction.
pub fn reconstruct_query_image_with_allocation_budget(
    image: &QueryImageV2,
    snapshot: &Arc<SemanticSnapshot>,
    world: &CanonicalSemanticWorldId,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
    limits: &phalcom_compiler_artifact::ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<(Arc<CanonicalSemanticQueryImage>, CanonicalDependencyFingerprints), CanonicalImageError> {
    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = transaction.child(section_limit);
    let reconstructed = reconstruct_query_image_inner(image, snapshot, world, modules, limits, &mut allocations)?;
    transaction.commit_child(allocations).map_err(allocation_error)?;
    Ok(reconstructed)
}

/// Reconstructs a current V3 query image into the P7 immutable image using
/// current portable owners and fresh receiving-workspace fingerprints.
/// The caller must first validate its product references and portable
/// structural fingerprints against the artifact's MODULE_WORLD and
/// SEMANTIC_WORLD sections.
pub fn reconstruct_query_image_v4_with_allocation_budget(
    image: QueryImageV4,
    snapshot: &Arc<SemanticSnapshot>,
    world: &CanonicalSemanticWorldId,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
    limits: &phalcom_compiler_artifact::ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<(Arc<CanonicalSemanticQueryImage>, CanonicalDependencyFingerprints), CanonicalImageError> {
    reconstruct_query_image_v4_with_owner_catalog_and_allocation_budget(image, snapshot, world, modules, None, limits, transaction)
}

/// Package-aware V4 query reconstruction. No project owner is recovered from
/// display names or arbitrary artifact bytes.
pub fn reconstruct_query_image_v4_with_project_catalog_and_allocation_budget(
    image: QueryImageV4,
    snapshot: &Arc<SemanticSnapshot>,
    world: &CanonicalSemanticWorldId,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
    module_world: &phalcom_compiler_artifact::module_world_v5::ModuleWorldV5,
    limits: &phalcom_compiler_artifact::ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<(Arc<CanonicalSemanticQueryImage>, CanonicalDependencyFingerprints), CanonicalImageError> {
    let catalog = crate::artifact::StableOwnerCatalog::new(module_world, modules.linked().universe.as_ref())
        .map_err(CanonicalImageError::Artifact)?;
    let _ambient_owners = crate::artifact::identity::ambient::AmbientOwnerScope::enter(module_world, modules.linked().universe.as_ref());
    reconstruct_query_image_v4_with_owner_catalog_and_allocation_budget(image, snapshot, world, modules, Some(&catalog), limits, transaction)
}

fn reconstruct_query_image_v4_with_owner_catalog_and_allocation_budget(
    image: QueryImageV4,
    snapshot: &Arc<SemanticSnapshot>,
    world: &CanonicalSemanticWorldId,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
    owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>,
    limits: &phalcom_compiler_artifact::ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<(Arc<CanonicalSemanticQueryImage>, CanonicalDependencyFingerprints), CanonicalImageError> {
    use crate::artifact::query_policy::{CanonicalQueryPersistence, canonical_query_persistence, query_kind_from_portable, query_root_is_reusable};

    image.validate(limits).map_err(CanonicalImageError::Artifact)?;

    for root in &image.supported_roots {
        if !query_root_is_reusable(query_kind_from_portable(root.key.kind)) {
            return Err(incomplete("query image contains a root outside the current reusable-root policy"));
        }
    }
    for evidence in &image.dependency_evidence {
        let kind = query_kind_from_portable(evidence.key.kind);
        if !matches!(canonical_query_persistence(kind), CanonicalQueryPersistence::EligibleStructural) && kind != QueryKind::HierarchyEdge {
            return Err(incomplete("query dependency evidence is outside the current persistence policy"));
        }
        if evidence.key.owner != evidence.stable_aggregate_owner {
            return Err(incomplete("query dependency key and aggregate owner differ"));
        }
    }

    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = transaction.child(section_limit);
    let root_start = census_checkpoint(&allocations);
    let mut root_keys = allocations.vector::<QueryKey>(image.supported_roots.len()).map_err(allocation_error)?;
    for root in &image.supported_roots {
        root_keys.push(resolve_query_key_v3(&root.key, &mut allocations, owner_catalog)?);
    }
    let mut evidence_keys = allocations.vector::<QueryKey>(image.dependency_evidence.len()).map_err(allocation_error)?;
    for evidence in &image.dependency_evidence {
        evidence_keys.push(resolve_query_key_v3(&evidence.key, &mut allocations, owner_catalog)?);
    }
    let key_count = root_keys
        .len()
        .checked_add(evidence_keys.len())
        .ok_or_else(|| allocation_error(AllocationError::Overflow))?;
    let mut all_keys = allocations.vector::<QueryKey>(key_count).map_err(allocation_error)?;
    for key in root_keys.iter().chain(evidence_keys.iter()) {
        all_keys.push(clone_loaded_query_key_with_allocation_budget(key, &mut allocations)?);
    }
    all_keys.sort_unstable();
    all_keys.dedup();

    let mut root_key_indices = allocations.vector::<usize>(root_keys.len()).map_err(allocation_error)?;
    for key in &root_keys {
        root_key_indices.push(
            all_keys
                .binary_search(key)
                .map_err(|_| incomplete("query root owner was lost during canonical sorting"))?,
        );
    }
    let mut evidence_key_indices = allocations.vector::<usize>(evidence_keys.len()).map_err(allocation_error)?;
    for key in &evidence_keys {
        evidence_key_indices.push(
            all_keys
                .binary_search(key)
                .map_err(|_| incomplete("query dependency owner was lost during canonical sorting"))?,
        );
    }

    let mut root_for_key = allocations.vector::<Option<usize>>(all_keys.len()).map_err(allocation_error)?;
    root_for_key.resize(all_keys.len(), None);
    for (root_index, key_index) in root_key_indices.iter().copied().enumerate() {
        if root_for_key[key_index].replace(root_index).is_some() {
            return Err(incomplete("multiple portable query roots relocate to the same live query owner"));
        }
    }
    let mut sorted_evidence_key_indices = allocations.vector::<usize>(evidence_key_indices.len()).map_err(allocation_error)?;
    sorted_evidence_key_indices.extend_from_slice(&evidence_key_indices);
    sorted_evidence_key_indices.sort_unstable();
    for adjacent in sorted_evidence_key_indices.windows(2) {
        if adjacent[0] == adjacent[1] {
            return Err(incomplete("multiple portable dependency evidence rows relocate to the same live query owner"));
        }
    }
    record_census_phase(1, root_start, &allocations);

    let product_start = census_checkpoint(&allocations);
    let product_buffer_start = census_checkpoint(&allocations);
    let mut live_products = allocations
        .vector::<(Arc<SemanticProduct>, ProductFingerprint, InputFingerprint)>(all_keys.len())
        .map_err(allocation_error)?;
    if QUERY_RECONSTRUCTION_CENSUS_ACTIVE.load(Ordering::Relaxed) {
        if let (Some(start), Some(end)) = (product_buffer_start.0, phalcom_type_meta::allocation::current_observed_bytes()) {
            QUERY_PRODUCT_BUFFER_OBSERVED.fetch_add(end - start, Ordering::Relaxed);
        }
        QUERY_PRODUCT_BUFFER_LEDGER.fetch_add(allocations.requested() - product_buffer_start.1, Ordering::Relaxed);
    }
    for key in &all_keys {
        let kind = key.kind();
        let product_kind_start = census_checkpoint(&allocations);
        live_products.push(product_and_fingerprints(snapshot, key, modules, &mut allocations)?);
        record_product_kind_census(kind, product_kind_start, &allocations);
    }
    record_census_phase(2, product_start, &allocations);

    let entry_start = census_checkpoint(&allocations);
    let mut entries = allocations
        .vector::<(QueryKey, CanonicalQueryEntry)>(all_keys.len())
        .map_err(allocation_error)?;
    let mut fingerprints = allocations.vector::<(QueryKey, ProductFingerprint)>(all_keys.len()).map_err(allocation_error)?;
    for (key_index, key) in all_keys.iter().enumerate() {
        let (product, product_fingerprint, input_fingerprint) = &live_products[key_index];
        let dependencies = if let Some(root_index) = root_for_key[key_index] {
            let root = &image.supported_roots[root_index];
            let mut edges = allocations.vector::<DependencyEdge>(root.dependency_indices.len()).map_err(allocation_error)?;
            for &evidence_index in &root.dependency_indices {
                let evidence_index = usize::try_from(evidence_index).map_err(|_| incomplete("query evidence index exceeds address space"))?;
                let dependency_key_index = *evidence_key_indices
                    .get(evidence_index)
                    .ok_or_else(|| incomplete("query root references absent dependency evidence"))?;
                let dependency = &all_keys[dependency_key_index];
                let observed_fingerprint = live_products[dependency_key_index].1;
                edges.push(DependencyEdge {
                    dependent: clone_loaded_query_key_with_allocation_budget(key, &mut allocations)?,
                    dependency: clone_loaded_query_key_with_allocation_budget(dependency, &mut allocations)?,
                    observed_fingerprint,
                });
            }
            edges.sort_unstable_by(|left, right| left.dependency.cmp(&right.dependency));
            allocations.arc_slice(edges).map_err(allocation_error)?
        } else {
            allocations.arc_slice(Vec::new()).map_err(allocation_error)?
        };
        entries.push((
            clone_loaded_query_key_with_allocation_budget(key, &mut allocations)?,
            CanonicalQueryEntry {
                input_fingerprint: *input_fingerprint,
                product_fingerprint: *product_fingerprint,
                dependencies,
                product: product.clone(),
            },
        ));
        fingerprints.push((clone_loaded_query_key_with_allocation_budget(key, &mut allocations)?, *product_fingerprint));
    }
    let entries = CanonicalQueryEntries::from_loaded_entries(entries, &mut allocations)?;
    let fingerprints = CanonicalDependencyFingerprints::from_loaded_entries(fingerprints, &mut allocations)?;
    record_census_phase(3, entry_start, &allocations);

    let validation_start = census_checkpoint(&allocations);
    let query_image = CanonicalSemanticQueryImage::from_loaded_artifact_entries(
        world.clone_with_allocation_budget(&mut allocations).map_err(allocation_error)?,
        entries,
        snapshot.store.clone(),
        snapshot.static_values.clone(),
        &mut allocations,
    )?;
    record_census_phase(4, validation_start, &allocations);
    let publication_start = census_checkpoint(&allocations);
    let query_image = allocations.arc(query_image).map_err(allocation_error)?;
    record_census_phase(5, publication_start, &allocations);
    transaction.commit_child(allocations).map_err(allocation_error)?;
    Ok((query_image, fingerprints))
}

/// Clones heap-backed identity components for loaded query tables. The
/// destination vector already budgets the inline `QueryKey` storage, so this
/// charges only the nested identity allocations.
fn clone_loaded_query_key_with_allocation_budget(key: &QueryKey, allocations: &mut AllocationBudget) -> Result<QueryKey, CanonicalImageError> {
    use QueryKey as Q;

    Ok(match key {
        Q::UnlinkedInterface(module) => Q::UnlinkedInterface(module.clone_with_allocation_budget(allocations).map_err(allocation_error)?),
        Q::ResolvedImports(module) => Q::ResolvedImports(module.clone_with_allocation_budget(allocations).map_err(allocation_error)?),
        Q::LinkedInterface(module) => Q::LinkedInterface(module.clone_with_allocation_budget(allocations).map_err(allocation_error)?),
        Q::DeclarationShell(declaration)
        | Q::HierarchyEdge(declaration)
        | Q::DeclarationSurface(declaration)
        | Q::TraitHeader(declaration)
        | Q::TraitSurface(declaration)
        | Q::DataDeclaration(declaration)
        | Q::EnumDeclaration(declaration)
        | Q::AssociatedSurface(declaration) => {
            let declaration = declaration.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
            match key {
                Q::DeclarationShell(_) => Q::DeclarationShell(declaration),
                Q::HierarchyEdge(_) => Q::HierarchyEdge(declaration),
                Q::DeclarationSurface(_) => Q::DeclarationSurface(declaration),
                Q::TraitHeader(_) => Q::TraitHeader(declaration),
                Q::TraitSurface(_) => Q::TraitSurface(declaration),
                Q::DataDeclaration(_) => Q::DataDeclaration(declaration),
                Q::EnumDeclaration(_) => Q::EnumDeclaration(declaration),
                Q::AssociatedSurface(_) => Q::AssociatedSurface(declaration),
                _ => unreachable!("guarded declaration query key"),
            }
        }
        Q::CallableDefinition(callable) => Q::CallableDefinition(callable.clone_with_allocation_budget(allocations).map_err(allocation_error)?),
        Q::CallableSignature(callable) => Q::CallableSignature(callable.clone_with_allocation_budget(allocations).map_err(allocation_error)?),
        Q::FieldSignature(field) => Q::FieldSignature(field.clone_with_allocation_budget(allocations).map_err(allocation_error)?),
        Q::TypeMemberDeclaration(member) => Q::TypeMemberDeclaration(member.clone_with_allocation_budget(allocations).map_err(allocation_error)?),
        Q::ModuleStaticBindingCatalog(module) => Q::ModuleStaticBindingCatalog(module.clone_with_allocation_budget(allocations).map_err(allocation_error)?),
        Q::ModuleStaticBindings(module) => Q::ModuleStaticBindings(module.clone_with_allocation_budget(allocations).map_err(allocation_error)?),
        _ => return Err(incomplete("loaded query key has no budget-aware clone")),
    })
}

fn clone_stable_project_with_budget(project: &StableProjectRef, allocations: &mut AllocationBudget) -> Result<StableProjectRef, CanonicalImageError> {
    Ok(match project {
        StableProjectRef::Builtin { namespace, version } => StableProjectRef::Builtin {
            namespace: allocations.boxed_str(namespace).map_err(allocation_error)?,
            version: allocations.boxed_str(version).map_err(allocation_error)?,
        },
        StableProjectRef::Package {
            package,
            version,
            artifact_fingerprint,
        } => StableProjectRef::Package {
            package: allocations.boxed_str(package).map_err(allocation_error)?,
            version: allocations.boxed_str(version).map_err(allocation_error)?,
            artifact_fingerprint: *artifact_fingerprint,
        },
        StableProjectRef::SourceArtifact {
            logical_uri,
            source_fingerprint,
        } => StableProjectRef::SourceArtifact {
            logical_uri: allocations.boxed_str(logical_uri).map_err(allocation_error)?,
            source_fingerprint: *source_fingerprint,
        },
        StableProjectRef::Session { session_fingerprint } => StableProjectRef::Session {
            session_fingerprint: *session_fingerprint,
        },
    })
}

fn clone_stable_module_with_budget(module: &StableModuleRef, allocations: &mut AllocationBudget) -> Result<StableModuleRef, CanonicalImageError> {
    let mut path = allocations.vector::<Box<str>>(module.path.len()).map_err(allocation_error)?;
    for component in module.path.iter() {
        path.push(allocations.boxed_str(component).map_err(allocation_error)?);
    }
    Ok(StableModuleRef {
        project: clone_stable_project_with_budget(&module.project, allocations)?,
        path: allocations.boxed_slice(path).map_err(allocation_error)?,
    })
}

fn clone_stable_declaration_with_budget(
    declaration: &StableDeclarationRef,
    allocations: &mut AllocationBudget,
) -> Result<StableDeclarationRef, CanonicalImageError> {
    let mut path = allocations.vector::<Box<str>>(declaration.path.len()).map_err(allocation_error)?;
    for component in declaration.path.iter() {
        path.push(allocations.boxed_str(component).map_err(allocation_error)?);
    }
    Ok(StableDeclarationRef {
        module: clone_stable_module_with_budget(&declaration.module, allocations)?,
        path: allocations.boxed_slice(path).map_err(allocation_error)?,
    })
}

/// Current-schema owner resolution with a validated project/module catalog.
fn consume_stable_module_authorized(
    module: StableModuleRef, allocations: &mut AllocationBudget,
    owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>,
) -> Result<phalcom_modules::ModuleId, CanonicalImageError> {
    match owner_catalog {
        Some(catalog) => catalog.module_parts_with_allocation_budget(&module.project, &module.path, allocations)
            .map_err(CanonicalImageError::Artifact),
        None => consume_stable_module(module, allocations),
    }
}

fn consume_stable_declaration_authorized(
    declaration: StableDeclarationRef, allocations: &mut AllocationBudget,
    owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>,
) -> Result<crate::identity::DeclarationId, CanonicalImageError> {
    match owner_catalog {
        Some(catalog) => catalog.declaration_with_allocation_budget(&declaration, allocations).map_err(CanonicalImageError::Artifact),
        None => consume_stable_declaration(declaration, allocations),
    }
}

fn consume_stable_field_authorized(
    field: StableFieldRef, allocations: &mut AllocationBudget,
    owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>,
) -> Result<crate::identity::FieldId, CanonicalImageError> {
    match owner_catalog {
        Some(catalog) => Ok(crate::identity::FieldId {
            owner: catalog.declaration_with_allocation_budget(&field.owner, allocations).map_err(CanonicalImageError::Artifact)?,
            name: field.name,
            side: crate::metadata::stable_identity::from_stable_dispatch_side(field.side),
        }),
        None => consume_stable_field(field, allocations),
    }
}

fn resolve_query_callable_authorized(
    callable: &StableCallableRef, allocations: &mut AllocationBudget,
    owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>,
) -> Result<crate::identity::CallableId, CanonicalImageError> {
    match owner_catalog {
        Some(catalog) => catalog.callable_with_allocation_budget(callable, allocations).map_err(CanonicalImageError::Artifact),
        None => import_stable_callable_with_allocation_budget(callable, allocations).map_err(CanonicalImageError::Artifact),
    }
}

fn resolve_query_key_v3(key: &PortableQueryKeyV1, allocations: &mut AllocationBudget, owner_catalog: Option<&crate::artifact::StableOwnerCatalog<'_>>) -> Result<QueryKey, CanonicalImageError> {
    use PortableQueryKindV1 as K;
    use PortableQueryOwnerKeyV1 as O;

    Ok(match key.kind {
        K::UnlinkedInterface => QueryKey::UnlinkedInterface(consume_stable_module_authorized(
            clone_stable_module_with_budget(stable_module_owner(&key.owner)?, allocations)?,
            allocations,
            owner_catalog,
        )?),
        K::ResolvedImports => QueryKey::ResolvedImports(consume_stable_module_authorized(
            clone_stable_module_with_budget(stable_module_owner(&key.owner)?, allocations)?,
            allocations,
            owner_catalog,
        )?),
        K::LinkedInterface => QueryKey::LinkedInterface(consume_stable_module_authorized(
            clone_stable_module_with_budget(stable_module_owner(&key.owner)?, allocations)?,
            allocations,
            owner_catalog,
        )?),
        K::DeclarationShell => QueryKey::DeclarationShell(consume_stable_declaration_authorized(
            clone_stable_declaration_with_budget(stable_declaration_owner(&key.owner)?, allocations)?,
            allocations,
            owner_catalog,
        )?),
        K::HierarchyEdge => QueryKey::HierarchyEdge(consume_stable_declaration_authorized(
            clone_stable_declaration_with_budget(stable_declaration_owner(&key.owner)?, allocations)?,
            allocations,
            owner_catalog,
        )?),
        K::CallableDefinition => {
            let O::Callable(callable) = &key.owner else {
                return Err(incomplete("portable query owner is not a callable"));
            };
            QueryKey::CallableDefinition(resolve_query_callable_authorized(callable, allocations, owner_catalog)?)
        }
        K::CallableSignature => {
            let O::Callable(callable) = &key.owner else {
                return Err(incomplete("portable query owner is not a callable"));
            };
            QueryKey::CallableSignature(resolve_query_callable_authorized(callable, allocations, owner_catalog)?)
        }
        K::FieldSignature => {
            let O::Field(field) = &key.owner else {
                return Err(incomplete("portable query owner is not a field"));
            };
            let stable = StableFieldRef {
                owner: clone_stable_declaration_with_budget(&field.owner, allocations)?,
                side: field.side,
                name: allocations.boxed_str(&field.name).map_err(allocation_error)?,
            };
            QueryKey::FieldSignature(consume_stable_field_authorized(stable, allocations, owner_catalog)?)
        }
        K::DeclarationSurface | K::TraitHeader | K::TraitSurface | K::DataDeclaration | K::EnumDeclaration | K::AssociatedSurface => {
            let declaration = consume_stable_declaration_authorized(
                clone_stable_declaration_with_budget(stable_declaration_owner(&key.owner)?, allocations)?,
                allocations,
                owner_catalog,
            )?;
            match key.kind {
                K::DeclarationSurface => QueryKey::DeclarationSurface(declaration),
                K::TraitHeader => QueryKey::TraitHeader(declaration),
                K::TraitSurface => QueryKey::TraitSurface(declaration),
                K::DataDeclaration => QueryKey::DataDeclaration(declaration),
                K::EnumDeclaration => QueryKey::EnumDeclaration(declaration),
                K::AssociatedSurface => QueryKey::AssociatedSurface(declaration),
                _ => unreachable!("guarded declaration query kind"),
            }
        }
        K::TypeMemberDeclaration => {
            let O::TypeMember(member) = &key.owner else {
                return Err(incomplete("portable query owner is not a type member"));
            };
            let owner = consume_stable_declaration_authorized(clone_stable_declaration_with_budget(&member.owner, allocations)?, allocations, owner_catalog)?;
            let selector = match member.selector.as_ref() {
                "[]" => crate::types::type_member::TypeMemberSelector::Subscript,
                "At" => return Err(incomplete("noncanonical stable type-member selector aliases the subscript identity")),
                "" => return Err(incomplete("empty stable type-member selector")),
                selector => crate::types::type_member::TypeMemberSelector::Named(allocations.boxed_str(selector).map_err(allocation_error)?),
            };
            QueryKey::TypeMemberDeclaration(crate::types::type_member::TypeMemberDeclarationId::new(owner, selector))
        }
        K::ModuleStaticBindingCatalog | K::ModuleStaticBindings => {
            let module = consume_stable_module_authorized(clone_stable_module_with_budget(stable_module_owner(&key.owner)?, allocations)?, allocations, owner_catalog)?;
            if key.kind == K::ModuleStaticBindingCatalog {
                QueryKey::ModuleStaticBindingCatalog(module)
            } else {
                QueryKey::ModuleStaticBindings(module)
            }
        }
        _ => return Err(incomplete("current query kind has no P7 loaded-image adapter")),
    })
}

fn stable_module_owner<'a>(owner: &'a PortableQueryOwnerKeyV1) -> Result<&'a StableModuleRef, CanonicalImageError> {
    match owner {
        PortableQueryOwnerKeyV1::Module(module) => Ok(module),
        _ => Err(incomplete("portable query owner is not a module")),
    }
}

fn stable_declaration_owner<'a>(owner: &'a PortableQueryOwnerKeyV1) -> Result<&'a StableDeclarationRef, CanonicalImageError> {
    match owner {
        PortableQueryOwnerKeyV1::Declaration(declaration) => Ok(declaration),
        _ => Err(incomplete("portable query owner is not a declaration")),
    }
}

fn allocation_error(error: AllocationError) -> CanonicalImageError {
    CanonicalImageError::Artifact(ArtifactError::Allocation(error))
}

fn reconstruct_query_image_inner(
    image: &QueryImageV2,
    snapshot: &Arc<SemanticSnapshot>,
    world: &CanonicalSemanticWorldId,
    modules: &phalcom_modules::canonical_products::CanonicalExternalModuleWorld,
    limits: &phalcom_compiler_artifact::ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<(Arc<CanonicalSemanticQueryImage>, CanonicalDependencyFingerprints), CanonicalImageError> {
    let closure_start = census_checkpoint(allocations);
    validate_query_image_closure_with_allocation_budget(image, limits, allocations)?;
    record_census_phase(0, closure_start, allocations);

    let owner_start = census_checkpoint(allocations);
    let mut keys = allocations.vector::<QueryKey>(image.supported_roots.len()).map_err(allocation_error)?;
    for root in &image.supported_roots {
        keys.push(resolve_query_key(&root.key, allocations)?);
    }
    record_census_phase(1, owner_start, allocations);

    let product_start = census_checkpoint(allocations);
    let product_buffer_start = census_checkpoint(allocations);
    let mut roots = allocations
        .vector::<(QueryKey, Arc<SemanticProduct>, ProductFingerprint, InputFingerprint)>(image.supported_roots.len())
        .map_err(allocation_error)?;
    if QUERY_RECONSTRUCTION_CENSUS_ACTIVE.load(Ordering::Relaxed) {
        if let (Some(start), Some(end)) = (product_buffer_start.0, phalcom_type_meta::allocation::current_observed_bytes()) {
            QUERY_PRODUCT_BUFFER_OBSERVED.fetch_add(end - start, Ordering::Relaxed);
        }
        QUERY_PRODUCT_BUFFER_LEDGER.fetch_add(allocations.requested() - product_buffer_start.1, Ordering::Relaxed);
    }
    for query_key in keys {
        let kind = query_key.kind();
        let product_kind_start = census_checkpoint(allocations);
        let (product, product_fingerprint, input_fingerprint) = product_and_fingerprints(snapshot, &query_key, modules, allocations)?;
        record_product_kind_census(kind, product_kind_start, allocations);
        roots.push((query_key, product, product_fingerprint, input_fingerprint));
    }
    record_census_phase(2, product_start, allocations);

    let entry_start = census_checkpoint(allocations);
    let mut entries = allocations
        .vector::<(QueryKey, CanonicalQueryEntry)>(image.supported_roots.len())
        .map_err(allocation_error)?;
    let mut fingerprints = allocations
        .vector::<(QueryKey, ProductFingerprint)>(image.supported_roots.len())
        .map_err(allocation_error)?;
    for (idx, root) in image.supported_roots.iter().enumerate() {
        let (query_key, product, product_fingerprint, input_fingerprint) = &roots[idx];
        let mut edges = allocations.vector::<DependencyEdge>(root.dependency_indices.len()).map_err(allocation_error)?;
        for &dep_idx in &root.dependency_indices {
            let (dependency, _, observed_fingerprint, _) = roots
                .get(dep_idx as usize)
                .ok_or_else(|| incomplete("query dependency references an absent root"))?;
            edges.push(DependencyEdge {
                dependent: query_key.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                dependency: dependency.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                observed_fingerprint: *observed_fingerprint,
            });
        }
        let dependencies = allocations.arc_slice(edges).map_err(allocation_error)?;
        entries.push((
            query_key.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            CanonicalQueryEntry {
                input_fingerprint: *input_fingerprint,
                product_fingerprint: *product_fingerprint,
                dependencies,
                product: product.clone(),
            },
        ));
        fingerprints.push((
            query_key.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            *product_fingerprint,
        ));
    }
    let entries = CanonicalQueryEntries::from_loaded_entries(entries, allocations)?;
    let fingerprints = CanonicalDependencyFingerprints::from_loaded_entries(fingerprints, allocations)?;
    record_census_phase(3, entry_start, allocations);

    let validation_start = census_checkpoint(allocations);
    let query_image = CanonicalSemanticQueryImage::from_loaded_artifact_entries(
        world.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
        entries,
        snapshot.store.clone(),
        snapshot.static_values.clone(),
        allocations,
    )?;
    record_census_phase(4, validation_start, allocations);
    let publication_start = census_checkpoint(allocations);
    let query_image = allocations.arc(query_image).map_err(allocation_error)?;
    record_census_phase(5, publication_start, allocations);
    Ok((query_image, fingerprints))
}

#[cfg(test)]
mod query_owner_catalog_regressions {
    use super::*;
    use crate::artifact::{StableOwnerCatalog, empty_owner_inputs};
    use phalcom_type_meta::identity::StableProjectRef;

    #[test]
    fn current_query_owner_rejects_missing_catalog_authority_without_fallback() {
        let (mut wire, projects) = empty_owner_inputs();
        let module = StableModuleRef {
            project: StableProjectRef::Builtin { namespace: "universe".into(), version: "0.1.0".into() },
            path: Box::new([]),
        };
        let key = PortableQueryKeyV1 {
            kind: PortableQueryKindV1::LinkedInterface,
            owner: PortableQueryOwnerKeyV1::Module(module.clone()),
        };
        let mut allocations = AllocationBudget::new(64 * 1024);
        let catalog = StableOwnerCatalog::new(&wire, &projects).unwrap();
        assert!(resolve_query_key_v3(&key, &mut allocations, Some(&catalog)).is_err());
        assert!(matches!(
            resolve_query_key_v3(&key, &mut allocations, None),
            Ok(QueryKey::LinkedInterface(_))
        ));

        wire.external_projects.push(module.project.clone());
        let catalog = StableOwnerCatalog::new(&wire, &projects).unwrap();
        let owner = resolve_query_key_v3(&key, &mut allocations, Some(&catalog)).unwrap();
        assert!(matches!(owner, QueryKey::LinkedInterface(id) if id.project == phalcom_modules::ProjectIdentity::Universe));
    }
}
