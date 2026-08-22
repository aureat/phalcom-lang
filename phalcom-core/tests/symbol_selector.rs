use phalcom_core::interpret::Interpreter;
use phalcom_core::value::Value;

fn eval_source(src: &str, name: &str) -> Result<Value, String> {
    let mut interpreter = Interpreter::new();
    let module = interpreter.vm.create_module("main", "symbol-selector");
    interpreter.vm.interpret_source(module, src).map_err(|error| error.to_string())?;
    let symbol = interpreter.vm.interner.intern(name);
    interpreter.vm.heap.module(module).get(symbol).ok_or_else(|| format!("missing binding {name}"))
}

#[test]
fn selector_pattern_class_identity_is_preserved() {
    let value = eval_source(
        "let a = #method(...); let b = #+...; let c = #+(...); let result = a.class == Symbol and b.class == Symbol and c.class == Symbol and SelectorPattern(a).class == SelectorPattern and SelectorPattern(b).class == SelectorPattern and SelectorPattern(c).class == SelectorPattern\n",
        "result",
    )
    .expect("selector patterns should evaluate");
    assert_eq!(value, Value::bool(true));
}

#[test]
fn exact_symbol_values_use_canonical_selector_spelling() {
    let value =
        eval_source("let result = #+ != #+(_) and #+ == #+ and #! == #! and #... == #\"...\"\n", "result").expect("exact symbol comparison should evaluate");
    assert_eq!(value, Value::bool(true));
}

#[test]
fn zero_allocation_symbol_pattern_literal() {
    let value = eval_source(
        "let a = #foo(...); let b = #foo(_, ..., bar); let c = #[_, ...]=(put); let result = a.class == Symbol and b.class == Symbol and c.class == Symbol\n",
        "result",
    )
    .expect("symbol patterns evaluate to symbol");
    assert_eq!(value, Value::bool(true));
}

#[test]
fn selector_materialization_and_introspection() {
    let value = eval_source(
        r#"
let sel = Selector(#foo(_));
let is_sel = sel.class == Selector;
let base = sel.base == #foo;
let kind = sel.kind == #method;
let str = sel.toString == "Selector(#foo(_))";
let sub = Selector(#[_]).toString == "Selector(#[_])";
let result = is_sel and base and kind and str and sub;
"#,
        "result",
    )
    .expect("Selector materialization should succeed");
    assert_eq!(value, Value::bool(true));
}

#[test]
fn selector_pattern_materialization_and_matching() {
    let value = eval_source(
        r#"
let pat = SelectorPattern(#foo(...));
let is_pat = pat.class == SelectorPattern;
let base = pat.base == #foo;
let str = pat.toString == "SelectorPattern(#foo(...))";
let match_pos = pat.matches(#foo(_));
let match_neg = pat.matches(#bar(_)) == false;
let result = is_pat and base and str and match_pos and match_neg;
"#,
        "result",
    )
    .expect("SelectorPattern materialization should succeed");
    assert_eq!(value, Value::bool(true));
}

#[test]
fn selector_materialization_type_error_guards() {
    let err1 = eval_source("let s = Selector(#foo(...))\n", "s").unwrap_err();
    assert!(
        err1.contains("Cannot construct Selector from selector pattern symbol") || err1.contains("Use SelectorPattern instead"),
        "unexpected error: {err1}"
    );

    let err2 = eval_source("let p = SelectorPattern(#foo(_))\n", "p").unwrap_err();
    assert!(
        err2.contains("SelectorPattern requires a selector pattern") || err2.contains("Received exact selector"),
        "unexpected error: {err2}"
    );
}
