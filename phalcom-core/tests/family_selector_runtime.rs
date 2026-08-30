use indexmap::IndexMap;
use phalcom_common::selector::{SelectorKindPattern, SelectorPattern};
use phalcom_core::bytecode::{Bytecode, FamilySpecKind};
use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::error::{PhError, RuntimeError};
use phalcom_core::heap::{BoundMethodFamilyObject, InstanceObject, MethodFamilyObject, Object};
use phalcom_core::method::{MethodObject, SignatureKind};
use phalcom_core::primitive::block::block_call;
use phalcom_core::primitive::class::{behavior_extract, class_new_};
use phalcom_core::primitive::method::method_bind;
use phalcom_core::primitive::method_family::{method_family_bind, method_family_method_for, method_family_selectors, method_family_size};
use phalcom_core::primitive::object::object_method_for;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

#[test]
#[ignore = "associated lowering scheduled for Part 3/4"]
fn make_family_uses_explicit_exact_discriminator_and_allows_future_method() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<family>");
    let closure = vm
        .compile_closure_as(module, "const f = 1::future::()\n", UnitKind::File)
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
#[ignore = "associated lowering scheduled for Part 3/4"]
fn make_family_compiles_pattern_object_without_punctuation_heuristic() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<family>");
    let closure = vm
        .compile_closure_as(module, "const f = 1::future::*\n", UnitKind::File)
        .expect("pattern family compiles");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    let pattern = chunk.constants.iter().find_map(|constant| {
        if let Some(id) = constant.as_obj() {
            if matches!(vm.heap.get(id), Object::SelectorPattern(_)) {
                return Some(id);
            }
        }
        None
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
#[ignore = "associated lowering scheduled for Part 3/4"]
fn family_pattern_mismatch_returns_typed_error_before_dispatch() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "family-pattern-mismatch");
    let closure = vm
        .compile_closure(
            module,
            "class Router { route() { 0 } route(_ value) { value } }\nconst family = Router.new()::route::*\nfamily()\n",
        )
        .expect("family mismatch fixture compiles");

    let error = vm.run_in_module(module, closure).expect_err("mismatched family call must fail");
    let PhError::Runtime(RuntimeError::SelectorPatternMismatch(ctx)) = error else {
        panic!("expected typed selector-pattern mismatch, got {error:?}");
    };
    let pattern = ctx.pattern;
    let selector = ctx.selector;
    let family_id = ctx.family.as_obj().expect("expected family obj");
    let receiver_id = ctx.receiver.as_obj().expect("expected receiver obj");

    assert_eq!(pattern.encode(), "route(_, ...)");
    assert_eq!(selector.encode(), "route()");
    assert!(matches!(vm.heap.get(family_id), Object::Family(_)));
    assert!(matches!(vm.heap.get(receiver_id), Object::Instance(_)));
}

#[test]
#[ignore = "associated lowering scheduled for Part 3/4"]
fn immediately_called_exact_method_ref_uses_direct_send_shape() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<family-specialization>");
    let closure = vm
        .compile_closure(module, "let result = 1::future(2)\n")
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
        chunk.constants[*selector_idx as usize].as_symbol().ok() == Some(selector)
    }));
}

#[test]
fn behavior_pattern_extraction_snapshots_effective_exact_methods() {
    fn replacement(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> phalcom_core::error::PhResult<Value> {
        Ok(Value::int(9))
    }

    let mut vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    let pattern_value = SelectorPattern::named(
        "name",
        SelectorKindPattern::AnyNamed,
        Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
        Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
        true,
    )
    .expect("valid pattern");
    let pattern = vm.alloc_selector_pattern(pattern_value);

    let first = behavior_extract(&mut vm, &Value::obj(object_class), &[Value::obj(pattern)]).expect("pattern extraction");
    let first_id = first.as_obj().expect("pattern extraction must return a MethodFamily");
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

    let second = behavior_extract(&mut vm, &Value::obj(object_class), &[Value::obj(pattern)]).expect("second pattern extraction");
    let second_id = second.as_obj().expect("second extraction must return a MethodFamily");
    assert_eq!(vm.heap.method_family(first_id).exact_methods.get(&selector), Some(&old_method));
    assert_eq!(vm.heap.method_family(second_id).exact_methods.get(&selector), Some(&replacement_method));
}

#[test]
fn method_family_bind_captures_receiver_without_live_selection() {
    let mut vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    let pattern_value = SelectorPattern::named(
        "name",
        SelectorKindPattern::AnyNamed,
        Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
        Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
        true,
    )
    .expect("valid pattern");
    let pattern = vm.alloc_selector_pattern(pattern_value);
    let family = behavior_extract(&mut vm, &Value::obj(object_class), &[Value::obj(pattern)]).expect("pattern extraction");
    let bound = method_family_bind(&mut vm, &family, &[Value::int(42)]).expect("family binding");
    let bound_id = bound.as_obj().expect("binding must return BoundMethodFamily");
    let Object::BoundMethodFamily(bound) = vm.heap.get(bound_id) else {
        panic!("wrong bound-family heap variant")
    };
    assert_eq!(bound.family, family.as_obj().expect("family handle"));
    assert_eq!(bound.receiver, Value::int(42));
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
    let method = object_method_for(&mut vm, &source, &[Value::symbol(label_selector)]).expect("method should exist");
    let bound = method_bind(&mut vm, &method, &[target]).expect("foreign receiver should bind");
    assert_eq!(block_call(&mut vm, &bound, &[]).expect("foreign method should activate"), Value::int(42));
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
    let method = object_method_for(&mut vm, &source, &[Value::symbol(read_selector)]).expect("method should exist");
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
    let method = object_method_for(&mut vm, &source, &[Value::symbol(read_selector)]).expect("method should exist");
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
    let read_method = object_method_for(&mut vm, &source, &[Value::symbol(read_selector)]).expect("read method should exist");
    let bound_read = method_bind(&mut vm, &read_method, &[child]).expect("subclass receiver should bind");
    assert!(block_call(&mut vm, &bound_read, &[]).is_ok(), "subclass field access should succeed");

    let super_selector = vm.get_or_intern("viaSuper");
    let super_method = object_method_for(&mut vm, &source, &[Value::symbol(super_selector)]).expect("super method should exist");
    let bound_super = method_bind(&mut vm, &super_method, &[child]).expect("foreign receiver should bind");
    assert_eq!(block_call(&mut vm, &bound_super, &[]).expect("lexical super should succeed"), Value::int(7));
}

#[test]
fn captured_primitive_method_accepts_foreign_receiver() {
    fn constant(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> phalcom_core::error::PhResult<Value> {
        Ok(Value::int(17))
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
    let method_value = Value::obj(method);
    let bound = method_bind(&mut vm, &method_value, &[Value::int(3)]).expect("primitive method should bind");
    assert_eq!(block_call(&mut vm, &bound, &[]).expect("primitive method should activate"), Value::int(17));
}

#[test]
fn method_family_reflection_exposes_snapshot_routes_without_allocation_access() {
    let mut vm = VM::new();
    let object_class = vm.universe.classes.object_class;
    let pattern_value = SelectorPattern::named(
        "name",
        SelectorKindPattern::AnyNamed,
        Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
        Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
        true,
    )
    .expect("valid pattern");
    let pattern = vm.alloc_selector_pattern(pattern_value);
    let family = behavior_extract(&mut vm, &Value::obj(object_class), &[Value::obj(pattern)]).expect("pattern extraction");

    let size = method_family_size(&mut vm, &family, &[]).expect("size should be readable");
    let selectors = method_family_selectors(&mut vm, &family, &[]).expect("selectors should be readable");
    let selector_values = if let Some(id) = selectors.as_obj() {
        vm.heap.list(id).elements().to_vec()
    } else {
        panic!("selectors should return List, got {selectors:?}")
    };
    assert_eq!(size, Value::int(selector_values.len() as i64));
    assert!(selector_values.contains(&Value::symbol(vm.get_or_intern("name"))));

    let name_selector = vm.get_or_intern("name");
    let method = method_family_method_for(&mut vm, &family, &[Value::symbol(name_selector)]).expect("methodFor should be readable");
    assert!(matches!(method.as_obj(), Some(id) if matches!(vm.heap.get(id), Object::Method(_))));

    let method_family_class = Value::obj(vm.universe.classes.method_family_class);
    assert!(matches!(
        class_new_(&mut vm, &method_family_class, &[]),
        Err(PhError::Runtime(RuntimeError::Type { .. }))
    ));
    let bound_method_family_class = Value::obj(vm.universe.classes.bound_method_family_class);
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
        "class Source { name { 1 } name() { 2 } name=(put value) { 3 } name(_ value) { 4 } }\nlet source = Source.new()\nlet family = Source >> #name(...)\nlet bound = family.bind(source)\nlet nullary = bound()\nlet unary = bound(9)\n",
    )
    .expect("accessor and method overloads should compile");
    let nullary = vm.heap.module(module).get(vm.interner.intern("nullary")).expect("nullary result should exist");
    let unary = vm.heap.module(module).get(vm.interner.intern("unary")).expect("unary result should exist");
    assert_eq!(nullary, Value::int(2));
    assert_eq!(unary, Value::int(4));
}

#[test]
#[ignore = "associated lowering scheduled for Part 3/4"]
fn exact_setter_family_accepts_family_set_shape() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "exact_setter_family_shape");
    vm.interpret_source(
        module,
        "class Source { @constructor new() { _name = 1 } name { _name } name=(put value) { _name = value } }\nlet source = Source.new()\nlet setter = source::name=(put)\nsetter.set(42)\nlet result = source.name\n",
    )
    .expect("exact setter Family should accept Family#set shape");
    let result = vm.heap.module(module).get(vm.interner.intern("result")).expect("result should exist");
    assert_eq!(result, Value::int(42));
}

#[test]
fn method_family_and_bound_receiver_are_gc_edges() {
    fn constant(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> phalcom_core::error::PhResult<Value> {
        Ok(Value::int(1))
    }

    let mut vm = VM::new();
    vm.force_gc();
    let object_class = vm.universe.classes.object_class;
    let selector = vm.get_or_intern("captured");
    let pattern_value = SelectorPattern::named(
        "captured",
        SelectorKindPattern::AnyNamed,
        Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
        Vec::<phalcom_common::selector::SelectorSlot>::new().into_boxed_slice(),
        true,
    )
    .expect("valid pattern");
    let pattern = vm.alloc_selector_pattern(pattern_value);
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
        receiver: Value::obj(receiver),
    }));

    vm.push_root_for_test(Value::obj(bound));
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
