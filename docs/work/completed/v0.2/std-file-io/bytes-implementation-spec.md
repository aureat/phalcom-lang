# Implementation spec — `Bytes` (U-BYTES)

> **Status:** **SHIPPED 2026-07-20** — `19c5db9` (spine), `9445d1f` (core.ph + bootstrap),
> `732189b` (golden lanes), plus the flat-entry follow-on `5ba6101`; clean-worktree
> verified 164/0. This document is now the **as-built record**; §7 lists the deviations
> discovered during implementation — read §7 before writing any future impl spec, its
> items are standing obligations. Scope addition at ratification:
> [PDR-0013](../../../pdr/0013-path-is-bytes-backed-filesystem-surface.md) ruling 4's
> `utf8Lossy_` shipped here as the **eleventh** primitive (own census constant
> `NEW_BYTES_LOSSY`). Surface contract: [`../stdlib/bytes.md`](../../spec/current/stdlib/bytes.md).
> `file:line` anchors below are as of the pre-implementation tree (`4c902b3`).
>
> **Rustdoc is mandatory on every item this unit adds**
> (`docs/rust-documentation-guidelines.md`) — an undocumented public item is an incomplete
> change.

## 1. Shape of the change

The ADR-0020 kernel pattern, copied from `List` end to end: one heap arm, one primitive
module, one bootstrap class, one `core.ph` protocol block, census + invariants rows. No
compiler, VM-loop, or `Value` changes. Estimated diff: ~2 new Rust files, ~7 edited files,
one `core.ph` block, two test files.

## 2. File-by-file

### 2.1 `phalcom-core/src/heap/bytes.rs` — new

```rust
/// A fixed-length, mutable native octet buffer (PDR-0011 ruling 1).
pub struct BytesObject {
    /// The octets. `Box<[u8]>`, NOT `Vec<u8>`: length is fixed at
    /// construction (spec law 3) — a realloc would strand secret copies
    /// beyond `zeroize`'s reach (spec §7).
    data: Box<[u8]>,
}
```

Methods (all trivial, all documented): `new_zeroed(len: usize)`, `from_vec(Vec<u8>)`
(one construction-time move, no copy), `len()`, `get(usize) -> Option<u8>`,
`set(usize, u8) -> bool` (false = out of range), `as_slice()`, `as_mut_slice()`.

No `Value` inside ⇒ nothing for the GC to trace and no interior mutability concerns.
`size_of::<BytesObject>()` is 16 (thin ptr + len) — well under the 40 B `Object` slot.

### 2.2 `phalcom-core/src/heap/object.rs` — arm

Add **unboxed**, next to `Tuple` (the same immutable-length family):

```rust
/// A fixed-length native octet buffer ([`BytesObject`], PDR-0011).
/// Contents mutable, length fixed — `Tuple`'s backing shape with `List`'s
/// mutability corner (bytes.md §1). Reached through `Value::Obj`; no
/// `Value::Bytes` arm (ADR-0010 minimalism).
Bytes(BytesObject),
```

Rationale for unboxed is the existing boxing note at `object.rs:29-33`: only payloads
fatter than the slot box.

### 2.3 `phalcom-core/src/heap/trace.rs` — explicit no-op arm

`Object::List` traces its `Value` children at `trace.rs:157`; `Bytes` holds none. Add an
**explicit** `Object::Bytes(_) => {}` arm with a doc comment saying why — never fold into a
`_` wildcard (the match must stay exhaustive so the next arm's author is forced to decide).

### 2.4 `phalcom-core/src/heap/mod.rs` + `accessors.rs`

- `mod.rs`: an `insert`-style constructor (pattern: the `Object::List` insertion at
  `mod.rs:157`) and a `"Bytes"` row in the class-name match at `mod.rs:231`.
- `accessors.rs`: `bytes(ObjRef) -> &BytesObject` / `bytes_mut(ObjRef) -> &mut BytesObject`,
  copying the `.fiber()`/`.fiber_mut()` pair (`accessors.rs:318`/`:331`) — same
  panic-on-wrong-arm contract, same rustdoc shape.
- **GC/drop note:** the sweep runs Rust drop glue (`heap/mod.rs:322`); dropping a
  `Box<[u8]>` frees memory and nothing else. No OS handle lives here, so PDR-0005 §4's
  back-door-finalizer hazard does not apply to this arm. Say so in the arm's rustdoc.

### 2.5 `phalcom-core/src/universe/core_classes.rs` — bootstrap

```rust
let bytes_class = make_core_class(heap, "Bytes", iterable_class, metaclass_class);
```

Placed **after `range_class`** (`core_classes.rs:125` region): needs `iterable_class`
(created at `:105`) as superclass, and `utf8_`/`fromString_` need `string_class`, created
earlier in the ADR-0020 load order (`Bool, Option, Number, Symbol, String → List → …`).
Add the `bytes_class` field to `CoreClasses` (documented), and the verify row in
`universe/invariants.rs` (pattern at `:96`) — an unwired bootstrap edge is a hard boot
failure with no user frame to blame (ADR-0019 Context), and the verify row is what turns
it into a named panic.

### 2.6 `phalcom-core/src/primitive/bytes.rs` — new, the ten primitives

Conventions copied from `primitive/list.rs:72-103` verbatim: an `expect_bytes` helper
(pattern `expect_list`), `expect_index` reuse, reads return bare-value-or-`vm.none_value()`,
bad writes return `RuntimeError::Type`, unit-returning ops return `vm.none_value()`.
A new `expect_octet(&Value) -> Result<u8, _>` helper enforces integer-in-0..=255 (spec §2):
accept `Value::Number(n)` iff `n.fract() == 0.0 && (0.0..=255.0).contains(&n)`.

| Rust fn | Signature bound | Validation | Behavior |
|---|---|---|---|
| `bytes_class_new` | `Bytes.new(_)` static | non-negative integer `Number` | `new_zeroed(n)` |
| `bytes_class_from_string` | `Bytes.fromString_(_)` static | arg is `Str` | `from_vec(s.as_bytes().to_vec())` |
| `bytes_raw_size` | `size_` | — | `len` as `Number` |
| `bytes_raw_at` | `at_(_)` | index a non-negative integer | `get` → octet as `Number`, OOB → `none_value` |
| `bytes_raw_set` | `set_(_,_)` | index in range, value an octet | write, return `none_value`; OOB/non-octet → `Type` error |
| `bytes_raw_fill` | `fill_(_)` | value an octet | `as_mut_slice().fill(b)` (memset) |
| `bytes_raw_slice` | `slice_(_,_)` | `0 <= start <= end <= len` | copy `[start, end)` into a fresh arm |
| `bytes_raw_copy_into` | `copyInto_(_,_)` | arg 0 a `Bytes`, `offset + self.len <= dst.len` | `copy_from_slice` at offset (memmove); self-copy safe via `copy_within` when `dst == self` |
| `bytes_raw_utf8` | `utf8_` | — | `str::from_utf8` → new `Str` on ok, `none_value` on err. Never panics, never truncates (law 7) |
| `bytes_raw_equals_ct` | `equalsConstantTime_(_)` | arg a `Bytes`, else `Type` error (spec §8.3) | see below |

**`equalsConstantTime_` implementation note.** Length mismatch → `false` immediately
(lengths are public, spec §8.2). Equal lengths: XOR-accumulate the whole buffer with **no
early exit** —

```rust
let mut acc: u8 = 0;
for i in 0..a.len() { acc |= a[i] ^ b[i]; }
std::hint::black_box(acc) == 0
```

`black_box` keeps the optimizer from rewriting the fold into a short-circuiting `memcmp`.
No new dependency (the `subtle` crate is the alternative; not worth a workspace dep for one
fold). The property must hold under `--release`; §5's timing check is the gate.

**`copyInto_` aliasing.** `heap.bytes_mut` on both receiver and argument is a double
mutable borrow when they alias. Branch on `ObjRef` equality first: same ref →
`copy_within` on the single buffer (memmove semantics, spec law 6); distinct refs → split
borrows. Do not "fix" this with an intermediate allocation on the distinct-ref path.

### 2.7 `phalcom-core/src/universe/primitives.rs` — bindings

The `list_cls` block at `primitives.rs:292-295` is the template:

```rust
let bytes_cls = vm.universe.classes.bytes_class;
primitive_static!(vm, bytes_cls, "new", SignatureKind::Method(1), bytes_class_new);
primitive_static!(vm, bytes_cls, "fromString_", SignatureKind::Method(1), bytes_class_from_string);
primitive!(vm, bytes_cls, "size_", SignatureKind::Getter, bytes_raw_size);
primitive!(vm, bytes_cls, "at_", SignatureKind::Method(1), bytes_raw_at);
// … set_ / fill_ / slice_ / copyInto_ / utf8_ / equalsConstantTime_
```

(Exact `SignatureKind` per selector follows ADR-0012's arity encoding; `size_`/`utf8_` are
getters iff `List`'s `length_` is — copy whatever `list_raw_length`'s binding uses.)

### 2.8 `phalcom-core/core/core.ph` — the protocol block

Placed with the collection classes. Sketch (final code follows `List`'s style in place):

```phalcom
class Bytes {
  size => self.size_
  at(i) => self.at_(i)                       // bare-or-None passthrough (core.ph:781 shape)
  iteratorValue(cursor) => self.at_(cursor)  // core.ph:1011 shape
  set(i, v) { /* range+octet check, raise on violation, else set_; return self */ }
  fill(v) { /* octet check, raise on violation, else fill_; return self */ }
  zeroize { self.fill_(0) return self }
  utf8 => self.utf8_
  slice(start, end) { /* validate, raise on violation, else slice_ */ }
  copyInto(dst, offset) { /* validate, raise on violation, else copyInto_; return self */ }
  concat(other) {
    const out = Bytes.new(self.size + other.size)
    self.copyInto_(out, 0)
    other.copyInto_(out, self.size)
    return out
  }
  ==(other) { /* List#== shape: isA guard, size, pairwise loop (collection-protocol §4) */ }
  !=(other) => (self == other).not
  toString => /* "Bytes(" + size + ")" */
  toList { /* each → push */ }
  toTuple => Tuple.fromList(self.toList)
}
class Bytes.static {
  fromString(s) => Bytes.fromString_(s)
  fromList(list) { /* validate octets (raise), Bytes.new(list.size), set_ loop */ }
}
```

Raises use the ADR-0037/0038 surface (`Error` + `raise`) — do **not** invent structure
from PDR-0010; it is unratified.

`each`/`map`/`filter`/`reduce`: **absent** — inherited from `Iterable`. Adding native or
even local `.ph` overrides is a spec violation (bytes.md §3.1, law 8).

### 2.9 Census + invariants — `phalcom-core/tests/invariants.rs`

- `const NEW_BYTES: usize = 10;` with a comment in the established running-total style
  (`137 -> 147`), added to the sum in `floor_census_matches_installed_bindings`
  (`invariants.rs:605`).
- A `Bytes` row in `core_class_rows` — the CB-5 lesson: a class absent from the census is
  a class the ADR-0019 freeze does not bind.

## 3. Ordering / hazard checklist

1. Arm + heap module first (compiles standalone).
2. Bootstrap class + `CoreClasses` field + `universe/invariants.rs` verify row.
3. Primitives + bindings. Boot the VM: `verify_invariants` green before any `.ph` exists.
4. `core.ph` block. Boot again; `cargo test` invariants lane green (census now 147).
5. Harness + golden tests (§5).
6. `graphify update . --no-cluster` after the dust settles.

Known hazards, each named in the spec: bootstrap edge (spec §1 / ADR-0020's
primitive-boundary ⊗ bootstrap-order hazard — `utf8_` needs `string_class` live);
drop-glue is safe here but say so in rustdoc (§2.4); the `copyInto_` alias borrow (§2.6).

## 4. What must NOT happen

- No `Vec<u8>` backing, no grow/shrink primitive (law 3 — supersedes-PDR-0011 territory).
- No native `each`/combinator, no `.ph` override of them (§3.1's hard-forbid; law 8's
  harness row is the tripwire).
- No `Value::Bytes` arm, no `Byte` value type (ruling 2).
- No `Result` built in Rust (natives return bare/`None`; `.ph` lifts).
- No `wrap_some` on `at_`'s hot path — the bare-or-`None` convention is `List`'s, and an
  octet is never `None`, so the union is unambiguous (bytes.md §3).

## 5. Test plan

Golden lanes per the established split (positive stdout-exact, negative subdir):

| Lane | Content |
|---|---|
| positive | one fixture per harness row of bytes.md §9 that prints values (`at` totality, roundtrip, `fill`, `slice`/`copyInto`/`concat` aliasing, decode, `fromString` roundtrip, `fromList`/`toList`, `toTuple`-keys-a-`Map`, structural `==`, identity `hash` pair, iteration order) |
| negative | every raise: `set(256)`, `set(-1)`, `set(1.5)`, OOB `set`, non-octet `fill`, bad `slice` range, overflowing `copyInto`, non-`Bytes` `equalsConstantTime`, non-octet `fromList`, negative `new` |
| fiber | **yield-mid-iteration** (law 8): a fiber `Fiber.yield`s inside `bytes.each { … }`, is resumed, and completes — the §3.1 boundary's tripwire |
| invariants | census 147; `Bytes` class row; `verify_invariants` boot check |
| timing (advisory) | `equalsConstantTime_` on equal-length inputs differing at index 0 vs index n-1: wall-time ratio within noise under `--release`. Advisory lane, not CI-gating — timing on a shared box is unreliable (perf-log discipline); the load-bearing assertion is the no-early-exit shape reviewed in code |

Worktree-verify the batch before commit (clean-checkout gate), and commit per green
checkpoint, never one end-of-unit batch.

## 6. Not in this unit

`BytesReader`/`BytesWriter` (stream-protocol §8 — separate unit, needs this one),
any literal syntax (BY-1, no owner), any encoding beyond UTF-8.

## 7. (as-built) Deviations and findings — obligations for every future impl spec

Three things the plan missed, found during implementation; each is now a checklist item
any new impl spec must address explicitly:

1. **A new kernel class MUST join `install_core`'s `add_class!` list**
   (`vm/bootstrap.rs`, next to `add_class!(range_class)`). Without the row, the core.ph
   `class X` block mints a **fresh** class that shadows the bootstrapped row carrying the
   native bindings — symptom: `<class X> does not understand 'new(_)'` at first use.
   §2.5's "bootstrap class + verify row" was necessary but not sufficient.
2. **`is_mutable_collection_key` (`primitive/mod.rs`) is a match site the file-by-file
   sweep missed.** Any new *mutable* container arm must join its rejection set or it
   silently becomes a corrupting `Map`/`Set` key. Found by compiler-driven exhaustive
   matching — which is also the method: grep every `Object::List` match site and decide
   each one for the new arm; `if let`/`matches!` sites do not error on omission.
3. **Law 8 was made true by VM surgery, not documentation** (`5ba6101`,
   `vm/send.rs::call_method`): an ordinary `f.call(...)` from bytecode on a
   `Block`/`Closure` receiver now enters the closure frame **flat** in the same dispatch
   loop (the stack window is already in `block_call`'s layout), so no native frame exists
   during the block body and `Fiber.yield` inside `each`/`map`/`filter` is legal. The
   restricted-switch guard survives only for genuine native re-entry (`.on(_)` handlers,
   `.ensure(_)` cleanup, `invokeOn`, `BoundMethod#call`, `@invariant`). Fixtures re-aimed
   accordingly (`each_generator_yields.ph`, `concurrency_fiber_yield_through_block_call.ph`);
   guard message reworded. **Standing doc debt:** `concurrency.md` §6's restriction table
   still describes the pre-fork rule.
