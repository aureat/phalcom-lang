//! Projection from session-local `StaticValueStore` into durable `StaticValueNodeEntry` table.

use crate::metadata::export::MetadataExportError;
use crate::metadata::stable_identity::{
    StableIdentityContext, to_field_key_ref, to_stable_data_component_with_context, to_stable_declaration_with_context, to_stable_variant_field_with_context,
    to_stable_variant_with_context,
};
use crate::static_value::{StaticSequenceFamily, StaticValueData, StaticValueRef, StaticValueStore};
use crate::types::id::TypeId;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::static_value::{StaticSequenceFamilyRef, StaticValueNode, StaticValueNodeEntry, StaticValueNodeId, compute_static_value_fingerprint};
use phalcom_type_meta::type_node::TypeNodeId;
use std::collections::HashMap;

pub struct StaticValueTableExporter<'a> {
    store: &'a StaticValueStore,
    context: StableIdentityContext<'a>,
    projected: HashMap<StaticValueNode, StaticValueNodeId>,
    entries: Vec<StaticValueNodeEntry>,
    ref_map: HashMap<StaticValueRef, StaticValueNodeId>,
    type_fingerprints: HashMap<TypeNodeId, Fingerprint128>,
}

impl<'a> StaticValueTableExporter<'a> {
    pub fn new(store: &'a StaticValueStore, context: StableIdentityContext<'a>) -> Self {
        Self {
            store,
            context,
            projected: HashMap::new(),
            entries: Vec::new(),
            ref_map: HashMap::new(),
            type_fingerprints: HashMap::new(),
        }
    }

    pub fn export_value<F>(&mut self, val_ref: StaticValueRef, export_type: &mut F) -> Result<StaticValueNodeId, MetadataExportError>
    where
        F: FnMut(TypeId) -> Result<(TypeNodeId, Fingerprint128), MetadataExportError>,
    {
        if let Some(&existing) = self.ref_map.get(&val_ref) {
            return Ok(existing);
        }

        let val_data = self.store.get(val_ref).map_err(|e| MetadataExportError::StaticValue(e.to_string()))?;

        let node = match val_data {
            StaticValueData::Unit => StaticValueNode::Unit,
            StaticValueData::Bool(b) => StaticValueNode::Bool(*b),
            StaticValueData::Char(c) => StaticValueNode::Char(*c),
            StaticValueData::Int(i) => StaticValueNode::Int(i.clone()),
            StaticValueData::Float(f) => StaticValueNode::FloatBits(f.bits()),
            StaticValueData::String(id) => {
                let s = self
                    .store
                    .get_string(*id)
                    .ok_or_else(|| MetadataExportError::StaticValue("unknown static string".into()))?;
                StaticValueNode::String(s.into())
            }
            StaticValueData::Symbol(id) => {
                let s = self
                    .store
                    .get_symbol(*id)
                    .ok_or_else(|| MetadataExportError::StaticValue("unknown static symbol".into()))?;
                StaticValueNode::Symbol(s.into())
            }
            StaticValueData::Type(ty) => {
                let (node_id, fp) = export_type(*ty)?;
                self.type_fingerprints.insert(node_id, fp);
                StaticValueNode::Type(node_id)
            }
            StaticValueData::StaticTerm(ty) => {
                let (node_id, fp) = export_type(*ty)?;
                self.type_fingerprints.insert(node_id, fp);
                StaticValueNode::StaticTerm(node_id)
            }
            StaticValueData::Tuple(id) => {
                let elements = self.store.get_tuple(*id).map_err(|e| MetadataExportError::StaticValue(e.to_string()))?.to_vec();
                let mut child_ids = Vec::with_capacity(elements.len());
                for elem in elements {
                    child_ids.push(self.export_value(elem, export_type)?);
                }
                StaticValueNode::Tuple(child_ids.into_boxed_slice())
            }
            StaticValueData::Sequence(id) => {
                let seq = self.store.get_sequence(*id).map_err(|e| MetadataExportError::StaticValue(e.to_string()))?;
                let family = match seq.family() {
                    StaticSequenceFamily::Array => StaticSequenceFamilyRef::Array,
                    StaticSequenceFamily::Vector => StaticSequenceFamilyRef::Vector,
                };
                let (result_type, fp) = export_type(seq.result_type())?;
                self.type_fingerprints.insert(result_type, fp);
                let elements = seq.elements().to_vec();
                let mut child_ids = Vec::with_capacity(elements.len());
                for elem in elements {
                    child_ids.push(self.export_value(elem, export_type)?);
                }
                StaticValueNode::Sequence {
                    family,
                    result_type,
                    elements: child_ids.into_boxed_slice(),
                }
            }
            StaticValueData::Record(id) => {
                let fields = self
                    .store
                    .get_record(*id)
                    .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                    .to_vec();
                let mut pairs = Vec::with_capacity(fields.len());
                for field in fields {
                    let key_ref = to_field_key_ref(&field.key);
                    let val_id = self.export_value(field.value, export_type)?;
                    pairs.push((key_ref, val_id));
                }
                StaticValueNode::Record(pairs.into_boxed_slice())
            }
            StaticValueData::Pack(id) => {
                let pos_slice = self
                    .store
                    .pack_positionals(*id)
                    .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                    .to_vec();
                let lab_slice = self.store.pack_labeled(*id).map_err(|e| MetadataExportError::StaticValue(e.to_string()))?;
                let mut pos_ids = Vec::with_capacity(pos_slice.len());
                for pos in pos_slice {
                    pos_ids.push(self.export_value(pos, export_type)?);
                }
                let mut lab_pairs = Vec::with_capacity(lab_slice.len());
                for (sym, val) in lab_slice {
                    let name = self
                        .store
                        .get_symbol(sym)
                        .ok_or_else(|| MetadataExportError::StaticValue("unknown pack symbol".into()))?
                        .into();
                    let val_id = self.export_value(val, export_type)?;
                    lab_pairs.push((name, val_id));
                }
                StaticValueNode::Pack {
                    positionals: pos_ids.into_boxed_slice(),
                    labeled: lab_pairs.into_boxed_slice(),
                }
            }
            StaticValueData::Data(id) => {
                let data_val = self.store.get_data(*id).map_err(|e| MetadataExportError::StaticValue(e.to_string()))?.clone();
                let owner = to_stable_declaration_with_context(&data_val.owner, &self.context);
                let (result_type, fp) = export_type(data_val.result_type)?;
                self.type_fingerprints.insert(result_type, fp);
                let mut comp_pairs = Vec::with_capacity(data_val.components.len());
                for (comp_id, val) in data_val.components {
                    let comp_ref = to_stable_data_component_with_context(&comp_id, &self.context);
                    let val_id = self.export_value(val, export_type)?;
                    comp_pairs.push((comp_ref, val_id));
                }
                StaticValueNode::Data {
                    owner,
                    result_type,
                    components: comp_pairs.into_boxed_slice(),
                }
            }
            StaticValueData::Variant(id) => {
                let var_val = self
                    .store
                    .get_variant(*id)
                    .map_err(|e| MetadataExportError::StaticValue(e.to_string()))?
                    .clone();
                let variant = to_stable_variant_with_context(&var_val.variant, &self.context);
                let (result_type, fp) = export_type(var_val.result_type)?;
                self.type_fingerprints.insert(result_type, fp);
                let mut field_pairs = Vec::with_capacity(var_val.fields.len());
                for (field_id, val) in var_val.fields {
                    let field_ref = to_stable_variant_field_with_context(&field_id, &self.context);
                    let val_id = self.export_value(val, export_type)?;
                    field_pairs.push((field_ref, val_id));
                }
                StaticValueNode::Variant {
                    variant,
                    result_type,
                    fields: field_pairs.into_boxed_slice(),
                }
            }
        };

        let node_id = if let Some(&existing) = self.projected.get(&node) {
            existing
        } else {
            let fp = compute_static_value_fingerprint(
                &node,
                |child_id| self.entries[child_id.0 as usize].structural_fingerprint,
                |type_id| *self.type_fingerprints.get(&type_id).unwrap_or(&Fingerprint128::ZERO),
            );
            let id = StaticValueNodeId(self.entries.len() as u32);
            self.entries.push(StaticValueNodeEntry {
                node: node.clone(),
                structural_fingerprint: fp,
            });
            self.projected.insert(node, id);
            id
        };

        self.ref_map.insert(val_ref, node_id);
        Ok(node_id)
    }

    pub fn entries(&self) -> &[StaticValueNodeEntry] {
        &self.entries
    }

    pub fn finish(self) -> Vec<StaticValueNodeEntry> {
        self.entries
    }
}
