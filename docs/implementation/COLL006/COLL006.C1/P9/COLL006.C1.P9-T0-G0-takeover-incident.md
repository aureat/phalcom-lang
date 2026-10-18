# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `COLL006`
- Checkpoint: `COLL006.C1`
- Plan: `COLL006.C1.P9`
- Task/Gate: T0 / G0
- Repository: `aureat/phalcom-lang`
- Branch: `feat/telos-collections`
- Starting revision: `06017339c36d4f8b06016b1d89aa9d2996673de9`
- Revision at incident creation: same as starting revision; subsequent P8 continuation is recorded in section 15.

## 2. Escalation trigger

The P9 hard predecessor, checkpoint status, and P8 evidence materially conflict. P9 requires a completed, focused-tested P8 with its GC/callback/randomized/complexity gates passed and forbids P9 production work before G0. The P8 checkpoint still says its verification gates remain to be run, and the P8 handoff does not record the required G7 evidence or residual list. This triggers the repository escalation rules for conflicting plan/checkpoint/current evidence and materially expanded scope.

## 3. Required invariant

Preserve P9-INV-01 and P9-INV-15: do not redesign P8's source-owned immutable persistent AVL or weaken its GC/persistence guarantees. Do not begin P9 production work before the takeover gate is resolved.

## 4. Expected architecture

P8 Route A is a source-owned `Interval<T>` kernel and immutable persistent AVL `RangeMap<K,V>` with no native-surface increase. P9 may converge the public association presentation, but it must not alter P8 interval semantics, overlap invariants, persistence, or complexity architecture.

## 5. Observed behavior

Historical scope: sections 5–14 record the initial 2026-10-12 takeover state. Section 15 records the later P8 continuation and supersedes their old current-status and next-action statements.


The target branch initially failed to parse `_RangeMapNode` and then failed Universe bootstrap on duplicate `RangeMap.new()`. Both defects were corrected mechanically: the private class was renamed `RangeMapNode`, and the duplicate `@class new()` wrapper was removed. The focused `core_range_map::range_map_construction_and_lookup` test now passes.

The required `modules_parse_package` command on the corrected branch fails because three parser fixtures encounter the same `ListSource` immutable-field diagnostic also seen on current main. The associated Cursor assertion is not reached. Only 9 of 12 runtime fixtures pass on the feature branch. P8's required G7 evidence is not present in the current test files or handoff, and no P8 residual ledger is available.

## 6. Minimal reproduction

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package
```

Feature-branch result after the two P8 corrections: 0/4 Rust tests pass; 9/12 runtime fixtures pass; the three failing fixtures report `_items` already initialized in `packages/parse/src/source/list-source.ph:33`.

Focused RangeMap check:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core core_range_map::range_map_construction_and_lookup -- --nocapture
```

Result: 1 passed.

## 7. Relevant code path

```text
modules_parse_package fixtures
  -> Phalcom CLI / canonical Universe bootstrap
  -> universe:collections.range_map
  -> RangeMap source declarations
```

The masked P5 residual is in `packages/parse/tests/runtime/src/core/source-test.ph` and the associated invocation fallback in `phalcom-core/src/compiler/lib/associated.rs`. The current observed fixture failure occurs earlier in `packages/parse/src/source/list-source.ph`.

## 8. Scoped implementation diff

Only `phalcom-core/core/universe/src/collections/range_map.ph` changed in source: rename the private node class and its references; remove one duplicate class-side factory. No fields became mutable, no new native binding was added, and P9 API work has not begun.

## 9. Evidence

- The corrected RangeMap construction/lookup test passed.
- The native surface generator passed with 488 declarations.
- The `Value` 16-byte and `Object` 48-byte tests passed.
- Current main reproduces the `ListSource` diagnostic; its parser-package run additionally fails on legacy metadata export.
- P8 G7 requires multi-version GC, endpoint/value reachability, callback allocation/GC, a randomized oracle, and lookup/overlay complexity evidence; these results are not recorded in the P8 handoff or test files.

## 10. Implementer's causal hypothesis

The two RangeMap bootstrap errors were local P8 implementation defects and are resolved by the minimal changes above. The remaining P9 blocker is a plan-order/evidence gap: P9 cannot pass G0 while its hard predecessor's required G7 proof is missing.

## 11. Alternative hypotheses

- Required P8 G7 evidence may exist in an unrecorded external run; no such evidence appears in the branch records inspected.
- The principal architect may intend P9 T8/T9 to take ownership of the missing P8 property/GC/performance evidence, which would require a material plan amendment.

## 12. Attempts already made

- Earlier uncommitted attempts changed node mutability and removed the duplicate wrapper together. Those edits were discarded before revalidation.
- The parser test isolated the invalid class name and duplicate selector. Only the two evidence-backed mechanical corrections were retained.
- Current RangeMap construction/lookup passed after those corrections.

## 13. Why local authority is insufficient

Choosing between reopening P8 to complete its G7 requirements and amending P9 to absorb those requirements changes the plan dependency and acceptance order. The executor cannot waive P9's explicit G0 hard predecessor.

## 14. Decision and disposition

**Decision (2026-10-12):** resume P8 at T7/G7, then complete G8 and its current
residual handoff. Do not amend P9. P9 §3 and G0 explicitly require P8's own
focused GC/callback/randomized/complexity evidence; P9 T8–T10 cover only part
of that surface and rely on P8's canonical model and seeds. The missing P8
evidence therefore remains a P8-owned prerequisite.

The recommendation came from a read-only `phalcom-architect` consultation
against this incident packet and the live P8/P9 plans. P9 production work
remains stopped at G0. The checkpoint records the ruling under
`COLL006.C1.A12`; active work returns to P8 T7/G7.

## 15. Current predecessor status — 2026-10-13

The prescribed P8 split/join/range-splice implementation and T7/G7 evidence
are now present. T6 normalization is implemented at mutation boundaries, and
the focused RangeMap filter passes 15/15. T8/G8 remains incomplete: the data
regression is 30/32, `memory_gc` is 13/19, the exact numeric Range generic
spread regression fails, and the workspace formatting check reports broad
diffs. These residuals remain ambiguous because no equivalent P8 baseline was
available; the C1 checkpoint and P9 verification/residual ledgers contain the
exact evidence. P8 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`, P9 T0 is
partial, and G0 remains blocked. P9 production implementation has not started.
The next action is to classify or resolve the P8 T8 residuals, pass G8, and
complete P8 T9 lifecycle records before resuming P9 G0.

## Parser-package finding closure — 2026-10-18

The parser-package observations above are historical. The current
`modules_parse_package` gate passes 4/4 Rust integration checks and 13/13
runtime fixtures. The ListSource immutable-field failure is resolved. The
token-backed Cursor issue was missing contextual owner evidence for
`Some(...)`; explicit `Option<Token>::Some(...)` uses the existing static
variant path. See the P5 G6 closeout and P9 residual/verification ledgers for
the current disposition. These results close the P5 package residual but do
not alter the P8 predecessor gate: P9 G0 remains blocked until P8 G8 and T9
closeout are complete.
