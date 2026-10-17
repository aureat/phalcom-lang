use phalcom_core::bytecode::Bytecode;
use phalcom_core::heap::{ObjRef, Object};
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn global(vm: &VM, module: ObjRef, name: &str) -> Value {
    vm.heap
        .module(module)
        .get(vm.interner.find(name).expect("interned global"))
        .expect("global value")
}

fn send0(vm: &mut VM, receiver: Value, selector: &str) -> Value {
    let sym = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, sym, &[]).unwrap_or_else(|_| panic!("send `{selector}` failed"))
}

fn send1(vm: &mut VM, receiver: Value, selector: &str, arg: Value) -> Value {
    let sym = vm.get_or_intern(selector);
    vm.send_dynamic(receiver, sym, &[arg])
        .unwrap_or_else(|err| panic!("send `{selector}` failed: {err:?}"))
}

fn eval_expr(vm: &mut VM, source: &str) -> Value {
    let module = vm.create_module("array-tests", "array-tests");
    let full = format!("let result = {source}\n");
    let closure = vm.compile_closure(module, &full).expect("expression compiles");
    vm.run_in_module(module, closure).expect("expression runs");
    let sym = vm.get_or_intern("result");
    vm.heap.module(module).get(sym).expect("binds result")
}

fn run_script(vm: &mut VM, source: &str) -> ObjRef {
    let module = vm.create_module("array-tests", "array-tests");
    let closure = vm.compile_closure(module, source).expect("script compiles");
    vm.run_in_module(module, closure).expect("script runs");
    module
}

fn is_none(val: Value) -> bool {
    val.is_none()
}

#[test]
fn array_runtime_class_and_fixed_size() {
    let mut vm = VM::new();
    let arr = eval_expr(&mut vm, "<Int>[10, 20, 30]");
    let class = arr.class(&vm);
    assert_eq!(class, vm.universe.classes.array_class);

    let size = send0(&mut vm, arr, "size");
    assert_eq!(size.as_int(), Some(3));
}

#[test]
fn array_element_read_and_subscript() {
    let mut vm = VM::new();
    let arr = eval_expr(&mut vm, "<Int>[10, 20, 30]");

    assert_eq!(send1(&mut vm, arr, "at(_)", Value::int(0)).as_int(), Some(10));
    assert_eq!(send1(&mut vm, arr, "at(_)", Value::int(1)).as_int(), Some(20));
    assert_eq!(send1(&mut vm, arr, "at(_)", Value::int(2)).as_int(), Some(30));

    // Subscript [_]
    assert_eq!(send1(&mut vm, arr, "[_]", Value::int(0)).as_int(), Some(10));
    assert_eq!(send1(&mut vm, arr, "[_]", Value::int(1)).as_int(), Some(20));
    assert_eq!(send1(&mut vm, arr, "[_]", Value::int(2)).as_int(), Some(30));

    // Negative indexing
    assert_eq!(send1(&mut vm, arr, "[_]", Value::int(-1)).as_int(), Some(30));
    assert_eq!(send1(&mut vm, arr, "[_]", Value::int(-2)).as_int(), Some(20));

    // Out of bounds raw/at returns None
    let oob = send1(&mut vm, arr, "at(_)", Value::int(3));
    assert!(is_none(oob));
}

#[test]
fn array_element_mutation_preserves_size() {
    let mut vm = VM::new();
    let script = r#"
let a = <Int>[10, 20, 30]
a[1] = 99
let finalSize = a.size
let elem1 = a[1]
"#;
    let module = run_script(&mut vm, script);
    let final_size_sym = vm.get_or_intern("finalSize");
    let elem1_sym = vm.get_or_intern("elem1");
    let final_size = vm.heap.module(module).get(final_size_sym).unwrap();
    let elem1 = vm.heap.module(module).get(elem1_sym).unwrap();
    assert_eq!(final_size.as_int(), Some(3));
    assert_eq!(elem1.as_int(), Some(99));
}

#[test]
fn array_has_no_resize_selectors() {
    let mut vm = VM::new();
    let arr = eval_expr(&mut vm, "<Int>[10, 20, 30]");

    let sym_append = vm.get_or_intern("append(_)");
    assert!(vm.send_dynamic(arr, sym_append, &[Value::int(40)]).is_err());

    let sym_insert = vm.get_or_intern("insert(_,at:)");
    assert!(vm.send_dynamic(arr, sym_insert, &[Value::int(40), Value::int(1)]).is_err());

    let sym_remove = vm.get_or_intern("remove(at:)");
    assert!(vm.send_dynamic(arr, sym_remove, &[Value::int(1)]).is_err());
}

#[test]
fn array_structural_equality() {
    let mut vm = VM::new();
    let a1 = eval_expr(&mut vm, "<Int>[1, 2, 3]");
    let a2 = eval_expr(&mut vm, "<Int>[1, 2, 3]");
    let a3 = eval_expr(&mut vm, "<Int>[1, 2, 4]");
    let a4 = eval_expr(&mut vm, "<Int>[1, 2]");
    let list = eval_expr(&mut vm, "List<Int>.new()");

    // Same elements -> equal
    let eq = send1(&mut vm, a1, "==(_)", a2);
    assert_eq!(eq, Value::bool(true));

    // Different element -> unequal
    let eq_diff = send1(&mut vm, a1, "==(_)", a3);
    assert_eq!(eq_diff, Value::bool(false));

    // Different size -> unequal
    let eq_size = send1(&mut vm, a1, "==(_)", a4);
    assert_eq!(eq_size, Value::bool(false));

    // Cross-family (Array == List) -> false
    let eq_cross = send1(&mut vm, a1, "==(_)", list);
    assert_eq!(eq_cross, Value::bool(false));

    // != routes through ==
    let neq = send1(&mut vm, a1, "!=(_)", a2);
    assert_eq!(neq, Value::bool(false));
    let neq_diff = send1(&mut vm, a1, "!=(_)", a3);
    assert_eq!(neq_diff, Value::bool(true));
}

#[test]
fn bare_array_literal_preserves_source_order_of_element_effects() {
    let source = r#"
let trace: List<Int> = []
class Probe {
    @class value(_ value: Int) -> Int {
        trace.append(value)
        value
    }
}
let values = [Probe.value(1), Probe.value(2)]
let first = trace[0]
let second = trace[1]
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(source.into())).expect("checked Array source");
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("materialize Array program");
    vm.run_compiled(&program).expect("run Array source");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(global(&vm, module, "first"), Value::int(1));
    assert_eq!(global(&vm, module, "second"), Value::int(2));
    let values = global(&vm, module, "values");
    assert_eq!(values.class(&vm), vm.universe.classes.array_class);
}

#[test]
fn array_iterator_and_live_view() {
    let mut vm = VM::new();
    let script = r#"
let a = <Int>[1, 2, 3]
let it = a.iterator
let first = it.next.match(some: |v| v, none: || 0)
a[1] = 42
let second = it.next.match(some: |v| v, none: || 0)
let third = it.next.match(some: |v| v, none: || 0)
let done = it.next
"#;
    let module = run_script(&mut vm, script);
    let first_sym = vm.get_or_intern("first");
    let second_sym = vm.get_or_intern("second");
    let third_sym = vm.get_or_intern("third");
    let done_sym = vm.get_or_intern("done");

    let first = vm.heap.module(module).get(first_sym).unwrap();
    let second = vm.heap.module(module).get(second_sym).unwrap();
    let third = vm.heap.module(module).get(third_sym).unwrap();
    let done = vm.heap.module(module).get(done_sym).unwrap();

    assert_eq!(first.as_int(), Some(1));
    assert_eq!(second.as_int(), Some(42)); // observed mutation in live view!
    assert_eq!(third.as_int(), Some(3));
    assert!(is_none(done));
}

#[test]
fn array_rejected_as_map_and_set_key() {
    let mut vm = VM::new();
    let script_map = r#"
let a = <Int>[1, 2, 3]
let m = Map.new()
m.put(a, 999)
"#;
    let module_map = vm.create_module("map-key-test", "map-key-test");
    let closure_map = vm.compile_closure(module_map, script_map).expect("script compiles");
    assert!(vm.run_in_module(module_map, closure_map).is_err(), "Map should reject Array as key");

    let script_set = r#"
let a = <Int>[1, 2, 3]
let s = Set.new()
s.add(a)
"#;
    let module_set = vm.create_module("set-key-test", "set-key-test");
    let closure_set = vm.compile_closure(module_set, script_set).expect("script compiles");
    assert!(vm.run_in_module(module_set, closure_set).is_err(), "Set should reject Array as key");
}

#[test]
fn array_spread_runtime_construction() {
    let mut vm = VM::new();
    let script = r#"
let base = <Int>[2, 3]
let expanded = <Int>[1, *base, 4]
let sz = expanded.size
let e0 = expanded[0]
let e1 = expanded[1]
let e2 = expanded[2]
let e3 = expanded[3]
"#;
    let module = run_script(&mut vm, script);
    let sz_sym = vm.get_or_intern("sz");
    let e0_sym = vm.get_or_intern("e0");
    let e1_sym = vm.get_or_intern("e1");
    let e2_sym = vm.get_or_intern("e2");
    let e3_sym = vm.get_or_intern("e3");

    let sz = vm.heap.module(module).get(sz_sym).unwrap();
    let e0 = vm.heap.module(module).get(e0_sym).unwrap();
    let e1 = vm.heap.module(module).get(e1_sym).unwrap();
    let e2 = vm.heap.module(module).get(e2_sym).unwrap();
    let e3 = vm.heap.module(module).get(e3_sym).unwrap();

    assert_eq!(sz.as_int(), Some(4));
    assert_eq!(e0.as_int(), Some(1));
    assert_eq!(e1.as_int(), Some(2));
    assert_eq!(e2.as_int(), Some(3));
    assert_eq!(e3.as_int(), Some(4));
}

#[test]
fn array_disassembly_has_direct_instructions() {
    let mut vm = VM::new();
    let module = vm.create_module("disasm-test", "disasm-test");

    let closure_fixed = vm.compile_closure(module, "let a = <Int>[1, 2, 3]\n").expect("compiles fixed");
    let ops_fixed = vm.heap.closure(closure_fixed).callable.chunk.code.clone();
    assert!(
        ops_fixed.iter().any(|op| matches!(op, Bytecode::BuildArray(3))),
        "fixed Array literal must emit BuildArray(3), got:\n{ops_fixed:?}"
    );
    assert!(
        !ops_fixed.iter().any(|op| matches!(op, Bytecode::BuildList(_))),
        "fixed Array literal must NOT emit BuildList, got:\n{ops_fixed:?}"
    );

    let closure_spread = vm
        .compile_closure(module, "class Container { @class test(rest: Array<Int>) { let a = <Int>[1, *rest, 3] } }\n")
        .expect("compiles spread");
    // Find the method closure in constants
    let mut found_spread_ops = false;
    for constant in vm.heap.closure(closure_spread).callable.chunk.constants.clone() {
        if let Some(obj) = constant.as_obj()
            && let Object::Method(method) = vm.heap.get(obj)
            && let phalcom_core::method::MethodKind::Closure(body) = method.kind
        {
            let ops = vm.heap.closure(body).callable.chunk.code.clone();
            if ops.iter().any(|op| matches!(op, Bytecode::BeginArrayLiteral)) {
                found_spread_ops = true;
                assert!(
                    ops.iter().any(|op| matches!(op, Bytecode::FinishArrayLiteral)),
                    "spread Array literal must emit FinishArrayLiteral, got: {ops:?}"
                );
                assert!(
                    !ops.iter().any(|op| matches!(op, Bytecode::BeginListLiteral)),
                    "spread Array literal must NOT emit BeginListLiteral, got: {ops:?}"
                );
            }
        }
    }
    assert!(found_spread_ops, "spread Array literal method body should be found in constants");
}

#[test]
fn array_gc_traces_contained_elements() {
    let mut vm = VM::new();
    // Allocate an Array containing a StringObject
    let script = r#"
let s = "hello_gc_trace"
let a = <String>[s]
"#;
    let module = run_script(&mut vm, script);
    let a_sym = vm.get_or_intern("a");
    let a_val = vm.heap.module(module).get(a_sym).unwrap();
    let a_ref = a_val.as_obj().unwrap();
    let str_val = vm.heap.array(a_ref).get(0).unwrap();
    let str_ref = str_val.as_obj().unwrap();

    // Trigger GC
    vm.force_gc();

    // Verify Array and String are both still alive
    assert!(matches!(vm.heap.get(a_ref), Object::Array(_)));
    assert!(matches!(vm.heap.get(str_ref), Object::Str(_)));
}
