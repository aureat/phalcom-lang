# Phalcom Callables, Execution, Arguments, and Reflection

**Status:** Normative language specification
**Scope:** callable object model, executable-body values, closures, methods, bound methods, families, calls, rest/spread, constructors, loop/return semantics, and the related reflection boundary.

This specification consolidates the ratified callable and execution decisions into one coherent language model. It is intentionally written as a language reference rather than as a decision log.

Where an older callable/Block/rest document conflicts with this specification, this specification defines the intended language semantics for the topics within its scope.

---

## 1. Chapters

1. [Core execution and value semantics](01-core-execution-and-values.md)
   Unit, final-expression semantics, assignment/declaration values, `if`, loops, `break`, `return`, lexical scope, constructors, and trailing-Closure syntax.

2. [Callable object model](02-callable-object-model.md)
   `Method`, `Function`, `Closure`, `BoundMethod`, `Family`, `self`, `super`, binding, and callable hierarchy.

3. [Calls, parameters, rest, and spread](03-calls-rest-and-spread.md)
   Message-call equivalence, parameter shape, `*`, `**`, `***`, Closure positional rest, rest capture, exact/rest dispatch, and `callWith`.

4. [Reflection and universal object protocol](04-reflection-and-object-protocol.md)
   `Behavior`, `methodFor`, `respondsTo`, `methods`, `Object#class`, `perform`, `doesNotUnderstand`, and retired reflective surface.

5. [Runtime conformance requirements](05-runtime-conformance.md)
   Normative implementation constraints required to preserve the language model: shape-aware invocation, native rest, flat activation, lexical authority, Unit representation, and removal of non-local-return machinery.

---

## 2. Standard terminology

### 2.1 Block

A **block** is a brace-delimited syntactic region of code.

A block establishes lexical scope when used as an executable code block.

A block is not itself a runtime callable object.

### 2.2 Closure

A **Closure** is a first-class callable runtime value containing executable code together with its captured lexical environment.

Closures are created by Closure literals such as:

```phalcom
|| {
    work()
}
```

or:

```phalcom
|value| {
    value * 2
}
```

### 2.3 Method

A **Method** is reified holder-owned behavior. A Method has an owning behavior, selector/parameter shape, implementation, lexical access context, and lexical `super` anchor.

A Method is not a Function because a Method still requires a compatible receiver.

### 2.4 BoundMethod

A **BoundMethod** is an exact Method paired with a compatible receiver.

```text
BoundMethod = exact Method + receiver
```

A BoundMethod is a Function because the receiver requirement has already been satisfied.

### 2.5 Function

A **Function** is a sealed VM-backed callable value whose remaining runtime inputs are only its explicitly supplied call arguments.

`Closure`, `BoundMethod`, and `Family` are Functions.

An arbitrary user object may define `call` and be callable without being a Function.

### 2.6 Family

A **Family** is the callable value produced by a `::` method-family reference.

A Family stores the receiver/reference context needed for a future ordinary message dispatch.

### 2.7 Unit and None

`()` is the sole value of `Unit`.

Unit denotes successful completion with no meaningful payload and is also the canonical empty product.

`None` denotes absence.

They are semantically distinct.

---

## 3. Callable hierarchy

```text
Object
├── Method                         sealed/final core class
└── Function                       abstract, sealed core class
    ├── Closure                    sealed/final core class
    ├── BoundMethod                sealed/final core class
    └── Family                     sealed/final core class
```

The VM-backed callable classes are sealed. User-defined callability is expressed by defining `call` on an ordinary object, not by subclassing `Function`.

---

## 4. Standard rest and spread notation

Phalcom uses only:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

Examples:

```phalcom
target(*values)
target(**fields)
target(***arguments)
```

Rest declarations use the same markers:

```phalcom
collect(_ first, *rest) {
    ...
}

forward(***arguments) {
    ...
}
```

Closure rest is positional only:

```phalcom
|head, *tail| {
    ...
}
```

A postfix ellipsis is not rest/spread syntax. In particular:

```text
arguments...
```

does not mean spread or rest.

---

## 5. Application is message syntax

For every value `f`:

```phalcom
f(a, b)
```

has the same language semantics as:

```phalcom
f.call(a, b)
```

Function calling is therefore part of ordinary message dispatch. An implementation may optimize the send, but optimization must remain observationally equivalent to ordinary message semantics.

---

## 6. Normative vocabulary

The words **must**, **must not**, **shall**, and **shall not** state requirements.

The word **may** states permitted behavior.

Examples are normative where they illustrate a rule stated in the surrounding text.

Internal runtime structures shown in the conformance chapter are conceptual; an implementation may choose different data structures if all observable semantics and stated invariants are preserved.
