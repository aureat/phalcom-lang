//! Interned, immutable static values with snapshot-stable store-qualified refs.

use crate::data_semantics::DataInfo;
use crate::enum_semantics::VariantInfo;
use crate::field_key::FieldKey;
use crate::identity::{DataComponentId, DataConstructorId, VariantId};
pub use crate::scalar::StaticInt;
use crate::static_domain::StaticValueDomain;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};
use crate::types::id::{KindId, TypeId};
use crate::types::relation::{Assignability, TypeHierarchy};
use crate::types::row::{RecordRowField, RecordRowTail};
use crate::types::store::{TupleTypeElement, TypeData, TypeStore};
use im::{HashMap, Vector};
use num_bigint::BigInt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Lazily allocated persistent-map overlay. Frozen artifact stores begin with
/// no local interner roots; the first session-local insertion creates only the
/// affected map.
#[derive(Clone)]
struct LazyHashMap<K, V>(Option<HashMap<K, V>>);

impl<K, V> std::fmt::Debug for LazyHashMap<K, V> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("LazyHashMap(..)")
    }
}

impl<K, V> Default for LazyHashMap<K, V> {
    fn default() -> Self {
        Self(None)
    }
}

impl<K, V> LazyHashMap<K, V> {
    fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q> + Hash + Eq,
        Q: Hash + Eq,
    {
        self.0.as_ref().and_then(|map| map.get(key))
    }

    fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Hash + Eq + Clone,
        V: Clone,
    {
        self.0.get_or_insert_with(HashMap::new).insert(key, value)
    }

    #[cfg(test)]
    fn ptr_eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(left), Some(right)) => left.ptr_eq(right),
            (None, None) => true,
            _ => false,
        }
    }
}

static NEXT_STATIC_VALUE_STORE_ID: AtomicU64 = AtomicU64::new(1);

/// Identity of one append-only static-value store lineage.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticValueStoreId(u64);

impl StaticValueStoreId {
    /// Returns the globally unique raw identity.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Index of a value within its allocating [`StaticValueStore`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticValueId(pub u32);

impl StaticValueId {
    /// Creates a store-local index from raw value.
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns this value's store-local index.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Reference that remains meaningful across clones of one static-value store.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticValueRef {
    pub(crate) store: StaticValueStoreId,
    pub(crate) value: StaticValueId,
}

impl StaticValueRef {
    /// Creates a new static value reference.
    pub const fn new(store: StaticValueStoreId, value: StaticValueId) -> Self {
        Self { store, value }
    }

    /// Returns the store lineage that owns this reference.
    pub const fn store_id(self) -> StaticValueStoreId {
        self.store
    }

    /// Returns the value's index within that lineage.
    pub const fn value_id(self) -> StaticValueId {
        self.value
    }
}

/// Store-local identity for interned UTF-8 string contents.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticStringId(pub u32);

impl StaticStringId {
    /// Returns the store-local string index.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Store-local identity for an interned symbol spelling.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticSymbolId(pub u32);

impl StaticSymbolId {
    /// Returns the store-local symbol index.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

macro_rules! structural_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub u32);
        impl $name {
            /// Returns this store-local structural node index.
            pub const fn raw(self) -> u32 {
                self.0
            }
        }
    };
}

structural_id!(StaticTupleId);
structural_id!(StaticSequenceId);
structural_id!(StaticRecordId);
structural_id!(StaticDataValueId);
structural_id!(StaticVariantValueId);
structural_id!(StaticPackBackingId);
structural_id!(StaticPackId);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticPackBacking {
    pub positionals: Box<[StaticValueRef]>,
    pub labels: Box<[StaticSymbolId]>,
    pub labeled_values: Box<[StaticValueRef]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticPackData {
    pub backing: StaticPackBackingId,
    pub positional_start: u32,
    pub positional_len: u32,
    pub labeled_start: u32,
    pub labeled_len: u32,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticRecordField {
    pub key: FieldKey,
    pub value: StaticValueRef,
}

/// Canonical family retained from a checked homogeneous sequence result type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StaticSequenceFamily {
    Array,
    Vector,
}

/// Immutable homogeneous sequence value with its already-checked ordinary type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticSequenceData {
    pub family: StaticSequenceFamily,
    pub result_type: TypeId,
    pub elements: Box<[StaticValueRef]>,
}

impl StaticSequenceData {
    /// Returns the canonical checked collection family.
    pub const fn family(&self) -> StaticSequenceFamily {
        self.family
    }

    /// Returns the canonical `Array<T>` or `Vector<T, N>` result type.
    pub const fn result_type(&self) -> TypeId {
        self.result_type
    }

    /// Returns the immutable logical contents in source order.
    pub fn elements(&self) -> &[StaticValueRef] {
        &self.elements
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticDataValue {
    pub owner: crate::identity::DeclarationId,
    pub constructor: DataConstructorId,
    pub result_type: TypeId,
    pub components: Box<[(DataComponentId, StaticValueRef)]>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticVariantValue {
    pub variant: VariantId,
    pub result_type: TypeId,
    pub fields: Box<[(crate::identity::VariantFieldId, StaticValueRef)]>,
}

/// IEEE-754 binary64 representation retained exactly for materialization.
///
/// Equality and hashing on this wrapper describe representation identity.
/// Use [`static_float_eq`] and [`static_float_hash`] for Telos Float semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticFloat {
    bits: u64,
}

impl StaticFloat {
    /// Stores a float without changing its payload or signed-zero bits.
    pub const fn from_bits(bits: u64) -> Self {
        Self { bits }
    }

    /// Captures the exact IEEE-754 representation of `value`.
    pub fn from_f64(value: f64) -> Self {
        Self::from_bits(value.to_bits())
    }

    /// Returns the retained IEEE-754 bits.
    pub const fn bits(self) -> u64 {
        self.bits
    }

    /// Reconstructs the represented binary64 value.
    pub fn to_f64(self) -> f64 {
        f64::from_bits(self.bits)
    }
}

/// Canonical semantic value node. Float `Eq`/`Hash` here preserve representation bits.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StaticValueData {
    /// The sole unit value.
    Unit,
    /// Boolean value.
    Bool(bool),
    /// Unicode scalar value, represented by Rust's validated `char`.
    Char(char),
    /// Exact integer value.
    Int(StaticInt),
    /// Exact IEEE-754 representation.
    Float(StaticFloat),
    /// UTF-8 string contents in the owning store.
    String(StaticStringId),
    /// Symbol spelling in the owning store.
    Symbol(StaticSymbolId),
    /// Proper type value.
    Type(TypeId),
    /// C3 static term that is not a proper type.
    StaticTerm(TypeId),
    /// Ordered immutable tuple value.
    Tuple(StaticTupleId),
    /// Immutable homogeneous `Array` or `Vector` value.
    Sequence(StaticSequenceId),
    /// Immutable Record value with canonical heterogeneous FieldKey keys.
    Record(StaticRecordId),
    /// Immutable declared data product.
    Data(StaticDataValueId),
    /// Immutable exact enum variant value.
    Variant(StaticVariantValueId),
    /// Immutable first-class Pack value (whole or slice).
    Pack(StaticPackId),
}

/// Invalid operation on a store-qualified static-value reference.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum StaticValueStoreError {
    /// The ref belongs to another store lineage.
    #[error("static value belongs to store {actual:?}, expected {expected:?}")]
    StoreMismatch { expected: StaticValueStoreId, actual: StaticValueStoreId },
    /// The store-local value or text index is absent.
    #[error("unknown static value index {0}")]
    UnknownValue(u32),
    /// The TypeId is outside the supplied TypeStore.
    #[error("type id {0:?} is not present in the supplied TypeStore")]
    UnknownType(TypeId),
    /// A Type-valued static value must have proper kind `Type`.
    #[error("type id {ty:?} has kind {kind:?}, expected proper kind Type")]
    NonProperType { ty: TypeId, kind: KindId },
    /// A StaticTerm must not be a proper type.
    #[error("type id {0:?} has proper kind Type and must be stored as StaticValueData::Type")]
    ProperTypeAsStaticTerm(TypeId),
    /// A structural value needs declaration products to derive its ordinary type.
    #[error("structural static value type requires semantic declaration products")]
    StructuralValueNeedsSemanticProducts,
    /// Structural record type formation was rejected by the canonical row store.
    #[error("static record could not form a canonical closed record type")]
    InvalidRecordType,
    /// A child ref was allocated by another store lineage.
    #[error("structural child belongs to another static-value store")]
    ForeignChild,
    /// The same record key occurred more than once.
    #[error("duplicate static record key `{0}`")]
    DuplicateRecordKey(FieldKey),
    /// A reduced static value is outside the closed FieldKey domain.
    #[error("static value is not a valid Record field key")]
    NotAFieldKey,
    /// The same static pack label occurred more than once.
    #[error("duplicate static pack label `{0}`")]
    DuplicatePackLabel(Box<str>),
    /// A static pack slice range is invalid.
    #[error("invalid static pack slice range")]
    InvalidPackSliceRange,
    /// The canonical data declaration/constructor metadata is inconsistent.
    #[error("static data components do not match canonical declaration metadata")]
    InvalidDataComponents,
    /// A data value has the wrong constructor identity or result type.
    #[error("static data constructor/result type does not match canonical declaration metadata")]
    InvalidDataIdentity,
    /// The variant payload does not match the exact canonical variant fields.
    #[error("static variant payload does not match canonical variant metadata")]
    InvalidVariantPayload,
    /// The retained sequence type is not a canonical applied Array or Vector type.
    #[error("type {0:?} is not a canonical Array or Vector sequence type")]
    InvalidSequenceType(TypeId),
    /// A Vector sequence type does not carry a closed exact static integer extent.
    #[error("Vector sequence type {0:?} does not carry a closed exact static Int extent")]
    InvalidSequenceExtent(TypeId),
    /// A closed Vector extent cannot be negative.
    #[error("Vector sequence extent {0} cannot be negative")]
    NegativeSequenceExtent(StaticInt),
    /// The closed Vector extent does not equal the sequence's logical element count.
    #[error("Vector sequence extent {extent} does not match logical element count {actual}")]
    SequenceExtentMismatch { extent: StaticInt, actual: usize },
}

/// Frozen immutable prefix for StaticValueStore loaded from a compiler artifact.
#[derive(Debug)]
pub struct FrozenStaticValuePrefix {
    pub id: StaticValueStoreId,
    pub values: Box<[StaticValueData]>,
    pub value_to_id: Box<[StaticValueId]>,
    pub strings: Box<[Box<str>]>,
    pub string_to_id: Box<[StaticStringId]>,
    pub symbols: Box<[Box<str>]>,
    pub symbol_to_id: Box<[StaticSymbolId]>,
    pub tuples: Box<[Box<[StaticValueRef]>]>,
    pub tuple_to_id: Box<[StaticTupleId]>,
    pub sequences: Box<[StaticSequenceData]>,
    pub sequence_to_id: Box<[StaticSequenceId]>,
    pub records: Box<[Box<[StaticRecordField]>]>,
    pub record_to_id: Box<[StaticRecordId]>,
    pub data_values: Box<[StaticDataValue]>,
    pub data_value_to_id: Box<[StaticDataValueId]>,
    pub variant_values: Box<[StaticVariantValue]>,
    pub variant_value_to_id: Box<[StaticVariantValueId]>,
    pub pack_backings: Box<[StaticPackBacking]>,
    pub pack_backing_to_id: Box<[StaticPackBackingId]>,
    pub packs: Box<[StaticPackData]>,
    pub pack_to_id: Box<[StaticPackId]>,
}

impl FrozenStaticValuePrefix {
    pub fn find_value(&self, data: &StaticValueData) -> Option<StaticValueId> {
        let idx = self.value_to_id.binary_search_by(|&id| self.values[id.0 as usize].cmp(data)).ok()?;
        Some(self.value_to_id[idx])
    }

    pub fn find_string(&self, data: &str) -> Option<StaticStringId> {
        let idx = self.string_to_id.binary_search_by(|&id| self.strings[id.0 as usize].as_ref().cmp(data)).ok()?;
        Some(self.string_to_id[idx])
    }

    pub fn find_symbol(&self, data: &str) -> Option<StaticSymbolId> {
        let idx = self.symbol_to_id.binary_search_by(|&id| self.symbols[id.0 as usize].as_ref().cmp(data)).ok()?;
        Some(self.symbol_to_id[idx])
    }

    pub fn find_tuple(&self, data: &[StaticValueRef]) -> Option<StaticTupleId> {
        let idx = self.tuple_to_id.binary_search_by(|&id| self.tuples[id.0 as usize].as_ref().cmp(data)).ok()?;
        Some(self.tuple_to_id[idx])
    }

    pub fn find_sequence(&self, data: &StaticSequenceData) -> Option<StaticSequenceId> {
        let idx = self.sequence_to_id.binary_search_by(|&id| self.sequences[id.0 as usize].cmp(data)).ok()?;
        Some(self.sequence_to_id[idx])
    }

    pub fn find_record(&self, data: &[StaticRecordField]) -> Option<StaticRecordId> {
        let idx = self.record_to_id.binary_search_by(|&id| self.records[id.0 as usize].as_ref().cmp(data)).ok()?;
        Some(self.record_to_id[idx])
    }

    pub fn find_data_value(&self, data: &StaticDataValue) -> Option<StaticDataValueId> {
        let idx = self.data_value_to_id.binary_search_by(|&id| self.data_values[id.0 as usize].cmp(data)).ok()?;
        Some(self.data_value_to_id[idx])
    }

    pub fn find_variant_value(&self, data: &StaticVariantValue) -> Option<StaticVariantValueId> {
        let idx = self
            .variant_value_to_id
            .binary_search_by(|&id| self.variant_values[id.0 as usize].cmp(data))
            .ok()?;
        Some(self.variant_value_to_id[idx])
    }

    pub fn find_pack_backing(&self, data: &StaticPackBacking) -> Option<StaticPackBackingId> {
        let idx = self
            .pack_backing_to_id
            .binary_search_by(|&id| self.pack_backings[id.0 as usize].cmp(data))
            .ok()?;
        Some(self.pack_backing_to_id[idx])
    }

    pub fn find_pack(&self, data: &StaticPackData) -> Option<StaticPackId> {
        let idx = self.pack_to_id.binary_search_by(|&id| self.packs[id.0 as usize].cmp(data)).ok()?;
        Some(self.pack_to_id[idx])
    }
}

/// Append-only canonical arena for semantic compile-time values.
///
/// Cloning the store preserves its identity and all existing indices, matching
/// the `TypeStore` clone-on-write snapshot model.
#[derive(Clone, Debug)]
pub struct StaticValueStore {
    id: StaticValueStoreId,
    frozen: Option<Arc<FrozenStaticValuePrefix>>,
    values: Vector<StaticValueData>,
    interner: LazyHashMap<StaticValueData, StaticValueId>,
    strings: Vector<Box<str>>,
    string_ids: LazyHashMap<Box<str>, StaticStringId>,
    symbols: Vector<Box<str>>,
    symbol_ids: LazyHashMap<Box<str>, StaticSymbolId>,
    tuples: Vector<Box<[StaticValueRef]>>,
    tuple_ids: LazyHashMap<Box<[StaticValueRef]>, StaticTupleId>,
    sequences: Vector<StaticSequenceData>,
    sequence_ids: LazyHashMap<StaticSequenceData, StaticSequenceId>,
    records: Vector<Box<[StaticRecordField]>>,
    record_ids: LazyHashMap<Box<[StaticRecordField]>, StaticRecordId>,
    data_values: Vector<StaticDataValue>,
    data_value_ids: LazyHashMap<StaticDataValue, StaticDataValueId>,
    variant_values: Vector<StaticVariantValue>,
    variant_value_ids: LazyHashMap<StaticVariantValue, StaticVariantValueId>,
    pack_backings: Vector<StaticPackBacking>,
    pack_backing_ids: LazyHashMap<StaticPackBacking, StaticPackBackingId>,
    packs: Vector<StaticPackData>,
    pack_ids: LazyHashMap<StaticPackData, StaticPackId>,
}

impl Default for StaticValueStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticValueStore {
    pub fn fresh_id() -> StaticValueStoreId {
        StaticValueStoreId(NEXT_STATIC_VALUE_STORE_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Validates all indices of a retained immutable root in this lineage.
    /// Equal store IDs alone do not prove that two independently extended forks
    /// contain the same values at a canonical index.
    pub fn extends_canonical_prefix(&self, prefix: &Self) -> bool {
        (self.id == prefix.id || self.frozen.as_ref().map_or(false, |f| f.id == prefix.id))
            && self.value_count() >= prefix.value_count()
            && (0..prefix.value_count()).all(|i| self.get_value(StaticValueId(i as u32)) == prefix.get_value(StaticValueId(i as u32)))
            && self.string_count() >= prefix.string_count()
            && (0..prefix.string_count()).all(|i| self.get_string(StaticStringId(i as u32)) == prefix.get_string(StaticStringId(i as u32)))
            && self.symbol_count() >= prefix.symbol_count()
            && (0..prefix.symbol_count()).all(|i| self.get_symbol(StaticSymbolId(i as u32)) == prefix.get_symbol(StaticSymbolId(i as u32)))
            && self.tuple_count() >= prefix.tuple_count()
            && (0..prefix.tuple_count()).all(|i| self.get_tuple(StaticTupleId(i as u32)) == prefix.get_tuple(StaticTupleId(i as u32)))
            && self.sequence_count() >= prefix.sequence_count()
            && (0..prefix.sequence_count()).all(|i| self.get_sequence(StaticSequenceId(i as u32)) == prefix.get_sequence(StaticSequenceId(i as u32)))
            && self.record_count() >= prefix.record_count()
            && (0..prefix.record_count()).all(|i| self.get_record(StaticRecordId(i as u32)) == prefix.get_record(StaticRecordId(i as u32)))
            && self.data_value_count() >= prefix.data_value_count()
            && (0..prefix.data_value_count()).all(|i| self.get_data(StaticDataValueId(i as u32)) == prefix.get_data(StaticDataValueId(i as u32)))
            && self.variant_value_count() >= prefix.variant_value_count()
            && (0..prefix.variant_value_count()).all(|i| self.get_variant(StaticVariantValueId(i as u32)) == prefix.get_variant(StaticVariantValueId(i as u32)))
            && self.pack_backing_count() >= prefix.pack_backing_count()
            && (0..prefix.pack_backing_count())
                .all(|i| self.get_pack_backing(StaticPackBackingId(i as u32)) == prefix.get_pack_backing(StaticPackBackingId(i as u32)))
            && self.pack_count() >= prefix.pack_count()
            && (0..prefix.pack_count()).all(|i| self.get_pack_data(StaticPackId(i as u32)) == prefix.get_pack_data(StaticPackId(i as u32)))
    }

    /// Creates a fresh store lineage.
    pub fn new() -> Self {
        Self {
            id: Self::fresh_id(),
            frozen: None,
            values: Vector::new(),
            interner: LazyHashMap::default(),
            strings: Vector::new(),
            string_ids: LazyHashMap::default(),
            symbols: Vector::new(),
            symbol_ids: LazyHashMap::default(),
            tuples: Vector::new(),
            tuple_ids: LazyHashMap::default(),
            sequences: Vector::new(),
            sequence_ids: LazyHashMap::default(),
            records: Vector::new(),
            record_ids: LazyHashMap::default(),
            data_values: Vector::new(),
            data_value_ids: LazyHashMap::default(),
            variant_values: Vector::new(),
            variant_value_ids: LazyHashMap::default(),
            pack_backings: Vector::new(),
            pack_backing_ids: LazyHashMap::default(),
            packs: Vector::new(),
            pack_ids: LazyHashMap::default(),
        }
    }

    /// Initializes a store from a frozen immutable prefix.
    pub fn from_frozen(prefix: Arc<FrozenStaticValuePrefix>) -> Self {
        Self {
            id: prefix.id,
            frozen: Some(prefix),
            values: Vector::new(),
            interner: LazyHashMap::default(),
            strings: Vector::new(),
            string_ids: LazyHashMap::default(),
            symbols: Vector::new(),
            symbol_ids: LazyHashMap::default(),
            tuples: Vector::new(),
            tuple_ids: LazyHashMap::default(),
            sequences: Vector::new(),
            sequence_ids: LazyHashMap::default(),
            records: Vector::new(),
            record_ids: LazyHashMap::default(),
            data_values: Vector::new(),
            data_value_ids: LazyHashMap::default(),
            variant_values: Vector::new(),
            variant_value_ids: LazyHashMap::default(),
            pack_backings: Vector::new(),
            pack_backing_ids: LazyHashMap::default(),
            packs: Vector::new(),
            pack_ids: LazyHashMap::default(),
        }
    }

    /// Create a fork from a canonical prefix.
    pub fn fork_from_baseline(&self) -> Self {
        let mut fork = self.clone();
        fork.id = Self::fresh_id();
        fork
    }

    /// Returns this store's lineage identity.
    pub const fn id(&self) -> StaticValueStoreId {
        self.id
    }

    #[inline]
    pub fn value_count(&self) -> usize {
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.values.len());
        frozen_len + self.values.len()
    }

    #[inline]
    pub fn string_count(&self) -> usize {
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.strings.len());
        frozen_len + self.strings.len()
    }

    #[inline]
    pub fn symbol_count(&self) -> usize {
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.symbols.len());
        frozen_len + self.symbols.len()
    }

    #[inline]
    pub fn tuple_count(&self) -> usize {
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.tuples.len());
        frozen_len + self.tuples.len()
    }

    #[inline]
    pub fn sequence_count(&self) -> usize {
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.sequences.len());
        frozen_len + self.sequences.len()
    }

    #[inline]
    pub fn record_count(&self) -> usize {
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.records.len());
        frozen_len + self.records.len()
    }

    #[inline]
    pub fn data_value_count(&self) -> usize {
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.data_values.len());
        frozen_len + self.data_values.len()
    }

    #[inline]
    pub fn variant_value_count(&self) -> usize {
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.variant_values.len());
        frozen_len + self.variant_values.len()
    }

    #[inline]
    pub fn pack_backing_count(&self) -> usize {
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.pack_backings.len());
        frozen_len + self.pack_backings.len()
    }

    #[inline]
    pub fn pack_count(&self) -> usize {
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.packs.len());
        frozen_len + self.packs.len()
    }

    /// Returns the number of interned value nodes.
    pub fn len(&self) -> usize {
        self.value_count()
    }

    /// Returns whether this store has no interned value nodes.
    pub fn is_empty(&self) -> bool {
        self.value_count() == 0
    }

    /// Interns and returns the unit value.
    pub fn intern_unit(&mut self) -> StaticValueRef {
        self.intern(StaticValueData::Unit)
    }

    /// Interns a boolean value.
    pub fn intern_bool(&mut self, value: bool) -> StaticValueRef {
        self.intern(StaticValueData::Bool(value))
    }

    /// Interns a Unicode scalar value.
    pub fn intern_char(&mut self, value: char) -> StaticValueRef {
        self.intern(StaticValueData::Char(value))
    }

    /// Interns an exact integer, normalizing values back to `i64` when possible.
    pub fn intern_int(&mut self, value: impl Into<BigInt>) -> StaticValueRef {
        self.intern(StaticValueData::Int(StaticInt::new(value.into())))
    }

    /// Interns a float by exact representation bits.
    pub fn intern_float(&mut self, value: StaticFloat) -> StaticValueRef {
        self.intern(StaticValueData::Float(value))
    }

    /// Interns UTF-8 string contents by value.
    pub fn intern_string(&mut self, value: impl AsRef<str>) -> StaticValueRef {
        let text = value.as_ref();
        let id = if let Some(&id) = self.string_ids.get(text) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_string(text)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.strings.len());
            let id = StaticStringId((frozen_len + self.strings.len()) as u32);
            let text: Box<str> = text.into();
            self.strings.push_back(text.clone());
            self.string_ids.insert(text, id);
            id
        };
        self.intern(StaticValueData::String(id))
    }

    /// Interns a symbol spelling and returns its store-local `StaticSymbolId`.
    pub fn intern_symbol_id(&mut self, spelling: impl AsRef<str>) -> StaticSymbolId {
        let text = spelling.as_ref();
        if let Some(&id) = self.symbol_ids.get(text) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_symbol(text)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.symbols.len());
            let id = StaticSymbolId((frozen_len + self.symbols.len()) as u32);
            let text: Box<str> = text.into();
            self.symbols.push_back(text.clone());
            self.symbol_ids.insert(text, id);
            id
        }
    }

    /// Interns a symbol spelling by content, independently of VM symbol identity.
    pub fn intern_symbol(&mut self, spelling: impl AsRef<str>) -> StaticValueRef {
        let id = self.intern_symbol_id(spelling);
        self.intern(StaticValueData::Symbol(id))
    }

    /// Interns an ordered tuple. Only already-existing same-lineage refs can
    /// be children, so the append-only arena can contain shared DAGs but not cycles.
    pub fn intern_tuple(&mut self, elements: Vec<StaticValueRef>) -> Result<StaticValueRef, StaticValueStoreError> {
        if elements.is_empty() {
            return Ok(self.intern_unit());
        }
        self.check_children(elements.iter().copied())?;
        let elements = elements.into_boxed_slice();
        let id = if let Some(&id) = self.tuple_ids.get(elements.as_ref()) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_tuple(&elements)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.tuples.len());
            let id = StaticTupleId((frozen_len + self.tuples.len()) as u32);
            self.tuples.push_back(elements.clone());
            self.tuple_ids.insert(elements, id);
            id
        };
        Ok(self.intern(StaticValueData::Tuple(id)))
    }

    /// Interns an immutable homogeneous Array or Vector value.
    ///
    /// Family selection and element assignability are owned by semantic
    /// checking. This boundary validates only the retained canonical type and
    /// the static representation's store and extent invariants.
    ///
    /// The production caller must supply the checked result type and expanded
    /// children from a committed `SequenceLiteralResolution`; this method does
    /// not implement a second element-subtyping check.
    pub fn intern_sequence(&mut self, result_type: TypeId, elements: Vec<StaticValueRef>, types: &TypeStore) -> Result<StaticValueRef, StaticValueStoreError> {
        self.check_children(elements.iter().copied())?;
        let result_kind = self.checked_kind(result_type, types)?;
        if result_kind != KindId::TYPE {
            return Err(StaticValueStoreError::NonProperType {
                ty: result_type,
                kind: result_kind,
            });
        }

        let (declaration, arguments) = match types.get(result_type) {
            crate::types::store::TypeData::Applied { origin, arguments } => {
                if origin.index() >= types.len() {
                    return Err(StaticValueStoreError::UnknownType(*origin));
                }
                let crate::types::store::TypeData::Nominal { declaration } = types.get(*origin) else {
                    return Err(StaticValueStoreError::InvalidSequenceType(result_type));
                };
                (declaration.clone(), arguments.to_vec())
            }
            _ => return Err(StaticValueStoreError::InvalidSequenceType(result_type)),
        };
        let _element_type = match arguments.first() {
            Some(crate::types::store::StaticArgument::Term(element_type)) => {
                let kind = self.checked_kind(*element_type, types)?;
                if kind != KindId::TYPE {
                    return Err(StaticValueStoreError::NonProperType { ty: *element_type, kind });
                }
                *element_type
            }
            _ => return Err(StaticValueStoreError::InvalidSequenceType(result_type)),
        };
        let declarations = crate::core_surface::CoreDeclarationIds::default();

        let family = if declaration == declarations.array {
            if arguments.len() != 1 {
                return Err(StaticValueStoreError::InvalidSequenceType(result_type));
            }
            StaticSequenceFamily::Array
        } else if declaration == declarations.vector {
            if arguments.len() != 2 {
                return Err(StaticValueStoreError::InvalidSequenceType(result_type));
            }
            let extent_type = match arguments[1] {
                crate::types::store::StaticArgument::Term(extent_type) => extent_type,
                crate::types::store::StaticArgument::RecordRow(_) | crate::types::store::StaticArgument::DynamicType => {
                    return Err(StaticValueStoreError::InvalidSequenceExtent(result_type));
                }
            };
            let extent_kind = self.checked_kind(extent_type, types)?;
            let crate::types::store::TypeData::ConstInt(extent) = types.get(extent_type) else {
                return Err(StaticValueStoreError::InvalidSequenceExtent(result_type));
            };
            if !matches!(
                types.get_kind(extent_kind),
                crate::types::KindData::StaticDomain(domain) if *domain == types.const_int_domain()
            ) {
                return Err(StaticValueStoreError::InvalidSequenceExtent(result_type));
            }
            if extent.to_bigint().sign() == num_bigint::Sign::Minus {
                return Err(StaticValueStoreError::NegativeSequenceExtent(extent.clone()));
            }
            if extent.to_bigint() != BigInt::from(elements.len()) {
                return Err(StaticValueStoreError::SequenceExtentMismatch {
                    extent: extent.clone(),
                    actual: elements.len(),
                });
            }
            StaticSequenceFamily::Vector
        } else {
            return Err(StaticValueStoreError::InvalidSequenceType(result_type));
        };

        let node = StaticSequenceData {
            family,
            result_type,
            elements: elements.into_boxed_slice(),
        };
        let id = if let Some(&id) = self.sequence_ids.get(&node) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_sequence(&node)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.sequences.len());
            let id = StaticSequenceId((frozen_len + self.sequences.len()) as u32);
            self.sequences.push_back(node.clone());
            self.sequence_ids.insert(node, id);
            id
        };
        Ok(self.intern(StaticValueData::Sequence(id)))
    }

    /// Looks up immutable homogeneous sequence data.
    pub fn get_sequence(&self, id: StaticSequenceId) -> Result<&StaticSequenceData, StaticValueStoreError> {
        let index = id.0 as usize;
        if let Some(frozen) = &self.frozen {
            if index < frozen.sequences.len() {
                return Ok(&frozen.sequences[index]);
            }
            self.sequences
                .get(index - frozen.sequences.len())
                .ok_or(StaticValueStoreError::UnknownValue(id.0))
        } else {
            self.sequences.get(index).ok_or(StaticValueStoreError::UnknownValue(id.0))
        }
    }

    /// Looks up the ordered children of an immutable positional tuple.
    pub fn get_tuple(&self, id: StaticTupleId) -> Result<&[StaticValueRef], StaticValueStoreError> {
        let index = id.0 as usize;
        if let Some(frozen) = &self.frozen {
            if index < frozen.tuples.len() {
                return Ok(&frozen.tuples[index]);
            }
            self.tuples
                .get(index - frozen.tuples.len())
                .map(Box::as_ref)
                .ok_or(StaticValueStoreError::UnknownValue(id.0))
        } else {
            self.tuples.get(index).map(Box::as_ref).ok_or(StaticValueStoreError::UnknownValue(id.0))
        }
    }

    /// Looks up the ordered fields of an immutable record.
    pub fn get_record(&self, id: StaticRecordId) -> Result<&[StaticRecordField], StaticValueStoreError> {
        let index = id.0 as usize;
        if let Some(frozen) = &self.frozen {
            if index < frozen.records.len() {
                return Ok(&frozen.records[index]);
            }
            self.records
                .get(index - frozen.records.len())
                .map(Box::as_ref)
                .ok_or(StaticValueStoreError::UnknownValue(id.0))
        } else {
            self.records.get(index).map(Box::as_ref).ok_or(StaticValueStoreError::UnknownValue(id.0))
        }
    }

    /// Looks up the components and metadata of an interned data value.
    pub fn get_data(&self, id: StaticDataValueId) -> Result<&StaticDataValue, StaticValueStoreError> {
        let index = id.0 as usize;
        if let Some(frozen) = &self.frozen {
            if index < frozen.data_values.len() {
                return Ok(&frozen.data_values[index]);
            }
            self.data_values
                .get(index - frozen.data_values.len())
                .ok_or(StaticValueStoreError::UnknownValue(id.0))
        } else {
            self.data_values.get(index).ok_or(StaticValueStoreError::UnknownValue(id.0))
        }
    }

    /// Looks up the fields and metadata of an interned variant value.
    pub fn get_variant(&self, id: StaticVariantValueId) -> Result<&StaticVariantValue, StaticValueStoreError> {
        let index = id.0 as usize;
        if let Some(frozen) = &self.frozen {
            if index < frozen.variant_values.len() {
                return Ok(&frozen.variant_values[index]);
            }
            self.variant_values
                .get(index - frozen.variant_values.len())
                .ok_or(StaticValueStoreError::UnknownValue(id.0))
        } else {
            self.variant_values.get(index).ok_or(StaticValueStoreError::UnknownValue(id.0))
        }
    }

    /// Interns a closed Record with canonical FieldKey ordering and exact duplicate rejection.
    pub fn intern_record(&mut self, fields: Vec<(FieldKey, StaticValueRef)>) -> Result<StaticValueRef, StaticValueStoreError> {
        self.check_children(fields.iter().map(|(_, value)| *value))?;
        let mut fields = fields.into_iter().map(|(key, value)| StaticRecordField { key, value }).collect::<Vec<_>>();
        fields.sort_by(|a, b| a.key.canonical_cmp(&b.key));
        for pair in fields.windows(2) {
            if pair[0].key == pair[1].key {
                return Err(StaticValueStoreError::DuplicateRecordKey(pair[0].key.clone()));
            }
        }
        let fields = fields.into_boxed_slice();
        let id = if let Some(&id) = self.record_ids.get(fields.as_ref()) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_record(&fields)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.records.len());
            let id = StaticRecordId((frozen_len + self.records.len()) as u32);
            self.records.push_back(fields.clone());
            self.record_ids.insert(fields, id);
            id
        };
        Ok(self.intern(StaticValueData::Record(id)))
    }

    /// Interns a validated pack backing without manufacturing a top-level
    /// StaticValueData::Pack. Artifact relocation needs backing identity before
    /// it can reconstruct one or more wire slices over that backing.
    pub(crate) fn intern_pack_backing_reconstructed(
        &mut self,
        backing: StaticPackBacking,
    ) -> Result<StaticPackBackingId, StaticValueStoreError> {
        self.check_children(
            backing
                .positionals
                .iter()
                .copied()
                .chain(backing.labeled_values.iter().copied()),
        )?;
        if backing.labels.len() != backing.labeled_values.len() {
            return Err(StaticValueStoreError::InvalidPackSliceRange);
        }
        let mut seen = std::collections::HashSet::new();
        for symbol in backing.labels.iter().copied() {
            let spelling = self
                .get_symbol(symbol)
                .ok_or(StaticValueStoreError::UnknownValue(symbol.0))?;
            if !seen.insert(spelling) {
                return Err(StaticValueStoreError::DuplicatePackLabel(spelling.into()));
            }
        }
        if let Some(&id) = self.pack_backing_ids.get(&backing) {
            return Ok(id);
        }
        if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_pack_backing(&backing)) {
            return Ok(id);
        }
        let frozen_len = self.frozen.as_ref().map_or(0, |f| f.pack_backings.len());
        let id = StaticPackBackingId((frozen_len + self.pack_backings.len()) as u32);
        self.pack_backings.push_back(backing.clone());
        self.pack_backing_ids.insert(backing, id);
        Ok(id)
    }

    /// Interns one validated slice descriptor over an already-interned backing
    /// and returns its canonical top-level static reference.
    pub(crate) fn intern_pack_slice_reconstructed(
        &mut self,
        data: StaticPackData,
    ) -> Result<StaticValueRef, StaticValueStoreError> {
        let backing = self.get_pack_backing(data.backing)?;
        let positional_end = data
            .positional_start
            .checked_add(data.positional_len)
            .ok_or(StaticValueStoreError::InvalidPackSliceRange)?;
        let labeled_end = data
            .labeled_start
            .checked_add(data.labeled_len)
            .ok_or(StaticValueStoreError::InvalidPackSliceRange)?;
        if positional_end as usize > backing.positionals.len()
            || labeled_end as usize > backing.labels.len()
            || labeled_end as usize > backing.labeled_values.len()
        {
            return Err(StaticValueStoreError::InvalidPackSliceRange);
        }
        let id = if let Some(&id) = self.pack_ids.get(&data) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_pack(&data)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.packs.len());
            let id = StaticPackId((frozen_len + self.packs.len()) as u32);
            self.packs.push_back(data);
            self.pack_ids.insert(data, id);
            id
        };
        Ok(self.intern(StaticValueData::Pack(id)))
    }

    /// Interns an immutable StaticPack from whole positional and labeled sequences.
    /// Labels must be unique and preserve their encounter order.
    pub fn intern_pack(
        &mut self,
        positionals: Vec<StaticValueRef>,
        labeled: Vec<(StaticSymbolId, StaticValueRef)>,
    ) -> Result<StaticValueRef, StaticValueStoreError> {
        self.check_children(positionals.iter().copied().chain(labeled.iter().map(|(_, v)| *v)))?;

        // Validate duplicate labels in labeled sequence
        let mut seen_labels = std::collections::HashSet::new();
        for (sym_id, _) in &labeled {
            let spelling = self.get_symbol(*sym_id).ok_or(StaticValueStoreError::UnknownValue(sym_id.0))?;
            if !seen_labels.insert(spelling.to_string()) {
                return Err(StaticValueStoreError::DuplicatePackLabel(spelling.into()));
            }
        }

        let pos_len = positionals.len() as u32;
        let lab_len = labeled.len() as u32;

        let (labels, labeled_values): (Vec<_>, Vec<_>) = labeled.into_iter().unzip();

        let backing_node = StaticPackBacking {
            positionals: positionals.into_boxed_slice(),
            labels: labels.into_boxed_slice(),
            labeled_values: labeled_values.into_boxed_slice(),
        };

        let backing_id = if let Some(&id) = self.pack_backing_ids.get(&backing_node) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_pack_backing(&backing_node)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.pack_backings.len());
            let id = StaticPackBackingId((frozen_len + self.pack_backings.len()) as u32);
            self.pack_backings.push_back(backing_node.clone());
            self.pack_backing_ids.insert(backing_node, id);
            id
        };

        let pack_data = StaticPackData {
            backing: backing_id,
            positional_start: 0,
            positional_len: pos_len,
            labeled_start: 0,
            labeled_len: lab_len,
        };

        let pack_id = if let Some(&id) = self.pack_ids.get(&pack_data) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_pack(&pack_data)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.packs.len());
            let id = StaticPackId((frozen_len + self.packs.len()) as u32);
            self.packs.push_back(pack_data);
            self.pack_ids.insert(pack_data, id);
            id
        };

        Ok(self.intern(StaticValueData::Pack(pack_id)))
    }

    /// Interns an O(1) slice of an existing StaticPack over the same backing.
    pub fn intern_pack_slice(
        &mut self,
        pack_ref: StaticValueRef,
        pos_offset: u32,
        pos_len: u32,
        lab_offset: u32,
        lab_len: u32,
    ) -> Result<StaticValueRef, StaticValueStoreError> {
        let pack_id = match *self.get(pack_ref)? {
            StaticValueData::Pack(id) => id,
            _ => return Err(StaticValueStoreError::InvalidPackSliceRange),
        };
        let pack_data = *self.get_pack_data(pack_id)?;

        if pos_offset.checked_add(pos_len).ok_or(StaticValueStoreError::InvalidPackSliceRange)? > pack_data.positional_len
            || lab_offset.checked_add(lab_len).ok_or(StaticValueStoreError::InvalidPackSliceRange)? > pack_data.labeled_len
        {
            return Err(StaticValueStoreError::InvalidPackSliceRange);
        }

        let new_pos_start = pack_data.positional_start + pos_offset;
        let new_lab_start = pack_data.labeled_start + lab_offset;

        let new_pack_data = StaticPackData {
            backing: pack_data.backing,
            positional_start: new_pos_start,
            positional_len: pos_len,
            labeled_start: new_lab_start,
            labeled_len: lab_len,
        };

        let id = if let Some(&id) = self.pack_ids.get(&new_pack_data) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_pack(&new_pack_data)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.packs.len());
            let id = StaticPackId((frozen_len + self.packs.len()) as u32);
            self.packs.push_back(new_pack_data);
            self.pack_ids.insert(new_pack_data, id);
            id
        };

        Ok(self.intern(StaticValueData::Pack(id)))
    }

    /// Looks up a pack's backing.
    pub fn get_pack_backing(&self, id: StaticPackBackingId) -> Result<&StaticPackBacking, StaticValueStoreError> {
        let index = id.0 as usize;
        if let Some(frozen) = &self.frozen {
            if index < frozen.pack_backings.len() {
                return Ok(&frozen.pack_backings[index]);
            }
            self.pack_backings
                .get(index - frozen.pack_backings.len())
                .ok_or(StaticValueStoreError::UnknownValue(id.0))
        } else {
            self.pack_backings.get(index).ok_or(StaticValueStoreError::UnknownValue(id.0))
        }
    }

    /// Looks up a pack's slice descriptor.
    pub fn get_pack_data(&self, id: StaticPackId) -> Result<&StaticPackData, StaticValueStoreError> {
        let index = id.0 as usize;
        if let Some(frozen) = &self.frozen {
            if index < frozen.packs.len() {
                return Ok(&frozen.packs[index]);
            }
            self.packs.get(index - frozen.packs.len()).ok_or(StaticValueStoreError::UnknownValue(id.0))
        } else {
            self.packs.get(index).ok_or(StaticValueStoreError::UnknownValue(id.0))
        }
    }

    /// Looks up a pack's slice descriptor and backing.
    pub fn get_pack(&self, id: StaticPackId) -> Result<(&StaticPackData, &StaticPackBacking), StaticValueStoreError> {
        let pack_data = self.get_pack_data(id)?;
        let backing = self.get_pack_backing(pack_data.backing)?;
        Ok((pack_data, backing))
    }

    /// Returns the visible positional slice for a pack.
    pub fn pack_positionals(&self, id: StaticPackId) -> Result<&[StaticValueRef], StaticValueStoreError> {
        let (data, backing) = self.get_pack(id)?;
        let start = data.positional_start as usize;
        let end = start + data.positional_len as usize;
        backing.positionals.get(start..end).ok_or(StaticValueStoreError::InvalidPackSliceRange)
    }

    /// Returns the visible labeled slice for a pack as an iterator of `(StaticSymbolId, StaticValueRef)`.
    pub fn pack_labeled(&self, id: StaticPackId) -> Result<Vec<(StaticSymbolId, StaticValueRef)>, StaticValueStoreError> {
        let (data, backing) = self.get_pack(id)?;
        let start = data.labeled_start as usize;
        let end = start + data.labeled_len as usize;
        let labels = backing.labels.get(start..end).ok_or(StaticValueStoreError::InvalidPackSliceRange)?;
        let values = backing.labeled_values.get(start..end).ok_or(StaticValueStoreError::InvalidPackSliceRange)?;
        Ok(labels.iter().copied().zip(values.iter().copied()).collect())
    }

    /// Reads one visible labeled value without materializing the Pack's labeled lane.
    pub fn pack_labeled_value_at(&self, id: StaticPackId, index: usize) -> Result<StaticValueRef, StaticValueStoreError> {
        let (data, backing) = self.get_pack(id)?;
        if index >= data.labeled_len as usize {
            return Err(StaticValueStoreError::InvalidPackSliceRange);
        }
        let absolute = (data.labeled_start as usize)
            .checked_add(index)
            .ok_or(StaticValueStoreError::InvalidPackSliceRange)?;
        backing
            .labeled_values
            .get(absolute)
            .copied()
            .ok_or(StaticValueStoreError::InvalidPackSliceRange)
    }

    /// Interns a declared data value using canonical component/constructor products.
    /// `result_type` is the already-resolved ordinary type from semantic checking.
    pub fn intern_data(
        &mut self,
        info: &DataInfo,
        result_type: TypeId,
        components: Vec<(DataComponentId, StaticValueRef)>,
        types: &TypeStore,
    ) -> Result<StaticValueRef, StaticValueStoreError> {
        self.check_children(components.iter().map(|(_, value)| *value))?;
        let result_kind = self.checked_kind(result_type, types)?;
        if result_kind != KindId::TYPE {
            return Err(StaticValueStoreError::NonProperType {
                ty: result_type,
                kind: result_kind,
            });
        }
        if info.constructor.constructor.owner != info.owner
            || types.nominal_origin_declaration(result_type) != Some(&info.owner)
            || info.shape_template.has_expansions()
            || components.len() != info.components.len()
            || info.constructor.parameters.len() != info.components.len()
            || info
                .constructor
                .parameters
                .iter()
                .zip(info.components.iter())
                .any(|(parameter, component)| parameter.component != component.id)
        {
            return Err(StaticValueStoreError::InvalidDataIdentity);
        }
        let mut by_id = components.into_iter().collect::<HashMap<_, _>>();
        let mut ordered = Vec::with_capacity(info.components.len());
        for component in info.components.iter() {
            let value = by_id.remove(&component.id).ok_or(StaticValueStoreError::InvalidDataComponents)?;
            ordered.push((component.id.clone(), value));
        }
        if !by_id.is_empty() {
            return Err(StaticValueStoreError::InvalidDataComponents);
        }
        let node = StaticDataValue {
            owner: info.owner.clone(),
            constructor: info.constructor.constructor.clone(),
            result_type,
            components: ordered.into_boxed_slice(),
        };
        let id = if let Some(&id) = self.data_value_ids.get(&node) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_data_value(&node)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.data_values.len());
            let id = StaticDataValueId((frozen_len + self.data_values.len()) as u32);
            self.data_values.push_back(node.clone());
            self.data_value_ids.insert(node, id);
            id
        };
        Ok(self.intern(StaticValueData::Data(id)))
    }

    /// Interns a value for one exact canonical enum variant identity.
    pub fn intern_variant(
        &mut self,
        info: &VariantInfo,
        result_type: TypeId,
        fields: Vec<(crate::identity::VariantFieldId, StaticValueRef)>,
        types: &TypeStore,
    ) -> Result<StaticValueRef, StaticValueStoreError> {
        self.check_children(fields.iter().map(|(_, value)| *value))?;
        let result_kind = self.checked_kind(result_type, types)?;
        if result_kind != KindId::TYPE {
            return Err(StaticValueStoreError::NonProperType {
                ty: result_type,
                kind: result_kind,
            });
        }
        let exact_variant_matches = match types.get(result_type) {
            TypeData::ExactCase { variant, .. } => types.variant_identity(*variant) == &info.id,
            _ => false,
        };
        if types.nominal_origin_declaration(result_type) != Some(&info.id.owner) || !exact_variant_matches || fields.len() != info.fields.len() {
            return Err(StaticValueStoreError::InvalidVariantPayload);
        }
        let mut by_id = fields.into_iter().collect::<HashMap<_, _>>();
        let mut ordered = Vec::with_capacity(info.fields.len());
        for field in info.fields.iter() {
            let value = by_id.remove(&field.id).ok_or(StaticValueStoreError::InvalidVariantPayload)?;
            ordered.push((field.id.clone(), value));
        }
        if !by_id.is_empty() {
            return Err(StaticValueStoreError::InvalidVariantPayload);
        }
        let node = StaticVariantValue {
            variant: info.id.clone(),
            result_type,
            fields: ordered.into_boxed_slice(),
        };
        let id = if let Some(&id) = self.variant_value_ids.get(&node) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_variant_value(&node)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.variant_values.len());
            let id = StaticVariantValueId((frozen_len + self.variant_values.len()) as u32);
            self.variant_values.push_back(node.clone());
            self.variant_value_ids.insert(node, id);
            id
        };
        Ok(self.intern(StaticValueData::Variant(id)))
    }

    /// Interns a data value reconstructed from durable metadata without re-checking declaration tables.
    pub fn intern_data_reconstructed(
        &mut self,
        owner: crate::identity::DeclarationId,
        constructor: crate::identity::DataConstructorId,
        result_type: TypeId,
        components: Vec<(crate::identity::DataComponentId, StaticValueRef)>,
    ) -> Result<StaticValueRef, StaticValueStoreError> {
        self.check_children(components.iter().map(|(_, value)| *value))?;
        let node = StaticDataValue {
            owner,
            constructor,
            result_type,
            components: components.into_boxed_slice(),
        };
        let id = if let Some(&id) = self.data_value_ids.get(&node) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_data_value(&node)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.data_values.len());
            let id = StaticDataValueId((frozen_len + self.data_values.len()) as u32);
            self.data_values.push_back(node.clone());
            self.data_value_ids.insert(node, id);
            id
        };
        Ok(self.intern(StaticValueData::Data(id)))
    }

    /// Interns a variant value reconstructed from durable metadata without re-checking declaration tables.
    pub fn intern_variant_reconstructed(
        &mut self,
        variant: crate::identity::VariantId,
        result_type: TypeId,
        fields: Vec<(crate::identity::VariantFieldId, StaticValueRef)>,
    ) -> Result<StaticValueRef, StaticValueStoreError> {
        self.check_children(fields.iter().map(|(_, value)| *value))?;
        let node = StaticVariantValue {
            variant,
            result_type,
            fields: fields.into_boxed_slice(),
        };
        let id = if let Some(&id) = self.variant_value_ids.get(&node) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_variant_value(&node)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.variant_values.len());
            let id = StaticVariantValueId((frozen_len + self.variant_values.len()) as u32);
            self.variant_values.push_back(node.clone());
            self.variant_value_ids.insert(node, id);
            id
        };
        Ok(self.intern(StaticValueData::Variant(id)))
    }

    /// Derives the ordinary Telos type for a static value through `TypeStore`
    /// and the supplied canonical declaration products.
    pub fn static_value_type(
        &mut self,
        reference: StaticValueRef,
        types: &mut TypeStore,
        data: &crate::data_semantics::DataSemanticTable,
        enums: &crate::enum_semantics::EnumSemanticTable,
    ) -> Result<TypeId, StaticValueStoreError> {
        let value = self.get(reference)?.clone();
        Ok(match value {
            StaticValueData::Unit => types.unit(),
            StaticValueData::Bool(_) => self.builtin_type(types, phalcom_native_meta::UniverseKey::Bool)?,
            StaticValueData::Char(_) => self.builtin_type(types, phalcom_native_meta::UniverseKey::Char)?,
            StaticValueData::Int(_) => self.builtin_type(types, phalcom_native_meta::UniverseKey::Int)?,
            StaticValueData::Float(_) => self.builtin_type(types, phalcom_native_meta::UniverseKey::Float)?,
            StaticValueData::String(_) => self.builtin_type(types, phalcom_native_meta::UniverseKey::String)?,
            StaticValueData::Symbol(_) => self.builtin_type(types, phalcom_native_meta::UniverseKey::Symbol)?,
            StaticValueData::Type(ty) | StaticValueData::StaticTerm(ty) => {
                self.checked_kind(ty, types)?;
                ty
            }
            StaticValueData::Tuple(id) => {
                let children = self.get_tuple(id)?.to_vec();
                let elements = children
                    .iter()
                    .map(|child| self.static_value_type(*child, types, data, enums).map(TupleTypeElement::unlabeled))
                    .collect::<Result<Vec<_>, _>>()?;
                types.tuple(elements.into_boxed_slice())
            }
            StaticValueData::Sequence(id) => {
                let sequence = self.get_sequence(id)?;
                let kind = self.checked_kind(sequence.result_type, types)?;
                if kind != KindId::TYPE {
                    return Err(StaticValueStoreError::NonProperType {
                        ty: sequence.result_type,
                        kind,
                    });
                }
                sequence.result_type
            }
            StaticValueData::Record(id) => {
                let fields = self.get_record(id)?.to_vec();
                let fields = fields
                    .iter()
                    .map(|field| {
                        self.static_value_type(field.value, types, data, enums)
                            .map(|ty| RecordRowField::required(field.key.clone(), ty))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                types
                    .record_row_type_checked(fields, RecordRowTail::Closed)
                    .map_err(|_| StaticValueStoreError::InvalidRecordType)?
            }
            StaticValueData::Data(id) => {
                let node = self.get_data(id)?;
                let info = data.data_info(&node.owner).ok_or(StaticValueStoreError::InvalidDataIdentity)?;
                if info.owner != node.owner
                    || info.constructor.constructor != node.constructor
                    || info.components.len() != node.components.len()
                    || info
                        .components
                        .iter()
                        .zip(node.components.iter())
                        .any(|(expected, (actual, _))| expected.id != *actual)
                {
                    return Err(StaticValueStoreError::InvalidDataIdentity);
                }
                self.checked_kind(node.result_type, types)?;
                node.result_type
            }
            StaticValueData::Variant(id) => {
                let node = self.get_variant(id)?;
                let info = enums.variant_info(&node.variant).ok_or(StaticValueStoreError::InvalidVariantPayload)?;
                if info.id != node.variant
                    || info.fields.len() != node.fields.len()
                    || info.fields.iter().zip(node.fields.iter()).any(|(expected, (actual, _))| expected.id != *actual)
                {
                    return Err(StaticValueStoreError::InvalidVariantPayload);
                }
                self.checked_kind(node.result_type, types)?;
                node.result_type
            }
            StaticValueData::Pack(id) => {
                let pos = self.pack_positionals(id)?.to_vec();
                let lab = self.pack_labeled(id)?;
                let mut elements = Vec::with_capacity(pos.len() + lab.len());
                for p in pos {
                    let ty = self.static_value_type(p, types, data, enums)?;
                    elements.push(crate::types::store::PackTypeElement::unlabeled(ty));
                }
                for (sym_id, val) in lab {
                    let spelling: Box<str> = self.get_symbol(sym_id).ok_or(StaticValueStoreError::UnknownValue(sym_id.0))?.into();
                    let ty = self.static_value_type(val, types, data, enums)?;
                    elements.push(crate::types::store::PackTypeElement::labeled(spelling, ty));
                }
                types
                    .pack(elements.into_boxed_slice())
                    .map_err(|_| StaticValueStoreError::StructuralValueNeedsSemanticProducts)?
            }
        })
    }

    /// Interns a proper type after validating it against the supplied type store.
    pub fn intern_type(&mut self, ty: TypeId, types: &TypeStore) -> Result<StaticValueRef, StaticValueStoreError> {
        let kind = self.checked_kind(ty, types)?;
        if kind != KindId::TYPE {
            return Err(StaticValueStoreError::NonProperType { ty, kind });
        }
        Ok(self.intern(StaticValueData::Type(ty)))
    }

    /// Interns a non-proper C3 static term after validating its kind.
    pub fn intern_static_term(&mut self, ty: TypeId, types: &TypeStore) -> Result<StaticValueRef, StaticValueStoreError> {
        let kind = self.checked_kind(ty, types)?;
        if kind == KindId::TYPE {
            return Err(StaticValueStoreError::ProperTypeAsStaticTerm(ty));
        }
        Ok(self.intern(StaticValueData::StaticTerm(ty)))
    }

    /// Looks up a value, rejecting refs from another store lineage.
    pub fn get(&self, reference: StaticValueRef) -> Result<&StaticValueData, StaticValueStoreError> {
        self.check_store(reference)?;
        self.get_value(reference.value).ok_or(StaticValueStoreError::UnknownValue(reference.value.0))
    }

    /// Looks up value data by id.
    pub fn get_value(&self, id: StaticValueId) -> Option<&StaticValueData> {
        let index = id.0 as usize;
        if let Some(frozen) = &self.frozen {
            if index < frozen.values.len() {
                return Some(&frozen.values[index]);
            }
            self.values.get(index - frozen.values.len())
        } else {
            self.values.get(index)
        }
    }

    /// Looks up interned string contents by id.
    pub fn get_string(&self, id: StaticStringId) -> Option<&str> {
        let index = id.0 as usize;
        if let Some(frozen) = &self.frozen {
            if index < frozen.strings.len() {
                return Some(&frozen.strings[index]);
            }
            self.strings.get(index - frozen.strings.len()).map(|s| s.as_ref())
        } else {
            self.strings.get(index).map(|s| s.as_ref())
        }
    }

    /// Looks up interned symbol spelling by id.
    pub fn get_symbol(&self, id: StaticSymbolId) -> Option<&str> {
        let index = id.0 as usize;
        if let Some(frozen) = &self.frozen {
            if index < frozen.symbols.len() {
                return Some(&frozen.symbols[index]);
            }
            self.symbols.get(index - frozen.symbols.len()).map(|s| s.as_ref())
        } else {
            self.symbols.get(index).map(|s| s.as_ref())
        }
    }

    /// Converts a reduced static value into an owned canonical Record key.
    /// String and Symbol contents are copied out of store-local intern tables.
    pub fn field_key_of(&self, reference: StaticValueRef) -> Result<FieldKey, StaticValueStoreError> {
        match self.get(reference)? {
            StaticValueData::String(id) => self.get_string(*id).map(FieldKey::string).ok_or(StaticValueStoreError::UnknownValue(id.0)),
            StaticValueData::Symbol(id) => self.get_symbol(*id).map(FieldKey::symbol).ok_or(StaticValueStoreError::UnknownValue(id.0)),
            StaticValueData::Int(value) => Ok(FieldKey::int(value.clone())),
            StaticValueData::Bool(value) => Ok(FieldKey::bool(*value)),
            _ => Err(StaticValueStoreError::NotAFieldKey),
        }
    }

    /// Returns a value's semantic domain using the paired canonical type store.
    pub fn domain_of(&self, reference: StaticValueRef, types: &TypeStore) -> Result<StaticValueDomain, StaticValueStoreError> {
        let domain = match self.get(reference)? {
            StaticValueData::Unit => StaticValueDomain::ValueType(types.unit()),
            StaticValueData::Bool(_) => StaticValueDomain::ValueType(self.builtin_type(types, phalcom_native_meta::UniverseKey::Bool)?),
            StaticValueData::Char(_) => StaticValueDomain::ValueType(self.builtin_type(types, phalcom_native_meta::UniverseKey::Char)?),
            StaticValueData::Int(_) => StaticValueDomain::ValueType(self.builtin_type(types, phalcom_native_meta::UniverseKey::Int)?),
            StaticValueData::Float(_) => StaticValueDomain::ValueType(self.builtin_type(types, phalcom_native_meta::UniverseKey::Float)?),
            StaticValueData::String(_) => StaticValueDomain::ValueType(self.builtin_type(types, phalcom_native_meta::UniverseKey::String)?),
            StaticValueData::Symbol(_) => StaticValueDomain::ValueType(self.builtin_type(types, phalcom_native_meta::UniverseKey::Symbol)?),
            StaticValueData::Type(ty) => {
                self.checked_kind(*ty, types)?;
                StaticValueDomain::Kind(KindId::TYPE)
            }
            StaticValueData::StaticTerm(ty) => StaticValueDomain::Kind(self.checked_kind(*ty, types)?),
            StaticValueData::Tuple(_)
            | StaticValueData::Sequence(_)
            | StaticValueData::Record(_)
            | StaticValueData::Data(_)
            | StaticValueData::Variant(_)
            | StaticValueData::Pack(_) => {
                return Err(StaticValueStoreError::StructuralValueNeedsSemanticProducts);
            }
        };
        Ok(domain)
    }

    /// Compares values by Telos value equality, separate from representation identity.
    pub fn semantic_eq(&self, left: StaticValueRef, right: StaticValueRef) -> Result<bool, StaticValueStoreError> {
        let left = self.get(left)?;
        let right = self.get(right)?;
        Ok(match (left, right) {
            (StaticValueData::Unit, StaticValueData::Unit) => true,
            (StaticValueData::Bool(a), StaticValueData::Bool(b)) => a == b,
            (StaticValueData::Char(a), StaticValueData::Char(b)) => a == b,
            (StaticValueData::Int(a), StaticValueData::Int(b)) => a.to_bigint() == b.to_bigint(),
            (StaticValueData::Float(a), StaticValueData::Float(b)) => static_float_eq(a, b),
            (StaticValueData::String(a), StaticValueData::String(b)) => self.get_string(*a) == self.get_string(*b),
            (StaticValueData::Symbol(a), StaticValueData::Symbol(b)) => self.get_symbol(*a) == self.get_symbol(*b),
            (StaticValueData::Type(a), StaticValueData::Type(b)) | (StaticValueData::StaticTerm(a), StaticValueData::StaticTerm(b)) => a == b,
            (StaticValueData::Tuple(a), StaticValueData::Tuple(b)) => {
                let left = self.get_tuple(*a)?;
                let right = self.get_tuple(*b)?;
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right.iter())
                        .try_fold(true, |same, (a, b)| Ok::<_, StaticValueStoreError>(same && self.semantic_eq(*a, *b)?))?
            }
            (StaticValueData::Sequence(a), StaticValueData::Sequence(b)) => {
                let left = self.get_sequence(*a)?;
                let right = self.get_sequence(*b)?;
                left.result_type == right.result_type
                    && left.elements.len() == right.elements.len()
                    && left
                        .elements
                        .iter()
                        .zip(right.elements.iter())
                        .try_fold(true, |same, (a, b)| Ok::<_, StaticValueStoreError>(same && self.semantic_eq(*a, *b)?))?
            }
            (StaticValueData::Record(a), StaticValueData::Record(b)) => {
                let left = self.get_record(*a)?;
                let right = self.get_record(*b)?;
                left.len() == right.len()
                    && left.iter().zip(right.iter()).try_fold(true, |same, (a, b)| {
                        Ok::<_, StaticValueStoreError>(same && a.key == b.key && self.semantic_eq(a.value, b.value)?)
                    })?
            }
            (StaticValueData::Data(a), StaticValueData::Data(b)) => {
                let a = self.get_data(*a)?;
                let b = self.get_data(*b)?;
                a.owner == b.owner
                    && a.constructor == b.constructor
                    && a.result_type == b.result_type
                    && a.components.len() == b.components.len()
                    && a.components.iter().zip(b.components.iter()).try_fold(true, |same, ((aid, av), (bid, bv))| {
                        Ok::<_, StaticValueStoreError>(same && aid == bid && self.semantic_eq(*av, *bv)?)
                    })?
            }
            (StaticValueData::Variant(a), StaticValueData::Variant(b)) => {
                let a = self.get_variant(*a)?;
                let b = self.get_variant(*b)?;
                a.variant == b.variant
                    && a.result_type == b.result_type
                    && a.fields.len() == b.fields.len()
                    && a.fields.iter().zip(b.fields.iter()).try_fold(true, |same, ((aid, av), (bid, bv))| {
                        Ok::<_, StaticValueStoreError>(same && aid == bid && self.semantic_eq(*av, *bv)?)
                    })?
            }
            (StaticValueData::Pack(a), StaticValueData::Pack(b)) => {
                let pos_a = self.pack_positionals(*a)?;
                let pos_b = self.pack_positionals(*b)?;
                if pos_a.len() != pos_b.len() {
                    return Ok(false);
                }
                for (&va, &vb) in pos_a.iter().zip(pos_b.iter()) {
                    if !self.semantic_eq(va, vb)? {
                        return Ok(false);
                    }
                }
                let lab_a = self.pack_labeled(*a)?;
                let lab_b = self.pack_labeled(*b)?;
                if lab_a.len() != lab_b.len() {
                    return Ok(false);
                }
                for ((sa, va), (sb, vb)) in lab_a.into_iter().zip(lab_b) {
                    let text_a = self.get_symbol(sa).ok_or(StaticValueStoreError::UnknownValue(sa.0))?;
                    let text_b = self.get_symbol(sb).ok_or(StaticValueStoreError::UnknownValue(sb.0))?;
                    if text_a != text_b || !self.semantic_eq(va, vb)? {
                        return Ok(false);
                    }
                }
                true
            }
            _ => false,
        })
    }

    /// Returns a deterministic content fingerprint for this primitive value.
    pub fn content_fingerprint(&self, reference: StaticValueRef) -> Result<u64, StaticValueStoreError> {
        let value = self.get(reference)?;
        let mut hash = StableFnv64::new();
        match value {
            StaticValueData::Unit => hash.byte(0),
            StaticValueData::Bool(value) => {
                hash.byte(1);
                hash.byte(u8::from(*value));
            }
            StaticValueData::Char(value) => {
                hash.byte(13);
                hash.u32(*value as u32);
            }
            StaticValueData::Int(value) => {
                hash.byte(2);
                hash.bytes(&value.to_bigint().to_signed_bytes_le());
            }
            StaticValueData::Float(value) => {
                hash.byte(3);
                hash.u64(value.bits());
            }
            StaticValueData::String(id) => {
                hash.byte(4);
                let s = self.get_string(*id).ok_or(StaticValueStoreError::UnknownValue(id.0))?;
                hash.bytes(s.as_bytes());
            }
            StaticValueData::Symbol(id) => {
                hash.byte(5);
                let s = self.get_symbol(*id).ok_or(StaticValueStoreError::UnknownValue(id.0))?;
                hash.bytes(s.as_bytes());
            }
            StaticValueData::Type(ty) => {
                hash.byte(6);
                hash.u32(ty.0);
            }
            StaticValueData::StaticTerm(ty) => {
                hash.byte(7);
                hash.u32(ty.0);
            }
            StaticValueData::Tuple(id) => {
                hash.byte(8);
                let children = self.get_tuple(*id)?;
                hash.u64(children.len() as u64);
                for child in children.iter() {
                    hash.u64(self.content_fingerprint(*child)?);
                }
            }
            StaticValueData::Sequence(id) => {
                hash.byte(14);
                let sequence = self.get_sequence(*id)?;
                hash.u32(sequence.result_type.0);
                hash.u64(sequence.elements.len() as u64);
                for child in sequence.elements.iter() {
                    hash.u64(self.content_fingerprint(*child)?);
                }
            }
            StaticValueData::Record(id) => {
                hash.byte(9);
                let fields = self.get_record(*id)?;
                hash.u64(fields.len() as u64);
                for field in fields.iter() {
                    hash.bytes(&field.key.stable_fingerprint_bytes());
                    hash.u64(self.content_fingerprint(field.value)?);
                }
            }
            StaticValueData::Data(id) => {
                hash.byte(10);
                let node = self.get_data(*id)?;
                node.owner.hash(&mut hash);
                node.constructor.hash(&mut hash);
                hash.u32(node.result_type.0);
                for (component, value) in node.components.iter() {
                    component.hash(&mut hash);
                    hash.u64(self.content_fingerprint(*value)?);
                }
            }
            StaticValueData::Variant(id) => {
                hash.byte(11);
                let node = self.get_variant(*id)?;
                node.variant.hash(&mut hash);
                hash.u32(node.result_type.0);
                for (field, value) in node.fields.iter() {
                    field.hash(&mut hash);
                    hash.u64(self.content_fingerprint(*value)?);
                }
            }
            StaticValueData::Pack(id) => {
                hash.byte(12);
                let pos = self.pack_positionals(*id)?;
                hash.u64(pos.len() as u64);
                let lab = self.pack_labeled(*id)?;
                hash.u64(lab.len() as u64);
                for &child in pos {
                    hash.u64(self.content_fingerprint(child)?);
                }
                for (sym_id, child) in lab {
                    let spelling = self.get_symbol(sym_id).ok_or(StaticValueStoreError::UnknownValue(sym_id.0))?;
                    hash.bytes(spelling.as_bytes());
                    hash.u64(self.content_fingerprint(child)?);
                }
            }
        }
        Ok(hash.finish())
    }

    /// Hashes a value by Telos equality, so semantically equal values have equal hashes.
    pub fn semantic_hash(&self, reference: StaticValueRef) -> Result<u64, StaticValueStoreError> {
        let value = self.get(reference)?;
        let mut hash = StableFnv64::new();
        match value {
            StaticValueData::Unit => hash.byte(0),
            StaticValueData::Bool(value) => {
                hash.byte(1);
                hash.byte(u8::from(*value));
            }
            StaticValueData::Char(value) => {
                hash.byte(13);
                hash.u32(*value as u32);
            }
            StaticValueData::Int(value) => {
                hash.byte(2);
                hash.bytes(&value.to_bigint().to_signed_bytes_le());
            }
            StaticValueData::Float(value) => {
                hash.byte(3);
                hash.u64(static_float_hash(value));
            }
            StaticValueData::String(id) => {
                hash.byte(4);
                let s = self.get_string(*id).ok_or(StaticValueStoreError::UnknownValue(id.0))?;
                hash.bytes(s.as_bytes());
            }
            StaticValueData::Symbol(id) => {
                hash.byte(5);
                let s = self.get_symbol(*id).ok_or(StaticValueStoreError::UnknownValue(id.0))?;
                hash.bytes(s.as_bytes());
            }
            StaticValueData::Type(ty) => {
                hash.byte(6);
                hash.u32(ty.0);
            }
            StaticValueData::StaticTerm(ty) => {
                hash.byte(7);
                hash.u32(ty.0);
            }
            StaticValueData::Tuple(id) => {
                hash.byte(8);
                let children = self.get_tuple(*id)?;
                hash.u64(children.len() as u64);
                for child in children.iter() {
                    hash.u64(self.semantic_hash(*child)?);
                }
            }
            StaticValueData::Sequence(id) => {
                hash.byte(14);
                let sequence = self.get_sequence(*id)?;
                hash.u32(sequence.result_type.0);
                hash.u64(sequence.elements.len() as u64);
                for child in sequence.elements.iter() {
                    hash.u64(self.semantic_hash(*child)?);
                }
            }
            StaticValueData::Record(id) => {
                hash.byte(9);
                let fields = self.get_record(*id)?;
                hash.u64(fields.len() as u64);
                for field in fields.iter() {
                    hash.bytes(&field.key.stable_fingerprint_bytes());
                    hash.u64(self.semantic_hash(field.value)?);
                }
            }
            StaticValueData::Data(id) => {
                hash.byte(10);
                let node = self.get_data(*id)?;
                node.owner.hash(&mut hash);
                node.constructor.hash(&mut hash);
                hash.u32(node.result_type.0);
                for (component, value) in node.components.iter() {
                    component.hash(&mut hash);
                    hash.u64(self.semantic_hash(*value)?);
                }
            }
            StaticValueData::Variant(id) => {
                hash.byte(11);
                let node = self.get_variant(*id)?;
                node.variant.hash(&mut hash);
                hash.u32(node.result_type.0);
                for (field, value) in node.fields.iter() {
                    field.hash(&mut hash);
                    hash.u64(self.semantic_hash(*value)?);
                }
            }
            StaticValueData::Pack(id) => {
                hash.byte(12);
                let pos = self.pack_positionals(*id)?;
                hash.u64(pos.len() as u64);
                let lab = self.pack_labeled(*id)?;
                hash.u64(lab.len() as u64);
                for &child in pos {
                    hash.u64(self.semantic_hash(child)?);
                }
                for (sym_id, child) in lab {
                    let spelling = self.get_symbol(sym_id).ok_or(StaticValueStoreError::UnknownValue(sym_id.0))?;
                    hash.bytes(spelling.as_bytes());
                    hash.u64(self.semantic_hash(child)?);
                }
            }
        }
        Ok(hash.finish())
    }

    /// Checks a static value's derived ordinary type with the canonical relation engine.
    pub fn check_value_type(
        &mut self,
        reference: StaticValueRef,
        expected: TypeId,
        types: &mut TypeStore,
        data: &crate::data_semantics::DataSemanticTable,
        enums: &crate::enum_semantics::EnumSemanticTable,
        hierarchy: &dyn TypeHierarchy,
    ) -> Result<Assignability, StaticValueStoreError> {
        let actual = self.static_value_type(reference, types, data, enums)?;
        Ok(crate::types::relation::check_assignability(
            types,
            hierarchy,
            &TypeKnowledge::established(actual, EvidenceOrigin::ContextualDerivation),
            &TypeKnowledge::established(expected, EvidenceOrigin::DeclarationSemantics),
        ))
    }

    fn intern(&mut self, data: StaticValueData) -> StaticValueRef {
        let id = if let Some(&id) = self.interner.get(&data) {
            id
        } else if let Some(id) = self.frozen.as_ref().and_then(|f| f.find_value(&data)) {
            id
        } else {
            let frozen_len = self.frozen.as_ref().map_or(0, |f| f.values.len());
            let id = StaticValueId((frozen_len + self.values.len()) as u32);
            self.values.push_back(data.clone());
            self.interner.insert(data, id);
            id
        };
        StaticValueRef { store: self.id, value: id }
    }

    fn check_store(&self, reference: StaticValueRef) -> Result<(), StaticValueStoreError> {
        if reference.store == self.id {
            return Ok(());
        }
        if let Some(frozen) = &self.frozen {
            if reference.store == frozen.id && (reference.value.0 as usize) < frozen.values.len() {
                return Ok(());
            }
        }
        Err(StaticValueStoreError::StoreMismatch {
            expected: self.id,
            actual: reference.store,
        })
    }

    fn check_children(&self, children: impl IntoIterator<Item = StaticValueRef>) -> Result<(), StaticValueStoreError> {
        for child in children {
            self.get(child)?;
        }
        Ok(())
    }

    fn checked_kind(&self, ty: TypeId, types: &TypeStore) -> Result<KindId, StaticValueStoreError> {
        if ty.index() >= types.len() {
            return Err(StaticValueStoreError::UnknownType(ty));
        }
        Ok(types.kind_of(ty))
    }

    fn builtin_type(&self, types: &TypeStore, key: phalcom_native_meta::UniverseKey) -> Result<TypeId, StaticValueStoreError> {
        let declaration = crate::core_surface::universe_declaration(key);
        let ty = types.nominal_type_id(&declaration).ok_or(StaticValueStoreError::UnknownType(TypeId::DUMMY))?;
        self.checked_kind(ty, types)?;
        Ok(ty)
    }
}

/// Telos Float equality: NaN is unequal to all values and both zeros compare equal.
pub fn static_float_eq(left: &StaticFloat, right: &StaticFloat) -> bool {
    left.to_f64() == right.to_f64()
}

/// Coherent Telos Float hash that normalizes signed zero and NaN payloads.
pub fn static_float_hash(value: &StaticFloat) -> u64 {
    let float = value.to_f64();
    if float == 0.0 {
        0
    } else if float.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        float.to_bits()
    }
}

#[derive(Clone, Copy)]
struct StableFnv64(u64);

impl StableFnv64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    const fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn byte(&mut self, byte: u8) {
        self.0 ^= u64::from(byte);
        self.0 = self.0.wrapping_mul(Self::PRIME);
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.u64(bytes.len() as u64);
        for byte in bytes {
            self.byte(*byte);
        }
    }

    fn u32(&mut self, value: u32) {
        for byte in value.to_le_bytes() {
            self.byte(byte);
        }
    }

    fn u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.byte(byte);
        }
    }
}

impl Hasher for StableFnv64 {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        self.bytes(bytes);
    }
}

#[cfg(test)]
mod persistent_root_tests {
    use super::StaticValueStore;
    use std::ptr;

    #[test]
    fn static_value_store_clone_shares_arenas_and_candidate_appends_are_isolated() {
        let mut committed = StaticValueStore::new();
        for value in 0..96 {
            committed.intern_int(value);
        }
        for index in 0..96 {
            committed.intern_string(format!("retained{index}"));
        }
        let committed_id = committed.id();
        let committed_len = committed.len();
        let first_value_address = &committed.values[0] as *const _;
        let mut candidate = committed.clone();

        assert!(committed.values.ptr_eq(&candidate.values));
        assert!(committed.interner.ptr_eq(&candidate.interner));
        let candidate_value = candidate.intern_int(10_000);

        assert_eq!(candidate.id(), committed_id);
        assert_eq!(committed.len(), committed_len);
        assert_eq!(candidate.len(), committed_len + 1);
        assert!(ptr::eq(first_value_address, &candidate.values[0] as *const _));
        assert!(committed.strings.ptr_eq(&candidate.strings));
        assert!(committed.string_ids.ptr_eq(&candidate.string_ids));
        assert!(candidate.get(candidate_value).is_ok());
        assert!(committed.get(candidate_value).is_err());
    }
}
