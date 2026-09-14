# Callables

This directory contains the normative specification of Phalcom's callable model.

Each semantic fact has one primary owner. Other callable documents should reference that owner rather than restating the same rule in slightly different language.

The specifications describe observable language semantics. VM stack layout, bytecodes, native ABI, cache structure, and implementation migration guidance belong outside this directory.

## Reading map

| Question | Canonical document |
| --- | --- |
| What is a selector and what makes two selectors identical? | [Selectors](selectors.md) |
| What argument shape does a call supply? What do `*`, `**`, and `***` mean? | [Arguments and Parameters](arguments.md) |
| How is a message resolved and dispatched? | [Dispatch](dispatch.md) |
| What does applying a value mean? What is `Function`? | [Functions](functions.md) |
| What is one exact reified behavior? | [Methods](method.md) |
| What is an exact Method paired with a receiver? | [Bound Methods](bound-method.md) |
| What does a Closure capture and how does it return? | [Closures](closure.md) |
| What does `&` mean and how does a `Family` activate? | [References and Families](references-and-families.md) |
| How are Methods and MethodFamily snapshots reflected? | [Callable Reflection](reflection.md) |
| What are `self`, lexical `super`, caller/callee authority, and callable-local `return`? | [Execution Contexts](execution-contexts.md) |

## Conceptual model

The callable model separates operation identity, call shape, behavior selection, and executable values:

```text
selector
    identifies an operation shape

argument shape
    carries positional and labeled values

dispatch
    selects behavior for a receiver and selector

Method
    reifies one exact behavior but still needs a receiver

Function
    represents a complete callable value

Closure
    code + lexical environment

BoundMethod
    exact Method + receiver

Family
    selector capability + target context

MethodFamily
    immutable reflected route snapshot

BoundMethodFamily
    MethodFamily snapshot + receiver
```

## Callable protocol and Function

Application syntax is an open protocol:

```phalcom
value(arguments)
```

performs the corresponding:

```phalcom
value.call(arguments)
```

operation after the expression has resolved as a value.

An ordinary user class may therefore define `call` and become callable without inheriting the core `Function` class.

`Function` is the closed abstract runtime hierarchy of complete first-class callable representations:

```text
Function
├── Closure
├── BoundMethod
├── Family
└── BoundMethodFamily
```

`Method` and `MethodFamily` remain outside `Function` because each still lacks a receiver.

## Selector, rest, and pattern punctuation

These syntactic systems are distinct:

```text
...    selector-pattern / family syntax
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

`args...` is not argument spread syntax.

The canonical associated/member/reference operators are:

```text
.      ordinary object member lookup / message send
::     associated lookup
&      callable reference / capture
```

## Normative ownership

The specifications intentionally avoid a rigid chapter template.

Each document is organized around the semantic structure of the concept it owns.

Coded invariants are used only when a rule is:

- genuinely invariant across multiple surfaces;
- independently useful to implementation and testing;
- testable;
- non-redundant with an invariant already owned elsewhere.

Definitions, syntax facts, and obvious consequences should not receive invariant codes merely to make the documents appear formal.

## Architecture and conformance

The following subjects are intentionally not normative callable-spec content:

```text
InvocationLayout
ArgumentView
stack windows
bytecode opcodes
native primitive ABI
CallOutcome
frame rewriting
dispatch-cache representation
migration adapters
test matrices
```

They should be documented under architecture and conformance documentation respectively.

A change to those implementation mechanisms does not require a language-spec change unless it changes observable semantics defined by the files in this directory.
