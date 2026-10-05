---
id: LANG007.C4.P3-walkthrough
kind: walkthrough
plan: LANG007.C4.P3
checkpoint: LANG007.C4
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
created: 2026-10-05
---

# LANG007.C4.P3 — Walkthrough

This walkthrough records what actually shipped for `LANG007.C4.P3` ("First-class Pack and Call/Rest Convergence").
All tasks (T0–T9) and verification gates (G1–G4) are implemented, verified, and certified under focused testing discipline.

---

## 1. Plan identity and state at completion

- **Plan**: `LANG007.C4.P3` — First-class Pack and Call/Rest Convergence
- **Owning Checkpoint**: `LANG007.C4` — Structural Static Upgrade
- **Status**: COMPLETE
- **Completion**: IMPLEMENTED (T0–T9 fully implemented)
- **Verification**: FOCUSED_TESTED (Gates G1–G4 passed; all P3-owned behavior verified)

---

## 2. Revision & Scope

- **Branch**: `main`
- **Baseline**: `5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521`
- **Scope**: First-class Pack type, StaticPack values, runtime Pack primitives/objects, bytecode generation, call-site rest convergence, spreading, one-step views, and verification.
- **Concurrent Boundary**: P3 is the authority for first-class Pack and Symbol-labeled lanes. P4 begins concurrently in a separate worktree and owns heterogeneous `FieldKey` for unordered records.

---

## 3. Architecture actually landed

### T0 — Takeover and baseline discriminators
- Validated baseline against P2 completion.
- Re-censused fixed-call fast paths; verified `static_calls_and_static_tuple_emit_no_pack_machinery` exists and passes.

### T1 — Core Semantic Pack Type
- Implemented `TypeData::Pack` in `phalcom-semantic`:
  - Ordered positional elements and Symbol-labeled elements.
  - Independent remainder states for positional and labeled lanes: `Closed`, `Typed(TypeId)`, and `Unconstrained`.
  - Type relation and subtyping for Pack: covariance over elements, remainder constraint satisfaction, lane isolation.
  - Formation from syntax: `Pack<*: T>`, `Pack<**: T>`, `Pack<***: T>`, `Pack<*: ...>`, `Pack<**: ...>`, `Pack<***: ...>`.
- Intrinsic aliases in `core/universe/src/collections/pack.ph`:
  - `PositionalPack<T>`, `LabeledPack<T>`, `CompletePack<T>`.
  - Maintained C3 `TypePack`, `LabeledTypePack`, and `CompleteTypePack` as kinded domains (resolved under `INC-P3-2`).

### T2 — StaticPack Value Domain & Parameters
- Implemented `StaticValueData::Pack`:
  - Stored positional values (`Vec<StaticValueId>`) and labeled values (`Vec<(SymbolId, StaticValueId)>`).
  - Hashing, structural equality, and fingerprinting for static Packs.
  - Variadic parameter modes: positional Pack parameter, labeled Pack parameter, complete Pack parameter.
  - Fingerprint invalidation: `StaticParameter.mode` is explicitly hashed in `static_definition_product_fingerprint` in `phalcom-semantic/src/db/fingerprint.rs`.

### T3 — Runtime Pack Primitive & Object
- Immediate representation:
  - Added `ValueTag::EmptyPack` (tag 12) in `phalcom-core/src/value/repr.rs` for `Pack()`. Zero allocation.
- Heap representations:
  - `Object::PackRoot`: immutable root storage for positional values and labeled (Symbol, value) pairs.
  - `Object::PackSlice`: sub-slice referencing the original Pack root directly, with composed positional and labeled offsets/lengths; slice-of-slice chains are normalized away.
- GC tracing: `ObjectTracer` traverses Pack root and Pack slice handles.
- Slicing semantics (`phalcom-core/src/primitive/pack.rs`):
  - `pack_raw_positional_rest_slice`: slices positional lane while preserving visible labeled lane length (`lab_len`).
  - `pack_raw_labeled_rest_slice`: slices labeled lane while preserving visible positional lane length (`pos_len`).

### T4 — Bytecode, Compilation & Builders
- Bytecode additions:
  - `FinishPack` (opcode 112): finalizes an `ArgumentPackBuilderObject` into an immutable Pack (or `EmptyPack` if empty).
  - `BuildStaticPack` (opcode 113): constructs a Pack directly from stack values matching static product shape.
  - `BuildPack` (opcode 114): constructs a standalone Pack from stack values without dynamic builder overhead.
- Builder isolation:
  - `ArgumentPackBuilderObject` remains internal to the runtime. Surface Telos code never observes the builder.

### T5 — Call-Site Rest Arguments & Forwarding
- Method rest parameters: positional rest (`*args`), labeled rest (`**kwargs`), split rest, and complete rest (`***rest`) capture into first-class `Pack`.
- Block rest arguments and Fiber closure rest parameters converge on first-class `Pack`.
- Forwarding reads Pack lanes directly without allocating intermediate Tuples or Lists.

### T6 — Structural Calling Fast Paths & Multi-Spread Fusion
- Fixed-call fast path: ordinary static calls emit direct `Invoke` bytecodes and allocate zero Pack builders or Packs.
- Multi-spread fusion: multiple spreads in construction or call sites fuse into a single destination builder.
- Duplicate detection: duplicate label validation precedes evaluation of subsequent explicit arguments.

### T7 — Telos Standard Surface & Views
- `Pack.telos` surface:
  - Decomposes positional and labeled lanes into `PackLaneEmpty` and `PackEntry(value, rest)` / `PackEntry(key, value, rest)` views.
  - Repeated and nested rest slicing operates in O(1) over shared root backing.
  - Structural equality `==` and hashing operate over visible lane elements.
  - Canonical rendering displays elements in lane order: `Pack(1, 2, right: 3, left: 4)`.

### T8 — Verification, Hardening & Specifications
- T8.1: Audited exhaustive visitors and enum match arms across all crates.
- T8.2: Added incremental test suite in `phalcom-semantic/tests/semantic/incremental/pack_types.rs` (4/4 tests pass).
- T8.3: Validated metadata export round-trip with schema v4 scoped Pack nodes.
- T8.4: Verified compatibility with C1–C3 and P2 suites.
- T8.5: Gathered structural proof for fixed-call no-pack invariant.
- T8.6: Updated specifications:
  - `docs/specs/vm/bytecodes.md`: documented opcodes 112, 113, 114.
  - `docs/specs/vm/values-and-objects.md`: documented tags 11 (`UninitializedBinding`) and 12 (`EmptyPack`), immediate representation, and class mapping.

### T9 — Lifecycle Closure
- Updated `LANG007.C4-CHECKPOINT.md` and `LANG007.C4.P3-luna-patch-grade-implementation-plan.md`.
- Generated final walkthrough and handoff records.

---

## 4. Verification Evidence (Gates G1–G4)

All gates were executed serially:

| Gate | Target / Test Suite | Executed Command | Result |
|---|---|---|---|
| **G1** | AST remainder syntax | `cargo test -p phalcom-ast pack_type_remainder` | 1 passed |
| **G1** | Type-meta Pack roundtrip & schema v4 | `cargo test -p phalcom-type-meta pack` | 3 passed |
| **G1** | Semantic Pack type foundations | `cargo test -p phalcom-semantic --test semantic foundations::pack_types` | 4 passed |
| **G1** | Semantic static Pack storage | `cargo test -p phalcom-semantic --test semantic foundations::static_values_pack` | 1 passed |
| **G1** | Static definitions & parameter modes | `cargo test -p phalcom-semantic --test semantic foundations::static_definitions` | 6 passed |
| **G1** | Incremental Pack invalidation & warm/cold | `cargo test -p phalcom-semantic --test semantic incremental::pack_types` | 4 passed |
| **G2** | Runtime Pack root/slice primitives | `cargo test -p phalcom-core --test core core_pack` | 5 passed |
| **G2** | Pack GC root/slice tracing | `cargo test -p phalcom-core --test core memory_pack_gc` | 3 passed |
| **G2** | Core Pack & rest integration | `cargo test -p phalcom-core --test core core_outgoing_packs_completion::p3_` | 11 passed |
| **G3** | Fixed-call zero-pack proof | `cargo test -p phalcom-core --test core static_calls_and_static_tuple_emit_no_pack_machinery` | 1 passed |
| **G4** | Affected crates build | `cargo check -p phalcom-ast -p phalcom-type-meta -p phalcom-semantic -p phalcom-core` | Clean (0 errors) |
| **G4** | Type alias compatibility | `cargo test -p phalcom-semantic --test semantic type_alias` | 2 passed |
| **G4** | ADT semantic suite | `cargo test -p phalcom-semantic --test semantic adts` | 318 passed (28 gated) |
| **G4** | Structural static values | `cargo test -p phalcom-semantic --test semantic foundations::static_values_structural` | 1 passed |
| **G4** | Static constants | `cargo test -p phalcom-semantic --test semantic static_constants` | 9 passed |
| **G4** | Core ADT execution | `cargo test -p phalcom-core --test core language::algebraic_data` | 45 passed (19 gated) |
| **G4** | Core static const execution | `cargo test -p phalcom-core --test core execution_static_const` | 9 passed |

---

## 5. Incidents and Corrections

1. **`INC-P3-1` (Typed Pack remainder source-form ownership)**:
   - Added semantic lowering for anonymous typed/unconstrained Pack remainders. Ordinary generic constructors reject remainder arguments; only the intrinsic Pack constructor admits them.
2. **`INC-P3-2` (Pack source-name collision)**:
   - Preserved C3 `TypePack`, `LabeledTypePack`, and `CompleteTypePack` as kinded domains; used `PositionalPack<T>`, `LabeledPack<T>`, and `CompletePack<T>` for proper Pack source aliases.
3. **Pack Rest Slice Lane Preservation Fix**:
   - `pack_raw_positional_rest_slice` and `pack_raw_labeled_rest_slice` previously zeroed the other lane's length when creating slice values. Fixed to preserve `lab_len` and `pos_len` respectively, ensuring slicing one lane does not drop the other.
4. **StaticParameter Mode Fingerprint Invalidation**:
   - Added `p.mode.hash(&mut hasher);` in `static_definition_product_fingerprint` in `phalcom-semantic/src/db/fingerprint.rs`.

---

## 6. Completion Classification

- **Status**: COMPLETE
- **Completion**: IMPLEMENTED
- **Verification**: FOCUSED_TESTED
- **Next Plan**: `LANG007.C4.P4`
