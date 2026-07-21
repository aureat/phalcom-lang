# Float protocol and explicit narrowing

**Status: Proposed.** Normative when [PDR-0027](../../../decisions/0027-float-protocol-and-explicit-narrowing.md)
is accepted. Depends on the `Int`/`Float` tower and PDR-0025's construction rule.

## 1. Surface

Every concrete `Number` responds to these selectors. `Int` answers are exact; `Float` uses
IEEE-754 classification and arithmetic.

| Selector | `Int` result | finite `Float` result | non-finite `Float` result |
|---|---|---|---|
| `abs` | `Int` magnitude | `Float` magnitude | IEEE result (`abs(NaN) == NaN`) |
| `sign` | `Int` `-1`/`0`/`1` | `Int` `-1`/`0`/`1` | `±inf` gives `±1`; `NaN` raises |
| `floor` | self | greatest integer, `Int` | raises |
| `ceil` | self | least integer, `Int` | raises |
| `truncated` | self | toward-zero integer, `Int` | raises |
| `rounded` | self | nearest integer, ties away from zero, `Int` | raises |
| `isInteger` | `true` | true iff fraction is zero | `false` |
| `isNaN` | `false` | IEEE `is_nan` | `true` only for `NaN` |
| `isFinite` | `true` | IEEE `is_finite` | `false` |
| `isInfinite` | `false` | IEEE `is_infinite` | `true` only for `±inf` |

`-0.0.sign == 0`; `(-0.0).isInteger` is true. `abs` preserves `Float` identity as a class even
when its numeric value is integral.

## 2. Exact conversion

For every finite Float conversion result `r`, the implementation converts `r` directly to
`BigInt` and passes it through the tower's `normalize(BigInt) -> Int` path. It must not pass
through `i64`, so values such as `1.0e300.floor` are legal `LargeInt` results.

The Float computation is IEEE-754: its representable input is authoritative. No selector
reconstructs a decimal source literal or promises a mathematical result unavailable from that
input. `rounded` is specified as:

```text
rounded(x) = floor(x + 0.5), x >= 0
           = ceil(x - 0.5),  x < 0
```

after the finite check. This makes half-ties deterministic and sign-symmetric.

## 3. Errors

`floor`, `ceil`, `truncated`, and `rounded` raise an argument-value error when the receiver is
`NaN` or infinite. Until typed error subclasses land, this means the current native error path;
the eventual surface subclass is `ArgumentError`. `sign` raises by the same route for `NaN`.

## 4. Implementation and conformance

Install ten native bindings on `Float`. Define the `Int` counterparts in `core.ph`, and do not
add a binding to abstract `Number`.

Required goldens cover both signs of zero; fractions around every half-tie; `±2^53`, a large
finite Float producing `LargeInt`; `NaN`; both infinities; result classes; selector symbols;
and PDR-0025's `Int.new(2.0)` rejection beside `2.0.truncated` success. Migrate the existing
pending Wren fixtures to `isNaN`, `isInfinite`, `rounded`, and `truncated` before promoting
them.
