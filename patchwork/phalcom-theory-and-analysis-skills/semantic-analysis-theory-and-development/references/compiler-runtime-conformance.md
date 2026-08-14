# Compiler and Runtime Conformance

## Static semantics must model execution

For every executable semantic rule, compare analyzer against compiler/VM:

- selector canonicalization;
- implicit self;
- super lookup;
- class/metaclass side;
- field access/visibility;
- block capture/non-local return;
- pack evaluation order;
- module identity/import behavior;
- native return/effects.

## Differential harness

Build fixtures where semantic engine predicts:

```text
resolved target/candidates
return shape/type
effect/control outcome
```

then execute and compare observed runtime behavior for representative inputs.

## Optimizations

Compiler inlining/specialization does not change semantic target. Analyzer should model source semantics, optionally consuming compiler intrinsics as trusted equivalent summaries.

## Core source and native floor

Phalcom core includes source declarations plus native primitives. Semantic surface should come from the same authoritative declarations/metadata as runtime registration to avoid hardcoded drift.

## Bytecode metadata

Future type/contracts encoded in bytecode must map back to semantic descriptor IDs/content and be validated by VM. Static analyzer should not assume bytecode-only metadata that source reflection cannot represent unless spec says so.

## Runtime mutation

If classes/methods can mutate reflectively, analyzer exactness is conditional on semantic generation/open-world policy. Runtime caches and static caches need parallel invalidation concepts.

## Bug triage

If analyzer and runtime disagree:

1. identify normative spec;
2. minimize source case;
3. inspect selector/scope/class-side identities;
4. determine compiler bug, analyzer bug, or spec ambiguity;
5. fix shared semantic rule, not consumer workaround.
