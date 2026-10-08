# Complex Numbers

## Scope and ownership

`Complex<T>` is a generic nominal data family owned by `universe.numbers`.
Each value has two immutable components, `real: T` and `imag: T`, and denotes
the rectangular value `real + imag*i`. Construction is explicit:

```phalcom
Complex(real, imag)
Complex.new(real, imag)
```

The family does not introduce imaginary literals, implicit numeric promotion, or
a global conversion lattice. `Complex<T>` has no numerical ordering and does
not provide `compare`, `<`, `<=`, `>`, or `>=` by default.

The component type remains part of the applied result type. `Complex<Int>` and
`Complex<Float>` are distinct exact applied types.

```phalcom
from universe.numbers import Complex
import universe.math.complex as complex

let exact = Complex(3, 4) * Complex(1, -2)  // Complex<Int>(11, -2)
let upper = complex.sqrt(Complex(-1.0, 0.0))  // +0 + 1i
let lower = complex.sqrt(Complex(-1.0, -0.0)) // +0 - 1i
let power = complex.powInt(Complex(1.0, 1.0), -2) // +0 - 0.5i
```

## Capability-dependent operations

Complex operations are available through independent component capabilities. An
operation is selected only when the component type supplies the corresponding
capability; one capability does not imply another.

| Applicability evidence | Complex operation |
| --- | --- |
| `T \|= Addable` | `+` |
| `T \|= Subtractable` | binary `-` |
| `T \|= Addable`, `T \|= Subtractable`, `T \|= Multipliable` | `*` |
| `T \|= Negatable` | unary `-`, `conjugate` |
| `T \|= Addable`, `T \|= Multipliable` | `normSquared` |
| `Complex<Int> \|= IntegerPower` or `Complex<Float> \|= IntegerPower` | `**(Int)` |

For `a + bi` and `c + di`, the algebraic operations are:

```text
(a + bi) + (c + di) = (a + c) + (b + d)i
(a + bi) - (c + di) = (a - c) + (b - d)i
(a + bi) * (c + di) = (ac - bd) + (ad + bc)i
conjugate(a + bi) = a - bi
normSquared(a + bi) = a*a + b*b
```

If no selected capability supports a send, ordinary Phalcom missing-message or
unsupported-operation dispatch applies. The absence of a capability does not
create a dynamic arithmetic fallback.

`universe.numbers` exports `Addable`, `Subtractable`, `Multipliable`, `Negatable`,
and `IntegerPower`. Int and Float supply the four component capabilities.
Third-party component types can implement them independently. The `IntegerPower`
conformances belong to the two concrete Complex families.

`toString` renders both components as `a + bi`, using their own formatting and
retaining visible zero signs. This display form does not define literal syntax.

## Integer components

`Complex<Int>` supports exact addition, subtraction, multiplication,
negation, conjugation, squared norm, and integer powers. Integer components use
the existing arbitrary-precision `Int` semantics and resource policy. Division
is unavailable on `Complex<Int>`; there is no implicit conversion to
`Complex<Float>`.

`Complex<Int>.toFloat` performs an explicit component conversion and returns a
`Complex<Float>`.

Integer powers use exponentiation by squaring. Exponent zero returns
`Complex(1, 0)`, including `0 ** 0`. A negative integer exponent raises
`ArgumentError`; it does not manufacture floating-point components.

## Floating-point components

`Complex<Float>` additionally provides:

```text
z / w
z.abs
z.arg
z.isFinite
z.isNaN
z.isInfinite
Complex.fromPolar(radius: Float, angle: Float)
```

Division uses a scaled component algorithm. Its result follows IEEE component
behavior for zero divisors, poles, infinities, subnormals, and indeterminate
inputs. The implementation avoids the textbook denominator calculation when
that calculation would introduce avoidable overflow or underflow.

`abs` is a scaled magnitude and `arg` is `atan2(imag, real)`. The phase keeps
the signed boundary values `-pi` and `+pi`. Signed zero is preserved where the
operation defines it. Float ordinary equality treats signed zeros as equal and
NaN components as unequal; exact sameness retains the component exactness rules.

Float integer powers use the same squaring algorithm. Negative exponents use
reciprocal complex division and therefore return `Complex<Float>` values.

## Principal complex mathematics

`universe.math.complex` exposes captured Families for:

```text
sqrt exp log log1p powInt pow
sin cos tan asin acos atan
sinh cosh tanh asinh acosh atanh
```

These functions accept explicit `Complex<Float>` values. They use principal
branches and return IEEE component results for poles, infinities, NaNs, and
indeterminate cases. They do not introduce Python-style `ValueError` or
`OverflowError` wrappers.

`powInt(z, n)` is the Float integer-power operation. `pow(z, w)` is separate
and uses the principal formula:

```text
pow(z, w) = exp(w * log(z))
```

The zero-base endpoints are defined before this formula:

```text
pow(0, 0 + 0i)       = 1 + 0i
pow(0, c + di)        = 0 + 0i       when c > 0 and d == 0
pow(0, c + di)        = +inf + 0i   when c < 0 and d == 0
pow(0, c + di)        = NaN + NaNi  when d != 0 or c is NaN
```

## Branch cuts and signed zeros

The principal branch cuts are:

| Function family | Cut |
| --- | --- |
| `sqrt`, `log` | negative real axis |
| `log1p` | real axis at and left of `-1` |
| `asin`, `acos`, `atanh` | real axis outside `[-1, 1]` |
| `atan`, `asinh` | imaginary axis outside `[-i, i]` |
| `acosh` | real axis at and left of `1` |
| `exp`, `sin`, `cos`, `sinh`, `cosh` | none |
| `tan`, `tanh` | meromorphic poles; no logarithmic branch cut |
| `pow` | the base `log` cut |

The sign of a zero component selects the approached side of a cut. In
particular:

```text
sqrt(-1 + +0i) = +0 + 1i
sqrt(-1 + -0i) = +0 - 1i
log(-1 + +0i)  = +0 + pi*i
log(-1 + -0i)  = +0 - pi*i
```

Near-zero `log1p` preserves the input signed zeros. Scaled algorithms are used
for square roots, logarithms, exponentials, hyperbolic functions, and inverse
functions where naive intermediate arithmetic could overflow, underflow, or
lose a representable correction.

Final transcendental binary64 bits are backend-dependent within the accuracy
allowed by the active Float math backend. NaN payload and sign bits are outside
the source contract.

For the 15 elementary functions also present in CPython `cmath`, nonfinite
component classifications and known-component zero signs follow the special-value
tables and finite-sign refinements in the pinned
[`cmathmodule.c`](https://github.com/python/cpython/blob/8183fa5e3f78ca6ab862de7fb8b14f3d929421e0/Modules/cmathmodule.c).
The component results apply before Python's exception wrappers. In particular,
`log(0)` has real `-inf`, the `atanh` endpoints at `+1` and `-1` have real signed
infinity, and `atan` endpoints at `+i` and `-i` have imaginary signed infinity.
The other component retains its prescribed signed zero.
`log1p(z)` uses the same `log(1+z)` branch and nonfinite classifications, with a
stable near-zero calculation and the explicit zero-input rule above.

## Representation and exactness

Complex values use the existing data-object representation. Physical storage,
generic construction, and direct construction may differ while retaining one
canonical exact applied descriptor. Complex exact sameness is determined by the
exact applied `Complex<T>` type and recursively by component exactness. Numeric
equality and exact sameness remain distinct relations.

## Implementation provenance

The source-authored Float kernels adapt scaled algorithms and special-value
cases from CPython 3.13.9 `Modules/cmathmodule.c`, commit
`8183fa5e3f78ca6ab862de7fb8b14f3d929421e0`. The adaptation and license are
recorded in `phalcom-core/core/universe/src/math/complex/README.md` and
`CPYTHON-LICENSE.txt`. Independent finite and high-precision references are
maintained in `phalcom-core/tests/reference/math_complex_reference.rs`.
