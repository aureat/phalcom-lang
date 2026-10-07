```markdown
# Telos Standard Library `math` and `random` — Implementation Guidance

## 0. Purpose and architectural ruling

Implement the first viable `universe.math` and `universe.random` standard-library surfaces on top of Telos's existing numeric tower, `Bytes`, iterator/collection infrastructure, native binding system, and source-authored Universe model.

The implementation must preserve the governing Telos architecture:

1. **The Universe source remains the semantic authority.**
2. **Native Rust code implements source-declared semantics; it does not create a parallel API ontology.**
3. **Prefer source-authored Telos algorithms whenever the language can express them cleanly and efficiently.**
4. **Use native implementation only at genuine machine/runtime boundaries or where high-quality floating-point primitives require it.**
5. **Do not introduce VM opcodes, value kinds, heap variants, or compiler special cases for ordinary library functionality unless a hard architectural blocker is demonstrated.**
6. **Do not manufacture closures merely to re-export callable functionality. Telos method-family capture is the correct mechanism.**
7. **Do not conflate deterministic pseudorandom generation with security-sensitive system randomness.**
8. **Do not make exact-to-inexact numeric conversion implicit merely for ergonomic mathematical APIs.**

The existing canonical package shells are:

```text
phalcom-core/core/universe/src/math/
phalcom-core/core/universe/src/random/
```

and the Universe root already exposes/imports/exports them. Extend those canonical modules rather than introducing alternate namespaces.

---

# Part I — `universe.math`

## 1. Public design

`math` is a **module/package of mathematical operations and constants**, not a `Math` class.

Canonical user experience:

```telos
import universe.math as math

const radius: Float = 5.0
const circumference = math.tau * radius

const diagonal = math.hypot(3.0, 4.0)
const divisor = math.gcd(84, 30)
const root = math.isqrt(10_000)
```

The initial API should have two implementation categories:

```text
Float mathematical kernels
    ↓
native implementation required or justified
    ↓
captured into the public math namespace as method families

Exact integer algorithms
    ↓
ordinary Telos source method families
    ↓
exported directly
```

Do not create redundant wrapper methods around native family members when a family capture expresses the relationship exactly.

---

# 2. Method-family export architecture

## 2.1 Use family captures, not closure wrappers

If the native mathematical implementation is declared on an internal kernel:

```telos
intrinsic class FloatMathKernel {
    @class native sqrt(_ value: Float) -> Float
    @class native exp(_ value: Float) -> Float
    @class native ln(_ value: Float) -> Float
}
```

then the public package should expose those operations through callable-family capture:

```telos
let sqrt = &FloatMathKernel.sqrt(_)
let exp = &FloatMathKernel.exp(_)
let ln = &FloatMathKernel.ln(_)

export (
    sqrt,
    exp,
    ln,
)
```

Do **not** write:

```telos
let sqrt = |value: Float| FloatMathKernel.sqrt(value)
```

and do not create a forwarding declaration solely for namespacing:

```telos
sqrt(_ value: Float) -> Float {
    FloatMathKernel.sqrt(value)
}
```

unless there is actual public-layer semantic logic that justifies a distinct declaration.

Family capture preserves callable/method-family semantics and avoids:

- artificial closure allocation or closure identity;
- duplicated signatures;
- wrapper implementation layers;
- degraded reflection/tooling information;
- unnecessary call indirection;
- accidental divergence between native and public declarations.

## 2.2 Selected member versus whole-family capture

Use a **selected capture** for the v1 public API where a specific signature is intentional:

```telos
let sqrt = &FloatMathKernel.sqrt(_)
let atan2 = &FloatMathKernel.atan2(_, _)
```

Do not automatically expose:

```telos
let sqrt = &FloatMathKernel.sqrt...
```

unless the complete overload family itself is deliberately part of the public contract.

This distinction matters.

```telos
&T.f(_)
```

should resolve a public capture to the intended callable/member shape.

```telos
&T.f...
```

means that the family itself is intentionally exposed.

For `math`, prefer the former because implementation-side overload growth must not accidentally expand public API.

A future internal addition such as:

```telos
FloatMathKernel.sqrt(_ value: SomeInternalNumericForm)
```

must not silently become callable through `math.sqrt` if `math` explicitly captured:

```telos
let sqrt = &FloatMathKernel.sqrt(_)
```

according to the public `Float -> Float` contract.

The implementation must use the compiler's canonical family/member identity machinery. Do not lower captured families into anonymous forwarding closures.

---

# 3. Numeric philosophy

The current numeric architecture already distinguishes:

```text
Int
    exact, arbitrary precision

Float
    IEEE-754 binary64

Number
    common numerical superclass/protocol surface
```

`math` must respect that distinction.

For transcendental and binary floating-point operations, accept `Float`, not generic `Number`:

```telos
math.sqrt(4.0)              // valid
math.sqrt(Float.new(4))     // explicit conversion

math.sqrt(4)                // should not silently convert Int to Float
```

Do not introduce:

```telos
sqrt(_ value: Number) -> Float
```

because this silently turns exact values into inexact binary64 values and weakens one of the most important semantic boundaries in the numeric model.

Do not add `sqrt(Int) -> Float` merely for convenience.

If Telos later gains `Decimal`, `Rational`, `Complex`, arbitrary-precision floating point, or other numerical domains, those may acquire deliberate mathematical overloads. The v1 design must not force all numerical domains through binary64.

---

# 4. Proposed `math` v1 surface

## 4.1 Constants

Provide at least:

```telos
const pi: Float
const tau: Float
const e: Float
const ln2: Float
const ln10: Float
const sqrt2: Float
```

These constants must have frozen binary64 values.

Their tests must assert exact `Float` bit patterns, not decimal rendering.

Do not dynamically compute fundamental constants at package initialization.

---

## 4.2 Elementary floating-point functions

Initial recommended set:

```telos
sqrt(_ value: Float) -> Float
cbrt(_ value: Float) -> Float

exp(_ value: Float) -> Float
exp2(_ value: Float) -> Float
expm1(_ value: Float) -> Float

ln(_ value: Float) -> Float
ln1p(_ value: Float) -> Float
log2(_ value: Float) -> Float
log10(_ value: Float) -> Float

hypot(_ x: Float, _ y: Float) -> Float
```

Public package:

```telos
let sqrt = &FloatMathKernel.sqrt(_)
let cbrt = &FloatMathKernel.cbrt(_)

let exp = &FloatMathKernel.exp(_)
let exp2 = &FloatMathKernel.exp2(_)
let expm1 = &FloatMathKernel.expm1(_)

let ln = &FloatMathKernel.ln(_)
let ln1p = &FloatMathKernel.ln1p(_)
let log2 = &FloatMathKernel.log2(_)
let log10 = &FloatMathKernel.log10(_)

let hypot = &FloatMathKernel.hypot(_, _)
```

---

## 4.3 Trigonometric functions

Provide:

```telos
sin(_ value: Float) -> Float
cos(_ value: Float) -> Float
tan(_ value: Float) -> Float

asin(_ value: Float) -> Float
acos(_ value: Float) -> Float
atan(_ value: Float) -> Float

atan2(_ y: Float, _ x: Float) -> Float
```

Arguments/results are in radians.

Do not add separate degree-oriented APIs in v1.

If degree conversion conveniences are later desirable, they should be ordinary source functionality rather than alternative native primitives.

---

## 4.4 Hyperbolic functions

Provide:

```telos
sinh(_ value: Float) -> Float
cosh(_ value: Float) -> Float
tanh(_ value: Float) -> Float

asinh(_ value: Float) -> Float
acosh(_ value: Float) -> Float
atanh(_ value: Float) -> Float
```

These should follow the same IEEE-oriented behavior as the rest of floating-point math.

---

# 5. IEEE behavior

Floating-point mathematical functions should normally model binary64 mathematics, not exception-oriented real-analysis APIs.

Do **not** add a `MathDomainError` merely because the real-number function is mathematically undefined for a given input.

Representative semantics:

```text
sqrt(-1.0)  → NaN
sqrt(+∞)    → +∞
sqrt(-0.0)  → -0.0

ln(+0.0)    → -∞
ln(-1.0)    → NaN
ln(+∞)      → +∞

sin(+∞)     → NaN
cos(+∞)     → NaN

acos(2.0)   → NaN

atan2(...)  → IEEE-compatible quadrant/sign semantics
```

Signed zero is semantically significant and must be tested.

NaN behavior must remain compatible with the accepted Float semantics already used by Telos.

Do not normalize all NaNs, zeros, or infinities through ad hoc Telos source logic unless the established runtime representation explicitly requires such normalization.

---

# 6. Floating-point backend

The implementer must investigate the current Rust/runtime dependency graph before selecting the implementation backend.

Required properties:

- binary64 semantics;
- high-quality correctly or near-correctly rounded implementations;
- explicit behavior for NaN, infinity, signed zero, and subnormal cases;
- no unnecessary heap allocation;
- no VM state mutation;
- no new numerical representation.

Do not blindly call arbitrary platform libc implementations if doing so would introduce uncontrolled OS-dependent results.

The intended contract for v1 should be:

> Mathematical functions produce high-quality IEEE binary64 results and deterministic behavior under the supported Telos runtime/toolchain contract. Tests establish explicit special-case behavior and a stringent finite-value accuracy bound.

Do **not** promise permanent bit-for-bit identity across all future runtime versions unless the chosen backend actually provides and commits to such an invariant.

A useful target for ordinary finite inputs is approximately ≤ 1 ULP where the chosen implementation supports that guarantee. Where a function/library has a stronger documented contract, preserve it.

---

# 7. Exact integer algorithms

Implement the following in Telos source unless profiling or missing primitive operations demonstrate a real blocker:

```telos
gcd(_ a: Int, _ b: Int) -> Int
lcm(_ a: Int, _ b: Int) -> Int
isqrt(_ value: Int) -> Int
```

These are exact integer algorithms and should dogfood the language/runtime rather than immediately delegating to Rust.

## 7.1 `gcd`

Required laws:

```text
gcd(a, b) >= 0
gcd(a, b) == gcd(b, a)
gcd(a, 0) == abs(a)
gcd(0, b) == abs(b)
gcd(0, 0) == 0
gcd(-a, b) == gcd(a, b)
```

Use Euclid's algorithm.

Be careful about which existing integer remainder semantics Telos provides. Ensure Euclidean/GCD correctness independent of operand sign by canonicalizing signs before the loop.

Conceptually:

```telos
gcd(_ a: Int, _ b: Int) -> Int {
    let mut x = absInt(a)
    let mut y = absInt(b)

    while y != 0 {
        const next = x % y
        x = y
        y = next
    }

    x
}
```

If `Int.abs` is not currently available, do not invent an undeclared receiver method silently. Either use a trivial local/source helper or identify `Int.abs` as a separate foundational numeric-library improvement.

---

## 7.2 `lcm`

Laws:

```text
lcm(a, b) >= 0
lcm(a, 0) == 0
lcm(0, b) == 0
```

For nonzero values:

```text
lcm(a, b) = abs((a ~/ gcd(a, b)) * b)
```

Divide before multiplying to reduce intermediate integer growth.

Do not implement naïvely as:

```text
abs(a * b) ~/ gcd(a, b)
```

when the reduced-product formulation is available.

---

## 7.3 `isqrt`

Semantics:

```telos
isqrt(_ n: Int) -> Int
```

for `n >= 0`, where the result `r` satisfies exactly:

```text
r² <= n < (r + 1)²
```

Negative input is invalid and should raise the existing appropriate argument-validation error, preferably `ArgumentError`, rather than inventing a broad math error hierarchy.

The implementation must:

- work for arbitrary precision `Int`;
- avoid conversion to `Float`;
- respect existing numeric resource limits;
- avoid pathological linear iteration.

Use a logarithmic/convergent integer algorithm such as integer Newton iteration or a bit-based algorithm.

Do not implement:

```text
Float.sqrt(n).floor
```

because that is incorrect for large integers and violates exactness.

---

# 8. What `math` should NOT duplicate

Existing scalar methods remain canonical for operations already owned by the numeric types.

Do not add redundant module-level versions of:

```text
Float.abs
Float.floor
Float.ceil
Float.truncated
Float.rounded
Float.isNaN
Float.isFinite
Float.isInfinite

Number arithmetic operators
Number exponentiation
```

Do not add:

```telos
math.abs(...)
math.floor(...)
math.ceil(...)
math.pow(...)
```

merely because other languages place everything under a `math` namespace.

Telos should preserve good ownership boundaries instead of imitating Python/C APIs mechanically.

Likewise, avoid adding generic:

```text
min
max
clamp
```

to `math` in v1. Those belong more naturally to ordering/generic utility abstractions.

---

# 9. Proposed `math` source layout

A likely layout:

```text
phalcom-core/core/universe/src/math/
├── package.ph
├── kernel.ph
└── integer.ph
```

Possible responsibilities:

```text
package.ph
    public constants
    imports internal declarations
    family captures
    exports

kernel.ph
    internal/native FloatMathKernel declaration

integer.ph
    source-authored gcd/lcm/isqrt families
```

Native implementation:

```text
phalcom-core/src/primitive/math.rs
```

The exact Rust file/module location should follow existing primitive-registration conventions discovered in the repository.

Do not create a special architecture just because the package is named `math`.

---

# 10. Illustrative `math/package.ph`

Treat syntax details as subject to the exact current package import/export grammar, but preserve this architecture:

```telos
@!documentation("Mathematical constants and numerical functions.")

import .kernel.FloatMathKernel
import .integer.gcd
import .integer.lcm
import .integer.isqrt

const pi: Float = 3.141592653589793
const tau: Float = 6.283185307179586
const e: Float = 2.718281828459045
const ln2: Float = 0.6931471805599453
const ln10: Float = 2.302585092994046
const sqrt2: Float = 1.4142135623730951

let sqrt = &FloatMathKernel.sqrt(_)
let cbrt = &FloatMathKernel.cbrt(_)

let exp = &FloatMathKernel.exp(_)
let exp2 = &FloatMathKernel.exp2(_)
let expm1 = &FloatMathKernel.expm1(_)

let ln = &FloatMathKernel.ln(_)
let ln1p = &FloatMathKernel.ln1p(_)
let log2 = &FloatMathKernel.log2(_)
let log10 = &FloatMathKernel.log10(_)

let sin = &FloatMathKernel.sin(_)
let cos = &FloatMathKernel.cos(_)
let tan = &FloatMathKernel.tan(_)

let asin = &FloatMathKernel.asin(_)
let acos = &FloatMathKernel.acos(_)
let atan = &FloatMathKernel.atan(_)
let atan2 = &FloatMathKernel.atan2(_, _)

let sinh = &FloatMathKernel.sinh(_)
let cosh = &FloatMathKernel.cosh(_)
let tanh = &FloatMathKernel.tanh(_)

let asinh = &FloatMathKernel.asinh(_)
let acosh = &FloatMathKernel.acosh(_)
let atanh = &FloatMathKernel.atanh(_)

let hypot = &FloatMathKernel.hypot(_, _)

export (
    pi,
    tau,
    e,
    ln2,
    ln10,
    sqrt2,

    sqrt,
    cbrt,

    exp,
    exp2,
    expm1,

    ln,
    ln1p,
    log2,
    log10,

    sin,
    cos,
    tan,

    asin,
    acos,
    atan,
    atan2,

    sinh,
    cosh,
    tanh,

    asinh,
    acosh,
    atanh,

    hypot,

    gcd,
    lcm,
    isqrt,
)
```

If the current import/export system supports cleaner grouped source declarations, adapt the syntax while retaining the semantic structure.

---

# Part II — `universe.random`

## 11. Randomness architecture

`random` must deliberately separate three concepts:

```text
1. deterministic generator algorithm
2. generic deterministic sampling operations/distributions
3. security-sensitive system randomness
```

These are not one abstraction.

The core public architecture should be approximately:

```text
Generator
    ↑
Xoshiro256PlusPlus

Distribution
    ↑
UniformInt
    ↑
Bernoulli

SecureRandom
    separate
    MUST NOT satisfy Generator merely because both can produce bytes/bits
```

The distinction between deterministic pseudorandomness and security authority is semantically important.

---

# 12. No global/default generator

Do not implement any of:

```telos
random.random()
random.nextInt(...)
random.seed(...)
random.reseed(...)
random.default
random.global
```

with hidden mutable package state.

Also do not silently create a fiber-local or thread-local default generator.

Hidden ambient RNG state causes:

- action-at-a-distance;
- poor reproducibility;
- order-sensitive tests;
- concurrency-dependent streams;
- unclear ownership;
- invisible mutation.

Require explicit generator state:

```telos
const rng = random.Xoshiro256PlusPlus.seed(42)

const x = rng.nextFloat
const n = rng.nextInt(below: 100)
```

This makes random consumption visible in the program's object graph.

---

# 13. `Generator` trait

Define a public deterministic-generator abstraction.

The minimal primitive requirement should be one 64-bit random word:

```telos
trait Generator {
    nextU64 -> Int

    // source-authored derived operations
    nextBool -> Bool
    nextFloat -> Float

    nextInt(below upper: Int) -> Int
    nextInt(from lower: Int, below upper: Int) -> Int

    fill(_ bytes: Bytes) -> Unit
    bytes(_ count: Int) -> Bytes

    choose<T>(_ values: List<T>) -> Option<T>
    shuffle<T>(_ values: List<T>) -> Unit
}
```

The exact declaration syntax should match current trait/default-method syntax.

`nextU64` returning `Int` is deliberate.

Contract:

```text
0 <= nextU64 < 2^64
```

Telos does not need a `UInt64` type merely to implement random numbers.

Internally the concrete algorithm must enforce 64-bit word wrapping explicitly.

All other generator functionality should be source-authored from `nextU64` where practical.

---

# 14. Initial concrete deterministic generator

Implement:

```telos
Xoshiro256PlusPlus
```

using the published xoshiro256++ algorithm with four 64-bit state words and SplitMix64 seed expansion.

Reasons for this v1 choice:

- small implementation;
- high performance;
- good statistical quality for ordinary non-cryptographic use;
- straightforward shifts/XOR/rotation;
- state can be represented using existing exact `Int`;
- deterministic across platforms when masking is explicit;
- jump-ahead support exists for later parallel-stream APIs;
- clearly distinct from cryptographic generation.

This generator must be documented explicitly as **not cryptographically secure**.

Do not name the type merely:

```text
Random
Rng
DefaultGenerator
```

The algorithm name should be visible because deterministic replay semantics depend on the algorithm.

---

# 15. 64-bit arithmetic in source Telos

Because `Int` is arbitrary precision, the implementation must explicitly emulate 64-bit unsigned wrapping.

Define internally:

```telos
const MASK64 = (1 << 64) - 1
```

and canonicalize all word-producing arithmetic:

```text
word & MASK64
```

Rotation must explicitly preserve 64-bit semantics.

Conceptually:

```telos
rotl64(_ x: Int, _ k: Int) -> Int {
    ((x << k) | (x >> (64 - k))) & MASK64
}
```

Do not rely on arbitrary-precision shifts alone without masking.

Every internal state word must remain in:

```text
0 <= state[i] < 2^64
```

Add invariant tests for this.

---

# 16. Seeding contract

Initial constructor:

```telos
Xoshiro256PlusPlus.seed(_ seed: Int) -> Xoshiro256PlusPlus
```

Use SplitMix64 expansion to initialize the four xoshiro state words.

For v1, require:

```text
0 <= seed < 2^64
```

Reject values outside that range with `ArgumentError`.

Do not silently compute:

```text
seed mod 2^64
```

because that causes distinct user-supplied seeds to alias unexpectedly.

Do not derive the public generator seed from:

- wall-clock time;
- PID;
- VM hash seed;
- object identity;
- memory addresses;
- system entropy implicitly.

Explicit deterministic construction must be genuinely deterministic.

A later API can add a separate system-seeded construction if there is a compelling use case, but it must remain explicit and must not blur `SecureRandom` with deterministic replay.

---

# 17. Frozen deterministic replay contract

`Xoshiro256PlusPlus.seed(s)` should be a stable reproducibility contract.

Freeze all of the following:

```text
seed acceptance range
SplitMix64 seed expansion
state word ordering
xoshiro256++ transition
nextU64 output
nextBool consumption
nextFloat derivation
bounded-integer derivation
byte derivation and byte order
shuffle algorithm
method call consumption behavior
```

Given:

```telos
const a = Xoshiro256PlusPlus.seed(42)
const b = Xoshiro256PlusPlus.seed(42)
```

the same sequence of calls must yield identical results across:

- process runs;
- supported OSes;
- supported CPU architectures;
- optimization levels.

The public algorithm name provides the versioned semantic anchor.

If a materially different algorithm is desired later, add a different named generator rather than silently replacing xoshiro256++ behind an abstract `DefaultRng`.

---

# 18. `nextFloat`

Implement normalized binary64 generation exactly.

Required range:

```text
0.0 <= result < 1.0
```

Use the top 53 random bits:

```text
word = nextU64()
mantissa = word >> 11
result = Float.new(mantissa) / 2^53
```

This is particularly appropriate in Telos because:

```text
0 <= mantissa < 2^53
```

and every such integer converts exactly to binary64.

Do not use:

```text
Float.new(nextU64()) / Float.new(2^64)
```

because not all 64-bit integers are exactly representable by binary64 and this produces less clean semantics.

Tests should assert exact output bit patterns for golden generator states.

---

# 19. `nextBool`

Use a frozen simple rule, preferably one complete generated word per Boolean:

```text
(nextU64() >> 63) != 0
```

Do not maintain a hidden bit reservoir in v1.

A bit reservoir can improve throughput but complicates call-order semantics:

```text
nextBool
nextFloat
nextBool
```

would consume the underlying stream in less obvious ways.

One-word-per-call makes deterministic replay and specification substantially simpler.

---

# 20. Bounded integer generation

## 20.1 Required semantic properties

```telos
nextInt(below upper: Int) -> Int
```

requires:

```text
upper > 0
0 <= result < upper
```

and must be unbiased.

Do not implement:

```text
nextU64() % upper
```

except in mathematically proven special cases.

Modulo reduction introduces bias whenever the generator's source range is not an exact multiple of the requested range.

---

## 20.2 Arbitrary-precision bounds

Because Telos `Int` is arbitrary precision, the public API should naturally support:

```telos
rng.nextInt(below: 10 ** 200)
```

rather than artificially limiting the operation to 64-bit bounds.

Implement rejection sampling using the target's bit length.

Conceptually:

```text
bits = bitLength(upper - 1)
words = ceil(bits / 64)

loop:
    concatenate enough nextU64 words
    mask away excess high bits
    candidate = resulting integer

    if candidate < upper:
        return candidate
```

Important:

- avoid bias;
- avoid Float conversion;
- avoid an artificial 64-bit ceiling;
- respect existing integer resource limits;
- handle powers of two efficiently;
- reject `upper <= 0` with `ArgumentError`.

The exact concatenation order must be specified and frozen for replay.

Recommended canonical ordering: earlier generated words contribute higher-order chunks, with the final assembled integer then masked to the required width.

Whatever ordering is selected must be tested with golden vectors.

---

## 20.3 Range overload

Provide:

```telos
nextInt(from lower: Int, below upper: Int) -> Int
```

with:

```text
lower <= result < upper
upper > lower
```

Implementation:

```text
lower + nextInt(below: upper - lower)
```

This naturally supports negative lower bounds and arbitrarily large ranges.

Do not add a large matrix of inclusive/exclusive variants in v1.

Half-open ranges are sufficient and compose well with indexing semantics.

---

# 21. Random bytes

## 21.1 Deterministic generator bytes

A `Generator` may provide deterministic bytes:

```telos
fill(_ bytes: Bytes) -> Unit
bytes(_ count: Int) -> Bytes
```

These bytes are **not secure merely because they look random**.

Freeze byte extraction order.

Recommended deterministic word-to-byte mapping:

```text
byte 0 = bits 63..56
byte 1 = bits 55..48
...
byte 7 = bits 7..0
```

i.e. canonical big-endian logical order independent of host architecture.

For a non-multiple-of-eight requested length, consume one additional word and use the leading required bytes. Discard the unused remainder rather than introducing hidden cached byte state.

This keeps method interleaving deterministic and simple.

Example:

```telos
const rng = random.Xoshiro256PlusPlus.seed(42)
const bytes = rng.bytes(32)
```

This must reproduce identically across architectures.

---

# 22. `choose`

Provide initially for `List<T>`:

```telos
choose<T>(_ values: List<T>) -> Option<T>
```

Semantics:

```text
empty list      → None
non-empty list  → Some(values[nextInt(below: values.size)])
```

Do not define generic:

```telos
choose<T>(_ values: Iterable<T>)
```

in v1.

An arbitrary `Iterable` may be:

- one-shot;
- expensive;
- infinite;
- lazily effectful;
- non-random-access.

Those semantics should not be hidden behind a seemingly trivial operation.

A future explicit `reservoirSample` can handle streaming iterables.

---

# 23. `shuffle`

Provide Fisher–Yates for `List<T>`:

```telos
shuffle<T>(_ values: List<T>) -> Unit
```

Use:

```text
for i from n-1 down to 1:
    j = nextInt(below: i + 1)
    swap(values[i], values[j])
```

The implementation must be unbiased assuming unbiased `nextInt`.

The precise iteration direction and random consumption order become part of deterministic replay. Add golden shuffle tests.

If `List`'s mutation API differs in current Universe source, use the established mutating operations rather than introducing random-specific collection primitives.

Do not generalize over an invented `MutableRandomAccessCollection` trait solely for this package in the first implementation.

---

# 24. `Distribution`

Add a typed distribution abstraction using Telos associated types.

Conceptually:

```telos
trait Distribution {
    type Output

    sample<G>(using generator: G) -> Self::Output
        where G |= Generator
}
```

Adapt exact generic/constraint syntax to current Telos.

The critical semantic property is:

```text
Distribution determines its result type.
Generator supplies deterministic entropy/state.
```

This is substantially preferable to dynamically typed distribution values returning `Dynamic`.

---

# 25. `UniformInt`

Provide a validated nominal distribution:

```telos
data UniformInt private(
    _ lower: Int,
    _ upperExclusive: Int,
)
```

Construction:

```telos
impl UniformInt {
    @class
    from(_ lower: Int, below upper: Int) -> UniformInt {
        if upper <= lower {
            ArgumentError.new(...).raise()
        }

        UniformInt(lower, upper)
    }
}
```

Sampling:

```telos
impl Distribution for UniformInt {
    type Output = Int

    sample<G>(using generator: G) -> Int
        where G |= Generator
    {
        generator.nextInt(
            from: self.lower,
            below: self.upperExclusive,
        )
    }
}
```

Usage:

```telos
const die = random.UniformInt.from(1, below: 7)

const rng = random.Xoshiro256PlusPlus.seed(42)

const first = die.sample(using: rng)
const second = die.sample(using: rng)
```

The constructor validates the invariant once. Sampling should not repeatedly rediscover malformed distribution state.

---

# 26. `Bernoulli`

Provide:

```telos
data Bernoulli private(
    _ probability: Float,
)
```

with construction validating:

```text
0.0 <= probability <= 1.0
probability is not NaN
```

Result type:

```telos
impl Distribution for Bernoulli {
    type Output = Bool
}
```

Boundary semantics must be exact:

```text
p = 0.0 → always false
p = 1.0 → always true
```

Avoid implementations where `p = 1.0` could theoretically fail because of floating-point comparison details.

For intermediate probabilities, a `nextFloat < probability` formulation is acceptable for v1 and has clear semantics.

The constructor should reject NaN explicitly because ordinary ordered comparisons with NaN do not establish the desired invariant.

---

# 27. Defer complex distributions

Do not initially implement:

```text
Normal
Gaussian
Exponential
Gamma
Beta
Poisson
weighted choice
alias sampling
```

until the deterministic replay policy for distribution algorithms is deliberately specified.

For example, a normal distribution may depend on:

- `ln`;
- `sqrt`;
- `sin`/`cos`;
- Box–Muller versus Ziggurat versus another algorithm;
- whether samples are cached in pairs.

Changing any of those changes the deterministic output sequence even with the same generator state.

The base PRNG should first establish a rigorous replay contract. Complex distributions can be layered later with their own named/frozen algorithms.

---

# 28. Secure system randomness

## 28.1 Separate abstraction

Implement secure system-provided randomness separately:

```telos
native class SecureRandom {
    @class
    native system -> SecureRandom

    native fill(_ bytes: Bytes) -> Unit

    bytes(_ count: Int) -> Bytes {
        // validate
        // allocate
        // fill
        // return
    }
}
```

Canonical use:

```telos
const secure = random.SecureRandom.system
const token = secure.bytes(32)
```

The exact class/data representation should follow existing native-class architecture.

The important rule is:

> `SecureRandom` must not implement `Generator` merely because both can produce random-looking bits.

This is intentional.

A deterministic generator is acceptable for simulation, randomized algorithms, tests, reproducible sampling, shuffling, and procedural generation.

A secure system source is security authority.

A type that accepts:

```telos
generator: Generator
```

must not accidentally imply cryptographic suitability.

Future security-sensitive APIs should be able to require a secure source and reject:

```telos
Xoshiro256PlusPlus.seed(42)
```

statically.

---

# 29. Secure randomness implementation boundary

Only the system entropy/CSPRNG interface genuinely requires a host-native boundary in the initial `random` package.

Investigate the existing Rust dependency tree and supported platforms. Use an established system randomness abstraction rather than implementing platform-specific `/dev/urandom`, BCrypt, `getentropy`, or equivalent code manually unless the runtime already centralizes that layer.

The Rust implementation must:

- use the operating system's secure random facility;
- propagate failure;
- never fall back to deterministic PRNG output;
- never seed from time/PID/address data;
- avoid exposing VM-private hash randomization keys;
- support zero-length buffers;
- safely fill arbitrary valid `Bytes` buffers;
- obey existing allocation/resource limits before large allocations.

The term `SecureRandom` should mean "system-provided cryptographically suitable random bytes", not raw physical entropy.

Do not claim that the bytes are direct hardware entropy.

---

# 30. Secure entropy failure

If system randomness fails:

```text
FAIL
```

Do not do:

```text
system RNG failed
    ↓
seed xoshiro from time
    ↓
continue silently
```

That would catastrophically violate the type/API's security meaning.

First investigate the existing Universe error hierarchy for the most appropriate host/system failure.

Only introduce a narrow error such as:

```telos
EntropyError
```

if the existing error taxonomy genuinely cannot represent the failure properly.

Do not create a generic `RandomError` bucket.

---

# 31. VM hash seed isolation

The runtime may already maintain private randomness for hash randomization or other VM internals.

That state is **not part of `universe.random`**.

Do not expose:

```text
VM master seed
hashing master key
runtime-internal random state
```

through public APIs.

Do not derive public deterministic streams from VM internal keys.

Do not use public deterministic seeds to initialize private hash-hardening state.

These domains must remain independent.

---

# 32. Concurrency semantics

A deterministic generator is a mutable stateful object.

If multiple fibers share one generator:

```telos
const rng = Xoshiro256PlusPlus.seed(42)
```

then the resulting stream order naturally depends on the order in which those fibers call it.

Do not pretend shared mutable RNG state is deterministically parallel.

For reproducible concurrent code, use explicit independent streams.

In a later version, xoshiro's jump operations may support:

```telos
generator.jump()
generator.longJump()
```

or an explicit stream-splitting abstraction.

Do not add those in the first slice unless the base implementation is already stable and the exact semantics are carefully frozen.

---

# 33. Proposed `random` layout

Recommended source layout:

```text
phalcom-core/core/universe/src/random/
├── package.ph
├── generator.ph
├── xoshiro256pp.ph
├── distribution.ph
├── uniform.ph
├── bernoulli.ph
└── secure.ph
```

Native implementation:

```text
phalcom-core/src/primitive/random.rs
```

or the corresponding established primitive-module location.

Expected ownership:

```text
generator.ph
    Generator trait
    derived operations

xoshiro256pp.ph
    deterministic source-authored PRNG
    SplitMix64 seed expansion
    64-bit helpers

distribution.ph
    Distribution trait

uniform.ph
    UniformInt

bernoulli.ph
    Bernoulli

secure.ph
    SecureRandom source declaration
    source helper methods where possible

primitive/random.rs
    only OS secure randomness / native secure boundary
```

Avoid implementing xoshiro, bounded sampling, shuffle, and basic distributions in Rust unless Telos source is demonstrably blocked.

These are valuable standard-library dogfooding targets.

---

# 34. Proposed `random/package.ph`

Conceptually:

```telos
@!documentation("Deterministic pseudorandom generators, distributions, and secure system randomness.")

import .generator.Generator
import .distribution.Distribution

import .xoshiro256pp.Xoshiro256PlusPlus

import .uniform.UniformInt
import .bernoulli.Bernoulli

import .secure.SecureRandom

export (
    Generator,
    Distribution,

    Xoshiro256PlusPlus,

    UniformInt,
    Bernoulli,

    SecureRandom,
)
```

Do not export an unnamed default generator.

---

# 35. Example developer experience

## Deterministic simulation

```telos
import universe.random as random

const rng = random.Xoshiro256PlusPlus.seed(42)

for _ in 0..<10 {
    print(rng.nextFloat)
}
```

Same program + same algorithm + same seed + same calls must reproduce exactly.

---

## Huge exact range

```telos
const rng = random.Xoshiro256PlusPlus.seed(42)

const value = rng.nextInt(
    below: 10 ** 200
)
```

This must be unbiased and must not artificially truncate the requested domain to 64 bits.

---

## Integer range

```telos
const rng = random.Xoshiro256PlusPlus.seed(42)

const temperature = rng.nextInt(
    from: -30,
    below: 50,
)
```

Result:

```text
-30 <= temperature < 50
```

---

## Distribution

```telos
const rng = random.Xoshiro256PlusPlus.seed(42)

const die = random.UniformInt.from(
    1,
    below: 7,
)

const roll = die.sample(using: rng)
```

---

## Bernoulli

```telos
const coin = random.Bernoulli.new(0.5)
const result = coin.sample(using: rng)
```

Use the actual chosen validated constructor name consistently; if `new` is used, ensure invalid state cannot bypass validation through another public constructor.

---

## Shuffle

```telos
const rng = random.Xoshiro256PlusPlus.seed(42)

let values = List(1, 2, 3, 4, 5)

rng.shuffle(values)
```

---

## Security-sensitive bytes

```telos
const secure = random.SecureRandom.system
const tokenBytes = secure.bytes(32)
```

No seed exists because this is not a replayable deterministic PRNG.

---

# Part III — Cross-package architecture

## 36. Dependency direction

Preserve a simple dependency graph:

```text
scalar numeric layer
       │
       ├─────────────► math
       │
       └─────────────► random deterministic core
                              │
collections.Bytes ────────────┼────► deterministic bytes
                              │
                              └────► distributions

collections.Bytes ─────────────────► SecureRandom

math
  must NOT depend on random

random v1
  should NOT need math
```

This is deliberate.

Deferring complex continuous distributions avoids making base random depend immediately on transcendental math.

---

# 37. No compiler/runtime special semantics for either package

Neither `math` nor deterministic `random` should require:

- new parser syntax;
- checker special cases;
- new VM instructions;
- new primitive value tags;
- new heap object categories;
- magical package-name recognition;
- ad hoc static evaluator branches.

They should exercise existing language facilities:

```text
packages/modules
classes/data
traits
associated types
method families
family capture
native declaration association
Int
Float
Bytes
List
Option
ordinary validation/errors
```

If implementation encounters a blocker, identify the missing general-purpose facility rather than smuggling package-specific behavior into the compiler.

---

# Part IV — Testing requirements

## 38. `math` compile-time/API tests

Verify:

```text
universe.math is canonical and importable
all intended exports resolve
captured math functions retain family/callable semantics
selected family captures resolve to intended signatures
math.sqrt accepts Float
math.sqrt does not implicitly accept Int
math.gcd accepts Int
no synthetic public Math class exists
```

Where possible, inspect semantic IDs to prove captured operations resolve through canonical declaration identities rather than generated wrapper closures.

Add regression coverage specifically for:

```telos
let sqrt = &FloatMathKernel.sqrt(_)
export sqrt
```

because family capture is an intentional part of this standard-library architecture.

---

# 39. `math` constant tests

For every constant:

```text
pi
tau
e
ln2
ln10
sqrt2
```

assert exact binary64 bits.

Do not assert only formatted strings.

---

# 40. `math` IEEE edge matrix

For every relevant float function test:

```text
+0.0
-0.0
smallest/subnormal representative values
normal positive values
normal negative values
+∞
-∞
NaN
```

Function-specific boundary values must also be included.

Examples:

```text
sqrt:
    -1
    -0
    +0
    +1
    +∞

ln:
    negative
    -0
    +0
    1
    +∞

asin/acos:
    -1
    +1
    immediately outside domain
    ±∞

atan2:
    all signed-zero quadrant combinations
    finite quadrant combinations
    infinities
```

Signed-zero behavior needs bit-level assertions where meaningful.

---

# 41. `math` accuracy tests

Use a high-precision reference oracle in the Rust/test harness for broad randomized finite inputs.

For each function:

```text
generate representative binary64 values
exclude/handle special cases separately
compute high-precision expected result
compare returned Float in ULPs
```

Establish an explicit permitted ULP bound per function/backend rather than a vague decimal tolerance such as:

```text
abs(actual - expected) < 0.000001
```

Decimal epsilon tests are insufficient across different magnitudes.

---

# 42. Integer algorithm tests

## `gcd`

Test:

```text
gcd(0, 0)
gcd(0, n)
gcd(n, 0)
gcd(1, n)
negative arguments
large BigInt-backed Ints
commutativity
divisibility
```

Property:

```text
g = gcd(a, b)

g divides a
g divides b
g >= 0
```

---

## `lcm`

Test:

```text
zeros
negative arguments
coprime values
shared factors
very large integers
```

For nonzero values:

```text
gcd(a,b) * lcm(a,b) == abs(a*b)
```

subject to exact integer semantics/resource policy.

---

## `isqrt`

Test boundaries around squares:

```text
isqrt(k*k - 1) == k - 1
isqrt(k*k) == k
isqrt(k*k + 1) == k
```

for:

- small `k`;
- machine-inline integer thresholds;
- values around heap-BigInt transition;
- very large `k`.

Universal invariant:

```text
r*r <= n
(n < (r+1)*(r+1))
```

Negative input must reliably raise the chosen argument error.

---

# 43. Xoshiro golden vectors

Add authoritative vectors for:

```text
seed = 0
seed = 1
seed = 42
seed = 2^64 - 1
```

Assert at least the first substantial sequence of `nextU64` outputs against independently verified xoshiro256++ vectors using the exact selected SplitMix64 initialization convention.

Do not generate expected values using the implementation under test.

Store frozen constants.

This is one of the most important correctness suites in `random`.

---

# 44. Seed validation tests

Verify:

```text
seed(-1)               → ArgumentError
seed(2^64)             → ArgumentError
seed(2^64 + n)         → ArgumentError
seed(0)                → valid
seed(2^64 - 1)         → valid
```

Also verify that seed validation occurs before any invalid internal state mutation.

---

# 45. `nextFloat` tests

Assert:

```text
0.0 <= x < 1.0
```

over large samples.

More importantly, verify golden exact outputs by using frozen `nextU64` words and testing:

```text
mantissa = word >> 11
Float bits of mantissa / 2^53
```

Test the largest possible mantissa:

```text
2^53 - 1
```

and confirm the output remains strictly below `1.0`.

---

# 46. `nextBool` tests

Given known generated words:

```text
high bit 0 → false
high bit 1 → true
```

Verify one call consumes exactly one generator word.

Explicitly test interleaving:

```telos
rng.nextBool
rng.nextFloat
rng.nextBool
```

against frozen expected state/output.

---

# 47. Bounded integer correctness tests

Test:

```text
upper = 1
upper = 2
upper = powers of two
upper = power of two ± 1
upper = 2^64 - 1
upper = 2^64
upper = 2^64 + 1
upper = 10^100
upper = 10^1000
```

Validation:

```text
upper <= 0 → ArgumentError
```

For range form:

```text
lower < upper
lower negative
cross-zero range
very large negative and positive endpoints
invalid empty/reversed ranges
```

Golden tests must force rejection paths.

Do not only test successful first candidates; construct/mock a generator stream where the first candidate is outside the bound and the next one succeeds.

This proves rejection sampling rather than biased reduction.

Statistical chi-square/frequency tests can supplement these tests but must never replace exact structural testing.

---

# 48. Deterministic byte tests

For known `nextU64`:

```text
0x0123456789abcdef
```

the canonical byte sequence should be exactly whichever order the specification chooses, recommended:

```text
01 23 45 67 89 ab cd ef
```

Test lengths:

```text
0
1
7
8
9
15
16
large
```

Verify exact state consumption after partial final words.

---

# 49. `choose` tests

Verify:

```text
empty       → None
singleton   → always that element
multiple    → exact index mapping from deterministic generator
```

Use deterministic mock generators to hit first and final valid indices.

---

# 50. Shuffle tests

Required:

```text
empty list
single item
two items
larger list
duplicate values
```

Assert:

1. output is a permutation;
2. size is unchanged;
3. same seed produces same permutation;
4. a frozen seed/list pair produces a golden exact permutation;
5. no index outside valid range is requested;
6. Fisher–Yates iteration consumes the expected exact number of bounded samples.

Statistical permutation-distribution tests may supplement but are not primary correctness tests.

---

# 51. Distribution tests

## `UniformInt`

Test constructor validation and sampling delegation semantics.

Ensure:

```text
upper <= lower → ArgumentError
```

Test huge integer ranges.

---

## `Bernoulli`

Test:

```text
p = NaN → ArgumentError
p < 0 → ArgumentError
p > 1 → ArgumentError
p = 0 → always false
p = 1 → always true
p = 0.5 → deterministic expected result for mock inputs
```

---

# 52. SecureRandom tests

Do not test secure output with fixed expected bytes.

Instead test API and failure invariants.

Required:

```text
bytes(0) succeeds and returns empty Bytes
bytes(n) returns exact requested length
fill fills an existing Bytes buffer
negative count is rejected before allocation
large requests obey resource limits
```

At the native boundary, inject/mock host entropy failure and verify:

```text
failure propagates
no deterministic fallback occurs
no partially fabricated result is returned
```

Also add a **type-system test** proving:

```text
SecureRandom !|= Generator
```

or the equivalent negative conformance assertion.

This is an architectural invariant, not incidental implementation detail.

---

# 53. Cross-platform deterministic tests

The deterministic `random` suite must run identically on all supported platforms.

Golden vectors must detect accidental dependence on:

- host endianness;
- pointer width;
- signed shift behavior;
- Rust integer wrapping choices;
- machine word size;
- platform libc;
- hash randomization.

All source-level word arithmetic must be semantically defined using exact `Int` plus explicit 64-bit masking.

---

# Part V — Performance and resource behavior

## 54. Avoid accidental BigInt pathologies

Although xoshiro uses Telos `Int`, its values conceptually remain 64-bit words.

The implementation should ensure that operations do not create unnecessarily enormous intermediate integers.

Mask after shifts/arithmetic where required.

Inspect current `Int` representation:

```text
inline/small Int
BigInt-backed large Int
```

and benchmark the resulting source-authored xoshiro implementation.

Do not prematurely move it to Rust solely because `Int` is arbitrary precision. First measure.

If performance proves unacceptable, optimize the **general numeric primitives or source lowering** where appropriate before creating a hidden native RNG semantic duplicate.

A native deterministic generator should be a measured optimization fallback, not the default architecture.

---

# 55. Numeric resource policies

Both packages must respect the runtime's existing numeric limits.

Examples:

```text
isqrt(1 << absurdly_large_count)
nextInt(below: huge_user_constructed_integer)
lcm with enormous operands
```

must not bypass the existing `Int` allocation/bit-width policy.

Do not catch numeric-resource errors and continue with truncated values.

---

# Part VI — Documentation requirements

## 56. `math` documentation

Document prominently:

```text
Float math operates on IEEE binary64 Float.
Int is not implicitly converted to Float.
Domain-invalid real inputs generally produce IEEE NaN/infinity behavior.
gcd/lcm/isqrt are exact arbitrary-precision integer operations.
Trigonometric arguments use radians.
```

Do not describe Float math as mathematically exact.

---

# 57. `random` documentation

The package docs must clearly distinguish:

```text
Xoshiro256PlusPlus
    deterministic
    reproducible
    non-cryptographic

SecureRandom
    system-backed
    non-deterministic
    security-sensitive
```

Add an explicit warning to the concrete deterministic generator:

> Do not use this generator for secrets, authentication tokens, cryptographic keys, nonces requiring unpredictability, or other security-sensitive values.

Likewise explain that deterministic `Generator.bytes(...)` is not secure.

---

# Part VII — Explicit v1 non-goals

## 58. `math` non-goals

Do not expand the first slice into a general scientific/numerical package.

Defer:

```text
factorial
primality APIs
combinatorics
gamma/beta
erf
complex arithmetic
rational arithmetic
decimal arithmetic
vectors/matrices
statistics
integration
differentiation
generic algebraic hierarchy
unit-aware mathematics
```

These deserve independent design.

---

# 59. `random` non-goals

Defer:

```text
global/default RNG
implicit auto-seeding
Normal/Gamma/Poisson/etc.
weighted sampling
generic Iterable choosing
secure deterministic generators
PRNG state serialization
parallel stream families
jump/longJump public API
generator splitting
counter-based RNG
random UUID/token string helpers
```

Do not let implementation scope drift into these areas.

---

# Part VIII — Implementation sequence

## 60. Phase M1 — `math`

Implement and stabilize:

```text
FloatMathKernel declaration
native Rust bindings
family captures from math package
constants
gcd
lcm
isqrt
full math correctness tests
```

This phase is especially useful for proving that:

```telos
let sqrt = &FloatMathKernel.sqrt(_)
export sqrt
```

works correctly through imports, type checking, runtime dispatch, reflection/tooling, and native binding.

If family capture reveals compiler defects, fix the general family-capture mechanism. Do not replace it with closure wrappers to avoid the issue.

---

## 61. Phase R1 — deterministic `random`

Implement entirely in source if possible:

```text
Generator
xoshiro256++
SplitMix64
nextBool
nextFloat
nextInt
bytes/fill
choose
shuffle
```

Freeze golden vectors before moving on.

---

## 62. Phase R2 — `SecureRandom`

Add the native host boundary:

```text
SecureRandom.system
SecureRandom.fill
SecureRandom.bytes
```

Test failure handling and ensure there is no deterministic fallback.

---

## 63. Phase R3 — distributions

Add:

```text
Distribution
UniformInt
Bernoulli
```

Keep the slice intentionally small.

---

# Part IX — Architectural rejection criteria

The implementation should be considered architecturally incorrect if it does any of the following without a compelling documented reason:

```text
• creates a public Math class;
• exports closures instead of family captures for simple kernel aliases;
• exposes an entire kernel overload family accidentally through `...`;
• silently converts Int to Float for transcendental math;
• duplicates existing Float scalar operations unnecessarily;
• creates a global mutable RNG;
• silently seeds deterministic RNG from the clock/system;
• uses modulo reduction for bounded random integers;
• restricts nextInt to 64-bit bounds despite arbitrary-precision Int;
• makes SecureRandom conform to Generator;
• falls back to deterministic data when secure system randomness fails;
• exposes VM-private hashing seeds;
• implements source-expressible deterministic PRNG/distributions natively without evidence;
• adds VM opcodes/value kinds specifically for these libraries;
• relies on host endianness for deterministic byte output;
• treats statistical tests as proof of PRNG algorithm correctness;
• changes a named PRNG algorithm while retaining the same public type identity.
```

---

# Part X — Required implementation investigation

Before coding, inspect and cite in the implementation plan/record the concrete current symbols responsible for:

```text
native source declaration registration
method-family capture resolution
package-level export of captured families
native class/static method binding
Int bit operations
Int bitLength
Int resource limits
Float construction/conversion
Bytes allocation and mutation
List random access and swapping/mutation
Option construction
ArgumentError raising
trait associated-type declarations
trait default methods
```

Use those existing mechanisms.

If any required operation is missing, classify it as either:

```text
A. legitimate standard-library prerequisite
B. general compiler/runtime capability gap
C. convenience that can remain local to math/random
```

Do not introduce broad new facilities merely to make the implementation aesthetically symmetric.

---

# Part XI — Standard-library gaps that may surface

The work may legitimately expose general gaps.

Likely candidates include:

```text
Int.abs
bit-rotation helpers
generic mutable random-access collection capability
machine-word unsigned integer types
more explicit mutation capability typing
source-level secure host capability modeling
```

Do not automatically implement all of them.

For this v1:

```text
Int.abs
```

may be worth adding if it is clearly foundational and aligns with existing scalar ownership.

A generic mutable-collection capability is unnecessary merely to generalize shuffle.

A `UInt64` type is unnecessary merely to implement xoshiro.

A full mutation/effect capability system is outside this scope.

The implementation record should note encountered gaps separately so they can inform later standard-library/language work.

---

# Final target architecture

The completed first slice should look conceptually like this:

```text
universe.math
│
├── pi/tau/e/...
│
├── sqrt ─────── captured family ─────► FloatMathKernel.sqrt(Float)
├── exp  ─────── captured family ─────► FloatMathKernel.exp(Float)
├── sin  ─────── captured family ─────► FloatMathKernel.sin(Float)
│   ...
│
├── gcd   ────── source Telos
├── lcm   ────── source Telos
└── isqrt ────── source Telos


universe.random
│
├── Generator
│      │
│      └── derived source-Telos sampling operations
│
├── Xoshiro256PlusPlus
│      ├── explicit state
│      ├── deterministic
│      ├── source Telos
│      └── frozen replay contract
│
├── Distribution
│      ├── UniformInt
│      └── Bernoulli
│
└── SecureRandom
       ├── separate from Generator
       ├── OS/system secure randomness
       └── narrow native boundary
```

The central implementation principle is:

> **Use Telos to implement Telos's standard library wherever possible. Preserve exact numerical semantics, explicit state, explicit authority, canonical family identity, and source-owned Universe declarations. Keep native Rust limited to the places where the runtime or host genuinely owns the operation.**

For `math`, method-family capture is the canonical bridge between an internal native kernel and the public module:

```telos
let sqrt = &FloatMathKernel.sqrt(_)
```

not a closure wrapper.

For `random`, explicit deterministic generators and secure system randomness are deliberately separate concepts:

```telos
const simulation = random.Xoshiro256PlusPlus.seed(42)
const secrets = random.SecureRandom.system
```

That separation should remain visible in both the type system and the developer experience.
```