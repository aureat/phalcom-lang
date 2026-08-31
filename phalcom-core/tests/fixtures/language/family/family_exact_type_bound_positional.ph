// area: family
// spec: docs/spec/callables/family.md §1 and §2
// status: PASS
// A class expression is the bound receiver for a class-side exact Family.

class MathUtil {
  @class
  square(_ n) { return n * n }
}
const f = MathUtil::square::(_)
System.print(f(6))
