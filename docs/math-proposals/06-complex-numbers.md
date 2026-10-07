# 6. Complex numbers

## 6.1 Recommendation

Add `Complex<T>` as a generic nominal data family in a later numerical-foundation
slice, but ship `Complex<Float>` before promising a broad arbitrary-component
complex ecosystem. The type is not merely a pair: its operations, equality,
branch behavior, and special values need an explicit contract.

The generic parameter should be a component type, not a second independent
numeric tower. Conceptually:

```telos
```

The exact generic `data` constructor spelling is supported by current data tests
and specs; the shown declaration is an API sketch whose final field-label form
must follow the live parser.

## 6.2 Component requirements

Do not require a giant `Number` bound. Define capability requirements per family:

| Operation | Minimum component law |
|---|---|
| construction/access | none beyond a value type |
| `+`, `-` | additive group-like capability |
| `*` | multiplicative/additive distributive capability |
| `/` | field-like division with zero behavior |
| `conjugate` | component operations plus additive/multiplicative identity |
| `normSquared` | multiplication/addition, result may be component or associated type |
| `abs` | ordered/real-valued norm result; not available for arbitrary `T` |
| `arg`, polar conversion | approximate real backend or an explicit angle type |
| complex transcendental functions | a documented elementary backend, not just operators |

This makes `Complex<Int>` meaningful for construction, addition, subtraction, and
multiplication, but division should either return `Complex<Rational>` through a
separate operation or be unavailable on `Complex<Int>`. `Complex<Rational>` is a
strong exact candidate for all field operations. `Complex<Float>` supports the
full approximate elementary API.

## 6.3 Representation and constructors

Required semantic accessors:

```text
real -> T
imag -> T
conjugate -> Complex<T>
```

Provide rectangular construction first. Polar construction belongs to the
approximate-real API:

```text
Complex.fromPolar(radius: Float, angle: Float) -> Complex<Float>
```

Do not make a generic `fromPolar<T>` without a real angle/radius contract.

Formatting should be stable and explicit around zero signs. A reasonable first
contract is `a + bi`, omitting a zero component only if that choice is specified;
do not let formatting accidentally become a parser syntax commitment.

## 6.4 Arithmetic and equality

For components `a,b,c,d`:

```text
(a+bi) + (c+di) = (a+c) + (b+d)i
(a+bi) * (c+di) = (ac-bd) + (ad+bc)i
conjugate(a+bi) = a-bi
normSquared(a+bi) = a*a + b*b
```

Complex numbers have **no ordinary ordering**. `<`, `<=`, `>`, and `>=` should
not be provided by the Complex family. `compare` should be unavailable unless a
caller explicitly chooses a lexicographic or total-order helper with a different
name.

Equality is component-wise under the component's ordinary numeric equality. For
`Complex<Float>`, that implies `NaN` behavior must be stated: component NaN makes
ordinary equality false, while exact sameness may still distinguish the full
component representations. Signed zero is numerically equal in `==` but can be
observed through component classification or `===`.

## 6.5 Division and powers

For `z = a+bi` and `w = c+di`, use a scaled algorithm to reduce overflow and
underflow rather than the textbook formula unconditionally. The semantic result
is:

```text
z / w = ((a+bi) * conjugate(w)) / normSquared(w)
```

but implementation must choose a robust evaluation path. Division by exact zero
should raise for exact components; Float zero/infinity/NaN behavior follows the
complex backend contract once selected.

`pow(z, Int)` can be exact/algorithmic for supported component families using
exponentiation by squaring. `pow(z, w)` is a transcendental operation defined as
`exp(w * log(z))` only after branch-cut semantics are specified.

## 6.6 Complex elementary functions and branch cuts

The following need a dedicated complex backend contract:

```text
sqrt exp log log1p pow
sin cos tan asin acos atan
sinh cosh tanh asinh acosh atanh
```

At minimum specify:

- principal branch for `sqrt`, `log`, and inverse functions;
- branch cut locations and which side is selected;
- continuity conventions at negative real values;
- signed zero on real and imaginary components;
- behavior for infinities and NaNs;
- overflow/underflow scaling;
- whether results are platform-stable or backend-dependent.

Example policy for `Complex<Float>` can follow a documented principal branch, but
the exact decision must be made in a separate specification. `sqrt(-1.0)` should
remain the existing Float result (`NaN`) until the caller explicitly constructs a
complex input:

```telos
let z = Complex(-1.0, 0.0)
let root = complex.sqrt(z)       // proposed: Complex<Float>(0.0, 1.0)
```

## 6.7 Mixed real/complex arithmetic

The preferred ergonomic rule is explicit library families or a narrowly-defined
promotion hook:

```text
Complex<Float> + Float -> Complex<Float>
Float + Complex<Float> -> Complex<Float>
Int + Complex<Float> -> only after explicit Float conversion or a declared built-in rule
```

Do not make arbitrary user-defined `T + Complex<U>` discoverable through a
global conversion lattice. If Telos later adds promotion, it must be a finite,
acyclic relation with one result per operand pair and a failure state for
ambiguous/inexact conversions.
