# Review and Debugging Guide

## Review questions

### Semantics

- What exact question does this change answer?
- Is the answer guaranteed by runtime/spec or heuristic?
- Does it preserve ordinary selector dispatch?
- Is `ValueShape` still advisory rather than language type?

### Identity

- Is every cross-file entity module-qualified?
- Are source-local IDs confined to snapshot lifetime?
- Is dispatch/field side explicit?
- Are selectors canonical?

### Flow

- What happens on branch merge?
- What happens on loop back-edge?
- Are unreachable paths excluded where required?
- Are abrupt exits modeled by category?
- Are block construction and execution distinct?

### Interprocedural

- Is there a summary rather than recursive AST descent?
- What is recursion seed and fixed-point equality?
- Are dynamic calls conservative?
- Which reverse dependency invalidates caller?

### Incrementality

- What edit makes this fact stale?
- Does file removal remove every contribution?
- Does unchanged summary avoid unnecessary fanout?
- Is publication atomic/coherent?

### Diagnostics

- Can the analyzer explain the fact?
- Is provenance bounded?
- Can recovery/heuristics accidentally produce a hard error?

### Performance

- Does this run every keystroke/every query?
- New allocations per expression/call?
- Can IDs replace clones/strings?
- Did rebuild frontier grow?
- Is union/constraint growth bounded?

## Debugging wrong inference

Trace from query backward:

```text
consumer output
 -> semantic query result
 -> snapshot fact
 -> local/field/parameter/callable source
 -> transfer expression/statement
 -> binding/dispatch identity
 -> AST/source
```

At each step inspect both value and provenance.

Common root causes:

- wrong binding due to scope/source-order bug;
- receiver class ID lost to bare name;
- selector labels canonicalized differently;
- branch state mutated sequentially;
- `Unknown` joined incorrectly;
- summary stale/not invalidated;
- dynamic call incorrectly marked exact;
- field side mismatch;
- consumer still uses legacy inference path.

## Debugging missing completion/member

1. exact occurrence/receiver target at cursor;
2. inferred receiver `ValueShape` and confidence;
3. class identity/module;
4. dispatch side;
5. class surface members/inheritance;
6. visibility context;
7. selector/member filtering;
8. union policy;
9. consumer conversion/ranking.

Do not start by adding special cases to completion.

## Debugging stale result after edit

1. confirm file revision updated;
2. confirm semantic generation advanced;
3. confirm updated file snapshot replaced;
4. inspect module dependent closure;
5. inspect callable reverse dependency;
6. compare old/new summary equality;
7. inspect final published snapshot;
8. confirm request did not intentionally use older document snapshot.

Add rebuild-trace regression.

## Debugging recursion/nontermination

Check:

- call graph SCC;
- domain growth;
- union/provenance canonicalization;
- summary equality includes volatile revision/provenance incorrectly;
- dynamic dependency expanding frontier repeatedly;
- missing widening cap.

Instrument iteration count and changed field of summary.

## Debugging false checker/prover error (future)

Determine whether evidence is:

```text
exact static proof
flow-derived sound fact
interprocedural sound summary
advisory shape
heuristic use-site guess
unknown
```

A checker consuming lower-strength evidence as proof is an architecture bug, not merely a
bad diagnostic.

## Debugging runtime/tool disagreement

Build minimal executable fixture and compare:

- selector formed by compiler;
- class/module identity at runtime;
- member lookup/inheritance;
- constructor behavior;
- native primitive result/effects.

Then read normative spec. Decide which implementation is wrong. Never make tooling match a
runtime bug silently if spec says otherwise.

## Review gate

Do not approve a semantic change until:

```text
correctness
recovery
incrementality
determinism
provenance
performance
future typing boundary
runtime agreement
```

are each explicitly considered.

A "small LSP fix" that adds a parallel inference path fails this gate even if the visible bug
is fixed.
