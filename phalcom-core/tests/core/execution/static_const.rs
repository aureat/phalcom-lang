use std::sync::Arc;
use num_bigint::BigInt;
use phalcom_core::bytecode::Bytecode;
use phalcom_core::heap::Object;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn run_inline_module(source: &str) -> (VM, phalcom_core::heap::ObjRef) {
    let mut vm = VM::new();
    let src: Arc<str> = source.into();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(src))
        .expect("source should compile cleanly");
    vm.run_compiled(&program).expect("program execution should succeed");
    let entry_id = program.initialization_order.last().expect("entry module");
    let mod_obj = vm.module_registry.get(entry_id).unwrap().object;
    (vm, mod_obj)
}

fn get_module_global(vm: &VM, mod_obj: phalcom_core::heap::ObjRef, name: &str) -> Value {
    let name_sym = vm.interner.find(name).expect("symbol should exist");
    let module = vm.heap.module(mod_obj);
    let slot = module.name_to_slot[&name_sym];
    module.globals[slot]
}

#[test]
fn test_unit_runtime_use() {
    let source = r#"
const U: Unit = ()
let mut y = U
"#;
    let (vm, mod_obj) = run_inline_module(source);
    let val = get_module_global(&vm, mod_obj, "y");
    assert!(val.is_unit(), "expected Unit value");
}

#[test]
fn test_bool_runtime_use() {
    let source = r#"
const B_True: Bool = true
const B_False: Bool = false
let mut t = B_True
let mut f = B_False
"#;
    let (vm, mod_obj) = run_inline_module(source);
    let val_t = get_module_global(&vm, mod_obj, "t");
    let val_f = get_module_global(&vm, mod_obj, "f");
    assert_eq!(val_t.as_bool(), Some(true));
    assert_eq!(val_f.as_bool(), Some(false));
}

#[test]
fn test_small_int_runtime_use() {
    let source = r#"
const A: Int = 10 + 32
let mut y = A + 8
"#;
    let (vm, mod_obj) = run_inline_module(source);
    let val_y = get_module_global(&vm, mod_obj, "y");
    assert_eq!(val_y.as_int(), Some(50));
}

#[test]
fn test_large_int_runtime_use() {
    let source = r#"
const Large: Int = 100000000000000000000
let mut y = Large + 1
"#;
    let (vm, mod_obj) = run_inline_module(source);
    let val_y = get_module_global(&vm, mod_obj, "y");
    let obj = val_y.as_obj().expect("LargeInt must be a heap object");
    let big = vm.heap.as_large_int(obj).expect("must be LargeInt");
    let expected: BigInt = "100000000000000000001".parse().unwrap();
    assert_eq!(big, &expected);
}

#[test]
fn test_float_exact_bits() {
    let source = r#"
const Pi: Float = 3.141592653589793
const Zero: Float = 0.0
let mut p = Pi
let mut z = Zero
"#;
    let (vm, mod_obj) = run_inline_module(source);
    let val_p = get_module_global(&vm, mod_obj, "p");
    let val_z = get_module_global(&vm, mod_obj, "z");

    let p_bits = val_p.as_float().expect("float p").to_bits();
    let expected_pi = 3.141592653589793f64.to_bits();
    assert_eq!(p_bits, expected_pi, "exact bits of Pi must be preserved");

    let z_bits = val_z.as_float().expect("float z").to_bits();
    let expected_zero = 0.0f64.to_bits();
    assert_eq!(z_bits, expected_zero, "exact bits of 0.0 must be preserved");
}

#[test]
fn test_string_content() {
    let source = r#"
const Greeting: String = "hello static world"
let mut s = Greeting
"#;
    let (vm, mod_obj) = run_inline_module(source);
    let val_s = get_module_global(&vm, mod_obj, "s");
    let obj = val_s.as_obj().expect("String must be a heap object");
    let str_content = vm.heap.as_string(obj).expect("must be String");
    assert_eq!(str_content.as_str(), "hello static world");
}

#[test]
fn test_symbol_spelling() {
    let source = r#"
const Sym: Symbol = #my_const_symbol
let mut s = Sym
"#;
    let (vm, mod_obj) = run_inline_module(source);
    let val_s = get_module_global(&vm, mod_obj, "s");
    let sym = val_s.as_symbol().expect("must be a symbol");
    assert_eq!(vm.resolve_symbol(sym), "my_const_symbol");
}

#[test]
fn test_repeated_reference_does_not_rerun_initializer() {
    let source = r#"
const ConstVal: Int = 40 + 2
let mut a = ConstVal
let mut b = ConstVal
"#;
    let (vm, mod_obj) = run_inline_module(source);
    let val_a = get_module_global(&vm, mod_obj, "a");
    let val_b = get_module_global(&vm, mod_obj, "b");
    assert_eq!(val_a.as_int(), Some(42));
    assert_eq!(val_b.as_int(), Some(42));

    // Verify chunk has no bytecode computing the constant
    let module = vm.heap.module(mod_obj);
    let closure_ref = module.closure.expect("module closure");
    let chunk = &vm.heap.closure(closure_ref).callable.chunk;
    let has_invoke = chunk.code.iter().any(|op| matches!(op, Bytecode::Invoke { .. }));
    assert!(!has_invoke, "static constant addition must not be executed at runtime via invoke");
}

#[test]
fn test_large_int_gc_rooting() {
    let source_str = r#"
const BigNum: Int = 999999999999999999999999999999
let mut result = BigNum
"#;
    let src: Arc<str> = source_str.into();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(src))
        .expect("compile entry selection");
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("materialize program");

    let entry_id = program.initialization_order.last().expect("entry module");
    let compiled_mod = &program.modules[entry_id];
    let source_text = compiled_mod.source_text.as_ref().expect("source text");
    vm.compile_program_module_closure(entry_id, source_text, &program)
        .expect("compile closure");

    let mod_obj = vm.module_registry.get(entry_id).unwrap().object;
    let closure_ref = vm.heap.module(mod_obj).closure.expect("closure must exist");

    // 1. Ensure runtime representation in constants is heap LargeInt
    let chunk = &vm.heap.closure(closure_ref).callable.chunk;
    let large_int_obj = chunk
        .constants
        .iter()
        .find_map(|v| {
            if let Some(obj) = v.as_obj() {
                if matches!(vm.heap.get(obj), Object::LargeInt(_)) {
                    return Some(obj);
                }
            }
            None
        })
        .expect("chunk constant pool must contain a heap LargeInt object");

    // 2. Force GC collection after compilation but before execution
    let swept = vm.force_gc();
    // 3. Ensure the LargeInt survived GC because it is rooted in chunk constants
    assert!(
        vm.heap.try_get(large_int_obj).is_some(),
        "LargeInt in constant pool must survive GC (swept {swept})"
    );

    // 4. Run compiled program
    vm.run_compiled(&program).expect("program execution should succeed");

    // 5. Verify exact numeric value survives
    let val_res = get_module_global(&vm, mod_obj, "result");
    let obj = val_res.as_obj().expect("result must be an Object");
    let big = vm.heap.as_large_int(obj).expect("result must be LargeInt");
    let expected: BigInt = "999999999999999999999999999999".parse().unwrap();
    assert_eq!(big, &expected, "exact BigInt value must survive execution");
}
