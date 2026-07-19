// area: inheritance
// spec: ADR-0011 (fixed slot layout); object-model.md §5.1; ADR-0040
// status: PASS
// U-INH §3.5: a subclass field with the SAME name as an inherited field gets
// its OWN fresh slot (fields stack, never alias). The parent initializer writes
// the parent's slot; the subclass writes its own — both survive independently.

class A {
  construct new() { _slot = "A-slot" }
  aValue => _slot
}
class B extends A {
  construct new() {
    super.new()
    _slot = "B-slot"
  }
  bValue => _slot
}
const b = B.new()
System.print(b.aValue)
System.print(b.bValue)
