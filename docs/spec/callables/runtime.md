# Runtime and activation

[Callables](README.md) · [Dispatch and lowering](dispatch.md) · [Arguments and rest](arguments.md) · [Execution contexts](execution.md) · [Function](function.md) · [Method](method.md) · [Closure](closure.md) · [BoundMethod](bound-method.md) · [Family](family.md) · [Reflection](reflection.md)

This chapter exposes the VM architecture behind the callable specification. It
is intentionally precise enough for compiler and runtime work while preserving
one boundary: internal Rust types are not public language classes.

## 1. Method implementation record

A reified Method is a heap object containing more than a function pointer. Its
semantic record includes:

```text
implementation: bytecode closure or native primitive
signature: selector, fixed shape, optional rest layout
holder: class or metaclass that owns the behavior
visibility and lexical access owner
lexical super anchor
```

The current implementation represents the executable portion as either a
compiled `ClosureObject` or a native `PrimitiveFn`:

```rust
pub enum MethodKind {
    Closure(ObjRef),
    Primitive(PrimitiveFn),
}
```

See [`method/object.rs`](../../../phalcom-core/src/method/object.rs). Sharing a
compiled closure representation between Method bodies and Closure literals is
an implementation economy; it does not make a Method a Function or a public
Closure.

## 2. Stack window and argument view

At a send boundary, the VM stores a conceptual window in this order:

```text
receiver
positional values, in source order
labeled values, in label order
```

Label identity remains in the selector/shape metadata rather than being
interleaved as ordinary stack values. Native primitives receive an
`ArgumentView`, a compact descriptor containing the receiver position,
positional count, labeled count, optional selector, and caller authority.

```rust
pub struct ArgumentView {
    receiver_index: usize,
    positional_count: usize,
    labeled_count: usize,
    selector: Option<Symbol>,
    caller_access: Option<ClassId>,
    caller_internal: bool,
}
```

The view owns no values and does not borrow the VM stack. Its accessors borrow
the VM only for a single read. This avoids holding an immutable Rust borrow of
`VM::stack` while a native primitive has `&mut VM`, and preserves GC rooting of
the original stack window. See
[`method/object.rs`](../../../phalcom-core/src/method/object.rs).

## 3. Flat activation result

A native Method may finish immediately or enter bytecode. Those two cases are
explicit:

```rust
pub enum CallOutcome {
    Returned(Value),
    EnteredFrame,
}
```

`Returned(value)` lets the dispatcher replace the receiver-and-arguments
window with one result. `EnteredFrame` means the native gateway has rewritten
the window as needed and pushed a bytecode frame; the existing interpreter loop
continues in that frame.

This is the key to flat language-level forwarding. `Function#call`,
`Method#invokeOn`, `Object#perform`, BoundMethod activation, and Family routing
must not call a nested `run_until` merely to recover a value. They return a
`CallOutcome` to the same dispatch loop. Synchronous host helpers may still
perform recursive interpreter entry where host code genuinely needs an
immediate value; that is not the language-level forwarding path.

## 4. Shared Function gateway

The VM installs one shape-aware primitive on the abstract `Function` class:

```phalcom
call(***arguments)
```

Its native body is intentionally small:

```rust
pub fn block_call_shape(
    vm: &mut VM,
    receiver: Value,
    args: ArgumentView,
) -> PhResult<CallOutcome> {
    vm.activate_function(receiver, args, SourceRange::default())
}
```

The historical Rust module name does not change the public model: this is the
Function gateway. It selects one of the sealed concrete representations:

```text
Closure      → validate Closure shape, bind, push Closure frame
BoundMethod  → validate stored exact Method, replace receiver, activate Method
Family       → derive or validate target selector, replace receiver, dispatch
```

The current `activate_function` switch lives in
[`vm/send.rs`](../../../phalcom-core/src/vm/send.rs). It reuses the existing
stack window and does not create a reified Message or argument vector for an
ordinary Function call.

## 5. Function activation flow

For a value application such as `f(a)`, the full runtime path is:

```text
compiler emits selector call(_)
    ↓
ordinary exact lookup misses on Function descendants
    ↓
Function's complete-rest call(***) is selected
    ↓
block_call_shape receives ArgumentView
    ↓
activate_function dispatches on Closure / BoundMethod / Family
    ↓
native result or pushed bytecode frame
```

This is why the Function gateway is a shared Method rather than a compiler-only
operation. An explicit `f.call(a)` and application `f(a)` converge at ordinary
selector dispatch before the concrete Function representation is examined.

## 6. Closure frame entry

Closure activation checks that labels are absent and that the positional count
matches the Closure's fixed/rest shape. It then truncates surplus call-window
arguments or replaces the residual positional suffix with its canonical
Unit/Tuple capture before pushing a call frame.

The current implementation is deliberately direct:

```rust
if shape.rest.is_some() {
    let capture = finish_tuple(vm, residual, Vec::new())?;
    vm.stack.truncate(receiver_idx + 1 + shape.fixed_positionals);
    vm.stack.push(capture);
}
let frame = vm.new_call_frame(closure_id, context, 0, receiver_idx, span);
vm.push_frame(frame)?;
```

The observable rule is in [Closure](closure.md): zero residual values become
`()`, and one or more become a Tuple. This frame rewrite is only the VM
realization of that binding rule.

## 7. Exact Method activation

When dispatch has selected an ordinary Method, the runtime first authorizes the
caller. A bytecode Method pushes a new frame using the receiver's execution
context. A shape-aware primitive receives `ArgumentView` and may return or
enter a frame itself. Native execution receives a separate lexical method
context so sends performed by native code use the callee's authority rather
than accidentally inheriting the caller's authority.

The authority split is deliberate:

```text
caller authority  → may this caller enter the selected Method?
callee authority  → what private/protected/internal sends may its body make?
```

This same split is retained when a Method is invoked through a BoundMethod,
`invokeOn`, `perform`, or a Function gateway. See
[Method](method.md#7-access-and-reflection) and
[Reflection](reflection.md#6-access-control).

## 8. Rest activation

An exact selector is always tried before rest. Once a rest Method is selected,
the VM either:

- leaves the shaped window available to a native rest primitive; or
- rewrites a bytecode Method's arguments into its declaration-local fixed and
  captured values.

For bytecode, the rest capture occurs at frame entry, not while matching the
selector. This preserves the distinction between lookup and binding and avoids
allocating a rest product when a native implementation does not need one.

## 9. Family routing is direct

Family activation already has the caller's shape. An open Family derives a
selector from its stored base name plus that shape. A pinned Family retains its
stored selector after validating the supplied total arity. The VM replaces the
window's Family receiver with the stored target receiver and calls shaped
ordinary dispatch.

No deliberate `doesNotUnderstand` round trip is needed to recover the original
labels. [Family](family.md) describes the public distinction and
[Dispatch and lowering](dispatch.md#8-misses-and-forwarding) describes the
miss boundary.

## 10. Transitional implementation note

The current source still contains a private legacy closure carrier and
home-frame-token plumbing. Surface classification maps that carrier to
`Closure`; it does not introduce a public callable class. The canonical
language model uses Closure-local return and has no implicit non-local return.
Runtime work must remove or isolate transitional machinery rather than expose
its representation as language semantics.

Likewise, a legacy primitive adapter remains for mechanical migration of
fixed-arity Rust primitives. The shape-aware `PrimitiveFn::Shape` ABI is the
semantic ABI for argument-sensitive gateways and native rest. Compatibility
adapters must not cause a visible difference in selector, labels, authorization,
return, or allocation-sensitive semantics.

## 11. Related chapters

- [Dispatch and lowering](dispatch.md) — selection before activation
- [Arguments and rest](arguments.md) — shape and parameter binding
- [Execution contexts](execution.md) — frame-visible semantics
- [Function](function.md) — public gateway contract
- [Method](method.md), [Closure](closure.md), [BoundMethod](bound-method.md), and [Family](family.md) — concrete routes
- [Reflection](reflection.md) — shaped reflective forwarding
- [Callable conformance requirements](conformance.md) — implementation invariants and test lanes
