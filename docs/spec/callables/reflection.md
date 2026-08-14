# Reflection and exact invocation

[Callables](README.md) · [Dispatch and lowering](dispatch.md) · [Arguments and rest](arguments.md) · [Runtime and activation](runtime.md) · [Method](method.md) · [BoundMethod](bound-method.md) · [Family](family.md) · [Function](function.md)

Reflection exposes Method structure without erasing the distinction between a
resolved Method, a captured MethodFamily snapshot, and a message sent to a
concrete object. This chapter defines that boundary and the callable
operations that cross it.

## 1. Structural reflection versus object execution

Structural reflection asks about behavior governed by a class or metaclass.
Dynamic execution sends a message to a particular receiver.

```text
Behavior reflection             Object execution
-------------------             ----------------
methods                          perform
methodFor                        doesNotUnderstand
respondsTo                       ordinary message sends
```

The exact public Behavior surface may grow in its own specification, but its
callable meaning is stable: `methodFor` returns resolved behavior; `methods`
enumerates direct definitions; `respondsTo` applies normal lookup without
turning a dNU proxy into universal positive capability.

## 2. `methodFor` and `respondsTo`

Conceptually:

```phalcom
const method = Person.methodFor(#greet())
const answers = Person.respondsTo(#greet())
```

`methodFor` performs normal inherited resolution for instances governed by the
Behavior. It returns the Method that ordinary dispatch would select for the
given selector, or absence when no accessible Method resolves. `methods`, by
contrast, describes the direct method dictionary and does not imply inherited
resolution.

Where a supplied selector shape is eligible for rest, reflection follows the
same language resolver:

```text
exact selector through inheritance
then compatible rest family
then absence
```

Reflection does not invoke `doesNotUnderstand`. A true send miss may reach dNU
after normal lookup; a probe must not execute the miss handler simply to answer
a structural question.

Behavior also implements ordinary `>>(_)` reflection. A selector Symbol returns
one effective Method or absence. A SelectorPattern returns an immutable
MethodFamily snapshot. Pattern capture walks live effective methods once,
applies the initiating caller's visibility authority, and omits inaccessible
routes; later method replacement does not mutate the snapshot.

MethodFamily exposes `selectors`, `size`, `methodFor(_)`, and `bind(_)`.
`selectors` returns canonical captured selectors in capture order; `size` counts
exact and rest routes; `methodFor(_)` returns only a captured accessible route;
`bind(_)` stores the snapshot with a receiver and never reselects from that
receiver. BoundMethodFamily invocation matches the incoming shape against the
captured exact map and rest chain, then activates the selected Method exactly.

## 3. Exact `Method#invokeOn`

```phalcom
method.invokeOn(receiver, ***arguments)
```

is exact invocation, not a send of the Method's selector to `receiver`. It:

1. confirms that `method` is a reified Method;
2. authorizes the original caller against that Method;
3. accepts any receiver value; bytecode field access installs a representation
   guard requiring the Method holder layout or a subclass layout;
4. removes the explicit receiver from the complete argument shape;
5. validates the exact Method's parameter shape;
6. activates that exact Method without selector redispatch.

The Method body still has dynamic `self == receiver`, dynamic ordinary sends,
and lexical `super` and access authority. See [Method](method.md) and
[Execution contexts](execution.md).

The shape-aware native implementation follows this outline:

```rust
let method_id = expect_method(vm, &receiver)?;
let target = args.positional(vm, 0)?;
vm.authorize_method_access_as(method_id, caller, internal)?;
validate_captured_method_shape(method_id, residual_shape)?;
// remove target from the argument window, then activate method_id exactly
```

See [`primitive/method.rs`](../../../phalcom-core/src/primitive/method.rs).
The operation does not use a packed List intermediary and returns a flat
`CallOutcome` when activation enters bytecode.

## 4. `Method#bind`

```phalcom
const bound = method.bind(receiver)
```

captures the receiver without a nominal compatibility check and creates a
[BoundMethod](bound-method.md): one exact Method handle plus one receiver
value. It does not clone the Method, synthesize a Closure, or manufacture
per-arity call Methods.

If the Method body accesses fields, the same representation guard is applied
at activation. A foreign layout raises `IncompatibleMethodLayout` before the
field slot is read or written; primitive Methods have no field-layout guard.

The resulting `bound(***arguments)` and
`method.invokeOn(receiver, ***arguments)` are semantic siblings: both activate
the stored exact Method with the same receiver and complete shape. Binding is
the reusable form; `invokeOn` is the one-shot exact form.

## 5. `Object#perform`

`perform` is receiver-specific dynamic execution:

```phalcom
receiver.perform(selector, ***arguments)
```

The first positional argument is a selector Symbol. The remaining positional
and labeled lanes are retained exactly, then ordinary shaped dispatch begins on
`receiver`. Unlike `invokeOn`, `perform` does perform selector lookup; unlike a
Family, it does not carry pre-bound selector-reference state.

The implementation removes the selector value from the stack window before
calling the shaped dispatcher:

```rust
let positional_count = args.positional_count() - 1;
let labels = args.labels(vm);
// retain residual values, then dispatch selector on original receiver
vm.dispatch_shape_at_as(receiver_index, selector, positional_count, &labels, ...)
```

See [`primitive/object.rs`](../../../phalcom-core/src/primitive/object.rs).

## 6. Access control

Lookup and authorization are separate. A method can exist and be selected but
remain inaccessible to the current caller. In that case the result is an
access error, not `doesNotUnderstand`.

The runtime preserves two contexts:

```text
caller authority  controls entry to the selected Method
callee authority  controls sends made while that Method executes
```

`private`, `protected`, and internal checks are applied consistently to normal
sends, exact invocation, binding-derived calls, `perform`, and native gateway
forwarding. A forwarding primitive must preserve the original caller authority
for the target access check, then install the target Method's lexical authority
for its execution.

## 7. Method, BoundMethod, and Family are different reflective values

```text
Method       exact behavior; receiver missing
BoundMethod  exact behavior; receiver stored
Family       receiver plus exact/pattern selector state; target route selected at call time
MethodFamily immutable captured exact/rest route snapshot
BoundMethodFamily captured MethodFamily plus receiver; no receiver-side lookup
```

```phalcom
const exact = Person.methodFor(#greet())
exact.invokeOn(person, ***())

const late = person::greet
late()
```

The first activation executes the Method returned by lookup. The second derives
or uses an exact getter Family selector and performs target lookup at call time.
Neither value should be described as the other.

## 8. Function reflection boundary

The canonical base `Function` protocol does not define scalar universal
`arity` or generic `name`. Rest shapes, labels, and Family routing make a
single scalar arity an incomplete description of callable acceptance. Richer
parameter-shape reflection can be added deliberately without changing the call
gateway.

The current tree still contains compatibility primitives in this area. They are
not a reason to extend the normative base Function protocol or to infer a
second call binder.

## 9. Related chapters

- [Method](method.md) — reified exact behavior
- [BoundMethod](bound-method.md) — reusable exact pairing
- [Family](family.md) — late ordinary dispatch
- [Arguments and rest](arguments.md) — complete-shape transport
- [Dispatch and lowering](dispatch.md) — lookup and dNU boundary
- [Runtime and activation](runtime.md) — flat forwarding implementation
