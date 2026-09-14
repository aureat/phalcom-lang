# SheetCalc — Language Findings (Probe Log)

Part of the [SheetCalc specification](README.md).

**Status:** Verified 2026-07-14 against `main` @ `5516504`, release build.

This document is the **fact base** for every other document in this spec. Each
claim below was established by *running* a probe program, not by reading source
and inferring. No downstream document may assume a language capability that is
not recorded here as `VERIFIED-PRESENT`.

> **Rule for spec authors and implementers.** If you need a capability that is
> listed here as `VERIFIED-ABSENT`, you must either design around it and record
> the workaround in [13-language-gaps.md](13-language-gaps.md), or land the
> capability in the runtime first. Do not write `.ph` code against a feature
> that this document says does not exist.

---

## 1. Probe method

Each probe is a self-contained `.ph` program run through
`target/release/phalcom`. A capability is recorded `VERIFIED-PRESENT` only if a
probe produced its expected output, and `VERIFIED-ABSENT` only if a probe
produced a `MessageNotUnderstood`, a parse error, or a demonstrably wrong value.

Where a probe failed because of *my own* syntax error rather than a language
gap, that is recorded explicitly (§9) — several such mistakes were made and
corrected during this survey, and distinguishing "the language lacks X" from "I
wrote X wrong" is the whole point of a probe log.

---

## 2. I/O surface — the defining constraint

| Capability | Status | Evidence |
|---|---|---|
| `System.print(_)` | VERIFIED-PRESENT | prints args + newline |
| `System.rawWrite(_)` | VERIFIED-PRESENT | write without newline |
| `System.schedule(_)` / `System.nextScheduled` | VERIFIED-PRESENT | fiber ready-queue |
| Read a file | VERIFIED-ABSENT | no primitive registered |
| Read stdin | VERIFIED-ABSENT | no primitive registered |
| Wall clock / monotonic time | VERIFIED-ABSENT | no primitive registered |
| Random | VERIFIED-ABSENT | no primitive registered |
| `argv` / env vars | VERIFIED-ABSENT | no primitive registered |
| Process exit code | VERIFIED-ABSENT | no primitive registered |

`System`'s entire native surface is `print`, `rawWrite`, `schedule`,
`nextScheduled`, `new` (which always raises).

**Consequence, and it shapes the whole program.** SheetCalc cannot be a CLI
tool. It cannot load a workbook, accept a formula from a user, or report a
failure to a shell. Every input is a literal in the source text, and the only
output channel is stdout. SheetCalc is therefore specified as a **self-driving
demonstration and test harness**, not an interactive application. See
[13-language-gaps.md §2](13-language-gaps.md) for the primitive set that would
change this, which is the single highest-value item this exercise produced.

---

## 3. Numbers

**All numbers are `f64`.** There is no integer type.

> **CORRECTION.** This section originally said "`Number` has zero methods" full
> stop. That is true of the **instance** surface but false of the **class**
> side: `Number.new(_)` is a real string-to-number parser, found by a reviewer
> and confirmed by probe:
>
> ```phalcom
> Number.new("42")    // => Ok(42)
> Number.new("5e3")   // => Ok(5000)     -- scientific notation
> Number.new("-3.5")  // => Ok(-3.5)
> Number.new("abc")   // => Err(<Error>)
> ```
>
> It is backed by Rust's `f64::parse` and is undocumented anywhere. This matters
> a lot for [04-formula-lexer.md](04-formula-lexer.md): the lexer does **not**
> need to hand-roll numeric parsing, which was the assumption before this was
> found. A reminder that "I enumerated the primitive table" is not the same as
> "I enumerated the surface" — I read the instance rows and stopped.

Full **instance** method surface on `Number`, exhaustively enumerated from the
primitive registration table and confirmed by probe:

```
+  -  *  /  %  <  <=  >  >=  ==  negated  hash  toString
```

That is the **complete** list. Note `negated` is a **0-arity method, not a
getter** — `n.negated` raises `MessageNotUnderstood`; `n.negated()` works. This
is GAP-DX-1 biting in the wild: I hit it while writing `Num.abs(_)` for this
very spec, having *already documented* the getter/method distinction in §10.
Unary minus (`-3.7`) parses fine and is the better idiom.

Verified absent, each returning `MessageNotUnderstood`:

```
floor  ceil  round  abs  sqrt  truncate  toInt  min  max  pow  isNan  isInfinite
```

Probe results that matter:

| Expression | Result | Comment |
|---|---|---|
| `7 / 2` | `3.5` | no integer division |
| `1 / 0` | `inf` | **no `DivideByZero` error is raised** |
| `(3.0).toString` | `"3"` | integral floats render without `.0` |
| `(0.1 + 0.2).toString` | `"0.30000000000000004"` | full float noise, no formatting control |
| `(3.7 % 1)` | `0.7000000000000002` | `%` works on floats |
| `(-3.7 % 1)` | `-0.7000000000000002` | sign follows dividend (truncated, not floored) |

**Consequences.**

1. `#DIV/0!` **must be detected by an explicit zero-check before dividing.** The
   runtime will hand back `inf`, not an error. This is a correctness landmine:
   the naive implementation silently produces `inf` and propagates it through
   the whole sheet. Specified in [02-value-model.md](02-value-model.md).
2. `floor` must be hand-rolled, and the obvious version is wrong. **Verified by
   probe:**

   | `n` | `n - (n % 1)` | correct `floor(n)` |
   |---|---|---|
   | `3.7` | `3` | `3` |
   | `-3.7` | **`-3`** | **`-4`** |
   | `-0.5` | **`0`** | **`-1`** |

   `n - (n % 1)` truncates toward zero because `%` follows the dividend's sign.
   The correction:

   ```phalcom
   static floor(n) {
     let t = n - (n % 1)
     if (n < 0 and (n % 1) != 0) { return t - 1 }
     return t
   }
   ```

   Every Phalcom program needing `floor` must know this. Most will not.
3. Number display formatting (`ROUND`, column alignment, `0.30000000000000004`)
   must be built from `%` and string surgery. See
   [09-rendering.md](09-rendering.md).

---

## 4. Arithmetic against user objects — no double dispatch

This was the probe I predicted would matter most, and it does.

```phalcom
class Err2 { toString => "#DIV/0!" }
let e = Err2.new()
{ 1 + e }.attempt()   // => Err(<Error>)                  -- Number#+ raises
{ e + 1 }.attempt()   // => Err(<MessageNotUnderstood>)   -- Err2 has no #+(_)
{ 1 + "x" }.attempt() // => Err(<Error>)
(1 == "x")            // => false                          -- == is total, never raises
```

`Number#+` is a **native primitive**. It type-checks its argument and raises.
It cannot be overridden from `.ph`, and Phalcom has no multimethods and no
coercion protocol (no `coerce:`, no `__radd__`). Therefore:

> **A native `Number` on the left of `+` can never cooperate with a user-defined
> value on the right.**

This is the single most consequential design fact in the whole exercise. A
spreadsheet's defining behavior is that errors propagate through arithmetic:
`=A1+B1` where `A1` is `#DIV/0!` must yield `#DIV/0!`, not a crash. Since
`1 + errorValue` is unfixable from user code, SheetCalc **cannot represent cell
values as bare native numbers**. Every cell value must be a user-class instance
implementing its own `+`, so the receiver is always under our control.

That decision is forced by the language, not chosen. It is recorded as
**DEC-VM-1** in [02-value-model.md](02-value-model.md), and it costs an
allocation per arithmetic step.

---

## 5. Strings — the escape gap

Full escape set in string literals, read from the lexer and confirmed by probe:

- `\\` → a literal backslash
- `\(expr)` → interpolation

**That is all.** The lexer's fallthrough is: *a backslash before any other
character is a literal backslash, and the next character is scanned normally.*

| Probe | Result |
|---|---|
| `"say \"hi\""` | **Lex error: `Invalid token`** |
| `"a\nb"` | prints literally `a\nb` — no newline |
| `String.new(34)` | `"34"` — stringifies the number; **not** a char from codepoint |

There is no `String.fromCodePoint`, no `String#at(_)`, and no char-from-byte
constructor. The only source of characters in a Phalcom program is a string
literal in the source text.

> **Therefore the `"` character is unreachable in Phalcom.** It cannot be
> written in a literal, and it cannot be constructed. A Phalcom program cannot
> emit a double quote on stdout.

**Consequence.** SheetCalc's formula language cannot use `"` as its string
delimiter, because the spec's own test fixtures are Phalcom string literals
containing formulas. `=CONCAT("a","b")` is unwritable. SheetCalc's formula
grammar therefore uses **single quotes** for text literals (`=CONCAT('a','b')`).

This is a workaround for a language gap wearing the costume of a design choice,
and it should be read that way. Recorded as **GAP-STR-1**.

**`Symbol#toString` includes the leading `#`.** `#DIV0.toString` is `"#DIV0"`,
not `"DIV0"`. Minor, but it bit the first draft of `ErrorVal#toString`, which
rendered `##DIV0!`. This is why [02-value-model.md §3](02-value-model.md)
specifies a name table rather than deriving the display string from the symbol.
Symbols work correctly as `Map` keys (verified).

String surface verified present: `+`, `size`, `hash`, `toString`,
`codePointAt(_)`, `indexOf(_)`, `split(_)`, `replace(_,_)`, `trim`/`trimStart`/
`trimEnd` (both bare and `(chars)` forms), `rawByteAt(_)`, `rawSlice(_,_)`,
`rawByteCount`, `bytes`, `codePoints`.

Verified absent: `padLeft`/`padRight`, `toUpper`/`toUpperCase`, `at(_)`,
`reversed`, `startsWith`, `endsWith`, `contains`.

---

## 6. `toString` divergence — a live inconsistency

```phalcom
class Cell { toString => "CELL" }
let c = Cell.new()

System.print(c)          // => CELL              -- sends #toString
System.print("\(c)")     // => <Cell instance>   -- does NOT send #toString
System.print("" + c.toString) // => CELL         -- explicit send
System.print([c])        // => [<Cell instance>] -- does NOT send #toString
```

**String interpolation and `List#toString` do not send the `toString` message to
user instances.** They use a native renderer (`Value::to_string`) instead, while
`System.print(_)` uses `Value::to_display_string`, which *does* send the message.

This is not a cosmetic quirk. Interpolation is the most idiomatic rendering
construct in the language — `showcase.ph` leads with it — and it silently
produces `<Cell instance>` for exactly the domain objects a user most wants to
render. There is no diagnostic; the wrong output just appears.

For SheetCalc, whose entire output is a rendered grid of user-class cell values,
this would corrupt every cell in every test if used naively.

**Mitigation (mandatory, spec-wide):** SheetCalc **never** interpolates a user
object. Every render site sends `.toString` explicitly:

```phalcom
"\(cell)"          // FORBIDDEN — renders <Cell instance>
"" + cell.toString // REQUIRED
```

A lint check in the test suite greps the sources for `\(` applied to known
cell-valued locals. Recorded as **BUG-TOSTR-1**; this is a genuine runtime bug
worth filing independently of SheetCalc, and the pre-existing note in
`core.ph`'s `System.write` comment ("pre-existing divergence between
`Value::to_string` and the `.toString` message") confirms it is known but
unfixed.

---

## 7. Collections

| Capability | Status | Notes |
|---|---|---|
| `Map`/`Set` keyed by user class | **VERIFIED-PRESENT** | `hash` + `==` are honored correctly |
| Insertion-ordered iteration | **VERIFIED-PRESENT** | stable across runs; golden output is viable |
| `Set#includes(_)` | VERIFIED-PRESENT | **`Set#contains(_)` does NOT exist** — it is `includes` |
| `Set#remove(_)` | VERIFIED-PRESENT | returns the `Set` |
| `for (x in aMap)` | VERIFIED-PRESENT | iterates **keys**, in insertion order |
| `for (x in aSet)` | VERIFIED-PRESENT | insertion order |
| Block literal as a positional argument | VERIFIED-PRESENT | `Sort.by(l, { a, b => a < b })` works — blocks are ordinary values |
| `List#sort` / `#sorted` | VERIFIED-ABSENT | `MessageNotUnderstood` |
| Explicit iterator pipelines `.iter.filter{}.map{}` | VERIFIED-PRESENT | compose lazily and repeatably |
| `break` inside `for` over a lazy view | VERIFIED-PRESENT | works |
| List literal `[1, 2, 3]` | VERIFIED-PRESENT | |
| Range literal `1..3` | **VERIFIED-ABSENT** | `..` *lexes* (`DotDot` token) but **does not parse** |
| `Range.new(start, end, inclusive)` | VERIFIED-PRESENT | 3-arg form is the only constructor |

Probe:

```phalcom
let m = Map.new()
m.at(Ref.at(1, 2), put: "hello")
m.at(Ref.at(1, 2))        // => "hello"  -- distinct but equal key resolves

let s = Set.new()
s.add(Ref.at(1, 2))
s.add(Ref.at(1, 2))
s.size                     // => 1        -- dedup honors ==
```

The `Ref`-as-`Map`-key contract — the thing I flagged as most likely to be
under-exercised and broken — **works correctly**. Good news, and it means
[03-references-and-grid.md](03-references-and-grid.md) can use the natural
design.

Map/Set **insertion order is deterministic across runs**, which is what makes a
stdout-exact golden test suite possible at all. This is load-bearing for
[10-testing.md](10-testing.md).

**`List#sort` is absent**, so SheetCalc ships its own `Sort.by(_, _)` merge sort
(§[09-rendering.md](09-rendering.md)) — needed for deterministic error listings
and column ordering.

---

## 8. Fibers — two hard traps

| Capability | Status |
|---|---|
| `Fiber.new { }`, `#call()`, `#try()`, `Fiber.yield(_)`, `Fiber.current`, `Fiber.abort(_)`, `#isDone`, `#error` | VERIFIED-PRESENT |
| Recursion depth (50 000 frames) | VERIFIED-PRESENT — no ceiling hit |

### Trap 1 — `CannotYieldAcrossNativeFrame`

> **CORRECTED 2026-07-14.** The first draft of this section overstated the trap,
> because **the probe that established it was flawed**. The correction is
> recorded in full below rather than quietly patched, because the error is more
> instructive than the finding.

**The rule.** Yielding across a **native call frame** is a hard error. The
canonical trip-wire is `Block#call`, which every block-taking combinator uses:

```phalcom
let f = Fiber.new {
  [1, 2, 3].each { x => Fiber.yield(x) }   // Block#call -> native frame
  "done"
}
f.call()
// => cannot switch fibers across a native call frame (e.g. inside .each { })
```

The runtime's own error text names the case exactly. `each`, `map`, `where`,
`filter`, `reduce` — anything that invokes a user block through the native
`Block#call` — is unsafe inside a yielding fiber.

**But `for` is safe.** Verified:

```phalcom
let f = Fiber.new {
  for (x in [1, 2, 3]) { Fiber.yield(x) }   // no Block object, no .call()
  "done"
}
f.call()   // => 1
f.call()   // => 2
f.call()   // => 3
f.call()   // => done
```

This works on a **native `List`** and on a **user-defined `Iterable`** whose
`iterate`/`iteratorValue` are written in `.ph`. The compiler lowers `for` to
direct `iterate`/`iteratorValue` sends with an inlined body — no `Block` is
allocated and no native frame is crossed. Hand-rolled indexed `while` loops are
also fine, as expected.

So the accurate statement is **per-method, not per-type**:

> Inside a yielding fiber, `for` and `while` are safe. The block-taking
> combinators (`each`/`map`/`where`/`filter`/`reduce`) are not.

That is a materially smaller restriction than "the entire collection API is
unavailable."

#### How the first draft got this wrong

The original probe was:

```phalcom
{ f.call() }.attempt()   // => Err(<CannotYieldAcrossNativeFrame>)
```

The `{ ... }.attempt()` wrapper **is itself a native `Block#call` frame**. The
probe harness introduced the very frame it was measuring, so *every* fiber
yield failed — including `for`, which is actually fine. I then read that
uniform failure as "all iteration breaks fibers" and generalized it into an
architectural finding. A bare, unwrapped `f.call()` tells the truth.

> **Commentary — the meta-finding.** This is the most valuable mistake in the
> exercise, and it argues for the method rather than against it. A wrong fact
> established by a *convenient* harness (`attempt()` was wrapped around
> everything precisely because it made probe output uniform and easy to print)
> propagated into four documents as a headline architectural claim before a
> second party's contradicting probe caught it. The lesson is narrow and
> generalizable: **when a probe harness wraps the thing under test, verify the
> harness is not the thing being tested.** ADR-0030 §4's constraint is real; my
> characterization of its blast radius was an artifact.
>
> It also vindicates the runtime's diagnostic. The error text —
> *"cannot switch fibers across a native call frame (e.g. inside .each { })"* —
> names the exact mechanism **and** gives the canonical example. Had I read it
> as carefully as I read my own hypothesis, I would have caught this in one
> step.

Recorded as **GAP-FIB-1** (severity revised down from architectural to
ergonomic).

### Trap 2 — `return` inside a fiber block is a `DeadFrameError`

```phalcom
let f = Fiber.new {
  Fiber.yield(1)
  return "done"     // => DeadFrameError on the final resume
}
```

`return` in a block is a **non-local return to the block's home method frame**.
For a fiber body, that frame is long gone by the time the body completes. The
correct idiom is the **implicit last expression**:

```phalcom
let f = Fiber.new {
  Fiber.yield(1)
  "done"            // correct — implicit last value
}
```

Verified working. The failure mode is nasty: the fiber runs correctly for every
`yield` and only explodes on the *final* resume, so a shallow test passes.
Recorded as **GAP-FIB-2**.

---

## 9. Reflection and attributes — richer than expected

Full surface, enumerated from the primitive table and probe-confirmed:

| Selector | On | Status |
|---|---|---|
| `perform(_)`, `perform(_,_)` | `Object` | PRESENT |
| `respondsTo(_)` | `Object` | PRESENT |
| `doesNotUnderstand(_)` | `Object` | PRESENT — **overridable** |
| `methodFor(_)` | `Object` | PRESENT — returns a `Method` |
| `__attributes`, `__attach(_)`, `__freezeAttributes` | `Object` | PRESENT |
| `methods` | `Behavior` | PRESENT — **returns `Symbol`s, not `Method`s** |
| `name`, `superclass` (getter **and setter**) | `Behavior` | PRESENT |
| `attributes`, `attributesOfType(_)` | `Behavior`, `Method` | PRESENT |
| `invokeOn(_,_)`, `bind(_)`, `selector`, `holder` | `Method` | PRESENT |
| `selector`, `name`, `labels`, `args` | `Message` | PRESENT |
| **Install a method into a class** | — | **VERIFIED-ABSENT** |

`Class#+(_)` is **string concatenation of two class names** — it is *not*
"add a method to a class". I misread it as an installer at first; it is not one.

> **There is no way to add, replace, or wrap a method on a class from `.ph`.**

This kills the classic decorator implementation (read method, wrap it, reinstall
it). What survives is the **`doesNotUnderstand` forwarding proxy**, verified
working end to end for both getter and method sends:

```phalcom
class TracingProxy {
  @constructor
  on(t) { _t = t }
  doesNotUnderstand(msg) {
    System.print("  -> " + msg.name.toString + " args=" + msg.args.toString)
    let r = _t.perform(msg.selector, msg.args)
    System.print("  <- " + r.toString)
    return r
  }
}
TracingProxy.on(Svc.new()).ping(7)
//   -> ping args=[7]
//   <- pong:7
```

This works, and it is the foundation of [11-decorators.md](11-decorators.md).

### The reflection-driven test runner is verified working

The full discovery path that [10-testing.md](10-testing.md) is built on was run
end to end:

```phalcom
suiteClass.methods.each { sel =>              // -> List<Symbol>
  let m = instance.methodFor(sel)             // -> Method
  let attrs = m.attributesOfType(Test)        // -> List<Test>
  if (not attrs.isEmpty) {
    let r = { m.invokeOn(instance, []) }.attempt()
    ...
  }
}
```

Output:

```
  [true] two plus three => Ok(5)
  [true] division guard => Ok(1)
ran: 2
```

Untagged methods (`helper`) are correctly skipped. **This is the single most
capable thing I built during the survey**, and it is pure Phalcom with no
workarounds. The reflection layer earns its keep.

Note the `methods`-returns-`Symbol`s detail forces the extra `methodFor(sel)`
round-trip; a `Behavior#methodObjects` (or having `methods` return `Method`s)
would remove it. Minor.

### Attributes

Retention works and is reflectable:

```phalcom
class Test is Attribute {
  @constructor
  new(n) { _n = n }
}
class Suite {
  @Test("adds two numbers")
  testAdd => 1 + 1
}
Suite.new().methodFor(#testAdd).attributesOfType(Test)  // => [<Test instance>]
```

**Spec-vs-implementation divergence found.** `attribute-classes.md` line 169
documents `@Author(name: "Ada")` — a keyword-label argument form. The parser
**rejects it**:

```
@Test(name: "adds two numbers")
      ^ Expected ")"
```

Only the **bare** (`@Loud`) and **positional** (`@Test("x")`) forms parse.
Recorded as **DIV-ATTR-1**: either the spec or the parser is wrong, and the
spec is the newer document, so the parser is most likely behind.

---

## 10. Syntax facts (mostly my own errors, logged for honesty)

These cost me probe cycles. They are **not** language defects, but they are
real friction and belong in an onboarding document.

| I wrote | Actual | Note |
|---|---|---|
| `a && b`, `a \|\| b` | `a and b`, `a or b` | `and`/`or`/`not` are keywords; `&&` is `Invalid token` |
| `{ _c = c _r = r }` | statements need `;` or newline separators | no statement juxtaposition |
| `f.call` | `f.call()` | **`call` is `Method(0)`, not a getter** |
| `x.isEmpty.not` | `not x.isEmpty` | **`not` is a prefix keyword, not a method.** `.not` is `MessageNotUnderstood`. Same for `and`/`or`. |
| `return [1, 2, 3]` | `var l = [1, 2, 3]` then `return l` | **`return` immediately followed by a list literal does not parse** (`Expected one of ";", newline` at the `[`). The arrow form `f => [1, 2, 3]` works. Genuine parser gap — logged as GAP-SYN-2. |
| `1..3` | `Range.new(1, 3, true)` | range literal does not parse |
| `Range.new(1, 3)` | `Range.new(1, 3, true)` | 3-arg only |
| `@Test(name: "x")` | `@Test("x")` | keyword attribute args do not parse |

The `f.call` vs `f.call()` distinction deserves emphasis. Phalcom's signature
kinds make a **getter** and a **0-arity method** two genuinely different
selectors. `f.call` raises `MessageNotUnderstood` while `f.call()` works. The
diagnostic (`<fiber> does not understand 'call'`) is accurate but reads as
baffling when `call` is *right there* in the primitive table. Recorded as
**GAP-DX-1**.

---

## 11. Summary — what the probes changed about the design

| Prediction (pre-probe) | Outcome |
|---|---|
| `1 + errorValue` will be a problem | **Confirmed, worse than expected.** Forces every cell value into a user class (DEC-VM-1). |
| `Ref` as `Map` key will break `hash`/`==` | **Refuted.** Works correctly. |
| Deep recursion will hit a ceiling | **Refuted.** 50 000 frames fine. |
| Fiber-under-native-block will break | **Confirmed.** `CannotYieldAcrossNativeFrame` (GAP-FIB-1). |
| Lazy views + `break` will misbehave | **Refuted.** Works. |
| Attributes are unexercised | **Partly confirmed.** They work, but the documented keyword form does not parse (DIV-ATTR-1). |
| *(unforeseen)* | **`"` is unreachable in the language** (GAP-STR-1). |
| *(unforeseen)* | **Interpolation bypasses user `toString`** (BUG-TOSTR-1). |
| *(unforeseen)* | **`1/0` returns `inf` silently** — no error. |
| *(unforeseen)* | **`Number` has zero methods** — no `floor`/`round`/`abs`. |

Two of the three highest-severity findings — the unreachable `"` and the
`toString` divergence — were **not predicted**. Both were found only by trying
to write real code. That is the argument for this exercise in one line.
