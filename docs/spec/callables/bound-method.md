# BoundMethod

[`BoundMethod`](README.md) is a sealed/final [`Function`](function.md) defined
by one equation:

```text
BoundMethod = exact Method + validated receiver
```

It is the receiver-complete form of a [`Method`](method.md). It contains no
synthetic cloned Method and requires no nested or rebound BoundMethod wrapper.

## Binding and activation

Binding validates receiver compatibility before producing the BoundMethod.
Holder/subclass compatibility is required; class-side Methods use analogous
metaclass ancestry. Receiver lookup and validation happen when the Method is
obtained or bound, not again when the BoundMethod is called.

Calling a BoundMethod uses the common Function activation gateway and executes
the underlying exact Method with the stored receiver as `self`. The exact
Method's lexical access authority and `super` anchor remain unchanged.

Ordinary sends inside that Method still dynamically dispatch on the stored
receiver. Exact activation does not redispatch the underlying Method's
selector.

There is no direct BoundMethod rebinding API initially. To obtain another
receiver pairing, obtain or bind an appropriate Method and perform its normal
compatibility checks.

## Arguments

Phalcom uses one rest/spread notation everywhere:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

Examples:

```phalcom
bound(*values)
bound(**labels)
bound(***arguments)
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator.

The BoundMethod forwards the resulting argument shape to its exact Method;
acceptance follows that Method's parameter shape. It does not create a second
binder or a new Method object merely to call.

## Related callable types

- [`Callable model`](README.md) — hierarchy and execution contexts
- [`Method`](method.md) — exact behavior supplied to the binding
- [`Function`](function.md) — common activation gateway
