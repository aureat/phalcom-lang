// test: core-basics
// Primitives over TextSource.
import parse_tests.support.test as Testing
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

let digitByte = Text.satisfy("digit") |byte| { if byte >= 48 and byte <= 57 { true } else { false } }

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitByte, TextSource.new("7x"))),
  "ok:55"
)
Testing.Test.expect(
  Testing.Test.show(Parse.run(digitByte, TextSource.new("x"))).contains("expected digit"),
  "satisfy failure names the expectation"
)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(Text.char("["), TextSource.new("[1"))),
  "ok:91"
)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(Text.literal("true"), TextSource.new("truex"))),
  "ok:true"
)
Testing.Test.expect(
  Testing.Test.show(Parse.run(Text.literal("true"), TextSource.new("tr"))).contains("true"),
  "literal failure quotes the expected text"
)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(Text.oneOf("abc"), TextSource.new("b"))),
  "ok:98"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(Text.noneOf("abc"), TextSource.new("d"))),
  "ok:100"
)

let letters = Text.takeWhile1("letter") |byte| {
  if (byte >= 97 and byte <= 122) or (byte >= 65 and byte <= 90) {
    true
  } else {
    false
  }
}

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(letters, TextSource.new("ab1c"))),
  "ok:ab"
)

let spaces = Text.takeWhile() |byte| { byte == 32 }

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(spaces, TextSource.new("   x"))),
  "ok:   "
)

Testing.Test.pass("core-basics")
