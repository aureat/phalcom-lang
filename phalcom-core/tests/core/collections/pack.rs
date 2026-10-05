//! Runtime Pack storage, view, slicing, and equality tests.
//!
//! Realizes LANG007.C4.P3 T3 requirements:
//! - EmptyPack immediate tag, identity, and Pack class mapping (CV-01).
//! - Pack root construction, shape, and PackView O(1) lane access (CV-11, CV-12).
//! - Pack slicing with offset composition and slice-of-slice direct root normalization (CV-13, CV-31).
//! - Zero visible lane slicing normalizes to EmptyPack.
//! - Pack semantic equality (CV-24).

use phalcom_core::product::{finish_pack_slice_value, finish_pack_value};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn settled_vm() -> VM {
    let mut vm = VM::new_kernel();
    vm.force_gc();
    vm
}

#[test]
fn empty_pack_immediate_identity_and_class() {
    let vm = settled_vm();
    let empty = Value::empty_pack();

    assert!(empty.is_empty_pack(), "Value::is_empty_pack() must return true for empty_pack()");
    assert!(!empty.is_unit(), "EmptyPack must not be Unit");
    assert!(!empty.is_empty_record(), "EmptyPack must not be EmptyRecord");
    assert!(!empty.is_nil(), "EmptyPack must not be Nil");

    assert_eq!(empty.type_name(), "pack");
    assert_eq!(empty.class(&vm), vm.universe.classes.pack_class);
    assert_eq!(empty.to_string(&vm), "Pack()");
    assert_eq!(empty.to_debug(&vm), "Pack()");

    // Value equality
    let empty2 = Value::empty_pack();
    assert!(empty.value_eq(&empty2, &vm.heap));
    assert!(empty.same_as(&empty2));
    assert!(!empty.value_eq(&Value::unit(), &vm.heap));
    assert!(!empty.value_eq(&Value::empty_record(), &vm.heap));

    // pack_view_value on empty_pack returns an empty PackView with 0 lanes
    let empty_view = vm.pack_view_value(empty).expect("pack_view_value on empty_pack");
    assert_eq!(empty_view.positional_len(), 0);
    assert_eq!(empty_view.labeled_len(), 0);
    assert_eq!(empty_view.len(), 0);
    assert!(empty_view.is_empty());
}

#[test]
fn finish_pack_empty_returns_empty_pack() {
    let mut vm = settled_vm();
    let pack_val = finish_pack_value(&mut vm, vec![], vec![]).expect("finish_pack empty");
    assert!(pack_val.is_empty_pack());
    assert_eq!(pack_val, Value::empty_pack());
}

#[test]
fn pack_root_preserves_lanes_and_order() {
    let mut vm = settled_vm();
    let sym_a = vm.interner.intern("a");
    let sym_b = vm.interner.intern("b");
    let sym_c = vm.interner.intern("c");

    let val1 = Value::int(10);
    let val2 = Value::int(20);
    let val3 = vm.alloc_string_value("hello".into());
    let val4 = Value::bool(true);

    let pack_val = finish_pack_value(
        &mut vm,
        vec![val1, val2],
        vec![(sym_a, val3), (sym_b, val4)],
    )
    .expect("finish_pack root");

    assert!(!pack_val.is_empty_pack());
    assert_eq!(pack_val.class(&vm), vm.universe.classes.pack_class);
    assert_eq!(pack_val.type_name(), "object");
    assert_eq!(vm.heap.kind_of_for_test(pack_val.as_obj().unwrap()), "Pack");

    let view = vm.pack_view_value(pack_val).expect("pack view");
    assert_eq!(view.positional_len(), 2);
    assert_eq!(view.labeled_len(), 2);
    assert_eq!(view.len(), 4);

    // Positional access
    assert_eq!(view.get_positional(0), Some(val1));
    assert_eq!(view.get_positional(1), Some(val2));
    assert_eq!(view.get_positional(2), None);

    // Labeled access
    assert_eq!(view.get_label(0), Some(sym_a));
    assert_eq!(view.get_label(1), Some(sym_b));
    assert_eq!(view.get_label(2), None);

    assert_eq!(view.get_labeled_value(0), Some(val3));
    assert_eq!(view.get_labeled_value(1), Some(val4));
    assert_eq!(view.get_labeled_value(2), None);

    // Lookup by label
    assert_eq!(view.get_by_label(sym_a), Some(val3));
    assert_eq!(view.get_by_label(sym_b), Some(val4));
    assert_eq!(view.get_by_label(sym_c), None);

    // String formatting
    assert_eq!(pack_val.to_string(&vm), "Pack(10, 20, a: hello, b: true)");
}

#[test]
fn pack_slice_and_composition() {
    let mut vm = settled_vm();
    let sym_x = vm.interner.intern("x");
    let sym_y = vm.interner.intern("y");
    let sym_z = vm.interner.intern("z");

    let p0 = Value::int(100);
    let p1 = Value::int(101);
    let p2 = Value::int(102);

    let lx = vm.alloc_string_value("vx".into());
    let ly = vm.alloc_string_value("vy".into());
    let lz = vm.alloc_string_value("vz".into());

    let root_val = finish_pack_value(
        &mut vm,
        vec![p0, p1, p2],
        vec![(sym_x, lx), (sym_y, ly), (sym_z, lz)],
    )
    .expect("finish_pack root");

    let root_ref = root_val.as_obj().expect("root obj");

    // Slice 1: drop 1 positional (offset 1, len 2), drop 1 labeled (offset 1, len 2)
    // Visible: pos [p1, p2], labeled [y: vy, z: vz]
    let slice1_val = finish_pack_slice_value(&mut vm, root_val, 1, 2, 1, 2).expect("slice 1");
    let slice1_view = vm.pack_view_value(slice1_val).expect("slice 1 view");

    assert_eq!(slice1_view.positional_len(), 2);
    assert_eq!(slice1_view.labeled_len(), 2);
    assert_eq!(slice1_view.get_positional(0), Some(p1));
    assert_eq!(slice1_view.get_positional(1), Some(p2));
    assert_eq!(slice1_view.get_label(0), Some(sym_y));
    assert_eq!(slice1_view.get_label(1), Some(sym_z));
    assert_eq!(slice1_view.get_labeled_value(0), Some(ly));
    assert_eq!(slice1_view.get_labeled_value(1), Some(lz));

    // Hidden prefixes not visible
    assert_eq!(slice1_view.get_by_label(sym_x), None);
    assert_eq!(slice1_view.get_by_label(sym_y), Some(ly));

    // Check internal storage: slice1 points directly to root_ref
    let slice1_obj = vm.heap.pack(slice1_val.as_obj().unwrap());
    match slice1_obj.storage() {
        phalcom_core::heap::pack::PackStorage::Slice { root } => {
            assert_eq!(*root, root_ref, "slice1 must point to root");
        }
        _ => panic!("slice1 must be a Slice variant"),
    }

    // Slice 2 (slice-of-slice): take 1 positional (offset 1, len 1 -> p2), 1 labeled (offset 0, len 1 -> y: vy)
    let slice2_val = finish_pack_slice_value(&mut vm, slice1_val, 1, 1, 0, 1).expect("slice 2");
    let slice2_view = vm.pack_view_value(slice2_val).expect("slice 2 view");

    assert_eq!(slice2_view.positional_len(), 1);
    assert_eq!(slice2_view.labeled_len(), 1);
    assert_eq!(slice2_view.get_positional(0), Some(p2));
    assert_eq!(slice2_view.get_label(0), Some(sym_y));
    assert_eq!(slice2_view.get_labeled_value(0), Some(ly));

    // Slice-of-slice NORMALIZATION: slice2 must point directly to root_ref, NOT slice1!
    let slice2_obj = vm.heap.pack(slice2_val.as_obj().unwrap());
    match slice2_obj.storage() {
        phalcom_core::heap::pack::PackStorage::Slice { root } => {
            assert_eq!(*root, root_ref, "slice2 must normalize directly to original root!");
        }
        _ => panic!("slice2 must be a Slice variant"),
    }
    // Ranges composed: pos_start was 1 + 1 = 2; lab_start was 1 + 0 = 1
    assert_eq!(slice2_obj.positional_start(), 2);
    assert_eq!(slice2_obj.positional_len(), 1);
    assert_eq!(slice2_obj.labeled_start(), 1);
    assert_eq!(slice2_obj.labeled_len(), 1);

    // Zero-length slice normalizes to EmptyPack
    let empty_slice = finish_pack_slice_value(&mut vm, slice2_val, 0, 0, 0, 0).expect("empty slice");
    assert!(empty_slice.is_empty_pack(), "zero-lane slice must normalize to EmptyPack");
}

#[test]
fn pack_semantic_same_equality() {
    let mut vm = settled_vm();
    let sym_a = vm.interner.intern("a");
    let val_pos = Value::int(42);
    let val_lab = vm.alloc_string_value("hello".into());

    // Root 1
    let p1 = finish_pack_value(&mut vm, vec![val_pos], vec![(sym_a, val_lab)]).expect("p1");

    // Root 2: distinct allocation with identical contents
    let p2 = finish_pack_value(&mut vm, vec![val_pos], vec![(sym_a, val_lab)]).expect("p2");

    assert!(vm.semantic_same(p1, p2), "structurally equal packs must be semantic_same");

    // Slice of larger root that matches p1
    let sym_b = vm.interner.intern("b");
    let p3_root = finish_pack_value(
        &mut vm,
        vec![Value::int(0), val_pos],
        vec![(sym_a, val_lab), (sym_b, Value::int(99))],
    )
    .expect("p3_root");

    let p3_slice = finish_pack_slice_value(&mut vm, p3_root, 1, 1, 0, 1).expect("p3_slice");
    assert!(vm.semantic_same(p1, p3_slice), "slice and root with same visible elements must be semantic_same");

    // Empty packs are semantic_same
    assert!(vm.semantic_same(Value::empty_pack(), Value::empty_pack()));
    assert!(!vm.semantic_same(p1, Value::empty_pack()));
}
