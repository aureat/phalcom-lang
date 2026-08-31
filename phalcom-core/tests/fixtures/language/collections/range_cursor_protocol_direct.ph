// area: collections
// spec: E.2; ADR-0035 §1; ADR-0048
// status: PASS
// E.2 cursor protocol: cursor is yielded value, and iteratorValue is identity.

const r = Range.new(1, 3, true)
System.print(r.iterate(None))
System.print(r.iterate(1))
System.print(r.iterate(2))
System.print(r.iterate(3))

const e = Range.new(1, 3, false)
System.print(e.iterate(None))
System.print(e.iterate(1))
System.print(e.iterate(2))

const empty = Range.new(1, 1, false)
System.print(empty.iterate(None))

System.print(r.iteratorValue(1))
System.print(r.iteratorValue(3))
System.print(r.iteratorValue(-2))
System.print(r.iteratorValue(5))

let c = r.iterate(None)
while (c != None) {
  System.print(r.iteratorValue(c))
  c = r.iterate(c)
}
