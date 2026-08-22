//! Declaration fingerprinting and fine-grained invalidation.

use crate::identity::DeclarationId;
use std::collections::HashMap;

/// Fingerprint representing the signature interface of a declaration.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DeclarationFingerprint(pub u64);

/// Fingerprint registry for incremental invalidation analysis.
#[derive(Clone, Debug, Default)]
pub struct InvalidationIndex {
    fingerprints: HashMap<DeclarationId, DeclarationFingerprint>,
}

impl InvalidationIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: DeclarationId, fp: DeclarationFingerprint) {
        self.fingerprints.insert(id, fp);
    }

    pub fn get(&self, id: &DeclarationId) -> Option<&DeclarationFingerprint> {
        self.fingerprints.get(id)
    }
}
