//! Compact runtime reflection metadata projection for ADTs and enums (Part 06).

use crate::adt::RuntimeAdtRepresentation;
use phalcom_common::selector::Selector;
use phalcom_modules::DeclarationId;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{VariantFamilyId, VariantFieldId, VariantId};
use phalcom_semantic::snapshot::SemanticSnapshot;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeVariantFieldReflectionSpec {
    pub id: VariantFieldId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub slot: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeVariantReflectionSpec {
    pub id: VariantId,
    pub selector: Selector,
    pub family: Option<VariantFamilyId>,
    pub shape: VariantShape,
    pub payload_fields: Box<[RuntimeVariantFieldReflectionSpec]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeEnumReflectionSpec {
    pub owner: DeclarationId,
    pub name: Box<str>,
    pub native: bool,
    pub representation: RuntimeAdtRepresentation,
    pub variants: Box<[RuntimeVariantReflectionSpec]>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModuleReflectionMetadata {
    pub enums: Box<[RuntimeEnumReflectionSpec]>,
}

impl ModuleReflectionMetadata {
    pub fn from_snapshot(snapshot: &SemanticSnapshot) -> Self {
        let core_ids = phalcom_semantic::core_surface::CoreDeclarationIds::default();
        let mut enums = Vec::new();
        for (owner, enum_info) in &snapshot.enum_semantics.enums {
            let is_native = core_ids.is_core_adt(owner);
            let representation = if core_ids.is_option(owner) {
                RuntimeAdtRepresentation::NativeOption
            } else if core_ids.is_result(owner) {
                RuntimeAdtRepresentation::NativeResult
            } else {
                RuntimeAdtRepresentation::General
            };

            let mut variants = Vec::new();
            for var_id in enum_info.variants.iter() {
                if let Some(var_info) = snapshot.enum_semantics.variant_info(var_id) {
                    let mut fields = Vec::new();
                    for (slot, f) in var_info.fields.iter().enumerate() {
                        fields.push(RuntimeVariantFieldReflectionSpec {
                            id: f.id.clone(),
                            local_name: f.local_name.clone(),
                            external_label: f.external_label.clone(),
                            slot: slot as u16,
                        });
                    }
                    variants.push(RuntimeVariantReflectionSpec {
                        id: var_info.id.clone(),
                        selector: var_info.id.selector.clone(),
                        family: var_info.family.clone(),
                        shape: var_info.shape,
                        payload_fields: fields.into_boxed_slice(),
                    });
                }
            }

            enums.push(RuntimeEnumReflectionSpec {
                owner: owner.clone(),
                name: owner.name.clone(),
                native: is_native,
                representation,
                variants: variants.into_boxed_slice(),
            });
        }

        Self {
            enums: enums.into_boxed_slice(),
        }
    }
}
