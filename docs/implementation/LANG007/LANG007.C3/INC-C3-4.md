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

## 8. Scoped implementation diff

The current P2 changes already add stable nominal/associated domain and case identity
types and source-index attachment for associated static-domain/case resolutions. No
nominal-case source-spelling fallback or speculative resolution variant is retained.
The checkpoint records this incident as `INC-C3-4`.

## 9. Evidence

- `synthesize_associated_lookup` records `StaticTerm` only after the receiver has
  `SemanticDenotation::StaticDomain`; its generic owner path rejects type-only
  declarations before recording a case resolution.
- `CallableSourceAttachment` converts recorded associated `StaticDomain` and
  `StaticTerm` products to stable semantic targets.
- The focused associated source-index test passes:
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic adts::declarations::associated_static_domain_and_case_paths_have_stable_source_index_targets`.
- `git diff --check` passes for the current worktree.
- A temporary nominal-case test observed no target, no callable resolution, and the
  runtime-value diagnostic described above; the temporary test was removed.

## 10. Implementer's causal hypothesis

Nominal type-data case selection is not exposed as a semantic resolution product at
the expression/source-index seam. Associated domains use `AssociatedSurface` and have
an explicit resolution path; nominal domains take the type-only owner rejection path.

## 11. Alternative hypotheses

The source index might instead consume an already-published nominal type-data
declaration product directly, if that product can authoritatively identify the
selected case and participate in dependencies/invalidation. The live call path has
not established such a selection interface.

## 12. Attempts already made

- Tried a local target mapping for nominal cases. Prediction: a nominal stable target
  could be derived once the case was found. Observation: no canonical resolution
  product reached the source index; using token spelling would have created a second
  lookup authority. The attempted code and temporary test were removed.
- Verified the existing associated-domain path with its focused source-index test;
  it passes and remains unchanged.

## 13. Why local authority is insufficient

Choosing whether callable semantic resolution or a declaration/query product owns
nominal case selection changes semantic authority and incremental dependency wiring.
The implementer must not choose between these architectures or introduce an
independent registry without architectural direction.

## 14. Decision requested

Should nominal `type data` case selection publish a canonical semantic resolution
product carrying `StaticCaseId` for source-index consumers, or should source-index
construction consume an existing declaration product that already owns the selected
case? In either design, what product/dependency is authoritative for case edits?

**Status: OPEN.** T8 nominal-case source-index work must remain stopped until this
decision is recorded in the checkpoint and the plan is amended if required.
