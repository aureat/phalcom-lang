# LANG005.C5.P3 to LANG005.C6 Handoff

## 1. Takeover summary

`LANG005.C5` is closed as `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

Associated types are fully integrated and certified across the compiler, VM runtime, metadata, incremental engine, and LSP layers for declared and exact conformance evidence.

The next checkpoint is `LANG005.C6` (Generic Conformance Assumptions, Evidence Systems, and Production Iteration Migration).

---

## 2. Established C5 architecture and frozen interfaces

C6 inherits these canonical products and invariants without rediscovery:

1. **Associated Requirement Identity**:
   - `AssociatedTypeRequirementId` is trait-owned and stable (`DeclarationId` + index).
   - Names and source ranges are not identity.
   - `TraitSurface.associated_types` maps `AssociatedTypeRequirementId` to `TraitAssociatedTypeRequirement`.

2. **Source Conformance Binding Plans**:
   - `ConformanceAssociatedTypePlan` owned by `ImplId`.
   - Binds source expressions under implementation type environments.
   - Preserves cyclic/recursive bindings deterministically.

3. **Exact Evidence**:
   - `ConformanceEvidence.associated_types` maps `AssociatedTypeRequirementId` to `ExactAssociatedTypeBinding`.
   - `ConformanceEvidence.requirement_views` materializes normalized behavioral signatures.
   - Singular `ConformanceCompleteness` unifies behavioral and associated proof state.

4. **Canonical Projection & Normalization**:
   - `TypeData::AssociatedProjection` carries subject, `TraitRef`, and `AssociatedTypeRequirementId`.
   - `types::normalize_type` is the sole recursive normalization and cycle detection authority across abstract, source, and exact modes.
   - Terminal states (unknown, blocked, ambiguous, recursive, dynamic boundary) remain distinct.

5. **Compiler and Runtime Anti-Authority**:
   - Lowering translates normalized semantic types into ordinary `RuntimeTypeRecipe` / `RuntimeTypeEnvironment` values.
   - The VM executes selected methods and never solves projections or conducts conformance proof search at runtime.

6. **Tooling & Incremental Integration**:
   - Source index and LSP consume `SemanticTargetId::AssociatedType(AssociatedTypeRequirementId)`.
   - Module mutations recompute exact evidence and invalidate consumers cleanly with full cold/incremental parity.

7. **Metadata Boundary (Branch A)**:
   - Raw symbolic `AssociatedProjection` forms remain rejected as `MetadataExportError::NonExportableForm`.
   - Compiler/runtime consumers normalize before export.

---

## 3. Concrete iterator reference model (Ratified for C6)

C5 established the executable concrete reference model in `phalcom-core/tests/core/language/traits_p3_closure.rs`:

```phalcom
trait Iterator {
  type Item
  next -> Option<Self::Item>
  iterator { self }
}

trait Iterable {
  type Item
  type Iterator
  iterator -> Self::Iterator
}
```

Verified behavioral laws:
- `iterator` and `next` are getter-shaped (no nullary parentheses `()`).
- Reusable collection `Iterable.iterator` creates a fresh, independent traversal instance.
- `Iterator.iterator` returns `self` (identity preservation `===`).
- `next` returns `Option<Item>` directly (`Some(val)` or `None`).
- Stable exhaustion: once `next` returns `None`, all subsequent calls return `None`.
- Errors propagate normally via `throw`/`raise`; `None` represents only clean exhaustion.
- No public `Cursor`, `hasNext`, `reset`, or iterator-specific mutability markers.
- Map target `Item` is a labeled tuple `(key: K, value: V)`.

---

## 4. Scope and obligations owned by C6

C6 must implement:

### A. Generic Conformance & Evidence Syntax

1. Conformance relation syntax: `T |= Trait` in generic `where` clauses.
2. Conjunction sugar: `T |= A + B` elaborating to independent conformance constraints.
3. Associated refinement/equality: `T |= Iterator<Item = A>` and/or `T::Item = A` in `where` clauses.
4. **No** `T: Trait` binder syntax.

### B. Generic Conformance Evidence System

1. Express `Iterable` obligations:
   - `Self::Iterator |= Iterator`
   - `Self::Iterator::Item = Self::Item`
2. Express the general theorem `Iterator ⇒ Iterable`:
   - `Iterable.Item = Iterator.Item`
   - `Iterable.Iterator = Self`
   - `iterator = self`
3. Generic-body trait dispatch through assumed evidence environments.

### C. Production Iteration Migration Sequence

1. Add generic/nested evidence and associated equality authority.
2. Promote `Iterator` and `Iterable` traits to canonical Universe declarations.
3. Migrate core collection concrete conformances (`List<T>`, `Set<T>`, `Map<K, V>`, `Tuple`, `Bytes`, `Range`).
4. Update collection `iterator` getters to create fresh stateful iterators.
5. Update `Iterator.iterator` to return `self`.
6. Migrate semantic `for` loop checker to evidence-driven `Item` typing.
7. Migrate compiler `for` lowering in `phalcom-core/src/compiler/lib/loops.rs` to stateful `iterator` + `next` + `match` pattern semantics.
8. Migrate `Map` direct iteration from keys to labeled entry tuples `(key: K, value: V)`.
9. Migrate `Range` to stateful `RangeIterator`.
10. Migrate lazy adapters (`filter`, `map`, `take`) to stateful iterator composition.
11. Retire runtime `Iterable` superclass from bootstrap.
12. Retire `iterate(_)` and `iteratorValue(_)` cursor protocol.
13. Update `../../../__unstructured__/spec/current/iteration.md` to normative stateful iterator specification once migration is active.

---

## 5. Inherited baseline ledger for C6

- **C5-BL-01 / C5-BL-04**: Legacy `Option.match` generic call-entry recipe instantiation error inside `range.ph`/`iterable.ph`. Deferred to C6 / runtime call-entry maintenance.
- **C5-BL-07**: Two Universe `Bool` dependency/capability failures in `capabilities::traits`. Baseline D.
