/// Cell references: location addressing, A1 notation, bijective base-26 columns.

import "../support/num" as NumModule

/// A cell address (column, row). Supports relative and absolute reference forms
/// ($A$1, $A1, A$1, A1). Identity is address-only (REQ-REF-1): two Refs with
/// the same col/row are == regardless of $ flags. Immutable (REQ-REF-2).
class Ref {
  /// Bare relative reference. Column and row are 1-indexed.
  @constructor
  at(_ c, _ r) {
    _col = c
    _row = r
    _colAbs = false
    _rowAbs = false
  }

  /// Full constructor with absoluteness flags. Used by A1 decoder and offset().
  @constructor
  full(_ c, _ r, _ colAbs, _ rowAbs) {
    _col = c
    _row = r
    _colAbs = colAbs
    _rowAbs = rowAbs
  }

  col { _col }
  row { _row }
  colAbs { _colAbs }
  rowAbs { _rowAbs }

  /// Identity: address only. $A$1 == A1 (REQ-REF-1).
  ==(_ o) {
    if (not (o is Ref)) {
      return false
    }
    return o.col == _col and o.row == _row
  }

  /// Fold-hash over col and row, matching core.ph's Range#hash (REQ-REF-3).
  hash {
    let acc = 17
    acc = (acc * 31 + _col.hash) % 999999937
    acc = (acc * 31 + _row.hash) % 999999937
    return acc
  }

  toString {
    return "Ref(" + _col.toString + ", " + _row.toString + ")"
  }

  /// Bijective base-26 column letters. All 26 entries hardcoded (findings §5).
  @class
  @private
  letters { ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"] }

  /// Decode bijective base-26 letters to column number. 'A'=1, 'Z'=26, 'AA'=27.
  @class
  @private
  decodeCol(_ letters) {
    let n = 0
    let i = 0
    while (i < letters.size) {
      n = (n * 26) + (letters.codePointAt(i) - 64)
      i = i + 1
    }
    return n
  }

  /// Encode column number to bijective base-26 letters. REQ-REF-5: handle n%26==0.
  @class
  encodeCol(_ n) {
    let col = n
    let out = ""
    while (col > 0) {
      let rem = col % 26
      if (rem == 0) {
        rem = 26
      }
      out = Ref.letters.at(rem - 1) + out
      col = NumModule.Num.round((col - rem) / 26)
    }
    return out
  }

  /// Parse A1 notation into a Ref. Supports A1, AA10, $A$1, $A1, A$1.
  /// Does not validate row >= 1 (Grid's job, REQ-GRID-2).
  @class
  fromA1(_ text) {
    let i = 0
    let colAbs = false
    let rowAbs = false

    if (i < text.size and Ref.isDollar(text.codePointAt(i))) {
      colAbs = true
      i = i + 1
    }

    let colStart = i
    while (i < text.size and Ref.isUpper(text.codePointAt(i))) {
      i = i + 1
    }
    let colNum = Ref.decodeCol(text.slice(colStart, i))

    if (i < text.size and Ref.isDollar(text.codePointAt(i))) {
      rowAbs = true
      i = i + 1
    }

    let rowStart = i
    while (i < text.size and Ref.isDigit(text.codePointAt(i))) {
      i = i + 1
    }
    let rowNum = Ref.parseDigits(text.slice(rowStart, i))

    return Ref.full(colNum, rowNum, colAbs, rowAbs)
  }

  /// Parse a string of decimal digits into a number.
  @class
  @private
  parseDigits(_ digits) {
    let n = 0
    let i = 0
    while (i < digits.size) {
      n = (n * 10) + (digits.codePointAt(i) - 48)
      i = i + 1
    }
    return n
  }

  /// Character class predicates for A1 parsing.
  @class
  @private
  isDollar(_ code) { code == 36      // '$' }
  @class
  @private
  isDigit(_ code) { code >= 48 and code <= 57 }
  @class
  @private
  isUpper(_ code) { code >= 65 and code <= 90 }

  /// Render Ref back to A1 notation. Preserves $ flags.
  toA1 {
    let out = ""
    if (_colAbs) {
      out = out + "$"
    }
    out = out + Ref.encodeCol(_col)
    if (_rowAbs) {
      out = out + "$"
    }
    out = out + _row.toString
    return out
  }

  /// Offset this Ref by (dCol, dRow). Relative axes shift; absolute axes frozen.
  /// Never mutates self (REQ-REF-2, REQ-REF-8). Performs no bounds check (REQ-REF-9).
  offset(_ dCol, _ dRow) {
    let newCol = if (_colAbs) { _col } else { _col + dCol }
    let newRow = if (_rowAbs) { _row } else { _row + dRow }
    return Ref.full(newCol, newRow, _colAbs, _rowAbs)
  }
}
