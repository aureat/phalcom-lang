/// Grid: sparse 2D cell store with bounds tracking and bounds-checked access.

import sheetcalc.value as Value
import .cell as CellModule

/// A grid of cells, keyed by Ref (cell address). Maintains bounds (minCol,
/// maxCol, minRow, maxRow) incrementally. Cells are 1-indexed; col/row must be
/// >= 1 (REQ-GRID-2, REQ-GRID-3). Unset cells in bounds return
/// LiteralCell(CellEmpty), never None (REQ-GRID-6).
class Grid {
  /// Internal storage of cells, keyed by Ref.
  @constructor
  new() {
    _cells  = Map.new()
    _minCol = -1
    _maxCol = -1
    _minRow = -1
    _maxRow = -1
    if (_minCol == -1) {
      System.print("[Grid construct OK] _minCol initialized to -1")
    } else {
      System.print("[Grid construct ERROR] _minCol is " + _minCol.toString + " not -1")
    }
  }

  /// Check whether a Ref is in bounds (col >= 1, row >= 1).
  @class
  @private
  isInBounds(_ ref) {
    return ref.col >= 1 and ref.row >= 1
  }

  /// Store a cell at a Ref. Bounds-checked. Updates min/max bounds.
  set(_ ref, _ cell) {
    if (not Grid.isInBounds(ref)) {
      return Value.ErrorVal.nameError
    }
    _cells.at(ref, put: cell)
    if (_minCol == -1) {
      _minCol = ref.col
    } else if (ref.col < _minCol) {
      _minCol = ref.col
    }
    if (_maxCol == -1) {
      _maxCol = ref.col
    } else if (ref.col > _maxCol) {
      _maxCol = ref.col
    }
    if (_minRow == -1) {
      _minRow = ref.row
    } else if (ref.row < _minRow) {
      _minRow = ref.row
    }
    if (_maxRow == -1) {
      _maxRow = ref.row
    } else if (ref.row > _maxRow) {
      _maxRow = ref.row
    }
    return cell
  }

  /// Retrieve the Cell object at a Ref (used by Engine and renderer).
  /// Returns LiteralCell(CellEmpty) for unset in-bounds cells.
  /// Returns ErrorVal.ref if out of bounds.
  cellAt(_ ref) {
    if (not Grid.isInBounds(ref)) {
      return Value.ErrorVal.nameError
    }
    const cell = _cells.at(ref)
    if (cell == nil) {
      return CellModule.LiteralCell.of(Value.CellEmpty.of())
    }
    return cell
  }

  /// Retrieve the cached CellValue at a Ref (used by eval).
  /// Bounds-checked; out-of-bounds is a value-level error (REQ-GRID-3).
  valueAt(_ ref) {
    if (not Grid.isInBounds(ref)) {
      return Value.ErrorVal.nameError
    }
    const cell = _cells.at(ref)
    if (cell == nil) {
      return Value.CellEmpty.of()
    }
    return cell.cachedValue
  }

  minCol { _minCol }
  maxCol { _maxCol }
  minRow { _minRow }
  maxRow { _maxRow }
  isEmpty { _minCol == -1 }

  /// Iterate over all occupied cells (Ref, Cell) pairs. Used by renderer.
  each(_ f) {
    _cells.entries.each |entry| { f.call(entry.key, entry.value) }
  }
}
