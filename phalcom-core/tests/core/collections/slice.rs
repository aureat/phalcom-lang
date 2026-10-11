use super::vm_support;
use phalcom_core::heap::ObjRef;
use phalcom_core::primitive::array::array_raw_set;
use phalcom_core::primitive::map::map_raw_put;
use phalcom_core::primitive::set::set_raw_add;
use phalcom_core::value::Value;

fn global(vm: &phalcom_core::vm::VM, module: ObjRef, name: &str) -> Value {
    vm.heap
        .module(module)
        .get(vm.interner.find(name).expect("interned global"))
        .expect("global value")
}

fn backing_range(vm: &phalcom_core::vm::VM, value: Value) -> (ObjRef, usize, usize) {
    let id = value.as_obj().expect("heap-backed sequence");
    match vm.heap.get(id) {
        phalcom_core::heap::Object::Array(array) => (id, 0, array.len()),
        phalcom_core::heap::Object::Vector(vector) => (id, 0, vector.len()),
        phalcom_core::heap::Object::Buffer(buffer) => (id, 0, buffer.len()),
        phalcom_core::heap::Object::Slice(slice) => (slice.owner(), slice.start(), slice.len()),
        _ => panic!("expected sequence owner, found {}", vm.heap.kind_of_for_test(id)),
    }
}

#[test]
fn slice_buffer_cross_family_views_mutation_gc_and_representation_invariants() {
    let source = r#"
let array = <Int>[10, 20, 30, 40]
let vector = <Int, 4>[10, 20, 30, 40]
let buffer = Buffer<Int>.new(5)
buffer.push(10)
buffer.push(20)
buffer.push(30)
buffer.push(40)
let rangeBuffer = Buffer<Int>.new(4)
rangeBuffer.push(10)
rangeBuffer.push(20)
rangeBuffer.push(30)
rangeBuffer.push(40)
let arraySlice = array.slice(1..3)
let vectorSlice = vector.slice(-3..-1)
let bufferSlice = buffer.slice(1..3)
let nestedSlice = bufferSlice.slice(1..)
let emptySlice = array.slice(2..2)
let first = arraySlice.first.unwrapOr(0)
let last = arraySlice.last.unwrapOr(0)
let negative = arraySlice.at(-1)
let missing = arraySlice.get(2)
let sameContent = arraySlice == vectorSlice
let differentLength = arraySlice == vectorSlice.slice(0..1)
let nestedValue = nestedSlice.at(0)
let emptySize = emptySlice.size
let emptyFirstMissing = emptySlice.first.isNone
let sliceListSize = arraySlice.toList.size
let iterator = arraySlice.iterator
let iteratorFirst = iterator.next.unwrapOr(0)
let iteratorSecond = iterator.next.unwrapOr(0)
let iteratorEnd = iterator.next.isNone
let bufferIterator = buffer.iterator
let bufferIteratorFirst = bufferIterator.next.unwrapOr(0)
let sameValuesLargerCapacity = Buffer<Int>.new(8)
sameValuesLargerCapacity.push(10)
sameValuesLargerCapacity.push(20)
sameValuesLargerCapacity.push(30)
sameValuesLargerCapacity.push(40)
let bufferEqualIgnoringCapacity = buffer == sameValuesLargerCapacity
let bufferListSize = buffer.toList.size
let allRange = Range.new(None, None, false)
let lowerOpenRange = Range.new(2, None, false)
let upperOpenRange = Range.new(None, 2, false)
let halfOpenRange = Range.new(1, 3, false)
let inclusiveRange = Range.new(1, 2, true)
let negativeLowerRange = Range.new(-2, None, false)
let negativeUpperRange = Range.new(None, -1, false)
let negativeBothRange = Range.new(-3, -1, false)
let lowerBeyondRange = Range.new(9, None, false)
let upperBeyondRange = Range.new(None, 9, false)
let negativeBeyondRange = Range.new(-9, None, false)
let zeroLengthRange = Range.new(2, 2, false)
let reversedRange = Range.new(3, 1, false)
let lastElementRange = Range.new(-1, -1, true)
let fractionalRange = Range.new(0.5, 2, false)
let rangeSlice = array.slice(allRange)
let emptyArray = <Int>[]
let emptyVector = <Int, 0>[]
let emptyBuffer = Buffer<Int>.new(0)
let oneArray = <Int>[7]
let oneVector = <Int, 1>[7]
let oneBuffer = Buffer<Int>.new(1)
oneBuffer.push(7)
let oneSlice = rangeSlice.slice(lastElementRange)
array.at(1, put: 21)
let liveArraySliceValue = arraySlice.at(0)
let arraySliceStillEqualsOriginalVector = arraySlice == vectorSlice
buffer.set(1, 22)
let liveBufferSliceValue = bufferSlice.at(0)
buffer.push(50)
let bufferSliceSizeAfterAppend = bufferSlice.size
let bufferSliceLastAfterAppend = bufferSlice.at(-1)
let bufferSizeAfterAppend = buffer.size
const fixedArray = <Int>[5, 6]
let fixedSlice = fixedArray.slice(..)
"#;
    let (mut vm, module) = vm_support::run_inline(source).expect("Slice/Buffer source APIs compile and run");

    for (name, expected) in [
        ("first", 20),
        ("last", 30),
        ("negative", 30),
        ("nestedValue", 30),
        ("sliceListSize", 2),
        ("iteratorFirst", 20),
        ("iteratorSecond", 30),
        ("bufferIteratorFirst", 10),
        ("bufferListSize", 4),
        ("liveArraySliceValue", 21),
        ("liveBufferSliceValue", 22),
        ("bufferSliceSizeAfterAppend", 2),
        ("bufferSliceLastAfterAppend", 30),
        ("bufferSizeAfterAppend", 5),
    ] {
        assert_eq!(global(&vm, module, name), Value::int(expected), "{name}");
    }
    assert!(global(&vm, module, "missing").is_none());
    assert_eq!(global(&vm, module, "sameContent"), Value::bool(true));
    assert_eq!(global(&vm, module, "differentLength"), Value::bool(false));
    assert_eq!(global(&vm, module, "arraySliceStillEqualsOriginalVector"), Value::bool(false));
    assert_eq!(global(&vm, module, "emptySize"), Value::int(0));
    assert_eq!(global(&vm, module, "emptyFirstMissing"), Value::bool(true));
    assert_eq!(global(&vm, module, "iteratorEnd"), Value::bool(true));
    assert_eq!(global(&vm, module, "bufferEqualIgnoringCapacity"), Value::bool(true));

    let range_sources = [
        ("Array", global(&vm, module, "array")),
        ("Vector", global(&vm, module, "vector")),
        ("Buffer", global(&vm, module, "rangeBuffer")),
        ("Slice", global(&vm, module, "rangeSlice")),
    ];
    let expected_ranges = [
        ("allRange", 0, 4),
        ("lowerOpenRange", 2, 4),
        ("upperOpenRange", 0, 2),
        ("halfOpenRange", 1, 3),
        ("inclusiveRange", 1, 3),
        ("negativeLowerRange", 2, 4),
        ("negativeUpperRange", 0, 3),
        ("negativeBothRange", 1, 3),
        ("lowerBeyondRange", 4, 4),
        ("upperBeyondRange", 0, 4),
        ("negativeBeyondRange", 0, 4),
        ("zeroLengthRange", 2, 2),
        ("reversedRange", 3, 3),
        ("lastElementRange", 3, 4),
    ];
    let slice_selector = vm.get_or_intern("slice(_)");
    for (source_name, source_value) in range_sources {
        let (root_owner, base_start, visible_len) = backing_range(&vm, source_value);
        assert_eq!(visible_len, 4, "{source_name} range fixture size");
        for (range_name, expected_start, expected_end) in expected_ranges {
            let range = global(&vm, module, range_name);
            let view = vm
                .send_dynamic(source_value, slice_selector, &[range])
                .unwrap_or_else(|error| panic!("{source_name}.slice({range_name}) failed: {error:?}"));
            let view_id = view.as_obj().expect("range result Slice");
            let view = vm.heap.slice(view_id);
            assert_eq!(view.owner(), root_owner, "{source_name} {range_name} owner");
            assert_eq!(view.start(), base_start + expected_start, "{source_name} {range_name} start");
            assert_eq!(view.len(), expected_end - expected_start, "{source_name} {range_name} length");
        }
        let fractional = global(&vm, module, "fractionalRange");
        assert!(
            vm.send_dynamic(source_value, slice_selector, &[fractional]).is_err(),
            "{source_name} rejects non-integral range bounds"
        );
    }

    let all_range = global(&vm, module, "allRange");
    for (source_name, source_value) in [
        ("empty Array", global(&vm, module, "emptyArray")),
        ("empty Vector", global(&vm, module, "emptyVector")),
        ("empty Buffer", global(&vm, module, "emptyBuffer")),
        ("empty Slice", global(&vm, module, "emptySlice")),
    ] {
        let view = vm
            .send_dynamic(source_value, slice_selector, &[all_range])
            .unwrap_or_else(|error| panic!("{source_name} full slice failed: {error:?}"));
        let view_id = view.as_obj().expect("empty result Slice");
        assert_eq!(vm.heap.slice(view_id).len(), 0, "{source_name} retains the empty boundary");
    }
    for (source_name, source_value) in [
        ("one-element Array", global(&vm, module, "oneArray")),
        ("one-element Vector", global(&vm, module, "oneVector")),
        ("one-element Buffer", global(&vm, module, "oneBuffer")),
        ("one-element Slice", global(&vm, module, "oneSlice")),
    ] {
        let view = vm
            .send_dynamic(source_value, slice_selector, &[all_range])
            .unwrap_or_else(|error| panic!("{source_name} full slice failed: {error:?}"));
        let view_id = view.as_obj().expect("one-element result Slice");
        assert_eq!(vm.heap.slice(view_id).len(), 1, "{source_name} preserves its sole element");
    }

    let buffer = global(&vm, module, "buffer").as_obj().expect("Buffer handle");
    let nested = global(&vm, module, "nestedSlice").as_obj().expect("nested Slice handle");
    assert_eq!(vm.heap.slice(nested).owner(), buffer, "nested Slice is normalized to its original Buffer");
    assert_eq!(vm.heap.slice(nested).start(), 2);

    let array_slice = global(&vm, module, "arraySlice");
    let buffer_slice = global(&vm, module, "bufferSlice");
    let vector_slice = global(&vm, module, "vectorSlice");
    let buffer_value = global(&vm, module, "buffer");
    let map = Value::obj(vm.heap.alloc_map());
    for (key, label) in [
        (array_slice, "Array-backed Slice"),
        (buffer_slice, "Buffer-backed Slice"),
        (vector_slice, "Vector-backed Slice"),
        (buffer_value, "Buffer"),
    ] {
        assert!(map_raw_put(&mut vm, &map, &[key, Value::int(1)]).is_err(), "{label} Map key rejection");
    }
    let set = Value::obj(vm.heap.alloc_set());
    for (key, label) in [
        (array_slice, "Array-backed Slice"),
        (buffer_slice, "Buffer-backed Slice"),
        (vector_slice, "Vector-backed Slice"),
    ] {
        assert!(set_raw_add(&mut vm, &set, &[key]).is_err(), "{label} Set key rejection");
    }

    let fixed_slice = global(&vm, module, "fixedSlice");
    let fixed_slice_id = fixed_slice.as_obj().expect("const Array Slice handle");
    let fixed_owner = vm.heap.slice(fixed_slice_id).owner();
    assert!(matches!(vm.heap.get(fixed_owner), phalcom_core::heap::Object::Array(array) if array.is_frozen()));
    assert!(
        array_raw_set(&mut vm, &Value::obj(fixed_owner), &[Value::int(0), Value::int(99)]).is_err(),
        "Slice must not bypass frozen const Array protection"
    );
    assert_eq!(vm.heap.slice_element(fixed_slice_id, 0), Some(Value::int(5)));
    let set_selector = vm.get_or_intern("set(_,_)");
    assert!(
        vm.send_dynamic(fixed_slice, set_selector, &[Value::int(0), Value::int(99)]).is_err(),
        "Slice exposes no mutation selector"
    );

    let retained_child = vm.alloc_string_value("only reachable through deepest Slice".to_owned());
    let retained_child_id = retained_child.as_obj().expect("GC child handle");
    let backing = vm.heap.alloc_array(vec![retained_child]);
    let first_view = vm.heap.alloc_slice_from_range(backing, 0, 1).expect("first GC view");
    let second_view = vm.heap.alloc_slice_from_range(first_view, 0, 1).expect("second GC view");
    let deepest_view = vm.heap.alloc_slice_from_range(second_view, 0, 1).expect("deepest GC view");
    vm.push_root_for_test(Value::obj(deepest_view));
    vm.force_gc();
    assert!(vm.heap.try_get(deepest_view).is_some(), "deepest Slice remains rooted");
    assert!(vm.heap.try_get(backing).is_some(), "deepest Slice retains the root owner");
    assert!(vm.heap.try_get(retained_child_id).is_some(), "root owner traces the child value");
    assert!(vm.heap.try_get(first_view).is_none(), "intermediate Slice is not part of the owner chain");
    assert!(vm.heap.try_get(second_view).is_none(), "nested views normalize to the root owner");
    vm.pop_root_for_test();

    assert_eq!(std::mem::size_of::<Value>(), 16);
    assert_eq!(std::mem::size_of::<phalcom_core::heap::Object>(), 48, "P5 does not grow the Object arena slot");
}
