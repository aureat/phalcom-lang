# UUID

`uuid` is an independent Phalcom package for RFC 9562 UUID values. Its public
value is `data UUID`, represented by two immutable, unsigned-range `Int` words.
No mutable `Bytes` or formatted `String` is stored in a UUID. The words are
network-order 64-bit fields; the current VM may box values above the signed
64-bit range, so this package does not promise an unboxed 16-byte runtime
representation.

```phalcom
from uuid import UUID, UUIDv7Generator

let id = UUID.v7()                         // time-sortable, not strictly monotonic
let random = UUID.v4()                     // OS-backed cryptographic entropy
let stable = UUID.v5(namespace: UUID.namespaceDNS, name: "www.example.com")
let parsed = UUID.parse("550e8400-e29b-41d4-a716-446655440000")
let strict = UUID.parseCanonical("550e8400-e29b-41d4-a716-446655440000")
let sequence = UUIDv7Generator.monotonic()
```

For application-domain identity, `TaggedUUID<Tag>` carries a phantom type
argument: `TaggedUUID<UserTag>` and `TaggedUUID<OrderTag>` are distinct types.
`TaggedUUID<Tag>.parse(text)` validates UUID syntax, while
`TaggedUUID<Tag>.fromUUID(value)` explicitly asserts the domain of an existing
UUID. Typed `v4()`, `v5(namespace: name:)`, and `v7()` generation preserve the
tag. A caller remains responsible for choosing the correct domain tag.

`UUIDv7.fromUUID(value)` and `UUIDv7.parse(text)` return
`Result<UUIDv7, UUIDv7ParseError>`; the former rejects non-RFC or non-v7 UUIDs,
and the latter distinguishes malformed text from a valid UUID of another
version. A checked `UUIDv7` exposes `timestamp -> Int` without an optional
result. `UUIDv7.generate()` and `UUIDv7Generator.generateTyped()` produce this
refined type while the untagged generation APIs remain available.

`parse` accepts lowercase or uppercase canonical text, 32 hex digits, a
lowercase `urn:uuid:` prefix, and braces around canonical text. `parseCanonical`
accepts exactly 36 lowercase ASCII characters with hyphens at the canonical
positions. Neither trims whitespace nor rejects unusual 128-bit UUID values.
Both return `Result<UUID, UUIDParseError>`; `fromBytes` checks for 16 bytes and
returns `Result<UUID, UUIDBytesError>`. `toBytes()` returns an independent
network-order copy. `toString`, `simple`, `urn`, and `braced` format without
changing the underlying value. Equality, hashing and ordering use all 128 bits;
the order is unsigned network-byte order.

`version` returns `Option<UUIDVersion>`: it is absent for Nil, Max, and non-RFC
variants. `timestamp` returns Unix milliseconds only for RFC-variant version 7.
`UUID.v7()` makes no strict ordering guarantee within one millisecond. The
separate `UUIDv7Generator` increments a 74-bit random tail for repeat calls
within a millisecond. Its **single-instance, sequential-call** guarantee is
strict ordering on success. Clock rollback, exhausted tail, and timestamps
outside the 48-bit range raise `UUIDGenerationError`; an entropy failure
propagates `EntropyError`. Do not share one generator concurrently: this
package does not provide atomic coordination across fibers or threads.

UUIDv5 hashes binary namespace bytes followed by UTF-8 name bytes via the
reusable `universe.crypto.sha1` module. `v5Bytes` accepts an arbitrary binary
name; no Unicode normalization is applied. SHA-1 is required by the UUIDv5
standard, but UUIDv5 is not a general collision-resistant security primitive.

## Integration status

The implementation and conformance sources live in `packages/uuid/`. The
`universe.crypto.sha1` source is in `packages/universe/src/crypto/` and uses
Telos byte and bitwise operations. The active runtime's embedded Universe
(`phalcom-core/core/universe/src/`) has not yet been migrated to that source
tree: at present a production project importing `universe.crypto.sha1` cannot
link. The conformance project under `tests/` is therefore authored but awaits
that Universe integration before it can be executed. Source parse/export checks
can be run with `cargo test -p phalcom-modules --test uuid_package`.
The runtime conformance fixtures for tagged and checked values remain staged
until that Universe integration. A shared generic `Id<Tag, Value>` abstraction,
runtime benchmarks, and a concurrency-coordinated monotonic generator have not
been established by this source-staged package.
