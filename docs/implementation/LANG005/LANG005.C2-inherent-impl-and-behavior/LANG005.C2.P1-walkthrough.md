# Walkthrough — LANG005.C2.P1 First-Class Inherent `impl` and Effective Declaration Surfaces

## Final state

```yaml
plan: LANG005.C2.P1
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
checkpoint: LANG005.C2
```

## Outcome

C2.P1 now treats inherent `impl` blocks as first-class semantic contributions
to an existing nominal declaration. Accepted members are compiled into the
target's ordinary behavior class, independent of source order, without an
impl-specific runtime object, dispatch table, or storage layout.

Tasks 18–21 closed the source-tooling, incremental, hostile-matrix, and
delivery obligations. The checkpoint remains in progress only because the
planned C2.P2 variants-only enum migration and C2.P3 constrained/specialized
applicability work are separate plans.

## Architecture implemented

- `DeclaredSurface`, `InherentImplContribution`, `InherentImplSet`, and the
  effective `DeclarationSurface` remain semantic products owned by
  `phalcom-semantic`.
- Accepted callable provenance is published in the immutable snapshot through
  `EffectiveCallableDefinition` and `callable_definitions`.
- `CallableId.owner` is the target `DeclarationId`; `ImplId` is provenance and
  AST attachment identity only.
- `InherentImplLoweringSpec` consumes snapshot provenance and authoritative
  source-member indexes. The compiler pre-indexes these specs by target and
  installs accepted members during class, data, and enum-root compilation.
- Impl bodies use the target nominal `self` type, target-private access rules,
  ordinary class-side/instance dispatch, and ordinary inheritance.
- Source indexing consumes the semantic target-reference mapping. Impl target
  occurrences resolve to the primary declaration, and impl member declarations
  resolve to target-owned callable IDs. LSP exposes the keyword/member token
  and canonical definition/reference behavior without creating an impl symbol.
- Impl signature/body fingerprints participate in semantic-shard reuse. Body
  bytes do not enter declaration-surface fingerprints, while signature/member
  changes invalidate the affected target and dependent callers.

## Important files

- `phalcom-semantic/src/impls.rs` — target resolution, contribution admission,
  effective surfaces, conflicts, and callable provenance.
- `phalcom-semantic/src/semantic_shard.rs` — impl signature/body incremental
  fingerprints.
- `phalcom-semantic/src/source_index/builder.rs` and `scope.rs` — canonical
  target/member source indexing.
- `phalcom-semantic/src/snapshot.rs` and
  `phalcom-core/src/modules/semantic_lowering.rs` — immutable lowering
  authorization boundary.
- `phalcom-core/src/compiler/lib/impl_decl.rs` — reusable behavior-member
  compiler and accepted-member installer.
- `phalcom-core/src/compiler/lib/mod.rs`, `class_decl.rs`, `data_decl.rs`, and
  `enum_decl.rs` — pre-indexing and target compilation hooks.
- `phalcom-lsp/src/semantic_tokens.rs` and
  `phalcom-lsp/tests/impl_navigation.rs` — editor behavior.

## Diagnostics and rejection boundary

P1 introduces or exercises structured diagnostics for foreign targets,
non-nominal/type-alias/enum-case targets, specialized or repeated generic
heads, unused impl parameters, `where` clauses, constructor-marked and
bodyless members, and effective-surface conflicts. Compiler-side missing
lowering and callable/AST mismatch failures remain fail-closed.

Rejected duplicate definitions are absent from `callable_definitions` and
cannot be installed by lowering. Valid definitions are admitted once in
deterministic source order; runtime behavior never relies on last-wins
replacement.

## Verification executed

- Full semantic package: **1,207 passed, 42 ignored, 0 failed**.
- Semantic impl suite: **35 passed, 0 failed**.
- Semantic incremental suite: **177 passed, 4 ignored, 0 failed**.
- Semantic source-index suite: **26 passed, 0 failed**.
- Full LSP integration suite: **60 passed, 0 failed**.
- Full AST package: **270 passed, 1 ignored, 0 failed**.
- Full modules package: **143 passed, 0 failed**.
- Core inherent-impl runtime suite: **9 passed, 0 failed**.
- Core data E2E filter: **18 passed, 0 failed**.
- Core algebraic-data filter: **44 passed, 19 ignored, 0 failed**.
- Workspace all-target check: passed.
- Workspace Clippy with `-D warnings`: passed.
- Negative runtime representation search: zero production hits for
  `ImplObject`, `impl_id.*Object`, and `runtime_impl`.
- Compiler reopen-path search: no prohibited impl-to-class-reopen path.

## Deliberately incomplete or deferred evidence

- `cargo fmt --all -- --check` is not clean in the shared checkout because of
  broad formatting drift across pre-existing work; it is not promoted to a
  release gate here.
- The full `cargo test -p phalcom-core` invocation was interrupted while its
  language-corpus target was still running. Its completed targets are not
  represented as a package-wide pass. Focused core corpus and required C1/ADT
  filters passed.
- The workspace-wide test gate was attempted and interrupted in the existing
  slow core language-corpus target after earlier workspace targets passed.
  Release certification is therefore not claimed.

## Follow-up ownership

- `LANG005.C2.P2` owns variants-only enum declarations and migration of root,
  default, and case behavior into impls.
- `LANG005.C2.P3` owns constrained, specialized, and repeated-parameter impl
  applicability beyond P1's covering generic bijection.
- Trait-conformance/trait-style impl syntax is outside the current P1 language
  surface and is not silently admitted.

No active C2.P1 implementation incident remains.
