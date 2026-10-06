from .source import Source

/// A position inside a source.
///
/// Cursors are persistent values: advancing produces another cursor and
/// never mutates the source. Advancing is O(1) for O(1)-indexable sources.
///
/// `Cursor` is a class (not data) because it carries behavior.
class Cursor<S> {
  mut _source: S
  mut _offset: Int

  @constructor
  new(_ source: S, _ offset: Int) {
    _source = source
    _offset = offset
  }

  /// The source this cursor reads from.
  source -> S {
    _source
  }

  /// Current position as a token offset from the start.
  offset -> Int {
    _offset
  }

  /// Token at the cursor, if any remains.
  current -> Option<<S as Source>::Token>
    where S |= Source
  {
    if _offset < _source.size {
      Some(_source.at(_offset))
    } else {
      None
    }
  }

  /// Cursor advanced by exactly one token.
  advance -> Cursor<S> {
    Cursor.new(_source, _offset + 1)
  }

  /// Cursor advanced by `amount` tokens.
  advanceBy(_ amount: Int) -> Cursor<S> {
    Cursor.new(_source, _offset + amount)
  }

  /// Whether no tokens remain at or after this cursor.
  atEnd -> Bool
    where S |= Source
  {
    _offset >= _source.size
  }

  /// Tokens remaining at or after this cursor.
  remaining -> Int
    where S |= Source
  {
    _source.size - _offset
  }
}

export Cursor
