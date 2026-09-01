# Phalcom Lightweight Native `Result<T, E>` Representation — Patch-Grade Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace heap-allocated `General` storage for canonical `Result::Ok` / `Result::Error` unary cases with a shared lightweight unary-wrapper representation in Phalcom's existing 16-byte `Value`, while preserving canonical semantic identity, mixed-wrapper order, pattern matching, `.class`, equality/hash behavior, GC correctness, and transparent overflow through a spill object.

**Dependency:** Execute only after `phalcom-post-universe-review-correctness-remediation-plan.md` is complete through its full verification gate. In particular:

- canonical `Result` and `Ordering` must already reuse their one authoritative runtime root `ClassId`;
- compiler match lowering must already be semantic-ID-driven rather than spelling-driven;
- `Option<T>`'s public signatures must already be sound;
- prelude visibility and Universe module-resolution defects must already be corrected.

**Repository baseline for this plan:** `main @ 1c78f5d23f11865dc5e3d55e15b6f9b48a927bcc`. If `main` moves before execution, re-ground every named symbol before applying exact replacement blocks.

**Physical encoding decision used by this plan:** retain the existing 24 reserved metadata bits untouched. Reuse only the current 32-bit `Some`-depth region (`meta` bits `8..=39`) as sixteen 2-bit unary-wrapper cells:

```text
bits  0..=7   ValueTag
bits  8..=39  16 × 2-bit native unary-wrapper cells
bits 40..=63  RESERVED — unchanged by this feature
```

Wrapper cell codes:

```text
00 = empty / end
01 = Option::Some
10 = Result::Ok
11 = Result::Error
```

Cell 0 (bits `8..=9`) is the **outermost** wrapper. Pushing an outer wrapper shifts the existing 32-bit wrapper field left by two bits and inserts the new code into cell 0. Popping reads cell 0 and shifts the wrapper field right by two bits. The 24-bit reserved region is never shifted or modified.

This gives sixteen inline unary wrappers while preserving the entire existing `40..=63` metadata budget for future uses such as dispatch metadata.

---

## 0. How to use this plan

This is not a design sketch. Treat it as a guided patch sequence.

For every task:

1. Read the **Why this layer owns the fix** paragraph.
2. Add or update the narrow regression test first where instructed.
3. Run the named narrow command and confirm the expected failure.
4. Make only the code change for that task.
5. Re-run the narrow command.
6. Run the task's crate-level gate.
7. Commit before moving to the next task unless the task explicitly says otherwise.

Do not merge physical representation with semantic identity. The core invariant is:

> `DeclarationId` / `VariantId` decide *what the value means*; `RuntimeVariantStorage` decides *how that exact variant is physically stored*.

A user enum named `Result`, `Ok`, or `Error` must never receive this native storage policy merely because its spelling matches the built-in names.

---

# 1. Architecture orientation

The files touched by this feature occupy distinct layers:

```text
phalcom-semantic
    canonical enum + VariantId semantics
    NO Value bit knowledge

phalcom-core/src/modules/semantic_lowering.rs
    projects canonical semantic identities into backend storage policy

phalcom-core/src/adt.rs
    VM/runtime descriptors for enum and variant identities + storage policy

phalcom-core/src/value/repr.rs
    16-byte physical Value encoding; pure, allocation-free bit operations

phalcom-core/src/value/option.rs
    Option convenience API; migrate off Some-depth assumptions

phalcom-core/src/heap/adt.rs
    heap payloads for ordinary ADT cases and wrapper spill nodes

phalcom-core/src/heap/object.rs
    Object enum

phalcom-core/src/heap/trace.rs
    precise GC outgoing-edge declaration

phalcom-core/src/vm/adt.rs
    storage-aware construction, inspection, payload extraction, case class
```

Keep these identities distinct:

```text
VariantId
    canonical semantic exact-case identity

RuntimeVariantId
    VM-local descriptor identity

RuntimeVariantStorage
    physical storage strategy of that RuntimeVariantId

NativeUnaryWrapperKind
    2-bit physical wrapper code carried by Value
```

Do not put `NativeUnaryWrapperKind` into `phalcom-semantic`.

---

# 2. End-state invariants

The implementation is complete only when all of the following are true.

- [ ] `std::mem::size_of::<Value>() == 16` remains true.
- [ ] `meta` bits `40..=63` remain untouched by native wrapper push/pop.
- [ ] `Option::Some`, `Result::Ok`, and `Result::Error` share one generic unary-wrapper mechanism.
- [ ] Short wrapper chains up to 16 wrappers allocate no wrapper object.
- [ ] Wrapper order is exact: `Some(Ok(x)) != Ok(Some(x))` structurally.
- [ ] `Option::None` remains an immediate nullary singleton.
- [ ] User-defined enums always retain ordinary `General` storage unless a future explicit policy says otherwise.
- [ ] `runtime_variant_of` returns the same logical variant for inline and spilled values.
- [ ] `case_payload_at(value, 0)` removes exactly one outer wrapper and preserves the remaining nesting.
- [ ] `.class` / `case_behavior_class` depend on `RuntimeVariantId`, not bit spelling.
- [ ] Spill objects are internal implementation objects and are never exposed as the surface `.class` of the wrapped ADT value.
- [ ] GC keeps an object payload alive when it is reachable only through inline wrappers or spill objects.
- [ ] Overflow is transparent: the 17th wrapper spills rather than raising an Option nesting limit.
- [ ] Language equality/hash are independent of whether a chain is currently inline or spilled.
- [ ] `===` may continue to use exact representation identity according to its existing contract; do not accidentally redefine it as ADT structural equality.

---

# Phase A — Freeze the current representation with red tests

## Task 1: Add representation-budget tests before changing `Value`

**Why this layer owns the fix:** `phalcom-core/src/value/repr.rs` is the only layer that should know exact metadata bit ranges. Tests here protect the 16-byte ABI and the reserved region before other runtime code begins depending on the new wrapper API.

**Files:**

- Modify: `phalcom-core/src/value/repr.rs`

### Steps

- [ ] **Step 1: Locate the current metadata constants.**

Open `phalcom-core/src/value/repr.rs` and find:

```rust
const TAG_MASK: u64 = 0xff;
const DEPTH_SHIFT: u32 = 8;
const DEPTH_MASK: u64 = 0xffff_ffffu64 << DEPTH_SHIFT;
const RESERVED_MASK: u64 = !(TAG_MASK | DEPTH_MASK);
```

Do not change them yet.

- [ ] **Step 2: Add a test that records the current 24-bit reserved region.**

Inside the existing `#[cfg(test)] mod tests`, add:

```rust
#[test]
fn current_value_metadata_reserves_upper_twenty_four_bits() {
    assert_eq!(RESERVED_MASK, 0x00ff_ffff_0000_0000u64);
}
```

This test will later be renamed to wrapper terminology, but it ensures the feature does not silently consume bits `40..=63`.

- [ ] **Step 3: Keep the existing size test unchanged.**

Verify the file still contains:

```rust
assert_eq!(std::mem::size_of::<Value>(), 16);
```

Do not weaken or cfg-gate this test.

- [ ] **Step 4: Run only Value representation tests.**

```bash
cargo +stable test -p phalcom-core value::repr::tests
```

Expected before any implementation change: pass.

**Commit:**

```text
test(value): pin metadata budget before native unary wrappers
```

---

# Phase B — Introduce storage vocabulary without changing behavior

## Task 2: Add the runtime variant-storage policy types

**Why this layer owns the fix:** `phalcom-core/src/adt.rs` already owns runtime descriptors. Storage is a property of a runtime variant descriptor, not a semantic type and not a raw `Value` tag.

**Files:**

- Modify: `phalcom-core/src/adt.rs`

### Steps

- [ ] **Step 1: Add native unary semantic storage kinds immediately after `RuntimeAdtRepresentation`.**

Insert:

```rust
/// Native unary ADT cases whose payload is one existing `Value`.
///
/// This is runtime storage policy, not source-level variant identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NativeAdtUnaryKind {
    OptionSome,
    ResultOk,
    ResultError,
}

/// Native nullary cases with a dedicated immediate value encoding.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NativeAdtSingletonKind {
    OptionNone,
}

/// Physical storage strategy for one exact runtime variant.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuntimeVariantStorage {
    /// Ordinary `Object::AdtCase` / `Value::adt_singleton` representation.
    General,
    /// A unary wrapper encoded in `Value` metadata, spilling transparently.
    NativeUnary(NativeAdtUnaryKind),
    /// A dedicated native singleton encoding.
    NativeSingleton(NativeAdtSingletonKind),
}
```

- [ ] **Step 2: Add storage to `RuntimeVariantDescriptor`.**

Find:

```rust
pub struct RuntimeVariantDescriptor {
    pub semantic_id: VariantId,
    pub runtime_id: RuntimeVariantId,
    pub enum_id: RuntimeEnumId,
    pub discriminant: CaseDiscriminant,
    pub shape: RuntimeVariantShape,
    pub payload_arity: u16,
    pub behavior_class: ClassId,
    pub singleton: Option<Value>,
}
```

Insert immediately after `payload_arity`:

```rust
pub storage: RuntimeVariantStorage,
```

- [ ] **Step 3: Add a `storage` argument to `RuntimeAdtRegistry::register_variant`.**

Change the signature from:

```rust
pub fn register_variant(
    &mut self,
    semantic_id: VariantId,
    enum_id: RuntimeEnumId,
    discriminant: CaseDiscriminant,
    shape: RuntimeVariantShape,
    payload_arity: u16,
    behavior_class: ClassId,
    singleton: Option<Value>,
) -> RuntimeVariantId
```

to:

```rust
pub fn register_variant(
    &mut self,
    semantic_id: VariantId,
    enum_id: RuntimeEnumId,
    discriminant: CaseDiscriminant,
    shape: RuntimeVariantShape,
    payload_arity: u16,
    storage: RuntimeVariantStorage,
    behavior_class: ClassId,
    singleton: Option<Value>,
) -> RuntimeVariantId
```

- [ ] **Step 4: Store the argument in the descriptor.**

In the `RuntimeVariantDescriptor` constructor add:

```rust
storage,
```

between `payload_arity` and `behavior_class`.

- [ ] **Step 5: Update the current call site in `phalcom-core/src/vm/adt.rs` temporarily with behavior-preserving storage.**

At the `register_variant(...)` call, pass:

```rust
let storage = if spec.representation == crate::adt::RuntimeAdtRepresentation::NativeOption {
    if shape == RuntimeVariantShape::Singleton {
        crate::adt::RuntimeVariantStorage::NativeSingleton(
            crate::adt::NativeAdtSingletonKind::OptionNone,
        )
    } else {
        crate::adt::RuntimeVariantStorage::NativeUnary(
            crate::adt::NativeAdtUnaryKind::OptionSome,
        )
    }
} else {
    crate::adt::RuntimeVariantStorage::General
};
```

Then pass `storage` immediately after `payload_arity`.

This is a temporary bridge. A later task moves the policy into semantic lowering so VM registration stops inferring it from enum representation.

- [ ] **Step 6: Compile only `phalcom-core`.**

```bash
cargo +stable check -p phalcom-core
```

Fix only missing new-argument compile errors. Do not change constructor behavior yet.

**Commit:**

```text
refactor(adt): add per-variant runtime storage policy
```

---

# Phase C — Replace Some-depth bits with generic inline wrapper cells

## Task 3: Define the 2-bit physical wrapper vocabulary

**Why this layer owns the fix:** physical wrapper codes are representation details. They must live beside `ValueTag`, not beside semantic `VariantId`.

**Files:**

- Modify: `phalcom-core/src/value/repr.rs`

### Steps

- [ ] **Step 1: Replace depth constants with wrapper-field constants.**

Replace:

```rust
const DEPTH_SHIFT: u32 = 8;
const DEPTH_MASK: u64 = 0xffff_ffffu64 << DEPTH_SHIFT;
const RESERVED_MASK: u64 = !(TAG_MASK | DEPTH_MASK);
```

with:

```rust
const WRAPPER_SHIFT: u32 = 8;
const WRAPPER_BITS: u32 = 32;
const WRAPPER_CELL_BITS: u32 = 2;
const WRAPPER_MASK: u64 = 0xffff_ffffu64 << WRAPPER_SHIFT;
const RESERVED_MASK: u64 = !(TAG_MASK | WRAPPER_MASK);

pub(crate) const INLINE_NATIVE_WRAPPER_CAPACITY: usize =
    (WRAPPER_BITS / WRAPPER_CELL_BITS) as usize;
```

- [ ] **Step 2: Update the file header comment.**

Replace the line describing “Some depth” with:

```text
- meta: u64
  - bits 0..=7: ValueTag
  - bits 8..=39: sixteen 2-bit native unary-wrapper cells
  - bits 40..=63: reserved for independent Value metadata
```

- [ ] **Step 3: Insert the physical wrapper enum beside `ValueTag`.**

```rust
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum NativeUnaryWrapperKind {
    Some = 0b01,
    ResultOk = 0b10,
    ResultError = 0b11,
}

impl NativeUnaryWrapperKind {
    #[inline]
    fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            0b01 => Some(Self::Some),
            0b10 => Some(Self::ResultOk),
            0b11 => Some(Self::ResultError),
            _ => None,
        }
    }
}
```

Do not add an enum variant for `0b00`; zero means “no wrapper cell”.

- [ ] **Step 4: Add raw wrapper-field accessors.**

Inside `impl Value`, near the old Some-depth helpers, replace the depth helpers with:

```rust
#[inline]
pub(crate) fn inline_wrapper_bits(self) -> u32 {
    ((self.meta & WRAPPER_MASK) >> WRAPPER_SHIFT) as u32
}

#[inline]
pub(crate) fn with_inline_wrapper_bits(self, bits: u32) -> Self {
    let meta = (self.meta & !WRAPPER_MASK) | ((bits as u64) << WRAPPER_SHIFT);
    debug_assert_eq!(
        meta & RESERVED_MASK,
        self.meta & RESERVED_MASK,
        "wrapper write must preserve reserved metadata bits"
    );
    Self {
        payload: self.payload,
        meta,
    }
}
```

- [ ] **Step 5: Add logical inline-stack helpers.**

Still in `impl Value`, add:

```rust
#[inline]
pub(crate) fn inline_wrapper_len(self) -> usize {
    let mut bits = self.inline_wrapper_bits();
    let mut len = 0usize;
    while len < INLINE_NATIVE_WRAPPER_CAPACITY && (bits & 0b11) != 0 {
        len += 1;
        bits >>= WRAPPER_CELL_BITS;
    }
    len
}

#[inline]
pub(crate) fn inline_outer_wrapper(self) -> Option<NativeUnaryWrapperKind> {
    NativeUnaryWrapperKind::from_bits((self.inline_wrapper_bits() & 0b11) as u8)
}

#[inline]
pub(crate) fn try_push_inline_wrapper(
    self,
    wrapper: NativeUnaryWrapperKind,
) -> Option<Self> {
    let bits = self.inline_wrapper_bits();
    if (bits >> (WRAPPER_BITS - WRAPPER_CELL_BITS)) != 0 {
        return None;
    }
    let shifted = (bits << WRAPPER_CELL_BITS) | wrapper as u32;
    Some(self.with_inline_wrapper_bits(shifted))
}

#[inline]
pub(crate) fn pop_inline_wrapper(self) -> Option<(NativeUnaryWrapperKind, Self)> {
    let outer = self.inline_outer_wrapper()?;
    let remaining = self.inline_wrapper_bits() >> WRAPPER_CELL_BITS;
    Some((outer, self.with_inline_wrapper_bits(remaining)))
}

#[inline]
pub(crate) fn without_inline_wrappers(self) -> Self {
    self.with_inline_wrapper_bits(0)
}
```

Important: `try_push_inline_wrapper` returns `None` on capacity exhaustion. Allocation-aware code will handle spill later.

- [ ] **Step 6: Update every plain scalar/object predicate to use wrapper emptiness instead of `some_depth_raw() == 0`.**

For example replace:

```rust
self.tag() == ValueTag::Int && self.some_depth_raw() == 0
```

with:

```rust
self.tag() == ValueTag::Int && self.inline_outer_wrapper().is_none()
```

Apply the same rule to:

```text
is_adt_singleton
is_nil
is_unit
is_bool
is_int
is_float
is_symbol
is_obj
```

- [ ] **Step 7: Keep `gc_obj_ref()` wrapper-transparent.**

Do **not** make `gc_obj_ref()` require “no wrappers”. The existing GC seam correctly traces an underlying `ObjRef` even when wrapper metadata is present.

The intended implementation remains equivalent to:

```rust
if self.tag() == ValueTag::Obj {
    Some(ObjRef::from_opaque_u64(self.payload))
} else {
    None
}
```

- [ ] **Step 8: Update `PartialEq` and `Hash` to incorporate wrapper bits instead of Some depth.**

At this phase, Rust-level `Value::PartialEq` remains **physical** for spill objects. For inline values, compare/hash `inline_wrapper_bits()` exactly where the old code compared/hashed `some_depth_raw()`.

- [ ] **Step 9: Replace representation tests.**

Delete depth-specific tests such as `option_depth_u32_max_saturates`. Add:

```rust
#[test]
fn native_wrapper_capacity_is_sixteen() {
    assert_eq!(INLINE_NATIVE_WRAPPER_CAPACITY, 16);
}

#[test]
fn wrapper_codes_round_trip_in_outermost_first_order() {
    let value = Value::int(1)
        .try_push_inline_wrapper(NativeUnaryWrapperKind::ResultOk)
        .unwrap()
        .try_push_inline_wrapper(NativeUnaryWrapperKind::Some)
        .unwrap();

    let (outer, inner) = value.pop_inline_wrapper().unwrap();
    assert_eq!(outer, NativeUnaryWrapperKind::Some);

    let (next, payload) = inner.pop_inline_wrapper().unwrap();
    assert_eq!(next, NativeUnaryWrapperKind::ResultOk);
    assert_eq!(payload, Value::int(1));
}
```

- [ ] **Step 10: Add a reserved-bit preservation test.**

Because ordinary constructors currently keep reserved bits zero, create a test-only value by directly setting `meta` inside the module test and verify wrapper push/pop does not change `meta & RESERVED_MASK`.

- [ ] **Step 11: Run the narrow representation suite.**

```bash
cargo +stable test -p phalcom-core value::repr::tests
```

At this point Option tests may fail because `option.rs` still refers to deleted depth helpers. That is expected until Task 4; if `cargo test` cannot compile because of those references, proceed immediately to Task 4 before committing Tasks 3+4 as one atomic commit.

---

## Task 4: Migrate `Option` from depth arithmetic to inline wrapper operations

**Why this layer owns the fix:** `value/option.rs` is an Option convenience layer. It may recognize the physical `Some` wrapper kind, but it must not know bit positions or masks.

**Files:**

- Modify: `phalcom-core/src/value/option.rs`

### Steps

- [ ] **Step 1: Replace the module comment.**

Use:

```rust
//! Native `Option` convenience operations over the shared unary-wrapper encoding.
//! `Some` is a native unary wrapper; `None` remains its dedicated immediate tag.
```

- [ ] **Step 2: Replace imports.**

Change:

```rust
use crate::value::repr::ValueTag;
```

to:

```rust
use crate::value::repr::{NativeUnaryWrapperKind, ValueTag};
```

- [ ] **Step 3: Remove `MAX_OPTION_NESTING`.**

The language must no longer expose a fixed Option nesting failure at the old `u32` depth. Deep wrapper chains will spill.

- [ ] **Step 4: Rewrite `is_option`, `is_none`, and `is_some`.**

Use:

```rust
#[inline]
pub fn is_option(self) -> bool {
    self.is_none() || self.is_some()
}

#[inline]
pub fn is_none(self) -> bool {
    self.tag() == ValueTag::None && self.inline_outer_wrapper().is_none()
}

#[inline]
pub fn is_some(self) -> bool {
    self.inline_outer_wrapper() == Some(NativeUnaryWrapperKind::Some)
}
```

These methods answer the **outer surface wrapper only**. That is correct for mixed wrappers:

```text
Some(Ok(x)).is_some() == true
Ok(Some(x)).is_some() == false
```

- [ ] **Step 5: Replace `option_depth()` with `inline_option_depth_for_test()` or delete it if production has no callers.**

Run first:

```bash
rg -n 'option_depth\(' phalcom-core
```

If all callers are tests, delete the public method and replace test assertions with repeated peeling. Do not retain a public “depth” API that cannot count through heap spills without `VM` access.

- [ ] **Step 6: Rename allocation-free `wrap_some` to make its limit explicit.**

Replace:

```rust
pub fn wrap_some(self) -> Result<Self, RuntimeError>
```

with:

```rust
pub(crate) fn try_wrap_some_inline(self) -> Result<Option<Self>, RuntimeError> {
    if self.is_nil() {
        return Err(RuntimeError::Internal(
            "private Nil cannot be wrapped in Some".into(),
        ));
    }
    Ok(self.try_push_inline_wrapper(NativeUnaryWrapperKind::Some))
}
```

Returning `Ok(None)` means “valid wrapper, but spill allocation is required.” It is not a language error.

- [ ] **Step 7: Replace `option_case()` with an inline-only helper and clearly name it.**

Use:

```rust
pub(crate) fn inline_option_case(self) -> OptionCase {
    if self.is_none() {
        OptionCase::None
    } else if self.is_some() {
        let (_, inner) = self
            .pop_inline_wrapper()
            .expect("is_some implies one inline Some wrapper");
        OptionCase::Some(inner)
    } else {
        OptionCase::NotOption
    }
}
```

A later VM-aware helper handles spill hydration.

- [ ] **Step 8: Rewrite Option unit tests to stay within inline capacity.**

Delete tests asserting arbitrary `u32` nesting. Add a test that 16 wrappers fit and the 17th returns `Ok(None)` from `try_wrap_some_inline`.

- [ ] **Step 9: Compile to discover production `wrap_some()` callers.**

```bash
cargo +stable check -p phalcom-core
```

Do not patch callers ad hoc. Task 7 introduces the total VM-aware API and then migrates them systematically.

**Commit Tasks 3+4 together if necessary:**

```text
refactor(value): replace Some depth with generic unary wrapper cells
```

---

# Phase D — Add transparent spill storage

## Task 5: Add the wrapper spill heap payload

**Why this layer owns the fix:** overflow is still a value representation concern, but heap ownership/tracing belongs to `heap`. The spill object must be an internal VM object containing only logical wrapper codes and a `Value` payload.

**Files:**

- Modify: `phalcom-core/src/heap/adt.rs`
- Modify: `phalcom-core/src/heap/mod.rs`
- Modify: `phalcom-core/src/heap/object.rs`
- Modify: `phalcom-core/src/heap/trace.rs`

### Steps

- [ ] **Step 1: Add the payload struct to `heap/adt.rs`.**

Below `AdtCaseObject`, insert:

```rust
use crate::value::repr::NativeUnaryWrapperKind;

/// Internal overflow node for native unary ADT wrappers.
///
/// `wrappers` is outermost-first and contains at most
/// `INLINE_NATIVE_WRAPPER_CAPACITY` entries. The surface value may itself carry
/// additional inline wrappers around this spill object's `ObjRef`.
#[derive(Clone, Debug)]
pub struct NativeWrapperSpillObject {
    pub payload: Value,
    pub wrappers: Box<[NativeUnaryWrapperKind]>,
}

impl NativeWrapperSpillObject {
    pub fn new(
        payload: Value,
        wrappers: Box<[NativeUnaryWrapperKind]>,
    ) -> Self {
        debug_assert!(!wrappers.is_empty());
        debug_assert!(wrappers.len() <= crate::value::repr::INLINE_NATIVE_WRAPPER_CAPACITY);
        Self { payload, wrappers }
    }
}
```

If `repr` is private to the `value` module and this import does not compile, re-export `NativeUnaryWrapperKind` and `INLINE_NATIVE_WRAPPER_CAPACITY` as `pub(crate)` from `phalcom-core/src/value/mod.rs`; do **not** make them `pub` outside the crate.

- [ ] **Step 2: Export the spill type from `heap/mod.rs`.**

Change:

```rust
pub use adt::AdtCaseObject;
```

to:

```rust
pub use adt::{AdtCaseObject, NativeWrapperSpillObject};
```

- [ ] **Step 3: Add an `Object` variant.**

In `heap/object.rs`, immediately after `Object::AdtCase`, add:

```rust
/// Internal spill node for native unary ADT wrapper chains.
/// This is never a source-visible object/class identity.
NativeWrapperSpill(Box<super::adt::NativeWrapperSpillObject>),
```

- [ ] **Step 4: Add a dedicated allocator to `Heap`.**

In `heap/mod.rs`, immediately after `alloc_adt_case`, add:

```rust
pub(crate) fn alloc_native_wrapper_spill(
    &mut self,
    payload: crate::value::Value,
    wrappers: Box<[crate::value::NativeUnaryWrapperKind]>,
) -> ObjRef {
    self.insert(Object::NativeWrapperSpill(Box::new(
        NativeWrapperSpillObject::new(payload, wrappers),
    )))
}
```

Adjust the path to the crate-private re-export chosen above.

- [ ] **Step 5: Make GC tracing exhaustive.**

In `heap/trace.rs`, immediately after the `Object::AdtCase` arm, add:

```rust
Object::NativeWrapperSpill(spill) => {
    trace_value(spill.payload, push);
}
```

The wrapper array contains only small enum values and has no edges.

- [ ] **Step 6: Update `Heap::kind_of_for_test`.**

Find the exhaustive match in `heap/mod.rs` and add:

```rust
Some(Object::NativeWrapperSpill(_)) => "NativeWrapperSpill",
```

- [ ] **Step 7: Compile to locate any other exhaustive `Object` matches.**

```bash
cargo +stable check -p phalcom-core
```

Because `Object` matching is deliberately exhaustive, the compiler is the checklist. Add a `NativeWrapperSpill` arm to every error location. For inspection/debug formatters, represent it as an internal object; never claim its surface class is a user class.

- [ ] **Step 8: Update normative GC documentation if the repository requires every new edge there.**

`heap/trace.rs` explicitly states that new handle-bearing object fields must be added to `docs/spec/v0.2/memory-management.md §2.3`. Add `NativeWrapperSpill.payload` to the outgoing-edge table in the same commit.

**Commit:**

```text
feat(value): add GC-traced native wrapper spill object
```

---

# Phase E — Add total VM-aware wrapper operations

## Task 6: Add conversion between runtime storage kinds and physical wrapper codes

**Why this layer owns the fix:** `adt.rs` expresses runtime storage policy; `value/repr.rs` expresses physical bits. The translation belongs in runtime code, not semantics.

**Files:**

- Modify: `phalcom-core/src/adt.rs` or `phalcom-core/src/vm/adt.rs`

### Steps

- [ ] **Step 1: Prefer an explicit conversion implementation in `adt.rs`.**

Add:

```rust
impl NativeAdtUnaryKind {
    pub(crate) const fn wrapper_kind(self) -> crate::value::NativeUnaryWrapperKind {
        match self {
            Self::OptionSome => crate::value::NativeUnaryWrapperKind::Some,
            Self::ResultOk => crate::value::NativeUnaryWrapperKind::ResultOk,
            Self::ResultError => crate::value::NativeUnaryWrapperKind::ResultError,
        }
    }
}
```

If the crate-private re-export is not yet present, add it in `value/mod.rs`:

```rust
pub(crate) use repr::{
    INLINE_NATIVE_WRAPPER_CAPACITY,
    NativeUnaryWrapperKind,
};
```

- [ ] **Step 2: Do not implement the reverse mapping by source strings.**

The reverse runtime map will come from `RuntimeAdtRegistry` in Task 10.

- [ ] **Step 3: Run:**

```bash
cargo +stable check -p phalcom-core
```

---

## Task 7: Implement canonical wrapper spill/hydration on `VM`

**Why this layer owns the fix:** pure `Value` cannot allocate. A total operation that promises “push a wrapper or spill” must receive `&mut VM`/heap access.

**Files:**

- Modify: `phalcom-core/src/vm/adt.rs`

### Representation rule

A spill is canonicalized in chunks of at most 16 wrappers:

```text
Value metadata
    up to 16 outer wrappers
    payload tag = Obj(spill)

spill.wrappers
    next up to 16 wrappers, outermost-first

spill.payload
    base Value or Obj(next spill)
```

The implementation never creates an empty spill node.

### Steps

- [ ] **Step 1: Add an internal helper to collect current inline wrappers.**

In `vm/adt.rs`, add:

```rust
fn inline_wrappers(value: Value) -> Vec<crate::value::NativeUnaryWrapperKind> {
    let mut current = value;
    let mut result = Vec::new();
    while let Some((wrapper, inner)) = current.pop_inline_wrapper() {
        result.push(wrapper);
        current = inner;
    }
    result
}
```

If `pop_inline_wrapper` is not visible from `vm`, expose it crate-wide (`pub(crate)`), not publicly.

- [ ] **Step 2: Add total `VM::wrap_native_unary`.**

Add to `impl VM`:

```rust
pub(crate) fn wrap_native_unary(
    &mut self,
    value: Value,
    wrapper: crate::value::NativeUnaryWrapperKind,
) -> Result<Value, RuntimeError> {
    if value.is_nil() {
        return Err(RuntimeError::Internal(
            "private Nil cannot be wrapped in a native ADT case".into(),
        ));
    }

    if let Some(inline) = value.try_push_inline_wrapper(wrapper) {
        return Ok(inline);
    }

    let wrappers = inline_wrappers(value);
    debug_assert_eq!(
        wrappers.len(),
        crate::value::INLINE_NATIVE_WRAPPER_CAPACITY,
    );

    let payload = value.without_inline_wrappers();
    let spill = self
        .heap
        .alloc_native_wrapper_spill(payload, wrappers.into_boxed_slice());

    Value::obj(spill)
        .try_push_inline_wrapper(wrapper)
        .ok_or_else(|| RuntimeError::Internal(
            "fresh native wrapper spill could not accept outer wrapper".into(),
        ))
}
```

This creates a new immutable spill node rather than mutating an existing one.

- [ ] **Step 3: Add a helper that recognizes only internal spill objects.**

```rust
fn native_wrapper_spill(
    &self,
    value: Value,
) -> Option<&crate::heap::NativeWrapperSpillObject> {
    let obj = value.as_obj()?;
    match self.heap.get(obj) {
        Object::NativeWrapperSpill(spill) => Some(spill),
        _ => None,
    }
}
```

Note: `Value::as_obj()` intentionally rejects values with inline wrappers. Call this helper only after the inline wrapper field is empty.

- [ ] **Step 4: Add `VM::unwrap_native_unary`.**

```rust
pub(crate) fn unwrap_native_unary(
    &self,
    value: Value,
) -> Option<(crate::value::NativeUnaryWrapperKind, Value)> {
    if let Some((wrapper, inner)) = value.pop_inline_wrapper() {
        return Some((wrapper, inner));
    }

    let spill = self.native_wrapper_spill(value)?;
    let (&outer, rest) = spill.wrappers.split_first()?;
    let mut inner = spill.payload;

    for wrapper in rest.iter().rev().copied() {
        inner = inner
            .try_push_inline_wrapper(wrapper)
            .expect("spill tail is bounded by inline wrapper capacity");
    }

    Some((outer, inner))
}
```

Why iterate `rest` in reverse? `try_push_inline_wrapper` pushes a new **outer** wrapper. If `rest` is outermost-first, rebuilding from the innermost entry outward preserves order.

- [ ] **Step 5: Add a VM test that wraps 17 times and unwraps 17 times.**

Use the lowest-level helper directly. Assert:

```text
wrapper pushes 1..16 => no spill object in outer base
push 17 => construction succeeds
17 unwrap operations all return the expected wrapper kind
final payload == original scalar
```

Do not assert a specific `ObjRef` value.

- [ ] **Step 6: Add a 33-wrapper test.**

This forces nested spill chunks. The final payload must still round-trip.

- [ ] **Step 7: Run:**

```bash
cargo +stable test -p phalcom-core native_wrapper
cargo +stable check -p phalcom-core
```

**Commit:**

```text
feat(value): add transparent native unary wrapper spill path
```

---

# Phase F — Migrate all Option construction/inspection onto the total VM API

## Task 8: Replace production `Value::wrap_some()` call sites

**Why this layer owns the fix:** after spill support, any operation that can create a user-visible `Some` must use the total VM path. Leaving one direct inline-only constructor would reintroduce a depth-dependent runtime failure.

**Files:** determined by search; known reviewed hits include:

- `phalcom-core/src/modules/builtin_materialize.rs`
- `phalcom-core/src/modules/materialize.rs`
- `phalcom-core/src/vm/adt.rs`
- other primitive/runtime files returned by the search

### Steps

- [ ] **Step 1: Inventory every direct caller.**

Run:

```bash
rg -n '\.wrap_some\(\)' phalcom-core/src phalcom-core/tests
```

Save the list in the task notes before editing.

- [ ] **Step 2: Replace runtime/module call sites that already have `&mut self`/`&mut VM`.**

For a current pattern such as:

```rust
Value::obj(module).wrap_some()?
```

replace with:

```rust
self.wrap_native_unary(
    Value::obj(module),
    crate::value::NativeUnaryWrapperKind::Some,
)?
```

or add a convenience wrapper on VM:

```rust
pub(crate) fn wrap_some_value(&mut self, value: Value) -> Result<Value, RuntimeError> {
    self.wrap_native_unary(value, crate::value::NativeUnaryWrapperKind::Some)
}
```

and use:

```rust
self.wrap_some_value(Value::obj(module))?
```

Prefer the convenience helper because Option creation occurs in multiple modules.

- [ ] **Step 3: Replace `Option` constructor behavior in `VM::construct_variant_value`.**

The current NativeOption branch returns:

```rust
return Ok(value.wrap_some()?);
```

Replace with:

```rust
return self.wrap_some_value(*value);
```

- [ ] **Step 4: Remove the old public `Value::wrap_some` method after all production callers migrate.**

Keep only `try_wrap_some_inline` for low-level unit tests, or delete even that convenience and use `try_push_inline_wrapper` directly.

- [ ] **Step 5: Search again.**

```bash
rg -n '\.wrap_some\(\)' phalcom-core/src
```

Expected: zero production hits.

- [ ] **Step 6: Run:**

```bash
cargo +stable check -p phalcom-core
cargo +stable test -p phalcom-core option
```

---

## Task 9: Replace Option case inspection with a VM-aware operation

**Why this layer owns the fix:** after the 17th wrapper, peeling can require reading a spill object. `Value` alone cannot do that safely because it has no heap access.

**Files:**

- Modify: `phalcom-core/src/vm/adt.rs`
- Modify: Option native primitive implementation(s) found by search
- Modify: any runtime consumers of `option_case()`

### Steps

- [ ] **Step 1: Add VM-level Option case.**

Use the existing `OptionCase` enum but expose it crate-wide if necessary. Add:

```rust
pub(crate) fn option_case(&self, value: Value) -> crate::value::option::OptionCase {
    if value.is_none() {
        return crate::value::option::OptionCase::None;
    }

    match self.unwrap_native_unary(value) {
        Some((crate::value::NativeUnaryWrapperKind::Some, inner)) => {
            crate::value::option::OptionCase::Some(inner)
        }
        _ => crate::value::option::OptionCase::NotOption,
    }
}
```

- [ ] **Step 2: Inventory direct `option_case()` / `is_some()` runtime uses.**

```bash
rg -n 'option_case\(|is_some\(\)|is_option\(\)' phalcom-core/src
```

Classify each call:

```text
surface outer-variant predicate only -> Value::is_some is okay
needs payload peeling / full Option semantics -> use VM::option_case
```

- [ ] **Step 3: Migrate the native `Option.match` primitive to `VM::option_case`.**

Do not make it inspect `Object::NativeWrapperSpill` directly.

- [ ] **Step 4: Change `VM::case_payload_at` NativeOption handling to use `unwrap_native_unary`.**

Delete arithmetic such as:

```rust
value.with_some_depth(value.some_depth_raw() - 1)
```

and return the inner `Value` from the generic unwrap helper.

- [ ] **Step 5: Delete `inline_option_case` if no caller remains outside low-level tests.**

- [ ] **Step 6: Run:**

```bash
cargo +stable test -p phalcom-core option
cargo +stable test -p phalcom-core --test native_adt_runtime
```

**Commit:**

```text
refactor(option): route construction and peeling through native wrapper runtime
```

---

# Phase G — Move storage authorization into semantic lowering

## Task 10: Add storage to `VariantLoweringSpec`

**Why this layer owns the fix:** by the time VM registration runs, the compiler already knows the exact canonical `VariantId`. The semantic-lowering projection should authorize storage once. VM registration should consume the decision rather than rediscovering “this is Option/Result”.

**Files:**

- Modify: `phalcom-core/src/modules/semantic_lowering.rs`
- Modify: tests constructing `VariantLoweringSpec` manually, especially `phalcom-core/tests/native_adt_runtime.rs`

### Steps

- [ ] **Step 1: Add the field.**

Find:

```rust
pub struct VariantLoweringSpec {
    pub id: VariantId,
    pub shape: VariantShape,
    pub payload_fields: Box<[VariantFieldLoweringSpec]>,
}
```

Change to:

```rust
pub struct VariantLoweringSpec {
    pub id: VariantId,
    pub shape: VariantShape,
    pub payload_fields: Box<[VariantFieldLoweringSpec]>,
    pub storage: crate::adt::RuntimeVariantStorage,
}
```

- [ ] **Step 2: Add a helper that computes storage from full canonical identity.**

Near `build_module_lowering_semantics`, add:

```rust
fn runtime_variant_storage(
    owner: &DeclarationId,
    variant: &VariantId,
    shape: VariantShape,
    payload_arity: usize,
) -> crate::adt::RuntimeVariantStorage {
    let ids = phalcom_semantic::core_surface::CoreDeclarationIds::default();

    if ids.is_option(owner) {
        if variant.selector.base.as_named() == Some("Some")
            && shape == VariantShape::Constructor
            && payload_arity == 1
        {
            return crate::adt::RuntimeVariantStorage::NativeUnary(
                crate::adt::NativeAdtUnaryKind::OptionSome,
            );
        }

        if variant.selector.base.as_named() == Some("None")
            && shape == VariantShape::Singleton
            && payload_arity == 0
        {
            return crate::adt::RuntimeVariantStorage::NativeSingleton(
                crate::adt::NativeAdtSingletonKind::OptionNone,
            );
        }
    }

    if ids.is_result(owner) {
        if variant.selector.base.as_named() == Some("Ok")
            && shape == VariantShape::Constructor
            && payload_arity == 1
        {
            return crate::adt::RuntimeVariantStorage::NativeUnary(
                crate::adt::NativeAdtUnaryKind::ResultOk,
            );
        }

        if variant.selector.base.as_named() == Some("Error")
            && shape == VariantShape::Constructor
            && payload_arity == 1
        {
            return crate::adt::RuntimeVariantStorage::NativeUnary(
                crate::adt::NativeAdtUnaryKind::ResultError,
            );
        }
    }

    crate::adt::RuntimeVariantStorage::General
}
```

If `SelectorBase` does not currently expose `as_named()`, do not add a string helper to `SelectorBase` just for this. Pattern-match:

```rust
match &variant.selector.base {
    SelectorBase::Named(name) => name.as_str(),
    _ => return General,
}
```

The **owner identity check must happen before name inspection**, so a user `Result::Ok` cannot receive native storage.

- [ ] **Step 3: Attach storage while projecting variants.**

In the enum loop, after `payload_fields` has been built, compute:

```rust
let storage = runtime_variant_storage(
    owner,
    variant_id,
    shape,
    payload_fields.len(),
);
```

Then include `storage` in `VariantLoweringSpec`.

- [ ] **Step 4: Add a structural validation failure for malformed canonical native enums.**

Do not silently fall back to General if the owner is canonical Option/Result but its expected variants have the wrong shape. The safest implementation is a separate `validate_native_adt_layout` pass returning a new `ProjectionError` such as:

```rust
#[error("canonical native ADT `{0}` no longer matches its required runtime layout")]
InvalidCanonicalNativeAdtLayout(DeclarationId),
```

Validate Option and Result exactly once per enum before projecting variants.

- [ ] **Step 5: Update manual test fixtures.**

Every `VariantLoweringSpec { ... }` in tests must now set `storage` explicitly. Use `General` unless the fixture is intentionally testing canonical native storage.

- [ ] **Step 6: Fix the existing native Option runtime test's declaration identity.**

The reviewed test currently constructs:

```rust
DeclarationId::new(ModuleId::universe_root(), "Option".into())
```

Replace it with:

```rust
phalcom_semantic::core_surface::universe_declaration(
    phalcom_native_meta::UniverseKey::Option,
)
```

The test should exercise the actual canonical declaration.

- [ ] **Step 7: Add lowering tests proving user shadowing stays General.**

Create a user enum named `Result` with unary `Ok`/`Error`. Assert both projected variants are `RuntimeVariantStorage::General`.

- [ ] **Step 8: Run:**

```bash
cargo +stable test -p phalcom-core semantic_lowering
cargo +stable check -p phalcom-core
```

**Commit:**

```text
feat(lowering): authorize native unary storage by canonical VariantId
```

---

## Task 11: Make runtime registration consume `VariantLoweringSpec.storage`

**Files:**

- Modify: `phalcom-core/src/vm/adt.rs`

### Steps

- [ ] **Step 1: Delete the temporary storage inference added in Task 2.**

Remove the `if spec.representation == NativeOption { ... }` block that manufactures storage at registration time.

- [ ] **Step 2: Pass `var_spec.storage` directly into `register_variant`.**

Use:

```rust
let runtime_var_id = self.adt_registry.register_variant(
    var_spec.id.clone(),
    enum_id,
    discriminant,
    shape,
    payload_arity,
    var_spec.storage,
    case_class_id,
    None,
);
```

- [ ] **Step 3: Keep enum-level `RuntimeAdtRepresentation` temporarily.**

Do not delete it yet. It still controls root/case binding behavior for Option at the reviewed baseline. A later cleanup task reduces its role only after the variant-storage path is proven.

- [ ] **Step 4: Run:**

```bash
cargo +stable test -p phalcom-core --test native_adt_runtime
```

---

# Phase H — Register native wrapper kinds to exact runtime variants

## Task 12: Generalize `RuntimeAdtRegistry` native mappings

**Why this layer owns the fix:** `runtime_variant_of(Value)` must map a physical wrapper code back to the exact runtime descriptor without reconstructing `VariantId` by strings. The registry is the correct bridge.

**Files:**

- Modify: `phalcom-core/src/adt.rs`
- Modify: `phalcom-core/src/vm/adt.rs`

### Steps

- [ ] **Step 1: Replace `NativeOptionVariantIds` with generic native maps.**

In `RuntimeAdtRegistry`, replace:

```rust
native_option: Option<NativeOptionVariantIds>,
```

with:

```rust
native_unary_variants: HashMap<NativeAdtUnaryKind, RuntimeVariantId>,
native_singleton_variants: HashMap<NativeAdtSingletonKind, RuntimeVariantId>,
```

`#[derive(Default)]` will initialize both maps.

- [ ] **Step 2: Add binding/query helpers.**

```rust
pub fn bind_native_unary_variant(
    &mut self,
    kind: NativeAdtUnaryKind,
    variant: RuntimeVariantId,
) -> Result<(), &'static str> {
    if self.native_unary_variants.insert(kind, variant).is_some() {
        return Err("native unary ADT kind already bound");
    }
    Ok(())
}

pub fn native_unary_variant(
    &self,
    kind: NativeAdtUnaryKind,
) -> Option<RuntimeVariantId> {
    self.native_unary_variants.get(&kind).copied()
}

pub fn bind_native_singleton_variant(
    &mut self,
    kind: NativeAdtSingletonKind,
    variant: RuntimeVariantId,
) -> Result<(), &'static str> {
    if self.native_singleton_variants.insert(kind, variant).is_some() {
        return Err("native singleton ADT kind already bound");
    }
    Ok(())
}

pub fn native_singleton_variant(
    &self,
    kind: NativeAdtSingletonKind,
) -> Option<RuntimeVariantId> {
    self.native_singleton_variants.get(&kind).copied()
}
```

- [ ] **Step 3: Delete `NativeOptionVariantIds`, `bind_native_option_variants`, and `native_option_variants` after callers migrate.**

- [ ] **Step 4: Bind every native storage descriptor during `register_enum_from_spec`.**

Immediately after `register_variant`, match `var_spec.storage`:

```rust
match var_spec.storage {
    RuntimeVariantStorage::NativeUnary(kind) => {
        self.adt_registry
            .bind_native_unary_variant(kind, runtime_var_id)
            .map_err(|message| RuntimeError::Internal(message.into()))?;
    }
    RuntimeVariantStorage::NativeSingleton(kind) => {
        self.adt_registry
            .bind_native_singleton_variant(kind, runtime_var_id)
            .map_err(|message| RuntimeError::Internal(message.into()))?;
    }
    RuntimeVariantStorage::General => {}
}
```

- [ ] **Step 5: Remove `some_runtime_opt` / `none_runtime_opt` local variables and the final Option-specific bind block.**

The generic storage binding replaces them.

- [ ] **Step 6: Add registry unit tests.**

Assert duplicate binding of the same native kind is rejected and querying a bound kind returns the exact runtime variant.

- [ ] **Step 7: Run:**

```bash
cargo +stable test -p phalcom-core adt
```

**Commit:**

```text
refactor(adt): generalize native storage runtime variant registry
```

---

# Phase I — Construct `Result` as native wrappers

## Task 13: Make `construct_variant_value` storage-driven

**Why this layer owns the fix:** `construct_variant_value` already has the exact `RuntimeVariantDescriptor`. It should switch on `variant_desc.storage`, not on enum names and not primarily on enum-wide representation.

**Files:**

- Modify: `phalcom-core/src/vm/adt.rs`

### Steps

- [ ] **Step 1: Refactor payload-arity validation to happen before storage switch.**

After loading `variant_desc`, compare:

```rust
payload.len()
```

with:

```rust
variant_desc.payload_arity as usize
```

and return a descriptive `RuntimeError::Message` if they differ. This avoids duplicating arity checks in each storage branch.

- [ ] **Step 2: Replace the outer `match enum_desc.representation` constructor switch with `match variant_desc.storage`.**

Target shape:

```rust
match variant_desc.storage {
    RuntimeVariantStorage::NativeUnary(kind) => {
        let [value] = payload.as_slice() else {
            unreachable!("arity validated above");
        };
        self.wrap_native_unary(*value, kind.wrapper_kind())
    }

    RuntimeVariantStorage::NativeSingleton(
        NativeAdtSingletonKind::OptionNone,
    ) => Ok(Value::none()),

    RuntimeVariantStorage::General => {
        if variant_desc.shape == RuntimeVariantShape::Singleton {
            return variant_desc.singleton.ok_or_else(|| {
                RuntimeError::Internal(
                    "general singleton variant missing canonical singleton value".into(),
                )
            });
        }

        let obj_ref = self
            .heap
            .alloc_adt_case(variant, payload.into_boxed_slice());
        Ok(Value::obj(obj_ref))
    }
}
```

- [ ] **Step 3: Preserve ordinary singleton behavior.**

Review `register_enum_from_spec`: General singleton variants currently receive `Value::adt_singleton(runtime_var_id)` in the descriptor. Do not allocate an `AdtCase` for them.

- [ ] **Step 4: Do not add a `NativeResult` enum representation branch.**

The point of this task is that `Result::Ok` and `Result::Error` are native by **variant storage**, not by another hard-coded enum-wide constructor path.

- [ ] **Step 5: Add runtime construction tests for canonical Result.**

Build the canonical Result `EnumLoweringSpec`, register it, construct `Ok(Value::int(7))`, and assert:

```text
returned Value is not Object::AdtCase
runtime_variant_of == canonical Result::Ok runtime variant
case_payload_at(..., 0) == Int(7)
```

Repeat for `Error`.

- [ ] **Step 6: Add a user `Result` control test.**

Construct a user enum named `Result` with the same variant names but `storage: General`. Assert construction allocates `Object::AdtCase` and does not report the canonical Universe runtime variant.

- [ ] **Step 7: Run:**

```bash
cargo +stable test -p phalcom-core --test native_adt_runtime
```

**Commit:**

```text
feat(result): construct canonical Result with native unary wrappers
```

---

# Phase J — Make variant inspection representation-independent

## Task 14: Rewrite `runtime_variant_of` around storage mappings

**Files:**

- Modify: `phalcom-core/src/vm/adt.rs`

### Steps

- [ ] **Step 1: Keep General singleton and `AdtCase` checks.**

The beginning may still recognize:

```text
Value::adt_singleton -> RuntimeVariantId
Object::AdtCase      -> case.variant
```

- [ ] **Step 2: Add native outer-wrapper recognition before treating wrapped Obj values as ordinary objects.**

Use:

```rust
if let Some(wrapper) = value.inline_outer_wrapper() {
    let kind = match wrapper {
        NativeUnaryWrapperKind::Some => NativeAdtUnaryKind::OptionSome,
        NativeUnaryWrapperKind::ResultOk => NativeAdtUnaryKind::ResultOk,
        NativeUnaryWrapperKind::ResultError => NativeAdtUnaryKind::ResultError,
    };
    return self.adt_registry.native_unary_variant(kind);
}
```

- [ ] **Step 3: Recognize immediate `None`.**

Use:

```rust
if value.is_none() {
    return self
        .adt_registry
        .native_singleton_variant(NativeAdtSingletonKind::OptionNone);
}
```

- [ ] **Step 4: Delete the old `value.is_option()` / `native_option_variants()` branch.**

- [ ] **Step 5: Add mixed-order tests.**

Construct:

```text
Some(Ok(1))
Ok(Some(1))
```

Assert the first outer runtime variant is Option::Some and the second is Result::Ok.

---

## Task 15: Rewrite payload length and payload extraction

**Files:**

- Modify: `phalcom-core/src/vm/adt.rs`

### Steps

- [ ] **Step 1: Make `case_payload_len` consult `runtime_variant_of` and descriptor shape/storage.**

Recommended flow:

```rust
pub fn case_payload_len(&self, value: Value) -> Option<usize> {
    let runtime_variant = self.runtime_variant_of(value)?;
    let descriptor = self.adt_registry.variant_descriptor(runtime_variant)?;
    Some(descriptor.payload_arity as usize)
}
```

This removes representation-specific branching entirely.

- [ ] **Step 2: Make `case_payload_at` descriptor-driven.**

First load `runtime_variant_of` and descriptor. Validate `index < payload_arity`.

Then:

```rust
match descriptor.storage {
    RuntimeVariantStorage::NativeUnary(_) => {
        let (_, inner) = self.unwrap_native_unary(value).ok_or_else(...)?;
        if index == 0 { Ok(inner) } else { unreachable!() }
    }
    RuntimeVariantStorage::NativeSingleton(_) => Err(...len 0...),
    RuntimeVariantStorage::General => {
        // existing AdtCase / singleton payload logic
    }
}
```

- [ ] **Step 3: Do not identify native wrapper kind twice.**

Once `runtime_variant_of(value)` has established the exact `RuntimeVariantId`, use its descriptor. The physical wrapper decoder is only the bridge into the registry.

- [ ] **Step 4: Add nested extraction tests.**

Assert:

```text
payload(Some(Ok(1)))      == Ok(1)
payload(Ok(Some(1)))      == Some(1)
payload(Error(Ok(1)))     == Ok(1)
payload(Ok(Error("e")))   == Error("e")
```

Then call `runtime_variant_of` on the extracted value to prove the inner wrapper remained intact.

- [ ] **Step 5: Force a spill boundary and repeat the exact same extraction assertions.**

Use helper construction to place 17+ wrappers around a scalar and peel them one at a time.

- [ ] **Step 6: Run:**

```bash
cargo +stable test -p phalcom-core --test native_adt_runtime
cargo +stable test -p phalcom-core adt
```

**Commit:**

```text
refactor(adt): make case inspection storage-independent
```

---

# Phase K — Preserve `.class`, dispatch, and reflection semantics

## Task 16: Audit every class-resolution path for wrapper transparency

**Why this layer owns the fix:** wrapper bits are physical. Surface class must continue to come from the exact runtime variant descriptor's `behavior_class`.

**Files:**

- Primary: `phalcom-core/src/vm/adt.rs`
- Inspect/modify if necessary: `phalcom-core/src/value/class.rs`
- Search all `case_behavior_class`, `runtime_variant_of`, `Value::class` call sites

### Steps

- [ ] **Step 1: Keep `case_behavior_class` structurally simple.**

It should remain equivalent to:

```rust
pub fn case_behavior_class(&self, value: Value) -> Option<ClassId> {
    let rid = self.runtime_variant_of(value)?;
    self.adt_registry
        .variant_descriptor(rid)
        .map(|descriptor| descriptor.behavior_class)
}
```

Do not add `Some`/`Ok`/`Error` bit checks here.

- [ ] **Step 2: Search direct Option-class special cases.**

```bash
rg -n 'some_class|none_class|option_class|case_behavior_class|runtime_variant_of' phalcom-core/src
```

For each value-class path, ensure native Result values reach `case_behavior_class` before falling back to the base scalar/object class.

- [ ] **Step 3: Add class tests.**

Assert:

```text
Ok(1).class       == hidden Result::Ok behavior class
Error("e").class == hidden Result::Error behavior class
Some(Ok(1)).class == Option::Some behavior class
Ok(Some(1)).class == Result::Ok behavior class
```

- [ ] **Step 4: Add spill parity.**

For a spilled chain whose outer wrapper is `ResultOk`, assert class resolution returns the same Result::Ok behavior class as an inline `Ok` value.

- [ ] **Step 5: Verify canonical Result root remains unchanged.**

Re-run the C-01 cross-registry identity tests from the correctness plan. This feature must not allocate another Result root class.

---

# Phase L — Equality and hash behavior

## Task 17: Keep Rust bit identity separate from language value equality

**Why this layer owns the fix:** independently allocated spill nodes have different `ObjRef`s. Therefore Rust `Value::PartialEq` cannot by itself represent logical ADT equality across spills without heap access. Phalcom language equality already has a method/VM semantic layer; keep the distinction explicit.

**Files:**

- Inspect: `phalcom-core/src/value/repr.rs`
- Inspect/modify: runtime equality/hash dispatch implementation
- Verify source methods: `option.ph`, `result.ph`

### Steps

- [ ] **Step 1: Do not make `Value::PartialEq` dereference the heap.**

`Value` deliberately has no heap reference. Keep `PartialEq` a representation-level operation for raw runtime infrastructure.

- [ ] **Step 2: Preserve `Value::same_bits` for `===`.**

Do not change its two-word comparison.

- [ ] **Step 3: Confirm language `Option#==` and `Result#==` compare through variant/payload semantics rather than raw `ObjRef`.**

If `Result#==` is absent, add/complete it at the Universe source level rather than teaching `Value::PartialEq` about Result.

- [ ] **Step 4: Confirm language hash methods derive from logical payload/variant.**

Equal values must hash equal even when wrapper chain storage crosses a spill boundary.

- [ ] **Step 5: Add a forced-spill equality test through language `==`.**

Construct two logically equal deep wrapper chains separately so they have different spill `ObjRef`s. Assert language equality is true.

- [ ] **Step 6: Add a hash parity test.**

The same two values must produce equal language `hash` results.

- [ ] **Step 7: Add an order-sensitivity test.**

Assert logically:

```text
Some(Ok(1)) != Ok(Some(1))
Ok(Error(1)) != Error(Ok(1))
```

---

# Phase M — GC correctness

## Task 18: Prove wrapped object payloads remain live inline and spilled

**Files:**

- Modify/add tests in existing GC test module(s)
- Already modified: `phalcom-core/src/heap/trace.rs`

### Steps

- [ ] **Step 1: Add an inline-wrapper GC test.**

Allocate an object, wrap it with `ResultOk` while still inline, remove every other root, trigger GC at a legal safepoint, then unwrap and verify the object handle remains live.

This validates that `Value::gc_obj_ref()` continues tracing the base `ObjRef` through metadata wrappers.

- [ ] **Step 2: Add a spill-wrapper GC test.**

Create more than 16 wrappers around an object so `Object::NativeWrapperSpill` is allocated. Retain only the outer wrapped `Value`. Trigger GC. Peel all wrappers and verify the underlying object remains live.

- [ ] **Step 3: Add a nested-spill test (>32 wrappers).**

This proves `NativeWrapperSpill.payload` recursively traces the next spill object.

- [ ] **Step 4: Run with GC stress if supported.**

```bash
PHALCOM_GC_STRESS=1 cargo +stable test -p phalcom-core native_wrapper
```

- [ ] **Step 5: Run the ordinary GC suite.**

```bash
cargo +stable test -p phalcom-core gc
```

**Commit:**

```text
test(gc): prove native wrapper spill reachability
```

---

# Phase N — Remove obsolete Option-depth architecture

## Task 19: Delete depth-only symbols and errors

**Files:**

- Modify: `phalcom-core/src/value/repr.rs`
- Modify: `phalcom-core/src/value/option.rs`
- Modify: `phalcom-core/src/error.rs` if `OptionNestingLimit` becomes unused
- Modify tests/docs mentioning Some depth

### Steps

- [ ] **Step 1: Search all old representation names.**

```bash
rg -n 'some_depth_raw|with_some_depth|without_some_wrappers|DEPTH_SHIFT|DEPTH_MASK|MAX_OPTION_NESTING|OptionNestingLimit' .
```

- [ ] **Step 2: Remove all production uses.**

There should be no runtime semantic path that reasons about a numeric Some depth after the shared wrapper stack is complete.

- [ ] **Step 3: Delete `RuntimeError::OptionNestingLimit` if no caller remains.**

Overflow now spills; it is not a user-visible error.

- [ ] **Step 4: Update comments/documents that claim nested Some is represented by a 32-bit depth counter.**

The new normative statement is:

```text
Native unary ADT wrappers use a bounded inline wrapper stack and spill
transparently when the inline metadata budget is exhausted.
```

- [ ] **Step 5: Re-run the search.**

Expected production result: zero hits for all deleted depth symbols.

---

# Phase O — Reduce obsolete enum-wide representation special casing

## Task 20: Reassess `RuntimeAdtRepresentation::NativeOption`

**Why this task comes last:** root-class binding and variant storage were historically both encoded by `RuntimeAdtRepresentation::NativeOption`. After the correctness plan fixes root identity and this plan moves value storage to `RuntimeVariantStorage`, the enum-wide flag may no longer be needed for construction.

**Files:**

- Modify: `phalcom-core/src/adt.rs`
- Modify: `phalcom-core/src/vm/adt.rs`
- Modify: `phalcom-core/src/modules/semantic_lowering.rs`

### Steps

- [ ] **Step 1: Search remaining uses.**

```bash
rg -n 'RuntimeAdtRepresentation|NativeOption' phalcom-core/src phalcom-core/tests
```

- [ ] **Step 2: Classify each use.**

```text
root/case class reuse          -> canonical enum binding policy
value storage/constructor      -> RuntimeVariantStorage
pattern matching               -> RuntimeVariantId / descriptor
reflection                     -> semantic/runtime descriptor
```

- [ ] **Step 3: If no independent purpose remains, collapse `RuntimeAdtRepresentation` to `General`/remove it.**

Do this only if the C-01 runtime-root solution already makes canonical Option root/case reuse independent of this enum.

- [ ] **Step 4: If it remains useful for root binding, rename it to express that purpose.**

For example:

```rust
RuntimeEnumClassBindingPolicy
```

would be clearer than a name that implies value representation.

- [ ] **Step 5: Do not add `NativeResult` here.**

Result's native values are already represented through per-variant storage.

---

# Phase P — Full regression matrix

## Task 21: Expand `phalcom-core/tests/native_adt_runtime.rs`

**Files:**

- Modify: `phalcom-core/tests/native_adt_runtime.rs`

### Required tests

- [ ] `canonical_option_some_uses_native_unary_storage`
- [ ] `canonical_option_none_uses_native_singleton_storage`
- [ ] `canonical_result_ok_uses_native_unary_storage`
- [ ] `canonical_result_error_uses_native_unary_storage`
- [ ] `user_result_named_like_builtin_remains_general`
- [ ] `some_ok_preserves_outer_variant_order`
- [ ] `ok_some_preserves_outer_variant_order`
- [ ] `ok_error_preserves_outer_variant_order`
- [ ] `error_ok_preserves_outer_variant_order`
- [ ] `native_unary_payload_peels_exactly_one_layer`
- [ ] `sixteen_wrappers_remain_inline`
- [ ] `seventeenth_wrapper_spills_transparently`
- [ ] `thirty_third_wrapper_uses_nested_spill_transparently`
- [ ] `inline_and_spilled_result_report_same_behavior_class`
- [ ] `inline_and_spilled_option_report_same_behavior_class`
- [ ] `native_wrapper_spill_keeps_object_payload_alive`
- [ ] `value_remains_exactly_sixteen_bytes`

### Construction rule for tests

Do not construct canonical declarations as:

```rust
DeclarationId::new(ModuleId::universe_root(), "Result".into())
```

Use:

```rust
phalcom_semantic::core_surface::universe_declaration(
    phalcom_native_meta::UniverseKey::Result,
)
```

and analogous canonical keys for Option.

---

# Phase Q — Verification and deletion gates

## Task 22: Run the complete verification sequence

Run in this exact order so failures are localized.

- [ ] **Formatting:**

```bash
cargo +stable fmt --all -- --check
```

- [ ] **Core check:**

```bash
cargo +stable check -p phalcom-core
```

- [ ] **Value unit tests:**

```bash
cargo +stable test -p phalcom-core value
```

- [ ] **Native ADT runtime integration:**

```bash
cargo +stable test -p phalcom-core --test native_adt_runtime
```

- [ ] **ADT runtime/compiler tests:**

```bash
cargo +stable test -p phalcom-core adt
cargo +stable test -p phalcom-core match
```

- [ ] **Semantic ADT tests:**

```bash
cargo +stable test -p phalcom-semantic adt
cargo +stable test -p phalcom-semantic generic
```

- [ ] **GC tests:**

```bash
cargo +stable test -p phalcom-core gc
PHALCOM_GC_STRESS=1 cargo +stable test -p phalcom-core --test native_adt_runtime
```

- [ ] **Workspace:**

```bash
cargo +stable test --workspace --all-targets
```

- [ ] **Clippy:**

```bash
cargo +stable clippy --workspace --all-targets -- -D warnings
```

- [ ] **Zero-hit obsolete representation search:**

```bash
rg -n 'some_depth_raw|with_some_depth|without_some_wrappers|DEPTH_SHIFT|DEPTH_MASK|MAX_OPTION_NESTING|OptionNestingLimit' phalcom-core/src
```

Expected: zero hits.

- [ ] **Forbidden semantic-name search in native storage code:**

```bash
rg -n 'owner\.name\s*==\s*"(Option|Result)"|"Ok"\s*=>|"Error"\s*=>' \
  phalcom-core/src/vm \
  phalcom-core/src/adt.rs
```

Expected: no name-based authorization of native storage. Exact selector names may exist **inside semantic lowering only after the canonical owner identity has already been established**.

- [ ] **Value size gate:** verify the test still asserts and passes:

```rust
assert_eq!(std::mem::size_of::<Value>(), 16);
```

- [ ] **Reserved metadata gate:** verify bits `40..=63` remain untouched by all wrapper tests.

---

# 23. Suggested commit sequence

Keep reviewable commits rather than one giant representation patch.

```text
1. test(value): pin metadata budget before native unary wrappers
2. refactor(adt): add per-variant runtime storage policy
3. refactor(value): replace Some depth with generic unary wrapper cells
4. feat(value): add GC-traced native wrapper spill object
5. feat(value): add transparent native unary wrapper spill path
6. refactor(option): route construction and peeling through native wrapper runtime
7. feat(lowering): authorize native unary storage by canonical VariantId
8. refactor(adt): generalize native storage runtime variant registry
9. feat(result): construct canonical Result with native unary wrappers
10. refactor(adt): make case inspection storage-independent
11. test(gc): prove native wrapper spill reachability
12. refactor(value): remove obsolete Some-depth representation
13. test(adt): complete native Result mixed-wrapper regression matrix
```

Do not combine this entire plan with the correctness-remediation plan in one commit. Runtime-identity correctness must be reviewable independently of the representation optimization.

---

# 24. Definition of done

The feature is done when all layers tell the same story:

```text
source
    Result::Ok(value)

semantic
    VariantId(canonical universe.errors.result::Result, Ok(_))

lowering
    RuntimeVariantStorage::NativeUnary(ResultOk)

runtime descriptor
    RuntimeVariantId -> canonical Result::Ok behavior class

physical short value
    base Value + 2-bit ResultOk wrapper cell

physical deep value
    outer wrapper cells + NativeWrapperSpill chain

match / payload / .class / reflection
    driven by RuntimeVariantId, identical for inline and spill
```

A successful implementation therefore proves more than “`Result::Ok(1)` no longer allocates.” It proves:

- no semantic identity depends on the wrapper bits;
- no user enum accidentally receives built-in storage;
- mixed Option/Result nesting preserves order;
- overflow does not change language behavior;
- GC and class behavior remain correct;
- the `Value` ABI stays exactly 16 bytes;
- the 24-bit reserved metadata region remains available for independent future metadata.
