// area: family
// spec: selectors.md §3 (Open form); ADR-0047
// status: PASS
// `Type::name` binds the family's receiver to the class object itself, so
// calling it dispatches a static (class-side) send — `MathUtil::square`
// resolves against `MathUtil class`'s own `square(_)`.

class MathUtil {
  static square(n) { return n * n }
}
const f = MathUtil::square
System.print(f(6))
