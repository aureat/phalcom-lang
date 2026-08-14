# BoundMethod

[Callables](README.md) · [Method](method.md) · [Function](function.md) · [Arguments and rest](arguments.md) · [Runtime and activation](runtime.md) · [Execution contexts](execution.md) · [Reflection](reflection.md)

`BoundMethod` is a sealed/final `Function` with one exact semantic identity:

```text
BoundMethod = exact Method + captured receiver
```

It completes a Method's missing receiver. It is neither a cloned Method, a synthetic Closure, a nested binding wrapper, nor a late-dispatch Family.

## 1. Construction and identity

```phalcom
const method = Person.methodFor(#greet())
const bound = method.bind(person)
```

`bind` captures `person` without requiring holder compatibility. Bytecode
field access is guarded when the bound Method runs: the receiver must have the
Method holder's layout or a subclass layout, otherwise the runtime raises
`IncompatibleMethodLayout` before slot access. Primitive Methods have no layout
requirement. Holderless public Methods cannot be bound until a later
specification defines a safe category.

Binding captures the exact Method chosen at that moment. It does not look up its selector again at each call. The VM may defensively verify the stored pair at activation, but that is an invariant check, not a second receiver lookup or method-family dispatch.

## 2. Calling

```phalcom
bound(arguments)
```

is ordinary application syntax:

```phalcom
bound.call(arguments)
```

The common `Function#call(***arguments)` gateway transports the complete shape, then routes to the stored Method and stored receiver.

```text
Function gateway
    ↓
stored exact Method + stored receiver
    ↓
access and parameter-shape validation
    ↓
exact Method activation
```

The entry selector is never redispatched. Sends inside the Method remain dynamically dispatched on the stored receiver. `super` and lexical access authority remain those of the stored Method.

## 3. Equivalence with exact invocation

```phalcom
method.invokeOn(receiver, ***arguments)
method.bind(receiver)(***arguments)
```

These are equivalent exact activations. `invokeOn` validates and enters immediately. `bind` validates once to make a reusable Function, which later enters through the Function gateway. Both use the exact Method parameter shape and never perform selector redispatch on `receiver`.

## 4. Arguments and rest

Phalcom uses only:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

```phalcom
bound(*values)
bound(**labels)
bound(***arguments)
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a spread operator.

The underlying exact Method accepts or rejects the transported shape, including fixed labels and rest layout. BoundMethod creates no second binder and never coerces labels into a positional collection. See [Arguments and rest](arguments.md#5-method-rest-parameters).

## 5. No rebinding API

There is initially no direct rebinding operation. To pair a Method with another receiver, use the Method's ordinary `bind` operation and its compatibility checks. This preserves one simple representation and avoids wrapper chains with competing receiver and authority rules.

## 6. Implementation note

The VM payload is minimal:

```rust
pub struct BoundMethodObject {
    pub method: ObjRef,
    pub receiver: Value,
}
```

Function activation replaces the call-window receiver with `receiver`, validates the stored Method, and activates it through the flat `CallOutcome` path. Primitive and bytecode Methods share this route; no Closure allocation or recursive interpreter entry is required. See [`heap/object.rs`](../../../phalcom-core/src/heap/object.rs) and [`vm/send.rs`](../../../phalcom-core/src/vm/send.rs).

## 7. Related chapters

- [Method](method.md) — exact behavior and receiver compatibility
- [Function](function.md) — shared call gateway
- [Reflection](reflection.md) — `bind` and `invokeOn`
- [Execution contexts](execution.md) — dynamic `self` and lexical `super`
- [Runtime and activation](runtime.md) — stored-pair routing
