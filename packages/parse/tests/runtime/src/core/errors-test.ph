// test: core-errors
// Error merging, named contexts, spans.
import parse_tests.support.test as Testing
from parse.core.error import ParseError
from parse.core.result import RunResult
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

// Furthest-offset preference: the alternative that got further wins.
let digit = Text.asciiDigit
let letterThenDigit = Text.asciiLetter.before(Text.asciiDigit)

// Both fail at offset 0; expectations merge in left-to-right order.
let digitOrLetter = digit.orElse(Text.asciiLetter)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitOrLetter, TextSource.new("!"))),
  "err:expected digit or letter at offset 0"
)

// The right branch advances further, so its error is reported.
let deeper = digit.orElse(letterThenDigit)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(deeper, TextSource.new("a!"))),
  "err:expected digit at offset 1"
)

// Three-way merges accumulate all expectations.
let triple = digit
  .orElse(Text.asciiLetter)
  .orElse(Text.char("("))

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(triple, TextSource.new(")"))),
  "err:expected digit, letter or ( at offset 0"
)

// named() frames the failure with grammar context.
let framed = digit
  .before(Text.char("="))
  .before(digit)
  .named("assignment")

let framedShow = Testing.Test.show(Parse.run(framed, TextSource.new("1=")))

Testing.Test.expect(
  framedShow.contains("expected digit"),
  "named still reports the inner expectation"
)
Testing.Test.expect(
  framedShow.contains("while parsing:"),
  "named adds a context header"
)
Testing.Test.expect(
  framedShow.contains("assignment"),
  "named names the context"
)

// spanned() records the exact consumed interval.
let spannedPair = Text.literal("ab").spanned
let outcome = Parse.run(spannedPair.before(Text.end), TextSource.new("ab"))

match outcome {
  RunResult::Ok(value) => {
    Testing.Test.expect(value.value == "ab", "spanned keeps the value")
    Testing.Test.expect(value.span.start == 0, "span start")
    Testing.Test.expect(value.span.end == 2, "span end covers the consumed prefix")
  }
  RunResult::Err(error) => {
    throw Error.new("spanned should succeed: " + error.describe)
  }
}

// ParseError merging is also directly testable.
let left = ParseError.expecting(3, "integer")
let right = ParseError.expecting(5, "identifier")

Testing.Test.expect(right.merge(left).offset == 5, "merge prefers the furthest offset")
Testing.Test.expectEqual(
  left.merge(ParseError.expecting(3, "identifier")).describe,
  "expected integer or identifier at offset 3"
)

Testing.Test.pass("core-errors")
