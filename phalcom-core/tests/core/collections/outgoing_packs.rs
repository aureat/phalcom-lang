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
fn positional_spread_captures_pack_for_tuple_unit_and_iterable_lanes() {
    let value = on_large_stack(|| {
        eval_source(
            r#"
class Receiver {
  collect(*values) { return values }
}

let receiver = Receiver.new()
let labeledTuple = (1, 2, label: 3)
let fromTuple = receiver.collect(*labeledTuple)
let fromUnit = receiver.collect(*())
let fromList = receiver.collect(*[4, 5, 6])
let result = fromTuple is Pack and fromUnit is Pack and fromList is Pack and fromUnit == Pack() and fromUnit != ()
"#,
            "result",
        )
    });
    assert_eq!(value.expect("spread source must execute"), Value::bool(true));
}

#[test]
fn generic_spread_accepts_range_user_iterable_and_bounded_pipeline() {
    let value = on_large_stack(|| {
        eval_source(
            r#"
from universe.collections.iterator_laws import TakeIterator

class Receiver {
  collect(*values) { return values.size }
}

class CountdownIterator {
  _n
  @constructor
  new(_ n) { _n = n }
  next {
    if (_n >= 0) {
      let current = _n
      _n = _n - 1
      return Some(current)
    }
    return None
  }
  iterator { self }
}
class Countdown {
  @constructor
  from(n) { _n = n }
  iterator { CountdownIterator.new(_n) }
}
let receiver = Receiver.new()
let fromRange = receiver.collect(*(1..=3))
let fromUser = receiver.collect(*Countdown.from(n: 2))
let fromPipeline = receiver.collect(*TakeIterator.new((0..).iterator, 2))
let result = fromRange * 100 + fromUser * 10 + fromPipeline
"#,
            "result",
        )
    });
    assert_eq!(value.expect("generic Iterable sources must execute"), Value::int(332));
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
    assert_eq!(value.expect("expansion source must execute"), Value::int(27));
}

#[test]
fn complete_expansion_allows_multiple_contributions_before_labeled_phase() {
    let value = on_large_stack(|| {
        eval_source(
            r#"
class Receiver {
  sum(_ a, _ b, _ c, _ d) { return a * 1000 + b * 100 + c * 10 + d }
}
let first = (1,)
let second = (2,)
let result = Receiver.new().sum(***first, *second, ***((3, 4)))
"#,
            "result",
        )
    });
    assert_eq!(value.expect("multiple complete expansions must preserve order"), Value::int(1234));
}

#[test]
fn tuple_lane_observers_preserve_exact_structural_types_and_values() {
    let value = on_large_stack(|| {
        eval_source(
            r#"
let mixed: (Int, String, tag: Bool) = (1, "two", tag: true)
let positionals: (Int, String) = mixed.positionals
let labeled: (tag: Bool) = mixed.labeled
let labeledValue: Option<Bool> = labeled.at(0)
let labeledMatches = labeledValue.match(some: |value| { value }, none: || { false })
let result = positionals.size == 2 and positionals[0] == 1 and positionals[1] == "two" and labeled.size == 1 and labeled.labelAt(0) == #tag and labeledMatches
"#,
            "result",
        )
    });
    assert_eq!(value.expect("Tuple lane observers preserve their structural products"), Value::bool(true));
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
    assert_eq!(value.expect("dynamic Tuple source must execute"), Value::int(532));
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
    assert_eq!(super_result.expect("dynamic super send must execute"), Value::int(2));
    let callable_result = on_large_stack(|| eval_source(source, "callableResult"));
    assert_eq!(callable_result.expect("dynamic callable send must execute"), Value::int(3));
    let dnu_result = on_large_stack(|| eval_source(source, "dnuResult"));
    assert_eq!(dnu_result.expect("dynamic dNU send must execute"), Value::bool(true));
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
  [_ index]=(_ value) {
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
    assert_eq!(value.expect("dynamic subscript assignment must execute"), Value::int(42));

    let read = on_large_stack(|| {
        eval_source(
            r#"
class Box {
  _values
  @constructor
  new() { _values = Map.new() }
  [_ index] { return _values[index] }
  [_ index]=(_ value) { _values[index] = value }
}
let box = Box.new()
let index = (5,)
box[*index] = 42
let read = box[*index]
"#,
            "read",
        )
    });
    assert_eq!(read.expect("dynamic subscript read must execute"), Value::int(42));
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
    assert_eq!(computed.expect("computed labels must dispatch"), Value::int(7));

    let iterator_error = on_large_stack(|| {
        run_source(
            r#"
class Broken {
  iterator { throw Error.new("iterate boom") }
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

#[test]
fn dynamic_pack_errors_remain_specific_and_stop_dispatch() {
    let duplicate = on_large_stack(|| {
        run_source(
            r#"
class Receiver {
  sum(left, right) { return left + right }
}
Receiver.new().sum(left: 1, **(left: 2, right: 3))
"#,
        )
    })
    .expect_err("duplicate labels contributed by ** must fail");
    assert!(duplicate.contains("duplicate argument label `left`"), "unexpected duplicate error: {duplicate}");

    let computed = on_large_stack(|| {
        run_source(
            r#"
class Receiver {
  sum(left, right) { return left + right }
}
Receiver.new().sum([1]: 2, right: 3)
"#,
        )
    })
    .expect_err("computed labels must be Symbols");
    assert!(
        computed.contains("computed argument label must be Symbol, got int"),
        "unexpected computed-label error: {computed}"
    );

    let labeled = on_large_stack(|| {
        run_source(
            r#"
class Receiver {
  sum(left, right) { return left + right }
}
Receiver.new().sum(**1)
"#,
        )
    })
    .expect_err("** must reject non-labeled products");
    assert!(
        labeled.contains("** expansion requires Tuple, Unit, Record, or Map; got int"),
        "unexpected ** error: {labeled}"
    );

    let complete = on_large_stack(|| {
        run_source(
            r#"
class Receiver {
  sum(left, right) { return left + right }
}
Receiver.new().sum(***Map.new())
"#,
        )
    })
    .expect_err("*** must reject Map");
    assert!(
        complete.contains("*** expansion requires Tuple or Unit; got object"),
        "unexpected *** error: {complete}"
    );
}
