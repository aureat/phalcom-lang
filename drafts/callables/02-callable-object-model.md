# Callable Object Model

[← Core execution](01-core-execution-and-values.md) · [Overview](README.md) · [Calls and rest →](03-calls-rest-and-spread.md)

---

## 1. Hierarchy

The canonical callable hierarchy is:

```text
Object
├── Method                         sealed/final
└── Function                       abstract, sealed
    ├── Closure                    sealed/final
    ├── BoundMethod                sealed/final
    └── Family                     sealed/final
```

These VM-backed classes are sealed.

A user-defined object may nevertheless be callable by defining a `call` Method. Such an object is callable by protocol but is not thereby a `Function`.

---

## 2. Function

A `Function` is a complete VM-backed callable: all execution context other than explicit call arguments is already contained in the value.

The canonical Function call gateway is:

```phalcom
call(***arguments)
```

The base selector family `call` is final throughout the Function hierarchy.

Concrete Function subclasses do not define competing finite overload families such as `call`, `call(_)`, `call(_,_)`, and so on.

`Function` is abstract and cannot be directly instantiated.

The base Function protocol does not define scalar callable reflection such as universal `arity` or universal `name`. Rich callable reflection is specified separately if introduced.

---

## 3. Closure

A `Closure` consists of:

```text
executable code
+
captured lexical environment
```

and, when created in a lexical context with `self`, captures the current `self` value.

Canonical literal forms include:

```phalcom
|| {
    work()
}
```

```phalcom
|value| {
    value * 2
}
```

```phalcom
|x, y| x + y
```

```phalcom
|head, *tail| {
    ...
}
```

Closures use the final-expression and local-return semantics specified in [Core execution](01-core-execution-and-values.md).

### 3.1 Captured `self`

If a Closure is created while a Method is running with receiver `receiver`, the Closure's lexical `self` is that receiver.

```phalcom
class Box {
    callback {
        || {
            self
        }
    }
}
```

Calling the resulting Closure later yields the captured receiver even after the creating Method has returned.

### 3.2 Closure versus block

A Closure is a runtime object.

A block is a brace-delimited syntactic/lexical region.

Not every block creates a Closure.

---

## 4. Method

A Method is reified holder-owned behavior.

A Method conceptually contains:

```text
holder
selector
parameter shape
implementation
lexical access authority
lexical super anchor
```

A Method is not a Function because it is incomplete without a receiver.

### 4.1 Exact invocation

A reified Method may be invoked exactly through:

```phalcom
method.invokeOn(receiver, ***arguments)
```

Exact invocation:

- validates receiver compatibility;
- validates argument shape;
- executes that exact Method;
- does not redispatch the Method's selector.

### 4.2 Receiver compatibility

For an instance-side Method owned by behavior `A`, a receiver is compatible when its runtime class is `A` or a subclass of `A`.

Class-side Method compatibility follows the corresponding metaclass ancestry.

Immediate values are validated through their actual runtime classes.

A holderless Method is not publicly bindable/invokable unless another specification defines a safe holderless Method category.

### 4.3 Dynamic `self`, lexical Method identity

When an `A` Method executes exactly on compatible subclass instance `b`:

```text
Method body        = exact A Method body
self               = b
ordinary sends     = dynamically dispatched on b
super anchor       = lexical A
access authority   = lexical A Method context
```

Thus exact Method invocation does not redispatch the entry Method, but sends performed from inside that Method remain ordinary dynamic sends.

### 4.4 `super`

`super` keeps the current dynamic receiver but begins lookup above the Method's lexically defining holder.

Binding a Method to a subclass receiver does not move the Method's `super` anchor.

---

## 5. BoundMethod

A BoundMethod is:

```text
exact Method + compatible receiver
```

Construction:

```phalcom
const method = Type.methodFor(selector)
const bound = method.bind(receiver)
```

`bind` validates receiver compatibility before creating the BoundMethod.

A BoundMethod does not contain:

- a cloned Method;
- a synthetic Closure wrapper;
- a set of generated per-arity `call` Methods.

Calling a BoundMethod activates its exact stored Method against its stored receiver.

### 5.1 No rebinding API

BoundMethod has no direct rebinding operation initially.

If a caller needs a differently bound callable, binding is performed from the relevant Method.

---

## 6. Family

A Family is the callable value produced by `::` method-reference syntax.

Typical conceptual forms include:

```phalcom
object::move
object::#move(_,to,duration)
```

An open Family identifies a base method family and derives the concrete target selector from the call's actual shape at invocation time.

A pinned Family preserves its pinned selector according to the method-reference rules.

A Family is a Function because it already contains the receiver/reference context required to proceed once explicit call arguments are supplied.

### 6.1 Family calls are ordinary sends

Calling a Family eventually performs an ordinary message send to the Family's stored receiver.

An open Family applies the actual call shape to its stored base name and dispatches the resulting selector.

A Family must not depend semantically on intentionally missing `call(...)` and using `doesNotUnderstand` as its normal call router.

`doesNotUnderstand` remains reserved for genuine message misses.

---

## 7. Application syntax

For any value `f`:

```phalcom
f(a, b)
```

is semantically:

```phalcom
f.call(a, b)
```

This applies both to Functions and to ordinary user objects defining `call`.

An implementation may optimize known Function receivers, but it must preserve the ordinary message semantics, including normal access, error, and dispatch behavior.

---

## 8. `callWith`

For Function values:

```phalcom
f.callWith(arguments)
```

means exactly:

```phalcom
f(***arguments)
```

`callWith` is convenience syntax/protocol over the same complete argument transport.

It does not introduce a second argument representation or binder.

---

## 9. Constructors and Methods

`@constructor` does not create a new callable category.

The compiler uses a constructor declaration to generate a class-side factory and an ordinary instance initializer Method.

The initializer therefore remains subject to ordinary Method execution semantics. The factory supplies the constructor-specific allocation/result behavior described in [Core execution](01-core-execution-and-values.md).

---

## 10. Sealing

The following classes are sealed VM-backed core classes:

```text
Function
Closure
BoundMethod
Family
Method
```

`Function` is additionally abstract.

User-defined callable abstractions should use ordinary objects with `call` Methods rather than subclassing the VM-backed Function hierarchy.
