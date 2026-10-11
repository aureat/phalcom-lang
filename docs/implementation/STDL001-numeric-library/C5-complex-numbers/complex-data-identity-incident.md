# IMPLEMENTATION INCIDENT: generic data identity depends on runtime type representation

## 1. Identity

- Program: STDL001
- Checkpoint: STDL001.C5
- Plan: STDL001.C5.P1
- Task/Gate: T1, exact generic identity
- Repository: `.worktrees/complex-math`
- Branch: `feat/complex-math`
- Starting/current revision: `ad1db231`, uncommitted implementation

## 2. Escalation trigger

Generic reification and canonical runtime data identity are nonlocal and extend
beyond the approved D4 operator-lowering correction. The implementation cannot
weaken exact data identity or patch Complex-specific equality to pass.

## 3. Required invariant

Data exactness is determined by exact nominal application and recursive
component exactness. Construction through a generic method or direct source
constructor must not change that identity. Preserve distinct component types,
phantom parameters, owner modules, and nested generic applications.

## 4. Expected architecture

Semantic typing owns the exact `Complex<Int>` application. Runtime metadata
and instantiated overlay types are representations of that same type. Data
descriptor interning must give equivalent exact types one canonical identity.

## 5. Observed behavior

Both values contain exact Int components 3 and 4 and share the same canonical
Complex declaration/behavior class, but `Complex.new(3,4) === Complex(3,4)` is
false. The generic factory materializes an Overlay type reference; direct
construction materializes a Base metadata reference. Their data descriptor IDs
are different.

## 6. Minimal reproduction

```phalcom
from universe.numbers import Complex
let z = Complex.new(3, 4)
let direct = Complex(3, 4)
let constructed = direct === z
```

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::complex::complex_int_arithmetic_preserves_exact_components_and_generic_identity -- --exact --nocapture
```

Expected: constructed is true. Observed: false.

Temporary diagnostic output (removed after collection):

```text
z: descriptor 13; owner universe:numbers.complex::Complex;
   exact_type Overlay(RuntimeOverlayTypeId(0)); component_slots [0,1]
direct: descriptor 14; same owner, behavior class, layout and component slots;
   exact_type Base { pool: MetadataPoolId(3), node: TypeNodeId(2) }
```

## 7. Relevant code path

```text
vm/data.rs::get_or_bind_data_descriptor
  -> instantiate_type_recipe (closed metadata or generic overlay)
  -> data.rs::RuntimeDataRegistry::register_with_component_slots
  -> key = (DeclarationId, RuntimeTypeRef)
  -> separate descriptor IDs for Base and Overlay
  -> VM::semantic_same / primitive::object::data_eq compare descriptor IDs
```

## 8. Scoped implementation diff

D4 adds proven operator/getter invocation consumption, conditional method
publication in the existing detached registry, and assumed instance trait Self
specialization to the rigid subject. The numerical library currently contains
generic data and basic capability arithmetic. No runtime type/data identity
change or consumer equality workaround exists.

## 9. Evidence

- D4's semantic rigid-subject discriminator passes.
- D4's two compiler/operator regressions pass.
- Third-party Complex arithmetic and Float equality edge tests pass.
- The Int constructor/arithmetic regression reaches constructed=false.
- `RuntimeDataRegistry` interns by raw RuntimeTypeRef handles.
- `typing::inspect::equivalent` returns false for a Base/Overlay pair. It cannot
  currently establish a shared exact identity for these representations.

## 10. Causal hypothesis

Raw metadata/overlay handle identity is being used where exact semantic type
identity is required. Runtime instantiation correctly creates the represented
generic application, but descriptor interning does not normalize its identity
against a precompiled closed application.

## 11. Alternative hypotheses

The overlay's arguments could retain uninstantiated or incorrectly selected
parameters. A focused generic-data reproducer should inspect exact argument
identity before normalizing descriptors. Matching only the nominal owner is
insufficient.

## 12. Attempts already made

Changed the convenience factory to a method-local component generic C to ensure
argument-driven inference. The exact identity mismatch remains. No runtime
canonicalization attempt has been made.

## 13. Why local authority is insufficient

Canonical runtime typing and data identity affect every generic data consumer,
equality/hash, phantom generics, reflection, and metadata-pool boundaries.
Correcting them requires a general owning-layer correction and explicit scope
approval. Removing the convenience factory would leave arithmetic-created
generic values exposed to the same representation distinction.

## 14. Decision requested

Authorize a general runtime typing/data identity correction: verify that closed
metadata and instantiated overlays represent the same exact application, then
intern data descriptors using canonical exact-type identity rather than raw
handle representation. Use stable canonical declaration/parameter products;
no type-name comparison, hash-only equality, or Complex-specific exception.
Add discriminators for direct versus generic construction, nested applications,
phantom parameters, and same-spelled owners in different modules.

## 15. Adopted decision

The user approved the recommended general runtime typing/data identity correction.
The plan and checkpoint now authorize the owning-layer correction and its focused
regression gate. No Complex-specific equality rule is introduced.

## 16. Implementation discriminator

Observed: nominal, phantom/row, and cross-module applications now share descriptor
identity. A direct-first `Box<Bool>` then generic construction was rejected by
the patch's physical-layout consistency guard.

Causal hypothesis and evidence: closed lowering uses compact Bool storage while
generic lowering uses Value storage. The existing raw-handle interner already
retains the first descriptor's layout, and `construct_data_value` consumes that
descriptor's layout rather than the incoming lowering spec. The data specification
explicitly permits different representations of the same exact type.

Correction: preserve the first descriptor and its physical layout; require only
consistent declaration-component-to-logical-slot mapping. This is the approved
canonical representative architecture, with no new value representation rule.

Discriminating test: direct-first Bool and generic-first Int construction,
recursive exactness, equality, and hashes in
`data_canonical_identity_matches_direct_and_generic_construction_in_either_order`.

## 17. Conditional recipe relocation

Observed: all three general data identity regressions pass. Complex factory versus
direct construction shares descriptor 13 and exact `Complex<Int>`. Arithmetic
produces correct components but descriptor 14 has `Complex<MapKeysIterator>`.
The component argument is `Base { pool: 0, node: 9 }` instead of `Base { pool: 1,
node: 9 }`.

Causal hypothesis and evidence: `remap_lowering_metadata_pool` relocates ordinary
call environment recipes and trait invocations to the materialized program pool,
but omits conditional invocation recipes and the conditional recipe map. The
operator compiler consumes the conditional spec's unrelocated recipe. Node 9 is
Int in the program pool and MapKeysIterator in the bootstrap pool.

Predicted effect: relocating conditional recipes through the existing materializer
will retain Int as the component argument; the new exact data interner then
reuses the correct `Complex<Int>` descriptor. This is a mechanical completion of
the existing metadata-pool contract, not a new identity rule.

Discriminators: Complex exact arithmetic and exact result identity in the existing
conditional operator/trait-slot regression.

## 18. Outcome

D5 is IMPLEMENTED + FOCUSED_TESTED. General identity regressions (3), exact
conditional operator regressions (2), current Complex regressions (3), graph
identity tests (5), and adjacent reflection tests (6) pass. Scoped format and
diff checks pass. The checkpoint records the owning suite's four unrelated
pre-runtime fixture failures. The full Complex plan remains PARTIAL.
