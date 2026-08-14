# Review Checklist and Validation Scenarios

## Review checklist

- Semantic question stated independently of UI.
- Normative dynamic/static semantics identified.
- New entity has explicit typed ID/lifetime.
- Name resolution precedes inference.
- Source range is not misused as durable identity.
- Existing semantic owner extended instead of duplicated.
- AST versus HIR/CFG choice justified by consumers.
- Runtime shape/type/effect/proof facts remain distinct.
- Dispatch matches selector/class/metaclass/access rules.
- Recovery path cannot panic or fabricate certainty.
- Interprocedural data uses summaries/SCCs.
- Module/package dependency invalidation defined.
- Query returns semantic data, not LSP protocol data.
- Incremental result checked against full rebuild.
- Runtime/compiler differential tests cover executable rule.
- Performance/rebuild frontier measured.

## Scenario 1 — checker starts from AST strings

Pressure: easiest new checker is a fresh AST walker with `HashMap<String, Type>`.

Expected: reject duplication; reuse scope/binding IDs and introduce type facts over semantic representation.

## Scenario 2 — HIR for prestige

Pressure: add 100-node IR before any consumer needs normalization.

Expected: require concrete duplication/analysis need; avoid architecture tax without payoff.

## Scenario 3 — no HIR despite five CFGs

Pressure: linter/prover/checker each already reconstruct loops/branches separately; avoid risky refactor.

Expected: this is exactly when shared semantic body IR is justified; migrate incrementally with equivalence tests.

## Scenario 4 — class name key

Pressure: cache member results by bare `"Point"`.

Expected: class identity is module-qualified; package/module evolution makes bare names unsafe.

## Scenario 5 — UI type in core

Pressure: semantic engine can directly return `CompletionItem` to save adapter code.

Expected: keep semantic query protocol-neutral; LSP renders.

## Scenario 6 — unknown import as global

Pressure: unresolved import/name should become `Global(String)` so completion continues.

Expected: keep unresolved reason/candidates; fabricating global certainty contaminates checker/refactorings.

## Scenario 7 — current source is closed world

Pressure: protocol/dispatch analyzer assumes indexed classes are all future subclasses/methods.

Expected: respect open-world/reflection/module policy; exactness is revision/closure dependent.

## Scenario 8 — proof fact stored as type

Pressure: after `x > 0`, intern `PositiveInt` as type everywhere.

Expected: unless refinement types are explicit language feature, keep proposition/path fact separate from canonical `Int`.

## Scenario 9 — stale cross-file clone

Pressure: copy imported class members into each module snapshot for query speed.

Expected: use semantic reference/query or explicit dependency/versioned cache; copied remote data becomes stale.

## Scenario 10 — runtime mismatch workaround

Pressure: completion expects a member runtime doesn't actually dispatch due to metaclass/access rule; special-case completion.

Expected: fix shared semantic dispatch model or runtime/spec discrepancy; never UI-only semantic fork.
