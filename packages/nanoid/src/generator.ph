from universe.collections.bytes import Bytes
from universe.random import SecureRandom, EntropyError
from .alphabet import Alphabet
from .errors import InvalidIdLengthError

/// Maximum supported identifier length to prevent unbounded memory allocation (1 MB).
const MAX_ID_SIZE = 1048576

/// Maximum entropy buffer requested per batch (4 KB).
const MAX_BATCH_SIZE = 4096

/// Reusable cryptographically secure random identifier generator.
class NanoIdGenerator {
  _alphabet: Alphabet
  _alphabetBytes: Bytes
  _size: Int
  _mask: Int
  _isPowerOfTwo: Bool

  @class mut _default

  @constructor
  private create(
    _ alphabet: Alphabet,
    _ size: Int,
    _ mask: Int,
    _ isPowerOfTwo: Bool
  ) {
    self._alphabet = alphabet
    self._alphabetBytes = Bytes.fromString(alphabet.symbols)
    self._size = size
    self._mask = mask
    self._isPowerOfTwo = isPowerOfTwo
  }

  @class 
  default -> NanoIdGenerator {
    if _default == None {
      _default = NanoIdGenerator.new(Alphabet.urlSafe, 21)
    }
    _default
  }

  /// Calculates the smallest bit mask (2^bits - 1) covering all alphabet indexes.
  @class
  maskFor(_ alphabetSize: Int) -> Int {
    let mut mask = 1
    while mask < alphabetSize - 1 {
      mask = (mask << 1) | 1
    }
    mask
  }

  /// Constructs a reusable generator for the given alphabet and default size.
  @class
  new(_ alphabet: Alphabet, _ size: Int) -> NanoIdGenerator {
    if size < 0 {
      throw InvalidIdLengthError.new("NanoIdGenerator: size must be non-negative, got: \(size)")
    }
    if size > MAX_ID_SIZE {
      throw InvalidIdLengthError.new("NanoIdGenerator: size exceeds maximum supported limit (\(MAX_ID_SIZE)), got: \(size)")
    }

    let mask = NanoIdGenerator.maskFor(alphabet.size)
    let isPowerOfTwo = alphabet.size == mask + 1

    NanoIdGenerator.create(alphabet, size, mask, isPowerOfTwo)
  }

  alphabet -> Alphabet { _alphabet }
  size -> Int { _size }

  /// Calculates the entropy batch size for remaining characters.
  internal batchSizeFor(_ remaining: Int) -> Int {
    if _isPowerOfTwo {
      if remaining > MAX_BATCH_SIZE { MAX_BATCH_SIZE } else { remaining }
    } else {
      // 1.6 multiplier over expected bytes to cover rejections
      let step = (((remaining * (_mask + 1) * 8) ~/ (_alphabet.size * 5)) + 1)
      if step > MAX_BATCH_SIZE { MAX_BATCH_SIZE } else if step < 1 { 1 } else { step }
    }
  }

  /// Generates an identifier using the configured default size.
  generate() -> String {
    self.generate(_size)
  }

  /// Generates an identifier with an explicitly requested size.
  generate(_ length: Int) -> String {
    if length < 0 {
      throw InvalidIdLengthError.new("NanoIdGenerator#generate: length must be non-negative, got: \(length)")
    }
    if length > MAX_ID_SIZE {
      throw InvalidIdLengthError.new("NanoIdGenerator#generate: length exceeds maximum supported limit (\(MAX_ID_SIZE)), got: \(length)")
    }
    if length == 0 {
      return ""
    }

    let output = Bytes.new(length)
    let random = SecureRandom.system
    let mut written = 0

    while written < length {
      let remaining = length - written
      let batchSize = self.batchSizeFor(remaining)
      let entropy = random.bytes(batchSize)
      let entropyLen = entropy.size

      if _isPowerOfTwo {
        let mut i = 0
        while i < entropyLen and written < length {
          let b = entropy.at(i)
          let index = b & _mask
          output.set(written, _alphabetBytes.at(index))
          written = written + 1
          i = i + 1
        }
      } else {
        let mut i = 0
        while i < entropyLen and written < length {
          let b = entropy.at(i)
          let index = b & _mask
          if index < _alphabet.size {
            output.set(written, _alphabetBytes.at(index))
            written = written + 1
          }
          i = i + 1
        }
      }
    }

    match output.utf8 {
      Option::Some(id) => id
      Option::None => {
        throw Error.new("NanoId invariant violated: output is not valid UTF-8")
      }
    }
  }

}

export NanoIdGenerator
