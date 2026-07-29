/// Cell types: literal values and formulas with cached results.

import "../value/cell_value" as Value

/// Base Cell class. Two variants: LiteralCell (value is source of truth) or
/// FormulaCell (value is cached, may go stale). Evaluation is driven externally
/// by Engine.recalc(), not by the Cell itself (REQ-GRID-1, REQ-GRID-2).
class Cell {
  isFormula => false
  isDirty   => false
}

/// A literal cell stores a CellValue directly. Its value is the source of truth
/// and never goes stale.
class LiteralCell is Cell {
  @constructor
  of(v) {
    _value = v
  }

  cachedValue => _value
}

/// A formula cell stores source text, parsed AST, a cached value, and a dirty
/// flag. Only Engine.recalc() writes _cached and _dirty (REQ-GRID-2).
class FormulaCell is Cell {
  @constructor
  of(source, ast) {
    _source = source
    _ast = ast
    _cached = Value.CellEmpty.of()
    _dirty = true
  }

  isFormula   => true
  source      => _source
  ast         => _ast
  cachedValue => _cached
  isDirty     => _dirty

  /// Called ONLY by Engine.recalc() to store a fresh value after ast.eval().
  store(v) {
    _cached = v
    _dirty = false
  }

  /// Called by Engine when a dependency of this cell is written to.
  markDirty {
    _dirty = true
  }
}
