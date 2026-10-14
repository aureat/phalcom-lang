from universe.collections.bytes import Bytes

/// SHA-1 one-shot binary digest. Provided for standards such as UUIDv5;
/// SHA-1 is not appropriate for new collision-resistant protocols.
class SHA1 {
  @class
  private rotate(_ word: Int, _ count: Int) -> Int {
    ((word << count) | (word >> (32 - count))) & 0xffffffff
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

    let mut h0 = 0x67452301
    let mut h1 = 0xefcdab89
    let mut h2 = 0x98badcfe
    let mut h3 = 0x10325476
    let mut h4 = 0xc3d2e1f0
    let mut offset = 0
    while offset < paddedLength {
      let words: List<Int> = []
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

      let mut a = h0
      let mut b = h1
      let mut c = h2
      let mut d = h3
      let mut e = h4
      t = 0
      while t < 80 {
        let f = if t < 20 { (b & c) | ((~b) & d) } else if t < 40 { b ^ c ^ d } else if t < 60 { (b & c) | (b & d) | (c & d) } else { b ^ c ^ d }
        let k = if t < 20 { 0x5a827999 } else if t < 40 { 0x6ed9eba1 } else if t < 60 { 0x8f1bbcdc } else { 0xca62c1d6 }
        let temp = (SHA1.rotate(a, 5) + f + e + k + words.at(t)) & 0xffffffff
        e = d
        d = c
        c = SHA1.rotate(b, 30)
        b = a
        a = temp
        t = t + 1
      }
      h0 = (h0 + a) & 0xffffffff
      h1 = (h1 + b) & 0xffffffff
      h2 = (h2 + c) & 0xffffffff
      h3 = (h3 + d) & 0xffffffff
      h4 = (h4 + e) & 0xffffffff
      offset = offset + 64
    }

    let digest = Bytes.new(20)
    SHA1.writeWord(digest, 0, h0)
    SHA1.writeWord(digest, 4, h1)
    SHA1.writeWord(digest, 8, h2)
    SHA1.writeWord(digest, 12, h3)
    SHA1.writeWord(digest, 16, h4)
    digest
  }
}

export SHA1
