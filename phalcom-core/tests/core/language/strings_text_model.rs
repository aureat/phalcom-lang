//! Conformance and integration tests for the Telos text model.

use super::vm_support::run_inline;
use phalcom_core::value::Value;

fn slot(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    vm.heap.module(module).get(vm.interner.find(name)?)
}

#[test]
fn string_index_and_counts() {
    let source = r#"
let hello = "hello"
let byteCount = hello.byteCount
let charCount = hello.charCount
let isAscii = hello.isAscii

let startIdx = hello.startIndex
let endIdx = hello.endIndex
let firstChar = hello[startIdx]

let nextIdx = hello.index(after: startIdx)
let secondChar = match nextIdx {
  Option::Some(idx) => hello[idx]
  _ => '?'
}

let prevIdx = match nextIdx {
  Option::Some(idx) => hello.index(before: idx)
  _ => None
}

let lower = "HELLO World".lowercase()
let upper = "hello world".uppercase()
let repeated = "ab".repeat(3)
let paddedStart = "42".padStart(5, with: '0')
let paddedEnd = "42".padEnd(5, with: '.')
let reprStr = "hello\nworld".repr
let reprChar = '\n'.repr
let charLower = 'A'.simpleLowercase
let charUpper = 'z'.simpleUppercase
"#;

    let (vm, module) = run_inline(source).expect("string_index_and_counts executes");

    assert_eq!(slot(&vm, module, "byteCount").and_then(|v| v.as_int()), Some(5));
    assert_eq!(slot(&vm, module, "charCount").and_then(|v| v.as_int()), Some(5));
    assert_eq!(slot(&vm, module, "isAscii").and_then(|v| v.as_bool()), Some(true));

    assert_eq!(slot(&vm, module, "firstChar").and_then(|v| v.as_char()), Some('h'));
    assert_eq!(slot(&vm, module, "secondChar").and_then(|v| v.as_char()), Some('e'));

    assert_eq!(slot(&vm, module, "lower").map(|v| v.to_string(&vm)), Some("hello world".to_string()));
    assert_eq!(slot(&vm, module, "upper").map(|v| v.to_string(&vm)), Some("HELLO WORLD".to_string()));
    assert_eq!(slot(&vm, module, "repeated").map(|v| v.to_string(&vm)), Some("ababab".to_string()));
    assert_eq!(slot(&vm, module, "paddedStart").map(|v| v.to_string(&vm)), Some("00042".to_string()));
    assert_eq!(slot(&vm, module, "paddedEnd").map(|v| v.to_string(&vm)), Some("42...".to_string()));
    assert_eq!(slot(&vm, module, "reprStr").map(|v| v.to_string(&vm)), Some("\"hello\\nworld\"".to_string()));
    assert_eq!(slot(&vm, module, "reprChar").map(|v| v.to_string(&vm)), Some("'\\n'".to_string()));
    assert_eq!(slot(&vm, module, "charLower").and_then(|v| v.as_char()), Some('a'));
    assert_eq!(slot(&vm, module, "charUpper").and_then(|v| v.as_char()), Some('Z'));
}
