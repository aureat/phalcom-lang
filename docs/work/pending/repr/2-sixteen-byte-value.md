# Implementation Specification 2 — Replace `Value` with a 16-Byte Explicit Tagged Representation

**Status:** Ratified
**Repository baseline:** apply after Implementation Specification 1
**Primary crate:** `phalcom-core`
**Representation target:** exactly 16 bytes
**NaN-boxing:** explicitly deferred; not part of this implementation

# 1. Purpose

Replace the current 24-byte Rust enum representation of `Value` with a deliberately specified two-word representation:

```text
16 bytes total

┌──────────────────────────┐
│ payload: u64             │  8 bytes
├──────────────────────────┤
│ metadata: u64            │  8 bytes
└──────────────────────────┘
```

The new representation must preserve:

- full signed `i64` immediate range;
- every `f64` bit pattern;
- the existing full opaque `ObjRef` handle;
- `Symbol`;
- private `Nil`;
- `Unit`;
- booleans;
- `None`;
- nested `Some` without the current seven-layer language-visible implementation limit;
- `Copy`;
- no reference counting;
- no ownership inside `Value`;
- GC precision;
- existing Phalcom equality/hash/object-model semantics.

This is a physical representation change, not a surface-language change.

---

# 2. Why 16 bytes is the chosen design

The current representation at:

```text
phalcom-core/src/value/mod.rs:41
```

is a Rust enum with:

```text
Nil
Unit
Bool
Int(i64)
Float(f64)
Symbol
Obj
None
Some1(OptionPayload)
...
Some7(OptionPayload)
```

`OptionPayload` is itself another tagged enum.

The nested discriminant + payload arrangement is what forces the current value to approximately 24 bytes.

A 16-byte explicit representation avoids that nested tag while keeping the full payload domain.

Unlike NaN-boxing, it does not require:

- reducing immediate integer width;
- reserving NaN payloads;
- changing `ObjRef` bit allocation;
- canonicalizing NaNs;
- introducing unsafe bit-punning;
- coupling the VM to undocumented pointer-width assumptions.

NaN-boxing remains a future experiment against this 16-byte baseline.

---

# 3. Context-budget contract for the implementing agent

This migration touches many pattern matches. The implementation process must be compiler-driven rather than repository-reading-driven.

## 3.1 Initial source reads

Read only:

| File | Window |
|---|---|
| `phalcom-core/src/value/mod.rs` | lines 1–140, 185–520 |
| `phalcom-core/src/value/option.rs` | entire file; it is small |
| `phalcom-core/src/value/render.rs` | entire file |
| `phalcom-core/src/value/boolean.rs` | entire file |
| `phalcom-core/src/value/nil.rs` | entire file |
| `phalcom-core/src/heap/mod.rs` | lines 45–85 |
| `phalcom-core/src/compiler/lib/expr.rs` | lines 700–835 |
| `phalcom-core/benches/vm_bench.rs` | lines 60–120 |
| ADR/spec files listed in §18 | only the relevant representation sections |

Do not initially inspect every primitive that matches `Value`.

## 3.2 Site discovery

After introducing the new representation, run:

```bash
cargo check \
  -p phalcom-core \
  --lib \
  --message-format short
```

Fix one source file at a time.

For each compiler-reported file:

1. open only error ±25 lines;
2. replace the old enum pattern with the appropriate accessor/tag check;
3. rerun the compiler.

Do not dump all matches into context.

If a complete file list is useful, generate silently:

```bash
rg -l \
  'Value::(Nil|Unit|Bool|Int|Float|Symbol|Obj|None|Some[1-7])' \
  phalcom-core/src \
  phalcom-core/tests \
  phalcom-core/benches \
  > /tmp/phalcom-value-sites.txt
```

Do not paste that file into the model context.

## 3.3 Compaction points

Compact after:

1. raw representation + representation tests;
2. Option rewrite;
3. central `value/mod.rs` semantics;
4. rendering;
5. compiler-reported peripheral migrations;
6. documentation.

---

# 4. New file — `phalcom-core/src/value/repr.rs`

Create this file and make it the sole owner of physical `Value` layout.

## 4.1 Tag definition

Use:

```rust
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ValueTag {
    Nil = 0,
    Unit = 1,
    Bool = 2,
    Int = 3,
    Float = 4,
    Symbol = 5,
    Obj = 6,
    None = 7,
}
```

These numeric values are an **internal VM representation contract**, not a serialized public format.

Do not expose them to Phalcom source programs.

## 4.2 Value layout

Define:

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Value {
    payload: u64,
    meta: u64,
}
```

Required metadata layout:

```text
bits  0..=7   base tag
bits  8..=39  Some nesting depth, u32
bits 40..=63  reserved; must currently be zero
```

Constants:

```rust
const TAG_MASK: u64 = 0xff;
const DEPTH_SHIFT: u32 = 8;
const DEPTH_MASK: u64 = 0xffff_ffffu64 << DEPTH_SHIFT;
const RESERVED_MASK: u64 = !(TAG_MASK | DEPTH_MASK);
```

No current code may store information in the reserved bits.

That space is intentionally left available for future VM representation flags without another immediate layout expansion.

---

# 5. Semantic interpretation of tag + depth

This is the key simplification.

## Ordinary values

```text
depth = 0, tag = Int     => Int(payload)
depth = 0, tag = Float   => Float(payload)
depth = 0, tag = Obj     => Obj(payload)
...
```

## `None`

```text
tag   = None
depth = 0
```

## `Some(x)`

`Some` does not need its own base tag.

Instead:

```text
base value tag/payload remain unchanged
depth = 1
```

Examples:

```text
Some(42)

tag     = Int
payload = 42
depth   = 1
```

```text
Some(Some(42))

tag     = Int
payload = 42
depth   = 2
```

```text
Some(None)

tag     = None
payload = 0
depth   = 1
```

```text
Some(Some(None))

tag     = None
payload = 0
depth   = 2
```

This means nesting depth is metadata instead of an explosion of Rust enum variants.

---

# 6. Representation invariants

`repr.rs` must document and enforce these invariants.

### I1 — Reserved bits

```text
meta & RESERVED_MASK == 0
```

### I2 — Canonical empty payloads

For:

```text
Nil
Unit
None
```

payload is zero.

### I3 — Bool payload

Bool payload is exactly:

```text
0 or 1
```

### I4 — Private Nil cannot be wrapped

No valid `Value` may have:

```text
tag == Nil && depth > 0
```

### I5 — Some depth

Any valid `depth > 0` denotes `Some`.

### I6 — None

Only:

```text
tag == None && depth == 0
```

is the `None` variant.

### I7 — ObjRef bits are opaque

The `u64` payload for an `Obj` is an opaque round-trip representation supplied by `slotmap`'s public key API.

No `Value` code may interpret index/generation bit positions.

---

# 7. `ObjRef` opaque round-trip support

Target:

```text
phalcom-core/src/heap/mod.rs:approximately 60–80
```

Import only the public slotmap key APIs needed for round-trip conversion.

Add internal helpers conceptually equivalent to:

```rust
impl ObjRef {
    #[inline]
    pub(crate) fn to_opaque_u64(self) -> u64 {
        self.data().as_ffi()
    }

    #[inline]
    pub(crate) fn from_opaque_u64(raw: u64) -> Self {
        slotmap::KeyData::from_ffi(raw).into()
    }
}
```

The exact `KeyData -> ObjRef` public API should be adjusted to the version of `slotmap = "1"` actually compiled.

If the signature differs, the agent is permitted one targeted read of slotmap's public `Key`/`KeyData` API.

Do **not** inspect or depend on slotmap's internal index/generation layout.

Add a test:

```text
allocate object
ObjRef -> u64 -> ObjRef
assert same key
assert heap lookup resolves same object
```

---

# 8. Constructors and accessors

Do not preserve enum-looking `Value::Int(...)` APIs through awkward non-snake-case associated functions.

This is a representation migration and Phalcom is pre-stability. Adopt ordinary Rust APIs.

Required constructors:

```rust
impl Value {
    pub(crate) const fn nil() -> Self;
    pub const fn unit() -> Self;
    pub const fn bool(value: bool) -> Self;
    pub const fn int(value: i64) -> Self;
    pub const fn float(value: f64) -> Self;
    pub const fn symbol(value: Symbol) -> Self;
    pub fn obj(value: ObjRef) -> Self;
    pub const fn none() -> Self;
}
```

Keep canonical constants where they improve call sites:

```text
NIL
TRUE
FALSE
```

Add:

```text
UNIT
NONE
```

only if repeated call sites become materially clearer. Do not add constants merely for symmetry.

## Required predicates/accessors

Provide:

```rust
pub fn is_nil(self) -> bool;
pub fn is_unit(self) -> bool;
pub fn is_bool(self) -> bool;
pub fn is_int(self) -> bool;
pub fn is_float(self) -> bool;
pub fn is_symbol(self) -> bool;
pub fn is_obj(self) -> bool;

pub fn is_option(self) -> bool;
pub fn is_none(self) -> bool;
pub fn is_some(self) -> bool;

pub fn as_bool(self) -> Option<bool>;
pub fn as_int(self) -> Option<i64>;
pub fn as_float(self) -> Option<f64>;
pub fn as_obj(&self) -> Option<ObjRef>;
```

Retain the existing semantically useful:

```rust
pub fn as_symbol(&self) -> Result<Symbol, String>
```

or split its internal cheap path into:

```rust
pub(crate) fn symbol_value(self) -> Option<Symbol>
```

while preserving external behavior.

All ordinary `as_*` methods must reject a wrapped value:

```text
Some(Int(1)).as_int() == None
```

The base payload is internal Option machinery, not the visible current value.

---

# 9. Internal tag helpers

Add private/crate-private methods:

```rust
fn tag(self) -> ValueTag;
fn some_depth_raw(self) -> u32;
fn with_some_depth(self, depth: u32) -> Self;
fn without_some_wrappers(self) -> Self;
```

`without_some_wrappers` must preserve tag/payload and zero only the depth field.

Do not expose raw `payload`/`meta` to ordinary VM modules.

The design goal is:

```text
VM code talks to semantic Value accessors.
repr.rs talks to bits.
```

---

# 10. Rewrite Option representation

Target:

```text
phalcom-core/src/value/option.rs
```

Delete `OptionPayload`.

Delete physical `Some1` through `Some7` concepts.

Keep:

```rust
pub(crate) enum OptionCase {
    None,
    Some(Value),
    NotOption,
}
```

## `is_option`

Implement:

```text
tag == None || depth > 0
```

## `is_none`

Implement:

```text
tag == None && depth == 0
```

## `is_some`

Implement:

```text
depth > 0
```

## `option_depth`

Change return type from `u8` to:

```rust
u32
```

Behavior remains:

```text
ordinary non-option -> 0
None                -> 0
Some(x)             -> 1
Some(Some(x))       -> 2
...
```

## New practical nesting ceiling

Change:

```rust
MAX_OPTION_NESTING: u8 = 7
```

to:

```rust
pub const MAX_OPTION_NESTING: u32 = u32::MAX;
```

This is a physical representability bound, not a language-level design bound.

No program should realistically be able to construct `2^32 - 1` nested wrappers.

Do not expose documentation suggesting that ordinary programs are expected to handle that many wrappers.

## `wrap_some`

Preserve the current fallible signature:

```rust
pub fn wrap_some(self) -> Result<Self, RuntimeError>
```

because many VM/compiler paths already compose it through `?`.

Implement:

```text
Nil -> internal error; Nil must never become surface data

otherwise:
    new_depth = current depth + 1 using checked_add

overflow:
    OptionNestingLimit { limit: u32::MAX }

success:
    return same payload/base tag with new depth
```

For an ordinary non-option:

```text
depth 0 -> depth 1
```

For `None`:

```text
tag None depth 0 -> tag None depth 1
```

which correctly represents:

```text
Some(None)
```

## `option_case`

Implement without allocation:

```text
None:
    OptionCase::None

depth > 0:
    OptionCase::Some(
        same Value with depth reduced by exactly one
    )

otherwise:
    OptionCase::NotOption
```

This naturally handles arbitrary nesting.

---

# 11. Update `RuntimeError::OptionNestingLimit`

Target:

```text
phalcom-core/src/error.rs:approximately 180–190
```

Change:

```rust
OptionNestingLimit { limit: u8 }
```

to:

```rust
OptionNestingLimit { limit: u32 }
```

Update its documentation.

Do not describe the new limit as an intentional language restriction.

Recommended wording:

```text
The compact Value metadata cannot represent another Option wrapper.
This is a physical representation overflow, not an ordinary semantic
nesting limit.
```

---

# 12. GC semantics

Current `Value::gc_obj_ref` deliberately sees through a `Some` wrapper while `as_obj` does not.

Preserve that exact distinction.

## `as_obj`

Return an object only when:

```text
tag == Obj && depth == 0
```

Thus:

```text
Obj(handle)       -> Some(handle)
Some(Obj(handle)) -> None
```

## `gc_obj_ref`

Return an object whenever:

```text
tag == Obj
```

regardless of nesting depth.

Thus:

```text
Obj(handle)                   -> Some(handle)
Some(Obj(handle))             -> Some(handle)
Some(Some(Obj(handle)))       -> Some(handle)
```

This keeps the collector precise without teaching GC about Option representation.

---

# 13. Rewrite `value/mod.rs`

Remove the current enum declaration from approximately:

```text
phalcom-core/src/value/mod.rs:41–82
```

At module top:

```rust
mod repr;

pub use repr::Value;
```

Do not publicly re-export `ValueTag`.

Rewrite the major semantic methods using predicates/accessors instead of representation matching.

## `class`

Required order:

```text
if None:
    none_class

else if Some:
    some_class

else:
    switch base tag
```

A `Some(Int)` must resolve to `Some`, not `Int`.

## `to_context`

Any Option value remains:

```rust
CallContext::Immediate { value: self }
```

An unwrapped Obj resolves through the heap as before.

## `type_name`

Preserve current observable coarse strings:

```text
nil
unit
bool
int
float
symbol
object
option
```

A wrapped scalar reports:

```text
option
```

not the base scalar type.

---

# 14. Preserve representation-level `PartialEq`

The old derived `PartialEq` had specific behavior distinct from Phalcom's language `value_eq`.

Implement `PartialEq` manually for the new struct.

Rules:

1. differing Some depth => false;
2. differing base tag => false;
3. `f64` compares with ordinary IEEE `==`;
4. all non-float payloads compare by semantic payload identity.

Therefore:

```text
Float(NaN) != Float(NaN)

Float(+0.0) == Float(-0.0)

Int(1) != Float(1.0)       // representation-level PartialEq

Some(Int(1)) == Some(Int(1))

Some(Int(1)) != Int(1)

Some(Int(1)) != Some(Some(Int(1)))
```

Do not derive `PartialEq` over raw `payload/meta`, because that would make:

```text
+0.0 != -0.0
```

at the representation level.

---

# 15. Repair the `Hash` contract while migrating

Current `Hash for Value` hashes float raw bits while Rust's `f64` equality treats:

```text
+0.0 == -0.0
```

That means the current physical `PartialEq` and `Hash` can disagree.

Correct this during the representation change.

For float hashing:

```rust
let bits = if value == 0.0 {
    0.0f64.to_bits()
} else {
    value.to_bits()
};
```

Hash:

1. base tag;
2. Some depth;
3. normalized payload.

NaNs remain unequal, so their differing hashes are allowed.

Add a test:

```rust
assert_eq!(
    hash(Value::float(0.0)),
    hash(Value::float(-0.0)),
);
```

---

# 16. Preserve Phalcom language equality

Target current implementation:

```text
phalcom-core/src/value/mod.rs:approximately 300–430
```

`value_eq` is not the same operation as host `PartialEq`.

Preserve all current language behavior:

- `Int` exact comparison;
- `Float` numeric comparison;
- cross `Int`/`Float` equality for integral finite floats;
- heap `LargeInt` numeric comparisons;
- String content equality;
- Symbol interned equality;
- module historical behavior;
- ordinary heap-object identity;
- Option depth semantics.

## Option fast path

Before comparing ordinary base tags:

```rust
if self.is_option() || other.is_option() {
    // handle all Option semantics
}
```

Required Option behavior:

```text
None == None

None != Some(None)

Some(x) == Some(y)
    iff wrapper depths are equal
    and base x value_eq base y

Some(x) != x
```

Do not recursively peel one wrapper at a time for deep Options.

Instead:

1. compare depths directly;
2. if depths differ, return false;
3. zero the depth on both values;
4. compare their bases once.

That makes Option equality O(1) with respect to wrapper depth.

---

# 17. Rewrite rendering without recursive Option depth

Target:

```text
phalcom-core/src/value/render.rs
```

Delete `OptionPayload` dependencies.

Delete:

```rust
option_parts(...)
```

and recursive:

```rust
fmt_option(depth - 1, ...)
```

A `u32` depth makes recursive formatting dangerous.

## Streaming formatter

For `Debug` and `Display`:

```text
write "Some(" depth times
render base once
write ")" depth times
```

This is O(depth) stack-safe streaming.

For owned `to_string`/`to_debug`, similarly build iteratively.

Do not repeatedly do:

```rust
rendered = format!("Some({rendered})")
```

for each layer; that is O(depth²) copying.

A practical implementation:

```rust
let depth = value.option_depth();
let base = value.without_some_wrappers();

let inner = ...render base...;

let depth_usize = usize::try_from(depth)
    .unwrap_or(usize::MAX);

let extra = depth_usize.saturating_mul(6);
let mut out = String::with_capacity(
    inner.len().saturating_add(extra)
);

for _ in 0..depth {
    out.push_str("Some(");
}

out.push_str(&inner);

for _ in 0..depth {
    out.push(')');
}
```

Ordinary realistic depths are small. `saturating_*` prevents arithmetic overflow in capacity calculation.

---

# 18. Update canonical constants

## `value/boolean.rs`

Replace:

```rust
pub const TRUE: Value = Value::Bool(true);
pub const FALSE: Value = Value::Bool(false);
```

with:

```rust
pub const TRUE: Value = Value::bool(true);
pub const FALSE: Value = Value::bool(false);
```

## `value/nil.rs`

Replace:

```rust
pub const NIL: Value = Value::Nil;
```

with:

```rust
pub const NIL: Value = Value::nil();
```

`nil()` may remain `pub(crate)` if `NIL` is the intended public Rust surface.

---

# 19. Compiler-driven peripheral migration

Once `repr.rs`, `option.rs`, central `mod.rs`, and `render.rs` compile, use compiler errors to migrate call sites.

## Construction mapping

Mechanical replacement family:

```text
Value::Unit
    -> Value::unit()

Value::Bool(x)
    -> Value::bool(x)

Value::Int(x)
    -> Value::int(x)

Value::Float(x)
    -> Value::float(x)

Value::Symbol(x)
    -> Value::symbol(x)

Value::Obj(x)
    -> Value::obj(x)

Value::None
    -> Value::none()
```

`Value::Nil` should usually become:

```text
NIL
```

or a semantic `is_nil()` test depending on context.

## Pattern matching mapping

Do not mechanically translate:

```rust
match value {
    Value::Int(n) => ...
}
```

into raw-bit inspection.

Use:

```rust
if let Some(n) = value.as_int() {
    ...
}
```

or a tag/accessor combination when a multi-arm switch is clearer.

Representative high-value files likely requiring changes include:

```text
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/primitive/int.rs
phalcom-core/src/primitive/float.rs
phalcom-core/src/primitive/number.rs
phalcom-core/src/primitive/boolean.rs
phalcom-core/src/primitive/list.rs
phalcom-core/src/primitive/map.rs
phalcom-core/src/primitive/set.rs
phalcom-core/src/primitive/tuple.rs
phalcom-core/src/primitive/object.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs
phalcom-core/benches/vm_bench.rs
```

This is not permission to read all of them up front.

Open only compiler-reported windows.

---

# 20. Benchmark-code migration

Current `phalcom-core/benches/vm_bench.rs` checks:

```rust
Some(Value::Int(got))
```

and:

```rust
Some(Value::Float(got))
```

Rewrite without enum matching:

```rust
let value = module
    .get(sym)
    .unwrap_or_else(|| {
        panic!("benchmark global `{name}` missing")
    });

if let Some(got) = value.as_int() {
    assert_eq!(got as f64, expected);
} else if let Some(got) = value.as_float() {
    assert_eq!(got, expected);
} else {
    panic!(
        "benchmark global `{name}` is not numeric: {value:?}"
    );
}
```

This keeps benchmark correctness independent of physical representation.

---

# 21. Representation tests

Place primary tests in:

```text
phalcom-core/src/value/repr.rs
```

under `#[cfg(test)]`.

## Exact layout

```rust
#[test]
fn value_is_exactly_sixteen_bytes() {
    assert_eq!(std::mem::size_of::<Value>(), 16);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn value_has_eight_byte_alignment_on_64_bit_targets() {
    assert_eq!(std::mem::align_of::<Value>(), 8);
}
```

The 16-byte size is now an intentional VM representation contract and should be exact.

## Scalar round trips

Test:

```text
i64::MIN
-1
0
1
i64::MAX

0.0
-0.0
NaN
+Infinity
-Infinity
representative finite values

true
false

Symbol(0)
large Symbol id
```

For `f64`, verify bit round trip:

```rust
assert_eq!(
    decoded.to_bits(),
    original.to_bits()
);
```

including NaNs.

## ObjRef

Allocate multiple objects and round-trip handles through the payload representation.

Verify stale/new-generation identity is not confused.

## Option

Test at minimum depths:

```text
0
1
2
7
8
255
65_535
```

The `8` case is important because it explicitly proves the old semantic ceiling has disappeared.

Do not actually iterate millions/billions of wraps in the test. Provide a crate-private test helper for constructing a valid depth when necessary, or loop only small depths.

## Nil invariant

Verify:

```text
NIL.wrap_some()
```

fails.

## GC

Verify:

```text
Obj(id).as_obj()               == Some(id)
Some(Obj(id)).as_obj()         == None
Some(Obj(id)).gc_obj_ref()     == Some(id)
nested Some Obj gc_obj_ref     == Some(id)
```

## Hash

Test:

```text
+0.0 and -0.0 equal => equal hashes
same Some depth/base => equal hashes
different Some depth => representation unequal
```

---

# 22. Integration tests that must stay green

Pay particular attention to existing suites covering:

- `Option`;
- nested Option;
- `None`;
- `Some(None)`;
- equality;
- map/set hashing;
- numeric equality;
- NaN;
- `-0.0`;
- GC tracing;
- fibers;
- closures;
- module globals;
- selector Families;
- error propagation.

The representation migration must not rewrite expected language outputs.

If a golden-output test changes, treat that as a suspected bug, not as an expected consequence of the migration.

---

# 23. Documentation updates

## `docs/adr/accepted/0010-tagged-value-enum.md`

Amend the ADR rather than erasing historical decisions.

Add a dated amendment:

```text
2026-08-16 — Physical representation changed from the
correctness-first Rust enum to an explicit 16-byte two-word tagged
Value. The semantic Value API and object-model rules are unchanged.
NaN-boxing remains deferred.
```

Document:

```text
payload u64
metadata u64
tag
u32 Some depth
reserved bits
```

Explicitly retire physical:

```text
Some1 ... Some7
```

while preserving the semantic distinction among nested Options.

## Option documentation

Remove wording that implies seven layers are a language rule.

State that the implementation carries a practical `u32` nesting depth and that overflow is a physical-resource limit.

## `docs/spec/current/performance.md`

Correct any stale `Value` size statement to:

```text
16 bytes
```

and note that it is now an explicit internal representation invariant.

## `docs/spec/current/memory-management.md`

Update memory-density examples based on 16-byte `Value`.

For example:

```text
1,000,000 contiguous Value slots:

old 24-byte representation:
    ~24 MB raw payload

new 16-byte representation:
    ~16 MB raw payload

raw slot reduction:
    33.3%
```

Do not claim an equivalent 33% whole-process RSS improvement; container overhead, capacity, heap objects, allocator behavior, and GC all matter.

## `docs/forge/DEFERRED.md`

Keep NaN-boxing listed as deferred.

Rewrite any stale language suggesting that NaN-boxing is the immediate next representation task.

It is now:

```text
a possible future 8-byte experiment benchmarked against the
16-byte explicit representation
```

---

# 24. Performance expectations — not acceptance assumptions

Likely wins:

- denser VM stack;
- denser `Vec<Value>`;
- denser list/tuple backing storage;
- denser instance fields;
- lower copy bandwidth;
- less backing-array growth/memmove traffic;
- denser GC scans;
- smaller error variants containing `Value`.

Possible costs:

- tag/depth bit extraction instead of direct Rust enum discrimination;
- `ObjRef` opaque encode/decode;
- loss of compiler conveniences around enum pattern matching;
- accessor call sites if not inlined.

Therefore correctness + exact 16-byte layout are acceptance criteria.

A speedup is **not** assumed.

Spec 3 measures whether the expected locality and memory wins actually occur.

---

# 25. Verification order

### Representation unit tests

```bash
cargo test -p phalcom-core --lib value::
```

### Core compile/test

```bash
cargo check -p phalcom-core --all-targets

cargo test -p phalcom-core
```

### Full workspace

```bash
cargo test --workspace
```

### Clippy

```bash
cargo clippy \
  --workspace \
  --all-targets \
  --all-features \
  --locked \
  -- \
  -D warnings
```

### Format

```bash
cargo fmt --all -- --check
```

---

# 26. Explicitly forbidden implementation shortcuts

Do not:

- use NaN-boxing;
- reduce `Int` to a 47/48-bit immediate;
- canonicalize floating NaNs;
- reinterpret pointers;
- make `Value` unsafe;
- use `transmute`;
- use `repr(packed)`;
- depend on slotmap's undocumented key bits;
- heap-box ordinary scalars;
- put an `Arc`/`Rc` inside `Value`;
- restore a seven-layer Option bound;
- expose raw `payload/meta` to ordinary VM code;
- redesign object identity at the same time;
- alter language equality to match the raw representation.

---

# 27. Definition of done

This migration is complete when:

```text
size_of::<Value>() == 16
```

and:

- `Value` remains `Copy`;
- every `i64` round-trips inline;
- every `f64` bit pattern round-trips inline;
- current `ObjRef` round-trips without bit-layout assumptions;
- nested Options beyond seven layers work;
- `Nil` remains impossible to wrap;
- `Some(Obj)` remains precisely GC-traceable;
- Option rendering is iterative, not recursive by depth;
- `+0.0` / `-0.0` obey `PartialEq`/`Hash` consistency;
- language `value_eq` behavior is unchanged;
- the whole test suite passes;
- Clippy passes with `-D warnings`;
- stale 24-byte/Some1…Some7 documentation has been corrected;
- no NaN-boxing code has entered production.