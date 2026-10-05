---
id: LANG007.C4.P3-handoff
kind: handoff
plan: LANG007.C4.P3
checkpoint: LANG007.C4
created: 2026-10-05
for: next implementer (taking over for LANG007.C4.P4)
---

# LANG007.C4.P3 — Handoff (P3 Complete; P4 Next)

This handoff records established work through complete closure of `LANG007.C4.P3` ("First-class Pack and Call/Rest Convergence").
All tasks (T0–T9) and verification gates (G1–G4) are IMPLEMENTED and FOCUSED_TESTED.
The next plan is `LANG007.C4.P4` ("Generalized `FieldKey` and unordered-product calculus").

**Current plan status**: COMPLETE / IMPLEMENTED / FOCUSED_TESTED

---

## 1. Current revision and state

- **Branch**: `main`
- **Revision**: baseline `5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521` with completed P3 commits.
- **Owning Checkpoint**: `docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md`
- **Active Plan for C4**: `LANG007.C4.P4`

---

## 2. Hard Architectural Boundary for P4 Takeover

The hard boundary between P3 and P4 must be strictly preserved:

```text
Pack / Tuple / call labels -> Symbol
Record unordered keys      -> P4 FieldKey
```

**CRITICAL RULES FOR P4 IMPLEMENTATION:**
1. **Do not generalize Pack labels, call labels, rest labeled lanes, selectors, or ordered-product labels to P4's heterogeneous `FieldKey`.**
2. Pack remains an ordered structural transport with Symbol labels in its labeled lane.
3. Call argument labels, method rest parameters, and dynamic selectors remain Symbol-based.
4. `ProductShape::Pack` and `ProductShape::Tuple` remain ordered products whose labels are `Symbol`.
5. P4 owns generalized `FieldKey` exclusively for unordered Record fields and Record row calculus.

---

## 3. Established Durable Invariants (Do Not Redesign)

### 3.1 Pack Type & Intrinsic Aliases
- `TypeData::Pack` in `phalcom-semantic/src/types/store.rs`:
  - Contains positional element types and Symbol-labeled element types.
  - Supports `Closed`, `Typed(TypeId)`, and `Unconstrained` remainder states per lane.
- Intrinsic aliases in `core/universe/src/collections/pack.ph`:
  - `PositionalPack<T> = Pack<*: T>`
  - `LabeledPack<T> = Pack<**: T>`
  - `CompletePack<T> = Pack<***: T>`
  - C3 kinded domains `TypePack`, `LabeledTypePack`, and `CompleteTypePack` are preserved as kinded domains; do not redefine proper type aliases under those names.

### 3.2 Static Pack Values & Invalidation
- `StaticValueData::Pack` in `phalcom-semantic/src/static_value/store.rs`:
  - Stores positional and labeled static value elements.
  - Has structural equality, hashing, and stable fingerprinting.
- Fingerprint Invalidation (`phalcom-semantic/src/db/fingerprint.rs`):
  - `StaticParameter.mode` is explicitly hashed into `static_definition_product_fingerprint`. Any change to a static parameter's variadic mode invalidates downstream definitions and applications.

### 3.3 Runtime Pack Storage & Slicing
- Immediate Value:
  - `ValueTag::EmptyPack` (tag 12) encodes `Pack()` without heap allocation.
- Heap Objects:
  - `Object::PackRoot`: immutable root backing for non-empty Packs.
  - `Object::PackSlice`: immutable sub-slice view pointing directly to the root backing.
- Rest Slicing Invariant (`phalcom-core/src/primitive/pack.rs`):
  - `pack_raw_positional_rest_slice` slices the positional lane and preserves `labeled_len`.
  - `pack_raw_labeled_rest_slice` slices the labeled lane and preserves `positional_len`.
  - Slicing one lane MUST NOT drop or zero out elements of the opposing lane.

### 3.4 Bytecode & Fast Paths
- Opcode 112 (`FinishPack`): finalizes argument pack builder into immutable `Pack` (or `EmptyPack`).
- Opcode 113 (`BuildStaticPack`): direct Pack construction from stack values matching static product shape.
- Opcode 114 (`BuildPack`): direct standalone Pack construction from positional values and (Symbol, value) pairs.
- Fixed-call zero-pack invariant: ordinary static method/function calls without spread emit direct invoke bytecodes and emit zero Pack opcodes.

### 3.5 Rest Capture & Forwarding Convergence
- Method rest (`*args`, `**kwargs`, split rest, `***rest`), block rest, and Fiber rest parameters all capture into first-class `Pack`.
- Forwarding accesses Pack lanes directly without allocating intermediate Tuples or Lists.

### 3.6 One-Step Views
- `Pack.telos` decomposes positional and labeled lanes into `PackLaneEmpty` and `PackEntry(value, rest)` / `PackEntry(key, value, rest)` views in O(1) over shared root backing.

---

## 4. Verification Baseline Inherited by P4

All verification gates G1–G4 passed:

- **G1 (Semantic/Static vertical)**:
  ```sh
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast pack_type_remainder -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta pack -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::pack_types -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_pack -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_definitions -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::pack_types -- --nocapture
  ```
- **G2 (Runtime/Rest/Spread vertical)**:
  ```sh
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_pack -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core memory_pack_gc -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_outgoing_packs_completion::p3_ -- --nocapture
  ```
- **G3 (Fixed-call / Resource invariant)**:
  ```sh
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_calls_and_static_tuple_emit_no_pack_machinery -- --nocapture
  ```
- **G4 (Focused compatibility/stabilization)**:
  ```sh
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast -p phalcom-type-meta -p phalcom-semantic -p phalcom-core
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic type_alias -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic adts -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_structural -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_constants -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::algebraic_data -- --nocapture
  RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core execution_static_const -- --nocapture
  ```

---

## 5. Next Actions for P4

1. Take over active plan `LANG007.C4.P4` in accordance with `docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4-luna-patch-grade-implementation-plan.md`.
2. Implement heterogeneous `FieldKey` (Symbol, String, Integer, etc.) for unordered records.
3. Keep Pack and call lane labels as `Symbol`.
