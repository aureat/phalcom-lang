// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PASS
// Ported from wren/test/core/map/churn.wren — regression for tombstone-slot
// reuse: repeatedly insert then remove an older key so the bucket churns
// through insert/remove cycles without leaking live entries or deadlocking.
// Wren drove the 0...100 span with a `for`/range; Phalcom has no `for`-over-
// numeric-range sugar, so this uses an equivalent `while` counter.

const m = Map.new()
let i = 0
while (i < 100) {
  m.at(i, put: i)
  (i >= 10).ifTrue { m.remove(i - 10) }
  i = i + 1
}
System.print(m.size)
