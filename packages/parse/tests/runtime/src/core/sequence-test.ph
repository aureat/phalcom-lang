// test: core-sequence
// map, zip, then, before over TextSource.
import parse_tests.support.test as Testing
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

let digitValue = Text.asciiDigit.map |byte| { byte - 48 }

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue, TextSource.new("7"))),
  "ok:7"
)

let pair = digitValue.zip(digitValue).map |both| { "\(both.first)/\(both.second)" }

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(pair, TextSource.new("73"))),
  "ok:7/3"
)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(Text.char("-").then(digitValue), TextSource.new("-5"))),
  "ok:5"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.before(Text.char("!")), TextSource.new("5!"))),
  "ok:5"
)

Testing.Test.pass("core-sequence")
