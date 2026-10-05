use phalcom_core::bytecode::Bytecode;
use phalcom_core::error::{PhError, RuntimeError};
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::sync::Arc;

#[test]
fn uninitialized_binding_is_not_none() {
    let v = Value::uninitialized_binding();
    assert!(v.is_uninitialized_binding());
    assert!(!v.is_none());
    assert!(!v.is_nil());
    assert_ne!(v, Value::none());
}

#[test]
fn uninitialized_binding_sentinel_does_not_allocate_heap_objects() {
    let vm = VM::new_kernel();
    let before = vm.heap.live_count();
    for _ in 0..128 {
        std::hint::black_box(Value::uninitialized_binding());
    }
    assert_eq!(vm.heap.live_count(), before, "the local sentinel is an immediate Value, not a heap object");
}

#[test]
fn uninitialized_binding_is_not_nil_if_separate_tag() {
    let v = Value::uninitialized_binding();
    let nil = Value::nil();
    assert_ne!(v, nil);
    assert!(!v.is_nil());
    assert!(!nil.is_uninitialized_binding());
}

#[test]
fn uninitialized_binding_cannot_wrap_some() {
    let v = Value::uninitialized_binding();
    assert!(v.wrap_some().is_err(), "uninitialized binding must never be wrapped in Some");
}

fn assert_uninitialized_binding(err: PhError, expected_name: &str) {
    match err {
        PhError::Runtime(RuntimeError::UninitializedBinding { name }) => {
            assert_eq!(name, expected_name);
        }
        PhError::ModuleInitialization(ref init_err) => match init_err.failure.as_ref() {
            phalcom_core::modules::ModuleFailure::Initializer { cause } => match cause.as_ref() {
                PhError::Runtime(RuntimeError::UninitializedBinding { name }) => {
                    assert_eq!(name, expected_name);
                }
                other => panic!("expected UninitializedBinding in Initializer, got: {:?}", other),
            },
            other => panic!("expected Initializer failure, got: {:?}", other),
        },
        other => panic!("expected UninitializedBinding error, got: {:?}", other),
    }
}

#[test]
fn checked_local_traps() {
    let source = r#"
class Checker {
  @class run(flag: Bool) -> Int {
    let mut x: Int
    if flag {
      x = 10
    }
    x
  }
}

let result = Checker.run(flag: false)
"#;
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source)))
        .map_err(PhError::from)
        .expect("source should compile");
    let err = vm.run_compiled(&program).expect_err("should trap at runtime on uninitialized read");
    assert_uninitialized_binding(err, "x");
}

#[test]
fn checked_local_after_assignment_succeeds() {
    let source = r#"
class Checker {
  @class run(flag: Bool) -> Int {
    let mut x: Int
    if flag {
      x = 10
    } else {
      x = 20
    }
    x
  }
}

let result = Checker.run(flag: true)
"#;
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source)))
        .map_err(PhError::from)
        .expect("source should compile");
    vm.run_compiled(&program).expect("should succeed after assignment");
}

#[test]
fn checked_upvalue_traps_then_succeeds() {
    let trap_source = r#"
class Checker {
  @class run(flag: Bool) -> Int {
    let mut x: Int
    let getter = || { x }
    if flag {
      x = 42
    }
    getter.call()
  }
}

let result = Checker.run(flag: false)
"#;
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(trap_source)))
        .map_err(PhError::from)
        .expect("source should compile");
    let err = vm.run_compiled(&program).expect_err("reading uninitialized upvalue should trap");
    assert_uninitialized_binding(err, "x");

    let ok_source = r#"
class Checker {
  @class run(flag: Bool) -> Int {
    let mut x: Int
    let getter = || { x }
    if flag {
      x = 42
    }
    getter.call()
  }
}

let result = Checker.run(flag: true)
"#;
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(ok_source)))
        .map_err(PhError::from)
        .expect("source should compile");
    vm.run_compiled(&program).expect("upvalue read after write should succeed");
}

#[test]
fn checked_global_traps() {
    let source = r#"
let mut g: Int

class Reader {
  @class read() -> Int {
    g
  }
}

let result = Reader.read()
"#;
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source)))
        .map_err(PhError::from)
        .expect("source should compile");
    let err = vm.run_compiled(&program).expect_err("reading uninitialized global should trap");
    assert_uninitialized_binding(err, "g");
}

#[test]
fn explicit_none_checked_read_succeeds() {
    let source = r#"
class OptionTester {
  @class run() -> Option[Int] {
    let mut x: Option[Int] = None
    x
  }
}

let result = OptionTester.run()
"#;
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source)))
        .map_err(PhError::from)
        .expect("source should compile");
    vm.run_compiled(&program).expect("reading explicit None should succeed");
}

#[test]
fn proven_local_uses_unchecked_getlocal() {
    let source = r#"
let compute = || {
  let mut x = 5
  x = x + 1
  x
}
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source)))
        .map_err(PhError::from)
        .expect("source should compile");
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("materialize");
    let closure = vm.compile_program_module_closure(&program.entry, source, &program).expect("compile closure");

    let top_chunk = &vm.heap.closure(closure).callable.chunk;
    let inner_closure_ref = top_chunk
        .constants
        .iter()
        .find_map(|v| {
            if let Some(obj) = v.as_obj() {
                if vm.heap.as_closure(obj).is_some() {
                    return Some(obj);
                }
            }
            None
        })
        .expect("inner closure found");
    let chunk = &vm.heap.closure(inner_closure_ref).callable.chunk;

    assert!(
        chunk.code.iter().any(|op| matches!(op, Bytecode::GetLocal(_))),
        "proven local should emit GetLocal"
    );
    assert!(
        !chunk.code.iter().any(|op| matches!(op, Bytecode::GetLocalChecked(..))),
        "proven local must not emit GetLocalChecked"
    );
}

#[test]
fn maybe_local_uses_checked_getlocal() {
    let source = r#"
let compute = |flag: Bool| {
  let mut x: Int
  if flag {
    x = 5
  }
  x
}
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source)))
        .map_err(PhError::from)
        .expect("source should compile");
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("materialize");
    let closure = vm.compile_program_module_closure(&program.entry, source, &program).expect("compile closure");

    let top_chunk = &vm.heap.closure(closure).callable.chunk;
    let inner_closure_ref = top_chunk
        .constants
        .iter()
        .find_map(|v| {
            if let Some(obj) = v.as_obj() {
                if vm.heap.as_closure(obj).is_some() {
                    return Some(obj);
                }
            }
            None
        })
        .expect("inner closure found");
    let chunk = &vm.heap.closure(inner_closure_ref).callable.chunk;

    assert!(
        chunk.code.iter().any(|op| matches!(op, Bytecode::GetLocalChecked(..))),
        "uncertain local should emit GetLocalChecked"
    );
}

#[test]
fn initialization_flow_edit_relowers_local_read_from_unchecked_to_checked() {
    fn read_opcodes(source: &str) -> (bool, bool) {
        let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source)))
            .map_err(PhError::from)
            .expect("source should compile");
        let mut vm = VM::new();
        vm.materialize_program(&program).expect("materialize");
        let closure = vm.compile_program_module_closure(&program.entry, source, &program).expect("compile closure");
        let top_chunk = &vm.heap.closure(closure).callable.chunk;
        let inner_closure_ref = top_chunk
            .constants
            .iter()
            .find_map(|value| value.as_obj().filter(|object| vm.heap.as_closure(*object).is_some()))
            .expect("compute closure");
        let chunk = &vm.heap.closure(inner_closure_ref).callable.chunk;
        (
            chunk.code.iter().any(|op| matches!(op, Bytecode::GetLocal(_))),
            chunk.code.iter().any(|op| matches!(op, Bytecode::GetLocalChecked(..))),
        )
    }

    let definitely_initialized = r#"
let compute = |flag: Bool| {
  let mut x: Int
  if flag { x = 5 } else { x = 6 }
  x
}
"#;
    let maybe_initialized = r#"
let compute = |flag: Bool| {
  let mut x: Int
  if flag { x = 5 }
  x
}
"#;

    let (has_unchecked, has_checked) = read_opcodes(definitely_initialized);
    assert!(has_unchecked, "definitely initialized local should use GetLocal");
    assert!(!has_checked, "definitely initialized local should not branch through GetLocalChecked");

    let (has_unchecked, has_checked) = read_opcodes(maybe_initialized);
    assert!(has_unchecked, "other proven locals, such as `flag`, still use GetLocal");
    assert!(has_checked, "maybe initialized local should use GetLocalChecked");
}

#[test]
fn checked_getlocal_is_not_fused_into_unchecked_invoke_local() {
    let source = r#"
let compute = |flag: Bool| {
  let mut x: Int
  if flag {
    x = 5
  }
  x.toString()
}
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source)))
        .map_err(PhError::from)
        .expect("source should compile");
    let mut vm = VM::new();
    vm.materialize_program(&program).expect("materialize");
    let closure = vm.compile_program_module_closure(&program.entry, source, &program).expect("compile closure");

    let top_chunk = &vm.heap.closure(closure).callable.chunk;
    let inner_closure_ref = top_chunk
        .constants
        .iter()
        .find_map(|v| {
            if let Some(obj) = v.as_obj() {
                if vm.heap.as_closure(obj).is_some() {
                    return Some(obj);
                }
            }
            None
        })
        .expect("inner closure found");
    let chunk = &vm.heap.closure(inner_closure_ref).callable.chunk;

    assert!(
        !chunk.code.iter().any(|op| matches!(op, Bytecode::InvokeLocal(..))),
        "checked getlocal must not fuse into InvokeLocal"
    );
    assert!(
        chunk.code.iter().any(|op| matches!(op, Bytecode::GetLocalChecked(..))),
        "checked getlocal must be preserved"
    );
}
