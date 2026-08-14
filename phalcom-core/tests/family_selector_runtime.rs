use phalcom_core::bytecode::{Bytecode, FamilySpecKind};
use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::heap::Object;
use phalcom_core::method::{MethodObject, SignatureKind};
use phalcom_core::primitive::class::behavior_extract;
use phalcom_core::primitive::method_family::method_family_bind;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_common::selector::{SelectorKindPattern, SelectorPattern};

#[test]
fn make_family_uses_explicit_exact_discriminator_and_allows_future_method() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<family>");
    let closure = vm
        .compile_closure_as(module, "const f = 1::future()\n", UnitKind::File)
        .expect("exact family compiles");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(chunk.code.iter().any(|opcode| matches!(
        opcode,
        Bytecode::MakeFamily {
            kind: FamilySpecKind::Exact,
            ..
        }
    )));
    vm.run_cell(module, closure).expect("family construction does not resolve target method");
}

#[test]
fn make_family_compiles_pattern_object_without_punctuation_heuristic() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<family>");
    let closure = vm
        .compile_closure_as(module, "const f = 1::future(...)\n", UnitKind::File)
        .expect("pattern family compiles");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    let pattern = chunk.constants.iter().find_map(|constant| match constant {
        Value::Obj(id) if matches!(vm.heap.get(*id), Object::SelectorPattern(_)) => Some(*id),
        _ => None,
    });
    assert!(pattern.is_some(), "pattern must be a first-class immutable heap object");
    assert!(chunk.code.iter().any(|opcode| matches!(
        opcode,
        Bytecode::MakeFamily {
            kind: FamilySpecKind::Pattern,
            ..
        }
    )));
    vm.run_cell(module, closure).expect("pattern family construction succeeds");
}

#[test]
fn behavior_pattern_extraction_snapshots_effective_exact_methods() {
    fn replacement(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> phalcom_core::error::PhResult<Value> {
        Ok(Value::Int(9))
    }

    let mut vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    let pattern = vm.heap.alloc(Object::SelectorPattern(Box::new(phalcom_core::heap::SelectorPatternObject {
        pattern: SelectorPattern::named("name", SelectorKindPattern::AnyNamed, Box::new([]), Box::new([]), true).expect("valid pattern"),
    })));

    let first = behavior_extract(&mut vm, &Value::Obj(object_class), &[Value::Obj(pattern)]).expect("pattern extraction");
    let Value::Obj(first_id) = first else { panic!("pattern extraction must return a MethodFamily") };
    let old_method = vm.heap.method_family(first_id).exact_methods.values().next().copied().expect("Object defines name");

    let selector = vm.get_or_intern("name");
    let replacement_method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_primitive(
        selector,
        SignatureKind::Getter,
        replacement,
        object_class,
    ))));
    vm.heap.class_mut(object_class).add_method(selector, replacement_method);

    let second = behavior_extract(&mut vm, &Value::Obj(object_class), &[Value::Obj(pattern)]).expect("second pattern extraction");
    let Value::Obj(second_id) = second else { panic!("second extraction must return a MethodFamily") };
    assert_eq!(vm.heap.method_family(first_id).exact_methods.get(&selector), Some(&old_method));
    assert_eq!(vm.heap.method_family(second_id).exact_methods.get(&selector), Some(&replacement_method));
}

#[test]
fn method_family_bind_captures_receiver_without_live_selection() {
    let mut vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    let pattern = vm.heap.alloc(Object::SelectorPattern(Box::new(phalcom_core::heap::SelectorPatternObject {
        pattern: SelectorPattern::named("name", SelectorKindPattern::AnyNamed, Box::new([]), Box::new([]), true).expect("valid pattern"),
    })));
    let family = behavior_extract(&mut vm, &Value::Obj(object_class), &[Value::Obj(pattern)]).expect("pattern extraction");
    let bound = method_family_bind(&mut vm, &family, &[Value::Int(42)]).expect("family binding");
    let Value::Obj(bound_id) = bound else { panic!("binding must return BoundMethodFamily") };
    let Object::BoundMethodFamily(bound) = vm.heap.get(bound_id) else { panic!("wrong bound-family heap variant") };
    assert_eq!(bound.family, family.as_obj().expect("family handle"));
    assert_eq!(bound.receiver, Value::Int(42));
}
