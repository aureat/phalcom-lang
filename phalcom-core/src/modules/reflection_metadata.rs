//! Compact runtime reflection metadata projection for ADTs and enums (Part 06).

use phalcom_common::selector::Selector;
use phalcom_modules::DeclarationId;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::snapshot::SemanticSnapshot;
use phalcom_semantic::stable_identity::{StableVariantFamilyKey, StableVariantFieldKey, StableVariantKey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeVariantFieldReflectionSpec {
    pub key: StableVariantFieldKey,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub slot: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeVariantReflectionSpec {
    pub key: StableVariantKey,
    pub selector: Selector,
    pub family: Option<StableVariantFamilyKey>,
    pub shape: VariantShape,
    pub payload_fields: Box<[RuntimeVariantFieldReflectionSpec]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeEnumReflectionSpec {
    pub owner: DeclarationId,
    pub name: Box<str>,
    pub native: bool,
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

            let mut variants = Vec::new();
            for var_id in enum_info.variants.iter() {
                if let Some(var_info) = snapshot.enum_semantics.variant_info(var_id) {
                    let var_key = StableVariantKey::new(var_info.id.owner.clone(), var_info.id.selector.clone());
                    let mut fields = Vec::new();
                    for (slot, f) in var_info.fields.iter().enumerate() {
                        let field_key = StableVariantFieldKey::new(var_key.clone(), slot as u32);
                        fields.push(RuntimeVariantFieldReflectionSpec {
                            key: field_key,
                            local_name: f.local_name.clone(),
                            external_label: f.external_label.clone(),
                            slot: slot as u16,
                        });
                    }
                    let family_key = var_info
                        .family
                        .as_ref()
                        .map(|fam| StableVariantFamilyKey::new(fam.owner.clone(), fam.base_name.clone()));
                    variants.push(RuntimeVariantReflectionSpec {
                        key: var_key,
                        selector: var_info.id.selector.clone(),
                        family: family_key,
                        shape: var_info.shape,
                        payload_fields: fields.into_boxed_slice(),
                    });
                }
            }

            enums.push(RuntimeEnumReflectionSpec {
                owner: owner.clone(),
                name: owner.name.clone(),
                native: is_native,
                variants: variants.into_boxed_slice(),
            });
        }

        Self {
            enums: enums.into_boxed_slice(),
        }
    }
}
