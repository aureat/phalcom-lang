# DESIGN AND IMPLEMENTATION PLAN
# `universe.crypto` — Native Cryptographic Foundations and Standalone Nano ID

**Program:** Telos Cryptographic Foundations  
**Target branch:** `feat/telos-collections`  
**Inspected revision:** `18aaf8b568a0b45dd450befc35932bb7a1fdd442`  
**Universe source:** `packages/universe/src/`  
**Standalone package:** `packages/nanoid/`  
**Implementation strategy:** Native cryptographic foundations, Telos-authored APIs and algorithms  
**Production integration:** Separately gated by the ongoing Universe migration

---

# 1. Architectural Objective

Introduce a coherent `universe.crypto` package providing secure, efficient cryptographic primitives implemented by the native Rust host.

The package must enable ordinary Telos libraries to implement security-sensitive functionality without directly binding to operating-system APIs, exposing cryptographic implementation details, or implementing complex cryptographic algorithms incorrectly in user code.

The initial motivating application is a standalone Nano ID implementation:

```text
packages/nanoid/
```

It must be possible to generate secure, URL-friendly identifiers through an API such as:

```telos
from nanoid import NanoId

let id = NanoId.generate()
let shortId = NanoId.generate(12)
```

And configure reusable custom generators:

```telos
from nanoid import Alphabet, NanoIdGenerator

let alphabet = Alphabet.new("0123456789abcdef")

let generator = NanoIdGenerator.new(
  alphabet: alphabet,
  size: 16
)

let id = generator.generate()
```

These examples describe the target API. Exact constructor selectors and imports must be verified against the current Telos compiler.

The final implementation must provide:

1. Cryptographically secure random bytes.
2. Reliable host entropy acquisition.
3. Explicit failure semantics.
4. Efficient byte-buffer operations.
5. Unbiased random character selection.
6. Configurable identifier length and alphabet.
7. Efficient ASCII string construction.
8. Safe concurrent generation.
9. A clean, reusable Telos API.
10. A standalone package without Nano ID-specific runtime dependencies.

The implementation must not require cryptographic functionality to be added directly to `System`.

---

# 2. Architectural Decisions

## D1 — Cryptography is a distinct Universe domain

Introduce:

```text
universe.crypto
```

Cryptographic operations do not belong directly under:

```text
universe.math
universe.collections
universe.system
```

Although cryptography involves mathematics and byte manipulation, its security requirements justify a distinct API domain.

## D2 — Reuse the existing secure entropy implementation

The repository already contains:

```text
packages/universe/src/random/secure.ph

phalcom-core/src/primitive/random.rs
```

The existing API is:

```telos
class SecureRandom {
  internal native primitive rawFill(_ bytes: Bytes) -> Option<String>

  fill(_ bytes: Bytes) -> Unit

  bytes(_ count: Int) -> Bytes
}
```

The native implementation invokes:

```rust
getrandom::fill(bytes)
```

This is a suitable secure randomness foundation.

**Do not implement a second OS entropy provider.**

The new cryptographic package should initially reuse this implementation.

## D3 — Separate deterministic randomness from cryptographic randomness

The repository also provides:

```text
universe.random.Generator
universe.random.Xoshiro256PlusPlus
universe.random.UniformInt
universe.random.Bernoulli
```

The deterministic generator is useful for simulation, procedural generation, reproducible testing, and numerical algorithms.

It is not suitable for security-sensitive identifiers.

The distinction must remain explicit:

```text
universe.random
    Deterministic generators
    Reproducible sequences
    Statistical distributions

universe.crypto
    Cryptographically secure randomness
    Cryptographic primitives
    Security-sensitive operations
```

Existing `universe.random.SecureRandom` consumers must remain supported.

## D4 — Native code implements mechanisms, Telos implements policies

The native host should own operations that require:

- OS integration.
- Verified cryptographic implementations.
- Efficient bulk byte manipulation.
- Constant-time behavior where required.
- Direct access to native cryptographic libraries.

Telos should own:

- Public API composition.
- Parameter validation.
- Typed configuration.
- Algebraic error handling where appropriate.
- Identifier-generation policies.
- Alphabet configuration.
- Generic integration.
- Higher-level package abstractions.

Do not implement cryptographic algorithms in Telos merely to avoid native code.

Conversely, do not move ordinary package logic into Rust merely because doing so is convenient.

## D5 — Nano ID remains a standalone package

Do not introduce:

```text
universe.crypto.nanoid
```

Instead:

```text
packages/nanoid/
```

The dependency relationship is:

```text
nanoid
    ↓
universe.crypto
    ↓
SecureRandom
    ↓
Rust getrandom
    ↓
Operating-system CSPRNG
```

The Nano ID package must not depend on `phalcom-core` or native Rust crates directly.

## D6 — Correctness precedes acceleration

Implement a correct Telos Nano ID algorithm first.

Profile it using the existing performance instrumentation.

If necessary, add a reusable native bulk operation to eliminate excessive VM boundary crossings.

Do not introduce Nano ID-specific compiler instructions.

---

# 3. Current Repository Findings

The following implementation already exists.

| Component | Location | Current behavior |
|---|---|---|
| Secure randomness API | `packages/universe/src/random/secure.ph` | OS-backed random bytes |
| Native randomness | `phalcom-core/src/primitive/random.rs` | `getrandom::fill` |
| Native class metadata | `phalcom-native-meta/src/universe.rs` | `UniverseKey::SecureRandom` |
| Mutable byte buffers | `packages/universe/src/collections/bytes.ph` | Byte allocation and mutation |
| Native byte operations | `phalcom-core/src/primitive/bytes.rs` | Bulk and indexed operations |
| UTF-8 decoding | `Bytes.utf8` | Converts valid bytes to String |
| Constant-time comparison | `Bytes.rawEqualsConstantTime` | Existing native primitive |
| Deterministic PRNG | `random/xoshiro256pp.ph` | Not cryptographically secure |

The existing native randomness primitive is registered using:

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
```

This is already a useful native architecture.

Do not replace it without a demonstrated correctness or performance requirement.

The existing Rust dependency is:

```toml
getrandom = "0.3.3"
```

Reuse the existing dependency rather than introducing another randomness library.

---

# 4. Target Universe Organization

Introduce the cryptographic domain incrementally.

```text
packages/universe/
└── src/
    ├── package.ph
    │
    ├── random/
    │   ├── package.ph
    │   ├── generator.ph
    │   ├── secure.ph
    │   └── ...
    │
    └── crypto/
        ├── package.ph
        │
        ├── random/
        │   └── package.ph
        │
        ├── hash/
        │   └── package.ph
        │
        ├── mac/
        │   └── package.ph
        │
        ├── kdf/
        │   └── package.ph
        │
        ├── aead/
        │   └── package.ph
        │
        └── signatures/
            └── package.ph
```

Only create concrete packages as their implementation begins.

Do not introduce empty cryptographic packages merely to mirror the target diagram.

The initial deliverable should contain:

```text
crypto/
├── package.ph
└── random/
    └── package.ph
```

Additional domains are future implementation stages.

---

# 5. Secure Randomness API

## 5.1 Preserve the canonical SecureRandom identity

Initially retain:

```text
universe.random.secure::SecureRandom
```

Expose the same declaration through:

```text
universe.crypto.random
```

and:

```text
universe.crypto
```

All import paths must resolve to the same canonical declaration.

Do not define another:

```telos
class SecureRandom
```

inside `universe.crypto`.

That would introduce competing secure-random service identities.

## 5.2 Proposed cryptographic package facade

Conceptually:

```telos
//! Cryptographic primitives and secure host services.
//!
//! This package exposes security-sensitive operations implemented
//! through the native host. It does not expose deterministic PRNGs.

expose .random

from .random import SecureRandom, EntropyError

export SecureRandom, EntropyError
```

The random package facade:

```telos
//! Cryptographically secure random byte generation.
//!
//! Randomness is acquired from the operating system.
//! Failure never causes a deterministic fallback.

from universe.random.secure import SecureRandom, EntropyError

export SecureRandom, EntropyError
```

The actual import/re-export implementation must respect current module interface and initialization semantics.

Do not assume that public re-exporting requires eagerly executing unrelated Universe packages.

## 5.3 Public usage

```telos
from universe.crypto import SecureRandom

let random = SecureRandom.system

let bytes = random.bytes(32)
```

Or:

```telos
let buffer = Bytes.new(64)

SecureRandom.system.fill(buffer)
```

Both operations must acquire cryptographically suitable bytes.

---

# 6. Native Host Requirements

## 6.1 Entropy source

Use:

```rust
getrandom::fill(&mut bytes)
```

Do not use:

```rust
rand::random()
```

without establishing that the selected source is a cryptographically secure generator with the required semantics.

Never use:

```text
Xoshiro256PlusPlus
Linear congruential generators
Time-based seeds
Process identifiers
Math.random equivalents
```

as a fallback for cryptographic randomness.

## 6.2 Failure semantics

Entropy acquisition can fail.

The existing implementation returns an optional error message from the native primitive, and the Telos wrapper raises `EntropyError`.

Preserve this behavior during the initial integration.

Conceptually:

```text
SecureRandom.fill(bytes)
    ↓
Host getrandom::fill
    ├── success → Unit
    └── failure → EntropyError
```

If the host partially modifies the destination buffer before failing, the buffer's contents are unspecified.

No successful result may be returned from that failed operation.

## 6.3 Resource limits

Every public allocation API must validate:

- Negative lengths.
- Integer overflow.
- Maximum supported buffer size.
- Host allocation failures.
- VM buffer limits.

Do not rely on Rust integer conversion truncation.

For example:

```telos
SecureRandom.system.bytes(-1)
```

must fail predictably.

## 6.4 Native effects

Secure random acquisition is:

```text
Nondeterministic
Potentially blocking
Potentially failing
```

Filling an existing buffer is additionally:

```text
Mutating
```

Native metadata must accurately describe these effects.

Do not mark a native entropy operation pure.

Do not allow constant evaluation to execute an entropy-producing primitive.

## 6.5 Concurrency

The underlying operating-system entropy facility must be safe for concurrent requests.

The wrapper must not accidentally share mutable output buffers between callers.

Two concurrent calls should produce independently owned buffers.

Do not introduce a global mutable byte pool without synchronization, ownership rules, and security review.

---

# 7. Cryptographic API Design Beyond Randomness

Although Nano ID initially needs only secure randomness, the architecture should support future cryptographic functionality.

Recommended domains:

| Domain | Operations | Native implementation |
|---|---|---|
| `crypto.random` | Secure random bytes | `getrandom` |
| `crypto.hash` | SHA-256, SHA-512, SHA-3, BLAKE3 | Established cryptographic libraries |
| `crypto.mac` | HMAC | Established cryptographic libraries |
| `crypto.kdf` | HKDF, approved password KDFs | Established cryptographic libraries |
| `crypto.aead` | Authenticated encryption | Established cryptographic libraries |
| `crypto.signatures` | Signing and verification | Established cryptographic libraries |

These are future modules, not requirements for the first Nano ID deliverable.

Do not design a single monolithic `Crypto` class containing every algorithm.

## 7.1 Hashing

Future usage might be:

```telos
from universe.crypto.hash import SHA256

let digest = SHA256.digest(data)
```

The actual native implementation should use a vetted cryptographic library.

Do not introduce a custom SHA-256 implementation without an independently justified reason.

## 7.2 Cryptographic state

Distinguish:

```text
Hash algorithm
Hash computation state
Final digest value
```

A streaming hasher is stateful.

It should not be modeled as a transparent immutable data product.

A completed digest is logically an immutable value.

Its representation should not expose a mutable `Bytes` buffer as though that buffer were deeply immutable.

## 7.3 Authenticated encryption

Do not introduce unauthenticated encryption as the default API.

Future encryption APIs should use authenticated encryption with explicitly specified:

- Algorithms.
- Nonce handling.
- Key ownership.
- Failure semantics.
- Associated data.
- Memory lifecycle.

Nonce uniqueness must be established by the algorithm's contract.

**A randomly generated Nano ID must not be treated as a universally safe cryptographic nonce.**

## 7.4 Constant-time comparisons

The repository already contains native constant-time byte comparison support.

Investigate reusing it for:

```text
crypto.timingSafeEqual
```

Do not create a second comparison algorithm without a reason.

Document whether differing lengths are observable.

The implementation must not claim that length itself is concealed unless that behavior is explicitly provided and tested.

---

# 8. Standalone Nano ID Package

## 8.1 Proposed package structure

```text
packages/nanoid/
├── project.toml
│
├── src/
│   ├── package.ph
│   ├── alphabet.ph
│   ├── generator.ph
│   ├── id.ph
│   └── errors.ph
│
├── tests/
│   ├── basic.ph
│   ├── alphabets.ph
│   ├── distribution.ph
│   ├── boundaries.ph
│   ├── error_handling.ph
│   └── typed_ids.ph
│
├── benchmarks/
│   ├── default.ph
│   ├── custom_alphabet.ph
│   ├── batch.ph
│   └── throughput.ph
│
└── README.md
```

Use the repository's established package manifest schema:

```toml
[project]
name = "nanoid"
namespace = "nanoid"
version = "0.1.0"
```

The package must be ordinary Telos source.

No special native package registration is permitted.

## 8.2 Public API

Recommended primary exports:

```text
NanoId
NanoIdGenerator
Alphabet
NanoIdError
```

Recommended operations:

```telos
NanoId.generate()

NanoId.generate(21)

NanoIdGenerator.new(
  alphabet: Alphabet.urlSafe,
  size: 21
)

generator.generate()

generator.generate(12)
```

The default ID is an ordinary `String`.

Do not require callers to use a branded or wrapped value unless they explicitly request type-safe identifiers.

## 8.3 Default length

Use:

```text
21 characters
```

Nano ID's conventional default alphabet has 64 symbols.

Therefore:

```text
21 × log₂(64) = 126 bits
```

The default format provides 126 bits of random-choice space when the underlying selection is unbiased.

This does not guarantee global uniqueness.

It also does not guarantee security if an application leaks the generated IDs, chooses a predictable alphabet, or uses IDs in a context requiring additional authentication.

---

# 9. Alphabet Representation

The alphabet is central to both performance and security.

## 9.1 Do not treat an alphabet as an arbitrary string without validation

Invalid alphabets may contain:

- No symbols.
- Duplicate symbols.
- Characters unsupported by the fast encoder.
- Control characters.
- Non-ASCII characters.
- An excessive number of symbols.

A generator must not silently accept an invalid alphabet.

## 9.2 Initial alphabet contract

For the first optimized implementation, support unique printable ASCII characters.

Recommended constraints:

```text
Alphabet size: 2..94 characters
Character range: printable ASCII, 0x21..0x7E
Duplicates: forbidden
```

This is an intentional, narrower API than the JavaScript reference implementation's general 1–256-symbol alphabet.

The advantage is a clear fast path:

```text
One alphabet symbol = one byte = one output character
```

Support for arbitrary Unicode alphabets can be added separately.

Do not silently treat UTF-8 bytes as Unicode characters.

## 9.3 Proposed immutable alphabet descriptor

Conceptually:

```telos
//! Validated alphabets for random identifier generation.

/// A validated set of distinct printable ASCII symbols.
///
/// The constructor is private so invalid alphabets cannot
/// be created through the ordinary public API.
data Alphabet private(
  symbols: String
)

impl Alphabet {
  @class
  new(_ symbols: String) -> Alphabet {
    // Validate ASCII range, uniqueness, and cardinality.
    // Construct the canonical alphabet value.
  }

  @class
  urlSafe -> Alphabet {
    Alphabet.new(URL_ALPHABET)
  }

  size -> Int {
    symbols.byteCount
  }
}

export Alphabet
```

This is a proposed design.

Validate private data constructor support and method syntax before implementation.

### Important representation decision

The immutable `Alphabet` value should contain the logical alphabet.

It should not expose a mutable `Bytes` array as an immutable data component.

A generator may privately cache an encoded byte lookup table for performance.

That cache belongs to the generator implementation, not the immutable semantic value.

---

# 10. Nano ID Randomness Algorithm

The algorithm must generate uniformly distributed symbols.

## 10.1 The modulo-bias problem

Suppose the alphabet has 62 symbols.

An incorrect implementation is:

```text
index = randomByte % 62
```

A random byte has 256 possible values.

Since:

```text
256 = 4 × 62 + 8
```

the first eight alphabet indexes occur five times each, while the remaining indexes occur four times each.

This introduces bias.

For security-sensitive identifiers, avoid that bias.

## 10.2 Use rejection sampling

Choose the smallest all-one bit mask covering the alphabet indexes.

For an alphabet of length `N`:

```text
bits = ceil(log₂(N))

mask = 2^bits - 1
```

For 62 symbols:

```text
bits = 6
mask = 63
```

Then:

```text
candidate = randomByte & mask

if candidate < N:
    accept candidate
else:
    reject candidate
```

Every accepted index has the same probability.

For the 62-symbol alphabet:

```text
Accepted: 0..61
Rejected: 62..63
```

Acceptance probability:

```text
62 / 64 = 96.875%
```

The expected number of random bytes required per character is:

```text
64 / 62 ≈ 1.0323
```

This is efficient.

## 10.3 Special case: power-of-two alphabets

For:

```text
N = 64
```

the mask is:

```text
63
```

Every candidate is valid.

No rejection is required.

Each random byte produces one output character:

```text
index = randomByte & 63
```

This is the preferred fast path for the default Nano ID generator.

## 10.4 Preserve cryptographic uniformity

Do not use:

```text
Random integer modulo N
Floating-point multiplication
Time-based seed
Deterministic PRNG
```

unless the algorithm explicitly proves unbiased secure sampling.

The proposed bit-mask rejection method is straightforward to verify and suitable for Nano ID.

---

# 11. Nano ID Generation — Telos Reference Algorithm

The first implementation should be written in Telos using the existing cryptographic random-byte service.

The code below is algorithmic pseudocode in Telos style. It is not a claim that all selectors, constructors, and overloads already compile unchanged.

```telos
//! Secure random identifier generation.

from universe.crypto import SecureRandom
from universe.collections.bytes import Bytes

/// Reusable secure identifier generator.
///
/// Its configuration is fixed after construction.
/// Every generation obtains fresh cryptographic randomness.
class NanoIdGenerator {
  _alphabet: Alphabet
  _alphabetBytes: Bytes
  _size: Int
  _mask: Int

  @constructor
  private create(
    _ alphabet: Alphabet,
    _ size: Int,
    _ mask: Int
  ) {
    _alphabet = alphabet
    _alphabetBytes = Bytes.fromString(alphabet.symbols)
    _size = size
    _mask = mask
  }

  @class
  new(alphabet: Alphabet, size: Int) -> NanoIdGenerator {
    // Validate the requested identifier length.

    let mask = NanoIdGenerator.maskFor(alphabet.size)

    NanoIdGenerator.create(alphabet, size, mask)
  }

  /// Generates an identifier using the configured length.
  generate() -> String {
    self.generate(_size)
  }

  /// Generates an identifier with an explicit length.
  generate(_ length: Int) -> String {
    // Validate length and allocation limits.

    if length == 0 {
      return ""
    }

    let output = Bytes.new(length)
    let random = SecureRandom.system

    let mut written = 0

    while written < length {
      let remaining = length - written

      // Request entropy in bulk rather than making one
      // native randomness call per output character.
      let count = self.randomBatchSize(remaining)

      let entropy = random.bytes(count)

      for byte in entropy {
        let index = byte & _mask

        if index < _alphabet.size {
          output.set(written, _alphabetBytes[index])

          written = written + 1

          if written == length {
            break
          }
        }
      }
    }

    // Alphabet validation guarantees that every output byte
    // is printable ASCII and therefore valid UTF-8.
    match output.utf8 {
      Option::Some(identifier) => identifier

      Option::None => {
        throw Error.new(
          "NanoId invariant violated: output is not valid UTF-8"
        )
      }
    }
  }
}

export NanoIdGenerator
```

The final implementation should not store the encoded alphabet in a publicly mutable field.

If the runtime permits mutable `Bytes` to escape through reflection or other APIs, keep the lookup table strictly private.

---

# 12. Batch Entropy Acquisition

Cryptographic random-byte acquisition has nonzero overhead.

The implementation should not request one byte at a time.

## 12.1 Default alphabet

For 64 symbols, every byte is accepted.

Therefore:

```text
Requested random bytes = requested ID length
```

A 21-character identifier requires 21 random bytes.

No oversampling is necessary.

## 12.2 Custom alphabet

When rejection is possible, request extra bytes.

A reasonable initial estimate is:

```text
expectedBytes = ceil(
    remainingCharacters × (mask + 1) / alphabetLength
)
```

The implementation may request a modest surplus to reduce repeated host calls.

For example:

```text
batchSize = ceil(expectedBytes × 1.25)
```

The multiplier is a performance heuristic, not part of the security proof.

Its value should be selected through benchmarks.

## 12.3 Bound temporary allocations

Do not allocate an arbitrarily large entropy buffer merely because the caller requested a very long ID.

Use bounded chunks.

For example:

```text
Maximum entropy batch: 4096 bytes
```

Larger IDs are generated through repeated batches.

Verify allocation limits and integer overflow.

## 12.4 No reliance on lucky acceptance

A random batch may theoretically contain no accepted indexes.

The algorithm must continue requesting secure bytes until the requested number of characters is produced.

Do not return a shorter identifier.

Do not append invalid fallback characters.

Do not impose a fixed number of rejection attempts that silently biases output.

---

# 13. Efficient String Construction

This is an important performance issue.

## 13.1 Avoid repeated string concatenation

Do not generate IDs through:

```telos
let mut id = ""

for byte in randomBytes {
  id = id + alphabet[byte]
}
```

Repeated immutable string concatenation can introduce unnecessary allocations and potentially quadratic work.

## 13.2 Prefer byte-buffer assembly

Use:

```text
Allocate output Bytes once
    ↓
Write one ASCII byte per accepted index
    ↓
Decode output as UTF-8 once
    ↓
Return String
```

The current Universe already supports:

```telos
Bytes.new(size)

bytes.set(index, value)

bytes.utf8
```

This gives the first implementation an efficient, allocation-bounded output representation.

## 13.3 Avoid one native call per character where possible

The initial Telos loop may incur native method dispatch for:

```text
Bytes indexed read
Bytes indexed write
```

For a 21-character ID, this may be acceptable.

However, a high-throughput generator could become dominated by dispatch overhead rather than entropy generation.

Measure this before designing additional native primitives.

## 13.4 Do not use StringBuilder as the default hot path

The current `StringBuilder` implementation accumulates string chunks.

That is appropriate for general text construction.

For ASCII Nano ID generation, it may require more allocations and conversions than writing directly into one byte buffer.

Prefer `Bytes` until benchmarks demonstrate a better alternative.

---

# 14. Native Acceleration Strategy

The architecture should support a second implementation tier.

## 14.1 Tier 1 — Telos implementation

Use:

```text
SecureRandom.fill
Bytes
Bitwise operations
Rejection sampling
UTF-8 conversion
```

Advantages:

- Simple implementation.
- Ordinary Telos source.
- No new native class.
- No Nano ID-specific VM dependency.
- Easy algorithm verification.
- Reusable outside Nano ID.

This must be the initial implementation.

## 14.2 Tier 2 — Generic native ASCII sampling

If benchmarks show excessive VM dispatch overhead, introduce a reusable native operation that fills an output buffer with uniformly selected ASCII symbols.

Conceptual API:

```telos
SecureRandom.system.fillAscii(
  output,
  alphabet: alphabetBytes
)
```

It should:

1. Validate the alphabet.
2. Obtain OS-backed random bytes.
3. Perform unbiased index selection.
4. Fill the destination buffer.
5. Return only after complete success.
6. Raise an entropy error on host failure.

The native operation is not Nano ID-specific.

It can support:

- Random token generation.
- Temporary identifiers.
- Recovery-code alphabets.
- URL-safe random strings.
- Other alphabet-constrained cryptographic values.

However, callers must not assume that alphabet-constrained random strings automatically satisfy the requirements of passwords, authentication, or cryptographic nonces.

## 14.3 Preferred Rust implementation concept

The internal algorithm may resemble:

```rust
fn fill_uniform_ascii(
    output: &mut [u8],
    alphabet: &[u8],
) -> Result<(), CryptoError> {
    validate_alphabet(alphabet)?;

    let mask = alphabet_mask(alphabet.len());

    let mut written = 0;

    while written < output.len() {
        let mut random = [0u8; 4096];

        let needed = output.len() - written;
        let count = calculate_batch_size(
            needed,
            alphabet.len(),
            mask,
        );

        getrandom::fill(&mut random[..count])
            .map_err(CryptoError::Entropy)?;

        for byte in &random[..count] {
            let index = (byte & mask) as usize;

            if index < alphabet.len() {
                output[written] = alphabet[index];
                written += 1;

                if written == output.len() {
                    break;
                }
            }
        }
    }

    Ok(())
}
```

This is illustrative Rust pseudocode.

The final implementation must account for:

- Correct numeric conversions.
- Exact error types.
- VM allocation boundaries.
- Borrow lifetimes.
- GC safety.
- Input validation.
- Buffer size limits.
- Entropy acquisition failures.

The native primitive must be registered through the existing canonical descriptor and source-ownership system.

Do not install a hidden primitive with an unmatched source declaration.

## 14.4 Avoid duplicating entropy services

The accelerated path should reuse:

```text
getrandom
```

through the existing native randomness implementation.

Do not create an independently seeded process-global CSPRNG without a separate security and lifecycle design.

## 14.5 Native primitive ownership

Because `SecureRandom` currently belongs to:

```text
universe.random.secure
```

a new native member must be associated with that canonical declaration.

Do not create an unrelated second native class merely to make registration easier.

A later, separately approved source migration may relocate the canonical declaration to:

```text
universe.crypto.random
```

At that point, update:

- `UniverseKey::source_path()`.
- Native source indexes.
- Source provider mappings.
- Bootstrap associations.
- Prelude/export metadata.
- Tests and caches.

Do not perform that identity migration implicitly.

---

# 15. Typed Nano ID Values

Ordinary Nano IDs should remain usable as strings.

However, Telos's type system permits a stronger optional API.

## 15.1 Problem

Consider:

```telos
let userId = NanoId.generate()
let orderId = NanoId.generate()
```

Both are strings.

The compiler cannot distinguish them.

This permits accidental misuse:

```telos
loadUser(orderId)
```

For applications with several identifier domains, stronger typing is useful.

## 15.2 Phantom-type design

Introduce an optional typed identifier value.

Conceptually:

```telos
/// An identifier whose Tag distinguishes its logical domain.
///
/// The tag carries no required runtime payload.
data Id<Tag> private(
  value: String
)
```

The generic `Tag` is a phantom type parameter.

For example:

```telos
data Users()
data Orders()
```

Then:

```text
Id<Users>
Id<Orders>
```

are distinct applied nominal types.

The package should provide a controlled typed-generation operation.

For example:

```telos
let userId: Id<Users> = NanoId.typed<Users>()
let orderId: Id<Orders> = NanoId.typed<Orders>()
```

Verify this exact generic call syntax before implementation.

The important semantic requirement is:

```text
Id<Users> must not implicitly convert to Id<Orders>.
```

## 15.3 Keep the wrapper immutable

Typed identifier values are appropriate `data` candidates.

Their state is exactly the contained identifier string.

Do not make typed IDs mutable classes.

## 15.4 Distinguish validation from generation

A typed ID may be loaded from storage or received from another service.

Therefore, provide an explicit validated parsing mechanism.

Conceptually:

```telos
Id<Users>.parse(text)
```

This may validate:

- Character set.
- Expected length.
- Application-specific formatting.

It cannot establish that the string was originally generated by a secure CSPRNG.

Do not confuse syntactic validity with trusted provenance.

## 15.5 Avoid unnecessary runtime representations

The phantom tag should not require a separate heavyweight runtime class for every application domain.

Use the language's existing applied nominal data identity machinery.

---

# 16. Nano ID Public API

Recommended surface:

| API | Result | Purpose |
|---|---|---|
| `NanoId.generate()` | `String` | Default secure ID |
| `NanoId.generate(size)` | `String` | Secure ID with custom length |
| `Alphabet.urlSafe` | `Alphabet` | Default alphabet |
| `Alphabet.new(text)` | `Alphabet` | Validated custom alphabet |
| `NanoIdGenerator.new(...)` | `NanoIdGenerator` | Reusable generator configuration |
| `generator.generate()` | `String` | Generate configured ID |
| `generator.generate(size)` | `String` | Override length |
| `NanoId.typed<Tag>()` | `Id<Tag>` | Optional typed identifier |
| `Id<Tag>.parse(text)` | Validated result | Decode external identifier |

Use the actual supported method selectors when implementing overloads.

If the language's module-level function declaration syntax is available and fully supported, an additional ergonomic API may be considered:

```telos
nanoid()
nanoid(12)
```

Do not add a fake module-level callable implementation merely to copy JavaScript syntax.

The initial class-side API is sufficient.

---

# 17. Error Model

## 17.1 Cryptographic failures

Use the existing:

```text
EntropyError
```

for OS secure randomness failure.

Do not automatically create another exception with identical meaning.

## 17.2 Configuration failures

Invalid alphabets and identifier lengths are input errors.

The standalone package may define focused errors:

```text
InvalidAlphabetError
InvalidIdLengthError
```

However, avoid creating unnecessary custom classes when ordinary `ArgumentError` already communicates the contract correctly.

Prefer one consistent public error model.

## 17.3 Recommended behavior

| Condition | Result |
|---|---|
| Default generation | Secure random String |
| Size 21 | 21 characters |
| Size 0 | Empty String |
| Negative size | ArgumentError |
| Size exceeds configured maximum | ArgumentError |
| Empty alphabet | ArgumentError |
| Duplicate alphabet characters | ArgumentError |
| Non-ASCII alphabet | ArgumentError in the initial implementation |
| OS entropy failure | EntropyError |
| Native allocation failure | Existing runtime allocation error |

Set and document a finite maximum ID length.

A reasonable initial limit is:

```text
1,048,576 characters
```

This is an API/resource-safety decision, not a cryptographic constraint.

Revisit it only with evidence.

---

# 18. Security Properties

## 18.1 Unpredictability

The generated characters must derive from cryptographically suitable random bytes.

No deterministic fallback is permitted.

## 18.2 Uniformity

For a valid alphabet of length `N`, every output position must select each symbol with probability:

```text
1 / N
```

under the assumption of uniformly random source bytes.

## 18.3 Independence

Generation must not intentionally correlate outputs.

Each request must obtain fresh secure randomness.

Do not reuse consumed random bytes across generated IDs.

## 18.4 Collision probability

For an alphabet of size `A` and identifier length `L`:

```text
Possible identifiers = A^L
```

For independently generated uniform IDs, the approximate birthday collision probability after generating `n` identifiers is:

```text
P(collision) ≈ n(n - 1) / (2 × A^L)
```

This is an approximation for sufficiently small collision probabilities.

For the default alphabet:

```text
A = 64
L = 21
```

Therefore:

```text
A^L = 2^126
```

The result is probabilistic uniqueness, not guaranteed uniqueness.

Applications requiring strict uniqueness must enforce that requirement separately, such as through a database uniqueness constraint.

## 18.5 Entropy of custom alphabets

A custom alphabet provides:

```text
Entropy bits = length × log₂(alphabetSize)
```

For example:

```text
16 hexadecimal characters:
16 × 4 = 64 bits
```

A smaller alphabet or shorter identifier reduces the available random space.

The package documentation must explain this.

## 18.6 Security documentation

Do not describe Nano ID as:

- Collision-free.
- Universally suitable for authentication.
- Guaranteed unique.
- A substitute for authorization.
- A universally safe AEAD nonce.
- A cryptographic signature.
- A password hashing mechanism.

Nano ID provides unpredictable random identifiers when correctly implemented and configured.

Application security still depends on how those identifiers are used.

## 18.7 Sensitive identifier handling

Random identifiers may be used as bearer tokens.

Ordinary Telos `String` values are not guaranteed to be securely erased from memory.

Do not claim secure zeroization of generated strings unless the runtime provides a verified mechanism.

For high-value secrets requiring controlled memory lifetime, a dedicated secret-buffer abstraction may be appropriate in a later cryptographic package.

---

# 19. Concurrency and Fiber Semantics

The Nano ID generator should not need mutable RNG state.

Conceptually:

```text
NanoIdGenerator
    Immutable configuration
    Fresh secure randomness per call
```

The generated output buffer must be local to the invocation.

Do not expose the output buffer before generation succeeds.

## 19.1 Stateless default implementation

The initial generator should not maintain a reusable random-byte pool.

This avoids:

- Shared mutable state.
- Pool synchronization.
- Cross-fiber consumption ordering.
- Accidental byte reuse.
- Complicated failure recovery.
- Long-lived unused random material.

## 19.2 Optional pooling

A pooled implementation may improve throughput.

The upstream Nano ID implementation uses string-pool optimizations.

However, pooling introduces additional runtime state and lifecycle requirements.

Do not implement pooling before benchmarking the stateless design.

If pooling is introduced later, verify:

- Every byte is consumed at most once.
- Pool access is synchronized.
- Fibers cannot observe overlapping output regions.
- Pool offsets are updated atomically.
- Failures do not expose stale bytes.
- Idle pooled material has an appropriate lifetime.
- Generator copies do not accidentally duplicate pool state.

Prefer a simpler native bulk-sampling operation over a complicated Telos-side pool when benchmarks show that native dispatch dominates runtime cost.

---

# 20. Testing Requirements

The implementation requires several distinct categories of tests.

## 20.1 API tests

Verify:

```text
NanoId.generate()
NanoId.generate(1)
NanoId.generate(12)
NanoId.generate(21)
NanoId.generate(64)
NanoId.generate(0)
```

Verify output length exactly matches the requested length.

## 20.2 Alphabet tests

Verify:

- Default URL-safe characters.
- Custom hexadecimal alphabet.
- Custom numeric alphabet.
- Two-character alphabet.
- Non-power-of-two alphabet.
- Duplicate characters rejected.
- Empty alphabet rejected.
- Unsupported Unicode rejected.
- Control characters rejected.
- Alphabet construction preserves ordering.

Do not sort alphabet characters during validation.

The caller's alphabet order defines the mapping from random indexes to symbols.

## 20.3 Deterministic algorithm tests

Use an isolated testing seam with controlled byte sequences.

Test the sampling algorithm independently from the real entropy provider.

Examples:

```text
Alphabet size 4:
    mask = 3
    all candidate indexes accepted

Alphabet size 3:
    mask = 3
    candidate 3 rejected

Alphabet size 62:
    mask = 63
    candidates 62 and 63 rejected

Alphabet size 64:
    mask = 63
    no candidates rejected
```

Verify that rejected bytes never produce output.

Verify that the generator requests more randomness when necessary.

The testing seam must not create an insecure production fallback.

## 20.4 Native failure tests

Inject a host entropy failure through a test-only mechanism.

Verify:

- `EntropyError` propagates.
- No partially constructed identifier is returned.
- No deterministic fallback executes.
- Subsequent behavior remains well-defined.
- A failed fill is never reported as successful.

The repository already has failure-injection precedent in:

```text
phalcom-core/src/primitive/random.rs
```

Reuse the existing testing architecture where possible.

## 20.5 Distribution tests

For controlled deterministic input, verify exact acceptance and rejection behavior.

For OS-backed randomness, run statistical smoke tests over many generated symbols.

Check approximate frequencies.

Such tests can detect obvious bias, but they cannot prove cryptographic security.

The mathematical rejection-sampling argument and native entropy contract remain the primary correctness evidence.

## 20.6 Concurrency tests

Generate IDs concurrently from multiple fibers.

Verify:

- No shared output buffers.
- No corruption.
- No duplicated pool offsets.
- No unsafe shared mutable state.
- Correct failure isolation.

Observed collision counts are diagnostic evidence, not proof of uniqueness.

## 20.7 GC tests

Force garbage collection around:

- Alphabet construction.
- Generator construction.
- Random byte acquisition.
- Output construction.
- UTF-8 conversion.
- Repeated generation.
- Concurrent generation.

Verify that live buffers and strings remain correctly rooted.

## 20.8 Type-system tests

For optional typed IDs:

```text
Id<Users> != Id<Orders>
```

At the type level, passing one where the other is expected must be rejected unless an explicit conversion exists.

Verify that phantom parameters preserve their intended applied nominal distinctions.

---

# 21. Performance Requirements

Performance is a first-class acceptance criterion.

The implementation must not optimize only for code size.

## 21.1 Measure these workloads

| Workload | Purpose |
|---|---|
| Default 21-character ID | Main fast path |
| 12-character ID | Short-ID overhead |
| 64-character ID | Medium allocation behavior |
| 128-character ID | Larger output |
| 1 KB ID | Buffer throughput |
| 62-character alphabet | Rejection overhead |
| 16-character alphabet | Power-of-two behavior |
| 65-character alphabet | High rejection rate |
| Reused generator | Configuration amortization |
| New generator per ID | Construction overhead |
| Multiple fibers | Concurrency overhead |

## 21.2 Compare implementation tiers

Benchmark:

```text
A. Existing SecureRandom.bytes

B. Pure Telos Nano ID

C. Pure Telos with reusable configuration

D. Native bulk ASCII sampling, if implemented
```

Also compare equivalent algorithms where feasible.

Do not use external JavaScript benchmarks as directly comparable measurements of the Telos VM.

## 21.3 Instrumentation

Use the existing observability infrastructure where applicable.

Measure:

- OS entropy acquisition time.
- Native dispatch count.
- Byte allocations.
- Bytes read.
- Bytes written.
- String allocations.
- UTF-8 conversion.
- Total generation time.
- Throughput.
- Peak temporary memory.

Keep compiler startup and Universe bootstrap costs separate from steady-state generation throughput.

## 21.4 Performance targets

Recommended initial goals:

1. One secure random-byte acquisition per ordinary default ID.
2. No random-byte rejection for the 64-character alphabet.
3. No per-character immutable string concatenation.
4. One final UTF-8 conversion.
5. No repeated alphabet validation for a reused generator.
6. No repeated alphabet encoding for a reused generator.
7. No unbounded temporary allocations.
8. No production-wide eager initialization of unrelated cryptographic modules.

The native accelerated implementation should demonstrate a meaningful improvement before being accepted.

Do not introduce additional native API complexity for a negligible gain.

## 21.5 Compiled Universe integration

The eventual compiled:

```text
universe.telosc
```

should include the compiled source-owned cryptographic interfaces.

The standalone Nano ID package should be compiled independently against the cached Universe artifact.

The compiler should not need to recompile unrelated Universe modules whenever Nano ID changes.

Coordinate this with the ongoing PERF003 implementation.

Do not modify its cache or image architecture as part of the initial cryptographic package.

---

# 22. Implementation Sequence

Implement in six bounded plans.

## CRYPTO.P1 — Cryptographic Namespace and API Foundation

### Scope

Introduce the `universe.crypto` package.

### Tasks

1. Audit existing secure randomness.
2. Confirm canonical SecureRandom identity.
3. Introduce the crypto package directory.
4. Add explicit public exports.
5. Preserve legacy random imports.
6. Document security semantics.
7. Validate parsing and module interfaces.
8. Verify that crypto discovery does not eagerly initialize unrelated modules.

### Deliverables

```text
packages/universe/src/crypto/package.ph
packages/universe/src/crypto/random/package.ph
```

### Gate

One canonical secure-random service is reachable through the intended new API.

No duplicate entropy source is introduced.

Production authority remains unchanged.

---

## CRYPTO.P2 — Native Security Contract and Verification

### Scope

Validate the existing native randomness boundary.

### Tasks

1. Inspect the current `getrandom` integration.
2. Validate full-buffer success semantics.
3. Validate entropy failure propagation.
4. Validate buffer limits.
5. Validate native effects.
6. Validate GC safety.
7. Validate concurrent calls.
8. Record performance characteristics.
9. Determine whether additional native primitives are justified.

### Gate

The native randomness service satisfies the cryptographic foundation contract.

Do not modify production-native ownership prematurely.

---

## NANOID.P1 — Standalone Package and Correct Algorithm

### Scope

Create the independent Nano ID package.

### Tasks

1. Create `packages/nanoid`.
2. Define its manifest.
3. Implement validated Alphabet.
4. Implement NanoIdGenerator.
5. Implement secure byte acquisition.
6. Implement bit-mask rejection sampling.
7. Implement ASCII byte-buffer output.
8. Implement UTF-8 conversion.
9. Expose the NanoId convenience API.
10. Add documentation and unit tests.

### Gate

Default and custom identifiers are generated correctly using the existing secure entropy primitive.

---

## NANOID.P2 — Type Safety and Comprehensive Verification

### Scope

Strengthen API contracts.

### Tasks

1. Add invalid-input tests.
2. Add deterministic sampling tests.
3. Add entropy-failure tests.
4. Add concurrency tests.
5. Add GC tests.
6. Add statistical smoke tests.
7. Introduce optional typed identifiers.
8. Validate nominal generic distinctions.
9. Document entropy and collision properties.

### Gate

Security-relevant invariants have direct evidence.

---

## NANOID.P3 — Performance Optimization

### Scope

Measure and optimize the hot path.

### Tasks

1. Establish benchmark baseline.
2. Measure VM boundary overhead.
3. Measure output assembly.
4. Measure entropy acquisition.
5. Optimize the pure Telos implementation.
6. Evaluate native bulk ASCII sampling.
7. Implement acceleration only if justified.
8. Compare correctness against the reference algorithm.
9. Verify equivalent distribution and failure behavior.

### Gate

The accepted implementation meets measured throughput and allocation requirements without weakening security.

---

## CRYPTO.P3 — Extended Cryptographic Foundations

### Scope

Expand Universe beyond secure randomness.

Potential future modules:

```text
crypto.hash
crypto.mac
crypto.kdf
crypto.aead
crypto.signatures
```

Implement these through separate plans with algorithm-specific security requirements.

Do not bundle their implementation into Nano ID delivery.

### Gate

Each additional cryptographic domain has a reviewed public contract, a vetted native implementation, and appropriate test vectors.

---

# 23. Branch and Migration Coordination

The new Universe remains a parallel source tree.

Do not modify:

```text
phalcom-core/core/universe/
```

to activate the new crypto namespace during shadow development.

Do not update production:

```text
UniverseSourceProvider
UniverseKey::source_path()
NativeSourceIndex
BuiltinInterfaceBuilder
VM bootstrap
```

until the corresponding source-integration phase.

The initial native work should reuse the already implemented `SecureRandom` primitive.

New native methods require a separate integration plan because their registration must match canonical authored declarations and the source provider.

The standalone `nanoid` package may initially be verified against the active Universe through the legacy `universe.random.SecureRandom` import while its planned `universe.crypto` import remains shadow-only.

This is acceptable as an implementation staging mechanism.

However, the public Nano ID package must not be marked fully integrated with `universe.crypto` until that namespace is compiler-visible and verified.

Do not maintain two divergent entropy implementations to accommodate the two source trees.

---

# 24. Required Deliverables

Produce:

```text
packages/universe/src/crypto/
```

and:

```text
packages/nanoid/
```

plus implementation records covering:

```text
API-DESIGN.md
NATIVE-SECURITY-CONTRACT.md
NANOID-ALGORITHM.md
NANOID-VALIDATION.md
NANOID-PERFORMANCE.md
INTEGRATION-STATUS.md
```

Record:

- Canonical source identities.
- Source exports.
- Native implementation references.
- Security assumptions.
- Algorithmic invariants.
- Test evidence.
- Performance measurements.
- Known limitations.
- Deferred features.
- Production integration prerequisites.

Use coherent, independently reviewable commits.

---

# 25. Definition of Completion

The initial cryptographic foundation and Nano ID program is complete when:

1. `universe.crypto` exists as a coherent source-authored package.
2. It reuses one canonical OS-backed entropy implementation.
3. Native randomness is correctly classified as nondeterministic.
4. Entropy acquisition failures never cause insecure fallback.
5. The secure randomness API is available through the intended module interfaces.
6. `packages/nanoid` is a standalone Telos package.
7. Nano ID supports a secure default 21-character identifier.
8. Custom validated ASCII alphabets are supported.
9. Random symbol selection is unbiased.
10. Default generation uses the power-of-two fast path.
11. Identifier construction avoids repeated string concatenation.
12. Reusable generators avoid repeated configuration work.
13. Concurrent generation is safe.
14. Errors and resource limits are documented.
15. Statistical, algorithmic, native, GC, and integration tests pass.
16. Type-safe identifier wrappers are available if approved.
17. Performance is measured and optimized.
18. No duplicate runtime Universe identity is introduced.
19. Production compiler and native architecture remain protected until integration.
20. The resulting package is suitable for ordinary compilation against the future cached `universe.telosc`.

---

# 26. Final Architectural Ruling

The preferred architecture is:

```text
                    APPLICATION
                         |
                         v
                  packages/nanoid
                         |
               +---------+----------+
               |                    |
               v                    v
        Alphabet / Config      NanoIdGenerator
               |                    |
               +---------+----------+
                         |
                         v
                universe.crypto
                         |
                         v
                  SecureRandom
                         |
                         v
                Native Rust Host
                         |
                         v
                    getrandom
                         |
                         v
              Operating-System CSPRNG
```

**Universe owns cryptographic mechanisms.**

**Nano ID owns identifier-generation policy.**

**Telos owns typed configuration, composition, and the public library design.**

**Rust owns secure entropy acquisition and any justified native acceleration.**

The first implementation should use the existing `SecureRandom` functionality and implement Nano ID's unbiased sampling algorithm in ordinary Telos.

Only after measuring that implementation should the program introduce generic native acceleration.

This preserves a clean language/runtime boundary while making high-performance secure identifier generation achievable without turning Nano ID into a compiler builtin.

---

# 27. External Technical References

1. Nano ID reference implementation:
   https://github.com/ai/nanoid

2. Nano ID secure algorithm:
   https://github.com/ai/nanoid/blob/main/index.js

3. Nano ID reference alphabet:
   https://github.com/ai/nanoid/blob/main/url-alphabet/index.js

4. Rust getrandom:
   https://docs.rs/getrandom/

5. Rust getrandom repository:
   https://github.com/rust-random/getrandom

6. Telos existing native randomness:
   `phalcom-core/src/primitive/random.rs`

7. Telos SecureRandom source:
   `packages/universe/src/random/secure.ph`

8. Telos Bytes:
   `packages/universe/src/collections/bytes.ph`

9. Telos native source metadata:
   `phalcom-native-meta/src/universe.rs`

10. Telos native primitive registration:
    `phalcom-core/src/native/`

**Security review note:** Existing platform and library guarantees must be verified against the exact versions and targets being shipped. Functional testing and statistical smoke tests are not substitutes for cryptographic review.