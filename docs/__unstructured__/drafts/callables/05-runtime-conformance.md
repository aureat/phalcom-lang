# Runtime Conformance Requirements

[← Reflection](04-reflection-and-object-protocol.md) · [Overview](README.md)

This chapter specifies implementation invariants that are normative because violations would change observable language semantics.

The names of internal structs or bytecodes are illustrative. An implementation may choose different names or representations while preserving every stated invariant.

---

## 1. One callable execution model

The implementation must not maintain semantically different execution models for:

```text
normal Method send
Closure call
BoundMethod call
Family call
Method#invokeOn
Object#perform
```

After a Method has been selected, activation behavior must be shared as far as the Method representation permits.

Equivalent exact Method activations must agree on:

- receiver;
- parameter binding;
- lexical authority;
- `super`;
- return behavior;
- exception behavior;
- upvalue lifetime;
- Fiber/yield behavior.

---

## 2. Argument shape representation

The runtime must represent actual argument shape independently from parameter acceptance.

Conceptually, the implementation should distinguish:

```text
ArgumentShape
ParameterShape
ParameterLayout
BindingPlan
```

### 2.1 ArgumentShape

Describes an actual call:

```text
positional count
ordered labels
```

and is associated with the corresponding values.

### 2.2 ParameterShape

Describes what a Method/Closure accepts:

```text
fixed positional count
fixed labels
rest mode
```

### 2.3 ParameterLayout

Describes frame/local placement, not dispatch semantics.

### 2.4 BindingPlan

Is the allocation-free structural match between actual and accepted shape.

Matching must not invoke user code.

Matching must not inspect runtime argument value types.

---

## 3. Allocation-free argument transport

The complete Function gateway:

```phalcom
call(***arguments)
```

does not require public rest-product allocation for every call.

The runtime must be able to inspect/forward the complete argument shape without first materializing a Tuple.

A conceptual internal view may resemble:

```text
ArgumentView
```

over a rooted stack window or dynamic-pack storage.

Only an observable rest binding must materialize its canonical Unit/Tuple capture.

---

## 4. Native Method ABI

Native/primitive Methods must receive enough argument-shape information to implement:

```text
exact parameters
* rest
** rest
*** rest
split rest
```

with the same semantics as bytecode Methods.

A primitive ABI carrying only an unannotated flat `&[Value]` is insufficient as the sole semantic input where labels/rest shape matter.

The implementation must preserve the no-allocation fast path for ordinary small exact calls.

---

## 5. Native rest

Native Methods may carry all Method rest modes.

They participate in the same:

```text
exact across inheritance
then rest across inheritance
```

resolver as bytecode Methods.

The runtime must not special-case native complete rest as a distinct language feature limited to `Function#call`.

---

## 6. Flat activation

Ordinary language-level forwarding must enter the target activation without recursively running an interpreter loop to completion inside a native helper.

Conceptually, activation can produce either:

```text
Returned(value)
```

or:

```text
EnteredFrame
```

This supports forwarding operations such as the Function gateway without native recursion.

A synchronous host helper may exist for runtime/native code that genuinely requires an immediate value, but ordinary language-level Function/BoundMethod/Family/invokeOn/perform forwarding must not depend on recursive interpreter re-entry.

---

## 7. Function routing

`Function#call(***arguments)` is the common Function gateway.

Concrete routing:

```text
Closure
    → validate Closure ParameterShape
    → bind
    → enter Closure activation

BoundMethod
    → exact stored Method
    → stored receiver
    → validate Method ParameterShape
    → activate exact Method

Family
    → derive/use target selector
    → ordinary target dispatch
```

No concrete Function subclass needs finite synthetic `call` overloads.

The `call` base family is final.

---

## 8. Family routing

A Family call must not intentionally trigger `doesNotUnderstand` merely to recover the original `call(...)` selector/labels.

The runtime already possesses the actual call shape and must route directly from that shape.

Open Family:

```text
stored base name + actual call shape → target selector → ordinary send
```

Pinned Family:

```text
stored pinned selector + validated supplied arguments → ordinary send
```

A genuine Family message miss may still use normal dNU.

---

## 9. BoundMethod representation

A BoundMethod must remain conceptually minimal:

```text
Method reference
receiver value
```

It must not require:

- cloning the Method;
- synthesizing a Closure;
- recursively wrapping another BoundMethod;
- synthesizing per-arity `call` Methods.

Receiver compatibility must be validated when binding and exact invocation are requested.

---

## 10. Lexical Method authority

Method visibility authorization has two distinct contexts.

### 10.1 Caller authorization

Before entering a Method, the runtime decides whether the current caller's lexical authority may invoke it.

### 10.2 Callee execution authority

While the Method executes, sends made by the Method use the Method's own lexical access authority.

This distinction applies equally to bytecode and native Methods.

Native Method execution therefore requires explicit callee lexical context rather than inheriting the caller's authority accidentally.

---

## 11. `self` and `super` conformance

For an exact Method `A#m` activated on compatible subclass receiver `b`:

```text
self = b
entry implementation = exact A#m
ordinary sends = dynamic on b
super lookup anchor = lexical A
```

A BoundMethod must preserve these rules exactly.

A Closure created during that execution captures `b` as lexical `self`.

---

## 12. Unit representation

The runtime must represent Unit distinctly from None.

A dedicated immediate `Unit` value is permitted and preferred.

The compiler/runtime must be able to explicitly produce Unit for:

- empty bodies;
- assignment/declaration results when needed;
- one-armed `if`;
- normal loop result;
- bare `break`;
- bare `return`.

An ordinary return instruction must not use None as a fallback result for a missing stack value.

The compiler should guarantee an explicit result value before return.

---

## 13. Value-needed optimization

The compiler may distinguish:

```text
Needed
Discarded
```

value contexts.

If a construct semantically produces Unit but that value is immediately unobservable, the compiler may omit the physical Unit push.

This optimization must not skip side effects or alter final-expression semantics.

---

## 14. No non-local return machinery

`return` in a Closure is local to that Closure.

Therefore conforming implementations must not require runtime concepts whose sole purpose is implicit non-local return, including:

```text
home-frame return target
ReturnNonLocal opcode
DeadFrameError for escaped Closure return
Block wrapper carrying a home-frame token
NLR-specific native stack repair
```

If frame-generation/token infrastructure serves another independent runtime feature, that independent feature may retain it.

---

## 15. Closure representation

A public Closure value should be directly represented/classified as `Closure`.

A separate public/runtime `Block` object is not part of the callable object model.

Compiled bytecode implementation objects may internally use closure-like structures for both Methods and Closure values. Such an internal implementation detail must not imply that a Method is a surface Closure or Function.

---

## 16. Direct lexical loops

Source-level `while`/loop control must be compiled as lexical control flow capable of implementing:

```phalcom
break value
continue
```

without cross-Closure control transfer.

A higher-order `Function#whileTrue`-style combinator may exist as library behavior, but source `while` must not depend semantically on creating condition/body Closures and then using non-local loop control.

---

## 17. Constructor conformance

A conforming constructor implementation may generate hidden Methods or internal names, but it must preserve this semantic decomposition:

```text
class-side factory
instance initializer Method
```

The initializer remains an ordinary Method activation.

The factory discards the initializer result and returns the allocated instance.

The source compiler rejects explicit `return value` in `@constructor` initializer source.

No constructor-specific Method return opcode is required or permitted as a semantic dependency.

---

## 18. Method-table and cache invariants

Dispatch caches must preserve:

```text
exact lookup precedence
rest fallback precedence
world/method-table invalidation
visibility semantics
```

A useful cache may record:

```text
Exact(Method)
Rest(Method, binding metadata)
Miss
```

for a receiver class and concrete argument shape.

Caching must not change inheritance ordering.

---

## 19. Reflection conformance

`Behavior#methods` reads direct behavior dictionary state.

`Behavior#methodFor` and `Behavior#respondsTo` must introspect the receiver Behavior's governed method hierarchy.

They must not accidentally answer:

> does this Behavior object itself respond to selector X?

when the intended question is:

> would an instance governed by this Behavior resolve selector X?

This distinction is especially important because `Person.methodFor(...)` is a message sent to a class object whose own class is a metaclass.

---

## 20. Sealed callable classes

The runtime/class system must enforce sealing for:

```text
Function
Closure
BoundMethod
Family
Method
```

including reflective class/method mutation paths where those paths could otherwise violate the Function call-family invariant.

A user object that wants custom call semantics defines ordinary `call` Methods outside the sealed Function hierarchy.

---

## 21. Conformance summary

A runtime conforms to this callable specification when all of the following hold:

```text
Method is not Function.
Closure, BoundMethod, and Family are Functions.
Application is ordinary call-message semantics.
Rest/spread uses only *, **, ***.
Closure rest is positional-only.
Native and bytecode Method rest are semantically identical.
Exact lookup precedes rest lookup across inheritance.
Function routing does not allocate public packs unnecessarily.
Family does not use dNU as its normal call router.
BoundMethod executes an exact stored Method.
Primitive execution has callee lexical Method authority.
Language forwarding uses flat activation.
Unit is distinct from None.
return inside Closure is local.
There is no implicit non-local return.
Source loops support break values lexically.
Constructors are generated factories over ordinary initializer Methods.
Structural method reflection belongs to Behavior.
```
