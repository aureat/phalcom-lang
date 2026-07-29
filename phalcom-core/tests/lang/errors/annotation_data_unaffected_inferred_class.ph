// area: errors
// spec: U-ANNOT-LAYOUT §3 "Rubric" (mixed declared/inferred fields hazard)
// status: PASS
// Regression guard: an ordinary class with zero declared `FieldDef`s (fields
// wholly inferred by first-assignment scan, the pre-U-ANNOT-LAYOUT path) and
// no `@data`/`@sealed`/`@variant` attribute anywhere is byte-for-byte
// unaffected by this unit's changes.

class Counter {
  @constructor
  new(start) {
    _count = start
  }

  increment() {
    _count = _count + 1
    return self
  }

  count => _count
}

const c = Counter.new(10)
c.increment()
c.increment()
System.print(c.count)
