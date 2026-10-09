//! Snapshot-safe semantic values used by compile-time evaluation.

mod store;

pub use store::{
    StaticDataValueId, StaticFloat, StaticInt, StaticPackBacking, StaticPackBackingId, StaticPackData, StaticPackId, StaticRecordId,
    StaticSequenceData, StaticSequenceFamily, StaticSequenceId, StaticStringId, StaticSymbolId, StaticTupleId, StaticValueData, StaticValueId, StaticValueRef,
    StaticValueStore, StaticValueStoreError, StaticValueStoreId, StaticVariantValueId, static_float_eq, static_float_hash,
};
