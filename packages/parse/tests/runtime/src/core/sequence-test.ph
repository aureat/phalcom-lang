// test: core-sequence
// map, zip, then, before over TextSource.
import parse_tests.support.test as Testing
from parse.core.pair import Pair
from parse.core.result import RunResult
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

let digitValue = Text.asciiDigit.map |byte| { byte - 48 }

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue, TextSource.new("7"))),
  "ok:7"
)

let pair = digitValue.zip(digitValue)
let paired: RunResult<Pair<Int, Int>> = Parse.run(pair, TextSource.new("73"))

match paired {
  RunResult::Ok(value) => {
    Testing.Test.expect(value.first == 7, "zip keeps the left value")
    Testing.Test.expect(value.second == 3, "zip keeps the right value")
  }
  RunResult::Err(error) => {
    throw Error.new("zip should succeed: " + error.describe)
  }
}

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(Text.char("-").then(digitValue), TextSource.new("-5"))),
  "ok:5"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(digitValue.before(Text.char("!")), TextSource.new("5!"))),
  "ok:5"
)

Testing.Test.pass("core-sequence")
