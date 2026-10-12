# INC-001 — Workspace baseline is blocked by parser and stale snapshots

Finding ID: `INC-001`  
Severity: P2 (baseline/release evidence blocker; no confirmed type-system defect)  
Confidence: High  
Status: Baseline / unrelated to active type-system audit implementation  

Subsystem: `phalcom-ast` parser and snapshot test fixtures  
Owning file: `phalcom-ast/tests/parser.rs`; AST snapshot fixtures  
Owning function: `parser::record_pattern_remainder_is_terminal_and_map_patterns_are_explicit`  
Affected consumers: Workspace test certification and parser/type-syntax integration evidence  

## Specification

The audit baseline procedure requires `cargo test --workspace` to establish
the repository's starting test state. Parser syntax behavior is governed by the
current `docs/specs/` corpus and parser tests; this finding does not assert a
specification violation without a dedicated syntax audit.

## Expected behavior

The workspace baseline should complete without failures, or each failure should
be classified before type-system results are interpreted.

## Actual behavior

`cargo test --workspace` completed with 300 passed, 7 failed, and 1 ignored.
One parser test failed with:

```text
SyntaxError { kind: UnrecognizedToken { token: "name", expected: ["\"#\""] }, range: 65..69 }
```

Six snapshot tests failed because actual AST output contains
`is_primitive: false`, absent from their checked-in snapshots. The test run
wrote six `.snap.new` files.

## Minimal reproducer

For the parser behavior, the existing test named above is the minimal
reproducer. For the snapshot state, run the six affected tests in
`phalcom-ast/tests/parser.rs` or rerun the workspace test.

## Reproduction command

```bash
cargo test --workspace
```

## Observed output

```text
test result: FAILED. 300 passed; 7 failed; 1 ignored
```

## Expected output

Baseline classification should be explicit, with no unclassified workspace
failures.

## Root cause

The snapshot failures are expected-output drift from the current AST shape. The
record-pattern failure has not been root-caused during this baseline step and
must not be silently attributed to the type checker.

## Violated invariant

No type-system invariant is established as violated. The affected audit
invariant is evidence discipline: workspace certification cannot be claimed
while these baseline failures remain unexplained.

## Failure classification

`E — baseline evidence blocker / unclassified parser behavior`, not a confirmed
type-system soundness, completeness, compiler-realization, robustness, or
semantic-consistency defect.

## Affected feature combinations

Parser record patterns; AST snapshot serialization. No type-system combination
has been established.

## Related historical incidents

None identified at baseline.

## Recommended correction layer

Outside this audit's authorized scope. A parser owner should separately decide
whether to update snapshots and investigate the record-pattern grammar.

## Required regression tests

Outside this audit. If the parser issue is confirmed, retain a minimal parser
regression and update affected snapshots through the normal AST workflow.

## Potential collateral effects

Until classified, parser integration failures reduce confidence in source-level
type-system reproductions that depend on these syntax forms. They do not imply
that already-parsed semantic tests are invalid.

## Verification status

Baseline reproduced once by `cargo test --workspace` on the reference revision.
Workspace certification: `BASELINE_BLOCKED`.
