//! Unicode-scalar `Char` execution and conformance tests.

use super::vm_support::{compile_inline, run_inline};
use phalcom_core::bytecode::Bytecode;
use phalcom_core::value::Value;

fn slot(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    vm.heap.module(module).get(vm.interner.find(name)?)
}

#[test]
fn unicode_version_guard() {
    let (major, minor, patch) = (std::char::UNICODE_VERSION.0, std::char::UNICODE_VERSION.1, std::char::UNICODE_VERSION.2);
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

#[test]
fn string_raw_byte_at_and_raw_slice_contracts() {
    let source = r#"
let s = "Aé"
let b0 = s.bytes.at(0)
let b1 = s.bytes.at(1)
let b2 = s.bytes.at(2)
let b_neg = s.bytes.at(-1)
let b_out = s.bytes.at(3)
let b_large = s.bytes.at(999999999999999999)

let slice_valid = s.slice(0, 1)
let slice_full = s.slice(0, 3)
"#;
    let (vm, module) = run_inline(source).expect("string raw byte should run");
    assert_eq!(slot(&vm, module, "b0"), Some(Value::int(65).wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "b1"), Some(Value::int(195).wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "b2"), Some(Value::int(169).wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "b_neg"), Some(Value::none()));
    assert_eq!(slot(&vm, module, "b_out"), Some(Value::none()));
    assert_eq!(slot(&vm, module, "b_large"), Some(Value::none()));

    let s1 = slot(&vm, module, "slice_valid").unwrap();
    assert_eq!(s1.to_string(&vm), "A");
    let s2 = slot(&vm, module, "slice_full").unwrap();
    assert_eq!(s2.to_string(&vm), "Aé");
}

#[test]
fn string_is_char_boundary_and_decode_char_at() {
    let source = r#"
let s = "Aé中🙂"
let b0 = s.isCharBoundary(0)
let b1 = s.isCharBoundary(1)
let b2 = s.isCharBoundary(2)
let b3 = s.isCharBoundary(3)
let b4 = s.isCharBoundary(4)
let b5 = s.isCharBoundary(5)
let b6 = s.isCharBoundary(6)
let b7 = s.isCharBoundary(7)
let b10 = s.isCharBoundary(10)
let b_neg = s.isCharBoundary(-1)
let b_out = s.isCharBoundary(11)

let d0 = s.decodeCharAt(0)
let d1 = s.decodeCharAt(1)
let d2 = s.decodeCharAt(2)
let d3 = s.decodeCharAt(3)
let d6 = s.decodeCharAt(6)
let d10 = s.decodeCharAt(10)
let d_neg = s.decodeCharAt(-1)
"#;
    let (vm, module) = run_inline(source).expect("char boundary test should run");
    assert_eq!(slot(&vm, module, "b0"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "b1"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "b2"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "b3"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "b4"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "b5"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "b6"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "b7"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "b10"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "b_neg"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "b_out"), Some(Value::bool(false)));

    assert_eq!(slot(&vm, module, "d0"), Some(Value::char('A').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "d1"), Some(Value::char('é').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "d2"), Some(Value::none()));
    assert_eq!(slot(&vm, module, "d3"), Some(Value::char('中').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "d6"), Some(Value::char('🙂').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "d10"), Some(Value::none()));
    assert_eq!(slot(&vm, module, "d_neg"), Some(Value::none()));
}

#[test]
fn string_chars_iteration_and_codepoint_sequence() {
    let source = r#"
let s = "Aé中🙂"
let chars_view = s.chars
let mut it = chars_view.iterator
let c0 = it.next
let c1 = it.next
let c2 = it.next
let c3 = it.next
let c4 = it.next

let cps = s.codePoints
let cp_size = cps.size
let cp0 = cps.at(0)
let cp1 = cps.at(1)
let cp2 = cps.at(2)
let cp3 = cps.at(3)
let cp4 = cps.at(4)
let cp_neg = cps.at(-1)
"#;
    let (vm, module) = run_inline(source).expect("chars iteration should run");
    assert_eq!(slot(&vm, module, "c0"), Some(Value::char('A').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "c1"), Some(Value::char('é').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "c2"), Some(Value::char('中').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "c3"), Some(Value::char('🙂').wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "c4"), Some(Value::none()));

    assert_eq!(slot(&vm, module, "cp_size"), Some(Value::int(4)));
    assert_eq!(slot(&vm, module, "cp0"), Some(Value::int(65).wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "cp1"), Some(Value::int(233).wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "cp2"), Some(Value::int(20013).wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "cp3"), Some(Value::int(128578).wrap_some().unwrap()));
    assert_eq!(slot(&vm, module, "cp4"), Some(Value::none()));
    assert_eq!(slot(&vm, module, "cp_neg"), Some(Value::none()));
}

#[test]
fn string_hygiene_methods() {
    let source = r#"
let s = "  hello world  "
let trimmed = s.trim()
let trimmed_start = s.trimStart()
let trimmed_end = s.trimEnd()

let custom = "__abc__".trim("_")
let custom_start = "__abc__".trimStart("_")
let custom_end = "__abc__".trimEnd("_")

let has_world = s.contains("world")
let has_foo = s.contains("foo")
let idx = s.indexOf("world")
let replaced = "foo bar foo".replace("foo", "baz")
let parts = "a,b,c".split(",")
let part_count = parts.size
let p0 = parts[0]
let p1 = parts[1]
let p2 = parts[2]
"#;
    let (vm, module) = run_inline(source).expect("string hygiene methods should run");
    assert_eq!(slot(&vm, module, "trimmed").unwrap().to_string(&vm), "hello world");
    assert_eq!(slot(&vm, module, "trimmed_start").unwrap().to_string(&vm), "hello world  ");
    assert_eq!(slot(&vm, module, "trimmed_end").unwrap().to_string(&vm), "  hello world");

    assert_eq!(slot(&vm, module, "custom").unwrap().to_string(&vm), "abc");
    assert_eq!(slot(&vm, module, "custom_start").unwrap().to_string(&vm), "abc__");
    assert_eq!(slot(&vm, module, "custom_end").unwrap().to_string(&vm), "__abc");

    assert_eq!(slot(&vm, module, "has_world"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "has_foo"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "idx"), Some(Value::int(8)));
    assert_eq!(slot(&vm, module, "replaced").unwrap().to_string(&vm), "baz bar baz");

    assert_eq!(slot(&vm, module, "part_count"), Some(Value::int(3)));
    assert_eq!(slot(&vm, module, "p0").unwrap().to_string(&vm), "a");
    assert_eq!(slot(&vm, module, "p1").unwrap().to_string(&vm), "b");
    assert_eq!(slot(&vm, module, "p2").unwrap().to_string(&vm), "c");
}

#[test]
fn string_slice_boundary_errors() {
    let source = r#"
let s = "Aé"
s.slice(1, 2)
"#;
    let err = match run_inline(source) {
        Ok(_) => panic!("misaligned slice should fail"),
        Err(err) => err,
    };
    assert!(format!("{err:?}").contains("char boundary"));
}
