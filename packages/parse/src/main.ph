from .core.result import RunResult
from .run import Parse
from .source.text_source import TextSource
from .text import Text

let outcome: RunResult<List<Int>> = Parse.runAll(Text.asciiDigit.many1, TextSource.new("1234"))
match outcome {
  RunResult::Ok(value) => {
    System.print(value)
  }
  RunResult::Err(error) => {
    System.print(error.describe)
  }
}
