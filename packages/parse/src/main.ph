from .core.result import RunResult
from .run import Parse
from .source.list_source import ListSource
from .source.text_source import TextSource
from .text import Text

// Smoke entry 1: parse digits over a text source.
let outcome: RunResult<List<Int>> = Parse.runAll(Text.asciiDigit.many1, TextSource.new("1234"))

match outcome {
  RunResult::Ok(value) => {
    System.print(value)
  }
  RunResult::Err(error) => {
    System.print(error.describe)
  }
}

// Smoke entry 2: parse a token stream over a list source.
let odds = ListSource<Int>.satisfy("odd number") |value| {
  if value % 2 == 1 {
    true
  } else {
    false
  }
}

let numbers = List<Int>.new()
numbers.append(21)

match Parse.runAll(odds, ListSource<Int>.new(numbers)) {
  RunResult::Ok(value) => {
    System.print(value)
  }
  RunResult::Err(error) => {
    System.print(error.describe)
  }
}
