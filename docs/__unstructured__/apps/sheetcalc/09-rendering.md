# SheetCalc — Rendering

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md) §2, §3, §5, §6, §7;
[01-architecture.md](01-architecture.md) §2, §4; [02-value-model.md](02-value-model.md) §4-§6.

A note on quoting before any code: every string literal below is an ordinary
Phalcom double-quoted string. `'single quotes'` is **SheetCalc's own formula
grammar's** convention (GAP-STR-1) for text literals *inside a formula* — a
domain-specific mini-language whose source text happens to live inside a
Phalcom string. It has nothing to do with how Phalcom source code itself is
written. This document never touches formula text, so it uses `"` throughout,
same as every code sample in [02-value-model.md](02-value-model.md).

## 1. Grid layout

The grid renders as a box-drawn table: a header row of column labels (`A`,
`B`, ..., `Z`, `AA`, `AB`, ...), a row-number column down the left edge, and
box-drawing border characters around every cell.

```
┌────┬───────┬────────┬─────────┐
│    │   A   │   B    │    C    │
├────┼───────┼────────┼─────────┤
│  1 │    12 │ hello  │ #DIV/0! │
├────┼───────┼────────┼─────────┤
│  2 │   3.5 │ world  │       0 │
└────┴───────┴────────┴─────────┘
```

**REQ-RENDER-1.** The grid renders as a box-drawn table: column headers
(base-26 letters), row numbers, and box-drawing borders (`┌┬┐├┼┤└┴┘─│`)
around every cell.

This document assumes the following minimal read surface on `Grid`, to be
formalized by 03-references-and-grid.md (not yet written):

- `rowCount`, `colCount` — 1-based extents.
- `valueAt(row, col)` — returns a `CellValue` (02-value-model.md §2). An
  unset cell returns `CellEmpty.new()`, never a native absence marker. This
  is what lets the renderer stay polymorphic over one interface
  (`isNum`/`isText`/`isBool`/`isEmpty`/`isError`, `toString`) instead of
  special-casing "nothing here" everywhere it touches a cell.

**REQ-RENDER-2.** `Renderer.render(grid, rows, cols)` makes exactly one full
pass over the grid to compute column widths before writing any output. A
column's printed width never changes mid-render — it cannot, because by the
time the first border character is written, the border character count for
every column is already fixed (see §5: output is one-way, line by line, with
no way to go back and widen a column after the fact).

## 2. Str — hand-rolled padding, centering, repetition

`String` has no `padLeft`, `padRight`, or `repeat` (findings §5). Rendering a
fixed-width grid is impossible without them, so `support/str.ph` builds the
entire missing surface from `+`, `size`, and `while` — the only operations
`String` actually has that are relevant to this job.

```phalcom
// support/str.ph
class Str {
  // Concatenates `s` to itself `n` times. `n <= 0` returns the empty string.
  static repeat(s, n) {
    var out = ""
    var i = 0
    while (i < n) {
      out = out + s
      i = i + 1
    }
    return out
  }

  // Left-pads `s` with `pad` until it reaches `width`. A no-op if `s` is
  // already at or past `width`. `pad` should be a single character —
  // there is no char type to enforce this, so a multi-character `pad`
  // silently overshoots `width`.
  static padLeft(s, width, pad) {
    var out = s
    while (out.size < width) {
      out = pad + out
    }
    return out
  }

  // Right-pads `s` with `pad` until it reaches `width`.
  static padRight(s, width, pad) {
    var out = s
    while (out.size < width) {
      out = out + pad
    }
    return out
  }

  // Centers `s` in a field of `width`, padding with `pad` on both sides.
  // An odd remainder goes on the right. A no-op if `s` is already at or
  // past `width`.
  static center(s, width, pad) {
    let total = width - s.size
    if (total <= 0) {
      return s
    }
    // `total` is never negative past this point, so plain truncation IS
    // floor here — the negative-floor bug (findings §3) does not apply.
    var left = total / 2
    left = left - (left % 1)
    let right = total - left
    return Str.repeat(pad, left) + s + Str.repeat(pad, right)
  }
}
```

> **Commentary — this is ~45 lines that should be 0.** `padLeft`/`padRight`
> are two of the most basic string operations a display layer needs, and
> Wren (Phalcom's closest relative) ships them. None of `Str`'s code is
> spreadsheet-specific — REQ-ARCH-2 requires exactly that, so this whole
> class is the concrete proposal for what `core.ph`'s `String` should grow.
> The one piece of actual cleverness (`center`'s truncation-is-floor
> argument) exists only because `Number` has no `floor` either — the same
> gap resurfaces here from a completely different angle (see §3, GAP-NUM-3).

## 3. Number formatting — `Num.format(_)`

`(0.1 + 0.2).toString` is `"0.30000000000000004"`; `(3.0).toString` is `"3"`
(findings §3). Neither noise nor a bare `Number#toString` is acceptable in a
grid cell. `02-value-model.md` §5 specifies `Num.format(_)`: round to
`displayPrecision` decimal places, strip trailing zeros and a trailing `.`,
and render `inf`/`-inf`/`nan` as `#NUM!`. `Number` has none of `floor`,
`round`, `pow`, `isNan`, `isInfinite` (findings §3), so every one of those
steps has to be built from `+ - * / %` first.

```phalcom
// support/num.ph (display-formatting subset — floor/round/pow10 also back
// the fuller Num surface sketched in 01-architecture.md's file layout)
class Num {
  // Decimal PLACES, not significant figures — a true significant-figure
  // round needs the base-10 exponent of the magnitude, which this
  // document does not need to solve. "significant decimals" in
  // 02-value-model.md §5 is read as "decimal places" throughout.
  static displayPrecision => 10

  // NaN is the one f64 value not equal to itself (IEEE-754). This is the
  // only NaN test available — `Number` has no `isNan` (findings §3).
  static isNan(n) => n != n

  // No `isInfinite` either. `1 / 0` is verified to silently return `inf`
  // (findings §3) — there is no `inf` literal, so the sentinel is built
  // from the same expression that produces it.
  static isInf(n) => (n == (1 / 0)) or (n == (0 - (1 / 0)))

  // 10^n for a non-negative integer-valued n. No `pow` to build this
  // from, so it is a loop.
  static pow10(n) {
    var out = 1
    var i = 0
    while (i < n) {
      out = out * 10
      i = i + 1
    }
    return out
  }

  // floor(n). `n - (n % 1)` truncates toward zero, which is only floor
  // for n >= 0 (findings §3's negative-floor bug: `(-3.7 % 1)` is
  // `-0.7...`, so truncation overshoots toward zero by one for a
  // negative, non-integer n). The correction applies in that case only.
  static floor(n) {
    let truncated = n - (n % 1)
    if ((n < 0) and (truncated != n)) {
      return truncated - 1
    }
    return truncated
  }

  // Rounds `n` to `decimals` decimal places, half-away-from-zero. Works
  // in magnitude (always >= 0) so `floor` above never has to handle a
  // negative input here, then reapplies the sign.
  static round(n, decimals) {
    let scale = Num.pow10(decimals)
    let shifted = n * scale
    let negative = shifted < 0
    let magnitude = negative.ifTrue({ 0 - shifted }, ifFalse: { shifted })
    let bumped = Num.floor(magnitude + 0.5)
    let signed = negative.ifTrue({ 0 - bumped }, ifFalse: { bumped })
    return signed / scale
  }

  // Strips trailing "0"s and a trailing "." from a formatted number
  // string. A no-op if `s` has no "." — an integral value (findings §3:
  // `(3.0).toString == "3"`) never has one. Indexes a literal "0"/"."
  // via `codePointAt` rather than a hardcoded byte value, and walks the
  // string with `rawByteAt`/`rawSlice` since `String` has no `at(_)`
  // (findings §5).
  static stripTrailingZeros(s) {
    if (s.indexOf(".") < 0) {
      return s
    }
    let zero = "0".codePointAt(0)
    let dot = ".".codePointAt(0)
    var out = s
    while (out.rawByteAt(out.size - 1) == zero) {
      out = out.rawSlice(0, out.size - 1)
    }
    if (out.rawByteAt(out.size - 1) == dot) {
      out = out.rawSlice(0, out.size - 1)
    }
    return out
  }

  // The grid-facing entry point (REQ-VM-7/8). Defends against any
  // inf/nan that slipped past a `#DIV0` zero-guard (02-value-model.md §4)
  // by rendering it as `#NUM!` instead of propagating the raw float.
  static format(n) {
    if (Num.isNan(n) or Num.isInf(n)) {
      return "#NUM!"
    }
    let rounded = Num.round(n, Num.displayPrecision)
    return Num.stripTrailingZeros(rounded.toString)
  }
}
```

**REQ-RENDER-5.** Every cell's numeric display goes through `Num.format(_)`
exclusively — no cell ever prints a raw `Number#toString`. `inf`, `-inf`, and
`nan` always render as `#NUM!` (restates REQ-VM-8 at the render layer).

> **Commentary — the round-trip trick mostly cleans itself, and that is not
> good enough.** Dividing `bumped` back by `scale` in `Num.round` usually
> lands exactly on the nearest representable double for the rounded decimal
> value — IEEE-754 division is correctly rounded, so `3000000000 /
> 10000000000` and the literal `0.3` typically produce the identical bit
> pattern, and `toString` prints `"0.3"` with no noise at all, no strip
> needed. But "typically" is an empirical observation about this runtime's
> float formatter, not a proof, and 02-value-model.md §5 specifies the strip
> step explicitly (REQ-VM-7) as the actual contract. `stripTrailingZeros`
> stays in as the real guarantee; the round-trip's cleanliness is a bonus,
> not the design.

## 4. Alignment

**REQ-RENDER-4.** Numbers align right, errors align center, everything else
(text, bool, empty) aligns left. This dispatches off the `CellValue` root's
own classification getters (`isNum`, `isError` — 02-value-model.md §2), not a
new field added to the value hierarchy:

```phalcom
// Part of render/renderer.ph — see §6.
static alignFor_(cell) {
  if (cell.isNum) {
    return "right"
  }
  if (cell.isError) {
    return "center"
  }
  return "left"
}

static alignedCell_(text, width, align) {
  if (align == "right") {
    return Str.padLeft(text, width, " ")
  }
  if (align == "center") {
    return Str.center(text, width, " ")
  }
  return Str.padRight(text, width, " ")
}
```

`CellBool` is treated as "everything else" (left) rather than given its own
right/center rule — the task only specifies numbers/text/errors, and there is
no forcing reason to special-case booleans in v1.

## 5. No `\n` escape — output is line by line, not string-then-print

The only string escapes are `\\` and `\(...)` (findings §5). There is no
`\n`. A newline is obtainable only as `System.print(_)`'s implicit trailing
newline; `System.rawWrite(_)` writes without one. **There is no way to build
a multi-line grid as one string and print it once** — `"line1" + "line2"`
concatenates them with nothing between, and there is no escape to insert a
line break into that string afterward.

**REQ-RENDER-7.** All grid output is written line by line: every partial
segment of a line uses `System.rawWrite(_)`, and each line is terminated by
exactly one `System.print(_)` call carrying that line's final segment. A
border line, a header line, and every data row are each built this way — see
`border_`, `headerRow_`, `dataRow_` in §6.

> **Commentary — this is the constraint that shapes every function in §6.**
> In a language with `\n`, `Renderer.render` could build one big string and
> call `System.print` once. Here, every one of `border_`/`headerRow_`/
> `dataRow_` is a sequence of `rawWrite` calls ended by one `print`, and
> getting the sequencing wrong doesn't crash — it just prints a ragged or
> misaligned grid, silently, with no diagnostic. The golden diff (§10 of the
> testing document) is the *only* thing that catches a dropped `rawWrite` or
> a `print` called one line early. This is a genuine ergonomic cost for
> something as basic as "print a table," and it applies to any Phalcom
> program that renders more than one line of structured output.

## 6. The renderer

```phalcom
// render/renderer.ph
import "../support/str.ph" as StrLib
import "../support/num.ph" as NumLib

// De-stutters the mandatory whole-module qualification (modules.md §3,
// GAP-MOD-1): `StrLib.Str` and `NumLib.Num` would otherwise read as
// `Str.Str`/`Num.Num` at every call site, since each support module
// exports exactly one class named after the module itself. A module-level
// `let` binding is an ordinary top-level member (modules.md §1) — this
// costs one line per import and buys back every call site below.
let Str = StrLib.Str
let Num = NumLib.Num

class Renderer {
  static alphabet_ => "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

  // 1-based column index -> spreadsheet column label (A, B, ..., Z, AA,
  // AB, ...). Built by indexing a literal alphabet string, NOT by
  // codepoint arithmetic: `String.new(65)` stringifies the number to
  // `"65"`, not `"A"` (findings §5) — there is no char-from-codepoint
  // constructor, so a character can only ever come from a literal already
  // in the source text. `rawSlice` on a 1-byte-per-char ASCII literal
  // doubles as a "character at index" operation.
  static columnLabel(n) {
    var out = ""
    var i = n
    while (i > 0) {
      let rem = (i - 1) % 26
      out = Renderer.alphabet_.rawSlice(rem, rem + 1) + out
      i = Num.floor((i - 1) / 26)
    }
    return out
  }

  static alignFor_(cell) {
    if (cell.isNum) {
      return "right"
    }
    if (cell.isError) {
      return "center"
    }
    return "left"
  }

  static alignedCell_(text, width, align) {
    if (align == "right") {
      return Str.padLeft(text, width, " ")
    }
    if (align == "center") {
      return Str.center(text, width, " ")
    }
    return Str.padRight(text, width, " ")
  }

  // Width of each column: the wider of its header label and every
  // rendered cell in it (REQ-RENDER-3). A full pass over the grid before
  // any output is written — see REQ-RENDER-2. Measures only ASCII content
  // (formatted numbers, letters, `Num.format`'s output), so `.size`'s
  // exact code-unit semantics don't matter here.
  static columnWidths_(grid, rows, cols) {
    var widths = List.new()
    var c = 1
    while (c <= cols) {
      var w = Renderer.columnLabel(c).size
      var r = 1
      while (r <= rows) {
        let rendered = grid.valueAt(r, c).toString    // NEVER "\(...)" — BUG-TOSTR-1
        if (rendered.size > w) {
          w = rendered.size
        }
        r = r + 1
      }
      widths.add(w)
      c = c + 1
    }
    return widths
  }

  // One border line: `left`/`mid`/`right` are single box-drawing
  // characters, `fill` is the horizontal rule. Each column reserves +2
  // for the padding space on either side of its content.
  static border_(rowLabelWidth, widths, left, mid, right, fill) {
    System.rawWrite(left)
    System.rawWrite(Str.repeat(fill, rowLabelWidth + 2))
    var c = 0
    while (c < widths.size) {
      System.rawWrite(mid)
      System.rawWrite(Str.repeat(fill, widths.at(c) + 2))
      c = c + 1
    }
    System.print(right)
  }

  static headerRow_(rowLabelWidth, widths, cols) {
    System.rawWrite("│")
    System.rawWrite(Str.repeat(" ", rowLabelWidth + 2))
    var c = 1
    while (c <= cols) {
      System.rawWrite("│")
      System.rawWrite(" ")
      System.rawWrite(Str.center(Renderer.columnLabel(c), widths.at(c - 1), " "))
      System.rawWrite(" ")
      c = c + 1
    }
    System.print("│")
  }

  static dataRow_(grid, r, rowLabelWidth, widths, cols) {
    System.rawWrite("│")
    System.rawWrite(" ")
    System.rawWrite(Str.padLeft(r.toString, rowLabelWidth, " "))
    System.rawWrite(" ")
    var c = 1
    while (c <= cols) {
      let cell = grid.valueAt(r, c)
      let rendered = cell.toString                 // NEVER "\(cell)" — BUG-TOSTR-1
      System.rawWrite("│")
      System.rawWrite(" ")
      System.rawWrite(Renderer.alignedCell_(rendered, widths.at(c - 1), Renderer.alignFor_(cell)))
      System.rawWrite(" ")
      c = c + 1
    }
    System.print("│")
  }

  // The entry point. Prints the whole grid line by line (§5) — there is
  // no way to build it as one string and print it once.
  static render(grid, rows, cols) {
    let widths = Renderer.columnWidths_(grid, rows, cols)
    let rowLabelWidth = rows.toString.size

    Renderer.border_(rowLabelWidth, widths, "┌", "┬", "┐", "─")
    Renderer.headerRow_(rowLabelWidth, widths, cols)
    Renderer.border_(rowLabelWidth, widths, "├", "┼", "┤", "─")

    var r = 1
    while (r <= rows) {
      Renderer.dataRow_(grid, r, rowLabelWidth, widths, cols)
      if (r < rows) {
        Renderer.border_(rowLabelWidth, widths, "├", "┼", "┤", "─")
      }
      r = r + 1
    }

    Renderer.border_(rowLabelWidth, widths, "└", "┴", "┘", "─")
  }
}
```

**REQ-RENDER-6.** No render-layer code interpolates a `CellValue`, `Ref`, or
`Cell` with `\(...)` (restates REQ-VM-9 at the render layer). Every render
site above uses an explicit `.toString` send, string literals, or plain
`Number#toString` (via `r.toString` and `rows.toString`, both of which are
native `Number`s, not `CellValue`s — so BUG-TOSTR-1 does not apply to them at
all, since the bug is specifically about *user instances*, not native types).

## 7. Requirements and test hooks

| REQ | Statement | Test |
|---|---|---|
| REQ-RENDER-1 | Box-drawn table: headers, row numbers, borders. | `suites/render_borders.ph` |
| REQ-RENDER-2 | One full width-computation pass before any output. | `suites/render_widths.ph` |
| REQ-RENDER-3 | Column width = max(header label, every cell in that column). | `suites/render_widths.ph` |
| REQ-RENDER-4 | Alignment: numbers right, errors center, else left. | `suites/render_alignment.ph` |
| REQ-RENDER-5 | All numeric display goes through `Num.format(_)`; inf/nan -> `#NUM!`. | `suites/render_format.ph` (shares fixtures with REQ-VM-7/8) |
| REQ-RENDER-6 | No `\(...)` on a `CellValue`/`Ref`/`Cell` in `render/`. | `suites/lint_interpolation.ph` (external — see [10-testing.md §5](10-testing.md)) |
| REQ-RENDER-7 | Output is `rawWrite`-segments-then-one-`print`, line by line. | Covered structurally by every golden fixture's exact-diff; no separate unit test — a sequencing bug here shows up as a misaligned or ragged golden diff, not a crash |

See [10-testing.md](10-testing.md) for how the golden corpus and the
interpolation lint actually run.
