# Family

[Callables](README.md) · [Dispatch and lowering](dispatch.md) · [Arguments and rest](arguments.md) · [Runtime and activation](runtime.md) · [Reflection](reflection.md) · [Function](function.md) · [Method](method.md)

Family is a sealed Function created by a callable-reference expression. It
stores one receiver and one selector specification. It is not a Method:
construction does not resolve or authorize a target implementation.

## 1. Reference syntax

The `&` prefix introduces a callable reference. A dot reference binds an
ordinary receiver, including a class object:

```phalcom
&object.method
&object.method()
&object.method(_)
&object.method(_, _, debug)
&object.method(...)
&object.method...
&object.method(_, _, ...)
&object.method(..., _, param)
&object.method=(_)
&object.method=
&object[...]
&object[...]=(_)
&object[...]=

&Fiber.new()
&Fiber.new(_)
```

The written selector shape determines the capability. A bare name is an exact
Getter; `method()` and `method(_)` are exact Methods; `method=(_)` is an exact
Setter. `method=` is the Getter-or-Setter accessor pattern, `method(...)` is a
Method-only pattern, and `method...` is the complete named family. Operator
references retain Method identity. Bracket references use SubscriptGet and
SubscriptSet; `[...]=` is the all-subscript-accessor pattern.

Selector labels are labels, not destructuring bindings. The receiver expression
is evaluated once and retained by the resulting Family.

Associated callable families use `::` only after `&`:

```phalcom
&Option::Some...
&Option::Some(_)
&Option::Some(...)
```

Direct associated invocation remains a normal associated operation:

```phalcom
Option::Some(42)
```

Class-side methods remain ordinary dot sends, so `Fiber.new { ... }` and
`Fiber.new()` are not associated lookups.

Exact Getter families are activated by `family.get()` or `family.value`.
`family()` remains Method-kind and does not fall back to Getter. Named setter
aliases are `family.set(rhs)` and `family.value = rhs`.

## 2. Exact Family calls

Exact Families retain selector identity and derive no replacement selector
from the call's labels. The incoming argument shape must satisfy that exact
selector. The runtime then looks up that selector on the stored receiver and
activates the selected Method using ordinary access and rest rules.

```phalcom
let family = &object.render(_)
family(value)
```

Replacing `render(_)` after Family construction is visible to the next call.
The Family did not capture a Method; it captured selector identity.

## 3. Pattern Family calls

Pattern Families retain an immutable structural predicate and the receiver:

```text
receiver: bound target
pattern: SelectorPattern
```

The predicate is created at reference construction, but method lookup remains
live. At each call the incoming shape is normalized, matched against the
predicate, and dispatched through the current method table on the stored
receiver. Construction does not invoke the receiver, probe a selector,
allocate a Message, or call `doesNotUnderstand`.

```phalcom
let family = &object.render(...)
family(value)
```

Named `AnyNamed` patterns consider getter, setter, and method forms; exact
getter and setter forms retain their own zero- and one-value shapes. Subscript
patterns retain their index labels and assigned-value lane. Replacing or
adding a matching method after Family construction is visible to the next
call.

A call with no matching route reaches ordinary `doesNotUnderstand` at the
target call boundary.

Subscript families have both direct and explicit tuple-shaped APIs:

```phalcom
family[index, debug: mode]
family[index, debug: mode] = rhs
family.get((index, debug: mode))
family.set((index, debug: mode), rhs)
```

The first explicit API argument is a real Tuple/product. Its positional and
labeled values retain their source order; the setter RHS remains outside that
product as the one dedicated value lane.

## 4. Function and reflection surface

Family participates in the shared Function call gateway. MethodFamily is a
separate immutable reflection payload returned by `Behavior#>>` for a pattern
and exposes:

```text
family.selectors
family.size
family.methodFor(selector)
family.bind(receiver)
```

`selectors` returns a fresh List of canonical captured selectors. `size` counts
exact and rest routes. `methodFor` returns only a captured route and applies
the current caller's access authority. `bind` stores the snapshot plus a
receiver; it does not inspect receiver behavior or re-capture routes.

## 5. Mutation and dispatch law

```text
Family construction
    -> immutable receiver + selector specification
    -> exact lookup or live pattern selection at call
    -> exact Method activation
```

The structural predicate remains immutable. The stored receiver determines the
current method table, target layout, and dynamic self inside the selected body,
so method-table changes can change the selected route. Ordinary sends inside
the body remain dynamically dispatched on that receiver. Lexical super keeps
the selected Method's defining holder.

## 6. Implementation boundary

The compiler emits `MakeFamily` for general bound references. Associated exact
references may lower to a resolved target or constructor thunk when the
semantic product proves that target; associated family references retain their
associated-family descriptor. Escaping references, structural patterns,
mismatched shapes, and dynamic packs retain Family construction and the shared
runtime gateway.

See `vm/send.rs`, `heap/object.rs`, `primitive/method_family.rs`, and
`compiler/lib/expr.rs` for the implementation paths.

## 7. Related chapters

- Function — shared call gateway
- Dispatch and lowering — selector identity and target sends
- Method — exact behavior and arbitrary receiver guards
- Reflection — MethodFamily and exact invocation
- Runtime and activation — direct routing implementation
