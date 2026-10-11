# Quaternion Values

## Type and construction

`universe.numbers.Quaternion<T>` is an immutable four-component value
`a + bi + cj + dk`. All components have the same applied type `T`; the exact
applied type remains part of the value's type identity. Construction is
explicit through `Quaternion(real, i, j, k)` or `Quaternion.new(real, i, j, k)`.
There is no implicit conversion among `Quaternion<Int>`,
`Quaternion<Rational>`, and `Quaternion<Float>`.

`real`, `i`, `j`, and `k` expose the components. `complexPart` returns
`Complex<T>(real, i)`; it does not discard or combine the `j` and `k` components
because it projects only the designated first two coordinates.

## Arithmetic

Operations are available only when `T` supplies the corresponding independent
capability from [`numeric-capabilities.md`](numeric-capabilities.md):

| Component capabilities | Quaternion operation |
| --- | --- |
| `Addable` | `+` |
| `Subtractable` | binary `-` |
| `Negatable` | unary `-`, `conjugate` |
| `Addable`, `Multipliable` | `normSquared` |
| `Addable`, `Subtractable`, `Multipliable` | `*` |
| `Zero` | `Quaternion.zero` and real embedding |
| `Zero`, `One` | `Quaternion.one` and basis embedding |

Multiplication uses Hamilton's product. It is generally noncommutative. The
conjugate negates the three imaginary components and preserves the real
component. `normSquared` is the sum of the four component squares; component
overflow or rounding follows the component type's arithmetic.

No quaternion division, ordering, magnitude, or integer-power operation is
introduced by this specification. Such operations require separate capability
and edge-case contracts. `Quaternion<Float>` in particular does not inherit a
division implementation merely because `Float` is divisible.

## Basis markers and embedding

`I`, `J`, and `K` are zero-field marker types. `Basis<Axis>` identifies one of
those axes; the exported `i`, `j`, and `k` values are their corresponding basis
values. They are type-level selectors and do not define literal suffix syntax.

`RealEmbeddable` exposes a scalar type and `fromReal`; `BasisRealizable<B>`
exposes a scalar type and constructs a pure value along basis `B`. For
Quaternion, the real embedding places the scalar in `real` and fills the other
components with `T.zero`; the `I`, `J`, and `K` realizations place the supplied
coefficient in their matching component and zero the remaining components.
The necessary `Zero`/`One` capabilities are explicit bounds.

Quaternion arithmetic and the `i`/`j`/`k` literal syntax are separate surfaces.
The latter remains governed by its language-design work and is not implied by
these value operations.
