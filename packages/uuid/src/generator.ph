from universe.collections.bytes import Bytes
from universe.random import SecureRandom
from universe.time.timestamp import Timestamp
from .uuid import UUID
from .errors import UUIDGenerationError
from .typed import UUIDv7

/// Strictly increasing UUIDv7 values per instance, for sequential callers.
/// Clock rollback and exhaustion raise instead of emitting out-of-order IDs.
class UUIDv7Generator {
  mut _lastMillis: Int
  mut _lastTail: Int

  @constructor
  private create() {
    self._lastMillis = -1
    self._lastTail = -1
  }

  @class
  monotonic() -> UUIDv7Generator { UUIDv7Generator.create() }

  generate() -> UUID {
    let millis = Timestamp.now.unixNanoseconds ~/ 1000000
    self.generateAt(millis)
  }

  /// The monotonic generator always constructs a checked RFC UUIDv7.
  generateTyped() -> UUIDv7 { UUIDv7.fromUUID(self.generate()).unwrap }

  /// Explicit time input permits reproducible testing of rollback and overflow.
  internal generateAt(_ millis: Int) -> UUID {
    if millis < 0 or millis > 0xffffffffffff {
      throw UUIDGenerationError.new("UUIDv7 timestamp exceeds 48-bit Unix milliseconds")
    }
    if millis < self._lastMillis { throw UUIDGenerationError.new("UUIDv7 clock moved backward") }
    if millis == self._lastMillis and self._lastTail == 0x3ffffffffffffffffff {
      throw UUIDGenerationError.new("UUIDv7 monotonic sequence exhausted")
    }

    let tail = if millis == self._lastMillis {
      self._lastTail + 1
    } else {
      let entropy = SecureRandom.system.bytes(10)
      if entropy.size != 10 { throw UUIDGenerationError.new("UUIDv7 entropy length mismatch") }
      let mut seed = 0
      let mut i = 0
      while i < 10 {
        seed = (seed << 8) | entropy.at(i)
        i = i + 1
      }
      seed & 0x3ffffffffffffffffff
    }

    let random = Bytes.new(10)
    let mut i = 9
    while i >= 0 {
      random.set(i, (tail >> ((9 - i) * 8)) & 255)
      i = i - 1
    }
    let id = UUID.fromV7Parts(millis, random)
    self._lastMillis = millis
    self._lastTail = tail
    id
  }
}

export UUIDv7Generator
