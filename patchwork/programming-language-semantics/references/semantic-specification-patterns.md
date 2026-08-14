# Semantic Specification Patterns

Reusable shapes for writing Phalcom semantic specifications that implementation agents can act on.

## 1. Semantic feature card

For every feature include:

```text
Syntax
Semantic entities
Evaluation order
Normal result
Abrupt outcomes
State effects
Dispatch/access behavior
Reflection behavior
Module/fiber interactions
Static correspondence
Lowering obligations
Conformance fixtures
```

If field is irrelevant, say why rather than silently omit it.

## 2. Rule + prose + fixture

Best specification unit:

### Rule

```text
formal/pseudocode relation
```

### Explanation

State intuition and edge cases.

### Distinguishing fixture

Small program that fails under most plausible alternative semantics.

This triple is more robust than prose alone.

## 3. Outcome-threading pattern

For sequential evaluation:

```text
result = eval(first)
if result abrupt: propagate
else continue with result.value and updated state
```

Use consistently for arguments, collections, interpolation, sequences, and initializers.

## 4. Identity qualification pattern

Never use bare names where identity matters:

```text
ClassId(module, name)
CallableId(ownerClass, selector, side)
FieldId(ownerClass, name, side)
BindingId(declaration identity)
```

This is semantic clarity and implementation guidance.

## 5. Lookup/access split pattern

Keep conceptually separate:

```text
find candidate implementation
check invocation authority
```

Even if optimized implementation combines them. This exposes decisions such as whether inaccessible declaration stops superclass lookup.

## 6. Lexical versus dynamic context pattern

Record both:

```text
lexical class = where current method was defined
runtime receiver class = classOf(self)
```

`super` uses lexical class for lookup start; ordinary nested sends use runtime receiver.

## 7. Latent-effect pattern

For blocks/callables:

```text
construction effects
invocation effects
```

Do not apply invocation effects at literal creation. Higher-order summaries propagate latent effects when invocation is known/reachable.

## 8. Open-world pattern

When reflection/native/dynamic dispatch prevents closure:

```text
Known case: precise rule
Unknown/open case: conservative fallback
Assumption required for stronger guarantee
Invalidation if assumption changes
```

Better than globally disabling analysis.

## 9. Versioned-assumption pattern

Caches/proofs can depend on:

```text
class hierarchy epoch
method table epoch
module revision
native metadata version
```

State which dependency invalidates which fact.

## 10. Recovery-boundary pattern

Tooling can define recovery facts:

```text
RecoveredSemanticFact(origin=Recovery, confidence=...)
```

Executable semantics rejects/avoids recovery-only constructs. Mark boundary explicitly.

## 11. Optimization justification pattern

For rewrite `A -> B`:

```text
Preconditions
Observations preserved
Effects preserved
Control preserved
Reflection preserved
Runtime guards/fallback
Invalidation
Tests
```

Do not write "safe optimization" without these.

## 12. Semantic-diff review

For language change:

```text
Before rule
After rule
New observations
Removed observations
Compiler changes
Analyzer/checker/prover changes
Compatibility impact
```

This prevents syntax-only review.

## 13. Static guarantee card

For checker/analyzer fact:

```text
Fact produced
Evidence/provenance
Dynamic meaning
Unknown/open-world case
Trust assumptions
Invalidation
Consumers allowed to rely on it
```

This prevents heuristic facts from becoming proof facts accidentally.

## 14. Module rule card

For import/module proposal, always state:

```text
identity
resolution
namespace binding
initialization state/order
cycle behavior
cache behavior
failure behavior
reload behavior
```

## 15. Fiber rule card

For concurrency API, state:

```text
scheduling point?
may block OS thread?
shared-state interference?
cancellation behavior?
exception propagation?
fairness/liveness promise?
```

## 16. Competency checks

1. What three artifacts accompany difficult semantic rule?
2. Why record lexical/runtime class separately?
3. How should latent block effects enter caller summary?
4. What invalidates proof based on method table?
5. Why should recovery facts carry origin/confidence?
