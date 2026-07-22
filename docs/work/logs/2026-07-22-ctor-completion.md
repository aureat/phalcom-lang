# U-CTOR Completion Implementation

- Date: 2026-07-22
- Specification: `docs/work/pending/ctor-completion-implementation-spec.md`
- Plan: `docs/work/pending/ctor/plan.md`

## 1. Summary

Closed U-CTOR planning, diagnostic, and documentation debt without altering constructor compiler or runtime semantics:
- Split generic member duplicate diagnostic `class.duplicate_member` into `class.duplicate_selector` (`DuplicateSelector`) for same-side canonical selector collisions and `class.duplicate_field` (`DuplicateField`) for field declaration collisions.
- Updated Pass -1 duplicate member scan in `class_decl.rs` and error span mapping in `dispatch.rs`.
- Updated test expectations for negative class duplicate fixtures.
- Purged live `ConstructStaticCollision` references from compiler comments, catalog, followup docs, and U-CTOR plan text.
- Updated `docs/work/pending/ctor/plan.md` status from `READY` to `IMPLEMENTATION COMPLETE (CLOSURE WORK PENDING)`, designating `docs/work/pending/ctor-completion-implementation-spec.md` as closure authority.

## 2. Key Code and Document Changes

### Core (`phalcom-core`)
- `compiler/lib/error.rs`: Added `CompilerError::DuplicateSelector` (`class.duplicate_selector`) and `CompilerError::DuplicateField` (`class.duplicate_field`).
- `compiler/lib/class_decl.rs`: Updated duplicate scan pass to emit distinct diagnostic contracts; removed stale `ConstructStaticCollision` compiler comment.
- `vm/dispatch.rs`: Updated `compiler_error` mapping to render spans for both duplicate variants.
- `tests/lang/classes/negative/class_duplicate_member_field.expected`: Updated prefix to `class.duplicate_field`.
- `tests/lang/classes/negative/class_duplicate_member_method.expected`: Updated prefix to `class.duplicate_selector`.

### Documentation (`docs/`)
- `docs/work/pending/ctor/plan.md`: Updated status to `IMPLEMENTATION COMPLETE (CLOSURE WORK PENDING)`, cited spec as authority, updated `ConstructStaticCollision` write-set/text.
- `docs/spec/current/traceback/output-catalog.md`: Removed `ConstructStaticCollision` from active unrendered error list.
- `docs/work/deferred/error-handling-followups.md`: Added supersession note for `ConstructStaticCollision`.

## 3. Deferred Verification

Per Section 5 of `docs/work/pending/ctor-completion-implementation-spec.md` and locked decision 2, final repository verification (`./scripts/verify.sh`, `cargo doc`, `graphify update .`, throwaway worktree verification) is deferred until explicit user instruction.
