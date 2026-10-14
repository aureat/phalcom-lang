//! Deterministic JSON / binary serialization wrappers.

use crate::bundle::SemanticMetadataBundle;
use crate::validate::{MetadataValidationError, ValidationLimits, validate_metadata_bundle};

/// Encodes a metadata bundle to a JSON string.
pub fn encode_metadata_json(bundle: &SemanticMetadataBundle) -> Result<String, serde_json::Error> {
    serde_json::to_string(bundle)
}

/// Decodes and validates a metadata bundle from a JSON string.
pub fn decode_metadata_json(json: &str, limits: &ValidationLimits) -> Result<SemanticMetadataBundle, MetadataDecodeError> {
    decode_metadata_json_with_symbol(json, limits, None)
}

/// Decode legacy Record remainders using the canonical Symbol declaration
/// supplied and validated by the loading environment.
pub fn decode_metadata_json_with_symbol(
    json: &str,
    limits: &ValidationLimits,
    symbol: Option<&crate::identity::StableDeclarationRef>,
) -> Result<SemanticMetadataBundle, MetadataDecodeError> {
    if json.len() > limits.max_total_bytes {
        return Err(MetadataDecodeError::Validation(Box::new(MetadataValidationError::BudgetExceeded {
            resource: "bytes",
            count: json.len(),
            limit: limits.max_total_bytes,
        })));
    }
    let mut value: serde_json::Value = serde_json::from_str(json).map_err(MetadataDecodeError::Json)?;
    let legacy = value
        .pointer("/header/schema_version")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|version| version < 13);
    let affected = legacy && has_legacy_record_remainder(&value);
    if legacy {
        normalize_legacy_record_field_names(&mut value);
        if affected {
            let symbol = symbol.ok_or(MetadataDecodeError::CanonicalSymbolContextRequired)?;
            if !crate::legacy_record::validate_symbol_identity(symbol) {
                return Err(MetadataDecodeError::InvalidCanonicalSymbolContext);
            }
            crate::legacy_record::normalize_remainder_json(&mut value);
        }
    }
    let mut bundle: SemanticMetadataBundle = serde_json::from_value(value).map_err(MetadataDecodeError::Json)?;
    if affected {
        bundle =
            crate::legacy_record::relocate(bundle, symbol.expect("checked above").clone()).map_err(|_| MetadataDecodeError::InvalidCanonicalSymbolContext)?;
    }
    validate_metadata_bundle(&bundle, limits)?;
    Ok(bundle)
}

fn has_legacy_record_remainder(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(object) => {
            object.get("remainder").is_some_and(|remainder| !remainder.is_null())
                || object.get("tail").and_then(|tail| tail.get("Anonymous")).is_some()
                || object.values().any(has_legacy_record_remainder)
        }
        serde_json::Value::Array(values) => values.iter().any(has_legacy_record_remainder),
        _ => false,
    }
}

/// Pre-v13 fixed Record fields used a Symbol-only `name`. This conversion does
/// not require a Symbol *type* identity; anonymous remainders do and are handled
/// separately by contextual legacy graph normalization.
fn normalize_legacy_record_field_names(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            for field_name in ["Record", "fields"] {
                if let Some(serde_json::Value::Array(fields)) = object.get_mut(field_name) {
                    for field in fields {
                        if let Some(field) = field.as_object_mut() {
                            if field.contains_key("ty") && !field.contains_key("key") {
                                if let Some(name) = field.remove("name") {
                                    field.insert("key".into(), serde_json::json!({"Symbol": name}));
                                }
                            }
                        }
                    }
                }
            }
            for child in object.values_mut() {
                normalize_legacy_record_field_names(child);
            }
        }
        serde_json::Value::Array(children) => {
            for child in children {
                normalize_legacy_record_field_names(child);
            }
        }
        _ => {}
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataDecodeError {
    #[error("json decode error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("validation error: {0}")]
    Validation(Box<MetadataValidationError>),
    #[error("legacy Record remainder requires a validated canonical Symbol identity to decode")]
    CanonicalSymbolContextRequired,
    #[error("invalid canonical Symbol identity for legacy Record metadata")]
    InvalidCanonicalSymbolContext,
}

impl From<MetadataValidationError> for MetadataDecodeError {
    fn from(error: MetadataValidationError) -> Self {
        Self::Validation(Box::new(error))
    }
}
