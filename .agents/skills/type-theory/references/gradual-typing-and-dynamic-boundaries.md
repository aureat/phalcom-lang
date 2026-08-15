# Gradual Typing, `Dynamic`, Precision, Casts, and Boundaries

## Purpose

Phalcom is dynamically executable while gaining optional correctness-participating typing. This reference explains how to add static guarantees without pretending untyped code has precise static types or allowing one permissive sentinel to contaminate all relations.

The key design question is not "static or dynamic?" It is:

> What guarantee is provided in checked regions, what happens when information crosses a dynamic boundary, and where are checks deferred?

## 1. Distinguish special concepts

Potential states/types include:

### Safe top `Any`

If ratified as top:

```text
T <: Any
```

A value of static type `Any` supports only operations guaranteed for every value/universal object contract.

### `Dynamic`

An explicit gradual escape. Specific operations are allowed statically even when not proven; runtime dispatch/checking decides success.

Phalcom's normative core typing design explicitly distinguishes `Dynamic` from ordinary top `Any`.

### Analysis unknown

The analyzer has insufficient knowledge. This is not necessarily user-denotable and must not authorize operations by itself.

### Missing annotation

Source fact: no annotation was written. Current proposed Phalcom type-expression design explicitly preserves missing annotations rather than rewriting them immediately to `Dynamic`, `Any`, or `Object`.

### Error recovery

Internal sentinel after a diagnosed error. Not a language type.

### Bottom `Never`

No normal value/path. Opposite of "we do not know".

These distinctions are mandatory.

## 2. Gradual typing adds a precision dimension

Gradual typing often defines a **precision** relation separate from subtyping.

Write:

```text
A ⊑ B
```

as "A is at least as precise as B" (direction varies by literature; define it).

Example intuition:

```text
Int ⊑ Dynamic
String ⊑ Dynamic
```

This is not:

```text
Int <: Dynamic
```

unless the language deliberately makes it so.

Subtyping concerns substitutability; precision concerns how much static information is present.

## 3. Consistency

A gradual consistency relation `~` allows a dynamic type to connect with static types:

```text
Int ~ Dynamic
Dynamic ~ String
```

Consistency is often:

- reflexive;
- symmetric;
- **not transitive**.

Non-transitivity prevents `Dynamic` from proving arbitrary static compatibility.

Implementation should never compute transitive closure of consistency.

## 4. Consistent subtyping

Some gradual systems define a directional relation combining subtyping and consistency, often called consistent subtyping.

This can answer assignment/call questions such as:

```text
actual Dynamic -> expected String
```

while recording a runtime cast obligation.

If Phalcom adopts such a relation, name it separately from `<:`. Its result should be richer than boolean because successful gradual acceptance may carry a check/coercion plan.

Conceptual result:

```text
AcceptResult =
  StaticProof
  RuntimeCheck(CastPlan)
  DynamicDispatch
  Reject(reason)
```

## 5. Cast insertion / runtime contracts

A sound gradually typed system can insert casts at typed/untyped boundaries.

Example conceptual flow:

```text
dynamic source -> expected String
```

creates check:

```text
cast_String(v)
```

Failure should identify boundary and contract, not merely crash later on an unrelated method send.

Phalcom may choose checker-only, typed-runner, or ordinary-runtime enforcement modes. Those modes can share metadata while differing in timing. The exact mode policy belongs in current typing specs.

## 6. Blame

Blame tracks which side of a typed/untyped boundary violated the contract.

Conceptual boundary:

```text
untyped module U -> typed module T expects (Int -> String)
```

A higher-order contract must check both directions:

- when typed code passes `Int` into dynamic function, typed side fulfilled parameter obligation;
- when dynamic function returns, result must satisfy `String`;
- if dynamic function cannot accept a value promised by its contract, blame dynamic provider.

Higher-order blame requires wrappers/contracts around callable values, not just one upfront class test.

If Phalcom typed-runner later validates callable contracts, this distinction matters.

## 7. Gradual guarantee

"Gradual guarantee" refers to formal properties relating programs as type precision changes. There are several variants.

An ergonomic informal goal might be:

> Removing annotations should not turn dynamically valid ordinary execution into compile-time rejection solely because less static information is available, while adding annotations should increase guarantees or move failures earlier.

A formal static gradual guarantee relates typing results under precision changes; a dynamic gradual guarantee relates runtime behavior/casts.

Do not claim Phalcom has "the gradual guarantee" without choosing the exact theorem and checking reflection/runtime metadata complications.

## 8. Dynamic sends

For:

```text
x : Dynamic
x.fly()
```

the checker may permit the send without proving selector availability. The result policy must be defined:

- `Dynamic` result;
- explicit declared dynamic-member metadata if available;
- runtime contract result when callable boundary carries one.

Do not look at current runtime shape and claim a static proof unless that bridge is sound for the checker mode.

## 9. Containment versus viral dynamic

A permissive `Dynamic` can spread:

```text
Dynamic + Dynamic -> Dynamic
Dynamic.foo -> Dynamic
```

This is ergonomic but can erase static guarantees far from the boundary.

Containment strategies:

- require explicit annotation to exit dynamic region;
- use safe `Any`/unknown-like boundary requiring narrowing;
- retain provenance of dynamic origin;
- permit operations but mark results dynamically tainted;
- typed-runner inserts checks at annotated boundaries.

Choose policy deliberately and test annotation economy on real Phalcom code.

## 10. Missing annotations are not automatically `Dynamic`

Suppose source:

```phalcom
f(x) { ... }
```

Reflection may need:

```text
parameter.annotation = None
```

Static checker may infer `x` from call sites/contracts or treat it under an unannotated-code policy. It should not mutate source metadata into "user declared Dynamic".

This matters for:

- documentation;
- typed-runner boundary insertion;
- diagnostics;
- API review;
- future stricter modes.

## 11. Unknown information versus explicit dynamic choice

Two states:

```text
x: Dynamic          # programmer/language policy authorizes deferred checking
x: ?unknown         # analyzer lacks sufficient evidence
```

If checker permits `x.foo()` in both, analysis failures silently turn into dynamic escapes. That is unsound for a correctness-oriented checker.

Unknown should usually require:

- more inference;
- an annotation;
- a checked runtime boundary;
- or an explicit dynamic policy.

## 12. Reflection as a dynamic boundary

Reflective operations may construct/select methods by runtime data. Static checker often cannot know exact result.

Model this explicitly:

```text
reflection query -> Dynamic/opaque/existential result + provenance
```

depending on API contract.

Do not crash the checker or fabricate a precise member type from one observed execution.

## 13. Native/FFI boundaries

Native Rust code can violate assumptions if it returns values not matching metadata.

Possible policies:

- trusted native signature is part of trusted computing base;
- debug/typed-runner validates returned runtime values;
- FFI adapters perform explicit conversions/checks;
- unknown native code yields dynamic/opaque boundary.

The checker theorem must state which native specifications it trusts.

## 14. Gradual generics

`List<Dynamic>` is not necessarily the same as raw/unannotated `List`.

Questions:

- Is a missing type argument legal?
- Does raw generic origin mean constructor object rather than value type?
- Is `List<Dynamic>` covariant/invariant according to normal generic rules?
- Are runtime checks needed on mutation?

Do not use "raw generic" and `Dynamic` interchangeably.

## 15. Dynamic and variance

If mutable `Cell<T>` is invariant, allowing:

```text
Cell<Int> ~ Cell<Dynamic>
```

through consistency may require runtime wrappers/checks on both reads and writes to preserve typed guarantees.

Higher-order gradual typing interacts with variance deeply. A shallow outer runtime class check is not enough to validate generic/callable contracts if type arguments are semantically meaningful.

## 16. Proof interaction

Static prover must not assume a dynamically accepted operation is proven safe.

Keep status:

```text
StaticallyProven
RuntimeChecked
DynamicDeferred
UnknownProof
```

separate from canonical type.

A path through `Dynamic` can invalidate stronger propositions unless a runtime check reestablishes them.

## 17. LSP interaction

LSP can show:

```text
inferred: String
confidence/provenance: flow-derived
boundary: from Dynamic call
```

without making checker acceptance depend on advisory inference.

Hover/completion can be optimistic if clearly classified, while diagnostics must follow checker policy. Both consume shared semantic identities and type/shape bridges.

## 18. Testing gradual boundaries

Build paired programs differing only in annotation precision.

Test:

- static-to-dynamic and dynamic-to-static values;
- dynamic callable parameters/results;
- generic mutable containers;
- missing annotation versus explicit `Dynamic` reflection;
- native/FFI return violation;
- reflection result;
- blame/provenance location;
- checker mode versus typed-runner mode;
- no transitive consistency laundering.

If claiming a gradual guarantee, encode its formal testable fragment as metamorphic/property tests.

## 19. Failure modes

- `Dynamic` implemented as subtype top.
- Analysis unknown authorizes dynamic operations.
- Missing annotation rewritten permanently to `Dynamic`.
- Consistency cached/transitively closed like subtyping.
- Higher-order casts check only outer runtime class.
- Proof engine treats dynamic acceptance as proof.
- Native metadata trusted implicitly without declared trust boundary.

## 20. Competency questions

1. What is the difference between subtyping and precision?
2. Why is gradual consistency often non-transitive?
3. Why can `Dynamic -> String` acceptance require a runtime obligation rather than subtype proof?
4. Why is missing annotation semantically different from explicit `Dynamic`?
5. What extra work is needed to validate a dynamic callable contract?
6. How can LSP remain useful without laundering advisory shape inference into checker proof?
