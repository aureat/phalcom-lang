// C-ITER-5 (iteration.md §1, spec §2.3): a non-`List` user type drives `for`
// purely through its own two `.ph` selectors — proof that
// `iterate(_)`/`iteratorValue(_)` stay non-inlined and overridable.
class Countdown {
  construct from(n:) { _n = n }
  iterate(cursor) {
    let next = cursor.map { c => c - 1 }.unwrapOr(_n)
    return (next >= 0).ifTrue { next }
  }
  iteratorValue(cursor) => cursor
}
for (x in Countdown.from(n: 3)) { System.print(x) }
