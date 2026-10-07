// test: core-recursion
// Deferred rules, recursive grammars, non-progressing repetition.
import parse_tests.support.test as Testing
from parse.core.rule import Rule
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

let digitValue = Text.asciiDigit.map |byte| { byte - 48 }

// Recursive grammar: expr = digit | '(' expr ')'
let expression: Rule<TextSource, Int> = Rule.defer

let parenthesized = expression.between(Text.char("("), Text.char(")"))
let actual = digitValue.orElse(parenthesized)

expression.define(actual)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(expression.before(Text.end), TextSource.new("5"))),
  "ok:5"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(expression.before(Text.end), TextSource.new("((7))"))),
  "ok:7"
)
Testing.Test.expect(
  Testing.Test.show(Parse.run(expression.before(Text.end), TextSource.new("((7)"))).contains("err:"),
  "unbalanced parentheses fail"
)

// Deeply nested recursion.
Testing.Test.expectEqual(
  Testing.Test.show(Parse.run(expression.before(Text.end), TextSource.new("((((((9))))))"))),
  "ok:9"
)

// A non-progressing repetition must be detected and rejected.
let alwaysNothing = digitValue.optional
let dangerous = alwaysNothing.many
let mut caughtProgress = false

try {
  let probe = Parse.run(dangerous, TextSource.new("x"))
  caughtProgress = false
} catch progressError {
  caughtProgress = true
}

Testing.Test.expect(caughtProgress, "optional.many throws NonProgressingParser")

// defer without define throws when executed.
let undefined: Rule<TextSource, Int> = Rule.defer
let mut caughtUndefined = false

try {
  let probe = Parse.run(undefined, TextSource.new("1"))
  caughtUndefined = false
} catch undefinedError {
  caughtUndefined = true
}

Testing.Test.expect(caughtUndefined, "executing an undefined deferred rule throws")

Testing.Test.pass("core-recursion")
