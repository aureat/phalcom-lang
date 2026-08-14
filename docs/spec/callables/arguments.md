# Arguments, parameters, rest, and spread

[Callables](README.md) · [Dispatch and lowering](dispatch.md) · [Runtime and activation](runtime.md) · [Function](function.md) · [Method](method.md) · [Closure](closure.md) · [BoundMethod](bound-method.md) · [Family](family.md)

This chapter separates the shape a caller supplies from the shape a callable
accepts. That separation is what permits one Function gateway,
`call(***arguments)`, without pretending that every Function accepts labels,
rest, or every possible arity.

## 1. One notation, three lanes of meaning

Phalcom uses only:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

```phalcom
target(*values)
target(**labels)
target(***arguments)

collect(*rest) { body }
collect(**rest) { body }
collect(***rest) { body }

|head, *tail| { body }
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator.

`*`, `**`, and `***` describe different data, not three spellings of one
untyped sequence:

| Marker | Carries | Typical use |
| --- | --- | --- |
| `*` | ordered positional values | positional rest and positional expansion |
| `**` | ordered labeled values | labeled rest and labeled expansion |
| `***` | both lanes together | complete forwarding and complete capture |

## 2. Actual argument shape

An actual call has an **argument shape** independent of any callee:

```text
ordered positional lane
ordered label Symbols
ordered labeled-value lane, aligned with labels
```

For example:

```phalcom
target(1, 2, to: point, duration: 10)
```

has two positional values, labels `to` and `duration` in that order, and two
corresponding labeled values. Labels belong to the shape because they are part
of selector identity.

An argument shape does not answer whether the target accepts it. It is merely
the complete result of evaluating the call expression. This prevents dispatch
from depending on a callee-specific interpretation of a pack.

## 3. Parameter shape and layout

The implementation and specification distinguish four concepts:

```text
ArgumentShape      actual lanes supplied by one call
ParameterShape     lanes and rest modes accepted by one callable
ParameterLayout    local/frame slots used after binding
BindingPlan        structural match from actual shape to layout
```

Only `ArgumentShape × ParameterShape` decides acceptance. `ParameterLayout`
must not leak into selector lookup, and a binding plan must not invoke user
code or inspect argument value types.

For an exact Method, fixed positional slots and fixed labels are encoded into
its selector. For a rest-capable Method, normalized rest metadata records the
fixed prefix and capture mode. In the VM, `RestLayout` deliberately stores this
as metadata rather than reparsing a wildcard selector string.

```rust
pub enum RestMode {
    Positional { .. },
    Labeled { .. },
    Split { .. },
    Complete { .. },
}
```

See [`method/mod.rs`](../../../phalcom-core/src/method/mod.rs). The public
syntax and exact capture values remain the language contract; this enum is an
implementation strategy for retaining that information after parsing.

## 4. Spread and dynamic packs

Spread is evaluated in source order. A call may mix ordinary arguments and
expansions:

```phalcom
target(before(), *values(), marker: after(), ***more())
```

Each expression runs exactly once from left to right. The final positional lane
preserves contribution order. The final labeled lane preserves label order and
must reject duplicate labels according to argument-pack rules.

Static calls encode a selector directly. Calls containing expansion use an
internal builder and `InvokePack`; the builder is a transport detail, not a
public argument value. At the send boundary the VM constructs the complete
selector from the assembled lanes and enters the same lookup path as a static
send. [Dispatch and lowering](dispatch.md#3-dynamic-argument-shapes) describes
that boundary.

The source `***arguments` means forward the complete shape. It does not demand
that the runtime allocate a Tuple merely to call a Function. It also does not
turn a complete pack into a universal parameter binder.

## 5. Method rest parameters

Methods may declare positional, labeled, split, or complete rest according to
their parameter declaration:

```phalcom
sum(*rest) { body }
configure(**rest) { body }
join(*items, **options) { body }
forward(***arguments) { body }
```

The resolver first searches for an exact selector across the full inheritance
chain. Only after that complete exact search misses does it search for a
compatible rest Method in the base-name family. A rest Method is accepted by
shape only; it does not perform dynamic type tests while resolving.

At most one rest-capable Method occupies a given base family on one class.
Exact Methods in that family may still coexist. This avoids ambiguous rest
specificity rules while retaining ordinary overload identity.

Rest capture is canonical product construction:

```text
empty captured product       → ()
non-empty positional capture → Tuple
non-empty labeled capture    → labeled Tuple/complete product
```

The current runtime uses `finish_tuple` at this representation boundary, which
normalizes an empty product to `Value::Unit` rather than allocating an empty
Tuple. See [`product.rs`](../../../phalcom-core/src/product.rs).

## 6. Closure parameters are intentionally narrower

Closure literals currently accept only fixed positional parameters and one
optional terminal positional rest parameter:

```phalcom
|| { body }
|x| { body }
|x, y| { body }
|head, *tail| { body }
```

They reject labeled parameters, `**rest`, `***rest`, multiple positional-rest
parameters, and fixed parameters after `*rest`.

```phalcom
|**labels| { body }          // rejected
|***arguments| { body }      // rejected
|head, *tail, last| { body } // rejected
```

A Closure therefore validates two conditions at activation:

```text
labeled lane is empty
positional count is exactly fixed count, or at least fixed count with *rest
```

For `|head, *tail|`, zero residual positional values bind `tail` to `()` and
one or more bind it to a Tuple. The rest value is never a List. Outgoing calls
may still use `*`, `**`, and `***`; a Closure simply rejects a resulting
non-empty labeled lane. See [Closure](closure.md#3-parameters-and-rest).

## 7. The shared Function gateway

Every concrete Function enters through:

```phalcom
call(***arguments)
```

The gateway is a complete-rest Method on the sealed `Function` root. A call
such as:

```phalcom
f(1, to: point)
```

lowers to the ordinary selector `call(_,to)`. Exact lookup does not need a
finite `call(_,to)` declaration: after the exact miss, the Function root's
complete-rest `call(***)` Method accepts the actual shape and routes to the
concrete representation.

This is transport, not acceptance:

| Concrete Function | Receives through gateway | Accepts |
| --- | --- | --- |
| Closure | complete argument shape | positional-only Closure shape |
| BoundMethod | complete argument shape | wrapped exact Method shape |
| Family | complete argument shape | shape needed to route and then target accepts |

The gateway permits a single forwarding protocol without generating finite
`call` overloads for every arity and label combination.

## 8. `callWith`

For a Function:

```phalcom
f.callWith(arguments)
```

is exactly:

```phalcom
f(***arguments)
```

It is not a List-based calling convention and it is not a second parameter
binder. It preserves both lanes and delegates to the same Function gateway.
The current native gateway reconstructs a shaped view from the supplied
complete pack before calling `activate_function`; see
[`primitive/block.rs`](../../../phalcom-core/src/primitive/block.rs).

## 9. Exact invocation and reflective forwarding

The same complete-shape rule applies to:

```phalcom
method.invokeOn(receiver, ***arguments)
receiver.perform(selector, ***arguments)
```

`invokeOn` removes its explicit receiver argument from the transported shape,
checks the exact Method's acceptance and any field representation guard, then
activates that Method without selector redispatch. `perform` removes its selector argument,
then performs ordinary shaped dispatch on the supplied receiver. Neither
operation may flatten labels into a positional List. See
[Reflection and exact invocation](reflection.md).

## 10. Related chapters

- [Dispatch and lowering](dispatch.md) — selector construction and two-pass lookup
- [Runtime and activation](runtime.md) — allocation-free shape transport
- [Function](function.md) — Function-level call contract
- [Method](method.md) — Method rest and exact invocation
- [Closure](closure.md) — positional-only Closure acceptance
- [BoundMethod](bound-method.md) and [Family](family.md) — forwarding targets
