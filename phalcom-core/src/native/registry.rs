//! Distributed static registry for native primitive descriptors.

use super::descriptor::PrimitiveDescriptor;
use linkme::distributed_slice;
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
