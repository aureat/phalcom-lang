/// RefRange: a rectangle of cell references. Extends Iterable for for/map/where.

import "../support/num" as NumModule
import "./ref" as RefModule

/// A rectangle of cell references from one corner to another. Normalizes corner
/// order (B7:A1 == A1:B7). Extends Iterable so it works with for/map/where
/// (REQ-GRID-7, REQ-GRID-8). Iterates in row-major order.
class RefRange is Iterable {
  /// Construct a range between two corners. Normalizes order.
  @constructor
  fromTo(a, b) {
    _minCol = NumModule.Num.min([a.col, b.col])
    _maxCol = NumModule.Num.max([a.col, b.col])
    _minRow = NumModule.Num.min([a.row, b.row])
    _maxRow = NumModule.Num.max([a.row, b.row])
    _width = (_maxCol - _minCol) + 1
  }

  topLeft     => RefModule.Ref.at(_minCol, _minRow)
  bottomRight => RefModule.Ref.at(_maxCol, _maxRow)
  size        => _width * ((_maxRow - _minRow) + 1)

  /// Row-major iteration. Inherited Iterable.iterate walks 0..size; we map
  /// each cursor to a Ref via iteratorValue.
  iteratorValue(cursor) {
    let dc = cursor % _width
    let dr = (cursor - dc) / _width
    return RefModule.Ref.at(_minCol + dc, _minRow + dr)
  }

  /// Test whether a Ref is in this range.
  contains(ref) {
    return (ref.col >= _minCol) and (ref.col <= _maxCol) and (ref.row >= _minRow) and (ref.row <= _maxRow)
  }

  toString {
    return self.topLeft.toA1 + ":" + self.bottomRight.toA1
  }

  /// Parse A1:B7 notation into a RefRange.
  static fromA1(text) {
    let parts = text.split(":")
    return RefRange.fromTo(RefModule.Ref.fromA1(parts.at(0)), RefModule.Ref.fromA1(parts.at(1)))
  }
}
