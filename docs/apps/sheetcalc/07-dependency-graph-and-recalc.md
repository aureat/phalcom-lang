# SheetCalc — Dependency Graph and Recalculation

Part of the [SheetCalc specification](README.md).
Grounded in [00-language-findings.md](00-language-findings.md) §7, §8 and
[01-architecture.md](01-architecture.md) §6.

This document specifies `recalc/depgraph.ph` and `recalc/engine.ph`: how
SheetCalc discovers which cells a formula reads, how it orders evaluation so
every dependency runs before its dependent, how it detects and contains
cycles, and — in §6, the section every other document in this spec points
at — why the demand-driven, fiber-based design that would make most of this
document unnecessary cannot be built with the current runtime.

**Dependencies on other layers.** This document is written against the
following minimal contract, owned by the documents named:

| Symbol | Contract | Owner |
|---|---|---|
| `Ref#==(_)`, `Ref#hash` | equality/hash honored by `Map`/`Set` | [03](03-references-and-grid.md) |
| `Ref#row`, `Ref#col` | integer accessors, native `Number` | [03](03-references-and-grid.md) |
| `Grid#cellAt(ref)` | returns the `Cell` at `ref` | [03](03-references-and-grid.md) |
| `Cell#isFormula` | `Bool` | [03](03-references-and-grid.md) |
| `Cell#ast` | the parsed `Ast`, valid only when `isFormula` | [03](03-references-and-grid.md) |
| `Grid#setValue(ref, value)` | caches a computed `CellValue` | [03](03-references-and-grid.md) |
| `Grid#setLiteral(ref, value)` | overwrites a literal cell's value | [03](03-references-and-grid.md) |
| `Ast#eval(ctx)` | returns a `CellValue` | [06](06-ast-and-eval.md) |
| `Value.ErrorVal.of(_)` | `CellValue` constructor | [02](02-value-model.md) |
| `Sort.by(list, cmp)` | merge sort, `cmp` a two-arg `Bool` block | [01](01-architecture.md) §1 |

**Assumed collection surface.** Everything below is buildable from the
collection methods the probe log actually exercised: `Map#at(_)` /
`Map#at(_, put:)`, `Set#add(_)`, `Set#size`, `List#add(_)`, `List#at(_)`,
`List#size`, and `Option#unwrapOr(_)` (findings §7, §9; the `unwrapOr`
pattern is used directly in [02-value-model.md](02-value-model.md) §3). The
one thing this document leans on that is **not** itself in the probe log is
`for (x in someMapOrSet) { }` iteration. This draft flagged that as an
unverified assumption; it has since been **probed and confirmed**:
`for (x in aMap)` iterates **keys** in insertion order, and `for (x in aSet)`
iterates in insertion order. Both are recorded in findings §7.

The same probe settled the containment question: **`Set#includes(_)` exists**
(`Set#contains(_)` does **not** — that spelling raises
`MessageNotUnderstood`), and `Set#remove(_)` exists. The
`markedMap_(_)` workaround in §4 below is therefore **no longer necessary** and
is retained only as a note; use `set.includes(r)` directly.

---

## 1. `DepGraph` — forward and reverse edges

```phalcom
class DepGraph {
  construct new() {
    _forward = Map.new()       // Ref -> Set<Ref>: cell -> cells it reads
    _reverse = Map.new()       // Ref -> Set<Ref>: cell -> cells that read it
    _formulaRefs = Set.new()   // every ref that currently owns a formula
  }

  formulaRefs => _formulaRefs

  dependenciesOf(ref) => _forward.at(ref).unwrapOr(Set.new())
  dependentsOf(ref)   => _reverse.at(ref).unwrapOr(Set.new())

  // Register ref's dependency set. Called once per formula cell, when the
  // grid is built. v1 has no live formula editing (no stdin -- findings
  // SS2), so edge removal/rebuild is out of scope; see the note at the end
  // of this section for what a live editor would need in addition.
  addFormula(ref, deps) {
    _forward.at(ref, put: deps)
    _formulaRefs.add(ref)
    for (dep in deps) {
      let existing = _reverse.at(dep).unwrapOr(Set.new())
      existing.add(ref)
      _reverse.at(dep, put: existing)
    }
  }
}
```

`Map<Ref, Set<Ref>>` is exactly the shape findings §7 predicted would be
"most likely to be under-exercised and broken" — a user-class key (`Ref`) in
both the key and the element position of a `Map`/`Set` pair, with dedup and
lookup depending entirely on `Ref#hash`/`Ref#==`. The probe **refuted** that
prediction: `Map`/`Set` keyed by a user class honor `hash`/`==` correctly,
and iteration order is insertion-ordered and deterministic across runs
(findings §7). That determinism is why `DepGraph`'s edge lists — and
therefore the topological order derived from them (§3) — are reproducible
run to run, which is what makes a stdout-exact golden test of recalculation
possible at all ([10-testing.md](10-testing.md)).

> **Note — what a live editor would add.** `addFormula` is write-once per
> ref for v1. A spreadsheet that accepted edited formulas would need a
> `removeFormula(ref)` that walks `_forward.at(ref)`, removes `ref` from each
> dependency's reverse set (`Set#remove(_)`, unverified — see the collection
> surface note above), and only then calls `addFormula` again with the new
> dependency set. That is a real (small) piece of unwritten work, deliberately
> excluded here because [README.md](README.md) puts interactive editing out
> of scope for v1.

---

## 2. Building the graph — `Ast#dependencies`

Every `Ast` node knows its own references. The root implements the default
(no references); only the node kinds that can *contain* a `Ref` override it.

```phalcom
class Ast {
  // Default: a leaf with no references (number/text/bool literals). Most
  // node kinds inherit this and never need to touch dependency tracking.
  dependencies => Set.new()
}

// A single-cell reference: A1, $A$1, B12.
class RefLit extends Ast {
  construct new(ref) { _ref = ref }

  dependencies {
    let s = Set.new()
    s.add(_ref)
    return s
  }
}

// A rectangular range: A1:B3. Only meaningful as a function-call argument
// (SUM(A1:B3)); see 06/08 for eval semantics. Its dependency set is every
// Ref in the closed rectangle.
class RangeLit extends Ast {
  construct new(topLeft, bottomRight) { _tl = topLeft; _br = bottomRight }

  dependencies {
    let s = Set.new()
    var r = _tl.row
    while (r <= _br.row) {
      var c = _tl.col
      while (c <= _br.col) {
        s.add(Ref.at(r, c))
        c = c + 1
      }
      r = r + 1
    }
    return s
  }
}

// Shared base for +, -, *, /, %, and comparisons: union of both operands.
// Add/Sub/Mul/Div/Mod/Lt/... extend this and add nothing but eval(ctx).
class BinOp extends Ast {
  construct new(left, right) { _left = left; _right = right }

  dependencies {
    let s = Set.new()
    for (r in _left.dependencies)  { s.add(r) }
    for (r in _right.dependencies) { s.add(r) }
    return s
  }
}

// A function call: SUM(A1:A3), IF(A1>0, 'yes', 'no'). Union of every arg's
// dependencies -- this is where a RangeLit's rectangle enters the graph.
class Call extends Ast {
  construct new(name, args) { _name = name; _args = args }

  dependencies {
    let s = Set.new()
    for (a in _args) {
      for (r in a.dependencies) { s.add(r) }
    }
    return s
  }
}
```

Building the whole graph is a single walk over every formula cell:

```phalcom
class Engine {
  construct new(grid, graph) {
    _grid = grid
    _graph = graph
  }

  // Called once, before the first recalc(). For each formula cell already
  // present in the grid, register its dependency edges.
  buildGraph() {
    for (ref in _grid.formulaRefs) {          // owned by 03: refs with isFormula
      let cell = _grid.cellAt(ref)
      _graph.addFormula(ref, cell.ast.dependencies)
    }
  }
}
```

**REQ-DEP-3** (below) pins the override discipline this relies on: a node
kind that cannot contain a `Ref` never needs to write a `dependencies`
method at all. This is the same boilerplate-collapse technique
[02-value-model.md](02-value-model.md) §2 uses for `CellValue`'s operator
defaults, applied to the AST instead of the value hierarchy.

---

## 3. Topological order — DFS with three-color marking

Evaluation order must put every dependency before its dependent. The
standard technique is a DFS post-order over the "depends-on" graph, with
three colors per node to distinguish "this is a cycle" from "this is a
diamond":

- **white (0)** — not yet visited.
- **grey (1)** — on the current DFS path (an ancestor of the node being
  visited, evaluation not yet finished).
- **black (2)** — fully processed; every one of its dependencies is already
  in the output order.

```phalcom
class Engine {
  // ... construct as above, plus:
  //   _color     : Map<Ref, Number>   -- 0/1/2, defaults to white
  //   _parent    : Map<Ref, Ref>      -- DFS parent, for cycle-path recovery
  //   _order     : List<Ref>          -- post-order output; IS the topo order
  //   _topoIndex : Map<Ref, Number>   -- ref -> its position in _order
  //   _cyclic    : Set<Ref>           -- every ref confirmed to be in a cycle

  colorOf_(ref) => _color.at(ref).unwrapOr(0)

  buildTopoOrder_() {
    _color = Map.new()
    _parent = Map.new()
    _order = List.new()
    _cyclic = Set.new()

    for (root in _graph.formulaRefs) {
      if (self.colorOf_(root) == 0) { self.visit_(root) }
    }

    _topoIndex = Map.new()
    var i = 0
    while (i < _order.size) {
      _topoIndex.at(_order.at(i), put: i)
      i = i + 1
    }
  }

  visit_(ref) {
    _color.at(ref, put: 1)                      // grey: entering
    for (dep in _graph.dependenciesOf(ref)) {
      let c = self.colorOf_(dep)
      if (c == 1) {
        self.markCycle_(ref, dep)                // grey-on-grey: a cycle
      } else if (c == 0) {
        _parent.at(dep, put: ref)
        self.visit_(dep)
      }
      // c == 2 (black): dep is already fully resolved, possibly by another
      // branch (a diamond). Not a cycle. Do nothing.
    }
    _color.at(ref, put: 2)                       // black: finished
    _order.add(ref)                              // post-order append
  }
}
```

Two things about this that are easy to get wrong:

**No reversal step.** Most explanations of DFS topological sort reverse the
finishing order, because they orient edges prerequisite-to-dependent
(`u -> v` means "`u` before `v`"). `DepGraph`'s forward edges point the other
way — `cell -> cells it reads` — so a node is appended to `_order` only after
every cell it depends on is already in `_order`. The post-order **is** the
evaluation order, unreversed. `A1 = B2 + 1`, `B2 = 5` visits `B2` before
finishing `A1`, giving `_order = [B2, A1]` directly.

**Black is not a cycle.** A node reached a second time with color black (not
grey) is a *shared* dependency reached through two different paths — a
diamond, not a cycle. A cheaper "have I visited this before" boolean marker
(two colors instead of three) cannot tell a diamond from a cycle and would
misfire `#CIRC!` on every `A1=B1+C1, B1=D1, C1=D1` shape. Three colors is the
minimum needed to keep diamonds (task item: REQ-RECALC-2, and the
`recalc_diamond` test hook in §7) and cycles (§4) distinguishable.

---

## 4. Cycle handling — the whole SCC, and everything downstream

A grey-on-grey encounter in `visit_` means `ref`'s dependency `dep` is an
**ancestor of `ref` on the current DFS path** — a cycle exists from `dep`
down to `ref` and back. `_parent` (recorded on every tree edge) recovers the
exact path: walk parents from `ref` up to `dep`.

```phalcom
class Engine {
  // fromRef -> ancestorRef is the back-edge just found. Every ref on the
  // DFS path from ancestorRef down to fromRef (inclusive of both ends) is
  // part of this cycle's strongly-connected component.
  markCycle_(fromRef, ancestorRef) {
    var cur = fromRef
    _cyclic.add(ancestorRef)
    while (not (cur == ancestorRef)) {
      _cyclic.add(cur)
      cur = _parent.at(cur).unwrapOr(cur)   // always reaches ancestorRef
    }
  }
}
```

For `A1 = A1` (self-reference): `fromRef == ancestorRef == A1`; the loop body
never runs; `_cyclic = {A1}`. For a mutual cycle `A1 = B2`, `B2 = A1`: DFS
visits `A1` (grey), recurses into `B2` (grey), finds `A1` grey again;
`markCycle_(B2, A1)` walks `B2 -> parent(B2)=A1`, giving `_cyclic = {A1, B2}`.

That marks only the cycle itself. The spreadsheet-correctness requirement is
broader: **every cell that transitively reads a cyclic cell must also become
`#CIRC!`**, because its formula can never produce a well-defined value
either — it is downstream of a value that doesn't exist. This is a reverse-edge
reachability closure, computed once per `recalc()`:

```phalcom
class Engine {
  // BFS over reverse (dependents) edges, seeded from `seeds`. Uses only
  // Set#add + Set#size for dedup (the "did size change" idiom below stands
  // in for an unverified Set#contains -- see the collection-surface note).
  closeOverDependents_(seeds) {
    let touched = Set.new()
    let queue = List.new()

    for (s in seeds) {
      let before = touched.size
      touched.add(s)
      if (touched.size > before) { queue.add(s) }
    }

    var i = 0
    while (i < queue.size) {
      let ref = queue.at(i)
      for (dependent in _graph.dependentsOf(ref)) {
        let before = touched.size
        touched.add(dependent)
        if (touched.size > before) { queue.add(dependent) }
      }
      i = i + 1
    }
    return touched
  }

  // SUPERSEDED: written when Set membership was unverified. `Set#includes(_)`
  // is now probe-confirmed to exist, so callers use `set.includes(r)` directly
  // and this helper is dead. Kept only to explain the §0 note.
  markedMap_(set) {
    let m = Map.new()
    for (x in set) { m.at(x, put: true) }
    return m
  }
  isMarked_(m, x) => m.at(x).unwrapOr(false)
}
```

`recalc()` ties §3 and §4 together: build the topological order, compute the
poisoned set (cycle SCC + everything downstream), assign `#CIRC!` to every
poisoned ref directly, then evaluate every remaining formula ref in
topological order.

```phalcom
class Engine {
  recalc() {
    self.buildTopoOrder_()

    let poisoned = self.closeOverDependents_(_cyclic)
    let poisonedMark = self.markedMap_(poisoned)

    for (r in poisoned) {
      _grid.setValue(r, Value.ErrorVal.of(#CIRC))
    }

    var i = 0
    while (i < _order.size) {
      let r = _order.at(i)
      if (not self.isMarked_(poisonedMark, r) and _grid.cellAt(r).isFormula) {
        self.evalOne_(r)
      }
      i = i + 1
    }
  }

  evalOne_(ref) {
    let cell = _grid.cellAt(ref)
    let ctx = Eval.Context.new(_grid)     // 06's contract: ctx.valueAt(ref) -> CellValue
    let value = cell.ast.eval(ctx)
    _grid.setValue(ref, value)
  }
}
```

**A cyclic cell's formula is never run, not even once.** This matters beyond
tidiness: if `evalOne_` were called on a cyclic ref, `Ast#eval` would try to
read a dependency that is itself unresolved (possibly the same ref), and
without the pre-computed `poisoned` guard there is nothing stopping infinite
recursion in the evaluator. Excluding poisoned refs from the eval sweep
entirely — deciding `#CIRC!` from graph shape *before* evaluation, rather
than discovering it *during* evaluation — is what keeps the evaluator itself
simple (no recursion-guard, no re-entrancy tracking needed in `Ast#eval`; see
[06-ast-and-eval.md](06-ast-and-eval.md)).

---

## 5. Dirty marking — incremental recalculation

`recalc()` above is a full sweep: every formula cell, every run. For
`fixtures/*.ph` that mutate a literal cell mid-test (to exercise "does
recalculation actually recompute the right things"), a full re-sweep would
still produce the right answer, but it defeats the point of the test — it
wouldn't distinguish "only the dependents recomputed" from "everything
recomputed by coincidence." `setLiteral` recomputes exactly the transitive
dependents of the changed cell, in the cached topological order:

```phalcom
class Engine {
  // Change a literal cell's value, then recompute exactly the formula
  // cells that could be affected -- ref's transitive dependents via
  // reverse edges, ordered by the topo index computed in the last
  // buildTopoOrder_() (graph shape doesn't change in v1, so the cached
  // order is still valid; see the DepGraph note in SS1 about live edits).
  setLiteral(ref, value) {
    _grid.setLiteral(ref, value)

    let dirty = self.closeOverDependents_(self.singleton_(ref))
    let dirtyFormulas = List.new()
    for (r in dirty) {
      if (_grid.cellAt(r).isFormula) { dirtyFormulas.add(r) }
    }

    let ordered = Sort.by(dirtyFormulas, { a, b => self.topoIndexOf_(a) < self.topoIndexOf_(b) })
    var i = 0
    while (i < ordered.size) {
      self.evalOne_(ordered.at(i))
      i = i + 1
    }
  }

  singleton_(ref) {
    let s = Set.new()
    s.add(ref)
    return s
  }

  topoIndexOf_(r) => _topoIndex.at(r).unwrapOr(0)
}
```

`closeOverDependents_` is exactly the same reverse-edge BFS used for cycle
poisoning in §4 — dirtying and cycle-downstream marking are the same
operation (transitive closure over reverse edges), just seeded differently
(a cycle's SCC vs. one edited cell). `topoIndexOf_` compares plain native
`Number`s, not `CellValue`s — these are engine-internal bookkeeping indices,
never exposed to a formula or an arithmetic operator, so DEC-VM-1 (every
*cell* value must be a user class) does not apply to them.

**REQ-RECALC-6.** `setLiteral(ref, value)` evaluates exactly `dirty ∩
formulaRefs` — the transitive dependents of `ref` that are formula cells —
and every other cell's cached value is untouched. `ref` itself, if a literal,
is not re-evaluated (there is nothing to evaluate); if `ref` is itself a
formula cell being independently overridden for a test, it is still included
in `dirty` (the BFS seed is included, per `closeOverDependents_`'s seeding
loop) and gets re-evaluated too.

---

## 6. The fiber question

This is the most important section in this document, because it is the one
place where the two most interesting things about Phalcom — the
Smalltalk-style block/collection API used everywhere above, and the fiber
concurrency primitive — turn out to actively fight each other.

### 6.1 The design that wants to exist

Recalculation does not have to be an explicit topological sweep. The more
natural design for a spreadsheet engine is **demand-driven** ("lazy pull")
evaluation: evaluating `A1` runs its `Ast#eval(ctx)`, which — on hitting a
reference to an uncomputed `B2` — suspends, lets a scheduler compute `B2`
(and whatever `B2` itself needs, recursively), and resumes once `B2`'s value
exists. `Fiber.yield(_)` / `System.schedule(_)` / `System.nextScheduled` are
exactly the primitives this calls for (findings §8; [01](01-architecture.md)
§6). Sketched:

```phalcom
// The design that GAP-FIB-1 blocks -- do not implement this as written.
class RefLit extends Ast {
  eval(ctx) {
    if (ctx.isComputed(_ref)) { return ctx.valueOf(_ref) }
    Fiber.yield(WaitingOn.new(_ref))   // suspend until _ref is ready
    return ctx.valueOf(_ref)           // resumed: now it is ready
  }
}
```

This is genuinely elegant. It needs no separate topological pre-pass at all
— dependency order falls out of demand order for free, and cycle detection
becomes "does a fiber end up waiting, transitively, on itself" rather than a
graph algorithm run up front.

### 6.2 Why it is blocked

`Fiber.yield(_)` inside a block driven by a **native** primitive raises
`CannotYieldAcrossNativeFrame` (GAP-FIB-1, findings §8, trap 1). The
demand-driven design above yields from deep inside two places that are both
native-driven in exactly the code this document already specifies:

1. **`SUM(A1:A10)` and every other range-consuming function** in
   [08-functions.md](08-functions.md) iterates its range with
   `range.each { ref => ... }` or `.reduce { ... }` — both native-driven
   blocks. A `RefLit#eval` that yields, called from inside that `each`, is
   exactly trap 1.
2. **Every `for`/`.each`/`.map` this very document uses** in §1–§5
   (`for (dep in _graph.dependenciesOf(ref))`, `for (r in poisoned)`, and so
   on) would carry the same restriction if any of that code ran inside a
   fiber that yields. It currently doesn't (v1 has no fibers — see 6.4(a)),
   which is exactly why none of it needed hand-rolling.

The dependency-walking code in §2 (`Ast#dependencies`) is unaffected either
way — it never yields, so it can use `for`/`.each` freely regardless of
which recalculation design is chosen. The constraint is only on code that
executes in the dynamic extent of a fiber that calls `Fiber.yield(_)`.

### 6.3 The workaround

> **CORRECTED.** This section was written against the original, overstated
> GAP-FIB-1 and called this "the only workaround". Since then the finding has
> been corrected (post-mortem in
> [00-language-findings.md §8](00-language-findings.md)): a yield cannot cross a
> **native call frame**, and `Block#call` is one — but **`for` is safe**,
> verified on native `List`s and user `Iterable`s alike.
>
> So the real requirement is narrower: within a yielding fiber's dynamic extent,
> replace the **block-taking combinators** (`.each`/`.map`/`.where`/`.filter`/
> `.reduce`) with **`for`** loops. An indexed `while` also works but is rarely
> necessary. `sumRange_` below is therefore more defensive than it needs to be;
> a `for (r in refs)` loop over the same body is fiber-safe and reads far
> better. The tradeoff table in §6.4 keeps its shape, but option (b)'s cost
> column is meaningfully overstated — it is a style tax on one subsystem, not a
> "total loss of idiomatic style".

The pattern below (an indexed `while`, per findings §8) is the maximally
conservative form:

```phalcom
// SUM, hand-rolled to be fiber-safe. Compare to the two-line `.reduce`
// this replaces.
sumRange_(refs, ctx) {
  var total = Value.CellNum.of(0)
  var i = 0
  while (i < refs.size) {
    total = total + ctx.valueOf(refs.at(i))   // may yield inside valueOf
    i = i + 1
  }
  return total
}
```

That is not a one-off cost. It applies to the evaluator's own dispatch loop,
to `SUM`/`COUNTIF`/every range function in [08](08-functions.md), and to
anything added later that touches a `List`/`Range` while a fiber above it
might yield. Inside any fiber that yields, **the entire idiomatic collection
API — `.each`, `.map`, `.reduce`, lazy `.where{}` views — is off-limits**,
which is the exact sentence findings §8 uses to describe the general trap,
now shown concretely on this program's own function library.

### 6.4 The tradeoff

| Option | Description | Cost | Payoff |
|---|---|---|---|
| **(a) v1 explicit topological sweep** (RECOMMENDED — this is what §1–§5 specify) | No fibers anywhere in `recalc/`. Full idiomatic collection API everywhere. | The topo pre-pass (§3) and cycle-closure (§4) must exist as separate algorithms. | Simple, provably correct, no new failure mode. Crucially: v1 always renders the **whole grid** to stdout every run (no stdin, §README "out of scope" — no partial/interactive view is possible anyway), so demand-driven laziness has **no payoff to capture** in this program's own scope. The "only compute what's asked for" motivation for (b) is moot here. |
| **(b) demand-driven, all loops hand-rolled** | The design in 6.1, implemented per 6.3. | Total loss of idiomatic style inside `eval/` and `recalc/`'s reachable call graph. Still needs its own cycle guard (a fiber transitively waiting on itself must be detected and aborted with `#CIRC!`, not hung forever — this is not free, it is the grey-marking idea again, just per-fiber instead of per-graph). One `Fiber` per pending cell (or an equivalent hand-rolled scheduler loop), adding its own bookkeeping. | The elegance of 6.1, and nothing else this program needs — see (a)'s payoff column. |
| **(c) fix the runtime** | Let `Fiber.yield(_)` cross a native block-driving frame safely (ADR-0030 §4 already treats this as a guarded diagnostic, not a crash, so the invariant it protects is known); or, per the in-flight "Iterable rehome" direction, implement `List#each`/`#map`/`#reduce` in `.ph` over a cursor/`Iterable` protocol instead of as a native primitive, so the block-calling frame is an ordinary Phalcom frame and yield works through it for free. | A language/runtime change, not something `docs/apps/sheetcalc` can do. | Fixes the conflict for every future Phalcom program that wants fibers and collections together, not just this one. |

**This choice is explicitly deferred.** [01-architecture.md](01-architecture.md)
§6 and [README.md](README.md) both point here and both say the same thing:
whether to hand-roll around GAP-FIB-1 (b) or accept the topological sweep
(a) — or push for (c) in the runtime — is a decision for the user, not
something this specification resolves on its own. This document specifies
(a) in full (§1–§5) because it is buildable today and because 6.4's payoff
column shows it is not even a compromise for this particular program; it
does **not** rule out (b) or (c) being the right call for a different
program, or for a v2 of this one, if a use case ever needs partial/on-demand
recomputation.

> **Commentary.** Phalcom has two flagship features: a Smalltalk-style
> block/collection API (`.each`, `.map`, `.reduce`, lazy `.where{}` views —
> the idiom every other document in this spec reaches for by default), and
> fibers (`Fiber.new{}`, `#call()`, `Fiber.yield(_)` — verified working,
> including 50,000 frames of recursion, findings §8). Individually, both are
> well-built and pleasant to use. **Used together in one call stack, they do
> not work**: the moment a fiber that yields calls into any native-driven
> block — which is to say, almost any nontrivial use of `List` or `Range` —
> it dies with `CannotYieldAcrossNativeFrame`. Neither feature's own
> documentation mentions the other's constraint. A user reaches for the
> obvious idiom in the obvious place (SUM over a range, inside a fiber that's
> waiting on a cell) and gets a runtime error that has nothing to do with
> anything they wrote wrong — the code is idiomatic Phalcom in both of its
> flagship styles, and idiomatic-times-idiomatic is exactly what breaks. This
> is not a permission to add multithreading or async wizardry; it's a request
> that the two most-advertised parts of the language be usable in the same
> function.

---

## 7. Requirements and test hooks

### `DepGraph`

**REQ-DEP-1.** `DepGraph` maintains forward edges `Ref -> Set<Ref>`: for each
formula cell, the set of cells its formula reads.
**REQ-DEP-2.** `DepGraph` maintains reverse edges `Ref -> Set<Ref>`: for each
cell, the set of formula cells that read it. Every call to `addFormula`
updates both sides atomically (§1).
**REQ-DEP-3.** `Ast#dependencies` defaults to the empty set on the root node.
Only node kinds that can contain a `Ref` (`RefLit`, `RangeLit`) or compose
child nodes (`BinOp`, `Call`) override it; number/text/bool literal nodes
never need their own override.
**REQ-DEP-4.** `RangeLit(topLeft, bottomRight)`'s dependency set is every
`Ref` in the closed rectangle `[topLeft.row..bottomRight.row] x
[topLeft.col..bottomRight.col]`.
**REQ-DEP-5.** `DepGraph.formulaRefs` iterates in the order `addFormula` was
called (`Set` insertion order, findings §7) — this is what makes the DFS
root order, and therefore the whole topological order, deterministic across
runs.

### `Engine` — topological order and cycles

**REQ-RECALC-1.** `Engine#recalc()` computes a topological order via
post-order DFS over `DepGraph`'s forward ("reads") edges. No reversal step:
a ref is appended to the order only after every ref it depends on is
already in the order (§3).
**REQ-RECALC-2.** Three colors — white (unvisited), grey (on the current
path), black (finished) — distinguish a cycle (grey-on-grey) from a diamond
(black-on-revisit). A diamond dependency must never be reported as `#CIRC!`.
**REQ-RECALC-3.** On a grey-on-grey encounter, every ref on the DFS path
from the grey ancestor (inclusive) to the current ref (inclusive), recovered
via parent pointers, is added to the cyclic set. `A1 = A1` produces a
one-element cyclic set.
**REQ-RECALC-4.** Every ref in the cyclic set, and every ref transitively
reachable from it via reverse edges, is assigned `Value.ErrorVal.of(#CIRC)`
before the eval sweep begins, and is never passed to `Ast#eval`.
**REQ-RECALC-5.** Every non-poisoned formula ref is evaluated by
`Ast#eval(ctx)` exactly once per `recalc()`, in topological order, so no
formula ever observes an uncomputed dependency.
**REQ-RECALC-6.** `Engine#setLiteral(ref, value)` (§5) recomputes exactly
the formula cells in `ref`'s transitive-dependents closure, in the cached
topological order, without rebuilding the graph or re-sweeping unrelated
cells.
**REQ-RECALC-7.** The fiber-based demand-driven design (§6.1) is not
implemented in v1. `recalc/` contains no `Fiber` usage. The decision to
revisit this is deferred per §6.4, not resolved by this document.

### Test hooks

| REQ | Fixture shape | Suite | Expected |
|---|---|---|---|
| REQ-RECALC-3 | `A1 = A1` | `suites/recalc_self_cycle.ph` | `A1` -> `#CIRC!` |
| REQ-RECALC-3/4 | `A1 = B1`, `B1 = A1` | `suites/recalc_mutual_cycle.ph` | `A1`, `B1` both -> `#CIRC!` |
| REQ-RECALC-1 | `A1 = A2+1`, `A2 = A3+1`, ..., 20 cells deep | `suites/recalc_long_chain.ph` | every cell resolves; order matches dependency order exactly |
| REQ-RECALC-2 | `A1 = B1+C1`, `B1 = D1`, `C1 = D1`, `D1 = 5` | `suites/recalc_diamond.ph` | all four resolve; `D1` evaluated exactly once; **no** `#CIRC!` |
| REQ-RECALC-4 | `A1 = B1`, `B1 = A1` (cycle), `C1 = A1+1` (downstream, not itself in the cycle) | `suites/recalc_cycle_downstream.ph` | `A1`, `B1`, **and** `C1` all -> `#CIRC!` |
| REQ-RECALC-6 | Build a diamond (as above), run `recalc()`, then `setLiteral(D1, 10)` | `suites/recalc_incremental.ph` | only `B1`, `C1`, `A1` re-evaluate; a counter on `Ast#eval` proves cells outside the closure are untouched |
| REQ-DEP-4 | `A1 = SUM(A2:A4)` | `suites/recalc_range_deps.ph` | `DepGraph.dependenciesOf(A1) == {A2, A3, A4}` |

Every row above is a `REQ` with at least one test, per
[01-architecture.md](01-architecture.md) §7's traceability rule. Indexed in
[14-traceability.md](14-traceability.md).
