# Numeric Update Migration and Compatibility

> **Status:** Normative inventory of semantic changes. The release mechanism remains open under **OD-NUM-010**.

## 1. Migration principle

The numeric update is a language-semantic correction, not a compatibility shim over the old flat binary64 Number model.

The implementation must not preserve obsolete behavior indefinitely through hidden coercions, representation aliases, or operator-specific exceptions. Where a transition period is chosen, warnings and compatibility behavior must have a named removal release.

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
floor / ceil / truncated / rounded / exact-integral conversion
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
- method invalidation/version guards.

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

## 6. Release strategies under consideration

The semantic inventory is fixed; the deployment strategy is open.

### Option A — Immediate pre-1.0 break

Ship all changes in one language release with migration notes and no compatibility mode.

Advantages:

- no dual semantics;
- smallest implementation surface;
- avoids users depending on transitional behavior.

Risks:

- abrupt source breakage;
- harder ecosystem migration if adoption is already broad.

### Option B — One-release warnings

Recognize selected old forms temporarily, emit warnings, and remove them in the next release.

Potential candidates:

- integral Float indexes;
- `Int.new(integralFloat)`;
- old half-away rounding requests if statically recognizable.

Float `%` cannot be safely dual-interpreted without explicit mode or operator spelling.

### Option C — Language edition

Bind source modules to a language edition, preserving old semantics for old-edition code.

Advantages:

- controlled migration;
- reproducible old code.

Risks:

- numeric values crossing edition boundaries become extremely complex;
- Map/Set equality and hashing cannot safely vary per module;
- VM and standard library must support two semantic universes.

### Option D — Runtime compatibility mode

A VM-wide flag selects old or new numeric behavior.

This is strongly disfavored because libraries cannot know which semantics their callers use, and cached artifacts become mode-sensitive.

## 7. Constraints on the release decision

Whatever OD-NUM-010 chooses:

1. equality and hashing may not be split across compatibility modes inside one VM;
2. Map/Set key semantics must be VM-global and coherent;
3. serialized bytecode must record any source-edition dependency;
4. warnings must have a removal target;
5. documentation and examples must default to the new semantics;
6. no compatibility behavior may leak LargeInt as a public class;
7. old exact-English error matching receives no compatibility guarantee.

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

- numeric ADR/PDR records and status;
- language specification index;
- primitive-floor amendment and census;
- standard library API docs;
- migration guide and release notes;
- compiler diagnostic catalogue;
- runtime Error catalogue;
- examples and tutorials that use `/`, `%`, rounding, indexes, or constructors.
