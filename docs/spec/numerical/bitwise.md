# Bitwise Operations on `Int`

This document defines the complete Int bitwise surface, infinite-two's-complement semantics, syntax, errors, huge-count behavior, and implementation floor.

## 1. Mathematical model

Int is exact and unbounded. Bitwise semantics use infinite two's complement:

- nonnegative Int values have infinitely many leading zero bits;
- negative Int values have infinitely many leading one bits.

For Int `x`, `y`, and nonnegative Int `n`:

```text
x << n == x * (2 ** n)
x >> n == x ~/ (2 ** n)
~x     == -x - 1
```

`&`, `|`, and `^` are defined pointwise over infinite sign-extended bit sequences.

## 2. Public selector surface

| Source | Selector | Result | Error conditions |
|---|---|---:|---|
| `x & y` | `&(_)` | Int | `y` is not Int |
| `x | y` | `|(_)` | Int | `y` is not Int |
| `x ^ y` | `^(_)` | Int | `y` is not Int |
| `~x` | `~()` | Int | none |
| `x << n` | `<<(_)` | Int | `n` not Int; negative `n`; `#numericLimit` |
| `x >> n` | `>>(_)` | Int | `n` not Int; negative `n` |
| `x.bitAt(i)` | `bitAt(_)` | Bool | `i` not Int; negative `i` |
| `x.bitCount` | `bitCount` | Int | none |
| `x.bitLength` | `bitLength` | Int | none |
| `x.trailingZeros` | `trailingZeros` | Int | `x == 0` |

Every row has an ordinary public selector identity. Symbolic selectors are reflectable and may be defined by user classes. The installed Int methods are frozen after kernel bootstrap and cannot be overridden, replaced, or reached through user subclasses. Compiler-generated exact-Int operations may execute equivalent intrinsics.

No named aliases such as `bitAnd`, `bitNot`, `shl`, or `shr` are introduced.

## 3. Query semantics

### 3.1 `bitAt(i)`

Returns the sign-extended bit at zero-based index `i`.

```phalcom
5.bitAt(0)          // true
5.bitAt(1)          // false
(-1).bitAt(1000)    // true
```

### 3.2 `bitLength`

Returns the number of bits required to represent `abs(x)` without a sign bit.

```phalcom
0.bitLength       // 0
1.bitLength       // 1
5.bitLength       // 3
(-5).bitLength    // 3
```

### 3.3 `bitCount`

Returns the count of one bits in the finite magnitude representation of `abs(x)`.

```phalcom
0.bitCount       // 0
5.bitCount       // 2
(-5).bitCount    // 2
```

It does not count the infinite leading ones of a negative value.

### 3.4 `trailingZeros`

Returns the exponent of the largest power of two dividing `abs(x)`. It is sign-independent.

```phalcom
8.trailingZeros       // 3
(-8).trailingZeros    // 3
```

Zero has no finite result and raises `#undefinedNumericOperation` with `operation: #trailingZeros`.

## 4. Huge counts and indexes

A nonnegative count or index is semantically valid even when it does not fit the host's pointer-sized integer.

### 4.1 Huge right shift

For a count larger than any significant magnitude bit:

```text
x >> huge == 0   when x >= 0
x >> huge == -1  when x < 0
```

An implementation must short-circuit these cases and must not reject the count merely because it cannot be represented as `usize`.

### 4.2 Huge `bitAt`

For an index larger than any magnitude bit:

```text
x.bitAt(huge) == false  when x >= 0
x.bitAt(huge) == true   when x < 0
```

### 4.3 Huge left shift

Left shift may require allocation proportional to the result bit length. It must preflight configured resource policy.

A policy violation raises `#numericLimit`; raw host allocation failure is not promised as a catchable language Error.

## 5. Syntax, tokens, and precedence

Where prefixes overlap, tokens use maximal munch:

```text
~/   before ~ and /
<<   before <
>>   before >
**   before *
```

`~` is prefix-only. Other bitwise operators are infix-only.

Complete relevant precedence, tight to loose:

```text
postfix sends/calls/indexing
**                              right-associative
prefix + - ~                    unary; see asymmetry below
* / % ~/
+ -
<< >>
&
^
|
comparison and equality
and
or
```

Power has unary asymmetry:

```phalcom
~2 ** 3      // ~(2 ** 3)
2 ** ~3      // 2 ** (~3)
-~2 ** 3     // -(~(2 ** 3))
```

All binary bitwise and shift levels are left-associative.

The precedence deliberately parses:

```phalcom
flags & mask == 0
```

as:

```phalcom
(flags & mask) == 0
```

No bitwise compound-assignment tokens are introduced.

## 6. Error contract

- Negative or non-Int shift count: `#invalidShift`.
- Negative or non-Int bit index: `#invalidBitIndex`.
- Left-shift resource-policy violation: `#numericLimit`.
- `trailingZeros` on zero: `#undefinedNumericOperation`.
- Non-Int ordinary operand: standard type error expecting Int.

Errors use the ordinary language Raise path and the source-span rules in [`text-and-errors.md`](text-and-errors.md).

## 7. Laws

For all Int `x`, `y` and nonnegative Int `n`, `m`, whenever every allocating operation completes within configured resource policy:

1. `(x & y) | (x ^ y) == x | y`.
2. `~(x & y) == (~x) | (~y)`.
3. `(x & y) + (x | y) == x + y`.
4. `x ^ y == (x | y) - (x & y)`.
5. `(x << n) << m == x << (n + m)`.
6. `(x << n) >> n == x`.
7. `~~x == x`.
8. `x >> n == x ~/ (2 ** n)`.
9. `x.bitAt(n)` equals the `n`th infinite-two's-complement bit.
10. Results normalize to the immediate Int representation whenever they fit its private range.

## 8. Implementation floor

All ten public selectors have direct VM primitive default implementations on `Int`:

```text
&(_) |(_) ^(_) ~() <<(_) >>(_)
bitAt(_) bitCount bitLength trailingZeros
```

Primitive ownership does not change the public selector model. Reflection and explicit method invocation still observe the installed methods. After the kernel freeze, ordinary exact-Int operations may execute equivalent intrinsics without lookup.

The primitives must not implement one public operation by sending another public arithmetic or bitwise selector. They may share non-reflective internal routines for immediate and large Int handling, count classification, resource preflight, limb access, and result normalization.

Operational requirements:

- `bitAt` indexes the relevant limb directly and allocates no shifted receiver or proportional mask.
- `bitCount` scans magnitude limbs using population count where available.
- `bitLength` inspects canonical magnitude metadata and the top limb.
- `trailingZeros` scans only through limbs before the first set bit.
- shifts operate directly and never materialize a power-of-two multiplier or divisor.
- arbitrary-precision counts are compared mathematically before conversion to a host index.

## 9. Required conformance

Tests must cover:

- every operator and selector;
- positive, zero, and negative operands;
- every sign pair;
- immediate/large promotion and demotion seams;
- counts around `0`, word boundaries, representation boundaries, and values larger than `usize`;
- parser precedence and maximal munch;
- selector symbols and symbolic operator method definitions on user classes;
- rejection of attempts to modify the frozen Int methods;
- explicit reflective invocation of the installed Int methods;
- error kinds and spans;
- resource-policy preflight;
- a Python differential oracle for shared infinite-two's-complement semantics;
- independent algebraic property tests.

## 10. Non-goals

Float bitwise operations, unsigned right shift, fixed-width wrapping, rotations, leading-zero counts without a width, bitwise Bytes bulk operations, and compound assignment are outside this specification.

A future width-bearing numeric family may define `Int32`, `UInt64`, wrapping arithmetic, logical right shift, rotations, and width-dependent zero counts without changing unbounded Int.
