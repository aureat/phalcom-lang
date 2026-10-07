// test: clients-toml
// A TOML-flavored fragment: dotted keys, values, inline arrays, spans.
// Mirrors the grammar shape from the package design document.
import parse_tests.support.test as Testing
from parse.core.rule import Rule
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

let ws = Text.whitespaceByte.many

let joinWith = |items: List<String>, separator: String| {
  let mut text = ""
  let mut i = 0
  while i < items.size {
    if i > 0 {
      text = text + separator
    }
    text = text + items.at(i)
    i = i + 1
  }
  text
}

let bareKeyBytes = Text.takeWhile() |byte| {
  if (byte >= 97 and byte <= 122) or (byte >= 65 and byte <= 90) or (byte >= 48 and byte <= 57) or byte == 45 or byte == 95 {
    true
  } else {
    false
  }
}

let bareKey = bareKeyBytes.named("bare key")

let quotedKey = Text.char("\"")
  .then(Text.takeWhile() |byte| { byte != 34 })
  .before(Text.char("\""))
  .named("quoted key")

let simpleKey = bareKey.orElse(quotedKey)

let dotSep = ws.then(Text.char(".")).then(ws)

let dottedKey = simpleKey.sepBy1(dotSep).named("dotted key")

// Values: integer, string, boolean, inline array. The value grammar is
// recursive through arrays; strings stand in for scalar payloads to keep
// rendering simple.
let value: Rule<TextSource, String> = Rule.defer

let intValue = Text.asciiDigit.many1.map |bytes| {
  let mut text = ""
  let mut i = 0
  while i < bytes.size {
    text = text + (bytes.at(i) - 48).toString
    i = i + 1
  }
  text
}

let basicString = Text.char("\"")
  .then(Text.takeWhile() |byte| { byte != 34 })
  .before(Text.char("\""))

let boolValue = Text.keyword("true").map |byte| { "true" }.orElse(
  Text.keyword("false").map |byte| { "false" }
)

let arrayElements = value.sepBy(ws.then(Text.char(",")).then(ws))

let inlineArray = Text.char("[")
  .then(ws)
  .then(arrayElements)
  .before(ws.optional)
  .before(Text.char("]"))
  .map |items| { "[" + joinWith.call(items, ", ") + "]" }

value.define(
  intValue
    .orElse(basicString)
    .orElse(boolValue)
    .orElse(inlineArray)
    .named("value")
)

let keyValue = dottedKey
  .before(ws)
  .before(Text.char("="))
  .before(ws)
  .zip(value)
  .spanned
  .map |parsed| {
    let key = joinWith.call(parsed.value.first, ".")
    "\(key) = \(parsed.value.second) [\(parsed.span.start):\(parsed.span.end)]"
  }
  .named("key/value pair")

let document = ws.then(keyValue).before(ws).before(Text.end)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(document, TextSource.new("namespace = \"core\""))),
  "ok:namespace = core [0:18]"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(document, TextSource.new("port = 8080"))),
  "ok:port = 8080 [0:11]"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(document, TextSource.new("debug = true"))),
  "ok:debug = true [0:12]"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(document, TextSource.new("a.b.c = 1"))),
  "ok:a.b.c = 1 [0:9]"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(document, TextSource.new("matrix = [1, 2, 3]"))),
  "ok:matrix = [1, 2, 3] [0:18]"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(document, TextSource.new("nested = [[1, 2], [3]]"))),
  "ok:nested = [[1, 2], [3]] [0:22]"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(document, TextSource.new("  spaced . out = 5"))),
  "ok:spaced.out = 5 [2:18]"
)

// Diagnostics carry grammar contexts.
let bad = Testing.Test.show(Parse.runAll(document, TextSource.new("a = ")))

Testing.Test.expect(bad.contains("err:"), "missing value fails")
Testing.Test.expect(bad.contains("value"), "failure points at the value grammar")

Testing.Test.pass("clients-toml")
