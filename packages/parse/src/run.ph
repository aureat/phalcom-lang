from .core.cursor import Cursor
from .core.error import ParseError
from .core.result import ParseResult, RunResult
from .core.rule import Rule
from .core.source import Source

/// Execution helpers distinct from parser construction.
class Parse {
  /// Runs `rule` against `source`; does not require exhaustion.
  @class
  run<S, T>(_ rule: Rule<S, T>, _ source: S) -> RunResult<T> {
    let cursor = Cursor.new(source, 0)
    let step = rule.parse(cursor)
    match step {
      ParseResult::Success(value, _) => {
        return RunResult<T>::Ok(value)
      }
      ParseResult::Failure(error, _) => {
        return RunResult<T>::Err(error)
      }
    }
  }

  /// Runs `rule` against `source` and requires end of input: parse one
  /// complete document and reject trailing garbage.
  @class
  runAll<S, T>(_ rule: Rule<S, T>, _ source: S) -> RunResult<T>
    where S |= Source
  {
    let cursor = Cursor.new(source, 0)
    let step = rule.parse(cursor)
    match step {
      ParseResult::Success(value, rest) => {
        if rest.atEnd {
          return RunResult<T>::Ok(value)
        }
        return RunResult<T>::Err(ParseError.expecting(rest.offset, "end of input"))
      }
      ParseResult::Failure(error, _) => {
        return RunResult<T>::Err(error)
      }
    }
  }
}

export Parse
