//! Deterministic JSON / binary serialization wrappers.

use crate::bundle::{MetadataExtensionSection, ModuleMetadataRoot, RuntimeTypeFormRoot, SemanticMetadataBundle, TypeUseRecord};
use crate::declaration::{CallableSemanticRecord, DeclarationTypeRecord, FieldSemanticRecord, StaticDomainDeclarationRecord, TypeAliasRecord};
use crate::generic::{GenericSignatureRecord, TypeParameterRecord};
use crate::kind::KindNodeEntry;
use crate::scoped_type::ScopedTypeNodeEntry;
use crate::type_node::TypeNodeEntry;
use crate::validate::{MetadataValidationError, ValidationLimits, validate_metadata_bundle};
use serde::de::{DeserializeSeed, Deserializer, Error, MapAccess, SeqAccess, Visitor};
use std::fmt;

/// Encodes a metadata bundle to a JSON string.
pub fn encode_metadata_json(bundle: &SemanticMetadataBundle) -> Result<String, serde_json::Error> {
    serde_json::to_string(bundle)
}

/// Decodes and validates a metadata bundle from a JSON string.
pub fn decode_metadata_json(json: &str, limits: &ValidationLimits) -> Result<SemanticMetadataBundle, MetadataDecodeError> {
    let mut remaining = limits.max_total_bytes;
    if crate::budgeted::schema_version(json)? < 13 {
        decode_metadata_json_legacy(json, limits, None, &mut remaining)
    } else {
        decode_metadata_json_budgeted(json, limits, None, &mut remaining)
    }
}

/// Decode legacy Record remainders using the canonical Symbol declaration
/// supplied and validated by the loading environment.
pub fn decode_metadata_json_with_symbol(
    json: &str,
    limits: &ValidationLimits,
    symbol: Option<&crate::identity::StableDeclarationRef>,
) -> Result<SemanticMetadataBundle, MetadataDecodeError> {
    let mut remaining = limits.max_total_bytes;
    if crate::budgeted::schema_version(json)? < 13 {
        decode_metadata_json_legacy(json, limits, symbol, &mut remaining)
    } else {
        decode_metadata_json_budgeted(json, limits, symbol, &mut remaining)
    }
}

/// Direct, budgeted decoding of metadata bundle JSON.
///
/// Current-schema construction reserves typed strings/collections before allocation
/// and avoids the Value intermediate. Validator and persistent native-owner budget
/// coverage is still partial (P7 review R1/R7); this is not a complete load memory bound.
/// Caller charges commit only after successful validation.
pub fn decode_metadata_json_budgeted(
    json: &str,
    limits: &ValidationLimits,
    symbol: Option<&crate::identity::StableDeclarationRef>,
    shared_budget: &mut usize,
) -> Result<SemanticMetadataBundle, MetadataDecodeError> {
    let mut budget = crate::allocation::AllocationBudget::new(*shared_budget);
    let bundle = decode_metadata_json_with_allocation_budget(json, limits, symbol, &mut budget)?;
    *shared_budget = budget.remaining();
    Ok(bundle)
}

/// Decodes current-schema metadata into the caller's load transaction. The
/// supplied ledger is replaced only after decoding and validation both pass.
pub fn decode_metadata_json_with_allocation_budget(
    json: &str,
    limits: &ValidationLimits,
    symbol: Option<&crate::identity::StableDeclarationRef>,
    transaction: &mut crate::allocation::AllocationBudget,
) -> Result<SemanticMetadataBundle, MetadataDecodeError> {
    if json.len() > limits.max_total_bytes {
        return Err(MetadataDecodeError::Validation(Box::new(MetadataValidationError::BudgetExceeded {
            resource: "bytes",
            count: json.len(),
            limit: limits.max_total_bytes,
        })));
    }

    let _ = symbol; // Legacy normalization is exposed through the separate compatibility API.
    if crate::budgeted::schema_version(json)? != crate::header::TYPE_METADATA_SCHEMA_VERSION {
        return Err(crate::budgeted::DecodeError::Syntax("budgeted decode requires current schema").into());
    }
    let mut budget = transaction.clone();
    let mut decoder = crate::budgeted::Decoder::new(json, &mut budget, 256)?;
    let bundle = <SemanticMetadataBundle as crate::budgeted::BudgetDecode>::decode(&mut decoder)?;
    decoder.cursor.finish()?;
    crate::validate::validate_metadata_bundle_with_budget(&bundle, limits, &mut budget)?;
    *transaction = budget;
    Ok(bundle)
}

fn decode_metadata_json_legacy(
    json: &str,
    limits: &ValidationLimits,
    symbol: Option<&crate::identity::StableDeclarationRef>,
    shared_budget: &mut usize,
) -> Result<SemanticMetadataBundle, MetadataDecodeError> {
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
    let (budget, _) = inspect_metadata_bundle_allocations(json)?;
    let charge = budget.total_bytes();
    if charge > *shared_budget {
        return Err(MetadataDecodeError::Validation(Box::new(MetadataValidationError::BudgetExceeded {
            resource: "memory",
            count: charge,
            limit: *shared_budget,
        })));
    }
    *shared_budget = shared_budget.saturating_sub(charge);
    Ok(bundle)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MetadataAllocationBudget {
    pub table_bytes: usize,
    pub string_bytes: usize,
    pub container_overhead_bytes: usize,
    pub validator_scratch_bytes: usize,
}

impl MetadataAllocationBudget {
    pub fn total_bytes(&self) -> usize {
        self.table_bytes
            .saturating_add(self.string_bytes)
            .saturating_add(self.container_overhead_bytes)
            .saturating_add(self.validator_scratch_bytes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TopField {
    Header,
    Kinds,
    Types,
    ScopedTypes,
    Parameters,
    GenericSignatures,
    Declarations,
    StaticDomains,
    Aliases,
    Callables,
    Fields,
    ModuleRoots,
    RuntimeRoots,
    Occurrences,
    Extensions,
    Other,
}

impl TopField {
    fn from_key(key: &str) -> Self {
        match key {
            "header" => Self::Header,
            "kinds" => Self::Kinds,
            "types" => Self::Types,
            "scoped_types" => Self::ScopedTypes,
            "parameters" => Self::Parameters,
            "generic_signatures" => Self::GenericSignatures,
            "declarations" => Self::Declarations,
            "static_domains" => Self::StaticDomains,
            "aliases" => Self::Aliases,
            "callables" => Self::Callables,
            "fields" => Self::Fields,
            "module_roots" => Self::ModuleRoots,
            "runtime_roots" => Self::RuntimeRoots,
            "occurrences" => Self::Occurrences,
            "extensions" => Self::Extensions,
            _ => Self::Other,
        }
    }

    fn entry_size(&self) -> usize {
        match self {
            Self::Kinds => std::mem::size_of::<KindNodeEntry>(),
            Self::Types => std::mem::size_of::<TypeNodeEntry>(),
            Self::ScopedTypes => std::mem::size_of::<ScopedTypeNodeEntry>(),
            Self::Parameters => std::mem::size_of::<TypeParameterRecord>(),
            Self::GenericSignatures => std::mem::size_of::<GenericSignatureRecord>(),
            Self::Declarations => std::mem::size_of::<DeclarationTypeRecord>(),
            Self::StaticDomains => std::mem::size_of::<StaticDomainDeclarationRecord>(),
            Self::Aliases => std::mem::size_of::<TypeAliasRecord>(),
            Self::Callables => std::mem::size_of::<CallableSemanticRecord>(),
            Self::Fields => std::mem::size_of::<FieldSemanticRecord>(),
            Self::ModuleRoots => std::mem::size_of::<ModuleMetadataRoot>(),
            Self::RuntimeRoots => std::mem::size_of::<RuntimeTypeFormRoot>(),
            Self::Occurrences => std::mem::size_of::<TypeUseRecord>(),
            Self::Extensions => std::mem::size_of::<MetadataExtensionSection>(),
            _ => 32,
        }
    }
}

#[derive(Default)]
struct AllocationScanner {
    current_top_field: Option<TopField>,
    in_header: bool,
    last_key_was_schema_version: bool,
    schema_version: Option<u32>,
    param_count: usize,
    static_domain_count: usize,
    table_bytes: usize,
    string_bytes: usize,
    container_overhead: usize,
}

struct ValueSeed<'a> {
    scanner: &'a mut AllocationScanner,
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for ValueSeed<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for ValueSeed<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("any JSON value")
    }

    fn visit_bool<E: Error>(self, _: bool) -> Result<(), E> {
        self.scanner.last_key_was_schema_version = false;
        Ok(())
    }

    fn visit_i64<E: Error>(self, v: i64) -> Result<(), E> {
        if self.scanner.last_key_was_schema_version {
            self.scanner.schema_version = Some(v as u32);
        }
        self.scanner.last_key_was_schema_version = false;
        Ok(())
    }

    fn visit_u64<E: Error>(self, v: u64) -> Result<(), E> {
        if self.scanner.last_key_was_schema_version {
            self.scanner.schema_version = Some(v as u32);
        }
        self.scanner.last_key_was_schema_version = false;
        Ok(())
    }

    fn visit_f64<E: Error>(self, _: f64) -> Result<(), E> {
        self.scanner.last_key_was_schema_version = false;
        Ok(())
    }

    fn visit_str<E: Error>(self, s: &str) -> Result<(), E> {
        self.scanner.last_key_was_schema_version = false;
        self.scanner.string_bytes = self.scanner.string_bytes.saturating_add(s.len()).saturating_add(24);
        Ok(())
    }

    fn visit_none<E: Error>(self) -> Result<(), E> {
        self.scanner.last_key_was_schema_version = false;
        Ok(())
    }

    fn visit_unit<E: Error>(self) -> Result<(), E> {
        self.scanner.last_key_was_schema_version = false;
        Ok(())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        self.scanner.last_key_was_schema_version = false;
        self.scanner.container_overhead = self.scanner.container_overhead.saturating_add(24);
        let next_depth = self.depth.saturating_add(1);

        if self.depth == 1 {
            let entry_size = self.scanner.current_top_field.map_or(32, |f| f.entry_size());
            let is_param = self.scanner.current_top_field == Some(TopField::Parameters);
            let is_static_domain = self.scanner.current_top_field == Some(TopField::StaticDomains);

            while seq
                .next_element_seed(ValueSeed {
                    scanner: self.scanner,
                    depth: next_depth,
                })?
                .is_some()
            {
                self.scanner.table_bytes = self.scanner.table_bytes.saturating_add(entry_size);
                if is_param {
                    self.scanner.param_count = self.scanner.param_count.saturating_add(1);
                }
                if is_static_domain {
                    self.scanner.static_domain_count = self.scanner.static_domain_count.saturating_add(1);
                }
            }
        } else {
            while seq
                .next_element_seed(ValueSeed {
                    scanner: self.scanner,
                    depth: next_depth,
                })?
                .is_some()
            {
                self.scanner.table_bytes = self.scanner.table_bytes.saturating_add(32);
            }
        }
        Ok(())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        self.scanner.last_key_was_schema_version = false;
        self.scanner.container_overhead = self.scanner.container_overhead.saturating_add(24);
        let next_depth = self.depth.saturating_add(1);

        while let Some(key) = map.next_key::<String>()? {
            if self.depth == 0 {
                let field = TopField::from_key(&key);
                self.scanner.current_top_field = Some(field);
                self.scanner.in_header = field == TopField::Header;
            } else if self.scanner.in_header && key == "schema_version" {
                self.scanner.last_key_was_schema_version = true;
            } else if self.depth > 1 {
                self.scanner.string_bytes = self.scanner.string_bytes.saturating_add(key.len()).saturating_add(24);
            }
            map.next_value_seed(ValueSeed {
                scanner: self.scanner,
                depth: next_depth,
            })?;
        }
        Ok(())
    }
}

/// Legacy diagnostic estimate, retained for compatibility/tests only.
/// This does not establish an allocation bound and is not used by the current codec.
pub fn inspect_metadata_bundle_allocations(json: &str) -> Result<(MetadataAllocationBudget, u32), MetadataDecodeError> {
    let mut scanner = AllocationScanner::default();
    let mut deserializer = serde_json::Deserializer::from_str(json);
    ValueSeed {
        scanner: &mut scanner,
        depth: 0,
    }
    .deserialize(&mut deserializer)
    .map_err(MetadataDecodeError::Json)?;

    let scratch_bytes = crate::validate::compute_validator_scratch_bytes(scanner.param_count, scanner.static_domain_count);
    let table_bytes = scanner.table_bytes.saturating_add(std::mem::size_of::<SemanticMetadataBundle>());
    let budget = MetadataAllocationBudget {
        table_bytes,
        string_bytes: scanner.string_bytes,
        container_overhead_bytes: scanner.container_overhead,
        validator_scratch_bytes: scratch_bytes,
    };
    Ok((budget, scanner.schema_version.unwrap_or(0)))
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
    #[error(transparent)]
    Budgeted(#[from] crate::budgeted::DecodeError),
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
