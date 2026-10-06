from .cursor import Cursor
from .error import ParseError

/// Algebraic result of running a parser at a cursor.
///
/// `committed` is fundamental to the semantics of alternatives: a
/// committed failure means a syntactic decision was made and competing
/// branches must not run (see `attempt` and `cut`).
data ParseResult<S, T>
{
  @variant
  Success(
    _ value: T,
    _ next: Cursor<S>
  )

  @variant
  Failure(
    _ error: ParseError,
    _ committed: Bool
  )
}

/// Public outcome of parsing a complete input, hiding cursor plumbing.
data RunResult<T>
{
  @variant
  Ok(_ value: T)

  @variant
  Err(_ error: ParseError)
}

export ParseResult, RunResult
