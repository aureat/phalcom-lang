---
kind: requirements-analysis
title: "Telos Standard Library `universe.random` — Comprehensive Requirements Analysis"
status: PROPOSED
completion: ANALYZED
verification: REPOSITORY-GROUNDED
repository: aureat/phalcom-lang
branch_observed: stdlib-random-math
prepared_against_revision: 9742d085580a98fee8822e1c66026d179ddd772e
prepared: 2026-10-07
scope:
  - universe.random
  - deterministic pseudorandom generation
  - generic typed sampling
  - simple distributions
  - secure system randomness
  - integration with Int, Float, Bytes, List, Option, effects, errors, resource policy
excludes:
  - implementation plan / patch sequencing
  - general cryptography package
  - global/default RNG
  - full statistical distribution catalog
---

# Telos Standard Library `universe.random`
## Comprehensive Requirements Analysis

## 0. Executive conclusion

`universe.random` should not be designed as a bag of convenience functions and should not imitate the ambient global-RNG model common in scripting languages.

The package is an unusually good fit for Telos's own development philosophy because it exercises several of the language's strongest architectural ideas simultaneously:

- **explicit state instead of hidden ambient state**;
- **source-authored abstractions over narrow native floors**;
- **traits and associated types rather than dynamically typed generic helpers**;
- **arbitrary-precision `Int` rather than machine-width leakage into the public language model**;
- **nominal, algorithm-specific deterministic generators whose identity carries replay meaning**;
- **security authority represented by a distinct type rather than a Boolean mode or runtime convention**;
- **effects that distinguish mutation from true nondeterminism**;
- **validated immutable `data` values for distribution descriptors**;
- **ordinary `Bytes`, `List`, `Option`, and errors rather than random-specific runtime containers**;
- **no compiler, VM, value-tag, or opcode special case for ordinary library behavior**.

The package should deliberately expose **three different semantic layers**:

```text
deterministic generator protocol
        │
        └── Xoshiro256PlusPlus
                │
                ├── nextU64
                ├── nextBool
                ├── nextFloat
                ├── nextInt
                ├── deterministic bytes
                ├── choose
                └── shuffle

typed distribution protocol
        │
        ├── UniformInt
        └── Bernoulli

secure host randomness capability
        │
        └── SecureRandom
                ├── fill(Bytes)
                └── bytes(count)
```

The crucial architectural rule is:

> **Deterministic pseudorandom generation and secure system randomness are different kinds of authority and must remain different in the type system.**

Accordingly:

```text
Xoshiro256PlusPlus |= Generator

SecureRandom !|= Generator
```

must be a tested architectural invariant, not a documentation suggestion.

The recommended complete first release is intentionally **feature-rich but bounded**. It should fully solve deterministic simulation, randomized algorithms, deterministic tests, unbiased exact integer sampling, deterministic byte streams, typed simple distributions, list selection/shuffling, and secure random bytes. It should deliberately leave complex continuous distributions, global RNG state, implicit seeding, state serialization, stream splitting, UUID/token formatting, and cryptographic protocol construction to later work.

---

# 1. Repository-grounded current state

The live `stdlib-random-math` branch currently contains only the package shell:

```telos
@!documentation("Pseudorandom number generation and distributions.")
```

at:

```text
phalcom-core/core/universe/src/random/package.ph
```

No random implementation exists yet.

The surrounding runtime/language substrate needed for a good implementation is already substantially stronger than the older standard-library planning documents assumed.

## 1.1 Exact arbitrary-precision `Int` is available

`phalcom-core/core/universe/src/scalar/number.ph` already exposes:

```telos
native class Int is Number {
  native &(_ other: Int) -> Int
  native |(_ other: Int) -> Int
  native ^(_ other: Int) -> Int
  native ~ -> Int
  native <<(_ shift: Int) -> Int
  native >>(_ shift: Int) -> Int
  native bitAt(_ index: Int) -> Int
  native bitCount -> Int
  native bitLength -> Int
  native trailingZeros -> Int
}
```

This is sufficient to express xoshiro-family word logic and arbitrary-precision bounded sampling without inventing a `UInt64` type.

The runtime already has a numeric resource policy:

```rust
pub numeric_policy: crate::value::NumericPolicy
```

with `max_integer_bits`, and arithmetic paths raise structured `#numericLimit` failures. `random` must preserve that model rather than bypass it.

## 1.2 `Float` has the needed explicit conversion boundary

`Float.new(Int)` is already the explicit exact-to-inexact conversion seam.

This permits a precise `nextFloat` construction from a 53-bit integer without adding implicit promotion or special random conversion behavior.

## 1.3 `Bytes` is a real mutable octet buffer

`Bytes` provides:

```text
Bytes.new(count)
size
at
set
fill
slice
copyInto
zeroize
```

and native storage is fixed-length and byte-oriented.

That gives both deterministic generators and `SecureRandom` a canonical destination for byte generation. No random-specific byte container is justified.

## 1.4 `List<T>` has usable random-access mutation

`List<T>` already supplies:

```telos
size
at(_)
[_]
at(_, put:)
[_]=(_)
swap(first:, second:)
```

which is enough for source-authored Fisher–Yates.

A new `MutableRandomAccessCollection` protocol is therefore not a prerequisite for v1.

## 1.5 Traits, associated types, generic methods, and default methods are live

`collections/iterator_laws.ph` demonstrates the exact style expected for `Generator` and `Distribution`:

```telos
trait Iterator {
  type Item

  next -> Option<Self::Item>

  map(_ f) { ... }

  flatMap<S>(_ f: (Self::Item) -> S) -> FlatMapIterator<Self, S>
    where S |= Iterable
  { ... }
}
```

and:

```telos
impl<I, U> Iterator for MappedIterator<I, U>
  where I |= Iterator
{
  ...
}
```

Therefore a typed random protocol should use the real type system rather than `Dynamic` escape hatches.

## 1.6 Validated private `data` construction is established style

Time types already use:

```telos
data Duration private(_ nanoseconds: Int)

impl Duration {
  @class
  ...
}
```

This is the right model for immutable distribution descriptors such as `UniformInt` and `Bernoulli`.

## 1.7 Source-owned classes with narrow native members are established style

`SystemClock` shows a useful pattern:

```telos
class SystemClock is Clock {
  ...
  @class
  @primitive
  internal native monotonicNanoseconds -> Int
}
```

This is preferable for `SecureRandom` to making the whole abstraction a native-owned opaque object. The public semantics can remain source-owned while only the host entropy operation is native.

## 1.8 Effects already distinguish mutation and nondeterminism

Native metadata supports:

```text
Mutation
Io
Scheduling
Reflection
Nondeterminism
Blocking
```

This is especially important for `random`.

A deterministic PRNG is:

```text
stateful
mutable
deterministic
```

whereas `SecureRandom` is:

```text
host-observing
nondeterministic
possibly blocking
and mutates a destination buffer
```

The implementation must not mark ordinary deterministic RNG methods as `Nondeterminism` merely because their outputs are random-looking.

---

# 2. Governing design philosophy

## 2.1 Randomness must be explicit in the object graph

Telos should require users to possess a generator value:

```telos
let rng = random.Xoshiro256PlusPlus.seed(42)

let x = rng.nextFloat
let n = rng.nextInt(below: 100)
```

The package must not provide:

```text
random()
rand()
random.nextInt(...)
random.seed(...)
random.default
random.global
thread-local RNG
fiber-local RNG
implicitly initialized default generator
```

This is not merely a reproducibility preference.

Hidden random state creates hidden mutation and hidden dependency edges. That conflicts directly with Telos's preference for explicit values, explicit capabilities, inspectable structure, and compositional semantics.

## 2.2 Deterministic pseudorandomness is not semantic nondeterminism

For a fixed generator state:

```text
next state + output
```

is deterministic.

Therefore:

```text
Xoshiro256PlusPlus.nextU64
```

conceptually carries a **mutation** effect, not a **nondeterminism** effect.

This distinction matters for future effect reasoning, testing, optimizer legality, static proofs, and user understanding.

By contrast:

```text
SecureRandom.fill
```

does observe the external host and is truly nondeterministic.

## 2.3 Named algorithms are semantic versions

The public concrete type should be:

```text
Xoshiro256PlusPlus
```

not:

```text
Random
Rng
DefaultGenerator
FastRandom
```

because a seeded deterministic generator is a replay contract.

If Telos later wants PCG, ChaCha-based deterministic streams, Philox, or another algorithm, those should become new named types rather than silently changing the output of an existing named generator.

## 2.4 Source Telos should implement source-expressible algorithms

The intended source/native split is:

```text
source Telos:
    Generator default methods
    Xoshiro256PlusPlus transition/state
    SplitMix64 seed expansion
    nextBool
    nextFloat
    nextInt
    deterministic bytes/fill
    choose
    shuffle
    Distribution
    UniformInt
    Bernoulli
    SecureRandom validation/convenience methods

native Rust:
    only secure OS random-byte acquisition
```

Moving xoshiro or bounded sampling directly to Rust should require measured evidence that the source implementation is not viable.

The package should be a dogfooding target for Telos's type system and ordinary runtime.

---

# 3. Recommended public developer experience

The package should make the common use cases obvious without sacrificing semantic precision.

## 3.1 Deterministic generator

```telos
import universe.random as random

let rng = random.Xoshiro256PlusPlus.seed(42)

let flag: Bool = rng.nextBool
let unit: Float = rng.nextFloat
let index: Int = rng.nextInt(below: 100)
let temperature: Int = rng.nextInt(from: -30, below: 50)
```

## 3.2 Arbitrary-precision integer range

A particularly Telos-like capability should be:

```telos
let huge = rng.nextInt(
  below: 10 ** 500
)
```

with no artificial machine-width ceiling.

## 3.3 Deterministic bytes

```telos
let packet = rng.bytes(32)
```

The output is deterministic and explicitly non-secure.

## 3.4 Choice

```telos
let result: Option<User> = rng.choose(users)
```

No exception for an empty list.

## 3.5 In-place shuffle

```telos
rng.shuffle(cards)
```

The operation mutates `cards` and generator state.

A `Unit` return is semantically clearer than pretending this is a pure transformation.

## 3.6 Typed distribution

```telos
let die = random.UniformInt.from(1, below: 7)
let roll: Int = die.sample(using: rng)

let coin = random.Bernoulli.new(0.5)
let heads: Bool = coin.sample(using: rng)
```

The distribution determines the result type statically.

## 3.7 Secure random bytes

```telos
let secure = random.SecureRandom.system
let token = secure.bytes(32)
```

No seed exists because this is not a deterministic replayable stream.

---

# 4. Public package architecture

The recommended public package exports:

```text
Generator
Distribution
Xoshiro256PlusPlus
UniformInt
Bernoulli
SecureRandom
EntropyError        if secure host failure becomes catchable source error
```

No default generator object should be exported.

No package-global convenience functions should hide generator selection.

## 4.1 Recommended source organization

```text
phalcom-core/core/universe/src/random/
├── package.ph
├── generator.ph
├── xoshiro256pp.ph
├── distribution.ph
├── uniform_int.ph
├── bernoulli.ph
└── secure.ph
```

Potential native implementation:

```text
phalcom-core/src/primitive/random.rs
```

The package should follow the same facade rule established for the corrected `math` package:

```text
registered physical child
!=
publicly exposed traversal child
```

`package.ph` should import and re-export the intended declarations without exposing internal organization as part of the public module path unless there is a deliberate reason.

The intended user namespace is:

```text
universe.random.Xoshiro256PlusPlus
universe.random.Generator
universe.random.UniformInt
```

not a requirement that users traverse:

```text
universe.random.xoshiro256pp.Xoshiro256PlusPlus
```

## 4.2 Dependency graph

The package should preserve:

```text
Number / Int / Float
        │
        ▼
Generator + Xoshiro
        │
        ├──────────────► Bytes
        │
        ├──────────────► List / Option
        │
        ▼
Distribution
        ├── UniformInt
        └── Bernoulli

Bytes ─────────────────► SecureRandom native fill

math ──X──► random core
random core ──X──► math
```

The initial deterministic package should not depend on `universe.math`.

That keeps replay independent from transcendental-backend differences.

---

# 5. `Generator` protocol requirements

## 5.1 Minimal primitive protocol

The essential required operation should be:

```telos
trait Generator {
  nextU64 -> Int
}
```

Contract:

```text
0 <= result < 2^64
```

Everything else should derive from this one primitive source of deterministic bits.

This gives custom generator authors a very small conformance obligation.

## 5.2 Recommended complete v1 trait surface

Conceptually:

```telos
trait Generator {
  nextU64 -> Int

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

All operations except `nextU64` should be default methods unless the language requires a concrete override for typing/lowering reasons.

## 5.3 Why `nextU64 -> Int` is acceptable

The method name describes the **generated word domain**, not a Telos machine integer type.

It does not imply the existence of a `UInt64` nominal type.

The return remains an exact Telos `Int`.

Alternative spelling:

```text
nextWord64
```

would be semantically clean, but `nextU64` has strong ecosystem recognizability and aligns directly with published PRNG reference vectors. The existing design should retain `nextU64` unless naming is deliberately reopened.

## 5.4 Consumption semantics are public semantics

For a named deterministic generator, these questions must not remain accidental implementation details:

```text
Does nextBool consume one full word?
Does nextInt(below: 1) consume a word?
Does choose(singleton) consume a word?
Does bytes(1) consume one full word or cache seven bytes?
Does Bernoulli(0) consume a word?
```

Recommended rules:

```text
nextU64:
    consumes exactly 1 generator word

nextBool:
    consumes exactly 1 generator word

nextFloat:
    consumes exactly 1 generator word

nextInt(below: 1):
    consumes 0 words

choose(empty):
    consumes 0 words

choose(singleton):
    consumes 0 words

shuffle(list of size 0 or 1):
    consumes 0 bounded samples

bytes(0):
    consumes 0 words

bytes(1..8):
    consumes 1 word

bytes(9..16):
    consumes 2 words

partial final byte word:
    unused bytes are discarded, not cached

Bernoulli(0.0):
    consumes 0 words

Bernoulli(1.0):
    consumes 0 words
```

These rules maximize transparency and avoid hidden internal reservoirs.

---

# 6. `Xoshiro256PlusPlus` requirements

## 6.1 Public state model

The generator should be an identity-bearing mutable class, not immutable `data`.

Conceptually:

```telos
class Xoshiro256PlusPlus {
  mut _s0: Int
  mut _s1: Int
  mut _s2: Int
  mut _s3: Int

  @constructor
  private create(...)

  @class
  seed(_ seed: Int) -> Xoshiro256PlusPlus
}
```

A PRNG is a stream-state object. Class identity and mutable fields are the correct model.

Do not define equality by comparing internal state. Default object identity is preferable.

Do not expose internal state as public fields.

Do not expose state serialization in the first release.

## 6.2 Seed contract

Required:

```text
0 <= seed < 2^64
```

Invalid:

```text
seed < 0
seed >= 2^64
```

must raise `ArgumentError`.

Do not normalize user seed values modulo `2^64`.

Reason:

```text
seed(1)
seed(2^64 + 1)
```

must not silently become aliases.

## 6.3 SplitMix64 initialization

The four state words should be expanded from the seed using the standard SplitMix64 sequence convention selected by the package.

The exact:

```text
increment
mix constants
word order
initial state order
```

must be frozen by independent golden vectors.

## 6.4 Unsigned 64-bit semantics over arbitrary-precision `Int`

Define:

```text
MASK64 = 2^64 - 1
```

and maintain:

```text
0 <= each logical state word < 2^64
```

for the simplest correctness model.

Every operation whose mathematical meaning is modulo `2^64` must explicitly mask after the relevant operation.

Rotation must be width-specific:

```text
rotl64(x, k)
```

and must not accidentally inherit arbitrary-precision shift semantics.

## 6.5 Important performance risk: LargeInt churn

Telos's public `Int` is arbitrary precision and its immediate representation is signed-machine-like.

A nonnegative logical 64-bit word is above signed `i64` range approximately half of the time.

Therefore a naïve source implementation that represents every xoshiro word as:

```text
0 .. 2^64-1
```

may allocate heap-backed `LargeInt` values frequently.

This is the most important performance question in the deterministic design.

Requirements:

1. **Correctness comes first.** The logical stream must exactly match xoshiro256++.
2. The first source implementation may use straightforward nonnegative 64-bit `Int` words.
3. It must be benchmarked before declaring source performance acceptable.
4. If allocations are pathological, optimize the **internal representation**, not the public API.
5. Acceptable later source-level optimization strategies include:
   - signed two's-complement internal words plus explicitly correct logical shifts;
   - fixed-width chunk decomposition such as 32-bit halves;
   - a general-purpose fixed-width integer helper abstraction if independently justified.
6. Moving the deterministic algorithm to Rust should be a measured fallback, not the first response.
7. Adding a public `UInt64` solely for this package is not justified.

This concern should be measured with:

```text
nextU64 throughput
allocation count
GC pressure
nextInt throughput
shuffle throughput
```

relative to existing source-authored PRNG benchmarks.

## 6.6 Replay contract

The following are part of the public deterministic semantics:

```text
seed range
SplitMix64 expansion
state ordering
xoshiro256++ output expression
transition ordering
rotation widths
masking semantics
nextBool mapping
nextFloat mapping
nextInt bit assembly
rejection behavior
deterministic byte order
byte consumption behavior
Fisher–Yates direction
distribution boundary consumption
```

A future implementation optimization must preserve all of them.

---

# 7. `nextBool` requirements

Recommended definition:

```text
(nextU64 >> 63) != 0
```

It consumes one complete generated word.

Do not add a bit reservoir.

The reservoir optimization would make method interleaving depend on hidden cache state and complicate replay:

```telos
rng.nextBool
rng.nextFloat
rng.nextBool
```

One word per Boolean is simpler to reason about, document, debug, and test.

---

# 8. `nextFloat` requirements

Required result domain:

```text
0.0 <= result < 1.0
```

Recommended construction:

```text
word = nextU64
mantissa = word >> 11
result = Float.new(mantissa) / 2^53
```

Reasons:

- uses exactly 53 random bits;
- `mantissa < 2^53`;
- every such integer is exactly representable in binary64;
- output never becomes `1.0`;
- behavior is platform-independent.

Do not use:

```text
Float.new(nextU64) / 2^64
```

because the 64-bit integer is not always exactly representable as binary64.

The exact resulting Float bits should be frozen in golden tests.

---

# 9. `nextInt` requirements

This is one of the most valuable parts of the Telos API because `Int` is arbitrary precision.

## 9.1 Single-bound form

```telos
nextInt(below upper: Int) -> Int
```

precondition:

```text
upper > 0
```

postcondition:

```text
0 <= result < upper
```

distribution:

```text
uniform over every Int in the half-open range
```

## 9.2 No modulo bias

This implementation is forbidden:

```text
nextU64 % upper
```

except in a mathematically proven exact special case.

The generic algorithm must use rejection sampling.

## 9.3 Arbitrary-precision rejection sampling

For:

```text
upper > 1
```

compute:

```text
bits = bitLength(upper - 1)
wordCount = ceil(bits / 64)
```

Then:

```text
repeat:
    candidate = concatenate wordCount generated 64-bit words
    remove excess high bits
until candidate < upper
```

Return candidate.

Recommended canonical concatenation:

```text
earlier nextU64 words occupy higher-order chunks
later words occupy lower-order chunks
```

The order must be frozen.

## 9.4 Degenerate upper bound

For:

```text
upper == 1
```

return:

```text
0
```

and consume **zero** generator words.

This is mathematically deterministic and avoids needless stream advancement.

## 9.5 Range form

```telos
nextInt(from lower: Int, below upper: Int) -> Int
```

precondition:

```text
upper > lower
```

implementation semantics:

```text
lower + nextInt(below: upper - lower)
```

This must support:

```text
negative ranges
cross-zero ranges
very large positive/negative endpoints
ranges wider than 64 bits
```

No inclusive/exclusive API matrix is needed initially. Half-open intervals are sufficient and align with collection indexing.

## 9.6 Resource semantics

The range width:

```text
upper - lower
```

and candidate assembly are ordinary exact integer operations.

They must remain subject to existing numeric resource limits.

Do not catch `#numericLimit` and silently reduce precision/range width.

---

# 10. Deterministic byte generation

## 10.1 `fill`

```telos
fill(_ bytes: Bytes) -> Unit
```

should fill the complete target in deterministic stream order.

Recommended mapping of each `nextU64`:

```text
bits 63..56 -> byte 0
bits 55..48 -> byte 1
...
bits 7..0   -> byte 7
```

This is a logical big-endian extraction order independent of host endianness.

## 10.2 Partial word handling

For lengths not divisible by 8:

```text
consume one final complete word
use only the leading requested bytes
discard the remainder
```

Do not maintain a hidden pending-byte reservoir.

## 10.3 `bytes`

```telos
bytes(_ count: Int) -> Bytes
```

requirements:

```text
count >= 0
```

Invalid counts raise `ArgumentError` before allocation.

Implementation concept:

```text
validate
allocate Bytes.new(count)
fill
return
```

The generated bytes must carry an explicit non-security warning in documentation.

---

# 11. Collection integration

## 11.1 `choose`

Required:

```telos
choose<T>(_ values: List<T>) -> Option<T>
```

Semantics:

```text
empty:
    None
    0 random words

singleton:
    Some(values[0])
    0 random words

size > 1:
    Some(values[nextInt(below: values.size)])
```

This API preserves `T` and should not return `Dynamic`.

Do not generalize to arbitrary `Iterable<T>`.

An iterable may be:

```text
infinite
one-shot
effectful
expensive
lazy
```

and uniform selection over a stream requires different semantics such as reservoir sampling.

## 11.2 `shuffle`

Required:

```telos
shuffle<T>(_ values: List<T>) -> Unit
```

Use in-place descending Fisher–Yates:

```text
i = n - 1 .. 1
j = nextInt(below: i + 1)
swap(i, j)
```

Use the existing `List` mutation surface.

Do not introduce native list-shuffle primitives.

Do not return a new List from `shuffle`; the name denotes mutation.

A future explicit:

```text
shuffled
```

copying convenience could be added separately if the collection library develops a consistent copied-transform naming convention.

---

# 12. `Distribution` protocol requirements

The random package should exercise Telos associated types directly.

Recommended concept:

```telos
trait Distribution {
  type Output

  sample<G>(using generator: G) -> Self::Output
    where G |= Generator
}
```

The exact syntax must match the current checker grammar, but the type relationship is required.

This yields:

```text
UniformInt::Output = Int
Bernoulli::Output  = Bool
```

and avoids:

```text
sample(...) -> Dynamic
```

## 12.1 Ownership direction

The canonical operation should be:

```telos
distribution.sample(using: generator)
```

rather than forcing the base `Generator` protocol to depend on `Distribution`.

That keeps the dependency graph acyclic:

```text
Generator
    ↑
Distribution references Generator
```

An ergonomic convenience:

```telos
rng.sample(distribution)
```

could be added later only if it can be expressed without creating awkward mutual source dependencies or duplicating protocol logic.

The first implementation should prefer architectural clarity over symmetrical sugar.

## 12.2 Distribution values should be immutable validated descriptors

Use `data` where appropriate.

A distribution parameter object is not a mutable entropy source.

This distinction is valuable:

```text
generator = mutable class
distribution = immutable validated data
```

---

# 13. `UniformInt`

Recommended:

```telos
data UniformInt private(
  _ lower: Int,
  _ upperExclusive: Int,
)
```

Validated construction:

```telos
UniformInt.from(lower, below: upper)
```

requires:

```text
upper > lower
```

Invalid bounds raise `ArgumentError`.

Sampling delegates to:

```text
generator.nextInt(from: lower, below: upperExclusive)
```

Validation should happen once at construction, not on every sample.

The type must support arbitrarily large bounds.

---

# 14. `Bernoulli`

Recommended:

```telos
data Bernoulli private(
  _ probability: Float,
)
```

Validated construction requires:

```text
probability is not NaN
0.0 <= probability <= 1.0
```

Invalid values raise `ArgumentError`.

Sampling:

```text
p == 0.0 -> false, consume 0 words
p == 1.0 -> true, consume 0 words
otherwise -> generator.nextFloat < p
```

The zero-word boundary rule should be frozen as replay semantics.

The public type result is:

```text
Bool
```

not `Dynamic`.

---

# 15. Why more distributions should not be added yet

A "comprehensive" random package does not require a large statistical distribution catalog in its first correct implementation.

Defer:

```text
Normal
LogNormal
Exponential
Gamma
Beta
Poisson
Binomial
weighted choice
alias tables
categorical
geometric
```

The reason is deterministic replay.

A named distribution can itself become a replay contract if users depend on seeded exact sequences.

For example, Normal sampling depends on:

```text
algorithm choice
Float transcendental backend
rejection strategy
pair caching
consumption order
```

Changing Box–Muller to Ziggurat would change output despite identical generator state.

Complex distributions should therefore be added with their own explicit deterministic-algorithm/version policy rather than casually layered into the first package.

---

# 16. `SecureRandom` as an explicit capability

## 16.1 Separate nominal type

`SecureRandom` must be separate from `Generator`.

Do not define:

```telos
impl Generator for SecureRandom
```

Do not define one broad:

```text
RandomSource
```

that erases the security distinction.

A security-sensitive API should be able to require:

```text
SecureRandom
```

and reject deterministic xoshiro statically.

## 16.2 Recommended source-owned shape

Prefer an ordinary source class with a narrow internal native method.

Conceptually:

```telos
class SecureRandom {
  @class mut _system

  @constructor
  private create() { self }

  @class
  system -> SecureRandom {
    if _system == None {
      _system = SecureRandom.create()
    }
    _system
  }

  internal native fillSystem(_ bytes: Bytes) -> Unit

  fill(_ bytes: Bytes) -> Unit {
    self.fillSystem(bytes)
  }

  bytes(_ count: Int) -> Bytes {
    ...
  }
}
```

Exact syntax and singleton mechanics should follow the established `SystemClock.instance` style.

This approach is superior to an entirely native class because:

- source remains semantic authority;
- validation stays source-authored;
- only host entropy acquisition is native;
- `SecureRandom.system` is an explicit capability value;
- future dependency injection remains possible.

## 16.3 Why `system` should return a value

A capability object:

```telos
let secure = SecureRandom.system
```

is preferable to package-global static host calls.

It gives security authority a first-class value that can be passed explicitly:

```telos
makeToken(using: secure)
```

without creating global ambient authority.

## 16.4 Public secure API should remain narrow

Required:

```text
fill(Bytes)
bytes(Int)
```

Do not add in the first release:

```text
secure.nextFloat
secure.shuffle
secure.choose
secure.password
secure.tokenString
secure.uuid
secure.int
```

Security-sensitive higher-level sampling often needs protocol-specific bias and encoding decisions.

A secure-byte capability is a much safer foundational boundary.

---

# 17. Host/native secure-random boundary

A direct dependency on:

```text
getrandom 0.3.x
```

is appropriate.

The repository already has `getrandom 0.3.3` transitively, but `phalcom-core` should declare it directly if it calls the crate directly.

The native primitive should use the crate's supported OS abstraction rather than hand-written platform code.

Required properties:

```text
OS-backed cryptographically suitable random source
no deterministic fallback
no clock/PID/address fallback
no VM hash seed reuse
zero-length buffer support
host failure propagation
no user-visible partially fabricated result from bytes(count)
```

The package documentation should call this **system secure randomness**, not "raw physical entropy".

---

# 18. SecureRandom effect requirements

This is an important Telos-specific semantic distinction.

## 18.1 Deterministic generator

`Xoshiro256PlusPlus.nextU64` should semantically contribute:

```text
Mutation
```

through source effect inference.

It should not be marked:

```text
Nondeterminism
```

because its output is a pure function of prior generator state.

## 18.2 Secure fill

The native system-fill operation must declare at least:

```text
Mutation
Nondeterminism
```

because it mutates the target buffer and observes host random state.

It should also be evaluated against the project effect policy for:

```text
Io
Blocking
```

`getrandom` may interact synchronously with the host and can have blocking behavior on some systems/startup states.

The final metadata should be explicit rather than left `Unknown`.

A likely strong declaration is conceptually:

```text
[mutation, nondeterminism, blocking]
```

with `io` included if the project's effect taxonomy classifies system entropy acquisition as I/O.

---

# 19. Secure failure model

This is the most important unresolved runtime-integration requirement.

The current source error hierarchy has `ArgumentError` and related classes, but there is no clearly established source-visible system-entropy failure class.

A secure-random acquisition failure is:

```text
not ArgumentError
not NumericLimit
not a deterministic RNG condition
not an ordinary absence value
```

The implementation must not convert it to:

```text
false
None
empty bytes
zero bytes
deterministic fallback
```

## 19.1 Recommended source error

A narrow error such as:

```telos
class EntropyError is Error {}
```

is reasonable if the existing host-error architecture has no better catchable source type.

The name should be documented carefully:

```text
EntropyError means failure to acquire system secure randomness.
It does not claim the API exposes raw physical entropy.
```

## 19.2 Required integration gate

Before secure random implementation is considered complete, the runtime must answer:

```text
How does a native getrandom::Error become a catchable source Error value?
```

If no canonical general native-error minting path exists, that is a **general runtime capability gap**, not permission to use `RuntimeError::Message` or misclassify the failure.

The correct solution may be:

- a narrow random-specific structured runtime error mapped to source `EntropyError`; or
- reuse of a general host/IO error object if one becomes canonical.

This decision should be resolved before R2 implementation, not buried in primitive code.

## 19.3 Partial destination mutation on failure

For:

```telos
secure.fill(buffer)
```

the contract must state what happens if the host API fails.

The safest minimum requirement is:

```text
the method raises;
no successful result is returned;
buffer contents after failure are unspecified.
```

`secure.bytes(count)` is easier:

```text
allocate zeroed Bytes
attempt fill
return only on full success
```

so a partial buffer is never returned to the caller.

---

# 20. SecureRandom memory and GC requirements

The native fill implementation should mutate an existing `Bytes` buffer directly.

Requirements:

- validate receiver and argument types;
- obtain the mutable byte slice;
- call the host random provider;
- do not re-enter the Telos interpreter while the mutable slice is borrowed;
- do not trigger GC during the borrowed slice lifetime;
- do not allocate Telos heap values while holding that mutable byte borrow;
- propagate host failure after releasing the borrow;
- mark native effects honestly.

Because the primitive does not need user callbacks or reentrant sends, it should remain a narrow non-reentrant host call.

---

# 21. Concurrency requirements

## 21.1 Shared deterministic generator

If multiple fibers share one generator:

```telos
let rng = Xoshiro256PlusPlus.seed(42)
```

the output sequence depends on actual call ordering.

This is expected.

Do not promise schedule-independent deterministic parallelism from one mutable stream.

## 21.2 Reproducible concurrent code

The documentation should recommend:

```text
one explicit generator per logical worker
```

with caller-selected deterministic seeds.

## 21.3 No implicit synchronization

Do not put an internal mutex, scheduler lock, or hidden fiber-local stream behind `Generator`.

If concurrent mutation races are impossible due current cooperative execution, preserve ordinary object semantics.

If future parallelism introduces true simultaneous access, mutation/capability design should address that globally rather than giving RNG a bespoke concurrency model.

## 21.4 Future stream splitting

xoshiro supports jump/long-jump mechanisms, but these should be deferred until:

```text
base replay contract is frozen
parallel stream semantics are designed
stream ownership is clear
```

---

# 22. Mutation and capability-system implications

`random` will become an important real-world case for future Telos mutation-capability design.

Today, a source object with `mut` fields can mutate internal state even if the binding holding it is immutable:

```telos
let rng = Xoshiro256PlusPlus.seed(42)
rng.nextU64
```

The `let` binding does not rebind, while the object mutates.

That is consistent with the current object model.

The random package should not wait for a future generalized `mut` capability system.

However, implementation records should explicitly note:

```text
Generator is semantically state-consuming.
```

This will be valuable input if Telos later adds:

```text
readonly views
mutation capabilities
borrow-like access
effect-qualified interfaces
```

Do not create RNG-specific mutation syntax.

---

# 23. Typing requirements in detail

## 23.1 No `Dynamic` escape for public APIs

Public signatures should retain strong types:

```text
nextU64 -> Int
nextBool -> Bool
nextFloat -> Float
nextInt -> Int
bytes -> Bytes
choose<T>(List<T>) -> Option<T>
shuffle<T>(List<T>) -> Unit
UniformInt.Output = Int
Bernoulli.Output = Bool
```

If a checker limitation prevents one of these signatures, record and fix the general typing limitation rather than permanently weakening the API to `Dynamic`.

## 23.2 Trait default methods must preserve exact self dispatch

Default `Generator` methods should call:

```text
self.nextU64
```

through ordinary trait dispatch.

A custom generator conforming to `Generator` should inherit all derived behavior without implementing each helper.

## 23.3 Associated-type selection must work in `Distribution`

Required type expression:

```text
Self::Output
```

or canonical equivalent.

A distribution sample call should infer its output without explicit casts.

## 23.4 Generic collection methods

Required:

```telos
choose<T>(_ values: List<T>) -> Option<T>
shuffle<T>(_ values: List<T>) -> Unit
```

These methods are valuable acceptance tests for method-local generics on trait defaults.

## 23.5 Negative conformance test

The type system must reject code requiring:

```text
SecureRandom |= Generator
```

This should be a compile-time test.

---

# 24. Developer ergonomics decisions

## 24.1 Prefer descriptive half-open labels

Recommended:

```telos
rng.nextInt(below: 10)
rng.nextInt(from: 10, below: 20)
```

This is clearer than ambiguous:

```text
between
range
to
max
```

and makes exclusivity explicit.

## 24.2 Keep state-consuming methods on the generator

Prefer:

```telos
rng.choose(values)
rng.shuffle(values)
```

over extending `List` with:

```telos
values.random(using: rng)
values.shuffle(using: rng)
```

Reasons:

- avoids `collections -> random` dependency;
- keeps entropy consumption visibly owned by the generator;
- avoids extending every collection with random-domain concerns.

## 24.3 Keep distribution construction separate from sampling

```telos
let die = UniformInt.from(1, below: 7)
let result = die.sample(using: rng)
```

is good Telos API design because:

- invalid state is rejected once;
- the distribution is reusable;
- the output type is attached to the descriptor;
- the generator remains a separate stateful dependency.

## 24.4 Do not add convenience that hides security choice

Avoid a helper such as:

```telos
random.bytes(32)
```

because the caller would not know whether it means deterministic or secure randomness.

Require one of:

```telos
rng.bytes(32)
secure.bytes(32)
```

The receiver communicates semantics.

---

# 25. Package topology and canonical source ownership

The `random` facade should behave like a normal Telos package.

Recommended physical graph:

```text
universe.random
    generator
    xoshiro256pp
    distribution
    uniform_int
    bernoulli
    secure
```

All child modules should be registered with the Universe source provider.

Public traversal exposure should be minimal.

The facade re-exports the public declarations.

The secure native owner needs a canonical source declaration and, if required by the primitive metadata system, a new `UniverseKey::SecureRandom` associated with:

```text
["random", "secure"]
```

The native descriptor must attach to that declaration.

No native parallel declaration should exist.

---

# 26. Native implementation requirements

The likely only required random primitive is conceptually:

```text
SecureRandom.fillSystem(Bytes) -> Unit
```

or an equivalent source-owned internal primitive.

The primitive must:

```text
use getrandom
fill exact target length
handle zero length
propagate failure
never fallback
declare effects
avoid interpreter reentry
avoid heap allocation while mutably borrowing Bytes
```

Do not implement in Rust:

```text
xoshiro transition
SplitMix64
nextInt
nextBool
nextFloat
choose
shuffle
UniformInt
Bernoulli
```

unless a measured source/runtime blocker is documented.

---

# 27. Resource policy requirements

## 27.1 Exact integer assembly

`nextInt` may construct arbitrarily large candidates.

The implementation must respect `NumericPolicy.max_integer_bits`.

It must not construct integers substantially wider than the requested bound.

## 27.2 Byte allocation

`bytes(count)` must validate:

```text
count >= 0
count representable by Bytes allocation limits
```

before attempting large allocation.

The existing `Bytes.new` boundary should remain authoritative for host allocation feasibility.

## 27.3 Rejection loops

For valid finite bounds, rejection sampling terminates probabilistically with expected bounded work.

Tests should force one or more rejection paths deterministically.

Do not add an arbitrary tiny retry cap that biases output or fails valid requests.

## 27.4 Secure source failure

Host entropy failure is not a resource-limit fallback opportunity.

Do not generate deterministic bytes under pressure/failure.

---

# 28. Determinism contract across platforms

Deterministic APIs must not depend on:

```text
host byte order
pointer width
Rust hash randomization
platform libc
OS RNG
system time
object address
VM hash seed
optimization level
```

All fixed-width behavior must be defined with explicit `Int` operations and masks.

Deterministic bytes must define logical byte order explicitly.

Golden vectors should run unchanged across all supported targets.

---

# 29. VM-private randomness isolation

If the VM has or later gains private randomness for:

```text
hash hardening
scheduler internals
runtime security
```

that state is a separate domain.

Public random must not expose:

```text
VM master seed
hash secret
internal keyed-hash state
```

and must not use those values as deterministic public seeds.

Conversely:

```text
Xoshiro256PlusPlus.seed(42)
```

must not weaken VM-private hash randomization.

---

# 30. Testing requirements

Statistical tests are useful but are not primary correctness evidence.

The test hierarchy should be:

```text
exact algorithm vectors
exact consumption semantics
invariants
type-system checks
boundary/error behavior
cross-platform replay
resource behavior
then statistical sanity
```

## 30.1 Generator conformance tests

Create a small deterministic test generator that yields a programmed sequence of words.

Use it to test every default `Generator` method independently of xoshiro.

This is important.

Otherwise a bug in xoshiro and a compensating bug in a derived method can accidentally make an end-to-end test pass.

The mock generator should allow exact assertions about:

```text
how many words were consumed
which words were consumed
rejection retries
byte extraction
```

## 30.2 Xoshiro golden vectors

Freeze authoritative vectors for:

```text
seed 0
seed 1
seed 42
seed 2^64 - 1
```

Test a substantial prefix, not only the first word.

Expected values must come from an independent reference implementation/reference vector, not the implementation under test.

## 30.3 State word invariants

After every transition in a test hook or reference model:

```text
0 <= logical state word < 2^64
```

and state is never the all-zero xoshiro state.

## 30.4 Seed validation

Test:

```text
-1
0
1
2^64 - 1
2^64
2^64 + 1
huge Int
```

## 30.5 `nextBool`

Test exact high-bit mapping and exactly one-word consumption.

## 30.6 `nextFloat`

Test:

```text
range [0,1)
0 word
max word
representative words
exact output bits
one-word consumption
```

## 30.7 `nextInt`

Required bound set:

```text
1
2
3
power of two
power of two - 1
power of two + 1
2^64 - 1
2^64
2^64 + 1
10^100
10^1000
```

Use mock streams to force:

```text
first candidate rejected
second candidate accepted
multiple rejected candidates
```

Test arbitrary negative lower bounds in range form.

## 30.8 Deterministic bytes

Given:

```text
0x0123456789abcdef
```

the first eight bytes must be exactly:

```text
01 23 45 67 89 ab cd ef
```

if the recommended logical big-endian order is adopted.

Test counts:

```text
0
1
7
8
9
15
16
17
large
```

and subsequent stream state.

## 30.9 `choose`

Test:

```text
empty -> None, no consumption
singleton -> same item, no consumption
multiple -> exact chosen index
generic T preserved
```

## 30.10 `shuffle`

Test:

```text
empty
one item
two items
distinct list
duplicate-value list
```

Required properties:

```text
same size
same multiset
same seed -> same output
golden permutation
correct sample consumption
```

## 30.11 `UniformInt`

Test construction validation and exact delegation behavior, including huge ranges.

## 30.12 `Bernoulli`

Test:

```text
NaN rejected
negative rejected
>1 rejected
0 -> false/no consumption
1 -> true/no consumption
0.5 exact mock behavior
```

## 30.13 SecureRandom

Do not assert fixed bytes.

Test:

```text
system returns SecureRandom
bytes(0) -> empty
bytes(n).size == n
fill mutates destination
negative count rejected before allocation
host failure propagates
no fallback
no result on failure
```

Add an injectable/test-only host entropy seam if necessary, without expanding public language API.

## 30.14 Type-system tests

Required negative/positive cases:

```text
Xoshiro256PlusPlus |= Generator          PASS
SecureRandom |= Generator                FAIL
UniformInt |= Distribution               PASS
UniformInt::Output == Int                PASS
Bernoulli::Output == Bool                PASS
choose<T> preserves T                    PASS
```

## 30.15 Cross-platform replay

Run deterministic vectors in CI on every supported architecture/OS combination.

This is a release requirement for a named deterministic generator.

## 30.16 Statistical sanity

Existing:

```text
benchmarks/math/monte_carlo.ph
benchmarks/math/random_walk.ph
```

currently contain a source MINSTD generator.

They should become useful integration/benchmark migration targets after `random` lands.

They can validate broad stochastic sanity such as:

```text
mean near 1/2
variance near 1/12
fair coin rate
random-walk diffusion
```

but they do not replace exact vectors.

---

# 31. Performance requirements

## 31.1 Source implementation must be measured

The package should explicitly benchmark:

```text
nextU64
nextFloat
nextInt small bound
nextInt 64-bit bound
nextInt 256-bit bound
bytes(1 KiB)
shuffle 100 elements
shuffle 10k elements
```

## 31.2 BigInt allocation profile

Track:

```text
allocations per nextU64
allocations per nextFloat
GC cycles under long streams
```

because 64-bit unsigned values may spill out of the immediate Int representation.

## 31.3 Native optimization threshold

A native deterministic backend should be considered only if:

1. source semantics are fully frozen;
2. source implementation is measurably inadequate;
3. a source-level/general numeric optimization is not sufficient;
4. the native backend can be proved bit-identical to the source contract;
5. the source declaration remains semantic authority.

This would be an optimization project, not part of the initial semantic design.

---

# 32. Error and validation requirements

Use `ArgumentError` for invalid caller-provided domains:

```text
seed out of range
nextInt upper <= 0
range upper <= lower
negative byte count
UniformInt invalid interval
Bernoulli invalid probability
```

Do not create a broad:

```text
RandomError
```

for argument mistakes.

Secure host failure should have a distinct narrow system-failure representation, likely `EntropyError` if no canonical existing host error is suitable.

Numeric resource exhaustion remains:

```text
#numericLimit
```

not `ArgumentError`.

---

# 33. Security documentation requirements

The package documentation must include strong non-ambiguous wording.

For `Xoshiro256PlusPlus`:

> Deterministic and reproducible. Not cryptographically secure. Do not use for passwords, authentication tokens, session secrets, cryptographic keys, security nonces, or any value whose security depends on unpredictability.

For `Generator.bytes`:

> These bytes are deterministic pseudorandom output and are not secure random bytes.

For `SecureRandom`:

> Obtains cryptographically suitable random bytes from the operating system. It is nondeterministic and is not seedable or replayable.

Do not call `SecureRandom` "true random".

Do not claim raw hardware entropy.

---

# 34. API non-goals for the first complete package

Defer:

```text
global/default RNG
implicit system seeding
seed-from-time
seed-from-Bytes
PRNG state export/import
clone/snapshot/restore
jump/longJump
stream split
parallel generators
counter-based generators
Normal
Gamma
Beta
Poisson
weighted sampling
reservoir sampling
random Iterable selection
UUID generation
token/password string encoding
random alphanumeric strings
cryptographic key generation
cryptographic nonce construction
secure integer arithmetic APIs
random permutation object
sampling without replacement API
```

These are legitimate later features, but each has semantics beyond "produce some random-looking value."

---

# 35. Important design rejections

The following designs should be rejected as architecturally inconsistent with Telos.

## 35.1 Global mutable RNG

Rejected because state and dependency are hidden.

## 35.2 Auto-seeded `Xoshiro256PlusPlus.new()`

Rejected because deterministic generator construction should be explicit and reproducible.

## 35.3 `SecureRandom : Generator`

Rejected because it erases security authority.

## 35.4 `Generator` implemented by secure host random source

Rejected even if the methods can technically be supplied.

## 35.5 `% upper` bounded sampling

Rejected because it is biased.

## 35.6 64-bit-only public `nextInt`

Rejected because Telos `Int` is arbitrary precision.

## 35.7 Generic `Iterable` choice

Rejected because uniformity requires assumptions not expressed by `Iterable`.

## 35.8 Native xoshiro from the first commit

Rejected because the algorithm is source-expressible and should dogfood Telos.

## 35.9 Host-endian deterministic bytes

Rejected because replay must be architecture independent.

## 35.10 Security fallback

Rejected categorically:

```text
OS secure random failure
-> deterministic pseudorandom fallback
```

must never occur.

## 35.11 One catch-all `random()` API

Rejected because result domain, state source, and security semantics become ambiguous.

---

# 36. General language/runtime gaps this package may expose

These should be recorded, not automatically implemented.

## 36.1 Fixed-width word efficiency

The xoshiro performance question may reveal demand for a general fixed-width integer abstraction or optimized modular bit operations.

Do not preemptively create `UInt64`.

## 36.2 Mutation capabilities

Generators are a compelling example for a future type-level mutation capability model.

Not a blocker.

## 36.3 Native host-error minting

`SecureRandom` may expose a missing general path for native host failures to become canonical catchable source Error values.

This **may be a real prerequisite** for R2.

## 36.4 Generic mutable random-access collections

Could later generalize shuffle.

Not needed for List-based v1.

## 36.5 Stream splitting / parallel reproducibility

Needs independent design.

Not a blocker.

---

# 37. Recommended completeness boundary

A `universe.random` release should be considered **functionally complete for the initial standard-library slice** when it provides all of:

```text
Generator
    nextU64
    nextBool
    nextFloat
    nextInt(below:)
    nextInt(from:below:)
    fill(Bytes)
    bytes(count)
    choose(List<T>)
    shuffle(List<T>)

Xoshiro256PlusPlus
    explicit seed(Int)
    standard SplitMix64 expansion
    frozen deterministic replay

Distribution
    associated Output
    typed sample(using:)

UniformInt
Bernoulli

SecureRandom
    explicit system capability
    fill(Bytes)
    bytes(count)
    host failure propagation
    no Generator conformance
```

with:

```text
full exact vector tests
type-system tests
resource tests
effect metadata
cross-platform replay tests
secure failure tests
benchmarks
documentation
```

That is a substantial and coherent package. It does not need a giant distribution catalog to be "complete" in the architectural sense.

---

# 38. Required acceptance invariants

The final implementation should be audited against the following explicit invariants.

```text
RND-ARCH-01
No hidden global/default deterministic generator exists.

RND-ARCH-02
SecureRandom does not conform to Generator.

RND-ARCH-03
Deterministic generator construction never implicitly reads system entropy.

RND-ARCH-04
SecureRandom never falls back to deterministic generation.

RND-ARCH-05
The deterministic implementation adds no random-specific VM opcode,
value tag, heap variant, parser syntax, or checker special case.

RND-GEN-01
nextU64 returns an exact Int in [0, 2^64).

RND-GEN-02
Xoshiro256PlusPlus.seed accepts exactly [0, 2^64).

RND-GEN-03
The algorithm stream is stable for a named generator type.

RND-GEN-04
All 64-bit word semantics are explicit and host-width independent.

RND-GEN-05
nextBool consumes exactly one word.

RND-GEN-06
nextFloat consumes exactly one word and yields [0,1).

RND-GEN-07
nextInt is unbiased.

RND-GEN-08
nextInt supports arbitrary-precision Int bounds.

RND-GEN-09
nextInt(below: 1) returns 0 without consuming generator state.

RND-BYTES-01
Deterministic byte order is specified and architecture independent.

RND-BYTES-02
No hidden partial-word byte cache exists.

RND-COLL-01
choose(empty) returns None with zero consumption.

RND-COLL-02
choose preserves element type T.

RND-COLL-03
shuffle uses unbiased Fisher–Yates and mutates in place.

RND-DIST-01
Distribution carries an associated Output type.

RND-DIST-02
UniformInt validates interval state exactly once at construction.

RND-DIST-03
Bernoulli rejects NaN and out-of-range probabilities.

RND-DIST-04
Degenerate Bernoulli distributions consume no generator state.

RND-SEC-01
SecureRandom.system yields an explicit security capability value.

RND-SEC-02
SecureRandom obtains bytes only from the configured OS secure source.

RND-SEC-03
Host secure-random failure is observable and catchable/structured.

RND-SEC-04
SecureRandom.bytes never returns a partially fabricated result.

RND-EFF-01
Deterministic Generator state transition is mutation, not nondeterminism.

RND-EFF-02
Secure host fill is marked with nondeterminism and mutation,
plus blocking/IO according to project effect policy.

RND-RES-01
Arbitrary-precision bounded sampling respects NumericPolicy.

RND-RES-02
Invalid large requests fail through established resource policy,
not host panic/OOM-by-design.

RND-TYPE-01
Public random APIs do not erase generic result types to Dynamic.

RND-TYPE-02
SecureRandom cannot satisfy APIs constrained to Generator.

RND-TEST-01
Exact golden vectors are primary PRNG correctness evidence.

RND-TEST-02
Statistical tests are supplemental only.

RND-TEST-03
Deterministic golden tests are cross-platform.
```

---

# 39. Requirements-to-repository mapping

| Requirement area | Existing repository substrate |
|---|---|
| arbitrary-precision word arithmetic | `scalar/number.ph`, `primitive/int.rs`, `primitive/number.rs` |
| bit width / shifts | `Int.bitLength`, `<<`, `>>`, `&`, `|`, `^` |
| exact Int resource limits | `VM::numeric_policy`, `RuntimeError::NumericLimit` |
| Float conversion | `Float.new` |
| mutable byte buffer | `collections/bytes.ph`, `primitive/bytes.rs` |
| generic mutable list | `collections/list.ph`, `primitive/list.rs` |
| optional choice result | `Option<T>` |
| trait associated types/defaults | `collections/iterator_laws.ph` |
| private validated data | `time/duration.ph`, `time/offset.ph`, calendar types |
| source-owned native capability pattern | `time/clock.ph`, `primitive/time.rs` |
| native effect metadata | `phalcom-native-meta::NativeEffect` |
| argument validation | `ArgumentError` |
| deterministic simulation examples | `benchmarks/math/monte_carlo.ph`, `random_walk.ph` |
| secure host RNG dependency candidate | transitive `getrandom 0.3.3` in `Cargo.lock` |

---

# 40. Final recommendation

The strongest `random` design for Telos is not the largest API.

It is the design that makes different kinds of randomness structurally explicit:

```text
simulation/test/procedural randomness
    = explicit mutable deterministic generator value

sampling policy
    = typed immutable Distribution value

security randomness
    = explicit separate host capability
```

This preserves Telos's key development philosophy:

> **Make authority, state, typing, and semantic identity visible in ordinary source code. Use the language's own abstractions first. Reserve native code for the actual host boundary.**

The resulting developer model is concise:

```telos
let rng = random.Xoshiro256PlusPlus.seed(42)
let roll = random.UniformInt.from(1, below: 7).sample(using: rng)

let secure = random.SecureRandom.system
let token = secure.bytes(32)
```

but the semantics remain rigorous:

```text
replayable vs nondeterministic
non-secure vs secure
mutable stream vs immutable distribution
exact Int vs binary64 Float
source algorithm vs host-native authority
```

That combination makes `universe.random` an excellent standard-library showcase for Telos rather than a thin port of another language's RNG API.
