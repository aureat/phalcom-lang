# Compiler Correctness and Semantic-Preserving Lowering

Compiler may aggressively change representation, but it must preserve source language's allowed observations. Lowering is implementation refinement, not a chance to redefine semantics.

## 1. Source and target relations

Let:

```text
S  source semantic configuration
V  VM/bytecode configuration
R(S,V) relation saying V correctly represents S
```

Forward-simulation intuition:

```text
R(S,V) and S -> S'
=> exists V'. V ->* V' and R(S',V')
```

`->*` permits administrative VM steps per source step.

For external/nondeterministic behavior use trace-aware simulation/refinement.

## 2. Why simulation matters practically

It asks:

> After this source action, what target state corresponds to source state, and which target steps account for same observation?

This catches hidden-local, stack-order, exception, and lifetime errors.

## 3. Hidden temporaries

Dynamic pack assembly, compound assignment, iterator lowering, interpolation, and spread often need compiler locals. Correctness requires:

- each source subexpression evaluated required number of times;
- required order preserved;
- temporary not source-observable except preserved effects/result;
- GC roots remain live while temporary holds objects.

## 4. Stack-machine invariant

For expression lowering document stack effect:

```text
before: depth d
normal completion: depth d+1 with result
abrupt completion: unwind according to control semantics
```

For statements define result pop/preservation. Stack imbalance is semantic corruption.

## 5. Send lowering

Preserve:

```text
receiver evaluation
argument evaluation order
selector construction
access context
lookup start (`super`)
message miss
self receiver binding
exception/non-local return propagation
```

Inline cache can replace lookup mechanism but not observations.

## 6. Intrinsics

Intrinsic optimization is refinement under guards:

```text
if exact primitive forms and no override-sensitive behavior:
    fast primitive
else:
    ordinary send fallback
```

Guard/fallback are part of correctness argument.

## 7. Desugaring correctness

For surface `D` lowered to core `C`:

```text
Obs(eval_source(D)) = Obs(eval_source(C))
```

validate at source semantics before bytecode. This separates language desugaring from backend correctness.

## 8. Control-sensitive lowering

Compiler-generated helper closure/method can accidentally change:

- non-local-return home frame;
- `super` lexical class;
- access authority;
- stack/source reflection;
- cancellation/handler scope.

Do not introduce callable boundaries casually.

## 9. Source mapping/reflection

If source ranges/method metadata are observable by diagnostics/reflection/debugger, lowering must retain mapping. Hidden instructions should not leak meaningless compiler frames unless specified.

## 10. GC correctness

Semantic liveness is broader than obvious operand stack. Temporary roots cover:

- receiver while arguments evaluate;
- partially assembled packs/collections;
- iterator/cursor sources;
- closures/home frames;
- pending exceptions/control values;
- suspended fiber frames.

Collecting live object is miscompilation.

## 11. Pass-by-pass refinement

Use discipline:

```text
AST
 -> semantic-normalized form
 -> bytecode
 -> VM execution
```

For each pass document invariants and semantic tests. Verified compilers show value of assigning semantics to intermediate languages and proving pass preservation; Phalcom can adopt this discipline incrementally without mechanizing entire compiler.

## 12. Forward versus backward simulation intuition

Forward simulation is convenient when source deterministic and target can stutter. Backward/refinement reasoning may be needed when target nondeterminism differs. Implementers need not formalize immediately, but should know "preserves behavior" is directional and assumption-sensitive.

## 13. Differential execution

Compare reference/slow semantic path with optimized bytecode path where possible:

- values;
- errors;
- side-effect traces;
- identity behavior;
- reflection metadata;
- scheduling traces where normative.

If no independent interpreter, compare optimization enabled/disabled and equivalent lower-level formulations.

## 14. Metamorphic tests

Transformations expected to preserve semantics under conditions:

- formatting;
- redundant parentheses;
- alpha-renaming locals;
- explicit annotation equal to inferred type when annotations erased;
- cache cold/warm;
- optimization off/on.

These detect drift without full oracle.

## 15. Validation of optimizer preconditions

For each optimization record:

```text
semantic preconditions
analysis facts proving them
runtime guards if facts uncertain
fallback/deopt
invalidations on reflection/module changes
```

An analysis bug must not silently become semantic miscompilation if runtime guard can protect it.

## 16. Common failures

- duplicated side-effect expression;
- wrong stack cleanup on throw;
- `super` loses lexical owner;
- cache ignores visibility/mutation version;
- hidden local not rooted;
- helper closure changes non-local return home;
- allocation removed despite identity/reflection effects.

## 17. Competency checks

1. What relation connects source and VM configurations?
2. Why are multiple target steps allowed per source step?
3. What must compound-assignment lowering prove about evaluation count?
4. Why can helper closure change semantics even if values match?
5. Which observations should optimized/unoptimized tests compare?
6. Why are runtime guards useful when optimization depends on uncertain static facts?
