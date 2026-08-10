# Calls, Parameters, Rest, and Spread

[← Callable object model](02-callable-object-model.md) · [Overview](README.md) · [Reflection →](04-reflection-and-object-protocol.md)

---

## 1. Argument lanes

A Phalcom call has two ordered argument lanes:

1. a positional lane;
2. a labeled lane whose labels retain source/pack order.

A complete argument shape consists of:

```text
ordered positional values
ordered labels
labeled values corresponding 1:1 with those labels
```

Dispatch is based on selector/argument shape, not on runtime argument value types.

---

## 2. Spread syntax

Spread contributes values to an outgoing call.

### 2.1 Positional spread

```phalcom
target(*source)
```

contributes positional values.

### 2.2 Labeled spread

```phalcom
target(**source)
```

contributes labeled values.

### 2.3 Complete spread

```phalcom
target(***source)
```

contributes a complete argument pack, preserving both lanes.

### 2.4 No ellipsis spread

A postfix ellipsis is not spread syntax.

```text
arguments...
```

does not mean spread.

Only `*`, `**`, and `***` have spread/rest meaning.

---

## 3. Rest parameters

Rest parameters capture residual incoming arguments.

### 3.1 Positional rest

```phalcom
collect(_ first, *rest) {
    ...
}
```

captures residual positional arguments.

### 3.2 Labeled rest

A Method may declare labeled rest:

```phalcom
configure(**options) {
    ...
}
```

which captures residual labeled arguments.

### 3.3 Complete rest

A Method may declare complete rest:

```phalcom
forward(***arguments) {
    ...
}
```

which captures the complete residual pack.

### 3.4 Split rest

A Method may use positional and labeled rest together where permitted by the ordinary Method parameter grammar:

```phalcom
dispatch(_ first, *rest, **options) {
    ...
}
```

The declaration grammar determines the fixed positional/labeled parameters. The rest markers themselves always have the lane meanings defined in this chapter.

---

## 4. Canonical rest capture values

Rest captures use canonical product values.

```text
empty residual capture      → ()
non-empty residual capture  → Tuple
```

This rule applies to positional, labeled, split, and complete rest captures.

Rest capture does not use:

```text
List
Map
Record
a public mutable argument-pack object
```

A non-empty Tuple preserves the lane/label information required by the captured rest shape.

---

## 5. Closure parameters

Closures currently support:

```text
zero or more fixed positional parameters
optional one terminal positional-rest parameter
```

Examples:

```phalcom
|| {
    ...
}
```

```phalcom
|value| {
    ...
}
```

```phalcom
|head, *tail| {
    ...
}
```

A Closure must reject:

```text
labeled parameters
**rest
***rest
multiple *rest parameters
fixed parameters after *rest
```

### 5.1 Closure rest value

For:

```phalcom
const f = |head, *tail| {
    tail
}
```

calling:

```phalcom
f(1)
```

binds:

```phalcom
tail == ()
```

while:

```phalcom
f(1, 2, 3)
```

binds `tail` to a Tuple containing the residual positional values.

---

## 6. Spread into Closures

Outgoing spread syntax is general even though Closure parameter acceptance is currently positional-only.

These are syntactically meaningful:

```phalcom
f(*values)
f(**fields)
f(***arguments)
```

The Closure accepts or rejects the **resulting argument shape**.

Therefore:

- `f(*values)` succeeds if the resulting positional count satisfies the Closure;
- `f(***arguments)` succeeds if the complete pack has no non-empty labeled lane and satisfies the positional shape;
- a non-empty labeled contribution from `**` or `***` is rejected by the current Closure parameter model.

The error concerns argument-shape compatibility, not spread syntax.

---

## 7. Function call gateway

The canonical Function call gateway is:

```phalcom
call(***arguments)
```

This declaration means the gateway accepts complete argument shape.

It does **not** imply that the implementation must allocate a Tuple or public pack before every Function call.

The concrete Function validates the supplied shape according to its own semantics:

```text
Closure       → Closure parameter shape
BoundMethod   → underlying exact Method parameter shape
Family        → Family routing semantics
```

---

## 8. Exact and rest Method resolution

Method dispatch uses two ordered passes.

### Pass 1 — exact lookup

The concrete selector is looked up across the applicable inheritance chain.

### Pass 2 — rest-family fallback

Only if exact lookup fails does dispatch search compatible rest Methods in the base family.

Therefore an inherited exact Method beats a subclass rest fallback.

Conceptually:

```text
exact across full hierarchy
    ↓ miss
compatible rest across full hierarchy
    ↓ miss
doesNotUnderstand
```

A rest match compares only structural argument shape and Method rest metadata. It does not inspect runtime argument value types.

---

## 9. Rest-family uniqueness

Within one class/behavior and one base selector family, there may be at most one rest-capable Method.

Exact Methods remain independently definable.

This avoids ambiguous overlapping wildcard/rest patterns and prevents the language from requiring a rest-specificity ordering.

---

## 10. Static and dynamic outgoing calls

A call with statically known argument shape may use the normal exact selector fast path.

A call containing spread or otherwise dynamically assembled shape may build/transport an argument pack at runtime.

Both forms must preserve:

- lexical evaluation order;
- positional order;
- labeled order;
- duplicate-label validation;
- the same exact→rest dispatch semantics;
- the same `doesNotUnderstand` behavior on final miss.

---

## 11. `callWith`

For Functions:

```phalcom
f.callWith(pack)
```

is semantically identical to:

```phalcom
f(***pack)
```

It must preserve complete argument shape, including ordered labels.

`callWith` is not a positional-List calling convention.

---

## 12. Exact Method invocation

```phalcom
method.invokeOn(receiver, ***arguments)
```

uses complete argument transport and the exact Method's parameter shape.

It does not perform selector redispatch.

Receiver compatibility is checked before execution.

---

## 13. BoundMethod invocation

A BoundMethod call uses:

```text
stored exact Method
stored receiver
actual call shape
```

The actual shape is matched against the Method's parameter shape exactly as if that Method had already been selected by normal dispatch.

---

## 14. Native Methods and rest

Native/primitive Methods may declare and implement all supported Method rest modes:

```text
*
**
***
split positional + labeled rest
```

Native Method rest has the same language semantics as bytecode Method rest.

The implementation representation may differ, but observable acceptance, capture, ordering, and exact→rest resolution must not differ.

---

## 15. Boundedness of positional spread

Where the language's argument-pack boundedness rules apply, positional spread must obey them regardless of whether the final receiver is a normal Method, Closure, BoundMethod, or Family.

The callable redesign does not create a separate unbounded-expansion loophole.

---

## 16. Selector terminology

A **selector** is the full message identity, including the applicable positional/labeled shape.

A **base family** is the selector name used to group exact/rest-compatible selector shapes.

A Family reference and a reified Method are distinct:

```text
Family
    callable late-dispatch reference

Method
    exact reified behavior
```

This distinction is further specified in [Reflection and object protocol](04-reflection-and-object-protocol.md).
