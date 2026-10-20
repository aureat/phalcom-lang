//! Certified PERF003 P7 declaration-dispatch reconstruction adapted to ARTF001.
//!
//! Current-main `DeclarationSurface` is HashMap-backed and has no allocation-
//! budgeted frozen constructor. Building it during artifact staging would escape
//! the import ledger. This module therefore reconstructs the exact semantic
//! member facts into sorted immutable rows; T4 materializes the one live
//! `DeclarationSurface` at the atomic generation-publication boundary.

use super::callable_import::{ImportedCallableAuthority, import_type_knowledge};
use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use crate::declaration_type::DeclaredTypeFact;
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide, FieldId};
use crate::signature::{CallableSemanticSignature, FieldSemanticSignature};
use crate::types::evidence::TypeKnowledge;
use phalcom_common::modifier::Visibility;
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::aggregates::VisibilityV1;
use phalcom_compiler_artifact::semantic::world::{DeclarationSurfaceRecordV1, DispatchCallableFactV1, FieldSignatureV1};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;
use std::sync::Arc;

/// One declaration-owned callable admitted to the final dispatch surface.
#[derive(Clone, Debug)]
pub struct ImportedDispatchCallable {
    pub callable: CallableId,
    /// Canonical callable contract reconstructed from the callable authority.
    pub signature: Arc<CallableSemanticSignature>,
    /// Final return/type knowledge retained by the dispatch surface.
    pub knowledge: TypeKnowledge,
    pub visibility: Visibility,
}

/// One declaration-owned field admitted to the final dispatch surface.
#[derive(Clone, Debug)]
pub struct ImportedDispatchField {
    pub field: FieldId,
    pub knowledge: TypeKnowledge,
    pub visibility: Visibility,
    /// Present exactly when the source field had a canonical field-signature
    /// product in P7. Absence is semantically meaningful.
    pub signature: Option<FieldSemanticSignature>,
}

#[derive(Clone, Debug)]
pub struct ImportedDeclarationDispatchSurface {
    pub declaration: DeclarationId,
    pub callables: Box<[ImportedDispatchCallable]>,
    pub fields: Box<[ImportedDispatchField]>,
}

/// Immutable staging authority for declaration dispatch and field signatures.
#[derive(Clone, Debug, Default)]
pub struct ImportedDispatchAuthority {
    surfaces: Box<[ImportedDeclarationDispatchSurface]>,
}

impl ImportedDispatchAuthority {
    pub fn surface_len(&self) -> usize {
        self.surfaces.len()
    }

    pub fn callable_len(&self) -> usize {
        self.surfaces.iter().map(|surface| surface.callables.len()).sum()
    }

    pub fn field_len(&self) -> usize {
        self.surfaces.iter().map(|surface| surface.fields.len()).sum()
    }

    pub fn field_signature_len(&self) -> usize {
        self.surfaces
            .iter()
            .flat_map(|surface| surface.fields.iter())
            .filter(|field| field.signature.is_some())
            .count()
    }

    pub(crate) fn surface(&self, declaration: &DeclarationId) -> Option<&ImportedDeclarationDispatchSurface> {
        self.surfaces
            .binary_search_by(|surface| surface.declaration.cmp(declaration))
            .ok()
            .map(|index| &self.surfaces[index])
    }
}

/// Port of P7's declaration-surface and field-signature reconstruction.
///
/// Unlike P7's old frozen HashMap builder, this current-main adaptation keeps
/// immutable sorted rows until T4 so every staging allocation remains charged
/// to the artifact transaction.
pub(crate) fn reconstruct_dispatch_authority(
    records: &[DeclarationSurfaceRecordV1],
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedDispatchAuthority, ArtifactError> {
    let mut surfaces = allocations
        .vector::<ImportedDeclarationDispatchSurface>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for record in records {
        let declaration = owners.declaration(&record.declaration)?;
        let callable_count = record
            .instance_callables
            .len()
            .checked_add(record.class_callables.len())
            .ok_or_else(|| invalid("declaration dispatch callable count overflow"))?;
        let mut imported_callables = allocations
            .vector::<ImportedDispatchCallable>(callable_count)
            .map_err(ArtifactError::Allocation)?;

        for (side, rows) in [
            (DispatchSide::Instance, record.instance_callables.as_slice()),
            (DispatchSide::Class, record.class_callables.as_slice()),
        ] {
            for row in rows {
                imported_callables.push(reconstruct_callable_fact(row, side, &declaration, callables, types, owners, allocations)?);
            }
        }
        imported_callables.sort_by(|left, right| left.callable.cmp(&right.callable));
        if imported_callables.windows(2).any(|pair| pair[0].callable == pair[1].callable) {
            return Err(invalid("duplicate declaration dispatch callable after stable-owner relocation"));
        }

        let mut imported_fields = allocations
            .vector::<ImportedDispatchField>(record.fields.len())
            .map_err(ArtifactError::Allocation)?;
        for row in &record.fields {
            imported_fields.push(reconstruct_field_fact(row, &declaration, types, owners, allocations)?);
        }
        imported_fields.sort_by(|left, right| left.field.cmp(&right.field));
        if imported_fields.windows(2).any(|pair| pair[0].field == pair[1].field) {
            return Err(invalid("duplicate declaration dispatch field after stable-owner relocation"));
        }

        surfaces.push(ImportedDeclarationDispatchSurface {
            declaration,
            callables: allocations.boxed_slice(imported_callables).map_err(ArtifactError::Allocation)?,
            fields: allocations.boxed_slice(imported_fields).map_err(ArtifactError::Allocation)?,
        });
    }

    surfaces.sort_by(|left, right| left.declaration.cmp(&right.declaration));
    if surfaces.windows(2).any(|pair| pair[0].declaration == pair[1].declaration) {
        return Err(invalid("duplicate declaration dispatch surface after stable-owner relocation"));
    }

    Ok(ImportedDispatchAuthority {
        surfaces: allocations.boxed_slice(surfaces).map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_callable_fact(
    row: &DispatchCallableFactV1,
    expected_side: DispatchSide,
    declaration: &DeclarationId,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedDispatchCallable, ArtifactError> {
    let callable = owners.callable(&row.callable)?;
    if callable.side != expected_side {
        return Err(invalid("declaration dispatch callable is stored under the wrong dispatch side"));
    }
    if !matches!(&callable.owner, CallableOwnerId::Declaration(owner) if owner == declaration) {
        return Err(invalid("declaration dispatch callable owner disagrees with its surface"));
    }
    let signature = callables
        .signature(&callable)
        .cloned()
        .ok_or_else(|| invalid("declaration dispatch callable has no canonical callable-signature authority"))?;
    if signature.callable != callable || signature.owner != *declaration || signature.side != expected_side {
        return Err(invalid("declaration dispatch callable signature disagrees with its exact surface identity"));
    }
    Ok(ImportedDispatchCallable {
        callable,
        signature,
        knowledge: import_type_knowledge(&row.knowledge, types, allocations)?,
        visibility: visibility(&row.visibility),
    })
}

fn reconstruct_field_fact(
    row: &FieldSignatureV1,
    declaration: &DeclarationId,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedDispatchField, ArtifactError> {
    let owner = owners.declaration(&row.owner)?;
    if &owner != declaration {
        return Err(invalid("declaration dispatch field owner disagrees with its surface"));
    }
    let side = match row.side {
        0 => DispatchSide::Instance,
        1 => DispatchSide::Class,
        _ => return Err(invalid("declaration dispatch field has an invalid dispatch side")),
    };
    let name = allocations.boxed_str(&row.name).map_err(ArtifactError::Allocation)?;
    let field = FieldId::new(owner.clone(), name.clone(), side);
    let knowledge = import_type_knowledge(&row.knowledge, types, allocations)?;

    if row.declared_type.is_some() != row.mutable.is_some() {
        return Err(invalid("field semantic signature is only partially represented"));
    }
    if row.declared_type.is_none() && row.source.is_some() {
        return Err(invalid("field source span exists without a field semantic signature"));
    }

    let signature = match (&row.declared_type, row.mutable) {
        (Some(declared_type), Some(mutable)) => {
            let declared_type: DeclaredTypeFact = super::semantic_import::reconstruct_declared_type(declared_type, types, owners, allocations)?;
            let source = row
                .source
                .as_ref()
                .map(|range| -> Result<SemanticSourceSpan, ArtifactError> {
                    if range.start > range.end {
                        return Err(invalid("field source span is inverted"));
                    }
                    Ok(SemanticSourceSpan::new(
                        owner.module.clone(),
                        SourceRange {
                            start: usize::try_from(range.start).map_err(|_| invalid("field source start exceeds address space"))?,
                            end: usize::try_from(range.end).map_err(|_| invalid("field source end exceeds address space"))?,
                        },
                    ))
                })
                .transpose()?;
            Some(FieldSemanticSignature {
                field: field.clone(),
                owner,
                side,
                name,
                mutable,
                declared_type,
                source,
            })
        }
        (None, None) => None,
        _ => unreachable!("validated paired optional fields"),
    };

    Ok(ImportedDispatchField {
        field,
        knowledge,
        visibility: visibility(&row.visibility),
        signature,
    })
}

fn visibility(value: &VisibilityV1) -> Visibility {
    match value {
        VisibilityV1::Public => Visibility::Public,
        VisibilityV1::Private => Visibility::Private,
        VisibilityV1::Protected => Visibility::Protected,
        VisibilityV1::Module => Visibility::Module,
        VisibilityV1::Package => Visibility::Package,
        VisibilityV1::Internal => Visibility::Internal,
    }
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
