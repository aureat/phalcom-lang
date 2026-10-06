from ..core.error import ParseError
from ..core.result import ParseResult
from ..core.rule import Rule
from ..core.source import Source

/// Internal indirection so selectors returning `Option<U>` are typed
/// outside `ListSource`'s own generic context.
class TokenPicker<T, U>
{
  mut _selector: (T) -> Option<U>

  @constructor
  new(_ selector: (T) -> Option<U>) {
    _selector = selector
  }

  pick(_ token: T) -> Option<U> {
    _selector.call(token)
  }
}

/// Stable snapshot source over a token list.
///
/// The constructor copies the caller's list into private backing storage
/// so a running parser observes an immutable sequence even if the caller
/// later mutates the original list.
class ListSource<T> {
  mut _items: List<T>

  @constructor
  new(_ items: List<T>) {
    let backing = List<T>.new()
    let mut i = 0
    while i < items.size {
      backing.append(items.at(i))
      i = i + 1
    }
    _items = backing
  }

  /// Number of tokens.
  size -> Int {
    _items.size
  }

  /// Token at `index`; out-of-bounds access is a precondition violation.
  at(_ index: Int) -> T {
    _items.at(index)
  }

  /// A rule matching any single token that satisfies `predicate`.
  @class
  satisfy(_ expected: String, _ predicate: (T) -> Bool) -> Rule<ListSource<T>, T> {
    Rule.new |cursor| {
      let source = cursor.source
      if cursor.offset < source.size {
        let token = source.at(cursor.offset)
        if predicate.call(token) {
          ParseResult<ListSource<T>, T>::Success(token, cursor.advance)
        } else {
          ParseResult<ListSource<T>, T>::Failure(
            ParseError.expecting(cursor.offset, expected),
            false
          )
        }
      } else {
        ParseResult<ListSource<T>, T>::Failure(
          ParseError.expecting(cursor.offset, expected),
          false
        )
      }
    }
  }

  /// Recognizes and transforms a token in one step: the selector returns
  /// `Some(value)` to accept or `None` to reject.
  @class
  select<U>(_ expected: String, _ selector: (T) -> Option<U>) -> Rule<ListSource<T>, U> {
    let picker = TokenPicker.new(selector)
    return Rule.new |cursor| {
      let source = cursor.source
      if cursor.offset < source.size {
        let token = source.at(cursor.offset)
        let picked: Option<U> = picker.pick(token)
        match picked {
          Option::Some(value) => {
            ParseResult<ListSource<T>, U>::Success(value, cursor.advance)
          }
          Option::None => {
            ParseResult<ListSource<T>, U>::Failure(
              ParseError.expecting(cursor.offset, expected),
              false
            )
          }
        }
      } else {
        ParseResult<ListSource<T>, U>::Failure(
          ParseError.expecting(cursor.offset, expected),
          false
        )
      }
    }
  }

  /// A rule that succeeds only at end of input.
  @class
  end -> Rule<ListSource<T>, Unit> {
    Rule.new |cursor| {
      if cursor.atEnd {
        ParseResult<ListSource<T>, Unit>::Success((), cursor)
      } else {
        ParseResult<ListSource<T>, Unit>::Failure(
          ParseError.expecting(cursor.offset, "end of input"),
          false
        )
      }
    }
  }
}

impl<T> Source for ListSource<T> {
  type Token = T
}

export ListSource
