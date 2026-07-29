# SheetCalc — Language Gaps and Wishlist

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md).

This is the deliverable. SheetCalc is the excuse; this document is the product.

Everything here was found by trying to write a real program and failing. Nothing
here is a survey opinion or a feature wish borrowed from another language — each
entry names the concrete thing in SheetCalc that could not be built, or could
only be built badly.

---

## 1. Severity summary

| ID | Gap | Severity | Cost to fix |
|---|---|---|---|
| [BUG-TOSTR-1](#bug-tostr-1) | Interpolation bypasses user `toString` | **Critical** — silent wrong output | Small |
| [GAP-NUM-1](#gap-num-1) | `1/0` is `inf`, and no `isNan`/`isInf` to detect it | **Critical** — silent wrong output | Small |
| [GAP-STR-1](#gap-str-1) | The `"` character is unreachable | **High** — whole feature classes unbuildable | Small |
| [GAP-NUM-3](#gap-num-3) | `Number` has zero methods | **High** — every program reimplements math | Small |
| [BUG-ATTR-2](#bug-attr-2) | Install-tier `wrap` accepted, silently inert | **High** — silent no-op | Small (error) / Medium (land it) |
| [GAP-IO-1](#gap-io-1) | No I/O whatsoever | **High** — no program can be a tool | Medium |
| [GAP-FIB-1](#gap-fib-1) | Block combinators unusable inside a yielding fiber | **Medium** (revised down from High) | Small (docs) |
| [GAP-NUM-2](#gap-num-2) | No double dispatch / coercion on primitives | **Medium** — forces boxing | Medium |
| [GAP-ERR-1](#gap-err-1) | No `?`-style error propagation | **Medium** — 40% of parser is plumbing | Medium |
| [DIV-ATTR-1](#div-attr-1) | Attribute keyword args don't parse | **Medium** — spec examples don't run | Small |
| [GAP-STR-2](#gap-str-2) | `String` missing padding/case/predicates | **Medium** | Small |
| [GAP-FIB-2](#gap-fib-2) | `return` in a fiber block is a `DeadFrameError` | **Medium** — late-surfacing trap | Small (diagnostic) |
| [GAP-COL-1](#gap-col-1) | No `List#sort` | **Low** | Small |
| [GAP-SYN-1](#gap-syn-1) | Range literal `1..3` doesn't parse | **Low** | Small |
| [GAP-MOD-1](#gap-mod-1) | No selective import | **Low** | Medium |
| [GAP-CLS-1](#gap-cls-1) | No class-side instance variables | **Low** | Medium |
| [GAP-DX-1](#gap-dx-1) | Getter vs 0-arity method confusion | **Low** — onboarding | Small (diagnostic) |

**The two Criticals share a shape, and it is the important observation in this
document.** Both produce *silently wrong output* rather than an error. Every
other gap on this list announces itself: you get a `MessageNotUnderstood`, a
parse error, or a clean guarded diagnostic, and you fix it in a minute. These
two hand you a plausible-looking answer and let you ship it. `<Cell instance>`
in a rendered grid and `inf` propagating through a sheet are both the kind of
bug that survives a test suite that only asks "does it run".

Phalcom is, in general, **very good at failing loudly**. These are the places it
isn't, and they are worth more than the rest of the list combined.

---

## 2. What a real CLI program needs (the answer to "which primitives?")

SheetCalc cannot be a CLI program. Not "would be nicer as" — *cannot be*. It
cannot load a workbook, accept a formula, or tell a shell it failed. It is a
self-driving demo whose input is a Phalcom string literal.

Here is the minimum primitive set, ordered by what it unlocks. This is scoped as
a concrete proposal, not a wish list.

### Tier 1 — makes a program a program (unblocks the most, costs the least)

| Primitive | Signature | Unlocks |
|---|---|---|
| `System.readFile(_)` | `String -> Result<String, IOError>` | Load a workbook. Load *anything*. The single highest-value primitive on this list. |
| `System.writeFile(_, _)` | `String, String -> Result<None, IOError>` | Save. Golden-test self-update. |
| `System.args` | `-> List<String>` | `phalcom sheet.ph --input book.csv`. Any configurability at all. |
| `System.exit(_)` | `Number -> never` | A test runner that can fail CI. **Today a failing test suite cannot make the process exit non-zero.** |
| `System.readLine` | `-> Option<String>` | An interactive REPL; a `sheet>` prompt; piping stdin. |
| `System.stderr(_)` | `String -> None` | Diagnostics that don't corrupt stdout. Golden tests currently can't distinguish output from errors. |

> **Commentary.** `System.exit(_)` deserves emphasis out of proportion to its
> size. Phalcom has a golden-test corpus and a test-shaped culture, and a `.ph`
> test suite **cannot currently report failure to anything**. It can print
> "FAIL" and exit 0. Every `.ph`-level test lane is therefore forced to be a
> stdout-diff, which is why [10-testing.md](10-testing.md) is built the way it
> is. One primitive changes the whole testing story.

### Tier 2 — makes a program useful

| Primitive | Signature | Unlocks |
|---|---|---|
| `System.clock` | `-> Number` (monotonic seconds) | `@Timed`, `Backoff.waitBefore(_)` (which `core.ph` **already ships** with no clock to wait on), benchmarks, any profiling in `.ph`. |
| `System.now` | `-> Number` (epoch) | `NOW()`, `TODAY()` — real spreadsheet functions, currently unbuildable. |
| `Random` | `seed(_)`, `next` | `RAND()`, property-based tests, shuffles. Must be **seedable** or it breaks golden determinism. |
| `System.env(_)` | `String -> Option<String>` | Config. |

### Tier 3 — nice, not blocking

`System.sleep(_)` (needs a scheduler story), directory listing, process spawn.

**None of this is exotic.** It is the set every scripting language ships on day
one. The absence is not a design position — `system.md` and ADR-0019 clearly
anticipate these — it is simply unlanded work. But the effect on what can be
*written* in Phalcom today is total: **no Phalcom program can interact with
anything.**

---

## 3. The gaps, in detail

### <a id="bug-tostr-1"></a>BUG-TOSTR-1 — interpolation bypasses user `toString` (Critical)

```phalcom
class Cell { toString => "CELL" }
System.print(c)         // => CELL              -- sends #toString
System.print("\(c)")    // => <Cell instance>   -- does NOT
System.print([c])       // => [<Cell instance>] -- does NOT
```

`System.print(_)` uses `Value::to_display_string` (which sends the message);
interpolation and `List#toString` use `Value::to_string` (which doesn't).

**What it cost SheetCalc.** A whole-program prohibition on the language's most
idiomatic string construct, enforced by a source lint (REQ-VM-9/10). Every
render site is `"" + x.toString`. In a program whose entire output is rendered
user objects, naive interpolation would have corrupted every cell in every test.

**Why it's Critical.** No diagnostic. The wrong output just appears, and it
looks like a plausible debug rendering rather than a bug.

**Fix.** Route interpolation and `List#toString` through `to_display_string`.
The `core.ph` comment on `System.write` already acknowledges the divergence as
known. It should be closed, not documented.

---

### <a id="gap-num-1"></a>GAP-NUM-1 — `1/0` is `inf`, undetectably (Critical)

```phalcom
{ 1 / 0 }.attempt()   // => Ok(inf)     -- not an error
```

IEEE-754 says this is correct, so the *primitive* is defensible. What is not
defensible is the combination: `Number` has **no `isNan`, no `isInf`, no
`isFinite`**, so once you have an `inf` you cannot detect it. It renders as
`inf` and propagates through every downstream cell.

**What it cost SheetCalc.** REQ-VM-6: every `/` and `%` must zero-guard *before*
dividing, plus a defensive net in `Num.format(_)` rendering `inf`/`nan` as
`#NUM!` for anything that slips through. Two independent mechanisms to
compensate for one missing predicate.

**Fix (either).** Add `isNan`/`isInfinite`/`isFinite` to `Number` (small, and
needed regardless), or raise on division by zero. The first is better: it keeps
IEEE semantics and gives the user a way out.

---

### <a id="gap-str-1"></a>GAP-STR-1 — the `"` character is unreachable (High)

The only string escapes are `\\` and `\(`. There is no `\"`, no `\n`, no `\t`,
and no char-from-codepoint constructor (`String.new(34)` gives `"34"`).

> **A Phalcom program cannot emit a double quote. There is no workaround.**

**What it cost SheetCalc.** The formula language uses `'single quotes'` for
text literals, because fixtures are Phalcom string literals containing formulas
and `="hi"` is unwritable. A design decision in the *specified language* dictated
by a lexer gap in the *host language*.

The missing `\n` is separately expensive: you cannot build a multi-line string.
All rendering is line-at-a-time via `rawWrite` + `print`
([09-rendering.md](09-rendering.md)).

**Fix.** Add `\"`, `\n`, `\t`, `\r`, `\0`, `\u{...}` to the lexer, and/or a
`String.fromCodePoint(_)`. This is an afternoon's work and it removes a hard
wall. Note the current fallthrough — *backslash before any other character is a
literal backslash* — means adding escapes is a **breaking change** for any
program relying on `"C:\path"`, so it needs a deliberate call.

---

### <a id="gap-num-3"></a>GAP-NUM-3 — `Number` has zero methods (High)

The complete surface: `+ - * / % < <= > >= == negated hash toString`.

Absent: `floor ceil round abs sqrt truncate toInt min max pow isNan isInfinite
sign clamp`.

**What it cost SheetCalc.** `support/num.ph` exists solely to backfill this, and
the spreadsheet function library (`ABS`, `ROUND`, `SQRT`, `MIN`, `MAX`) must
**reimplement the host language's math library before it can implement its own**.
`SQRT` needs hand-rolled Newton's method. `ROUND` needs a hand-rolled
power-of-ten (no `pow`) and a hand-rolled `floor` — and the obvious
`n - (n % 1)` is **wrong for negatives** (it truncates toward zero rather than
flooring), which is exactly the kind of thing that ships.

Every one of these is a place to introduce a bug, in service of functions every
other language ships.

**Fix.** Add the standard set to `Number`. Highest ratio of value to effort on
this entire list.

---

### <a id="bug-attr-2"></a>BUG-ATTR-2 — Install tier accepted, silently inert (High)

`@On(Method, Install)` + `wrap(m)` parses, passes the correctness floor, and the
hook is **never called**. `M-INSTALL` is planned and unlanded, so this is
expected — but the floor is **asymmetric**: `wrap` *without* a tier raises
`attr.undeclared_hook`, while `wrap` *with* a tier silently no-ops. The floor
catches the lesser mistake and passes the greater one.

A user who follows the spec exactly gets a `@Memoize` that memoizes nothing.

**Fix (now).** Raise `attr.tier_not_implemented` at class-definition time until
`M-INSTALL` lands. **Fix (real).** Land `M-INSTALL`. See
[11-decorators.md](11-decorators.md).

---

### <a id="gap-fib-1"></a>GAP-FIB-1 — block combinators are unusable inside a yielding fiber (Medium)

> **Severity revised down from High/architectural.** The original entry claimed
> fibers and the collection API were mutually exclusive. That was **wrong** — an
> artifact of a probe harness that wrapped every call in `{ ... }.attempt()`,
> itself a native block frame. Post-mortem in
> [00-language-findings.md §8](00-language-findings.md).

```phalcom
Fiber.new { [1,2,3].each { x => Fiber.yield(x) } }.call()
// => cannot switch fibers across a native call frame (e.g. inside .each { })

Fiber.new { for (x in [1,2,3]) { Fiber.yield(x) } }.call()
// => 1     (works; and on a user-defined Iterable too)
```

A yield cannot cross a **native call frame**, and `Block#call` is one. So
`each`/`map`/`where`/`filter`/`reduce` are unsafe inside a yielding fiber, while
`for` and `while` are safe.

**What it cost SheetCalc.** Less than first thought. A demand-driven evaluator
must use `for` rather than the block combinators — `SUM`'s
`range.reduce(0) { }` becomes a `for` loop. A real style tax, not a blocker. v1
uses an explicit topological sweep because it is simpler, not because fibers are
unavailable.

**What remains genuinely wrong.** The constraint is invisible at the point of
use: `range.reduce(0) { }` and `for (r in range) { }` look equally innocent and
only one works. Neither `iteration.md` nor `concurrency.md` mentions the other's
constraint.

**Fix.** Mostly documentation: state the rule in both features' specs. A real
fix (reifying native block frames so they can be suspended) is large and, at
this revised severity, probably not worth it.

> **Commentary.** Worth recording that the runtime came out of this *better*
> than the investigator. The diagnostic names the mechanism and the canonical
> example — `cannot switch fibers across a native call frame (e.g. inside
> .each { })` — which is precisely the information needed, and I still managed
> to over-generalize it by trusting my harness over the error text.

---

### <a id="gap-num-2"></a>GAP-NUM-2 — no double dispatch or coercion on primitives (Medium)

```phalcom
{ 1 + userObject }.attempt()   // => Err(<Error>).  Unfixable from .ph.
```

`Number#+` is native, type-checks, and raises. No multimethods, no `coerce:`,
no `__radd__`, no way to override it.

**What it cost SheetCalc.** DEC-VM-1: **every** cell value — including plain
numbers — must be a heap-allocated user object, so that the receiver of every
arithmetic send is under our control. One five-line probe dictated the
allocation strategy of the entire program, and allocation is this runtime's #1
measured cost.

**Prior art.** Smalltalk: `retryRelationalOp:coercing:` — the primitive fails
and re-dispatches. Python: `__radd__`. Ruby: `coerce`. Phalcom has no
equivalent.

**Fix.** On a primitive-argument type mismatch, re-dispatch to the argument
(e.g. send `#addedTo(_)` / a `retry:coercing:` equivalent) before raising. This
would let user classes cooperate with native numbers and would make boxing a
*choice* rather than a requirement.

---

### <a id="gap-err-1"></a>GAP-ERR-1 — no `?`-style error propagation (Medium)

`Result` is well-built (`map`/`mapErr`/`andThen`/`match(ok:err:)`), but there is
no syntactic affordance, so every parser frame writes:

```phalcom
let lhs = self.parsePrimary()
if (lhs.isErr) { return lhs }
let l = lhs.unwrapOr(None)
```

`andThen` nests rather than sequences, which a Pratt loop can't use.

**What it cost SheetCalc.** Roughly **40% of the parser's line count is error
plumbing**. A recursive-descent parser is the canonical `Result`-heavy program,
and it pays a 3-line tax per frame.

**Fix.** A `?`-style postfix operator, or a `do`-notation equivalent. Note this
interacts with the language's no-exceptions position — which is a *good*
position; it just needs the ergonomics that make it livable.

---

### <a id="div-attr-1"></a>DIV-ATTR-1 — attribute keyword args don't parse (Medium)

The spec documents `@Author(name: "Ada")` and `@On(Method, tier: Install)`.
Neither parses. The gap is narrow and precisely located: labeled parameters work
in declarations (`@constructor
new(name:)`) and at normal call sites
(`Author.new(name: "Ada")`). **Only the attribute call-site parser lacks them.**

**Fix.** Reuse the existing keyword-argument parser at attribute call sites. The
spec is the newer document; the parser is behind it.

---

### <a id="gap-str-2"></a>GAP-STR-2 — `String` missing the basics (Medium)

Absent: `padLeft`/`padRight`, `toUpper`/`toLower`, `at(_)`, `reversed`,
`startsWith`/`endsWith`/`contains`, `isDigit`/`isAlpha` character predicates.

**What it cost SheetCalc.** `support/str.ph` (padding for grid rendering), and
the lexer hand-rolls `isDigit`/`isAlpha` from codepoint ranges. `UPPER()`/
`LOWER()` need codepoint arithmetic, which means they are ASCII-only unless
someone writes a Unicode case table — in a language whose `String` is otherwise
carefully Unicode-correct (`codePointAt` does real multi-byte decoding).

**Fix.** Grow `String`. Same argument as GAP-NUM-3.

---

### <a id="gap-fib-2"></a>GAP-FIB-2 — `return` in a fiber block is a `DeadFrameError` (Medium)

```phalcom
Fiber.new {
  Fiber.yield(1)
  return "done"     // DeadFrameError on the FINAL resume
}
```

`return` in a block is a non-local return to the block's home method frame,
which for a fiber body is long dead. The implicit last expression is the correct
idiom and works.

**Why it's worse than it looks.** The fiber runs correctly for every `yield` and
only explodes on the *final* resume. A shallow test passes.

**Fix.** The diagnostic is accurate but arrives late. Detect a `return` in a
fiber-body block at compile time, or at least name the idiom in the error text.

---

### Low severity

- **<a id="gap-col-1"></a>GAP-COL-1 — no `List#sort`.** `support/sort.ph` ships a merge sort. Every non-trivial program needs this.
- **<a id="gap-syn-1"></a>GAP-SYN-1 — range literal `1..3` doesn't parse.** `..` *lexes* (`DotDot` token exists) but the parser doesn't accept it; `Range.new(a, b, true)` is the only path. Half-landed syntax.
- **<a id="gap-syn-2"></a>GAP-SYN-2 — `return [1, 2, 3]` doesn't parse.** `return` followed immediately by a list literal errors with `Expected one of ";", newline` at the `[`. `var l = [...]` + `return l` works, as does the arrow form `f => [...]`. A pure parser bug with a silly workaround, and one an implementer hits within an hour of writing real code — any factory method returning a literal list.
- **<a id="gap-str-3"></a>GAP-STR-3 — no character can be constructed from a number.** Sharper than GAP-STR-1: there is no codepoint-to-character path *at all*, so no string transformation is *computable*. `UPPER()`/`LOWER()` cannot shift a codepoint; they require a hand-spelled 52-entry literal lookup table. Generating column labels (`A`, `B`, ... `AA`) requires slicing a literal alphabet string. **Any output character must already exist as a literal in the source.**
- **<a id="gap-mod-1"></a>GAP-MOD-1 — no selective import.** `import "./x" as N` binds the whole module; no `import a, b from "./x"` (`from`/`export` reserved but unlexed). Every cross-module reference is qualified, which in a deep layer stack is a real readability tax.
- **<a id="gap-cls-1"></a>GAP-CLS-1 — no class-side instance variables.** No `static var`. Singleton caches must live in module-level `var`s. Odd in a language with a full metaclass tower.
- **<a id="gap-dx-1"></a>GAP-DX-1 — getter vs 0-arity method.** `f.call` and `f.call()` are different selectors. Correct by design, but `<fiber> does not understand 'call'` when `call` is right there in the primitive table is a baffling first encounter. **Fix:** when a send misses, check whether the other signature kind exists and say so — "`<fiber>` has no getter `call`; did you mean `call()`?" Cheap, high-value.

---

## 4. What would have eased my life most

Ranked by how much of this exercise's pain each removes:

1. **`System.readFile` + `System.args` + `System.exit`.** Turns "a demo that prints" into "a program". Unblocks the most for the least.
2. **Grow `Number` and `String`.** Deletes `support/` entirely, and with it every hand-rolled `floor`/`round`/`sqrt`/`pad` bug. Pure win, no design risk.
3. **Fix BUG-TOSTR-1 and GAP-NUM-1.** The two silent-wrong-answer bugs. Small fixes; they remove the only two traps on this list that a test suite wouldn't catch.
4. **String escapes (`\"`, `\n`).** Removes a hard wall.
5. **Land `M-INSTALL`, or fail loudly until it lands.** Makes the decorator story real, or at least honest.
6. **A `?` operator for `Result`.** Removes ~40% of the parser.
7. **Address GAP-FIB-1.** The biggest and hardest. Until then, fibers and the collection API are separate languages, and that should at minimum be documented in both features' specs.

## 5. What I would tell the language's designer

The object model is not the problem. It is the best part of this language: the
metaobject protocol is complete, keyword constructors beat every mainstream
language's constructor overloading, sealed `Option` kills a bug class, operator
overloading is clean, inheritance is solid, and the diagnostics — where they
exist — are better than most production runtimes.

**Every serious wound in this document is a library gap or a half-landed
feature, not a design error.** `Number` with zero methods. `String` without
padding. No file I/O. An attribute system whose behavioral half isn't wired. A
range literal that lexes but doesn't parse. Those are all *finishing* problems,
and finishing problems are the good kind to have.

The two things that genuinely worry me are different in kind:

1. **The two silent-wrong-answer bugs** (BUG-TOSTR-1, GAP-NUM-1). A language
   whose culture is this strong on correctness floors and honest diagnostics has
   exactly two places where it hands you a wrong answer with a straight face.
   They should be zero.

2. **GAP-FIB-1**, because it is the one finding that is *architectural* rather
   than unfinished. Two flagship features that cannot share a call stack is not
   a missing function; it is a seam in the design. It deserves a deliberate
   decision — fix it, or document it prominently in both features' specs — rather
   than remaining a thing users discover by writing the obvious code.
