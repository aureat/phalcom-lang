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
use crate::metadata::stable_identity::{to_stable_declaration, to_stable_variant};
use crate::static_value::store::{
    FrozenStaticValuePrefix, StaticDataValue, StaticDataValueId, StaticFloat, StaticPackBacking, StaticPackBackingId, StaticPackData, StaticPackId,
    StaticRecordField, StaticRecordId, StaticSequenceData, StaticSequenceFamily, StaticSequenceId, StaticStringId, StaticSymbolId, StaticTupleId,
    StaticValueData, StaticValueId, StaticValueRef, StaticValueStore, StaticValueStoreError, StaticValueStoreId, StaticVariantValue, StaticVariantValueId,
};
use crate::types::store::artifact::{TypeExportContext, TypeRelocation};
use phalcom_compiler_artifact::semantic::static_value::*;
use phalcom_compiler_artifact::semantic::type_store::{CanonicalTypeRootKey, FieldKeyV1};
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::{StableDeclarationRef, StableVariantRef};
use std::collections::{BTreeMap, BTreeSet};

/// Relocation from artifact-local static identities into one fresh store lineage.
#[derive(Clone, Debug)]
pub struct StaticRelocation {
    store_id: StaticValueStoreId,
    values: Box<[StaticValueRef]>,
    symbols: Box<[StaticSymbolId]>,
}

impl StaticRelocation {
    /// Returns the relocated reference for one artifact value identity.
    pub fn value(&self, id: ArtifactStaticValueId) -> Result<StaticValueRef, ArtifactError> {
        self.values
            .get(id.0 as usize)
            .copied()
            .ok_or_else(|| referential(format!("unrelocated static value {}", id.0)))
    }

    /// Returns the relocated symbol identity for one artifact symbol identity.
    pub fn symbol(&self, id: ArtifactStaticSymbolId) -> Result<StaticSymbolId, ArtifactError> {
        self.symbols
            .get(id.0 as usize)
            .copied()
            .ok_or_else(|| referential(format!("unrelocated static symbol {}", id.0)))
    }

    /// Fails when used against a foreign store lineage.
    pub fn verify_store(&self, store: &StaticValueStore) -> Result<(), ArtifactError> {
        if self.store_id != store.id() {
            return Err(referential("static relocation belongs to a different store lineage"));
        }
        Ok(())
    }
}

/// Guarded export context for projecting static values into a portable graph.
#[derive(Debug)]
pub struct StaticExportContext {
    store_id: StaticValueStoreId,
    closed: bool,
    roots: BTreeMap<CanonicalTypeRootKey, StaticValueRef>,
    value_ids: BTreeMap<StaticValueId, ArtifactStaticValueId>,
    string_ids: BTreeMap<StaticStringId, ArtifactStaticStringId>,
    symbol_ids: BTreeMap<StaticSymbolId, ArtifactStaticSymbolId>,
    tuple_ids: BTreeMap<StaticTupleId, ArtifactStaticTupleId>,
    sequence_ids: BTreeMap<StaticSequenceId, ArtifactStaticSequenceId>,
    record_ids: BTreeMap<StaticRecordId, ArtifactStaticRecordId>,
    data_ids: BTreeMap<StaticDataValueId, ArtifactStaticDataId>,
    variant_ids: BTreeMap<StaticVariantValueId, ArtifactStaticVariantId>,
    pack_backing_ids: BTreeMap<StaticPackBackingId, ArtifactStaticPackBackingId>,
    pack_ids: BTreeMap<StaticPackId, ArtifactStaticPackId>,
    value_order: Vec<StaticValueId>,
    string_order: Vec<StaticStringId>,
    symbol_order: Vec<StaticSymbolId>,
    tuple_order: Vec<StaticTupleId>,
    sequence_order: Vec<StaticSequenceId>,
    record_order: Vec<StaticRecordId>,
    data_order: Vec<StaticDataValueId>,
    variant_order: Vec<StaticVariantValueId>,
    pack_backing_order: Vec<StaticPackBackingId>,
    pack_order: Vec<StaticPackId>,
}

impl StaticExportContext {
    pub fn new(store: &StaticValueStore) -> Self {
        Self {
            store_id: store.id(),
            closed: false,
            roots: BTreeMap::new(),
            value_ids: BTreeMap::new(),
            string_ids: BTreeMap::new(),
            symbol_ids: BTreeMap::new(),
            tuple_ids: BTreeMap::new(),
            sequence_ids: BTreeMap::new(),
            record_ids: BTreeMap::new(),
            data_ids: BTreeMap::new(),
            variant_ids: BTreeMap::new(),
            pack_backing_ids: BTreeMap::new(),
            pack_ids: BTreeMap::new(),
            value_order: Vec::new(),
            string_order: Vec::new(),
            symbol_order: Vec::new(),
            tuple_order: Vec::new(),
            sequence_order: Vec::new(),
            record_order: Vec::new(),
            data_order: Vec::new(),
            variant_order: Vec::new(),
            pack_backing_order: Vec::new(),
            pack_order: Vec::new(),
        }
    }

    pub fn expected_lineage(&self) -> StaticValueStoreId {
        self.store_id
    }

    pub fn verify_store(&self, store: &StaticValueStore) -> Result<(), ArtifactError> {
        if self.store_id != store.id() {
            return Err(referential("foreign static value store lineage"));
        }
        Ok(())
    }

    pub fn register_root(&mut self, key: CanonicalTypeRootKey, reference: StaticValueRef) -> Result<(), ArtifactError> {
        if self.store_id != reference.store_id() {
            return Err(referential("registered static root belongs to foreign store lineage"));
        }
        if self.closed {
            return Err(referential("static export graph is already closed"));
        }
        if self.roots.get(&key).is_some_and(|previous| *previous != reference) {
            return Err(referential("conflicting stable static root"));
        }
        self.roots.insert(key, reference);
        Ok(())
    }

    /// Register result and embedded type references from reachable static values
    /// before the shared type graph closes.
    pub fn register_type_roots(&self, store: &StaticValueStore, types: &mut TypeExportContext, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        self.verify_store(store)?;
        fn visit(
            store: &StaticValueStore,
            value: StaticValueRef,
            key: &CanonicalTypeRootKey,
            ordinal: &mut u32,
            types: &mut TypeExportContext,
            active: &mut BTreeSet<StaticValueId>,
            depth: usize,
            limits: &ValidationLimits,
        ) -> Result<(), ArtifactError> {
            if depth > limits.max_recursion_depth || !active.insert(value.value_id()) {
                return Err(referential("cyclic or too deep static root"));
            }
            let data = store.get_value(value.value_id()).ok_or_else(|| referential("unknown static root"))?;
            let mut children = Vec::new();
            let ty = match data {
                StaticValueData::Type(ty) | StaticValueData::StaticTerm(ty) => Some(*ty),
                StaticValueData::Tuple(id) => {
                    children.extend(store.get_tuple(*id).map_err(store_error)?.iter().copied());
                    None
                }
                StaticValueData::Sequence(id) => {
                    let s = store.get_sequence(*id).map_err(store_error)?;
                    children.extend(s.elements().iter().copied());
                    Some(s.result_type())
                }
                StaticValueData::Record(id) => {
                    children.extend(store.get_record(*id).map_err(store_error)?.iter().map(|f| f.value));
                    None
                }
                StaticValueData::Data(id) => {
                    let d = store.get_data(*id).map_err(store_error)?;
                    children.extend(d.components.iter().map(|(_, v)| *v));
                    Some(d.result_type)
                }
                StaticValueData::Variant(id) => {
                    let v = store.get_variant(*id).map_err(store_error)?;
                    children.extend(v.fields.iter().map(|(_, v)| *v));
                    Some(v.result_type)
                }
                StaticValueData::Pack(id) => {
                    let (_, backing) = store.get_pack(*id).map_err(store_error)?;
                    children.extend(backing.positionals.iter().copied());
                    children.extend(backing.labeled_values.iter().copied());
                    None
                }
                StaticValueData::Unit
                | StaticValueData::Bool(_)
                | StaticValueData::Char(_)
                | StaticValueData::Int(_)
                | StaticValueData::Float(_)
                | StaticValueData::String(_)
                | StaticValueData::Symbol(_) => None,
            };
            if let Some(ty) = ty {
                let mut key = key.clone();
                key.family_tag = phalcom_compiler_artifact::semantic::type_store::CanonicalRootFamilyTag::StaticValue;
                key.product_subkey = *ordinal;
                *ordinal = ordinal.checked_add(1).ok_or_else(|| referential("static type root ordinal overflow"))?;
                types.register_type_root(types.expected_lineage(), key, ty)?;
            }
            for child in children {
                visit(store, child, key, ordinal, types, active, depth + 1, limits)?;
            }
            active.remove(&value.value_id());
            Ok(())
        }
        let mut ordinals = BTreeMap::new();
        for (key, &value) in &self.roots {
            let ordinal = ordinals.entry(key.owner.clone()).or_insert(0);
            visit(store, value, key, ordinal, types, &mut BTreeSet::new(), 0, limits)?;
        }
        Ok(())
    }

    pub fn close_graph(&mut self, store: &StaticValueStore, type_export: &TypeExportContext, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        self.verify_store(store)?;
        if self.closed {
            return Err(referential("static export graph is already closed"));
        }
        let roots: Vec<StaticValueId> = self.roots.values().map(|r| r.value_id()).collect();

        struct Walker<'a> {
            store: &'a StaticValueStore,
            type_export: &'a TypeExportContext,
            limits: &'a ValidationLimits,
            seen: BTreeSet<StaticValueId>,
            context: &'a mut StaticExportContext,
        }

        impl<'a> Walker<'a> {
            fn walk(&mut self, val_id: StaticValueId, depth: usize) -> Result<(), ArtifactError> {
                if depth > self.limits.max_recursion_depth {
                    return Err(referential("static value graph recursion limit exceeded"));
                }
                if self.context.value_ids.contains_key(&val_id) {
                    return Ok(());
                }
                if !self.seen.insert(val_id) {
                    return Err(referential("cycle detected in static value graph"));
                }
                let data = self
                    .store
                    .get_value(val_id)
                    .ok_or_else(|| referential(format!("unknown static value id {}", val_id.raw())))?;

                match data {
                    StaticValueData::Unit | StaticValueData::Bool(_) | StaticValueData::Char(_) | StaticValueData::Int(_) | StaticValueData::Float(_) => {}
                    StaticValueData::String(sid) => {
                        if !self.context.string_ids.contains_key(sid) {
                            let wid = ArtifactStaticStringId(self.context.string_order.len() as u32);
                            self.context.string_ids.insert(*sid, wid);
                            self.context.string_order.push(*sid);
                        }
                    }
                    StaticValueData::Symbol(sym) => {
                        if !self.context.symbol_ids.contains_key(sym) {
                            let wid = ArtifactStaticSymbolId(self.context.symbol_order.len() as u32);
                            self.context.symbol_ids.insert(*sym, wid);
                            self.context.symbol_order.push(*sym);
                        }
                    }
                    StaticValueData::Type(ty) | StaticValueData::StaticTerm(ty) => {
                        self.type_export.ty(*ty)?;
                    }
                    StaticValueData::Tuple(tid) => {
                        let tuple = self.store.get_tuple(*tid).map_err(store_error)?;
                        for child in tuple.iter() {
                            self.walk(child.value_id(), depth + 1)?;
                        }
                        if !self.context.tuple_ids.contains_key(tid) {
                            let wid = ArtifactStaticTupleId(self.context.tuple_order.len() as u32);
                            self.context.tuple_ids.insert(*tid, wid);
                            self.context.tuple_order.push(*tid);
                        }
                    }
                    StaticValueData::Sequence(sid) => {
                        let seq = self.store.get_sequence(*sid).map_err(store_error)?;
                        self.type_export.ty(seq.result_type())?;
                        for child in seq.elements().iter() {
                            self.walk(child.value_id(), depth + 1)?;
                        }
                        if !self.context.sequence_ids.contains_key(sid) {
                            let wid = ArtifactStaticSequenceId(self.context.sequence_order.len() as u32);
                            self.context.sequence_ids.insert(*sid, wid);
                            self.context.sequence_order.push(*sid);
                        }
                    }
                    StaticValueData::Record(rid) => {
                        let rec = self.store.get_record(*rid).map_err(store_error)?;
                        for field in rec.iter() {
                            self.walk(field.value.value_id(), depth + 1)?;
                        }
                        if !self.context.record_ids.contains_key(rid) {
                            let wid = ArtifactStaticRecordId(self.context.record_order.len() as u32);
                            self.context.record_ids.insert(*rid, wid);
                            self.context.record_order.push(*rid);
                        }
                    }
                    StaticValueData::Data(did) => {
                        let data_val = self.store.get_data(*did).map_err(store_error)?;
                        self.type_export.ty(data_val.result_type)?;
                        for (_, child) in data_val.components.iter() {
                            self.walk(child.value_id(), depth + 1)?;
                        }
                        if !self.context.data_ids.contains_key(did) {
                            let wid = ArtifactStaticDataId(self.context.data_order.len() as u32);
                            self.context.data_ids.insert(*did, wid);
                            self.context.data_order.push(*did);
                        }
                    }
                    StaticValueData::Variant(vid) => {
                        let var_val = self.store.get_variant(*vid).map_err(store_error)?;
                        self.type_export.ty(var_val.result_type)?;
                        for (_, child) in var_val.fields.iter() {
                            self.walk(child.value_id(), depth + 1)?;
                        }
                        if !self.context.variant_ids.contains_key(vid) {
                            let wid = ArtifactStaticVariantId(self.context.variant_order.len() as u32);
                            self.context.variant_ids.insert(*vid, wid);
                            self.context.variant_order.push(*vid);
                        }
                    }
                    StaticValueData::Pack(pid) => {
                        let (pack_data, backing) = self.store.get_pack(*pid).map_err(store_error)?;
                        for p in backing.positionals.iter() {
                            self.walk(p.value_id(), depth + 1)?;
                        }
                        for l in backing.labels.iter() {
                            if !self.context.symbol_ids.contains_key(l) {
                                let wid = ArtifactStaticSymbolId(self.context.symbol_order.len() as u32);
                                self.context.symbol_ids.insert(*l, wid);
                                self.context.symbol_order.push(*l);
                            }
                        }
                        for lv in backing.labeled_values.iter() {
                            self.walk(lv.value_id(), depth + 1)?;
                        }
                        if !self.context.pack_backing_ids.contains_key(&pack_data.backing) {
                            let wid = ArtifactStaticPackBackingId(self.context.pack_backing_order.len() as u32);
                            self.context.pack_backing_ids.insert(pack_data.backing, wid);
                            self.context.pack_backing_order.push(pack_data.backing);
                        }
                        if !self.context.pack_ids.contains_key(pid) {
                            let wid = ArtifactStaticPackId(self.context.pack_order.len() as u32);
                            self.context.pack_ids.insert(*pid, wid);
                            self.context.pack_order.push(*pid);
                        }
                    }
                }
                self.seen.remove(&val_id);
                let wid = ArtifactStaticValueId(self.context.value_order.len() as u32);
                self.context.value_ids.insert(val_id, wid);
                self.context.value_order.push(val_id);
                Ok(())
            }
        }

        let mut walker = Walker {
            store,
            type_export,
            limits,
            seen: BTreeSet::new(),
            context: self,
        };

        for root in roots {
            walker.walk(root, 0)?;
        }

        self.closed = true;
        Ok(())
    }

    pub fn value(&self, reference: StaticValueRef) -> Result<ArtifactStaticValueId, ArtifactError> {
        if reference.store_id() != self.store_id {
            return Err(referential("static value reference belongs to foreign store lineage"));
        }
        self.value_ids
            .get(&reference.value_id())
            .copied()
            .ok_or_else(|| referential(format!("unregistered static value reference {}", reference.value_id().raw())))
    }

    pub fn symbol(&self, symbol: StaticSymbolId) -> Result<ArtifactStaticSymbolId, ArtifactError> {
        self.symbol_ids
            .get(&symbol)
            .copied()
            .ok_or_else(|| referential(format!("unregistered static symbol reference {}", symbol.raw())))
    }

    pub fn project_graph(
        &self,
        store: &StaticValueStore,
        type_export: &TypeExportContext,
        limits: &ValidationLimits,
    ) -> Result<StaticValueGraphV1, ArtifactError> {
        self.verify_store(store)?;
        if !self.closed {
            return Err(referential("static export graph is not closed"));
        }

        let strings: Vec<String> = self
            .string_order
            .iter()
            .map(|sid| {
                store
                    .get_string(*sid)
                    .map(|s| s.to_string())
                    .ok_or_else(|| referential("unknown static string"))
            })
            .collect::<Result<_, _>>()?;

        let symbols: Vec<String> = self
            .symbol_order
            .iter()
            .map(|sym| {
                store
                    .get_symbol(*sym)
                    .map(|s| s.to_string())
                    .ok_or_else(|| referential("unknown static symbol"))
            })
            .collect::<Result<_, _>>()?;

        let tuples: Vec<Vec<ArtifactStaticValueId>> = self
            .tuple_order
            .iter()
            .map(|tid| {
                let tuple = store.get_tuple(*tid).map_err(store_error)?;
                tuple.iter().map(|&v| self.value(v)).collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<_, _>>()?;

        let sequences: Vec<StaticSequenceV1> = self
            .sequence_order
            .iter()
            .map(|sid| {
                let seq = store.get_sequence(*sid).map_err(store_error)?;
                let family = match seq.family() {
                    StaticSequenceFamily::Array => SequenceFamilyV1::Array,
                    StaticSequenceFamily::Vector => SequenceFamilyV1::Vector,
                };
                let result_type = type_export.ty(seq.result_type())?;
                let elements = seq.elements().iter().map(|&v| self.value(v)).collect::<Result<Vec<_>, _>>()?;
                Ok(StaticSequenceV1 { family, result_type, elements })
            })
            .collect::<Result<Vec<StaticSequenceV1>, ArtifactError>>()?;

        let records: Vec<Vec<StaticRecordFieldV1>> = self
            .record_order
            .iter()
            .map(|rid| {
                let rec = store.get_record(*rid).map_err(store_error)?;
                rec.iter()
                    .map(|f| {
                        Ok(StaticRecordFieldV1 {
                            key: key_to_wire(&f.key),
                            value: self.value(f.value)?,
                        })
                    })
                    .collect::<Result<Vec<_>, ArtifactError>>()
            })
            .collect::<Result<Vec<_>, ArtifactError>>()?;

        let data_values: Vec<StaticDataValueV1> = self
            .data_order
            .iter()
            .map(|did| {
                let d = store.get_data(*did).map_err(store_error)?;
                let owner = to_stable_declaration(&d.owner);
                let constructor_owner = to_stable_declaration(&d.constructor.owner);
                let result_type = type_export.ty(d.result_type)?;
                let components = d
                    .components
                    .iter()
                    .map(|(cid, v)| {
                        Ok(StaticDataComponentV1 {
                            index: cid.index,
                            value: self.value(*v)?,
                        })
                    })
                    .collect::<Result<Vec<_>, ArtifactError>>()?;
                Ok(StaticDataValueV1 {
                    owner,
                    constructor_owner,
                    result_type,
                    components,
                })
            })
            .collect::<Result<Vec<StaticDataValueV1>, ArtifactError>>()?;

        let variant_values: Vec<StaticVariantValueV1> = self
            .variant_order
            .iter()
            .map(|vid| {
                let v = store.get_variant(*vid).map_err(store_error)?;
                let variant = to_stable_variant(&v.variant);
                let result_type = type_export.ty(v.result_type)?;
                let fields = v
                    .fields
                    .iter()
                    .map(|(fid, val)| {
                        Ok(StaticVariantFieldV1 {
                            index: fid.index,
                            value: self.value(*val)?,
                        })
                    })
                    .collect::<Result<Vec<_>, ArtifactError>>()?;
                Ok(StaticVariantValueV1 { variant, result_type, fields })
            })
            .collect::<Result<Vec<StaticVariantValueV1>, ArtifactError>>()?;

        let pack_backings: Vec<StaticPackBackingV1> = self
            .pack_backing_order
            .iter()
            .map(|bid| {
                let b = store.get_pack_backing(*bid).map_err(store_error)?;
                let positionals = b.positionals.iter().map(|&v| self.value(v)).collect::<Result<Vec<_>, _>>()?;
                let labels = b.labels.iter().map(|&s| self.symbol(s)).collect::<Result<Vec<_>, _>>()?;
                let labeled_values = b.labeled_values.iter().map(|&v| self.value(v)).collect::<Result<Vec<_>, _>>()?;
                Ok(StaticPackBackingV1 {
                    positionals,
                    labels,
                    labeled_values,
                })
            })
            .collect::<Result<Vec<StaticPackBackingV1>, ArtifactError>>()?;

        let packs: Vec<StaticPackSliceV1> = self
            .pack_order
            .iter()
            .map(|pid| {
                let p = store.get_pack_data(*pid).map_err(store_error)?;
                let backing = *self.pack_backing_ids.get(&p.backing).ok_or_else(|| referential("unrelocated pack backing"))?;
                Ok(StaticPackSliceV1 {
                    backing,
                    positional_start: p.positional_start,
                    positional_len: p.positional_len,
                    labeled_start: p.labeled_start,
                    labeled_len: p.labeled_len,
                })
            })
            .collect::<Result<Vec<StaticPackSliceV1>, ArtifactError>>()?;

        let values: Vec<StaticValueV1> = self
            .value_order
            .iter()
            .map(|vid| {
                let data = store.get_value(*vid).ok_or_else(|| referential("unknown static value"))?;
                Ok(match data {
                    StaticValueData::Unit => StaticValueV1::Unit,
                    StaticValueData::Bool(b) => StaticValueV1::Bool { value: *b },
                    StaticValueData::Char(c) => StaticValueV1::Char { scalar: *c as u32 },
                    StaticValueData::Int(i) => StaticValueV1::Int { decimal: i.to_string() },
                    StaticValueData::Float(f) => StaticValueV1::Float { bits: f.bits() },
                    StaticValueData::String(sid) => StaticValueV1::String {
                        id: *self.string_ids.get(sid).ok_or_else(|| referential("missing string id"))?,
                    },
                    StaticValueData::Symbol(sym) => StaticValueV1::Symbol {
                        id: *self.symbol_ids.get(sym).ok_or_else(|| referential("missing symbol id"))?,
                    },
                    StaticValueData::Type(ty) => StaticValueV1::Type { ty: type_export.ty(*ty)? },
                    StaticValueData::StaticTerm(ty) => StaticValueV1::StaticTerm { ty: type_export.ty(*ty)? },
                    StaticValueData::Tuple(tid) => StaticValueV1::Tuple {
                        id: *self.tuple_ids.get(tid).ok_or_else(|| referential("missing tuple id"))?,
                    },
                    StaticValueData::Sequence(sid) => StaticValueV1::Sequence {
                        id: *self.sequence_ids.get(sid).ok_or_else(|| referential("missing sequence id"))?,
                    },
                    StaticValueData::Record(rid) => StaticValueV1::Record {
                        id: *self.record_ids.get(rid).ok_or_else(|| referential("missing record id"))?,
                    },
                    StaticValueData::Data(did) => StaticValueV1::Data {
                        id: *self.data_ids.get(did).ok_or_else(|| referential("missing data id"))?,
                    },
                    StaticValueData::Variant(vid) => StaticValueV1::Variant {
                        id: *self.variant_ids.get(vid).ok_or_else(|| referential("missing variant id"))?,
                    },
                    StaticValueData::Pack(pid) => StaticValueV1::Pack {
                        id: *self.pack_ids.get(pid).ok_or_else(|| referential("missing pack id"))?,
                    },
                })
            })
            .collect::<Result<_, ArtifactError>>()?;

        let graph = StaticValueGraphV1 {
            values,
            strings,
            symbols,
            tuples,
            sequences,
            records,
            data_values,
            variant_values,
            pack_backings,
            packs,
        };
        graph.validate(limits)?;
        Ok(graph)
    }
}

pub fn key_to_wire(key: &FieldKey) -> FieldKeyV1 {
    match key {
        FieldKey::Symbol(s) => FieldKeyV1::Symbol { value: s.to_string() },
        FieldKey::String(s) => FieldKeyV1::String { value: s.to_string() },
        FieldKey::Int(i) => FieldKeyV1::Int { decimal: i.to_string() },
        FieldKey::Bool(b) => FieldKeyV1::Bool { value: *b },
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
pub type BudgetedDeclarationResolver<'a> = &'a dyn Fn(&StableDeclarationRef, &mut AllocationBudget) -> Result<DeclarationId, ArtifactError>;
pub type BudgetedVariantResolver<'a> = &'a dyn Fn(&StableVariantRef, &mut AllocationBudget) -> Result<VariantId, ArtifactError>;

/// Reconstructs a validated static-value graph into a fresh store lineage with allocation budgeting.
pub fn import_static_value_graph_with_allocation_budget(
    graph: &StaticValueGraphV1,
    _types: &crate::types::store::TypeStore,
    type_relocation: &TypeRelocation,
    data: &DataSemanticTable,
    enums: &EnumSemanticTable,
    resolve_declaration: BudgetedDeclarationResolver<'_>,
    resolve_variant: BudgetedVariantResolver<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<(StaticValueStore, StaticRelocation), ArtifactError> {
    let _order = graph.validate_with_allocation_budget(limits, allocations)?;
    let store_id = StaticValueStore::fresh_id();

    // 1. Strings
    let mut strings_buf = allocations.vector(graph.strings.len()).map_err(ArtifactError::Allocation)?;
    for s in &graph.strings {
        strings_buf.push(allocations.boxed_str(s).map_err(ArtifactError::Allocation)?);
    }
    let strings = allocations.boxed_slice(strings_buf).map_err(ArtifactError::Allocation)?;

    let mut string_to_id_vec = allocations.vector(graph.strings.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.strings.len() {
        string_to_id_vec.push(StaticStringId(i as u32));
    }
    string_to_id_vec.sort_unstable_by(|&a, &b| strings[a.0 as usize].cmp(&strings[b.0 as usize]));
    if string_to_id_vec
        .windows(2)
        .any(|pair| strings[pair[0].0 as usize] == strings[pair[1].0 as usize])
    {
        return Err(referential("duplicate canonical static string node"));
    }
    let string_to_id = allocations.boxed_slice(string_to_id_vec).map_err(ArtifactError::Allocation)?;

    // 2. Symbols
    let mut symbols_buf = allocations.vector(graph.symbols.len()).map_err(ArtifactError::Allocation)?;
    for s in &graph.symbols {
        symbols_buf.push(allocations.boxed_str(s).map_err(ArtifactError::Allocation)?);
    }
    let symbols = allocations.boxed_slice(symbols_buf).map_err(ArtifactError::Allocation)?;

    let mut symbol_to_id_vec = allocations.vector(graph.symbols.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.symbols.len() {
        symbol_to_id_vec.push(StaticSymbolId(i as u32));
    }
    symbol_to_id_vec.sort_unstable_by(|&a, &b| symbols[a.0 as usize].cmp(&symbols[b.0 as usize]));
    if symbol_to_id_vec
        .windows(2)
        .any(|pair| symbols[pair[0].0 as usize] == symbols[pair[1].0 as usize])
    {
        return Err(referential("duplicate canonical static symbol node"));
    }
    let symbol_to_id = allocations.boxed_slice(symbol_to_id_vec).map_err(ArtifactError::Allocation)?;

    let mut relocation_symbols_vec = allocations.vector(graph.symbols.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.symbols.len() {
        relocation_symbols_vec.push(StaticSymbolId(i as u32));
    }
    let relocation_symbols = allocations.boxed_slice(relocation_symbols_vec).map_err(ArtifactError::Allocation)?;

    // 3. Relocation values (dense pre-allocated)
    let mut relocation_values_vec = allocations.vector(graph.values.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.values.len() {
        relocation_values_vec.push(StaticValueRef {
            store: store_id,
            value: StaticValueId(i as u32),
        });
    }
    let relocation_values = allocations.boxed_slice(relocation_values_vec).map_err(ArtifactError::Allocation)?;
    let relocation = StaticRelocation {
        store_id,
        values: relocation_values,
        symbols: relocation_symbols,
    };

    // 4. Tuples
    let mut tuples_buf = allocations.vector(graph.tuples.len()).map_err(ArtifactError::Allocation)?;
    for tuple in &graph.tuples {
        let mut elements_vec = allocations.vector(tuple.len()).map_err(ArtifactError::Allocation)?;
        for &child in tuple {
            elements_vec.push(relocation.value(child)?);
        }
        let elements = allocations.boxed_slice(elements_vec).map_err(ArtifactError::Allocation)?;
        tuples_buf.push(elements);
    }
    let tuples = allocations.boxed_slice(tuples_buf).map_err(ArtifactError::Allocation)?;

    let mut tuple_to_id_vec = allocations.vector(graph.tuples.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.tuples.len() {
        tuple_to_id_vec.push(StaticTupleId(i as u32));
    }
    tuple_to_id_vec.sort_unstable_by(|&a, &b| tuples[a.0 as usize].cmp(&tuples[b.0 as usize]));
    if tuple_to_id_vec.windows(2).any(|pair| tuples[pair[0].0 as usize] == tuples[pair[1].0 as usize]) {
        return Err(referential("duplicate canonical static tuple node"));
    }
    let tuple_to_id = allocations.boxed_slice(tuple_to_id_vec).map_err(ArtifactError::Allocation)?;

    // 5. Sequences
    let mut sequences_buf = allocations.vector(graph.sequences.len()).map_err(ArtifactError::Allocation)?;
    for seq in &graph.sequences {
        let result_type = type_relocation.ty(seq.result_type)?;
        let mut elements_vec = allocations.vector(seq.elements.len()).map_err(ArtifactError::Allocation)?;
        for &child in &seq.elements {
            elements_vec.push(relocation.value(child)?);
        }
        let family = match seq.family {
            SequenceFamilyV1::Array => StaticSequenceFamily::Array,
            SequenceFamilyV1::Vector => StaticSequenceFamily::Vector,
        };
        let elements = allocations.boxed_slice(elements_vec).map_err(ArtifactError::Allocation)?;
        sequences_buf.push(StaticSequenceData { family, result_type, elements });
    }
    let sequences = allocations.boxed_slice(sequences_buf).map_err(ArtifactError::Allocation)?;

    let mut sequence_to_id_vec = allocations.vector(graph.sequences.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.sequences.len() {
        sequence_to_id_vec.push(StaticSequenceId(i as u32));
    }
    sequence_to_id_vec.sort_unstable_by(|&a, &b| sequences[a.0 as usize].cmp(&sequences[b.0 as usize]));
    if sequence_to_id_vec
        .windows(2)
        .any(|pair| sequences[pair[0].0 as usize] == sequences[pair[1].0 as usize])
    {
        return Err(referential("duplicate canonical static sequence node"));
    }
    let sequence_to_id = allocations.boxed_slice(sequence_to_id_vec).map_err(ArtifactError::Allocation)?;

    // 6. Records
    let mut records_buf = allocations.vector(graph.records.len()).map_err(ArtifactError::Allocation)?;
    for record in &graph.records {
        let mut fields_vec = allocations.vector(record.len()).map_err(ArtifactError::Allocation)?;
        for field in record {
            let key = key_from_wire_budgeted(&field.key, limits, allocations)?;
            let value = relocation.value(field.value)?;
            fields_vec.push(StaticRecordField { key, value });
        }
        let fields = allocations.boxed_slice(fields_vec).map_err(ArtifactError::Allocation)?;
        records_buf.push(fields);
    }
    let records = allocations.boxed_slice(records_buf).map_err(ArtifactError::Allocation)?;

    let mut record_to_id_vec = allocations.vector(graph.records.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.records.len() {
        record_to_id_vec.push(StaticRecordId(i as u32));
    }
    record_to_id_vec.sort_unstable_by(|&a, &b| records[a.0 as usize].cmp(&records[b.0 as usize]));
    if record_to_id_vec
        .windows(2)
        .any(|pair| records[pair[0].0 as usize] == records[pair[1].0 as usize])
    {
        return Err(referential("duplicate canonical static record node"));
    }
    let record_to_id = allocations.boxed_slice(record_to_id_vec).map_err(ArtifactError::Allocation)?;

    // 7. Data values
    let mut data_values_buf = allocations.vector(graph.data_values.len()).map_err(ArtifactError::Allocation)?;
    for value in &graph.data_values {
        let owner = resolve_declaration(&value.owner, allocations)?;
        let info = data.data_info(&owner).ok_or_else(|| referential("static data owner has no loaded contract"))?;
        if value.components.len() != info.components.len() {
            return Err(referential("static data component count disagrees with loaded contract"));
        }
        let result_type = type_relocation.ty(value.result_type)?;
        let mut components_vec = allocations.vector(info.components.len()).map_err(ArtifactError::Allocation)?;
        for (component, wire) in info.components.iter().zip(value.components.iter()) {
            let child_ref = relocation.value(wire.value)?;
            components_vec.push((
                component.id.clone_with_allocation_budget(allocations).map_err(ArtifactError::Allocation)?,
                child_ref,
            ));
        }
        let components = allocations.boxed_slice(components_vec).map_err(ArtifactError::Allocation)?;
        let owner_cloned = owner.clone_with_allocation_budget(allocations).map_err(ArtifactError::Allocation)?;
        let constructor_cloned = info
            .constructor
            .constructor
            .clone_with_allocation_budget(allocations)
            .map_err(ArtifactError::Allocation)?;
        data_values_buf.push(StaticDataValue {
            owner: owner_cloned,
            constructor: constructor_cloned,
            result_type,
            components,
        });
    }
    let data_values = allocations.boxed_slice(data_values_buf).map_err(ArtifactError::Allocation)?;

    let mut data_value_to_id_vec = allocations.vector(graph.data_values.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.data_values.len() {
        data_value_to_id_vec.push(StaticDataValueId(i as u32));
    }
    data_value_to_id_vec.sort_unstable_by(|&a, &b| data_values[a.0 as usize].cmp(&data_values[b.0 as usize]));
    if data_value_to_id_vec
        .windows(2)
        .any(|pair| data_values[pair[0].0 as usize] == data_values[pair[1].0 as usize])
    {
        return Err(referential("duplicate canonical static data node"));
    }
    let data_value_to_id = allocations.boxed_slice(data_value_to_id_vec).map_err(ArtifactError::Allocation)?;

    // 8. Variant values
    let mut variant_values_buf = allocations.vector(graph.variant_values.len()).map_err(ArtifactError::Allocation)?;
    for value in &graph.variant_values {
        let variant = resolve_variant(&value.variant, allocations)?;
        let info = enums
            .variant_info(&variant)
            .ok_or_else(|| referential("static variant has no loaded contract"))?;
        if value.fields.len() != info.fields.len() {
            return Err(referential("static variant field count disagrees with loaded contract"));
        }
        let result_type = type_relocation.ty(value.result_type)?;
        let mut fields_vec = allocations.vector(info.fields.len()).map_err(ArtifactError::Allocation)?;
        for (field, wire) in info.fields.iter().zip(value.fields.iter()) {
            let child_ref = relocation.value(wire.value)?;
            fields_vec.push((
                field.id.clone_with_allocation_budget(allocations).map_err(ArtifactError::Allocation)?,
                child_ref,
            ));
        }
        let fields = allocations.boxed_slice(fields_vec).map_err(ArtifactError::Allocation)?;
        variant_values_buf.push(StaticVariantValue { variant, result_type, fields });
    }
    let variant_values = allocations.boxed_slice(variant_values_buf).map_err(ArtifactError::Allocation)?;

    let mut variant_value_to_id_vec = allocations.vector(graph.variant_values.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.variant_values.len() {
        variant_value_to_id_vec.push(StaticVariantValueId(i as u32));
    }
    variant_value_to_id_vec.sort_unstable_by(|&a, &b| variant_values[a.0 as usize].cmp(&variant_values[b.0 as usize]));
    if variant_value_to_id_vec
        .windows(2)
        .any(|pair| variant_values[pair[0].0 as usize] == variant_values[pair[1].0 as usize])
    {
        return Err(referential("duplicate canonical static variant node"));
    }
    let variant_value_to_id = allocations.boxed_slice(variant_value_to_id_vec).map_err(ArtifactError::Allocation)?;

    // 9. Pack backings
    let mut pack_backings_buf = allocations.vector(graph.pack_backings.len()).map_err(ArtifactError::Allocation)?;
    for backing in &graph.pack_backings {
        let mut pos_vec = allocations.vector(backing.positionals.len()).map_err(ArtifactError::Allocation)?;
        for &p in &backing.positionals {
            pos_vec.push(relocation.value(p)?);
        }
        let positionals = allocations.boxed_slice(pos_vec).map_err(ArtifactError::Allocation)?;

        let mut lab_sym_vec = allocations.vector(backing.labels.len()).map_err(ArtifactError::Allocation)?;
        for &l in &backing.labels {
            lab_sym_vec.push(relocation.symbol(l)?);
        }
        let labels = allocations.boxed_slice(lab_sym_vec).map_err(ArtifactError::Allocation)?;

        let mut lab_val_vec = allocations.vector(backing.labeled_values.len()).map_err(ArtifactError::Allocation)?;
        for &v in &backing.labeled_values {
            lab_val_vec.push(relocation.value(v)?);
        }
        let labeled_values = allocations.boxed_slice(lab_val_vec).map_err(ArtifactError::Allocation)?;

        pack_backings_buf.push(StaticPackBacking {
            positionals,
            labels,
            labeled_values,
        });
    }
    let pack_backings = allocations.boxed_slice(pack_backings_buf).map_err(ArtifactError::Allocation)?;

    let mut pack_backing_to_id_vec = allocations.vector(graph.pack_backings.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.pack_backings.len() {
        pack_backing_to_id_vec.push(StaticPackBackingId(i as u32));
    }
    pack_backing_to_id_vec.sort_unstable_by(|&a, &b| pack_backings[a.0 as usize].cmp(&pack_backings[b.0 as usize]));
    if pack_backing_to_id_vec
        .windows(2)
        .any(|pair| pack_backings[pair[0].0 as usize] == pack_backings[pair[1].0 as usize])
    {
        return Err(referential("duplicate canonical static pack backing node"));
    }
    let pack_backing_to_id = allocations.boxed_slice(pack_backing_to_id_vec).map_err(ArtifactError::Allocation)?;

    // 10. Packs
    let mut packs_buf = allocations.vector(graph.packs.len()).map_err(ArtifactError::Allocation)?;
    for pack in &graph.packs {
        packs_buf.push(StaticPackData {
            backing: StaticPackBackingId(pack.backing.0),
            positional_start: pack.positional_start,
            positional_len: pack.positional_len,
            labeled_start: pack.labeled_start,
            labeled_len: pack.labeled_len,
        });
    }
    let packs = allocations.boxed_slice(packs_buf).map_err(ArtifactError::Allocation)?;

    let mut pack_to_id_vec = allocations.vector(graph.packs.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.packs.len() {
        pack_to_id_vec.push(StaticPackId(i as u32));
    }
    pack_to_id_vec.sort_unstable_by(|&a, &b| packs[a.0 as usize].cmp(&packs[b.0 as usize]));
    if pack_to_id_vec.windows(2).any(|pair| packs[pair[0].0 as usize] == packs[pair[1].0 as usize]) {
        return Err(referential("duplicate canonical static pack node"));
    }
    let pack_to_id = allocations.boxed_slice(pack_to_id_vec).map_err(ArtifactError::Allocation)?;

    // 11. Values
    let mut values_buf = allocations.vector(graph.values.len()).map_err(ArtifactError::Allocation)?;
    for val in &graph.values {
        let data = match val {
            StaticValueV1::Unit => StaticValueData::Unit,
            StaticValueV1::Bool { value } => StaticValueData::Bool(*value),
            StaticValueV1::Char { scalar } => {
                let scalar = char::from_u32(*scalar).ok_or_else(|| referential("invalid static character scalar"))?;
                StaticValueData::Char(scalar)
            }
            StaticValueV1::Int { decimal } => {
                let value = crate::types::store::artifact::integer_in_with_allocation_budget(decimal, limits, allocations)?;
                StaticValueData::Int(value)
            }
            StaticValueV1::Float { bits } => StaticValueData::Float(StaticFloat::from_bits(*bits)),
            StaticValueV1::String { id } => StaticValueData::String(StaticStringId(id.0)),
            StaticValueV1::Symbol { id } => StaticValueData::Symbol(StaticSymbolId(id.0)),
            StaticValueV1::Type { ty } => StaticValueData::Type(type_relocation.ty(*ty)?),
            StaticValueV1::StaticTerm { ty } => StaticValueData::StaticTerm(type_relocation.ty(*ty)?),
            StaticValueV1::Tuple { id } => StaticValueData::Tuple(StaticTupleId(id.0)),
            StaticValueV1::Sequence { id } => StaticValueData::Sequence(StaticSequenceId(id.0)),
            StaticValueV1::Record { id } => StaticValueData::Record(StaticRecordId(id.0)),
            StaticValueV1::Data { id } => StaticValueData::Data(StaticDataValueId(id.0)),
            StaticValueV1::Variant { id } => StaticValueData::Variant(StaticVariantValueId(id.0)),
            StaticValueV1::Pack { id } => StaticValueData::Pack(StaticPackId(id.0)),
        };
        values_buf.push(data);
    }
    let values = allocations.boxed_slice(values_buf).map_err(ArtifactError::Allocation)?;

    let mut value_to_id_vec = allocations.vector(graph.values.len()).map_err(ArtifactError::Allocation)?;
    for i in 0..graph.values.len() {
        value_to_id_vec.push(StaticValueId(i as u32));
    }
    value_to_id_vec.sort_unstable_by(|&a, &b| values[a.0 as usize].cmp(&values[b.0 as usize]));
    if value_to_id_vec.windows(2).any(|pair| values[pair[0].0 as usize] == values[pair[1].0 as usize]) {
        return Err(referential("duplicate canonical static value node"));
    }
    let value_to_id = allocations.boxed_slice(value_to_id_vec).map_err(ArtifactError::Allocation)?;

    // 12. Create FrozenStaticValuePrefix and StaticValueStore
    let prefix = FrozenStaticValuePrefix {
        id: store_id,
        values,
        value_to_id,
        strings,
        string_to_id,
        symbols,
        symbol_to_id,
        tuples,
        tuple_to_id,
        sequences,
        sequence_to_id,
        records,
        record_to_id,
        data_values,
        data_value_to_id,
        variant_values,
        variant_value_to_id,
        pack_backings,
        pack_backing_to_id,
        packs,
        pack_to_id,
    };
    let frozen_arc = allocations.arc(prefix).map_err(ArtifactError::Allocation)?;
    let store = StaticValueStore::from_frozen(frozen_arc);
    Ok((store, relocation))
}

/// Relocates one validated static-value graph into an existing receiving
/// store lineage. Reconstruction uses the same checked factories as ordinary
/// semantic evaluation and commits only after the entire artifact graph passes.
pub fn import_static_value_graph_into_store_with_allocation_budget(
    receiver: &mut StaticValueStore,
    graph: &StaticValueGraphV1,
    types: &crate::types::store::TypeStore,
    type_relocation: &TypeRelocation,
    data: &DataSemanticTable,
    enums: &EnumSemanticTable,
    resolve_declaration: BudgetedDeclarationResolver<'_>,
    resolve_variant: BudgetedVariantResolver<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<StaticRelocation, ArtifactError> {
    type_relocation
        .verify_store(types)
        .map_err(|error| referential(error.to_string()))?;
    let order = graph.validate_with_allocation_budget(limits, allocations)?;
    let mut candidate = receiver.clone();
    let store_id = candidate.id();

    let mut symbols = allocations
        .vector::<StaticSymbolId>(graph.symbols.len())
        .map_err(ArtifactError::Allocation)?;
    for spelling in &graph.symbols {
        symbols.push(candidate.intern_symbol_id(spelling));
    }

    let mut values = allocations
        .vector::<Option<StaticValueRef>>(graph.values.len())
        .map_err(ArtifactError::Allocation)?;
    values.resize(graph.values.len(), None);

    let relocated = |values: &[Option<StaticValueRef>], id: ArtifactStaticValueId| -> Result<StaticValueRef, ArtifactError> {
        values
            .get(id.0 as usize)
            .and_then(|value| *value)
            .ok_or_else(|| referential("static dependency was not reconstructed before its consumer"))
    };

    for artifact_id in order {
        let value = match &graph.values[artifact_id.0 as usize] {
            StaticValueV1::Unit => candidate.intern_unit(),
            StaticValueV1::Bool { value } => candidate.intern_bool(*value),
            StaticValueV1::Char { scalar } => {
                let value = char::from_u32(*scalar).ok_or_else(|| referential("invalid static character scalar"))?;
                candidate.intern_char(value)
            }
            StaticValueV1::Int { decimal } => {
                let value = crate::types::store::artifact::integer_in_with_allocation_budget(decimal, limits, allocations)?;
                candidate.intern_int(value.to_bigint())
            }
            StaticValueV1::Float { bits } => candidate.intern_float(StaticFloat::from_bits(*bits)),
            StaticValueV1::String { id } => {
                let spelling = graph
                    .strings
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("static string identity is out of range"))?;
                candidate.intern_string(spelling)
            }
            StaticValueV1::Symbol { id } => {
                let spelling = graph
                    .symbols
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("static symbol identity is out of range"))?;
                candidate.intern_symbol(spelling)
            }
            StaticValueV1::Type { ty } => candidate
                .intern_type(type_relocation.ty(*ty)?, types)
                .map_err(store_error)?,
            StaticValueV1::StaticTerm { ty } => candidate
                .intern_static_term(type_relocation.ty(*ty)?, types)
                .map_err(store_error)?,
            StaticValueV1::Tuple { id } => {
                let wire = graph
                    .tuples
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("static tuple identity is out of range"))?;
                let mut children = Vec::with_capacity(wire.len());
                for child in wire {
                    children.push(relocated(&values, *child)?);
                }
                candidate.intern_tuple(children).map_err(store_error)?
            }
            StaticValueV1::Sequence { id } => {
                let wire = graph
                    .sequences
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("static sequence identity is out of range"))?;
                let mut children = Vec::with_capacity(wire.elements.len());
                for child in &wire.elements {
                    children.push(relocated(&values, *child)?);
                }
                let value = candidate
                    .intern_sequence(type_relocation.ty(wire.result_type)?, children, types)
                    .map_err(store_error)?;
                let StaticValueData::Sequence(sequence_id) = candidate.get(value).map_err(store_error)? else {
                    return Err(referential("sequence factory did not return a sequence value"));
                };
                let sequence = candidate.get_sequence(*sequence_id).map_err(store_error)?;
                let expected = match wire.family {
                    SequenceFamilyV1::Array => StaticSequenceFamily::Array,
                    SequenceFamilyV1::Vector => StaticSequenceFamily::Vector,
                };
                if sequence.family != expected {
                    return Err(referential("static sequence family disagrees with relocated result type"));
                }
                value
            }
            StaticValueV1::Record { id } => {
                let wire = graph
                    .records
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("static record identity is out of range"))?;
                let mut fields = Vec::with_capacity(wire.len());
                for field in wire {
                    fields.push((
                        key_from_wire_budgeted(&field.key, limits, allocations)?,
                        relocated(&values, field.value)?,
                    ));
                }
                candidate.intern_record(fields).map_err(store_error)?
            }
            StaticValueV1::Data { id } => {
                let wire = graph
                    .data_values
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("static data identity is out of range"))?;
                let owner = resolve_declaration(&wire.owner, allocations)?;
                let info = data
                    .data_info(&owner)
                    .ok_or_else(|| referential("static data owner has no loaded contract"))?;
                if wire.components.len() != info.components.len() {
                    return Err(referential("static data component count disagrees with loaded contract"));
                }
                let mut components = Vec::with_capacity(info.components.len());
                for (component, wire_component) in info.components.iter().zip(wire.components.iter()) {
                    components.push((component.id.clone(), relocated(&values, wire_component.value)?));
                }
                candidate
                    .intern_data(info, type_relocation.ty(wire.result_type)?, components, types)
                    .map_err(store_error)?
            }
            StaticValueV1::Variant { id } => {
                let wire = graph
                    .variant_values
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("static variant identity is out of range"))?;
                let variant = resolve_variant(&wire.variant, allocations)?;
                let info = enums
                    .variant_info(&variant)
                    .ok_or_else(|| referential("static variant has no loaded contract"))?;
                if wire.fields.len() != info.fields.len() {
                    return Err(referential("static variant field count disagrees with loaded contract"));
                }
                let mut fields = Vec::with_capacity(info.fields.len());
                for (field, wire_field) in info.fields.iter().zip(wire.fields.iter()) {
                    fields.push((field.id.clone(), relocated(&values, wire_field.value)?));
                }
                candidate
                    .intern_variant(info, type_relocation.ty(wire.result_type)?, fields, types)
                    .map_err(store_error)?
            }
            StaticValueV1::Pack { id } => {
                let wire = graph
                    .packs
                    .get(id.0 as usize)
                    .ok_or_else(|| referential("static pack identity is out of range"))?;
                let backing = graph
                    .pack_backings
                    .get(wire.backing.0 as usize)
                    .ok_or_else(|| referential("static pack backing identity is out of range"))?;
                let mut positionals = Vec::with_capacity(backing.positionals.len());
                for child in &backing.positionals {
                    positionals.push(relocated(&values, *child)?);
                }
                let mut labels = Vec::with_capacity(backing.labels.len());
                for label in &backing.labels {
                    labels.push(
                        *symbols
                            .get(label.0 as usize)
                            .ok_or_else(|| referential("static pack label identity is out of range"))?,
                    );
                }
                let mut labeled_values = Vec::with_capacity(backing.labeled_values.len());
                for child in &backing.labeled_values {
                    labeled_values.push(relocated(&values, *child)?);
                }
                let backing_id = candidate
                    .intern_pack_backing_reconstructed(StaticPackBacking {
                        positionals: positionals.into_boxed_slice(),
                        labels: labels.into_boxed_slice(),
                        labeled_values: labeled_values.into_boxed_slice(),
                    })
                    .map_err(store_error)?;
                candidate
                    .intern_pack_slice_reconstructed(StaticPackData {
                        backing: backing_id,
                        positional_start: wire.positional_start,
                        positional_len: wire.positional_len,
                        labeled_start: wire.labeled_start,
                        labeled_len: wire.labeled_len,
                    })
                    .map_err(store_error)?
            }
        };
        values[artifact_id.0 as usize] = Some(value);
    }

    let mut relocated_values = allocations
        .vector::<StaticValueRef>(values.len())
        .map_err(ArtifactError::Allocation)?;
    for value in values {
        relocated_values.push(value.ok_or_else(|| referential("incomplete shared static-value relocation"))?);
    }

    let relocation = StaticRelocation {
        store_id,
        values: allocations.boxed_slice(relocated_values).map_err(ArtifactError::Allocation)?,
        symbols: allocations.boxed_slice(symbols).map_err(ArtifactError::Allocation)?,
    };
    *receiver = candidate;
    Ok(relocation)
}

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
    let mut allocations = AllocationBudget::new(limits.max_decoded_section_bytes as usize);
    import_static_value_graph_with_allocation_budget(
        graph,
        types,
        type_relocation,
        data,
        enums,
        &|decl, _| resolve_declaration(decl),
        &|var, _| resolve_variant(var),
        limits,
        &mut allocations,
    )
}

fn key_from_wire_budgeted(key: &FieldKeyV1, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<FieldKey, ArtifactError> {
    Ok(match key {
        FieldKeyV1::Symbol { value } => FieldKey::Symbol(allocations.arc_str(value).map_err(ArtifactError::Allocation)?),
        FieldKeyV1::String { value } => FieldKey::String(allocations.arc_str(value).map_err(ArtifactError::Allocation)?),
        FieldKeyV1::Int { decimal } => {
            let value = crate::types::store::artifact::integer_in_with_allocation_budget(decimal, limits, allocations)?;
            FieldKey::int(value)
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
