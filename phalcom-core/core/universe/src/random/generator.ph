/// Explicit deterministic state protocol. A word is an Int in 0..<2^64.
/// No derived operation caches bits or bytes between calls.
trait Generator {
  nextU64 -> Int

  /// Consumes one word and uses its most significant bit.
  nextBool -> Bool { (self.nextU64 >> 63) != 0 }

  /// Consumes one word; its top 53 bits give an exact binary64 grid in [0,1).
  nextFloat -> Float { Float.new(self.nextU64 >> 11) / 9007199254740992.0 }

  /// Consumes until the binary64 grid draw is nonzero, producing a value in (0,1).
  /// This helper is for algorithms that take a logarithm of the draw.
  openUnit -> Float {
    let mut value = self.nextFloat
    while value == 0.0 { value = self.nextFloat }
    value
  }

  /// Unbiased rejection sampling; earlier words form higher-order chunks.
  nextInt(below upper: Int) -> Int {
    if upper <= 0 { ArgumentError.new("Generator.nextInt: upper bound must be positive").raise() }
    if upper == 1 { return 0 }
    let bits = (upper - 1).bitLength
    let words = (bits + 63) ~/ 64
    let headBits = bits - ((words - 1) * 64)
    let mut candidate = upper
    while candidate >= upper {
      candidate = self.nextU64
      if headBits < 64 { candidate = candidate & ((1 << headBits) - 1) }
      let mut index = 1
      while index < words {
        candidate = (candidate << 64) | self.nextU64
        index = index + 1
      }
    }
    candidate
  }

  nextInt(from lower: Int, below upper: Int) -> Int {
    if upper <= lower { ArgumentError.new("Generator.nextInt: upper bound must be greater than lower bound").raise() }
    lower + self.nextInt(below: upper - lower)
  }

  /// Big-endian word extraction; discard unused bytes of the final word.
  fill(_ out: Bytes) -> Unit {
    let mut offset = 0
    while offset < out.size {
      let word = self.nextU64
      let mut index = 0
      while index < 8 and offset < out.size {
        out.set(offset, (word >> (56 - index * 8)) & 255)
        offset = offset + 1
        index = index + 1
      }
    }
    ()
  }

  /// Deterministic pseudorandom bytes, unsuitable for secrets.
  bytes(_ count: Int) -> Bytes {
    if count < 0 { ArgumentError.new("Generator.bytes: count must be non-negative").raise() }
    let out = Bytes.new(count)
    self.fill(out)
    out
  }

  choose<T>(_ values: List<T>) -> Option<T> {
    if values.size == 0 { return Option<T>::None }
    if values.size == 1 { return Option<T>::Some(values.at(0)) }
    Option<T>::Some(values.at(self.nextInt(below: values.size)))
  }

  /// Descending unbiased Fisher–Yates, mutating the supplied list in place.
  shuffle<T>(_ values: List<T>) -> Unit {
    if values.size <= 1 { return () }
    let mut index = values.size - 1
    while index > 0 {
      let other = self.nextInt(below: index + 1)
      values.swap(first: index, second: other).unwrap
      index = index - 1
    }
    ()
  }
}

export Generator
