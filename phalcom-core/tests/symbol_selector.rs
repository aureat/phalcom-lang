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
        "let a = #method(...); let b = #+...; let c = #+(...); let result = a.class == SelectorPattern and b.class == SelectorPattern and c.class == SelectorPattern\n",
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
