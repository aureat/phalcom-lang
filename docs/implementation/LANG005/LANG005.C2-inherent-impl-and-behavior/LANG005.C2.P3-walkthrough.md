# Walkthrough — LANG005.C2.P3 Constrained and Specialized Inherent `impl` Applicability

## Result

```yaml
plan: LANG005.C2.P3
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

P3 extends inherent behavior with receiver-specialized and constrained impls while keeping applicability in semantic analysis and keeping conditional members out of shared unconditional runtime classes.

## Supported source forms

```phalcom
data Box<T>(_ value: T)
impl Box<Int> { intValue() -> Int { self.value } }
impl<T> Pair<T, T> { same() -> Bool { true } }
impl<T> Box<T> where T == Int { intValue() -> Int { self.value } }
```

Concrete heads, repeated parameters, nested applied heads, and bounded `where` assumptions are matched structurally. Covering P1 impls remain the unconditional fast path.

## Semantic authority

The landed product is `ConditionalDispatchSelection`:

```text
ImplId + CallableId + declaring DeclarationId + DispatchSide
```

`ResolvedDispatch` and `ExpressionAnalysis` retain that selection for each applicable expression. `BoundBehavioralMember.conditional` carries the same evidence into bound families. The matcher binds impl-owned parameters structurally, checks repeated bindings and constraints, and returns proof-backed selection; no consumer re-solves the domain.

Receiver-effective lookup starts with the ordinary surface and adds only applicable conditional members. Ordinary selectors take precedence across the traversed dispatch-owner hierarchy. Same-owner duplicate selectors remain rejected regardless of domain; different selectors may coexist in overlapping domains.

Impl member bodies receive the specialized receiver substitution, callable-owned generics, and proven impl constraints. `Self` therefore denotes the impl target head, and a conditional peer is callable only when the same semantic proof is available.

P2 exact-case identity remains `VariantId`-based. Case environments are established before P3 matching, so exact-case and GADT refinements compose without a second case matcher. Enum root defaults and requirements continue through the P2 closed-enum path; P3 adds no unbounded requirement solver or trait constraint system.

## Lowering and execution

The compiler projects the semantic conditional selection into `ConditionalInvocation` and into executable conditional-family descriptors. A family descriptor stores exact operation, selector, declaring owner, dispatch side, and a closure constant-pool index for the fallback callable. The fallback is therefore rooted by the existing closure constant machinery rather than by an untraced descriptor `ObjRef`.

At execution, `VM::select_conditional_method` probes only for an ordinary method defined by a strict subclass of the semantic declaring owner. If one exists it wins; otherwise the selected fallback executes. The conditional fallback shadows ancestors above its declaring owner. Class-side dispatch resolves the exact declaring class and uses its metaclass; it does not allocate one runtime class per applied generic type.

Conditional members are never installed in the shared generic behavior class. An inapplicable direct or bound receiver consequently follows ordinary missing-member behavior, and erased/dynamic runtime class identity cannot reconstruct generic applicability.

Direct calls, bound method references, behavioral families, getters, setters, index selectors, and class-side calls use the same semantic selection fields and runtime conditional dispatch boundary. The focused regressions cover direct and bound specialized behavior plus applied class-side behavior.

## Tooling and incrementality

Editor receiver alternatives retain the formal `TypeId` and an independent instance/class mode. Applied class-side receivers are accepted only with an applied type form; unknown or insufficient evidence fails closed. Completion delegates to semantic receiver-effective conditional lookup. Source definitions and references continue to use canonical impl-origin callable identity.

Callable-body fingerprints include conditional selection identity. Impl-domain/target changes invalidate affected effective lookup products, while body-only edits preserve unrelated surfaces and callable products. The semantic incremental queries compare the changed snapshot with cold analysis for the affected receiver surfaces.

## Implementation deviations and consultation

Private names and descriptor decomposition adapted to the live tree. The only material design question was recorded in `LANG005.C2.P3-INC-001`: the advisor confirmed the plan and required canonical per-expression selection, rooted bound-family projections, exact class-side identity, and a shared strict-subclass override probe. No plan amendment was required.

## Tests and coverage

Added/updated regressions:

- `bound_family_retains_the_selected_specialized_impl` proves selected `Box<Int>` behavior is retained and `Box<String>` is rejected.
- `specialized_class_side_impl_requires_the_applied_receiver` proves class-side `Box<Int>` selection and `Box<String>` fail-closed behavior.
- Conditional selection is included in callable-body fingerprints; existing semantic test literals were updated for the published product field.

| Coverage | Evidence |
|---|---|
| CA-01, CA-03, CA-05, CA-06, CA-07 | `semantic::impls` focused suite, 40/40 passed; specialized/repeated/nested/unbound target lanes |
| CA-02, CA-04, CA-08 | Same semantic suite; exact-head, repeated mismatch, and covering-surface lanes |
| CA-09–CA-14 | Same semantic suite; constraint, symbolic proof, and inherited receiver lanes |
| CA-15–CA-18 | Same semantic suite; conflict, overlap, and exact-case coherence lanes |
| CA-19–CA-26 | Same semantic suite; substituted signatures, body environments, peer calls, and union availability lanes |
| CA-27–CA-30 | Semantic impl/ADT lanes; exact-case, GADT, root leakage, and requirement/default behavior |
| CA-31–CA-33, CA-36–CA-37 | Core `language::inherent_impl`, 15/15 passed; lowering, fallback, inapplicable, and bound-family behavior |
| CA-34–CA-35 | Shared `VM::select_conditional_method` implementation and core hierarchy dispatch lane |
| CA-38 | Selector-preserving semantic/lowering/runtime paths; getter/setter/index selectors remain one applicability domain |
| CA-39–CA-40 | Exact class-side regression passed; class-side applied form and dynamic fail-closed paths |
| CA-41 | Callable fingerprint regression passed |
| CA-42–CA-44 | `semantic::impls::queries`, 13/13 passed; target/body/exact-case/unrelated invalidation and cold agreement |
| CA-45 | Source-index canonical identity regression and LSP impl navigation, 2/2 passed |
| CA-46 | Semantic editor lane, 6/6 passed, plus LSP semantic completion, 9/9 passed |
| CA-47–CA-48 | Scoped negative searches found no production source-order dependence or consumer-side impl applicability scans |

Focused commands actually run:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic -p phalcom-core -p phalcom-lsp
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::impls
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::impls::queries
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::inherent_impl
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::integration::editor
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::integration::source_index::source_index_publishes_impl_target_and_member_with_canonical_identity
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic semantic::incremental::fingerprints::callable_body_product_includes_resolved_callable_identity
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --test integration impl_navigation
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp --test integration semantic_completion
```

`git diff --check` passed. Broad workspace build/test/clippy and release certification were intentionally deferred; they are not required for P3 focused completion. LANG005 remains in progress because C3 trait work is separate.

## Residual risks and next boundaries

- C3 owns trait declarations, abstract surfaces, and conformance; P3 must not be generalized into that system.
- Per-applied-type runtime classes and public reflection of impl domains remain outside P3.
- Broader workspace/release gates remain uncertified.
