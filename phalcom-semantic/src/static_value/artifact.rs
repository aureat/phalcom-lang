//! Semantic-owned static-value graph relocation (`PERF003.C2.P7.T6`).
//!
//! Import reconstructs every canonical value through the ordinary validated
//! factories in dependency order, so a loaded store has fresh lineage and
//! participates in normal interning. Wire validation is owned by the artifact
//! crate; declaration/type resolution is supplied by the loader.

use crate::data_semantics::DataSemanticTable;
use crate::enum_semantics::EnumSemanticTable;
use crate::field_key::FieldKey;
use crate::identity::{DeclarationId, VariantId};
use crate::scalar::StaticInt;
use crate::static_value::store::{StaticFloat, StaticSymbolId, StaticValueRef, StaticValueStore, StaticValueStoreError, StaticValueStoreId};
use crate::types::artifact::TypeRelocation;
use num_bigint::BigInt;
use phalcom_compiler_artifact::semantic::static_value::{ArtifactStaticPackBackingId, ArtifactStaticSymbolId, ArtifactStaticValueId, StaticValueGraphV1, StaticValueV1};
use phalcom_compiler_artifact::semantic::type_store::FieldKeyV1;
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::identity::{StableDeclarationRef, StableVariantRef};
use std::collections::BTreeMap;

/// Relocation from artifact-local static identities into one fresh store lineage.
#[derive(Clone, Debug)]
pub struct StaticRelocation {
    store_id: StaticValueStoreId,
    values: BTreeMap<ArtifactStaticValueId, StaticValueRef>,
    symbols: BTreeMap<ArtifactStaticSymbolId, StaticSymbolId>,
}

impl StaticRelocation {
    /// Returns the relocated reference for one artifact value identity.
    pub fn value(&self, id: ArtifactStaticValueId) -> Result<StaticValueRef, ArtifactError> {
        self.values.get(&id).copied().ok_or_else(|| referential(format!("unrelocated static value {}", id.0)))
    }

    /// Returns the relocated symbol identity for one artifact symbol identity.
    pub fn symbol(&self, id: ArtifactStaticSymbolId) -> Result<StaticSymbolId, ArtifactError> {
        self.symbols.get(&id).copied().ok_or_else(|| referential(format!("unrelocated static symbol {}", id.0)))
    }

    /// Fails when used against a foreign store lineage.
    pub fn verify_store(&self, store: &StaticValueStore) -> Result<(), ArtifactError> {
        if self.store_id != store.id() {
            return Err(referential("static relocation belongs to a different store lineage"));
        }
        Ok(())
    }
}

/// Resolves stable declaration owners against loaded canonical authority.
pub type DeclarationResolver<'a> = &'a dyn Fn(&StableDeclarationRef) -> Result<DeclarationId, ArtifactError>;
/// Resolves stable exact-case owners against loaded canonical authority.
pub type VariantResolver<'a> = &'a dyn Fn(&StableVariantRef) -> Result<VariantId, ArtifactError>;

/// Reconstructs a validated static-value graph into a fresh store lineage.
///
/// `graph` must already be decoded (the artifact codec validates local edges,
/// canonical scalar encodings, and acyclicity, and returns dependency-first
/// value order). Type references relocate through `type_relocation`; data and
/// variant payloads resolve through the supplied structural tables so
/// `intern_data`/`intern_variant` retain full owner/type/index validation.
pub fn import_static_value_graph(
    graph: &StaticValueGraphV1,
    types: &crate::types::store::TypeStore,
    type_relocation: &TypeRelocation,
    data: &DataSemanticTable,
    enums: &EnumSemanticTable,
    resolve_declaration: DeclarationResolver<'_>,
    resolve_variant: VariantResolver<'_>,
    limits: &ValidationLimits,
) -> Result<(StaticValueStore, StaticRelocation), ArtifactError> {
    let order = graph.validate(limits)?;
    let mut store = StaticValueStore::new();
    let mut relocation = StaticRelocation {
        store_id: store.id(),
        values: BTreeMap::new(),
        symbols: BTreeMap::new(),
    };
    for (index, spelling) in graph.symbols.iter().enumerate() {
        let id = store.intern_symbol_id(spelling);
        relocation.symbols.insert(ArtifactStaticSymbolId(index as u32), id);
    }
    let mut whole_packs: BTreeMap<ArtifactStaticPackBackingId, StaticValueRef> = BTreeMap::new();
    for id in order {
        let reference = match &graph.values[id.0 as usize] {
            StaticValueV1::Unit => store.intern_unit(),
            StaticValueV1::Bool { value } => store.intern_bool(*value),
            StaticValueV1::Char { scalar } => {
                let scalar = char::from_u32(*scalar).ok_or_else(|| referential("invalid static character scalar"))?;
                store.intern_char(scalar)
            }
            StaticValueV1::Int { decimal } => {
                let value = BigInt::parse_bytes(decimal.as_bytes(), 10).ok_or_else(|| referential("unparseable static integer decimal"))?;
                store.intern_int(value)
            }
            StaticValueV1::Float { bits } => store.intern_float(StaticFloat::from_bits(*bits)),
            StaticValueV1::String { id } => {
                let text = graph.strings.get(id.0 as usize).ok_or_else(|| referential("unknown static string identity"))?;
                store.intern_string(text.as_str())
            }
            StaticValueV1::Symbol { id } => {
                let spelling = graph.symbols.get(id.0 as usize).ok_or_else(|| referential("unknown static symbol identity"))?;
                store.intern_symbol(spelling.as_str())
            }
            StaticValueV1::Type { ty } => store
                .intern_type(type_relocation.ty(*ty)?, types)
                .map_err(store_error)?,
            StaticValueV1::StaticTerm { ty } => store
                .intern_static_term(type_relocation.ty(*ty)?, types)
                .map_err(store_error)?,
            StaticValueV1::Tuple { id } => {
                let elements = graph
                    .tuples
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("unknown static tuple identity"))?
                    .iter()
                    .map(|child| relocation.value(*child))
                    .collect::<Result<Vec<_>, _>>()?;
                store.intern_tuple(elements).map_err(store_error)?
            }
            StaticValueV1::Sequence { id } => {
                let sequence = graph.sequences.get(id.0 as usize).ok_or_else(|| referential("unknown static sequence identity"))?;
                let elements = sequence
                    .elements
                    .iter()
                    .map(|child| relocation.value(*child))
                    .collect::<Result<Vec<_>, _>>()?;
                store
                    .intern_sequence(type_relocation.ty(sequence.result_type)?, elements, types)
                    .map_err(store_error)?
            }
            StaticValueV1::Record { id } => {
                let fields = graph
                    .records
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("unknown static record identity"))?
                    .iter()
                    .map(|field| Ok::<_, ArtifactError>((key_from_wire(&field.key)?, relocation.value(field.value)?)))
                    .collect::<Result<Vec<_>, _>>()?;
                store.intern_record(fields).map_err(store_error)?
            }
            StaticValueV1::Data { id } => {
                let value = graph.data_values.get(id.0 as usize).ok_or_else(|| referential("unknown static data identity"))?;
                let owner = resolve_declaration(&value.owner)?;
                let info = data.data_info(&owner).ok_or_else(|| referential("static data owner has no loaded contract"))?;
                if value.components.len() != info.components.len() {
                    return Err(referential("static data component count disagrees with loaded contract"));
                }
                let components = info
                    .components
                    .iter()
                    .zip(value.components.iter())
                    .map(|(component, wire)| Ok((component.id.clone(), relocation.value(wire.value)?)))
                    .collect::<Result<Vec<_>, ArtifactError>>()?;
                store
                    .intern_data(info, type_relocation.ty(value.result_type)?, components, types)
                    .map_err(store_error)?
            }
            StaticValueV1::Variant { id } => {
                let value = graph.variant_values.get(id.0 as usize).ok_or_else(|| referential("unknown static variant identity"))?;
                let variant = resolve_variant(&value.variant)?;
                let info = enums.variant_info(&variant).ok_or_else(|| referential("static variant has no loaded contract"))?;
                if value.fields.len() != info.fields.len() {
                    return Err(referential("static variant field count disagrees with loaded contract"));
                }
                let fields = info
                    .fields
                    .iter()
                    .zip(value.fields.iter())
                    .map(|(field, wire)| Ok((field.id.clone(), relocation.value(wire.value)?)))
                    .collect::<Result<Vec<_>, ArtifactError>>()?;
                store
                    .intern_variant(info, type_relocation.ty(value.result_type)?, fields, types)
                    .map_err(store_error)?
            }
            StaticValueV1::Pack { id } => {
                let pack = graph.packs.get(id.0 as usize).ok_or_else(|| referential("unknown static pack identity"))?;
                let backing = graph
                    .pack_backings
                    .get(pack.backing.0 as usize)
                    .ok_or_else(|| referential("unknown static pack backing identity"))?;
                let whole = pack.positional_start == 0
                    && pack.positional_len as usize == backing.positionals.len()
                    && pack.labeled_start == 0
                    && pack.labeled_len as usize == backing.labels.len();
                if whole {
                    let positionals = backing
                        .positionals
                        .iter()
                        .map(|child| relocation.value(*child))
                        .collect::<Result<Vec<_>, _>>()?;
                    let labeled = backing
                        .labels
                        .iter()
                        .zip(backing.labeled_values.iter())
                        .map(|(label, child)| Ok((relocation.symbols.get(label).copied().ok_or_else(|| referential("unrelocated static pack label"))?, relocation.value(*child)?)))
                        .collect::<Result<Vec<_>, ArtifactError>>()?;
                    store.intern_pack(positionals, labeled).map_err(store_error)?
                } else {
                    let whole_ref = match whole_packs.get(&pack.backing) {
                        Some(reference) => *reference,
                        None => {
                            let positionals = backing
                                .positionals
                                .iter()
                                .map(|child| relocation.value(*child))
                                .collect::<Result<Vec<_>, _>>()?;
                            let labeled = backing
                                .labels
                                .iter()
                                .zip(backing.labeled_values.iter())
                                .map(|(label, child)| {
                                    Ok((
                                        relocation.symbols.get(label).copied().ok_or_else(|| referential("unrelocated static pack label"))?,
                                        relocation.value(*child)?,
                                    ))
                                })
                                .collect::<Result<Vec<_>, ArtifactError>>()?;
                            let reference = store.intern_pack(positionals, labeled).map_err(store_error)?;
                            whole_packs.insert(pack.backing, reference);
                            reference
                        }
                    };
                    store
                        .intern_pack_slice(
                            whole_ref,
                            pack.positional_start,
                            pack.positional_len,
                            pack.labeled_start,
                            pack.labeled_len,
                        )
                        .map_err(store_error)?
                }
            }
        };
        relocation.values.insert(id, reference);
    }
    Ok((store, relocation))
}

fn key_from_wire(key: &FieldKeyV1) -> Result<FieldKey, ArtifactError> {
    Ok(match key {
        FieldKeyV1::Symbol { value } => FieldKey::symbol(value.as_str()),
        FieldKeyV1::String { value } => FieldKey::string(value.as_str()),
        FieldKeyV1::Int { decimal } => {
            let value = BigInt::parse_bytes(decimal.as_bytes(), 10).ok_or_else(|| referential("unparseable static record key"))?;
            FieldKey::int(StaticInt::new(value))
        }
        FieldKeyV1::Bool { value } => FieldKey::bool(*value),
    })
}

fn referential(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

fn store_error(error: StaticValueStoreError) -> ArtifactError {
    ValidationError::ReferentialIntegrity(error.to_string()).into()
}
