# Handoff — LANG005.C2.P1 First-Class Inherent `impl` and Effective Declaration Surfaces

## State

```yaml
plan: LANG005.C2.P1
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
checkpoint: LANG005.C2
next_plan: LANG005.C2.P2
```

C2.P1 is complete. The owning checkpoint remains `IN_PROGRESS` for the
planned variants-only enum migration (C2.P2) and constrained/specialized
applicability work (C2.P3). No active P1 incident is open.

## Inherited architecture

- Semantic effective surfaces are canonical. Lowering consumes immutable
  `SemanticSnapshot.callable_definitions`, never a reconstruction from
  selectors, dispatch side, or source order.
- `CallableId.owner` is always the target nominal `DeclarationId`. `ImplId`
  exists for provenance and AST attachment, not runtime ownership.
- `source_member_index` in `InherentImplMemberLowering` selects the accepted
  AST member. Rejected definitions are not published and cannot be installed.
- `Statement::Impl` is a top-level compile-time no-op. Accepted members are
  pre-indexed by canonical target and installed from class/data/enum compilers.
- Runtime uses existing behavior classes, inheritance, and layouts. There is
  no `ImplObject`, impl dispatch table, or layout mutation.
- P1 accepts same-module nominal class/data/enum targets with a covering
  generic bijection. Foreign, structural, aliased, specialized, repeated, and
  conditional heads remain rejected.
- Source index and LSP use semantic target mappings and target-owned callable
  identities. The impl block itself is not a declaration symbol.
- Impl body fingerprints are incremental inputs; body bytes are excluded from
  effective-surface fingerprints.

## Verification anchor

The checkpoint record is the current source of truth:
`LANG005.C2-CHECKPOINT.md`. The retrospective evidence is in
`LANG005.C2.P1-walkthrough.md`.

Key completed evidence:

- semantic package: 1,207 passed, 42 ignored;
- impl semantic suite: 35 passed;
- semantic incremental suite: 177 passed, 4 ignored;
- source-index suite: 26 passed;
- LSP integration: 60 passed;
- AST package: 270 passed, 1 ignored;
- modules package: 143 passed;
- core inherent-impl: 9 passed;
- core data E2E: 18 passed;
- core algebraic-data: 44 passed, 19 ignored;
- workspace all-target check: passed;
- workspace Clippy with `-D warnings`: passed;
- prohibited runtime representation search: zero production hits.

The full core package test and workspace-wide test were interrupted during the
slow language-corpus execution; Clippy completed successfully. The shared
checkout also has broad pre-existing formatting drift. These are documented as
incomplete release evidence, not as P1 behavior failures.

## Next implementer action

Read and execute:

`LANG005.C2.P2-variants-only-enums-and-impl-behavior-migration.md`

Preserve the P1 effective-surface, snapshot-provenance, target-owned callable,
same-module ownership, and no-layout-mutation invariants. C2.P2 should own the
variants-only enum/root/default/case behavior migration; do not broaden P1 to
admit specialized/conditional impls, which belong to C2.P3.

Recommended first checks:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::algebraic_data
```
