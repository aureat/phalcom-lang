//! Targeted unit tests for Task 1-4 Declaration Grammar:
//! - Parser parameter, setter, subscript verification
//! - Duplicate selector checks (getter vs setter duplicate behavior)
//! - Compiler & VM dispatch for method declarations and calls

use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::error::PhResult;
use phalcom_core::heap::Object;
use phalcom_core::interpret::Interpreter;
use phalcom_core::method::{MethodKind, RestLayout, RestMode};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn native_stub(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::unit())
}

fn run_source(src: &str) -> Result<(), String> {
    let mut interp = Interpreter::new();
    let main = interp.vm.create_module("main", "<test>");
    interp.vm.interpret_source(main, src).map_err(|e| e.to_string())
}

#[test]
fn class_keyword_at_module_level_is_undefined_variable() {
    let error = run_source("class\n").expect_err("module-level `class` must not bind to self.class");
    assert!(error.contains("undefined variable 'class'"), "unexpected error: {error}");
}

fn eval_source(src: &str, var_name: &str) -> Result<Value, String> {
    let mut interp = Interpreter::new();
    let main = interp.vm.create_module("main", "<test>");
    interp.vm.interpret_source(main, src).map_err(|e| e.to_string())?;
    let sym = interp.vm.interner.intern(var_name);
    let module = match interp.vm.heap.get(main) {
        phalcom_core::heap::Object::Module(m) => m,
        _ => return Err("main module is not Module".into()),
    };
    module.get(sym).ok_or_else(|| format!("variable `{var_name}` not found"))
}

#[test]
fn static_duplicate_labels_fail_before_static_send_lowering() {
    let error = run_source(
        r#"
class Receiver {
  target(x) { return x }
}
let receiver = Receiver.new()
receiver.target(x: 1, x: 2)
"#,
    )
    .expect_err("duplicate static labels must be a compiler error");
    assert!(error.contains("duplicate argument label `x`"), "unexpected error: {error}");
}

#[test]
fn constructor_rest_is_rejected_in_f3_scope() {
    let error = run_source(
        r#"
class C {
  @constructor
  new(*args) { }
}
"#,
    )
    .expect_err("constructor rest is outside F.3 method scope");
    assert!(
        error.contains("not supported on constructors or subscript methods in F.3"),
        "unexpected error: {error}"
    );
}

#[test]
fn native_rest_method_installs_during_method_definition() {
    let mut vm = VM::new();
    let main = vm.create_module("main", "native_rest_installation");
    let closure = vm
        .compile_closure_as(main, "class C {\n  target() { }\n}\n", UnitKind::File)
        .expect("class should compile");
    let method_id = vm
        .heap
        .closure(closure)
        .callable
        .chunk
        .constants
        .iter()
        .copied()
        .find_map(|constant| {
            let id = constant.as_obj()?;
            (matches!(vm.heap.get(id), Object::Method(_)) && vm.resolve_symbol(vm.heap.method(id).selector()) == "target()").then_some(id)
        })
        .expect("compiled class should carry target method constant");
    {
        let method = vm.heap.method_mut(method_id);
        method.kind = MethodKind::Primitive(phalcom_core::method::PrimitiveFn::Legacy(native_stub));
        method.signature.rest = Some(RestLayout::new(0, Vec::new().into_boxed_slice(), RestMode::Positional { param_index: 0 }));
    }

    vm.run_cell(main, closure).expect("shape-compatible native rest methods should install");
}

#[test]
fn reflective_send_uses_rest_family_fallback() {
    let mut vm = VM::new();
    let main = vm.create_module("main", "reflective_rest_send");
    vm.interpret_source(main, "class C {\n  sum(*args) { return args.size }\n}\nlet c = C.new()\n")
        .expect("rest method should install");
    let receiver = vm.heap.module(main).get(vm.interner.intern("c")).expect("receiver should exist");
    let selector = vm.get_or_intern("sum(_,_)");
    let result = vm
        .send_dynamic(receiver, selector, &[Value::int(1), Value::int(2)])
        .expect("reflective rest send should resolve");
    assert_eq!(result, Value::int(2));
}

#[test]
fn positional_spread_of_non_iterable_has_structured_boundary() {
    // Bootstrap and traceback paths currently exceed the test harness's
    // default worker stack when this runtime error unwinds through core.ph.
    // Keep the regression itself isolated without weakening production logic.
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            let error = run_source(
                r#"
class Receiver {
  target(_ value) { return value }
}
let receiver = Receiver.new()
receiver.target(*1)
"#,
            )
            .expect_err("non-Iterable * operand must not fall through to dNU");
            assert!(
                error.contains("* expansion requires Tuple, Unit, or an iterable value; got int"),
                "unexpected error: {error}"
            );
        })
        .expect("spawn isolated stack")
        .join()
        .expect("non-Iterable regression thread");
}

// --- Duplicate Selector Checks ---

#[test]
fn test_subscript_getter_and_setter_do_not_trigger_duplicate_selector() {
    let src = r#"
class Container {
  _items
  @constructor
  new() {
    _items = Map.new()
  }
  [_ key] {
    return _items[key]
  }
  [_ key]=(put val) {
    _items[key] = val
  }
}

let c = Container.new()
c[10] = 42
let result = c[10]
"#;
    let res = eval_source(src, "result");
    assert!(
        res.is_ok(),
        "subscript getter and setter must not trigger duplicate selector warning/error: {:?}",
        res.err()
    );
    assert_eq!(res.unwrap(), Value::int(42));
}

#[test]
fn test_duplicate_subscript_getters_rejected() {
    let src = r#"
class Foo {
  [_ index] { return 1 }
  [_ key] { return 2 }
}
"#;
    let res = run_source(src);
    assert!(res.is_err(), "duplicate subscript getters must trigger duplicate selector error");
    let err = res.unwrap_err();
    assert!(err.contains("class.duplicate_selector"), "expected class.duplicate_selector error, got: {err}");
}

#[test]
fn test_duplicate_subscript_setters_rejected() {
    let src = r#"
class Foo {
  [_ index]=(put v1) { }
  [_ key]=(put v2) { }
}
"#;
    let res = run_source(src);
    assert!(res.is_err(), "duplicate subscript setters must trigger duplicate selector error");
    let err = res.unwrap_err();
    assert!(err.contains("class.duplicate_selector"), "expected class.duplicate_selector error, got: {err}");
}

#[test]
fn test_duplicate_method_getters_rejected() {
    let src = r#"
class Foo {
  bar() { return 1 }
  bar() { return 2 }
}
"#;
    let res = run_source(src);
    assert!(res.is_err(), "duplicate method getters must trigger duplicate selector error");
    let err = res.unwrap_err();
    assert!(err.contains("class.duplicate_selector"), "expected class.duplicate_selector error, got: {err}");
}

#[test]
fn test_duplicate_method_setters_rejected() {
    let src = r#"
class Foo {
  bar=(put v1) { }
  bar=(put v2) { }
}
"#;
    let res = run_source(src);
    assert!(res.is_err(), "duplicate method setters must trigger duplicate selector error");
    let err = res.unwrap_err();
    assert!(err.contains("class.duplicate_selector"), "expected class.duplicate_selector error, got: {err}");
}

// --- Dispatch Tests for New Method Signatures ---

#[test]
fn test_dispatch_labeled_method_and_shorthand() {
    let src = r#"
class Calculator {
  add(_ base, to val) {
    return base + val
  }
  compute(multiplier mult, offset) {
    return mult * 10 + offset
  }
}

let calc = Calculator.new()
let r1 = calc.add(10, to: 20)
let r2 = calc.compute(multiplier: 3, offset: 5)
"#;
    let res1 = eval_source(src, "r1");
    assert_eq!(res1.unwrap(), Value::int(30));

    let res2 = eval_source(src, "r2");
    assert_eq!(res2.unwrap(), Value::int(35));
}

#[test]
fn test_dispatch_setter_name_put_value() {
    let src = r#"
class Box {
  _val
  val { _val }
  val=(put v) {
    _val = v
  }
}

let b = Box.new()
b.val = 99
let r = b.val
"#;
    let res = eval_source(src, "r");
    assert_eq!(res.unwrap(), Value::int(99));
}

#[test]
fn test_dispatch_subscript_getter_and_setter() {
    let src = r#"
class Storage {
  _m
  @constructor
  new() {
    _m = Map.new()
  }
  [_ k] {
    return _m[k]
  }
  [_ k]=(put v) {
    _m[k] = v
  }
}

let s = Storage.new()
s[100] = 777
let r = s[100]
"#;
    let res = eval_source(src, "r");
    assert_eq!(res.unwrap(), Value::int(777));
}

// --- Access-control coverage ---

#[test]
fn private_member_allows_defining_class_and_rejects_external_call() {
    let ok = eval_source(
        "class Vault {\n  @private\n  secret { 42 }\n  reveal { secret }\n}\nlet result = Vault.new().reveal\n",
        "result",
    );
    assert_eq!(ok.unwrap(), Value::int(42));

    let err = run_source("class Vault {\n  @private\n  secret { 42 }\n}\nVault.new().secret\n").unwrap_err();
    assert!(err.contains("member.private_access"), "unexpected error: {err}");
}

#[test]
fn protected_member_allows_subclass_and_rejects_external_call() {
    let ok = eval_source(
        "class Base {\n  @protected\n  secret { 42 }\n}\nclass Child is Base {\n  reveal { secret }\n}\nlet result = Child.new().reveal\n",
        "result",
    );
    assert_eq!(ok.unwrap(), Value::int(42));

    let err = run_source("class Base {\n  @protected\n  secret { 42 }\n}\nBase.new().secret\n").unwrap_err();
    assert!(err.contains("member.protected_access"), "unexpected error: {err}");
}

#[test]
fn private_subscript_enforces_visibility() {
    let err = run_source("class SecretList {\n  @private\n  [_ index] { return index }\n}\nSecretList.new()[0]\n").unwrap_err();
    assert!(err.contains("member.private_access"), "unexpected error: {err}");
}

#[test]
fn reflection_cannot_bypass_private_visibility() {
    let err = run_source("class Vault {\n  @private\n  secret { 42 }\n}\nVault.new().perform(#secret)\n").unwrap_err();
    assert!(err.contains("member.private_access"), "unexpected error: {err}");
}

#[test]
fn ordinary_source_cannot_spell_internal_selector() {
    let err = run_source("List.new()._$at(0)\n").unwrap_err();
    assert!(err.contains("internal.namespace_reserved"), "unexpected error: {err}");
}
