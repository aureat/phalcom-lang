# Semantic Model Review Checklist and Pressure Tests

Use this before approving semantic design, implementation plans, or production Rust changes. The checklist is deliberately cross-subsystem: a local feature is not complete if it creates a second semantic truth, breaks invalidation, or turns advisory evidence into a correctness claim.

## 1. Status and authority

- [ ] Every repository-specific statement is classified as `CURRENT`, `NORMATIVE/RATIFIED`, `PROPOSED`, `EXPERIMENTAL`, `FUTURE`, or `RECOMMENDATION` where status is material.
- [ ] Relevant `docs/spec/current/`, ADR/PDR and implementation source were inspected.
- [ ] Proposed typing/design documents are not presented as current VM behavior.
- [ ] Runtime/compiler behavior was checked when the semantic fact describes dynamic execution.
- [ ] A repository implementation disagreement with a spec was investigated rather than papered over.

## 2. Semantic question and ownership

- [ ] The exact question is written in one sentence.
- [ ] The answer belongs in shared semantics rather than a consumer adapter.
- [ ] Existing fact/query/domain ownership was inspected before adding a parallel structure.
- [ ] The new fact does not combine unrelated domains merely for convenience.
- [ ] Neighboring skills own implementation/type/proof details that should not be duplicated here.

Example of a precise question:

```text
At program point p, what possible runtime shapes can BindingId b have in semantic
snapshot G, and what source/call evidence justifies that approximation?
```

## 3. Identity

- [ ] Fact key uses semantic identity, not bare source spelling.
- [ ] Module qualification is present where same-named declarations can coexist.
- [ ] Selector identity is canonical and agrees with compiler/runtime rules.
- [ ] Instance/class side is represented where relevant.
- [ ] Scope/binding IDs have documented lifetime.
- [ ] Source range is location/provenance unless intentionally snapshot-local identity.
- [ ] Stable-across-edit claims define which edits preserve identity.
- [ ] Source declaration identity and runtime object identity remain distinct.

## 4. Resolution

- [ ] Lexical resolution happens before value/type inference.
- [ ] Declaration-order visibility is honored where the language requires it.
- [ ] Shadowing uses `BindingId`, not spelling.
- [ ] Import alias and resolved target identities are both represented.
- [ ] Unresolved import/dependency edges remain representable.
- [ ] Ambiguous, unresolved and recovery-blocked states are not silently collapsed.
- [ ] References/rename operate from semantic occurrence targets rather than text replacement.

## 5. Knowledge-domain semantics

- [ ] Runtime value shape, language type, proof proposition, effect and optimizer assumption are explicitly separated.
- [ ] `Unknown` is not being used as `Any`, `Dynamic`, bottom, error, timeout and unresolved simultaneously.
- [ ] "No annotation" is distinct from explicit dynamic typing policy.
- [ ] Heuristic evidence cannot create a hard checker/proof result without an explicit bridge.
- [ ] Recovery facts are not treated as valid alternate runtime semantics.
- [ ] Domain bridges list preconditions and information loss.

## 6. Abstract domain

- [ ] Concrete meaning/concretization is stated.
- [ ] Precision order `⊑` is understood.
- [ ] Top and bottom are defined if relevant.
- [ ] Join `⊔` is defined and conservative.
- [ ] May/must polarity is correct.
- [ ] Transfers are monotone or non-monotonicity is explicitly handled.
- [ ] Widening/cap is defined if ascending chains can grow.
- [ ] Fixed-point equality is semantic/deterministic rather than pointer/order identity.
- [ ] Budget exhaustion has a semantic status distinct from convergence.

## 7. Flow

- [ ] Facts are valid at a program point, not file-global guesses.
- [ ] Branch merges join reachable predecessors only.
- [ ] Unreachable does not become `Unknown`.
- [ ] `return`/throw/break/continue/non-local exits affect continuation correctly.
- [ ] Loop-carried facts iterate to a fixed point or widen conservatively.
- [ ] Closure construction is distinct from closure execution.
- [ ] Captured mutation can invalidate refinements.
- [ ] Future CFG/IR introduction is justified by repeated semantic needs, not compiler fashion.

## 8. Dispatch and call sites

- [ ] Receiver category distinguishes instance/class object/module/`super`/unknown where required.
- [ ] `super` changes lookup start while preserving receiver.
- [ ] Inheritance resolution returns the actual declaring target.
- [ ] Dynamic packs/labels do not fabricate an exact selector.
- [ ] Argument mapping preserves labels and source evaluation order.
- [ ] Union receiver distinguishes member-on-any versus member-on-all semantics.
- [ ] Passing a block does not imply executing it.
- [ ] Type annotations do not silently alter ordinary selector identity.
- [ ] Reflection/dynamic mutation prevents unjustified closed-world call-graph claims.

## 9. Interprocedural summaries

- [ ] Summary inputs and outputs are defined.
- [ ] Effects are part of the summary where relevant.
- [ ] Caller-to-callee and reverse dependency edges exist.
- [ ] Recursion uses worklist/SCC/fixed-point logic rather than naive recursive AST descent.
- [ ] Parameter/field/effect evidence has contribution ownership if edits can retract it.
- [ ] Summary change equality determines propagation.
- [ ] Operational iteration budgets do not masquerade as mathematical convergence.

## 10. Provenance and diagnostics

- [ ] Important facts retain bounded evidence at derivation time.
- [ ] Join/widening preserves or summarizes explanation causally.
- [ ] Diagnostics can distinguish expected-source and observed-source chains.
- [ ] Provenance cap affects explanation detail, not semantic truth.
- [ ] Recovery/budget/ambiguity reasons survive far enough for useful diagnostics.
- [ ] A future "why?" query would not require re-running a second heuristic analysis.

## 11. Modules and incrementality

- [ ] Source contribution unit is explicit.
- [ ] Old contribution is removable/retractable.
- [ ] Dependency edges have meaning and reverse invalidation ownership.
- [ ] File revision and semantic generation are distinct.
- [ ] Cache key/value/dependencies/validity/invalidation/concurrency/memory bound are specified.
- [ ] Updating/removing a provider cannot leave stale consumer facts.
- [ ] Unresolved dependencies can be repaired when providers appear.
- [ ] Same-named declarations in different modules remain distinct.
- [ ] A semantic delta can stop propagation when new fact equals old fact.
- [ ] Published snapshot is coherent and immutable to readers.
- [ ] Incremental result is tested against a clean full rebuild.

## 12. Typing/checker integration

- [ ] Type syntax, resolved type, canonical type and inference metavariables remain distinct.
- [ ] Subtyping/assignability/consistency/equality are not collapsed.
- [ ] `ValueShape` is not promoted to canonical `TypeId` representation.
- [ ] Shape-to-type inference is an explicit bridge with soundness/policy stated.
- [ ] Open-world use-site observations do not automatically become declared contracts.
- [ ] Typed-runner/runtime check behavior is specified separately from static checking.
- [ ] Type metadata does not silently change ordinary dispatch.

## 13. Static proving/contracts

- [ ] Proof status distinguishes `Proved`, `Refuted`, `Unknown`.
- [ ] Solver timeout is `Unknown`, not false/proved.
- [ ] Finite testing/unrolling is not presented as proof.
- [ ] Dynamic dispatch/reflection assumptions are part of proof obligations/trusted base.
- [ ] Mutation/aliasing invalidates path predicates when required.
- [ ] Contracts produce proof/runtime obligations according to a specified mode.
- [ ] Native/FFI trust assumptions are explicit.

## 14. Consumers

- [ ] Hover/completion/checker/lint/refactor query shared facts.
- [ ] Consumer policy for uncertainty is explicit.
- [ ] Completion heuristics do not mutate semantic truth.
- [ ] Definition/references/rename use semantic identity.
- [ ] Diagnostics use shared checker/semantic/proof results rather than re-inference.
- [ ] Formatter only consumes semantics when formatting behavior genuinely depends on it.
- [ ] Optimizer requires facts strong enough for its assumptions or adds guards/fallback.

## 15. Rust/data-structure safety

- [ ] Typed IDs/newtypes are used where identity categories differ.
- [ ] Long-lived raw references into mutable semantic storage are avoided.
- [ ] Immutable published facts/snapshots have clear ownership.
- [ ] Deterministic ordering is preserved for observable facts/tests.
- [ ] Large ASTs/strings/provenance chains are not cloned into every hot-path fact.
- [ ] Derived caches record dependency ownership.
- [ ] File-local IDs do not escape their generation/snapshot contract.
- [ ] Native/unsafe boundaries expose semantic effects and errors conservatively.

## 16. Performance

- [ ] Baseline exists before optimization.
- [ ] Complexity is proportional to the changed semantic frontier where practical.
- [ ] Repeated AST walks are justified or replaced with shared indexes.
- [ ] Union/domain growth is bounded intentionally.
- [ ] Recursive/SCC analysis has termination/latency policy.
- [ ] Snapshot publication avoids unnecessary deep clones.
- [ ] Query path does not require broad global mutation/lock contention.
- [ ] Cache memory has a bound/eviction lifetime.
- [ ] Optimization preserved semantic invariants and was benchmarked again.

## 17. Testing

- [ ] Positive semantic fixture.
- [ ] Negative/error fixture.
- [ ] Unknown/ambiguous/recovery fixture.
- [ ] Shadowing/module-identity fixture when relevant.
- [ ] Branch merge fixture for flow facts.
- [ ] Loop needing multiple iterations.
- [ ] Recursive/call-chain fixture.
- [ ] Dynamic/reflection conservative fixture.
- [ ] Incremental edit fixture that removes evidence, not only adds it.
- [ ] Provider removal/creation fixture for module facts.
- [ ] Full rebuild versus incremental equivalence.
- [ ] Deterministic ordering/equality test.
- [ ] Metamorphic property considered.
- [ ] Fuzz/property testing considered for domain/parse/incremental state.
- [ ] Performance/rebuild-counter regression considered.

## 18. Skill pressure tests

The following scenarios test whether an agent has internalized the model. The expected response is the semantic distinction/invariant, not a specific Rust patch.

### Pressure test A — tempting `ValueShape` type reuse

Proposal:

> "Typing is optional, and `ValueShape` already has unions/tuples/classes. Rename it to `Type` and use it in the checker."

Expected answer:

- reject the rename-as-design;
- explain runtime shape versus normative language type;
- identify missing type constructs/relations/substitution/generics/special states;
- define an explicit shape-to-type bridge instead;
- preserve LSP shape precision policies such as union cap independently from type algebra.

### Pressure test B — one-pass loop

```phalcom
let x = A.new()
while cond {
  x = B.new()
}
use(x)
```

Temptation: analyze body once with entry `A` and finish.

Expected answer: identify the back-edge/fixed-point equation, iterate to `A | B` (or the domain's conservative equivalent), and distinguish unreachable/unknown.

### Pressure test C — stale monotone parameter inference

Generation 1 has a caller passing `Cat`. Generation 2 edits the same call to pass `Dog`.

Expected answer: an append-only join producing `Cat | Dog` is stale. Evidence needs contribution ownership/retraction or recomputation. Current Phalcom `ParameterContributions` is the relevant pattern.

### Pressure test D — range as stable declaration ID

Proposal: use `(file, SourceRange)` as a cross-edit method ID because it is simple.

Expected answer: range is revision-local location; whitespace can move it. Define identity lifetime/matching policy. Current callable identity should be semantic owner+selector+side; ranges remain provenance/location.

### Pressure test E — type-directed ordinary dispatch

Proposal: add parameter types to the ordinary selector key so the checker can choose the best method.

Expected answer: reject unless an explicit typed-dispatch feature is ratified. Static typing must describe current dynamic sends; it cannot silently create a second dispatcher.

### Pressure test F — reflection versus inlining

Proposal: because analysis resolves every `C.foo()` send to one source method, inline it without a guard.

Expected answer: source-known resolution is insufficient if runtime method installation/replacement is observable. Require frozen/closed-world semantics or a runtime version/class/selector guard plus correct fallback/deoptimization.

### Pressure test G — parser recovery poisoning facts

A half-written method body fails to resolve one expression. Temptation: mark every class/member fact in the file `Unknown`.

Expected answer: distinguish source recovery from semantic uncertainty; preserve unaffected declarations/scopes/surfaces where reliable, localize blocked facts, and avoid interpreting invalid complete source as valid dynamic semantics.

### Pressure test H — solver timeout

The prover times out on an invariant.

Expected answer: status is `Unknown`/timeout; it is neither `Proved` nor `Refuted`. Testing or bounded unrolling cannot upgrade it to proof.

### Pressure test I — module cycle

Two modules import each other. Temptation: run the same SCC fixed-point algorithm used for callable summaries and declare the cycle valid if it converges.

Expected answer: SCC is only graph structure. Module cycle legality/initialization/partial-state semantics come from the module specification; callable abstract interpretation does not decide runtime module-cycle behavior.

### Pressure test J — cache without validity

Proposal:

```text
cache[(file, offset)] = completion_members
```

Expected answer: reject until dependencies are specified. Imported class surfaces, hierarchy, module graph and semantic generation can change without touching the file. Define key, dependencies, validity, invalidation, concurrency and memory bound.

### Pressure test K — closure capture refinement

A branch proves captured variable `x != None`, then a callable that may mutate the captured cell runs before use.

Expected answer: refinement survives only if alias/effect analysis proves no invalidating write. Captured mutable state and effects must participate in validity.

### Pressure test L — fiber suspension

An analysis assumes a mutable global/class field remains unchanged across a suspension point.

Expected answer: the assumption needs concurrency/scheduler/shared-state semantics. Suspension may permit other execution to mutate state; do not carry the refinement unless isolation/effect rules justify it.

## 19. Final review questions

A reviewer should be able to answer, without hand-waving:

- What semantic identity does the feature operate on?
- What is observable to Phalcom code versus implementation-only?
- What is the dynamic evaluation/lookup/control behavior?
- Which abstract domain and uncertainty states represent it statically?
- Where are joins/fixed points, and what guarantees termination?
- Which evidence can be retracted after edits?
- What dependencies invalidate each derived fact?
- What provenance must survive for diagnostics?
- What does incomplete source do?
- What does reflection/native code/concurrency do to assumptions?
- Does typing alter execution or merely check it?
- What would an optimizer need beyond advisory semantics?
- Which tests distinguish the correct design from the tempting shortcut?
- Which future language choice would this representation accidentally lock in?
