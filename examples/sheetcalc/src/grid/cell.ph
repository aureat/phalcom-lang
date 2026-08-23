//! Cell types: literal values and formulas with cached results.
from sheetcalc.value import (CellValue, CellNum, CellText, CellBool, CellEmpty, ErrorVal)

/// Base Cell class. Two variants: LiteralCell (value is source of truth) or
/// FormulaCell (value is cached, may go stale). Evaluation is driven externally
/// by Engine.recalc(), not by the Cell itself (REQ-GRID-1, REQ-GRID-2).
class Cell {
  isFormula -> Bool { false }
  isDirty -> Bool { false }
}

/// A literal cell stores a CellValue directly. Its value is the source of truth
/// and never goes stale.
class LiteralCell is Cell {
  _value: CellValue

  @constructor
  of(_ v) {
    _value = v
  }

  cachedValue -> CellValue { _value }
}

/// A formula cell stores source text, parsed AST, a cached value, and a dirty
/// flag. Only Engine.recalc() writes _cached and _dirty (REQ-GRID-2).
class FormulaCell is Cell {
  _source: String
  _ast: Object
  _cached: CellValue
  _dirty: Bool

  @constructor
  of(_ source, _ ast) {
    _source = source
    _ast = ast
    _cached = CellEmpty.of()
    _dirty = true
    ()
  }

  isFormula -> Bool { true }
  source -> String { _source }
  ast -> Object { _ast }
  cachedValue -> CellValue { _cached }
  isDirty -> Bool { _dirty }

  /// Called ONLY by Engine.recalc() to store a fresh value after ast.eval().
  store(_ v) {
    _cached = v
    _dirty = false
  }

  /// Called by Engine when a dependency of this cell is written to.
  markDirty {
    _dirty = true
  }
}
