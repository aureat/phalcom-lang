# Callable conformance requirements

[Callables](README.md) · [Dispatch and lowering](dispatch.md) · [Arguments and rest](arguments.md) · [Runtime and activation](runtime.md) · [Execution contexts](execution.md) · [Reflection](reflection.md) · [Method](method.md) · [Function](function.md) · [Closure](closure.md) · [BoundMethod](bound-method.md) · [Family](family.md)

This chapter is a developer-facing conformance checklist. It states the behavior that compiler, VM, native primitive, reflection, and test changes must preserve. It does not prescribe one Rust data structure beyond the invariants needed for observable semantics.

## 1. Application and dispatch

- A resolved value application `f(arguments)` must be observationally equivalent to `f.call(arguments)`.
- An unqualified call with no lexical/global value binding must remain an implicit-`self` method send.
- Static and dynamic-pack calls must evaluate receiver and argument expressions in the same source order.
- Static and dynamic-pack calls must encode the same final selector shape and take the same exact-then-rest lookup path.
- A real terminal miss may reach `doesNotUnderstand(_)`; Function, Family, and reflection gateways must not manufacture intentional misses.
- A dispatch cache may only optimize the ordinary resolver. It must not preserve stale behavior after an observable method-table change.

## 2. Argument transport

Phalcom uses only:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a spread operator.

- Argument shape must retain ordered positional values, ordered labels, and their aligned labeled values.
- Shape matching must not invoke user code or inspect dynamic argument types.
- A `***` gateway must transport complete shape without requiring a public Tuple allocation on every call.
- Duplicate labels and invalid spread operands must fail consistently for static and dynamic paths.
- `callWith(arguments)` must equal `self(***arguments)` and preserve both lanes.
- `invokeOn(receiver, ***arguments)` and `perform(selector, ***arguments)` must preserve residual complete shape after removing their leading control operand.

## 3. Method selection and rest

- Exact selector lookup runs through the complete inheritance chain before rest-family lookup begins.
- An inherited exact Method therefore beats a compatible rest Method on a derived class.
- At most one rest-capable Method may occupy one base-name family on one class.
- Native and bytecode Methods must have the same rest acceptance and capture semantics.
- Exact invocation must run the reified Method itself, never redispatch its selector.
- Exact invocation and binding must reject incompatible receivers before entering user code.

## 4. Function routing

```text
Closure      → positional-only Closure validation and Closure frame
BoundMethod  → stored exact Method and stored receiver
Family       → direct selector routing, then ordinary target dispatch
```

- The `call` family is final throughout the sealed Function hierarchy.
- No concrete Function requires synthesized finite call overloads.
- A Closure rejects any non-empty labeled lane.
- Closure positional rest captures `()` for zero residual values and Tuple for one or more.
- BoundMethod must not clone its Method or synthesize a Closure wrapper.
- Open Family routing derives selector from base name plus actual shape.
- Pinned Family routing retains its selector identity and validates supplied slot count.

## 5. Frame, authority, and control-flow invariants

- Language-level forwarding returns a native completion or enters a frame; it must not recursively drive the interpreter to completion.
- The call window remains rooted for the full activation, including native primitives receiving an argument view.
- Caller authority governs whether a Method may be entered; callee authority governs sends made by its body.
- Exact Method activation keeps the supplied receiver as `self`, ordinary sends dynamic, and `super` lexical to the defining holder.
- A Closure captures the current `self` when one exists without changing `super` semantics.
- Empty bodies and bare `return` produce Unit. `None` remains absence.
- Closure `return` is local. No implicit non-local-return state may become public language behavior.
- Constructor initialization is an ordinary Method activation whose result the generated factory discards; `return value` in source constructor code is rejected by the compiler.

## 6. Required test lanes

Changes to callable code should retain focused coverage in these lanes:

| Lane | Required observations |
| --- | --- |
| application lowering | lexical value call versus implicit-`self` send; explicit `call` equivalence |
| selector and packs | positional/labeled ordering, dynamic expansion, duplicate labels, boundedness |
| lookup | exact-before-rest across inheritance, native rest, final dNU miss |
| Closure | literal forms, captures, local return, positional rest Unit/Tuple capture, label rejection |
| Method | holder/subclass compatibility, class-side compatibility, exact invocation, `super`, visibility |
| BoundMethod | bind validation, exact stored Method execution, no rebinding surface |
| Family | open and pinned references, inherited target dispatch, labels, direct routing |
| forwarding | `callWith`, `perform`, `invokeOn`, flat frame entry, authority preservation |
| values | final expression, Unit versus None, bare return, constructor return restriction |

The existing language fixtures under [`phalcom-core/tests/lang`](../../../phalcom-core/tests/lang) and runtime invariants in [`phalcom-core/tests/invariants.rs`](../../../phalcom-core/tests/invariants.rs) are primary evidence for these lanes. A focused pass is not proof of a broad migration; search compiler, VM, native primitives, reflection, tooling, and fixtures for retired callable concepts before declaring a callable change complete.

## 7. Implementation migration boundary

The current VM contains compatibility paths while the callable redesign is completed. These may preserve behavior temporarily, but they must not become normative alternatives:

- legacy fixed-arity primitive adapters do not replace shape-aware native input;
- synchronous host helpers do not justify recursive language-level forwarding;
- internal closure carrier names do not define public callable classes;
- scalar compatibility reflection does not extend the canonical Function protocol;
- legacy non-local-return plumbing does not define Closure return semantics.

## 8. Related chapters

- [Dispatch and lowering](dispatch.md) — source-to-selector contract
- [Arguments and rest](arguments.md) — lane and binding contract
- [Runtime and activation](runtime.md) — VM contract
- [Execution contexts](execution.md) — body-result and lexical contract
- [Reflection](reflection.md) — exact and dynamic reflective execution
