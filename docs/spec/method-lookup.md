# Method Lookup

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

## 1. Resolution order

1. **Inline cache** at the call site (monomorphic → polymorphic → megamorphic).
2. **Exact selector probe** on the receiver's class, walking the superclass chain
   ([Object Model §5](object-model.md)). For a class-side send the walk starts at
   the metaclass.
3. **Variadic table probe** ([Messages & Selectors §4](messages-and-selectors.md)).
4. **`doesNotUnderstand(_:)`**.

`super.sel` starts the walk at the superclass of the method's *defining* class,
with the original receiver.

## 2. `doesNotUnderstand`

```phalcom
class Proxy {
  doesNotUnderstand(msg) {
    // msg.selector -> Symbol
    // msg.name     -> String
    // msg.labels   -> List of String
    // msg.args     -> List
    _target.perform(msg.selector, msg.args)
  }
}
```

The failed send is **reified** as a `Message` object. This is what makes proxies,
DSLs, delegation, and `Object.respondsTo(_:)` fall out for free.

`Object.doesNotUnderstand(_:)` is defined to raise `MessageNotUnderstood`
([Object Model §4](object-model.md)).

**Implementation.** A pure slow path: the inline cache misses, lookup walks the
chain, fails, *then* re-sends `doesNotUnderstand(_:)`. Cache the resolved handler
per receiver class so proxy-heavy code does not re-walk the chain on every call.

## 3. `perform` and reflection

`Object.perform(selector, args)` is a first-class reflective send. It shares the
`SEND_DYNAMIC` machinery with spread call sites and `doesNotUnderstand` forwarding
([Messages & Selectors §5](messages-and-selectors.md)).
</content>
