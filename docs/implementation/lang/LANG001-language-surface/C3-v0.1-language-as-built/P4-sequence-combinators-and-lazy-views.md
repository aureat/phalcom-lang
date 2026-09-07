# U-SEQ — Work order: `Sequence`-breadth combinators + lazy view classes over `Iterable`

_Self-contained implementation plan for **one** implementer. **Reviewer OFF candidate** (pure `.ph`
over an already-landed protocol, zero primitives, zero compiler/VM touch — same shape as U-STD, which
ran reviewer-OFF) — confirm against the current reviewer roster policy at dispatch time; if any doubt,
default reviewer ON since this unit reopens a kernel file (`core.ph`) live editors share. Green gate:
`./scripts/verify.sh` exits 0 + `cargo doc --workspace --no-deps` clean. Grounded in
**[iteration.md](../../../spec/current/iteration.md) §5**, **[ADR-0035](../../../adr/0035-iteration-protocol-cursor.md)**,
**[collection-protocol.md](../../../spec/current/core/collection-protocol.md)**, and Wren's `Sequence` +
view-class precedent (`wren_core.wren` L7–182, cited throughout as precedent-with-consequence, not
gospel — see §3.4 for the one place the port deliberately diverges). New governing artifact: **none**
required for the combinator breadth (pure derivation, ADR-0019 unaffected); **DEC-SEQ-A** (§8) is a real
fork in the surface and is flagged, not picked.

## 1. Mission (one sentence)
Extend the kernel `Iterable` root — landed by **U-ITERABLE** (hard precondition, §2) — with the
Wren-`Sequence` combinator breadth Phalcom lacks (`all`/`any`/`count`/`count(_)`/`find`/`join`/`join(_)`/
`toList`) and four lazy view classes (`MapView`/`WhereView`/`SkipView`/`TakeView`) that let a pipeline
like `coll.where(p).map(f).take(3)` allocate only small wrappers and materialize nothing until iterated —
**all pure `.ph` over `iterate`/`iteratorValue`, zero new floor primitives, zero new opcode.**

## 2. Preconditions (verify on actual dispatch HEAD — do not assume; this unit does NOT run until these hold)
- **U-ITERABLE landed — hard, non-negotiable precondition.** U-SEQ is a pure follow-on; it does not
  create `Iterable`, does not wire any class's `superclass`, and does not touch `universe.rs`. Before
  writing a line, confirm on HEAD:
  1. `class Iterable` exists in `core.ph` and `List`/`Map`/`Set`/`Tuple`/`Range` all have `superclass ==
     Iterable` (`graphify explain "Iterable"` / `grep -n "class Iterable" core.ph` / check `universe.rs`
     `CoreClasses` wiring).
  2. `Iterable` carries `each`/`map`/`filter`/`reduce`/`includes`/`isEmpty` (moved off `List` per
     U-ITERABLE's mission) — confirm these are **no longer** `List`-local (they should be inherited).
  3. **The bare-cursor contract is live**: `iterate(_)` returns the raw next cursor value while
     iterating and the **`None` singleton** at exhaustion — **not** `Some(cursor)`/`None` (the
     pre-U-ITERABLE shape captured in ADR-0035 as originally ratified, and still visible in the current
     HEAD `List#iterate`/`Map#iterate`/`Set#iterate`/`Tuple#iterate`/`Range#iterate` bodies, e.g.
     `phalcom-core/core/core.ph` L318–321, L482–485, L545–548, L595–598, L695–698 as of this grounding —
     **all five are expected to change shape under U-ITERABLE**; re-read them fresh, don't trust these
     line numbers). This is the single fact every view class body below depends on — if U-ITERABLE
     shipped a different cursor convention than "bare value / `None`", **stop and re-derive §3.3 from
     whatever actually landed** before writing any view class.
  4. Confirm U-ITERABLE amended (or superseded) **ADR-0035** to record the bare-cursor shape — if it
     landed as an undocumented behavior change with no ADR delta, flag that as a gap for the orchestrator
     (not this unit's job to fix, but don't build silently on an undocumented contract).
- **`for`/`break`/`continue` landed (U-ITER, confirmed landed at this grounding — `c35171a` "migrate
  List#each off index size/at onto cursor protocol" and `for` desugar are in `core.ph`/`compiler/lib.rs`
  already).** All combinators below are written over `for (x in self)`, never raw index math.
- **`Tuple` landed (U-COLLTYPES Phase 2, confirmed in `core.ph` L581+).** `TakeView`'s cursor
  representation (§3.3) needs the `(a, b)` literal and `Tuple#at(_)`.
- **`Error`/`throw` landed (U-ERR, confirmed — `7c901cf` "throw/try/catch/on/ensure + Result/Ok/Err").**
  Guard clauses raise via `throw Error(msg)`. **`ArgumentError` is NOT yet landed** (U-CORE-6's as-built
  explicitly scopes it out — "U-CORE-6 does not pick this up... only the dNU→`MessageNotUnderstood`
  slice lands here"; `TypeError`/`ArgumentError`/`RangeError` remain ❌ in `catalog-delta.md` §2 at this
  grounding). **Re-verify at dispatch time** — if `ArgumentError` has landed by then, use it (matches the
  mission's stated idiom); if not, use `Error` and leave a one-line `DEFERRED.md` pointer ("mechanical
  rename `Error`→`ArgumentError` once it lands, zero behavior diff beyond the raised class").
- Baseline `./scripts/verify.sh` green before the first edit. Re-run `graphify affected "core.ph"` and
  check concurrent `core.ph` editors (§4.1) — **do not co-schedule with U-ITERABLE itself** (same file,
  hard sequence, not parallel).

## 3. Design (realise iteration.md §5's promise — "combinators are `.ph` over the protocol" — for the
Wren-parity residual; do not re-litigate the protocol itself, that is ADR-0035/U-ITERABLE's job)

### 3.1 Combinator breadth — reopen `Iterable`, unconditional, unblocked by DEC-SEQ-A
Ported from `wren_core.wren` `Sequence` (L7–119), one contract for every conformer (`List`, `Map`,
`Set`, `Tuple`, `Range`, user iterables, and the view classes in §3.3 — all get these for free the
moment `iterate`/`iteratorValue` exist):

```phalcom
// U-SEQ (iteration.md §5 extension): Wren `Sequence` breadth Phalcom lacked, ported to the bare-
// cursor protocol. All written over `for (x in self)` — NOT `self.each { }` — so a per-class `each`
// override with a different block arity (Map's `each(f)` is 2-arg: key+value) never leaks into these;
// `for` always yields exactly one value per step (Map/Set's cursor yields the KEY, DEC-CT-E), which is
// the uniform shape every combinator below needs.

all(f) {
  for (x in self) {
    f.call(x).ifFalse { return false }
  }
  return true
}

any(f) {
  for (x in self) {
    f.call(x).ifTrue { return true }
  }
  return false
}

// 0-arg: full-traversal length. Deliberately NOT delegating to `size` — `Iterable` has no native
// `size` (only the concrete kernel types + Range do), and the view classes in §3.3 have none at all —
// this is the generic form every conformer gets whether or not it has an O(1) `size`.
count {
  var n = 0
  for (x in self) { n = n + 1 }
  return n
}

count(f) {
  var n = 0
  for (x in self) { f.call(x).ifTrue { n = n + 1 } }
  return n
}

// Option-returning (no surface nil, Invariant 4) — the one place this port differs textually from
// Wren's `for/if/return null` shape.
find(f) {
  for (x in self) {
    f.call(x).ifTrue { return Some.new(x) }
  }
  return None
}

join => self.join("")

join(sep) {
  var first = true
  var result = ""
  for (x in self) {
    first.ifFalse { result = result + sep }
    first = false
    result = result + x.toString
  }
  return result
}

toList {
  var result = List.new()
  for (x in self) { result.add(x) }
  return result
}
```

**Deliberately not ported** (scope-locked, do not add): Wren's `contains(element)` — Phalcom already has
`includes(x)` (U-ITERABLE) doing the identical job; Wren's unseeded `reduce(f)` (aborts on empty) — the
locked scope is the eight selectors above only, `reduce(_,_)` already exists (U-STD/U-ITERABLE). Adding
either is scope creep for a future unit, not this one.

**`all`/`any`/`find` short-circuit via an ordinary `return`, not `break`.** The `for` body is *inlined*
into this method's own frame (ADR-0035 §4 — `for` never allocates a block), so `return` inside it is a
plain function return, no non-local-return/frame-token machinery, and **no `break`/`continue` is used
anywhere in this unit** — sidesteps the live, documented deopt-`break`-in-materialized-block hazard
(`U-ITER-FIX` item 1, "silent no-op" on a break reached through a materialized block) entirely, because
that hazard is specific to `break`/`continue`, not `return`.

### 3.2 `Map`/`Set` conformance note (not new work — a documented consequence)
`Map`/`Set`'s `iterate`/`iteratorValue` yield **keys** (DEC-CT-E, already landed). So
`someMap.all(f)`/`.count(f)`/`.find(f)`/`.join`/`.toList` all traverse **keys**, matching
`for (k in someMap)` today. Not a new decision — just make it explicit so nobody is surprised
`someMap.toList` returns a `List` of keys, not entries. A view built over a `Map`
(`someMap.where(p)` under any DEC-SEQ-A branch) inherits this — `p` receives keys too.

### 3.3 Lazy view classes (Wren `MapSequence`/`WhereSequence`/`SkipSequence`/`TakeSequence`,
`wren_core.wren` L121–182) — the classes themselves are unconditional; only how they are *reached* by
sugar-method name is gated by DEC-SEQ-A (§8). Each wraps a source `Iterable` (+ closure/count) and
implements `iterate`/`iteratorValue` lazily over the bare-cursor contract (§2 item 3):

```phalcom
// Lazy view classes (Wren precedent, wren_core.wren L121-182), ported to Phalcom's bare-cursor
// protocol. `extends Iterable` so every §3.1 combinator, `for`, and every other view work on a view
// for free. PORTING NOTE (precedent-with-consequence): Wren's `while (iterator = seq.iterate(iterator))`
// idiom is assignment-as-truthy-condition — does not port. Phalcom conditions must be a real `Bool`
// (ADR-0021, no truthiness; no assignment expressions), so every loop below is an explicit
// `cur = self._source.iterate(cur); (cur == None).ifTrue { ... }` — a mechanical, not semantic, rewrite.
// Cross-tag `==` (comparing a Number/other cursor value against the `None` singleton) is safe and
// returns `false`, never a dNU or panic — the tagged-`Value` `object_eq` floor compares by variant
// first (ADR-0010); confirm this still holds on dispatch HEAD before relying on it in a hot loop.

class MapView is Iterable {
  @constructor
  new(source, fn) {
    _source = source
    _fn = fn
  }
  iterate(cursor) => self._source.iterate(cursor)
  iteratorValue(cursor) => self._fn.call(self._source.iteratorValue(cursor))
}

class WhereView is Iterable {
  @constructor
  new(source, pred) {
    _source = source
    _pred = pred
  }
  // The one place a view does real work inside `iterate` (skips non-matching source elements at the
  // step level, wren_core.wren L174-179) — still lazy overall: nothing runs until the caller asks.
  iterate(cursor) {
    var cur = cursor
    while (true) {
      cur = self._source.iterate(cur)
      (cur == None).ifTrue { return None }
      self._pred.call(self._source.iteratorValue(cur)).ifTrue { return cur }
    }
  }
  iteratorValue(cursor) => self._source.iteratorValue(cursor)
}

class SkipView is Iterable {
  @constructor
  new(source, count) {
    (count.is(Number) and (count >= 0)).ifFalse {
      throw Error("skip: count must be a non-negative Number")   // -> ArgumentError once landed, §2
    }
    _source = source
    _count = count
  }
  iterate(cursor) {
    (cursor != None).ifTrue { return self._source.iterate(cursor) }
    var cur = self._source.iterate(None)
    var n = self._count
    while ((n > 0) and (cur != None)) {
      cur = self._source.iterate(cur)
      n = n - 1
    }
    return cur
  }
  iteratorValue(cursor) => self._source.iteratorValue(cursor)
}
```

`TakeView` is deliberately **not** a line-for-line port — see §3.4.

### 3.4 `TakeView` — the one deliberate divergence from Wren (fixing a real law violation, not style)
Wren's `TakeSequence` tracks progress in a **mutable instance field** `_taken`, incremented every
`iterate` call and never reset:
```wren
iterate(iterator) {
  if (!iterator) _taken = 1 else _taken = _taken + 1
  return _taken > _count ? null : _sequence.iterate(iterator)
}
```
This means a `TakeSequence` instance is only *safely* iterable **once** — a second `for` over the same
value silently returns the wrong (empty, or truncated-early) result, because `_taken` keeps counting
from where the first traversal left off. That directly violates
[collection-protocol.md](../../../spec/current/core/collection-protocol.md) **law 2 — deterministic
iteration: "two traversals of an unmutated collection agree."** This is a known, real Wren gotcha, not
a semantic worth reproducing — **the fix is to make `iterate` a pure function of its `cursor` argument**,
carrying the running "taken so far" count *inside the cursor* rather than in instance state, using the
already-ratified `(a, b)` `Tuple` literal (ADR-0032) as the composite cursor:

```phalcom
class TakeView is Iterable {
  @constructor
  new(source, count) {
    (count.is(Number) and (count >= 0)).ifFalse {
      throw Error("take: count must be a non-negative Number")
    }
    _source = source
    _count = count
  }
  // Cursor = (sourceCursor, takenSoFar) — a pure function of `cursor`, so re-iterating the SAME
  // TakeView instance twice gives identical results both times (law 2). Unlike Wren's TakeSequence
  // (see prose above), no instance field is mutated by `iterate`.
  iterate(cursor) {
    var srcCursor = None
    var taken = 0
    (cursor != None).ifTrue {
      srcCursor = cursor.at(0)
      taken = cursor.at(1)
    }
    ((taken + 1) > self._count).ifTrue { return None }
    var next = self._source.iterate(srcCursor)
    (next == None).ifTrue { return None }
    return (next, taken + 1)
  }
  iteratorValue(cursor) => self._source.iteratorValue(cursor.at(0))
}
```
Cost: one `Tuple` allocation per step instead of zero (Wren's version is allocation-free per step). This
is the correct trade for a *default* combinator that must satisfy law 2 unconditionally — flag it in the
Return contract as a named, deliberate deviation, not an oversight.

### Rubric — hazards & preclusion (mandatory)

- **Fiber-generator interaction — inherited hazard, not a new one; verify the boundary, don't assume it
  away.** `for (x in view)` itself contains **no `block_call`** — same disasm-golden guarantee `for`
  already has for any iterable (ADR-0035 §4/§6; U-ITER's disasm golden), because `view.iterate`/
  `view.iteratorValue` are ordinary sends, and `for`'s own scaffold is inlined. **But** `MapView#iteratorValue`
  and `WhereView#iterate` internally call `self._fn.call(...)`/`self._pred.call(...)` — the user's stored
  closure, invoked via the same native `Function#call` primitive that `.each { yield }` already goes
  through (this is what `iteration.md` §6/ADR-0030 §4 call the "`block_call` inside a combinator" case,
  documented by the existing `each_generator_raises` PENDING fixture, U-ITER-FIX item 5). **This is not a
  new hazard U-SEQ introduces** — it is the *same* documented hazard, now reachable through more call
  sites: `all`/`any`/`count(_)`/`find` (§3.1, they call `f.call(x)` directly) and any view that stores a
  closure (§3.3). The generalizable rule to record: **a `Fiber.yield` lexically inside a `Block` object
  invoked via `.call(_)` is unsafe (raises `CannotYieldAcrossNativeFrame`); a `Fiber.yield` lexically
  inside an inlined `for`/`while` body is safe.** This holds transitively through a view chain — `xs
  .where(p).map(f).take(3)` is exactly as yield-unsafe inside `p`/`f` as `xs.each { yield }` is today, and
  exactly as yield-safe as any other iterable in a bare `for` loop. No new fixture is required to prove
  this (the mechanism is already covered); optionally add one combinator-specific PENDING fixture
  (`all_generator_raises`) for redundant documentation once Fiber lands, not mandatory.
- **Laziness ⊗ effect-timing (the reason DEC-SEQ-A matters, §8).** Under an eager `map`/`filter`, `f`'s
  side effects run immediately, once per element, at the `.map(f)` call site. Under a `MapView`/
  `WhereView`, `f`/`pred` runs **later, incrementally, only for elements actually visited** by a
  subsequent `for`/`.toList`/combinator — and **never runs at all** if the resulting view is never
  iterated. This is exactly the well-known Wren/Python-generator surprise (`print` inside `.map` doesn't
  fire until forced). Phalcom already has *block*-laziness precedent (`and`/`or`/`ifTrue` args are
  unevaluated until sent, control-flow.md §1 — "laziness falls out of the object model") but **not**
  *collection*-laziness precedent — this would be new user-observable surface behavior if DEC-SEQ-A
  picks (A). Document this explicitly in whichever DEC-SEQ-A branch ships; it is not a bug in any branch,
  but silence about it would be.
- **`TakeView` law-2 compliance — the fix in §3.4, verified by a dedicated golden (§7).** Any future view
  class must be checked against collection-protocol.md law 2 the same way; this is the template.
- **Fresh-cursor purity, not shared mutable view state.** `MapView`/`WhereView`/`SkipView`/`TakeView` all
  store only their construction-time source/closure/count — **no field is ever reassigned after
  `construct`** (verified: `TakeView`'s fix in §3.4 is precisely what makes this true for all four).
  Multiple independent `for` loops over the *same* view instance, or nested views sharing a source, are
  therefore safe by construction — pin this with the repeatable-traversal golden (§7).
- **No `Value`/dispatch/opcode change.** Every method above is `.ph` over existing sends (`for`, `.call`,
  `.at`, `.isA`, `==`/`!=`, arithmetic). Net floor delta: **0**. No ADR-0019 amendment.
- **Precedent, explicitly bounded.** Wren's `Sequence`+view-class model is the direct precedent for
  *why* this stays small (one mixin + four wrapper classes cover the whole combinator surface); the cost
  side of that precedent — a wrapper allocation per view and a virtual dispatch per step versus a hot
  monomorphic native loop — is the same "collection protocol ⊗ dispatch cost" trade-off the language-design
  skill's `stdlib.md` names as a crown-jewel hazard (mitigated later by IC population, ADR-0012, already
  deferred by design — not this unit's problem to solve).

## 4. Confirmed write-set (tight & disjoint; re-validate with `graphify affected` on HEAD)

| File | Why | Slice |
|---|---|---|
| `phalcom-core/core/core.ph` | reopen `Iterable` (§3.1) + four new view classes (§3.3/§3.4) + the DEC-SEQ-A sugar-method wiring (§8) | protocol |
| `phalcom-core/tests/lang/sequence/` (**new label**) + `tests/lang/MANIFEST.md` | goldens + negatives + the law-2 repeatability golden | tests |

**Deliberately NOT in scope:** `phalcom-ast/*`, `compiler/lib.rs`, `vm.rs`, `bytecode.rs`, `primitive/*`,
`universe.rs`, `heap.rs`, `value.rs` — zero Rust touched, this is a `.ph`-only unit exactly like U-STD.
Also **not** `iteration.md`/`catalog-delta.md`/`collection-protocol.md` themselves (spec docs — the
orchestrator updates those against whichever DEC-SEQ-A branch is picked and this unit's as-built).

### 4.1 Write-set collision risk (flag, don't resolve)
- **`phalcom-core/core/core.ph` — never two editors.** Hard-sequence after **U-ITERABLE** (same file,
  same class, direct dependency — not a parallelizable pair despite touching "the same layer"). Also
  check for any other live `core.ph` unit at dispatch time (the U-CORE track has a history of concurrent
  sessions on this file, memory: "core.ph is contended"); if one is in flight, wait, don't interleave.
- **`tests/lang/MANIFEST.md`** — shared ledger; append-only, last-writer-wins on the git level, but
  confirm no concurrent unit is mid-edit on it before committing (same discipline as every other unit
  touching it, e.g. U-ITER §4.1).

## 5. Build order (small, independently-green diffs)
1. **Combinator breadth (§3.1)** — reopen `Iterable`, add all eight selectors. Independently testable
   without any DEC-SEQ-A resolution — **this slice can land regardless of how §8 resolves.** Green.
2. **View classes, unconstructed sugar (§3.3/§3.4)** — add `MapView`/`WhereView`/`SkipView`/`TakeView`
   as bare classes, reachable only via `ClassName.new(...)` (this is exactly DEC-SEQ-A option (C)'s
   shape) + goldens proving each view's `iterate`/`iteratorValue` contract directly. Still
   DEC-SEQ-A-agnostic — every branch needs these classes to exist first.
3. **DEC-SEQ-A sugar wiring** — **BLOCKED until the user picks a branch (§8).** Wire whichever selector
   names/eagerness the resolution picks. This is the only step gated on the open decision; steps 1–2 are
   unblocked and can dispatch today.
4. **Pipeline golden + repeatability golden** — `coll.where(p).map(f).take(3)` (using whatever selector
   names step 3 wired) + the `TakeView` law-2 repeatable-traversal golden (§3.4) + the Map-yields-keys
   conformance golden (§3.2). Green.

## 6. Mandatory rules
- **Docs:** `///`-equivalent `.ph` doc comments on every new class/method, citing iteration.md §5 /
  ADR-0035 / this plan, matching the density already in `core.ph`'s `List`/`Map`/`Set`/`Tuple`/`Range`
  blocks. `cargo doc --workspace --no-deps` unaffected (no Rust) but re-run for safety (zero new
  warnings expected).
- **Green gate:** `./scripts/verify.sh` exits 0; no new clippy (no Rust changed, but the gate still runs
  the full suite); no `unsafe` (moot, no Rust).
- **Reviewer:** default ON per the file-sensitivity note in the header; downgrade to OFF only if the
  live roster policy explicitly allows self-verified `.ph`-only units at dispatch time (U-STD's
  precedent).

## 7. Test strategy (the green gate must assert) — new `sequence` label
- **`all`/`any` (PASS):** true/false cases; **short-circuit proof** — a side-effecting predicate
  (increments a captured counter) run over a collection where the answer is decided at element 2 of 5;
  assert the counter stops at 2, not 5.
- **`count`/`count(f)` (PASS):** arity-0 traversal length on a `List` **and** on a view (no native
  `size`, proving the traversal-based derivation is load-bearing, not redundant with `size`); predicate
  form counts matches only.
- **`find` (PASS):** hit → `Some(x)`; miss → `None` (not a surface `nil` — Invariant 4 check).
- **`join`/`join(sep)` (PASS):** default `""` separator; explicit separator; **empty collection → `""`**
  (no leading/trailing separator — the `first` flag boundary case); elements stringified via
  `.toString`, not raw concatenation (mixed-type collection golden).
- **`toList` (PASS):** from a concrete collection (identity-preserving copy) **and** from a view
  (the materialization proof — the view was never a `List` until this call).
- **View classes, direct (PASS):** `MapView`/`WhereView`/`SkipView`/`TakeView` each driven by a raw
  `for` loop, asserting the exact element sequence against a hand-computed expectation (mirrors the
  Countdown-style user-iterable proof from U-ITER §7).
- **Laziness (PASS — the DEC-SEQ-A-branch-specific golden, write once the branch is picked):** a view
  built via whatever the chosen sugar method is, with a side-effecting closure; assert the closure has
  **not** run at construction time, and runs exactly once per element **only when iterated** (§ Rubric,
  laziness ⊗ effect-timing).
- **`TakeView` repeatability (PASS — the law-2 golden, §3.4):** iterate the **same** `TakeView` instance
  twice (two independent `for` loops, or `.toList` called twice); assert byte-identical results both
  times. This is the regression guard against the Wren mutable-`_taken` wart.
- **Map/Set conformance (PASS — §3.2):** `someMap.toList`/`.find(_)`/etc. operate over **keys**,
  matching `for (k in someMap)`; a view over a `Map` also sees keys.
- **Pipeline (PASS — the flagship):** `coll.where(p).map(f).take(3).toList` (selector names per §8's
  resolution) matches a hand-computed expected `List`, exercising all four views chained.
- **NEGATIVE:** `skip`/`take` (or their DEC-SEQ-A equivalents) with a negative count → raised error
  (`ArgumentError` if landed, else `Error`, per §2); a non-`Number` count → same.
- **PENDING (optional, `sequence/pending/`):** `all_generator_raises` — documents the inherited
  block_call/Fiber hazard (§ Rubric) for this unit's specific combinators; `#[ignore]` until Fiber lands,
  same pattern as U-ITER's `for_generator_suspends`/`each_generator_raises`. Not mandatory for green.

## 8. Decisions flagged (flag, don't pick)

| ID | Decision | Options | Architect recommendation |
|---|---|---|---|
| **DEC-SEQ-A** ✅ **RESOLVED (user, 2026-07-13): (A) lazy `map`/`where`/`skip`/`take`, breaking — run the migration audit below before merge; step 3 of §5 is now unblocked.** | **Eager-vs-lazy default for `map`/`filter`/the new `where`/`skip`/`take` sugar.** U-ITERABLE ships `map`/`filter` **eager** (return a `List`). Wren's `map`/`where` are **lazy** (return view sequences). | **(A)** `map` becomes lazy-by-default (returns `MapView`, **BREAKING** — `list.map(f)` no longer returns a `List`; any caller chaining a `List`-only selector on the result breaks, though the result `is Iterable` so `for`/every §3.1 combinator/`toList` still work); add `where(pred)`/`skip(n)`/`take(n)` as new lazy sugar; `filter` is left as-is (stays eager, untouched — Wren has no separate eager/lazy filter name, but Phalcom already has a landed eager `filter` selector this option does not touch). **(B)** keep `map`/`filter` eager exactly as U-ITERABLE shipped them (zero breakage); add distinctly-named lazy sugar `lazyMap(f)`/`where(pred)`/`skip(n)`/`take(n)` — fully additive. **(C)** keep everything eager; views exist only as explicit constructors (`MapView.new(coll, f)`) — zero sugar, zero risk, but defeats the fluent-pipeline ergonomics the mission asks for (`TakeView.new(WhereView.new(coll, p), f, 3)`-style nesting is not what `coll.where(p).map(f).take(3)` is for). | **(A)**, with a migration audit before merge: grep the tree (core.ph, examples/*.ph, every `.ph` test fixture) for `.map(` call sites and confirm none chain a `List`-only selector (`add`/`at(_,put:)`/`rawSet` etc.) directly off the result without an intervening `.toList`. If the audit finds zero such call sites (plausible — `map` is young, U-STD only landed it recently), (A) is a clean, low-risk breaking change that buys the idiomatic Wren-parity pipeline for free and matches the mission's explicit framing. If the audit finds real call sites, fall back to **(B)** rather than fixing them speculatively. **This is a real behavior fork the user must confirm — do not dispatch step 3 of §5 until they do.** |
| **DEC-SEQ-B** (minor, low-stakes — pick a default if the user doesn't weigh in, but note the pick) | **View class naming** (`MapView`/`WhereView`/`SkipView`/`TakeView` vs. a `*Sequence` suffix mirroring Wren verbatim vs. a `*Cursor` suffix). | **(A)** `*View` (this plan's choice — reads as "a view over a source", avoids implying an allocated sequence/collection); **(B)** `*Sequence` (verbatim Wren naming, but Phalcom has no `Sequence` root class — `Iterable` is the root — so the family-name echo would be misleading); **(C)** `*Cursor`. | **(A)** — already used consistently in this plan; grepped clean (no existing `View`-suffixed class in `core.ph` or the specs at this grounding). Low-stakes; the implementer may rename mechanically if the user prefers otherwise, no design impact either way. |
| **DEC-SEQ-C** (tracks §2's precondition) | **`ArgumentError` vs `Error` in the two guard clauses (§3.3 `SkipView`/`TakeView` constructors).** | **(A)** use `ArgumentError` if landed by dispatch time; **(B)** use `Error` and leave a `DEFERRED.md` rename pointer. | Verify at dispatch time and pick automatically (not a real design fork — mechanical). Not user-blocking. |

## 9. Must-not-preclude check
- **A future `Fiber`-backed generator/`Stream` layer (iteration.md §6/§7):** not precluded — views are
  cursor-based, not `Fiber`-based, and orthogonal; a `Stream` layer, if ever built, sits *beside* this
  unit's views, not on top of them. The transitive `block_call` hazard (§ Rubric) is already the
  documented boundary; U-SEQ adds no new boundary.
- **String conforming to `Iterable` later (byte/codepoint iteration, Wren's `String is Sequence`):** not
  precluded — Phalcom's `String` doesn't implement `iterate`/`iteratorValue` yet (out of scope here); the
  moment a future unit adds them, `String` inherits every §3.1 combinator and can be wrapped by every
  §3.3 view "for free," exactly as the protocol promises. Nothing in this unit assumes only List-shaped
  sources.
- **The Int/Float surface split (ADR-0024, ratified but not yet implemented — `core.ph` still has a
  single flat `class Number {}` at this grounding):** not precluded — `SkipView`/`TakeView`'s guard
  clauses check `count.is(Number)` generically, not `Int` specifically (there is no surface `Int` class
  to check against yet). When the split lands, tightening the guard to `Int` is a strictly additive,
  non-breaking follow-up (a real `Int` argument already satisfies `isA(Number)` today under any sane
  tower design, so no existing caller breaks).
- **A future `IC`/inline-cache pass (ADR-0012, deferred by design):** not precluded — nothing here
  assumes megamorphic-safe dispatch or forecloses per-call-site specialization; the "collection protocol
  ⊗ dispatch cost" trade (§ Rubric) is explicitly named as a *known, deferred* cost, not something this
  unit works around.
- **A future `match`/pattern-matching unit (open-questions.md Q7 residual — map patterns, match arms,
  still deferred beyond the U14 destructuring slice):** not precluded — `TakeView`'s `(sourceCursor,
  takenSoFar)` `Tuple` cursor uses only `Tuple#at(_)`, never destructuring syntax, so it has no
  dependency on and creates no obligation toward a future `match` unit.
- **DEC-SEQ-A's non-chosen branches:** not foreclosed by shipping any one of them — (A)/(B)/(C) differ
  only in which selector names route to the (unconditionally-shipped) view classes; switching branches
  later is a sugar-method rename/add, not a class redesign, since §3.1/§3.3/§3.4 are identical across all
  three.

## 10. Return contract (report to whoever gates this unit)
The exact `Iterable` reopen diff (§3.1, all eight selectors) · confirmation combinators are written over
`for`, not `each`, and why (Map's 2-arg `each` override, §3.1 header comment) · the four view classes
(§3.3) + the `TakeView` law-2 fix and its dedicated repeatability golden (§3.4) · which DEC-SEQ-A branch
the user picked (or confirmation step 3 of §5 stayed undispatched pending that pick) · the resulting
exact sugar-method surface (`map`/`filter`/`where`/`skip`/`take`/`lazyMap` — whichever subset the branch
produced) · the Fiber/`block_call` transitive-hazard note and whether the optional PENDING fixture was
added · the migration-audit result if (A) was picked (call sites checked, none/some found, how resolved)
· confirmation **net floor delta = 0**, no Rust file touched · the `sequence` corpus label + MANIFEST
bump · how DEC-SEQ-B/C resolved · `verify.sh` + `cargo doc` tails · any new `DEFERRED.md` entries
(`ArgumentError` rename pointer if applicable; the optional `all_generator_raises` PENDING fixture).
