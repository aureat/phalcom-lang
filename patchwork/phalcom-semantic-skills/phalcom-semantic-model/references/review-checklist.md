# Semantic Model Review Checklist

Use this for design reviews and before merging semantic changes.

## Specification alignment

- [ ] Relevant current spec/ADR/PDR read.
- [ ] Proposed typing documents are not treated as implemented runtime behavior.
- [ ] Dynamic object model and selector semantics remain unchanged unless explicitly specified.
- [ ] Core/native behavior agrees with visible/spec contract.

## Layer ownership

- [ ] Fact belongs in the semantic layer rather than an LSP/lint/checker feature module.
- [ ] Existing semantic fact/query was checked before adding a parallel representation.
- [ ] Syntax and semantic identity remain distinct.
- [ ] Runtime shape and language type remain distinct.

## Identity

- [ ] Fact is keyed by correct module-qualified/lexical semantic ID.
- [ ] Bare spelling is used only for display/lookup input, not durable identity.
- [ ] Snapshot-local IDs do not escape their lifetime.
- [ ] Dispatch side/field side is represented where relevant.
- [ ] Selector is canonical and spec/runtime-compatible.

## Domain semantics

- [ ] Meaning of exact/unknown/ambiguous/error/unreachable is documented.
- [ ] Precision/order relation is understood.
- [ ] Join is defined.
- [ ] Widening/cap is defined if domain can grow.
- [ ] Fixed-point equality is deterministic/canonical.
- [ ] Must-vs-may polarity is correct.

## Flow

- [ ] Facts are valid at the queried program point.
- [ ] Branch merges join reachable states.
- [ ] return/throw/break/continue semantics are accounted for.
- [ ] Loop-carried facts converge or widen conservatively.
- [ ] Closure construction is not confused with closure execution.
- [ ] Capture/non-local-return effects are modeled where relevant.

## Dispatch/calls

- [ ] Name resolution happens before value inference.
- [ ] `self`, `super`, class object, module, instance and unknown receiver cases are distinct.
- [ ] Dynamic selector/pack cases remain conservative.
- [ ] Inheritance returns actual declaring target.
- [ ] Call-site argument labels/order are preserved.
- [ ] Recursive calls use summary/fixed-point machinery.

## Provenance and diagnostics

- [ ] Important facts retain bounded origin/evidence.
- [ ] Diagnostic consumer can explain expected vs observed.
- [ ] Heuristic evidence cannot accidentally produce hard correctness errors.
- [ ] Recovery state does not masquerade as exact certainty.

## Modules/incrementality

- [ ] Dependency edges needed for invalidation are recorded.
- [ ] Updating/removing a file cannot leave stale facts.
- [ ] Cross-module identical class names remain distinct.
- [ ] Batch updates publish one coherent generation.
- [ ] Cache lifetime/invalidation rule is explicit.
- [ ] Rebuild-frontier tests cover the new dependency.

## Future typing/proving

- [ ] New representation leaves a clean bridge to `TypeId`/checker facts.
- [ ] It does not force type metadata into ordinary dispatch identity.
- [ ] Proof/contract semantics are not encoded as ad-hoc runtime shapes.
- [ ] Effects needed by fibers/FFI/concurrency are not silently assumed away.

## Consumers

- [ ] Hover/completion/checker/lint can query shared fact instead of recomputing it.
- [ ] Consumer policy for unions/unknown/confidence is explicit.
- [ ] Refactorings use identity, not text replacement.
- [ ] Optimizer does not consume heuristic facts without guards.

## Testing

- [ ] Positive fixture.
- [ ] Negative/unknown fixture.
- [ ] Shadowing/module-identity fixture if relevant.
- [ ] Branch/loop fixture if flow-sensitive.
- [ ] Recursive/call-chain fixture if interprocedural.
- [ ] Incomplete syntax/recovery fixture.
- [ ] Incremental edit/removal fixture.
- [ ] Determinism test.
- [ ] Metamorphic test considered.
- [ ] Performance/rebuild counter regression considered.

## Final questions

- [ ] What future feature does this representation preclude?
- [ ] What happens when the analyzer knows less than expected?
- [ ] What happens when the runtime is more dynamic than the analyzer assumed?
- [ ] Can an edit make this fact stale without touching the fact's source file?
- [ ] Can a user understand why the tool believes this fact?
