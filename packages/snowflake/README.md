# Snowflake

An independent Telos package for 63-bit Twitter-style Snowflake IDs. Its
default layout reserves 41 bits for milliseconds since the Twitter epoch
(`1288834974657`), 10 bits for the node ID and 12 bits for the sequence.
Snowflakes are compact, sortable IDs; they are **not** random or secret tokens.

```phalcom
from snowflake import SnowflakeNode, SnowflakeId, SnowflakeLayout

let node = SnowflakeNode.new(1) // node IDs must be unique across generators
let id = node.generate()
let parsed = SnowflakeId.parse(id.toString) // Result<SnowflakeId, SnowflakeParseError>

let custom = SnowflakeLayout.configured(
  epochMillis: 0, nodeBits: 8, sequenceBits: 14
)
let otherNode = SnowflakeNode.configured(nodeId: 2, layout: custom)
let customId = otherNode.generate()
let timestamp = customId.timeMillis(using: custom)
```

For domain- and layout-specific values, use
`TypedSnowflakeNode<Domain, Format>.configured(nodeId, layout)`. Its
`generate()` and `generateAt(unixMillis)` methods return
`TypedSnowflake<Domain, Format>`, which retains the generator's actual layout.
The typed value exposes `timeMillis`, `node`, and `step` using that stored
layout. `TypedSnowflake<Domain, Format>.parse(text, layout)` validates the
63-bit integer; `fromId(id, layout)` explicitly asserts tags for an existing
`SnowflakeId`. Tags are caller-chosen: they prevent accidental mixing of
different domain/format *types*, but cannot prove independently supplied
runtime layouts have identical epoch and bit widths. Equality and hashing of
typed values include both the raw ID and the stored layout.

The value stores one exact, nonnegative `Int` limited to `0..2^63-1`. The
layout belongs to a generator; use `timeMillis(using:)`, `node(using:)`, and
`step(using:)` to decode IDs from a custom layout. The no-argument observations
assume the default Twitter layout. Each generator freezes its layout rather than
reading mutable package-level settings. Nonnegative node and sequence widths
totaling at most 22 bits leave the other bits for time. Equality,
hashing, and ordering compare the 63-bit integer, regardless of layout.
The custom epoch may be before 1970; generation still requires a nonnegative
elapsed millisecond count from that epoch.

Sequential calls on one `SnowflakeNode` advance a step within the same
millisecond and reset it when time advances. `generateAt(unixMillis)` accepts
an explicit time for reproducible or externally clocked use. Clock rollback,
step exhaustion within a millisecond, and epoch/range overflow raise
`SnowflakeGenerationError` without changing generator state. `generate()` uses
`Timestamp.now` (Unix nanoseconds divided by one million). A generator is
**not synchronized** for concurrent use; independent processes must coordinate
node IDs and preserve their clock assumptions to avoid duplicates.

`SnowflakeId` supports decimal, binary, z-base-32, base-36, base-58, and Go
Snowflake's standard Base64 encoding of **decimal UTF-8 text**, plus quoted
decimal JSON strings. Parsers return `Result` and reject negative, invalid, or
out-of-range IDs. `toBytes()`/`fromBytes()` use eight big-endian binary octets
(Go `IntBytes()`); `toTextBytes()`/`parseTextBytes()` use decimal ASCII (Go
`Bytes()`). No global bit width or epoch is consulted while parsing; supply a
layout explicitly for custom decoding.

## Integration status

This package uses the authored `packages/universe/src/` clock and byte APIs.
The active compiler still embeds the older `phalcom-core/core/universe/src/`
tree, whose `universe.collections.bytes` interface does not export `Bytes`.
Consequently, the Telos conformance project in `tests/` is source-authored but
cannot yet compile or run through the active runtime. Its sources parse, its
interfaces build, and its selected imports link against the authored `Bytes`
public interface, under `cargo test -p phalcom-modules --test snowflake_package`.
That link check treats Universe modules as external interface dependencies;
it does not type-check or execute their staged implementations.
