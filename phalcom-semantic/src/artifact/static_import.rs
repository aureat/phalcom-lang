//! Relocates portable static-value rows into the current `StaticValueStore`.

use crate::artifact::type_import::ReconstructedTypeGraph;
use crate::field_key::FieldKey;
use crate::identity::{DataComponentId, DataConstructorId, VariantFieldId, VariantId};
use crate::metadata::import::resolve_stable_declaration;
use crate::scalar::StaticInt;
use crate::static_value::{StaticFloat, StaticSequenceFamily, StaticValueData, StaticValueRef, StaticValueStore};
use crate::types::store::TypeData;
use num_bigint::BigInt;
use phalcom_common::selector::Selector;
use phalcom_compiler_artifact::semantic::static_value::{
    ArtifactStaticPackBackingId, ArtifactStaticValueId, SequenceFamilyV1, StaticValueGraphV1, StaticValueV1,
};
use phalcom_compiler_artifact::semantic::type_store::FieldKeyV1;
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_modules::ProjectUniverse;
use phalcom_type_meta::allocation::AllocationBudget;
use std::str::FromStr;

/// Current-main static-value authority reconstructed from a portable graph.
///
/// The row map is private because artifact IDs are transport references, not
/// semantic identities.
#[derive(Clone, Debug)]
pub struct ImportedStaticValueGraph {
    store: StaticValueStore,
    values: Box<[StaticValueRef]>,
}

impl ImportedStaticValueGraph {
    /// Returns the current-main static-value store.
    pub fn store(&self) -> &StaticValueStore {
        &self.store
    }

    /// Resolves one value row from the artifact's portable graph.
    pub fn value(&self, id: ArtifactStaticValueId) -> Option<StaticValueRef> {
        self.values.get(id.0 as usize).copied()
    }
}

pub(crate) fn reconstruct_static_values(
    graph: &StaticValueGraphV1,
    types: &ReconstructedTypeGraph,
    projects: &ProjectUniverse,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedStaticValueGraph, ArtifactError> {
    graph.validate_type_references(types.types.len())?;
    let order = graph.validate_with_allocation_budget(limits, allocations)?;
    let mut importer = StaticValueImporter::new(graph, types, projects, allocations)?;
    for value in order {
        importer.resolve_value(value)?;
    }

    let StaticValueImporter {
        store,
        values: imported_values,
        ..
    } = importer;
    let values = imported_values
        .into_iter()
        .enumerate()
        .map(|(index, value)| value.ok_or_else(|| referential(format!("static value {index} was not reconstructed"))))
        .collect::<Result<Vec<_>, _>>()?;
    let values = allocations.boxed_slice(values).map_err(ArtifactError::Allocation)?;
    Ok(ImportedStaticValueGraph { store, values })
}

struct StaticValueImporter<'a> {
    graph: &'a StaticValueGraphV1,
    types: &'a ReconstructedTypeGraph,
    projects: &'a ProjectUniverse,
    allocations: &'a mut AllocationBudget,
    store: StaticValueStore,
    values: Vec<Option<StaticValueRef>>,
    pack_bases: Vec<Option<StaticValueRef>>,
}

impl<'a> StaticValueImporter<'a> {
    fn new(
        graph: &'a StaticValueGraphV1,
        types: &'a ReconstructedTypeGraph,
        projects: &'a ProjectUniverse,
        allocations: &'a mut AllocationBudget,
    ) -> Result<Self, ArtifactError> {
        let mut values = allocations
            .vector::<Option<StaticValueRef>>(graph.values.len())
            .map_err(ArtifactError::Allocation)?;
        values.resize(graph.values.len(), None);
        let mut pack_bases = allocations
            .vector::<Option<StaticValueRef>>(graph.pack_backings.len())
            .map_err(ArtifactError::Allocation)?;
        pack_bases.resize(graph.pack_backings.len(), None);
        Ok(Self {
            graph,
            types,
            projects,
            allocations,
            store: StaticValueStore::new(),
            values,
            pack_bases,
        })
    }

    fn resolve_value(&mut self, id: ArtifactStaticValueId) -> Result<StaticValueRef, ArtifactError> {
        if let Some(value) = self.values.get(id.0 as usize).copied().flatten() {
            return Ok(value);
        }
        let row = self
            .graph
            .values
            .get(id.0 as usize)
            .ok_or_else(|| referential(format!("invalid static value row {}", id.0)))?
            .clone();
        let value = match row {
            StaticValueV1::Unit => self.store.intern_unit(),
            StaticValueV1::Bool { value } => self.store.intern_bool(value),
            StaticValueV1::Char { scalar } => {
                let ch = char::from_u32(scalar).ok_or_else(|| referential("invalid static character scalar"))?;
                self.store.intern_char(ch)
            }
            StaticValueV1::Int { decimal } => {
                let integer = BigInt::from_str(&decimal).map_err(|_| referential("invalid static integer"))?;
                if integer.to_string() != decimal {
                    return Err(referential("noncanonical static integer"));
                }
                self.store.intern_int(integer)
            }
            StaticValueV1::Float { bits } => self.store.intern_float(StaticFloat::from_bits(bits)),
            StaticValueV1::String { id } => {
                let text = self
                    .graph
                    .strings
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("invalid static string reference"))?;
                self.store.intern_string(text)
            }
            StaticValueV1::Symbol { id } => {
                let text = self
                    .graph
                    .symbols
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("invalid static symbol reference"))?;
                self.store.intern_symbol(text)
            }
            StaticValueV1::Type { ty } => {
                let ty = self.type_id(ty)?;
                self.store.intern_type(ty, &self.types.store).map_err(|error| referential(error.to_string()))?
            }
            StaticValueV1::StaticTerm { ty } => {
                let ty = self.type_id(ty)?;
                self.store
                    .intern_static_term(ty, &self.types.store)
                    .map_err(|error| referential(error.to_string()))?
            }
            StaticValueV1::Tuple { id: tuple_id } => {
                let row = self
                    .graph
                    .tuples
                    .get(tuple_id.0 as usize)
                    .ok_or_else(|| referential("invalid static tuple reference"))?;
                let mut children = self.allocations.vector::<StaticValueRef>(row.len()).map_err(ArtifactError::Allocation)?;
                for child in row {
                    children.push(self.value_ref(*child)?);
                }
                self.store.intern_tuple(children).map_err(|error| referential(error.to_string()))?
            }
            StaticValueV1::Sequence { id: sequence_id } => {
                let row = self
                    .graph
                    .sequences
                    .get(sequence_id.0 as usize)
                    .ok_or_else(|| referential("invalid static sequence reference"))?;
                let result_type = self.type_id(row.result_type)?;
                let mut children = self
                    .allocations
                    .vector::<StaticValueRef>(row.elements.len())
                    .map_err(ArtifactError::Allocation)?;
                for child in &row.elements {
                    children.push(self.value_ref(*child)?);
                }
                let value = self
                    .store
                    .intern_sequence(result_type, children, &self.types.store)
                    .map_err(|error| referential(error.to_string()))?;
                let actual_family = match self.store.get(value).map_err(|error| referential(error.to_string()))? {
                    StaticValueData::Sequence(id) => self.store.get_sequence(*id).map_err(|error| referential(error.to_string()))?.family(),
                    _ => return Err(referential("sequence reconstruction produced a non-sequence value")),
                };
                let expected_family = match row.family {
                    SequenceFamilyV1::Array => StaticSequenceFamily::Array,
                    SequenceFamilyV1::Vector => StaticSequenceFamily::Vector,
                };
                if actual_family != expected_family {
                    return Err(referential("static sequence family disagrees with its relocated result type"));
                }
                value
            }
            StaticValueV1::Record { id: record_id } => {
                let row = self
                    .graph
                    .records
                    .get(record_id.0 as usize)
                    .ok_or_else(|| referential("invalid static record reference"))?;
                let mut fields = self
                    .allocations
                    .vector::<(FieldKey, StaticValueRef)>(row.len())
                    .map_err(ArtifactError::Allocation)?;
                for field in row {
                    fields.push((self.field_key(&field.key)?, self.value_ref(field.value)?));
                }
                self.store.intern_record(fields).map_err(|error| referential(error.to_string()))?
            }
            StaticValueV1::Data { id: data_id } => {
                let row = self
                    .graph
                    .data_values
                    .get(data_id.0 as usize)
                    .ok_or_else(|| referential("invalid static data reference"))?;
                let owner = resolve_stable_declaration(&row.owner, self.projects).ok_or_else(|| referential("unresolved static data owner"))?;
                let constructor_owner =
                    resolve_stable_declaration(&row.constructor_owner, self.projects).ok_or_else(|| referential("unresolved static data constructor owner"))?;
                if owner != constructor_owner {
                    return Err(referential("static data constructor does not belong to its data declaration"));
                }
                let result_type = self.type_id(row.result_type)?;
                if self.types.store.nominal_origin_declaration(result_type) != Some(&owner) {
                    return Err(referential("static data result type belongs to another declaration"));
                }
                let mut components = self
                    .allocations
                    .vector::<(DataComponentId, StaticValueRef)>(row.components.len())
                    .map_err(ArtifactError::Allocation)?;
                for component in &row.components {
                    components.push((DataComponentId::new(owner.clone(), component.index), self.value_ref(component.value)?));
                }
                self.store
                    .intern_data_reconstructed(owner.clone(), DataConstructorId::new(owner), result_type, components)
                    .map_err(|error| referential(error.to_string()))?
            }
            StaticValueV1::Variant { id: variant_id } => {
                let row = self
                    .graph
                    .variant_values
                    .get(variant_id.0 as usize)
                    .ok_or_else(|| referential("invalid static variant reference"))?;
                let owner = resolve_stable_declaration(&row.variant.owner, self.projects).ok_or_else(|| referential("unresolved static variant owner"))?;
                let selector =
                    Selector::try_decode_exact(&row.variant.selector).map_err(|error| referential(format!("invalid static variant selector: {error}")))?;
                let variant = VariantId::new(owner, selector);
                let result_type = self.type_id(row.result_type)?;
                match self.types.store.get(result_type) {
                    TypeData::ExactCase { variant: exact, .. } if self.types.store.variant_identity(*exact) == &variant => {}
                    _ => return Err(referential("static variant identity disagrees with its relocated exact-case type")),
                }
                let mut fields = self
                    .allocations
                    .vector::<(VariantFieldId, StaticValueRef)>(row.fields.len())
                    .map_err(ArtifactError::Allocation)?;
                for field in &row.fields {
                    fields.push((VariantFieldId::new(variant.clone(), field.index), self.value_ref(field.value)?));
                }
                self.store
                    .intern_variant_reconstructed(variant, result_type, fields)
                    .map_err(|error| referential(error.to_string()))?
            }
            StaticValueV1::Pack { id: pack_id } => {
                let pack = self
                    .graph
                    .packs
                    .get(pack_id.0 as usize)
                    .ok_or_else(|| referential("invalid static pack reference"))?;
                let base = self.pack_base(pack.backing)?;
                self.store
                    .intern_pack_slice(base, pack.positional_start, pack.positional_len, pack.labeled_start, pack.labeled_len)
                    .map_err(|error| referential(error.to_string()))?
            }
        };
        let slot = self
            .values
            .get_mut(id.0 as usize)
            .ok_or_else(|| referential("static value relocation slot is missing"))?;
        *slot = Some(value);
        Ok(value)
    }

    fn value_ref(&self, id: ArtifactStaticValueId) -> Result<StaticValueRef, ArtifactError> {
        self.values
            .get(id.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| referential(format!("static value dependency {} was not relocated first", id.0)))
    }

    fn type_id(&self, id: phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId) -> Result<crate::types::id::TypeId, ArtifactError> {
        self.types
            .types
            .get(id.0 as usize)
            .copied()
            .ok_or_else(|| referential(format!("invalid relocated type reference {}", id.0)))
    }

    fn field_key(&self, key: &FieldKeyV1) -> Result<FieldKey, ArtifactError> {
        Ok(match key {
            FieldKeyV1::Symbol { value } => FieldKey::symbol(value.as_str()),
            FieldKeyV1::String { value } => FieldKey::string(value.as_str()),
            FieldKeyV1::Int { decimal } => {
                let integer = BigInt::from_str(decimal).map_err(|_| referential("invalid static record integer key"))?;
                if integer.to_string() != *decimal {
                    return Err(referential("noncanonical static record integer key"));
                }
                FieldKey::int(StaticInt::new(integer))
            }
            FieldKeyV1::Bool { value } => FieldKey::bool(*value),
        })
    }

    fn pack_base(&mut self, id: ArtifactStaticPackBackingId) -> Result<StaticValueRef, ArtifactError> {
        if let Some(base) = self.pack_bases.get(id.0 as usize).copied().flatten() {
            return Ok(base);
        }
        let row = self
            .graph
            .pack_backings
            .get(id.0 as usize)
            .ok_or_else(|| referential("invalid static pack backing reference"))?;
        let mut positionals = self
            .allocations
            .vector::<StaticValueRef>(row.positionals.len())
            .map_err(ArtifactError::Allocation)?;
        for value in &row.positionals {
            positionals.push(self.value_ref(*value)?);
        }
        let mut labeled = self
            .allocations
            .vector::<(crate::static_value::StaticSymbolId, StaticValueRef)>(row.labels.len())
            .map_err(ArtifactError::Allocation)?;
        for (label, value) in row.labels.iter().zip(&row.labeled_values) {
            let spelling = self
                .graph
                .symbols
                .get(label.0 as usize)
                .ok_or_else(|| referential("invalid static pack label reference"))?;
            let symbol = self.store.intern_symbol_id(spelling);
            labeled.push((symbol, self.value_ref(*value)?));
        }
        let base = self.store.intern_pack(positionals, labeled).map_err(|error| referential(error.to_string()))?;
        let slot = self
            .pack_bases
            .get_mut(id.0 as usize)
            .ok_or_else(|| referential("static pack backing relocation slot is missing"))?;
        *slot = Some(base);
        Ok(base)
    }
}

fn referential(message: impl Into<String>) -> ArtifactError {
    ArtifactError::Validation(ValidationError::ReferentialIntegrity(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_compiler_artifact::semantic::static_value::{
        ArtifactStaticSymbolId, ArtifactStaticTupleId, StaticPackBackingV1, StaticPackSliceV1, StaticRecordFieldV1, StaticValueV1,
    };
    use phalcom_compiler_artifact::semantic::type_store::TypeGraphV1;

    fn empty_type_graph() -> TypeGraphV1 {
        TypeGraphV1 {
            types: vec![],
            kinds: vec![],
            parameters: vec![],
            current_parameters: vec![],
            rows: vec![],
            families: vec![],
            variants: vec![],
            static_domains: vec![],
            static_constructors: vec![],
            scoped: vec![],
            lambdas: vec![],
            variance_overrides: vec![],
        }
    }

    #[test]
    fn reconstructs_static_scalars_and_structural_values_into_the_live_store() {
        let type_graph = empty_type_graph();
        let mut budget = AllocationBudget::new(1024 * 1024);
        let types = super::super::type_import::reconstruct_type_graph(&type_graph, &ValidationLimits::default(), &ProjectUniverse::new(), &mut budget).unwrap();
        let graph = StaticValueGraphV1 {
            values: vec![
                StaticValueV1::Int { decimal: "42".into() },
                StaticValueV1::Int { decimal: "7".into() },
                StaticValueV1::String {
                    id: phalcom_compiler_artifact::semantic::static_value::ArtifactStaticStringId(0),
                },
                StaticValueV1::Tuple { id: ArtifactStaticTupleId(0) },
                StaticValueV1::Record {
                    id: phalcom_compiler_artifact::semantic::static_value::ArtifactStaticRecordId(0),
                },
                StaticValueV1::Pack {
                    id: phalcom_compiler_artifact::semantic::static_value::ArtifactStaticPackId(0),
                },
            ],
            strings: vec!["payload".into()],
            symbols: vec!["named".into()],
            tuples: vec![vec![ArtifactStaticValueId(0), ArtifactStaticValueId(1)]],
            sequences: vec![],
            records: vec![vec![StaticRecordFieldV1 {
                key: FieldKeyV1::String { value: "field".into() },
                value: ArtifactStaticValueId(2),
            }]],
            data_values: vec![],
            variant_values: vec![],
            pack_backings: vec![StaticPackBackingV1 {
                positionals: vec![ArtifactStaticValueId(0)],
                labels: vec![ArtifactStaticSymbolId(0)],
                labeled_values: vec![ArtifactStaticValueId(1)],
            }],
            packs: vec![StaticPackSliceV1 {
                backing: ArtifactStaticPackBackingId(0),
                positional_start: 0,
                positional_len: 1,
                labeled_start: 0,
                labeled_len: 1,
            }],
        };

        let imported = reconstruct_static_values(&graph, &types, &ProjectUniverse::new(), &ValidationLimits::default(), &mut budget).unwrap();
        let tuple = imported.value(ArtifactStaticValueId(3)).unwrap();
        let StaticValueData::Tuple(tuple_id) = imported.store.get(tuple).unwrap() else {
            panic!("portable tuple did not become a live tuple");
        };
        assert_eq!(imported.store.get_tuple(*tuple_id).unwrap().len(), 2);

        let record = imported.value(ArtifactStaticValueId(4)).unwrap();
        let StaticValueData::Record(record_id) = imported.store.get(record).unwrap() else {
            panic!("portable record did not become a live record");
        };
        assert_eq!(imported.store.get_record(*record_id).unwrap()[0].key.as_string(), Some("field"));

        let pack = imported.value(ArtifactStaticValueId(5)).unwrap();
        let StaticValueData::Pack(pack_id) = imported.store.get(pack).unwrap() else {
            panic!("portable pack did not become a live pack");
        };
        assert_eq!(imported.store.pack_positionals(*pack_id).unwrap().len(), 1);
        assert_eq!(imported.store.pack_labeled(*pack_id).unwrap().len(), 1);
    }

    #[test]
    fn rejects_a_static_value_graph_cycle_before_reconstruction() {
        let type_graph = empty_type_graph();
        let mut budget = AllocationBudget::new(1024 * 1024);
        let types = super::super::type_import::reconstruct_type_graph(&type_graph, &ValidationLimits::default(), &ProjectUniverse::new(), &mut budget).unwrap();
        let graph = StaticValueGraphV1 {
            values: vec![StaticValueV1::Tuple { id: ArtifactStaticTupleId(0) }],
            strings: vec![],
            symbols: vec![],
            tuples: vec![vec![ArtifactStaticValueId(0)]],
            sequences: vec![],
            records: vec![],
            data_values: vec![],
            variant_values: vec![],
            pack_backings: vec![],
            packs: vec![],
        };
        let error = reconstruct_static_values(&graph, &types, &ProjectUniverse::new(), &ValidationLimits::default(), &mut budget).unwrap_err();
        assert!(matches!(error, ArtifactError::Validation(ValidationError::ReferentialIntegrity(message)) if message.contains("cyclic")));
    }
}
