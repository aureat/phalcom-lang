//! Distributed static registry for native primitive descriptors.

use super::descriptor::PrimitiveDescriptor;
use linkme::distributed_slice;
use phalcom_native_meta::{PrimitiveKey, UniverseKey};
use phalcom_native_surface::NATIVE_MEMBERS;
use std::collections::HashSet;

#[distributed_slice]
pub static PRIMITIVES: [PrimitiveDescriptor];

/// Validates that all registered descriptors have unique (owner, side, selector) keys.
pub fn validate_registry() {
    let mut seen = HashSet::new();
    for desc in PRIMITIVES {
        let key = desc.surface.key.sort_key();
        if !seen.insert(key) {
            panic!(
                "duplicate primitive descriptor registered for owner {:?}, side {:?}, selector '{}' in {}:{}",
                key.0, key.1, key.2, desc.source.file, desc.source.line
            );
        }
    }
}

/// Returns descriptor keys in registration order for census and diagnostics.
pub fn primitive_keys() -> impl Iterator<Item = PrimitiveKey> {
    PRIMITIVES.iter().map(|descriptor| descriptor.surface.key)
}

/// Returns whether descriptor installation covers the transitional legacy
/// member projection. Until this becomes true, descriptor-only startup keeps
/// the legacy installer as a compatibility floor.
pub fn descriptor_floor_is_complete() -> bool {
    let descriptors = PRIMITIVES.iter().map(|descriptor| descriptor.surface.key).collect::<HashSet<_>>();
    let legacy = NATIVE_MEMBERS
        .iter()
        .filter_map(|member| {
            Some(PrimitiveKey {
                owner: UniverseKey::from_name(member.class)?,
                side: member.side,
                selector: member.selector,
            })
        })
        .collect::<HashSet<_>>();
    descriptors == legacy
}
