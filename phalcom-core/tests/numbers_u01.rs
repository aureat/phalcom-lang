use num_bigint::BigInt;
use phalcom_core::heap::Object;
use phalcom_core::value::{Value, normalize_bigint};
use phalcom_core::vm::VM;

#[test]
fn test_number_classes_and_abstract_instantiation() {
    let mut vm = VM::new();
    let c = vm.universe.classes;

    // Check class hierarchy & is_abstract flag
    assert!(vm.heap.class(c.number_class).is_abstract, "Number class must be marked abstract");
    assert!(!vm.heap.class(c.int_class).is_abstract, "Int class must be concrete");
    assert!(!vm.heap.class(c.float_class).is_abstract, "Float class must be concrete");

    assert_eq!(vm.heap.class(c.int_class).superclass, Some(c.number_class), "Int superclass must be Number");
    assert_eq!(vm.heap.class(c.float_class).superclass, Some(c.number_class), "Float superclass must be Number");

    // Value::class return values
    let int_val = Value::Int(42);
    let float_val = Value::Float(3.14);
    assert_eq!(int_val.class(&vm), c.int_class);
    assert_eq!(float_val.class(&vm), c.float_class);

    // LargeInt class is Int
    let big = BigInt::parse_bytes(b"999999999999999999999999999999", 10).unwrap();
    let large_val = normalize_bigint(big, &mut vm.heap);
    assert_eq!(large_val.class(&vm), c.int_class);

    // Instantiation rejection for abstract Number class at both constructor arities.
    for (signature, args) in [("new()", &[][..]), ("new(_)", &[Value::Int(1)][..])] {
        let new_sym = vm.get_or_intern(signature);
        let res = vm.send_dynamic(Value::Obj(c.number_class), new_sym, args);
        assert!(res.is_err(), "Number.{signature} must raise an error");
        let err = res.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("abstractClass") || err_str.contains("cannot instantiate abstract class"),
            "error must indicate abstract class: {err_str}",
        );
    }

    let new_one = vm.get_or_intern("new(_)");
    assert_eq!(vm.send_dynamic(Value::Obj(c.int_class), new_one, &[Value::Int(1)]).unwrap(), Value::Int(1));
    assert_eq!(vm.send_dynamic(Value::Obj(c.float_class), new_one, &[Value::Int(1)]).unwrap(), Value::Float(1.0));
}

#[test]
fn test_large_int_normalization_and_gc() {
    let mut vm = VM::new();

    // Small BigInt -> Value::Int
    let small_big = BigInt::from(100i64);
    let norm_small = normalize_bigint(small_big, &mut vm.heap);
    assert_eq!(norm_small, Value::Int(100));

    // Large BigInt -> Object::LargeInt
    let huge_big: BigInt = BigInt::from(i64::MAX) + 1;
    let norm_huge = normalize_bigint(huge_big.clone(), &mut vm.heap);
    match norm_huge {
        Value::Obj(id) => {
            let heap_obj = vm.heap.get(id);
            match heap_obj {
                Object::LargeInt(val) => assert_eq!(val, &huge_big),
                _ => panic!("expected Object::LargeInt"),
            }
        }
        _ => panic!("expected Value::Obj for huge BigInt"),
    }

    // Force GC, ensure LargeInt is preserved if rooted
    vm.push_root_for_test(norm_huge);
    vm.force_gc();
    vm.pop_root_for_test();

    match norm_huge {
        Value::Obj(id) => {
            let heap_obj = vm.heap.get(id);
            match heap_obj {
                Object::LargeInt(val) => assert_eq!(val, &huge_big),
                _ => panic!("expected Object::LargeInt to survive GC"),
            }
        }
        _ => panic!("expected Value::Obj"),
    }
}

#[test]
fn test_negated_i64_min_overflow() {
    let mut vm = VM::new();
    let min_val = Value::Int(i64::MIN);
    let neg_sym = vm.get_or_intern("negated()");
    let neg_res = vm.send_dynamic(min_val, neg_sym, &[]).unwrap();

    // -i64::MIN overflows i64 and becomes a LargeInt
    match neg_res {
        Value::Obj(id) => {
            let heap_obj = vm.heap.get(id);
            match heap_obj {
                Object::LargeInt(val) => {
                    let expected = -BigInt::from(i64::MIN);
                    assert_eq!(val, &expected);
                }
                _ => panic!("expected Object::LargeInt for -i64::MIN"),
            }
        }
        _ => panic!("expected Value::Obj for -i64::MIN"),
    }
}
