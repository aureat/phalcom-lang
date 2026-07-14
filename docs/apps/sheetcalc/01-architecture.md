# SheetCalc — Architecture

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md).

## 1. Layer map

Data flows one way. Each layer depends only on layers below it.

```
                 ┌─────────────────────────────┐
  fixtures/*.ph  │  main.ph — self-driving demo│   (no stdin: workbook is source)
                 └──────────────┬──────────────┘
                                │
        ┌───────────────────────▼───────────────────────┐
  L7    │  render/   Grid → stdout, number formatting    │
        └───────────────────────┬───────────────────────┘
        ┌───────────────────────▼───────────────────────┐
  L6    │  recalc/   dep graph, topo order, cycles       │
        └───────────────────────┬───────────────────────┘
        ┌───────────────────────▼───────────────────────┐
  L5    │  eval/     AST eval, function library          │
        └───────────────────────┬───────────────────────┘
        ┌───────────────────────▼───────────────────────┐
  L4    │  parse/    Pratt parser → Ast, Result errors   │
        └───────────────────────┬───────────────────────┘
        ┌───────────────────────▼───────────────────────┐
  L3    │  lex/      formula text → tokens               │
        └───────────────────────┬───────────────────────┘
        ┌───────────────────────▼───────────────────────┐
  L2    │  grid/     Ref, Cell, Grid store               │
        └───────────────────────┬───────────────────────┘
        ┌───────────────────────▼───────────────────────┐
  L1    │  value/    CellValue hierarchy (DEC-VM-1)      │
        └───────────────────────┬───────────────────────┘
        ┌───────────────────────▼───────────────────────┐
  L0    │  support/  Num, Str, Sort — the missing stdlib │
        └───────────────────────────────────────────────┘
```

> **Commentary — why there is an L0 at all.**
> `support/` exists entirely to backfill things a language of this maturity
> would normally ship: `floor`, `abs`, `round`, `min`, `max` (`Number` has
> **zero** methods — see findings §3), string padding (`String` has no
> `padLeft`), and a sort (`List` has no `sort`). None of this is
> spreadsheet-specific. It is ~200 lines of code that exists only because the
> core library doesn't have it, and it is the clearest single argument in this
> exercise for growing `Number` and `String`. See
> [13-language-gaps.md §3](13-language-gaps.md).

## 2. File layout

Modules are files; `import "./path" as Name` is verified working (findings §7
of the module probe — a two-file import round-trip was confirmed to run).
Binding is whole-module and `as Name` is mandatory.

```
docs/apps/sheetcalc/src/
  main.ph                  -- entry: builds fixtures, evaluates, renders
  support/
    num.ph                 -- Num: floor, ceil, round, abs, min, max, isInt
    str.ph                 -- Str: padLeft, padRight, repeat, startsWith
    sort.ph                -- Sort: by(list, comparator) — merge sort
  value/
    cell_value.ph          -- CellValue root + Num/Text/Bool/Empty/ErrorVal
  grid/
    ref.ph                 -- Ref, A1 encode/decode
    cell.ph                -- Cell: literal | formula, cached value
    grid.ph                -- Grid: Map<Ref, Cell>
  lex/
    token.ph               -- Token + kinds
    lexer.ph               -- Lexer: String → List<Token>
  parse/
    ast.ph                 -- Ast node hierarchy
    parser.ph              -- Parser: List<Token> → Result<Ast, ParseError>
  eval/
    evaluator.ph           -- Ast#eval(ctx)
    functions.ph           -- FunctionTable: SUM, IF, ...
  recalc/
    depgraph.ph            -- DepGraph: forward + reverse edges
    engine.ph              -- Engine: topo order, cycle detect, recalc
  render/
    renderer.ph            -- Grid → stdout
  test/
    framework.ph           -- @Test attribute + runner (reflection)
    suites/*.ph            -- the suites
fixtures/
  *.golden                 -- expected stdout
```

> **Commentary — one import gap.** `import "./x" as Name` binds the *whole
> module*; there is no selective `import a, b from "./x"` (modules.md §3 —
> `from`/`export` are reserved but unlexed). So every cross-module reference is
> qualified: `Value.CellNum.of(1)`, not `CellNum.of(1)`. In a program with a
> deep layer stack this is a real readability tax — `eval/evaluator.ph` refers
> to `Value.`, `Grid.`, `Ast.`, and `Fn.` constantly. Not a blocker; recorded
> as GAP-MOD-1.

## 3. Module dependency rules

| Module | May import |
|---|---|
| `support/*` | nothing |
| `value/*` | `support/` |
| `grid/*` | `support/`, `value/` |
| `lex/*` | `support/` |
| `parse/*` | `support/`, `lex/`, `grid/` (for `Ref` literals) |
| `eval/*` | `support/`, `value/`, `grid/`, `parse/` |
| `recalc/*` | `support/`, `value/`, `grid/`, `parse/`, `eval/` |
| `render/*` | `support/`, `value/`, `grid/` |
| `main.ph` | all |

Cycles are forbidden. `parse/` depending on `grid/` for `Ref` is the one edge
that looks wrong and is deliberate: an `A1` token must become a `Ref` at parse
time, and duplicating `Ref` into `parse/` would be worse.

**REQ-ARCH-1.** No module may import a module above it in the L0–L7 stack.
**REQ-ARCH-2.** `support/` must contain nothing spreadsheet-specific — it is
the "missing stdlib" shim and must be liftable into `core.ph` unchanged. This
constraint is the point: whatever ends up in `support/` is the concrete
proposal for what `core.ph` should grow.

## 4. Data flow — one cell, end to end

```
  "=SUM(A1:A3)*2"                       source text (a Phalcom string literal)
        │  Lexer.tokenize(_)
        ▼
  [Fn(SUM), LParen, Ref(A1), Colon, Ref(A3), RParen, Star, Number(2)]
        │  Parser.parse(_)              → Result<Ast, ParseError>
        ▼
  Mul(Call(SUM, [RangeLit(A1, A3)]), Lit(CellNum(2)))
        │  Engine.dependenciesOf(_)     walks the Ast for Refs
        ▼
  {A1, A2, A3}                          → DepGraph edges
        │  Engine.recalc()              topo order, then per node:
        ▼
  ast.eval(ctx)                         → CellValue
        │
        ▼
  CellNum(12)                           cached on the Cell
        │  Renderer.render(_)
        ▼
  "|    12 |"                           stdout
```

## 5. The error model, in one place

Two *different* error channels, deliberately not unified. Getting this wrong is
the most likely design mistake in the program, so it is stated up front.

| | Channel | Type | Why |
|---|---|---|---|
| **Structural failure** | `Result<Ast, ParseError>` | `Err` | A malformed formula is a *program* error. It has a position, a message, and it aborts the parse. |
| **Value-level failure** | `ErrorVal` (a `CellValue`) | a normal value | `#DIV/0!` is *data*. It flows through `+`, gets stored in cells, renders in the grid, and is what `ISERROR` tests. It must not abort anything. |

A spreadsheet's `#DIV/0!` is emphatically **not** an exception. It is a first-class
value with arithmetic behavior (`#DIV/0! + 1` is `#DIV/0!`). Modelling it as a
`Result` or as a raised error would be a category error and would fight the
engine at every step.

> **Commentary — this is where the language pushed back hardest.**
> Because `ErrorVal` must survive `+`, and because `1 + userObject` raises
> unfixably (findings §4), the error-value requirement propagates all the way
> down and forces **every** cell value — including plain numbers — to be a user
> class. One language limitation, discovered in a five-line probe, dictates the
> allocation strategy of the entire program. See
> [02-value-model.md](02-value-model.md).

## 6. Concurrency posture

**v1 is single-threaded and fiber-free.** Recalculation is an explicit
topological sweep.

The natural design — demand-driven evaluation where hitting an uncomputed
dependency yields to a scheduler — is **constrained, but not blocked**, by
GAP-FIB-1.

> **CORRECTED.** This section previously claimed demand-driven recalc was
> blocked outright and implementable "only by hand-rolling every loop as an
> indexed `while`". That was based on a flawed probe (see
> [00-language-findings.md §8](00-language-findings.md) for the post-mortem).
> The accurate constraint follows.

The real rule: **`Block#call` cannot be crossed by a yield.** So inside a fiber
that yields:

| Construct | Safe? |
|---|---|
| `for (x in anything)` | **Yes** — verified on native `List` and user `Iterable` |
| `while (...)` | **Yes** |
| `.each`/`.map`/`.where`/`.filter`/`.reduce { }` | **No** — `CannotYieldAcrossNativeFrame` |

So a demand-driven evaluator is buildable; it must simply use `for` rather than
the block-taking combinators in the evaluator and function library. `SUM`'s
`range.reduce(0) { ... }` becomes a `for` loop. That is a real constraint and a
real style tax, but it is **not** the wholesale abandonment of the collection
API the first draft claimed.

v1 remains a single explicit topological sweep, because that is simpler and
sufficient, not because fibers are unavailable. The decision is deferred in
[07-dependency-graph-and-recalc.md §6](07-dependency-graph-and-recalc.md).

> **Commentary.** What survives the correction is still worth saying: the
> constraint is invisible at the point of use. `range.reduce(0) { ... }` and
> `for (r in range) { }` look equally innocent, and only one of them works
> inside a yielding fiber. Neither the iteration docs nor the concurrency docs
> mention the other's constraint. The runtime's diagnostic is genuinely
> excellent — it names the mechanism *and* the canonical example — but you only
> see it after writing the code.

## 7. Traceability

Every requirement is `REQ-<AREA>-<n>`, defined in the document that owns it and
indexed in [14-traceability.md](14-traceability.md). Every `REQ` must have at
least one golden test. A `REQ` with no test is a spec bug.
