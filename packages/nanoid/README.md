# nanoid

Standalone, secure, URL-friendly unique identifier generator for Telos.

## Features

- **Cryptographically Secure:** Backed by host operating-system entropy via `getrandom`.
- **Unbiased Uniformity:** Uses bit-mask rejection sampling to guarantee that every symbol has identical probability.
- **Power-of-Two Fast Path:** 64-symbol URL-safe alphabet (and other power-of-two alphabets) generates IDs without character rejection.
- **Batch Acquisition:** Entropy is acquired in bulk buffers to minimize host system call overhead.
- **Memory Efficient:** Assembles output directly in byte buffers before single UTF-8 conversion; no repeated string concatenation.
- **Domain-Safe Typing:** Optional phantom-tagged `Id<Tag>` prevents cross-domain identifier substitution at compile time.
- **Configurable Alphabets:** Custom printable ASCII alphabets (2..94 characters) with strict validation.

## Usage

### Default Identifiers

```telos
from nanoid import NanoId

// Generate default 21-character URL-safe ID (~126 bits of entropy)
let id = NanoId.generate()

// Generate identifier of custom length
let shortId = NanoId.generate(12)
```

### Custom Alphabets and Reusable Generators

```telos
from nanoid import Alphabet, NanoIdGenerator

// Define a validated custom alphabet (printable ASCII 0x21..0x7E, unique symbols)
let hexAlphabet = Alphabet.new("0123456789abcdef")

// Construct a reusable generator to amortize configuration
let hexGen = NanoIdGenerator.new(hexAlphabet, 16)

let token = hexGen.generate()
```

### Strongly-Typed Identifiers

```telos
from nanoid import NanoId, NanoIdGenerator, TypedNanoIdGenerator, Alphabet, NanoIdParseError, Id

data UserTag()
data OrderTag()

// Distinct nominal types
let userId: Id<UserTag> = NanoId.typed<UserTag>()
let orderId: Id<OrderTag> = NanoId.typed<OrderTag>()

// userId cannot be passed where Id<OrderTag> is expected
let generator = NanoIdGenerator.new(Alphabet.new("0123456789abcdef"), 16)
let hexId: Id<UserTag> = TypedNanoIdGenerator<UserTag>.fromGenerator(generator).generate()
let checked: Result<Id<UserTag>, NanoIdParseError> =
  Id<UserTag>.parse(hexId.value, generator.alphabet, generator.size)
```

`Id<Tag>.parse` returns `InvalidLength` or
`InvalidCharacter` as cases of `NanoIdParseError`. `Id<Tag>.wrap` explicitly
asserts a domain tag without validating text; use `parse` at untrusted input
boundaries. Tags distinguish exact applied data types; they do not constrain
which domain a caller chooses when wrapping or parsing.

## Security Invariants

1. **Unpredictability:** Every byte originates from the OS cryptographic randomness facility. No deterministic fallback is used.
2. **Rejection Sampling:** For an alphabet of size $N$, candidate indexes $\ge N$ are rejected rather than modulo-reduced, eliminating modulo bias.
3. **Local Generation Buffers:** The default generator is cached, while entropy and output buffers are local to each invocation.
4. **Collision Resistance:** A 21-character identifier over a 64-character alphabet offers $64^{21} = 2^{126}$ states.

## Integration status

The typed source and conformance fixtures parse, and the package's `Id<Tag>`
export links to its original declaration in `nanoid.id` under
`cargo test -p phalcom-modules --test nanoid_package`. The current embedded
Universe does not yet expose the authored `Bytes` API used by this package, so
the Telos fixtures are not executable through the active runtime.
