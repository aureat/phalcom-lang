# SheetCalc — References and Grid

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md) §3, §7, §10,
[01-architecture.md](01-architecture.md), [02-value-model.md](02-value-model.md).

Every code sample in this document was run against `target/release/phalcom` at
the SHA this spec set is verified against. Where a claim matters — hash/`==`
correctness for a `Map` key, the base-26 round trip, the fiber/iteration
interaction — it is stated because a probe confirmed it, not because it looks
right.

---

## 1. `Ref` — a cell address

A `Ref` is a **location**, not a value. It is never stored in a cell and never
flows through `+`. It is the thing a formula token like `A1` or `$B$7` compiles
to, and the key type of the `Grid`'s `Map`. It has no relationship to the
`CellValue` hierarchy in [02-value-model.md](02-value-model.md); it does not
extend it and does not need the `#VALUE!`-default protocol, because a `Ref`
never receives `+`, `-`, or any arithmetic send.

```phalcom
class Ref {
  // Bare relative reference. Column and row are 1-indexed, matching A1
  // notation directly: Ref.at(1, 1) is A1.
  @constructor
  at(c, r) {
    _col = c
    _row = r
    _colAbs = false
    _rowAbs = false
  }

  // Full constructor — used by the A1 decoder and by offset(), which must
  // preserve the absoluteness flags of the ref being shifted.
  @constructor
  full(c, r, colAbs, rowAbs) {
    _col = c
    _row = r
    _colAbs = colAbs
    _rowAbs = rowAbs
  }

  col     => _col
  row     => _row
  colAbs  => _colAbs
  rowAbs  => _rowAbs

  // Identity is the ADDRESS ONLY. $A$1, $A1, A$1, and A1 are == and hash
  // equal — see REQ-REF-1 and the commentary below.
  ==(o) => o.is(Ref) and o.col == _col and o.row == _row

  // Fold-hash over the two fields, mirroring core.ph's own Range#hash
  // (phalcom-core/core/core.ph, class Range) rather than a naive
  // multiply-and-add, which collides across plausible sheet sizes (see
  // commentary).
  hash {
    var acc = 17
    acc = (acc * 31 + _col.hash) % 999999937
    acc = (acc * 31 + _row.hash) % 999999937
    return acc
  }

  toString => "Ref(" + _col.toString + ", " + _row.toString + ")"
}
```

**REQ-REF-1 (address-only identity).** `Ref#==` and `Ref#hash` depend only on
`col` and `row`. `colAbs`/`rowAbs` never participate. Two `Ref`s that name the
same cell are `==` regardless of how many `$` the formula that produced them
had.

**REQ-REF-2 (immutability).** `Ref` has no setters. All four fields are fixed
at construction. `offset()` (§4) and the A1 parser (§2) always produce a *new*
`Ref`, never mutate one in place. This is required, not stylistic: a `Ref`
already installed as a `Map`/`Set` key whose `hash` changed under it would
corrupt every bucket that key lives in.

**REQ-REF-3 (Map/Set key correctness).** `Ref` instances are usable as `Map`
and `Set` keys with correct `hash`/`==` semantics, confirmed by
[00-language-findings.md §7](00-language-findings.md): *"The `Ref`-as-`Map`-key
contract — the thing I flagged as most likely to be under-exercised and
broken — works correctly."* This was re-confirmed here with the absoluteness
flags specifically in play, which the original probe did not exercise:

```phalcom
let m = Map.new()
m.at(Ref.at(1, 1), put: "x")
m.at(Ref.full(1, 1, true, true))   // => "x"  -- $A$1 finds what A1 stored
```

Probe output: `true` for `Ref.at(1,1) == Ref.full(1,1,true,true)`, `true` for
their `hash` equality, and `"x"` for the cross-lookup. This is exactly the
behavior REQ-REF-1 requires and it was **predicted to be the risky part** —
the working part turned out to be `Map`/`Set` honoring a user `hash`/`==` at
all (already settled by findings §7); the part that needed independent
verification was whether the *extra* fields (`colAbs`/`rowAbs`) could be
excluded from identity without special-casing anything in `Map`'s own code,
and they can, because `Map` only ever calls the two messages `Ref` defines.

> **Commentary — the hash design, and why the obvious version is wrong.**
> The first draft of `hash` was `(_col * 131) + _row` — cheap, obvious, wrong.
> `Ref(2, 1)` and `Ref(1, 133)` collide under it, and any sheet with more than
> ~130 rows starts colliding column-adjacent cells against far-away ones. A
> spreadsheet's `Ref` space is dense and two-dimensional, exactly the shape
> that punishes a linear combining function. Reading `core.ph`'s own `Range`
> class settled it: `Range#hash` folds its three bound fields through
> `acc = (acc * 31 + field.hash) % 999999937`, discarding the "clever"
> instinct to invent a spreadsheet-specific scheme in favor of the modular
> multiply-fold the runtime authors already use elsewhere. There is no
> library `Hash.combine(_, _)` to call — `Number` has no such thing and
> neither does any core class (findings §3, §9) — so this is copied by hand
> into `grid/ref.ph`, which is a small but real instance of "the core library
> doesn't give you combinators, so every class re-derives them," the same
> complaint `support/` exists to fix for `floor`/`round` (see §2 below and
> [01-architecture.md §1](01-architecture.md)).

---

## 2. A1 notation

### 2.1 Column letters — bijective base-26

Column letters are **bijective base-26**: `A`=1, `Z`=26, `AA`=27 (not 0, as in
positional base-26 with a leading-zero rule — there is no letter for zero).
Two operations are needed: `Ref.encodeCol(n)` (number → letters, used when
rendering a `Ref` back to text) and the inverse (letters → number, used by the
decoder).

**Decode is the easy direction.** `String#codePointAt(_)` is
`VERIFIED-PRESENT` (findings §5) and gives the numeric codepoint of an ASCII
letter directly, so `'A'..'Z'` → `1..26` is arithmetic, no lookup table:

```phalcom
// letters -> column number. 'A' is codepoint 65, so subtracting 64 maps
// 'A'..'Z' to 1..26 directly. No floor, no lookup table needed.
static decodeCol_(letters) {
  var n = 0
  var i = 0
  while (i < letters.size) {
    n = (n * 26) + (letters.codePointAt(i) - 64)
    i = i + 1
  }
  return n
}
```

**Encode is the hard direction, and it is hard for a language reason.** There
is no `String.fromCodePoint` and no char-from-byte constructor of any kind
(findings §5: *"the only source of characters in a Phalcom program is a
string literal in the source text"*). Going from a number back to a letter
therefore cannot be computed — it must be a **literal table**, all 26 entries
spelled out by hand in the source:

```phalcom
class Ref {
  // ... (§1 continues) ...

  // No String.fromCodePoint exists (findings §5), so there is no way to
  // compute a letter from an index. This table is the only way to go from
  // a column number back to a letter, and it has to be written out by hand.
  static letters_ => [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
    "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"
  ]

  static encodeCol(n) {
    var col = n
    var out = ""
    while (col > 0) {
      var rem = col % 26
      if (rem == 0) { rem = 26 }           // bijective base-26: 0 maps to Z, not "AA-1"
      out = Ref.letters_.at(rem - 1) + out
      col = Num.round((col - rem) / 26)    // see commentary — Num.round is defensive, not optional
    }
    return out
  }
}
```

`Num.round` here is the helper already specified in
[02-value-model.md §5](02-value-model.md), built there from `%` plus a
hand-corrected `floor` (findings §3: `n - (n % 1)` truncates toward zero and
is wrong for negatives). `grid/ref.ph` imports `support/num.ph` and does not
re-derive `floor`/`round` — see the module dependency table in
[01-architecture.md §3](01-architecture.md) (`grid/` may import `support/`).

**Verified round trip** (`Ref.encodeCol` then `Ref.decodeCol_`, run against
the release binary):

| n | encode | decode(encode(n)) |
|---|---|---|
| 1 | `A` | 1 |
| 26 | `Z` | 26 |
| 27 | `AA` | 27 |
| 52 | `AZ` | 52 |
| 53 | `BA` | 53 |
| 702 | `ZZ` | 702 |
| 703 | `AAA` | 703 |
| 18278 | `ZZZ` | 18278 |

The multiples-of-26 boundary (`26→Z`, `52→AZ`, `702→ZZ`) is the specific case
bijective base-26 gets wrong if you copy ordinary base-26 division without the
`rem == 0 → rem = 26` correction; it is verified explicitly above because it
is exactly where a naive port breaks.

### 2.2 Full A1 parse: letters, digits, and `$`

Row numbers are parsed with the mirror-image trick: `codePointAt` on a digit
character gives its codepoint directly, and `'0'` is codepoint 48, so
`digit = codePointAt(i) - 48` needs no lookup table (unlike the column
direction, going *from* a character *to* a number never needs
`fromCodePoint`).

```phalcom
class Ref {
  // ... continued ...

  static isDollar_(code) => code == 36     // '$'
  static isDigit_(code)  => code >= 48 and code <= 57
  static isUpper_(code)  => code >= 65 and code <= 90

  // Parses "A1", "AA10", "$A$1", "$A1", "A$1". Does not validate row >= 1 —
  // that is Grid's job (REQ-GRID-2), not the parser's.
  static fromA1(text) {
    var i = 0
    var colAbs = false
    var rowAbs = false

    if (i < text.size and Ref.isDollar_(text.codePointAt(i))) {
      colAbs = true
      i = i + 1
    }

    var colStart = i
    while (i < text.size and Ref.isUpper_(text.codePointAt(i))) {
      i = i + 1
    }
    var colNum = Ref.decodeCol_(text.rawSlice(colStart, i))

    if (i < text.size and Ref.isDollar_(text.codePointAt(i))) {
      rowAbs = true
      i = i + 1
    }

    var rowStart = i
    while (i < text.size and Ref.isDigit_(text.codePointAt(i))) {
      i = i + 1
    }
    var rowNum = Ref.parseDigits_(text.rawSlice(rowStart, i))

    return Ref.full(colNum, rowNum, colAbs, rowAbs)
  }

  static parseDigits_(digits) {
    var n = 0
    var i = 0
    while (i < digits.size) {
      n = (n * 10) + (digits.codePointAt(i) - 48)
      i = i + 1
    }
    return n
  }

  // The inverse: Ref -> A1 text, for rendering and error messages.
  toA1 {
    var out = ""
    if (_colAbs) { out = out + "$" }
    out = out + Ref.encodeCol(_col)
    if (_rowAbs) { out = out + "$" }
    out = out + _row.toString
    return out
  }
}
```

Verified round trip, run against the release binary:

| input | col | row | colAbs | rowAbs | `.toA1` |
|---|---|---|---|---|---|
| `A1` | 1 | 1 | false | false | `A1` |
| `AA10` | 27 | 10 | false | false | `AA10` |
| `$A$1` | 1 | 1 | true | true | `$A$1` |
| `$A1` | 1 | 1 | true | false | `$A1` |
| `A$1` | 1 | 1 | false | true | `A$1` |
| `ZZ100` | 702 | 100 | false | false | `ZZ100` |

**REQ-REF-4.** `Ref.fromA1(_)` and `#toA1` round-trip for every column in
`1..18278` (`A`..`ZZZ`) and every non-negative row, verified by
`suites/ref_a1.ph`.
**REQ-REF-5.** `Ref.encodeCol(_)`/`Ref.decodeCol_(_)` correctly handle the
`n % 26 == 0` boundary (multiples of 26 map to a trailing `Z`, not a
zero-width or off-by-one letter).

> **Commentary — this is 30 lines of code that should be 3, and the reason is
> specific.** The *decode* direction (letter → number, digit → number) is
> free: `codePointAt` gives you an integer and subtraction does the rest,
> because ASCII already encodes the alphabet as consecutive small integers.
> The *encode* direction (number → letter) has no such shortcut, because
> Phalcom has no way to go from an integer back to a character — no
> `String.fromCodePoint`, no `Char` type, nothing (findings §5). So the
> 26-entry literal table in `Ref.letters_` isn't a stylistic choice, it is
> the **only** implementation available. And every division inside the loop
> — even though the bijective-base-26 math guarantees an exact integer
> result — has to route through `Num.round` rather than a bare `/`, because
> `Number` has no integer type at all (findings §3) and there is no way to
> assert "this float is exactly an integer" without a hand-rolled check. A
> genuinely "integer-only" algorithm about spreadsheet columns still can't
> get through a single division without borrowing the `floor`-built `round`
> helper from [02-value-model.md §5](02-value-model.md), which itself exists
> only because `Number` shipped with nothing. It is the same finding
> (`Number`'s empty method surface, GAP-NUM-3) showing up a second time in a
> completely different part of the program — which is exactly the kind of
> thing this whole exercise is supposed to surface.
>
> One parser-level trap, found while writing this section and **not** in
> [00-language-findings.md](00-language-findings.md): `return [1, 2, 3]`
> (a `return` keyword immediately followed by a list literal) fails to parse
> — `Expected one of ";", newline` at the `[`. The parser appears to treat a
> bare `return` as complete whenever the next token cannot start an
> expression in its lookahead set, and `[` isn't in that set at this
> position, so it stops after `return` and then chokes on `[` as a stray
> token. Confirmed with a minimal repro (`static a() { return [1,2,3] }`)
> against the release binary. Workaround, used throughout this document:
> assign to a local first (`var l = [...]; return l`) or, better, make the
> list an arrow-form getter body (`static letters_ => [...]`), which parses
> fine because there is no `return` keyword in the way. `Ref.letters_` above
> uses the arrow form for exactly this reason, not for brevity.

---

## 3. Absolute vs. relative references

Excel has four reference forms per axis-pair: `A1` (relative/relative), `$A1`
(absolute col/relative row), `A$1` (relative col/absolute row), `$A$1`
(absolute/absolute). The question is whether that is two booleans on one
class or four subclasses.

### DEC-REF-1 — two boolean fields, not a subclass per combination

**Decision.** `Ref` carries `colAbs`/`rowAbs` as plain boolean fields (already
shown in §1). There is no `RelativeRef`/`AbsoluteRef`/`MixedRef` hierarchy.

**Why not subclasses.** Four combinations of two independent booleans is the
textbook case *against* subclassing: it produces `2^2` classes whose only
behavioral difference is in a single method (`offset`, §4), and every other
piece of code that touches a `Ref` (`Grid`, the parser, the `Map` key
contract in §1) would need to pattern-match or duplicate logic across all
four. [02-value-model.md](02-value-model.md)'s `CellValue` hierarchy uses
subclassing because each variant (`CellNum`, `CellText`, `ErrorVal`, ...) has
*genuinely different arithmetic and rendering behavior* dispatched
polymorphically through `+`/`toString`. `Ref`'s absoluteness flags have no
such polymorphic surface — every `Ref`, regardless of `$` markers, supports
exactly the same operations (`col`, `row`, `hash`, `==`, `offset`, `toA1`).
Two fields plus one `if`-shaped method (`offset`) covers it completely, with
zero class proliferation and, per REQ-REF-1, zero interference with the
`hash`/`==` contract `Map` depends on — a fifth class variant would need an
explicit override or an accidental leak of `colAbs`/`rowAbs` into identity,
which is exactly the bug REQ-REF-1 exists to prevent.

**REQ-REF-6.** `colAbs` and `rowAbs` are independent booleans; all four
combinations are representable and distinguishable via `.colAbs`/`.rowAbs`,
but **not** via `==` or `hash` (REQ-REF-1).

---

## 4. Offsetting — the fill/copy primitive

Copying a formula from one cell to another (Excel's "fill handle") shifts
every *relative* reference inside it by the row/column delta and leaves every
*absolute* reference untouched. This is the entire reason `colAbs`/`rowAbs`
exist on `Ref` rather than living only in the parser: the AST node for a cell
reference needs to carry them forward into `offset()` at fill time.

```phalcom
class Ref {
  // ... continued ...

  // dCol/dRow are signed deltas. A relative axis shifts; an absolute axis
  // is returned unchanged. Always produces a NEW Ref (REQ-REF-2).
  offset(dCol, dRow) {
    var newCol = _colAbs.ifTrue({ _col }, ifFalse: { _col + dCol })
    var newRow = _rowAbs.ifTrue({ _row }, ifFalse: { _row + dRow })
    return Ref.full(newCol, newRow, _colAbs, _rowAbs)
  }
}
```

**REQ-REF-7.** `ref.offset(dCol, dRow)` shifts `col` by `dCol` unless
`colAbs`, and shifts `row` by `dRow` unless `rowAbs`; the returned `Ref`
carries the same `colAbs`/`rowAbs` as the original (offsetting a reference
does not change whether it is absolute — `$A$1` offset by anything is still
`$A$1`, not merely "some absolute ref").
**REQ-REF-8.** `offset` never mutates `self`.
**REQ-REF-9.** `offset` performs no bounds check. An offset that walks a
column or row below 1 produces a structurally valid but out-of-grid `Ref`; it
is `Grid`'s job (§6, REQ-GRID-2) to turn that into `ErrorVal(#REF)` at lookup
time, not the parser's or `Ref`'s. This mirrors the two-channel error model in
[01-architecture.md §5](01-architecture.md): a `Ref` is structurally always
valid the moment it is constructed, and going out of bounds is a *value-level*
event discovered on lookup, exactly like a `#DIV/0!`.

---

## 5. `Cell`

A `Cell` holds one of two things: a literal `CellValue`, or a formula (source
text, parsed `Ast`, a cached `CellValue`, and a dirty flag). This mirrors
`CellValue`'s own subclass-per-variant shape
([02-value-model.md §2](02-value-model.md)) rather than one class with
nullable fields, for the same reason: the two variants have genuinely
different behavior (a literal cell's value never goes stale; a formula
cell's does), and `extends`/`super` are solid in this runtime
(findings §9, `attribute` probes), so the boilerplate cost of a small
hierarchy is low.

Per [01-architecture.md §4](01-architecture.md)'s data flow, evaluation is
driven **externally** by `recalc/engine.ph`'s topological sweep — a `Cell`
does not evaluate itself. It exposes a mutation surface the engine calls
(`store`, `markDirty`) and a read surface everything else calls
(`cachedValue`, `isDirty`, `isFormula`).

```phalcom
class Cell {
  isFormula => false
  isDirty   => false
}

class LiteralCell is Cell {
  @constructor
  of(v) { _value = v }

  // A literal's value IS the source of truth; it is never stale.
  cachedValue => _value
}

class FormulaCell is Cell {
  @constructor
  of(source, ast) {
    _source = source
    _ast = ast
    _cached = CellEmpty.of()
    _dirty = true          // never evaluated yet
  }

  isFormula   => true
  source      => _source   // the original formula text, e.g. "=SUM(A1:A3)*2"
  ast         => _ast      // parsed Ast root (05-formula-parser.md)
  cachedValue => _cached
  isDirty     => _dirty

  // Called ONLY by Engine.recalc() after ast.eval(ctx) produces a fresh
  // value in topological order. Nothing else may write _cached/_dirty.
  store(v) {
    _cached = v
    _dirty = false
  }

  // Called by Engine when a dependency of this cell is written. The cell
  // stays stale until the next recalc sweep reaches it.
  markDirty {
    _dirty = true
  }
}
```

**REQ-GRID-1.** Every stored cell is a `LiteralCell` or a `FormulaCell`; there
is no bare `CellValue` or raw `Ast` stored directly in the `Grid`.
**REQ-GRID-2 (mutation discipline).** Only `Engine.recalc()` (owned by
[07-dependency-graph-and-recalc.md](07-dependency-graph-and-recalc.md)) calls
`FormulaCell#store`/`#markDirty`. No other layer — not `Grid`, not
`render/`, not a test — writes `_cached` or `_dirty` directly. This is a
convention, not something the language enforces (Phalcom fields have no
visibility modifier), and it is exactly the kind of invariant that needs a
test hook (§8) rather than a compiler.

---

## 6. `Grid`

`Grid` is a `Map<Ref, Cell>` with bounds tracking for rendering and a
bounds-checked value accessor for evaluation.

```phalcom
class Grid {
  @constructor
  new() {
    _cells  = Map.new()
    _minCol = None
    _maxCol = None
    _minRow = None
    _maxRow = None
  }

  // 1-indexed, unbounded above. There is no upper column/row limit in v1 —
  // see REQ-GRID-4.
  static isInBounds_(ref) => ref.col >= 1 and ref.row >= 1

  // Raw cell storage. Bounds are still enforced here: a formula that
  // offset()s itself into col/row <= 0 must not silently create a phantom
  // entry in the Map.
  set(ref, cell) {
    Grid.isInBounds_(ref).ifFalse { return ErrorVal.of(#REF) }
    _cells.at(ref, put: cell)
    _minCol = (_minCol == None).ifTrue({ ref.col }, ifFalse: { Num.min(_minCol, ref.col) })
    _maxCol = (_maxCol == None).ifTrue({ ref.col }, ifFalse: { Num.max(_maxCol, ref.col) })
    _minRow = (_minRow == None).ifTrue({ ref.row }, ifFalse: { Num.min(_minRow, ref.row) })
    _maxRow = (_maxRow == None).ifTrue({ ref.row }, ifFalse: { Num.max(_maxRow, ref.row) })
    return cell
  }

  // Raw cell lookup — used by the recalc engine and renderer, which need
  // the Cell object itself (its dirty flag, its ast), not just a value.
  // An unset but in-bounds address is a blank cell, not an error: returns a
  // fresh LiteralCell wrapping CellEmpty, never None.
  cellAt(ref) {
    Grid.isInBounds_(ref).ifFalse { return ErrorVal.of(#REF) }
    return _cells.at(ref).unwrapOr(LiteralCell.of(CellEmpty.of()))
  }

  // Value-level accessor — what eval/evaluator.ph calls when a formula
  // dereferences a Ref. Bounds-checked; out-of-bounds is a CellValue-level
  // error (REQ-GRID-3), not a Rust panic and not a Result::Err.
  valueAt(ref) {
    Grid.isInBounds_(ref).ifFalse { return ErrorVal.of(#REF) }
    return self.cellAt(ref).cachedValue
  }

  minCol  => _minCol
  maxCol  => _maxCol
  minRow  => _minRow
  maxRow  => _maxRow
  isEmpty => _minCol == None

  // Entry traversal is explicit because Map#each is one-value traversal.
  // This walks every occupied address without knowing the bounds rectangle.
  each(f) {
    _cells.entries.each { entry => f.call(entry.key, entry.value) }
  }
}
```

**REQ-GRID-3 (out-of-bounds is a value error).** `Grid#valueAt(_)` returns
`ErrorVal.of(#REF)` for any `Ref` with `col < 1` or `row < 1`. This is the
concrete mechanism behind
[02-value-model.md](02-value-model.md)'s `#REF` kind ("reference to an
out-of-bounds cell") and it is the only place in the codebase that produces
it.
**REQ-GRID-4 (unbounded above, deliberately).** There is no maximum column or
row. A real spreadsheet engine caps this (Excel: 16384 columns, 1048576 rows)
to bound memory; SheetCalc v1 does not, because the `Grid` is a sparse `Map`
— an out-of-range-but-positive reference just never gets an entry, at zero
cost, and adding an arbitrary upper cap would be a policy decision with no
forcing reason. If a future version wants one, it is one more comparison in
`isInBounds_`.
**REQ-GRID-5 (bounds tracked incrementally).** `minCol`/`maxCol`/`minRow`/
`maxRow` update on every `set(_,_)` in O(1); they are not recomputed by
scanning `_cells` at render time. `isEmpty` is `_minCol == None` — the four
bounds fields are always all-`None` or all-set together, since every `set`
call updates all four.
**REQ-GRID-6.** `Grid#cellAt(_)` never returns `None` for an in-bounds
address; an unset cell is represented as `LiteralCell.of(CellEmpty.of())`, so
callers never need a `None`-check before reading `.cachedValue`.

---

## 7. `RefRange` — `A1:B7`

A range reference enumerates the rectangle of `Ref`s between two corners. It
is speced as an `Iterable` and can use the explicit `.iter` pipeline rather
than a bespoke iterator class, so it gets `for`, `.toList`, and eager collection
operations for free once `iterate`/`iteratorValue` are defined.

```phalcom
class RefRange is Iterable {
  // Normalizes either corner order: RefRange.fromTo(B7, A1) is the same
  // range as RefRange.fromTo(A1, B7).
  @constructor
  fromTo(a, b) {
    _minCol = Num.min(a.col, b.col)
    _maxCol = Num.max(a.col, b.col)
    _minRow = Num.min(a.row, b.row)
    _maxRow = Num.max(a.row, b.row)
    _width  = (_maxCol - _minCol) + 1
  }

  topLeft     => Ref.at(_minCol, _minRow)
  bottomRight => Ref.at(_maxCol, _maxRow)
  size        => _width * ((_maxRow - _minRow) + 1)

  // Row-major cursor over a flat 0..size index (the default Iterable.iterate
  // is reused unmodified — it already walks 0..self.size). Only
  // iteratorValue needs to know the 2D shape.
  iteratorValue(cursor) {
    var dc = cursor % _width
    var dr = (cursor - dc) / _width
    return Ref.at(_minCol + dc, _minRow + dr)
  }

  contains(ref) {
    return (ref.col >= _minCol) and (ref.col <= _maxCol)
       and (ref.row >= _minRow) and (ref.row <= _maxRow)
  }

  toString => self.topLeft.toA1 + ":" + self.bottomRight.toA1

  static fromA1(text) {
    var parts = text.split(":")
    return RefRange.fromTo(Ref.fromA1(parts.at(0)), Ref.fromA1(parts.at(1)))
  }
}
```

Verified (2×2 range `A1:B2`): `size` → `4`; `for (r in range) { ... }` visits
`R1_1, R2_1, R1_2, R2_2` in row-major order; `.toList.size` → `4`.

**REQ-GRID-7.** `RefRange.fromTo(a, b)` normalizes corner order; `col`/`row`
bounds are `min`/`max` of the two corners on each axis independently (so a
"backwards" range like `B7:A1` is legal and equivalent to `A1:B7`).
**REQ-GRID-8.** `RefRange` extends `Iterable` and defines only
`iteratorValue` — `size` supplies the walk bound to the inherited
`Iterable#iterate`, per the same pattern `core.ph`'s own `Range` class uses.

### GAP-FIB-1, refined for this type

[00-language-findings.md §8](00-language-findings.md) and
[01-architecture.md §6](01-architecture.md) record `GAP-FIB-1`: yielding
inside a block driven by a native collection method
(`[1,2,3].each { x => Fiber.yield(x) }`) raises
`CannotYieldAcrossNativeFrame`. Writing this section required knowing exactly
what `RefRange` inherits that trap from, since `Iterable#each` (`core.ph`) is
itself written in `.ph`, not Rust:

```phalcom
each(f) {
  for (x in self) {
    f.call(x)
  }
}
```

Two additional probes, first run for this document and since **promoted into
00-language-findings.md §8 as a correction to the original finding**, narrow the
trap:

```phalcom
// SAFE — confirmed on both a native List and a user-defined RefRange:
let f = Fiber.new {
  for (x in someIterable) { Fiber.yield(x) }
  "done"
}
// runs to completion across repeated .try() calls, yielding each element.

// UNSAFE — confirmed on the SAME RefRange instance:
let f2 = Fiber.new {
  someIterable.each { x => Fiber.yield(x) }
  "done"
}
// f2.try() => Err(<CannotYieldAcrossNativeFrame>)
```

So the trap is not "iterating a collection breaks fibers" — it is
specifically **invoking a `Block` via `.call()` breaks fibers**, because
`Block#call` itself is a native (Rust-implemented) frame the VM cannot
suspend across. `Iterable#each`'s body calls `f.call(x)`, which hits it;
a bare `for (x in self) { ... }` loop, which the compiler apparently lowers
to direct `iterate`/`iteratorValue` sends with an inlined body (no `Block`
object, no `.call()`), does not. This means a `RefRange` can be safely
iterated with `for` inside a yielding fiber, but `.each { }`/`.map { }`/
`.iter.filter { }` on the same `RefRange` cannot — the restriction is per-*method*,
not per-*type*.

**This does not change the v1 scope decision.** SheetCalc v1 is still
specified fiber-free
([01-architecture.md §6](01-architecture.md)); demand-driven recalc is
deferred pending the user's call on GAP-FIB-1. But the refinement matters for
whoever picks that decision back up: it means a hand-rolled `while` is not
actually the *only* escape hatch [01-architecture.md §6](01-architecture.md)
suggests — a `for`-loop over a `RefRange`/`Range`/`List` is *also* safe inside
a yielding fiber, and only the block-taking combinators
(`each`/`map`/`filter`/`fold`/`reduce`) need to be avoided. That is a
materially smaller restriction than "hand-roll every loop as an indexed
`while`."

**This has been re-verified, and upstream is now corrected.** The original
architectural claim in `00-language-findings.md` §8, `README.md`,
`01-architecture.md` §6 and `13-language-gaps.md` was an artifact of a probe
harness that wrapped every fiber call in `{ ... }.attempt()` — itself a native
`Block#call` frame, which made even safe `for` loops fail. All four documents
now carry the corrected rule plus a post-mortem, and GAP-FIB-1's severity is
revised from High/architectural to Medium/ergonomic. This document's probe is
what caught it.

**REQ-GRID-9.** `RefRange#each`/`#map`/`#where` (all inherited from
`Iterable`) are documented as **fiber-unsafe** (same restriction as `List`);
direct `for (r in range)` iteration is fiber-safe. v1 never runs inside a
fiber, so neither path is exercised at runtime yet, but the distinction must
be preserved in comments so a future demand-driven implementation does not
have to rediscover it.

---

## 8. Test hooks

| REQ | Test |
|---|---|
| REQ-REF-1, REQ-REF-3 | `suites/ref_hash_map.ph` — `Ref`/`Ref.full` cross-lookup in `Map`/`Set`, with differing `$` flags on equal addresses |
| REQ-REF-4, REQ-REF-5 | `suites/ref_a1.ph` — encode/decode round trip through `ZZZ` (col 18278), including the `n % 26 == 0` boundary at `Z`/`AZ`/`ZZ` |
| REQ-REF-6 | `suites/ref_abs_flags.ph` — all four `$`-combinations parse and re-render identically via `.toA1` |
| REQ-REF-7, REQ-REF-8, REQ-REF-9 | `suites/ref_offset.ph` — relative axis shifts, absolute axis frozen, no mutation of the receiver, negative-offset produces an out-of-grid (not out-of-parse) `Ref` |
| REQ-GRID-1, REQ-GRID-2 | `suites/cell_mutation_discipline.ph` — `store`/`markDirty` are the only writers of `_cached`/`_dirty` (grep-based lint, same style as `REQ-VM-10`'s interpolation lint in [02-value-model.md](02-value-model.md)) |
| REQ-GRID-3, REQ-GRID-4 | `suites/grid_out_of_bounds.ph` — `valueAt` on col 0, row 0, and negative coordinates all yield `ErrorVal(#REF)`; a very large positive `Ref` (e.g. col 100000) is accepted |
| REQ-GRID-5, REQ-GRID-6 | `suites/grid_bounds.ph` — bounds track across `set` calls in non-monotonic order; `cellAt` on an unset in-bounds address returns `LiteralCell(CellEmpty)`, never `None` |
| REQ-GRID-7, REQ-GRID-8 | `suites/refrange_iterate.ph` — backwards corners normalize; `for`, `.toList`, `.size` agree on element count and order |
| REQ-GRID-9 | `suites/refrange_fiber_note.ph` — documents (not asserts, since v1 has no fiber recalc path) the `for`-safe/`.each`-unsafe split, so it is re-verified automatically if a later change introduces fiber-driven recalc |
