// area: classes
// spec: classes.md; values-and-absence.md; ADR-0017
// status: PASS

// DEC-D (ADR-0017): an in-layout but unwritten static slot reads `None` via the
// same absence helper as instance fields — the private Nil sentinel never leaks.
// `_last` is in the static layout (assigned in `register`), but never written.
class Registry {
  @class
  register(_ v) { _last = v }
  @class
  last => _last
}
System.print(Registry.last)
