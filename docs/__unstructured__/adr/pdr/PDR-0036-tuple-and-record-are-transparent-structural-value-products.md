# PDR-0036 — Tuple and Record Are Transparent Structural Value Products

- Status: Accepted
- Date: 2026-09-13
- Supersedes: none
- Amends: PDR-0035 (extends value-semantic optimization freedom to anonymous structural products); `docs/spec/collections-next/tuple-record-and-symbols-spec.md`; `docs/spec/typing/phalcom-tuples-records-sets-maps-spec.md`; `docs/implementation/LANG003-language-semantics/C1-language-semantics/language-semantics-spec.md`
- Related: LANG005.C1 (P1, P2, P3); PDR-0035 (`data` is a nominal transparent immutable value product); ADR-0010 (`Value` representation); ADR-0050 (precise tracing); SEMA005 runtime typing metadata and reflection
- Repository evidence revision: `aureat/phalcom-lang@d8b823e857b4054420cc070cda9d6b030f151391`

## Context

PDR-0035 established nominal `data` declarations as transparent immutable value products with semantic value identity rather than allocation identity. This allowed `data` values to be scalar-replaced, nested-flattened, packed with native-width slots, rematerialized on demand, or shared without exposing their backing allocation (`ObjRef`, heap address, or allocation count) to language semantics.

Historically, Phalcom's anonymous `Tuple` and `Record` products were represented as native heap objects (`TupleObject` and `RecordObject`) carrying per-instance `Box<[Value]>` component arrays and per-instance `Box<[Symbol]>` label arrays. Furthermore, language-level sameness `===` for heap objects was originally defined in LANG003 as exact backing object/`ObjRef` identity.

This created an architectural tension:
1. Anonymous structural products (`(...)`, `#{...}`) had the same logical immutability and value nature as `data`, but were permanently bound to heap allocation identity.
2. An optimizer could not legally scalar-replace, rematerialize, or layout-pack a `Tuple` or `Record` if language-level `===` could observe whether two structurally equal products shared the same heap handle.
3. Carrying per-instance label arrays and universal 16-byte `Value` boxes in every tuple and record instance introduced unnecessary memory and GC overhead.

LANG005.C1.P3 resolves this tension by extending the value-product ruling to anonymous structural products.

## Decision

### 1. Tuple and Record are transparent structural value products

`Tuple` and `Record` have semantic value identity, not allocation identity.

They share the same physical representation and optimization freedom as nominal `data`:
- the compiler/runtime may scalar-replace them into virtual leaf locals;
- nested products may be flattened recursively;
- components may be stored in ordinary VM locals or registers;
- primitive-width components may use packed storage;
- immutable materialized backing boxes may be shared or rematerialized on demand;
- observations such as `.class`, exact structural type reification, and `===` may be answered directly without heap materialization when statically or runtime-provably sound;
- Unit `()` / `#{}` remains the sole canonical zero product and has no positive heap allocation.

The backing `ObjRef`, heap address, allocation count, or chosen physical layout is an unobservable implementation detail.

### 2. Semantic product categories remain distinct

Sharing the underlying `ProductLayout` and `ProductStorage` substrate does not collapse language categories:
- `data`: nominal transparent immutable product;
- `Tuple`: structural ordered transparent immutable product (ordered positional + ordered labeled lanes);
- `Record`: structural key-based transparent immutable product (order-independent key set);
- `enum`: nominal closed sum; payload uses product machinery;
- `class`: nominal opaque identity-bearing object abstraction.

Physical layout identity (`ProductLayoutId`) is never semantic type identity.

### 3. Representation-independent language-level `===`

Language-level `===` is exact value sameness, not heap handle identity, for all transparent products (`data`, `Tuple`, `Record`, `Unit`).

The canonical language rules for `===` are:
1. **Unit:** `Unit === Unit` is always true.
2. **Tuple:** Two positive `Tuple` values are `===` iff:
   - they have the identical tuple structural shape (same total arity, same positional/labeled lane boundary, and identical ordered labels in the labeled lane); AND
   - corresponding components are recursively `===`.
   Tuple label ordering is semantic: `(x: 1, y: 2) === (y: 2, x: 1)` is false.
3. **Record:** Two positive `Record` values are `===` iff:
   - they have the same key set (set of Symbol fields); AND
   - corresponding values for each key are recursively `===`.
   Record `===` is independent of encounter/presentation order: `#{ name: "A", age: 24 } === #{ age: 24, name: "A" }` is true.
4. **Data:** Two `data` values are `===` iff:
   - they have the identical exact reified nominal data type (including exact generic and phantom type arguments); AND
   - corresponding components are recursively `===`.
5. **Opaque objects (classes):** Class instances retain identity-based `===` (`ObjRef` identity).

`Value::same_as` remains available as an internal VM helper for low-level representation identity (sentinels, handles), but `Bytecode::Same` and `Object#===` execute the VM-aware language exactness relation.

### 4. Distinct contracts for `==`, `hash`, and `===`

PDR-0036 preserves the existing public contracts of `==` and `hash`:
- `Tuple ==`: structural equality over ordered lanes, recursing via `==`.
- `Tuple hash`: ordered hash agreeing with `Tuple ==`.
- `Record ==`: key/value equality independent of encounter order, recursing via `==`.
- `Record hash`: order-independent hash agreeing with `Record ==`.
- `===`: exact recursive representation-independent sameness.

`===` does not replace `==`, and `==` does not forward to `===`.

### 5. Record encounter order vs canonical storage

Record semantic type identity, record `==`, record `hash`, and record `===` are order-independent.
However, Record encounter/presentation order remains observable for:
- iteration / traversal;
- string rendering / debug printing;
- reflection presentation;
- record expansion (`**record`);
- map/sequence conversions.

The runtime separates:
- **`RecordProductShape`**: captures presentation labels, canonical logical labels, and the `presentation_to_logical` mapping.
- **`ProductLayout`**: captures physical storage slots and GC trace maps in canonical logical coordinate order.

Two records with differing presentation order may share the same `ProductLayout` and exact structural `RuntimeTypeRef` while retaining distinct `ProductShapeId`s.

### 6. Reflection, `.class`, and generic reification

- `.class` on any `Tuple` value evaluates to `Tuple`.
- `.class` on any `Record` value evaluates to `Record`.
- Applied/structural product types do not create per-specialization `ClassObject`s.
- Exact structural types remain runtime type descriptors (`RuntimeTypeRef`), not physical layouts.
- No per-value generic argument arrays are stored. Generic evidence is supplied by compile-time facts, call-site activation environments (`RuntimeTypeEnvironmentId`), or compact materialized descriptor IDs.
- Weak references, finalizers, and raw object addresses must not expose the backing box of a transparent product as identity.

## Consequences

- Anonymous `Tuple` and `Record` instances can be scalar-replaced and eliminated by the compiler just like `data`.
- Materialized tuples and records share the compact `ProductStorage` substrate, eliminating per-instance `Box<[Symbol]>` and `Box<[Value]>` arrays.
- Static literals can use native-width packed scalar slots (e.g. two 64-bit integers stored in two words rather than two 16-byte `Value`s).
- Language exactness `===` is robust against optimization, scalar replacement, and rematerialization.
