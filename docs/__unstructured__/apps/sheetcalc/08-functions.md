# SheetCalc — Function Library

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md) §3, §4, §9, and
[06-ast-and-eval.md](06-ast-and-eval.md) §5.

**Note on scope.** This document adds one `ErrorVal` kind beyond
[02-value-model.md](02-value-model.md) §3's table: `#NUM` (rendered `#NUM!`),
for math-domain failures (`SQRT` of a negative number) that have no home in
02's original six kinds (`#DIV0 #VALUE #REF #NAME #CIRC #NA`). This is an
addition to 02's kind table, not a contradiction of it.

## 1. The dispatch mechanism

`Call#eval` (06-ast-and-eval.md §5) hands `FunctionTable#invoke` a function
name, a `List<Ast>` of unevaluated argument nodes, and an `EvalContext`. Three
ways to map the name to behavior were considered.

### (a) `Map<String, Block>`

```phalcom
_table.at("ABS", put: { argNodes, ctx => ... })
```

Works syntactically — blocks are first-class values and `Map#at` is `O(1)`-ish.
The problem is everything a function needs *besides* its body: arity bounds,
and whether it participates in default error propagation (§4). A bare block
has no attributes to hang that metadata on, so it ends up living in a parallel
`Map<String, Int>` for min-arity, another for max-arity, another for the
propagation flag — four maps kept in sync by hand, one per function property.

### (b) A class per function, with a common polymorphic `call`

```phalcom
class AbsFn is Fn {
  minArity => 1
  maxArity => 1
  call(argNodes, ctx) { ... }
}
```

Each function is a small object that can carry exactly the metadata it needs
as ordinary overridable methods/getters, dispatched the same polymorphic way
`Ast#eval` is (06 §2). `FunctionTable` still needs *some* way to go from the
name `"ABS"` to the `AbsFn` instance — which brings back the Map, just one
level up: `Map<String, Fn-instance>` instead of `Map<String, Block>`.

### (c) `Object#perform` + a selector built from the name string

The tempting shortcut: skip the `Map` and every per-function class, and just
`perform` a method named after the function directly —

```phalcom
class Functions {
  sum(args) => ...
  abs(args) => ...
}
Functions.new().perform(Symbol.new(name.toLowerCase), [args])
```

**This does not work, and it is worth showing exactly why, since it looked
plausible enough to nearly ship.** Phalcom encodes a method's selector with
its arity and kind baked into the interned string
(`phalcom-core/src/method/mod.rs::encode_selector`): a zero-arg method named
`sum` interns as `"sum()"`, a two-arg method interns as `"sum(_,_)"`, and a
bare getter interns as just `"sum"`. `Symbol.new(_)` does **not** reproduce
this — it interns whatever string you hand it, verbatim, with no arity suffix
added. Verified directly:

```phalcom
class Fn { sum(a, b) => a + b }
let f = Fn.new()

f.respondsTo(Symbol.new("sum"))                          // => false
{ f.perform(Symbol.new("sum"), [1, 2]) }.attempt()        // => Err(<MessageNotUnderstood>)

f.respondsTo(Symbol.new("sum(_,_)"))                      // => true
{ f.perform(Symbol.new("sum(_,_)"), [1, 2]) }.attempt()   // => Ok(3)
```

`Symbol.new("sum")` builds a symbol that only matches a **getter** named
`sum`; it never matches the two-arg method, no matter what you pass in the
args list to `perform`. To make (c) work at all, you would have to hand-encode
the exact arity-suffixed selector text yourself — `"sum(_,_)"`, not `"sum"` —
which means knowing each function's arity *at the call site*, which is
precisely the information a name-only dispatch was supposed to let you avoid
knowing. And every function still needs to be a literal, individually spelled
method on some class, so (c) buys nothing over (b) except a strictly worse
failure mode: a typo'd arity produces a runtime `MessageNotUnderstood` instead
of a compile-time-checkable table lookup.

**Recommendation: (b), a `Map<String, Fn-instance>`.** It gets the metadata
benefits of a class per function without paying for a fragile dynamically-built
selector. This was not a hedge — it is confirmed by the probe above, which is
new evidence beyond what 00-language-findings.md's probe log currently
records; that log should gain this exact result.

```phalcom
class Fn {
  minArity => 0
  maxArity => 0             // 0 means "unbounded above minArity"
  propagatesErrors => true  // see §4

  // Never actually invoked -- every concrete subclass overrides this. Same
  // root-default idiom as CellValue (02 §2) and Ast (06 §2).
  call(argNodes, ctx) => ErrorVal.of(#VALUE)
}

class FunctionTable {
  @constructor
  new() {
    _table = Map.new()
    _table.at("SUM", put: SumFn.new())
    _table.at("AVERAGE", put: AverageFn.new())
    _table.at("MIN", put: MinFn.new())
    _table.at("MAX", put: MaxFn.new())
    _table.at("COUNT", put: CountFn.new())
    _table.at("COUNTA", put: CountaFn.new())
    _table.at("IF", put: IfFn.new())
    _table.at("AND", put: AndFn.new())
    _table.at("OR", put: OrFn.new())
    _table.at("NOT", put: NotFn.new())
    _table.at("ABS", put: AbsFn.new())
    _table.at("ROUND", put: RoundFn.new())
    _table.at("SQRT", put: SqrtFn.new())
    _table.at("CONCAT", put: ConcatFn.new())
    _table.at("LEN", put: LenFn.new())
    _table.at("LEFT", put: LeftFn.new())
    _table.at("RIGHT", put: RightFn.new())
    _table.at("MID", put: MidFn.new())
    _table.at("UPPER", put: UpperFn.new())
    _table.at("LOWER", put: LowerFn.new())
    _table.at("ISERROR", put: IsErrorFn.new())
    _table.at("IFERROR", put: IfErrorFn.new())
    _table.at("VLOOKUP", put: VlookupFn.new())
    _table.at("COUNTIF", put: CountIfFn.new())
  }

  invoke(name, argNodes, ctx) {
    let fn = _table.at(name)
    (fn == None).ifTrue { return ErrorVal.of(#NAME) }
    let arity = argNodes.size
    (arity < fn.minArity).ifTrue { return ErrorVal.of(#VALUE) }
    (fn.maxArity > 0 and arity > fn.maxArity).ifTrue { return ErrorVal.of(#VALUE) }
    return fn.call(argNodes, ctx)
  }
}
```

`_table.at(name)` returning the raw `Fn` instance on a hit and the `None`
singleton on a miss (rather than an `Option`-wrapped hit) is `Map`'s own
documented contract (core.ph: "Total lookup: ... raw value on hit, the `None`
singleton on miss") — `(fn == None)` is the correct, sufficient guard.

**REQ-FN-1.** `FunctionTable` maps an exact, case-sensitive formula name
(`"SUM"`, not `"sum"`) to one `Fn` instance. An unrecognized name returns
`ErrorVal(#NAME)`.
**REQ-FN-2.** Every `Fn` declares `minArity`/`maxArity`; `invoke` checks arity
before calling `call`, returning `ErrorVal(#VALUE)` on mismatch — never a
Phalcom-level `RuntimeError::Arity`.

> **Commentary — this is a case where reading the runtime, not just probing
> it, was necessary.** `.ph`-level probing (00-language-findings.md's method)
> answers "does this call work"; it does not by itself explain *why* `perform`
> plus a name-derived symbol fails in a way that generalizes. The generalizing
> fact — selectors are arity/kind-encoded strings, not bare names — lives in
> `phalcom-core/src/method/mod.rs::encode_selector` and is confirmed, not just
> inferred, by the four-line probe above. Both matter: the source explains the
> mechanism, the probe confirms it actually behaves that way at the surface
> `.ph` programs see.

## 2. Function set (v1)

`SUM AVERAGE MIN MAX COUNT COUNTA IF AND OR NOT ABS ROUND SQRT CONCAT LEN LEFT
RIGHT MID UPPER LOWER ISERROR IFERROR VLOOKUP COUNTIF` — 24 functions.

### 2.1 `SUM` — where DEC-VM-1's cost becomes visible

```phalcom
class SumFn is Fn {
  minArity => 1

  call(argNodes, ctx) {
    var total = CellNum.of(0)
    for (node in argNodes) {
      for (v in node.evalRange(ctx)) {
        total = total + v
        (total.isError).ifTrue { return total }
      }
    }
    return total
  }
}
```

No `isNum`/`isError`/`isEmpty` check appears in this method. `CellNum#+`
(02-value-model.md §4) already handles all three: an error argument absorbs
(`isError -> return o`), a number accumulates, `CellEmpty` coerces to zero,
and anything else — text, bool — falls through to `ErrorVal(#VALUE)`. Once
`total` itself becomes an `ErrorVal`, `ErrorVal#+` (02 §3, `+(o) => self`)
makes every subsequent `+` a no-op that returns the same error, so the early
`return` above is purely an optimization, not a correctness requirement.

> **Commentary — this loop is 02 §1's allocation cost, made concrete.**
> 02-value-model.md §1 predicted `SUM(A1:A1000)` allocates roughly 1000
> intermediate `CellNum`s and called it "the single largest performance
> decision in the program," forced by a five-line probe rather than chosen.
> `SumFn#call` above is that prediction with a line number: `total = total + v`
> allocates a new `CellNum` every iteration, and there is no way to avoid it
> without abandoning `CellNum#+`'s free type-checking — the alternative is
> accumulating into a native `var total = 0` and re-deriving `isError`/`isNum`/
> `isEmpty` handling by hand inside `SumFn` itself, which is exactly the
> "4-branch dispatch in the evaluator" 02 §1 already rejected as an
> alternative to DEC-VM-1. The cost is real and it is not local to this one
> function — every range-consuming function below pays a version of it.

### 2.2 `AVERAGE`, `MIN`, `MAX`, `COUNT`, `COUNTA` — where the elegant trick stops working

`SUM` gets to lean on `CellNum#+` because `+` conveniently returns a usable
`CellValue` to keep accumulating with. `MIN`/`MAX` have no equivalent to lean
on, and the reason is a real asymmetry in the value model worth stating
plainly.

> **Commentary — `CellValue` comparisons return `CellValue`, not `Bool`, and
> that is correct but inconvenient.** `CellNum#<(o)` must return a `CellValue`
> (a `CellBool`, or an `ErrorVal` on a type mismatch) rather than a native
> `Bool`, because REQ-VM-1 requires every value an evaluator produces to be a
> `CellValue` — `=A1<A2` is a formula-visible comparison and its result must be
> able to hold `#VALUE!` just like `+` can hold `#DIV/0!`. That is the right
> design for a *formula's own* `<`. It is the wrong tool the moment a
> function's *internal* logic wants to branch on a comparison: `(a < b)` where
> `a`/`b` are `CellValue`s returns a `CellBool` instance, and `CellBool` is not
> a native `Bool` — it cannot be handed to `if`/`ifTrue` without first
> unwrapping it (mirroring the unwrap-before-branch note in
> 02-value-model.md §6 about `.bool`/`.num`/`.text` accessors always requiring
> a type guard first). `MinFn`/`MaxFn` below sidestep this entirely by
> comparing the **unwrapped native `.num` payloads** — which is safe precisely
> because both operands have already been guarded with `isNum` by that point —
> rather than routing through `CellNum`'s own `<` operator at all.

```phalcom
class MinFn is Fn {
  minArity => 1

  call(argNodes, ctx) {
    var best = None
    for (node in argNodes) {
      for (v in node.evalRange(ctx)) {
        (v.isError).ifTrue { return v }
        (v.isEmpty).ifFalse {
          (v.isNum).ifFalse { return ErrorVal.of(#VALUE) }
          (best == None or v.num < best.num).ifTrue { best = v }
        }
      }
    }
    return (best == None).ifTrue({ ErrorVal.of(#NA) }, ifFalse: { best })
  }
}
```

`MaxFn` is the mirror image (`v.num > best.num`). `AverageFn` tracks a running
`total`/`n` pair over native `.num` values and divides once at the end,
guarding the empty-range case exactly the way `CellNum#/` guards zero
(REQ-VM-6):

```phalcom
class AverageFn is Fn {
  minArity => 1

  call(argNodes, ctx) {
    var total = 0
    var n = 0
    for (node in argNodes) {
      for (v in node.evalRange(ctx)) {
        (v.isError).ifTrue { return v }
        (v.isEmpty).ifFalse {
          (v.isNum).ifFalse { return ErrorVal.of(#VALUE) }
          total = total + v.num
          n = n + 1
        }
      }
    }
    return (n == 0).ifTrue({ ErrorVal.of(#DIV0) }, ifFalse: { CellNum.of(total / n) })
  }
}
```

`COUNT`/`COUNTA` are Excel's actual exceptions to "any error argument
propagates" (§4) — `COUNT` simply doesn't count non-numbers (including
errors), and `COUNTA` counts *everything* non-blank, errors included:

```phalcom
class CountFn is Fn {
  minArity => 1
  propagatesErrors => false

  call(argNodes, ctx) {
    var n = 0
    for (node in argNodes) {
      for (v in node.evalRange(ctx)) { (v.isNum).ifTrue { n = n + 1 } }
    }
    return CellNum.of(n)
  }
}

class CountaFn is Fn {
  minArity => 1
  propagatesErrors => false

  call(argNodes, ctx) {
    var n = 0
    for (node in argNodes) {
      for (v in node.evalRange(ctx)) { (v.isEmpty).ifFalse { n = n + 1 } }
    }
    return CellNum.of(n)
  }
}
```

### 2.3 `ABS`, `ROUND`, `SQRT` — reimplementing a math library that shouldn't be missing

**Number's entire method surface is `+ - * / % < <= > >= == negated hash
toString` (00-language-findings.md §3). There is no `floor`, `abs`, `round`,
`min`, `max`, `pow`, or `sqrt`.** Every one of these three functions has to be
built from that surface, in `support/num.ph` (01-architecture.md §1/§2),
before `08` can call it.

```phalcom
class AbsFn is Fn {
  minArity => 1
  maxArity => 1

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    (v.isError).ifTrue { return v }
    (v.isNum).ifFalse { return ErrorVal.of(#VALUE) }
    return CellNum.of(Num.abs(v.num))
  }
}
```

```phalcom
// support/num.ph
class Num {
  static abs(n) => (n < 0).ifTrue({ 0 - n }, ifFalse: { n })

  static min(a, b) => (a < b).ifTrue({ a }, ifFalse: { b })
  static max(a, b) => (a > b).ifTrue({ a }, ifFalse: { b })

  // Newton's method: x_{k+1} = (x_k + n / x_k) / 2. No `pow`, no `sqrt`
  // primitive exists on Number at all (findings §3) -- there is no shortcut,
  // this is the actual algorithm, spelled out, in the spreadsheet's own
  // support layer.
  static sqrt(n) {
    (n == 0).ifTrue { return 0 }
    var guess = n
    var i = 0
    while (i < 40) {
      guess = (guess + n / guess) / 2
      i = i + 1
    }
    return guess
  }
}
```

```phalcom
class SqrtFn is Fn {
  minArity => 1
  maxArity => 1

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    (v.isError).ifTrue { return v }
    (v.isNum).ifFalse { return ErrorVal.of(#VALUE) }
    (v.num < 0).ifTrue { return ErrorVal.of(#NUM) }
    return CellNum.of(Num.sqrt(v.num))
  }
}
```

`RoundFn` delegates to `Num.round(_,_)`, already specified in
02-value-model.md §5 (built from `%`/`*`/`/` by powers of ten, with an
explicit floor-toward-negative-infinity correction because
`n - (n % 1)` truncates toward zero for negative `n`, per findings §3):

```phalcom
class RoundFn is Fn {
  minArity => 1
  maxArity => 2

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    (v.isError).ifTrue { return v }
    (v.isNum).ifFalse { return ErrorVal.of(#VALUE) }
    var digits = 0
    (argNodes.size == 2).ifTrue {
      let d = argNodes.at(1).eval(ctx)
      (d.isError).ifTrue { return d }
      (d.isNum).ifFalse { return ErrorVal.of(#VALUE) }
      digits = d.num
    }
    return CellNum.of(Num.round(v.num, digits))
  }
}
```

> **Commentary — quantifying the missing-stdlib tax.** `Num.abs`/`Num.min`/
> `Num.max` are one line each. `Num.round` (02 §5) is ~30 lines once the floor
> correction is included. `Num.sqrt` above is a 6-line Newton loop — not
> difficult, but it is an algorithm a spreadsheet author has to *get right*,
> by hand, to ship the single most basic math function a spreadsheet has,
> because the host language does not have it. Total: roughly 45 lines of
> `support/num.ph` exist purely to backfill `Number`'s missing method surface,
> before a single spreadsheet-specific line of `08-functions.md` runs. A
> spreadsheet engine's math function library, in this language, has to
> reimplement the host language's own missing math library first, and that
> reimplementation is strictly larger than the spreadsheet logic sitting on
> top of it (`AbsFn`/`RoundFn`/`SqrtFn` combined are ~25 lines). This is
> GAP-NUM-3's concrete cost, not a restatement of it.

### 2.4 `CONCAT`, `LEN`, `LEFT`, `RIGHT`, `MID`

Straightforward over `String`'s verified-present surface (`+`, `size`,
`rawSlice(_,_)`, `rawByteCount`). All four assume single-byte (ASCII) text —
`rawSlice` indexes by byte offset, which only equals character offset for
ASCII. Multi-byte UTF-8 is out of scope for v1.

```phalcom
class ConcatFn is Fn {
  minArity => 1

  call(argNodes, ctx) {
    var result = ""
    for (node in argNodes) {
      let v = node.eval(ctx)
      (v.isError).ifTrue { return v }
      (v.isText).ifFalse { return ErrorVal.of(#VALUE) }
      result = result + v.text
    }
    return CellText.of(result)
  }
}

class LenFn is Fn {
  minArity => 1
  maxArity => 1

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    (v.isError).ifTrue { return v }
    (v.isText).ifFalse { return ErrorVal.of(#VALUE) }
    return CellNum.of(v.text.size)
  }
}

class LeftFn is Fn {
  minArity => 1
  maxArity => 2

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    (v.isError).ifTrue { return v }
    (v.isText).ifFalse { return ErrorVal.of(#VALUE) }
    var n = 1
    (argNodes.size == 2).ifTrue {
      let nv = argNodes.at(1).eval(ctx)
      (nv.isError).ifTrue { return nv }
      (nv.isNum).ifFalse { return ErrorVal.of(#VALUE) }
      n = nv.num
    }
    return CellText.of(v.text.rawSlice(0, n))
  }
}

class RightFn is Fn {
  minArity => 1
  maxArity => 2

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    (v.isError).ifTrue { return v }
    (v.isText).ifFalse { return ErrorVal.of(#VALUE) }
    let count = v.text.rawByteCount
    var n = 1
    (argNodes.size == 2).ifTrue {
      let nv = argNodes.at(1).eval(ctx)
      (nv.isError).ifTrue { return nv }
      (nv.isNum).ifFalse { return ErrorVal.of(#VALUE) }
      n = nv.num
    }
    return CellText.of(v.text.rawSlice(count - n, count))
  }
}

class MidFn is Fn {
  minArity => 3
  maxArity => 3

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    (v.isError).ifTrue { return v }
    (v.isText).ifFalse { return ErrorVal.of(#VALUE) }
    let startV = argNodes.at(1).eval(ctx)
    (startV.isError).ifTrue { return startV }
    (startV.isNum).ifFalse { return ErrorVal.of(#VALUE) }
    let lenV = argNodes.at(2).eval(ctx)
    (lenV.isError).ifTrue { return lenV }
    (lenV.isNum).ifFalse { return ErrorVal.of(#VALUE) }
    let start = startV.num - 1     // MID is 1-indexed, like Excel
    return CellText.of(v.text.rawSlice(start, start + lenV.num))
  }
}
```

### 2.5 `UPPER`, `LOWER` — the finding sharper than the one this document was asked to expect

The brief for this document assumed `UPPER`/`LOWER` would need "codepoint
arithmetic" — compute a shifted codepoint, build the result character from it.
**That assumption does not survive contact with 00-language-findings.md §5,
and the correction matters.** Findings §5 is explicit: there is no
`String.fromCodePoint`, no char-from-codepoint constructor of any kind, and
"the only source of characters in a Phalcom program is a string literal in
the source text." Arithmetic on a codepoint integer produces another integer —
it does not, and cannot, produce a one-character string back. `'a'.codePointAt(0)
- 32` is a computable number; there is no method anywhere in the language that
turns that number back into the string `"A"`.

So case conversion cannot be arithmetic at all. It can only be a **literal
lookup table** — every character pair spelled out by hand as source-text
literals, because a literal is the only way a character can enter the program:

```phalcom
// support/str.ph — extends 01-architecture.md's documented Str
// (padLeft, padRight, repeat, startsWith) with upper/lower.
let upperPairs_ = Map.new()
upperPairs_.at("a", put: "A")
upperPairs_.at("b", put: "B")
upperPairs_.at("c", put: "C")
// ... all 26, spelled out; there is no loop that can generate this table.
upperPairs_.at("z", put: "Z")

let lowerPairs_ = Map.new()
lowerPairs_.at("A", put: "a")
// ... the mirror 26 entries.
lowerPairs_.at("Z", put: "z")

class Str {
  static upper(s) {
    var result = ""
    var i = 0
    while (i < s.rawByteCount) {
      let ch = s.rawSlice(i, i + 1)
      let mapped = upperPairs_.at(ch)
      result = result + (mapped == None).ifTrue({ ch }, ifFalse: { mapped })
      i = i + 1
    }
    return result
  }

  static lower(s) {
    var result = ""
    var i = 0
    while (i < s.rawByteCount) {
      let ch = s.rawSlice(i, i + 1)
      let mapped = lowerPairs_.at(ch)
      result = result + (mapped == None).ifTrue({ ch }, ifFalse: { mapped })
      i = i + 1
    }
    return result
  }
}
```

```phalcom
class UpperFn is Fn {
  minArity => 1
  maxArity => 1

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    (v.isError).ifTrue { return v }
    (v.isText).ifFalse { return ErrorVal.of(#VALUE) }
    return CellText.of(Str.upper(v.text))
  }
}
```

`LowerFn` is the mirror image over `Str.lower`.

> **Commentary — GAP-STR-2, and it is worse than "tedious."** This is not the
> "hand-roll it with `%` and `*`" tax that `Num.round`/`Num.sqrt` pay (§2.3) —
> those are genuinely computable, just missing. Case conversion in Phalcom is
> **not computable at all** without a pre-existing literal table, because
> there is no operation in the language that constructs a character from a
> computed value. The 52-entry table above covers ASCII only because ASCII is
> small enough to spell out by hand; the equivalent for full Unicode case
> folding (thousands of pairs, several of them context-sensitive) is not a
> "write more code" problem, it is a "the language provides no way to do this
> at all beyond enumerating every pair as a literal" problem. This is a
> stronger, more precise version of GAP-STR-1 (00-language-findings.md §5's
> unreachable `"`) and belongs in that document as its own entry: **GAP-STR-2
> — there is no codepoint-to-character constructor, so no function of a
> string's characters can be computed; it can only be looked up against a
> pre-enumerated literal table.** `String.fromCodePoint(_)` (or equivalent) is
> the single highest-value string primitive missing from the language, ahead
> of `padLeft`/`toUpper` themselves — either of those two could be built from
> `fromCodePoint` in a few lines; neither can currently be built at all
> without one.

### 2.6 `IF`, `AND`, `OR`, `NOT` — laziness `Call#eval`'s design already bought

Because `Call#eval` (06-ast-and-eval.md §5) hands `IfFn` the raw, unevaluated
`argNodes`, `IF` gets short-circuit evaluation of its branches for free — it
never calls `.eval(ctx)` on the branch it doesn't take, so `=IF(A1=0, 0,
1/A1)` never evaluates `1/A1` when `A1` is `0`, without `IfFn` doing anything
special to arrange that.

```phalcom
class IfFn is Fn {
  minArity => 2
  maxArity => 3

  call(argNodes, ctx) {
    let cond = argNodes.at(0).eval(ctx)
    (cond.isError).ifTrue { return cond }
    (cond.isBool).ifFalse { return ErrorVal.of(#VALUE) }
    (cond.bool).ifTrue { return argNodes.at(1).eval(ctx) }
    (argNodes.size == 3).ifTrue { return argNodes.at(2).eval(ctx) }
    return CellEmpty.new()
  }
}
```

(`.bool` is the natural extension of 02-value-model.md §2's `num`/`text` root
accessors to `CellBool` — same idiom, added here because `IF`/`AND`/`OR`/`NOT`
are the first consumers that need it.)

`AND`/`OR` evaluate every argument (Excel does not short-circuit these) and
propagate the first error found, left to right:

```phalcom
class AndFn is Fn {
  minArity => 1

  call(argNodes, ctx) {
    for (node in argNodes) {
      let v = node.eval(ctx)
      (v.isError).ifTrue { return v }
      (v.isBool).ifFalse { return ErrorVal.of(#VALUE) }
      (v.bool).ifFalse { return CellBool.of(false) }
    }
    return CellBool.of(true)
  }
}

class OrFn is Fn {
  minArity => 1

  call(argNodes, ctx) {
    for (node in argNodes) {
      let v = node.eval(ctx)
      (v.isError).ifTrue { return v }
      (v.isBool).ifFalse { return ErrorVal.of(#VALUE) }
      (v.bool).ifTrue { return CellBool.of(true) }
    }
    return CellBool.of(false)
  }
}

class NotFn is Fn {
  minArity => 1
  maxArity => 1

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    (v.isError).ifTrue { return v }
    (v.isBool).ifFalse { return ErrorVal.of(#VALUE) }
    return CellBool.of(not v.bool)
  }
}
```

### 2.7 `ISERROR`, `IFERROR` — the functions that must NOT propagate

Every function above returns its first error argument unexamined. `ISERROR`
and `IFERROR` are the two functions whose entire job is to look *at* the
error rather than through it — they are the documented exceptions to §4's
default rule, not a special case bolted on top of it.

```phalcom
class IsErrorFn is Fn {
  minArity => 1
  maxArity => 1
  propagatesErrors => false

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    return CellBool.of(v.isError)
  }
}

class IfErrorFn is Fn {
  minArity => 2
  maxArity => 2
  propagatesErrors => false

  call(argNodes, ctx) {
    let v = argNodes.at(0).eval(ctx)
    return (v.isError).ifTrue({ argNodes.at(1).eval(ctx) }, ifFalse: { v })
  }
}
```

### 2.8 `VLOOKUP` — the function that forced `evalRows`

`VLOOKUP(key, table, colIndex, [rangeLookup])` needs its `table` argument's
**row structure** — compare `key` against column 1, and if it matches, return
column `colIndex` of *that same row*. A flattened `List<CellValue>` has
already lost which values belonged to which row (06 §5's commentary on why
`Call#eval` does not flatten centrally). This is the concrete function that
forced that decision; `RangeNode` needs one more evaluation entry point beyond
`evalRange`:

```phalcom
// Addition to RangeNode (06-ast-and-eval.md §5), needed only by VLOOKUP:
class RangeNode is Ast {
  // ... eval, evalRange, dependencies as in 06 §5 ...

  evalRows(ctx) {
    var rows = List.new()
    for (rowRefs in Grid.rowsInRect(_from, _to)) {
      var row = List.new()
      for (ref in rowRefs) { row.add(ctx.grid.at(ref).cachedValue) }
      rows.add(row)
    }
    return rows
  }
}
```

(`Grid.rowsInRect(_,_)` — a row-grouped variant of `Grid.refsInRect` — is
specified alongside it in 03-references-and-grid.md.)

```phalcom
class VlookupFn is Fn {
  minArity => 3
  maxArity => 4    // 4th arg (range-lookup flag) is accepted but ignored -- DEC-FN-3

  call(argNodes, ctx) {
    let key = argNodes.at(0).eval(ctx)
    (key.isError).ifTrue { return key }
    let colArg = argNodes.at(2).eval(ctx)
    (colArg.isError).ifTrue { return colArg }
    (colArg.isNum).ifFalse { return ErrorVal.of(#VALUE) }
    let col = colArg.num
    for (row in argNodes.at(1).evalRows(ctx)) {
      (col < 1 or col > row.size).ifTrue { return ErrorVal.of(#REF) }
      (row.at(0) == key).ifTrue { return row.at(col - 1) }
    }
    return ErrorVal.of(#NA)
  }
}
```

**DEC-FN-3.** v1's `VLOOKUP` always does an exact-match lookup against column
1, regardless of the 4th argument. Excel's approximate-match mode (sorted
binary search when the 4th argument is `TRUE`/omitted) is out of scope for v1.

### 2.9 `COUNTIF` — exact-match only

```phalcom
class CountIfFn is Fn {
  minArity => 2
  maxArity => 2

  call(argNodes, ctx) {
    let crit = argNodes.at(1).eval(ctx)
    (crit.isError).ifTrue { return crit }
    var n = 0
    for (v in argNodes.at(0).evalRange(ctx)) {
      (v == crit).ifTrue { n = n + 1 }
    }
    return CellNum.of(n)
  }
}
```

**DEC-FN-2.** v1's `COUNTIF` supports exact-match criteria only — the
criterion argument is evaluated to a `CellValue` and compared with `==`.
Excel's operator-prefixed text criteria (`">5"`, `"<>foo"`) require parsing
the criterion string into an operator + operand and are out of scope for v1.

## 3. Argument coercion and the guard-before-accessor rule

Every function body above follows one fixed pattern per argument, in order:

1. `let v = argNodes.at(i).eval(ctx)` (or `.evalRange`/`.evalRows` for a range
   position).
2. `(v.isError).ifTrue { return v }` — unless `propagatesErrors => false`
   (`ISERROR`, `IFERROR`, `COUNT`, `COUNTA`).
3. `(v.isNum` / `isText` / `isBool `).ifFalse { return ErrorVal.of(#VALUE) }`
   before ever calling `.num`/`.text`/`.bool`.

Step 3 is still required, but **the reason changed after this document was
drafted.** This section originally argued that the guard was load-bearing
because `CellValue`'s root `num`/`text` accessors *raise* (via `.raise_`), so
skipping the check would crash the function.

That was based on a sketch in 02-value-model.md that **did not work**:
`raise_` is not a selector (it is `raise()`), and `raise` is installed only on
`Error` and its subclasses — `ErrorVal` extends `CellValue`, so the send would
have been `MessageNotUnderstood`, not a graceful error. 02 has since been
corrected: the root accessors **return `ErrorVal.of(#VALUE)`**, per REQ-VM-1
(a bad payload access is spreadsheet data, not a program fault).

So an unguarded `.num` now degrades to `#VALUE!` rather than crashing. The
guard remains mandatory for a different and better reason: **it lets the
function choose the right error kind and the right coercion** (a range argument
containing text should often be *skipped*, as `COUNT` does, not poisoned to
`#VALUE!`). Relying on the root default silently flattens those distinctions.

**Every function in this document checks the classification predicate before
the accessor, with no exceptions.**

**REQ-FN-3.** No function implementation calls `.num`/`.text`/`.bool` on a
`CellValue` without a preceding `isNum`/`isText`/`isBool` guard on that exact
value.

## 4. Error propagation policy

**Default rule: the first error value encountered, scanning arguments left to
right (and within a range argument, in `Grid.refsInRect`/`rowsInRect` order),
is returned unchanged.** This is `Fn#propagatesErrors => true` (the root
default) implemented inline in each function as
`(v.isError).ifTrue { return v }` at the point each value is read — there is
no shared helper that intercepts this centrally, because each function reads
its arguments in its own shape (scalar vs. range vs. row) per 06 §5, so the
check naturally sits at the one place each function already touches a raw
value.

**Exceptions**, each overriding `propagatesErrors => false`:

| Function | Why it must inspect rather than propagate |
|---|---|
| `ISERROR` | Its entire purpose is to answer "is this an error" — propagating would make it unable to ever return `false` next to an error. |
| `IFERROR` | Same reason, plus it substitutes a fallback value on the error path. |
| `COUNT` | Only counts numeric cells; an error cell is simply not counted, not propagated. |
| `COUNTA` | Counts every non-blank cell, including error cells, as present. |

**REQ-FN-4.** Every function not listed in the exceptions table above
propagates the first `ErrorVal` it encounters, unmodified, as its own return
value.
**REQ-FN-5.** `IfFn#call` evaluates only the branch selected by its condition;
the untaken branch's `Ast` subtree is never sent `eval` or `evalRange`.

## 5. Requirements summary

| REQ | Statement |
|---|---|
| REQ-FN-1 | `FunctionTable` maps an exact function name to one `Fn` instance; unknown name → `ErrorVal(#NAME)`. |
| REQ-FN-2 | Arity is checked centrally in `invoke`; mismatch → `ErrorVal(#VALUE)`. |
| REQ-FN-3 | No function calls `.num`/`.text`/`.bool` without a preceding classification guard. |
| REQ-FN-4 | Default policy: first error argument propagates unchanged, except the four listed exceptions. |
| REQ-FN-5 | `IF` evaluates only the taken branch. |
| REQ-FN-6 | `MIN`/`MAX`/`ABS`/`ROUND`/`SQRT` are built entirely over `support/num.ph`'s `Num`; none call a `Number` method beyond `+ - * / % < <= > >= == negated`. |
| REQ-FN-7 | `UPPER`/`LOWER` cover ASCII `a-z`/`A-Z` only, via a 52-entry literal table (GAP-STR-2); non-ASCII bytes pass through unchanged. |
| REQ-FN-8 | `LEFT`/`RIGHT`/`MID`/`LEN` assume single-byte (ASCII) text. |
| REQ-FN-9 | `VLOOKUP` is exact-match on column 1 only (DEC-FN-3); the 4th argument is accepted and ignored. |
| REQ-FN-10 | `COUNTIF` supports exact-match criteria only (DEC-FN-2). |

## 6. Test hooks

| REQ | Test |
|---|---|
| REQ-FN-1/2 | `suites/functions_dispatch.ph` — unknown name, too-few args, too-many args |
| REQ-FN-3/4 | `suites/functions_propagation.ph` — one error-bearing argument per function, per position |
| REQ-FN-5 | `suites/functions_if_laziness.ph` — `IF` branch containing a `1/0`-shaped subtree that must never evaluate |
| REQ-FN-6 | `suites/functions_math.ph` — `MIN`/`MAX`/`ABS`/`ROUND`/`SQRT` against known values, including `SQRT` of a perfect square and of a negative number |
| REQ-FN-7 | `suites/functions_text_case.ph` — `UPPER`/`LOWER` over the full ASCII alphabet plus a non-letter pass-through character |
| REQ-FN-8 | `suites/functions_text_slice.ph` — `LEFT`/`RIGHT`/`MID`/`LEN` boundary cases (n=0, n=size, n>size) |
| REQ-FN-9 | `suites/functions_vlookup.ph` — hit, miss (`#N/A`), out-of-range column (`#REF!`) |
| REQ-FN-10 | `suites/functions_countif.ph` — exact match against text and number criteria |
