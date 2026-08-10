# Function

[Callables](README.md) · [Dispatch and lowering](dispatch.md) · [Arguments and rest](arguments.md) · [Runtime and activation](runtime.md) · [Execution contexts](execution.md) · [Closure](closure.md) · [BoundMethod](bound-method.md) · [Family](family.md) · [Method](method.md) · [Reflection](reflection.md)

`Function` is the abstract, sealed, VM-backed root of complete callables. Its
concrete descendants are `Closure`, `BoundMethod`, and `Family`. A Function
already contains every execution-context input except explicitly supplied call
arguments.

`Method` is not a Function: it represents exact behavior but still lacks a
receiver. [Method](method.md) describes how binding or `invokeOn` supplies it.

## 1. Complete callable versus callable protocol

Application syntax is ordinary message syntax:

```phalcom
f(a, b)
```

is equivalent to:

```phalcom
f.call(a, b)
```

Any object can define `call` and participate in this protocol. Only the sealed
core representations are Functions.

```phalcom
class Counter {
    call(amount) { value = value + amount }
}

const counter = Counter.new()
counter(2) // ordinary call message; Counter need not inherit Function
```

This distinction keeps user-defined callability open without making the VM's
representation-routing hierarchy open. A user object still receives a normal
message send; it does not enter `activate_function` merely because it answers
`call`.

## 2. One final call family

The canonical Function gateway is:

```phalcom
call(***arguments)
```

The `call` base family is final for Function descendants. Function subclasses
do not define a finite collection such as `call()`, `call(_)`, `call(_,_)`, and
so on. Instead, every source application produces its ordinary shape-specific
`call` selector, and Function's complete-rest Method accepts that shape after
exact selector lookup misses.

```text
f()                  → call()
f(value)             → call(_)
f(to: point)         → call(to)
f(value, to: point)  → call(_,to)
```

All four are ordinary messages. The Function root's `call(***)` Method is the
single fallback gateway, then concrete Function activation decides whether the
particular representation accepts the transported shape. See
[Dispatch and lowering](dispatch.md#2-value-application-lowering).

## 3. Rest and spread notation

Phalcom uses only:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

```phalcom
f(*values)
f(**labels)
f(***arguments)
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator.

The `***` in `call(***arguments)` means complete transport. It does not mean
that Function accepts all complete shapes without further validation, and it
does not require a public Tuple or List allocation before each call.

## 4. Concrete Function acceptance

| Function representation | Context already stored | Acceptance after gateway |
| --- | --- | --- |
| [Closure](closure.md) | compiled code and lexical captures | fixed positional parameters plus optional positional rest; labels rejected |
| [BoundMethod](bound-method.md) | exact Method and validated receiver | underlying Method parameter shape |
| [Family](family.md) | receiver/reference and open-or-pinned selector state | route, then selected target parameter shape |

For example:

```phalcom
const collect = |head, *tail| { tail }
```

may be called with positional arguments or positional spread. It rejects a
non-empty labeled lane even though the Function gateway successfully transports
that lane. Transport and acceptance are deliberately separate.

## 5. `callWith`

```phalcom
f.callWith(arguments)
```

is exactly:

```phalcom
f(***arguments)
```

`callWith` is a convenience spelling for complete forwarding. It must preserve
label order and positionals, and it must not define a second binder, a
List-only ABI, or alternate dispatch semantics. See
[Arguments and rest](arguments.md#8-callwith).

## 6. Abstract and sealed

`Function` is abstract: source code cannot instantiate it directly. `Function`,
`Closure`, `BoundMethod`, and `Family` are sealed VM-owned classes. The sealing
invariant permits the runtime to route Function activation by concrete
representation without exposing an extension protocol whose implementations
could violate stack, argument-shape, or authority invariants.

This does not prevent open method dictionaries elsewhere in the language. It
only prevents user subclasses from changing the set of VM Function
representations behind the final `call` family.

## 7. Reflection boundary

The canonical base Function protocol does not define scalar universal `arity`
or generic `name`. Labels, rest modes, and late-bound Families make one scalar
arity incomplete. A later reflection design may expose structured parameter
shape without changing application semantics.

Compatibility primitives that exist during migration are implementation detail,
not a reason to make these fields part of the language contract. See
[Reflection](reflection.md#8-function-reflection-boundary).

## 8. Implementation note

The Function root installs one shape-aware native `call` rest Method. Its
implementation delegates to the VM's concrete Function router:

```rust
pub fn block_call_shape(
    vm: &mut VM,
    receiver: Value,
    args: ArgumentView,
) -> PhResult<CallOutcome> {
    vm.activate_function(receiver, args, SourceRange::default())
}
```

The router reuses the existing stack window and selects Closure, BoundMethod,
or Family. `CallOutcome::EnteredFrame` lets it continue the same interpreter
loop when a bytecode body begins. See
[`primitive/block.rs`](../../../phalcom-core/src/primitive/block.rs) and
[`vm/send.rs`](../../../phalcom-core/src/vm/send.rs).

## 9. Related chapters

- [Dispatch and lowering](dispatch.md) — `f()` lowering and selector identity
- [Arguments and rest](arguments.md) — complete-shape transport
- [Runtime and activation](runtime.md) — flat gateway routing
- [Closure](closure.md), [BoundMethod](bound-method.md), [Family](family.md) — concrete descendants
- [Method](method.md) — incomplete exact behavior
