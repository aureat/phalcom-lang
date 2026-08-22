//! Validates and loads immutable semantic metadata pools into VM runtime structures.

use crate::error::{PhResult, RuntimeError};
use crate::heap::ClassId;
use crate::typing::handle::MetadataPoolId;
use phalcom_type_meta::bundle::SemanticMetadataBundle;
use phalcom_type_meta::identity::StableDeclarationRef;
use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};
use std::collections::HashMap;
use std::sync::Arc;

/// Validated immutable metadata pool owned by the VM.
#[derive(Clone, Debug)]
pub struct LoadedSemanticMetadata {
    pub id: MetadataPoolId,
    pub bundle: Arc<SemanticMetadataBundle>,
}

impl LoadedSemanticMetadata {
    pub fn new(id: MetadataPoolId, bundle: Arc<SemanticMetadataBundle>) -> Self {
        Self { id, bundle }
    }
}

/// Runtime table mapping stable declaration references to loaded runtime `ClassId`s.
#[derive(Clone, Debug, Default)]
pub struct RuntimeNominalBindingTable {
    bindings: HashMap<StableDeclarationRef, ClassId>,
}

impl RuntimeNominalBindingTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, decl: StableDeclarationRef, class: ClassId) {
        self.bindings.insert(decl, class);
    }

    pub fn get(&self, decl: &StableDeclarationRef) -> Option<ClassId> {
        self.bindings.get(decl).copied()
    }
}

/// Loads and validates a `SemanticMetadataBundle`.
pub fn load_metadata_bundle(pool_id: MetadataPoolId, bundle: Arc<SemanticMetadataBundle>, limits: &ValidationLimits) -> PhResult<LoadedSemanticMetadata> {
    validate_metadata_bundle(&bundle, limits).map_err(|e| RuntimeError::Internal(format!("metadata validation failed: {e}")))?;
    Ok(LoadedSemanticMetadata::new(pool_id, bundle))
}
