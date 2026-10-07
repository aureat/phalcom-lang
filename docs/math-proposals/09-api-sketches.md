# 9. API sketches and usage examples

These examples are divided into three categories:

- **Current syntax/current API:** already represented by repository fixtures.
- **Proposed library API:** syntax follows current conventions but the member is
  not implemented.
- **Proposed extension:** requires a language/type-system decision and is labeled.

## 9.1 Current Float math

```telos
import universe.math as math

let x = math.sqrt(2.0)
let y = math.exp(x)
let angle = math.radians(45.0)
let root = math.sin(angle)
```

This follows `core/universe/src/math/package.ph` and
`tests/fixtures/language/math/math_elementary.ph`. The functions are public
captured Families backed by Float-only kernel methods.

## 9.2 Current exact integer math

```telos
import universe.math as math

let q = math.gcd(84, 30)
let l = math.lcm(-21, 6)
let root = math.isqrt(10 ** 100)
```

These are current fixtures. Their results are exact `Int` values subject to the
numeric policy.

## 9.3 Proposed Rational usage

```telos
from universe.numbers import Rational

let q = Rational.new(2, 3)
let r = q + Rational.new(1, 6)
let exact = r == Rational.new(5, 6)
let numerator = r.numerator
let denominator = r.denominator
let approximate = r.toFloat
```

**Proposed library API.** `Rational.new(2, 4)` would normalize to `1/2`; the
constructor must reject denominator zero before publishing a value.

## 9.4 Proposed Complex usage

```telos
from universe.numbers import Complex
import universe.math.complex as complex

let z = Complex.new(3.0, 4.0)
let magnitude = z.norm
let conjugate = z.conjugate
let root = complex.sqrt(Complex.new(-1.0, 0.0))
```

**Proposed library API.** The complex `sqrt` result requires a separately
specified principal-branch contract. `sqrt(-1.0)` in the existing Float math
package remains a Float NaN; explicit complex construction selects a different
domain.

## 9.5 Proposed exact complex arithmetic

```telos
from universe.numbers import Complex, Rational

let oneHalf = Rational.new(1, 2)
let z = Complex.new(oneHalf, Rational.new(3, 4))
let product = z * z
let exactNorm = z.normSquared
```

**Proposed library API.** This should be valid for core arithmetic if
`Complex<T>` only requires the component capabilities actually used. It should
not automatically imply `abs`, `arg`, or transcendental functions.

## 9.6 Explicit conversion and mixed arithmetic

```telos
let i = 10 ** 30
let f = Float.new(i)          // explicit binary64 rounding
let q = Rational.new(i, 3)
let qf = q.toFloat             // explicit approximate conversion
let exactFloor = q.numerator ~/ q.denominator
```

The following should remain rejected or require an explicit conversion until a
future promotion decision is ratified:

```telos
// Proposed negative examples:
// math.sqrt(2)                 // Float API does not accept Int implicitly
// Rational.new(1.0, 2.0)       // no implicit Float -> exact Int conversion
// Complex.new(1, 2.0)           // do not infer a component promotion silently
```

## 9.7 Generic algorithm sketch

```telos
// PROPOSED: exact trait names and static syntax require a later type-system decision.
trait Addable {
  +(_ other: Self) -> Self
}

trait Zero {
  @class zero -> Self
}

class Polynomial {
  @class
  evaluate<T>(_ coefficients: List<T>, _ x: T) -> T
    where T |= Addable, T |= Zero
  {
    let mut result = T.zero
    for coefficient in coefficients {
      result = result * x + coefficient
    }
    result
  }
}
```

This intentionally exposes a missing capability: multiplication is also needed.
That is the point of the example. A generic algorithm should not claim that
`T <: Number` is enough when it actually requires zero, addition, and
multiplication.

## 9.8 Generic scalar consumer for future linear algebra

```telos
// PROPOSED: forward-compatibility sketch, not a linear algebra design.
class ScalarKernel {
  @class dotTerm<T>(_ left: T, _ right: T) -> T
    where T |= Addable, T |= Multipliable
  {
    left * right
  }
}
```

The future linear algebra package will likely need both `Float` and
`Complex<Float>`, and possibly Rational for exact small matrices. This foundation
should make those values satisfy the same capability vocabulary without
introducing arrays or matrix semantics here.

## 9.9 Numerical robustness usage

```telos
import universe.math as math

let stable = math.expm1(0.0000001)
let stableLog = math.ln1p(0.0000001)
let distance = math.hypot(3.0, 4.0)
let quadrant = math.atan2(1.0, -1.0)
```

These APIs matter to later statistics/probability libraries because they avoid
cancellation and overflow in common formulas. Their backend precision contract
must remain honest.
