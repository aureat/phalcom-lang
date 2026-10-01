//! Checkpoint C2 runtime representation, GC, and relation tests.

use phalcom_core::data::RuntimeDataDescriptorId;
use phalcom_core::primitive::object::{object_hash, object_same};
use phalcom_core::product::{ProductComponentLayout, ProductLayout, ProductSlotRepr, ProductStorage};
use phalcom_core::typing::handle::{RuntimeOverlayTypeId, RuntimeTypeRef};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::DeclarationId;

#[test]
fn value_is_exactly_sixteen_bytes() {
    assert_eq!(std::mem::size_of::<Value>(), 16);
    assert_eq!(std::mem::align_of::<Value>(), 8);
}

#[test]
fn p4_registry_separates_declaration_class_from_exact_descriptors() {
    let mut vm = VM::new();
    let root_mod = vm.runtime_roots.unwrap().universe;
    let owner = DeclarationId::new(vm.heap.module(root_mod).id.clone(), "ShapeGeneric".into());
    let behavior_class = vm.universe.classes.object_class;

    assert_eq!(vm.data_registry.register_behavior_class(owner.clone(), behavior_class), behavior_class);
    assert_eq!(vm.data_registry.behavior_class_for_declaration(&owner), Some(behavior_class));
    assert_eq!(vm.data_registry.descriptor_by_declaration(&owner), None);

    let layout_one = vm.heap.product_layouts.register(
        ProductLayout::new(vec![ProductComponentLayout {
            logical_index: 0,
            word_offset: 0,
            repr: ProductSlotRepr::Int64,
        }])
        .expect("one-slot layout"),
    );
    let layout_two = vm.heap.product_layouts.register(
        ProductLayout::new(vec![
            ProductComponentLayout {
                logical_index: 0,
                word_offset: 0,
                repr: ProductSlotRepr::Int64,
            },
            ProductComponentLayout {
                logical_index: 1,
                word_offset: 1,
                repr: ProductSlotRepr::Value,
            },
        ])
        .expect("two-slot layout"),
    );
    let exact_one = RuntimeTypeRef::Overlay(RuntimeOverlayTypeId(1));
    let exact_two = RuntimeTypeRef::Overlay(RuntimeOverlayTypeId(2));

    let one = vm
        .data_registry
        .register_with_component_slots(owner.clone(), behavior_class, Some(exact_one), layout_one, Some(vec![0].into_boxed_slice()));
    let two = vm
        .data_registry
        .register_with_component_slots(owner.clone(), behavior_class, Some(exact_two), layout_two, Some(vec![0, 1].into_boxed_slice()));
    let one_again =
        vm.data_registry
            .register_with_component_slots(owner.clone(), behavior_class, Some(exact_one), layout_two, Some(vec![0].into_boxed_slice()));

    assert_eq!(one, one_again, "the same owner and exact type reuse their descriptor");
    assert_ne!(one, two, "different exact types receive distinct descriptors");
    let one_descriptor = vm.data_registry.descriptor(one).expect("one descriptor");
    let two_descriptor = vm.data_registry.descriptor(two).expect("two descriptor");
    assert_eq!(one_descriptor.behavior_class, behavior_class);
    assert_eq!(two_descriptor.behavior_class, behavior_class);
    assert_eq!(one_descriptor.exact_type, Some(exact_one));
    assert_eq!(two_descriptor.exact_type, Some(exact_two));
    assert_eq!(one_descriptor.layout, layout_one, "re-registration must not replace the exact layout");
    assert_eq!(two_descriptor.layout, layout_two);
    assert_eq!(one_descriptor.component_slots.as_deref(), Some(&[0][..]));
    assert_eq!(two_descriptor.component_slots.as_deref(), Some(&[0, 1][..]));
}

#[test]
fn row_specializations_with_equal_layout_keep_distinct_exact_descriptors() {
    let mut vm = VM::new();
    let root_mod = vm.runtime_roots.unwrap().universe;
    let owner = DeclarationId::new(vm.heap.module(root_mod).id.clone(), "RowGeneric".into());
    let behavior_class = vm.universe.classes.object_class;
    let layout = vm.heap.product_layouts.register(
        ProductLayout::new(vec![ProductComponentLayout {
            logical_index: 0,
            word_offset: 0,
            repr: ProductSlotRepr::Value,
        }])
        .expect("one shared physical slot layout"),
    );
    let int_row_type = RuntimeTypeRef::Overlay(RuntimeOverlayTypeId(31));
    let string_row_type = RuntimeTypeRef::Overlay(RuntimeOverlayTypeId(32));

    let int_descriptor = vm.data_registry.register(owner.clone(), behavior_class, Some(int_row_type), layout);
    let string_descriptor = vm.data_registry.register(owner.clone(), behavior_class, Some(string_row_type), layout);
    let int_descriptor_again = vm.data_registry.register(owner.clone(), behavior_class, Some(int_row_type), layout);

    assert_ne!(
        int_descriptor, string_descriptor,
        "exact row types remain descriptor identity even with equal layouts"
    );
    assert_eq!(int_descriptor, int_descriptor_again, "the exact owner/type pair is interned");
    let int = vm.data_registry.descriptor(int_descriptor).expect("int row descriptor");
    let string = vm.data_registry.descriptor(string_descriptor).expect("string row descriptor");
    assert_eq!(int.layout, string.layout);
    assert_eq!(int.behavior_class, string.behavior_class);
    assert_eq!(int.exact_type, Some(int_row_type));
    assert_eq!(string.exact_type, Some(string_row_type));
}

#[test]
fn data_nullary_singleton_is_zero_allocation_and_specialization_sensitive() {
    let vm = VM::new();
    let initial_objects = vm.heap.iter_handles_for_test().len();

    let did0 = RuntimeDataDescriptorId(0);
    let did1 = RuntimeDataDescriptorId(1);

    let val0 = Value::data_singleton(did0);
    let val1 = Value::data_singleton(did1);

    assert!(val0.is_data_singleton());
    assert!(val1.is_data_singleton());
    assert_eq!(val0.as_data_singleton(), Some(did0));
    assert_eq!(val1.as_data_singleton(), Some(did1));

    // Zero heap allocation
    assert_eq!(vm.heap.iter_handles_for_test().len(), initial_objects);

    // Specialization sensitivity: distinct descriptor IDs are unequal
    assert_ne!(val0, val1);
    assert!(!vm.semantic_same(val0, val1));
    assert!(vm.semantic_same(val0, Value::data_singleton(did0)));
}

#[test]
fn data_packed_value_slot_keeps_gc_child_alive() {
    let mut vm = VM::new();

    let comp0 = ProductComponentLayout {
        logical_index: 0,
        word_offset: 0,
        repr: ProductSlotRepr::Int64,
    };
    let comp1 = ProductComponentLayout {
        logical_index: 1,
        word_offset: 1,
        repr: ProductSlotRepr::Value,
    };
    let layout = ProductLayout::new(vec![comp0, comp1]).expect("valid layout");
    let layout_id = vm.heap.product_layouts.register(layout.clone());

    let root_mod = vm.runtime_roots.unwrap().universe;
    let dummy_decl = DeclarationId::new(vm.heap.module(root_mod).id.clone(), "DataPacked".into());
    let rdesc_id = vm.data_registry.register(dummy_decl, vm.universe.classes.object_class, None, layout_id);

    // Allocate a string on the heap
    let child_str = vm.alloc_string_value("child_string_payload".to_string());
    let child_ref = child_str.as_obj().expect("string is an object");

    let mut storage = ProductStorage::new(layout_id, &layout);
    storage.store_component(&layout, 0, Value::int(42)).expect("store int");
    storage.store_component(&layout, 1, child_str).expect("store value");

    let data_ref = vm.heap.alloc_data(rdesc_id, storage);
    let data_val = Value::obj(data_ref);

    // Push data object as a root
    vm.push_root_for_test(data_val);

    // Trigger GC
    vm.force_gc();

    // The data object and its child string in slot 1 must both survive GC
    assert!(vm.heap.as_data(data_ref).is_some());
    assert!(vm.heap.as_string(child_ref).is_some());
    assert_eq!(vm.heap.as_string(child_ref).unwrap().value(), "child_string_payload");

    // Read back through storage
    let data_obj = vm.heap.as_data(data_ref).unwrap();
    let loaded_int = data_obj.storage.load_component(&layout, 0).expect("load int");
    let loaded_str = data_obj.storage.load_component(&layout, 1).expect("load str");

    assert_eq!(loaded_int, Value::int(42));
    assert_eq!(loaded_str, child_str);

    vm.pop_root_for_test();
    vm.force_gc();

    // After popping root and GC, both should be collected
    assert!(vm.heap.try_get(data_ref).is_none());
    assert!(vm.heap.try_get(child_ref).is_none());
}

#[test]
fn data_exact_relation_ignores_backing_handle() {
    let mut vm = VM::new();

    let comp0 = ProductComponentLayout {
        logical_index: 0,
        word_offset: 0,
        repr: ProductSlotRepr::Int64,
    };
    let comp1 = ProductComponentLayout {
        logical_index: 1,
        word_offset: 1,
        repr: ProductSlotRepr::Float64,
    };
    let layout = ProductLayout::new(vec![comp0, comp1]).expect("valid layout");
    let layout_id = vm.heap.product_layouts.register(layout.clone());

    let root_mod = vm.runtime_roots.unwrap().universe;
    let dummy_decl = DeclarationId::new(vm.heap.module(root_mod).id.clone(), "Point".into());
    let rdesc_id = vm.data_registry.register(dummy_decl, vm.universe.classes.object_class, None, layout_id);

    let mut storage1 = ProductStorage::new(layout_id, &layout);
    storage1.store_component(&layout, 0, Value::int(10)).expect("store x");
    storage1.store_component(&layout, 1, Value::float(20.5)).expect("store y");
    let obj1 = vm.heap.alloc_data(rdesc_id, storage1);

    let mut storage2 = ProductStorage::new(layout_id, &layout);
    storage2.store_component(&layout, 0, Value::int(10)).expect("store x");
    storage2.store_component(&layout, 1, Value::float(20.5)).expect("store y");
    let obj2 = vm.heap.alloc_data(rdesc_id, storage2);

    assert_ne!(obj1, obj2); // distinct handles

    let v1 = Value::obj(obj1);
    let v2 = Value::obj(obj2);

    // Structural sameness and equality ignore heap handle
    assert!(vm.semantic_same(v1, v2));
    assert!(v1.value_eq(&v2, &vm.heap));

    let same_result = object_same(&mut vm, &v1, &[v2]).expect("object_same");
    assert_eq!(same_result, Value::bool(true));

    // Modify a component in obj2
    let mut storage3 = ProductStorage::new(layout_id, &layout);
    storage3.store_component(&layout, 0, Value::int(99)).expect("store x");
    storage3.store_component(&layout, 1, Value::float(20.5)).expect("store y");
    let obj3 = vm.heap.alloc_data(rdesc_id, storage3);
    let v3 = Value::obj(obj3);

    assert!(!vm.semantic_same(v1, v3));
    assert!(!v1.value_eq(&v3, &vm.heap));

    let same_diff = object_same(&mut vm, &v1, &[v3]).expect("object_same");
    assert_eq!(same_diff, Value::bool(false));
}

#[test]
fn data_hash_equal_values_equal_hash() {
    let mut vm = VM::new();

    let comp0 = ProductComponentLayout {
        logical_index: 0,
        word_offset: 0,
        repr: ProductSlotRepr::Int64,
    };
    let comp1 = ProductComponentLayout {
        logical_index: 1,
        word_offset: 1,
        repr: ProductSlotRepr::Symbol,
    };
    let layout = ProductLayout::new(vec![comp0, comp1]).expect("valid layout");
    let layout_id = vm.heap.product_layouts.register(layout.clone());

    let root_mod = vm.runtime_roots.unwrap().universe;
    let dummy_decl = DeclarationId::new(vm.heap.module(root_mod).id.clone(), "Item".into());
    let rdesc_id = vm.data_registry.register(dummy_decl, vm.universe.classes.object_class, None, layout_id);

    let sym = vm.get_or_intern("test_symbol");

    let mut storage1 = ProductStorage::new(layout_id, &layout);
    storage1.store_component(&layout, 0, Value::int(123)).expect("store int");
    storage1.store_component(&layout, 1, Value::symbol(sym)).expect("store sym");
    let obj1 = vm.heap.alloc_data(rdesc_id, storage1);

    let mut storage2 = ProductStorage::new(layout_id, &layout);
    storage2.store_component(&layout, 0, Value::int(123)).expect("store int");
    storage2.store_component(&layout, 1, Value::symbol(sym)).expect("store sym");
    let obj2 = vm.heap.alloc_data(rdesc_id, storage2);

    let v1 = Value::obj(obj1);
    let v2 = Value::obj(obj2);

    let h1 = object_hash(&mut vm, &v1, &[]).expect("object_hash");
    let h2 = object_hash(&mut vm, &v2, &[]).expect("object_hash");

    assert_eq!(h1, h2);
}
