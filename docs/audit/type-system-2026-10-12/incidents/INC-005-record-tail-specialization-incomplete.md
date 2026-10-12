# INC-005 — Record-row specialization fails to realize captured and anonymous tails

**Finding ID:** `INC-005`  
**Severity:** High / P1  
**Confidence:** High for source-level loss of environment binding and typed-tail substitution; downstream source-program impact unverified  
**Status:** Source-confirmed incomplete materialization; direct API and end-to-end fixtures pending  
**Audit slice:** 2026-10-12, continuation / canonical record-row specialization  
**Repository/revision:** `aureat/phalcom-lang`, `main`, source consistent with GitHub index commit `608c8c6cd3346556f3bbc98f0713a6df6e0a72ac`.

## 1. Ownership, scope, consumers

- `phalcom-semantic/src/types/environment.rs`: `TypeEnvironment::bind_row`, `to_generic_instantiation`, `TypeView::materialize`, and `materialize_view` Record arm; specifically lines 14–72, 95–108, 224–240.
- `phalcom-semantic/src/types/instantiation.rs`: `materialize_type_inner`, `materialize_row`, `materialize_row_tail`, lines 185–187 and 347–405.
- `phalcom-semantic/src/types/row.rs`: canonical `RecordRowTail` and `AnonymousRecordRemainder` variants, lines 16–46 and 87–100.
- Control path: `phalcom-semantic/src/types/substitution.rs::apply_record_row_with_arena`, around lines 207–245, explicitly traverses `AnonymousRecordRemainder::{Unconstrained,Typed}` payloads.
- Potential consumers of `TypeView::materialize`: checker member signatures, traits/implementation specialization, callable dispatch, inherited member contexts. Concrete row-tail entrypoint reachability through these consumers remains to be isolated. `TypeView::materialize_checked` is publicly exposed but no production call other than the definition was found in the indexed search; do not assert it is used in compiler production.

## 2. Contract

The normative `docs/specs/structural-products.md` §3.4 represents Record as `Record(Fields, Tail)` with captured, anonymous typed, anonymous unconstrained, and closed row states. Captured `R` preserves correlated row identity until specialized; `**: T` constrains every extra value. A generic specialization with binding `R -> row` must realize that row where the API claims materialization, and substitution of `T -> Int` must update the `Typed` remainder's value type. The unchanged open/closed state alone is insufficient.

## 3. Two independent semantic discrepancies

### 3.1 `TypeView::materialize` ignores `TypeEnvironment::row_bindings`

`environment.rs` stores rows in `row_bindings`; `is_empty` counts these bindings, and `to_generic_instantiation` copies them. Yet `materialize_view` handles records as:

```rust
TypeData::Record(row_id) => {
    let (fields, tail) = {
        let row = store.record_row(row_id);
        (row.fields.to_vec(), row.tail)
    };
    let subst_fields = fields.into_iter()
        .map(|f| RecordRowField {
            key: f.key,
            ty: materialize_view(store, f.ty, env),
            presence: f.presence,
        }).collect();
    store.record_row_type_checked(subst_fields, tail).expect(...)
}
```

No reference to `env.get_row` occurs in this Record branch, so a captured `RecordRowTail::Parameter(R)` remains unresolved despite an active `R -> concrete_row` binding. Nested `StaticArgument::RecordRow`, `TypeMemberArgument::RecordRow`, and `ProjectionSubject::RecordRow` branches elsewhere in the same function similarly preserve raw row IDs. The checked materializer instead calls `materialize_row`, whose `materialize_row_tail` resolves a captured `Parameter(R)` and merges the bound fields, thus the two APIs disagree.

### 3.2 Both materialization paths leave anonymous tail payloads unspecialized

The lazy Record branch copies `tail` without recursively processing its value/key type terms. The checked `instantiation.rs::materialize_row_tail` uses:

```rust
let RecordRowTail::Parameter(parameter) = tail else {
    return Ok((fields, tail));
};
```

This immediately returns any `RecordRowTail::Anonymous(Typed { key_type, value_type })`, `Anonymous(Unconstrained { key_type })`, or `Projection(TypeId)` unchanged. In particular, substitution `T -> Int` in a Record with `**: T` specializes fixed fields but not `value_type` (and potentially `key_type`), so a Record source contract remains partially parameterized. `substitution.rs` explicitly handles both anonymous remainder variants, confirming that payload specialization is representable and expected in the canonical data structure. The `Projection` variant requires its own domain-aware evaluation policy; its unchanged treatment is an adjacent unverified case, not a conclusively specified bug in this report.

## 4. Minimal audit-only reproducer designs (not executed)

**Case A — captured tail binding:** Build a Record with a fixed `a: Int` and `RecordRowTail::Parameter(R)` where `R` is a properly registered `RecordRow`-kinded parameter. Bind `R` to a canonical closed row containing `b: String`. Then invoke `TypeView::new(original, env).materialize(store)`. Expected result has required `a`, `b` and closed tail. The current source preserves `Parameter(R)` instead. Compare with `TypeView::materialize_checked(store, RequireSolvedTail)`; the checked row materializer explicitly expands the binding.

**Case B — typed anonymous tail:** Build `Record` with an anonymous remainder whose `value_type` is canonical `T`. Bind `T -> Int` in a nonempty `TypeEnvironment`, and materialize both `materialize` and `materialize_checked`. Expected `value_type == Int`; source retains `T` in both APIs. Include an independently bound `key_type` parameter when allowed by its proper kind contract.

**Case C — negative companion:** With unbound `R`, `RequireSolvedTail` returns `TypeMaterializationError::UnresolvedRowParameter(R)`; `PreserveUnboundStableTail` retains it. No failed materialization may fabricate a closed row. Include repeated/cyclic bound tails to validate the existing `RecursiveRowSubstitution` guard.

**Reproduction command:** Not executed; repository checkout and pinned Cargo toolchain are not available in the local container. Do not construe the source trace as an observed compiler test. No repository fixtures or source files were edited.

## 5. Root cause and outcome taxonomy

- **Root cause:** Specialized `TypeEnvironment` carries heterogeneous bindings, but its unchecked recursive walker handles only fixed field type terms and copies row tails verbatim. The checked instantiation implementation resolves captured `Parameter` tails, but treats every other tail variant as opaque. These partial traversals violate the full canonical Record representation.
- **B — completeness:** A concretely bound captured row fails to materialize and valid generic Record use may remain artificially symbolic.
- **E — semantic consistency:** Checked and unchecked TypeView materialization can produce different structural types for the same row environment. TypeSubstitution differs again for anonymous typed tails.
- **A — conditional risk:** A wrong row type can become an incorrect proof or compiler lowering contract; **no invalid-program acceptance is established yet**.
- **Invariants:** T2 canonical type identity/structure, T4 evidence integrity (downstream risk), T7 incremental equivalence (not demonstrated), and T9 consistent specialization behavior.

## 6. Recommended owning-layer changes (not authorized)

1. Centralize Record-row specialization through a domain-aware recursion supporting fixed fields, captured tail binding, anonymous tail payloads, and projection tails. Preserve the API distinction between checked (`Result`) and best-effort lazy materialization, and do not silently turn unresolved rows into closed rows.
2. Review nested row-bearing constructs in `TypeData::Applied`, type-member projections, and static-data terms, rather than only fixing `TypeData::Record`.
3. Ensure `RowMaterializationMode::RequireSolvedTail` reports unresolved captured parameters, while `PreserveUnboundStableTail` preserves them. Typed anonymous remainders are not row variables and should still specialize their constituent type terms.
4. Evaluate whether `TypeView::materialize_checked` must also preserve `Self` and dynamic bindings; `to_generic_instantiation` currently exports only type and row bindings. This is an **adjacent audit question**, not a confirmed consumer defect in INC-005.

## 7. Required regressions

1. `R -> closed row` captured tail merges fields correctly; no residual `Parameter(R)` survives.
2. Captured `R -> open row` preserves the bound row's tail state.
3. Nested row bindings and cyclic row bindings exercise successful recursion and controlled cycle failure.
4. `T -> Int` substitutes typed anonymous Record remainder value type in both materializers.
5. Proper kinded `key_type` payloads receive defined substitution where applicable.
6. `Unconstrained` anonymous remainder remains open while its key constraint specializes appropriately.
7. Closed Record remains unchanged under unrelated nonempty environments.
8. `materialize`, `materialize_checked`, and `TypeSubstitution` are differential-tested for overlapping domains; intentional partial behavior must be explicitly documented.
9. Negative relation tests ensure a stale/unspecialized row is not treated as a successful proof.
10. Source-level row-polymorphic callable/member specialization, trait conformance, and incremental edit-equivalence fixtures when production entrypoints are established.

## 8. Collateral effects and limitations

- Adding a bound row may introduce duplicate keys: retain `record_row_checked` formation failures; do not auto-overwrite fields.
- Specializing row tails must not turn abstract row parameters into fresh inference variables or erase correlation.
- Need an executable witness and caller-trace before upgrading failure severity to P0 or claiming actual user-code miscompilation.

## 9. Source references

- [`environment.rs` L14–72 and L224–240](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/types/environment.rs#L224-L240)
- [`instantiation.rs` L347–405](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/types/instantiation.rs#L347-L405)
- [`substitution.rs` L207–245](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/types/substitution.rs#L207-L245)
- [`row.rs` L16–46 and L87–100](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/types/row.rs#L16-L100)
- [Normative `structural-products.md` §3.4](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/docs/specs/structural-products.md#L256-L280)

**Verification:** High-confidence static reproduction by direct control-flow inspection; no compiler execution, no remote working-tree visibility, no source patch.
