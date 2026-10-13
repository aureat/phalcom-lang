from .generator import Generator

const WORD_MODULUS: Int = 18446744073709551616
const WORD_MASK: Int = 18446744073709551615
const GAMMA: Int = 0x9E3779B97F4A7C15
const MUL1: Int = 0xBF58476D1CE4E5B9
const MUL2: Int = 0x94D049BB133111EB

/// Reproducible xoshiro256++ with increment-before-mix SplitMix64 seeding.
/// Its algorithm, seed expansion, and stream are fixed for replay. This
/// generator is deterministic and not cryptographically secure. Do not use it
/// for secrets, authentication tokens, cryptographic keys, or security-
/// sensitive nonces. Shared use across fibers assigns the stream according to
/// call ordering; use separate explicitly seeded instances for independent,
/// reproducible workflows.
class Xoshiro256PlusPlus {
  mut _s0: Int
  mut _s1: Int
  mut _s2: Int
  mut _s3: Int

  @constructor
  private create(_ s0: Int, _ s1: Int, _ s2: Int, _ s3: Int) {
    _s0 = s0
    _s1 = s1
    _s2 = s2
    _s3 = s3
  }

  @class
  private rotl(_ value: Int, _ shift: Int) -> Int {
    ((value << shift) | (value >> (64 - shift))) & WORD_MASK
  }

  @class
  private mix(_ value: Int) -> Int {
    let mut z = value
    z = ((z ^ (z >> 30)) * MUL1) & WORD_MASK
    z = ((z ^ (z >> 27)) * MUL2) & WORD_MASK
    (z ^ (z >> 31)) & WORD_MASK
  }

  @class
  seed(_ seed: Int) -> Xoshiro256PlusPlus {
    if seed < 0 or seed >= WORD_MODULUS { ArgumentError.new("Xoshiro256PlusPlus.seed: seed must be in 0..<2^64").raise() }
    let mut x = seed
    x = (x + GAMMA) & WORD_MASK
    let s0 = Xoshiro256PlusPlus.mix(x)
    x = (x + GAMMA) & WORD_MASK
    let s1 = Xoshiro256PlusPlus.mix(x)
    x = (x + GAMMA) & WORD_MASK
    let s2 = Xoshiro256PlusPlus.mix(x)
    x = (x + GAMMA) & WORD_MASK
    let s3 = Xoshiro256PlusPlus.mix(x)
    Xoshiro256PlusPlus.create(s0, s1, s2, s3)
  }

  nextU64 -> Int {
    let mut result = Xoshiro256PlusPlus.rotl((_s0 + _s3) & WORD_MASK, 23)
    result = (result + _s0) & WORD_MASK
    let t = (_s1 << 17) & WORD_MASK
    _s2 = _s2 ^ _s0
    _s3 = _s3 ^ _s1
    _s1 = _s1 ^ _s2
    _s0 = _s0 ^ _s3
    _s2 = _s2 ^ t
    _s3 = Xoshiro256PlusPlus.rotl(_s3, 45)
    result
  }
}

impl Generator for Xoshiro256PlusPlus {}

export Xoshiro256PlusPlus
