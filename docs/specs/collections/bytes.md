# `Bytes`

`Bytes` is a mutable, indexed sequence of octets in `[0, 256)`. Its size is an
Int byte count, independent of Unicode character count. Text encoding and
decoding rules are owned by [`../text.md`](../text.md); a Bytes value is not a
String and does not promise valid UTF-8.

## Value operations

- Construction by size creates that many zero bytes and requires a
  nonnegative count. `fromString` encodes the String as UTF-8. `fromList` copies
  byte values from a List and rejects values outside the byte domain.
- `at(index)` and `set(index, value)` access or replace one byte with bounds and
  domain checks. Indexed assignment is equivalent to `set`.
- `fill(value)` replaces every byte after validating the value. `slice(start,
  end)` returns an independent byte copy for the selected half-open interval.
- `zeroize` overwrites every byte with zero. `utf8` returns `Option<String>`
  when the bytes decode as valid UTF-8; `utf8Lossy` replaces invalid sequences.
- `copyInto(destination, offset)` copies into the supplied destination only
  when the destination range is valid. `concat(other)` returns a new Bytes
  value. `equalsConstantTime(other)` compares bytes without data-dependent early
  exit and is the explicit comparison operation for secret-bearing buffers.
- `toList` returns a fresh list of byte values; `toTuple` returns an immutable
  tuple snapshot.
- Iteration yields bytes in increasing index order. Equality and hashing follow
  the shared collection contract; because Bytes is mutable, it cannot be a Map
  key or a Set member.

## Byte streams and paths

The `universe.collections.bytes` module also exports `Path`, `OpenMode`,
`BytesReader`, `BytesWriter`, `BufferedReader`, `BufferedWriter`, and the
associated stream errors. These resource and filesystem APIs are specified in
[`../io.md`](../io.md); they are not implied by the `Bytes` sequence rules on
this page.
