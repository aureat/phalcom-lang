//! F.2 completion proofs that are intentionally sharper than the broad
//! `outgoing_packs.rs` smoke coverage.
//!
//! These tests pin the F.3 boundary: lexical timing, duplicate short-circuiting,
//! E.3 integration, dynamic arity boundaries, selector behavior, static/dynamic
//! bytecode separation, and source non-forgeability of internal authority.

use phalcom_core::heap::{ObjRef, Object};
use phalcom_core::interpret::Interpreter;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
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

fn run_compiled_source(source: &str) -> (VM, ObjRef) {
    use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};

    let temporary = tempfile::tempdir().expect("temporary Pack view project");
    let root = temporary.path();
    std::fs::create_dir_all(root.join("src")).expect("create temporary source directory");
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"pack_view_test\"\nnamespace = \"pack_view_test\"\nversion = \"0.1.0\"\nentry = \"pack_view_test.main\"\n",
    )
    .expect("write temporary project manifest");
    std::fs::write(root.join("src/package.ph"), "expose .main\n").expect("write temporary package manifest");
    std::fs::write(root.join("src/main.ph"), source).expect("write temporary entry module");
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Project(root.to_path_buf())).expect("compile source project");
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("materialize source");
    vm.run_compiled(&program).expect("execute source");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    (vm, module)
}

fn compiled_global_value(vm: &mut VM, module: ObjRef, name: &str) -> Value {
    let symbol = vm.interner.intern(name);
    let Object::Module(module) = vm.heap.get(module) else {
        panic!("compiled test module is not a module");
    };
    module.get(symbol).unwrap_or_else(|| panic!("global `{name}` was not defined"))
}

fn instance_slot(interp: &Interpreter, value: Value, slot: usize) -> Value {
    let id = value.as_obj().expect("expected instance object");
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

class SourceIterator {
  _probe
  _current
  @constructor
  new(_ probe) { _probe = probe; _current = 0 }

  next {
    _probe.spreadStep()
    if (_current < 2) {
      _current = _current + 1
      return Some(_current)
    }
    return None
  }

  iterator { self }
}

class Source {
  _probe
  @constructor
  from(probe) { _probe = probe }

  iterator { SourceIterator.new(_probe) }
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
    assert_eq!(result.expect("timing probe must execute"), Value::int(123339));
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
        Value::int(0),
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
        Value::int(1),
        "label expression must run, but value expression must not run after Symbol validation fails"
    );
}

#[test]
fn put_is_an_ordinary_subscript_label_and_rhs_stays_separate() {
    let (mut interp, main, result) = run_source_keep(
        r#"
class Probe {
  _state
  @constructor
  new() { _state = 0 }
  sideEffect() { _state = 1; return 42 }
}
class Sink {
  [put]=(_ value) { return value }
}
let probe = Probe.new()
let sink = Sink.new()
let assignment = sink[**(put: 1,)] = probe.sideEffect()
"#,
    );

    result.expect("`put` must remain available as a user-written subscript label");
    let assignment = global_value(&mut interp, main, "assignment");
    assert_eq!(assignment, Value::int(42), "subscript assignment must evaluate to the original RHS");
    let probe = global_value(&mut interp, main, "probe");
    assert_eq!(instance_slot(&interp, probe, 0), Value::int(1), "RHS must run after dynamic index expansion");
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
from universe.collections.iterator_laws import TakeIterator

class Receiver { collect(*values) { return values.size } }
let result = Receiver.new().collect(*TakeIterator.new((0..).iterator, 3))
"#,
            "result",
        )
    });
    assert_eq!(bounded.expect("take must bound an otherwise open pipeline"), Value::int(3));
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
    assert_eq!(labeled.expect("**Unit must contribute no labels"), Value::int(7));

    let complete = on_large_stack(|| {
        eval_source(
            r#"
class Receiver { zero() { return 8 } }
let result = Receiver.new().zero(***())
"#,
            "result",
        )
    });
    assert_eq!(complete.expect("***Unit must contribute no values"), Value::int(8));
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
    assert_eq!(result.expect("label order must remain selector-significant"), Value::int(21));
}

#[test]
fn labeled_dynamic_send_does_not_fall_back_to_positional_rest() {
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
    assert_eq!(result.expect("labeled dynamic miss must go to dNU, not collect(*)"), Value::int(99));
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
    assert_eq!(result.expect("dynamic super miss must reach dNU"), Value::bool(true));
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
    assert_eq!(accepted.expect("255 dynamic args must be legal"), Value::int(255));

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
fn dynamic_subscript_set_arity_counts_the_rhs_value() {
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
    assert_eq!(accepted.expect("254 indices + RHS = 255 total args must be legal"), Value::int(7));

    let indices_255 = tuple_literal(255);
    let overflow_source = format!(
        r#"
class Sink {{ doesNotUnderstand(_ message) {{ throw Error.new("SETTER-DNU-RAN") }} }}
let indices = {indices_255}
let sink = Sink.new()
sink[*indices] = 7
"#
    );
    let error = on_large_stack(move || run_source(&overflow_source)).expect_err("255 indices + RHS must overflow");
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
    assert_eq!(value.expect("dynamic Tuple may contain more than 255 values"), Value::int(300));
}

#[test]
fn ordinary_source_cannot_mint_compiler_internal_pack_authority() {
    let error = on_large_stack(|| {
        run_source(
            r#"
class Receiver {}
let receiver = Receiver.new()
let args = (1,)
receiver.f2InternalProbe(*args)
"#,
        )
    })
    .expect_err("ordinary source must not mint compiler-internal dispatch authority");
    assert!(
        error.contains("does not understand 'f2InternalProbe(_)'"),
        "ordinary source must receive ordinary missing-message behavior: {error}"
    );
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

fn checked_disasm_source(source: &str) -> String {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(source.into())).expect("analyze source");
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("materialize analyzed program");
    let closure = vm
        .compile_program_module_closure(&program.entry, source, &program)
        .expect("compile with canonical semantic lowering");
    format!("{:?}", vm.heap.closure(closure).callable.chunk.code)
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
    let text = checked_disasm_source("let list: List<Int> = [1, 2, 3]\n");
    assert!(text.contains("BuildList(3)"), "static List did not emit BuildList:\n{text}");
    for forbidden in ["BeginListLiteral", "ListLiteralAppend", "FinishListLiteral", "ListTryExpandTuplePositionals"] {
        assert!(!text.contains(forbidden), "static List unexpectedly emitted `{forbidden}`:\n{text}");
    }
}

#[test]
fn dynamic_list_uses_rooted_incremental_builder_and_shared_spread_lane() {
    let text = checked_disasm_source("let list: List<Int> = [1, *(2, 3), 4]\n");
    for required in ["BeginListLiteral", "ListLiteralAppend", "FinishListLiteral", "ListTryExpandTuplePositionals"] {
        assert!(text.contains(required), "dynamic List did not emit `{required}`:\n{text}");
    }
    assert!(!text.contains("BuildList("), "dynamic List unexpectedly emitted BuildList:\n{text}");
}

#[test]
fn p3_fixed_pack_construction_executes() {
    on_large_stack(|| {
        let (mut interp, main, result) = run_source_keep("let result = Pack(1, 2, label: 3)\n");
        result.expect("canonical fixed Pack construction must execute");
        let symbol = interp.vm.interner.intern("result");
        let value = interp.vm.heap.module(main).get(symbol).expect("result value");
        assert_eq!(value.to_string(&interp.vm), "Pack(1, 2, label: 3)");
    });
}

#[test]
fn p3_pack_construction_bytecode_boundaries() {
    let fixed = disasm_source("let result = Pack(1, 2, label: 3)\n");
    assert!(fixed.contains("BuildPack"), "{fixed}");
    assert!(!fixed.contains("NewArgumentPack"), "{fixed}");
    let expanded = disasm_source("let result = Pack(*(1, 2), **(label: 3,))\n");
    assert_eq!(expanded.matches("NewArgumentPack").count(), 1, "{expanded}");
    assert_eq!(expanded.matches("FinishPack").count(), 1, "{expanded}");
    assert!(!expanded.contains("FinishTuplePack"), "{expanded}");
    let tuple = disasm_source("let result = (0, *(1, 2))\n");
    assert!(tuple.contains("FinishTuplePack"), "{tuple}");
    assert!(!tuple.contains("FinishPack"), "{tuple}");
    let call = disasm_source("class Receiver { take(_ value) { value } }\nReceiver.new().take(1)\n");
    for forbidden in ["NewArgumentPack", "FinishPack", "BuildPack", "BuildStaticPack"] {
        assert!(!call.contains(forbidden), "{call}");
    }
}

#[test]
fn p3_expanded_pack_and_empty_pack_execute() {
    on_large_stack(|| {
        let (mut interp, main, result) = run_source_keep("let result = Pack(*(1, 2), **(label: 3,))\nlet empty = Pack()\n");
        result.expect("expanded and empty Pack construction must execute");
        let value = global_value(&mut interp, main, "result");
        assert_eq!(value.to_string(&interp.vm), "Pack(1, 2, label: 3)");
        assert_eq!(global_value(&mut interp, main, "empty"), Value::empty_pack());
    });
}

#[test]
fn p3_analyzed_pack_construction_uses_canonical_static_product() {
    on_large_stack(|| {
        use phalcom_core::bytecode::Bytecode;
        use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
        let source = "let result = Pack(1, 2, label: 3)\nlet empty = Pack()\n";
        let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(source.into())).expect("analyze Pack construction");
        let lowering = &program.modules.get(&program.entry).expect("entry").lowering;
        assert_eq!(lowering.pack_constructions.len(), 2);
        assert_eq!(lowering.anonymous_products.len(), 2);
        let mut vm = phalcom_core::vm::VM::new();
        vm.materialize_program(&program).expect("materialize");
        let closure = vm.compile_program_module_closure(&program.entry, source, &program).expect("compile Pack");
        let code = &vm.heap.closure(closure).callable.chunk.code;
        assert_eq!(code.iter().filter(|op| matches!(op, Bytecode::BuildStaticPack { .. })).count(), 2);
        assert!(!code.iter().any(|op| matches!(op, Bytecode::NewArgumentPack | Bytecode::ConstructData { .. })));
        vm.run_compiled(&program).expect("run analyzed Pack");
        let main = vm.module_registry.get(&program.entry).expect("entry module").object;
        let result_symbol = vm.interner.intern("result");
        let value = vm.heap.module(main).get(result_symbol).expect("result");
        assert_eq!(value.to_string(&vm), "Pack(1, 2, label: 3)");
    });
}

#[test]
fn p3_shadowed_pack_name_remains_an_ordinary_call() {
    let text = disasm_source("let Pack = List<Int>.new()\nlet result = Pack(7)\n");
    for forbidden in ["BuildPack", "BuildStaticPack", "FinishPack", "NewArgumentPack"] {
        assert!(!text.contains(forbidden), "{text}");
    }
}

#[test]
fn p3_analyzed_expanded_pack_uses_one_builder() {
    on_large_stack(|| {
        use phalcom_core::bytecode::Bytecode;
        use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
        let source = "let result = Pack(*(1, 2), **(label: 3,))\n";
        let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(source.into())).expect("analyze expanded Pack");
        let lowering = &program.modules.get(&program.entry).expect("entry").lowering;
        assert_eq!(lowering.pack_constructions.len(), 1);
        let mut vm = phalcom_core::vm::VM::new();
        vm.materialize_program(&program).expect("materialize");
        let closure = vm
            .compile_program_module_closure(&program.entry, source, &program)
            .expect("compile expanded Pack");
        let code = &vm.heap.closure(closure).callable.chunk.code;
        assert_eq!(code.iter().filter(|op| matches!(op, Bytecode::NewArgumentPack)).count(), 1);
        assert_eq!(code.iter().filter(|op| matches!(op, Bytecode::FinishPack)).count(), 1);
        vm.run_compiled(&program).expect("run analyzed expanded Pack");
        let main = vm.module_registry.get(&program.entry).expect("entry module").object;
        let symbol = vm.interner.intern("result");
        let value = vm.heap.module(main).get(symbol).expect("result");
        assert_eq!(value.to_string(&vm), "Pack(1, 2, label: 3)");
    });
}

#[test]
fn p3_language_rest_capture_converges_on_pack() {
    on_large_stack(|| {
        let source = r#"
class Receiver {
  positional(*values) { return values }
  labeled(**values) { return values }
  split(*positions, **labels) { return (positions, labels) }
  complete(***values) { return values }
  relay(***values) { return self.complete(***values) }
}
let receiver = Receiver.new()
let positional_empty = receiver.positional()
let positional = receiver.positional(1, 2)
let labeled_empty = receiver.labeled()
let labeled = receiver.labeled(right: 3, left: 4)
let split = receiver.split(1, 2, right: 3, left: 4)
let complete_empty = receiver.complete()
let complete = receiver.complete(1, 2, right: 3, left: 4)
let forwarded = receiver.relay(1, 2, right: 3, left: 4)
let forwarded_from_pack = receiver.relay(***Pack(1, 2, right: 3, left: 4))
let block = |*values| { values }
let block_empty = block.call()
let block_rest = block.call(5, 6)
let fiber_empty = Fiber.new(|*values| { Fiber.yield(values) })
let fiber_empty_rest = fiber_empty.call()
let fiber_rest = Fiber.new(|*values| { Fiber.yield(values) }).call(7)
"#;
        let (mut interp, main, result) = run_source_keep(source);
        result.expect("all language rest activation paths must accept Pack captures");

        assert_eq!(global_value(&mut interp, main, "positional_empty"), Value::empty_pack());
        assert_eq!(global_value(&mut interp, main, "labeled_empty"), Value::empty_pack());
        assert_eq!(global_value(&mut interp, main, "complete_empty"), Value::empty_pack());
        assert_eq!(global_value(&mut interp, main, "block_empty"), Value::empty_pack());
        assert_eq!(global_value(&mut interp, main, "fiber_empty_rest"), Value::empty_pack());

        let rendered = |name: &str, interp: &mut Interpreter| global_value(interp, main, name).to_string(&interp.vm);
        assert_eq!(rendered("positional", &mut interp), "Pack(1, 2)");
        assert_eq!(rendered("labeled", &mut interp), "Pack(right: 3, left: 4)");
        assert_eq!(rendered("split", &mut interp), "(Pack(1, 2), Pack(right: 3, left: 4))");
        assert_eq!(rendered("complete", &mut interp), "Pack(1, 2, right: 3, left: 4)");
        assert_eq!(rendered("forwarded", &mut interp), "Pack(1, 2, right: 3, left: 4)");
        assert_eq!(rendered("forwarded_from_pack", &mut interp), "Pack(1, 2, right: 3, left: 4)");
        assert_eq!(rendered("block_rest", &mut interp), "Pack(5, 6)");
        assert_eq!(rendered("fiber_rest", &mut interp), "Pack(7)");
    });
}

#[test]
fn p3_pack_spreading_preserves_lanes_and_fuses_builders() {
    on_large_stack(|| {
        let source = r#"
class Receiver {
  positional(*values) { return values }
  labeled(**values) { return values }
  complete(***values) { return values }
}
let receiver = Receiver.new()
let source = Pack(1, 2, right: 3, left: 4)
let positionals = receiver.positional(*source)
let labels = receiver.labeled(**source)
let complete = receiver.complete(***source)
let tuple = (***source,)
let roundtrip = Pack(***tuple)
let multi = Pack(*Pack(1, 2), *Pack(3, 4))
let call_multi = receiver.positional(*Pack(5, 6), *Pack(7, 8))
let identities = tuple is not Pack and roundtrip is Pack
"#;
        let (mut interp, main, result) = run_source_keep(source);
        result.expect("Pack spreads must preserve their visible lanes");
        let rendered = |name: &str, interp: &mut Interpreter| global_value(interp, main, name).to_string(&interp.vm);
        assert_eq!(rendered("positionals", &mut interp), "Pack(1, 2)");
        assert_eq!(rendered("labels", &mut interp), "Pack(right: 3, left: 4)");
        assert_eq!(rendered("complete", &mut interp), "Pack(1, 2, right: 3, left: 4)");
        let tuple_id = global_value(&mut interp, main, "tuple").as_obj().expect("nonempty Tuple object");
        let expected_labels = [interp.vm.interner.intern("right"), interp.vm.interner.intern("left")];
        let tuple_view = interp.vm.tuple_view(tuple_id).expect("complete spread constructs Tuple");
        assert_eq!(tuple_view.positionals(), [Value::int(1), Value::int(2)]);
        assert_eq!(tuple_view.labels(), expected_labels);
        assert_eq!(tuple_view.labeled_values(), [Value::int(3), Value::int(4)]);
        assert_eq!(rendered("roundtrip", &mut interp), "Pack(1, 2, right: 3, left: 4)");
        assert_eq!(rendered("multi", &mut interp), "Pack(1, 2, 3, 4)");
        assert_eq!(rendered("call_multi", &mut interp), "Pack(5, 6, 7, 8)");
        assert_eq!(global_value(&mut interp, main, "identities"), Value::bool(true));

        let construction = disasm_source("let result = Pack(*Pack(1, 2), *Pack(3, 4))\n");
        assert_eq!(construction.matches("NewArgumentPack").count(), 1, "{construction}");
        assert_eq!(construction.matches("FinishPack").count(), 1, "{construction}");
        let call = disasm_source(
            "class Receiver { collect(*values) { return values } }\nlet receiver = Receiver.new()\nlet result = receiver.collect(*Pack(1), *Pack(2))\n",
        );
        assert_eq!(call.matches("NewArgumentPack").count(), 1, "{call}");
        assert_eq!(call.matches("InvokePack").count(), 1, "{call}");
    });
}

#[test]
fn p3_duplicate_from_earlier_pack_spread_prevents_later_value() {
    let (mut interp, main, result) = run_source_keep(
        r#"
let mut state = 0
let probe = || { state = 1; return 99 }
let result = Pack(**Pack(left: 1), left: probe())
"#,
    );
    let error = result.expect_err("a duplicate from Pack expansion must fail");
    assert!(error.contains("duplicate argument label `left`"), "unexpected error: {error}");
    assert_eq!(global_value(&mut interp, main, "state"), Value::int(0), "the later explicit value must not run");
}

#[test]
fn p3_pack_source_views_decompose_one_step_over_shared_roots() {
    on_large_stack(|| {
        let source = r#"
from universe.collections.pack import PositionalPackView, LabeledPackView
let empty_positional = match Pack().positionalView {
  PositionalPackView::Empty => true
  _ => false
}
let empty_labeled = match Pack().labeledView {
  LabeledPackView::Empty => true
  _ => false
}
let positional_root = Pack(10, 20, 30)
let positional_view = positional_root.positionalView
let positional_first = match positional_view {
  PositionalPackView::Empty => -1
  PositionalPackView::Entry(value, _) => value
  _ => -2
}
let positional_rest_a = match positional_view {
  PositionalPackView::Empty => Pack()
  PositionalPackView::Entry(_, rest) => rest
  _ => Pack()
}
let positional_rest_b = match positional_root.positionalView {
  PositionalPackView::Empty => Pack()
  PositionalPackView::Entry(_, rest) => rest
  _ => Pack()
}
let positional_rest_next = match positional_rest_a.positionalView {
  PositionalPackView::Empty => Pack()
  PositionalPackView::Entry(_, rest) => rest
  _ => Pack()
}
let labeled_root = Pack(right: 40, left: 50)
let labeled_view = labeled_root.labeledView
let labeled_first = match labeled_view {
  LabeledPackView::Empty => -1
  LabeledPackView::Entry(_, value, _) => value
  _ => -2
}
let labeled_first_name = match labeled_view {
  LabeledPackView::Empty => Symbol.new("")
  LabeledPackView::Entry(label, _, _) => label
  _ => Symbol.new("")
}
let labeled_rest = match labeled_view {
  LabeledPackView::Empty => Pack()
  LabeledPackView::Entry(_, _, rest) => rest
  _ => Pack()
}
let labeled_second_name = match labeled_rest.labeledView {
  LabeledPackView::Empty => Symbol.new("")
  LabeledPackView::Entry(label, _, _) => label
  _ => Symbol.new("")
}
"#;
        let (mut vm, main) = run_compiled_source(source);

        assert_eq!(compiled_global_value(&mut vm, main, "empty_positional"), Value::bool(true));
        assert_eq!(compiled_global_value(&mut vm, main, "empty_labeled"), Value::bool(true));
        assert_eq!(compiled_global_value(&mut vm, main, "positional_first"), Value::int(10));
        assert_eq!(compiled_global_value(&mut vm, main, "labeled_first"), Value::int(40));
        for (name, expected) in [("labeled_first_name", "right"), ("labeled_second_name", "left")] {
            let symbol = compiled_global_value(&mut vm, main, name).as_symbol().expect("view labels are Symbols");
            assert_eq!(vm.resolve_symbol(symbol), expected);
        }

        let root = compiled_global_value(&mut vm, main, "positional_root")
            .as_obj()
            .expect("positional root object");
        for (name, expected_start) in [("positional_rest_a", 1), ("positional_rest_b", 1), ("positional_rest_next", 2)] {
            let slice = compiled_global_value(&mut vm, main, name).as_obj().expect("rest view must be a Pack slice");
            let pack = vm.heap.pack(slice);
            let phalcom_core::heap::pack::PackStorage::Slice { root: slice_root } = pack.storage() else {
                panic!("{name} must retain shared root storage");
            };
            assert_eq!(*slice_root, root, "{name} must point directly at the original Pack root");
            assert_eq!(pack.positional_start(), expected_start);
        }

        let labeled_root = compiled_global_value(&mut vm, main, "labeled_root").as_obj().expect("labeled root object");
        let labeled_rest = compiled_global_value(&mut vm, main, "labeled_rest").as_obj().expect("labeled rest Pack slice");
        let labeled_pack = vm.heap.pack(labeled_rest);
        let phalcom_core::heap::pack::PackStorage::Slice { root: slice_root } = labeled_pack.storage() else {
            panic!("labeled rest must retain shared root storage");
        };
        assert_eq!(*slice_root, labeled_root);
        assert_eq!(labeled_pack.labeled_start(), 1);
    });
}

#[test]
fn p3_pack_source_equality_hash_and_rendering_use_visible_lanes() {
    on_large_stack(|| {
        let source = r#"
from universe.collections.pack import PositionalPackView, LabeledPackView
let root = Pack(1, 2, right: 3, left: 4)
let equal_root = Pack(1, 2, right: 3, left: 4)
let padded = Pack(0, 1, 2, hidden: 99, right: 3, left: 4)
let after_positional = match padded.positionalView {
  PositionalPackView::Empty => Pack()
  PositionalPackView::Entry(_, rest) => rest
  _ => Pack()
}
let slice = match after_positional.labeledView {
  LabeledPackView::Empty => Pack()
  LabeledPackView::Entry(_, _, rest) => rest
  _ => Pack()
}
let root_equal = root == equal_root
let slice_equal = root == slice
let root_hash_equal = root.hash == equal_root.hash
let slice_hash_equal = root.hash == slice.hash
let position_order_differs = Pack(1, 2) != Pack(2, 1)
let label_order_differs = Pack(left: 1, right: 2) != Pack(right: 2, left: 1)
let label_identity_differs = Pack(left: 1) != Pack(right: 1)
let lane_differs = Pack(1, item: 2) != Pack(1, 2)
let nested_equal = Pack(Pack(7)) == Pack(Pack(7))
let nested_hash_equal = Pack(Pack(7)).hash == Pack(Pack(7)).hash
"#;
        let (mut vm, main) = run_compiled_source(source);

        for name in [
            "root_equal",
            "slice_equal",
            "root_hash_equal",
            "slice_hash_equal",
            "nested_equal",
            "nested_hash_equal",
        ] {
            assert_eq!(compiled_global_value(&mut vm, main, name), Value::bool(true), "{name}");
        }
        for name in ["position_order_differs", "label_order_differs", "label_identity_differs", "lane_differs"] {
            assert_eq!(compiled_global_value(&mut vm, main, name), Value::bool(true), "{name}");
        }

        let root = compiled_global_value(&mut vm, main, "root");
        assert_eq!(root.to_string(&vm), "Pack(1, 2, right: 3, left: 4)");
    });
}
