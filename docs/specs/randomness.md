# Random Generators and Distributions

## Ownership

`universe.random` owns random-generator state, built-in generators, sampling
distribution protocol, integer sampling, and secure entropy access. Optional
probability laws in [`numerics/probability.md`](numerics/probability.md) add
probability evaluation to selected distributions but do not define a competing
sampling protocol.

## Generator protocol

`Generator.nextU64` produces an integer `n` satisfying `0 <= n < 2^64`.
Derived operations consume generator words in their documented order and do
not cache unused bits or bytes between calls. Replaying the same generator
state and the same sequence of operations reproduces the same outputs.

- `nextBool` consumes one word and uses its most significant bit.
- `nextFloat` consumes one word and maps its upper 53 bits to the binary64 grid
  in `[0, 1)`.
- `openUnit` consumes until `nextFloat` is nonzero, yielding `(0, 1)` for
  algorithms that take a logarithm.
- `nextInt(below: upper)` samples without modulo bias from `[0, upper)` and
  requires `upper > 0`; the singleton range returns zero without consuming a
  word. The lower/upper form samples `[lower, upper)` and requires
  `lower < upper`.
- `fill` writes bytes in big-endian word order and discards unused bytes from
  the final word. `bytes(count)` requires a nonnegative count.
- `choose` returns `None` for an empty List, the only value for a singleton,
  and otherwise selects one entry uniformly. Empty and singleton inputs consume
  no generator word.
- `shuffle` mutates its supplied List using descending Fisher–Yates. Lists of
  size zero or one are unchanged and consume no generator word.

These derived operations have specified consumption behavior because it is
observable during deterministic replay. A change in consumption order is a
behavioral change even if the resulting distribution remains statistically
uniform.

## Distribution protocol

`Distribution` has an associated `Output` type and `sample(using:)`, which
receives its generator explicitly. Sampling must not obtain hidden global random
state. Probability-evaluation capabilities such as mass, density, tails, and
quantiles are independent of this sampling protocol.

`Xoshiro256PlusPlus.seed` accepts an integer in `[0, 2^64)`. The generator is
deterministic and pseudorandom, not a cryptographic generator. `SecureRandom`
is the explicit OS-backed source for security-sensitive random bytes and may
report `EntropyError`. Deterministic
generator bytes must not be used for secrets, keys, authentication tokens, or
security-sensitive nonces.

## Type ownership

`Bernoulli`, `UniformInt`, and `Xoshiro256PlusPlus` are owned by
`universe.random`. `numerics.probability` augments the Universe `Bernoulli` with
mass, log-mass, ordered-tail, and quantile capabilities; it does not replace its
sampling identity or constructor.
