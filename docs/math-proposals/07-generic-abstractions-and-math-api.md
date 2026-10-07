# 7. Generic abstractions and mathematical API

## 7.1 Design rule

Generic algorithms should state the smallest capability they require. Current
Telos traits are nominal, explicit, coherent, associated-type-capable, and do not
inherit (`docs/specs/objects/traits.md:418-453, 920-1004`). That is enough for a
useful numerical layer if the traits remain small.

The exact trait names below are recommendations. They are not source syntax that
currently exists as a standard numeric package.

## 7.2 Suggested capability vocabulary

### Identities and operations

```telos
trait Additive {
  type Element
  zero -> Self::Element
  add(_ left: Self::Element, _ right: Self::Element) -> Self::Element
  negate(_ value: Self::Element) -> Self::Element
}

trait Multiplicative {
  type Element
  one -> Self::Element
  multiply(_ left: Self::Element, _ right: Self::Element) -> Self::Element
}
```

This sketch uses existing trait/associated-type vocabulary, but a final standard
trait must choose whether operations are class-side methods, instance methods,
or callable families. A value-oriented alternative is more natural for Telos:

```telos
trait Addable {
  +(_ other: Self) -> Self
}
```

The second form is preferable for ordinary operator algorithms because Telos
already dispatches operators as messages. An identity such as zero then needs a
separate capability:

```telos
trait Zero {
  @class zero -> Self
}
```

Do not bundle these until real algorithms use them.

### Comparison

```telos
trait OrderedValue {
  compare(_ other: Self) -> Ordering
}
```

This must not be required by Complex. Float's `compare` should distinguish
ordinary partial numeric comparison from a future total ordering if NaN makes the
ordinary relation unordered.

### Exact integer capability

```telos
trait EuclideanInteger {
  gcd(_ other: Self) -> Self
  lcm(_ other: Self) -> Self
  ~/(_ other: Self) -> Self
  %(_ other: Self) -> Self
}
```

This is a candidate for `Int` and future exact integer types. It should not be
implemented as a hierarchy of `Integral`, `Signed`, and `Unsigned` unless a
second concrete integer family justifies those distinctions.

### Conjugation and norm

```telos
trait Conjugate {
  conjugate -> Self
}

trait Normed {
  norm -> Float
}
```

`Normed` returning Float is suitable for approximate geometry but too narrow for
exact algebra. A future associated result `type Norm` may be better once actual
consumers exist. Start with concrete Complex APIs rather than prematurely
standardizing this trait.

## 7.3 Generic algorithms that earn abstractions

The following demonstrate value without designing the entire algebra library:

### Horner evaluation

```telos
class PolynomialTools {
  @class
  evaluate<T>(_ coefficients: List<T>, _ x: T) -> T
    where T |= Addable, T |= Multipliable
  {
    let mut result = T.zero
    for coefficient in coefficients {
      result = result * x + coefficient
    }
    result
  }
}
```

This example is **proposed syntax/shape**, not confirmed current standard-library
syntax. It shows the requirement: a generic algorithm needs addition,
multiplication, and zero, not “Number.”

### Generic compensated accumulation

```telos
class Accumulation {
  @class
  pairwise<T>(_ values: List<T>) -> T
    where T |= Addable
  { ... }
}
```

The actual algorithm needs a list/iteration contract and may need a stronger
associativity or error-bound capability. It should not claim mathematical
associativity for Float.

### Exact rational reduction

`Rational.new(a, b)` needs `gcd`, sign normalization, exact multiplication, and
division. This is a concrete consumer of `EuclideanInteger`, justifying that
capability more strongly than a full `Ring` hierarchy.

## 7.4 Mathematical function ownership

| API | Owner |
|---|---|
| `+ - * / % ~/ **`, `abs`, sign, rounding, classification | `Number`/`Int`/`Float`/future concrete types |
| `min`, `max`, `clamp` | generic ordered-value utility package or `OrderedValue` capability |
| Float transcendental functions | `universe.math` Float facade |
| exact gcd/lcm/isqrt | `universe.math` exact integer facade |
| Rational construction/normalization | `universe.numbers.rational` or `universe.numbers` |
| Complex arithmetic | `universe.numbers.complex` |
| Complex elementary functions | a complex subpackage with explicit backend contract |
| `erf`, gamma, beta | future `math.special`, then probability/statistics consumers |

## 7.5 Function and predicate matrix

| Function family | Initial domain/result | Home | Notes |
|---|---|---|---|
| `abs`, unary sign | `Int -> Int`, `Float -> Float`; future Rational/Complex-specific forms | scalar protocols/concrete types | `Complex.abs` returns a real norm, so it is not a universal `Number` method |
| `min`, `max`, `clamp` | ordered values | generic ordered utility | Complex deliberately excluded; Float NaN policy must be named |
| `floor`, `ceil`, `trunc`, `round`, `fract` | Float-specific plus exact-domain helpers | `Float` and exact numeric types | current Float converts to exact Int for integral results; do not duplicate every name in `math` |
| `sqrt`, `cbrt`, `hypot` | current `Float -> Float` | `universe.math` | Complex and exact roots need distinct result contracts |
| `pow` / `**` | built-in operation-specific result | scalar types | exact nonnegative Int powers preserve Int; general Float path is approximate |
| `exp`, `exp2`, `expm1` | `Float -> Float` | `universe.math` | `expm1` is stability-critical |
| `ln`, `ln1p`, `log`, `log2`, `log10` | `Float -> Float` | `universe.math` | `ln1p` is stability-critical; no implicit Int conversion |
| trigonometric/inverse trig | Float radians | `universe.math` | `atan2(y, x)` argument order is part of the contract |
| hyperbolic/inverse hyperbolic | Float | `universe.math` | preserve backend special-value behavior |
| `degrees`, `radians` | `Float -> Float` | `universe.math` | source-owned helpers are sufficient |
| `isNaN`, `isFinite`, `isInfinite`, subnormal/sign predicates | Float | `Float` | representation classification belongs to the scalar |
| `gcd`, `lcm`, `isqrt` | `Int -> Int` | `universe.math` | exact multi-argument integer algorithms; current implementation |
| `erf`, `erfc`, gamma, beta | future Float/complex domains | `math.special` | defer backend/accuracy contract |

This matrix keeps generic convenience functions from becoming a duplicate
numeric semantics layer. A future overload should be added only when its result
type, exactness, and special-value behavior are independently specified.

## 7.6 Constants and metadata

Current math constants are typed `Float` `const` members: `pi`, `tau`, `e`, `ln2`,
`ln10`, `sqrt2`. Keep these exact binary64 values in V1.

Do not invent generic constant syntax such as `pi<T>` now. A generic constant is
only meaningful if a numeric type can provide a compile-time representation and
the static-value domain can carry it. The current static evaluation system can
carry typed static values and value parameters, but no current repository
precedent proves generic per-type constant materialization.

Future numeric packages may expose representation-specific constants:

```text
FloatMath.pi -> Float
RationalMath.pi -> unavailable (pi is not rational)
BigFloatMath.pi(precision: Int) -> BigFloat  // future explicit computation
```

This is clearer than pretending one polymorphic `pi` has one exact value in every
numeric domain.
