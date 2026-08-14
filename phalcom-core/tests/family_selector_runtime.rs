use phalcom_core::bytecode::{Bytecode, FamilySpecKind};
use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::error::{PhError, RuntimeError};
use phalcom_core::heap::Object;
use phalcom_core::method::{MethodObject, SignatureKind};
use phalcom_core::primitive::class::behavior_extract;
use phalcom_core::primitive::block::block_call;
use phalcom_core::primitive::method::method_bind;
use phalcom_core::primitive::method_family::method_family_bind;
use phalcom_core::primitive::object::object_method_for;
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

#[test]
fn captured_method_can_use_dynamic_behavior_on_foreign_receiver() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "captured_method_dynamic_foreign_receiver");
    vm.interpret_source(
        module,
        "class Source { label { self.title } }\nclass Target { title { 42 } }\nlet source = Source.new()\nlet target = Target.new()\n",
    )
    .expect("source and target classes should compile");

    let source = vm.heap.module(module).get(vm.interner.intern("source")).expect("source should exist");
    let target = vm.heap.module(module).get(vm.interner.intern("target")).expect("target should exist");
    let label_selector = vm.get_or_intern("label");
    let method = object_method_for(&mut vm, &source, &[Value::Symbol(label_selector)]).expect("method should exist");
    let bound = method_bind(&mut vm, &method, &[target]).expect("foreign receiver should bind");
    assert_eq!(block_call(&mut vm, &bound, &[]).expect("foreign method should activate"), Value::Int(42));
}

#[test]
fn captured_method_rejects_foreign_field_layout_before_slot_access() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "captured_method_foreign_field_layout");
    vm.interpret_source(
        module,
        "class Source { @constructor new() { _field = 41 } read { _field } }\nclass Target { @constructor new() { _other = 99 } }\nlet source = Source.new()\nlet target = Target.new()\n",
    )
    .expect("field-bearing classes should compile");

    let source = vm.heap.module(module).get(vm.interner.intern("source")).expect("source should exist");
    let target = vm.heap.module(module).get(vm.interner.intern("target")).expect("target should exist");
    let read_selector = vm.get_or_intern("read");
    let method = object_method_for(&mut vm, &source, &[Value::Symbol(read_selector)]).expect("method should exist");
    let bound = method_bind(&mut vm, &method, &[target]).expect("foreign receiver should bind");
    let result = block_call(&mut vm, &bound, &[]);
    assert!(
        matches!(result, Err(PhError::Runtime(RuntimeError::IncompatibleMethodLayout { selector, .. })) if selector == "read"),
        "foreign field access should fail with layout error, got {result:?}"
    );
}

#[test]
fn captured_method_allows_subclass_layout_and_lexical_super() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "captured_method_subclass_layout_and_lexical_super");
    vm.interpret_source(
        module,
        "class Parent { value { 7 } }\nclass Source is Parent { @constructor new() { _field = 41 } read { _field } viaSuper { super.value } }\nclass Child is Source { @constructor new() { _field = 42 } }\nlet source = Source.new()\nlet child = Child.new()\n",
    )
    .expect("parent and subclass classes should compile");

    let source = vm.heap.module(module).get(vm.interner.intern("source")).expect("source should exist");
    let child = vm.heap.module(module).get(vm.interner.intern("child")).expect("child should exist");

    let read_selector = vm.get_or_intern("read");
    let read_method = object_method_for(&mut vm, &source, &[Value::Symbol(read_selector)]).expect("read method should exist");
    let bound_read = method_bind(&mut vm, &read_method, &[child]).expect("subclass receiver should bind");
    assert_eq!(block_call(&mut vm, &bound_read, &[]).expect("subclass field access should succeed"), Value::Int(42));

    let super_selector = vm.get_or_intern("viaSuper");
    let super_method = object_method_for(&mut vm, &source, &[Value::Symbol(super_selector)]).expect("super method should exist");
    let bound_super = method_bind(&mut vm, &super_method, &[child]).expect("foreign receiver should bind");
    assert_eq!(block_call(&mut vm, &bound_super, &[]).expect("lexical super should succeed"), Value::Int(7));
}

#[test]
fn captured_primitive_method_accepts_foreign_receiver() {
    fn constant(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> phalcom_core::error::PhResult<Value> {
        Ok(Value::Int(17))
    }

    let mut vm = VM::new();
    let selector = vm.get_or_intern("constant");
    let object_class = vm.universe.classes.object_class;
    let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_primitive(
        selector,
        SignatureKind::Getter,
        constant,
        object_class,
    ))));
    let method_value = Value::Obj(method);
    let bound = method_bind(&mut vm, &method_value, &[Value::Int(3)]).expect("primitive method should bind");
    assert_eq!(block_call(&mut vm, &bound, &[]).expect("primitive method should activate"), Value::Int(17));
}
