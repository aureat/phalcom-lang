# Numeric Update Migration and Compatibility

This document defines the breaking-change inventory and the single supported transition from the old flat numeric model to the canonical Int/Float model.

## 1. Migration principle

The numeric update is a language-semantic correction, not a compatibility shim over the old flat binary64 Number model.

The implementation must not preserve obsolete behavior through hidden coercions, representation aliases, operator-specific exceptions, or a legacy execution mode.

## 2. Breaking changes

### 2.1 Class identity

Before the update, whole and fractional numeric literals may have shared one flat numeric representation. After the update:

```phalcom
1.class       // Int
1.0.class     // Float
```

Code performing exact class checks must be audited.

### 2.2 Exact integer arithmetic

Int arithmetic no longer rounds through binary64. Programs accidentally relying on large-integer precision loss will change result.

### 2.3 Division

`/` always returns Float, including divisible Int operands:

```phalcom
6 / 2        // 3.0
```

Use `~/` when an Int quotient is required.

### 2.4 Remainder

`%` follows floor-division semantics for both Int and Float. Existing Float code relying on truncating `fmod` sign behavior changes:

```phalcom
-7.0 % 2.0   // 1.0 after this update
```

A future named `fmod` operation is not part of this release.

### 2.5 Rounding

`rounded` uses ties to even rather than ties away from zero:

```phalcom
2.5.rounded       // 2
(-2.5).rounded    // -2
```

### 2.6 Int-to-Float overflow

Converting a finite exact Int outside the finite binary64 range raises `#numericOverflow` rather than producing infinity.

Float source/text overflow may still produce infinity because it begins in the Float domain.

### 2.7 Float-to-Int construction

`Int.new(Float)` always raises. Replace it with the explicitly selected narrowing operation:

```text
floor / ceil / truncated / rounded / toIntExact
```

### 2.8 Integer-only boundaries

Integral Float no longer qualifies as an index, arity, count, shift count, or similar integer-only quantity.

```phalcom
list.at(2.0)     // type error
```

Migrate by preserving Int throughout the calculation or explicitly narrowing before the boundary.

### 2.9 Numeric keys

Map and Set merge equal numeric representations, signed zeroes, and NaNs. The first inserted representative is preserved during replacement.

Programs iterating keys may observe the first representative rather than the most recent equivalent key spelling.

### 2.10 Error contracts

Tooling must inspect structured error kinds and fields rather than exact English messages.

### 2.11 Numeric tokenization

`5.e2` is an ordinary send on Int `5`, while `5e2` is a Float literal.

Identifier-like suffixes directly adjacent to numeric candidates are rejected as one malformed candidate.

### 2.12 Number allocation

Every attempt to allocate `Number` raises `#abstractClass`, including reflective and inherited allocation paths.

### 2.13 User-defined numeric classes

User classes cannot subclass `Number`, `Int`, or `Float`, and no implicit coercion or reverse-operation protocol is provided. Numeric-like user classes should inherit from `Object` or another user class and implement the ordinary selectors they support. Operand order may therefore matter: `custom + 3` dispatches to `custom`, while `3 + custom` follows the built-in Int domain rules.

## 3. Source-audit checklist

Search for:

```text
/ used where an Int result is required
% with negative Float operands
rounded at half-integer boundaries
Int.new receiving Float
Float.new receiving very large Int
collection indexes or sizes represented as Float
class checks against Number
user classes inheriting Number/Int/Float or relying on numeric coercion
Map/Set keys mixing Int, Float, NaN, or signed zero
string comparisons against exact error messages
5.e... source forms
user-defined hash returning Float or assuming i64 width
```

## 4. Library and native-extension audit

Native APIs must update:

- Value pattern matches for separate Int and Float arms;
- arbitrary-precision Int handling;
- canonical normalization;
- numeric equality and hash callbacks;
- Int-only argument extraction;
- structured numeric Error construction;
- constant serialization;
- closed-kernel primitive and intrinsic operand handling.

A wildcard match that silently routes Int or Float to “other” is not an acceptable migration.

## 5. Data and serialization

`toString` is deterministic but not a bit-preserving NaN payload serialization format.

Persistent formats must specify:

- Int arbitrary precision;
- Float exact bits or a canonical decimal contract;
- signed zero policy;
- NaN payload policy;
- versioning independent of VM hash values.

Hash results must never be stored as persistent identities.

## 6. Release strategy

The complete numeric model ships as one atomic pre-1.0 breaking release. With the current package version `0.1.0`, the intended version boundary is `0.2.0`.

There is:

- no legacy numeric runtime mode;
- no source edition preserving old numeric semantics;
- no transitional coercion layer;
- no mixed old/new bytecode;
- no partial release containing a hybrid semantic model.

Migration-focused diagnostics may identify recognizable obsolete patterns, but they do not execute old behavior.

The bytecode and compiled-artifact format version advances. Stale artifacts are rejected with a clear instruction to recompile. The loader does not reinterpret old flat-number constants as new Int or Float values. All compiler caches are invalidated across the boundary.

One VM loads only modules compiled for the current numeric semantic generation:

```text
one VM
= one Number model
= one equality relation
= one hashing relation
= one Map/Set key relation
```

Native extensions and embedders that inspect runtime values, constants, hashes, numeric arguments, or bytecode must rebuild against the new interface.

## 7. Compatibility guarantees

The release guarantees:

- one coherent numeric model;
- source-level migration guidance;
- structured diagnostics for detectable obsolete patterns;
- clear stale-bytecode rejection;
- canonical current documentation.

It does not guarantee:

- previous numerical results;
- previous class identities;
- previous public hash values;
- previous Map/Set representative behavior;
- previous exact English error messages;
- loading old bytecode;
- binary compatibility for native extensions.

Old behavior appears only in this migration document and release notes. All language references and examples use the new model.

## 8. Recommended migration order for users

1. Replace Float indexes/counts with Int-producing calculations.
2. Replace implicit Float-to-Int construction with explicit narrowing.
3. Audit `/` result assumptions.
4. Audit negative `%`, especially Float operands.
5. Audit half-integer rounding.
6. Audit large Int to Float conversion.
7. Audit numeric Map/Set keys and key iteration.
8. Switch error handling to structured kinds/fields.
9. Re-run parser diagnostics for numeric-adjacent identifiers.
10. Rebaseline only the goldens whose semantic reason is understood.

## 9. Documentation landing requirements

The release change must update in one coherent series:

- language specification index;
- primitive-floor census;
- standard library API docs;
- migration guide and release notes;
- compiler diagnostic catalogue;
- runtime Error catalogue;
- examples and tutorials that use `/`, `%`, rounding, indexes, or constructors.
