//! Reconstruction-safe stable semantic keys for persistent metadata and reflection (Part 06).

use crate::identity::DeclarationId;
use phalcom_common::selector::Selector;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StableVariantKey {
    pub owner: DeclarationId,
    pub selector: Selector,
}

impl StableVariantKey {
    pub fn new(owner: DeclarationId, selector: Selector) -> Self {
        Self { owner, selector }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StableVariantFamilyKey {
    pub owner: DeclarationId,
    pub base_name: Box<str>,
}

impl StableVariantFamilyKey {
    pub fn new(owner: DeclarationId, base_name: impl Into<Box<str>>) -> Self {
        Self {
            owner,
            base_name: base_name.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StableVariantFieldKey {
    pub variant: StableVariantKey,
    pub index: u32,
}

impl StableVariantFieldKey {
    pub fn new(variant: StableVariantKey, index: u32) -> Self {
        Self { variant, index }
    }
}
