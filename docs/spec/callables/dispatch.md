# Dispatch and lowering

[Callables](README.md) · [Arguments and rest](arguments.md) · [Runtime and activation](runtime.md) · [Method](method.md) · [Function](function.md) · [Family](family.md)

This chapter specifies how source call syntax becomes a selector-bearing
message send, how the receiver's hierarchy is searched, and where a Function
activation begins. Dispatch selects behavior; activation runs the selected
behavior. The distinction is central to exact Method invocation and Families.

## 1. Three source forms

Phalcom has three source situations that can look similar.

```phalcom
receiver.move(to: point)  // explicit receiver method send
helper(value)             // lexical value call or implicit-self method send
f(value)                  // value application when f resolves as a value
```

The first form always sends `move` to its explicit receiver. The second and
third use bare-name resolution before lowering.

For an unqualified `name(arguments)` expression, resolution is ordered:

1. local binding or parameter;
2. captured lexical binding;
3. module/global binding;
4. otherwise, implicit `self` when the surrounding context has one.

Cases 1–3 compile as an application of the resolved value: `name.call(...)`.
Case 4 compiles as a normal method send to `self`: `self.name(...)`. This is
why source spelling alone does not decide whether `helper()` is a call to a
Function or a message to the current receiver.

```phalcom
class Example {
    helper { 10 }

    useMethod {
        helper()           // self.helper(), not self.call()
    }

    useValue {
        const helper = || { 20 }
        helper()           // helper.call()
    }
}
```

The same rule makes lexical bindings win over implicit receiver methods. It
prevents a method added to a class from silently changing a previously lexical
call site.

## 2. Value application lowering

For a resolved value `f`, application is ordinary message syntax:

```phalcom
f(a, label: b)
```

means:

```phalcom
f.call(a, label: b)
```

The resulting selector records argument shape. With one positional and one
labeled argument, it is conceptually `call(_,label)`, not merely a string named
`call` plus an integer arity. A no-argument call is `call()`.

For a statically shaped call, the compiler evaluates the receiver and argument
expressions in source order, encodes the selector, and emits `Invoke`. The
current compiler path is compact and direct:

```rust
let selector = encode_selector("call", &labels, SignatureKind::Method(arity));
let selector_sym = self.vm.interner.intern(&selector);
let selector_idx = self.add_constant(Value::Symbol(selector_sym));
self.emit(Bytecode::Invoke(arity, selector_idx), call.range);
```

This excerpt is from
[`compiler/lib/expr.rs`](../../../phalcom-core/src/compiler/lib/expr.rs).
It does not bypass method lookup. `Invoke` is the bytecode representation of
the ordinary send described above.

## 3. Dynamic argument shapes

Spread requires the final shape to be assembled at run time. A dynamic call
uses a private argument-pack builder, records positional and labeled
contributions in source evaluation order, and emits `InvokePack` with base name
`call` for value application.

```text
evaluate receiver
evaluate and append pack items left to right
construct selector from actual lanes
send that selector to receiver
```

The dynamic path is semantically identical to the static path. It must enforce
the same duplicate-label rules, selector shape, exact-then-rest lookup, access
checks, and final miss behavior. The VM implementation takes the builder apart
only at the send boundary, extends the current stack window in lane order, and
uses the same dynamic selector construction as ordinary spread sends. See
[Arguments and rest](arguments.md#4-spread-and-dynamic-packs).

## 4. Selector identity

A selector is the complete message identity: base name, ordered positional
slots, ordered labels, and selector kind. Phalcom's canonical rendered form is
comma-separated:

```text
zero()                   no-argument method
sum(_,_)                 two positional arguments
move(_,to,duration)      one positional, then two labeled arguments
name                     getter
name=(put)               setter
```

The implementation uses an interned Symbol for this complete identity. The
Rust selector encoder makes the positional/labeled distinction explicit:

```rust
SignatureKind::Method(0) => format!("{name}()"),
SignatureKind::Method(_) => format!("{name}({})", comma_form_slots(labels)),
```

See [`method/mod.rs`](../../../phalcom-core/src/method/mod.rs). Labels are
selector identity, not only parameter names. Consequently, default arguments
and label-erasing call rewrites are not implicit features of dispatch.

## 5. Ordinary lookup

Given a receiver and complete selector, ordinary dispatch proceeds in this
order:

```text
receiver runtime class
    ↓
exact selector lookup through full superclass chain
    ↓ on miss
compatible rest-method lookup through full superclass chain
    ↓ on miss
doesNotUnderstand(_)
```

The exact pass completes across the whole hierarchy before the rest pass
starts. Therefore an inherited exact Method wins over a rest-capable Method on
a more-derived class. Rest fallback never parses wildcard text out of a
selector; it consults normalized rest metadata indexed by base family.

The current shaped-dispatch path makes that ordering visible:

```rust
if let Some(method) = receiver.lookup_method(self, selector) {
    // exact selector selected
} else if let Some(method) = self.lookup_rest_method(/* base + shape */) {
    // compatible rest Method selected
} else {
    self.forward_does_not_understand_as(/* original selector */)?;
}
```

See [`vm/send.rs`](../../../phalcom-core/src/vm/send.rs) and
[`vm/dispatch.rs`](../../../phalcom-core/src/vm/dispatch.rs). A visibility
failure for an existing Method is an access error, not a message miss.

## 6. Lookup is not activation

Normal dispatch selects behavior from the receiver's hierarchy. Exact Method
invocation instead begins with behavior that has already been selected:

```phalcom
method.invokeOn(receiver, ***arguments)
```

It checks access, parameter acceptance, and any field representation guard, then
executes that exact Method. It does not look up the Method's selector again.
However, ordinary sends *inside* that Method still dispatch dynamically on the
supplied receiver. [Method](method.md#4-exact-invocation) and
[Reflection](reflection.md) cover the distinction in detail.

Similarly, a BoundMethod stores an exact Method and receiver, whereas an exact
Family stores a selector and receiver and a pattern Family stores an immutable
structural predicate with its receiver. Exact lookup or live pattern routing
returns to ordinary activation after its routing step. Family construction
itself never probes receiver behavior.

## 7. `super` is a different send origin

`super.name(arguments)` is not a normal lookup from the receiver's class. The
compiler emits `SuperSend` or `SuperSendPack` with the lexically defining
holder. The dynamic receiver remains `self`, but lookup begins at the defining
holder's superclass. This is why calling a Method exactly on a subclass does
not move its `super` anchor.

The compiler keeps this distinction before any ordinary-send optimization. In
particular, a `super` send cannot be mistaken for a receiver-class-specialized
send. See [Execution contexts](execution.md#3-self-and-super).

## 8. Misses and forwarding

Only a real terminal lookup miss is reified as a `Message` and sent to
`doesNotUnderstand(_)`. The original selector and argument values are retained
for that hook. `doesNotUnderstand` is not a normal implementation mechanism
for Function calls, BoundMethod calls, Family routing, `methodFor`, or
`respondsTo`.

That boundary matters for proxies: a proxy may intentionally handle a real
miss, but reflection and callable routing must not manufacture misses merely to
recover information the compiler and VM already possess.

## 9. Dispatch cache boundary

The bytecode runtime has per-site cache storage keyed by receiver class, Method
handle, and VM world version. A cache may accelerate exact or selected rest
lookup, but it is an implementation cache only. A cache hit must be
observationally identical to walking the hierarchy, including method
redefinition and authorization behavior. The cache cannot make a failed lookup
permanent because `doesNotUnderstand` and open method dictionaries remain
observable.

## 10. Reflection operator dispatch

`>>(_)` remains an ordinary selector and ordinary dynamic dispatch. Behavior
implements it as reflective exact/pattern extraction; Int implements its own
shift operation. The operator spelling does not grant either implementation a
special parser or VM dispatch path, and reflection must preserve the original
caller authority when Behavior extracts a Method or MethodFamily.

## 11. Related chapters

- [Arguments and rest](arguments.md) — shape construction and rest acceptance
- [Runtime and activation](runtime.md) — what happens after selection
- [Method](method.md) — exact reified behavior
- [Function](function.md) — shared `call(***arguments)` gateway
- [Family](family.md) — routing before ordinary target dispatch
- [Reflection and exact invocation](reflection.md) — lookup versus execution
