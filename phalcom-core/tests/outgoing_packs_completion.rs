//! F.2 completion proofs that are intentionally sharper than the broad
//! `outgoing_packs.rs` smoke coverage.
//!
//! These tests pin the F.3 boundary: lexical timing, duplicate short-circuiting,
//! E.3 integration, dynamic arity boundaries, selector behavior, static/dynamic
//! bytecode separation, and source non-forgeability of internal authority.

use phalcom_core::heap::{ObjRef, Object};
use phalcom_core::interpret::Interpreter;
use phalcom_core::value::Value;
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn on_large_stack<T: Send + 'static>(test: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(test)
        .expect("spawn F.2 completion test thread")
        .join()
        .expect("F.2 completion test thread")
}

fn eval_source(src: &str, var_name: &str) -> Result<Value, String> {
    let mut interp = Interpreter::new();
    let main = interp.vm.create_module("main", "<f2 completion test>");
    interp.vm.interpret_source(main, src).map_err(|error| error.to_string())?;
    let symbol = interp.vm.interner.intern(var_name);
    let Object::Module(module) = interp.vm.heap.get(main) else {
        return Err("main module is not a module".to_string());
    };
    module.get(symbol).ok_or_else(|| format!("variable `{var_name}` not found"))
}

fn run_source(src: &str) -> Result<(), String> {
    let mut interp = Interpreter::new();
    let main = interp.vm.create_module("main", "<f2 completion test>");
    interp.vm.interpret_source(main, src).map_err(|error| error.to_string())
}

fn run_source_keep(src: &str) -> (Interpreter, ObjRef, Result<(), String>) {
    let mut interp = Interpreter::new();
    let main = interp.vm.create_module("main", "<f2 completion test>");
    let result = interp.vm.interpret_source(main, src).map_err(|error| error.to_string());
    (interp, main, result)
}

fn global_value(interp: &mut Interpreter, module: ObjRef, name: &str) -> Value {
    let symbol = interp.vm.interner.intern(name);
    let Object::Module(module) = interp.vm.heap.get(module) else {
        panic!("test module is not a module");
    };
    module
        .get(symbol)
        .unwrap_or_else(|| panic!("global `{name}` was not defined before the expected failure"))
}

fn instance_slot(interp: &Interpreter, value: Value, slot: usize) -> Value {
    let Value::Obj(id) = value else {
        panic!("expected instance object, got {value:?}");
    };
    let Object::Instance(instance) = interp.vm.heap.get(id) else {
        panic!("expected instance object");
    };
    instance.slots[slot]
}

fn tuple_literal(count: usize) -> String {
    let values = (0..count).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
    format!("({values})")
}

#[test]
fn receiver_runs_first_spread_operand_runs_once_and_generic_spread_finishes_before_later_item() {
    let result = on_large_stack(|| {
        eval_source(
            r#"
class Probe {
  _state
  @constructor
  new() { _state = 0 }

  receiver(_ value) {
    _state = _state * 10 + 1
    return value
  }

  makeSource() {
    _state = _state * 10 + 2
    return Source.from(probe: self)
  }

  spreadStep() { _state = _state * 10 + 3 }

  later() {
    _state = _state * 10 + 9
    return _state
  }
}

class Source is Iterable {
  _probe
  @constructor
  from(probe) { _probe = probe }

  iterate(_ cursor) {
    _probe.spreadStep()
    let next = (cursor == None).ifTrue(|| { 0 }, ifFalse: || { cursor + 1 })
    return (next < 2).ifTrue(|| { next }, ifFalse: || { None })
  }

  iteratorValue(_ cursor) { return cursor + 1 }
}

class Receiver {
  accept(_ first, _ second, _ later) { return later }
}

let probe = Probe.new()
let receiver = Receiver.new()
let result = probe.receiver(receiver).accept(*probe.makeSource(), probe.later())
"#,
            "result",
        )
    });

    // 1 receiver, 2 operand creation, 3/3/3 complete cursor exhaustion, 9 later.
    // Any operand duplication, receiver reordering, or interleaving of `later()`
    // changes this value.
    assert_eq!(result.expect("timing probe must execute"), Value::Int(123339));
}

#[test]
fn duplicate_from_earlier_expansion_prevents_later_explicit_value() {
    let (mut interp, main, result) = run_source_keep(
        r#"
class Probe {
  _state
  @constructor
  new() { _state = 0 }
  sideEffect() { _state = 1; return 99 }
}
class Receiver {
  take(left) { return left }
}
let probe = Probe.new()
Receiver.new().take(**(left: 1,), left: probe.sideEffect())
"#,
    );

    let error = result.expect_err("duplicate label must fail");
    assert!(error.contains("duplicate argument label `left`"), "unexpected error: {error}");
    let probe = global_value(&mut interp, main, "probe");
    assert_eq!(
        instance_slot(&interp, probe, 0),
        Value::Int(0),
        "duplicate reservation must precede value evaluation"
    );
}

#[test]
fn computed_label_validation_precedes_its_value_expression() {
    let (mut interp, main, result) = run_source_keep(
        r#"
class Probe {
  _state
  @constructor
  new() { _state = 0 }
  badLabel() { _state = 1; return 7 }
  sideEffect() { _state = 99; return 2 }
}
class Receiver {
  take(value) { return value }
}
let probe = Probe.new()
Receiver.new().take([probe.badLabel()]: probe.sideEffect())
"#,
    );

    let error = result.expect_err("non-Symbol computed label must fail");
    assert!(error.contains("computed argument label must be Symbol, got int"), "unexpected error: {error}");
    let probe = global_value(&mut interp, main, "probe");
    assert_eq!(
        instance_slot(&interp, probe, 0),
        Value::Int(1),
        "label expression must run, but value expression must not run after Symbol validation fails"
    );
}

#[test]
fn implicit_put_duplicate_prevents_subscript_rhs() {
    let (mut interp, main, result) = run_source_keep(
        r#"
class Probe {
  _state
  @constructor
  new() { _state = 0 }
  sideEffect() { _state = 1; return 42 }
}
class Sink {}
let probe = Probe.new()
let sink = Sink.new()
sink[**(put: 1,)] = probe.sideEffect()
"#,
    );

    let error = result.expect_err("dynamic setter must reserve compiler-owned put before RHS");
    assert!(error.contains("duplicate argument label `put`"), "unexpected error: {error}");
    let probe = global_value(&mut interp, main, "probe");
    assert_eq!(instance_slot(&interp, probe, 0), Value::Int(0), "RHS must not run after duplicate `put`");
}

#[test]
fn positional_spread_reuses_e3_at_the_actual_pack_site() {
    let direct = on_large_stack(|| {
        run_source(
            r#"
class Receiver { collect(*values) { return values.size } }
Receiver.new().collect(*(0..))
"#,
        )
    })
    .expect_err("open Range must be rejected before generic exhaustion");
    assert!(
        direct.contains("cannot exhaust a provably unbounded source with `expansion`"),
        "unexpected direct unbounded error: {direct}"
    );

    let through_const = on_large_stack(|| {
        run_source(
            r#"
class Receiver { collect(*values) { return values.size } }
const source = (0..)
Receiver.new().collect(*source)
"#,
        )
    })
    .expect_err("immutable source fact must preserve unboundedness");
    assert!(
        through_const.contains("cannot exhaust a provably unbounded source with `expansion`"),
        "unexpected const-fact error: {through_const}"
    );

    let bounded = on_large_stack(|| {
        eval_source(
            r#"
class Receiver { collect(*values) { return values.size } }
let result = Receiver.new().collect(*(0..).iter.take(3))
"#,
            "result",
        )
    });
    assert_eq!(bounded.expect("take must bound an otherwise open pipeline"), Value::Int(3));
}

#[test]
fn non_iterable_star_operand_is_a_pack_specific_runtime_error() {
    let error = on_large_stack(|| {
        run_source(
            r#"
class Receiver { collect(*values) { return values.size } }
Receiver.new().collect(*42)
"#,
        )
    })
    .expect_err("non-Iterable positional spread must fail");
    assert!(
        error.contains("* expansion requires Tuple, Unit, or an iterable value; got int"),
        "unexpected error: {error}"
    );
}

#[test]
fn labeled_and_complete_expansion_edge_errors_are_structured() {
    let map_error = on_large_stack(|| {
        run_source(
            r#"
class Receiver { take(value) { return value } }
let labels = Map.new()
labels[1] = 2
Receiver.new().take(**labels)
"#,
        )
    })
    .expect_err("non-Symbol Map key must fail during **");
    assert!(
        map_error.contains("Map key in ** expansion must be Symbol; got int"),
        "unexpected Map expansion error: {map_error}"
    );

    let duplicate = on_large_stack(|| {
        run_source(
            r#"
class Receiver { take(left) { return left } }
Receiver.new().take(**(left: 1,), **(left: 2,))
"#,
        )
    })
    .expect_err("duplicate across two ** contributions must fail");
    assert!(duplicate.contains("duplicate argument label `left`"), "unexpected duplicate error: {duplicate}");

    let complete = on_large_stack(|| {
        run_source(
            r#"
class Receiver { take(_ value) { return value } }
Receiver.new().take(***[1])
"#,
        )
    })
    .expect_err("*** List must fail");
    assert!(
        complete.contains("*** expansion requires Tuple or Unit; got object"),
        "unexpected complete-expansion error: {complete}"
    );
}

#[test]
fn unit_expansions_are_empty_in_labeled_and_complete_lanes() {
    let labeled = on_large_stack(|| {
        eval_source(
            r#"
class Receiver { zero() { return 7 } }
let result = Receiver.new().zero(**())
"#,
            "result",
        )
    });
    assert_eq!(labeled.expect("**Unit must contribute no labels"), Value::Int(7));

    let complete = on_large_stack(|| {
        eval_source(
            r#"
class Receiver { zero() { return 8 } }
let result = Receiver.new().zero(***())
"#,
            "result",
        )
    });
    assert_eq!(complete.expect("***Unit must contribute no values"), Value::Int(8));
}

#[test]
fn label_encounter_order_selects_the_concrete_dynamic_selector() {
    let result = on_large_stack(|| {
        eval_source(
            r#"
class Receiver {
  pick(left, right) { return 12 }
  pick(right, left) { return 21 }
}
let result = Receiver.new().pick(**(right: 2, left: 1))
"#,
            "result",
        )
    });
    assert_eq!(result.expect("label order must remain selector-significant"), Value::Int(21));
}

#[test]
fn labeled_dynamic_send_does_not_fall_back_to_positional_variadic() {
    let result = on_large_stack(|| {
        eval_source(
            r#"
class Receiver {
  collect(*values) { return 1 }
  doesNotUnderstand(_ message) { return 99 }
}
let result = Receiver.new().collect(**(x: 1,))
"#,
            "result",
        )
    });
    assert_eq!(result.expect("labeled dynamic miss must go to dNU, not collect(*)"), Value::Int(99));
}

#[test]
fn dynamic_super_miss_forwards_the_concrete_selector_to_dnu() {
    let result = on_large_stack(|| {
        eval_source(
            r#"
class Parent {
  doesNotUnderstand(_ message) { return message.selector == #missing(_) }
}
class Child is Parent {
  probe(_ args) { return super.missing(*args) }
}
let result = Child.new().probe((1,))
"#,
            "result",
        )
    });
    assert_eq!(result.expect("dynamic super miss must reach dNU"), Value::Bool(true));
}

#[test]
fn dynamic_method_arity_accepts_255_and_rejects_256_before_dispatch() {
    let tuple_255 = tuple_literal(255);
    let ok_source = format!(
        r#"
class Receiver {{ collect(*values) {{ return values.size }} }}
let args = {tuple_255}
let result = Receiver.new().collect(*args)
"#
    );
    let accepted = on_large_stack(move || eval_source(&ok_source, "result"));
    assert_eq!(accepted.expect("255 dynamic args must be legal"), Value::Int(255));

    let tuple_256 = tuple_literal(256);
    let overflow_source = format!(
        r#"
class Receiver {{
  doesNotUnderstand(_ message) {{ throw Error.new("DNU-RAN") }}
}}
let args = {tuple_256}
Receiver.new().missing(*args)
"#
    );
    let error = on_large_stack(move || run_source(&overflow_source)).expect_err("256 dynamic args must fail");
    assert!(
        error.contains("dynamic send has 256 arguments; limit is 255"),
        "unexpected arity error: {error}"
    );
    assert!(!error.contains("DNU-RAN"), "arity validation must happen before lookup/dNU: {error}");
}

#[test]
fn dynamic_subscript_set_arity_counts_the_implicit_put_value() {
    let indices_254 = tuple_literal(254);
    let ok_source = format!(
        r#"
class Sink {{ doesNotUnderstand(_ message) {{ return 0 }} }}
let indices = {indices_254}
let sink = Sink.new()
let result = sink[*indices] = 7
"#
    );
    let accepted = on_large_stack(move || eval_source(&ok_source, "result"));
    assert_eq!(accepted.expect("254 indices + put/RHS = 255 total args must be legal"), Value::Int(7));

    let indices_255 = tuple_literal(255);
    let overflow_source = format!(
        r#"
class Sink {{ doesNotUnderstand(_ message) {{ throw Error.new("SETTER-DNU-RAN") }} }}
let indices = {indices_255}
let sink = Sink.new()
sink[*indices] = 7
"#
    );
    let error = on_large_stack(move || run_source(&overflow_source)).expect_err("255 indices + put/RHS must overflow");
    assert!(
        error.contains("dynamic send has 256 arguments; limit is 255"),
        "unexpected setter arity error: {error}"
    );
    assert!(
        !error.contains("SETTER-DNU-RAN"),
        "setter overflow must be rejected before selector lookup/dNU: {error}"
    );
}

#[test]
fn dynamic_tuple_is_not_subject_to_the_send_arity_limit() {
    let source_tuple = tuple_literal(300);
    let source = format!(
        r#"
let source = {source_tuple}
let tuple = (***source,)
let result = tuple.size
"#
    );
    let value = on_large_stack(move || eval_source(&source, "result"));
    assert_eq!(value.expect("dynamic Tuple may contain more than 255 values"), Value::Int(300));
}

#[test]
fn ordinary_source_cannot_mint_compiler_internal_pack_authority() {
    let error = on_large_stack(|| {
        run_source(
            r#"
class Receiver {}
let receiver = Receiver.new()
let args = (1,)
receiver._$f2InternalProbe(*args)
"#,
        )
    })
    .expect_err("ordinary source must not spell an internal dynamic selector");
    assert!(error.contains("internal.namespace_reserved"), "unexpected internal-namespace error: {error}");
}

static DISASM_COUNTER: AtomicU64 = AtomicU64::new(0);

fn disasm_source(source: &str) -> String {
    let n = DISASM_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("phalcom-f2-pack-disasm-{}-{n}.ph", std::process::id()));
    fs::write(&path, source).expect("write temporary Phalcom disassembly source");
    let output = Command::new(env!("CARGO_BIN_EXE_phalcom"))
        .arg("disasm")
        .arg(&path)
        .output()
        .expect("run phalcom disasm");
    let _ = fs::remove_file(&path);
    assert!(
        output.status.success(),
        "disassembly failed. stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn static_calls_and_static_tuple_emit_no_pack_machinery() {
    let text = disasm_source(
        r#"
class Receiver {
  zero() { return 0 }
  one(_ value) { return value }
  labeled(timeout) { return timeout }
  [_ index] { return index }
}
let receiver = Receiver.new()
receiver.zero()
receiver.one(1)
receiver.labeled(timeout: 2)
receiver[3]
let product = (1, 2, label: 3)
"#,
    );

    assert!(text.contains("Invoke("), "expected ordinary Invoke in static disassembly:\n{text}");
    assert!(text.contains("BuildTuple"), "expected BuildTuple in static Tuple disassembly:\n{text}");
    for forbidden in [
        "NewArgumentPack",
        "PackPushPositional",
        "PackReserveStaticLabel",
        "PackReserveComputedLabel",
        "PackFillReservedLabel",
        "PackExpandLabels",
        "PackExpandComplete",
        "PackTryExpandTuplePositionals",
        "InvokePack",
        "SuperSendPack",
        "FinishTuplePack",
    ] {
        assert!(!text.contains(forbidden), "static source unexpectedly emitted `{forbidden}`:\n{text}");
    }
}

#[test]
fn dynamic_send_and_dynamic_tuple_emit_the_pack_lane() {
    let text = disasm_source(
        r#"
class Receiver { collect(*values) { return values.size } }
let values = (1, 2)
let count = Receiver.new().collect(*values)
let product = (0, *values)
"#,
    );

    for required in [
        "NewArgumentPack",
        "PackTryExpandTuplePositionals",
        "PackPushPositional",
        "InvokePack",
        "FinishTuplePack",
        "JumpIfNone(",
        "Loop(",
    ] {
        assert!(text.contains(required), "dynamic source did not emit `{required}`:\n{text}");
    }
}

#[test]
fn static_list_keeps_direct_build_fast_path() {
    let text = disasm_source("let list = [1, 2, 3]\n");
    assert!(text.contains("BuildList(3)"), "static List did not emit BuildList:\n{text}");
    for forbidden in ["BeginListLiteral", "ListLiteralAppend", "FinishListLiteral", "ListTryExpandTuplePositionals"] {
        assert!(!text.contains(forbidden), "static List unexpectedly emitted `{forbidden}`:\n{text}");
    }
}

#[test]
fn dynamic_list_uses_rooted_incremental_builder_and_shared_spread_lane() {
    let text = disasm_source("let list = [1, *(2, 3), 4]\n");
    for required in ["BeginListLiteral", "ListLiteralAppend", "FinishListLiteral", "ListTryExpandTuplePositionals"] {
        assert!(text.contains(required), "dynamic List did not emit `{required}`:\n{text}");
    }
    assert!(!text.contains("BuildList("), "dynamic List unexpectedly emitted BuildList:\n{text}");
}
