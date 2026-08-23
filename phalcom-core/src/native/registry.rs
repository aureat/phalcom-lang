//! Distributed static registry for native primitive descriptors.

use super::descriptor::PrimitiveDescriptor;
use linkme::distributed_slice;
use phalcom_native_meta::PrimitiveKey;
use phalcom_native_surface::NATIVE_SURFACES;
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

/// Returns whether descriptor installation covers the canonical native surface
/// catalog.
pub fn descriptor_floor_is_complete() -> bool {
    let descriptors = PRIMITIVES.iter().map(|descriptor| descriptor.surface.key).collect::<HashSet<_>>();
    let surfaces = NATIVE_SURFACES.iter().map(|record| record.surface.key).collect::<HashSet<_>>();
    descriptors == surfaces
}
