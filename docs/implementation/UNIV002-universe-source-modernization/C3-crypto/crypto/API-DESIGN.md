# API-DESIGN: Cryptographic Foundations and Standalone Nano ID

**Program:** Telos Cryptographic Foundations (`UNIV002.C3`)  
**Packages:**  
- `packages/universe/src/crypto/` (`universe.crypto`)
- `packages/nanoid/` (`nanoid`)  

---

## 1. Domain Organization

Cryptography is separated from general-purpose mathematics (`universe.math`) and pseudo-random simulation (`universe.random`). The new domain `universe.crypto` owns cryptographically secure operations backed by host operating-system entropy.

```text
universe
  └── crypto
        ├── package.ph            (facade: re-exports SecureRandom, EntropyError)
        └── random
              └── package.ph      (OS-backed secure byte services)
```

Applications and independent libraries access high-level identifier generation through the standalone `nanoid` package:

```text
nanoid
  ├── errors.ph                   (InvalidAlphabetError, InvalidIdLengthError, EntropyError)
  ├── alphabet.ph                 (Alphabet)
  ├── generator.ph                (NanoIdGenerator)
  ├── id.ph                       (NanoId, Id<Tag>)
  └── package.ph                  (public re-exports)
```

---

## 2. `universe.crypto` Package Surface

### 2.1 `universe.crypto.random`
```telos
from universe.random.secure import SecureRandom, EntropyError

export SecureRandom, EntropyError
```

- Reuses the canonical native-backed `SecureRandom` class identity defined in `universe.random.secure`.
- No competing duplicate random service class is introduced.

### 2.2 `universe.crypto` Facade
```telos
expose .random

from .random import SecureRandom, EntropyError

export SecureRandom, EntropyError
```

Allows callers to import directly from `universe.crypto`:
```telos
from universe.crypto import SecureRandom
let bytes = SecureRandom.system.bytes(32)
```

---

## 3. `nanoid` Public API Reference

### 3.1 `Alphabet`

An immutable descriptor representing a validated set of distinct printable ASCII characters.

```telos
data Alphabet private(
  _ symbols: String,
  _ size: Int
)

impl Alphabet {
  /// Default 64-symbol URL-safe alphabet:
  /// "57X6y3wQZNzYjJB2L_10pxbackgroundhimselfUTF-8DEMOGRAPHICSVqvt4KW9"
  @class urlSafe -> Alphabet

  /// Validates and constructs an alphabet from distinct printable ASCII characters.
  /// Rejects empty strings, sizes < 2 or > 94, duplicate characters, and non-ASCII codes.
  @class new(_ symbols: String) -> Alphabet

  symbols -> String
  size -> Int
  byteCount -> Int
}
```

### 3.2 `NanoIdGenerator`

A reusable generator encapsulating an alphabet, precalculated bit-mask, and batching heuristics.

```telos
class NanoIdGenerator {
  /// Returns the shared process-wide generator using Alphabet.urlSafe and length 21.
  @class default -> NanoIdGenerator

  /// Calculates the smallest bit-mask (2^bits - 1) covering alphabetSize.
  @class maskFor(_ alphabetSize: Int) -> Int

  /// Constructs a generator for the given alphabet and default length.
  @class new(_ alphabet: Alphabet, _ size: Int) -> NanoIdGenerator

  alphabet -> Alphabet
  size -> Int

  /// Generates an identifier using the configured default size.
  generate() -> String

  /// Generates an identifier with an explicit length override.
  generate(_ length: Int) -> String
}
```

### 3.3 `NanoId` Convenience Facade

Static class methods for rapid, unconfigured identifier generation.

```telos
class NanoId {
  /// Generates a 21-character URL-safe identifier (126 bits of entropy).
  @class generate() -> String

  /// Generates a URL-safe identifier of specified length.
  @class generate(_ size: Int) -> String

  /// Generates a phantom-tagged typed identifier with default length.
  @class typed<Tag>() -> Id<Tag>

  /// Generates a phantom-tagged typed identifier with specified length.
  @class typed<Tag>(_ size: Int) -> Id<Tag>
}
```

### 3.4 `Id<Tag>` Strongly-Typed Identifiers

Phantom-tagged wrapper preventing accidental domain cross-contamination at compile time.

```telos
data Id<Tag> private(
  _ value: String
)

impl<Tag> Id<Tag> {
  value -> String
  toString -> String

  @class wrap(_ value: String) -> Id<Tag>

  /// Validates external text against an alphabet and expected length.
  @class parse(_ text: String, _ alphabet: Alphabet, _ expectedLength: Int) -> Result<Id<Tag>, Error>
}
```

### 3.5 Error Model

- `InvalidAlphabetError is Error`: Raised when an alphabet string has size outside `2..94`, contains non-printable ASCII, or contains duplicate characters.
- `InvalidIdLengthError is Error`: Raised when a requested identifier length is negative or exceeds `1,048,576` bytes.
- `EntropyError is Error`: Re-exported from `universe.random` for OS-level entropy acquisition failures.
