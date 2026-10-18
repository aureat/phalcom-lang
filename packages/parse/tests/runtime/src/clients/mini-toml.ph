// Internal typed parser used by the Mini-TOML runtime integration fixture.
from parse.core.rule import Rule
from parse.core.span import Spanned
from parse.source.text_source import TextSource
from parse.text import Text

data MiniValue {
  Text(_ value: String)
  Integer(_ value: Int)
  Boolean(_ value: Bool)
  Array(_ values: List<MiniValue>)
}

data MiniStatement {
  Assignment(_ key: String, _ value: MiniValue)
  Table(_ name: String)
}

data MiniDocument(
  statements: List<Spanned<MiniStatement>>
)

class MiniToml {
  /// The value grammar is recursive only through arrays.
  @class
  value -> Rule<TextSource, MiniValue> {
    let isKeyByte = |byte: Int| {
      if (byte >= 97 and byte <= 122) or (byte >= 65 and byte <= 90) or (byte >= 48 and byte <= 57) or byte == 45 or byte == 95 {
        true
      } else {
        false
      }
    }
    let value: Rule<TextSource, MiniValue> = Rule.defer
    let hws = Text.takeWhile() |byte| { if byte == 32 or byte == 9 { true } else { false } }
    let digits = Text.asciiDigit.many1.map |bytes| {
      let mut number = 0
      let mut i = 0
      while i < bytes.size {
        number = number * 10 + (bytes.at(i) - 48)
        i = i + 1
      }
      number
    }
    let integer = digits.map |number| { MiniValue::Integer(number) }
    let stringContents = Text.takeWhile() |byte| {
      if byte != 34 and byte != 92 and byte != 10 and byte != 13 { true } else { false }
    }
    let string = Text.char("\"")
      .then(stringContents)
      .before(Text.char("\""))
      .map |text| { MiniValue::Text(text) }
      .named("string")
    let booleanBoundary = Text.satisfy("Boolean token character") |byte| { isKeyByte.call(byte) }
    let trueValue = Text.literal("true")
      .before(Text.notFollowedBy(booleanBoundary, "end of Boolean token"))
      .map |word| { MiniValue::Boolean(true) }
    let falseValue = Text.literal("false")
      .before(Text.notFollowedBy(booleanBoundary, "end of Boolean token"))
      .map |word| { MiniValue::Boolean(false) }
    let boolean = trueValue.orElse(falseValue).named("Boolean")
    let array = Text.char("[")
      .then(hws)
      .then(value.sepBy(hws.then(Text.char(",")).then(hws)))
      .before(hws)
      .before(Text.char("]"))
      .map |items| { MiniValue::Array(items) }
      .named("array")
    value.define(string.orElse(boolean).orElse(integer).orElse(array).named("value"))
    value
  }

  /// Parses source-ordered statements, comments, blank lines, and tables.
  @class
  document -> Rule<TextSource, MiniDocument> {
    let value = MiniToml.value
    let isSpace = |byte: Int| { if byte == 32 or byte == 9 { true } else { false } }
    let keyByte = |byte: Int| {
      if (byte >= 97 and byte <= 122) or (byte >= 65 and byte <= 90) or (byte >= 48 and byte <= 57) or byte == 45 or byte == 95 {
        true
      } else {
        false
      }
    }
    let hws = Text.takeWhile() |byte| { isSpace.call(byte) }
    let hws1 = Text.takeWhile1("horizontal whitespace") |byte| { isSpace.call(byte) }
    let bareKey = Text.takeWhile1("bare key") |byte| { keyByte.call(byte) }
    let assignment = bareKey
      .before(hws)
      .before(Text.char("="))
      .before(hws)
      .zip(value)
      .map |parts| { MiniStatement::Assignment(parts.first, parts.second) }
      .named("assignment")
    let table = Text.char("[")
      .then(bareKey)
      .before(Text.char("]"))
      .map |name| { MiniStatement::Table(name) }
      .named("table")
    let statement = assignment.orElse(table).named("statement")
    let lf = Text.char("\n").map |byte| { () }
    let crlf = Text.literal("\r\n").map |text| { () }
    let newline = crlf.orElse(lf)
    let lineEnd = newline.orElse(Text.end)
    let commentBody = Text.char("#").then(Text.takeWhile() |byte| {
      if byte != 10 and byte != 13 { true } else { false }
    })
    let noStatement: Option<Spanned<MiniStatement>> = None
    let commentLine: Rule<TextSource, Option<Spanned<MiniStatement>>> = hws
      .then(commentBody)
      .before(lineEnd)
      .map |ignored| { noStatement }
    let blankLine: Rule<TextSource, Option<Spanned<MiniStatement>>> = hws
      .then(newline)
      .map |ignored| { noStatement }
    let trailingWhitespaceLine: Rule<TextSource, Option<Spanned<MiniStatement>>> = hws1
      .before(Text.end)
      .map |ignored| { noStatement }
    let statementLine: Rule<TextSource, Option<Spanned<MiniStatement>>> = hws
      .then(statement.spanned)
      .before(hws)
      .before(commentBody.optional)
      .before(lineEnd)
      .map |parsed| { Some(parsed) }
      .named("statement line")
    let line: Rule<TextSource, Option<Spanned<MiniStatement>>> = commentLine.attempt
      .orElse(blankLine.attempt)
      .orElse(trailingWhitespaceLine.attempt)
      .orElse(statementLine)
    let lines: Rule<TextSource, List<Option<Spanned<MiniStatement>>>> = line.many
    lines.before(Text.end).map |parsedLines| {
      let statements = List<Spanned<MiniStatement>>.new()
      let mut i = 0
      while i < parsedLines.size {
        match parsedLines.get(i) {
          Option::Some(Option::Some(parsed)) => { statements.append(parsed) }
          Option::Some(Option::None) => {}
          Option::None => {}
        }
        i = i + 1
      }
      MiniDocument(statements: statements)
    }
  }
}

export MiniValue, MiniStatement, MiniDocument, MiniToml
