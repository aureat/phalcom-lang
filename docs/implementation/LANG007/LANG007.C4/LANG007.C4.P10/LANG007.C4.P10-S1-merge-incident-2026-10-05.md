# IMPLEMENTATION MERGE INCIDENT

## 1. Identity

- Incident: `INC-LANG007-C4-S1-2026-10-05`
- Program/checkpoint: `LANG007.C4`
- Plans: `LANG007.C4.P4` and `LANG007.C4.P10`
- Trigger: user-directed consolidation of interrupted P4 work with the P10
  worktree into `main`
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Main before merge: `18750433`
- P10 source head: `46d00056`
- Merge commit: recorded in Git history; checkpoint carries the durable state

## 2. Impact

The merge had two content conflicts and exposed compile breaks caused by P4's
canonical Record-row representation reaching consumers and test fixtures that
still expected string-only fields and untyped anonymous remainders. No P4 or
P10 commits were discarded. P10's pre-barrier work was merged into `main`; the
P10 checkout is preserved detached at the consolidated revision, including any
ignored local workspace data.

## 3. Required invariants

- Canonical Record-row identity uses `FieldKey`; string literals in legacy
  fixtures must explicitly identify Symbol keys.
- Anonymous Record remainders carry their key domain and optional value type.
- The existing anonymous-product lowering optimization remains Symbol-only and
  must fail closed for non-Symbol row keys.
- P4 owns Record-specific key syntax; P10's explicit `while` identity remains
  separate and must not be inferred from the `whileTrue` selector.
- `CheckingContext` must retain both P4's stable row-lacks facts and P10's
  expected kinded return contract.

## 4. Conflict analysis and resolution

The three-way merge found:

1. `phalcom-semantic/src/checker/context.rs`: P4 changed stable Record-row
   lacks from names to `FieldKey`; P10 added `expected_kinded_return`. The
   resolved file preserves both independent changes and all constructor/clone
   propagation.
2. `LANG007.C4-CHECKPOINT.md`: main carried the latest P4 T2/T7.0 decisions;
   P10 carried its ledger and pre-barrier evidence. The merged checkpoint
   preserves both plans' states and records S1 completion.

Git auto-merged the overlapping changes in `checker/expression.rs`,
`checker/statement.rs`, `lib.rs`, and `tests/semantic/foundations/mod.rs`. The
P10 plan was identical on both sides. P10's pre-barrier implementation did not
modify the P4 AST/parser files.

The post-P4 AST/parser review confirmed that P4 introduced
`RecordKeySyntax`/`AnonymousRecordRemainderSyntax` without broadening tuple
`ProductLabel`. Plain `while` still parses to the `whileTrue` method-call
shape; `while let` remains `Expr::WhileLet`. This did not conflict with P10's
planned explicit `Expr::While` work, so S1 has no remaining architecture
blocker. T6 was not implemented as part of this merge.

## 5. Compile failures and corrections

The first merged check identified four stale consumers in
`phalcom-core/src/modules/semantic_lowering.rs`. The adapter now compares
canonical `FieldKey` values, traverses the new anonymous-tail key/value types,
and exits the optimization path if a row key is not a Symbol. This preserves
the existing lowering contract and does not create a second name conversion.

The first semantic test-target compile then failed with 145 errors from stale
row fixtures and audit probes. Those fixtures now use explicit
`symbol_required`/`symbol_optional` constructors, `FieldKey` lookups/lacks, and
the canonical FieldKey union when constructing anonymous remainders. No
`From<&str>` compatibility conversion was added. A second compile pass found
remaining type-specific fixture sites; after migrating them, the semantic
integration target compiled successfully.

## 6. Verification evidence

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast -p phalcom-semantic -p phalcom-core --message-format=short` — passed after the core adapter.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --no-run -p phalcom-semantic --test semantic --message-format=short` — passed after fixture migration.
- `cargo test -p phalcom-ast record_field_key -- --nocapture` — 6 passed.
- Semantic focused tests passed: `foundations::field_keys` (7),
  `foundations::record_row_materialization` (3),
  `foundations::record_row_inference` (6), `advanced::record_rows` (12),
  `advanced::structural_presence` (16),
  `foundations::structural_classification` (2),
  `adts::structural_reflection` (2), and
  `integration::static_definition_bodies` (5).

No workspace-wide tests, clippy, or release certification ran. These results
close the S1 directly affected checks; they do not certify either full plan.

## 7. Lifecycle state and next action

- S1 synchronization barrier: complete.
- P4: remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; T2 focused evidence has
  advanced, while full T2 acceptance and downstream P4 work remain open.
- P10: remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; T6 and while-dependent T9
  cases remain unstarted. The inherited fixed-call Pack runtime test remains
  unverified.
- Next implementation action: continue P4 T2 acceptance; P10 T6 may begin in a
  later turn now that S1 is complete.
- Durable shared state: [`LANG007.C4-CHECKPOINT.md`](../LANG007.C4-CHECKPOINT.md).

No architectural consultation was required: both conflicts had
representation-preserving resolutions, and the AST audit found no conflict
with P10's fixed `while` design.
