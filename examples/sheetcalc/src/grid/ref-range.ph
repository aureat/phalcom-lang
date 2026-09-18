/// RefRange: a rectangle of cell references. Implements Iterable for for/map/where.

from universe.collections.iterator_laws import Iterator as CanonicalIterator, Iterable as CanonicalIterable
import .num as NumModule
import .ref as RefModule

/// A rectangle of cell references from one corner to another. Normalizes corner
/// order (B7:A1 == A1:B7). Implements Iterable so it works with for/map/where
/// (REQ-GRID-7, REQ-GRID-8). Iterates in row-major order.
class RefRange is Collection {
  /// Construct a range between two corners. Normalizes order.
  @constructor
  fromTo(_ a, _ b) {
    _minCol = NumModule.Num.min([a.col, b.col])
    _maxCol = NumModule.Num.max([a.col, b.col])
    _minRow = NumModule.Num.min([a.row, b.row])
    _maxRow = NumModule.Num.max([a.row, b.row])
    _width = (_maxCol - _minCol) + 1
  }

  topLeft { RefModule.Ref.at(_minCol, _minRow) }
  bottomRight { RefModule.Ref.at(_maxCol, _maxRow) }
  size { _width * ((_maxRow - _minRow) + 1) }

  minCol { _minCol }
  maxCol { _maxCol }
  minRow { _minRow }
  maxRow { _maxRow }
  width { _width }

  iterator -> RefRangeIterator { RefRangeIterator.new(self) }

  /// Test whether a Ref is in this range.
  contains(_ ref) {
    return (ref.col >= _minCol) and (ref.col <= _maxCol) and (ref.row >= _minRow) and (ref.row <= _maxRow)
  }

  toString {
    return self.topLeft.toA1 + ":" + self.bottomRight.toA1
  }

  /// Parse A1:B7 notation into a RefRange.
  @class
  fromA1(_ text) {
    let parts = text.split(":")
    return RefRange.fromTo(RefModule.Ref.fromA1(parts.at(0)), RefModule.Ref.fromA1(parts.at(1)))
  }
}

class RefRangeIterator {
  _range: RefRange
  _cursor: Int
  _size: Int
  _done: Bool

  @constructor
  new(_ range: RefRange) {
    _range = range
    _cursor = 0
    _size = range.size
    _done = false
  }

  next -> Option<Dynamic> {
    if (_done or _cursor >= _size) {
      _done = true
      return None
    }
    let dc = _cursor % _range.width
    let dr = (_cursor - dc) / _range.width
    let ref = RefModule.Ref.at(_range.minCol + dc, _range.minRow + dr)
    _cursor = _cursor + 1
    return Some(ref)
  }

  iterator { self }
}

impl CanonicalIterator for RefRangeIterator {
  type Item = Dynamic
}

impl CanonicalIterable for RefRange {
  type Item = Dynamic
  type Iterator = RefRangeIterator
  iterator -> RefRangeIterator { RefRangeIterator.new(self) }
}
