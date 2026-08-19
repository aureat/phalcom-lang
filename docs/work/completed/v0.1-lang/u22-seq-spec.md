# u-22-seq-spec

> Companion to [`err-plan.md`](u22-seq-plan.md) — read that first for rationale/rubric/hazards. This file is the
> copy-paste-ready source: §1 (combinator breadth) and §2 (view class bodies) are **unconditional** and
> can be dropped into `core.ph` today. §3 (sugar-method wiring) has one literal block **per DEC-SEQ-A
> branch** — drop in whichever the user picks; do not merge more than one.

## 0. Placement
- §1 goes inside the `Iterable` class body (wherever U-ITERABLE landed it — reopen or extend the same
  block, verify exact line on HEAD).
- §2's four classes are new top-level `class … extends Iterable { … }` blocks. Suggested placement:
  immediately after the `Range` block (after the kernel collection classes, before `Future`), mirroring
  `core.ph`'s existing ordering (kernel collections, then concurrency primitives).
- §3 goes inside `Iterable` alongside §1 (the sugar methods are ordinary `Iterable` selectors).

## 1. Combinator breadth (unconditional — reopen `Iterable`)

```phalcom
// U-SEQ (iteration.md §5 extension, wren_core.wren Sequence L7-119 precedent): combinator breadth
// Phalcom lacked. All written over `for (x in self)`, never `self.each { }` (Map's `each(f)` is 2-arg —
// see plan.md §3.1) and never index math. Zero new floor primitives.

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

## 2. View classes (unconditional)

```phalcom
// Lazy view classes (wren_core.wren MapSequence/WhereSequence/SkipSequence L121-152, 168-182), ported
// to Phalcom's bare-cursor protocol (post-U-ITERABLE: `iterate` returns the raw next cursor, or the
// `None` singleton at exhaustion — never Some-wrapped). `extends Iterable` so §1's combinators, `for`,
// and every other view work on a view for free.

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
      throw Error("skip: count must be a non-negative Number")   // -> ArgumentError once landed
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

// NOT a line-for-line Wren port — see plan.md §3.4. Wren's TakeSequence mutates an instance field
// (`_taken`) inside `iterate`, so a Wren TakeSequence is only safely iterable ONCE; that violates
// collection-protocol.md law 2 (deterministic, repeatable iteration). This version carries the running
// count INSIDE the cursor (a `(sourceCursor, takenSoFar)` Tuple), so `iterate` stays a pure function of
// its argument and the SAME TakeView instance can be traversed any number of times with identical
// results (pin: the repeatability golden, plan.md §7).
class TakeView is Iterable {
  @constructor
  new(source, count) {
    (count.is(Number) and (count >= 0)).ifFalse {
      throw Error("take: count must be a non-negative Number")
    }
    _source = source
    _count = count
  }
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

## 3. DEC-SEQ-A sugar wiring — pick exactly ONE branch, inside `Iterable`

### Branch (A) — recommended; `map` becomes lazy (BREAKING); `where`/`skip`/`take` new; `filter` untouched
```phalcom
// U-SEQ DEC-SEQ-A branch (A): `map` is redefined lazy (was eager List-returning under U-ITERABLE —
// BREAKING, see plan.md §8 migration-audit gate). `filter` is NOT touched (stays the eager U-ITERABLE
// selector); `where` is the new Wren-parity lazy filter. `.toList` is the materializer for all of them.
map(f) => MapView.new(self, f)
where(pred) => WhereView.new(self, pred)
skip(n) => SkipView.new(self, n)
take(n) => TakeView.new(self, n)
```
**This literally replaces/overrides whatever `Iterable#map(f)` U-ITERABLE shipped** — verify the reopen
lands *after* U-ITERABLE's definition in file order (or in the same class body if reopening a single
block) so this is the winning definition, not a dead duplicate.

### Branch (B) — additive only; nothing existing changes
```phalcom
// U-SEQ DEC-SEQ-A branch (B): fully additive. `map`/`filter` are untouched (stay eager, as
// U-ITERABLE shipped them). `lazyMap`/`where`/`skip`/`take` are new lazy sugar with distinct names —
// zero risk, zero migration audit needed.
lazyMap(f) => MapView.new(self, f)
where(pred) => WhereView.new(self, pred)
skip(n) => SkipView.new(self, n)
take(n) => TakeView.new(self, n)
```

### Branch (C) — no sugar at all
Ship nothing in this section. Views are reached only via `MapView.new(coll, f)` /
`WhereView.new(coll, pred)` / `SkipView.new(coll, n)` / `TakeView.new(coll, n)` directly. (Not
recommended — defeats the fluent-pipeline goal; document why if the user picks this anyway.)

## 4. Test fixture skeletons (`phalcom-core/tests/lang/sequence/`)

Positive lane (byte-exact stdout, `.ph` + `.expected` pairs — mirror the existing `list`/`iteration`
label conventions):
- `sequence_all_true.ph` / `sequence_all_false_short_circuits.ph` (counter proves early stop)
- `sequence_any_true_short_circuits.ph` / `sequence_any_false.ph`
- `sequence_count_arity0.ph` (over a `List` **and** over a `WhereView` — no native `size` on the view)
- `sequence_count_predicate.ph`
- `sequence_find_hit.ph` / `sequence_find_miss_returns_none.ph`
- `sequence_join_default.ph` / `sequence_join_custom_sep.ph` / `sequence_join_empty_collection.ph`
- `sequence_tolist_from_range.ph` / `sequence_tolist_from_view.ph`
- `sequence_mapview_basic.ph` / `sequence_whereview_basic.ph` / `sequence_skipview_basic.ph` /
  `sequence_takeview_basic.ph` (each: raw `for` over the view, hand-computed expected sequence)
- `sequence_pipeline_where_map_take.ph` (the flagship composed pipeline, selector names per whichever
  DEC-SEQ-A branch shipped)
- `sequence_takeview_repeatable.ph` (traverse the SAME `TakeView` instance twice — law-2 golden)
- `sequence_view_over_map_yields_keys.ph` (§3.2 conformance)
- `sequence_laziness_closure_runs_on_iteration_only.ph` (branch-specific — write once §3 is picked)

Negative lane (`compile-errors/` or `runtime-errors/`, whichever matches how `Error`/`ArgumentError`
raises are asserted elsewhere in the corpus — check the `errors/` label's convention before adding):
- `sequence_skip_negative_count_raises.ph`
- `sequence_take_non_number_count_raises.ph`

Pending (optional, `sequence/pending/`, `#[ignore]` until Fiber lands):
- `all_generator_raises.ph` — `xs.all { x => Fiber.yield(x); true }` inside `Fiber.new { … }`, expected
  `CannotYieldAcrossNativeFrame`, documenting the transitive block_call hazard (plan.md § Rubric).
