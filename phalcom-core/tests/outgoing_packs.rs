//! F.2 outgoing-pack integration coverage. These tests use the surface
//! compiler and VM rather than inspecting builder internals, so they pin the
//! observable source-order and dispatch contracts at the F.3 boundary.

use phalcom_core::heap::Object;
use phalcom_core::interpret::Interpreter;
use phalcom_core::value::Value;

fn on_large_stack<T: Send + 'static>(test: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(test)
        .expect("spawn outgoing-pack test thread")
        .join()
        .expect("outgoing-pack test thread")
}

fn eval_source(src: &str, var_name: &str) -> Result<Value, String> {
    let mut interp = Interpreter::new();
    let main = interp.vm.create_module("main", "<outgoing-pack test>");
    interp.vm.interpret_source(main, src).map_err(|error| error.to_string())?;
    let symbol = interp.vm.interner.intern(var_name);
    let Object::Module(module) = interp.vm.heap.get(main) else {
        return Err("main module is not a module".to_string());
    };
    module.get(symbol).ok_or_else(|| format!("variable `{var_name}` not found"))
}

fn run_source(src: &str) -> Result<(), String> {
    let mut interp = Interpreter::new();
    let main = interp.vm.create_module("main", "<outgoing-pack test>");
    interp.vm.interpret_source(main, src).map_err(|error| error.to_string())
}

#[test]
fn positional_spread_uses_tuple_unit_and_iterable_lanes() {
    let value = on_large_stack(|| {
        eval_source(
            r#"
class Receiver {
  collect(*values) { return values.size }
}
let receiver = Receiver.new()
let labeledTuple = (1, 2, label: 3)
let fromTuple = receiver.collect(*labeledTuple)
let fromUnit = receiver.collect(*())
let fromList = receiver.collect(*[4, 5, 6])
let result = fromTuple * 100 + fromUnit * 10 + fromList
"#,
            "result",
        )
    });
    assert_eq!(value.expect("spread source must execute"), Value::Int(203));
}

#[test]
fn complete_and_labeled_expansion_derive_concrete_selectors() {
    let value = on_large_stack(|| {
        eval_source(
            r#"
class Receiver {
  collect(*values) { return values.size }
  sum(left, right) { return left + right }
}
let receiver = Receiver.new()
let fromComplete = receiver.collect(*** (1, 2))
let fromLabels = receiver.sum(**(left: 3, right: 4))
let result = fromComplete * 10 + fromLabels
"#,
            "result",
        )
    });
    assert_eq!(value.expect("expansion source must execute"), Value::Int(27));
}

#[test]
fn dynamic_tuple_preserves_lanes_and_normalizes_empty_expansion() {
    let value = on_large_stack(|| {
        eval_source(
            r#"
let positional = (1, 2)
let labels = (tail: 4,)
let unit = ()
let result = (0, *positional, ***unit, tag: 3, **labels)
let count = result.size
let positionalCount = result.positionals.size
let firstLabel = result.labelAt(0)
let secondLabel = result.labelAt(1)
let summary = count * 100 + positionalCount * 10 + (firstLabel == #tag).ifTrue(|| { 1 }, ifFalse: || { 0 }) + (secondLabel == #tail).ifTrue(|| { 1 }, ifFalse: || { 0 })
"#,
            "summary",
        )
    });
    assert_eq!(value.expect("dynamic Tuple source must execute"), Value::Int(532));
}

#[test]
fn dynamic_super_callable_and_dnu_sends_keep_their_dispatch_family() {
    let source = r#"
class Parent {
  collect(*values) { return values.size }
}
class Child is Parent {
  forward(_ source) { return super.collect(*source) }
}
class Recorder {
  doesNotUnderstand(_ message) { return message.selector == #missing(_) }
}
let child = Child.new()
let source = (1, 2)
let callable = |left, right| { left + right }
let superResult = child.forward(source)
let callableResult = callable(*source)
let dnuResult = Recorder.new().missing(*((7,)))
"#;
    let super_result = on_large_stack(|| eval_source(source, "superResult"));
    assert_eq!(super_result.expect("dynamic super send must execute"), Value::Int(2));
    let callable_result = on_large_stack(|| eval_source(source, "callableResult"));
    assert_eq!(callable_result.expect("dynamic callable send must execute"), Value::Int(3));
    let dnu_result = on_large_stack(|| eval_source(source, "dnuResult"));
    assert_eq!(dnu_result.expect("dynamic dNU send must execute"), Value::Bool(true));
}

#[test]
fn dynamic_subscripts_respect_put_and_assignment_result() {
    let value = on_large_stack(|| {
        eval_source(
            r#"
class Box {
  _values
  @constructor
  new() { _values = Map.new() }
  [_ index] { return _values[index] }
  [_ index]=(put value) {
    _values[index] = value
    return -1
  }
}
let box = Box.new()
let index = (5,)
let assignment = box[*index] = 42
let read = box[*index]
"#,
            "assignment",
        )
    });
    assert_eq!(value.expect("dynamic subscript assignment must execute"), Value::Int(42));

    let read = on_large_stack(|| {
        eval_source(
            r#"
class Box {
  _values
  @constructor
  new() { _values = Map.new() }
  [_ index] { return _values[index] }
  [_ index]=(put value) { _values[index] = value }
}
let box = Box.new()
let index = (5,)
box[*index] = 42
let read = box[*index]
"#,
            "read",
        )
    });
    assert_eq!(read.expect("dynamic subscript read must execute"), Value::Int(42));
}

#[test]
fn computed_labels_and_iterator_failures_stay_structured() {
    let computed = on_large_stack(|| {
        eval_source(
            r#"
class Receiver {
  sum(left, right) { return left + right }
}
let receiver = Receiver.new()
let result = receiver.sum([#left]: 3, [#right]: 4)
"#,
            "result",
        )
    });
    assert_eq!(computed.expect("computed labels must dispatch"), Value::Int(7));

    let iterator_error = on_large_stack(|| {
        run_source(
            r#"
class Broken {
  iterate(_ cursor) { throw Error.new("iterate boom") }
  iteratorValue(_ cursor) { return 0 }
}
class Receiver {
  collect(*values) { return values.size }
}
Receiver.new().collect(*Broken.new())
"#,
        )
    })
    .expect_err("iterator implementation failure must escape the generic spread loop");
    assert!(iterator_error.contains("iterate boom"), "unexpected iterator error: {iterator_error}");
}
