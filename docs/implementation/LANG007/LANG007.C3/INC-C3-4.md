# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C3
- Plan: LANG007.C3.P2
- Task/Gate: T8 — nominal static-case source-index identity
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `79e6726093349d3269460a29a7c1c4c3035ec2cf` (P2 baseline recorded in checkpoint)
- Current revision: `c99f7b50` (working tree contains uncommitted P2 work)

## 2. Escalation trigger

Mandatory triggers 11 and 14: plausible fixes assign semantic resolution and
invalidation to different layers; a source-spelling fallback would create a second
authority for canonical case selection.

## 3. Required invariant

Source-index targets must be derived from canonical semantic selection and stable
declaration identity. Do not infer a target from the token text or add a separate
case-name registry. Preserve the existing runtime-value rejection for static terms.

## 4. Expected architecture

T8 requires stable declaration-based static-domain and case targets. The associated
case path already records a `StaticCaseId` in the semantic resolution product, which
the source index consumes. Nominal type-data cases need an equally authoritative
selection path without weakening that boundary.

## 5. Observed behavior

For `type data Shape { Scalar }` and a method expression `Shape::Scalar`, callable
analysis publishes no associated/case resolution and the source index has no target
for the `Scalar` token. The checker emits the existing `TypeOnlyRuntimeValue`
diagnostic for the expression. By contrast, an associated path such as
`Parser::State::Done` publishes a static-domain result followed by a static-term
result, and source-index attachment can assign stable domain/case targets.

This report concerns missing canonical identity for the nominal source reference; it
does not claim that producing a runtime value is valid.

## 6. Minimal reproduction

```phalcom
type data Shape { Scalar }
class Probe {
  @class status() { Shape::Scalar }
}
```

Exploratory semantic regression (temporary test, removed after establishing the
authority gap): analyze the source, inspect the `Scalar` occurrence in the module
source index, and inspect callable `associated_resolutions`.

Observed:

```text
source-index target for Scalar: None
callable associated resolutions: []
diagnostic: TypeOnlyRuntimeValue
```

Expected: a stable `StaticCaseId` for the nominal `Shape` declaration is available to
the source index, while runtime use remains rejected.

## 7. Relevant code path

```text
Expr::AssociatedLookup
  -> synthesize_associated_lookup (checker/expression.rs)
  -> resolve_associated_owner
  -> declaration_is_type_only rejection
  -> no case-resolution product recorded

associated static-domain path
  -> AssociatedResolutionKind::StaticTerm
  -> CallableSourceAttachment (source_index/mod.rs)
  -> stable StaticCase target
```

Relevant identity types are `StaticDomainDeclarationId` and `StaticCaseId` in
`phalcom-semantic/src/identity.rs`.

## 8. Advisor decision and plan amendment

The advisor resolved the ownership question: nominal type-data case selection must
publish the existing canonical `StaticTerm` resolution carrying stable
`StaticCaseId` before runtime-value legality is checked. `CallableSourceAttachment`
consumes that product and must not inspect source spelling or search declarations.
The semantic product/query dependency that owns the static-domain case set controls
case-edit invalidation. No new resolution variant is required.

Applied plan amendment: nominal and associated static domains converge before case
selection; the selected domain records its owning declaration dependency, emits the
canonical static term, and only then reports `TypeOnlyRuntimeValue` for illegal
runtime use. The checkpoint records INC-C3-4 as resolved.

## 9. Implemented seam

- `synthesize_associated_lookup` routes both a static-domain denotation and a
  type-only nominal owner through `resolve_static_domain_case`.
- The helper selects from the canonical `TypeStore` static-constructor set, constructs
  the stable nominal or associated `StaticCaseId`, records `StaticTerm`, and returns
  the semantic term. The nominal runtime diagnostic is emitted after this selection.
- Nominal selection records the owning enum/data declaration dependency. Associated
  case-set products are query-published and the owner's `AssociatedSurface` depends
  on that product; no second lookup registry was added.
- Source-index construction remains a consumer of semantic resolutions only.

## 10. Verification evidence

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic` — PASS.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic nominal_static_case` — PASS (2 tests): canonical target plus runtime diagnostic; missing case has diagnostic and no target.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic nominal_static_case_rename` — PASS: rename recomputes the selecting body and source target, incremental target matches cold analysis, unrelated type-member product remains reusable.
- The pre-existing associated static source-index regression remains part of the focused T8 gate and must be rerun at plan stabilization.

**Status: RESOLVED; implementation is focused-tested.** This incident does not assert
that all P2 tasks or gates are complete.
