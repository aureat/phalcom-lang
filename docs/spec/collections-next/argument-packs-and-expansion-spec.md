# Phalcom Argument Packs and Expansion Specification

**Status:** Ratified language design specification  
**Scope:** Argument-pack construction, selector derivation context, positional and labeled lanes, expansion operators, variadic capture, forwarding, and eager expansion constraints.  
**Out of scope:** Full Tuple semantics, Record and Map type specifications, Symbol grammar in full, iterator protocol details, and generic capability hierarchy except where required to define expansion behavior.

---

## 1. Purpose

This specification defines how Phalcom represents arguments before dispatch and how argument-like structure is expanded, captured, forwarded, and composed.

Phalcom dispatch is selector-based rather than type-based. Consequently, argument-pack construction is a semantic phase that occurs before method lookup. In particular, the ordered sequence of labeled arguments participates in selector identity.

The design separates:

1. source evaluation order;
2. argument-lane construction;
3. selector derivation;
4. method lookup;
5. parameter binding.

These concepts MUST NOT be conflated by an implementation.

---

## 2. Normative Terminology

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

### 2.1 Positional lane

The **positional lane** is the ordered sequence of unlabeled argument values in an argument pack or Tuple-like argument structure.

### 2.2 Labeled lane

The **labeled lane** is the ordered sequence of `(Symbol, Value)` labeled arguments in an argument pack or Tuple-like argument structure.

Labels are Symbols. No implicit String-to-Symbol conversion occurs.

### 2.3 Argument pack

An **argument pack** contains two logically distinct lanes:

```text
ArgumentPack {
    positional: [Value, ...]
    labeled:    [(Symbol, Value), ...]
}
```

The positional lane precedes the labeled lane semantically.

### 2.4 Lane projection

A **lane projection** extracts contribution suitable for one or both argument lanes from a source value.

The three expansion operators select projections:

```text
*source
    positional/element projection

**source
    labeled/association projection

***source
    complete two-lane argument projection
```

### 2.5 Encounter order

**Encounter order** is the stable traversal order exposed by an ordered source for expansion.

Encounter order is distinct from equality semantics. A Record or Map may have order-insensitive equality while still preserving a stable encounter order used by `**`.

### 2.6 Eager exhaustor

An **eager exhaustor** is an operation that must consume a source until exhaustion before it can complete successfully.

Examples include positional expansion from a general Iterable and materialization operations such as `toList`.

### 2.7 Boundedness classifications

A source may be classified statically as:

- **statically bounded** — known to terminate after a finite number of elements;
- **provably unbounded** — known not to terminate by exhaustion;
- **unknown-boundedness** — termination cannot be proven either way.

---

## 3. Dispatch Context

Phalcom dispatch is purely selector-based.

There is no runtime type-based overload selection and no multimethod candidate ranking.

The conceptual dispatch pipeline is:

```text
evaluate call operands
    ↓
construct argument pack
    ↓
derive selector
    ↓
lookup method
    ↓
bind parameters
    ↓
execute
```

Selector identity includes:

```text
selector family
+
positional arity
+
ordered labeled argument sequence
```

Therefore these calls have distinct selector identities:

```phalcom
foo(a: 1, b: 2)
```

```phalcom
foo(b: 2, a: 1)
```

An implementation MUST preserve this distinction.

---

## 4. Argument-Pack Invariants

An argument pack has exactly two lanes:

```text
positional lane
labeled lane
```

The following invariants apply:

1. positional arguments semantically precede labeled arguments;
2. positional values preserve their contribution order;
3. labeled entries preserve their contribution order;
4. duplicate labels are invalid;
5. labels are Symbols;
6. no implicit duplicate overriding occurs;
7. source expressions are evaluated according to lexical source order;
8. lexical evaluation order does not imply lexical interleaving of the final lanes.

---

## 5. Source Syntax Boundary

Phalcom source syntax has a disciplined transition from the positional phase to the labeled phase.

### 5.1 Positional phase

Before the labeled phase begins, a call or Tuple construction MAY contain:

- ordinary positional values;
- `*` expansions;
- `***` expansions.

Example:

```phalcom
foo(
    1,
    *items,
    ***args,
)
```

### 5.2 Labeled phase

The labeled phase begins when source syntax introduces a construct that contributes only to the labeled lane, including:

- an explicit labeled argument;
- a `**` expansion;
- a labeled trailing closure.

After the labeled phase has begun, source syntax MUST NOT contain any construct that may contribute positional values.

In particular, the following are invalid after the boundary:

- ordinary positional values;
- `*` expansion;
- `***` expansion.

Valid:

```phalcom
foo(
    1,
    *items,
    ***forwarded,
    timeout: 10,
    **options,
)
```

Invalid:

```phalcom
foo(
    timeout: 10,
    *items,
)
```

Invalid:

```phalcom
foo(
    timeout: 10,
    ***args,
)
```

The same source-order discipline applies to Tuple construction.

---

## 6. Expansion Operators

### 6.1 `*` — positional expansion

`*source` contributes to an element-oriented or positional destination.

In a call or Tuple, `*source` contributes values to the positional lane.

In an element-oriented collection literal, such as a List or Set literal, `*source` contributes elements.

For ordinary Iterable sources, `*` consumes values in encounter/iteration order.

Example:

```phalcom
const numbers = [1, 2, 3]

foo(0, *numbers)
```

constructs a positional lane equivalent to:

```phalcom
foo(0, 1, 2, 3)
```

### 6.2 `**` — labeled or association expansion

`**source` contributes association-like structure.

Its exact validation is target-sensitive.

For a call, Tuple, or Record destination, contributed keys MUST be Symbols and MUST have stable encounter order.

For a Map destination, arbitrary valid Map keys may be contributed because the destination itself is an arbitrary-key association.

Example:

```phalcom
const options = #{
    timeout: 10,
    retries: 3,
}

foo(**options)
```

contributes labeled arguments in Record encounter order.

### 6.3 `***` — complete two-lane expansion

`***source` contributes both the positional and labeled lanes of a source that intrinsically carries both lanes.

For built-in values, Tuple is the canonical source for `***`.

Example:

```phalcom
const args = (
    url,
    body,
    timeout: 10,
    retries: 3,
)

request(***args)
```

is equivalent in argument-pack structure to:

```phalcom
request(
    url,
    body,
    timeout: 10,
    retries: 3,
)
```

`***` MUST NOT be defined merely as redundant spelling for single-lane values.

Thus, core semantics do not define:

```phalcom
***list
***record
***map
```

as aliases for `*list` or `**record` / `**map`.

---

## 7. Built-In Expansion Sources

The following core behavior is ratified.

| Source | `*` | `**` | `***` |
|---|---|---|---|
| Tuple | positional lane | labeled lane | both lanes |
| List | iteration elements | unsupported | unsupported |
| Range / Progression | iteration elements | unsupported | unsupported |
| Bytes | iteration elements | unsupported | unsupported |
| Set | encounter-order elements, subject to Set semantics | unsupported | unsupported |
| Record | unsupported | fields in encounter order | unsupported |
| Map | unsupported | entries in insertion encounter order | unsupported |
| Iterator / Iterable | iteration elements | unsupported by default | unsupported |
| future `HashMap` | unsupported as labeled source | unsupported for ordered labeled expansion | unsupported |

This table defines built-in semantics only. A future capability/protocol design MAY generalize expansion to user-defined types.

Such future generalization MUST preserve the semantic constraints in this specification.

---

## 8. Tuple-Specific Projection Rules

Tuple is the first-class value representation of Phalcom's two-lane argument structure.

A Tuple may contain:

```text
ordered positional lane
+
ordered labeled lane
```

Given:

```phalcom
const t = (
    1,
    2,
    x: 3,
    y: 4,
)
```

the projections are:

```phalcom
*t
```

contributes:

```text
1
2
```

while:

```phalcom
**t
```

contributes:

```text
x: 3
y: 4
```

and:

```phalcom
***t
```

contributes both lanes.

Normal Tuple iteration is specified separately and MUST NOT redefine the meaning of `*Tuple`. `*Tuple` is a positional-lane projection, not generic iteration over the Tuple's linearized total product.

---

## 9. Lane-Wise Splicing

Expansion composition is lane-wise rather than lexically interleaved.

Consider:

```phalcom
const a = (
    1,
    x: 10,
)

const b = (
    2,
    y: 20,
)
```

Then:

```phalcom
foo(***a, ***b)
```

constructs:

```text
positional lane:
    1
    2

labeled lane:
    x → 10
    y → 20
```

and therefore has the same argument-pack structure as:

```phalcom
foo(
    1,
    2,
    x: 10,
    y: 20,
)
```

It does NOT produce a semantically interleaved sequence equivalent to:

```text
1, x: 10, 2, y: 20
```

### 9.1 Evaluation order remains lexical

Lane-wise splicing does not change expression evaluation order.

For:

```phalcom
foo(***aExpr(), ***bExpr())
```

`aExpr()` is evaluated before `bExpr()`.

An implementation MAY append contributions to internal lane builders while evaluating each expression, but the observable final pack MUST obey lane-wise composition.

---

## 10. Duplicate Labels

Duplicate labels are invalid after all explicit and expanded labeled contributions are composed.

Examples:

```phalcom
const options = (
    timeout: 10,
)

foo(
    **options,
    timeout: 20,
)
```

is invalid.

Likewise:

```phalcom
foo(
    timeout: 20,
    **options,
)
```

is invalid.

And if:

```phalcom
foo(***a, ***b)
```

causes both `a` and `b` to contribute the same label, argument-pack construction fails.

No last-wins or first-wins rule exists for argument packs.

If a duplicate can be proven statically, an implementation SHOULD diagnose it statically.

Otherwise it MUST fail before method lookup.

---

## 11. Label Requirements

Tuple labels, Record fields, and argument-pack labels are Symbols.

There is no implicit conversion:

```text
String → Symbol
```

during argument-pack construction or labeled expansion.

A Map used as a labeled source may contain arbitrary key types in general, but every key contributed into a call, Tuple, or Record labeled destination MUST be a Symbol.

Example:

```phalcom
const options = {
    timeout: 10,
    retries: 3,
}

foo(**options)
```

is valid because the bare Map keys are Symbol keys.

But:

```phalcom
const options = {
    ["timeout"]: 10,
}

foo(**options)
```

fails because `"timeout"` is a String key, not a Symbol.

No coercion is attempted.

Full Symbol and label literal syntax is specified elsewhere.

---

## 12. Stable Encounter Order Requirement

Ordered labeled expansion requires a stable encounter order.

The order observed during `**` contributes directly to the ordered labeled lane and may therefore affect selector identity.

Consequently:

- Tuple supports `**` using its labeled-lane order;
- Record supports `**` using preserved field encounter order;
- Map supports `**` using insertion encounter order;
- a future `OrderedMap` MAY support `**` if it has stable encounter order;
- a future `HashMap` with unspecified encounter order MUST NOT support `**` into an ordered labeled destination.

This restriction applies even if every `HashMap` key is a Symbol.

The following MUST NOT be permitted if `HashMap` encounter order is unspecified:

```phalcom
foo(**hashMap)
```

because selector identity must not depend on hash seed, table layout, rehashing, or other unspecified implementation details.

---

## 13. Target-Sensitive `**`

`**` means association/labeled projection, but validation depends on the destination.

### 13.1 Call, Tuple, and Record destinations

For these destinations:

```text
every contributed key MUST be Symbol
encounter order MUST be stable
duplicate labels/fields MUST fail
```

Example:

```phalcom
#{
    **someMap,
}
```

requires every key contributed by `someMap` to be a Symbol.

### 13.2 Map destination

For a Map destination, `**source` contributes arbitrary Map associations in source encounter order.

Keys need only satisfy the Map's key requirements, such as hashability.

Thus this is valid:

```phalcom
{
    **{
        ["name"]: "Ada",
        [42]: "answer",
    },
}
```

even though those keys could not be expanded into an argument pack.

Map duplicate-key literal semantics are specified in the Map/literal specification rather than here.

---

## 14. Construction Machinery Shared by Calls and Tuples

Call construction and Tuple construction use the same two-lane composition model.

Example:

```phalcom
const base = (
    1,
    2,
    timeout: 10,
)

const extended = (
    0,
    ***base,
    retries: 3,
)
```

produces the same lane structure that would be constructed by the corresponding call syntax:

```text
positional:
    0
    1
    2

labeled:
    timeout → 10
    retries → 3
```

Implementations SHOULD reuse the same semantic argument-pack/lane builder logic for calls and Tuple construction where practical.

This is an implementation recommendation, not a required runtime representation identity.

---

## 15. Expansion in Collection Literals

`*` and `**` are also used in collection construction according to destination shape.

### 15.1 List and Set

Element-oriented literals use `*`.

Example:

```phalcom
[
    0,
    *numbers,
]
```

Example:

```phalcom
{
    1,
    2,
    *otherValues,
}
```

where the latter is a Set literal under the collection-literal grammar.

### 15.2 Map

Association-oriented Map literals use `**`.

Example:

```phalcom
{
    **defaults,
    timeout: 10,
}
```

### 15.3 Record

Record literals use `**` for labeled field expansion.

Example:

```phalcom
#{
    name: "Ada",
    **metadata,
}
```

Record fields contributed through expansion MUST have Symbol keys and stable encounter order.

---

## 16. Variadic Capture

Variadic capture mirrors expansion.

### 16.1 Positional capture

```phalcom
fn positional(*args) {
    ...
}
```

captures the positional lane into a positional-only Tuple value.

### 16.2 Labeled capture

```phalcom
fn labeled(**args) {
    ...
}
```

captures the ordered labeled lane into a labeled-only Tuple value.

### 16.3 Complete capture

```phalcom
fn complete(***args) {
    ...
}
```

captures both lanes into a Tuple.

The capture operators are semantically dual to their corresponding expansion operators.

---

## 17. Forwarding

Variadic capture and expansion permit lossless forwarding.

Example:

```phalcom
fn proxy(***args) {
    target(***args)
}
```

The forwarded call MUST preserve:

- positional values and positional order;
- labels;
- labeled values;
- labeled order.

Selective forwarding is also valid:

```phalcom
fn proxy(***args) {
    log(*args)
    target(**args)
}
```

subject to the existence of the target selectors implied by the resulting argument packs.

---

## 18. Empty Product Interaction

Phalcom's canonical zero-arity product is `Unit`, written `()`.

A Tuple capture with zero positional and zero labeled arguments normalizes to the same zero-product value.

Thus:

```phalcom
fn proxy(***args) {
    target(***args)
}

proxy()
```

captures an empty product, and forwarding contributes nothing.

Conceptually:

```phalcom
foo(***())
```

is equivalent to:

```phalcom
foo()
```

The detailed zero-product normalization rules, including the empty Record `#{}`, are specified in the product normalization and Unit specification.

---

## 19. Eager Expansion and Infinite Sources

Positional expansion from a general Iterable is eager with respect to argument-pack completion.

For:

```phalcom
foo(*source)
```

the implementation must know the final positional arity before it can derive the selector and dispatch the call.

Therefore the source must be consumed until exhaustion before dispatch.

Conceptually:

```text
evaluate source
    ↓
repeatedly consume elements
    ↓
append positional values
    ↓
observe exhaustion
    ↓
derive final positional arity
    ↓
derive selector
    ↓
lookup and execute method
```

A method is not invoked partially while expansion is still consuming its source.

---

## 20. Provably Unbounded Sources

Applying an eager exhaustor to a source that is statically provable to be unbounded is invalid.

The implementation MUST diagnose such a case before executing the program when the unboundedness is statically known.

Examples that MUST be rejected statically:

```phalcom
foo(*(0..))
```

```phalcom
[
    *(0..),
]
```

and, under the corresponding materialization specification:

```phalcom
(0..).toList
```

The diagnostic is semantic, not an arbitrary runtime element limit.

### 20.1 Bounded transformations

An operation that introduces a statically finite bound may make the source valid:

```phalcom
foo(*(0..).take(3))
```

or equivalently according to final method precedence/syntax:

```phalcom
foo(*((0..).take(3)))
```

The exact surface grouping rules are defined by the expression grammar.

### 20.2 Unknown boundedness

A source whose termination cannot be proven remains legal.

Example:

```phalcom
someIterator.toList
```

or:

```phalcom
foo(*someIterator)
```

If the source never exhausts at runtime, the eager operation does not complete unless interrupted or failed.

Phalcom does not impose an implicit truncation limit.

---

## 21. Boundedness Metadata

The language specification does not require boundedness to be represented as a public type hierarchy.

An implementation MAY track boundedness as compiler semantic metadata.

At minimum, the semantic model distinguishes:

```text
statically bounded
provably unbounded
unknown-boundedness
```

Known iterator transformations SHOULD propagate this information where it can be determined soundly.

Examples:

```text
map over provably unbounded source
    → provably unbounded

filter over provably unbounded source
    → generally not sufficient to prove boundedness;
      exhaustibility semantics remain source-dependent

take(n) with finite n
    → statically bounded
```

Precise boundedness propagation for every iterator combinator is deferred to the iterator specification.

The implementation MUST NOT reject an unknown-boundedness source merely because it might be infinite.

---

## 22. Evaluation Failures During Expansion

Expansion may fail while evaluating or consuming its source.

If expansion fails before argument-pack completion:

- selector derivation does not occur;
- method lookup does not occur;
- the target method is not invoked.

Examples of failure include:

- source expression evaluation failure;
- iterator failure during `*`;
- invalid non-Symbol key during labeled `**`;
- duplicate label after composition;
- other target-specific validation failure.

The original exception/error propagation model applies.

---

## 23. Static Versus Dynamic Validation

An implementation SHOULD perform validation statically whenever it can prove the relevant property.

Examples include:

- syntactically duplicated explicit labels;
- duplicate labels from statically known Tuple/Record structures;
- invalid statically known Map key types for labeled expansion;
- `**` from a type whose encounter order is known to be unspecified;
- eager expansion of a provably unbounded source.

When a property depends on runtime data, validation occurs while constructing the argument pack or target collection.

Static diagnostics MUST preserve the same semantics as the equivalent dynamic check.

---

## 24. No Type-Based Dispatch Through Expansion

Expansion changes the argument pack; it does not introduce type-based overload selection.

After all expansion is complete, dispatch still depends solely on the resulting selector identity.

For example, if two `**` sources contribute the same values but different label orders, they may derive different selectors because labeled order is part of selector identity.

The runtime types of the contributed values do not participate in selector selection.

---

## 25. Examples

### 25.1 Mixed positional and labeled expansion

```phalcom
const positional = [1, 2]

const tuple = (
    3,
    timeout: 10,
)

const options = #{
    retries: 2,
}

foo(
    0,
    *positional,
    ***tuple,
    **options,
)
```

Resulting lanes:

```text
positional:
    0
    1
    2
    3

labeled:
    timeout → 10
    retries → 2
```

### 25.2 Map as a labeled source

```phalcom
const options = {
    timeout: 10,
    retries: 2,
}

foo(**options)
```

is valid because the Map preserves insertion encounter order and the bare keys are Symbols.

### 25.3 Invalid String key

```phalcom
const options = {
    ["timeout"]: 10,
}

foo(**options)
```

fails because labeled expansion requires Symbol keys.

### 25.4 Invalid unspecified-order source

```phalcom
const options = HashMap{
    timeout: 10,
    retries: 2,
}

foo(**options)
```

is invalid if `HashMap` encounter order is unspecified.

### 25.5 Duplicate after expansion

```phalcom
const defaults = (
    timeout: 5,
)

foo(
    **defaults,
    timeout: 10,
)
```

fails before lookup.

### 25.6 Source boundary violation

```phalcom
foo(
    timeout: 10,
    *items,
)
```

is invalid source syntax.

### 25.7 Lossless forwarding

```phalcom
fn proxy(***args) {
    target(***args)
}
```

preserves both lanes exactly.

---

## 26. Implementation Model

A conforming implementation may use any internal representation that preserves the semantics above.

A straightforward model is:

```text
ArgumentPackBuilder {
    positional: Vec<Value>
    labeled:    Vec<(Symbol, Value)>
    labelsSeen: Set<Symbol>
    phase:      Positional | Labeled
}
```

This representation is illustrative rather than normative.

The builder may:

1. evaluate each source expression in lexical order;
2. project the requested lane(s);
3. append positional contributions to the positional buffer;
4. append labeled contributions to the labeled buffer;
5. detect duplicate labels;
6. finalize the pack;
7. derive selector identity from final lane structure.

For `***`, the implementation MUST append each projected sub-lane to its corresponding destination lane rather than flattening both lanes into one lexical sequence.

---

## 27. Deferred Issues

The following are intentionally deferred and MUST NOT be inferred from this specification:

1. the final generic capability/protocol names that may generalize `*`, `**`, or `***`;
2. whether arbitrary user-defined types may implement expansion capabilities;
3. full Tuple indexing, slicing, equality, hashing, and reflection APIs;
4. full Record typing, row polymorphism, open-row semantics, and dynamic Record-shape typing;
5. complete Symbol lexical grammar and all valid bare-label spellings;
6. complete Map literal duplicate-key semantics and Map merge APIs;
7. exact Set encounter-order semantics beyond what is required by future `*Set`;
8. the full iterator protocol and boundedness propagation rules;
9. optimizer strategies for materializing or streaming expansion sources;
10. exact diagnostic classes/messages for expansion failures;
11. whether a separate first-class runtime `ArgumentPack` object is exposed reflectively.

Future specifications may refine these areas but MUST preserve the ratified semantics in this document unless explicitly superseded.

---

## 28. Conformance Summary

A conforming Phalcom implementation MUST satisfy the following core laws:

```text
Argument pack
    = ordered positional lane
    + ordered labeled lane

selector identity
    includes positional arity
    and ordered label sequence

*source
    → positional/element projection

**source
    → labeled/association projection

***source
    → complete two-lane projection

lane composition
    → lane-wise

source evaluation
    → lexical

duplicate labels
    → error

implicit String → Symbol conversion
    → never

ordered labeled expansion
    → requires stable encounter order

HashMap with unspecified encounter order
    → cannot feed ordered labeled expansion

calls and Tuples
    → share the same lane-composition semantics

variadic capture
    → mirrors *, **, ***

provably unbounded eager expansion
    → static diagnostic

unknown-boundedness eager expansion
    → legal; may fail to terminate dynamically
```
