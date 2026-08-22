//! Deterministic JSON / binary serialization wrappers.

use crate::bundle::SemanticMetadataBundle;
use crate::validate::{MetadataValidationError, ValidationLimits, validate_metadata_bundle};

/// Encodes a metadata bundle to a JSON string.
pub fn encode_metadata_json(bundle: &SemanticMetadataBundle) -> Result<String, serde_json::Error> {
    serde_json::to_string(bundle)
}

/// Decodes and validates a metadata bundle from a JSON string.
pub fn decode_metadata_json(json: &str, limits: &ValidationLimits) -> Result<SemanticMetadataBundle, MetadataDecodeError> {
    if json.len() > limits.max_total_bytes {
        return Err(MetadataDecodeError::Validation(MetadataValidationError::BudgetExceeded {
            resource: "bytes",
            count: json.len(),
            limit: limits.max_total_bytes,
        }));
    }
    let bundle: SemanticMetadataBundle = serde_json::from_str(json).map_err(MetadataDecodeError::Json)?;
    validate_metadata_bundle(&bundle, limits).map_err(MetadataDecodeError::Validation)?;
    Ok(bundle)
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataDecodeError {
    #[error("json decode error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("validation error: {0}")]
    Validation(#[from] MetadataValidationError),
}
