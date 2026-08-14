use indexmap::IndexMap;
use phalcom_common::selector::{SelectorKindPattern, SelectorPattern};
use phalcom_core::bytecode::{Bytecode, FamilySpecKind};
use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::error::{PhError, RuntimeError};
use phalcom_core::heap::{BoundMethodFamilyObject, InstanceObject, MethodFamilyObject, Object, SelectorPatternObject};
use phalcom_core::method::{MethodObject, SignatureKind};
use phalcom_core::primitive::block::block_call;
use phalcom_core::primitive::class::{behavior_extract, class_new_};
use phalcom_core::primitive::method::method_bind;
use phalcom_core::primitive::method_family::{method_family_bind, method_family_method_for, method_family_selectors, method_family_size};
use phalcom_core::primitive::object::object_method_for;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

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
fn immediately_called_exact_method_ref_uses_direct_send_shape() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<family-specialization>");
    let closure = vm
        .compile_closure(module, "let result = 1::future(_)(2)\n")
        .expect("immediate exact MethodRef compiles");
    let selector = vm.get_or_intern("future(_)");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(
        !chunk.code.iter().any(|opcode| matches!(opcode, Bytecode::MakeFamily { .. })),
        "specialized call must not allocate Family"
    );
    assert!(chunk.code.iter().any(|opcode| {
        let Bytecode::Invoke(1, selector_idx) = opcode else {
            return false;
        };
        matches!(chunk.constants[*selector_idx as usize], Value::Symbol(symbol) if symbol == selector)
    }));
}

#[test]
fn behavior_pattern_extraction_snapshots_effective_exact_methods() {
    fn replacement(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> phalcom_core::error::PhResult<Value> {
        Ok(Value::Int(9))
    }

    let mut vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    let pattern = vm.heap.alloc(Object::SelectorPattern(Box::new(phalcom_core::heap::SelectorPatternObject {
        pattern: SelectorPattern::named(
            "name",
            SelectorKindPattern::AnyNamed,
            Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
            Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
            true,
        )
        .expect("valid pattern"),
    })));

    let first = behavior_extract(&mut vm, &Value::Obj(object_class), &[Value::Obj(pattern)]).expect("pattern extraction");
    let Value::Obj(first_id) = first else {
        panic!("pattern extraction must return a MethodFamily")
    };
    let old_method = vm
        .heap
        .method_family(first_id)
        .exact_methods
        .values()
        .next()
        .copied()
        .expect("Object defines name");

    let selector = vm.get_or_intern("name");
    let replacement_method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_primitive(
        selector,
        SignatureKind::Getter,
        replacement,
        object_class,
    ))));
    vm.heap.class_mut(object_class).add_method(selector, replacement_method);

    let second = behavior_extract(&mut vm, &Value::Obj(object_class), &[Value::Obj(pattern)]).expect("second pattern extraction");
    let Value::Obj(second_id) = second else {
        panic!("second extraction must return a MethodFamily")
    };
    assert_eq!(vm.heap.method_family(first_id).exact_methods.get(&selector), Some(&old_method));
    assert_eq!(vm.heap.method_family(second_id).exact_methods.get(&selector), Some(&replacement_method));
}

#[test]
fn method_family_bind_captures_receiver_without_live_selection() {
    let mut vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    let pattern = vm.heap.alloc(Object::SelectorPattern(Box::new(phalcom_core::heap::SelectorPatternObject {
        pattern: SelectorPattern::named(
            "name",
            SelectorKindPattern::AnyNamed,
            Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
            Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
            true,
        )
        .expect("valid pattern"),
    })));
    let family = behavior_extract(&mut vm, &Value::Obj(object_class), &[Value::Obj(pattern)]).expect("pattern extraction");
    let bound = method_family_bind(&mut vm, &family, &[Value::Int(42)]).expect("family binding");
    let Value::Obj(bound_id) = bound else {
        panic!("binding must return BoundMethodFamily")
    };
    let Object::BoundMethodFamily(bound) = vm.heap.get(bound_id) else {
        panic!("wrong bound-family heap variant")
    };
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
        matches!(&result, Err(PhError::Runtime(RuntimeError::IncompatibleMethodLayout { selector, .. })) if selector == "read"),
        "foreign field access should fail with layout error, got {result:?}"
    );
}

#[test]
fn captured_method_nested_native_block_preserves_foreign_layout_guard() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "captured_method_nested_native_block");
    vm.interpret_source(
        module,
        "class Source { @constructor new() { _field = 41 } read { true.ifTrue { _field } } }\nclass Target { @constructor new() { _other = 99 } }\nlet source = Source.new()\nlet target = Target.new()\n",
    )
    .expect("nested block fixture should compile");

    let source = vm.heap.module(module).get(vm.interner.intern("source")).expect("source should exist");
    let target = vm.heap.module(module).get(vm.interner.intern("target")).expect("target should exist");
    let read_selector = vm.get_or_intern("read");
    let method = object_method_for(&mut vm, &source, &[Value::Symbol(read_selector)]).expect("method should exist");
    let bound = method_bind(&mut vm, &method, &[target]).expect("foreign receiver should bind");
    let result = block_call(&mut vm, &bound, &[]);
    assert!(matches!(result, Err(PhError::Runtime(RuntimeError::IncompatibleMethodLayout { selector, .. })) if selector == "read"));
}

#[test]
fn captured_method_allows_subclass_layout_and_lexical_super() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "captured_method_subclass_layout_and_lexical_super");
    vm.interpret_source(
        module,
        "class Parent { value { 7 } }\nclass Source is Parent { @constructor new() { _field = 41 } read { _field } viaSuper { super.value } }\nclass Child is Source { @constructor new() { _child = 42 } }\nlet source = Source.new()\nlet child = Child.new()\n",
    )
    .expect("parent and subclass classes should compile");

    let source = vm.heap.module(module).get(vm.interner.intern("source")).expect("source should exist");
    let child = vm.heap.module(module).get(vm.interner.intern("child")).expect("child should exist");

    let read_selector = vm.get_or_intern("read");
    let read_method = object_method_for(&mut vm, &source, &[Value::Symbol(read_selector)]).expect("read method should exist");
    let bound_read = method_bind(&mut vm, &read_method, &[child]).expect("subclass receiver should bind");
    assert!(block_call(&mut vm, &bound_read, &[]).is_ok(), "subclass field access should succeed");

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

#[test]
fn method_family_reflection_exposes_snapshot_routes_without_allocation_access() {
    let mut vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    let pattern = vm.heap.alloc(Object::SelectorPattern(Box::new(SelectorPatternObject {
        pattern: SelectorPattern::named(
            "name",
            SelectorKindPattern::AnyNamed,
            Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
            Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
            true,
        )
        .expect("valid pattern"),
    })));
    let family = behavior_extract(&mut vm, &Value::Obj(object_class), &[Value::Obj(pattern)]).expect("pattern extraction");

    let size = method_family_size(&mut vm, &family, &[]).expect("size should be readable");
    let selectors = method_family_selectors(&mut vm, &family, &[]).expect("selectors should be readable");
    let selector_values = match selectors {
        Value::Obj(id) => vm.heap.list(id).elements().to_vec(),
        other => panic!("selectors should return List, got {other:?}"),
    };
    assert_eq!(size, Value::Int(selector_values.len() as i64));
    assert!(selector_values.contains(&Value::Symbol(vm.get_or_intern("name"))));

    let name_selector = vm.get_or_intern("name");
    let method = method_family_method_for(&mut vm, &family, &[Value::Symbol(name_selector)]).expect("methodFor should be readable");
    assert!(matches!(method, Value::Obj(id) if matches!(vm.heap.get(id), Object::Method(_))));

    let method_family_class = Value::Obj(vm.universe.classes.method_family_class);
    assert!(matches!(
        class_new_(&mut vm, &method_family_class, &[]),
        Err(PhError::Runtime(RuntimeError::Type { .. }))
    ));
    let bound_method_family_class = Value::Obj(vm.universe.classes.bound_method_family_class);
    assert!(matches!(
        class_new_(&mut vm, &bound_method_family_class, &[]),
        Err(PhError::Runtime(RuntimeError::Type { .. }))
    ));
}

#[test]
fn any_named_bound_family_prefers_method_shape_over_accessor_shapes() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "any_named_bound_family_shapes");
    vm.interpret_source(
        module,
        "class Source { name { 1 } name() { 2 } name=(put value) { 3 } name(value) { 4 } }\nlet source = Source.new()\n",
    )
    .expect("accessor and method overloads should compile");
    let source = vm.heap.module(module).get(vm.interner.intern("source")).expect("source should exist");
    let source_class = vm.heap.module(module).get(vm.interner.intern("Source")).expect("Source class should exist");
    let pattern = vm.heap.alloc(Object::SelectorPattern(Box::new(SelectorPatternObject {
        pattern: SelectorPattern::named(
            "name",
            SelectorKindPattern::AnyNamed,
            Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
            Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
            true,
        )
        .expect("valid pattern"),
    })));
    let family = behavior_extract(&mut vm, &source_class, &[Value::Obj(pattern)]).expect("pattern extraction");
    let bound = method_family_bind(&mut vm, &family, &[source]).expect("family should bind");
    assert_eq!(block_call(&mut vm, &bound, &[]).expect("nullary method call"), Value::Int(2));
    assert_eq!(block_call(&mut vm, &bound, &[Value::Int(9)]).expect("one-argument method call"), Value::Int(4));
}

#[test]
fn exact_setter_family_accepts_family_set_shape() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "exact_setter_family_shape");
    vm.interpret_source(
        module,
        "class Source { @constructor new() { _name = 1 } name { _name } name=(put value) { _name = value } }\nlet source = Source.new()\nlet setter = source::name=(put)\nsetter.set(42)\nlet result = source.name\n",
    )
    .expect("exact setter Family should accept Family#set shape");
    let result = vm.heap.module(module).get(vm.interner.intern("result")).expect("result should exist");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn method_family_and_bound_receiver_are_gc_edges() {
    fn constant(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> phalcom_core::error::PhResult<Value> {
        Ok(Value::Int(1))
    }

    let mut vm = VM::new();
    vm.force_gc();
    let object_class = vm.universe.classes.object_class;
    let selector = vm.get_or_intern("captured");
    let pattern = vm.heap.alloc(Object::SelectorPattern(Box::new(SelectorPatternObject {
        pattern: SelectorPattern::named(
            "captured",
            SelectorKindPattern::AnyNamed,
            Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
            Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
            true,
        )
        .expect("valid pattern"),
    })));
    let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_primitive(
        selector,
        SignatureKind::Getter,
        constant,
        object_class,
    ))));
    let mut exact_methods = IndexMap::new();
    exact_methods.insert(selector, method);
    let family = vm.heap.alloc(Object::MethodFamily(Box::new(MethodFamilyObject {
        source_behavior: object_class,
        pattern,
        exact_methods,
        rest_candidates: Box::new([]),
    })));
    let receiver = vm.heap.alloc(Object::Instance(InstanceObject::new(object_class, 0)));
    let bound = vm.heap.alloc(Object::BoundMethodFamily(BoundMethodFamilyObject {
        family,
        receiver: Value::Obj(receiver),
    }));

    vm.push_root_for_test(Value::Obj(bound));
    vm.force_gc();
    assert!(vm.heap.try_get(family).is_some());
    assert!(vm.heap.try_get(method).is_some());
    assert!(vm.heap.try_get(pattern).is_some());
    assert!(vm.heap.try_get(receiver).is_some());

    vm.pop_root_for_test();
    vm.force_gc();
    assert!(vm.heap.try_get(bound).is_none());
    assert!(vm.heap.try_get(family).is_none());
    assert!(vm.heap.try_get(method).is_none());
    assert!(vm.heap.try_get(pattern).is_none());
    assert!(vm.heap.try_get(receiver).is_none());
}
