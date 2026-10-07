//! Unicode-scalar `Char` execution and conformance tests.

use super::vm_support::{compile_inline, run_inline};
use phalcom_core::bytecode::Bytecode;
use phalcom_core::value::Value;

fn slot(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    vm.heap.module(module).get(vm.interner.find(name)?)
}

#[test]
fn unicode_version_guard() {
    let (major, minor, patch) = (
        std::char::UNICODE_VERSION.0,
        std::char::UNICODE_VERSION.1,
        std::char::UNICODE_VERSION.2,
    );
    assert_eq!((major, minor, patch), (17, 0, 0));
}

#[test]
fn char_properties_and_methods() {
    let source = r#"
let a = 'a'
let a_code_point = a.codePoint
let a_utf8_len = a.utf8Length
let a_is_ascii = a.isAscii
let a_is_digit = a.isAsciiDigit
let a_is_alpha = a.isAsciiAlphabetic
let a_is_lower = a.isAsciiLowercase
let a_is_upper = a.isAsciiUppercase
let a_is_alnum = a.isAsciiAlphanumeric
let a_is_hex = a.isAsciiHexDigit
let a_is_ws = a.isWhitespace
let a_is_ctrl = a.isControl
let a_str = a.toString

let e_acute = 'é'
let e_utf8_len = e_acute.utf8Length
let e_str = e_acute.toString

let cjk = '中'
let cjk_utf8_len = cjk.utf8Length

let emoji = '🙂'
let emoji_utf8_len = emoji.utf8Length
let emoji_cp = emoji.codePoint

let ws = ' '
let ws_is_ws = ws.isWhitespace

let nl = '\n'
let nl_is_ctrl = nl.isControl
"#;
    let (vm, module) = run_inline(source).expect("char test should run");
    assert_eq!(slot(&vm, module, "a_code_point"), Some(Value::int(97)));
    assert_eq!(slot(&vm, module, "a_utf8_len"), Some(Value::int(1)));
    assert_eq!(slot(&vm, module, "a_is_ascii"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "a_is_digit"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "a_is_alpha"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "a_is_lower"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "a_is_upper"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "a_is_alnum"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "a_is_hex"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "a_is_ws"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "a_is_ctrl"), Some(Value::bool(false)));

    assert_eq!(slot(&vm, module, "e_utf8_len"), Some(Value::int(2)));
    assert_eq!(slot(&vm, module, "cjk_utf8_len"), Some(Value::int(3)));
    assert_eq!(slot(&vm, module, "emoji_utf8_len"), Some(Value::int(4)));
    assert_eq!(slot(&vm, module, "emoji_cp"), Some(Value::int(0x1F642)));

    assert_eq!(slot(&vm, module, "ws_is_ws"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "nl_is_ctrl"), Some(Value::bool(true)));
}

#[test]
fn char_from_code_point() {
    let source = r#"
let c0 = Char.fromCodePoint(0)
let c97 = Char.fromCodePoint(97)
let cd7ff = Char.fromCodePoint(0xD7FF)
let ce000 = Char.fromCodePoint(0xE000)
let c10ffff = Char.fromCodePoint(0x10FFFF)
let c_neg = Char.fromCodePoint(-1)
let c_d800 = Char.fromCodePoint(0xD800)
let c_dfff = Char.fromCodePoint(0xDFFF)
let c_110000 = Char.fromCodePoint(0x110000)
"#;
    let (vm, module) = run_inline(source).expect("char from code point should run");
    assert_eq!(slot(&vm, module, "c0"), Some(Value::char('\0').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "c97"), Some(Value::char('a').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "cd7ff"), Some(Value::char('\u{D7FF}').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "ce000"), Some(Value::char('\u{E000}').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "c10ffff"), Some(Value::char('\u{10FFFF}').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "c_neg"), Some(Value::none()));
    assert_eq!(slot(&vm, module, "c_d800"), Some(Value::none()));
    assert_eq!(slot(&vm, module, "c_dfff"), Some(Value::none()));
    assert_eq!(slot(&vm, module, "c_110000"), Some(Value::none()));
}

#[test]
fn char_equality_and_identity() {
    let source = r#"
let eq_same = 'a' == 'a'
let id_same = 'a' === 'a'
let ne_diff = not ('a' === 'b')
let neq_diff = 'a' != 'b'
"#;
    let (vm, module) = run_inline(source).expect("char equality should run");
    assert_eq!(slot(&vm, module, "eq_same"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "id_same"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "ne_diff"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "neq_diff"), Some(Value::bool(true)));
}

#[test]
fn char_pattern_matching_execution() {
    let source = r#"
let c = 'b'
let result = match c {
    'a' => 1
    'b' => 2
    _ => 3
}
let if_let_res = if let 'b' = c { 42 } else { 0 }
"#;
    let (vm, module) = run_inline(source).expect("char match should run");
    assert_eq!(slot(&vm, module, "result"), Some(Value::int(2)));
    assert_eq!(slot(&vm, module, "if_let_res"), Some(Value::int(42)));
}

#[test]
fn char_pattern_compilation_uses_same_not_equals_send() {
    let source = "let c = 'a'\nlet result = match c { 'a' => 1 _ => 0 }\n";
    let (vm, _program, closure) = compile_inline(source).expect("compile");
    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(chunk.code.iter().any(|op| matches!(op, Bytecode::Same)));
}
