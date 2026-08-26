# Spec B.1 — Map Runtime Contract Migration

Status: implementation specification. Requires Spec A.2 at minimum for canonical `Unit`; implementing after A.3 is preferred so Record/Tuple hash-key regressions can be tested together. This phase is otherwise independent of brace-literal grammar and is dispatchable immediately.

## 1. Mission

Migrate the already-native Map implementation from its older sequence-like, raw-`None`, chainable-mutation contract to the ratified default mutable association semantics without replacing the working ordered-hash architecture.

The end state is:

```text
mutable Map
arbitrary valid hashable keys
Phalcom hash + == key identity
first-insertion encounter order
updates retain position
remove deletes position
remove + reinsert appends at end
order-insensitive mapping equality
Map not admissible as a Map/Set key
safe get -> Option
strict [] -> value or KeyError
insert -> Option previous
remove -> Option removed
clear -> Unit
```

Do not touch Map literal construction in this phase except where tests need temporary explicit constructors. Do not implement views here beyond preserving `keyAt_` / `valueAt_` behavior; B.2 replaces copied projections.

## 2. Repository baseline to preserve

HEAD already contains the correct high-level native representation and key semantics:

- `phalcom-core/src/heap/map.rs` defines `MapObject` as a dense insertion-ordered entry vector plus `HashMap<i64, Vec<usize>>` bucket index.
- each entry caches the Phalcom hash bucket used at insertion;
- `phalcom-core/src/primitive/map.rs::locate` computes key identity by re-entering the VM to send Phalcom `hash` and `==`;
- a reentrant-depth lock prevents a key's own `hash` / `==` callback from structurally mutating the collection mid-lookup;
- `Map` is already a distinct `Object::Map` native arm and core class;
- `size_`, `get_`, `put_`, `has_`, `remove_`, `keyAt_`, `valueAt_`, and class-side allocation are already admitted floor bindings.

Keep those architectural wins. B.1 is a semantic correction, not a new container implementation.

Do not replace Phalcom key hashing with Rust `Hash for Value`. Heap objects may define value equality/hash different from ObjRef identity; Tuple and Record keys rely on that.

## 3. Stable encounter order — fix `swap_remove`

### 3.1 Current defect

`MapObject::remove_at` currently calls `entries.swap_remove(slot)`. If the removed entry is not last, the last entry jumps into its slot. Example:

```text
insert A, B, C
remove A
```

currently yields an internal order equivalent to `C, B`, while the ratified result is `B, C`.

This defect also leaks through `keys`, `values`, `entries`, printing, iteration, and future `**Map` expansion.

### 3.2 Required storage behavior

Keep the dense `Vec` + bucket-index architecture and make deletion stable. The first implementation should prefer correctness and simplicity over inventing a tombstone/linked-order scheme.

Recommended algorithm in `MapObject::remove_at(slot)`:

1. validate the reentrancy lock and slot as today;
2. capture the removed entry's `(key, value, bucket)`;
3. remove `slot` from that bucket's candidate vector; delete the bucket row if empty;
4. call `entries.remove(slot)`, not `swap_remove`;
5. every old entry at `slot + 1 .. old_len` has shifted left by one;
6. for each shifted entry, read its cached bucket from the now-current entry and replace that bucket candidate's old slot `new_slot + 1` with `new_slot`;
7. return the removed `(key, value)`.

No re-hashing or Phalcom callback is needed during reindexing because every entry already caches its bucket.

Complexity of removal becomes O(n) in the number of shifted entries. That is acceptable for the initial deterministic default Map. Do not introduce a second order vector, stable IDs, tombstones, compaction epochs, or linked-list fields until profiling demonstrates a need.

### 3.3 Order invariants

Add direct Rust tests around `MapObject` or the closest existing heap-level test seam for:

```text
A B C D
remove B -> A C D
remove A -> C D
insert B -> C D B
update C -> C D B
```

The update must not move C.

## 4. Refactor the raw lookup boundary to preserve stored `None`

### 4.1 Current defect

`map_raw_get` currently returns:

```text
hit  -> raw stored Value
miss -> None singleton
```

That makes these two states observationally identical:

```phalcom
map[key] = None
map missing key
```

The new contract requires:

```text
get(present-None) -> Some(None)
get(missing)      -> None
```

### 4.2 Reuse the existing Option substrate

`../../../../phalcom-core/src/primitive/option.rs` already exposes the internal `wrap_some(vm, value)` helper used by `Some.new` and `WrapSome`. Reuse it. Do not add a new Map-specific Some allocator.

Change the existing `get_` raw binding's semantics to:

```text
Map::get_(key)
    present -> wrap_some(vm, stored_value)
    absent  -> vm.none_value()
```

A stored surface `None` is a normal `Value::Obj(none_singleton)` and is therefore legal to wrap. Only the private `Value::Nil` sentinel is forbidden inside `Some`; Map user values must never expose that sentinel anyway.

No new floor binding is required.

### 4.3 Shared internal lookup helper

The current module-private `locate` is useful beyond raw `get_`: B.3's atomic Map literal builder will need the same duplicate-aware key lookup without dispatching through the public API.

Refactor without changing semantics so the key lookup is one reusable internal function, for example:

```rust
pub(crate) fn locate_key(vm: &mut VM, id: ObjRef, key: Value)
    -> PhResult<(i64, Option<usize>)>
```

The exact name is implementation-local. Preserve the crucial borrow discipline:

- copy `Value` / slot candidates out before calling `send_hash` / `send_eq`;
- never hold `&Heap` or `&mut Heap` across re-entrant language execution;
- pair every `enter_reentrant_send` with `exit_reentrant_send`, including error paths.

Do not duplicate this logic later in bytecode handlers.

## 5. Correct raw mutation results

### 5.1 `put_`

Change the existing raw `put_` semantics from "return receiver" to:

```text
new key      -> None
existing key -> Some(previous_value)
```

Required ordering:

1. reject an inadmissible mutable key before mutation;
2. locate using Phalcom `hash` + `==`;
3. on an existing key, copy the previous value, overwrite only that entry's value, preserve slot and stored key-object policy exactly as HEAD currently does, return `Some(previous)`;
4. on a new key, append at end, return `None`.

The specification deliberately does not settle whether updating with an equal-but-nonidentical key replaces the stored key object. Do not change HEAD's current retention behavior merely to make this phase "cleaner". Assert only that the logical mapping has one slot and the value updates.

### 5.2 `remove_`

Change the existing raw `remove_` semantics to:

```text
present -> Some(removed_value)
absent  -> None
```

It must use the stable removal algorithm from §3.

### 5.3 `has_`, `size_`, `keyAt_`, `valueAt_`

Their semantic role remains unchanged. `keyAt_` and `valueAt_` are raw encounter-order observations for core-library implementation, not public arbitrary-key lookup APIs.

Do not make them negative-index-aware; Spec C owns sequence index normalization and Map keys are not indices.

## 6. Public Map API in `phalcom-core/core/core.ph`

Rewrite the Map facade against the corrected raw boundary.

### 6.1 Safe lookup

```phalcom
get(key) => self.get_(key)
```

returns `Option<V>`.

No alternate sentinel-returning public lookup should remain as the canonical API.

### 6.2 Strict subscript lookup

`map[key]` is strict:

```text
present -> stored value
absent  -> raise KeyError
```

Implement `KeyError` as an ordinary `.ph` subclass of `Error` unless HEAD has gained an equivalent class by dispatch time:

```phalcom
class KeyError is Error {}
```

The exact message/payload schema is deferred. Use a stable useful message without over-designing fields. A simple message identifying a missing Map key is sufficient for B.1.

Implement the bracket method through `get` / `Option.match`, not by calling `has_` and then `get_`, which would hash/compare the key twice.

Conceptually:

```phalcom
[k] {
  return self.get(k).match(
    some: { value => value },
    none: { throw KeyError.new("Map key not found") },
  )
}
```

Adapt to current block/call spelling on HEAD.

### 6.3 Explicit insert

Canonical surface:

```phalcom
map.insert(value, for: key)
// Option<previous>
```

Implement as a thin wrapper over `put_`.

The parameter order is intentionally value first, key labeled `for:`. Do not reverse it to accommodate current internals; raw `put_` may remain `(key, value)` internally.

### 6.4 Keyword label parsing prerequisite: `for:`

HEAD's `parse_arg_list` recognizes a label only when the token before `:` is `Token::Identifier`. `for` is a reserved `Token::For`, so the ratified selector is currently impossible both at call sites and method definitions.

Fix the parser generically at the label slot rather than special-casing Map's method body.

Add a helper that can recover a source label name from:

- `Token::Identifier(name)`;
- reserved keyword tokens when they occur in a syntactically colon-marked label position.

Use it in both:

- argument label parsing (`parse_arg_list`);
- parameter label parsing (`parse_param_list`, and any equivalent labeled method-definition path).

A keyword token without the label colon retains its normal grammar meaning. The change therefore does not make `for` an ordinary identifier or variable name.

At minimum test:

```phalcom
map.insert(1, for: #a)
```

and a small user-defined method with a `for:` parameter, proving definition and call encode the same selector.

Selector identity remains the existing base-name + arity + ordered-label sequence; no dispatch change belongs here.

### 6.5 Subscript assignment storage

The Map setter selector remains:

```phalcom
[k, put:] { ... }
```

It must store through the same `put_` operation, so assignment and explicit `insert` share key identity/order semantics.

Spec C owns the general language rule that `obj[index] = rhs` evaluates to the original RHS regardless of setter return. Do not modify `phalcom-core/src/compiler/lib/expr.rs` in B.1 solely for Map.

Until Spec C lands, preserve repository compatibility where practical, but do not design a second Map mutation result contract around the temporary compiler behavior. Tests in B.1 should validate that storage occurred; the cross-language assignment-expression-value invariant is a Spec C gate.

### 6.6 Remove

Canonical:

```phalcom
map.remove(key)
// Option<V>
```

Return `remove_` directly. Remove the old chainable/idempotent-self return semantics from Map tests and docs.

### 6.7 Clear

Canonical:

```phalcom
map.clear
// Unit
```

No new raw primitive is needed. Implement in `.ph` over the existing floor, for example by repeatedly removing the key at encounter slot zero until size is zero, then returning `()`.

Because removal is now stable, repeatedly deleting slot zero naturally walks original encounter order. The result is canonical Unit from Spec A.

If profiling later justifies a native bulk clear, that is a separate floor decision; do not expand the primitive floor now.

## 7. Equality and Map hashability boundary

### 7.1 Equality

Retain order-insensitive extensional mapping equality:

```text
same size
for each key in left encounter order:
    right contains equal key
    corresponding values are ==
```

Do not compare `keyAt_(i)` against `other.keyAt_(i)`; equal Maps may have different encounter histories.

Update the `.ph` implementation to use safe/strict lookup correctly after `at` is retired or changed. Avoid observing a missing key as a false `None` value.

### 7.2 Map as key

The repository's object model has universal `Object#hash`, so "Map is unhashable" is enforced at keyed-collection admission rather than by deleting a `hash` selector from Map. Preserve that architecture for B.1.

`is_mutable_collection_key` already rejects `List`, `Map`, `Set`, and `Bytes`. Keep that protection and make sure A's positive immutable Tuple and Record remain accepted when their contained values satisfy the normal hash/equality contract.

Do not generalize this phase into a new public `Hashable` capability or type system.

## 8. Retire the obsolete sequence-style Map contract

`phalcom-core/tests/collections_contract.rs` currently builds Map with numeric keys `0..n` and then treats `Map#at(i)` as sequence element access. That contract conflicts with the new design.

Modify the harness so Map no longer instantiates the generic finite-sequence laws. Do not weaken List/Tuple/Set tests simply to keep Map in the same abstraction.

Create or expand Map-specific runtime tests that exercise association semantics directly.

If `at(_)` survives temporarily for compatibility, it must not be the conformance definition of Map. Prefer migrating repository uses toward `get` or `[]` and removing the stale Map-specific sequence comments/tests.

## 9. Repository migration audit

Search the entire repository for:

```text
Map#at / .at(...)
at(_,put) on Map
Map#remove return-value chaining
.keys / .values assumptions
collections_contract build_map
map_raw_get / map_raw_put / map_raw_remove
```

Classify each use:

- strict required -> `map[key]`;
- absence expected -> `map.get(key)`;
- insertion/update -> `insert(value, for: key)` or subscript assignment;
- return value ignored -> migrate mechanically;
- old chainability relied upon -> rewrite explicitly; do not preserve stale semantics solely for examples/benchmarks.

Do not change List's `at` API in this phase.

## 10. Required tests

### 10.1 Encounter order

Test public traversal/raw order after:

1. insert A, B, C;
2. overwrite A;
3. remove B;
4. insert D;
5. remove A;
6. reinsert A.

Required final order:

```text
C, D, A
```

Also specifically test middle removal from four entries to catch any return of `swap_remove`.

### 10.2 Safe lookup and stored `None`

```phalcom
const m = Map.new()
m[#present] = None

m.get(#present)  // Some(None)
m.get(#missing)  // None
```

Use `Option.match`/class checks appropriate to HEAD to assert the two are distinct.

### 10.3 Strict lookup

- existing key through `m[key]` returns value;
- missing key raises catchable `KeyError`;
- a stored `None` is returned as `None`, not mistaken for absence/KeyError.

### 10.4 Insert/remove results

```text
insert new      -> None
insert existing -> Some(previous)
remove present  -> Some(value)
remove absent   -> None
clear           -> Unit and size 0
```

Overwriting must keep size and order unchanged.

### 10.5 Key domain

Exercise at least:

- Symbol key;
- String key distinct from equal-looking Symbol spelling;
- Number key;
- positive hashable Tuple key;
- positive hashable Record key after A.3;
- negative integer key treated as ordinary key, not index normalization.

Reject mutable List/Map/Set/Bytes keys through the existing admission rule.

### 10.6 Equality

Build equal Maps in opposite insertion orders and assert `==` true while encounter traversal differs. Change one corresponding value and assert false.

### 10.7 Reentrancy regression

Preserve existing tests around keys whose `hash` / `==` call back into the same Map. Stable removal and Option wrapping must not weaken the reentrant mutation lock or leave it permanently incremented after an error.

### 10.8 Keyword label

Parser/compiler/runtime test for a `for:` label at both declaration and call sites. Also test that ordinary statement-leading `for` loops still parse unchanged.

## 11. Files expected to change

Re-read HEAD; likely write set:

```text
phalcom-core/src/heap/map.rs
phalcom-core/src/primitive/map.rs
phalcom-core/src/primitive/mod.rs        # only if shared helper visibility/imports change
phalcom-core/core/core.ph
phalcom-ast/src/parser.rs                # `for:` / keyword label support
phalcom-core/tests/collections_contract.rs
phalcom-core/tests/... Map-specific Rust tests
phalcom-core/tests/lang/collections/... goldens/negatives
phalcom-ast/tests/... parser snapshots/tests for keyword labels
relevant docs/status files required by repository process
```

Changing the semantics of already-admitted raw Map bindings does not itself require new primitive-floor entries. If implementation unexpectedly needs another native binding, stop and justify it through the current floor-governance process rather than slipping it into this phase.

## 12. Explicit non-goals

Do not implement:

- Map views/Entry/Record conversion — B.2;
- literal AST/building/duplicate literal checks — B.3;
- `**` expansion — F;
- general subscript assignment semantics — C;
- general `toMap` conversions/grouping — D;
- mutation-during-iteration policy;
- equal-but-nonidentical key retention policy;
- Map typing/generics/reflection;
- full Set changes beyond keeping shared backing behavior intact.

## 13. Completion gate

Before completion:

```sh
./scripts/verify.sh --full
```

The implementation report must include:

- exact stable-removal algorithm;
- raw `get_` / `put_` / `remove_` before/after semantics;
- proof/example that stored `None` yields `Some(None)`;
- final Map public selector list changed in this phase;
- keyword-label parser change and selector-encoding parity test;
- disposition of the old Map sequence contract;
- repository call-site migration summary;
- primitive-floor delta, expected `0`;
- final verification tail.
