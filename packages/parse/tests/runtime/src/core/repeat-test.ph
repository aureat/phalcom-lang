// test: core-repeat
// optional, many, many1, repeat, sepBy, sepBy1.
import parse_tests.support.test as Testing
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

let digitValue = Text.asciiDigit.map |byte| { byte - 48 }

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.optional, TextSource.new("x"))),
  "ok:None"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.optional, TextSource.new("4"))),
  "ok:Some(4)"
)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.many1, TextSource.new("123x"))),
  "ok:[1, 2, 3]"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.many, TextSource.new("x"))),
  "ok:[]"
)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.repeat(3), TextSource.new("123"))),
  "ok:[1, 2, 3]"
)
Testing.Test.expect(
  Testing.Test.show(Parse.run(digitValue.repeat(3), TextSource.new("12"))) != "ok",
  "repeat fails on insufficient input"
)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.sepBy(Text.char(",")), TextSource.new("1,2,3"))),
  "ok:[1, 2, 3]"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.sepBy(Text.char(",")), TextSource.new("x"))),
  "ok:[]"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.sepBy1(Text.char(",")), TextSource.new("1,"))),
  "ok:[1]"
)

Testing.Test.pass("core-repeat")
