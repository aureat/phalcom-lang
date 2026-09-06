use phalcom_core::{bytecode::Bytecode, chunk::{Chunk, InlineCache, GlobalCache}, frame::CallFrame, heap::{ClassObject, Object}, method::{MethodObject, SignatureKind}, value::Value, vm::VM};
use std::{cell::Cell, panic::{catch_unwind, AssertUnwindSafe}, rc::Rc};

fn recurse(vm: &mut VM, receiver: &Value, args: &[Value]) -> phalcom_core::error::PhResult<Value> {
    let n = args[0].as_int().unwrap();
    if n == 0 { return Ok(Value::int(40)); }
    let selector = vm.get_or_intern("auditRecurse(_)");
    vm.send_dynamic(*receiver, selector, &[Value::int(n - 1)])
}

fn main() {
    println!("LAYOUT value={} align={} opcode={} frame={} ic={} gc={}", size_of::<Value>(), align_of::<Value>(), size_of::<Bytecode>(), size_of::<CallFrame>(), size_of::<Cell<Option<InlineCache>>>(), size_of::<Cell<Option<GlobalCache>>>());
    let mut chunk = Chunk::new();
    for i in 0..=65536 { let index = chunk.add_constant(Value::int(i)); if [0,1,65534,65535,65536].contains(&i) { println!("CONSTANT input={i} index={index} selected={:?}", chunk.constants[index as usize].as_int()); } }
    let mut vm = VM::new_kernel();
    let module = vm.create_module("audit", "");
    let closure = vm.compile_closure(module, "42\n").expect("kernel literal compile");
    let owner = vm.heap.alloc_class(ClassObject::bare("AuditOwner"));
    vm.heap.closure_mut(closure).lexical_class = Some(owner);
    vm.push_root_for_test(Value::obj(closure));
    vm.force_gc();
    println!("GC lexical_owner_survives={}", vm.heap.try_get(owner).is_some());
    vm.pop_root_for_test();
    let class = vm.universe.classes.int_class;
    let selector = vm.get_or_intern("auditRecurse(_)");
    let method = vm.heap.alloc(Object::Method(Box::new(MethodObject::new_primitive(selector, SignatureKind::Method(1), recurse, class))));
    vm.heap.class_mut(class).add_method(selector, method);
    println!("NATIVE recursion_40={:?}", vm.send_dynamic(Value::int(1), selector, &[Value::int(40)]));
    let bad_arity = catch_unwind(AssertUnwindSafe(|| vm.send_dynamic(Value::int(1), selector, &[])));
    println!("NATIVE wrong_arity_panics={}", bad_arity.is_err());
    vm.unwind_cell();
    let empty = vm.compile_closure(module, "42\n").unwrap();
    Rc::make_mut(&mut vm.heap.closure_mut(empty).callable).chunk = Chunk::new();
    let result = catch_unwind(AssertUnwindSafe(|| vm.run_in_module(module, empty)));
    println!("BYTECODE empty_chunk_panics={}", result.is_err());
    let mut vm = VM::new_kernel();
    let module = vm.create_module("large", "");
    let source = (0..=65536).map(|n| format!("{n}\n")).collect::<String>();
    let closure = vm.compile_closure(module, &source).unwrap();
    let c = &vm.heap.closure(closure).callable.chunk;
    let last = c.code.iter().rev().find_map(|op| if let Bytecode::Constant(i) = op {Some(*i)} else {None}).unwrap();
    println!("SOURCE constants={} last_index={} last_value={:?}", c.constants.len(), last, c.constants[last as usize].as_int());
    let result = vm.run_cell(module, closure);
    println!("SOURCE execution={result:?}");
    let mut vm = VM::new_native();
    let selector = vm.get_or_intern("+(_)");
    let result = catch_unwind(AssertUnwindSafe(|| vm.send_dynamic(Value::int(1), selector, &[])));
    println!("SHIPPING_NATIVE wrong_arity_panics={}", result.is_err());
    let mut vm = VM::new();
    let module = vm.create_module("repl_audit", "");
    let src = "class A {\n @constructor\n new() {}\n value { 1 }\n}\nclass B is A {\n value { super.value }\n}\nlet b = B.new()\n";
    let cl = vm.compile_closure_as(module, src, phalcom_core::compiler::lib::UnitKind::Repl).unwrap();
    vm.run_cell(module, cl).unwrap();
    let b_sym = vm.get_or_intern("b");
    let b = vm.heap.module(module).globals[vm.heap.module(module).slot_of(b_sym).unwrap()];
    let value = vm.get_or_intern("value");
    println!("SUPER before={:?}", vm.send_dynamic(b, value, &[]));
    let cl = vm.compile_closure_as(module, "class C { value { 2 } }\nclass B is C { value { super.value } }\n", phalcom_core::compiler::lib::UnitKind::Repl).unwrap();
    vm.run_cell(module, cl).unwrap();
    println!("SUPER old_receiver_after_rebind={:?}", vm.send_dynamic(b, value, &[]));
}
