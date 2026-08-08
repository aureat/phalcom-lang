use phalcom_core::compiler::lib::CompilerError;
use phalcom_core::error::PhError;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn comma_separated(prefix: &str, count: usize) -> String {
    (0..count).map(|i| format!("{prefix}{i}")).collect::<Vec<_>>().join(", ")
}

fn positional_arguments(count: usize) -> String {
    comma_separated("", count)
}

fn parameters(count: usize) -> String {
    (0..count).map(|i| format!("_ arg{i}")).collect::<Vec<_>>().join(", ")
}

fn placeholders(count: usize) -> String {
    vec!["_"; count].join(", ")
}

fn fields(count: usize) -> String {
    (0..count).map(|i| format!("  _field{i}\n")).collect()
}

fn compile_error(source: &str) -> CompilerError {
    let mut vm = VM::new();
    let module = vm.create_module("main", "send_arity_limit");
    match vm.compile_closure(module, source) {
        Err(PhError::Compile(error)) => error,
        Err(other) => panic!("expected a compile error, got {other:?}"),
        Ok(_) => panic!("source should not compile"),
    }
}

fn assert_arity_limit(source: &str, subject: &'static str, found: usize) {
    match compile_error(source) {
        CompilerError::ArityLimit {
            subject: actual_subject,
            found: actual_found,
            limit,
            span,
        } => {
            assert_eq!(actual_subject, subject);
            assert_eq!(actual_found, found);
            assert_eq!(limit, u8::MAX);
            assert!(span.start < span.end, "the diagnostic must retain the offending syntax span");
        }
        other => panic!("expected ArityLimit, got {other:?}"),
    }
}

fn strip_ansi(s: &str) -> String {
    let mut output = String::new();
    let mut in_escape = false;
    for character in s.chars() {
        if character == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if character.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            output.push(character);
        }
    }
    output
}

#[test]
fn ordinary_send_with_255_arguments_compiles_and_executes() {
    let params = parameters(255);
    let args = positional_arguments(255);
    let source = format!("class Sink {{\n@constructor\nnew() {{}}\n  accept({params}) {{ return 7 }}\n}}\nlet result = Sink.new().accept({args})\n");
    let mut vm = VM::new();
    let module = vm.create_module("main", "ordinary_send_255");
    let closure = vm.compile_closure(module, &source).expect("255 arguments must compile");
    vm.run_in_module(module, closure).expect("255-argument send must execute");

    let result = vm.get_or_intern("result");
    let module_object = vm.heap.module(module);
    let slot = module_object.slot_of(result).expect("result binding must exist");
    assert_eq!(module_object.globals[slot], Value::Int(7));
}

#[test]
fn ordinary_send_with_256_arguments_is_compile_error() {
    assert_arity_limit(&format!("target.send({})", positional_arguments(256)), "message send", 256);
}

#[test]
fn cli_reports_oversized_send_as_a_single_compile_diagnostic() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("phalcom_send_arity_limit_{}_{}.ph", std::process::id(), unique));
    fs::write(&path, format!("target.send({})\n", positional_arguments(256))).expect("fixture source must be written");

    let output = Command::new(env!("CARGO_BIN_EXE_phalcom"))
        .arg(&path)
        .env_remove("RUST_LOG")
        .env_remove("RUST_LOG_STYLE")
        .output()
        .expect("phalcom binary must run");
    fs::remove_file(&path).expect("fixture source must be removed");

    assert_eq!(output.status.code(), Some(65));
    let stderr = strip_ansi(&String::from_utf8_lossy(&output.stderr));
    assert_eq!(stderr.trim(), "error: message send has 256 arguments; bytecode supports at most 255");
    assert!(!stderr.contains("Traceback"));
}

#[test]
fn super_send_with_256_arguments_is_compile_error() {
    assert_arity_limit(
        &format!("class Child {{\n  call() {{ super.send({}) }}\n}}\n", positional_arguments(256)),
        "super send",
        256,
    );
}

#[test]
fn subscript_reads_accept_255_and_reject_256_arguments() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "subscript_read_255");
    vm.compile_closure(module, &format!("target[{}]", positional_arguments(255)))
        .expect("255-argument subscript read must compile");

    assert_arity_limit(&format!("target[{}]", positional_arguments(256)), "subscript read", 256);
}

#[test]
fn subscript_writes_accept_254_explicit_and_reject_255_explicit_arguments() {
    let mut vm = VM::new();
    let module = vm.create_module("main", "subscript_write_255");
    vm.compile_closure(module, &format!("target[{}] = 0", positional_arguments(254)))
        .expect("254 explicit subscript arguments plus put: must compile");

    assert_arity_limit(&format!("target[{}] = 0", positional_arguments(255)), "subscript write", 256);
}

#[test]
fn pinned_selector_references_with_256_arguments_are_compile_errors() {
    assert_arity_limit(&format!("target::#send({})", placeholders(256)), "pinned selector", 256);
    assert_arity_limit(&format!("#send({})", placeholders(256)), "pinned selector", 256);
}

#[test]
fn declarations_with_256_parameters_are_compile_errors() {
    let params = parameters(256);
    assert_arity_limit(&format!("class C {{\n  method({params}) {{}}\n}}\n"), "method declaration", 256);
    assert_arity_limit(&format!("class C {{\n@constructor\nnew({params}) {{}}\n}}\n"), "constructor declaration", 256);
    assert_arity_limit(&format!("class C {{\n  [{params}] {{}}\n}}\n"), "subscript declaration", 256);
}

#[test]
fn generated_constructor_with_256_fields_is_a_compile_error() {
    assert_arity_limit(&format!("@construct\nclass C {{\n{}}}\n", fields(256)), "constructor declaration", 256);
}
