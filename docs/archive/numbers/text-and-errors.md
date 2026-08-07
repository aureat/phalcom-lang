# Numeric text and errors

**Status:** Normative. Ratified by [PDR-0027](../../../pdr/0027-float-protocol-and-explicit-narrowing.md).

## 1. Constructors

`Int.new()` returns `0`; `Float.new()` returns `0.0`. `Number.new` always raises `#abstractClass`.
`Int.new(Int)` is identity and `Float.new(Float)` is identity. `Float.new(Int)` widens to binary64.
`Int.new(Float)` always raises `#numericConversion`, including an integral Float. Bool is accepted
by neither constructor.

String constructors have one strict grammar. They do not trim whitespace, accept radix prefixes,
or accept a trailing type suffix.

```ebnf
INT-TEXT    := [ "+" | "-" ] ( "0" | NZ-DIGIT { DIGIT | "_" DIGIT } )
FLOAT-TEXT  := [ "+" | "-" ] DECIMAL | "Infinity" | "-Infinity" | "NaN"
DECIMAL     := INT-TEXT-UNSIGNED [ "." DIGITS ] [ EXPONENT ]
             | "." DIGITS [ EXPONENT ]
             | INT-TEXT-UNSIGNED EXPONENT
```

`INT-TEXT-UNSIGNED`, `DIGITS`, and `EXPONENT` use the literal separator rules in
[numeric literals](numeric-literals.md). `Int.new` accepts only `INT-TEXT`.
`Float.new` accepts `FLOAT-TEXT`; `+Infinity` and signed NaN are rejected. Valid finite decimal
text whose magnitude overflows binary64 becomes signed infinity. A malformed string raises
`#numericText`, identifies `Int` or `Float`, and reports a zero-based byte offset within the
string; its source label covers the argument expression, not an invented substring span.

## 2. Rendering

`Int.toString` is ungrouped base-10 with a leading `-` only for negative values. It has no size
cutoff and never uses exponent notation.

`Float.toString` is locale-independent and canonical:

- `NaN`, `Infinity`, and `-Infinity` spell exactly so.
- `-0.0` preserves its sign; positive zero is `0.0`.
- finite values use the shortest decimal which round-trips to the same binary64 value;
  among equal-length candidates choose lexicographically smallest;
- use fixed notation for scientific exponent `-6 <= e <= 20`, otherwise lowercase scientific
  notation with no `+` exponent sign; an integral fixed result includes `.0`.

Thus output remains visibly Float where fixed notation is used. `toString` is not a serialization
or constant-pool format.

## 3. Numeric error contract

All rows are `Error` values. `kind` is a Symbol and message templates are stable.

| Condition | `kind` | Message template | Primary span |
|---|---|---|---|
| `~/` by zero, or exact `Int % 0` | `#divideByZero` | `cannot <operator> by zero` | operator token |
| `0 **` negative | `#divideByZero` | `zero cannot be raised to a negative power` | `**` token |
| non-finite exact narrowing or `~/` | `#nonFiniteNumber` | `cannot <operation> a non-finite Float` | receiver / operator |
| invalid shift count | `#invalidShift` | `shift count must be a non-negative Int` | shift operator, secondary count if available |
| invalid bit index | `#invalidBitIndex` | `bit index must be a non-negative Int` | call argument |
| malformed numeric text | `#numericText` | `invalid <Int|Float> text at byte <n>` | constructor argument |
| rejected numeric conversion | `#numericConversion` | `cannot construct <target> from <source>` | constructor argument |
| `Number` allocation | `#abstractClass` | `cannot construct abstract class Number` | constructor/class expression |
| non-Int hash result | `#invalidHash` | `hash must return Int, got <type>` | keyed-operation key expression |
| integer resource exhaustion | `#numericLimit` | `numeric operation exceeds configured resource limit` | allocating operation |

Literal syntax failures remain compiler diagnostics with a primary span over the malformed literal
and the stable `numeric.literal` code. Runtime failures use the existing `RuntimeError::Raise`
traceback path. If a frame has source and an instruction span, it must render the innermost source
line and caret label. If it has no source/span, render the structured error and traceback without
fabricating a location. This requirement covers binary operations, unary conversion selectors,
string constructors, shifts, bit operations, and allocation guards.

`/` is Float division and follows IEEE division by zero (`1 / 0` is infinity; `0 / 0` is NaN).
Float `%` follows `fmod`, including NaN for a zero divisor. The exact-Int `%` row above raises so
the floor-division identity remains defined only for nonzero divisors.

## 4. Reflection

`1.class == Int`; `1.0.class == Float`; `Int.isA(Number)` and `Float.isA(Number)` are true in
the ordinary class relationship sense. `Number`, `Int`, and `Float` participate in normal selector
reflection and method lookup. Abstractness changes allocation only: it does not hide `Number` or
make its inherited selectors disappear.
