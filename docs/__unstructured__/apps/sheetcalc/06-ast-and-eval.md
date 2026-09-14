# SheetCalc — AST and Evaluation

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md) §4, §7, §8, and
[02-value-model.md](02-value-model.md).

## 1. Scope

This document specifies the `Ast` node hierarchy that `05-formula-parser.md`
builds and that `07-dependency-graph-and-recalc.md` walks and evaluates. Two
questions dominate: how a node evaluates itself, and how a node tells the
dependency graph which cells it reads.

## 2. The key design choice — polymorphic `eval` vs a Visitor

**Decision: polymorphic `eval(ctx)`.** Every concrete `Ast` subclass defines
its own `eval(ctx) -> CellValue`. There is no `Evaluator` class that
pattern-matches on node type.

```phalcom
class Ast {
  // Never actually invoked -- every concrete subclass overrides this. Kept
  // here (rather than omitted) so the protocol is visible in one place, the
  // same idiom 02-value-model.md §2 uses for CellValue's root defaults.
  eval(ctx) => ErrorVal.of(#VALUE)

  isRange => false
  evalRange(ctx) => [self.eval(ctx)]
  dependencies() => List.new()
}
```

**Why not a Visitor.** The classic argument for a Visitor is that it moves
per-node-type behavior out of the node classes and into a separate object,
so you can add a new operation (pretty-printer, optimizer, evaluator) without
touching the node classes. That argument requires **double dispatch**:
`node.accept(visitor)` calls back into `visitor.visitBinOp(self)`,
`visitor.visitCall(self)`, and so on, so the right `visit*` overload runs for
the node's *runtime* type rather than whatever type the caller's variable is
declared as.

Phalcom has no method overloading and no static type on a variable to
dispatch from — every send resolves purely on the receiver's runtime class
(single dispatch), which is exactly the mechanism that forced DEC-VM-1 in the
value model (findings §4: `1 + errorValue` can't cooperate because there is no
second dispatch pass on the argument's type). A Visitor over `Ast` would hit
the identical wall from the other direction: `visitor.visit(node)` alone can
never pick `visitBinOp` over `visitCall` unless every node class supplies its
own `accept` method that hand-writes the second call —

```phalcom
class BinOp is Ast {
  accept(visitor) => visitor.visitBinOp(self)
}
class Call is Ast {
  accept(visitor) => visitor.visitCall(self)
}
```

— which is double dispatch built by hand, one `accept` override per node
class, to get back to exactly the same place `eval(ctx)` already is: one
method per node class, one clear override site. The Visitor buys nothing here
because there is currently exactly one operation over the `Ast` (`eval`);
`dependencies()` is the second, and it fits the same polymorphic-method shape
with no `accept`/`visit` machinery at all. If a third or fourth tree-walking
operation appears later (a pretty-printer for error messages, a formula
optimizer), each is still just one more method defined per node class — no
worse than a Visitor, and it never needs `accept`.

> **Commentary — this is the same fork DEC-VM-1 hit, from the AST side.**
> Both the value model and the AST needed "the right behavior for this
> object's runtime type," and Phalcom answers that question exactly once, via
> ordinary single-dispatch message sends. A Visitor is a workaround for
> languages that lack that and need a second dispatch axis bolted on. Since
> Phalcom's message-send model already gives every node its own `eval`, layering
> a Visitor on top would be reintroducing, by hand, the exact double-dispatch
> machinery the language doesn't have a shortcut for. Recommend polymorphic
> `eval` without reservation.

## 3. Node hierarchy

```
Ast                              (abstract root; eval/evalRange/dependencies)
├── Lit          wraps a CellValue literal
├── RefNode      a single cell reference, e.g. A1, $B$2
├── RangeNode    a rectangular reference, e.g. A1:B7
├── BinOp        left, right, and a 2-arity operator block
├── UnaryOp      operand and a 1-arity operator block
└── Call         a function name (String) and a List<Ast> of argument nodes
```

**REQ-AST-1.** `Ast` is the sole root of the node hierarchy; every node used
by the parser (`05-formula-parser.md`) is one of the six classes above.
**REQ-AST-2.** No `Ast` subclass stores a native `f64`/`String`/`Bool` as a
formula constant. `Lit` always wraps a `CellValue` (REQ-VM-1/2).

### 3.1 `BinOp` — one class, not one per operator

**Decision: a single `BinOp` class parameterized by a captured block, not one
subclass per operator.** The candidates:

| Design | Verdict |
|---|---|
| One subclass per operator (`AddOp`, `SubOp`, ... ten classes for `+ - * / % < <= > >= ==`) | Rejected. Each `eval` would be one line (`_left.eval(ctx) + _right.eval(ctx)`), and the ten classes differ in nothing except which literal operator token appears in that one line. Ten classes for zero shared behavior beyond "hold two children" is pure boilerplate. |
| One `BinOp` class with an operator **kind** field and a branch chain inside `eval` (`if (kind == #add) ... else if (kind == #sub) ...`) | Rejected. Moves the boilerplate from the class list into one long method; same ten-way duplication, just relocated. |
| One `BinOp` class holding a **block** chosen once at parse time | **Chosen.** `eval` is one line with zero branching: `_op.call(_left.eval(ctx), _right.eval(ctx))`. The parser (05) picks the block from a small table keyed by token kind, once, when it builds the node — not on every evaluation. |

```phalcom
class BinOp is Ast {
  // `op` is a 2-arity block, e.g. `{ l, r => l + r }`, selected by the
  // parser from the operator token — see 05-formula-parser.md. Not a Symbol,
  // not a `perform` selector: a plain closure captured once at parse time.
  @constructor
  new(left, right, op) {
    _left = left
    _right = right
    _op = op
  }

  eval(ctx) => _op.call(_left.eval(ctx), _right.eval(ctx))

  dependencies() {
    var result = List.new()
    for (r in _left.dependencies()) { result.add(r) }
    for (r in _right.dependencies()) { result.add(r) }
    return result
  }
}
```

Example construction, as the parser would do it for `A1 + B1`:

```phalcom
BinOp.new(RefNode.new(a1), RefNode.new(b1), { l, r => l + r })
```

`UnaryOp` (used only for prefix `-`, e.g. `-A1`) follows the identical shape
with a 1-arity block:

```phalcom
class UnaryOp is Ast {
  @constructor
  new(operand, op) {
    _operand = operand
    _op = op
  }

  eval(ctx) => _op.call(_operand.eval(ctx))
  dependencies() => _operand.dependencies()
}
```

**REQ-AST-3.** `BinOp`/`UnaryOp` perform no operator branching in `eval`; the
operator is a block resolved once, at parse time, by the parser.

### 3.2 Where DEC-VM-1 pays off

`_op.call(_left.eval(ctx), _right.eval(ctx))` sends `+`/`-`/`*`/`/`/`%`/`<`/...
directly to whatever `CellValue` the operands evaluated to. `BinOp#eval` does
not ask "is this a number," "is this an error," or "do these types match" —
it has no `isNum`/`isError` check anywhere in it. Every one of those checks
lives in `CellNum#+`, `ErrorVal#+`, etc. (02-value-model.md §3–4). This is the
entire payoff DEC-VM-1 was forced to buy: the evaluator is a thin dispatcher,
and the value model is where correctness lives.

```phalcom
Lit
```

```phalcom
class Lit is Ast {
  // `value` is already a CellValue — CellNum.of(2), CellText.of("x"), etc.
  // built by the parser directly from the token, never a bare native.
  @constructor
  of(value) { _value = value }
  eval(ctx) => _value
}
```

## 4. `EvalContext`

```phalcom
class EvalContext {
  @constructor
  new(grid, functions, currentRef, visiting) {
    _grid = grid              // Grid — see 03-references-and-grid.md
    _functions = functions    // FunctionTable — see 08-functions.md
    _currentRef = currentRef  // Ref of the cell currently being evaluated
    _visiting = visiting      // Set<Ref> — cells on the current eval call stack
  }

  grid => _grid
  functions => _functions
  currentRef => _currentRef

  isVisiting(ref) => _visiting.includes(ref)

  // Returns a new context scoped to evaluating `ref`; never mutates the
  // caller's context (each frame gets its own visiting set).
  enter(ref) {
    let next = Set.new()
    for (r in _visiting) { next.add(r) }
    next.add(ref)
    return EvalContext.new(_grid, _functions, ref, next)
  }
}
```

**Cycle detection is layered, and `EvalContext` is the inner, defensive
layer, not the authoritative one.** In v1's topological-sweep design
(07-dependency-graph-and-recalc.md), the dependency graph is built statically
from `dependencies()` before any cell evaluates, cycles are detected there,
and every cycle member is pre-seeded with `ErrorVal(#CIRC)` before `eval` ever
runs. By the time `RefNode#eval` reads another cell, recalc guarantees that
cell's cached value is already current — `RefNode#eval` never triggers another
cell's evaluation itself.

`currentRef`/`isVisiting` exist as a **fail-safe**: if a bug in the dependency
graph ever let a self-reference or cycle slip through to `eval`, `RefNode`
returns `ErrorVal(#CIRC)` rather than recursing into a stack overflow. It is
belt-and-suspenders, not the mechanism — 07 owns the real cycle detection.

```phalcom
class RefNode is Ast {
  @constructor
  new(ref) { _ref = ref }

  eval(ctx) {
    (ctx.grid.inBounds(_ref)).ifFalse { return ErrorVal.of(#REF) }
    (_ref == ctx.currentRef or ctx.isVisiting(_ref)).ifTrue {
      return ErrorVal.of(#CIRC)
    }
    return ctx.grid.at(_ref).cachedValue
  }

  dependencies() => [_ref]
}
```

**REQ-EVAL-1.** `RefNode#eval` returns `ErrorVal(#REF)` for a `Ref` outside
the grid's populated bounds (03-references-and-grid.md defines "in bounds").
**REQ-EVAL-2.** `RefNode#eval` never recomputes another cell; it only reads
`Grid#at(_).cachedValue`. Recalculating a stale dependency is 07's job, not
eval's.
**REQ-EVAL-3.** `RefNode#eval` returns `ErrorVal(#CIRC)` for a self-reference
or a `Ref` already in `ctx`'s visiting set, independent of whatever 07's own
cycle detection does.

## 5. `RangeNode` and the two evaluation shapes

A range (`A1:A10`) cannot collapse to one `CellValue` — it is only ever
meaningful as an argument to a function. `Ast` therefore exposes **two**
evaluation entry points, with the second defaulting to the first:

```phalcom
class Ast {
  eval(ctx) => ErrorVal.of(#VALUE)      // scalar position
  isRange => false
  evalRange(ctx) => [self.eval(ctx)]    // range position: default wraps the scalar
  dependencies() => List.new()
}
```

A `Lit`, `RefNode`, `BinOp`, `UnaryOp`, or `Call` used where a range is
syntactically allowed (rare, but `SUM(5)` is legal — a bare scalar "range" of
one) automatically gets the singleton-list behavior for free from the root.
Only `RangeNode` overrides it:

```phalcom
class RangeNode is Ast {
  // from/to are Ref values — see 03-references-and-grid.md.
  @constructor
  new(from, to) {
    _from = from
    _to = to
  }

  isRange => true

  // A RangeNode is only ever constructed as a Call argument (05's grammar
  // restricts `:` to argument position), so plain `eval` is never the normal
  // path. It exists so Ast's interface stays uniform; it resolves to the
  // range's top-left cell as a defensible single-value fallback.
  eval(ctx) => ctx.grid.at(_from).cachedValue

  evalRange(ctx) {
    var result = List.new()
    for (ref in Grid.refsInRect(_from, _to)) {
      result.add(ctx.grid.at(ref).cachedValue)
    }
    return result
  }

  dependencies() => Grid.refsInRect(_from, _to)
}
```

(`Grid.refsInRect(_,_)` is specified in
[03-references-and-grid.md](03-references-and-grid.md); it enumerates every
`Ref` in the rectangle, row-major.)

### `Call#eval` deliberately does not flatten its arguments

```phalcom
class Call is Ast {
  @constructor
  new(name, args) {
    _name = name    // String, e.g. "SUM"
    _args = args    // List<Ast>
  }

  name => _name
  args => _args

  eval(ctx) => ctx.functions.invoke(_name, _args, ctx)

  dependencies() {
    var result = List.new()
    for (a in _args) {
      for (r in a.dependencies()) { result.add(r) }
    }
    return result
  }
}
```

`Call#eval` hands the **unevaluated** `List<Ast>` argument nodes, plus `ctx`,
straight to `FunctionTable#invoke` (08-functions.md). It does not pre-evaluate
them into a single flat `List<CellValue>`.

> **Commentary — flattening looked obviously correct, and was wrong.** The
> first draft of this design had `Call#eval` walk every argument, calling
> `evalRange(ctx)` on each and concatenating into one flat `List<CellValue>`
> before handing it to the function. That works for `SUM`/`AVERAGE`/`MIN`/
> `MAX`/`COUNT` — they only ever want "all the numbers, in order." It breaks
> for `VLOOKUP`, which needs its range argument's **row shape** (compare
> against column 1, return from column N of the *same row*) — information a
> flat list has already destroyed. It also breaks for `COUNTIF(range,
> criterion)`, which needs to know which argument was the range and which was
> the scalar criterion, not a merged bag of values. The fix was to stop
> guessing the shape centrally and let each function decide: `Call#eval`
> passes raw `argNodes` + `ctx`, and each `Fn` in 08-functions.md calls
> `.eval(ctx)` on the positions it wants scalar and `.evalRange(ctx)` (or, for
> `VLOOKUP`, a table-shaped `.evalRows(ctx)`) on the positions it wants
> ranged. The lesson: "evaluate the arguments" is not one operation with one
> right shape — it is a per-function decision, so it has to live with the
> function, not with `Call`.

**REQ-EVAL-4.** `Call#eval` never evaluates its arguments; it forwards
`(name, args, ctx)` to `FunctionTable#invoke` unchanged.
**REQ-EVAL-5.** `Ast#evalRange(ctx)` is defined on the root (default:
singleton list of `eval(ctx)`) so every node — not only `RangeNode` — is a
legal range-position argument.

## 6. `dependencies()` — feeding the dependency graph

`dependencies()` walks an `Ast` subtree and returns every `Ref` it reads,
independent of `eval`. It never touches the grid or a `CellValue` — it is a
pure structural walk over the tree the parser already built.

| Node | `dependencies()` |
|---|---|
| `Lit` | `[]` (root default; not overridden) |
| `RefNode` | `[self.ref]` |
| `RangeNode` | every `Ref` in the rectangle (`Grid.refsInRect`) |
| `BinOp` | `left.dependencies() ++ right.dependencies()` |
| `UnaryOp` | `operand.dependencies()` |
| `Call` | the concatenation of every argument's `dependencies()` |

**REQ-AST-4.** `dependencies()` is total and side-effect-free: it never reads
`Grid#at(_).cachedValue`, never raises, and terminates on any well-formed
`Ast` (the tree is finite — the parser cannot construct a cyclic `Ast`).
**REQ-AST-5.** `Engine.dependenciesOf(cell)` (07-dependency-graph-and-recalc.md)
calls exactly `cell.formula.dependencies()` — no separate re-walk of the
formula text.

## 7. Forward note for 07 — `for`, not `.each`/`.reduce`, and why it matters

Every loop in this document (`BinOp#dependencies`, `RangeNode#evalRange`,
`Call#dependencies`) is written as a hand-rolled `for (x in xs) { ... }`, not
`xs.each { }` / `xs.fold(initial: ..., using: ...)`. That was a style choice made for a reason
worth stating once, here, rather than repeating in every function
implementation in 08.

`for (x in xs) { }` compiles to a direct loop in the *current* frame
(confirmed: `break` and, per core.ph's own `any(where:)`/`count(where:)`, `return` both
work correctly inside it). `xs.each { block }` and
`xs.fold(initial: init, using: { block })`
instead hand the block to a **native Rust primitive** that calls back into it.
Findings §8 (`CannotYieldAcrossNativeFrame`, GAP-FIB-1) is specifically about
that second shape: yielding a fiber from inside a block a native primitive is
driving is a hard error, while the equivalent `for`/`while` loop is fine.

v1 never yields — recalculation is a synchronous topological sweep
(01-architecture.md §6), so this distinction has zero runtime consequence
today. It matters only if 07's deferred demand-driven-recalc-on-fibers
question (01 §6, GAP-FIB-1) is ever resolved "yes." At that point, whichever
loop resolves a cell's dependency would need to `Fiber.yield` while waiting
for that dependency to be computed — and every `for`-loop site in this
document (and in 08's `SUM`/`AVERAGE`/`MIN`/`MAX`/`COUNT`/`COUNTIF`/`VLOOKUP`)
would remain yield-safe *only because* they are `for`, not `.each`/`.reduce`.
Had this spec instead used the more idiomatic `Iterable` combinators
(`xs.fold(initial: 0, using: { acc, x => acc + x })` — genuinely the more natural style, and
the one `core.ph` itself uses internally for `Iterable#each`/`#reduce`), it
would need a rewrite before a fiber-based evaluator could touch it. This is
recorded here so 07 does not have to rediscover it.

## 8. Requirements summary

| REQ | Statement |
|---|---|
| REQ-AST-1 | `Ast` is the sole root; six concrete node classes. |
| REQ-AST-2 | No node stores a native constant; `Lit` always wraps a `CellValue`. |
| REQ-AST-3 | `BinOp`/`UnaryOp` hold a pre-selected block; `eval` has no operator branching. |
| REQ-AST-4 | `dependencies()` is pure, total, and side-effect-free. |
| REQ-AST-5 | `Engine.dependenciesOf` uses `Ast#dependencies()` directly. |
| REQ-EVAL-1 | `RefNode#eval` → `ErrorVal(#REF)` when out of bounds. |
| REQ-EVAL-2 | `RefNode#eval` reads the cache; it never recomputes another cell. |
| REQ-EVAL-3 | `RefNode#eval` → `ErrorVal(#CIRC)` as a fail-safe, independent of 07. |
| REQ-EVAL-4 | `Call#eval` forwards unevaluated `(name, args, ctx)`; no central flattening. |
| REQ-EVAL-5 | `Ast#evalRange` default makes every node a legal range-position argument. |

## 9. Test hooks

| REQ | Test |
|---|---|
| REQ-AST-1/2/3 | `suites/ast_construction.ph` — build one instance of each node kind, assert class and field shape |
| REQ-AST-4/5 | `suites/ast_dependencies.ph` — nested `BinOp`/`Call`/`RangeNode` trees, assert the exact `Ref` set |
| REQ-EVAL-1 | `suites/eval_ref_bounds.ph` — `RefNode` past the grid's populated extent |
| REQ-EVAL-2 | `suites/eval_ref_reads_cache.ph` — mutate a cell's cached value directly, confirm `RefNode#eval` sees it without recomputing |
| REQ-EVAL-3 | `suites/eval_circ_fallback.ph` — construct a self-referencing `RefNode` directly (bypassing 07) and confirm `#CIRC!` rather than a stack overflow |
| REQ-EVAL-4/5 | `suites/eval_call_shapes.ph` — one function reading a scalar position, one reading a range position, from the same `Call` node |
