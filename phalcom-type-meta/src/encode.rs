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
    if json.len() > limits.max_total_bytes {
        return Err(MetadataDecodeError::Validation(Box::new(MetadataValidationError::BudgetExceeded {
            resource: "bytes",
            count: json.len(),
            limit: limits.max_total_bytes,
        })));
    }
    let bundle: SemanticMetadataBundle = serde_json::from_str(json).map_err(MetadataDecodeError::Json)?;
    validate_metadata_bundle(&bundle, limits)?;
    Ok(bundle)
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataDecodeError {
    #[error("json decode error: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Budgeted(#[from] crate::budgeted::DecodeError),
    #[error("validation error: {0}")]
    Validation(Box<MetadataValidationError>),
}

/// Decodes current-schema metadata into the caller's load transaction. The
/// supplied ledger is replaced only after decoding and validation both pass.
pub fn decode_metadata_json_with_allocation_budget(
    json: &str,
    limits: &ValidationLimits,
    _symbol: Option<&crate::identity::StableDeclarationRef>,
    transaction: &mut crate::allocation::AllocationBudget,
) -> Result<SemanticMetadataBundle, MetadataDecodeError> {
    if json.len() > limits.max_total_bytes {
        return Err(MetadataDecodeError::Validation(Box::new(MetadataValidationError::BudgetExceeded {
            resource: "bytes",
            count: json.len(),
            limit: limits.max_total_bytes,
        })));
    }

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
    StaticDefinitions,
    StaticConstants,
    StaticValues,
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
            "static_definitions" => Self::StaticDefinitions,
            "static_constants" => Self::StaticConstants,
            "static_values" => Self::StaticValues,
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
            Self::StaticDefinitions => std::mem::size_of::<crate::static_definition::StaticDefinitionRecord>(),
            Self::StaticConstants => std::mem::size_of::<crate::static_value::StaticConstantRecord>(),
            Self::StaticValues => std::mem::size_of::<crate::static_value::StaticValueNodeEntry>(),
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

impl From<MetadataValidationError> for MetadataDecodeError {
    fn from(error: MetadataValidationError) -> Self {
        Self::Validation(Box::new(error))
    }
}
