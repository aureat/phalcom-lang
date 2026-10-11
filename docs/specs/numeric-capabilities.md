# Numeric Capabilities

`universe.numbers` exports independent arithmetic capability traits. A
conformance supplies only the operation named by its trait; no capability
implies another.

| Capability | Required operation |
| --- | --- |
| `Addable` | `+` with a same-type operand and same-type result |
| `Subtractable` | binary `-` with a same-type operand and same-type result |
| `Multipliable` | `*` with a same-type operand and same-type result |
| `Negatable` | unary `-` with same-type result |
| `IntegerPower` | `**(Int)` with same-type result; negative exponent policy belongs to the type |
| `Zero` | class-side `zero` producing the additive identity |
| `One` | class-side `one` producing the multiplicative identity |

The trait names express available operations, not unproved algebraic laws.
Implementations must preserve the conforming type's documented arithmetic,
exactness, rounding, overflow, and error behavior. For example, `Float` addition
is rounded binary64 arithmetic, while `Rational` addition is exact.

`Int` and `Float` provide addition, subtraction, multiplication, negation, zero,
and one. `Rational` provides those operations and integer power. `Complex<T>`
and `Quaternion<T>` acquire operations according to their component
capabilities; see [`complex-numbers.md`](complex-numbers.md) and
[`quaternions.md`](quaternions.md).

The generic capability model does not add an automatic conversion lattice,
common supertype, or mixed-type arithmetic rule. A concrete type may define an
explicit conversion or a separately specified operation.
