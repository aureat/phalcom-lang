from .cursor import Cursor
from .error import ParseError
from .pair import Pair
from .result import ParseResult
from .span import Span, Spanned

/// Internal indirection so callbacks returning `Rule<S, U>` are typed
/// outside `Rule`'s own self-referential generic context.
class NextPicker<S, T, U>
{
  mut _callback: (T) -> Rule<S, U>

  @constructor
  new(_ callback: (T) -> Rule<S, U>) {
    _callback = callback
  }

  pick(_ value: T) -> Rule<S, U> {
    _callback.call(value)
  }
}

/// The typed parser boundary.
///
/// `Rule<S, T>` parses values of type `T` from a source of type `S`. It
/// erases the concrete parser implementation type while statically
/// preserving both the source type and the result type.
///
/// v1 composition discipline ("erased joints"): combinator methods build
/// composed rules directly; sources and primitive factories in
/// `parse.source` and `parse.text` construct leaf rules. Ordinary
/// composition never erases values to `Dynamic`.
///
/// Commitment semantics:
/// - primitives fail uncommitted unless input was meaningfully consumed;
/// - sequencing commits a failure that happens after the sequence made
///   progress;
/// - `orElse` only tries its alternative after an uncommitted failure;
/// - `attempt` re-enables backtracking; `cut` forbids it.
///
/// Implementation note: closures stored in rules escape their defining
/// frames, so combinator bodies never use `return`; control flow is
/// expressed through flags, `while`, and tail expressions.
class Rule<S, T> {
  mut _run: (Cursor<S>) -> ParseResult<S, T>

  @constructor
  new(_ run: (Cursor<S>) -> ParseResult<S, T>) {
    _run = run
  }

  /// The underlying run closure. Used by `define` to link deferred rules.
  runFn -> (Cursor<S>) -> ParseResult<S, T> {
    _run
  }

  /// Runs this rule at `cursor`.
  run(_ cursor: Cursor<S>) -> ParseResult<S, T> {
    _run.call(cursor)
  }

  /// Links a deferred rule to its actual body. Defining a parser with the
  /// wrong source or output type is impossible: the parameter is
  /// `Rule<S, T>` of this rule's exact types.
  define(_ body: Rule<S, T>) -> Rule<S, T> {
    _run = body.runFn
    return self
  }

  /// Creates a placeholder for a recursive rule. Executing it before
  /// `define` violates a parser-library invariant and throws.
  @class
  defer -> Rule<S, T> {
    Rule.new |cursor| {
      throw Error.new("parse: deferred rule executed before its definition")
    }
  }

  /// A rule that succeeds with `value` without consuming input.
  @class
  pure(_ value: T) -> Rule<S, T> {
    Rule.new |cursor| {
      ParseResult<S, T>::Success(value, cursor)
    }
  }

  /// A rule that always fails with `message` at the current offset.
  @class
  fail(_ message: String) -> Rule<S, T> {
    Rule.new |cursor| {
      ParseResult<S, T>::Failure(ParseError.messageAt(cursor.offset, message), false)
    }
  }

  /// Transforms the produced value: `Parser<T> + (T)->U = Parser<U>`.
  ///
  /// The transform is statically required to consume exactly `T`.
  map<U>(_ transform: (T) -> U) -> Rule<S, U> {
    let inner = self
    return Rule.new |cursor| {
      let step = inner.run(cursor)
      match step {
        ParseResult::Success(value, next) => {
          ParseResult<S, U>::Success(transform.call(value), next)
        }
        ParseResult::Failure(error, committed) => {
          ParseResult<S, U>::Failure(error, committed)
        }
      }
    }
  }

  /// Value-guided sequencing: the parsed value decides the next rule.
  ///
  /// `Parser<T> + (T)->Rule<S,U> = Parser<U>`.
  flatMap<U>(_ next: (T) -> Rule<S, U>) -> Rule<S, U> {
    let inner = self
    let picker = NextPicker<S, T, U>.new(next)
    return Rule.new |cursor| {
      let step = inner.run(cursor)
      match step {
        ParseResult::Success(value, rest) => {
          let after: ParseResult<S, U> = picker.pick(value).run(rest)
          match after {
            ParseResult::Success(result, tail) => {
              ParseResult<S, U>::Success(result, tail)
            }
            ParseResult::Failure(error, committed) => {
              if committed or (rest.offset != cursor.offset) {
                ParseResult<S, U>::Failure(error, true)
              } else {
                ParseResult<S, U>::Failure(error, false)
              }
            }
          }
        }
        ParseResult::Failure(error, committed) => {
          ParseResult<S, U>::Failure(error, committed)
        }
      }
    }
  }

  /// Heterogeneous sequencing producing a typed product.
  zip<U>(_ other: Rule<S, U>) -> Rule<S, Pair<T, U>> {
    let left = self
    return Rule.new |cursor| {
      let first = left.run(cursor)
      match first {
        ParseResult::Success(a, rest) => {
          let second = other.run(rest)
          match second {
            ParseResult::Success(b, tail) => {
              ParseResult<S, Pair<T, U>>::Success(Pair<T, U>(first: a, second: b), tail)
            }
            ParseResult::Failure(error, committed) => {
              if committed or (rest.offset != cursor.offset) {
                ParseResult<S, Pair<T, U>>::Failure(error, true)
              } else {
                ParseResult<S, Pair<T, U>>::Failure(error, false)
              }
            }
          }
        }
        ParseResult::Failure(error, committed) => {
          ParseResult<S, Pair<T, U>>::Failure(error, committed)
        }
      }
    }
  }

  /// Sequences `self` then `other`, keeping the right result.
  then<U>(_ other: Rule<S, U>) -> Rule<S, U> {
    self.zip(other).map |pair| { pair.second }
  }

  /// Sequences `self` then `other`, keeping the left result.
  before<U>(_ other: Rule<S, U>) -> Rule<S, T> {
    self.zip(other).map |pair| { pair.first }
  }

  /// Controlled-backtracking choice.
  ///
  /// Runs `other` only when `self` fails uncommitted, restoring the
  /// original cursor. Failures merge by furthest-offset preference.
  orElse(_ other: Rule<S, T>) -> Rule<S, T> {
    let left = self
    return Rule.new |cursor| {
      let first = left.run(cursor)
      match first {
        ParseResult::Success(value, next) => {
          ParseResult<S, T>::Success(value, next)
        }
        ParseResult::Failure(error, committed) => {
          if committed {
            ParseResult<S, T>::Failure(error, true)
          } else {
            let second = other.run(cursor)
            match second {
              ParseResult::Success(value, next) => {
                ParseResult<S, T>::Success(value, next)
              }
              ParseResult::Failure(otherError, otherCommitted) => {
                ParseResult<S, T>::Failure(error.merge(otherError), otherCommitted)
              }
            }
          }
        }
      }
    }
  }

  /// `Parser<T>` into `Parser<Option<T>>`.
  ///
  /// An uncommitted failure becomes `None` without consuming input; a
  /// committed failure propagates.
  optional -> Rule<S, Option<T>> {
    let inner = self
    return Rule.new |cursor| {
      let step = inner.run(cursor)
      match step {
        ParseResult::Success(value, next) => {
          ParseResult<S, Option<T>>::Success(Some(value), next)
        }
        ParseResult::Failure(error, committed) => {
          if committed {
            ParseResult<S, Option<T>>::Failure(error, true)
          } else {
            ParseResult<S, Option<T>>::Success(None, cursor)
          }
        }
      }
    }
  }

  /// Zero or more repetitions. Iterations must advance the cursor; a
  /// successful non-advancing iteration violates the repetition progress
  /// invariant and throws `NonProgressingParser`.
  many -> Rule<S, List<T>> {
    let inner = self
    return Rule.new |cursor| {
      let acc = List<T>.new()
      let mut current = cursor
      let mut committedFailure: Option<ParseError> = None
      let mut done = false
      while not done {
        let step = inner.run(current)
        match step {
          ParseResult::Success(value, next) => {
            if next.offset == current.offset {
              throw Error.new("parse: NonProgressingParser in many()")
            }
            acc.append(value)
            current = next
          }
          ParseResult::Failure(error, failureCommitted) => {
            if failureCommitted {
              committedFailure = Some(error)
            }
            done = true
          }
        }
      }
      match committedFailure {
        Some(error) => {
          ParseResult<S, List<T>>::Failure(error, true)
        }
        None => {
          ParseResult<S, List<T>>::Success(acc, current)
        }
      }
    }
  }

  /// One or more repetitions. An uncommitted failure on the first
  /// iteration propagates as a failure.
  many1 -> Rule<S, List<T>> {
    let inner = self
    return Rule.new |cursor| {
      let first = inner.run(cursor)
      match first {
        ParseResult::Failure(error, committed) => {
          ParseResult<S, List<T>>::Failure(error, committed)
        }
        ParseResult::Success(value, next) => {
          if next.offset == cursor.offset {
            throw Error.new("parse: NonProgressingParser in many1()")
          }
          let acc = List<T>.new()
          acc.append(value)
          let mut current = next
          let mut committedFailure: Option<ParseError> = None
          let mut done = false
          while not done {
            let step = inner.run(current)
            match step {
              ParseResult::Success(value2, next2) => {
                if next2.offset == current.offset {
                  throw Error.new("parse: NonProgressingParser in many1()")
                }
                acc.append(value2)
                current = next2
              }
              ParseResult::Failure(error2, failureCommitted) => {
                if failureCommitted {
                  committedFailure = Some(error2)
                }
                done = true
              }
            }
          }
          match committedFailure {
            Some(error3) => {
              ParseResult<S, List<T>>::Failure(error3, true)
            }
            None => {
              ParseResult<S, List<T>>::Success(acc, current)
            }
          }
        }
      }
    }
  }

  /// Exactly `count` repetitions.
  repeat(_ count: Int) -> Rule<S, List<T>> {
    let inner = self
    return Rule.new |cursor| {
      let acc = List<T>.new()
      let mut current = cursor
      let mut failure: Option<ParseError> = None
      let mut failed = false
      let mut i = 0
      while i < count and not failed {
        let step = inner.run(current)
        match step {
          ParseResult::Success(value, next) => {
            if next.offset == current.offset {
              throw Error.new("parse: NonProgressingParser in repeat()")
            }
            acc.append(value)
            current = next
          }
          ParseResult::Failure(error, failureCommitted) => {
            failure = Some(error)
            failed = true
          }
        }
        i = i + 1
      }
      if failed {
        match failure {
          Some(error) => {
            ParseResult<S, List<T>>::Failure(error, true)
          }
          None => {
            ParseResult<S, List<T>>::Failure(
              ParseError.internalAt(cursor.offset, "repeat ended without error"),
              true
            )
          }
        }
      } else {
        ParseResult<S, List<T>>::Success(acc, current)
      }
    }
  }

  /// Zero or more `item` separated by `sep`. The separator's output is
  /// discarded; a trailing separator attempt is not consumed.
  sepBy<Sep>(_ sep: Rule<S, Sep>) -> Rule<S, List<T>> {
    self.sepBy1(sep).orElse(Rule.pure(List<T>.new()))
  }

  /// One or more `item` separated by `sep`.
  sepBy1<Sep>(_ sep: Rule<S, Sep>) -> Rule<S, List<T>> {
    let item = self
    return Rule.new |cursor| {
      let first = item.run(cursor)
      match first {
        ParseResult::Failure(error, committed) => {
          ParseResult<S, List<T>>::Failure(error, committed)
        }
        ParseResult::Success(value, next) => {
          if next.offset == cursor.offset {
            throw Error.new("parse: NonProgressingParser in sepBy1()")
          }
          let acc = List<T>.new()
          acc.append(value)
          let mut current = next
          let mut committedFailure: Option<ParseError> = None
          let mut done = false
          while not done {
            let sepStep = sep.run(current)
            match sepStep {
              ParseResult::Failure(sepError, sepCommitted) => {
                if sepCommitted {
                  committedFailure = Some(sepError)
                }
                done = true
              }
              ParseResult::Success(_, afterSep) => {
                let itemStep = item.run(afterSep)
                match itemStep {
                  ParseResult::Failure(itemError, itemCommitted) => {
                    if itemCommitted {
                      committedFailure = Some(itemError)
                    }
                    done = true
                  }
                  ParseResult::Success(value2, afterItem) => {
                    if afterItem.offset == current.offset {
                      throw Error.new("parse: NonProgressingParser in sepBy1()")
                    }
                    acc.append(value2)
                    current = afterItem
                  }
                }
              }
            }
          }
          match committedFailure {
            Some(error3) => {
              ParseResult<S, List<T>>::Failure(error3, true)
            }
            None => {
              ParseResult<S, List<T>>::Success(acc, current)
            }
          }
        }
      }
    }
  }

  /// `open value close`, preserving `value`'s output.
  between<Open, Close>(_ open: Rule<S, Open>, _ close: Rule<S, Close>) -> Rule<S, T> {
    open.then(self).before(close)
  }

  /// Runs `self` without consuming input on success.
  lookAhead -> Rule<S, T> {
    let inner = self
    return Rule.new |cursor| {
      let step = inner.run(cursor)
      match step {
        ParseResult::Success(value, _) => {
          ParseResult<S, T>::Success(value, cursor)
        }
        ParseResult::Failure(error, committed) => {
          ParseResult<S, T>::Failure(error, committed)
        }
      }
    }
  }

  /// Succeeds when `other` does not match here, consuming nothing.
  ///
  /// Used for lexical boundaries: a keyword must not be a prefix of a
  /// longer identifier.
  notFollowedBy<U>(_ other: Rule<S, U>, _ boundary: String) -> Rule<S, Unit> {
    return Rule.new |cursor| {
      let step = other.run(cursor)
      match step {
        ParseResult::Success(_, _) => {
          ParseResult<S, Unit>::Failure(
            ParseError.expecting(cursor.offset, "not followed by " + boundary),
            false
          )
        }
        ParseResult::Failure(_, _) => {
          ParseResult<S, Unit>::Success((), cursor)
        }
      }
    }
  }

  /// Converts any failure into an uncommitted (backtrackable) one.
  attempt -> Rule<S, T> {
    let inner = self
    return Rule.new |cursor| {
      let step = inner.run(cursor)
      match step {
        ParseResult::Success(value, next) => {
          ParseResult<S, T>::Success(value, next)
        }
        ParseResult::Failure(error, _) => {
          ParseResult<S, T>::Failure(error, false)
        }
      }
    }
  }

  /// Converts any failure into a committed one: no alternative may retry.
  cut -> Rule<S, T> {
    let inner = self
    return Rule.new |cursor| {
      let step = inner.run(cursor)
      match step {
        ParseResult::Success(value, next) => {
          ParseResult<S, T>::Success(value, next)
        }
        ParseResult::Failure(error, _) => {
          ParseResult<S, T>::Failure(error, true)
        }
      }
    }
  }

  /// Adds a grammar context to any failure, enriching diagnostics.
  named(_ context: String) -> Rule<S, T> {
    let inner = self
    return Rule.new |cursor| {
      let step = inner.run(cursor)
      match step {
        ParseResult::Success(value, next) => {
          ParseResult<S, T>::Success(value, next)
        }
        ParseResult::Failure(error, committed) => {
          ParseResult<S, T>::Failure(error.withContext(context), committed)
        }
      }
    }
  }

  /// Captures the exact consumed interval: `Parser<T>` into
  /// `Parser<Spanned<T>>`.
  spanned -> Rule<S, Spanned<T>> {
    let inner = self
    return Rule.new |cursor| {
      let step = inner.run(cursor)
      match step {
        ParseResult::Success(value, next) => {
          let span = Span(start: cursor.offset, end: next.offset)
          ParseResult<S, Spanned<T>>::Success(Spanned<T>(value: value, span: span), next)
        }
        ParseResult::Failure(error, committed) => {
          ParseResult<S, Spanned<T>>::Failure(error, committed)
        }
      }
    }
  }

  /// Left-associative operator chain. `operator` produces the semantic
  /// reduction function itself: `Operator.Output == (T, T) -> T`.
  chainLeft1(_ operator: Rule<S, (T, T) -> T>) -> Rule<S, T> {
    let operand = self
    return Rule.new |cursor| {
      let first = operand.run(cursor)
      match first {
        ParseResult::Failure(error, committed) => {
          ParseResult<S, T>::Failure(error, committed)
        }
        ParseResult::Success(initial, start) => {
          let mut acc = initial
          let mut current = start
          let mut failure: Option<ParseError> = None
          let mut done = false
          while not done {
            let opStep = operator.run(current)
            match opStep {
              ParseResult::Failure(opError, opCommitted) => {
                if opCommitted {
                  failure = Some(opError)
                }
                done = true
              }
              ParseResult::Success(f, afterOp) => {
                let rhsStep = operand.run(afterOp)
                match rhsStep {
                  ParseResult::Failure(rhsError, rhsCommitted) => {
                    if rhsCommitted or (afterOp.offset != current.offset) {
                      failure = Some(rhsError)
                    }
                    done = true
                  }
                  ParseResult::Success(rhs, afterRhs) => {
                    if afterRhs.offset == current.offset {
                      throw Error.new("parse: NonProgressingParser in chainLeft1()")
                    }
                    acc = f.call(acc, rhs)
                    current = afterRhs
                  }
                }
              }
            }
          }
          match failure {
            Some(error2) => {
              ParseResult<S, T>::Failure(error2, true)
            }
            None => {
              ParseResult<S, T>::Success(acc, current)
            }
          }
        }
      }
    }
  }

  /// Right-associative operator chain.
  chainRight1(_ operator: Rule<S, (T, T) -> T>) -> Rule<S, T> {
    let operand = self
    return Rule.new |cursor| {
      let first = operand.run(cursor)
      match first {
        ParseResult::Failure(error, committed) => {
          ParseResult<S, T>::Failure(error, committed)
        }
        ParseResult::Success(initial, start) => {
          if start.offset == cursor.offset {
            throw Error.new("parse: NonProgressingParser in chainRight1()")
          }
          let items = List<T>.new()
          items.append(initial)
          let ops = List<(T, T) -> T>.new()
          let mut current = start
          let mut failure: Option<ParseError> = None
          let mut done = false
          while not done {
            let opStep = operator.run(current)
            match opStep {
              ParseResult::Failure(opError, opCommitted) => {
                if opCommitted {
                  failure = Some(opError)
                }
                done = true
              }
              ParseResult::Success(f, afterOp) => {
                let rhsStep = operand.run(afterOp)
                match rhsStep {
                  ParseResult::Failure(rhsError, rhsCommitted) => {
                    if rhsCommitted {
                      failure = Some(rhsError)
                    }
                    done = true
                  }
                  ParseResult::Success(rhs, afterRhs) => {
                    if afterRhs.offset == afterOp.offset {
                      throw Error.new("parse: NonProgressingParser in chainRight1()")
                    }
                    ops.append(f)
                    items.append(rhs)
                    current = afterRhs
                  }
                }
              }
            }
          }
          match failure {
            Some(error2) => {
              ParseResult<S, T>::Failure(error2, true)
            }
            None => {
              let mut result = items.at(items.size - 1)
              let mut i = ops.size - 1
              while i >= 0 {
                result = ops.at(i).call(items.at(i), result)
                i = i - 1
              }
              ParseResult<S, T>::Success(result, current)
            }
          }
        }
      }
    }
  }
}

export Rule
