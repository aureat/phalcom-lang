# Review Checklist and Validation Scenarios

Use these scenarios to pressure-test an implementation plan, PR, or agent. Each presents a tempting shortcut. A competent analysis agent should identify the violated invariant, state the correct abstraction/algorithm, and describe a discriminating test. Merely naming “soundness” is not enough.

## 1. Core review checklist

Before approving any static analysis, verify:

```text
[ ] Concrete property/question named.
[ ] Consumer named.
[ ] May/must and sound/advisory/proof contract named.
[ ] Dynamic/world/native assumptions named.
[ ] Abstract domain and semantic meaning documented.
[ ] Order/join/top/bottom documented where applicable.
[ ] Unreachable/bottom distinct from unknown/top.
[ ] Transfer functions preserve evaluation order.
[ ] Transfer/join monotonicity justified.
[ ] Branch edge refinement modeled only for trusted predicates.
[ ] Loop zero-iteration path and back-edge modeled.
[ ] Loop convergence/widening policy defined.
[ ] Recursion/interprocedural convergence policy defined.
[ ] Unknown/dynamic calls have explicit effect/havoc policy.
[ ] Reflection/FFI/fibers have explicit assumption/effect policy.
[ ] Provenance and precision-loss reasons retained where needed.
[ ] Contribution retraction/removal handled incrementally.
[ ] Cache key/dependencies/validity/invalidation/publication/memory bound defined.
[ ] Semantic equality is canonical and excludes irrelevant metadata.
[ ] Incremental final facts equal clean rebuild.
[ ] Malformed source policy distinguishes recovery from language error.
[ ] Consumer trust threshold prevents heuristic promotion.
[ ] Performance is measured against semantic frontier, not guessed.
```

## 2. Scenario — one-pass loop

### Pressure

Analyze loop body once and use that state after the loop.

```text
x = 0
while condition {
    x = x + 1
}
use(x)
```

### Required response

A loop denotes an equation over a header state and includes a zero-iteration path. Compute/join the back-edge until a fixed point or widening. Exit state joins false-condition and break paths. One body pass is not generally a post-fixpoint.

### Test

Construct a loop where second iteration adds a new union alternative or effect that first iteration cannot see.

## 3. Scenario — unknown means no effect

### Pressure

Unresolved dynamic call returns `Unknown`; preserve all fields/globals because no target was found.

### Required response

Target-resolution failure is analysis uncertainty, not runtime no-op. Apply a dynamic effect envelope/havoc over mutable state reachable by the unknown call, plus throw/yield/reflection behavior allowed by semantics. Preserve only facts proven insulated.

### Test

Runtime target mutates a field through an alias; static analysis must not retain old field refinement.

## 4. Scenario — union cap used by checker

### Pressure

Reuse LSP bounded `ValueShape` union. Nine alternatives widen to `Unknown`; checker treats `Unknown` as permissive and accepts the send.

### Required response

Reject. Current union cap is an advisory-analysis policy. Checker needs its own formal type/dynamic/unknown policy whose fallback preserves correctness. “Analysis lost precision” is not “language opted into Dynamic.”

### Test

Create more than the advisory cap with one alternative lacking required member; checker must not accept merely due to cap overflow.

## 5. Scenario — branch latest value

### Pressure

AST visitor walks `then` then `else`; whichever assignment is visited last becomes variable result.

### Required response

Branch results join over reachable predecessors. Join must be commutative/associative/idempotent so traversal order does not determine semantics.

### Test

Reverse branch traversal and assert identical final abstract state.

## 6. Scenario — recursive summary stack overflow

### Pressure

When analyzing `f`, recursively analyze callee `g`; `g` analyzes `f` again.

### Required response

Use callable summaries, a dependency graph/worklist, and SCC/fixed-point reasoning. Seed unknown/bottom according to domain convention; update monotonically; re-enqueue dependents on semantic summary change.

### Test

Mutual recursion whose return facts require multiple rounds to stabilize.

## 7. Scenario — path explosion

### Pressure

Keep every branch path separately to maximize precision.

### Required response

Unbounded trace enumeration is exponential/nonterminating with loops. Define selected predicate partitions, maximum partitions/context, merge strategy, and conservative fallback with reason.

### Test

Generate `n` independent booleans; ensure state count remains bounded and result still over-approximates behaviors.

## 8. Scenario — stale class cache

### Pressure

```text
cache[(ClassId, selector)] = target
```

No revision/dependency information.

### Required response

Cache validity depends on governing class/member/superclass surfaces and possibly reflective world state. Declare invalidation conditions. Static generation cache and runtime inline cache may have different keys but both require validity.

### Test

Change/replace method or superclass; cached target must update.

## 9. Scenario — may versus must confusion

### Pressure

One branch initializes `x`; another does not. Mark `x` definitely initialized because initialization is possible.

### Required response

Definite initialization is a must-property: fact survives merge only if true on every reachable predecessor. Use meet/intersection-like behavior or a dedicated definite-assignment domain.

### Test

Diamond with one uninitialized predecessor; diagnostic must remain.

## 10. Scenario — native trust by absence

### Pressure

Rust primitive has no Phalcom source body, therefore assume no side effects.

### Required response

Opaque code requires a trusted native summary or conservative boundary. Absence of analyzable body increases uncertainty.

### Test

Native stub declared `MayWrite` versus `NoWrite`; verify caller facts differ and summary version invalidates dependents.

## 11. Scenario — heuristic optimizer

### Pressure

LSP says receiver is probably `String`; replace dynamic send with direct String method call.

### Required response

Heuristic editor fact is insufficient. Optimizer needs a sound exact receiver/world fact or runtime class/method-table guard plus fallback/deoptimization.

### Test

Run same source with runtime subclass/alternate receiver; optimized program must remain behaviorally equivalent.

## 12. Scenario — type-directed method selection sneaks in

### Pressure

Future type annotations identify a more specific implementation, so checker/compiler selects it.

### Required response

Reject unless Phalcom separately ratifies type-directed dispatch. Current selector identity and dynamic receiver lookup remain language semantics; typing verifies contracts, not target identity.

### Test

Add/remove a semantically redundant explicit annotation and assert runtime dispatch target unchanged.

## 13. Scenario — `Unknown` equals bottom

### Pressure

No fact for a binding, so treat path unreachable and suppress diagnostics.

### Required response

Unknown/top means represented executions exist but analysis lacks precision. Bottom means no represented execution can reach the point. Conflating them causes false proofs.

### Test

Unresolved call returns unknown then dangerous operation; analysis must keep path reachable.

## 14. Scenario — impossible branch becomes unknown

### Pressure

Trusted exact class test contradicts exact receiver fact; represent false branch as `Unknown` state.

### Required response

Contradiction makes edge unreachable/bottom. Unknown would add behaviors and lose terminating-branch precision.

### Test

`if exact-test { return }` pattern should preserve opposite refinement on continuing path.

## 15. Scenario — confidence is trust

### Pressure

`Confidence::Exact` means fact is safe for prover/optimizer.

### Required response

Current confidence describes evidence strength in an advisory domain. Future consumers need an explicit trust/soundness contract. Exact syntax can still be irrelevant to a proof if dynamic boundaries invalidate it.

### Test

Exact declared/shape fact crosses unchecked native mutation; optimizer must not preserve dependent heap fact without effect proof.

## 16. Scenario — captured variable copied into each block

### Pressure

Each closure stores current value of mutable `x`; analyze captures independently.

### Required response

If normative semantics capture mutable storage by shared cell, closures alias the same cell. Model `CapturedCell(BindingId, home)` and escape/lifetime accordingly.

### Test

One closure writes `x`, another reads it after invocation; analysis must observe possible updated value.

## 17. Scenario — closure body effects happen at construction

### Pressure

A block literal contains IO/write, so mark those effects at declaration site.

### Required response

Separate closure construction effects from latent body effects. Apply body effects only when invocation is possible/proven according to higher-order summary and timing.

### Test

Construct but never invoke a block that mutates local; post-state must remain unchanged except capture/allocation facts.

## 18. Scenario — callback invoked once

### Pressure

Callee summary says parameter is invoked; caller applies block effects exactly once.

### Required response

“Invoked” does not imply cardinality/timing. If callee may invoke many times, solve repeated effect/state transfer or widen. If deferred, do not apply as synchronous post-state.

### Test

`repeatTwice` callback increments captured number; distinguish from once.

## 19. Scenario — fiber yield preserves shared field refinement

### Pressure

No local statement writes `shared._state`, so refinement survives `yield`.

### Required response

A yield may allow another fiber to mutate shared reachable state. Apply interference havoc unless ownership/isolation guarantees preservation.

### Test

Two fibers mutate/read shared field across yield; static fact must not remain exact without protection.

## 20. Scenario — blocking equals yielding

### Pressure

Represent both as `may_suspend` and use same transfer.

### Required response

In cooperative scheduling, `may_yield` lets other fibers run; `blocks_thread` can stop scheduler progress. Keep distinct effects for lints/runtime reasoning.

### Test

A blocking native call should trigger blocking-in-fiber lint even when it does not create an interleaving point.

## 21. Scenario — exact points-to singleton gets strong update

### Pressure

`Pts(x) = {AllocSite#7}`, so replace field value strongly.

### Required response

One abstract allocation site may represent many concrete objects, especially in loops. Strong update requires a must-singleton abstract location, not merely a singleton may-set.

### Test

Allocation inside loop; field writes on different iterations must join.

## 22. Scenario — FFI argument does not escape

### Pressure

Rust call returns immediately, so passed object remains callable-local.

### Required response

Native code may retain object/callback unless summary guarantees no retention. Mark escape/alias according to explicit FFI contract.

### Test

`NoRetain` and `MayRetain` native summaries lead to different escape facts.

## 23. Scenario — GC reachability proves no alias

### Pressure

Runtime uses stable GC handles; therefore each handle is unique and no alias analysis needed.

### Required response

Multiple program values can hold same handle. GC object identity does not imply static no-alias. Points-to/escape solve a different question from tracing reachability.

### Test

`a = obj; b = a; b.field = ...` must invalidate/read through `a`.

## 24. Scenario — field refinement survives unknown callback

### Pressure

Callback receives no explicit receiver argument, so cannot mutate `self`.

### Required response

Closures/reflection/globals may still reach `self`; prove non-reachability/escape or use conservative write envelope. Lexical parameter lists are not a heap reachability proof.

### Test

Callback captures alias to `self` and mutates field.

## 25. Scenario — provenance is discarded to save memory

### Pressure

Keep only final type/shape; diagnostics can reconstruct reasons later by walking AST.

### Required response

Interprocedural/widening/dynamic reasons are not reliably reconstructible from syntax. Keep bounded structural provenance or derivation IDs where explanations matter; budget it explicitly.

### Test

Parameter inferred from distant caller return should explain source chain without heuristic re-analysis.

## 26. Scenario — provenance participates in convergence

### Pressure

Every iteration adds a new source origin; summary equality compares full provenance, so solver never stabilizes.

### Required response

Separate semantic equality from explanatory metadata where appropriate. Canonical/bounded provenance should not cause semantic churn if abstract value/effect is unchanged.

### Test

Same semantic summary with different provenance insertion order should stop propagation.

## 27. Scenario — source offset is semantic identity

### Pressure

Cache facts by declaration start byte; edit comment above it.

### Required response

Source position changed but semantic declaration may not. Use semantic IDs/fingerprints and revision-aware source mapping; offsets are positions, not durable identities.

### Test

Insert comment before unchanged member; body frontier should not mark unrelated callable changed if current identity design promises that.

## 28. Scenario — no contribution retraction

### Pressure

Parameter facts only join new caller evidence; removed callers leave old contribution in aggregate.

### Required response

Incremental joins need ownership/retraction. Store per-source contributions or recompute from authoritative contributors. Current Phalcom uses contribution-indexed parameter facts; preserve pattern.

### Test

Remove the only `String` caller; parameter union must shrink to remaining alternatives after clean/incremental analysis.

## 29. Scenario — cache text hash only

### Pressure

File text unchanged, so semantic cache valid.

### Required response

Imports, core/native surfaces, package versions, configuration/type mode, and dependencies can change without local text. Cache validity includes semantic inputs, not text alone.

### Test

Change imported provider method/type/native summary while consumer source text stays unchanged; consumer result must invalidate if dependent.

## 30. Scenario — cancellation publishes partial products

### Pressure

Update classes as soon as parsed, summaries later; newer edit cancels halfway.

### Required response

Queries must observe one coherent semantic generation. Build candidate state or transaction; publish atomically after successful completion/epoch check.

### Test

Inject cancellation between summary and parameter phases; live snapshot remains previous coherent generation.

## 31. Scenario — CFG drops non-local return

### Pressure

Treat block invocation as ordinary call returning to next block.

### Required response

If normative block semantics permit non-local return, CFG/structured flow needs abrupt edge to home callable exit. Omitting it creates false reachability and invalid dominance.

### Test

Invoked block returns non-locally before statement following higher-order call; following statement must be unreachable on that path.

## 32. Scenario — SSA fixes heap aliasing

### Pressure

Convert locals to SSA; now field values are unique versions too.

### Required response

SSA gives scalar definitions unique names. Heap locations remain aliased mutable memory unless MemorySSA/alias/effect analysis models them.

### Test

SSA local aliases to same object with unknown call between field write/read; cannot retain field constant merely from scalar SSA.

## 33. Scenario — dynamic world treated as workspace-closed

### Pressure

Only classes A/B implement selector in indexed project, so target set is complete.

### Required response

Workspace enumeration is not runtime closure unless project/profile/reflection semantics guarantee it. Retain dynamic remainder or closed-world assumption token.

### Test

Load/reflectively install an additional applicable behavior under an open profile; unguarded optimization must not rely on A/B-only set.

## 34. Scenario — exact reflected selector treated as ordinary syntax

### Pressure

Reflection has exact selector string, so all ordinary send rules apply automatically.

### Required response

Target analysis can reuse selector resolution, but reflected invocation may have distinct access/binding/fallback semantics. Inspect normative reflection contract and model those differences.

### Test

A case where reflective access context differs from lexical send should produce the specified target/error.

## 35. Scenario — malformed source makes whole file dynamic

### Pressure

Parser recovery encountered one error; widen every fact in file to unknown.

### Required response

Recovery uncertainty should be localized. Unaffected declarations/scopes/facts remain useful if parser/semantic identity supports it. Do not reinterpret invalid complete source as valid, but do not poison unrelated editor facts.

### Test

Half-write one method call in class B; class A definitions/completion/refs remain stable.

## 36. Scenario — solver cap means fixed point

### Pressure

After ten rounds, label result “converged.”

### Required response

A cap is not convergence. Either equality reached a fixed/post-fixpoint or fallback widening produced a conservative post-fixpoint according to documented policy. Otherwise report budget failure/unknown.

### Test

Instrument a domain that changes on round 11; raw cap must not claim fixed-point precision.

## 37. Scenario — proof by testing

### Pressure

Generated 100,000 executions found no counterexample; mark contract proved.

### Required response

Testing is evidence, not proof over unbounded semantics. A prover needs VC/symbolic/abstract proof with explicit assumptions. Runtime differential testing can falsify soundness but cannot establish universal proof.

### Test

Not applicable as a unit assertion; review should demand proof object/solver obligation or label result tested-only.

## 38. Scenario — solver timeout means property holds

### Pressure

SMT solver did not find counterexample before timeout, so accept.

### Required response

Timeout/unknown is not `unsat`. Result is `Unknown(Timeout)` and checker/prover policy decides whether to fail, warn, require annotation/invariant, or defer to runtime check.

## 39. Scenario — type change is body-only

### Pressure

Future annotations are added but executable body text unchanged; current body-delta fingerprint ignores them, so no dependent invalidation.

### Required response

Once annotations become semantic declaration inputs, declaration fingerprints and dependency keys must include them according to ratified semantics. Incremental architecture must evolve with language surface.

### Test

Change parameter/return contract only; callers/checker/LSP hover/type consumers recompute while runtime selector identity remains unchanged.

## 40. Scenario — optimizer strips provenance/source IDs

### Pressure

Optimization only needs bytecode, so discard semantic/source identity early.

### Required response

Diagnostics, deoptimization, reflection, tooling, and future proof explanations may require mappings. Keep semantically required provenance through the appropriate IR boundary; optimize representation, not observability.

### Test

Optimized diagnostic/error still maps to correct source construct and semantic callable.

## 41. Final reviewer prompt

For any proposal, ask the author to complete this sentence precisely:

```text
This analysis approximates __________
using domain __________ ordered by __________.
It is [may/must] and [sound under assumptions/advisory].
Loops/recursion terminate because __________.
Dynamic/reflection/native behavior is handled by __________.
Facts become invalid when __________.
When budget is exceeded, __________ happens and remains valid because __________.
Consumers allowed to rely on the result are __________.
The regression test that distinguishes the design from the tempting shortcut is __________.
```

If that cannot be filled in without hand-waving, the implementation is not ready.
