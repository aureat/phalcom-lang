# SheetCalc — Value Model

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md) §3, §4, §6.

## 1. The forcing constraint

A spreadsheet must satisfy this, or it is not a spreadsheet:

```
=A1/0     ->  #DIV/0!
=A2+1     ->  #DIV/0!      (where A2 holds #DIV/0!)
=SUM(A1:A5) -> #DIV/0!     (if any cell in the range is an error)
```

Errors are **values that propagate through arithmetic**. That single sentence
determines the entire value model, because of this probe result (findings §4):

```phalcom
class Err2 { toString => "#DIV/0!" }
{ 1 + Err2.new() }.attempt()   // => Err(<Error>)   -- Number#+ raises
```

`Number#+` is a native primitive. It type-checks and raises. Phalcom has:

- no multimethods,
- no coercion protocol (`coerce:`, `__radd__`, or equivalent),
- no way to override a primitive from `.ph`.

So when a native `Number` is the **receiver**, the operation is out of our
hands. There is no arrangement of user code that makes `1 + errorValue` return
an error value.

### DEC-VM-1 — every cell value is a user-class instance

**Decision.** No `CellValue` is ever a bare native `Number`, `String`, or
`Bool`. Every value in the grid — including the number `2` — is an instance of
a `CellValue` subclass wrapping a native payload.

**Forcing reason.** It is the only way to guarantee the receiver of every
arithmetic send is a class we control. If `CellNum` wraps the `2`, then
`CellNum(2) + ErrorVal(#DIV/0!)` dispatches to **our** `CellNum#+`, which can
inspect the argument and return the error. If the `2` were native, the send
dispatches to the primitive and raises.

**Cost, stated honestly.** One heap allocation per number, per arithmetic step.
`SUM(A1:A1000)` allocates ~1000 intermediate `CellNum`s. Given the measured
baseline (allocation is the #1 cost mechanism in this runtime), this is the
single largest performance decision in the program — and it was **not chosen for
performance reasons or for design elegance. It was forced by a five-line probe.**

**Alternatives considered and rejected:**

| Alternative | Why rejected |
|---|---|
| Native numbers, errors as a separate channel (`Result`) | `#DIV/0!` stops being a value. `ISERROR(A1)`, `IFERROR`, and "the cell displays `#DIV/0!`" all break. Category error. |
| Native numbers, check types before every op | Every `+` site becomes a 4-branch dispatch in the evaluator. Pushes the whole value model into the evaluator, and `SUM` still can't use `reduce`. Strictly worse and still needs a wrapper for the error case. |
| Sentinel `NaN` for errors | `f64` NaN propagates through `+` for free — genuinely tempting. But there is one NaN, and a spreadsheet needs *distinct* errors (`#DIV/0!` vs `#REF!` vs `#VALUE!`), plus `Number` has no `isNan` (findings §3) so you couldn't even detect it. Dead end. |
| Make `Number#+` dispatch to the argument on type mismatch | The right fix, but it is a **runtime change**, not something this program can do. Filed as [GAP-NUM-2](13-language-gaps.md). |

> **Commentary.** This is the exercise working exactly as intended. A language
> limitation that is invisible in `showcase.ph` — which only ever adds numbers
> to numbers — dictates the memory strategy of a real program. Wren, for
> comparison, has the same limitation and the same consequence. Smalltalk
> solves it with `retryRelationalOp:coercing:` (double dispatch on primitive
> failure); Python with `__radd__`; Ruby with `coerce`. Phalcom has no
> equivalent, and it is a real hole (see GAP-NUM-2).

### DEC-VM-1 is verified, not just designed

The hierarchy below was executed against `main` @ `5516504` before this document
was finalized. Result:

```
2 + 3      = 5
2 / 0      = #DIV/0!      <- explicit zero-guard, not inf
err + 2    = #DIV/0!      <- left absorption  (REQ-VM-3)
2 + err    = #DIV/0!      <- right absorption (REQ-VM-4)
(2/0) + 1  = #DIV/0!      <- propagation through a chain
```

Boxing every value in a user class does deliver the propagation semantics a
spreadsheet needs, and the operator-overload dispatch is clean. The design
works. It just costs an allocation per step, for a reason that is not a design
preference.

## 2. Hierarchy

```
CellValue                       (abstract root)
├── CellNum      wraps f64
├── CellText     wraps String
├── CellBool     wraps Bool
├── CellEmpty    singleton — an unset cell
└── ErrorVal     wraps an ErrorKind symbol
```

**REQ-VM-1.** Every value stored in a `Cell` or returned by `Ast#eval(_)` is a
`CellValue`.
**REQ-VM-2.** No `CellValue` subclass may be instantiated with another
`CellValue` as its payload. Payloads are native.

### `CellValue` — the root protocol

```phalcom
class CellValue {
  // --- classification (default false; each subclass overrides its own) ---
  isNum   => false
  isText  => false
  isBool  => false
  isEmpty => false
  isError => false

  // --- payload access ---
  // NOTE: an earlier draft wrote `ErrorVal.of(#VALUE).raise_` here. That was
  // wrong twice over, and a reviewer's probe caught it:
  //   1. The selector is `raise()` (a 0-arity METHOD), not `raise_`.
  //   2. `raise` is installed only on `Error` and its subclasses. `ErrorVal`
  //      extends `CellValue`, so the send would have been
  //      MessageNotUnderstood, not the intended error.
  // Returning the error VALUE is the correct move anyway (REQ-VM-1): a bad
  // payload access is spreadsheet data, not a program fault.
  num  { return ErrorVal.of(#VALUE) }
  text { return ErrorVal.of(#VALUE) }

  // --- arithmetic: every subclass implements all of these ---
  +(o)  { return ErrorVal.of(#VALUE) }
  -(o)  { return ErrorVal.of(#VALUE) }
  *(o)  { return ErrorVal.of(#VALUE) }
  /(o)  { return ErrorVal.of(#VALUE) }
  %(o)  { return ErrorVal.of(#VALUE) }
  negated => ErrorVal.of(#VALUE)

  // --- comparison ---
  ==(o) { return false }
  <(o)  { return ErrorVal.of(#VALUE) }

  // --- rendering (NEVER via interpolation — see BUG-TOSTR-1) ---
  toString => "?"
  display  => self.toString          // what the grid renders
}
```

> **Commentary — the default-to-`#VALUE!` root.** Defining every operator on
> the root with a `#VALUE!` default means a nonsense operation like
> `CellText('a') * CellBool(true)` returns `#VALUE!` instead of
> `MessageNotUnderstood`. That is correct spreadsheet behavior *and* it collapses
> a 5×5 type-pair matrix into "override the pairs that are meaningful."
> Without it, every subclass would need a full set of type checks. This is the
> single biggest boilerplate saving in the value layer, and it works because
> Phalcom's inheritance is ordinary and reliable (`extends`/`super` are solid —
> see [12-design-patterns.md](12-design-patterns.md)).

## 3. `ErrorVal` — the propagation engine

```phalcom
class ErrorVal extends CellValue {
  construct of(kind) { _kind = kind }        // #DIV0, #VALUE, #REF, #NAME, #CIRC, #NA

  kind    => _kind
  isError => true

  // THE propagation rule: an error absorbs every operation and returns itself.
  +(o) => self
  -(o) => self
  *(o) => self
  /(o) => self
  %(o) => self
  negated => self
  <(o)  => self

  ==(o) => o.isError and o.kind == _kind

  toString {
    // No Map literal in the language; built once, statically.
    return ErrorVal.names_.at(_kind).unwrapOr("#ERR!")
  }
}
```

**REQ-VM-3 (left absorption).** For any `CellValue` `v` and any operator `op`
in `+ - * / %`, `ErrorVal(k) op v == ErrorVal(k)`.
**REQ-VM-4 (right absorption).** For any **non-error** `CellValue` `v`,
`v op ErrorVal(k) == ErrorVal(k)`. Enforced in each subclass's operators, not
inherited — see §4.
**REQ-VM-5 (first-error-wins).** `ErrorVal(a) op ErrorVal(b) == ErrorVal(a)`.
Matches Excel; makes propagation associative and deterministic.

Error kinds:

| Kind | Rendered | Raised when |
|---|---|---|
| `#DIV0` | `#DIV/0!` | division or modulo by zero |
| `#VALUE` | `#VALUE!` | type mismatch (`'a' * 2`) |
| `#REF` | `#REF!` | reference to an out-of-bounds cell |
| `#NAME` | `#NAME?` | unknown function name |
| `#CIRC` | `#CIRC!` | cell participates in a dependency cycle |
| `#NA` | `#N/A` | lookup found nothing |

## 4. `CellNum` — where the real work is

```phalcom
class CellNum extends CellValue {
  construct of(n) { _n = n }

  num   => _n
  isNum => true

  +(o) {
    // Right-absorption (REQ-VM-4) must be checked FIRST in every operator.
    if (o.isError) { return o }
    if (o.isNum)   { return CellNum.of(_n + o.num) }
    if (o.isEmpty) { return CellNum.of(_n) }        // empty coerces to 0
    return ErrorVal.of(#VALUE)
  }

  /(o) {
    if (o.isError) { return o }
    if (o.isEmpty) { return ErrorVal.of(#DIV0) }
    if (o.isNum) {
      // CRITICAL: Phalcom's 1/0 returns `inf`, NOT an error (findings §3).
      // Without this explicit guard, `inf` propagates silently through the
      // entire sheet and renders as "inf". There is no runtime help here.
      if (o.num == 0) { return ErrorVal.of(#DIV0) }
      return CellNum.of(_n / o.num)
    }
    return ErrorVal.of(#VALUE)
  }

  toString => Num.format(_n)     // NOT _n.toString — see §5
}
```

**REQ-VM-6 (zero guard).** Every `/` and `%` implementation must test its
divisor against zero *before* dividing, and return `ErrorVal(#DIV0)`.

> **Commentary — the silent `inf`.** `1 / 0` returning `Ok(inf)` (findings §3)
> is the most dangerous finding for this program, precisely because it is
> silent. Every other trap announces itself with a diagnostic;
> this one hands you a plausible-looking value that poisons every downstream
> cell and renders as `inf` in the grid. A test that checked only "does it run"
> would pass. IEEE-754 says `1.0/0.0` is `inf`, so the runtime is *defensible* —
> but a language with no `isInf`, no `isNan`, and no integer type gives the user
> no way to detect it after the fact. See GAP-NUM-1.

## 5. Number formatting — `Num.format(_)`

`(0.1 + 0.2).toString` is `"0.30000000000000004"` and `(3.0).toString` is
`"3"` (findings §3). Neither is acceptable in a grid cell, and `Number` has no
`round`.

`Num.format(_)` (in `support/num.ph`) must therefore:

1. Round to `Num.displayPrecision` (default 10 significant decimals) using a
   hand-rolled `Num.round(_, _)` built from `%` and `*`/`/` by powers of ten.
2. Strip trailing zeros and a trailing `.`.
3. Render `inf`/`-inf`/`nan` as `#NUM!` — a defensive net for any zero-guard
   that slips through REQ-VM-6.

**REQ-VM-7.** `Num.format(0.1 + 0.2)` renders `"0.3"`.
**REQ-VM-8.** `Num.format(_)` never emits `inf`, `-inf`, or `nan`.

> **Commentary — `Num.round` is 30 lines that should be 0.** Building `round`
> from `%` requires: a power-of-ten scale (no `pow` — hand-rolled loop), a
> `floor` (no `floor` — and `n - (n % 1)` is **wrong for negatives**, it
> truncates toward zero rather than flooring, so it needs a sign correction),
> and a half-away-from-zero rule. Every step of this is a place to introduce a
> bug, in service of a function every other language ships. `Number`'s empty
> method surface is the highest-frequency papercut in the exercise. See
> GAP-NUM-3.

## 6. Rendering — the `toString` trap

Per BUG-TOSTR-1 (findings §6), string interpolation **does not send `toString`**
to user instances:

```phalcom
System.print("\(cellValue)")            // => <CellNum instance>   WRONG
System.print("" + cellValue.toString)   // => 42                   CORRECT
```

**REQ-VM-9.** No SheetCalc source may interpolate a `CellValue`, a `Ref`, a
`Cell`, an `Ast` node, or a `Token`. All rendering uses explicit `.toString`
concatenation.
**REQ-VM-10.** A source lint fails the suite if `\(` is applied to a known
domain-typed local.

> **This lint cannot be written in Phalcom.** A `.ph` program cannot read a
> file, including its own sources (findings §2), so the lint must be an
> external check (a `grep` in the test lane), not a module inside SheetCalc.
> An earlier draft of this requirement said "`test/framework.ph` includes a
> source lint", which contradicted findings §2 — caught in review. It is a small
> thing, but it is the second time in this spec that "no file I/O" turned out to
> have a consequence I did not think through at the time
> ([GAP-IO-1](13-language-gaps.md)). The absence of I/O is not just a missing
> convenience; it silently removes whole categories of solution from
> consideration, and you notice one at a time.

> **Commentary.** A whole-program prohibition on the language's most idiomatic
> string construct, enforced by a grep, is an absurd thing for a spec to
> contain. It is here because the alternative is silently wrong output in every
> cell. This is the finding I would fix first: interpolation should route
> through the same `to_display_string` path `System.print` already uses.

## 7. Truthiness and comparison

Phalcom has no implicit truthiness (`if` demands a `Bool`), which is a genuine
help here — spreadsheet coercion rules are explicit and testable rather than
inherited from the host language.

**REQ-VM-11.** Ordering for mixed types follows Excel: numbers < text < bool.
**REQ-VM-12.** `CellEmpty` compares equal to `CellNum(0)` and to `CellText('')`
in `==`, but renders as the empty string. (This is Excel's rule, and it is
genuinely strange; it is specified so the golden tests pin it deliberately
rather than by accident.)

## 8. Test hooks

| REQ | Test |
|---|---|
| REQ-VM-3/4/5 | `suites/value_propagation.ph` — full 6×6 operator × error-kind matrix |
| REQ-VM-6 | `suites/value_divzero.ph` — `/0`, `%0`, and `inf` never reaching the grid |
| REQ-VM-7/8 | `suites/value_format.ph` — `0.1+0.2`, `3.0`, `inf`, big/small magnitudes |
| REQ-VM-9 | `suites/lint_interpolation.ph` — source lint |
| REQ-VM-11/12 | `suites/value_compare.ph` |
