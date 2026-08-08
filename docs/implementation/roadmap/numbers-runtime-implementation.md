# Numeric Runtime and Compiler Implementation Contract

**Status:** Implementation planning record. Public numeric behavior is specified
in [`docs/spec/library/numbers/`](../../spec/library/numbers/README.md).

This document defines the mandatory private architecture that realizes the public numeric semantics without exposing representation tiers or duplicating semantic algorithms.

## 1. Architectural requirements

The implementation must provide:

1. separate runtime Int and Float values;
2. one canonical Int normalization path;
3. operation-specific semantic functions shared by primitives and intrinsics;
4. heap-independent, canonical large numeric constants;
5. exact mixed comparison and exact floor division;
6. coherent numeric-key equality and seeded hashing;
7. immutable deterministic numeric resource policy;
8. structured Error construction through one raise path;
9. a post-bootstrap kernel freeze;
10. closed-kernel numeric intrinsics that bypass dispatch only for exact built-in representation tuples.

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

Compiler-owned artifacts contain no live VM object references.

```rust
enum Constant {
    Int(i64),
    LargeIntV1 {
        negative: bool,
        magnitude_be: Arc<[u8]>,
    },
    FloatBits(u64),
    // ...
}
```

`LargeIntV1` uses a positive-or-negative sign and a minimal nonempty unsigned big-endian magnitude. The first byte is nonzero. Zero and every signed-64-bit value use `Int(i64)`; encoding such a value as `LargeIntV1` is malformed bytecode.

The compiler deduplicates equal canonical LargeInt constants within each module. Every constant-pool entry is materialized at most once per loaded module instance and memoized. Materialization may be lazy, but structural and policy validation occurs before module initialization.

Loader validation, without proportional BigInt allocation, checks:

- known entry and artifact version;
- valid sign and declared length;
- nonempty minimal magnitude;
- rejection of signed-64-bit values;
- exact bit length against `maxIntegerBits`;
- magnitude byte length against `maxNumericAllocationBytes`.

Malformed or noncanonical constant encoding is rejected as invalid bytecode. A canonical constant that exceeds the active VM policy is rejected as a numeric-policy load failure; the two conditions are not conflated.

`maxSourceNumericDigits` is not reapplied to bytecode. Cross-module weak deduplication is optional and unobservable. Whole-artifact integrity belongs to the bytecode container; LargeInt entries carry no separate checksum.

## 6. Numeric semantic kernel

Do not route all operations through one universal promotion helper. Internal semantic functions are operation-specific, conceptually including:

```rust
numeric_add(lhs, rhs, policy, span)
numeric_subtract(lhs, rhs, policy, span)
numeric_multiply(lhs, rhs, policy, span)
numeric_divide(lhs, rhs, policy, span)
exact_compare(lhs, rhs)
total_compare(lhs, rhs)
exact_floor_divide(lhs, rhs, policy)
floor_remainder(lhs, rhs, policy)
numeric_equal(lhs, rhs)
numeric_key_equal(lhs, rhs)
numeric_hash_input(value)
convert_int_to_float(value)
convert_float_to_int(value, mode)
```

Every public primitive, interpreter intrinsic, compiler specialization, and future native-code path must call these functions or prove bit-for-bit and error-for-error equivalence.

The semantic kernel owns result normalization, resource checks, special-value dispatch, error fields, and span selection. Intrinsic execution is an invocation strategy, not a second semantics implementation.

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
- total order .

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

Non-finite dispatch follows the complete Float remainder table in the
[numeric Float protocol](../../spec/library/numbers/float-protocol.md).

## 12. Power implementation

Use separate paths:

- exact BigInt exponentiation for nonnegative Int exponent;
- binary64 integer-exponent algorithm when the exponent is Int but the result domain is Float;
- general Float exponent algorithm for Float exponent;
- explicit special-case dispatch before the generic library call;
- one-ULP conformance against a high-precision oracle.

An exact Int exponent must not be approximated to Float merely because it exceeds a host integer width. Resource policy may reject excessive computation deterministically.

A host `pow` may be an implementation component only after the runtime enforces the specified special cases and accuracy contract.

## 13. Numeric hashing

### 13.1 Canonical stream

A finite numeric key is reduced to exact rational `m/n`, with arbitrary-precision signed `m`, positive `n`, and `gcd(|m|, n) = 1`.

- Int `i` becomes `i/1`.
- finite Float becomes its exact binary64 dyadic rational;
- both signed zeroes become `0/1`;
- every NaN becomes one domain-separated private token;
- the infinities become distinct domain-separated private tokens.

The complete sign, length, numerator bits, denominator bits, or special token are streamed into the backend. No textual encoding or fixed-width mathematical prehash is normative.

### 13.2 Public built-in hash

Built-in `hash` returns a nonnegative Int in `0..<2**64`. It uses a VM-specific keyed streaming hash. The initial backend should be a portable SipHash-1-3 implementation, but the algorithm is replaceable and non-normative if the semantic requirements remain satisfied.

The VM master seed contains at least 128 unpredictable bits from secure operating-system randomness and is immutable. Independent keys for public numeric hashing and collection finalization are derived from it with distinct domain labels. A deterministic fixed seed is available only as VM/embedder configuration for tests and debugging.

Built-in hash values are stable only within one VM instance and must not be persisted.

### 13.3 Collection finalization

Each Map and Set receives a salt derived from the VM master seed, a collection nonce, and a domain separator. Bucket placement applies a keyed per-collection finalizer.

For built-in keys:

```text
canonical numeric value
→ VM-keyed public 64-bit hash
→ per-collection keyed finalizer
→ bucket hash
```

For user-defined keys, `hash` may return any signed arbitrary-precision Int. The collection finalizer consumes the sign, significant length, and every magnitude bit without truncation.

Collision never implies equality. Collections always check key equivalence after hash match. A user method returning a constant hash remains semantically correct but may cause poor performance; collection finalization cannot repair intentionally identical inputs.

## 14. Map and Set integration

Map and Set must use the same numeric-key equality and hash canonicalization.

Insertion behavior:

- probe by canonical hash and numeric-key equality;
- if equivalent key exists, preserve stored key and insertion position;
- replace only Map value;
- retain existing Set representative.

Required internal tests must insert equivalent keys in both orders and inspect retrieval, deletion, size, iteration key class, signed-zero rendering, and NaN behavior.

## 15. Class and protocol layout

`Number` is abstract. `Int` and `Float` are the only concrete built-in descendants. User classes cannot subclass `Number`.

Kernel bootstrap proceeds in this order:

```text
install class rows and primitive methods
→ load core Phalcom protocol
→ verify numeric and metaclass invariants
→ freeze kernel classes and metaclasses
→ load user modules
```

After the freeze, no source declaration, reflection API, extension mechanism, or embedder mutation may add, replace, or remove methods on `Number`, `Int`, or `Float`, or change their superclass links.

Public methods remain reflectable and explicitly invocable. Kernel closure permits ordinary compiled numeric syntax to bypass lookup through equivalent intrinsics.

## 16. Primitive floor

The frozen primitive census records each public binding individually.

Numeric representation-sensitive operations are direct primitives on `Int` and `Float`. The ten Int bitwise selectors are all primitive defaults:

```text
&(_) |(_) ^(_) ~() <<(_) >>(_)
bitAt(_) bitCount bitLength trailingZeros
```

No multiplexed public or private raw bitwise selector substitutes for this census. Shared Rust helpers do not create a Phalcom selector surface.

Derived representation-independent protocol may remain authored in Phalcom over the primitive floor.

## 17. Integer-only boundaries

Every primitive accepting indexes, arities, counts, offsets, lengths, or loop counters must require Int. Every primitive and compiler-generated path producing such quantities must produce Int, including collection sizes, arities, loop counters, and pattern constants.

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

Each compiler context and VM owns one immutable `NumericPolicy`, chosen before source processing or module loading. Standalone tools use `standard` by default unless the host explicitly selects another policy. Built-in profiles are:

| Limit | `standard` | `sandbox` |
|---|---:|---:|
| `maxSourceNumericDigits` | `100_000` | `4_096` |
| `maxTextConversionDigits` | `100_000` | `4_096` |
| `maxIntegerBits` | `8_388_608` | `262_144` |
| `maxNumericAllocationBytes` | `2_097_152` | `65_536` |

Custom policies use a nonnegative Int or `None` for each field; `None` disables that specific policy check. Zero is a valid active limit. Profiles and custom values are platform-independent and do not scale with host memory or pointer width.

`maxSourceNumericDigits` counts source-token digits. `maxTextConversionDigits` counts digits consumed by runtime parsing or produced by runtime rendering. Both count raw digit characters uniformly across radices. Signs, prefixes, decimal points, exponent markers, and separators do not count. Exponent digits and leading zeroes do count.

For Int `n`:

```text
integerBits(0) = 0
integerBits(n) = floor(log2(abs(n))) + 1, otherwise
logicalBytes(n) = ceil(integerBits(n) / 8)
```

Float has a logical numeric charge of eight bytes. Future composite built-in numeric values use the sum of canonical component payloads. Object headers, allocator metadata, spare capacity, GC metadata, and implementation temporaries do not affect the logical charge.

Cheap domain and semantic errors are checked before policy estimation when the operation does not require an oversized result; for example, zero to a negative power raises `#divideByZero`. A policy failure must be exact or based on an unavoidable lower bound. A loose upper estimate alone cannot cause rejection. Operations may compute incrementally and abort before crossing the limit. Huge right shifts and bit queries short-circuit when the result is known.

Source policy failure is compiler diagnostic `numeric.limit`; runtime failure is `#numericLimit`. Actual allocator failure below permitted policy remains an out-of-memory failure.

Compilation cache identity includes the compile-policy fingerprint. Bytecode records normalized constant requirements and is revalidated against the active VM policy. Source digit limits are not reapplied to valid bytecode.

Source modules cannot raise, replace, disable, or dynamically scope the active policy. If policy introspection is exposed, it is read-only. A stricter or more permissive execution environment uses a distinct compiler context or VM.

Numeric policy supplements general heap and execution budgets; it does not replace them.

## 19. Error construction

All runtime numeric failures construct language Error values and return through one Raise mechanism.

Do not expose parallel host-only variants for divide by zero, invalid shift, or conversion. Internal helper enums may exist, but they must be translated before crossing the user-observable boundary.

Error builders should accept structured fields and source-span context, avoiding string parsing in tests and tooling.

## 20. Closed-kernel numeric intrinsics

After the kernel freeze, primitive numeric method identity cannot change. Compiler-generated ordinary numeric operations may therefore bypass selector lookup, inline-cache lookup, method-object invocation, call-context construction, and version guards.

For statically proven exact operands, the compiler may emit a fully specialized operation such as `IntAdd` with no runtime class guard.

For dynamically typed operands, a polymorphic intrinsic inspects value representations:

```text
(Int, Int)       → direct exact Int semantic kernel
(Int, Float)     → direct mixed numeric kernel
(Float, Int)     → direct mixed numeric kernel
(Float, Float)   → direct Float kernel
other tuple      → ordinary selector send
```

The same rule applies to arithmetic, comparison, conversion, hashing, rendering, and bitwise operations where profitable.

Explicit reflection, `perform`, method-object invocation, and core-authored super sends use the installed public methods. Intrinsics apply only to non-reflective compiler-generated sends—including operator syntax and direct calls of known numeric selectors—and exact built-in representation tuples.

The intrinsic and primitive paths share the semantic kernel and must preserve evaluation order, values, result class, errors, structured fields, spans, signed zero, NaN behavior, and resource-policy behavior.

Future built-in numeric representations are not automatically recognized. They require explicit addition to the intrinsic operand matrix; otherwise execution falls back to dispatch.

## 21. Parsing and rendering implementation

Finite Float text is first validated by the Phalcom scanner, including digit-policy checks before scratch allocation, and then converted by Rust core `f64::from_str`. Special values are handled directly, with textual NaN canonicalized to `0x7ff8000000000000`.

Float rendering uses Ryū for shortest significant digits and a Phalcom-owned notation layer for exponent thresholds and spelling. Int rendering uses exact decimal conversion over the canonical magnitude.

Exact built-in rendering in interpolation, diagnostics, and numeric intrinsics may bypass `toString` lookup after kernel freeze. Unknown user values still receive the ordinary `toString` send where the surface semantics require it.

## 22. Dependency policy

The initial implementation uses:

- `num-bigint` for arbitrary-precision integer values and exact-oracle support;
- Rust core `f64::from_str` for correctly rounded finite decimal parsing;
- the Rust `ryu` crate for shortest binary64 decimal generation.

Dependencies that participate in user-visible value semantics are pinned centrally and audited for license, portability, maintenance, and behavior. Internal algorithm versions are non-normative where the specification pins exact outputs independently.

## 23. Implementation phases

Land in this order:

1. split numeric tokens and runtime values into Int and Float;
2. add canonical Int normalization and LargeInt storage;
3. install the closed class tower and allocation guard;
4. implement exact arithmetic, conversion, comparison, division, and remainder kernels;
5. implement numeric-key equality and seeded hashing;
6. implement text parsing, Ryū rendering, and structured errors;
7. implement resource policies and bytecode constant validation;
8. implement all ten bitwise primitives;
9. freeze kernel classes after core bootstrap;
10. add numeric intrinsics over the already-conforming semantic kernels;
11. update Map/Set, integer-only boundaries, reflection, tooling, and documentation;
12. run the complete conformance and migration gates.

The update is atomic: no distributable release may expose a hybrid old/new numeric model.

## 24. Documentation requirements

Every public runtime item and every semantic helper must explain:

- exact versus Float-domain behavior;
- canonicalization responsibility;
- errors and resource policy;
- why a host primitive is or is not semantically equivalent.

Comments that retain obsolete flat-Number, stable-English-message, or Float-index assumptions are correctness defects.
