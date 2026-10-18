# IMPLEMENTATION INCIDENT

> Historical incident packet at `78cfd920`; implementation was subsequently
> committed as `f3403d70`. Resolved by the user-supplied 2026-10-11 audit and
> approved disposition 2: split G6.A snapshot acceptance from retained G6.B
> package certification. The amended P5 plan and C1 checkpoint own current
> acceptance and evidence. The audit establishes Option<Int> byte misuse as the
> Some/Int failure's cause; this packet's earlier comparator uncertainty is
> superseded.

## 1. Identity

- Program: `COLL006`
- Checkpoint: `COLL006.C1`
- Plan: `COLL006.C1.P5`
- Task/Gate: T7 / G6, parser/P14 convergence
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Worktree: `.worktrees/telos-collections-upgrade`
- Branch: `feat/telos-collection-upgrade`
- Starting revision: `78cfd9202f5dea7dd2713bff71d3e475f20810d6`
- Current revision: `78cfd9202f5dea7dd2713bff71d3e475f20810d6` (same HEAD; P5 changes are uncommitted in the working tree)

## 2. Escalation trigger

No mandatory RED trigger has been established. The consultation protocol explicitly treats a broad test failure already present at baseline as a non-trigger by itself.

This is an advisory request about acceptance and ownership: P5 T7.4 names the complete parse-package test as a gate, while the same failures were recorded at P5 G0 before P5 production changes. The implementer cannot waive the named gate or change its acceptance criteria without an owner/architect decision. No P5 architecture defect has been demonstrated.

## 3. Required invariant

- Mutating the caller's `List<T>` after `ListSource<T>` construction must not change the source's membership or order.
- `ListSource<T>` must use the P5 canonical private `Array<T>` snapshot retained by a read-only `Slice<T>`.
- Preserve parser behavior (`Cursor`, `ListSource.satisfy/select`, `Parse.run`, and ordered `Parse.runAll`).
- Do not introduce a runtime-sized `Vector<T>` snapshot or a new `ImmutableSequence<T>` runtime type.
- Keep the unrelated `TextSource.slice` and comparison failures outside P5 unless their owners authorize scoped work.

## 4. Expected architecture

T7 replaces `ListSource<T>`'s private copied `List<T>` with a private `Array<T>` snapshot retained through `Slice<T>`. T7.3 adds caller-mutation and parser-use assertions. T7.4 requires:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package
```

The plan says not to treat a single `source-test.ph` pass as complete parser convergence. G6's exit criterion is that the parser snapshot use case works on the canonical P5 collection model.

## 5. Observed behavior

The named command was rerun after T7's source and fixture changes. It selected two Rust tests; both failed on the same issues already recorded at G0:

1. The declared parse project fails to compile because `packages/parse/src/source/text-source.ph:40` declares `TextSource.slice` as returning `String`, but `_text.slice(start, end)` returns `Substring`.
2. The runtime fixture corpus reports the same imported `TextSource.slice` mismatch in 10 fixtures. The remaining operator fixture reaches a separate runtime failure: `byte >= 48` dispatches through `Object.>=` and reports unsupported `<=>` operands `Some` and `Int` (`object.ph:84` in the runtime diagnostic).

The updated `source-test.ph` has no diagnostic attributed to its new ListSource assertions, but it cannot execute past the imported TextSource failure. Therefore the P5 ListSource runtime behavior has not been certified by the required package gate.

## 6. Minimal reproduction

Command:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package -- --nocapture
```

Observed result (relevant diagnostics; the runner includes surrounding compiler/runtime context):

```text
selected 2 tests; both failed
parse_project_compiles_from_declared_entry:
  packages/parse/src/source/text-source.ph:40
  declared String, found Substring from _text.slice(start, end)

parse_runtime_fixtures_pass:
  10 fixtures: same TextSource.slice String/Substring mismatch
  expr/operators-test.ph: byte >= 48
  unsupported <=> operands Some and Int (Object.>= at object.ph:84)
```

Expected: both Rust tests pass; the parse project compiles, every runtime fixture reports its PASS marker, and the new ListSource snapshot assertions execute successfully.

Baseline: the checkpoint records this same package failure at G0, before P5 production edits. The P5 rerun therefore does not establish that the P5 changes caused either failure.

## 7. Relevant code path

```text
phalcom-core/tests/core/modules/parse_package.rs
  -> compile declared parse project / run every runtime *-test.ph fixture
  -> packages/parse/src/source/text-source.ph::TextSource.slice
       declares String; delegates to String.slice, observed as Substring
  -> packages/parse/tests/runtime/src/expr/operators-test.ph
       byte comparison reaches Object.>= -> Object.<=> dispatch
```

P5-owned path:

```text
packages/parse/tests/runtime/src/core/source-test.ph
  -> packages/parse/src/source/list-source.ph::ListSource<T>
       copies caller List<T> into Array<T>, retains backing through Slice<T>
  -> Cursor / Parse.run / Parse.runAll assertions
```

## 8. Scoped implementation diff

For this blocker, the only P5 behavior change is `ListSource<T>` storage: a private copied `List<T>` was replaced with `Array<T>` plus a retained `Slice<T>`. Its fixture now appends, replaces, and removes elements from the caller list and checks source size, cursor reads, `Parse.run`, and ordered `Parse.runAll`. The parse README describes the Array/Slice ownership model.

P5 does not modify `TextSource.slice`, the String/Substring API, `Object.>=`, `<=>` dispatch, or operator-fixture expectations. The working tree contains other P5 Slice/Buffer implementation work; it is outside this incident's causal claim.

## 9. Evidence

- The P5 checkpoint records both parse-package tests failing at G0 before production changes, with the String/Substring mismatch and the separate operator comparison failure.
- After T7, the same named command again selected two tests and both failed with those same categories of errors.
- `TextSource.slice` source declares `String` and returns `_text.slice(start, end)`.
- `ListSource<T>` now snapshots into `Array<T>` and retains `backing.slice(..)`.
- The changed `source-test.ph` has no remaining diagnostic of its own in the package run, but the import-level failure prevents runtime certification.
- Workspace all-target `cargo check`, native-surface check, and `git diff --check` passed; none certifies the blocked parse-package runtime gate.
- Workspace tests, clippy, and release certification have not been run for P5.

## 10. Implementer's causal hypothesis

The package gate is blocked by two inherited parse/runtime issues outside the T7 ListSource change. The String/Substring diagnostic is a source-level API/annotation mismatch in `TextSource`. The operator failure is a separate runtime comparison/type-dispatch issue involving `Some` and `Int`. Available evidence does not establish the underlying cause or owning plan for either issue.

## 11. Alternative hypotheses

- The `TextSource.slice` mismatch may reflect an intentional `Substring` return contract whose consumer annotation or migration is incomplete; changing either side has API/spec implications that should be decided by its owner.
- The operator failure may arise from the value returned by `text.bytes.at(index)`, the selected comparison implementation, or another runtime representation/dispatch issue. The present evidence does not distinguish these causes.
- Although both failures predate P5, a new source interaction could theoretically expose them differently. The repeated same diagnostics and unchanged blocker sites provide no evidence of a P5-caused regression.

## 12. Attempts already made

- At G0, the complete parse-package command was run before P5 production edits. Both tests failed with the recorded baseline issues.
- After T7, the same complete command was rerun. It again selected two tests and failed on the same String/Substring mismatch and `Some`/`Int` operator failure.
- No attempt was made to alter `TextSource.slice`, comparison semantics, or fixture expectations, because those are outside T7's assigned scope and no causal fix has been established.

## 13. Why local authority is insufficient

The implementer may not silently waive T7.4 or treat a single source fixture as full parser convergence; the plan explicitly requires the complete parse-package gate. Conversely, repairing the inherited failures would change a separate parse/text or runtime-comparison surface without a scoped owner decision. The remaining question is whether to keep G6 blocked until those owning issues are fixed, or formally amend the P5 acceptance boundary while preserving the inherited package failure as a checkpoint residual.

## 14. Decision requested

Please advise which disposition is correct:

1. **Keep G6 blocked:** P5 may continue other work, but G6 stays `BASELINE_BLOCKED` until the owners of both parse-package failures repair them; then rerun the complete named gate.
2. **Amend P5 acceptance:** define the minimum independent, non-circular validation that proves the ListSource snapshot contract, permit that evidence to close the P5-owned behavior, and retain the full parse-package command as an inherited blocker outside P5. If selected, specify the required plan/checkpoint amendment and whether G6 itself may be marked complete.

Please also identify the owning work item/team for (a) the `TextSource.slice` String/Substring mismatch and (b) the `Some`/`Int` comparison failure, if known. No implementation fix is requested in this consultation.

## Current disposition — 2026-10-18

The earlier parse adapter and fixture issues were corrected, and the full
`modules_parse_package` gate now passes 4/4 Rust integration checks and 13/13
runtime fixtures. G6.A, G6.B, and aggregate G6 are PASS. This current result
supersedes the consultation's original blocked-state recommendation; the
checkpoint and P5 walkthrough record the evidence and the later diagnosis of
the token-constructor failure.
