# INC-004 — TypeView and checked generic materialization silently close open Tuple types

**Finding ID:** `INC-004`  
**Severity:** High / P1 (potential P0 if an established typing proof is consumed after shape erasure; not demonstrated)  
**Confidence:** High for source-level defect; behavioral end-to-end reachability is not yet reproduced  
**Status:** Source-confirmed semantic transformation defect; execution fixture pending  
**Audit slice:** 2026-10-12, continuation / canonical materialization  
**Repository:** `aureat/phalcom-lang`, live `main` at or after `608c8c6cd3346556f3bbc98f0713a6df6e0a72ac` (reference baseline `5942666901b387519c5cd502bed2611cf97b53c4`)

## 1. Owners and affected consumers

- **Semantic owner:** `phalcom-semantic/src/types/environment.rs`, `materialize_view`, lines 213–223, invoked by `TypeView::materialize` around lines 95–108.
- **Independent affected owner:** `phalcom-semantic/src/types/instantiation.rs`, `materialize_type_inner`, lines 172–184, invoked by `materialize_type` around lines 60–67 and `TypeView::materialize_checked` around lines 120–123.
- **Canonical formation authority:** `phalcom-semantic/src/types/store.rs`, `TypeStore::tuple` and `TypeStore::open_tuple`, lines 1057–1097.
- **Control implementation:** `phalcom-semantic/src/types/substitution.rs`, `try_apply_with_arena`, lines 134–155, and `apply_with_arena`, lines 289–306, preserve and specialize both remainders correctly; both `Pack` branches likewise preserve them.
- **Known usage sites:** `TypeView::materialize` appears in signature projection, checker expression/constructor elaboration, trait witness/callable specialization, implementation-member views, and inheritance projections. Specific source-level reachability from a user-written open Tuple must still be isolated.

## 2. Authoritative contract

`docs/specs/structural-products.md` §3.2 defines `Tuple(P, L, Rem_P, Rem_L)` where positional and labeled remainders are **independent parts of the structural type**. `Closed`, `Typed(T)`, and `Unconstrained` denote different accepted sets of values. The companion `docs/specs/tuple-and-record-typing.md` §§2–5 agrees: tuples are closed by default and only explicit remainders open lanes. Therefore a specialization that merely changes a type parameter must preserve lane openness and specialize the types *inside* typed remainders.

## 3. Current implementation and deterministic consequence

In `environment.rs:213–223`:

```rust
TypeData::Tuple(elems) => {
    let subst_elems: Vec<TupleTypeElement> = elems.elements.iter()
        .map(|e| TupleTypeElement {
            label: e.label.clone(),
            ty: materialize_view(store, e.ty, env),
        })
        .collect();
    store.tuple(subst_elems.into_boxed_slice())
}
```

In `instantiation.rs:172–184`, the corresponding checked materializer also collects only fixed elements and returns `Ok(store.tuple(elements.into_boxed_slice()))`.

`store.rs:1057–1060` defines `tuple(elements)` as:

```rust
self.open_tuple(elements, AnonymousTupleRemainder::None,
                          AnonymousTupleRemainder::None)
```

Consequently **both materializers turn any open Tuple into a closed Tuple** whenever their Tuple branch executes. They also fail to substitute type variables inside `Typed(T)` remainders. For a Tuple containing no fixed elements, the result can become canonical `Unit`, despite the source denoting any compatible open-length Tuple.

Contrast `substitution.rs:134–155,289–306`, which reads both remainder fields, substitutes their `Typed` payloads, and calls `store.open_tuple(...)`; see also `environment.rs:292–309` and `instantiation.rs:245–268` for `Pack` handling.

## 4. Minimal unexecuted reproducer (existing public semantic APIs)

Rust-level scenario to exercise in an audit-only standalone harness, not a repository test modification:

```rust
// Pseudocode; use the repository's actual constructors for TypeParameterId.
let t = store.parameter_form(param_t);
let src = store.open_tuple(
    Box::new([]),
    AnonymousTupleRemainder::Typed(t),
    AnonymousTupleRemainder::Unconstrained,
);
let mut env = TypeEnvironment::new();
env.bind_param(param_t, int_ty);
let view = TypeView::new(src, env);
let actual = view.materialize(&mut store);
// EXPECT: Tuple with Typed(int_ty) positional and Unconstrained labeled tail.
// SOURCE PREDICTS: Unit (both remainders erased).
assert_ne!(actual, store.unit());
```

Repeat using `view.materialize_checked(&mut store, RowMaterializationMode::PreserveUnboundStableTail)`; repeat with one fixed entry to distinguish tail erasure from empty-Tuple-to-Unit collapse. Test `Unconstrained` and `Typed` for each lane independently.

**Reproduction command:** Not yet executed. The original repository/toolchain is not locally mounted in this audit session, and no semantic test, Rust binary, or fixed-source patch was inserted. `cargo test` cannot be claimed from source review.

**Observed output:** Exact branch and constructor behavior is visible in the current GitHub source; **no observed runtime/test output** exists in this slice.

**Expected output:** Structurally equivalent specialized open Tuple with preserved tail states and correctly specialized tail element types.

## 5. Failure taxonomy and violated invariants

- **A — potential soundness:** A nominally open Tuple can be incorrectly treated as closed/`Unit`, changing permitted values and future proof results. Whether this accepts invalid source or produces incorrect executable behavior requires a consumer-level probe.
- **B — completeness:** Valid open Tuple specialization can appear as the wrong closed structural shape, leading to spurious rejections.
- **E — semantic consistency:** The `TypeSubstitution` path preserves remainders, while both materialization paths erase them, making the result dependent on which specialization API is used.
- **T2:** Structural type identity is not preserved.
- **T4/T5:** Downstream evidence could reference the wrong structural constraint; this consequence is not independently proven.
- **T9:** Equivalent specialization semantics disagree by API path.

Do not mark this as a confirmed source-program soundness exploit without direct fixture evidence.

## 6. Root cause

The Tuple handling was reconstructed using the **closed-only** `TypeStore::tuple` convenience constructor rather than the full `TypeStore::open_tuple(elements, positional_remainder, labeled_remainder)` structural constructor. Both independent specializers omit the remainder payload and thus bypass specialization of its parameterized contents. The implementation already demonstrates the correct transformation pattern in `substitution.rs` and both Pack branches.

## 7. Recommended correction layer (NOT AUTHORIZED BY THIS AUDIT)

Correct **both** `types/environment.rs::materialize_view` and `types/instantiation.rs::materialize_type_inner`. Rebuild through `open_tuple` with independently preserved `None`, `Unconstrained`, and recursively specialized `Typed(TypeId)` states. Make tests compare both canonical identity and the two remainder values. Investigate the possibility of one shared constructor-preserving traversal to eliminate divergence with `TypeSubstitution`, while not obscuring differences in checked row handling, dynamic bindings, and owner-relative `Self`.

## 8. Required regressions

1. Identity under an unrelated, nonempty environment: an open Tuple must remain identical.
2. Typed positional remainder `*: T` specializes to `*: Int` without closing either lane.
3. Typed labeled remainder `**: T` specializes independently.
4. Dual remainders `***: T` specialize both lanes.
5. `Unconstrained` lane states survive unchanged.
6. Empty-fixed open Tuple **never** canonicalizes to `Unit`.
7. Closed Tuple still remains closed, including its empty `Unit` case.
8. `TypeView::materialize`, `TypeView::materialize_checked`, and `TypeSubstitution::apply` agree where their inputs/domains overlap.
9. Positive and negative subtype/assignability checks downstream of specialization reflect source constraints.
10. Source-to-semantic integration for generic tuple-bearing member, inherited signature, trait signature, and residual specializer, followed by relevant compiler regression if lowering depends on open shape.

## 9. Potential collateral effects and limitations

- Do not merge Tuple and Pack materialization semantics indiscriminately: both are distinct proper products with distinct empty values.
- Preserve row materialization error propagation and explicitly documented `Self`/dynamic behavior.
- Evaluate `TypeStore::project_positionals/project_labeled` separately; their remainder behavior is an adjacent finding candidate, not established here.
- No source/tests were modified and no direct reproduction was run. Severity and confidence must not be conflated.

## 10. Source links

- [`environment.rs` L213–223](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/types/environment.rs#L213-L223)
- [`instantiation.rs` L172–184](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/types/instantiation.rs#L172-L184)
- [`store.rs` L1057–1097](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/types/store.rs#L1057-L1097)
- [`substitution.rs` L289–306](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/types/substitution.rs#L289-L306)
- [Normative `structural-products.md` §3.2](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/docs/specs/structural-products.md#L217-L240)

**Verification:** Source contradiction confirmed on live GitHub `main` code consistent with indexed commit `608c8c6...`; direct test, local working-tree state, and source-level reachable exploit remain **unverified**.
