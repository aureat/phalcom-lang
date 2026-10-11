# IMPLEMENTATION INCIDENT

> **Current status (2026-10-11):** Both the stack overflow and canonical
> runtime bootstrap blocker are resolved and verified. The carrier/frame
> regression and canonical Universe bootstrap integration test pass cleanly.
> See §16 and §17 for measurements, root cause analysis, and verification evidence.

## 1. Identity

- Program: `STDL001` — numeric library; cross-cutting source compilation failure
- Checkpoints: `STDL001.C6`, `STDL001.C7`, `STDL001.C8`; source acceptance for the `LANG007.C4` algebraic-literals follow-on is also affected
- Plans: `STDL001.C6.P1`, `STDL001.C7.P1`, `STDL001.C8.P1`; `LANG007.C4.algebraic-literals-follow-on`
- Task/Gate: focused Phalcom source/package compile-and-run acceptance
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `feat/complex-math`
- Starting revision: `fdbc2c41`
- Current revision at observation: `fdbc2c41` (incident reproduced in the uncommitted worktree)
- Incident date: 2026-10-11

## 2. Escalation trigger

RED trigger 15 fired: compiler/semantic behavior is nonlocal and its owner is
uncertain. Trigger 18 also applies: source acceptance is blocked by a failure
that persists after removing the active package dependencies and spans beyond
the planned math implementation.

## 3. Required invariant

A valid, dependency-free project containing a trivial integer binding must
compile and run without aborting the process. A recursive compiler or semantic
failure must be reported at its owning layer; a stack overflow must not be
treated as a normal source diagnostic or worked around by weakening semantic
authority, generic evidence, or canonical type identity.

## 4. Expected architecture

The CLI should resolve and compile the selected project against the canonical
Universe, consuming semantic products through the existing compiler boundary.
The project source and optional-package graph must not make unrelated compiler
or Universe work recurse without bound. The relevant owner—Universe loading,
semantic analysis, lowering, or VM execution—has not yet been identified.

## 5. Observed behavior

Running the CLI on a minimal project with no declared dependencies and one
exported `Int` binding aborts with a native stack overflow. The same result
persists with the shell stack limit raised to 8192 KiB. The failure does not
identify whether it occurs during Universe bootstrap, project resolution,
semantic analysis, lowering, or execution; no phase marker or stack trace was
captured. Attribution to a particular feature or layer would be inference.

## 6. Minimal reproduction

The temporary project used for the reproduction had this manifest:

```toml
[project]
name = "math_acceptance"
namespace = "math_acceptance"
version = "0.1.0"
source = "src"
entry = "math_acceptance.main"
```

Its `src/main.ph` contained only:

```phalcom
let value = 1
export value
```

For the final minimal reproduction, the CLI was freshly rebuilt while generic
Complex division and the `MathReductions.meanList` facade were temporarily
disabled. The same failure persisted; both source changes were restored after
the run. The command was:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core --bin phalcom
(ulimit -s 8192; target/debug/phalcom /var/folders/jf/p7qvhf1s29b2fdxg3hl8l7z00000gn/T/phalcom-math-package-check-vrjvlckf)
```

Observed output:

```text
thread 'main' (9406906) has overflowed its stack
fatal runtime error: stack overflow, aborting
```

The CLI exited with status 134. The temporary directory is ephemeral; the
manifest and source above are the complete reproduction input.

Expected behavior: the dependency-free project completes successfully.

## 7. Relevant code path

The exact failing frame is unknown. The normal candidate path is:

```text
phalcom-core/bin/phalcom/main.rs::main
  -> phalcom-core/bin/phalcom/cli.rs::cmd_run
  -> project/Universe loading and module compilation
  -> semantic analysis and compiler lowering
  -> VM execution, if compilation completes
```

Relevant owning code to instrument after the layer is established includes
`phalcom-core/src/modules/compile.rs`,
`phalcom-core/src/modules/semantic_lowering.rs`, the semantic checker, and the
Universe source loader. This list is a search boundary, not a claim that one of
these files contains the recursion.

## 8. Scoped implementation diff

The worktree contains the authorized math source and lifecycle changes, plus
cross-layer class-side trait and contextual algebraic-literal work needed by
that implementation. This incident produced no stack-overflow fix. During
isolation, `ComplexDivision` and the `MathReductions.meanList` facade were
temporarily removed; the no-dependency reproduction still overflowed. Those
temporary removals have been restored so the implementation worktree remains
coherent. The package APIs and their behavior remain unverified.

## 9. Evidence

- The CLI was freshly built from the current source with
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core --bin phalcom`.
- A statistics-package project first produced source diagnostics. After those
  local issues were corrected, subsequent CLI invocations aborted with the
  stack overflow above.
- Removing the probability dependency did not change the outcome.
- Replacing the entry source with the trivial binding above and declaring no
  package dependencies did not change the outcome.
- Raising the shell stack limit to 8192 KiB did not change the outcome.
- Temporarily removing `ComplexDivision` and `Complex<T>` division, and
  separately removing `MathReductions.meanList`, did not change the outcome in
  the minimal project.
- These observations show the failure is not dependent on the probability
  dependency or those two math additions. They do not identify its cause.

## 10. Implementer's causal hypothesis

The leading hypothesis is recursive re-entry or unexpectedly deep traversal in
an automatically loaded compiler/semantic path, likely reached while loading
the canonical Universe or resolving its generic declarations. The absence of
project dependencies makes a probability-package-specific loop less likely.
This remains unproven because the CLI emitted no phase information or frames.

## 11. Alternative hypotheses

- A recursive generic, trait, or associated-type query in the current Universe
  declarations, possibly involving the class-side trait changes.
- A parser or syntax-tree traversal that recurses on an automatically loaded
  source shape.
- Recursion during lowering, materialization, or VM startup/execution rather
  than semantic analysis.
- A finite but exceptionally deep traversal that exceeds the native stack;
  the 8192 KiB result does not rule this out.
- A defect in project entry handling that enters a recursive path even for a
  trivial module.

## 12. Attempts already made

1. **Reduce optional dependencies.** Removed the probability dependency while
   retaining the statistics import. Prediction: a probability-specific cycle
   would disappear. Result: stack overflow persisted.
2. **Reduce project and source to a minimum.** Removed all project dependencies
   and replaced the source with the exported integer binding above. Prediction:
   if package implementation caused the failure, the minimal app would finish.
   Result: stack overflow persisted.
3. **Increase the native stack.** Set the stack limit to 8192 KiB. Prediction:
   a modestly deeper but finite call chain would complete. Result: stack
   overflow persisted.
4. **Isolate two recent source additions.** Temporarily removed generic Complex
   division/`ComplexDivision`, then removed the List mean facade, rebuilding the
   CLI. Prediction: if either addition alone caused the overflow, the minimal
   project would complete. Result: it still overflowed. Both removals were
   restored.

## 13. Why local authority is insufficient

Further source-level workarounds could change trait evidence, generic receiver
reification, or Universe ownership without locating the recursion. The same
failure occurs when the project contains no package dependencies, and the
available output does not identify the failing phase. Choosing cycle detection,
an explicit worklist, a recursion bound, or a source-level correction requires
an architecture decision about which layer owns the recursive query and its
termination contract.

## 14. Decision requested

Please advise on the next bounded diagnostic step that identifies the failing
phase and owning call path without changing semantic behavior. In particular:

1. Which existing tracing or compiler entry points should receive temporary
   phase markers (or what narrow stack-trace method should be used) to separate
   Universe loading, semantic analysis, lowering, and VM execution?
2. Once the recursive call is located, which layer should own termination for
   this case, and what invariant must the repair preserve if the cycle involves
   generic trait or associated-type resolution?

Implementation of the affected source-acceptance work is paused until that
guidance is recorded.

## 15. Architect ruling and follow-up evidence — 2026-10-11

The architect ruled that the active T3/T4 implementation has three concrete
defects to repair before source acceptance: contextual realization must
classify source and context before capability lookup; arithmetic-region
summarization must use one iterative post-order worklist; and ownerless generic
type-form dispatch must preserve class-side selection while binding `Self` to
the conforming subject type. The worktree implements those corrections and has
focused semantic regressions for assumed `T.zero`/`T.one`, wrong-side
selection, ordinary `Int` candidate rejection, arithmetic summary facts, and a
50,000-node stack-safety input.

A bounded LLDB run on the feature worktree stopped in repeated
`analyze_expression_inner -> analyze_expression -> synthesize_binary_expr`
frames while canonical Universe construction was querying the body of
`SpecialKernel.logGammaOnePlusRatio` in `universe:math.special.kernel` (source
range `1304..2033`). The CLI had completed project compilation and entered
`VM::new`; this is still compile-time semantic analysis during canonical
Universe construction, before program execution. It did not show recursion in
the iterative algebraic-region scanner, generic conformance proof, projection
normalization, or VM execution. An environment-gated callable trace confirms
that this body is the last query before abort.

The same dependency-free project was then built and run from a clean checkout
of `origin/main` at `c2dd635e`; it completed successfully with exit status 0
(about 65 seconds). The feature worktree's subprocess regression still aborts
with stack overflow (status 6, after 35.46 seconds) after the iterative scanner,
candidate gate, and class-side fixes. A focused attempt moved arithmetic-region
selection out of the central expression wrapper and into binary/unary analysis,
retaining direct imaginary-literal inference; the same callable still
overflows. The attempted relocation was reverted because it did not improve the
reproducer and its semantic effect was not independently accepted. It is not
part of the retained patch.

After reverting that experiment, the retained feature worktree reproduced the
same failure again: the minimal-project bootstrap regression exited with
status 6 after 26.18 seconds, and its last callable trace entry was again
`SpecialKernel.logGammaOnePlusRatio`. The clean-main pass remains the control;
this current-tree rerun confirms the blocker was not removed by the revert.

This narrows the incident to the added semantic expression-analysis path or
its stack cost on a supported nested scalar expression, rather than a general
`origin/main` baseline failure. The exact repeated analyzer edge and the
owner-level correction remain unresolved. Do not rewrite the standard-library
polynomial, raise stack limits, add a depth cutoff, or resume source acceptance
without an architectural ruling on the general expression-analysis owner.

### Follow-up decision requested

1. Given the clean-main pass and the feature-branch failure in
   `analyze_expression_inner` while analyzing a nested Float arithmetic body,
   which semantic-owner repair is acceptable: make expression analysis
   stack-safe for nested ASTs, or refactor the algebraic hooks so they do not
   increase stack use on ordinary scalar expressions?
2. What bounded regression should protect the owning invariant, in addition to
   the retained minimal-project CLI reproduction and the 50,000-node
   arithmetic-summary test?
3. The attempted relocation of inference from the central wrapper into
   binary/unary synthesis was reverted after the CLI regression remained red.
   Is the general expression-analysis owner repair a bounded stack-safe walk,
   or a different semantic-layer correction?

Source-level math/package acceptance remains paused. The branch-specific
failure is not classified as a baseline failure.

The follow-up evidence and decision request were sent on 2026-10-11 to the
existing architect conversation, **Incident audit analysis**. Its response was
subsequently received and applied as recorded below; the preceding trace and
failed attempt remain historical evidence.

## 16. Carrier/frame correction and distinct bootstrap blocker — 2026-10-11

### Accepted ruling and implementation

The architect identified inline contextual realization as the main regression
amplifier: measured `TypedExpression` was 6,640 bytes on this branch versus
744 bytes at clean-main `c2dd635e`; `ContextualRealization` was 5,896 bytes.
Both transient `TypedExpression` and durable `ExpressionAnalysis` now store
`Option<Arc<ContextualRealization>>`. Construction wraps the canonical product
once, publication clones its `Arc`, and fingerprinting dereferences it. No
evidence fields, identities, dependencies, or semantic checks were removed.
Ordinary binary synthesis has a separate helper from algebraic realization.

The required 64-coefficient Float Horner test still exhausted its 2 MiB
semantic thread after this correction. The first test setup also exhausted
the ordinary test thread while parsing parentheses; setup parsing now runs on
an explicit 8 MiB thread, separately from the 2 MiB analysis being tested.
The actual semantic regression then reproduced in the formal checker.

LLDB disassembly of the arm64 unoptimized test binary established:

| Function | Stack frame before local dispatch split | After |
| --- | ---: | ---: |
| formal `analyze_expression_inner` | 51,728 bytes | 64 bytes |
| formal `analyze_expression` | 8,000 bytes | unchanged |
| `synthesize_ordinary_binary_expr` | 1,616 bytes | unchanged |
| `synthesize_binary_expr` | 64 bytes | unchanged |
| advisory `analyze_expr_inner` | 18,928 bytes | 48 bytes |
| new advisory `analyze_binary_expr` | — | 976 bytes |

The formal analyzer's single large `Expr` match retained unrelated branches'
temporaries at every binary recursion level. Separating binary dispatch from
the remaining match removed that frame from the recursive path. The same
test then advanced to an independent advisory stack overflow with repeated
`advisory::analyzer::analyze_expr_inner` / `analyze_expr` frames. The identical
Binary arm was moved to an advisory helper behind small dispatch. This keeps
operand order, advisory observers, formal publication, and canonical target
consumption unchanged. The complete regression now passes: Float result,
no diagnostics, and no contextual realization products anywhere in the probe.
No general iterative expression machine or stack-limit increase was added.

Source owners: `phalcom-semantic/src/checker/expression.rs`,
`checker/typed_expr.rs`, `checker/analysis.rs`, `checker/algebraic_realization.rs`,
`advisory/analyzer.rs`, and `db/fingerprint.rs`. Regression owner:
`phalcom-semantic/tests/semantic/capabilities/algebraic_realization.rs`.

### Historical intermediate bootstrap failure

An intermediate bootstrap failure (`AssociatedInvokeNotLoweredYet` in
`universe:callable.function`) was encountered before the root cause was
diagnosed. See §17 below for the full diagnosis, fix, and passing evidence.

## 17. Canonical bootstrap resolution and verification — 2026-10-11

### Root cause diagnosis

Investigation identified two distinct interacting issues:

1. **Semantic dependency pollution on class-side trait dispatch:**
   In `numbers/complex.ph`, `impl<T> Zero for Complex<T> where T |= Zero` has
   `@class zero -> Complex<T> { Complex<T>(T.zero, T.zero) }`. When analyzing
   `T.zero`, `synthesize_get_property` called `specialize_class_side_target`.
   Because `callable.side == DispatchSide::Class`, it called
   `ctx.declaration_generic_signature(callable.declaration_owner())`.
   `CheckingContext::declaration_generic_signature` unconditionally recorded
   `SemanticDependency::DeclarationShell(Zero)`. Because `Zero` is a trait (not
   a nominal declaration or type alias in `declarations`),
   `ensure_formal_semantic_dependency_current` blocked with
   `SuppressedDependency`. This caused `session.update_with_delta` to fall back
   to an empty `SemanticSnapshot` with 0 callables, which emptied
   `lowering.associated` and produced the symptom at `Function.attempt`.

2. **Omission of impl generic evidence for class-side conformance witnesses:**
   In `phalcom-core/src/modules/semantic_lowering.rs`, `owner_sig` collection
   was gated on `if callable_id.side == DispatchSide::Instance`. Generic
   class-side conformance witnesses such as `@class one` in
   `impl<T> One for Complex<T> where T |= Zero, T |= One` were left with empty
   `generic_evidence`, causing `MissingTraitRequirementSlot(One.one)` when
   lowering `T.one`.

### Implemented corrections

1. In `phalcom-semantic/src/checker/expression.rs::specialize_class_side_target`:
   return early if `target.authority == CallTargetAuthority::AbstractContract`.
2. In `phalcom-semantic/src/checker/context.rs::declaration_info` and
   `declaration_generic_signature`: only record `DeclarationShell` dependencies
   when `self.declarations.contains(declaration)`.
3. In `phalcom-core/src/modules/semantic_lowering.rs`: retrieve the conformance
   plan / derived rule generic signature for any `CallableOwnerId::Conformance`
   callable regardless of dispatch side.

### Executed verification evidence

All Cargo commands run with `RUSTFLAGS='' RUSTC_WRAPPER=''`, serially:

- `cargo check -p phalcom-ast -p phalcom-semantic -p phalcom-core`: PASS.
- `cargo test -p phalcom-semantic --test semantic semantic::capabilities::algebraic_realization`:
  PASS, 5/5 passed.
- `cargo test -p phalcom-core --test canonical-bootstrap-stack dependency_free_project_compiles_then_bootstraps_and_runs`:
  PASS, 1/1 passed (63.92s). Full Universe compiles and bootstraps cleanly.

