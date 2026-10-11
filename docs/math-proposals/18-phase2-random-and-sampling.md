# 18. RNG integration and sampling

Status: recommendation. E05–E06 show that universe.random is a stub; E14–E16
provide source-authored state/seeding and generic generation precedents.

## 18.1 Explicit state and one RNG contract

RandomBits is a nominal capability with `nextBits32 -> Int` in [0,2^32).
Simulation helpers consume it explicitly. A mutable generator is an ordinary
class whose immutable binding may refer to mutable state, as current iterators
do. There is no Rust borrow syntax, mutable parameter annotation, global RNG,
or per-distribution private seed.

```telos
// Proposed library API; random currently has no such members.
from universe.random import Pcg32
from numerics.probability import Normal

let rng = Pcg32.new(seed: 42, stream: 7)
let normal = Normal.new(mean: 0.0, stddev: 1.0)
let x: Float = normal.sample(using: rng)
```

Passing/capturing rng shares the same stateful object. A second binding is not a
stream clone. No sampler mutates a distribution descriptor; optional precomputed
sampler state is a separate future object. E07/E08 make shared iterator state a
familiar model; E24 warns that storing a mutable reference in data does not
freeze it.

## 18.2 V1 generator: named PCG32, no unspecified Random alias

Recommend a pinned PCG-XSH-RR 64/32 generator as `Pcg32`, implemented in source
using exact Int arithmetic, masks, and shifts. This choice follows the source
arithmetic precedent rather than requiring a new VM word type. Future optimized
execution must preserve the specified bit stream. The algorithm is for
statistical simulation, with no cryptographic security claim.

Mathematical algorithm contract (non-Telos pseudocode):

```text
seed: Int in [0, 2^64); stream: Int in [0, 2^63)
increment = 2*stream + 1
state = 0
advance once; state = (state + seed) mod 2^64; advance once

nextBits32:
  old = state
  state = (old*6364136223846793005 + increment) mod 2^64
  x = (((old >> 18) xor old) >> 27) mod 2^32
  r = old >> 59
  return rotateRight32(x, r)
```

The two warm-up steps use the same state transition. Rotation has explicit
32-bit masking; unbounded Int never supplies implicit machine-word overflow.
Negative/oversized seeds or streams raise ArgumentError. Deterministic seed
expansion from arbitrary bytes can be a later named API; V1 does not truncate a
large Int seed silently.

Distinct stream IDs select distinct sequences; this is not a proof that
arbitrarily consumed parallel workloads are statistically independent or
nonoverlapping. Keep stream assignment explicit. Counter-based/splittable
generators are a later capability, not a generic `.split` method invented for
PCG32. `seed + workerIndex` is not the V1 stream partition contract.

## 18.3 Shared uniform helpers

`universe.random` owns generic helpers over RandomBits:

| Helper | Contract and algorithm |
|---|---|
| bits53 | Consume two 32-bit draws, take first>>5 then second>>6; concatenate 27+26 bits in that order |
| unit | bits53 / 2^53, Float grid in [0,1) |
| openUnit | Same grid conditioned on nonzero by rejection; Float in (0,1), for logarithmic/rejection algorithms |
| index(upper:, using:) | Int in [0,upper), upper>0; enough bits for upper.bitLength, reject candidates >=upper |

Integer rejection avoids `% upper` bias. Helpers specify their consumption order
and endpoint behavior; they do not accept an all-zero scripted source as a
uniform PRNG. Deterministic scripted sources are useful endpoint/failure test
fixtures but cannot establish sampler quality.

Grid-based continuous sampling approximates a real law; it does not promise all
binary64 values are equally likely. Bernoulli comparisons with unit draws have
probability discretization bounded by one grid step. Keep that approximation
documented. A future bit-exact Bernoulli procedure or higher-resolution source
must have a distinct sampler-version contract if consumption changes.

## 18.4 Sampler algorithm choices and boundaries

| Descriptor | Recommended V1 algorithm | Boundary/stability requirements |
|---|---|---|
| Bernoulli | unit<p | p=0/1 returns a constant without consuming RNG |
| Binomial | Inversion for small expected tail count; BTRD-style transformed rejection for large count, symmetry for p>0.5 | No O(n) Bernoulli loop for large n; direct log-mass acceptance; n=0/p=0/1 no draws |
| Categorical | Copied max-scaled weights and cumulative table; draw one unit and upper-bound search | Skip zero-weight entries; boundary search selects first prefix strictly greater than draw; last positive category absorbs only rounding residue |
| Poisson | Inversion for small rate; PTRS-style transformed rejection for large rate | lambda=0 returns 0 without draws; no linear waiting-time loop for huge rate; range failures explicit |
| Uniform | Range-aware interpolation with unit | Return a<=x<b; if rounding reaches b, choose next representable below b; descriptor has finite a<b |
| Normal | Stateless Box–Muller using two unit/openUnit draws | No descriptor cache of spare draw; scaled affine transform; no ln(0) |
| Exponential | -ln1p(-unit)/rate | Includes representable zero; unit<1 avoids infinity endpoint |
| Gamma | Marsaglia–Tsang rejection for shape>=1; shape augmentation with openUnit for shape<1 | Evaluate acceptance and small-shape power in log/scaled form; no naive overflowing intermediates |
| Beta | Two independent unit-rate Gamma draws, ratio in log/scaled form | Avoid both draws underflowing to zero or sum overflow; use log-Gamma sample internally if necessary |

The selected regimes are algorithm requirements for a later implementation;
this task does not introduce sampler code. Version 1 must record actual regime
thresholds, constants, and consumption in the sampler's implementation contract
and tests. Exact sequence compatibility is only promised with that pinned
sampler version and backend.

Uniform's upper-endpoint correction depends on the Phase 1 nextDown facility or
an equivalent scalar-owned representability helper. It is a bounded Float
rounding policy, not truncation of an unbounded distribution. For Gamma, a
positive mathematical draw rounded to zero, or any count/continuous draw outside
the representable backend range, raises NumericalRangeError. Beta's internal
log draws should avoid avoidable intermediate underflow; a final binary64 result
may round to 0 or 1 as a documented finite-precision approximation. No silent
retry conditioned on representability: that would change the law and stream.

CDF inversion is permitted for laws with an efficient validated inverse, but
Sampleable never requires HasQuantile. Likewise HasDensity/HasMass does not imply
a sampler exists. Acceptance/rejection failure budgets must raise a named
evaluation error rather than loop forever; arbitrary iteration caps are not a
mathematical replacement for sampler validation.

## 18.5 One, many, lazy, and caller-owned storage

- `d.sample(using: rng)` produces one outcome.
- `sampleMany(d, count: n, using: rng)` returns List<Sample>; validates n>=0
  and allocation policy before drawing. n=0 returns empty without touching rng.
- `samples(d, using: rng)` returns a SampleIterator<D,R> implementing canonical
  Iterator. Creating it consumes nothing; each next consumes one sample.
- `samples(...).take(n)` reuses existing bounded iteration; infinite sources
  never go directly into an unbounded count/fold terminal.
- Streaming Monte Carlo aggregates without a list. A later sink/buffer API can
  supply caller-owned storage; it need not complicate Sampleable.

SampleIterator retains references to d and rng and is single-pass. Its iterator
getter returns itself. Consuming count then mean from the same iterator does
not replay samples. Reusing a List is different: it provides fresh iterators.
An exhausted take wrapper never requests more RNG draws (current iterator
zero-take fixture is the precedent).

## 18.6 Reproducibility contract

| Level | Recommended promise |
|---|---|
| Pcg32 raw words | Pinned algorithm/seed/stream/mask contract; platform-independent Int stream and fixed golden vectors |
| Uniform integer helper | Pinned bit concatenation/rejection algorithm; deterministic given raw stream |
| Distribution samples | Deterministic for identical seed, stream, sampler version, numerical backend, call order, and parameter values |
| Across releases | Algorithm changes may be statistically equivalent without sequence identity; record sampler version |
| Across platforms/toolchains | Elementary/special functions follow Phase 1 backend precision; no universal bitwise sample guarantee |
| Concurrency/parallel simulation | V1 serial only; later explicit task streams and deterministic merge trees |

Provenance for serious simulation records seed, stream, raw generator version,
sampler/library version, backend/platform, data order, and merge order. Sharing
one RNG across interleaved tasks makes scheduling observable. Reproducible
floating reduction likewise needs a fixed operation order.

OS entropy and cryptographic RNGs belong to separate random adapters with their
own fallibility/security contracts. They can later implement a sampling source
where appropriate; simulation's seeded source does not acquire security claims
from a host library's name.
