# Numeric Text, Rendering, and Errors

> **Status:** Normative.
>
> **Decision set:** NUM-006, NUM-007, NUM-016, NUM-018, NUM-022.
>
> **Related:** [`numeric-literals.md`](numeric-literals.md), [`numeric-tower.md`](numeric-tower.md), [`float-protocol.md`](float-protocol.md).

## 1. Constructor behavior

```phalcom
Int.new()       // 0
Float.new()     // 0.0
```

`Number.new` always raises `#abstractClass` through the non-overridable shared allocation guard.

| Argument | `Int.new(_)` | `Float.new(_)` |
|---|---|---|
| Int | identity | finite binary64 conversion or `#numericOverflow` |
| Float | always `#numericConversion` | identity |
| Bool | `#numericConversion` | `#numericConversion` |
| String | strict `INT-TEXT` | strict `FLOAT-TEXT` |
| Other | `#numericConversion` | `#numericConversion` |

`Int.new(Float)` rejects even an integral Float. Exact narrowing uses the explicit selector whose final spelling remains open under **OD-NUM-001**.

## 2. Shared text productions

```ebnf
DIGIT        := "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
NZ-DIGIT     := "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
DEC-DIGITS   := DIGIT { DIGIT | "_" DIGIT } ;
SIGN         := "+" | "-" ;
EXPONENT     := ( "e" | "E" ) [ SIGN ] DEC-DIGITS ;

ZERO-TEXT    := "0" { "0" | "_" "0" } ;
NONZERO-TEXT := NZ-DIGIT { DIGIT | "_" DIGIT } ;
UINT-TEXT    := ZERO-TEXT | NONZERO-TEXT ;
INT-TEXT     := [ SIGN ] UINT-TEXT ;

FINITE-FLOAT-TEXT := [ SIGN ] (
      DEC-DIGITS [ "." DEC-DIGITS ] [ EXPONENT ]
    | "." DEC-DIGITS [ EXPONENT ]
) ;

FLOAT-TEXT   := FINITE-FLOAT-TEXT
              | "Infinity"
              | "-Infinity"
              | "NaN" ;
```

Consequences:

- `Float.new("123")` is valid and produces `123.0`.
- `Int.new` is decimal-only.
- Text constructors do not accept `0x`, `0o`, or `0b` prefixes.
- Neither constructor trims whitespace.
- `+Infinity`, `+NaN`, `-NaN`, `nan`, and `inf` are rejected.
- No trailing type suffix is accepted.

## 3. Text conversion rounding

Finite Float text is converted directly to binary64 using round-to-nearest, ties to even.

- Overflow becomes signed infinity because the user explicitly requested a Float-domain value.
- Underflow may produce a subnormal or signed zero.
- Locale never affects parsing.

This differs intentionally from converting an existing finite Int to Float, where an out-of-range Int raises `#numericOverflow` rather than becoming infinity.

## 4. Numeric-text error offsets

Malformed text raises `#numericText` with a zero-based UTF-8 byte offset.

The offset is the first byte that prevents the entire input from matching the required grammar. If the failure is missing input at end-of-string, the offset equals the string byte length.

| Input | Target | Offset | Reason |
|---|---:|---:|---|
| `""` | Int or Float | `0` | expected first byte |
| `" 1"` | Int or Float | `0` | whitespace is not trimmed |
| `"1 "` | Int or Float | `1` | trailing byte |
| `"1_"` | Int or Float | `1` | separator requires following digit |
| `"1__0"` | Int or Float | `2` | second separator invalid |
| `"1e"` | Float | `2` | missing exponent digits at EOF |
| `"1e+"` | Float | `3` | missing exponent digits at EOF |
| `"+Infinity"` | Float | `0` | special value has no leading plus form |
| `"NaN0"` | Float | `3` | trailing byte |
| `"é"` | Int or Float | `0` | first UTF-8 byte is invalid |
| `"1é"` | Int or Float | `1` | first byte after valid prefix is invalid |

The primary source span covers the constructor argument expression. Implementations must not fabricate a source span inside the runtime String.

## 5. Int rendering

`Int.toString` returns:

- ungrouped base-10 digits;
- one leading `-` for negative values;
- no leading `+`;
- no exponent notation;
- no size cutoff;
- no representation-tier marker.

Examples:

```text
0
-1
9223372036854775808
1000000000000000000000000000000
```

## 6. Float rendering

`Float.toString` is deterministic and locale-independent.

### 6.1 Special values

```text
NaN
Infinity
-Infinity
0.0
-0.0
```

All NaN payloads render as `NaN`.

### 6.2 Finite values

Finite output is the shortest decimal that round-trips to the same binary64 value under the specified parser.

- Fixed notation is used when the scientific exponent `e` satisfies `-6 <= e <= 20`.
- Otherwise lowercase scientific notation is used.
- Scientific notation has no `+` sign in a positive exponent.
- An integral fixed result includes `.0` so the value remains visibly Float.
- The canonical tie-break between equal-length shortest candidates must be a standard, explicitly selected algorithm under **OD-NUM-009**; host default formatting is not normative.

Examples of required shape:

```text
1.0
0.000001
1e-7
100000000000000000000.0
1e21
```

Exact boundary outputs must be pinned by conformance fixtures after OD-NUM-009 selects the algorithm.

## 7. Error model

Every user-visible runtime numeric failure is a language `Error` value delivered through the ordinary raise/traceback path.

The stable contract is:

- error `kind`;
- structured fields;
- primary and secondary span rules;
- operation semantics.

Complete English messages are informative and may improve without a language-version change.

## 8. Stable numeric error taxonomy

| Condition | Kind | Required structured fields | Primary span |
|---|---|---|---|
| Exact `~/` or Int `%` by zero | `#divideByZero` | `operator` | operator token |
| Numeric zero to negative power | `#divideByZero` | `operator: #**` | `**` token |
| NaN/infinity in exact narrowing or `~/` | `#nonFiniteNumber` | `operation`, `valueClass` | receiver or operator |
| Int-to-Float finite-range overflow | `#numericOverflow` | `operation`, `targetType: Float` | conversion argument or operator |
| Rejected constructor/narrowing conversion | `#numericConversion` | `sourceType`, `targetType`, `operation` | constructor argument or receiver |
| Malformed numeric text | `#numericText` | `targetType`, `byteOffset` | constructor argument expression |
| Configured numeric policy exceeded | `#numericLimit` | `operation`, `limit`, `requested` where known | allocating operation |
| Negative or non-Int shift count | `#invalidShift` | `countType` or `count` | shift operator; count secondary span |
| Negative or non-Int bit index | `#invalidBitIndex` | `indexType` or `index` | call argument |
| `trailingZeros` of zero or analogous partial query | `#undefinedNumericOperation` | `operation`, `receiverType` | receiver/call |
| Allocation targeting Number | `#abstractClass` | `class: Number` | constructor/class expression |
| User hash returns non-Int | `#invalidHash` | `actualType` | keyed-operation key expression |

Ordinary type-domain failures, such as a Float index passed to a list, use the language's standard type error with expected type `Int`.

## 9. Informative message forms

Implementations should produce concise messages such as:

```text
cannot floor-divide by zero
cannot convert Int to finite Float: value is out of range
invalid Float text at byte 3
numeric power exceeds configured bit limit
shift count must be a non-negative Int
hash must return Int, got Float
```

These examples are not byte-for-byte compatibility requirements.

## 10. Traceback and source spans

If a failing frame has source and an instruction span, rendering must show the innermost source line and caret label.

If source or span is unavailable, rendering must show the structured Error and traceback without inventing a location.

This applies uniformly to:

- binary arithmetic;
- unary narrowing;
- constructors;
- shifts and bit queries;
- resource-policy guards;
- reflective allocation;
- keyed collection hash consumption.

No dedicated host exception path may bypass language Error construction.

## 11. Compiler diagnostics

Numeric source failures are not runtime Error values.

| Code | Meaning |
|---|---|
| `numeric.literal` | malformed numeric source candidate |
| `numeric.limit` | syntax-valid numeric source exceeds compiler policy |

Compiler diagnostics must carry source spans and structured metadata analogous to runtime errors where practical.

## 12. Reflection

`Number`, `Int`, and `Float` remain visible to normal reflection. `Number.respondsTo(#new)` may be true even though invoking allocation raises `#abstractClass`.

Abstractness is enforced in the allocation mechanism, not by deleting selectors or relying on an overridable method body.
