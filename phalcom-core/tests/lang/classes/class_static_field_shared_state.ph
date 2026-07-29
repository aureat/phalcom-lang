// area: classes
// spec: classes.md; object-model.md; ADR-0017
// status: PASS

// DEC-D (ADR-0017): `static _count` is class-side STORED state living on the
// class object's own slot vector, shared across all instances. Each
// construction mutates the one shared slot, not a per-instance field.
class Counter {
  static _count = 0
  @constructor
  new() { _count = _count + 1 }
  static count => _count
}
Counter.new()
Counter.new()
Counter.new()
System.print(Counter.count)
