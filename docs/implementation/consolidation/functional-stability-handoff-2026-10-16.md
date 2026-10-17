# Consolidation Universe functional-stability handoff

Continue from this handoff. The original IMPLEMENTATION DIRECTIVE —
Consolidation Universe Functional Stability remains authoritative; attach it
alongside this file when delegating. The user switched this session to handoff
preparation so a cheaper agent can perform routine implementation and validation,
with complex work delegated back to the supervising agent.

## Mission and boundaries

Work in `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom`,
branch `feat/consolidation-universe`. Deliver reliable standalone Telos analysis,
compilation, finalized linkage, and VM execution. Continue P4 under its approved
architecture. Implement verified corrections in bounded slices, then commit and
push each completed slice before proceeding.

Preserve immutable finalized read slots, exact binding/product/stage proofs,
strict runtime-cycle rejection, canonical semantic ownership, and the difference
between native-floor and fully initialized Universe availability. Do not add late
compiler reads, relax cycle checks, duplicate semantic authority, integrate
PERF003/artifact caching, or perform production Universe cutover.

The implementation priority remains:

1. Protect the baseline and classify reproduced failures.
2. Complete P4 mode consistency, generated requirements, CV-01–CV-09, and
   cold/incremental equivalence.
3. Repair supported-feature defects, beginning with the demonstrated Horner
   stack overflow and expression-carrier regression, then inference/dispatch,
   collections/iteration, imports, and incremental products.
4. Run broader gates when coherent slices justify them. Assert actual VM values
   in representative multi-module programs.

Use routine mechanical judgment. Bring complex root-cause or architectural
questions back to the supervising agent with a reproducer, causal evidence, the
required invariant, and a precise decision request. Do not change semantic test
oracles merely to reduce the failure count.

## Repository state at handoff

- HEAD: `2b0e41ce85c9a2d434d52d8e6aefa20641499c4d`.
- The worktree was clean at entry and matched the audited baseline. Before this
  handoff document, all 11 modified files belonged to this session's mode slice.
- HEAD and upstream had zero commits of divergence when checked.
- No commits or pushes were made. Preserve the uncommitted work below.
- P4 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED` for its previously
  established scope. The new mode slice has partial verification; do not mark
  it complete or claim branch stability.
- No subagents were used. No source-level stack repair or contract-error repair
  has been applied.

## Minimal first reads

Read repository `AGENTS.md` and the relevant lifecycle guidance, then these
existing records under the repository root:

- `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/docs/implementation/MODL001-module-architecture/C2-module-project-model/MODL001.C2.P4-handoff.md`
- `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/docs/implementation/MODL001-module-architecture/C2-module-project-model/CHECKPOINT.md`
- `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/docs/implementation/MODL001-module-architecture/C2-module-project-model/MODL001.C2.P4-phase-aware-source-dependencies.md`
- `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/docs/implementation/consolidation/integration-universe-consolidation.md`
- `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md`

Consume the established invariants and prior evidence. Do not repeat the
historical bootstrap investigation or rebuild a code index.

## Implemented, uncommitted mode slice

Observed gap: `ProgramAnalyzer` projected requirements for the selected mode,
but `AnalyzedProgram` and `CompiledProgram` did not retain that mode. The VM
could lower and execute a finalized layout using a different guard policy.

The patch carries a private-to-the-crate `CompileMode` through both products,
exposes `CompiledProgram::compile_mode()`, and rejects a mismatched VM before
materialization, module closure compilation, or initialization. It also passes
the VM's selected mode through the interpreter and module-context entry paths.
Canonical bootstrap remains explicitly Debug.

The retained fixture session and corpus runner had a related mismatch: fixtures
were compiled through a Debug-only session, while the VM subsequently received
Release or Unchecked. The session API now accepts the selected mode and threads that
mode through semantic projection and the compiled product. Its existing
conservative compatibility finalizer remains unchanged. This does not prove
that compatibility finalizer covers all generated reads; do not infer P4 closure.

Modified source map, with paths relative to the absolute repository above:

| File | Change |
| --- | --- |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/src/modules/compile.rs` | Mode fields, getter, all three analyzer constructors, compiled projection; semantic mode conversion made visible to sibling modules. |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/src/modules/materialize.rs` | `VM::validate_program_compile_mode`; checks before allocation and closure compilation. |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/src/modules/initialize.rs` | Checks both public initialization entry points. |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/src/interpret.rs` | `Interpreter::run_entry` compiles for `self.vm.compile_mode`. |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/src/modules/context.rs` | User-module compilation passes `vm.compile_mode`. |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/src/modules/canonical_universe.rs` | Canonical analyzed product explicitly records Debug. |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/src/modules/compile_session.rs` | `compile_fixture_with_mode`, Debug compatibility wrapper, semantic projection plumbing, retained-transition regression. |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-semantic/src/session.rs` | `update_module_workspace_with_execution_mode_and_cancel`; existing method retains Debug default. |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/tests/support/mod.rs` | Corpus passes fixture flags into the production session API. |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/tests/core/modules/linking.rs` | New matching/mismatching mode regression with a two-module project and VM result 42. |
| `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-semantic/tests/semantic/integration/phase_evidence.rs` | Existing mode test now checks retained Debug → Release → Unchecked → Debug transitions against cold phase products, generated requirements, and finalized reads. |

The new core regression is
`modules_linking::finalized_compile_modes_reject_mismatched_vm_before_mutation_and_execute_matching_guards`.
It checks all six mismatches, rejection without entry-module allocation,
rejection after a VM mode change between materialization and lowering or
initialization, and matching-mode execution in all three modes. Its provider is
used only by an ensures predicate; its linked read must be present only in Debug.

The new library regression is
`modules::compile_session::tests::retained_fixture_mode_transitions_preserve_the_selected_execution_policy`.
Inspect the selected full test name if an exact filter needs adjustment.

## Completed validation

All Cargo commands used the pinned toolchain and this environment prefix:

```sh
CARGO_TARGET_DIR=/tmp/phalcom-consolidation-check-20261016 RUSTFLAGS='' RUSTC_WRAPPER=''
```

Apply that prefix to the commands below. Run Cargo validation serially.

### Audited baseline inventory

```sh
cargo test -p phalcom-semantic --test semantic -- --skip ordinary_float_horner_expression_is_stack_safe_without_realization --quiet
```

Result: 2,131 passed, 94 failed, 23 ignored, 1 filtered out, exit 101,
978.92 seconds. Raw output:
`/tmp/consolidation-semantic-baseline.log`.

This binary was built before this session's semantic source edits. All 94
failing function names occur in the existing coverage ledger. The pass count is eight higher than the older 2,123-pass record; the
current phase suite has 20 tests compared with the earlier 12.
The ignored tests were not run. The filtered stack test was not rerun here.

The carrier regression failed with 1,072 bytes, against the unchanged
744-byte baseline plus a two-pointer allowance. The older ledger's 896-byte
measurement is stale on this audited HEAD; preserve the assertion.

### Phase-evidence suite

Before edits: 20 passed, 15.93 seconds.
Log: `/tmp/consolidation-phase-baseline.log`.

After the mode changes, including retained-mode transitions:

```sh
cargo test -p phalcom-semantic --test semantic integration::phase_evidence -- --nocapture
```

Result: 20 passed, 0 failed, 23.22 seconds, exit 0.
Log: `/tmp/consolidation-phase-mode.log`.

Changed Rust files were formatted individually. `git diff --check` passed at
handoff preparation. No workspace formatting, Clippy, build/test certification,
or post-change full semantic suite was run.

## Running validation: inspect before launching another Cargo command

```sh
cargo test -p phalcom-core --test core modules_linking::finalized_compile_modes_reject_mismatched_vm_before_mutation_and_execute_matching_guards -- --exact --nocapture
```

Log: `/tmp/consolidation-core-mode.log`.
The build finished successfully in 13.05 seconds and selected one test.
At handoff preparation the test was still running; no execution PASS is
claimed. The original tool session ID is `50991`, if accessible. A new agent
can inspect the log and process state instead. Avoid a duplicate test run while
this one is active.

## Next routine work

1. Inspect the running core test's result and current diff. Diagnose any failure
   before changing code. The semantic mode-transition test already passed.
2. Run the new retained-session library test, confirming a nonzero selection:

   ```sh
   cargo test -p phalcom-core --lib retained_fixture_mode_transitions_preserve_the_selected_execution_policy -- --nocapture
   ```

3. Run directly affected module-linking regressions. The full
   `cargo test -p phalcom-core --test core modules_linking -- --nocapture`
   includes canonical bootstrap, ordinary collection execution, type/conformance
   fixtures, strict cycle negatives, and finalized-read checks. Classify failures
   against existing evidence; a previous pass is not a current-tree pass.
4. Update the P4 checkpoint, handoff/walkthrough, consolidation record, and
   coverage ledger with completed results. These existing records have not
   been updated by this session. Apply repository Sloptrim guidance to prose.
5. Review and commit only this coherent mode slice, including its tests and
   relevant records, then push to `origin/feat/consolidation-universe`. The
   original directive authorizes commit/push. Do not commit unverified changes
   as a completed correction.
6. Continue the prioritized backlog. Return complex investigations below to the
   supervising agent with focused evidence rather than applying speculative fixes.

## Focused findings for supervising-agent follow-up

### Horner stack and recursive carrier

The existing test is
`semantic::capabilities::algebraic_realization::ordinary_float_horner_expression_is_stack_safe_without_realization`.
Its historical 2 MiB analysis-thread overflow remains unresolved. Do not increase
that stack or rewrite its polynomial. Relevant source:
`/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-semantic/src/checker/{expression.rs,typed_expr.rs}`.

LLDB disassembly of the baseline unoptimized binary measured
`analyze_expression_with_publication` at 13,568 bytes of stack allocation
(including the saved-register area). The earlier incident record documented an
8,000-byte formal expression frame. Large post-descent publication/explanation
and error temporaries now occupy recursive frames.

Candidate repair, not applied or verified: keep the optional solver-local
`SymbolicInferenceResult` indirect and separate post-descent publication work
from the live recursive descent frame. Preserve single ownership, publication
dependencies, evidence, and diagnostics. A draft transformation exists at
`/tmp/consolidation-carrier-slice.py`; it is scratch work, not an approved plan.
Do not execute it blindly. Have the supervising agent review the approach, then
verify the unchanged carrier budget and exact 2 MiB regression. Also compile
and execute a representative Horner program and assert its numeric result.

### Contract-error generated reads

Source inspection found that
`/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/src/compiler/attributes.rs::build_check_stmt` synthesizes ordinary
variable reads of `PreconditionError`, `PostconditionError`, and `InvariantError`.
`/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-semantic/src/phase_evidence.rs::push_compiler_generated_read_requirement`
currently recognizes Error and ZipLengthMismatchError, not those contract classes.
Their source declarations live in
`/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-core/core/universe/src/errors/contracts.ph`.

This is a source-backed suspected correctness gap, not an executed reproducer.
The mode fixture's predicate returns true, so it does not prove the failure
branch. Reproduce with a false contract predicate and inspect the actual error
class/message plus emitted canonical reads. A string-named GetGlobal may escape
an audit restricted to GetLinked instructions. Do not fix this by making private
classes globally visible, by adding late reads, or by inventing compiler-side
identity authority. Route a required canonical-lowering design decision back.

### Incremental disposition/reporting mismatch

Several failures stop on `stats.callables_recomputed == 0`, even though query
work counters report recomputed formal bodies. Example:
`export_invalidation::test_previously_missing_public_name_appears_invalidates_consumer`
reported `callables_recomputed: 0` and `callable_bodies_recomputed: 1`.
`case_a_unchanged_caller_changed_callee_signature_recomputes` and case B fail at
initial expected counts before reaching their later behavioral assertions.

Inspect `/Users/altunhasanli/.codex/worktrees/universe-consolidation/phalcom/phalcom-semantic/src/session.rs` at the `callable_dispositions`
collection, source-index rebuild frontier, and final stats/recomputed-key
publication. The database already exposes `revision_recomputed_keys()` and
`revision_revalidated_keys()`. Hypothesis: some formal-body paths fail to update
the disposition map; this may affect both reporting and source-index refresh.
It is not yet established. Fix the owning inconsistency if proved; do not
change expected counts just to bypass the behavioral checks that follow.

### Other baseline classifications remain open

- A trait/default test panics because the canonical fixture cannot find
  `universe:random::SourceSequence`.
- Native conformance reports Pack native-marker, superclass, and arity
  mismatches. Check current structural-product/native-floor authority before
  deciding whether source, metadata, or the oracle is wrong.
- Iteration tests publish Unknown(DynamicMessageSend) where protocol-derived
  element types are expected.
- A diagnostic cold/incremental comparison shows duplicated ReturnMismatch
  diagnostics on one side; investigate aggregation ownership.
- The remaining 94-test inventory has not been semantically classified in full.
  It must not be described as unrelated or unsupported by default.

## Do not re-explore or overclaim

The baseline inventory and the passing 20-test phase suite are established.
Do not rerun the full baseline merely to begin a new chat. Do not reopen the
historical metadata-export issue unless it reproduces on an active gate.
Existing bootstrap availability, canonical None preservation, strict DAG
validation, and finalized read ownership remain the starting contract.

No slice has been committed. No runtime result from the pending mode test is
claimed. P4 and full functional-stability acceptance remain incomplete.

## Continuation update — 2026-10-17

The mode slice was committed and pushed as `324ee315`; the worktree was clean
when this continuation began. Its completed results are recorded in the P4
checkpoint: module linking 20/20, retained fixture mode transitions 1/1,
semantic phase evidence 20/20.

The supervising agent resumed implementation. The Horner and carrier findings
above were repaired and pushed as `0665588e`: keep exceptional query keys and
optional symbolic results indirect, separate post-descent semantic publication,
and dispatch binary lowering outside the large compiler expression match.
The unchanged semantic algebraic suite passes 9/9; query ownership 7/7;
publication rollback/retry 13/13. The production Horner package executes and
asserts `2080.0` (1/1, 128.88s). The unchanged package test uses an 8 MiB stack;
its first execution exposed a separate compiler-lowering overflow that the
binary-dispatch extraction repairs. See the consolidation record for final
verification and delivery. The scratch transformation is obsolete.

The generated contract-error and incremental disposition findings remain open.
P4 and branch-wide stability acceptance remain incomplete.

### Retained executable finalization — 2026-10-17

Retained fixture compilation now supplies the canonical symbolic link to the
existing semantic session, then shares fresh compilation's availability-aware
finalizer. Generated reads and runtime metadata roots receive immutable slots
before lowering. The change preserves transactional deltas, canonical ownership,
strict runtime-cycle rejection, and exact full-Universe availability proofs.
It removes the conservative fixture wrapper; it does not migrate the production
Universe or add late compiler reads.

Verification: five existing retained-session tests passed (131.67s), and the new
canonical-read add/remove/restore versus cold regression passed (99.59s).
Publication rollback/retry passed 13/13 (8.02s); semantic phase evidence passed
20/20 (16.20s). The Boolean corpus passed all 12 fixtures; the arithmetic corpus
passed all 33 fixtures after adding the missing explicit `Ellipsis` import to
one fixture. Its expected output and runtime assertions are unchanged. The
arithmetic rerun passed in 108.05s; the earlier combined run proved the Boolean
group and exposed that invalid fixture. See the consolidation record for commands.

P4 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. The next priorities are
implicit contract-error dependencies, remaining attribute/witness forms,
conformance-cycle execution, and incremental invalidation failures. The full
semantic failure inventory and workspace test acceptance have not been recertified.
