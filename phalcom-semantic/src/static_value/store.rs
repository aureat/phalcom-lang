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
use num_bigint::BigInt;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

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
pub struct StaticValueId(u32);

impl StaticValueId {
    /// Returns this value's store-local index.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Reference that remains meaningful across clones of one static-value store.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticValueRef {
    store: StaticValueStoreId,
    value: StaticValueId,
}

impl StaticValueRef {
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
pub struct StaticStringId(u32);

impl StaticStringId {
    /// Returns the store-local string index.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Store-local identity for an interned symbol spelling.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticSymbolId(u32);

impl StaticSymbolId {
    /// Returns the store-local symbol index.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

macro_rules! structural_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u32);
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StaticPackBacking {
    pub positionals: Box<[StaticValueRef]>,
    pub labels: Box<[StaticSymbolId]>,
    pub labeled_values: Box<[StaticValueRef]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StaticPackData {
    pub backing: StaticPackBackingId,
    pub positional_start: u32,
    pub positional_len: u32,
    pub labeled_start: u32,
    pub labeled_len: u32,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StaticRecordField {
    pub key: FieldKey,
    pub value: StaticValueRef,
}

/// Canonical family retained from a checked homogeneous sequence result type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StaticSequenceFamily {
    Array,
    Vector,
}

/// Immutable homogeneous sequence value with its already-checked ordinary type.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StaticSequenceData {
    family: StaticSequenceFamily,
    result_type: TypeId,
    elements: Box<[StaticValueRef]>,
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StaticDataValue {
    pub owner: crate::identity::DeclarationId,
    pub constructor: DataConstructorId,
    pub result_type: TypeId,
    pub components: Box<[(DataComponentId, StaticValueRef)]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StaticVariantValue {
    pub variant: VariantId,
    pub result_type: TypeId,
    pub fields: Box<[(crate::identity::VariantFieldId, StaticValueRef)]>,
}

/// IEEE-754 binary64 representation retained exactly for materialization.
///
/// Equality and hashing on this wrapper describe representation identity.
/// Use [`static_float_eq`] and [`static_float_hash`] for Telos Float semantics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

/// Append-only canonical arena for semantic compile-time values.
///
/// Cloning the store preserves its identity and all existing indices, matching
/// the `TypeStore` clone-on-write snapshot model.
#[derive(Clone, Debug)]
pub struct StaticValueStore {
    id: StaticValueStoreId,
    values: Vec<StaticValueData>,
    interner: HashMap<StaticValueData, StaticValueId>,
    strings: Vec<Box<str>>,
    string_ids: HashMap<Box<str>, StaticStringId>,
    symbols: Vec<Box<str>>,
    symbol_ids: HashMap<Box<str>, StaticSymbolId>,
    tuples: Vec<Box<[StaticValueRef]>>,
    tuple_ids: HashMap<Box<[StaticValueRef]>, StaticTupleId>,
    sequences: Vec<StaticSequenceData>,
    sequence_ids: HashMap<StaticSequenceData, StaticSequenceId>,
    records: Vec<Box<[StaticRecordField]>>,
    record_ids: HashMap<Box<[StaticRecordField]>, StaticRecordId>,
    data_values: Vec<StaticDataValue>,
    data_value_ids: HashMap<StaticDataValue, StaticDataValueId>,
    variant_values: Vec<StaticVariantValue>,
    variant_value_ids: HashMap<StaticVariantValue, StaticVariantValueId>,
    pack_backings: Vec<StaticPackBacking>,
    pack_backing_ids: HashMap<StaticPackBacking, StaticPackBackingId>,
    packs: Vec<StaticPackData>,
    pack_ids: HashMap<StaticPackData, StaticPackId>,
}

impl Default for StaticValueStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticValueStore {
    /// Creates a fresh store lineage.
    pub fn new() -> Self {
        Self {
            id: StaticValueStoreId(NEXT_STATIC_VALUE_STORE_ID.fetch_add(1, Ordering::Relaxed)),
            values: Vec::new(),
            interner: HashMap::new(),
            strings: Vec::new(),
            string_ids: HashMap::new(),
            symbols: Vec::new(),
            symbol_ids: HashMap::new(),
            tuples: Vec::new(),
            tuple_ids: HashMap::new(),
            sequences: Vec::new(),
            sequence_ids: HashMap::new(),
            records: Vec::new(),
            record_ids: HashMap::new(),
            data_values: Vec::new(),
            data_value_ids: HashMap::new(),
            variant_values: Vec::new(),
            variant_value_ids: HashMap::new(),
            pack_backings: Vec::new(),
            pack_backing_ids: HashMap::new(),
            packs: Vec::new(),
            pack_ids: HashMap::new(),
        }
    }

    /// Returns this store's lineage identity.
    pub const fn id(&self) -> StaticValueStoreId {
        self.id
    }

    /// Returns the number of interned value nodes.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether this store has no interned value nodes.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
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
        let id = if let Some(id) = self.string_ids.get(text) {
            *id
        } else {
            let id = StaticStringId(self.strings.len() as u32);
            let text: Box<str> = text.into();
            self.strings.push(text.clone());
            self.string_ids.insert(text, id);
            id
        };
        self.intern(StaticValueData::String(id))
    }

    /// Interns a symbol spelling and returns its store-local `StaticSymbolId`.
    pub fn intern_symbol_id(&mut self, spelling: impl AsRef<str>) -> StaticSymbolId {
        let text = spelling.as_ref();
        if let Some(id) = self.symbol_ids.get(text) {
            *id
        } else {
            let id = StaticSymbolId(self.symbols.len() as u32);
            let text: Box<str> = text.into();
            self.symbols.push(text.clone());
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
        let id = if let Some(id) = self.tuple_ids.get(elements.as_ref()) {
            *id
        } else {
            let id = StaticTupleId(self.tuples.len() as u32);
            self.tuples.push(elements.clone());
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
    pub fn intern_sequence(
        &mut self,
        result_type: TypeId,
        elements: Vec<StaticValueRef>,
        types: &TypeStore,
    ) -> Result<StaticValueRef, StaticValueStoreError> {
        self.check_children(elements.iter().copied())?;
        let result_kind = self.checked_kind(result_type, types)?;
        if result_kind != KindId::TYPE {
            return Err(StaticValueStoreError::NonProperType { ty: result_type, kind: result_kind });
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
        let id = if let Some(id) = self.sequence_ids.get(&node) {
            *id
        } else {
            let id = StaticSequenceId(self.sequences.len() as u32);
            self.sequences.push(node.clone());
            self.sequence_ids.insert(node, id);
            id
        };
        Ok(self.intern(StaticValueData::Sequence(id)))
    }

    /// Looks up immutable homogeneous sequence data.
    pub fn get_sequence(&self, id: StaticSequenceId) -> Result<&StaticSequenceData, StaticValueStoreError> {
        self.sequences.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))
    }

    /// Looks up the ordered children of an immutable positional tuple.
    pub fn get_tuple(&self, id: StaticTupleId) -> Result<&[StaticValueRef], StaticValueStoreError> {
        self.tuples
            .get(id.0 as usize)
            .map(Box::as_ref)
            .ok_or(StaticValueStoreError::UnknownValue(id.0))
    }

    /// Looks up the ordered fields of an immutable record.
    pub fn get_record(&self, id: StaticRecordId) -> Result<&[StaticRecordField], StaticValueStoreError> {
        self.records
            .get(id.0 as usize)
            .map(Box::as_ref)
            .ok_or(StaticValueStoreError::UnknownValue(id.0))
    }

    /// Looks up the components and metadata of an interned data value.
    pub fn get_data(&self, id: StaticDataValueId) -> Result<&StaticDataValue, StaticValueStoreError> {
        self.data_values.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))
    }

    /// Looks up the fields and metadata of an interned variant value.
    pub fn get_variant(&self, id: StaticVariantValueId) -> Result<&StaticVariantValue, StaticValueStoreError> {
        self.variant_values.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))
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
        let id = if let Some(id) = self.record_ids.get(fields.as_ref()) {
            *id
        } else {
            let id = StaticRecordId(self.records.len() as u32);
            self.records.push(fields.clone());
            self.record_ids.insert(fields, id);
            id
        };
        Ok(self.intern(StaticValueData::Record(id)))
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

        let backing_id = if let Some(id) = self.pack_backing_ids.get(&backing_node) {
            *id
        } else {
            let id = StaticPackBackingId(self.pack_backings.len() as u32);
            self.pack_backings.push(backing_node.clone());
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

        let pack_id = if let Some(id) = self.pack_ids.get(&pack_data) {
            *id
        } else {
            let id = StaticPackId(self.packs.len() as u32);
            self.packs.push(pack_data);
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
        let pack_data = *self.packs.get(pack_id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(pack_id.0))?;

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

        let id = if let Some(id) = self.pack_ids.get(&new_pack_data) {
            *id
        } else {
            let id = StaticPackId(self.packs.len() as u32);
            self.packs.push(new_pack_data);
            self.pack_ids.insert(new_pack_data, id);
            id
        };

        Ok(self.intern(StaticValueData::Pack(id)))
    }

    /// Looks up a pack's slice descriptor and backing.
    pub fn get_pack(&self, id: StaticPackId) -> Result<(&StaticPackData, &StaticPackBacking), StaticValueStoreError> {
        let pack_data = self.packs.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?;
        let backing = self
            .pack_backings
            .get(pack_data.backing.0 as usize)
            .ok_or(StaticValueStoreError::UnknownValue(pack_data.backing.0))?;
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
        let id = if let Some(id) = self.data_value_ids.get(&node) {
            *id
        } else {
            let id = StaticDataValueId(self.data_values.len() as u32);
            self.data_values.push(node.clone());
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
        let id = if let Some(id) = self.variant_value_ids.get(&node) {
            *id
        } else {
            let id = StaticVariantValueId(self.variant_values.len() as u32);
            self.variant_values.push(node.clone());
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
                let children = self.tuples.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?.clone();
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
                    return Err(StaticValueStoreError::NonProperType { ty: sequence.result_type, kind });
                }
                sequence.result_type
            }
            StaticValueData::Record(id) => {
                let fields = self.records.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?.clone();
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
                let node = self.data_values.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?;
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
                let node = self.variant_values.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?;
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
        self.values
            .get(reference.value.0 as usize)
            .ok_or(StaticValueStoreError::UnknownValue(reference.value.0))
    }

    /// Looks up interned string contents by id.
    pub fn get_string(&self, id: StaticStringId) -> Option<&str> {
        self.strings.get(id.0 as usize).map(|s| s.as_ref())
    }

    /// Looks up interned symbol spelling by id.
    pub fn get_symbol(&self, id: StaticSymbolId) -> Option<&str> {
        self.symbols.get(id.0 as usize).map(|s| s.as_ref())
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
            (StaticValueData::String(a), StaticValueData::String(b)) => self.strings.get(a.0 as usize) == self.strings.get(b.0 as usize),
            (StaticValueData::Symbol(a), StaticValueData::Symbol(b)) => self.symbols.get(a.0 as usize) == self.symbols.get(b.0 as usize),
            (StaticValueData::Type(a), StaticValueData::Type(b)) | (StaticValueData::StaticTerm(a), StaticValueData::StaticTerm(b)) => a == b,
            (StaticValueData::Tuple(a), StaticValueData::Tuple(b)) => {
                let left = self.tuples.get(a.0 as usize).ok_or(StaticValueStoreError::UnknownValue(a.0))?;
                let right = self.tuples.get(b.0 as usize).ok_or(StaticValueStoreError::UnknownValue(b.0))?;
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
                let left = self.records.get(a.0 as usize).ok_or(StaticValueStoreError::UnknownValue(a.0))?;
                let right = self.records.get(b.0 as usize).ok_or(StaticValueStoreError::UnknownValue(b.0))?;
                left.len() == right.len()
                    && left.iter().zip(right.iter()).try_fold(true, |same, (a, b)| {
                        Ok::<_, StaticValueStoreError>(same && a.key == b.key && self.semantic_eq(a.value, b.value)?)
                    })?
            }
            (StaticValueData::Data(a), StaticValueData::Data(b)) => {
                let a = self.data_values.get(a.0 as usize).ok_or(StaticValueStoreError::UnknownValue(a.0))?;
                let b = self.data_values.get(b.0 as usize).ok_or(StaticValueStoreError::UnknownValue(b.0))?;
                a.owner == b.owner
                    && a.constructor == b.constructor
                    && a.result_type == b.result_type
                    && a.components.len() == b.components.len()
                    && a.components.iter().zip(b.components.iter()).try_fold(true, |same, ((aid, av), (bid, bv))| {
                        Ok::<_, StaticValueStoreError>(same && aid == bid && self.semantic_eq(*av, *bv)?)
                    })?
            }
            (StaticValueData::Variant(a), StaticValueData::Variant(b)) => {
                let a = self.variant_values.get(a.0 as usize).ok_or(StaticValueStoreError::UnknownValue(a.0))?;
                let b = self.variant_values.get(b.0 as usize).ok_or(StaticValueStoreError::UnknownValue(b.0))?;
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
                hash.bytes(self.strings[id.0 as usize].as_bytes());
            }
            StaticValueData::Symbol(id) => {
                hash.byte(5);
                hash.bytes(self.symbols[id.0 as usize].as_bytes());
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
                let children = self.tuples.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?.clone();
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
                let fields = self.records.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?.clone();
                hash.u64(fields.len() as u64);
                for field in fields.iter() {
                    hash.bytes(&field.key.stable_fingerprint_bytes());
                    hash.u64(self.content_fingerprint(field.value)?);
                }
            }
            StaticValueData::Data(id) => {
                hash.byte(10);
                let node = self.data_values.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?.clone();
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
                let node = self.variant_values.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?.clone();
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
                hash.bytes(self.strings[id.0 as usize].as_bytes());
            }
            StaticValueData::Symbol(id) => {
                hash.byte(5);
                hash.bytes(self.symbols[id.0 as usize].as_bytes());
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
                let children = self.tuples.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?.clone();
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
                let fields = self.records.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?.clone();
                hash.u64(fields.len() as u64);
                for field in fields.iter() {
                    hash.bytes(&field.key.stable_fingerprint_bytes());
                    hash.u64(self.semantic_hash(field.value)?);
                }
            }
            StaticValueData::Data(id) => {
                hash.byte(10);
                let node = self.data_values.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?.clone();
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
                let node = self.variant_values.get(id.0 as usize).ok_or(StaticValueStoreError::UnknownValue(id.0))?.clone();
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
        let id = if let Some(id) = self.interner.get(&data) {
            *id
        } else {
            let id = StaticValueId(self.values.len() as u32);
            self.values.push(data.clone());
            self.interner.insert(data, id);
            id
        };
        StaticValueRef { store: self.id, value: id }
    }

    fn check_store(&self, reference: StaticValueRef) -> Result<(), StaticValueStoreError> {
        if reference.store != self.id {
            return Err(StaticValueStoreError::StoreMismatch {
                expected: self.id,
                actual: reference.store,
            });
        }
        Ok(())
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
