# Numeric Runtime and Compiler Implementation Contract

> **Status:** Binding implementation architecture; repository paths and exact type names are non-normative examples.
>
> **Purpose:** Realize the public specifications without leaking representation tiers, duplicating semantic algorithms, or binding compiler artifacts to one VM heap.

## 1. Architectural requirements

The implementation must provide:

1. separate runtime Int and Float values;
2. one canonical Int normalization path;
3. operation-specific numeric semantic functions;
4. heap-independent large numeric constants;
5. exact mixed comparison and exact floor division;
6. coherent numeric-key equality and hashing;
7. deterministic numeric resource guards;
8. structured Error construction through one Raise path;
9. guarded, invalidatable fast paths only after semantics are complete.

## 2. Runtime value representation

A representative Rust shape is:

```rust
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Symbol(Symbol),
    Obj(ObjRef),
}

pub enum Object {
    // ...
    LargeInt(BigInt),
}
```

Exact names are not normative. Required behavior is:

- immediate Int covers the common private range;
- LargeInt holds every other integer;
- both surface as class `Int`;
- Float stores binary64 exactly, including signed zero and NaN payload bits;
- the old flat numeric arm is removed rather than retained as a migration alias.

Removing the old arm is an intentional exhaustiveness tool. Semantic matches must be updated explicitly rather than hidden by wildcard arms.

## 3. Canonical Int normalization

All arithmetic, parsing, constant loading, deserialization, and FFI paths that construct an arbitrary-precision integer must call one function equivalent to:

```rust
fn normalize(value: BigInt) -> Value
```

Invariant:

```text
Value::Obj(LargeInt(x)) implies x is outside the immediate Int range.
```

No other code path may create a LargeInt runtime value.

Debug and test builds must assert this invariant. Deserialization and module loading are included; canonicalization is not limited to arithmetic results.

## 4. Heap and GC

`LargeInt` contains no VM object references and therefore traces no child `ObjRef`s. It still requires an explicit tracer arm.

A runtime large constant is rooted exactly like every other loaded constant. The compiler must not rely on a temporary compilation heap object remaining alive.

Whether `BigInt` is inline or boxed inside the object union is an implementation measurement. Confirm actual object size and arena slot policy; do not encode guessed library layout into the language specification.

## 5. Heap-independent constant pool

Compiler output must contain an abstract constant form, for example:

```rust
enum Constant {
    Int(i64),
    LargeInt {
        negative: bool,
        magnitude: Arc<[u8]>,
    },
    FloatBits(u64),
    // ...
}
```

Requirements:

- no live VM `ObjRef` in serialized/compiler-owned bytecode;
- Float constants preserve exact bits;
- large Int constants preserve exact value;
- module loading materializes large Int through `normalize`;
- loaded constants are rooted by the module/function constant owner;
- encoding is versioned.

Sign representation, endianness, deduplication, and sharing remain open under **OD-NUM-008**.

## 6. Numeric semantic kernel

Do not use one `Promoted` helper for all operations.

The implementation should expose internal functions conceptually equivalent to:

```rust
float_arithmetic(lhs, rhs, op)
exact_compare(lhs, rhs)
exact_floor_divide(lhs, rhs)
floor_remainder(lhs, rhs)
numeric_equal(lhs, rhs)
numeric_key_equal(lhs, rhs)
numeric_hash(value)
convert_int_to_float(value)
convert_float_to_int(value, mode)
```

These are semantic boundaries, not necessarily one module or one public API.

Every primitive and fast path must share these algorithms or prove bit-for-bit/exception-for-exception equivalence against them.

## 7. Small-Int fast path

Common Int/Int arithmetic must not allocate BigInt unconditionally.

For `+`, `-`, `*`, and negation:

1. attempt checked immediate arithmetic;
2. on overflow, promote operands to BigInt;
3. compute exactly;
4. call `normalize`.

Division, remainder, power, shifts, and conversions use operation-specific paths.

## 8. Exact Float decomposition

Implement one binary64 decoder returning an exact classification:

```text
FiniteDyadic {
    sign,
    integer_significand,
    binary_exponent
}
PositiveInfinity
NegativeInfinity
NaN
```

The decoder is reused by:

- exact Int/Float comparison;
- Float-to-Int narrowing;
- exact `~/`;
- Float floor remainder;
- integral-Float hash canonicalization;
- total order once OD-NUM-002 closes.

Do not independently rederive exponent/significand logic in each primitive.

## 9. Exact comparison algorithm

The comparator returns:

```text
Less | Equal | Greater | Unordered
```

Rules:

- Int/Int compares arbitrary-precision integers.
- Float/Float ordinary finite order may use binary64 comparison after classifying NaN and signed zero consistently.
- Int/finite Float compares the Int against the Float's exact dyadic value without converting the Int to Float.
- NaN returns `Unordered`.
- infinities order outside finite values.

Public predicates map `Unordered` to false except `!=`, which follows negated public equality.

## 10. Exact floor division algorithm

The general `~/` path converts finite values to an exact rational pair.

A practical representation is:

```text
Int              => numerator / 1
finite Float     => signed integer significand × 2^e
```

Normalize both operands to integer numerator and a power-of-two denominator, then compute mathematical floor quotient using arbitrary-precision integers.

The implementation must handle all sign pairs explicitly and must not rely on truncating host division semantics.

Optimization is permitted for:

- Int/Int immediate operands;
- powers of two;
- huge quotient short-circuits;
- exact integral Float values.

Every optimization must match the general exact model.

## 11. Floor remainder algorithms

### 11.1 Int remainder

Compute quotient using floor division and derive:

```text
r = a - q * b
```

This is correct for negative divisors. `rem_euclid` alone is not.

### 11.2 Float remainder

After Float-domain conversion of any Int operand:

1. decode both finite Floats exactly;
2. compute exact floor quotient;
3. compute exact dyadic remainder;
4. round once to binary64, ties to even;
5. force a zero result to carry the divisor's sign.

Non-finite dispatch follows the table to be ratified under OD-NUM-003.

## 12. Power implementation

Use separate paths:

- exact BigInt exponentiation for nonnegative Int exponent;
- binary64 integer-exponent algorithm when the exponent is Int but the result domain is Float;
- general Float exponent algorithm for Float exponent;
- explicit special-case dispatch before the generic library call;
- one-ULP conformance against a high-precision oracle.

An exact Int exponent must not be approximated to Float merely because it exceeds a host integer width. Resource policy may reject excessive computation deterministically.

A host `pow` may be an implementation component only after the runtime enforces the ratified special cases and accuracy contract.

## 13. Numeric hashing

The hash subsystem has two layers:

1. **Canonical numeric hash input**, which makes equal numeric keys identical across Int/Float representations.
2. **VM hash mixing**, which may use a per-run seed and internal width.

Required canonicalization:

- Int: exact arbitrary-precision value;
- finite Float: exact dyadic rational;
- integral Float: same canonical numeric value as equal Int;
- signed zero: one canonical zero;
- NaN: one canonical NaN token;
- infinities: distinct positive and negative tokens.

A user-defined `hash` result is an arbitrary Int. Consume every significant bit before or during reduction; accepting only immediate Int is a representation leak.

Algorithm constants and seed policy remain open under OD-NUM-004.

## 14. Map and Set integration

Map and Set must use the same numeric-key equality and hash canonicalization.

Insertion behavior:

- probe by canonical hash and numeric-key equality;
- if equivalent key exists, preserve stored key and insertion position;
- replace only Map value;
- retain existing Set representative.

Required internal tests must insert equivalent keys in both orders and inspect retrieval, deletion, size, iteration key class, signed-zero rendering, and NaN behavior.

## 15. Class and protocol layout

Representation-sensitive methods are installed independently on Int and Float:

```text
+ - * / % ~/ **
< <= > >=
negated
hash
toString
new and new(_), as applicable to class side
```

Number carries no shared VM arithmetic implementation. It may carry ordinary derived protocol where semantics are genuinely common.

The exact placement of classification/narrowing defaults should minimize primitive-floor growth without lying about implementation ownership. Any inherited Number method must be semantically correct for every future Number subtype or be documented as sealed to the kernel tower.

## 16. Primitive floor

Every new VM-blessed binding must be recorded in the frozen primitive census.

Requirements:

- preserve explicit removal and addition counts rather than hiding removals inside a net number;
- keep Number in the census as a zero-primitive tripwire where intended;
- add Int and Float to every core-class census/list;
- amend the governing floor decision in the same change;
- resolve bitwise primitive composition under OD-NUM-006 before U-BITWISE lands.

Live tests, not prose totals, are the source of record.

## 17. Integer-only boundaries

Every primitive accepting indexes, arities, counts, offsets, lengths, or loop counters must require Int.

Centralize extraction helpers:

```rust
expect_int(value)
expect_nonnegative_int(value)
expect_index(value, bound)
```

Do not retain an integral-Float compatibility arm. Conversion is explicit at the call site.

A nonnegative Int larger than host address space is still an Int; boundary helpers distinguish:

- semantic type validity;
- collection bounds;
- host-size representability;
- configured resource policy.

They must not report “expected Int” when the actual problem is out-of-range Int.

## 18. Numeric resource policy

Per-VM policy must support at least:

```text
maxSourceNumericDigits
maxTextConversionDigits
maxIntegerBits
maxNumericAllocationBytes
```

Source policy may be owned by compiler configuration derived from the VM/project profile.

Operations must preflight when a safe upper bound is available:

- left shift;
- exact nonnegative power;
- text-to-Int conversion;
- oversized literal materialization;
- multiplication/addition where result-size bounds trigger configured policy.

`#numericLimit` is not an OOM recovery mechanism. Host allocation failure follows the runtime's general fatal/resource-exhaustion policy unless a separate allocator contract exists.

Defaults and configuration API remain open under OD-NUM-005.

## 19. Error construction

All runtime numeric failures construct language Error values and return through one Raise mechanism.

Do not expose parallel host-only variants for divide by zero, invalid shift, or conversion. Internal helper enums may exist, but they must be translated before crossing the user-observable boundary.

Error builders should accept structured fields and source-span context, avoiding string parsing in tests and tooling.

## 20. Guarded fast paths

Numeric selectors remain dynamically dispatched.

A fast path must guard:

- concrete receiver and argument representation;
- expected class identity;
- selector/method version or pristine epoch;
- integer overflow or exceptional conditions;
- resource-policy preconditions.

On failure, it falls back to ordinary selector dispatch.

A generic method-version invalidation system is preferred. If unavailable, independent Int and Float epochs are required. The mechanism remains open under OD-NUM-007.

No arithmetic opcode may be permanently specialized against the old flat Float representation.

## 21. Rendering fast paths

Int and Float `toString` overrides invalidate independently.

A Number-only pristine flag is incorrect because installing `Int#toString` or `Float#toString` does not modify Number's method row.

Prefer generic selector-version guards shared with arithmetic and other leaf types.

## 22. Dependency policy

Use a mature arbitrary-precision integer library rather than hand-rolling BigInt arithmetic.

Pin value-semantic dependencies centrally and deliberately. Take conversion-trait dependencies only when actually used.

The library is an implementation component; Phalcom's public arithmetic, floor division, remainder, limits, text, and hash semantics remain defined by these specifications, not by library defaults.

## 23. Implementation phases

Each phase must end at a compiling, testable checkpoint:

1. **Semantic kernel and constants:** exact Float decoder, normalization, heap-independent constant forms.
2. **Representation:** Int/Float runtime arms, LargeInt object, exhaustive match migration.
3. **Tower and reflection:** classes, abstract allocation, class identity, Number protocol placement.
4. **Literals and text:** token/AST split, exact constants, corrected grammar, conversion diagnostics.
5. **Arithmetic and conversions:** exact Int arithmetic, Float-domain conversion, `/`, narrowing.
6. **Division and remainder:** exact `~/`, floor `%`, all sign cases.
7. **Equality, keys, and hashing:** land atomically to avoid `==`/hash incoherence.
8. **Strict Int boundaries:** remove integral-Float indexes and update fixtures.
9. **Float protocol and power:** after OD-NUM-003 and OD-NUM-009 close.
10. **Bitwise:** after OD-NUM-006 and resource limits close.
11. **Fast paths:** only after semantic conformance is green.
12. **Docs/status/migration:** ADR/PDR status and release notes in the same landing sequence.

## 24. Documentation requirements

Every public runtime item and every semantic helper must explain:

- exact versus Float-domain behavior;
- canonicalization responsibility;
- errors and resource policy;
- why a host primitive is or is not semantically equivalent.

Comments that retain obsolete flat-Number, stable-English-message, or Float-index assumptions are correctness defects.
