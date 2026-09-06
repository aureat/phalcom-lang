# AUD-RUNTIME-001 — Unchecked operand narrowing silently changes executable meaning

## Classification

- Severity: High
- Category: Correctness, Bytecode, Compiler-Runtime Contract
- Confidence: Confirmed for constants; high confidence for unbounded local and field indices; other widths need targeted verification
- Priority: Fix immediately
- Runtime impact: Large source units can execute a different constant; oversized layouts can alias slots or fail.
- Affected components: Chunk construction, compiler constant/local/field emission, VM operand consumers

## Executive Finding

The compiler successfully compiles 65,537 integer expressions, but the final expression `65536` executes as `0`. Constant index 65,536 wraps to zero. This is source-reachable incorrect execution, not merely malformed-bytecode behavior.

## Observed Architecture

Instructions are Rust enum values with `u16` constant/slot indices. The constant vector itself has a `usize` length. Several newer semantic side tables already reject overflow, but the ordinary constant pool does not.

## Evidence

- `phalcom-core/src/chunk.rs:207`, `Chunk::add_constant`: appends a value and returns `(constants.len() - 1) as u16`.
- `phalcom-core/src/compiler/lib/scope.rs:31`: compiler forwards to that infallible API.
- `phalcom-core/src/compiler/lib/expr.rs:709`: integer literal emission consumes that index.
- `phalcom-core/src/vm/dispatch.rs:1183`: `Constant` indexes the pool using the narrowed operand.
- `phalcom-core/src/compiler/lib/scope.rs:146`, `add_local`: increments `usize` local counts without an operand limit; numerous `GetLocal`, `SetLocal`, scratch and upvalue emissions narrow to `u16`.
- `phalcom-core/src/compiler/lib/class_decl.rs:851`: inherited field offsets narrow to `u16`; total counts use unchecked `u16` addition at lines 854 and 860.
- `phalcom-core/src/compiler/lib/jumps.rs:55,73`: offsets use `usize as i32`, then arithmetic.
- `phalcom-core/src/compiler/lib/mod.rs:238,246`: linked-read emission includes narrowing and a saturating fallback. Reachability of overflow through production linking remains unverified.
- Positive controls: `ExecutableSemanticPool::add_*` uses `u16::try_from`; `compiler/lib/error.rs:372` checks send arity; `expr.rs:969` checks list length.
- Executed evidence: [probe source](evidence/runtime_audit_probe.rs), [output](evidence/probe-output.txt).

## Runtime / Compilation Path

Integer expression → `compile_expr` → `add_constant` → `Constant(u16)` → vector lookup → wrong integer pushed → wrong result returned.

## Invariant

Every emitted operand must denote the same entity as the producer's original index. An index outside the encoding domain must produce a compilation error before publication.

## Current Behavior

Probe results:

| Original constant index | Encoded index | Selected integer |
| --- | --- | --- |
| 0 | 0 | 0 |
| 1 | 1 | 1 |
| 65,534 | 65,534 | 65,534 |
| 65,535 | 65,535 | 65,535 |
| 65,536 | 0 | 0 |

Compiling the source sequence `0\n1\n...65536\n` yielded `constants=65537`, `last_index=0`, and `SOURCE execution=Ok(0)`.

## Failure Scenario

A generated module or sufficiently large method adds enough constants. Subsequent values can become earlier integers, strings, object handles, or selectors. A wrong selector constant can additionally trip a VM type assumption. Only the integer wrong-result case was executed during this audit.

## Root Cause

Encoding limits are enforced at selected syntax sites rather than at the shared construction boundary. The physical vector can grow beyond what its bytecode references can address.

## Impact

### Correctness

Silent wrong results; possible incorrect dispatch or controlled/uncontrolled errors depending on the aliased constant. Field total overflow may panic in debug or wrap in release; oversized local indices can select unrelated slots.

### Architecture

The bytecode's representability invariant has no single owner. A verifier cannot reconstruct the intended index once truncation has occurred.

## Recommended Direction

Make constant insertion fallible and propagate a span-bearing compilation error. Centralize checked conversion for every encoded index/count, including compiler-generated locals and inherited field totals. Check arithmetic before narrowing. Preserve the compact instruction representation unless realistic workloads justify wider operands.

## Alternative Solutions

- Wider operands increase capacity but merely move the defect without checked conversion.
- Constant deduplication reduces pressure but does not establish correctness.
- VM bounds checks cannot detect a wrapped index that validly addresses the wrong entry.

## Implementation Outline

1. Introduce checked pool insertion and compiler error propagation.
2. Bound locals/upvalues and both instance/static inherited layouts at their producers.
3. Check signed jump differences using a wider intermediate.
4. Reconcile linked-read limits with materialization before removing or replacing fallback behavior.

## Required Tests

Boundary tests at 0, 1, maximum minus one, maximum and maximum plus one for each operand family. Retain the source-generated constant reproducer. Cover inherited field totals, scratch locals introduced by packs/matches, selectors added after many literals, and jump arithmetic without allocating a multi-billion-instruction chunk. Existing 255/256 argument tests are useful controls, not evidence for other widths.

## Verification Criteria

No successful compilation can publish an unrepresentable index. The reproducer either returns the intended value with an explicitly wider format or fails deterministically before execution. No wrapped aliasing or debug/release divergence.

## Related Findings

[005 — grouped contract and performance concerns](AUD-RUNTIME-005-grouped-contract-and-performance-findings.md).

## Open Questions

Production reachability of oversized linked-read and semantic-pool indices; desired product limit for large generated source. Full local/field boundary reproducers remain to be executed.
