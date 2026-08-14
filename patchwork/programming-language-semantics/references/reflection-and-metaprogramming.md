# Reflection and Metaprogramming Semantics

Reflection is not merely debugging support. Once programs can observe or mutate classes, methods, selectors, modules, or metadata, those operations constrain identity, access control, caching, optimization, and static reasoning.

## 1. Reflective semantic objects

Potential entities:

```text
Class / Metaclass / Behavior
Method
Selector / Symbol
Message
Module
Protocol / Type descriptors
attributes/annotations
source metadata
Family / MethodFamily
```

Specify which are snapshots, live views, immutable descriptors, or mutable handles.

## 2. Identity versus equality

For every descriptor decide:

- declaration/allocation identity;
- semantic equality;
- hashing;
- canonicalization/interning;
- lifetime.

Future applied types may be semantically canonicalized while ordinary class declarations retain declaration identity.

## 3. Method enumeration and authority

Enumeration can reveal physical existence without granting invocation permission.

Do not infer:

```text
method ∈ methods(C) => caller may invoke method
```

unless explicitly policy.

## 4. Reified methods

A `Method` descriptor should define:

- holder/owner;
- selector and side;
- source metadata;
- code/implementation identity;
- behavior after replacement;
- binding semantics;
- access checks on invocation.

If method is replaced, old `Method` may continue denoting captured implementation or become invalid; choose one rule.

## 5. Reflective method mutation

```text
installMethod(C, s, m)
```

is a semantic write to dispatch state. Consequences:

- inline caches invalidate/version;
- analyzer closed-world assumptions weaken;
- devirtualization needs guards/closure proof;
- existing instances ordinarily observe changed lookup.

## 6. `perform`

Dynamic sends formed from selector values should use ordinary lookup/access after selector validation. Privileged bypass, if any, must be explicit and narrowly named.

## 7. Families and selector patterns

Reflection over method sets must define capture mode:

```text
snapshot of concrete implementations
live query view
delayed receiver dispatch pattern
```

Phalcom's `MethodFamily` and `Family` intentionally differ; preserve that distinction.

## 8. Metadata retention

Future typing metadata is reflectively observable but does not silently change ordinary dispatch. Compiler/bytecode must retain promised annotations even if checker information is otherwise erased.

## 9. Attributes/decorators/macros by stage

Possible stages:

```text
parse-time syntax transform
AST/declaration transform
metadata attachment
method wrapping/replacement
runtime registration
```

Specify stage, evaluation order, identity preservation, and error timing. "Attribute" alone does not determine semantics.

## 10. Reflective class construction

If runtime APIs can create classes/metaclasses, define:

- superclass/metaclass wiring;
- module/owner identity;
- field layout finalization;
- method-table mutation;
- hierarchy-version invalidation;
- visibility/privilege requirements.

Static analysis must then treat hierarchy as open unless bounded by mode/version assumptions.

## 11. Open-world precision ladder

Do not treat all reflection equally:

```text
read-only metadata reflection       low invalidation risk
method enumeration                  identity observation
perform(dynamic selector)           dynamic dispatch uncertainty
method-table mutation               invalidates dispatch facts
class/hierarchy mutation            invalidates subtype/lookup closure
privileged VM reflection            trust/security boundary
```

This supports targeted conservatism.

## 12. Stack/source reflection

If call stack/source locations are observable, inlining/tail calls/helper lowering may change visible behavior. Decide whether stack reflection is normative, debug-only, or implementation-defined.

## 13. Security/authority

Separate:

```text
discovery
inspection
binding
invocation
mutation
privileged VM access
```

Each may require different authority.

## 14. Static/prover boundary

A proof depending on method `m` remaining installed should record version/closed-world assumption. Reflective mutation must invalidate proof or be excluded from sound mode.

## 15. Competency checks

1. Why does method-table mutation affect inline-cache semantics?
2. Can enumeration reveal private method while invocation stays forbidden?
3. What happens to reified old method after replacement?
4. Why can type metadata be reflectable without selector identity?
5. Which reflection capabilities force open-world assumptions?
