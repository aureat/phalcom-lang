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
use crate::scalar::StaticInt;
use crate::static_value::store::{
    StaticDataValueId, StaticFloat, StaticPackBackingId, StaticPackId, StaticRecordId, StaticSequenceFamily, StaticSequenceId,
    StaticStringId, StaticSymbolId, StaticTupleId, StaticValueData, StaticValueId, StaticValueRef, StaticValueStore,
    StaticValueStoreError, StaticValueStoreId, StaticVariantValueId,
};
use crate::types::store::artifact::{TypeExportContext, TypeRelocation};
use num_bigint::BigInt;
use phalcom_compiler_artifact::semantic::static_value::*;
use phalcom_compiler_artifact::semantic::type_store::{FieldKeyV1, CanonicalTypeRootKey};
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::identity::{StableDeclarationRef, StableVariantRef};
use std::collections::{BTreeMap, BTreeSet};

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
        fn visit(store: &StaticValueStore, value: StaticValueRef, key: &CanonicalTypeRootKey, ordinal: &mut u32,
            types: &mut TypeExportContext, active: &mut BTreeSet<StaticValueId>, depth: usize, limits: &ValidationLimits) -> Result<(), ArtifactError> {
            if depth > limits.max_recursion_depth || !active.insert(value.value_id()) { return Err(referential("cyclic or too deep static root")); }
            let data=store.values_arena().get(value.value_id().raw() as usize).ok_or_else(|| referential("unknown static root"))?;
            let mut children=Vec::new();
            let ty=match data {
                StaticValueData::Type(ty)|StaticValueData::StaticTerm(ty)=>Some(*ty),
                StaticValueData::Tuple(id)=>{children.extend(store.tuples_arena().get(id.raw() as usize).ok_or_else(||referential("unknown tuple"))?.iter().copied());None},
                StaticValueData::Sequence(id)=>{let s=store.sequences_arena().get(id.raw() as usize).ok_or_else(||referential("unknown sequence"))?;children.extend(s.elements().iter().copied());Some(s.result_type())},
                StaticValueData::Record(id)=>{children.extend(store.records_arena().get(id.raw() as usize).ok_or_else(||referential("unknown record"))?.iter().map(|f|f.value));None},
                StaticValueData::Data(id)=>{let d=store.data_values_arena().get(id.raw() as usize).ok_or_else(||referential("unknown data"))?;children.extend(d.components.iter().map(|(_,v)|*v));Some(d.result_type)},
                StaticValueData::Variant(id)=>{let v=store.variant_values_arena().get(id.raw() as usize).ok_or_else(||referential("unknown variant"))?;children.extend(v.fields.iter().map(|(_,v)|*v));Some(v.result_type)},
                StaticValueData::Pack(id)=>{let p=store.packs_arena().get(id.raw() as usize).ok_or_else(||referential("unknown pack"))?;let backing=store.pack_backings_arena().get(p.backing.raw() as usize).ok_or_else(||referential("unknown pack backing"))?;children.extend(backing.positionals.iter().copied());children.extend(backing.labeled_values.iter().copied());None},
                StaticValueData::Unit|StaticValueData::Bool(_)|StaticValueData::Char(_)|StaticValueData::Int(_)|StaticValueData::Float(_)|StaticValueData::String(_)|StaticValueData::Symbol(_)=>None,
            };
            if let Some(ty)=ty {let mut key=key.clone();key.family_tag=phalcom_compiler_artifact::semantic::type_store::CanonicalRootFamilyTag::StaticValue;key.product_subkey=*ordinal;
                *ordinal=ordinal.checked_add(1).ok_or_else(||referential("static type root ordinal overflow"))?;
                types.register_type_root(types.expected_lineage(),key,ty)?;
            }
            for child in children {visit(store,child,key,ordinal,types,active,depth+1,limits)?;}
            active.remove(&value.value_id());Ok(())
        }
        let mut ordinals=BTreeMap::new();
        for (key,&value) in &self.roots {let ordinal=ordinals.entry(key.owner.clone()).or_insert(0);visit(store,value,key,ordinal,types,&mut BTreeSet::new(),0,limits)?;}
        Ok(())
    }

    pub fn close_graph(
        &mut self,
        store: &StaticValueStore,
        type_export: &TypeExportContext,
        limits: &ValidationLimits,
    ) -> Result<(), ArtifactError> {
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
                    .values_arena()
                    .get(val_id.raw() as usize)
                    .ok_or_else(|| referential(format!("unknown static value id {}", val_id.raw())))?;

                match data {
                    StaticValueData::Unit
                    | StaticValueData::Bool(_)
                    | StaticValueData::Char(_)
                    | StaticValueData::Int(_)
                    | StaticValueData::Float(_) => {}
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
                        let tuple = self
                            .store
                            .tuples_arena()
                            .get(tid.raw() as usize)
                            .ok_or_else(|| referential(format!("unknown static tuple id {}", tid.raw())))?;
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
                        let seq = self
                            .store
                            .sequences_arena()
                            .get(sid.raw() as usize)
                            .ok_or_else(|| referential(format!("unknown static sequence id {}", sid.raw())))?;
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
                        let rec = self
                            .store
                            .records_arena()
                            .get(rid.raw() as usize)
                            .ok_or_else(|| referential(format!("unknown static record id {}", rid.raw())))?;
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
                        let data_val = self
                            .store
                            .data_values_arena()
                            .get(did.raw() as usize)
                            .ok_or_else(|| referential(format!("unknown static data id {}", did.raw())))?;
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
                        let var_val = self
                            .store
                            .variant_values_arena()
                            .get(vid.raw() as usize)
                            .ok_or_else(|| referential(format!("unknown static variant id {}", vid.raw())))?;
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
                        let pack_data = self
                            .store
                            .packs_arena()
                            .get(pid.raw() as usize)
                            .ok_or_else(|| referential(format!("unknown static pack id {}", pid.raw())))?;
                        let backing = self
                            .store
                            .pack_backings_arena()
                            .get(pack_data.backing.raw() as usize)
                            .ok_or_else(|| referential(format!("unknown static pack backing id {}", pack_data.backing.raw())))?;
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
                    .strings_arena()
                    .get(sid.raw() as usize)
                    .map(|s| s.to_string())
                    .ok_or_else(|| referential("unknown static string"))
            })
            .collect::<Result<_, _>>()?;

        let symbols: Vec<String> = self
            .symbol_order
            .iter()
            .map(|sym| {
                store
                    .symbols_arena()
                    .get(sym.raw() as usize)
                    .map(|s| s.to_string())
                    .ok_or_else(|| referential("unknown static symbol"))
            })
            .collect::<Result<_, _>>()?;

        let tuples: Vec<Vec<ArtifactStaticValueId>> = self
            .tuple_order
            .iter()
            .map(|tid| {
                let tuple = store
                    .tuples_arena()
                    .get(tid.raw() as usize)
                    .ok_or_else(|| referential("unknown static tuple"))?;
                tuple.iter().map(|&v| self.value(v)).collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<_, _>>()?;

        let sequences: Vec<StaticSequenceV1> = self
            .sequence_order
            .iter()
            .map(|sid| {
                let seq = store
                    .sequences_arena()
                    .get(sid.raw() as usize)
                    .ok_or_else(|| referential("unknown static sequence"))?;
                let family = match seq.family() {
                    StaticSequenceFamily::Array => SequenceFamilyV1::Array,
                    StaticSequenceFamily::Vector => SequenceFamilyV1::Vector,
                };
                let result_type = type_export.ty(seq.result_type())?;
                let elements = seq.elements().iter().map(|&v| self.value(v)).collect::<Result<Vec<_>, _>>()?;
                Ok(StaticSequenceV1 {
                    family,
                    result_type,
                    elements,
                })
            })
            .collect::<Result<Vec<StaticSequenceV1>, ArtifactError>>()?;

        let records: Vec<Vec<StaticRecordFieldV1>> = self
            .record_order
            .iter()
            .map(|rid| {
                let rec = store
                    .records_arena()
                    .get(rid.raw() as usize)
                    .ok_or_else(|| referential("unknown static record"))?;
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
                let d = store
                    .data_values_arena()
                    .get(did.raw() as usize)
                    .ok_or_else(|| referential("unknown static data"))?;
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
                let v = store
                    .variant_values_arena()
                    .get(vid.raw() as usize)
                    .ok_or_else(|| referential("unknown static variant"))?;
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
                Ok(StaticVariantValueV1 {
                    variant,
                    result_type,
                    fields,
                })
            })
            .collect::<Result<Vec<StaticVariantValueV1>, ArtifactError>>()?;

        let pack_backings: Vec<StaticPackBackingV1> = self
            .pack_backing_order
            .iter()
            .map(|bid| {
                let b = store
                    .pack_backings_arena()
                    .get(bid.raw() as usize)
                    .ok_or_else(|| referential("unknown static pack backing"))?;
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
                let p = store
                    .packs_arena()
                    .get(pid.raw() as usize)
                    .ok_or_else(|| referential("unknown static pack"))?;
                let backing = *self
                    .pack_backing_ids
                    .get(&p.backing)
                    .ok_or_else(|| referential("unrelocated pack backing"))?;
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
                let data = store
                    .values_arena()
                    .get(vid.raw() as usize)
                    .ok_or_else(|| referential("unknown static value"))?;
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
