// test: clients-dsl
// A command DSL exercising alternatives, optional sections, and errors.
import parse_tests.support.test as Testing
from parse.run import Parse
from parse.source.text_source import TextSource
from parse.text import Text

let ws = Text.whitespaceByte.many

// name: letter first, then identifier bytes.
let nameBytes = Text.takeWhile() |byte| {
  if (byte >= 97 and byte <= 122) or (byte >= 65 and byte <= 90) or (byte >= 48 and byte <= 57) or byte == 95 {
    true
  } else {
    false
  }
}

let name = Text.asciiLetter.lookAhead.then(nameBytes)

// integer: optional '-' followed by digits.
let digits = Text.asciiDigit.many1.map |bytes| {
  let mut value = 0
  let mut i = 0
  while i < bytes.size {
    value = value * 10 + (bytes.at(i) - 48)
    i = i + 1
  }
  value
}

let number = Text.char("-").optional.zip(digits).map |parts| {
  let magnitude = parts.second
  if parts.first == Some(45) {
    0 - magnitude
  } else {
    magnitude
  }
}

// 'set' name '=' integer
let setCommand = Text.keyword("set")
  .then(ws)
  .then(name)
  .then(ws)
  .then(Text.char("="))
  .then(ws)
  .then(digits)
  .map |value| { "set:" + value.toString }

// 'run' name
let runCommand = Text.keyword("run")
  .then(ws)
  .then(name)
  .map |target| { "run:" + target }

// 'quit' with an optional status code.
let optionalStatus = ws.then(digits).optional

let quitCommand = Text.keyword("quit")
  .then(optionalStatus)
  .map |status| {
    status.match(
      some: |code| { "quit:" + code.toString },
      none: || { "quit:0" }
    )
  }

let command = setCommand.orElse(runCommand).orElse(quitCommand).named("command")

let line = ws.then(command).before(ws).before(Text.end)

Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(line, TextSource.new("run worker"))),
  "ok:run:worker"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(line, TextSource.new("  run worker  "))),
  "ok:run:worker"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(line, TextSource.new("set limit = 42"))),
  "ok:set:42"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(line, TextSource.new("quit"))),
  "ok:quit:0"
)
Testing.Test.expectEqual(
  Testing.Test.show(Parse.runAll(line, TextSource.new("quit 3"))),
  "ok:quit:3"
)

// Unknown commands produce merged, contextual diagnostics.
let unknown = Testing.Test.show(Parse.runAll(line, TextSource.new("frobnicate")))

Testing.Test.expect(unknown.contains("err:"), "unknown command fails")
Testing.Test.expect(unknown.contains("while parsing:"), "failure carries the command context")
Testing.Test.expect(unknown.contains("command"), "context names the grammar")

// 'run' with a missing target fails after consuming the keyword.
let broken = Testing.Test.show(Parse.runAll(line, TextSource.new("run")))

Testing.Test.expect(broken.contains("err:"), "run without target fails")

// Keywords do not match prefixes of longer identifiers.
Testing.Test.expect(
  Testing.Test.show(Parse.runAll(line, TextSource.new("runrunner"))).contains("err:"),
  "keyword does not match an identifier prefix"
)

Testing.Test.pass("clients-dsl")
