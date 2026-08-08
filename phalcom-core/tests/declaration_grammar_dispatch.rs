//! Targeted unit tests for Task 1-4 Declaration Grammar:
//! - Parser parameter, setter, subscript verification
//! - Duplicate selector checks (getter vs setter duplicate behavior)
//! - Compiler & VM dispatch for method declarations and calls

use phalcom_core::interpret::Interpreter;
use phalcom_core::value::Value;

fn run_source(src: &str) -> Result<(), String> {
    let mut interp = Interpreter::new();
    let main = interp.vm.create_module("main", "<test>");
    interp.vm.interpret_source(main, src).map_err(|e| e.to_string())
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
    assert_eq!(res.unwrap(), Value::Int(42));
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
    assert_eq!(res1.unwrap(), Value::Int(30));

    let res2 = eval_source(src, "r2");
    assert_eq!(res2.unwrap(), Value::Int(35));
}

#[test]
fn test_dispatch_setter_name_put_value() {
    let src = r#"
class Box {
  _val
  val => _val
  val=(put v) {
    _val = v
  }
}

let b = Box.new()
b.val = 99
let r = b.val
"#;
    let res = eval_source(src, "r");
    assert_eq!(res.unwrap(), Value::Int(99));
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
    assert_eq!(res.unwrap(), Value::Int(777));
}

// --- Access-control coverage ---

#[test]
fn private_member_allows_defining_class_and_rejects_external_call() {
    let ok = eval_source(
        "class Vault {\n  @private\n  secret => 42\n  reveal => secret\n}\nlet result = Vault.new().reveal\n",
        "result",
    );
    assert_eq!(ok.unwrap(), Value::Int(42));

    let err = run_source("class Vault {\n  @private\n  secret => 42\n}\nVault.new().secret\n").unwrap_err();
    assert!(err.contains("member.private_access"), "unexpected error: {err}");
}

#[test]
fn protected_member_allows_subclass_and_rejects_external_call() {
    let ok = eval_source(
        "class Base {\n  @protected\n  secret => 42\n}\nclass Child is Base {\n  reveal => secret\n}\nlet result = Child.new().reveal\n",
        "result",
    );
    assert_eq!(ok.unwrap(), Value::Int(42));

    let err = run_source("class Base {\n  @protected\n  secret => 42\n}\nBase.new().secret\n").unwrap_err();
    assert!(err.contains("member.protected_access"), "unexpected error: {err}");
}

#[test]
fn private_subscript_enforces_visibility() {
    let err = run_source("class SecretList {\n  @private\n  [_ index] { return index }\n}\nSecretList.new()[0]\n").unwrap_err();
    assert!(err.contains("member.private_access"), "unexpected error: {err}");
}

#[test]
fn reflection_cannot_bypass_private_visibility() {
    let err = run_source("class Vault {\n  @private\n  secret => 42\n}\nVault.new().perform(#secret)\n").unwrap_err();
    assert!(err.contains("member.private_access"), "unexpected error: {err}");
}

#[test]
fn ordinary_source_cannot_spell_internal_selector() {
    let err = run_source("List.new()._$at(0)\n").unwrap_err();
    assert!(err.contains("internal.namespace_reserved"), "unexpected error: {err}");
}
