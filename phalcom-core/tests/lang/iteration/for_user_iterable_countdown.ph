class Countdown is Iterable {
  @constructor
  from(n:) { _n = n }
  iterate(cursor) {
    const next = (cursor == None).ifTrue({ _n }, ifFalse: { cursor - 1 })
    return (next >= 0).ifTrue({ next }, ifFalse: { None })
  }
  iteratorValue(cursor) => cursor
}
for (x in Countdown.from(n: 3)) { System.print(x) }
