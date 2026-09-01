//! Executable ADT/match scenarios shared by runtime and vertical conformance.

use super::vm_support::{compile_inline, run_inline};
use phalcom_core::bytecode::Bytecode;
use phalcom_core::value::Value;

fn slot(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    vm.heap.module(module).get(vm.interner.find(name)?)
}

#[test]
fn adt_run_01_singleton_execution_and_variant_identity() {
    let source = "enum State { @variant Ready @variant Done }\nlet value = State::Ready\nlet result = match value { State::Ready => 1 State::Done => 0 }\n";
    let (vm, module) = run_inline(source).expect("singleton match should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(1)));
}

#[test]
fn adt_run_02_nullary_constructor_is_not_singleton() {
    let source =
        "enum State { @variant Ready() @variant Done() }\nlet first = State::Ready()\nlet second = State::Ready()\nlet distinct = not (first === second)\n";
    let (vm, module) = run_inline(source).expect("nullary construction should run");
    assert_eq!(slot(&vm, module, "distinct"), Some(Value::bool(true)));
}

#[test]
fn adt_run_03_singleton_and_nullary_patterns_select_separately() {
    let source = "enum State { @variant Ready @variant Ready() }\nlet singleton = State::Ready\nlet nullary = State::Ready()\nlet a = match singleton { State::Ready => 1 State::Ready() => 2 }\nlet b = match nullary { State::Ready => 1 State::Ready() => 2 }\n";
    let (vm, module) = run_inline(source).expect("shape-specific match should run");
    assert_eq!(slot(&vm, module, "a"), Some(Value::int(1)));
    assert_eq!(slot(&vm, module, "b"), Some(Value::int(2)));
}

#[test]
fn adt_run_04_payload_binding_reads_payload_slot() {
    let source = "enum Boxed { @variant Value(_ value: Int) }\nlet boxed = Boxed::Value(42)\nlet result = match boxed { Boxed::Value(value) => value }\n";
    let (vm, module) = run_inline(source).expect("payload match should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
fn adt_run_05_labeled_payload_uses_declared_slot() {
    let source = "enum Boxed { @variant Value(named value: Int) }\nlet boxed = Boxed::Value(named: 42)\nlet result = match boxed { Boxed::Value(named: value) => value }\n";
    let (vm, module) = run_inline(source).expect("labeled payload match should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
fn adt_run_06_nested_adt_match_extracts_inner_value() {
    let source = "enum Inner { @variant Value(_ value: Int) }\nenum Outer { @variant Some(_ value: Inner) }\nlet value = Outer::Some(Inner::Value(42))\nlet result = match value { Outer::Some(Inner::Value(x)) => x }\n";
    let (vm, module) = run_inline(source).expect("nested payload match should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
fn adt_run_07_wildcard_payload_has_no_extract_instruction() {
    let source = "enum Boxed { @variant Value(_ value: Int) }\nlet value = Boxed::Value(42)\nlet result = match value { Boxed::Value(_) => 1 }\n";
    let (vm, program, closure) = compile_inline(source).expect("wildcard match should compile");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert_eq!(chunk.code.iter().filter(|op| matches!(op, Bytecode::GetVariantPayload(..))).count(), 0);
    assert!(!program.modules[&program.entry].lowering.enums.is_empty());
}

#[test]
fn adt_run_08_or_pattern_selects_first_successful_alternative() {
    let source = "enum Choice { @variant Left(_ value: Int) @variant Right(_ value: Int) }\nlet value = Choice::Right(42)\nlet result = match value { Choice::Left(x) | Choice::Right(x) => x }\n";
    let (vm, module) = run_inline(source).expect("or-pattern match should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
fn adt_run_09_family_pattern_routes_to_exact_member() {
    let source =
        "enum State { @variant Ready @variant Ready() @variant Done }\nlet value = State::Ready()\nlet result = match value { Ready* => 1 Done => 2 }\n";
    let (vm, module) = run_inline(source).expect("family match should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(1)));
}

#[test]
#[ignore = "RED: selector-gap family runtime projection remains incomplete"]
fn adt_run_10_selector_gap_family_routes_candidate_specific_slots() {
    let source = "enum Animal { @variant Dog(_ name: String) @variant Dog(_ name: String, age: Int, breed: String) }\nlet value = Animal::Dog(\"rex\", 4, \"collie\")\nlet result = match value { Dog(_, age, breed: _) => age _ => 0 }\n";
    let (vm, module) = run_inline(source).expect("selector-gap family fixture should execute");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(4)));
}

#[test]
fn adt_run_11_match_expression_produces_selected_branch_value() {
    let source = "enum State { @variant Ready @variant Done }\nlet result = match State::Done { State::Ready => 1 State::Done => 2 }\n";
    let (vm, module) = run_inline(source).expect("match expression should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(2)));
}

#[test]
fn adt_run_12_braced_arm_tail_is_branch_result() {
    let source = "enum State { @variant Ready }\nlet result = match State::Ready { State::Ready => { let x = 40; x + 2 } }\n";
    let (vm, module) = run_inline(source).expect("braced match arm should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
#[ignore = "GATED: enclosing-return source fixture is required"]
fn adt_run_13_return_inside_braced_arm_exits_enclosing_callable() {
    let source = "enum State { @variant Ready }\nclass Test { run() { match State::Ready { State::Ready => { return 42 } } 0 } }\nlet result = Test::run()\n";
    let (vm, module) = run_inline(source).expect("return-in-arm fixture should execute");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
#[ignore = "GATED: side-effect counter fixture is required"]
fn adt_run_14_scrutinee_is_evaluated_once_for_many_candidates() {
    let source = "enum State { @variant Ready @variant Done }\nlet counter = 0\nclass Test { next() { counter = counter + 1 State::Done } }\nlet result = match Test::next() { State::Ready => 1 State::Done => 2 }\n";
    let (vm, module) = run_inline(source).expect("single-scrutinee fixture should execute");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(2)));
    assert_eq!(slot(&vm, module, "counter"), Some(Value::int(1)));
}

#[test]
#[ignore = "GATED: selected-branch counter fixture is required"]
fn adt_run_15_only_selected_branch_executes() {
    let source = "enum State { @variant Ready @variant Done }\nlet counter = 0\nclass Test { ready() { counter = counter + 1 1 } done() { counter = counter + 10 2 } }\nlet result = match State::Ready { State::Ready => Test::ready() State::Done => Test::done() }\n";
    let (vm, module) = run_inline(source).expect("selected-branch fixture should execute");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(1)));
    assert_eq!(slot(&vm, module, "counter"), Some(Value::int(1)));
}

#[test]
#[ignore = "GATED: trap layout assertion awaits stable post-match bytecode contract"]
fn adt_run_16_exhaustive_fallthrough_has_internal_trap_only() {
    let source = "enum State { @variant Ready @variant Done }\nlet result = match State::Ready { State::Ready => 1 State::Done => 2 }\n";
    let (vm, _program, closure) = compile_inline(source).expect("exhaustive match should compile");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(!chunk.code.is_empty(), "lowered match must have executable bytecode");
}

#[test]
fn adt_run_17_gadt_runtime_has_no_type_equality_bytecode() {
    let source = "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nlet expr = Expr<Int>::Int(42)\n";
    let (vm, _program, closure) = compile_inline(source).expect("GADT construction should compile");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(chunk.code.iter().all(|op| !matches!(op, Bytecode::IsVariant(_))));
}

#[test]
fn adt_vert_01_generic_result_crosses_constructor_match_and_runtime() {
    let source = "enum Result<T, E> { @variant Ok(_ value: T) -> Result<T, E> @variant Err(_ error: E) -> Result<T, E> }\nlet value = Result<Int, String>::Ok(42)\nlet result = match value { Result::Ok(x) => x Result::Err(_) => 0 }\n";
    let (vm, module) = run_inline(source).expect("generic Result vertical program should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
fn adt_vert_02_gadt_evaluator_erases_proof_at_runtime() {
    let source = "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nlet value = Expr<Int>::Int(42)\nlet result = match value { Expr::Int(x) => x }\n";
    let (vm, module) = run_inline(source).expect("GADT vertical program should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
#[ignore = "RED: multi-selector family runtime routing remains incomplete"]
fn adt_vert_03_multi_selector_family_routes_all_exact_shapes() {
    let source =
        "enum Animal { @variant Dog @variant Dog() @variant Dog(_ age: Int) }\nlet value = Animal::Dog(4)\nlet result = match value { Dog* => 1 _ => 0 }\n";
    let (vm, module) = run_inline(source).expect("multi-selector family fixture should execute");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(1)));
}

#[test]
#[ignore = "RED: nested or runtime binding commit remains incomplete"]
fn adt_vert_04_nested_option_result_or_pattern_has_no_partial_binding_leak() {
    let source = "enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nenum Result<T, E> { @variant Ok(_ value: T) -> Result<T, E> @variant Err(_ error: E) -> Result<T, E> }\nlet value = Result<Int, String>::Ok(Option<Int>::Some(42))\nlet result = match value { Result::Ok(Option::Some(x) | Option::None) => x _ => 0 }\n";
    let (vm, module) = run_inline(source).expect("nested or fixture should execute");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
#[ignore = "GATED: cross-module visibility fixture is required"]
fn adt_vert_05_visibility_crosses_semantic_and_runtime_boundaries() {
    let source = "enum Public { @variant Ready }\nlet result = match Public::Ready { Public::Ready => 1 }\n";
    let (vm, module) = run_inline(source).expect("public visibility fixture should execute");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(1)));
}

#[test]
#[ignore = "GATED: multi-module runtime compiler fixture is required"]
fn adt_vert_06_cross_module_labeled_payload_uses_imported_field_slot() {
    let source = "enum Payload { @variant Item(named value: Int) }\nlet result = match Payload::Item(named: 42) { Payload::Item(named: value) => value }\n";
    let (vm, module) = run_inline(source).expect("labeled payload fixture should execute");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(42)));
}

#[test]
fn adt_vert_07_core_option_native_representation_execution() {
    let source = r#"
let some_val = Option<Int>::Some(42)
let none_val = Option<Int>::None
let a = match some_val {
    Some(x) => x
    None => 0
}
let b = match none_val {
    Some(x) => x
    None => 100
}
"#;
    let (vm, module) = run_inline(source).expect("Option matching should execute");
    assert_eq!(slot(&vm, module, "a"), Some(Value::int(42)));
    assert_eq!(slot(&vm, module, "b"), Some(Value::int(100)));
}

#[test]
fn adt_vert_08_core_result_error_variant_execution() {
    let source = r#"
let ok_val = Result<Int, String>::Ok(10)
let err_val = Result<Int, String>::Error("fail")
let a = match ok_val {
    Ok(v) => v
    Error(_) => -1
}
let b = match err_val {
    Ok(v) => v
    Error(_) => -1
}
"#;
    let (vm, module) = run_inline(source).expect("Result matching should execute");
    assert_eq!(slot(&vm, module, "a"), Some(Value::int(10)));
    assert_eq!(slot(&vm, module, "b"), Some(Value::int(-1)));
}

#[test]
fn adt_vert_09_core_ordering_four_state_execution() {
    let source = r#"
let l = Ordering::Less
let e = Ordering::Equal
let g = Ordering::Greater
let u = Ordering::Unordered

let check = fn(ord: Ordering) {
    match ord {
        Less => 1
        Equal => 2
        Greater => 3
        Unordered => 4
    }
}
let res_l = check(l)
let res_e = check(e)
let res_g = check(g)
let res_u = check(u)
"#;
    let (vm, module) = run_inline(source).expect("Ordering matching should execute");
    assert_eq!(slot(&vm, module, "res_l"), Some(Value::int(1)));
    assert_eq!(slot(&vm, module, "res_e"), Some(Value::int(2)));
    assert_eq!(slot(&vm, module, "res_g"), Some(Value::int(3)));
    assert_eq!(slot(&vm, module, "res_u"), Some(Value::int(4)));
}

