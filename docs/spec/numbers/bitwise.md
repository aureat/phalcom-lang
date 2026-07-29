# Bitwise operations on `Int` — surface specification

**Status: Normative.** Ratified by [PDR-0020](../../../pdr/0020-bitwise-operations-on-int.md) on 2026-07-21. Implementation unit **U-BITWISE** is gated on the numeric tower landing.

## 1. Model

`Int` is exact and unbounded. Bitwise semantics are infinite two's complement: non-negative integers have infinite leading zeroes; negative integers have infinite leading ones. This is the model used by Python, Ruby, Haskell, and Smalltalk arbitrary-precision integers.

For `Int` `x`, `y`, and `n >= 0`:

- `x << n == x * 2.pow(n)`.
- `x >> n == x ~/ 2.pow(n)`; right shift is arithmetic because `~/` floors.
- `~x == -x - 1`.
- `&`, `|`, and `^` are the unique functions defined by infinite-two's-complement bit pairs.

Floored `%` is load-bearing: `x % 2` is `0` or `1` even for negative `x`, making the digit definition correct.

## 2. Selector surface

| Source / selector | Result | Errors |
|---|---|---|
| `x & y` / `&(_)` | `Int` | non-`Int` `y` |
| `x | y` / `|(_)` | `Int` | non-`Int` `y` |
| `x ^ y` / `^(_)` | `Int` | non-`Int` `y` |
| `~x` / `~()` | `Int` | — |
| `x << n` / `<<(_)` | `Int` | negative or non-`Int` `n`; allocation failure |
| `x >> n` / `>>(_)` | `Int` | negative or non-`Int` `n` |
| `x.bitAt(i)` | `Bool` | negative or non-`Int` `i` |
| `x.bitCount` | `Int` | — |
| `x.bitLength` | `Int` | — |
| `x.trailingZeros` | `Int` | `x == 0` |

Every row is an ordinary dynamically dispatched selector. `#&`, `#|`, `#^`, `#<<`, and `#>>` are unary-arity selector symbols; `#~` is the nullary selector symbol. No named aliases (`bitAnd`, `bitNot`, `shl`, and so on) exist.

`bitAt(_)` is sign-aware: `(-1).bitAt(1000) == true`. `bitCount` and `bitLength` are magnitude queries, so `(-5).bitCount == 2` and `0.bitLength == 0`. `trailingZeros` is magnitude-independent and raises on zero because no width exists to return.

## 3. Syntax and precedence

Tokens are maximal-munched in this order where prefixes overlap: `~/`, `<<`, `>>`, then their one-character prefixes. `~` is prefix only; all other bitwise tokens are infix only.

Binary precedence, tight to loose:

```text
* / % ~/   →   + -   →   << >>   →   &   →   ^   →   |
→ comparison/equality → and → or
```

All binary rows are left-associative. This deliberately makes `flags & mask == 0` a mask test, not `flags & (mask == 0)`. No bitwise compound-assignment tokens exist.

## 4. Laws and conformance

For all `Int` x, y and non-negative n, m:

1. `(x & y) | (x ^ y) == x | y`.
2. `~(x & y) == (~x) | (~y)`.
3. `(x & y) + (x | y) == x + y`.
4. `x ^ y == (x | y) - (x & y)`.
5. `(x << n) << m == x << (n + m)` and `(x << n) >> n == x`.
6. `~~x == x`.
7. Results demote to the immediate `Int` representation whenever they fit `i64`.

Implementation must compare against a Python-generated oracle across `0`, `±1`, `±2^62`, `±2^63`, and `±2^100`; test every sign pair, all laws, parser precedence, selector literals, symbolic method definitions and super-sends, errors, and promotion/demotion seams.

## 5. Non-goals

`Float` bitwise, `ushr`, `leadingZeros`, wrapping or fixed-width operations, bitwise `Bytes` bulk operations, and compound assignments are outside this specification. They require a future width-bearing type or a separate measured demand record.
