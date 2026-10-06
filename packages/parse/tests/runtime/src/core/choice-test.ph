// test: core-choice
// between, orElse, end, runAll, attempt, cut, lookAhead, notFollowedBy.
import parse_tests.support.test as Testing
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

let digitValue = Text.asciiDigit.map |byte| { byte - 48 }
let bracketed = digitValue.between(Text.char("["), Text.char("]"))

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(bracketed, TextSource.new("[4]"))),
  "ok:4"
)

let digitOrDash = digitValue.orElse(Text.char("-").map |byte| { -1 })

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitOrDash, TextSource.new("-"))),
  "ok:-1"
)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.before(Text.end), TextSource.new("9"))),
  "ok:9"
)
Testing.Test.expect(
  Testing.Test.show(Parse.runAll(digitValue, TextSource.new("91"))).contains("end of input"),
  "runAll reports trailing input"
)

// Commit-on-progress: after a consuming prefix succeeds, a failure in the
// rest is committed, so the alternative branch is not tried even though
// it would match.
let abThenC = Text.literal("ab").before(Text.char("c"))
let abFallback = Text.literal("ab").map |text| { "AB" }

Testing.Test.expect(
  Testing.Test.show(Parse.run(abThenC.orElse(abFallback), TextSource.new("ab"))).contains("err:"),
  "committed failure blocks the alternative"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(abThenC.attempt.orElse(abFallback), TextSource.new("ab"))),
  "ok:AB"
)

// cut converts an uncommitted failure into a committed one, blocking an
// alternative that would otherwise succeed.
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(Text.char("[").orElse(digitValue), TextSource.new("7"))),
  "ok:7"
)
Testing.Test.expect(
  Testing.Test.show(Parse.run(Text.char("[").cut.orElse(digitValue), TextSource.new("7"))).contains("err:"),
  "cut blocks the alternative branch"
)

// lookAhead consumes nothing: the digit parser still sees the same byte.
let peek = Text.asciiDigit.lookAhead
let peekThenRead = peek.zip(digitValue).map |pair| { pair.first * 100 + pair.second }

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(peekThenRead, TextSource.new("5"))),
  "ok:5305"
)

// notFollowedBy guards keyword boundaries.
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(Text.literal("true"), TextSource.new("trueX"))),
  "ok:true"
)
Testing.Test.expect(
  Testing.Test.show(Parse.run(Text.keyword("true"), TextSource.new("trueX"))).contains("err:"),
  "keyword rejects identifier prefixes"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(Text.keyword("true"), TextSource.new("true"))),
  "ok:true"
)

Testing.Test.pass("core-choice")
