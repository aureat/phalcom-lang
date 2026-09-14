# SheetCalc — a spreadsheet engine in Phalcom

**Status:** Specification, Draft 1. No implementation yet.
**Verified against:** `main` @ `5516504`, release build, 2026-07-14.

SheetCalc is a spreadsheet engine written in Phalcom: a cell grid, a formula
language with a real lexer and parser, a dependency graph, topological
recalculation with cycle detection, a builtin function library, and a rendered
grid output.

## Why this exists

SheetCalc is **not** a product. It is a deliberately-chosen stress test whose
job is to break Phalcom and tell us where.

The selection criteria were: exercise the whole language surface at once, be a
real program rather than a feature checklist, and put load on the parts of the
language that the existing test corpus does not reach. A spreadsheet does all
three. It needs a class hierarchy with polymorphic dispatch (AST nodes, cell
values), user classes as hash keys (cell references), deep recursion (a
recursive-descent parser), error values that propagate through arithmetic,
lazy iteration (range functions like `SUM`), string surgery (the formula
lexer), reflection (the test runner), and — if recalculation is made
demand-driven — fibers.

This document set is **a specification plus a commentary**. The specification
half says what to build. The commentary half — carried in
[00-language-findings.md](00-language-findings.md) and
[13-language-gaps.md](13-language-gaps.md), and in `> Commentary` blocks
throughout — records what the language made hard, what had to be worked around,
and what should exist but doesn't. The commentary is the more valuable half.
The engine is the excuse; the findings are the product.

## Read in this order

| # | Document | What it settles |
|---|---|---|
| 00 | [Language findings](00-language-findings.md) | **Read first.** The probe log: what the language actually does, established by running code. Every other document is grounded in this one. |
| 01 | [Architecture](01-architecture.md) | Layer map, module boundaries, data flow, file layout. |
| 02 | [Value model](02-value-model.md) | `CellValue` hierarchy, error propagation, why cells cannot be native numbers. |
| 03 | [References and grid](03-references-and-grid.md) | `Ref`, A1 notation, the `Grid` store. |
| 04 | [Formula lexer](04-formula-lexer.md) | Tokenizing formula text. |
| 05 | [Formula parser](05-formula-parser.md) | Pratt parser, `Result`-based error handling. |
| 06 | [AST and evaluation](06-ast-and-eval.md) | Node hierarchy, polymorphic `eval`. |
| 07 | [Dependency graph and recalc](07-dependency-graph-and-recalc.md) | Topological order, cycle detection, the fiber question. |
| 08 | [Function library](08-functions.md) | `SUM`, `IF`, `COUNTIF`, and the dispatch mechanism. |
| 09 | [Rendering](09-rendering.md) | Grid output, number formatting, the padding problem. |
| 10 | [Testing](10-testing.md) | Attribute-driven test runner, golden corpus, traceability. |
| 11 | [Runtime decorators](11-decorators.md) | What decorators are possible, and what isn't. |
| 12 | [Design patterns](12-design-patterns.md) | Boilerplate reduction; which classic patterns survive contact with Phalcom. |
| 13 | [Language gaps and wishlist](13-language-gaps.md) | **The payload.** What I needed, what I worked around, what Phalcom should have. |
| 14 | [Traceability matrix](14-traceability.md) | Requirement → spec § → test. Also the **corrections log**: every claim this spec made and retracted, and why. |

## The four findings that shaped everything

Established in [00-language-findings.md](00-language-findings.md), repeated here
because they are load-bearing and counterintuitive:

1. **`1 + userObject` is unfixable.** `Number#+` is a native primitive that
   raises on a non-number argument. No multimethods, no coercion protocol, no
   overriding it from `.ph`. A spreadsheet must propagate `#DIV/0!` through
   `+`, so **cell values cannot be native numbers** — every one is a user-class
   instance. Forced by the language, not chosen. (DEC-VM-1)

2. **The `"` character is unreachable.** The only string escapes are `\\` and
   `\(`. There is no `\"`, and no char-from-codepoint constructor. A Phalcom
   program cannot emit a double quote. SheetCalc's formula grammar therefore
   uses `'single quotes'` for text literals. (GAP-STR-1)

3. **String interpolation silently bypasses user `toString`.** `System.print(c)`
   prints `CELL`; `System.print("\(c)")` prints `<Cell instance>`. The most
   idiomatic rendering construct in the language produces wrong output for
   exactly the objects you most want to render, with no diagnostic. SheetCalc
   forbids interpolating user objects anywhere. (BUG-TOSTR-1)

4. **Block-taking combinators are unusable inside a yielding fiber.**
   `[1,2,3].each { x => Fiber.yield(x) }` raises
   `CannotYieldAcrossNativeFrame`, because `Block#call` is a native frame.
   `each`/`map`/`where`/`filter`/`reduce` are all affected. **`for` and `while`
   are safe** — verified on native `List`s and user `Iterable`s alike. A clean
   guarded diagnostic, not a crash. (GAP-FIB-1)

   > This item said something much stronger in the first draft ("fibers and the
   > collection API are mutually exclusive"). That was **wrong, and wrong
   > because of a flawed probe** — the harness wrapped every call in
   > `{ ... }.attempt()`, which is itself a native block frame, so everything
   > failed uniformly. A second party's contradicting probe caught it. The full
   > post-mortem is in [00-language-findings.md §8](00-language-findings.md);
   > it is left in deliberately, because how a plausible finding became a
   > four-document architectural claim is worth more than the finding.

## What SheetCalc deliberately is not

**It is not a CLI program, because it cannot be one.** Phalcom has no file
read, no stdin, no clock, no random, no argv, and no exit code. `System`'s
entire surface is `print`, `rawWrite`, `schedule`, and `nextScheduled`. Every
workbook is a literal in the source; the only output is stdout.

So SheetCalc is specified as a **self-driving demonstration**: fixture workbooks
declared in `.ph` source, evaluated, rendered to stdout, and diffed against
golden files by the existing test lanes. The primitive set that would make it a
real tool is specified in [13-language-gaps.md §2](13-language-gaps.md) — that
list is one of the main deliverables here, because "what does a real program
need that we don't have" is answerable only by trying to write one.

## Scope

**In scope (v1).** Number/text/bool/empty/error cell values; formulas with
`+ - * / %`, comparisons, parentheses; A1 and `$A$1` references; ranges
(`A1:B7`); the function set in [08-functions.md](08-functions.md); dependency
tracking; topological recalc; cycle detection (`#CIRC!`); error propagation;
grid rendering; a golden test suite.

**Out of scope (v1).** Interactive editing (no stdin). Persistence (no file
I/O). `NOW`/`TODAY`/`RAND` (no clock, no random). Multi-sheet workbooks.
Formatting/styling. Array formulas.

**Deferred pending a language decision.** Demand-driven recalculation on fibers
— see [07-dependency-graph-and-recalc.md §6](07-dependency-graph-and-recalc.md).
GAP-FIB-1 makes the natural implementation impossible, and the decision of
whether to hand-roll around it or to fix the runtime is the user's, not the
spec's.

## Running it (once implemented)

```sh
cargo build --release
./target/release/phalcom docs/apps/sheetcalc/src/main.ph
```

Golden tests run through the existing `.ph` corpus lanes — see
[Phalcom golden-test lanes](../../forge/README.md) and
[10-testing.md](10-testing.md).

## Conventions used in these documents

- `VERIFIED-PRESENT` / `VERIFIED-ABSENT` — established by running a probe, per
  [00-language-findings.md](00-language-findings.md).
- `DEC-*` — a design decision, with the forcing reason recorded.
- `GAP-*` — a language gap that required a workaround.
- `BUG-*` — a runtime defect worth filing independently.
- `DIV-*` — a spec-vs-implementation divergence.
- `REQ-*` — a requirement, tracked in [14-traceability.md](14-traceability.md).
- `> Commentary` blocks — the developer's-eye view: what hurt and why.
