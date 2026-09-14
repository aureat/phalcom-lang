# SheetCalc — Traceability Matrix

Part of the [SheetCalc specification](README.md).

Every requirement maps to the document that defines it and the test that proves
it. A `REQ` with no test is a spec bug. A test with no `REQ` is either a missing
requirement or a redundant test.

**Totals:** 102 requirements across 15 areas; 29 findings (`GAP`/`BUG`/`DEC`/`DIV`).

---

## 1. How to use this document

1. **Implementing a unit?** Find its area below, read the owning document, and
   check off each `REQ` with a test before calling it done.
2. **Adding a requirement?** Add the `REQ-<AREA>-<n>` to the owning document
   *and* a row here, in the same change.
3. **Changing behavior?** Find the `REQ` that pins it. If none exists, the
   behavior was never specified and you are writing a new requirement, not
   changing an old one.

**REQ-TRACE-1.** Every `REQ-*` in any SheetCalc document appears in §2 with a
named test.
**REQ-TRACE-2.** Every test file in `test/suites/` names at least one `REQ-*` in
its header comment.
**REQ-TRACE-3.** No `REQ-*` may be marked satisfied by a test that does not
execute the code path it constrains. Compile-only coverage does not count.

---

## 2. Requirements by area

| Area | Count | Owning document | Suite |
|---|---|---|---|
| `REQ-ARCH-*` | 2 | [01-architecture.md](01-architecture.md) | `suites/lint_layering.ph` (external check) |
| `REQ-VM-*` | 12 | [02-value-model.md](02-value-model.md) | `suites/value_*.ph` |
| `REQ-REF-*` | 9 | [03-references-and-grid.md](03-references-and-grid.md) | `suites/ref_*.ph` |
| `REQ-GRID-*` | 9 | [03-references-and-grid.md](03-references-and-grid.md) | `suites/grid_*.ph` |
| `REQ-LEX-*` | 10 | [04-formula-lexer.md](04-formula-lexer.md) | `suites/lex_*.ph` |
| `REQ-PARSE-*` | 9 | [05-formula-parser.md](05-formula-parser.md) | `suites/parse_*.ph` |
| `REQ-AST-*` | 5 | [06-ast-and-eval.md](06-ast-and-eval.md) | `suites/ast_*.ph` |
| `REQ-EVAL-*` | 5 | [06-ast-and-eval.md](06-ast-and-eval.md) | `suites/eval_*.ph` |
| `REQ-DEP-*` | 5 | [07-dependency-graph-and-recalc.md](07-dependency-graph-and-recalc.md) | `suites/dep_*.ph` |
| `REQ-RECALC-*` | 7 | [07-dependency-graph-and-recalc.md](07-dependency-graph-and-recalc.md) | `suites/recalc_*.ph` |
| `REQ-FN-*` | 5 | [08-functions.md](08-functions.md) | `suites/fn_*.ph` |
| `REQ-RENDER-*` | 6 | [09-rendering.md](09-rendering.md) | `fixtures/*.golden` |
| `REQ-TEST-*` | 8 | [10-testing.md](10-testing.md) | self-hosted |
| `REQ-DEC-*` | 3 | [11-decorators.md](11-decorators.md) | `suites/decorator_*.ph` |
| `REQ-PAT-*` | 1 | [12-design-patterns.md](12-design-patterns.md) | `suites/lint_layering.ph` |
| `REQ-TRACE-*` | 3 | this document | external check |

### The critical path

These pin the findings that would otherwise silently produce wrong output. If
only a handful of tests are ever written, write these.

| REQ | Pins | Finding | Why it matters |
|---|---|---|---|
| REQ-VM-6 | `/0` and `%0` return `#DIV/0!`, never `inf` | GAP-NUM-1 | The runtime gives `inf` **silently**. Nothing else catches this. |
| REQ-VM-8 | `Num.format` never emits `inf`/`nan` | GAP-NUM-1 | Defence in depth for the above. |
| REQ-VM-9/10 | no interpolation of domain objects | BUG-TOSTR-1 | Would corrupt every rendered cell, with no diagnostic. |
| REQ-VM-3/4/5 | error absorption in both directions | DEC-VM-1 | The reason the value model exists at all. |
| REQ-VM-7 | `0.1 + 0.2` renders `0.3` | GAP-NUM-3 | Hand-rolled rounding is the likeliest home for a subtle bug. |
| REQ-RECALC-* (cycles) | every cell in a cycle gets `#CIRC!` | — | The classic spreadsheet correctness case. |

---

## 3. Findings index

Where each finding is defined, and what pins it.

### Bugs — file these independently of SheetCalc

| ID | Finding | Defined in | Pinned by |
|---|---|---|---|
| BUG-TOSTR-1 | Interpolation and `List#toString` bypass user `toString` | [00 §6](00-language-findings.md) | REQ-VM-9/10 |
| BUG-ATTR-2 | Install-tier `wrap` accepted, silently inert | [11 §1](11-decorators.md) | `suites/attr_install_inert.ph` (a **pinning** test: asserts the hook is NOT called, so it fails loudly when `M-INSTALL` lands) |

### Divergences — spec and implementation disagree

| ID | Finding | Defined in |
|---|---|---|
| DIV-ATTR-1 | `@Attr(label: value)` documented but doesn't parse | [11 §1](11-decorators.md) |

### Gaps — worked around

| ID | Gap | Defined in | Workaround |
|---|---|---|---|
| GAP-IO-1 | No I/O at all | [13 §2](13-language-gaps.md) | Fixtures as source literals; no CLI |
| GAP-NUM-1 | `1/0` is `inf`, undetectable | [00 §3](00-language-findings.md) | Explicit zero-guards + format net |
| GAP-NUM-2 | No double dispatch on primitives | [00 §4](00-language-findings.md) | DEC-VM-1 (box everything) |
| GAP-NUM-3 | `Number` has no instance methods | [00 §3](00-language-findings.md) | `support/num.ph` |
| GAP-STR-1 | `"` unreachable; no `\n` | [00 §5](00-language-findings.md) | `'single quotes'`; line-at-a-time output |
| GAP-STR-2 | `String` lacks padding/case/predicates | [13](13-language-gaps.md) | `support/str.ph` |
| GAP-STR-3 | No character constructible from a number | [13](13-language-gaps.md) | Literal alphabet + `rawSlice` |
| GAP-LEX-1/2 | No `isDigit`/`isAlpha`; no byte→codepoint index | [04](04-formula-lexer.md) | Hand-rolled from codepoint ranges |
| GAP-ERR-1 / GAP-PARSE-1 | No `?`-style propagation | [13](13-language-gaps.md), [05 §5](05-formula-parser.md) | Manual `isErr` check per frame |
| GAP-FIB-1 | Block combinators unsafe in a yielding fiber | [00 §8](00-language-findings.md) | v1 is fiber-free; use `for` if that changes |
| GAP-FIB-2 | `return` in a fiber block is `DeadFrameError` | [00 §8](00-language-findings.md) | Implicit last expression |
| GAP-COL-1 | No `List#sort` | [13](13-language-gaps.md) | `support/sort.ph` |
| GAP-SYN-1 | `1..3` doesn't parse | [00 §7](00-language-findings.md) | `Range.new(a, b, true)` |
| GAP-SYN-2 | `return [1,2,3]` doesn't parse | [00 §10](00-language-findings.md) | Bind to a local first |
| GAP-MOD-1 | No selective import; `Name.Name` stutter | [01 §2](01-architecture.md) | Module-level alias after import |
| GAP-CLS-1 | No class-side instance variables | [12 §4](12-design-patterns.md) | Module-level `var` |
| GAP-DX-1 | Getter vs 0-arity method | [00 §10](00-language-findings.md) | Know which; read the primitive table's `SignatureKind` |

### Decisions

| ID | Decision | Defined in | Forced? |
|---|---|---|---|
| DEC-VM-1 | Every cell value is a user-class instance | [02 §1](02-value-model.md) | **Yes** — by GAP-NUM-2 |
| DEC-REF-1 | `Ref` absolute/relative modelling | [03](03-references-and-grid.md) | No |
| DEC-PARSE-1 | Pratt parser | [05](05-formula-parser.md) | No |
| DEC-FN-2/3 | `Map<String, Fn>` dispatch, not `perform` | [08](08-functions.md) | **Yes** — selectors are arity-encoded, so a name-derived symbol never matches |
| DEC-PAT-1 | Polymorphic `eval`, not Visitor | [12 §1](12-design-patterns.md) | No |
| DEC-PAT-2 | `Grid#at(_)` returns `CellEmpty`, not `Option` | [12 §2](12-design-patterns.md) | No |

---

## 4. Corrections log

Claims this spec made and later retracted. Kept visible because a spec that
hides its own error rate is not trustworthy, and because each of these is a
lesson about method.

| Claim | Status | Caught by | Root cause |
|---|---|---|---|
| "Fibers and the collection API are mutually exclusive" | **RETRACTED** — `for` is fiber-safe; only `Block#call` combinators are not | A reviewer's contradicting probe | **The probe harness was the bug.** Every fiber call was wrapped in `{ ... }.attempt()`, itself a native block frame. It made safe constructs fail uniformly, and the uniformity read as a rule. |
| "`Number` has zero methods" | **CORRECTED** — true for instances; `Number.new(_)` parses strings, including scientific notation | A reviewer reading the class side of the primitive table | I enumerated the instance rows and called it "the complete list". |
| "`Ref` as a `Map` key will break `hash`/`==`" | **REFUTED** (pre-spec prediction) | Probe | Works correctly. |
| "Deep recursion will hit a ceiling" | **REFUTED** (pre-spec prediction) | Probe | 50 000 frames fine. |
| `ErrorVal.of(#VALUE).raise_` in 02's sample | **FIXED** — no such selector; `raise()` exists only on `Error` subclasses | A reviewer's probe | Wrote plausible-looking code without running it. |
| "`test/framework.ph` includes a source lint" | **FIXED** — a `.ph` program cannot read files; the lint must be external | A reviewer cross-checking against findings §2 | Forgot my own §2 while writing §6 of another document. |
| `Set#contains(_)` | **CORRECTED** — the selector is `includes(_)` | Probe | Assumed a spelling. |
| 08's "guard `.num` or it crashes" | **CORRECTED** — it degrades to `#VALUE!`; the guard is still required, for error-kind fidelity | Follow-on from the `raise_` fix | A downstream document built a load-bearing argument on an upstream sketch that had never been run. |

> **Commentary.** Eight corrections, and the pattern is not random. Two came from
> *not running the code* (`raise_`, `Set#contains`). Two came from *stopping an
> enumeration early* (`Number`'s class side, the lint's feasibility). One — the
> big one — came from *a harness that contaminated its own measurement*, and it
> was the single most-emphasized claim in the whole spec, repeated across four
> documents, before anyone checked it.
>
> The eighth is the most worrying, because it shows the others *compounding*:
> 08-functions.md built a "this is load-bearing, never skip it" argument on top
> of 02's unrun `raise_` sketch. A wrong fact in an upstream document became a
> confident downstream rule. Nobody wrote anything careless; the error simply
> inherited. That is the argument for the probe log existing as a **separate,
> executable-facts-only document** that every other document must cite rather
> than reason from memory.
>
> The two pre-spec predictions that were **refuted** (`Ref` hashing, recursion
> depth) are worth as much as the confirmed ones: both were things I was
> confident would break, and both work fine. Confidence was not correlated with
> correctness in either direction here.
>
> If there is one methodological rule to carry out of this exercise, it is the
> one that would have caught the most: **run it bare before you generalize.**
> Convenience wrappers around a probe are a hypothesis about the runtime, not a
> neutral observation of it.
