//! Certified PERF003 P7 constructor-initialization reconstruction adapted to ARTF001.
//!
//! Constructor initialization products are snapshot semantic authority. Current
//! persistence policy does not promote this family as an unconditional Ready
//! query root, so the importer stages exact typed products for T4 publication.

use super::callable_import::ImportedCallableAuthority;
use super::identity::StableOwnerCatalog;
use crate::db::ProductFingerprint;
use crate::identity::{CallableId, CallableOwnerId, CallableParameterId, DispatchSide, FieldId};
use crate::signature::{ConstructorInitializationProduct, ConstructorOrigin, ConstructorParameterField};
use phalcom_compiler_artifact::semantic::world::ConstructorInitializationRecordV1;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;

/// Immutable sorted constructor authority staged for the imported generation.
#[derive(Clone, Debug, Default)]
pub struct ImportedConstructorInitializationAuthority {
    products: Box<[(CallableId, ConstructorInitializationProduct)]>,
}

impl ImportedConstructorInitializationAuthority {
    pub fn len(&self) -> usize {
        self.products.len()
    }

    pub fn is_empty(&self) -> bool {
        self.products.is_empty()
    }

    pub(crate) fn get(&self, factory: &CallableId) -> Option<&ConstructorInitializationProduct> {
        self.products
            .binary_search_by(|(candidate, _)| candidate.cmp(factory))
            .ok()
            .map(|index| &self.products[index].1)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&CallableId, &ConstructorInitializationProduct)> {
        self.products.iter().map(|(factory, product)| (factory, product))
    }
}

/// Port of P7 constructor-initialization import with current exact owner
/// relocation and additional canonical callable-signature checks.
pub(crate) fn reconstruct_constructor_initializations(
    records: &[ConstructorInitializationRecordV1],
    callables: &ImportedCallableAuthority,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedConstructorInitializationAuthority, ArtifactError> {
    let mut products = allocations
        .vector::<(CallableId, ConstructorInitializationProduct)>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for record in records {
        let factory = owners.callable(&record.factory)?;
        let initializer = owners.callable(&record.initializer)?;

        let (CallableOwnerId::Declaration(factory_owner), CallableOwnerId::Declaration(initializer_owner)) = (&factory.owner, &initializer.owner) else {
            return Err(invalid("constructor initialization factory and initializer must be declaration-owned"));
        };
        if factory_owner != initializer_owner {
            return Err(invalid("constructor initialization factory and initializer have different declaration owners"));
        }
        if factory.side != DispatchSide::Class || initializer.side != DispatchSide::Instance {
            return Err(invalid("constructor initialization factory/initializer dispatch sides are invalid"));
        }

        let factory_signature = callables
            .signature(&factory)
            .ok_or_else(|| invalid("constructor initialization factory has no canonical callable signature"))?;
        if callables.signature(&initializer).is_none() {
            return Err(invalid("constructor initialization initializer has no canonical callable signature"));
        }

        let origin = match record.origin {
            0 => ConstructorOrigin::Source,
            1 => ConstructorOrigin::Construct,
            2 => ConstructorOrigin::Data,
            3 => ConstructorOrigin::DefaultOnly,
            _ => return Err(invalid("constructor initialization origin tag is invalid")),
        };

        let mut default_fields = allocations.vector::<FieldId>(record.default_fields.len()).map_err(ArtifactError::Allocation)?;
        for name in &record.default_fields {
            default_fields.push(FieldId::new(
                initializer_owner.clone(),
                allocations.boxed_str(name).map_err(ArtifactError::Allocation)?,
                DispatchSide::Instance,
            ));
        }

        let mut parameter_fields = allocations
            .vector::<ConstructorParameterField>(record.parameter_fields.len())
            .map_err(ArtifactError::Allocation)?;
        for binding in &record.parameter_fields {
            if binding.parameter_index as usize >= factory_signature.parameters.len() {
                return Err(invalid("constructor parameter-field binding references an absent factory parameter"));
            }
            let side = match binding.field_side {
                0 => DispatchSide::Instance,
                1 => DispatchSide::Class,
                _ => return Err(invalid("constructor parameter-field binding has an invalid field side")),
            };
            parameter_fields.push(ConstructorParameterField {
                parameter: CallableParameterId::new(factory.clone(), binding.parameter_index),
                field: FieldId::new(
                    initializer_owner.clone(),
                    allocations.boxed_str(&binding.field_name).map_err(ArtifactError::Allocation)?,
                    side,
                ),
            });
        }

        let product = ConstructorInitializationProduct {
            factory: factory.clone(),
            initializer,
            origin,
            field_input_fingerprint: ProductFingerprint::new(record.field_input_fingerprint),
            default_fields: allocations.boxed_slice(default_fields).map_err(ArtifactError::Allocation)?,
            parameter_fields: allocations.boxed_slice(parameter_fields).map_err(ArtifactError::Allocation)?,
        };
        products.push((factory, product));
    }

    products.sort_by(|left, right| left.0.cmp(&right.0));
    if products.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(invalid("duplicate constructor initialization factory after stable-owner relocation"));
    }

    Ok(ImportedConstructorInitializationAuthority {
        products: allocations.boxed_slice(products).map_err(ArtifactError::Allocation)?,
    })
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
