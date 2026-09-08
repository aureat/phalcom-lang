# Bytes v2 Implementation Specification

> **Status:** proposed hardening revision over the shipped U-BYTES implementation.
> This specification does **not** redesign `Bytes`, change its public collection model,
> or add grow/shrink behavior. It tightens the shipped implementation at the numeric,
> allocation, and security-contract boundaries exposed by the completed numeric tower
> and by the needs of Streams/FS.
>
> **Supersedes only the affected implementation obligations** in the shipped
> `bytes-implementation-spec.md`. All unaffected architectural decisions remain in
> force: `Bytes` is a fixed-length mutable native octet buffer backed by `Box<[u8]>`,
> represented by `Object::Bytes`, reached through ordinary `Value::Obj`, traced as a
> no-op, rejected as a mutable `Map`/`Set` key, and surfaced through the existing
> `.ph` protocol.
>
> **No new public surface is introduced by this revision.**

## 1. Purpose

The shipped `Bytes` architecture is sound. The v2 work exists to close four implementation
gaps that became clearer after the original unit landed:

1. numeric indices and lengths now cross a completed `Int`/`Float` split rather than a
   monolithic `Number`;
2. user-controlled byte lengths must not flow unchecked into host allocation;
3. integer-to-host conversions must be checked rather than relying on Rust casts;
4. the guarantees of `zeroize` and `equalsConstantTime` must match what the implementation
   can actually prove.

This is therefore a **hardening unit**, not a semantic redesign.

## 2. Architectural decisions retained unchanged

The following shipped decisions are reaffirmed:

```rust
pub struct BytesObject {
    data: Box<[u8]>,
}
```

`Box<[u8]>` remains the backing store because length is immutable after construction.
`Bytes` remains mutable in contents and immutable in extent.

The implementation MUST continue to satisfy:

- no `Vec<u8>` field stored in the live `BytesObject`;
- no resize/grow/shrink primitive;
- no `Value::Bytes` value arm;
- no `Value` or `ObjRef` children inside the byte buffer;
- slices are copies, never views;
- `copyInto` has memmove semantics for aliasing;
- `utf8` is strict and total through `String | None`;
- `utf8Lossy` is display-oriented and never a data round-trip mechanism;
- block-taking collection operations remain inherited/derived rather than native.

The existing heap, bootstrap, tracing, and mutable-key decisions require no redesign.

## 3. Numeric contract after the `Int` / `Float` split

### 3.1 Public semantic type

Indexes, lengths, byte offsets, and octet values are conceptually **integers**.

The preferred user-level contract is therefore:

```text
Bytes.new(length: Int)
Bytes#at(index: Int)
Bytes#set(index: Int, value: Int)
Bytes#slice(start: Int, end: Int)
Bytes#copyInto(dst, offset: Int)
```

This revision does not require a source-breaking removal of integral `Float` acceptance if
the existing language-wide collection convention still permits it. However, all Bytes
documentation MUST stop describing these values merely as arbitrary `Number`s.

The implementation boundary MUST treat numeric inputs through a single checked conversion
discipline.

### 3.2 Checked conversion helpers

Do not use unchecked Rust casts for user-controlled sizes or indices.

Introduce or centralize helpers with semantics equivalent to:

```rust
fn expect_non_negative_index(value: &Value) -> PhResult<usize>;
fn expect_byte_length(value: &Value) -> PhResult<usize>;
fn expect_octet(value: &Value) -> PhResult<u8>;
```

`expect_non_negative_index` MAY remain shared with other collection primitives. Its contract
MUST reject:

- negative integers;
- negative floats;
- NaN;
- positive or negative infinity;
- fractional floats;
- integral floats outside the exactly/host-representable accepted range;
- integers larger than `usize::MAX`.

A direct cast such as:

```rust
*n as usize
```

is insufficient unless range validity has already been established.

### 3.3 `Float` compatibility rule

If integral `Float` indices remain accepted for compatibility, acceptance MUST be defined as:

```text
finite
fraction == 0
value >= 0
value <= usize::MAX
conversion round-trips exactly
```

The runtime MUST NOT rely on Rust's saturating/implementation-defined numeric-cast behavior to
decide the result.

A future language-wide cleanup may narrow collection indexing to `Int` only. That is not part of
this unit.

## 4. Byte-length allocation policy

### 4.1 User-controlled allocation is fallible

The shipped construction path conceptually performs:

```rust
vec![0u8; len].into_boxed_slice()
```

For a language runtime, an arbitrary user-provided `len` must not be allowed to reach an
infallible allocation path unchecked.

`Bytes.new(length)` MUST reject a length before allocation when the requested extent is outside
the implementation's supported allocation range.

### 4.2 Deliberate implementation maximum

Define one VM-wide or heap-wide maximum for a single contiguous managed allocation:

```rust
const MAX_MANAGED_ALLOCATION: usize = /* deliberate implementation limit */;
```

The exact constant is an implementation-policy decision and SHOULD be shared with other future
large contiguous objects rather than embedded uniquely in `Bytes`.

The required semantic law is:

```text
0 <= length <= MAX_MANAGED_ALLOCATION
```

A request larger than the limit produces a normal language/runtime error. It MUST NOT panic,
abort, wrap, truncate, or silently clamp.

The maximum SHOULD be conservative enough that all intermediate arithmetic and allocator calls
remain representable on every supported host architecture.

### 4.3 Fallible zeroed allocation

Replace the conceptual infallible allocation:

```rust
vec![0u8; len]
```

with a fallible path.

A suitable shape is:

```rust
let mut data = Vec::new();
data.try_reserve_exact(len).map_err(|_| allocation_error(len))?;
data.resize(len, 0);
let boxed = data.into_boxed_slice();
```

or an equivalent allocator API that reports failure.

If the project's error taxonomy already has an allocation-capacity or resource-exhaustion error,
use it. Do not invent a fatal process exit for ordinary allocation refusal.

### 4.4 Construction from existing host buffers

`BytesObject::from_vec(Vec<u8>)` remains a zero-copy ownership transfer at the Rust layer.

Callers that build the `Vec<u8>` from user-controlled lengths MUST themselves use checked/fallible
allocation. `from_vec` does not absolve callers from allocation policy.

## 5. Arithmetic safety

All range and copy calculations involving user-controlled indices MUST use checked arithmetic.

The shipped `copyInto_` posture is correct:

```rust
offset.checked_add(src_len)
```

and MUST be retained.

The same rule applies anywhere future Bytes code computes:

```text
start + length
offset + length
total concatenated length
chunk count * chunk size
```

No arithmetic used for indexing or allocation may rely on release-mode integer wrapping.

`.ph` composition such as `Bytes#concat` MUST surface a language-level failure if its aggregate
size cannot be represented or allocated.

## 6. `zeroize`: tighten implementation and wording

### 6.1 Problem

The fixed `Box<[u8]>` backing guarantees that the live `Bytes` object never reallocates and
therefore does not leave old copies behind through buffer growth. That is valuable, but ordinary:

```rust
slice.fill(0)
```

does not by itself establish a cryptographic secure-erasure guarantee against compiler
optimization.

Therefore the shipped wording that treats ordinary fill as a complete secure-erasure mechanism
is too strong.

### 6.2 Required implementation

`Bytes#zeroize` SHOULD use an audited secure-zeroing primitive.

Preferred implementation:

```text
RustCrypto `zeroize`
```

or an equivalent mechanism whose contract uses volatile writes/compiler barriers appropriate to
secure erasure.

The `BytesObject` representation remains unchanged.

The primitive floor need not grow merely to add a new selector. The existing `fill_` may remain
for ordinary filling while `zeroize` gains a dedicated native primitive **only if** a secure
erase cannot be expressed safely through the existing floor.

If adding a dedicated primitive is required, the implementation specification and census MUST
explicitly account for it. Do not silently repurpose ordinary `fill_` into a security primitive
whose implementation differs from normal fill semantics.

### 6.3 Fallback if no audited dependency is accepted

If the project deliberately declines an audited secure-erasure dependency, the normative
contract MUST be weakened to:

> `zeroize` explicitly overwrites every octet in the live buffer with zero and performs no
> reallocation. It is best-effort erasure and does not promise that every compiler/target removes
> all recoverable copies from registers, temporary stack locations, or other implementation
> artifacts.

The specification MUST choose one of §6.2 or §6.3 before this revision lands.

## 7. `equalsConstantTime`: define a defensible contract

### 7.1 Existing implementation shape

The shipped source performs:

```rust
let mut acc: u8 = 0;
for i in 0..a.len() {
    acc |= a[i] ^ b[i];
}
std::hint::black_box(acc) == 0
```

This has no source-level content-dependent early exit, which is useful.

However, `std::hint::black_box` is not a cryptographic constant-time guarantee.

### 7.2 Preferred implementation

Use a small audited constant-time comparison primitive, for example the `subtle` crate or an
equivalent reviewed implementation.

Length mismatch MAY return `false` immediately. Buffer lengths are not secret under the existing
Bytes contract.

The semantic contract should be:

> For equal-length operands, comparison does not intentionally branch or terminate based on
> differing byte contents. Length is public. The implementation uses an audited constant-time
> primitive appropriate to the supported compiler/target model.

Do not promise universal wall-clock equality across CPUs, compilers, caches, speculative
execution, or operating systems.

### 7.3 Fallback contract

If no audited dependency is accepted, retain the XOR accumulation but rename the guarantee in the
specification to a source-level property:

> Equal-length comparison visits every byte and contains no explicit content-dependent early exit.

The public selector spelling MAY remain `equalsConstantTime` for compatibility, but its
documentation MUST state the implementation limitation.

## 8. Octet validation

The existing octet rule remains:

```text
0 <= value <= 255
integer
```

With the numeric split:

- `Int` values in `0..=255` are accepted;
- integral finite `Float` values MAY remain accepted only under the global compatibility rule;
- all other values raise a type/argument contract failure.

No path may write a non-octet into the backing buffer.

## 9. Error channel

The revision does not change the existing native/.ph layering rule:

- malformed arguments are contract/type failures;
- out-of-range writes are hard contract failures;
- out-of-range reads remain total and return `None`;
- allocation refusal is a language/runtime failure, not process abort;
- no native primitive constructs `.ph` `Result` wrappers unless that is already the floor
  convention for the operation.

The exact allocation-error class/kind MUST follow the project's existing error taxonomy at
implementation time.

## 10. File-by-file changes

### 10.1 `phalcom-core/src/primitive/list.rs` or shared numeric helper module

Harden the shared index conversion so host conversion is checked.

If changing `expect_index` would alter unrelated shipped semantics, introduce a new checked helper
and migrate Bytes first; then file the shared collection migration separately.

### 10.2 `phalcom-core/src/heap/bytes.rs`

Change zeroed allocation to a fallible construction API.

Recommended shape:

```rust
pub fn try_new_zeroed(len: usize) -> Result<Self, AllocErrorLike>
```

`BytesObject` remains:

```rust
pub struct BytesObject {
    data: Box<[u8]>,
}
```

No layout change.

If secure zeroing is implemented directly on the object, add a documented method such as:

```rust
pub fn zeroize(&mut self)
```

using the accepted audited mechanism.

### 10.3 `phalcom-core/src/primitive/bytes.rs`

Update:

- `bytes_class_new` to call the checked byte-length conversion;
- allocation path to propagate allocation refusal;
- numeric rustdocs from generic `Number` wording to integer semantics;
- `equalsConstantTime_` implementation/documentation per §7;
- zeroize primitive/binding only if §6 requires a dedicated native.

Retain checked arithmetic in `copyInto_`.

### 10.4 `phalcom-core/core/core.ph`

No general Bytes redesign.

If a dedicated secure-zero primitive is added:

```phalcom
zeroize {
  self.zeroize_
  return self
}
```

Otherwise retain the existing derivation but update documentation to the best-effort contract.

### 10.5 Documentation and census

If the primitive count changes, update:

- primitive census;
- invariant totals;
- class binding documentation;
- completed Bytes as-built record.

If no primitive is added, the floor count remains unchanged.

## 11. Test specification

### 11.1 Numeric conversion

Positive:

```text
Bytes.new(0)
Bytes.new(1)
Bytes.new(small Int)
at/set/slice/copy offsets using Int
```

Compatibility lane if integral Float remains accepted:

```text
Bytes.new(3.0)
b.at(1.0)
```

Negative:

```text
Bytes.new(-1)
Bytes.new(-1.0)
Bytes.new(1.5)
Bytes.new(NaN)
Bytes.new(+Infinity)
Bytes.new(-Infinity)
index > usize::MAX where representable
```

### 11.2 Allocation limits

Test the explicit boundary without attempting dangerous allocations in CI.

Required:

```text
length == MAX_MANAGED_ALLOCATION boundary validation
length > MAX_MANAGED_ALLOCATION rejected before allocation
```

The test SHOULD allow a small test-only configured maximum so the rejection path is exercised
without reserving enormous memory.

Add a Rust unit test that simulates allocator refusal where the chosen allocation abstraction
permits injection or controlled failure.

### 11.3 Arithmetic

Retain and strengthen:

```text
copyInto offset + length overflow
concat aggregate overflow
slice malformed bounds
```

No case may panic.

### 11.4 Secure erase

If audited zeroize lands, test source-level postcondition:

```text
all bytes are zero after zeroize
```

and unit-test that the implementation invokes the accepted secure-zeroing path.

Do not introduce timing-based CI tests for secure erasure.

### 11.5 Constant-time comparison

Required functional tests:

```text
equal buffers -> true
first-byte difference -> false
last-byte difference -> false
length mismatch -> false
non-Bytes -> contract failure
```

If an audited library is used, code review plus dependency pinning is the load-bearing timing
assurance. Wall-clock microbenchmarks remain advisory only.

## 12. Compatibility and migration

This revision intentionally preserves:

```text
Bytes object identity
fixed length
mutability of contents
slice-copy semantics
all existing public selector spellings
UTF-8 behavior
collection protocol behavior
Map/Set mutable-key rejection
```

Code written against the shipped Bytes surface should not require source changes unless the
project separately chooses to remove integral-Float indexing across the language.

## 13. What this revision does not do

- no resizable Bytes;
- no immutable byte-string type;
- no zero-copy slices/views;
- no encoding API beyond the existing UTF-8 operations;
- no mmap/file-backed Bytes;
- no change to Path's defensive-copy contract;
- no general numeric-tower redesign;
- no promise of impossible platform-independent cryptographic timing.

## 14. Acceptance gates

Bytes v2 is complete when:

1. no user-controlled byte length reaches an unchecked host allocation;
2. numeric-to-`usize` conversion is checked;
3. allocation refusal is a language/runtime error rather than process failure;
4. `copyInto` and aggregate-size arithmetic remain checked;
5. the `zeroize` contract matches the actual implementation;
6. the `equalsConstantTime` contract matches the actual implementation;
7. the existing Bytes conformance/golden suite stays green;
8. new boundary tests are green under debug and release builds.
