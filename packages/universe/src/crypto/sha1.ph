from universe.collections.bytes import Bytes

/// SHA-1 one-shot binary digest. Provided for standards such as UUIDv5;
/// SHA-1 is not appropriate for new collision-resistant protocols.
class SHA1 {
  @class
  private rotate(_ word: Int, _ count: Int) -> Int {
    ((word << count) | (word >> (32 - count))) & 0xffffffff
  }

  @class
  private roundFunction(_ t: Int, _ b: Int, _ c: Int, _ d: Int) -> Int {
    if t < 20 { return (b & c) | ((~b) & d) }
    if t < 40 { return b ^ c ^ d }
    if t < 60 { return (b & c) | (b & d) | (c & d) }
    b ^ c ^ d
  }

  @class
  private roundConstant(_ t: Int) -> Int {
    if t < 20 { return 0x5a827999 }
    if t < 40 { return 0x6ed9eba1 }
    if t < 60 { return 0x8f1bbcdc }
    0xca62c1d6
  }

  @class
  private schedule(_ padded: Bytes, _ offset: Int) -> List<Int> {
    let words = List<Int>.new()
    let mut t = 0
    while t < 16 {
      let start = offset + t * 4
      words.append((padded.at(start) << 24) | (padded.at(start + 1) << 16) |
        (padded.at(start + 2) << 8) | padded.at(start + 3))
      t = t + 1
    }
    while t < 80 {
      words.append(SHA1.rotate(words.at(t - 3) ^ words.at(t - 8) ^
        words.at(t - 14) ^ words.at(t - 16), 1))
      t = t + 1
    }
    words
  }

  @class
  private compress(_ padded: Bytes, _ offset: Int, _ h: (Int, Int, Int, Int, Int)) -> (Int, Int, Int, Int, Int) {
    let words = SHA1.schedule(padded, offset)
    let mut a = h[0]
    let mut b = h[1]
    let mut c = h[2]
    let mut d = h[3]
    let mut e = h[4]
    let mut t = 0
    while t < 80 {
      let f = SHA1.roundFunction(t, b, c, d)
      let k = SHA1.roundConstant(t)
      let temp = (SHA1.rotate(a, 5) + f + e + k + words.at(t)) & 0xffffffff
      e = d
      d = c
      c = SHA1.rotate(b, 30)
      b = a
      a = temp
      t = t + 1
    }
    (
      (h[0] + a) & 0xffffffff,
      (h[1] + b) & 0xffffffff,
      (h[2] + c) & 0xffffffff,
      (h[3] + d) & 0xffffffff,
      (h[4] + e) & 0xffffffff
    )
  }

  @class
  private writeWord(_ out: Bytes, _ offset: Int, _ word: Int) -> Unit {
    let mut i = 0
    while i < 4 {
      out.set(offset + i, (word >> ((3 - i) * 8)) & 255)
      i = i + 1
    }
  }

  /// Returns 20 digest octets for the supplied bytes, without hex encoding.
  @class
  hash(_ input: Bytes) -> Bytes {
    let length = input.size
    if length >= (1 << 61) {
      throw ArgumentError.new("SHA-1 message exceeds the 64-bit length field")
    }
    let paddedLength = ((length + 9 + 63) ~/ 64) * 64
    let padded = Bytes.new(paddedLength)
    input.copyInto(padded, 0)
    padded.set(length, 0x80)
    let bitLength = length * 8
    let mut i = 0
    while i < 8 {
      padded.set(paddedLength - 8 + i, (bitLength >> ((7 - i) * 8)) & 255)
      i = i + 1
    }

    let mut h = (0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0)
    let mut offset = 0
    while offset < paddedLength {
      h = SHA1.compress(padded, offset, h)
      offset = offset + 64
    }

    let digest = Bytes.new(20)
    SHA1.writeWord(digest, 0, h[0])
    SHA1.writeWord(digest, 4, h[1])
    SHA1.writeWord(digest, 8, h[2])
    SHA1.writeWord(digest, 12, h[3])
    SHA1.writeWord(digest, 16, h[4])
    digest
  }
}

export SHA1
