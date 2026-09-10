use indexmap::IndexMap;
use phalcom_common::selector::{SelectorKindPattern, SelectorPattern};
use phalcom_core::bytecode::{Bytecode, FamilySpecKind};
use phalcom_core::error::{PhError, RuntimeError};
use phalcom_core::heap::{BoundMethodFamilyObject, InstanceObject, MethodFamilyObject, Object};
use phalcom_core::method::{MethodObject, SignatureKind};
use phalcom_core::modules::ModuleFailure;
use phalcom_core::modules::compile::{CompiledProgram, EntrySelection, ProgramCompiler};
use phalcom_core::primitive::block::block_call;
use phalcom_core::primitive::class::{behavior_extract, class_new_};
use phalcom_core::primitive::method::method_bind;
use phalcom_core::primitive::method_family::{method_family_bind, method_family_method_for, method_family_selectors, method_family_size};
use phalcom_core::primitive::object::object_method_for;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::sync::Arc;

fn compile_inline(source: &str) -> Result<(VM, CompiledProgram, phalcom_core::heap::ObjRef), PhError> {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    let mut vm = VM::new();
    vm.materialize_program(&program)?;
    let closure = vm.compile_program_module_closure(&program.entry, source, &program)?;
    Ok((vm, program, closure))
}

#[test]
fn make_family_uses_explicit_exact_discriminator() {
    let source = "const f = &1.compare(_)\n";
    let (mut vm, program, closure) = compile_inline(source).expect("exact family compiles");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(chunk.code.iter().any(|opcode| matches!(
        opcode,
        Bytecode::MakeFamily {
            kind: FamilySpecKind::Exact,
            ..
        }
    )));
    vm.run_compiled(&program).expect("family construction does not resolve target method");
}

#[test]
fn make_family_compiles_pattern_object_without_punctuation_heuristic() {
    let source = "const f = &1.compare...\n";
    let (mut vm, program, closure) = compile_inline(source).expect("pattern family compiles");
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
    vm.run_compiled(&program).expect("pattern family construction succeeds");
}

#[test]
fn family_pattern_mismatch_returns_typed_error_before_dispatch() {
    let source = "class Router { route() { 0 } route(_ value) { value } }\nlet family = &Router.new().route(_, ...)\nfamily()\n";
    let (mut vm, program, _closure) = compile_inline(source).expect("family mismatch fixture compiles");

    let error = vm.run_compiled(&program).expect_err("mismatched family call must fail");
    let PhError::ModuleInitialization(initialization) = error else {
        panic!("expected module initialization envelope, got {error:?}");
    };
    let ModuleFailure::Initializer { cause } = initialization.failure.as_ref() else {
        panic!("expected initializer failure, got {:?}", initialization.failure);
    };
    let PhError::Runtime(RuntimeError::SelectorPatternMismatch(ctx)) = cause.as_ref() else {
        panic!("expected typed selector-pattern mismatch, got {cause:?}");
    };
    let pattern = &ctx.pattern;
    let selector = &ctx.selector;
    let family_id = ctx.family.as_obj().expect("expected family obj");
    let receiver_id = ctx.receiver.as_obj().expect("expected receiver obj");

    assert_eq!(pattern.encode(), "route(_, ...)");
    assert_eq!(selector.encode(), "route()");
    assert!(matches!(vm.heap.get(family_id), Object::Family(_)));
    assert!(matches!(vm.heap.get(receiver_id), Object::Instance(_)));
}

#[test]
fn immediately_called_exact_method_ref_keeps_family_shape() {
    let source = "let result = (&1.compare(_))(2)\n";
    let (vm, _program, closure) = compile_inline(source).expect("immediate exact callable reference compiles");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(
        chunk.code.iter().any(|opcode| matches!(
            opcode,
            Bytecode::MakeFamily {
                kind: FamilySpecKind::Exact,
                ..
            }
        )),
        "bound exact references must retain Family construction"
    );
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
        "class Source { name { 1 } name() { 2 } name=(_ value) { 3 } name(_ value) { 4 } }\nlet source = Source.new()\nlet family = Source >> #name(...)\nlet bound = family.bind(source)\nlet nullary = bound()\nlet unary = bound(9)\n",
    )
    .expect("accessor and method overloads should compile");
    let nullary = vm.heap.module(module).get(vm.interner.intern("nullary")).expect("nullary result should exist");
    let unary = vm.heap.module(module).get(vm.interner.intern("unary")).expect("unary result should exist");
    assert_eq!(nullary, Value::int(2));
    assert_eq!(unary, Value::int(4));
}

#[test]
fn bound_subscript_family_activates_get_and_set_lanes() {
    let source = "class Table { [_ index] { index } [_ index]=(_ value) { value } }\nlet table = Table.new()\nlet family = &table[_]\nlet family_set = &table[_]=(_)\nlet got = family[3]\nlet stored = family_set[3] = 9\n";
    let (mut vm, program, _closure) = compile_inline(source).expect("bound subscript family should compile");
    vm.run_compiled(&program).expect("bound subscript family should dispatch both lanes");
    let module = vm.module_registry.get(&program.entry).expect("entry module should be materialized").object;
    let got = vm.heap.module(module).get(vm.interner.intern("got")).expect("getter result should exist");
    let stored = vm.heap.module(module).get(vm.interner.intern("stored")).expect("setter result should exist");
    assert_eq!(got, Value::int(3));
    assert_eq!(stored, Value::int(9));
}

#[test]
fn family_accessor_aliases_activate_named_getter_and_setter_lanes() {
    let source = "class Box { value { 7 } value=(_ newValue) { newValue } }\nlet box = Box.new()\nlet getter = &box.value\nlet setter = &box.value=(_)\nlet getCall = getter.get()\nlet getValue = getter.value\nlet setCall = setter.set(9)\nlet setValue = setter.value = 11\n";
    let (mut vm, program, _closure) = compile_inline(source).expect("family accessor aliases should compile");
    vm.run_compiled(&program).expect("family accessor aliases should activate");
    let module = vm.module_registry.get(&program.entry).expect("entry module should be materialized").object;
    for (name, expected) in [("getCall", 7), ("getValue", 7), ("setCall", 9), ("setValue", 11)] {
        let actual = vm.heap.module(module).get(vm.interner.intern(name)).expect("family result should exist");
        assert_eq!(actual, Value::int(expected), "unexpected {name} result");
    }
}

#[test]
fn family_subscript_tuple_apis_preserve_index_shape_and_rhs_lane() {
    let source = "class Table { [_ index] { index } [_ index]=(_ value) { value } }\nlet table = Table.new()\nlet getter = &table[_]\nlet setter = &table[_]=(_)\nlet getValue = getter.get((3,))\nlet setValue = setter.set((4,), 12)\n";
    let (mut vm, program, _closure) = compile_inline(source).expect("family subscript APIs should compile");
    vm.run_compiled(&program).expect("family subscript APIs should activate");
    let module = vm.module_registry.get(&program.entry).expect("entry module should be materialized").object;
    let get_value = vm
        .heap
        .module(module)
        .get(vm.interner.intern("getValue"))
        .expect("subscript getter result should exist");
    let set_value = vm
        .heap
        .module(module)
        .get(vm.interner.intern("setValue"))
        .expect("subscript setter result should exist");
    assert_eq!(get_value, Value::int(3));
    assert_eq!(set_value, Value::int(12));
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
