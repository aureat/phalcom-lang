# Method

[Callables](README.md) · [Dispatch and lowering](dispatch.md) · [Arguments and rest](arguments.md) · [Execution contexts](execution.md) · [Runtime and activation](runtime.md) · [Reflection](reflection.md) · [Function](function.md) · [Closure](closure.md) · [BoundMethod](bound-method.md)

A `Method` is reified, holder-owned exact behavior. It is an object that can be
inspected, authorized, bound, and invoked exactly. It is not a `Function`,
because it still requires a compatible receiver before it can execute.

## 1. Semantic record

Every Method has one coherent semantic identity:

| Field | Meaning |
| --- | --- |
| holder | class or metaclass owning the behavior |
| selector | complete message identity, including argument labels |
| parameter shape | fixed lanes and optional rest layout accepted by this Method |
| implementation | compiled bytecode body or native primitive |
| lexical `super` anchor | holder-relative origin for `super` lookup |
| access authority | lexical owner governing private/protected/internal sends |

The Method's **selector** identifies the exact behavior registered in a method
dictionary. The **parameter shape** controls acceptance after that behavior has
been selected. For rest-capable Methods, the selector's base family and rest
metadata collaborate during fallback lookup; they do not turn the Method into a
wildcard string.

## 2. Method is incomplete by design

A Method can describe `Person#greet`, but it cannot run until there is a
receiver. The missing receiver distinguishes it from a Function:

```text
Method       exact implementation + holder + authority + super anchor
Function     executable context already complete except explicit arguments
```

Calling an unbound Method as a Function is rejected. The supported ways to
supply a receiver are one-shot exact invocation and reusable binding:

```phalcom
method.invokeOn(receiver, ***arguments)
method.bind(receiver)
```

Neither operation selects a different overload after the Method has been
chosen. That is the defining difference between Method exactness and
[Family](family.md) late dispatch.

## 3. Selector and rest notation

Phalcom uses only:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

```phalcom
sum(*rest) { body }
configure(**rest) { body }
forward(***rest) { body }
method.invokeOn(receiver, ***arguments)
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator.

Exact Method selectors record fixed positional and labeled slots. Rest Method
metadata accepts an extension of those slots. Normal dispatch searches exact
selectors through the complete hierarchy before it begins rest-family fallback;
an inherited exact Method therefore wins over a rest-capable Method on a more
derived class. See [Arguments and rest](arguments.md#5-method-rest-parameters).

## 4. Exact invocation

```phalcom
method.invokeOn(receiver, ***arguments)
```

executes `method` itself. It does not send the Method's selector to `receiver`
and does not re-run overload lookup. Exact invocation performs these checks in
order:

1. `method` must be a reified Method value.
2. The initiating caller must be authorized to enter it.
3. The supplied receiver may be any runtime value. If the exact bytecode body
   accesses instance or static fields, activation installs a representation
   guard requiring the receiver's layout owner to be the Method holder or a
   subclass; incompatible access raises `IncompatibleMethodLayout` before slot
   access. Primitive Methods have no field layout requirement.
4. The residual complete argument shape must satisfy the exact Method.
5. The VM activates the exact bytecode or native implementation.

The receiver is not pre-filtered by holder ancestry. For class-side Methods,
the same layout guard uses metaclass ancestry when the body accesses static
fields. A holderless Method is not publicly bindable or invokable until a later
specification introduces a safe holderless category.

```phalcom
const describe = Base.methodFor(#describe())
describe.invokeOn(Derived.new(), ***())
```

This runs `Base#describe` exactly when `Derived` is compatible with `Base`. It
does not select a possible `Derived#describe` override as the entry body.

## 5. Exact body, dynamic sends

Exact invocation fixes the entry behavior, not the entire dispatch universe.
For an exact Method defined on `Base` and activated on a compatible `Derived`
instance:

```text
entry implementation     Base's exact Method body
self                     supplied Derived instance
ordinary sends           dynamically dispatched on Derived
super sends              start above lexical Base
access authority         lexical authority of Base Method
```

This rule makes reflection useful without turning methods into statically bound
objects in their own body. It also makes `super` stable under binding: binding
changes `self`, not the defining holder. See
[Execution contexts](execution.md#2-self) and
[Execution contexts](execution.md#3-super).

## 6. Binding

```phalcom
const bound = method.bind(receiver)
```

validates receiver compatibility before constructing a
[BoundMethod](bound-method.md). The resulting Function stores only the exact
Method and validated receiver. It contains no cloned Method, synthetic Closure,
or nested rebinding wrapper.

```phalcom
method.invokeOn(receiver, ***arguments)
bound(***arguments)
```

are equivalent exact activations after the receiver has been supplied. Binding
is the reusable form; `invokeOn` is the one-shot form. The current runtime also
defensively rechecks a stored pair at activation, but that check is an internal
invariant guard, not a second receiver lookup or a source-visible redispatch.

## 7. Access and reflection

Method lookup, access authorization, and execution are separate operations.
An inaccessible Method remains a Method; invoking it is an access error rather
than a `doesNotUnderstand` miss. Native and bytecode Methods carry the same
lexical authority model, so reflective calls cannot become accidental visibility
bypasses.

Method reflection is reached through the Behavior/Object boundary:

```phalcom
const m = Person.methodFor(#greet())
const selector = m.selector
const holder = m.holder
```

The precise public reflection surface is in [Reflection](reflection.md). A
Family is not a Method: it stores a future dispatch reference and can select a
target later.

## 8. Constructors

The initializer generated from an `@constructor` source declaration is an
ordinary Method at runtime. It has ordinary final-expression and bare-return
semantics, but its generated class-side factory ignores that initializer result
and returns the allocated instance. The compiler rejects `return value` in a
source constructor initializer. See
[Execution contexts](execution.md#7-constructors).

## 9. Implementation note

In the VM, `MethodObject` contains either a compiled closure handle or a native
primitive plus `Signature`, holder, visibility, and access-owner fields. A
shape-aware primitive receives `ArgumentView` and returns `CallOutcome`, which
lets `invokeOn` activate bytecode without recursively running the interpreter.

```rust
pub enum MethodKind {
    Closure(ObjRef),
    Primitive(PrimitiveFn),
}
```

See [`method/object.rs`](../../../phalcom-core/src/method/object.rs) and
[`primitive/method.rs`](../../../phalcom-core/src/primitive/method.rs). This
representation sharing is private VM machinery; Method remains a distinct
public class and never becomes a Function subclass.

## 10. Related chapters

- [Reflection](reflection.md) — `methodFor`, `invokeOn`, `bind`, and `perform`
- [BoundMethod](bound-method.md) — complete exact Method pairing
- [Function](function.md) — complete callable root
- [Closure](closure.md) — lexical executable value
- [Arguments and rest](arguments.md) — rest matching and capture
- [Runtime and activation](runtime.md) — Method frame and native entry
