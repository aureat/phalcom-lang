// test: clients-mini-toml
// Typed, recursive Mini-TOML parser integration through parse's public API.
import parse_tests.support.test as Testing
from parse_tests.clients.mini_toml import MiniDocument, MiniStatement, MiniToml, MiniValue
from parse.core.error import ParseError
from parse.core.result import RunResult
from parse.run import Parse
from parse.source.text_source import TextSource

let valueParser = MiniToml.value
let documentParser = MiniToml.document
let parseValue = |input: String| { Parse.runAll(valueParser, TextSource.new(input)) }
let parseDocument = |input: String| { Parse.runAll(documentParser, TextSource.new(input)) }
let valueOrThrow = |input: String| {
  match parseValue.call(input) {
    RunResult::Ok(parsed) => { parsed }
    RunResult::Err(error) => { throw Error.new("Unexpected value failure: " + error.describe) }
  }
}
let documentOrThrow = |input: String| {
  match parseDocument.call(input) {
    RunResult::Ok(parsed) => { parsed }
    RunResult::Err(error) => { throw Error.new("Unexpected document failure: " + error.describe) }
  }
}
let statementAt = |parsed: MiniDocument, index: Int| {
  match parsed.statements.get(index) {
    Option::Some(item) => { item }
    Option::None => { throw Error.new("Missing statement " + index.toString) }
  }
}
let expectIntegerValue = |parsed: MiniValue, expected: Int| {
  match parsed {
    MiniValue::Integer(number) => { Testing.Test.expect(number == expected, "integer payload matches") }
    _ => { throw Error.new("Expected MiniValue::Integer") }
  }
}
let expectTextValue = |parsed: MiniValue, expected: String| {
  match parsed {
    MiniValue::Text(text) => { Testing.Test.expect(text == expected, "text payload matches") }
    _ => { throw Error.new("Expected MiniValue::Text") }
  }
}
let expectBooleanValue = |parsed: MiniValue, expected: Bool| {
  match parsed {
    MiniValue::Boolean(flag) => { Testing.Test.expect(flag == expected, "Boolean payload matches") }
    _ => { throw Error.new("Expected MiniValue::Boolean") }
  }
}
let expectIntegerAt = |items: List<MiniValue>, index: Int, expected: Int| {
  match items.get(index) {
    Option::Some(item) => { expectIntegerValue.call(item, expected) }
    Option::None => { throw Error.new("Missing array integer") }
  }
}
let expectIntAssignment = |parsed: MiniDocument, index: Int, key: String, expected: Int| {
  match statementAt.call(parsed, index).value {
    MiniStatement::Assignment(actualKey, actualValue) => {
      Testing.Test.expect(actualKey == key, "assignment order and key match")
      expectIntegerValue.call(actualValue, expected)
    }
    _ => { throw Error.new("Expected assignment") }
  }
}
let expectTextAssignment = |parsed: MiniDocument, index: Int, key: String, expected: String| {
  match statementAt.call(parsed, index).value {
    MiniStatement::Assignment(actualKey, actualValue) => {
      Testing.Test.expect(actualKey == key, "assignment order and key match")
      expectTextValue.call(actualValue, expected)
    }
    _ => { throw Error.new("Expected assignment") }
  }
}
let expectBoolAssignment = |parsed: MiniDocument, index: Int, key: String, expected: Bool| {
  match statementAt.call(parsed, index).value {
    MiniStatement::Assignment(actualKey, actualValue) => {
      Testing.Test.expect(actualKey == key, "assignment order and key match")
      expectBooleanValue.call(actualValue, expected)
    }
    _ => { throw Error.new("Expected assignment") }
  }
}
let expectTable = |parsed: MiniDocument, index: Int, expected: String| {
  match statementAt.call(parsed, index).value {
    MiniStatement::Table(name) => { Testing.Test.expect(name == expected, "table name matches") }
    _ => { throw Error.new("Expected table statement") }
  }
}
let contextExists = |error: ParseError, expected: String| {
  let mut found = false
  let mut i = 0
  while i < error.contexts.size {
    if error.contexts.at(i) == expected { found = true }
    i = i + 1
  }
  found
}
let expectDocumentError = |input: String, expectedOffset: Int, expectedContext: String| {
  match parseDocument.call(input) {
    RunResult::Err(error) => {
      Testing.Test.expect(error.offset == expectedOffset, "error offset is the expected UTF-8 byte offset")
      Testing.Test.expect(contextExists.call(error, expectedContext), "error retains a named grammar context")
    }
    RunResult::Ok(_) => { throw Error.new("Malformed input was accepted: " + input) }
  }
}

// P01-P08: scalar variants and nested generic collections of MiniValue.
expectIntegerValue.call(valueOrThrow.call("42"), 42)
expectTextValue.call(valueOrThrow.call("\"Telos\""), "Telos")
expectBooleanValue.call(valueOrThrow.call("true"), true)
expectBooleanValue.call(valueOrThrow.call("false"), false)
match valueOrThrow.call("[]") {
  MiniValue::Array(items) => { Testing.Test.expect(items.size == 0, "empty array") }
  _ => { throw Error.new("Expected empty array") }
}
match valueOrThrow.call("[1, 2, 3]") {
  MiniValue::Array(items) => {
    Testing.Test.expect(items.size == 3, "three integer elements")
    expectIntegerAt.call(items, 0, 1)
    expectIntegerAt.call(items, 1, 2)
    expectIntegerAt.call(items, 2, 3)
  }
  _ => { throw Error.new("Expected integer array") }
}
match valueOrThrow.call("[1, true, \"x\"]") {
  MiniValue::Array(items) => {
    Testing.Test.expect(items.size == 3, "three heterogeneous elements")
    expectIntegerAt.call(items, 0, 1)
    match items.get(1) {
      Option::Some(item) => { expectBooleanValue.call(item, true) }
      Option::None => { throw Error.new("Missing Boolean item") }
    }
    match items.get(2) {
      Option::Some(item) => { expectTextValue.call(item, "x") }
      Option::None => { throw Error.new("Missing text item") }
    }
  }
  _ => { throw Error.new("Expected heterogeneous array") }
}
match valueOrThrow.call("[[1], [2, 3]]") {
  MiniValue::Array(outer) => {
    Testing.Test.expect(outer.size == 2, "two nested arrays")
    match outer.get(0) {
      Option::Some(first) => {
        match first {
          MiniValue::Array(inner) => {
            Testing.Test.expect(inner.size == 1, "first nested array size")
            expectIntegerAt.call(inner, 0, 1)
          }
          _ => { throw Error.new("Expected first nested array") }
        }
      }
      Option::None => { throw Error.new("Missing first nested array") }
    }
    match outer.get(1) {
      Option::Some(second) => {
        match second {
          MiniValue::Array(inner) => {
            Testing.Test.expect(inner.size == 2, "second nested array size")
            expectIntegerAt.call(inner, 0, 2)
            expectIntegerAt.call(inner, 1, 3)
          }
          _ => { throw Error.new("Expected second nested array") }
        }
      }
      Option::None => { throw Error.new("Missing second nested array") }
    }
  }
  _ => { throw Error.new("Expected recursive array") }
}

// Parser reuse keeps earlier results independent of later invocations.
let retained = valueOrThrow.call("[7, [8]]")
expectBooleanValue.call(valueOrThrow.call("false"), false)
expectTextValue.call(valueOrThrow.call("\"reuse\""), "reuse")
match retained {
  MiniValue::Array(items) => { expectIntegerAt.call(items, 0, 7) }
  _ => { throw Error.new("Parser reuse changed an earlier result") }
}

// P09-P13: typed assignments, source order, tables, comments, and spans.
let sample = "# Application configuration\n\ntitle = \"Telos\"\nversion = 2\ndebug = true\n\n[server]\nhost = \"localhost\"\nport = 8080\nenabled = false\n\n[features]\nexperimental = true\nworkers = [2, 4, 8]\n"
let parsed = documentOrThrow.call(sample)
Testing.Test.expect(parsed.statements.size == 10, "ten source-ordered statements")
expectTextAssignment.call(parsed, 0, "title", "Telos")
let titleSpan = statementAt.call(parsed, 0).span
Testing.Test.expect(titleSpan.start == 29, "title span starts at byte 29")
Testing.Test.expect(titleSpan.end == 44, "title span ends at byte 44")
expectIntAssignment.call(parsed, 1, "version", 2)
expectBoolAssignment.call(parsed, 2, "debug", true)
expectTable.call(parsed, 3, "server")
let tableSpan = statementAt.call(parsed, 3).span
Testing.Test.expect(tableSpan.start == 71, "table span starts at byte 71")
Testing.Test.expect(tableSpan.end == 79, "table span ends at byte 79")
expectTextAssignment.call(parsed, 4, "host", "localhost")
expectIntAssignment.call(parsed, 5, "port", 8080)
expectBoolAssignment.call(parsed, 6, "enabled", false)
expectTable.call(parsed, 7, "features")
expectBoolAssignment.call(parsed, 8, "experimental", true)
match statementAt.call(parsed, 9).value {
  MiniStatement::Assignment(key, parsedValue) => {
    Testing.Test.expect(key == "workers", "workers is last in the document")
    match parsedValue {
      MiniValue::Array(items) => {
        Testing.Test.expect(items.size == 3, "workers array contains three items")
        expectIntegerAt.call(items, 0, 2)
        expectIntegerAt.call(items, 1, 4)
        expectIntegerAt.call(items, 2, 8)
      }
      _ => { throw Error.new("Expected typed workers array") }
    }
  }
  _ => { throw Error.new("Expected workers assignment") }
}

let crlf = documentOrThrow.call("\tworker-2_\t=\ttrue\r\nother=2")
Testing.Test.expect(crlf.statements.size == 2, "CRLF and tabs parse as two statements")
expectBoolAssignment.call(crlf, 0, "worker-2_", true)
expectIntAssignment.call(crlf, 1, "other", 2)

// UTF-8 is preserved while the span counts source bytes.
let utf8Input = "title = \"café\""
let utf8 = documentOrThrow.call(utf8Input)
let utf8Span = statementAt.call(utf8, 0).span
Testing.Test.expect(utf8Span.start == 0, "UTF-8 span starts at zero")
Testing.Test.expect(utf8Span.end == TextSource.new(utf8Input).size, "UTF-8 span ends at byte length")
Testing.Test.expect(utf8Span.end == 15, "café contributes five encoded bytes")
expectTextAssignment.call(utf8, 0, "title", "café")

// N01-N10: failures include concrete offsets and parser contexts.
expectDocumentError.call("port =", 6, "assignment")
expectDocumentError.call("port = \"unterminated", 20, "string")
expectDocumentError.call("port = [1, 2", 12, "array")
expectDocumentError.call("port = [1,]", 9, "array")
expectDocumentError.call("port = truex", 11, "Boolean")
expectDocumentError.call("port = 12oops", 9, "statement line")
expectDocumentError.call("[server", 7, "table")
expectDocumentError.call("port = 1.5", 8, "statement line")
expectDocumentError.call("port = [1, [2]", 14, "array")
expectDocumentError.call("port = 1 garbage", 9, "statement line")
let broken = "[server]\nport = 8080\nenabled = true\nbroken ="
expectDocumentError.call(broken, TextSource.new(broken).size, "assignment")

Testing.Test.pass("clients-mini-toml")
