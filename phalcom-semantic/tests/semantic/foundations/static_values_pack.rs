use num_bigint::BigInt;
use phalcom_semantic::data_semantics::DataSemanticTable;
use phalcom_semantic::enum_semantics::EnumSemanticTable;
use phalcom_semantic::materialization::{DeferredStructuralKind, MaterializationCapability, MaterializationDescriptor};
use phalcom_semantic::static_value::{StaticValueData, StaticValueStore, StaticValueStoreError};
use phalcom_semantic::types::TypeStore;

#[test]
fn static_pack_storage_slicing_and_equality() {
    let mut types = TypeStore::new();
    let mut values = StaticValueStore::new();

    let int_one = values.intern_int(BigInt::from(1));
    let int_two = values.intern_int(BigInt::from(2));
    let int_three = values.intern_int(BigInt::from(3));
    let str_a = values.intern_string("alpha");
    let str_b = values.intern_string("beta");

    let sym_x = values.intern_symbol_id("x");
    let sym_y = values.intern_symbol_id("y");

    // 1. intern_pack with positional and labeled elements
    let pack1 = values
        .intern_pack(
            vec![int_one, int_two, int_three],
            vec![(sym_x, str_a), (sym_y, str_b)],
        )
        .expect("intern valid pack");

    let StaticValueData::Pack(pack1_id) = values.get(pack1).unwrap() else {
        panic!("expected Pack");
    };

    let (pack1_data, _) = values.get_pack(*pack1_id).unwrap();
    let backing_id = pack1_data.backing;
    assert_eq!(pack1_data.positional_len, 3);
    assert_eq!(pack1_data.labeled_len, 2);

    // 2. Slice creation shares backing (O(1) tail)
    let slice1 = values
        .intern_pack_slice(pack1, 1, 2, 0, 1)
        .expect("valid slice");
    let StaticValueData::Pack(slice1_id) = values.get(slice1).unwrap() else {
        panic!("expected Pack slice");
    };
    let (slice1_data, _) = values.get_pack(*slice1_id).unwrap();
    assert_eq!(slice1_data.backing, backing_id, "slice shares the same backing");
    assert_eq!(slice1_data.positional_start, 1);
    assert_eq!(slice1_data.positional_len, 2);
    assert_eq!(slice1_data.labeled_start, 0);
    assert_eq!(slice1_data.labeled_len, 1);

    // Slice-of-slice (rest chain) also shares backing
    let slice2 = values
        .intern_pack_slice(slice1, 1, 1, 0, 0)
        .expect("valid sub-slice");
    let StaticValueData::Pack(slice2_id) = values.get(slice2).unwrap() else {
        panic!("expected Pack sub-slice");
    };
    let (slice2_data, _) = values.get_pack(*slice2_id).unwrap();
    assert_eq!(slice2_data.backing, backing_id, "sub-slice still shares the original backing");
    assert_eq!(slice2_data.positional_start, 2); // 1 + 1
    assert_eq!(slice2_data.positional_len, 1);
    assert_eq!(slice2_data.labeled_len, 0);

    // 3. Invalid slice bounds rejected
    let invalid_slice = values.intern_pack_slice(pack1, 0, 4, 0, 2);
    assert_eq!(invalid_slice, Err(StaticValueStoreError::InvalidPackSliceRange));

    // 4. Duplicate label in intern_pack rejected
    let dup_label = values.intern_pack(
        vec![int_one],
        vec![(sym_x, str_a), (sym_x, str_b)],
    );
    assert_eq!(
        dup_label,
        Err(StaticValueStoreError::DuplicatePackLabel("x".into()))
    );

    // 5. Foreign child rejected
    let mut other_store = StaticValueStore::new();
    let foreign_int = other_store.intern_int(BigInt::from(99));
    let foreign_attempt = values.intern_pack(vec![foreign_int], vec![]);
    assert!(matches!(foreign_attempt, Err(StaticValueStoreError::StoreMismatch { .. })));

    // 6. Representation-distinct equal slices compare equal and have identical hashes/fingerprints
    // Create pack2 with directly interned elements that match slice1: positionals [int_two, int_three], labeled [(sym_x, str_a)]
    let pack2 = values
        .intern_pack(
            vec![int_two, int_three],
            vec![(sym_x, str_a)],
        )
        .expect("intern matching pack");
    let StaticValueData::Pack(pack2_id) = values.get(pack2).unwrap() else {
        panic!("expected Pack");
    };
    let (pack2_data, _) = values.get_pack(*pack2_id).unwrap();
    assert_ne!(pack2_data.backing, backing_id, "pack2 has a different backing");

    assert!(
        values.semantic_eq(slice1, pack2).unwrap(),
        "representation-distinct equal packs must compare equal under semantic_eq"
    );
    assert_eq!(
        values.semantic_hash(slice1).unwrap(),
        values.semantic_hash(pack2).unwrap(),
        "semantic_hash must match for structurally equal packs"
    );
    assert_eq!(
        values.content_fingerprint(slice1).unwrap(),
        values.content_fingerprint(pack2).unwrap(),
        "content_fingerprint must match for structurally equal packs"
    );

    // 7. Store clone/append preserves existing Pack refs
    let clone = values.clone();
    assert_eq!(clone.id(), values.id());
    assert_eq!(clone.get(pack1).unwrap(), values.get(pack1).unwrap());
    let appended = values.intern_int(BigInt::from(999));
    assert_eq!(clone.get(pack1).unwrap(), values.get(pack1).unwrap());
    assert!(values.get(appended).is_ok());

    // 8. Empty StaticPack derives proper Pack type (not Unit)
    let empty_pack = values.intern_pack(vec![], vec![]).expect("empty pack");
    let data_table = DataSemanticTable::new();
    let enum_table = EnumSemanticTable::new();
    let empty_pack_type = values
        .static_value_type(empty_pack, &mut types, &data_table, &enum_table)
        .expect("empty pack type");
    assert!(
        matches!(types.get(empty_pack_type), phalcom_semantic::types::TypeData::Pack(_)),
        "empty StaticPack must type as Pack, got {:?}",
        types.get(empty_pack_type)
    );
    assert_ne!(empty_pack_type, types.unit(), "empty Pack is not Unit");

    // 9. MaterializationDescriptor for Pack is DeferredStructural(DeferredStructuralKind::Pack)
    let mat_desc = MaterializationDescriptor::from_static_value(&values, pack1).expect("materialization descriptor");
    assert_eq!(
        mat_desc,
        MaterializationDescriptor::DeferredStructural(DeferredStructuralKind::Pack)
    );
    assert_eq!(mat_desc.capability(), MaterializationCapability::DeferredStructural);
}
