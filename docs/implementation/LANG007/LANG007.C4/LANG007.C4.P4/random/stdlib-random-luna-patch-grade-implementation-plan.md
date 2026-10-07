---
kind: implementation-plan
title: "universe.random — Luna Patch-Grade Implementation Plan"
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
branch: main
branch_state_observed:
  base: main
  ahead: 4
  behind: 0
prepared: 2026-10-07
source_requirements: stdlib-random-requirements-analysis.md
scope:
  - universe.random
  - Generator
  - Xoshiro256PlusPlus
  - deterministic sampling helpers
  - Distribution
  - UniformInt
  - Bernoulli
  - SecureRandom
  - secure host random-byte primitive
excludes:
  - global/default RNG
  - implicit seeding
  - complex continuous distributions
  - PRNG state serialization
  - jump/split/parallel stream APIs
  - cryptographic protocol construction
---

# `universe.random`
## Guided Luna Patch-Grade Implementation Plan

This plan translates the approved `universe.random` requirements analysis into an implementation sequence suitable for a Luna/implementer agent working directly in `aureat/phalcom-lang`.

The plan is deliberately architecture-first. `random` is not to be implemented as a pile of native helpers. The slice should become a strong example of Telos's own design philosophy:

```text
source-authored semantics
+
explicit mutable state
+
typed protocols
+
exact Int arithmetic
+
ordinary collections
+
narrow host-native authority
```

The intended final architecture is:

```text
universe.random
│
├── Generator
│      ├── nextU64                         required primitive protocol
│      ├── nextBool                        default source method
│      ├── nextFloat                       default source method
│      ├── nextInt(below:)                 default source method
│      ├── nextInt(from:below:)            default source method
│      ├── fill(Bytes)                     default source method
│      ├── bytes(Int)                      default source method
│      ├── choose<T>(List<T>)              default source method
│      └── shuffle<T>(List<T>)             default source method
│
├── Xoshiro256PlusPlus
│      ├── explicit mutable state
│      ├── SplitMix64 seeding
│      ├── exact 64-bit wrapping over Int
│      └── implements Generator
│
├── Distribution
│      └── associated type Output
│
├── UniformInt : Distribution<Output = Int>
│
├── Bernoulli : Distribution<Output = Bool>
│
└── SecureRandom
       ├── explicit system capability value
       ├── fill(Bytes)
       ├── bytes(Int)
       ├── does NOT implement Generator
       └── narrow native OS randomness primitive
```

---

# 0. Executor contract

## 0.1 Branch discipline

Work from the intended `stdlib-random-math` branch/worktree.

Before editing:

```sh
git status --short
git rev-parse HEAD
git merge-base HEAD origin/main
git diff --check
```

The branch was observed as:

```text
main..stdlib-random-math
ahead:  4
behind: 0
```

The branch currently also carries incomplete/corrective `math` work. Preserve it.

Do not reset, rewrite, or discard unrelated math/static-constant changes.

## 0.2 Shared-file conflict warning

`random` will touch files also relevant to the math corrective work:

```text
phalcom-modules/src/builtin.rs
phalcom-native-meta/src/universe.rs
phalcom-native-surface/src/generated.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/Cargo.toml
```

Before editing any shared file, re-read the live file.

If the math slice has advanced, merge the architectural intent rather than applying stale line-oriented edits.

## 0.3 Scope rule

The implementation owns:

```text
universe.random
+
the minimal native/runtime plumbing required by SecureRandom
```

It does **not** own a general redesign of:

```text
numeric tower
mutation/capability typing
effect system
error hierarchy
collection protocols
VM hashing entropy
thread/fiber scheduler
```

If random exposes a real general gap, fix only the smallest correct general seam or stop for architecture review.

## 0.4 No-special-case rule

Do not add any random-specific:

```text
parser syntax
checker branch
VM opcode
Value tag
heap object variant
bytecode instruction
compiler intrinsic
package-name special case
```

All deterministic functionality must be ordinary Telos source.

## 0.5 Native-boundary rule

Native Rust is permitted initially only for:

```text
OS secure random-byte acquisition
```

Everything else stays source-authored unless a measured blocker proves source implementation non-viable.

---

# 1. Required architectural invariants

Treat these as release blockers.

## 1.1 State and authority

```text
RND-INV-001
No package-global, thread-local, fiber-local, or implicit default Generator exists.

RND-INV-002
Xoshiro256PlusPlus construction is explicit and deterministic.

RND-INV-003
SecureRandom is a separate nominal capability.

RND-INV-004
SecureRandom does not implement Generator.

RND-INV-005
SecureRandom never falls back to deterministic output.

RND-INV-006
VM-private hash/security seeds remain isolated from public random APIs.
```

## 1.2 Generator semantics

```text
RND-INV-010
nextU64 returns 0 <= x < 2^64.

RND-INV-011
Generator has exactly one primitive entropy/state requirement: nextU64.

RND-INV-012
All derived deterministic methods are source-authored default methods.

RND-INV-013
nextBool consumes exactly one nextU64 word.

RND-INV-014
nextFloat consumes exactly one nextU64 word.

RND-INV-015
nextFloat returns 0.0 <= x < 1.0.

RND-INV-016
nextInt is unbiased.

RND-INV-017
nextInt supports arbitrary-precision Int bounds.

RND-INV-018
nextInt(below: 1) returns 0 without consuming state.

RND-INV-019
All deterministic consumption behavior is frozen and testable.
```

## 1.3 Xoshiro semantics

```text
RND-INV-020
The algorithm is standard xoshiro256++.

RND-INV-021
The seed expander is frozen SplitMix64.

RND-INV-022
Seed domain is exactly 0 <= seed < 2^64.

RND-INV-023
No seed modulo reduction is performed.

RND-INV-024
Every logical word obeys unsigned 64-bit modulo arithmetic.

RND-INV-025
Host word size and endianness cannot affect output.

RND-INV-026
The named generator's stream remains stable across supported platforms.
```

## 1.4 Bytes/collections

```text
RND-INV-030
Deterministic byte order is explicitly defined.

RND-INV-031
No hidden byte reservoir exists.

RND-INV-032
choose(empty) returns None and consumes no state.

RND-INV-033
choose<T> preserves T.

RND-INV-034
shuffle<T> mutates List<T> in place using unbiased Fisher–Yates.
```

## 1.5 Distribution typing

```text
RND-INV-040
Distribution has associated type Output.

RND-INV-041
UniformInt::Output == Int.

RND-INV-042
Bernoulli::Output == Bool.

RND-INV-043
Distribution values are validated immutable descriptors.

RND-INV-044
No public distribution sample path returns Dynamic merely to avoid typing work.
```

## 1.6 Secure randomness

```text
RND-INV-050
SecureRandom.system returns an explicit capability object.

RND-INV-051
SecureRandom obtains bytes from the OS secure randomness facility.

RND-INV-052
Host failure propagates as a source-observable structured failure.

RND-INV-053
SecureRandom.bytes(count) never returns a partial result.

RND-INV-054
SecureRandom native implementation does not expose VM hash seeds.

RND-INV-055
Secure host randomness is marked nondeterministic in native effect metadata.
```

## 1.7 Effects

```text
RND-INV-060
Deterministic generator advancement is mutation, not nondeterminism.

RND-INV-061
Secure host fill carries mutation + nondeterminism.

RND-INV-062
Blocking/IO metadata follows the project's existing effect taxonomy.

RND-INV-063
Native effect metadata is explicit, not Unknown.
```

## 1.8 Resources

```text
RND-INV-070
Arbitrary-precision sampling respects NumericPolicy.

RND-INV-071
Candidate assembly does not require avoidable >target-width intermediate Ints.

RND-INV-072
Negative byte counts fail before allocation.

RND-INV-073
No resource failure is converted into truncated/randomly biased output.
```

---

# 2. Target package topology

Create:

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

Native:

```text
phalcom-core/src/primitive/random.rs
```

## 2.1 Visibility posture

`package.ph` is the public facade.

The implementation child modules should be registered in the Universe graph so the parent can import them, but the facade should not expose them as public traversal paths unless later architecture explicitly requests that.

Target public names:

```text
universe.random.Generator
universe.random.Distribution
universe.random.Xoshiro256PlusPlus
universe.random.UniformInt
universe.random.Bernoulli
universe.random.SecureRandom
universe.random.EntropyError          if adopted
```

Avoid requiring public users to import:

```text
universe.random.generator.Generator
universe.random.xoshiro256pp.Xoshiro256PlusPlus
```

## 2.2 `package.ph`

Conceptual final shape:

```telos
@!documentation(
  "Deterministic pseudorandom generators, typed distributions, " +
  "and secure system randomness."
)

from .generator import Generator
from .distribution import Distribution
from .xoshiro256pp import Xoshiro256PlusPlus
from .uniform_int import UniformInt
from .bernoulli import Bernoulli
from .secure import SecureRandom, EntropyError

export (
  Generator,
  Distribution,
  Xoshiro256PlusPlus,
  UniformInt,
  Bernoulli,
  SecureRandom,
  EntropyError,
)
```

If the final secure failure design reuses an existing canonical host error, omit `EntropyError`.

Do not add `expose` declarations for the children unless public path traversal is intentionally required.

---

# 3. Execution DAG

Implement in this order:

```text
R0  Baseline / syntax probes / reference vectors
 │
 ▼
R1  Universe random child topology
 │
 ▼
R2  Generator trait + scripted test generator
 │
 ▼
R3  Xoshiro256PlusPlus + SplitMix64 core
 │
 ├──────────────┐
 ▼              │
R4 nextBool     │
R4 nextFloat    │
 │              │
 ▼              │
R5 nextInt      │
 │              │
 ▼              │
R6 bytes/fill   │
 │              │
 ▼              │
R7 choose/shuffle
 │
 ▼
R8 Distribution + UniformInt + Bernoulli
 │
 ▼
R9 SecureRandom source API + EntropyError seam
 │
 ▼
R10 Native getrandom integration + UniverseKey
 │
 ▼
R11 Effect/source/native metadata verification
 │
 ▼
R12 Comprehensive deterministic and secure tests
 │
 ▼
R13 Performance/resource profiling
 │
 ▼
R14 Generated native surface + docs/lifecycle
 │
 ▼
R15 Final verification
```

Do not begin the secure native boundary before the deterministic source architecture is green. This keeps native failure/debugging separate from PRNG correctness.

---

# 4. R0 — Baseline, syntax proof, and vector freeze

## Purpose

Establish exact current branch behavior and remove syntax ambiguity before source implementation.

## R0.1 Baseline

Run:

```sh
git status --short
git diff --check

RUSTFLAGS='' cargo check -p phalcom-core
RUSTFLAGS='' cargo check -p phalcom-modules
RUSTFLAGS='' cargo check -p phalcom-native-meta
```

Record failures already caused by the in-progress math slice separately.

Do not attribute pre-existing failures to random.

## R0.2 Confirm language syntax

Before authoring the actual files, verify with the parser/checker:

1. trait default method with method-local generic:
   ```telos
   choose<T>(_ values: List<T>) -> Option<T>
   ```

2. trait method with associated return:
   ```telos
   trait Distribution {
     type Output
     sample<G>(using generator: G) -> Self::Output
       where G |= Generator
   }
   ```

3. labeled selectors:
   ```telos
   nextInt(below upper: Int)
   nextInt(from lower: Int, below upper: Int)
   ```

4. generic trait implementation:
   ```telos
   impl Distribution for UniformInt { ... }
   ```

5. `data ... private(...)` constructor behavior.

Use minimal parser/semantic fixtures rather than guessing spelling.

If a desired syntax is not accepted, preserve the semantic design and adapt to the canonical current syntax. Do not weaken types.

## R0.3 Freeze reference vectors outside the implementation

Before writing `xoshiro256pp.ph`, prepare authoritative expected values for:

```text
seed 0
seed 1
seed 42
seed 2^64 - 1
```

using an independently checked implementation of:

```text
SplitMix64 seed expansion
xoshiro256++ output/transition
```

Store expected words directly in tests.

The implementation under test must never generate its own expected vectors.

## R0.4 Decide exact replay details now

Freeze these before implementation:

```text
seed expansion:
    standard SplitMix64
    increment-before-mix

nextBool:
    bit 63
    one complete word

nextFloat:
    top 53 bits
    word >> 11
    divide by 2^53

multiword nextInt:
    earlier generated words are higher-order chunks

partial first chunk:
    keep the low `headBits` of the first generated word
    equivalent to masking away excess high bits after concatenation

bytes:
    big-endian logical extraction from each word

partial byte word:
    use leading bytes
    discard remainder

shuffle:
    descending Fisher–Yates

Bernoulli 0/1:
    no generator consumption
```

If a different choice is made, update the requirements record and golden vectors before coding.

## R0 exit criterion

A small syntax probe suite passes and independent xoshiro vectors are frozen.

---

# 5. R1 — Register the random source module graph

## Files

```text
phalcom-modules/src/builtin.rs
phalcom-core/core/universe/src/random/package.ph
NEW random child files
```

## R1.1 Extend `UNIVERSE_NODES`

Change the `random` package node from:

```rust
UniverseNodeSpec {
    path: &["random"],
    kind: ModuleKind::Package,
    children: &[],
}
```

to children:

```text
generator
xoshiro256pp
distribution
uniform_int
bernoulli
secure
```

Add one `Module` node for each child.

Provider registration is physical topology.

It does **not** imply public exposure.

## R1.2 Extend `UniverseSourceProvider::source_text`

Add exact `include_str!` arms for:

```text
random/generator.ph
random/xoshiro256pp.ph
random/distribution.ph
random/uniform_int.ph
random/bernoulli.ph
random/secure.ph
```

## R1.3 Add parseable skeletons

Create the six child files with minimal declarations sufficient to prove:

```text
provider discovery
relative import resolution
parent facade imports
source indexing
bootstrap dependency order
```

Do not implement logic yet.

## R1.4 Facade import test

Add a small fixture proving:

```telos
import universe.random as random
```

resolves.

Add negative traversal tests if hidden-child access is supposed to be denied externally:

```telos
import universe.random.xoshiro256pp
```

should fail according to current package exposure semantics.

## R1 exit criterion

All random child sources are discoverable internally, facade imports resolve, and the public path remains `universe.random`.

---

# 6. R2 — Implement the `Generator` protocol first

## File

```text
phalcom-core/core/universe/src/random/generator.ph
```

## Purpose

Prove the generic abstraction independently of xoshiro.

## R2.1 Trait declaration

Target conceptual shape:

```telos
trait Generator {
  nextU64 -> Int

  nextBool -> Bool { ... }

  nextFloat -> Float { ... }

  nextInt(below upper: Int) -> Int { ... }

  nextInt(from lower: Int, below upper: Int) -> Int { ... }

  fill(_ bytes: Bytes) -> Unit { ... }

  bytes(_ count: Int) -> Bytes { ... }

  choose<T>(_ values: List<T>) -> Option<T> { ... }

  shuffle<T>(_ values: List<T>) -> Unit { ... }
}
```

Only `nextU64` is abstract.

## R2.2 Imports

Use canonical public collection/numeric types.

Avoid importing implementation-private collection primitives.

## R2.3 Add a scripted test generator

Create a test-only source class implementing `Generator`:

```telos
class SequenceGenerator {
  mut _words: List<Int>
  mut _index: Int

  ...
}

impl Generator for SequenceGenerator {
  nextU64 -> Int {
    ...
  }
}
```

It should emit a fixed word sequence and expose a test-only consumption count.

This test double is essential because it lets every default method be verified independently from xoshiro.

Do not make it public stdlib API.

## R2 exit criterion

A custom generator needs to implement only `nextU64`; inherited default methods type-check.

---

# 7. R3 — Implement source-authored Xoshiro256PlusPlus

## File

```text
phalcom-core/core/universe/src/random/xoshiro256pp.ph
```

## R3.1 Representation

Use mutable private `Int` fields:

```telos
class Xoshiro256PlusPlus {
  mut _s0: Int
  mut _s1: Int
  mut _s2: Int
  mut _s3: Int

  @constructor
  private create(
    _ s0: Int,
    _ s1: Int,
    _ s2: Int,
    _ s3: Int
  ) {
    _s0 = s0
    _s1 = s1
    _s2 = s2
    _s3 = s3
  }

  @class
  seed(_ seed: Int) -> Xoshiro256PlusPlus { ... }
}
```

Do not expose state accessors.

Do not add structural equality.

## R3.2 Constants

Prefer module-private compile-time constants:

```telos
const WORD_BITS: Int = 64
const WORD_MODULUS: Int = 1 << 64
const WORD_MASK: Int = WORD_MODULUS - 1

const SPLITMIX_GAMMA: Int = 0x9E3779B97F4A7C15
const SPLITMIX_MUL1: Int = 0xBF58476D1CE4E5B9
const SPLITMIX_MUL2: Int = 0x94D049BB133111EB
```

These are implementation constants, not exports.

If branch-local static LargeInt materialization is still unstable because of the math work, stop and resolve the general static-constant blocker rather than replacing constants with mutable runtime bindings.

## R3.3 Word helpers

Keep helpers private and source-authored.

Target:

```telos
@class
private wrap64(_ value: Int) -> Int {
  value & WORD_MASK
}

@class
private rotl64(_ value: Int, _ shift: Int) -> Int {
  (
    (value << shift)
    | (value >> (WORD_BITS - shift))
  ) & WORD_MASK
}
```

All inputs to `rotl64` are nonnegative logical 64-bit words.

No signed right-shift ambiguity should enter the algorithm.

## R3.4 SplitMix64 mixer

Implement:

```text
x = (x + gamma) mod 2^64
z = x
z = ((z xor (z >> 30)) * mul1) mod 2^64
z = ((z xor (z >> 27)) * mul2) mod 2^64
z = (z xor (z >> 31)) mod 2^64
```

Recommended source decomposition:

```telos
@class
private splitMixValue(_ state: Int) -> Int {
  let mut z = state
  z = ((z ^ (z >> 30)) * SPLITMIX_MUL1) & WORD_MASK
  z = ((z ^ (z >> 27)) * SPLITMIX_MUL2) & WORD_MASK
  (z ^ (z >> 31)) & WORD_MASK
}
```

`seed` advances the SplitMix state four times:

```text
x = wrap64(seed + gamma) -> s0
x = wrap64(x + gamma)    -> s1
x = wrap64(x + gamma)    -> s2
x = wrap64(x + gamma)    -> s3
```

and mixes each advanced `x`.

## R3.5 Seed validation

Before state construction:

```text
seed < 0      -> ArgumentError
seed >= 2^64  -> ArgumentError
```

Do not reduce modulo `2^64`.

Use the house source idiom:

```telos
throw ArgumentError.new(...)
```

or `.raise()` consistently with nearby Universe source.

## R3.6 `Generator` implementation

Implement only:

```telos
impl Generator for Xoshiro256PlusPlus {
  nextU64 -> Int {
    ...
  }
}
```

Algorithm order:

```text
result = rotl64((s0 + s3) & MASK64, 23)
result = (result + s0) & MASK64

t = (s1 << 17) & MASK64

s2 = s2 xor s0
s3 = s3 xor s1
s1 = s1 xor s2
s0 = s0 xor s3

s2 = s2 xor t
s3 = rotl64(s3, 45)

return result
```

Use local snapshots or mutation order exactly matching the reference algorithm.

Do not "simplify" the transition algebra unless golden vectors prove equivalence.

## R3.7 Invariant tests

For each tested transition:

```text
0 <= output < 2^64
0 <= each state word < 2^64
```

The SplitMix expansion should not produce the all-zero four-word state.

## R3.8 Golden vectors

Run the frozen R0 vector suite.

No derived methods yet.

## R3.9 Performance checkpoint

Run a preliminary long `nextU64` loop and record:

```text
throughput
allocation profile if available
GC count
```

Do not optimize yet unless the implementation is pathologically unusable.

## R3 exit criterion

`nextU64` matches authoritative vectors exactly.

---

# 8. R4 — Implement `nextBool` and `nextFloat`

## File

```text
random/generator.ph
```

## R4.1 `nextBool`

Implement:

```text
(self.nextU64 >> 63) != 0
```

Required:

```text
exactly one word consumed
```

No bit reservoir.

## R4.2 `nextFloat`

Implement:

```text
word = nextU64
mantissa = word >> 11
Float.new(mantissa) / 9007199254740992.0
```

The denominator is:

```text
2^53
```

Use a representation that is exactly binary64.

Do not convert the full 64-bit word to Float.

## R4.3 Exact tests

With `SequenceGenerator`, test words:

```text
0
1
1 << 63
0xffffffffffffffff
representative mixed words
```

Assert:

```text
0.0 <= result < 1.0
exact Float bits
exactly one consumed word
```

## R4 exit criterion

Default derived behavior is correct without relying on xoshiro internals.

---

# 9. R5 — Implement arbitrary-precision unbiased `nextInt`

This is the most subtle deterministic helper.

## R5.1 Single-bound validation

```telos
nextInt(below upper: Int) -> Int
```

Rules:

```text
upper <= 0 -> ArgumentError
upper == 1 -> 0, consume zero words
```

## R5.2 Width

For `upper > 1`:

```text
bits = (upper - 1).bitLength
wordCount = (bits + 63) ~/ 64
headBits = bits - ((wordCount - 1) * 64)
```

## R5.3 Resource-safe assembly

Do **not** assemble `wordCount * 64` bits and then mask.

That can transiently construct an Int up to 63 bits wider than the actual requested domain and may spuriously cross `NumericPolicy.max_integer_bits`.

Instead construct exactly at target width.

Algorithm:

```text
first = nextU64

if headBits < 64:
    firstMask = (1 << headBits) - 1
    candidate = first & firstMask
else:
    candidate = first

for each remaining word:
    candidate = (candidate << 64) | nextU64
```

This preserves the frozen rule:

```text
earlier words are higher-order chunks
excess high bits are masked from the first chunk
```

The candidate never needs more than `bits` significant bits.

## R5.4 Rejection

```text
if candidate < upper:
    return candidate
else:
    restart from a fresh candidate
```

Do not recycle rejected bits.

Do not use modulo reduction.

## R5.5 Range form

```telos
nextInt(from lower: Int, below upper: Int) -> Int
```

Validate:

```text
upper > lower
```

Then:

```text
width = upper - lower
return lower + nextInt(below: width)
```

## R5.6 Numeric policy

All Int arithmetic must use ordinary operators so existing numeric-limit behavior remains authoritative.

Do not catch `#numericLimit`.

## R5.7 Forced rejection tests

The scripted generator must cover:

```text
candidate rejected once
candidate rejected repeatedly
candidate accepted
```

for non-power-of-two bounds.

## R5.8 Bound matrix

Required:

```text
1
2
3
7
8
9
2^32 - 1
2^32
2^32 + 1
2^64 - 1
2^64
2^64 + 1
10^100
10^1000
```

Range form:

```text
negative-only
positive-only
cross-zero
huge negative -> huge positive
```

## R5 exit criterion

Exact structural tests prove rejection sampling and arbitrary-precision bounds.

---

# 10. R6 — Deterministic bytes

## R6.1 `fill(Bytes)`

For each word:

```text
byte0 = (word >> 56) & 0xff
byte1 = (word >> 48) & 0xff
...
byte7 = word & 0xff
```

Write bytes through the existing public `Bytes` mutation API.

Do not add random-specific native byte mutation.

## R6.2 Partial final word

If fewer than eight bytes remain:

```text
consume one fresh word
write the leading bytes only
discard the rest
```

No byte cache.

## R6.3 `bytes(count)`

```text
count < 0 -> ArgumentError
count == 0 -> Bytes.new(0), consume zero words
otherwise:
    out = Bytes.new(count)
    self.fill(out)
    return out
```

Let `Bytes.new` own actual allocation limits.

## R6.4 Golden bytes

For scripted word:

```text
0x0123456789abcdef
```

expect:

```text
01 23 45 67 89 ab cd ef
```

Test:

```text
0,1,7,8,9,15,16,17
```

and subsequent generator consumption.

## R6 exit criterion

Byte output is architecture-independent and stream consumption is frozen.

---

# 11. R7 — `choose` and `shuffle`

## R7.1 `choose<T>`

Implement:

```text
size == 0 -> None
size == 1 -> Some(values.at(0))
otherwise -> Some(values.at(nextInt(below: size)))
```

The singleton optimization is semantically equivalent to `nextInt(below: 1)` and explicitly preserves zero consumption.

Public type:

```text
Option<T>
```

not `Option<Dynamic>`.

## R7.2 `shuffle<T>`

Use descending Fisher–Yates:

```text
if size <= 1:
    return ()

i = size - 1
while i > 0:
    j = nextInt(below: i + 1)
    values.swap(first: i, second: j)
    i = i - 1
```

Use the actual current `List.swap` call syntax and handle its `Result` with the standard `Result` API.

Indices are internally proven valid; any unexpected failure is an invariant breach, not random-domain behavior.

If using `.unwrap` is the established source idiom, use it.

Do not manually reach `rawSet`.

## R7.3 Tests

`choose`:

```text
empty
singleton
first index
last index
generic class value
```

`shuffle`:

```text
empty
singleton
two values
five distinct values
duplicates
golden permutation
same seed -> same permutation
```

Also verify output multiset/size.

## R7 exit criterion

Collection helpers are typed, unbiased, source-authored, and deterministic.

---

# 12. R8 — Typed distributions

## Files

```text
random/distribution.ph
random/uniform_int.ph
random/bernoulli.ph
```

## R8.1 `Distribution`

Target:

```telos
trait Distribution {
  type Output

  sample<G>(using generator: G) -> Self::Output
    where G |= Generator
}
```

If current syntax prefers a different placement of `where`, adapt only syntax.

The semantic contract is fixed:

```text
distribution chooses Output type
generator supplies deterministic state
```

## R8.2 `UniformInt`

Target:

```telos
data UniformInt private(
  _ lower: Int,
  _ upperExclusive: Int,
)

impl UniformInt {
  @class
  from(_ lower: Int, below upper: Int) -> UniformInt {
    if upper <= lower {
      throw ArgumentError.new(
        "UniformInt.from: upper bound must be greater than lower bound"
      )
    }

    UniformInt(lower, upper)
  }
}

impl Distribution for UniformInt {
  type Output = Int

  sample<G>(using generator: G) -> Int
    where G |= Generator
  {
    generator.nextInt(
      from: self.lower,
      below: self.upperExclusive
    )
  }
}
```

Use actual generated accessors for the private `data` fields.

Validation occurs only at construction.

## R8.3 `Bernoulli`

Target:

```telos
data Bernoulli private(
  _ probability: Float,
)

impl Bernoulli {
  @class
  new(_ probability: Float) -> Bernoulli {
    if probability.isNaN
      or probability < 0.0
      or probability > 1.0
    {
      throw ArgumentError.new(
        "Bernoulli.new: probability must be in 0.0..=1.0"
      )
    }

    Bernoulli(probability)
  }
}

impl Distribution for Bernoulli {
  type Output = Bool

  sample<G>(using generator: G) -> Bool
    where G |= Generator
  {
    if self.probability == 0.0 { return false }
    if self.probability == 1.0 { return true }

    generator.nextFloat < self.probability
  }
}
```

## R8.4 Consumption semantics

Freeze:

```text
Bernoulli(0) -> false, zero words
Bernoulli(1) -> true, zero words
```

## R8.5 No reverse convenience yet

Do not add:

```telos
rng.sample(distribution)
```

in this slice.

The canonical dependency direction is:

```text
Distribution -> Generator
```

Avoid a source cycle solely for symmetrical syntax.

## R8 exit criterion

Associated result typing works with no `Dynamic` erosion.

---

# 13. R9 — Design the secure source layer before native code

## File

```text
random/secure.ph
```

## R9.1 Error type

Recommended:

```telos
class EntropyError is Error {}
```

This represents:

```text
failure to acquire system secure randomness
```

It does not imply direct access to raw physical entropy.

## R9.2 Avoid native source-error minting if unnecessary

Use a source-owned error-lifting seam.

Recommended internal primitive shape:

```telos
internal native rawFill(_ bytes: Bytes) -> Option<String>
```

Semantics:

```text
None
    success

Some(message)
    host secure-random acquisition failed
```

Then source code turns the internal result into a canonical source error:

```telos
fill(_ bytes: Bytes) -> Unit {
  self.rawFill(bytes).match(
    some: |message| {
      throw EntropyError.new(message)
    },
    none: || {
      ()
    }
  )
}
```

This design is preferred because:

- the source error hierarchy remains source-owned;
- no new `RuntimeError` variant is required merely to construct `EntropyError`;
- native code returns plain data;
- native code does not re-enter the interpreter to instantiate/raise a source class;
- failure remains catchable as `EntropyError`;
- the public API does not expose the `Option<String>` seam because `rawFill` is internal.

If the existing native/source error minting infrastructure has a clearly better canonical mechanism by implementation time, it may be used instead, but the final public behavior must remain `EntropyError`.

Do not return a `Bool`; preserve host failure text.

## R9.3 Capability object

Prefer a source-owned singleton pattern matching `SystemClock`:

```telos
class SecureRandom {
  @class mut _instance

  @constructor
  private create() { self }

  @class
  system -> SecureRandom {
    if _instance == None {
      _instance = SecureRandom.create()
    }
    _instance
  }

  internal native rawFill(_ bytes: Bytes) -> Option<String>

  fill(_ bytes: Bytes) -> Unit { ... }

  bytes(_ count: Int) -> Bytes {
    if count < 0 {
      throw ArgumentError.new(
        "SecureRandom.bytes: count must be non-negative"
      )
    }

    let result = Bytes.new(count)
    self.fill(result)
    result
  }
}
```

The capability stores no PRNG state.

The singleton is identity/convenience only.

## R9.4 Failure-state contract

For:

```text
SecureRandom.fill(existingBytes)
```

if host acquisition fails:

```text
method raises EntropyError
buffer contents after failure are unspecified
```

Document this.

For:

```text
SecureRandom.bytes(count)
```

failure means:

```text
no Bytes value is returned
```

because the fresh zeroed buffer becomes unreachable when the call raises.

## R9 exit criterion

The complete public secure API is source-defined before native implementation begins.

---

# 14. R10 — Add the native secure-random boundary

## Files

```text
phalcom-core/src/primitive/random.rs               NEW
phalcom-core/src/primitive/mod.rs
phalcom-core/Cargo.toml
phalcom-native-meta/src/universe.rs
phalcom-native-surface/src/generated.rs            regenerate later
```

## R10.1 Direct dependency

`Cargo.lock` already contains:

```text
getrandom 0.3.3
```

transitively.

Because `phalcom-core` will call it directly, add a direct dependency:

```toml
getrandom = "0.3.3"
```

to `phalcom-core/Cargo.toml`.

Do not rely on another crate's transitive dependency.

If workspace dependency centralization has changed by implementation time, follow the current convention.

## R10.2 Add `UniverseKey::SecureRandom`

Mirror the source-owned class pattern used by:

```text
SystemClock
SystemWallClock
TimeZone
FloatMathKernel
```

Update all required total matches:

```text
UniverseKey enum
source_path()
name()
from_name()
class relation catalog if required by the current source-owned native owner model
tests/snapshots generated from UniverseKey
```

Canonical source path:

```text
["random", "secure"]
```

Canonical name:

```text
SecureRandom
```

Do not make `SecureRandom` a primordial VM class.

Its source declaration owns identity.

## R10.3 Primitive module

Add:

```rust
pub mod random;
```

to `phalcom-core/src/primitive/mod.rs`.

## R10.4 Primitive signature

Target descriptor:

```text
SecureRandom::rawFill(Bytes) -> Option<String>
visibility = internal
effects = [...]
```

Conceptual Rust:

```rust
#[phalcom_native_macros::primitive(
    SecureRandom,
    "rawFill(_)",
    params = [Bytes],
    returns = "Option<String>",
    types = "(Bytes) -> Option<String>",
    visibility = internal,
    effects = [mutation, nondeterminism, blocking]
)]
pub fn secure_random_raw_fill(
    vm: &mut VM,
    _receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    ...
}
```

Verify exact macro spelling for generic return metadata from existing primitives before committing.

## R10.5 Byte-buffer mutation

Use the existing `expect_bytes` helper.

Algorithm:

```text
validate argument as Bytes
obtain ObjRef
borrow mutable backing slice
call getrandom::fill(slice)
release mutable borrow
on success:
    return None
on failure:
    allocate error message String
    return Some(message)
```

### Critical borrow rule

Do not allocate a Telos String while holding the mutable heap borrow to the byte slice.

Structure Rust scopes so:

```rust
let outcome = {
    let bytes = vm.heap.bytes_mut(id).as_mut_slice();
    getrandom::fill(bytes)
};

match outcome {
    Ok(()) => Ok(vm.none_value()),
    Err(error) => {
        let message = vm.alloc_string_value(...);
        Ok(message.wrap_some())
    }
}
```

Adapt exact APIs.

No interpreter reentry occurs.

## R10.6 Zero length

`rawFill(Bytes.new(0))` must succeed without special failure.

## R10.7 No fallback

Never call:

```text
time
PID
object address
VM seed
xoshiro
hash seed
```

when `getrandom` fails.

Return the failure message.

## R10.8 No native `bytes(count)`

Keep allocation/validation in source.

The native floor remains only:

```text
fill this already-allocated byte buffer from the OS
```

## R10 exit criterion

SecureRandom's only native semantic is system byte filling.

---

# 15. R11 — Effects and semantic metadata

## Purpose

Ensure tooling understands the difference between deterministic state transition and host nondeterminism.

## R11.1 Deterministic source effects

Do not annotate xoshiro with native `Nondeterminism`.

Its source body mutates fields, so normal source effect inference should derive mutation.

Verify semantic presentation/effect output for:

```text
Xoshiro256PlusPlus.nextU64
Generator.nextBool
Generator.nextFloat
```

The derived methods are deterministic but call a mutating method.

## R11.2 Secure native effects

The native `rawFill` descriptor must include:

```text
mutation
nondeterminism
```

Evaluate `blocking` using the existing project taxonomy.

Recommended first contract:

```text
[mutation, nondeterminism, blocking]
```

Add `io` only if the project's effect policy classifies secure host entropy acquisition as I/O. Check adjacent host primitives before finalizing.

Do not leave effects `Unknown`.

## R11.3 Reflection/native source ownership

Verify:

```text
source declaration:
    universe.random.secure::SecureRandom.rawFill(_)

native descriptor:
    owner UniverseKey::SecureRandom

runtime method:
    NativePrimitive implementation provenance

semantic/public declaration:
    one canonical source member
```

No duplicate declaration identity.

## R11 exit criterion

Effect and native/source metadata agree with actual semantics.

---

# 16. R12 — Comprehensive deterministic tests

Use a layered test structure.

## 16.1 Unit layer: scripted Generator

Prove default methods independent of xoshiro.

Cover:

```text
nextBool
nextFloat
nextInt
fill
bytes
choose
shuffle
Bernoulli/UniformInt via scripted generator
```

## 16.2 Algorithm layer: Xoshiro

Use authoritative golden sequences.

Required seeds:

```text
0
1
42
0xffffffffffffffff
```

Test substantial prefixes.

## 16.3 Replay interleaving

Freeze method interaction:

```telos
let rng = Xoshiro256PlusPlus.seed(42)

rng.nextBool
rng.nextFloat
rng.nextInt(below: 10)
rng.bytes(3)
rng.nextU64
```

Assert every result exactly.

This detects hidden caches or altered consumption semantics.

## 16.4 Seed validation

Compile/run:

```text
seed(-1) -> ArgumentError
seed(0) -> valid
seed(2^64 - 1) -> valid
seed(2^64) -> ArgumentError
seed(10^100) -> ArgumentError
```

## 16.5 `nextInt` forced rejection

Use scripted streams that force rejection.

For example choose a bound whose bit width admits candidates above the bound and program:

```text
first candidate >= upper
second candidate < upper
```

Assert exact number of consumed words.

## 16.6 Huge range

Test:

```telos
rng.nextInt(below: 10 ** 1000)
```

and verify range law.

Also exercise a configured numeric-policy boundary.

## 16.7 Byte order

Freeze exact sequences around:

```text
0
1
7
8
9
15
16
17
```

bytes.

## 16.8 Choose typing

Compile a case where:

```telos
List<User>
```

passed to `choose` yields:

```text
Option<User>
```

not `Option<Dynamic>`.

## 16.9 Shuffle invariants

For a golden seed:

```text
exact final permutation
size unchanged
multiset unchanged
```

## R12 exit criterion

Deterministic behavior is structurally proven, not statistically inferred.

---

# 17. R13 — Distribution tests

## `UniformInt`

Test:

```text
from(0, below: 1)
from(-10, below: 10)
huge lower/upper
upper == lower -> ArgumentError
upper < lower -> ArgumentError
```

Verify sample result type is `Int`.

Verify sampling delegates to exact generator range behavior.

## `Bernoulli`

Test constructor:

```text
NaN -> ArgumentError
-0.1 -> ArgumentError
1.1 -> ArgumentError
0.0 -> valid
1.0 -> valid
0.5 -> valid
```

Sampling:

```text
p=0 -> false / zero consumption
p=1 -> true / zero consumption
intermediate -> exact nextFloat comparison
```

Verify `Output == Bool`.

---

# 18. R14 — SecureRandom tests

## R14.1 Public behavior

Test:

```text
SecureRandom.system is SecureRandom
SecureRandom.system returns stable singleton identity if singleton chosen
bytes(0).size == 0
bytes(1).size == 1
bytes(32).size == 32
fill(existing) returns Unit
negative count -> ArgumentError
```

Do not assert the bytes differ between calls as a correctness proof.

Do not assert fixed output.

## R14.2 Native success seam

Add a native-level test that fills a buffer and validates:

```text
full requested length addressed
no panic
valid buffer object remains
```

## R14.3 Failure injection

The secure failure path must be deterministically testable.

Do not rely on causing the real OS RNG to fail.

Use the smallest test-only seam.

Preferred Rust structure:

```rust
fn fill_system_random(bytes: &mut [u8]) -> Result<(), getrandom::Error> {
    getrandom::fill(bytes)
}
```

and isolate conversion to source result so tests can exercise a helper with a controlled error if practical.

If direct dependency injection is awkward, factor:

```rust
fn encode_fill_result(...)
```

and unit-test error conversion separately.

Do not add a public Telos mock-secure provider solely for native failure tests.

## R14.4 Error lifting

Test source:

```telos
try {
  ...
} on EntropyError e {
  ...
}
```

against an injected native failure path if the test harness allows native provider substitution.

If the current test infrastructure cannot inject one primitive implementation cleanly, add a `cfg(test)`-only provider seam in Rust.

Do not weaken the production API to make testing easy.

## R14.5 Nonconformance

Compile-time test:

```text
SecureRandom |= Generator
```

must fail.

## R14 exit criterion

Security failure is explicit, catchable, and never replaced by pseudorandom output.

---

# 19. R15 — Cross-platform determinism tests

The deterministic suite must detect:

```text
endianness dependence
pointer-width dependence
signed-shift leakage
machine-word truncation
host random dependence
```

At minimum, all xoshiro and deterministic byte golden vectors should run in ordinary CI.

If CI currently includes Linux and macOS, both should execute the same vector fixtures.

Do not make cross-platform equality conditional on platform.

`SecureRandom` naturally has no cross-platform byte equality requirement.

---

# 20. Performance and representation checkpoint

The source implementation deliberately starts with nonnegative 64-bit logical `Int` words.

That may trigger `LargeInt` allocation for values above signed immediate range.

Measure before optimizing.

## 20.1 Required microbenchmarks

Add or adapt benchmarks for:

```text
10^6 nextU64 calls
10^6 nextFloat calls
10^6 nextInt(below: 100)
10^5 nextInt(below: 2^128)
bytes(1 KiB)
shuffle 100
shuffle 10_000
```

## 20.2 Compare against current source baseline

Existing:

```text
benchmarks/math/monte_carlo.ph
benchmarks/math/random_walk.ph
```

use source-authored MINSTD.

Port copies or variants to `universe.random` after correctness is green.

Compare wall-clock behavior.

## 20.3 Allocation/GC concern

Investigate:

```text
heap LargeInt creation per generated word
GC pressure
temporary allocation in SplitMix multiplication
temporary allocation in rotation
```

## 20.4 Optimization decision tree

If performance is acceptable:

```text
keep straightforward source representation
```

If performance is poor:

1. inspect whether integer primitives already avoid unnecessary allocation;
2. consider an internal source representation with narrower chunks;
3. consider general fixed-width numeric helpers only if broadly justified;
4. only then consider a native deterministic backend.

Do not change the public `Generator` contract or named stream.

## 20.5 Native optimization invariant

Any later native xoshiro implementation must be bit-identical to the frozen source semantics.

---

# 21. Resource-limit verification

## 21.1 Candidate width

The R5 assembly algorithm intentionally limits intermediates to requested bit width.

Add a test near:

```text
NumericPolicy.max_integer_bits
```

that demonstrates a valid bound is not rejected merely because the implementation temporarily builds `bits + 63`.

## 21.2 Giant request

Test a bound that exceeds configured numeric limits.

Expect normal:

```text
#numericLimit
```

not:

```text
ArgumentError
panic
OOM
truncation
```

## 21.3 Secure bytes allocation

A huge `SecureRandom.bytes(count)` request must fail through the ordinary `Bytes.new` allocation/resource boundary.

Do not allocate an intermediate Vec in the random primitive.

---

# 22. Documentation requirements

## 22.1 Package docs

Explain three distinct concepts:

```text
Generator
    deterministic state protocol

Xoshiro256PlusPlus
    reproducible non-cryptographic algorithm

SecureRandom
    OS-backed security capability
```

## 22.2 Xoshiro warning

Include prominently:

> `Xoshiro256PlusPlus` is deterministic and not cryptographically secure. Do not use it for secrets, authentication tokens, cryptographic keys, security-sensitive nonces, or any value whose safety depends on unpredictability.

## 22.3 `Generator.bytes`

State clearly:

> `Generator.bytes` produces deterministic pseudorandom bytes. It is not a secure-random API.

## 22.4 SecureRandom wording

State:

> `SecureRandom` acquires cryptographically suitable random bytes from the operating system.

Do not say:

```text
true random
hardware entropy
raw entropy
```

## 22.5 Replay documentation

Document that xoshiro freezes:

```text
algorithm
seed expansion
nextU64
derived method consumption
bounded sampling
byte order
shuffle order
```

## 22.6 Concurrent use

Document:

```text
sharing one mutable generator across fibers makes stream assignment depend on call ordering
```

Recommend explicit separate streams for reproducible concurrent workflows.

---

# 23. Generated native surface

After `SecureRandom.rawFill` and `UniverseKey::SecureRandom` are final:

```sh
RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root .
RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

Review generated diff.

Expected new native surface should be narrowly centered on:

```text
SecureRandom.rawFill(_)
```

Random deterministic methods should not appear as new native rows.

If unrelated generated baseline drift appears, isolate or document it exactly as in the math plan.

Never hand-edit `generated.rs`.

---

# 24. Recommended commit sequence

Keep review/bisection boundaries meaningful.

## Commit 1

```text
feat: register universe random source modules
```

Scope:

```text
random child files
UNIVERSE_NODES
source_text
facade imports
topology tests
```

## Commit 2

```text
feat: add deterministic random generator protocol
```

Scope:

```text
Generator
scripted test Generator
nextBool
nextFloat
nextInt
bytes/fill
choose/shuffle
default-method tests
```

If this commit becomes too large, split at:

```text
Generator scalar sampling
Generator collection/byte helpers
```

## Commit 3

```text
feat: add xoshiro256++ deterministic generator
```

Scope:

```text
SplitMix64
Xoshiro256PlusPlus
seed validation
golden vectors
```

## Commit 4

```text
feat: add typed random distributions
```

Scope:

```text
Distribution
UniformInt
Bernoulli
typing tests
```

## Commit 5

```text
feat: add secure system randomness
```

Scope:

```text
SecureRandom
EntropyError
UniverseKey::SecureRandom
getrandom dependency
native rawFill
secure failure tests
effect metadata
```

## Commit 6

```text
test: harden random replay and resource contracts
```

Scope:

```text
interleaving vectors
huge Int ranges
numeric policy
cross-platform fixtures
shuffle golden
```

## Commit 7

```text
docs: finalize universe random implementation record
```

Scope:

```text
package docs
implementation handoff/walkthrough
generated surface synchronization
benchmark notes
```

---

# 25. Exact file checklist

## New source files

```text
phalcom-core/core/universe/src/random/generator.ph
phalcom-core/core/universe/src/random/xoshiro256pp.ph
phalcom-core/core/universe/src/random/distribution.ph
phalcom-core/core/universe/src/random/uniform_int.ph
phalcom-core/core/universe/src/random/bernoulli.ph
phalcom-core/core/universe/src/random/secure.ph
```

## Modify

```text
phalcom-core/core/universe/src/random/package.ph
phalcom-modules/src/builtin.rs
phalcom-native-meta/src/universe.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/Cargo.toml
phalcom-native-surface/src/generated.rs
```

## New native file

```text
phalcom-core/src/primitive/random.rs
```

## Tests likely touching/adding

Use the repository's existing organization, likely under:

```text
phalcom-core/tests/core/language/
phalcom-core/tests/core/modules/
phalcom-core/tests/fixtures/language/random/
```

Potential fixture names:

```text
random_foundation.ph
random_xoshiro_vectors.ph
random_derived_sampling.ph
random_bytes.ph
random_collections.ph
random_distributions.ph
random_secure.ph
```

Compile-fail/type fixtures:

```text
random_secure_not_generator.ph
random_uniform_output_type.ph
random_choose_preserves_type.ph
```

Exact harness naming should follow existing corpus conventions.

---

# 26. Suggested final source architecture

The following is architectural pseudocode. Confirm exact current syntax during R0.

## `generator.ph`

```telos
trait Generator {
  nextU64 -> Int

  nextBool -> Bool {
    (self.nextU64 >> 63) != 0
  }

  nextFloat -> Float {
    let bits = self.nextU64 >> 11
    Float.new(bits) / 9007199254740992.0
  }

  nextInt(below upper: Int) -> Int {
    if upper <= 0 {
      throw ArgumentError.new(
        "Generator.nextInt: upper bound must be positive"
      )
    }

    if upper == 1 {
      return 0
    }

    let bits = (upper - 1).bitLength
    let words = (bits + 63) ~/ 64
    let headBits = bits - ((words - 1) * 64)

    while true {
      let mut candidate = self.nextU64

      if headBits < 64 {
        candidate = candidate & ((1 << headBits) - 1)
      }

      let mut index = 1
      while index < words {
        candidate = (candidate << 64) | self.nextU64
        index = index + 1
      }

      if candidate < upper {
        return candidate
      }
    }
  }

  nextInt(from lower: Int, below upper: Int) -> Int {
    if upper <= lower {
      throw ArgumentError.new(
        "Generator.nextInt: upper bound must be greater than lower bound"
      )
    }

    lower + self.nextInt(below: upper - lower)
  }

  fill(_ bytes: Bytes) -> Unit {
    let mut offset = 0

    while offset < bytes.size {
      let word = self.nextU64
      let mut byteIndex = 0

      while byteIndex < 8 and offset < bytes.size {
        let shift = 56 - (byteIndex * 8)
        bytes.set(
          offset,
          (word >> shift) & 255
        )
        offset = offset + 1
        byteIndex = byteIndex + 1
      }
    }

    ()
  }

  bytes(_ count: Int) -> Bytes {
    if count < 0 {
      throw ArgumentError.new(
        "Generator.bytes: count must be non-negative"
      )
    }

    let out = Bytes.new(count)
    self.fill(out)
    out
  }

  choose<T>(_ values: List<T>) -> Option<T> {
    if values.size == 0 {
      return None
    }

    if values.size == 1 {
      return Some(values.at(0))
    }

    Some(
      values.at(
        self.nextInt(below: values.size)
      )
    )
  }

  shuffle<T>(_ values: List<T>) -> Unit {
    let mut i = values.size - 1

    while i > 0 {
      let j = self.nextInt(below: i + 1)
      values.swap(first: i, second: j).unwrap
      i = i - 1
    }

    ()
  }
}

export Generator
```

If subtraction from an empty list causes `i = -1`, the loop still exits, but an explicit:

```telos
if values.size <= 1 { return () }
```

is preferable for clarity and frozen zero-consumption semantics.

## `xoshiro256pp.ph`

```telos
from .generator import Generator

const WORD_BITS = 64
const WORD_MODULUS = 1 << WORD_BITS
const WORD_MASK = WORD_MODULUS - 1

const SPLITMIX_GAMMA = 0x9E3779B97F4A7C15
const SPLITMIX_MUL1 = 0xBF58476D1CE4E5B9
const SPLITMIX_MUL2 = 0x94D049BB133111EB

class Xoshiro256PlusPlus {
  mut _s0: Int
  mut _s1: Int
  mut _s2: Int
  mut _s3: Int

  @constructor
  private create(
    _ s0: Int,
    _ s1: Int,
    _ s2: Int,
    _ s3: Int
  ) {
    _s0 = s0
    _s1 = s1
    _s2 = s2
    _s3 = s3
  }

  @class
  private wrap64(_ value: Int) -> Int {
    value & WORD_MASK
  }

  @class
  private splitMixValue(_ state: Int) -> Int {
    let mut z = state
    z = ((z ^ (z >> 30)) * SPLITMIX_MUL1) & WORD_MASK
    z = ((z ^ (z >> 27)) * SPLITMIX_MUL2) & WORD_MASK
    (z ^ (z >> 31)) & WORD_MASK
  }

  @class
  private rotl64(_ value: Int, _ shift: Int) -> Int {
    (
      (value << shift)
      | (value >> (WORD_BITS - shift))
    ) & WORD_MASK
  }

  @class
  seed(_ seed: Int) -> Xoshiro256PlusPlus {
    if seed < 0 or seed >= WORD_MODULUS {
      throw ArgumentError.new(
        "Xoshiro256PlusPlus.seed: seed must be in 0..<2^64"
      )
    }

    let mut x = seed

    x = Xoshiro256PlusPlus.wrap64(x + SPLITMIX_GAMMA)
    let s0 = Xoshiro256PlusPlus.splitMixValue(x)

    x = Xoshiro256PlusPlus.wrap64(x + SPLITMIX_GAMMA)
    let s1 = Xoshiro256PlusPlus.splitMixValue(x)

    x = Xoshiro256PlusPlus.wrap64(x + SPLITMIX_GAMMA)
    let s2 = Xoshiro256PlusPlus.splitMixValue(x)

    x = Xoshiro256PlusPlus.wrap64(x + SPLITMIX_GAMMA)
    let s3 = Xoshiro256PlusPlus.splitMixValue(x)

    Xoshiro256PlusPlus.create(s0, s1, s2, s3)
  }
}

impl Generator for Xoshiro256PlusPlus {
  nextU64 -> Int {
    let result = (
      Xoshiro256PlusPlus.rotl64(
        (_s0 + _s3) & WORD_MASK,
        23
      )
      + _s0
    ) & WORD_MASK

    let t = (_s1 << 17) & WORD_MASK

    _s2 = _s2 ^ _s0
    _s3 = _s3 ^ _s1
    _s1 = _s1 ^ _s2
    _s0 = _s0 ^ _s3

    _s2 = _s2 ^ t
    _s3 = Xoshiro256PlusPlus.rotl64(_s3, 45)

    result
  }
}

export Xoshiro256PlusPlus
```

## `distribution.ph`

```telos
from .generator import Generator

trait Distribution {
  type Output

  sample<G>(using generator: G) -> Self::Output
    where G |= Generator
}

export Distribution
```

## `uniform_int.ph`

```telos
from .generator import Generator
from .distribution import Distribution

data UniformInt private(
  _ lower: Int,
  _ upperExclusive: Int,
)

impl UniformInt {
  @class
  from(_ lower: Int, below upper: Int) -> UniformInt {
    if upper <= lower {
      throw ArgumentError.new(
        "UniformInt.from: upper bound must be greater than lower bound"
      )
    }

    UniformInt(lower, upper)
  }
}

impl Distribution for UniformInt {
  type Output = Int

  sample<G>(using generator: G) -> Int
    where G |= Generator
  {
    generator.nextInt(
      from: self.lower,
      below: self.upperExclusive
    )
  }
}

export UniformInt
```

## `bernoulli.ph`

```telos
from .generator import Generator
from .distribution import Distribution

data Bernoulli private(
  _ probability: Float,
)

impl Bernoulli {
  @class
  new(_ probability: Float) -> Bernoulli {
    if probability.isNaN
      or probability < 0.0
      or probability > 1.0
    {
      throw ArgumentError.new(
        "Bernoulli.new: probability must be in 0.0..=1.0"
      )
    }

    Bernoulli(probability)
  }
}

impl Distribution for Bernoulli {
  type Output = Bool

  sample<G>(using generator: G) -> Bool
    where G |= Generator
  {
    if self.probability == 0.0 {
      return false
    }

    if self.probability == 1.0 {
      return true
    }

    generator.nextFloat < self.probability
  }
}

export Bernoulli
```

## `secure.ph`

```telos
class EntropyError is Error {}

class SecureRandom {
  @class mut _instance

  @constructor
  private create() { self }

  @class
  system -> SecureRandom {
    if _instance == None {
      _instance = SecureRandom.create()
    }

    _instance
  }

  internal native rawFill(_ bytes: Bytes) -> Option<String>

  fill(_ bytes: Bytes) -> Unit {
    self.rawFill(bytes).match(
      some: |message| {
        throw EntropyError.new(message)
      },
      none: || {
        ()
      }
    )
  }

  bytes(_ count: Int) -> Bytes {
    if count < 0 {
      throw ArgumentError.new(
        "SecureRandom.bytes: count must be non-negative"
      )
    }

    let out = Bytes.new(count)
    self.fill(out)
    out
  }
}

export SecureRandom, EntropyError
```

The exact property accessor names generated for `data` fields and exact `Option.match` argument spelling must be confirmed from live source.

---

# 27. Example final user experience

## Deterministic simulation

```telos
import universe.random as random

let rng = random.Xoshiro256PlusPlus.seed(42)

let samples = List<Float>.new()

let mut i = 0
while i < 1000 {
  samples.append(rng.nextFloat)
  i = i + 1
}
```

## Exact huge range

```telos
let rng = random.Xoshiro256PlusPlus.seed(42)

let value = rng.nextInt(
  below: 10 ** 500
)
```

## Integer interval

```telos
let temperature = rng.nextInt(
  from: -30,
  below: 50
)
```

## Choice

```telos
let candidate: Option<User> = rng.choose(users)
```

## Shuffle

```telos
rng.shuffle(cards)
```

## Typed distribution

```telos
let die = random.UniformInt.from(
  1,
  below: 7
)

let roll: Int = die.sample(
  using: rng
)
```

## Bernoulli

```telos
let coin = random.Bernoulli.new(0.5)
let heads: Bool = coin.sample(using: rng)
```

## Secure bytes

```telos
let secure = random.SecureRandom.system
let token = secure.bytes(32)
```

These examples should become documentation/corpus fixtures where appropriate.

---

# 28. Rejected implementation shortcuts

## Shortcut A — global RNG

```telos
random.nextInt(...)
```

with hidden state.

**Rejected.**

## Shortcut B — implicit system seed

```telos
Xoshiro256PlusPlus.new()
```

which seeds itself from time/OS randomness.

**Rejected.**

## Shortcut C — secure generator conformance

```telos
impl Generator for SecureRandom
```

**Rejected.**

## Shortcut D — modulo bounded sampling

```telos
nextU64 % upper
```

**Rejected.**

## Shortcut E — 64-bit public range ceiling

```text
upper <= 2^64
```

**Rejected.**

## Shortcut F — native deterministic PRNG first

**Rejected** without measured source failure.

## Shortcut G — generic Iterable choose

**Rejected** in v1.

## Shortcut H — hidden byte cache

**Rejected** because consumption becomes less transparent.

## Shortcut I — use VM hash seed

**Rejected.**

## Shortcut J — secure failure fallback

```text
getrandom fails
-> seed xoshiro
-> continue
```

**Categorically rejected.**

## Shortcut K — host error as ArgumentError

**Rejected.**

## Shortcut L — expose internal child modules for convenience

**Rejected** unless public package topology deliberately changes.

---

# 29. Stop-and-consult conditions

Stop implementation and request architectural review if any of these occur:

1. trait default methods cannot preserve method-local generic result types without `Dynamic`;
2. `Distribution::Output` cannot be expressed with the current associated-type system;
3. source xoshiro requires a new VM numeric representation merely to be correct;
4. static LargeInt constants cannot be used safely in ordinary Universe source;
5. package-private child imports require public `expose`;
6. source-owned `SecureRandom` cannot be a canonical native owner without creating a duplicate runtime class;
7. `getrandom` failure cannot be carried through the internal source-lifting seam without interpreter reentry or unsafe heap borrowing;
8. numeric-policy rules reject valid `nextInt` bounds because of unavoidable implementation overshoot;
9. deterministic vectors differ by platform despite explicit masks;
10. source xoshiro performance is catastrophically poor and a general numeric fix is not apparent.

Do not conceal these by changing public semantics.

---

# 30. Verification sequence

Run gates in order.

## Gate A — hygiene

```sh
git diff --check
cargo fmt --all -- --check
```

Avoid formatting unrelated dirty files.

## Gate B — compilation

```sh
RUSTFLAGS='' cargo check -p phalcom-modules
RUSTFLAGS='' cargo check -p phalcom-native-meta
RUSTFLAGS='' cargo check -p phalcom-core
RUSTFLAGS='' cargo check -p phalcom-native-surface
```

## Gate C — topology

Run focused tests proving:

```text
universe.random loads
facade exports exist
internal children resolve from facade
external hidden traversal rejected
```

## Gate D — Generator default methods

Run scripted-generator tests.

## Gate E — xoshiro vectors

All seeds and prefixes must pass exactly.

## Gate F — bounded integer tests

Run:

```text
forced rejection
powers of two
2^64 boundaries
10^100
10^1000
negative/cross-zero intervals
numeric-policy boundary
```

## Gate G — bytes/collections

Run exact byte and shuffle goldens.

## Gate H — distributions

Run typing + boundary + consumption tests.

## Gate I — SecureRandom

Run:

```text
zero length
normal fill
bytes count
negative count
native success
injected failure
EntropyError catch
SecureRandom !|= Generator
```

## Gate J — native surface

```sh
RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

## Gate K — language corpus

Run the random corpus filter and confirm tests are actually selected.

## Gate L — broader directly affected suites

Run:

```text
module source/provider suites
native source verification
semantic effect tests
numeric resource tests
core language corpus
```

## Gate M — broad core

```sh
RUSTFLAGS='' cargo test -p phalcom-core
```

If not run due cost, record it truthfully as NOT RUN.

---

# 31. Final merge-readiness matrix

The slice is complete only when:

| Area | Gate |
|---|---|
| Package facade | PASS |
| Hidden child topology | PASS |
| Generator minimal conformance | PASS |
| Custom scripted Generator inherits defaults | PASS |
| xoshiro seed 0 vector | PASS |
| xoshiro seed 1 vector | PASS |
| xoshiro seed 42 vector | PASS |
| xoshiro max-seed vector | PASS |
| invalid seed rejection | PASS |
| nextBool exact consumption | PASS |
| nextFloat exact bits/range | PASS |
| nextInt unbiased forced-rejection proof | PASS |
| nextInt arbitrary precision | PASS |
| nextInt numeric policy | PASS |
| deterministic byte order | PASS |
| partial byte consumption | PASS |
| choose typing and empty behavior | PASS |
| shuffle golden permutation | PASS |
| Distribution associated Output | PASS |
| UniformInt validation/output | PASS |
| Bernoulli validation/output | PASS |
| Bernoulli boundary zero-consumption | PASS |
| SecureRandom.system | PASS |
| SecureRandom fill/bytes | PASS |
| secure host failure propagation | PASS |
| no deterministic secure fallback | PASS |
| SecureRandom !|= Generator | PASS |
| deterministic mutation effect | PASS |
| secure nondeterminism/mutation effects | PASS |
| native source identity | PASS |
| native surface generator check | PASS |
| performance checkpoint recorded | PASS |
| docs updated | PASS |

---

# 32. Final delivery record

The implementer handoff should report:

```text
branch / HEAD
commits
changed files
new public API
exact replay choices
xoshiro reference-vector provenance
effect metadata
getrandom version
numeric-policy results
performance numbers
all test commands and results
remaining deferred work
```

Deferred work should explicitly include:

```text
jump / longJump
stream splitting
parallel reproducibility
complex distributions
weighted sampling
reservoir sampling
state serialization
secure token/UUID helpers
fixed-width integer optimization if later justified
mutation-capability integration
```

Do not describe deferred items as implemented or partially implied.

---

# 33. Architectural success criterion

The implementation is successful when `random` reads as ordinary Telos code rather than a runtime extension.

The core should look conceptually like:

```text
Generator trait
    ↓
ordinary default Telos methods
    ↓
Xoshiro256PlusPlus source class
    ↓
explicit mutable state

Distribution trait
    ↓
validated data descriptors

SecureRandom source class
    ↓
one narrow native host-byte primitive
```

That shape is the actual design objective.

The package should demonstrate that Telos can express a serious standard-library subsystem using its own:

```text
classes
traits
associated types
generic methods
private data construction
mutable fields
exact Int
Float
List
Bytes
Option
errors
effects
module system
native source ownership
```

without inventing random-specific language machinery.
