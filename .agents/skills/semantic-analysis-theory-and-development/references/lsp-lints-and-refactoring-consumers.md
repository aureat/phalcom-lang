# LSP, Lints, and Refactoring as Semantic Consumers

## 1. Adapter rule

The LSP should translate semantic queries into protocol responses. It should not own a second resolver/inference engine.

```text
hover          -> occurrence/target + facts + provenance renderer
completion     -> receiver/context + member surface/candidate query
definition     -> SemanticTarget -> declaration origin
references     -> occurrence index by target
rename         -> target + conflict simulation
signature help -> selector family/candidate signatures
diagnostics    -> syntax/semantic/checker/proof diagnostics
inlay hints    -> selected advisory/formal facts rendered with trust
```

## 2. Current Phalcom anchor

**CURRENT:** LSP semantic infrastructure already exposes occurrence-at-offset, visible bindings, binding info/facts, class/member surfaces, inherited completion members, callable summaries, parameter facts, inferred expressions, imports, and snapshot stamps. This is exactly the direction to preserve when extracting a shared semantic core.

## 3. Hover

Hover should identify the semantic target first. It may render multiple domains separately:

```text
Greeter#greet
runtime-shape inference: String (interprocedural)
declared type: String        // future, if present
```

Do not present advisory shape as a declared/formal type. Provenance can power “why?” detail without changing the compact default.

## 4. Completion

Completion needs incomplete-source tolerance and candidate ranking, not necessarily proof. Resolve receiver as far as possible, obtain semantic member surface, then apply prefix/access/context filters.

Heuristics may rank candidates. They must not alter semantic existence. A dynamic receiver can fall back to broad workspace/core member discovery with clearly lower confidence if desired.

## 5. Definition/references

Navigation should be identity-based. Source text equality is insufficient under shadowing/modules. Occurrence indexes should be built once per generation and reused.

For ambiguous/recovery occurrences, navigation can return multiple targets or none; do not arbitrarily choose the first.

## 6. Rename

Rename requires stronger guarantees than completion. It must preserve resolution and detect conflicts after hypothetical edits. If the semantic engine cannot establish safety across dynamic reflection or generated names, narrow/refuse the operation rather than text-replace.

## 7. Lints

A lint declares its trust requirement. Examples:

```text
exact semantic identity -> unused local lint
sound reachability      -> unreachable code lint
heuristic shape         -> advisory style hint only
```

Do not emit high-severity correctness claims from heuristic flow unless the lint is explicitly probabilistic/advisory.

## 8. Snapshot/cancellation discipline

All fields of one LSP response should come from one semantic snapshot/generation. Do not resolve the target from generation `g` and fetch hover facts from `g+1` after an edit. Clone one snapshot/stamp at request start.

Long workspace reference/rename operations should be cancellable but must not publish semantic mutation.

## 9. Latency budgets

Interactive targets commonly need sub-100ms perceived response, but never sacrifice identity correctness for arbitrary deadlines. Prefer stale-but-explicit snapshot policy or bounded advisory precision over half-mutated current state. Track p95/p99 and query fan-out.

## 10. Tests

- hover/definition agree on target identity;
- shadowed names navigate/rename correctly;
- completion works on incomplete selector without claiming resolved call;
- inlay hint labels advisory `ValueShape` as inference, not annotation;
- all response components use same generation;
- missing dependency suppresses cascaded member errors;
- rename conflict simulation catches shadowing/import collisions;
- handler does not trigger full workspace rebuild for cached query.

## 11. Review questions

1. Which semantic query powers this handler?
2. Is any resolver/inference duplicated in presentation code?
3. What trust level is rendered to the user?
4. Does the response come from one snapshot?
5. Is the operation safe under ambiguity/recovery?
6. Does the query scale with semantic frontier rather than workspace size?
