// test: core-semantic
// flatMap, pure, fail, value-guided parsing.
import parse_tests.support.test as Testing
from parse.core.rule import Rule
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

let digitValue = Text.asciiDigit.map |byte| { byte - 48 }

let doubled = digitValue.flatMap |value| {
  Text.pure(value * 2)
}

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(doubled, TextSource.new("4"))),
  "ok:8"
)

// flatMap routes into different rules based on the parsed value.
let spelled = digitValue.flatMap |value| {
  if value == 0 {
    Text.literal("zero")
  } else {
    Text.literal("many")
  }
}

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(spelled, TextSource.new("0zero"))),
  "ok:zero"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(spelled, TextSource.new("7many"))),
  "ok:many"
)

// Semantic rejection during parsing.
let reject: Rule<TextSource, String> = Text.fail("expected non-zero integer")
let nonZero = digitValue.flatMap |value| {
  if value == 0 {
    reject
  } else {
    Text.pure(value.toString)
  }
}

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(nonZero, TextSource.new("5"))),
  "ok:5"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(nonZero, TextSource.new("0"))),
  "err:expected non-zero integer at offset 1"
)

// A literal byte-count reader: flatMap decides how much input to take.
let takeBytes = digitValue.flatMap |count| {
  Text.asciiDigit.repeat(count)
}

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(takeBytes, TextSource.new("3987654"))),
  "ok:[57, 56, 55]"
)

Testing.Test.pass("core-semantic")
