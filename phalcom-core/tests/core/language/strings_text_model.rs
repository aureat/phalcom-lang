//! Conformance and integration tests for the Telos text model.

use super::vm_support::run_inline;
use phalcom_core::value::Value;

fn slot(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    vm.heap.module(module).get(vm.interner.find(name)?)
}

fn run_standalone(source: &str) -> Result<(phalcom_core::vm::VM, phalcom_core::heap::ObjRef), Box<dyn std::error::Error>> {
    let mut session = phalcom_core::modules::compile_session::ProgramCompilationSession::standalone()?;
    let program = session.compile_fixture(std::path::Path::new("text_test.ph"), source)?;
    let mut vm = super::vm_support::universe_vm();
    vm.run_compiled(&program)?;
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    Ok((vm, module))
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

#[test]
fn substring_slicing() {
    let source = r#"
let text = "hello world"
let sub = text.slice(0, 5)
let subByteCount = sub.byteCount
let subCharCount = sub.charCount
let subStr = sub.toString
let subFirst = sub.first
let subLast = sub.last
let subEq = sub == "hello"
let strEq = "hello" == sub

let subSlice = sub.slice(1, 4)
let subSliceStr = subSlice.toString

let rangeSub = text[6..11]
let rangeSubStr = rangeSub.toString
"#;

    let (vm, module) = run_inline(source).expect("substring_slicing executes");

    assert_eq!(slot(&vm, module, "subByteCount").and_then(|v| v.as_int()), Some(5));
    assert_eq!(slot(&vm, module, "subCharCount").and_then(|v| v.as_int()), Some(5));
    assert_eq!(slot(&vm, module, "subStr").map(|v| v.to_string(&vm)), Some("hello".to_string()));
    assert_eq!(slot(&vm, module, "subFirst").and_then(|v| v.unwrap_some()).and_then(|v| v.as_char()), Some('h'));
    assert_eq!(slot(&vm, module, "subLast").and_then(|v| v.unwrap_some()).and_then(|v| v.as_char()), Some('o'));
    assert_eq!(slot(&vm, module, "subEq").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "strEq").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "subSliceStr").map(|v| v.to_string(&vm)), Some("ell".to_string()));
    assert_eq!(slot(&vm, module, "rangeSubStr").map(|v| v.to_string(&vm)), Some("world".to_string()));
}

#[test]
fn text_pattern_matching() {
    let source = r#"
from universe.text import Text, TextPattern, Substring

class Measure {
  @class
  count<T>(_ t: T) -> Int where T |= Text {
    t.byteCount
  }

  @class
  asSub<T>(_ t: T) -> Substring where T |= Text {
    t.asSubstring
  }
}

let str = "hello world"
let sub = str.slice(0, 5)

let strLen = Measure.count(str)
let subLen = Measure.count(sub)
let strSubStr = Measure.asSub(str).toString

let matchChar = 'o'.findIn(str.asSubstring, 0)
let matchCharSecond = 'o'.findIn(str.asSubstring, 5)
let matchStr = "world".findIn(str.asSubstring, 0)
let matchSub = str.slice(6, 11).findIn(str.asSubstring, 0)
let matchNotFound = "world".findIn(sub, 0)
"#;

    let (vm, module) = run_standalone(source).expect("text_pattern_matching executes");

    assert_eq!(slot(&vm, module, "strLen").and_then(|v| v.as_int()), Some(11));
    assert_eq!(slot(&vm, module, "subLen").and_then(|v| v.as_int()), Some(5));
    assert_eq!(slot(&vm, module, "strSubStr").map(|v| v.to_string(&vm)), Some("hello world".to_string()));

    let mc = slot(&vm, module, "matchChar").and_then(|v| v.unwrap_some()).expect("found char match");
    assert_eq!(mc.to_string(&vm), "(4, 5)");

    let mc2 = slot(&vm, module, "matchCharSecond").and_then(|v| v.unwrap_some()).expect("found second char match");
    assert_eq!(mc2.to_string(&vm), "(7, 8)");

    let ms = slot(&vm, module, "matchStr").and_then(|v| v.unwrap_some()).expect("found str match");
    assert_eq!(ms.to_string(&vm), "(6, 11)");

    let msub = slot(&vm, module, "matchSub").and_then(|v| v.unwrap_some()).expect("found sub match");
    assert_eq!(msub.to_string(&vm), "(6, 11)");

    let mnf = slot(&vm, module, "matchNotFound").expect("matchNotFound slot");
    assert!(mnf.is_none());
}

#[test]
fn search_trim_strip() {
    let source = r#"
from universe.text import Substring
from universe.scalar import StringIndex

let text = "  --hello world.ph  "
let trimmed = text.trimmed()
let trimmedStr = trimmed.toString
let trimmedStartStr = text.trimmedStart().toString
let trimmedEndStr = text.trimmedEnd().toString

let strippedPre = trimmed.stripPrefix("--")
let strippedPreStr = match strippedPre {
  Option::Some(s) => s.toString
  _ => ""
}

let strippedSuf = strippedPre.unwrap.stripSuffix(".ph")
let strippedSufStr = match strippedSuf {
  Option::Some(s) => s.toString
  _ => ""
}

let strippedNone = text.stripPrefix("xyz")

let containsChar = text.contains('w')
let containsStr = text.contains("world")
let containsSub = text.contains(strippedSuf.unwrap)
let containsNone = text.contains("missing")

let startsWithTrue = text.startsWith("  --")
let startsWithFalse = text.startsWith("world")
let endsWithTrue = text.endsWith(".ph  ")
let endsWithFalse = text.endsWith("world")

let findIdx = text.find("world")
let findOffset = match findIdx {
  Option::Some(idx) => idx.offset
  _ => -1
}

let findLastIdx = text.findLast(' ')
let findLastOffset = match findLastIdx {
  Option::Some(idx) => idx.offset
  _ => -1
}

let rangeOpt = text.findRange("world")
let rangeStr = match rangeOpt {
  Option::Some(r) => text[r].toString
  _ => ""
}
"#;

    let (vm, module) = run_standalone(source).expect("search_trim_strip executes");

    assert_eq!(slot(&vm, module, "trimmedStr").map(|v| v.to_string(&vm)), Some("--hello world.ph".to_string()));
    assert_eq!(slot(&vm, module, "trimmedStartStr").map(|v| v.to_string(&vm)), Some("--hello world.ph  ".to_string()));
    assert_eq!(slot(&vm, module, "trimmedEndStr").map(|v| v.to_string(&vm)), Some("  --hello world.ph".to_string()));
    assert_eq!(slot(&vm, module, "strippedPreStr").map(|v| v.to_string(&vm)), Some("hello world.ph".to_string()));
    assert_eq!(slot(&vm, module, "strippedSufStr").map(|v| v.to_string(&vm)), Some("hello world".to_string()));
    assert!(slot(&vm, module, "strippedNone").expect("strippedNone slot").is_none());

    assert_eq!(slot(&vm, module, "containsChar").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "containsStr").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "containsSub").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "containsNone").and_then(|v| v.as_bool()), Some(false));

    assert_eq!(slot(&vm, module, "startsWithTrue").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "startsWithFalse").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(slot(&vm, module, "endsWithTrue").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "endsWithFalse").and_then(|v| v.as_bool()), Some(false));

    assert_eq!(slot(&vm, module, "findOffset").and_then(|v| v.as_int()), Some(10));
    assert_eq!(slot(&vm, module, "findLastOffset").and_then(|v| v.as_int()), Some(19));
    assert_eq!(slot(&vm, module, "rangeStr").map(|v| v.to_string(&vm)), Some("world".to_string()));
}

#[test]
fn lazy_splitting_and_lines() {
    let source = r#"
from universe.text import Substring, Split, SplitWhitespace, Lines

let csv = "a,,b,c"
let mut parts = []
for p in csv.split(',') {
  parts.append(p.toString)
}

let kv = "key=value=extra"
let kvSplit = kv.splitOnce('=')
let kStr = if kvSplit.isSome { kvSplit.unwrap.first.unwrap.toString } else { "" }
let vStr = if kvSplit.isSome { kvSplit.unwrap.last.unwrap.toString } else { "" }

let ws = "  foo   bar \n baz  "
let mut words = []
for w in ws.splitWhitespace() {
  words.append(w.toString)
}

let multiline = "first\r\nsecond\nthird\rlast"
let mut lineList = []
for l in multiline.lines() {
  lineList.append(l.toString)
}

let mut linesWithTerms = []
for l in multiline.lines(keepingTerminators: true) {
  linesWithTerms.append(l.toString)
}

let replaced1 = "foo bar foo".replaceFirst("foo", with: "baz")
let replacedAll = "foo bar foo".replaceAll("foo", with: "baz")
"#;

    let (vm, module) = run_standalone(source).expect("lazy_splitting_and_lines executes");

    let parts_val = slot(&vm, module, "parts").expect("parts slot");
    let parts_list = vm.heap.list(parts_val.as_obj().expect("parts list"));
    assert_eq!(parts_list.len(), 4);
    assert_eq!(parts_list.get(0).unwrap().to_string(&vm), "a");
    assert_eq!(parts_list.get(1).unwrap().to_string(&vm), "");
    assert_eq!(parts_list.get(2).unwrap().to_string(&vm), "b");
    assert_eq!(parts_list.get(3).unwrap().to_string(&vm), "c");

    assert_eq!(slot(&vm, module, "kStr").map(|v| v.to_string(&vm)), Some("key".to_string()));
    assert_eq!(slot(&vm, module, "vStr").map(|v| v.to_string(&vm)), Some("value=extra".to_string()));

    let words_val = slot(&vm, module, "words").expect("words slot");
    let words_list = vm.heap.list(words_val.as_obj().expect("words list"));
    assert_eq!(words_list.len(), 3);
    assert_eq!(words_list.get(0).unwrap().to_string(&vm), "foo");
    assert_eq!(words_list.get(1).unwrap().to_string(&vm), "bar");
    assert_eq!(words_list.get(2).unwrap().to_string(&vm), "baz");

    let lines_val = slot(&vm, module, "lineList").expect("lineList slot");
    let lines_list = vm.heap.list(lines_val.as_obj().expect("lines list"));
    assert_eq!(lines_list.len(), 4);
    assert_eq!(lines_list.get(0).unwrap().to_string(&vm), "first");
    assert_eq!(lines_list.get(1).unwrap().to_string(&vm), "second");
    assert_eq!(lines_list.get(2).unwrap().to_string(&vm), "third");
    assert_eq!(lines_list.get(3).unwrap().to_string(&vm), "last");

    let terms_val = slot(&vm, module, "linesWithTerms").expect("linesWithTerms slot");
    let terms_list = vm.heap.list(terms_val.as_obj().expect("linesWithTerms list"));
    assert_eq!(terms_list.len(), 4);
    assert_eq!(terms_list.get(0).unwrap().to_string(&vm), "first\r\n");
    assert_eq!(terms_list.get(1).unwrap().to_string(&vm), "second\n");
    assert_eq!(terms_list.get(2).unwrap().to_string(&vm), "third\r");
    assert_eq!(terms_list.get(3).unwrap().to_string(&vm), "last");

    assert_eq!(slot(&vm, module, "replaced1").map(|v| v.to_string(&vm)), Some("baz bar foo".to_string()));
    assert_eq!(slot(&vm, module, "replacedAll").map(|v| v.to_string(&vm)), Some("baz bar baz".to_string()));
}

#[test]
fn string_builder_and_writer() {
    let source = r#"
from universe.text import StringBuilder, TextWriter, Substring

let sb = StringBuilder.new()
let emptyBefore = sb.isEmpty
let countBefore = sb.byteCount

sb.append("hello")
sb.append(' ')
let worldStr = "world and more"
let sub = worldStr.slice(0, 5)
sb.append(sub)
sb.appendLine("!")

let countAfter = sb.byteCount
let emptyAfter = sb.isEmpty
let built = sb.toString

class StreamLogger {
  @class
  log<W>(_ writer: W, _ msg: String) where W |= TextWriter {
    writer.write(msg)
  }
}

let writerSb = StringBuilder.new()
StreamLogger.log(writerSb, "streaming message")
let writerResult = writerSb.toString

let joined = ["apple", "banana", "cherry"].join(", ")

sb.clear()
let clearedCount = sb.byteCount
let clearedEmpty = sb.isEmpty
let clearedStr = sb.toString
"#;

    let (vm, module) = run_standalone(source).expect("string_builder_and_writer executes");

    assert_eq!(slot(&vm, module, "emptyBefore").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "countBefore").and_then(|v| v.as_int()), Some(0));
    assert_eq!(slot(&vm, module, "emptyAfter").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(slot(&vm, module, "countAfter").and_then(|v| v.as_int()), Some(13));
    assert_eq!(slot(&vm, module, "built").map(|v| v.to_string(&vm)), Some("hello world!\n".to_string()));

    assert_eq!(slot(&vm, module, "writerResult").map(|v| v.to_string(&vm)), Some("streaming message".to_string()));
    assert_eq!(slot(&vm, module, "joined").map(|v| v.to_string(&vm)), Some("apple, banana, cherry".to_string()));

    assert_eq!(slot(&vm, module, "clearedCount").and_then(|v| v.as_int()), Some(0));
    assert_eq!(slot(&vm, module, "clearedEmpty").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "clearedStr").map(|v| v.to_string(&vm)), Some("".to_string()));
}

#[test]
fn text_cursor_and_line_map() {
    let source = r#"
from universe.text import TextCursor, LineMap, SourceLocation, Substring
from universe.scalar import StringIndex

let input = "let x = 42"
let cursor = TextCursor.new(input)
let peekBefore = cursor.peek()
let isAtEndBefore = cursor.isAtEnd

let consumedLet = cursor.consume("let ")
let peekAfterLet = cursor.peek()
let remAfterLet = cursor.remaining.toString

let cp = cursor.checkpoint()
let consumedWrong = cursor.consume("wrong")
cursor.restore(cp)

let consumedX = cursor.consume("x")
let nextChar = cursor.next()
let remEnd = cursor.remaining.toString

let sourceText = "hello\nworld\r\nphalcom"
let lineMap = LineMap.new(sourceText)

let loc1 = lineMap.location(of: StringIndex(0))
let loc2 = lineMap.location(of: StringIndex(6))
let loc3 = lineMap.location(of: StringIndex(13))
let loc1Str = loc1.toString
let loc2Str = loc2.toString
let loc3Str = loc3.toString
let loc1Line = loc1.line
let loc1Col = loc1.column
"#;

    let (vm, module) = run_standalone(source).expect("text_cursor_and_line_map executes");

    assert_eq!(slot(&vm, module, "peekBefore").and_then(|v| v.unwrap_some()).and_then(|v| v.as_char()), Some('l'));
    assert_eq!(slot(&vm, module, "isAtEndBefore").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(slot(&vm, module, "consumedLet").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "peekAfterLet").and_then(|v| v.unwrap_some()).and_then(|v| v.as_char()), Some('x'));
    assert_eq!(slot(&vm, module, "remAfterLet").map(|v| v.to_string(&vm)), Some("x = 42".to_string()));
    assert_eq!(slot(&vm, module, "consumedWrong").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(slot(&vm, module, "consumedX").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "nextChar").and_then(|v| v.unwrap_some()).and_then(|v| v.as_char()), Some(' '));
    assert_eq!(slot(&vm, module, "remEnd").map(|v| v.to_string(&vm)), Some("= 42".to_string()));

    assert_eq!(slot(&vm, module, "loc1Line").and_then(|v| v.as_int()), Some(1));
    assert_eq!(slot(&vm, module, "loc1Col").and_then(|v| v.as_int()), Some(1));
    assert_eq!(slot(&vm, module, "loc1Str").map(|v| v.to_string(&vm)), Some("1:1".to_string()));
    assert_eq!(slot(&vm, module, "loc2Str").map(|v| v.to_string(&vm)), Some("2:1".to_string()));
    assert_eq!(slot(&vm, module, "loc3Str").map(|v| v.to_string(&vm)), Some("3:1".to_string()));
}

#[test]
fn parsing_and_formatting() {
    let source = r#"
from universe.text import Substring, Utf8Error, IntParseError, FloatParseError, BoolParseError

let validBytes = Bytes.fromString("hello")
let validStrRes = String.fromUtf8(validBytes)
let validStr = validStrRes.unwrap

let invalidBytes = Bytes.new(2)
invalidBytes[0] = 255
invalidBytes[1] = 255
let invalidRes = String.fromUtf8(invalidBytes)
let isInvalidErr = invalidRes.isErr
let lossyStr = String.fromUtf8Lossy(invalidBytes)

let parsedInt = Int.parse("123").unwrap
let parsedHex = Int.parse("ff", radix: 16).unwrap
let parsedBin = Int.parse("1101", radix: 2).unwrap
let invalidInt = Int.parse("not_a_number")
let isInvalidIntErr = invalidInt.isErr

let parsedFloat = Float.parse("1.25").unwrap
let invalidFloat = Float.parse("abc")
let isInvalidFloatErr = invalidFloat.isErr

let parsedTrue = Bool.parse("true").unwrap
let parsedFalse = Bool.parse("false").unwrap
let invalidBool = Bool.parse("yes")
let isInvalidBoolErr = invalidBool.isErr

let hexFormatted = 255.toString(radix: 16)
let binFormatted = 255.toString(radix: 2)

let charLower = 'A'.simpleLowercase
let charUpper = 'z'.simpleUppercase
let charRepr = '\n'.repr
"#;

    let (vm, module) = run_standalone(source).expect("parsing_and_formatting executes");

    assert_eq!(slot(&vm, module, "validStr").map(|v| v.to_string(&vm)), Some("hello".to_string()));
    assert_eq!(slot(&vm, module, "isInvalidErr").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "lossyStr").map(|v| v.to_string(&vm)), Some("\u{FFFD}\u{FFFD}".to_string()));

    assert_eq!(slot(&vm, module, "parsedInt").and_then(|v| v.as_int()), Some(123));
    assert_eq!(slot(&vm, module, "parsedHex").and_then(|v| v.as_int()), Some(255));
    assert_eq!(slot(&vm, module, "parsedBin").and_then(|v| v.as_int()), Some(13));
    assert_eq!(slot(&vm, module, "isInvalidIntErr").and_then(|v| v.as_bool()), Some(true));

    assert_eq!(slot(&vm, module, "parsedFloat").and_then(|v| v.as_float()), Some(1.25));
    assert_eq!(slot(&vm, module, "isInvalidFloatErr").and_then(|v| v.as_bool()), Some(true));

    assert_eq!(slot(&vm, module, "parsedTrue").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(slot(&vm, module, "parsedFalse").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(slot(&vm, module, "isInvalidBoolErr").and_then(|v| v.as_bool()), Some(true));

    assert_eq!(slot(&vm, module, "hexFormatted").map(|v| v.to_string(&vm)), Some("ff".to_string()));
    assert_eq!(slot(&vm, module, "binFormatted").map(|v| v.to_string(&vm)), Some("11111111".to_string()));

    assert_eq!(slot(&vm, module, "charLower").and_then(|v| v.as_char()), Some('a'));
    assert_eq!(slot(&vm, module, "charUpper").and_then(|v| v.as_char()), Some('Z'));
    assert_eq!(slot(&vm, module, "charRepr").map(|v| v.to_string(&vm)), Some("'\\n'".to_string()));
}






